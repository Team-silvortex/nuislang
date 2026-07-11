macro_rules! lower_loop_chain {
    ($node:expr, $body:expr, $registers:expr, $next_reg:expr, $next_block:expr, $last_cpu_value:expr) => {{
        let node = $node;
        let mut body = $body;
        let registers = $registers;
        let mut next_reg = $next_reg;
        let mut next_block = $next_block;
        let last_cpu_value = $last_cpu_value;
                let loop_instruction = canonical_loop_instruction(&node.op.instruction);
                let loop_block_prefix = canonical_loop_block_prefix(&node.op.instruction);
                let initial_value = registers.get(&node.op.args[0]).cloned();
                let limit_value = registers.get(&node.op.args[1]).cloned();
                let step_value = registers.get(&node.op.args[2]).cloned();
                let (Some(initial_value), Some(limit_value), Some(step_value)) =
                    (initial_value, limit_value, step_value)
                else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name,
                    ));
                    continue;
                };
                let cmp_kind = node.op.args[3].as_str();
                let step_kind = node.op.args[4].as_str();
                let mut carry_initial_values = Vec::new();
                let mut carry_specs_raw = Vec::new();
                let mut cursor = 5usize;
                while cursor < node.op.args.len() {
                    let carry_initial_name = &node.op.args[cursor];
                    let carry_kind = &node.op.args[cursor + 1];
                    let payload_len = loop_carry_payload_len(carry_kind);
                    let payload_names = &node.op.args[cursor + 2..cursor + 2 + payload_len];
                    let carry_initial_value = registers.get(carry_initial_name).cloned();
                    let Some(carry_initial_value) = carry_initial_value else {
                        body.push(format!(
                            "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry initials are outside the current CPU LLVM slice",
                            node.name,
                        ));
                        continue;
                    };
                    let mut payload_values = Vec::new();
                    let mut missing_payload = false;
                    for payload_name in payload_names {
                        let Some(payload_value) = registers.get(payload_name).cloned() else {
                            body.push(format!(
                                "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry payloads are outside the current CPU LLVM slice",
                                node.name,
                            ));
                            missing_payload = true;
                            break;
                        };
                        payload_values.push(payload_value);
                    }
                    if missing_payload {
                        continue;
                    }
                    carry_initial_values.push(carry_initial_value);
                    carry_specs_raw.push((carry_kind.clone(), payload_values));
                    cursor += 2 + payload_len;
                }
                let Some(loop_scalar_kind) = infer_loop_scalar_kind(
                    [&initial_value, &limit_value, &step_value]
                        .into_iter()
                        .chain(carry_initial_values.iter()),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its loop values are not representable as one scalar kind",
                        node.name,
                    ));
                    continue;
                };
                let Some(initial) = coerce_to_loop_scalar(
                    &initial_value,
                    loop_scalar_kind,
                    &mut body,
                    &mut next_reg,
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its initial value is not coercible to the selected loop scalar kind",
                        node.name,
                    ));
                    continue;
                };
                let Some(limit) =
                    coerce_to_loop_scalar(&limit_value, loop_scalar_kind, &mut body, &mut next_reg)
                else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its limit value is not coercible to the selected loop scalar kind",
                        node.name,
                    ));
                    continue;
                };
                let Some(step) =
                    coerce_to_loop_scalar(&step_value, loop_scalar_kind, &mut body, &mut next_reg)
                else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its step value is not coercible to the selected loop scalar kind",
                        node.name,
                    ));
                    continue;
                };
                let mut carry_initials = Vec::new();
                for carry_initial_value in &carry_initial_values {
                    let Some(carry_initial) = coerce_to_loop_scalar(
                        carry_initial_value,
                        loop_scalar_kind,
                        &mut body,
                        &mut next_reg,
                    ) else {
                        body.push(format!(
                            "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry initials are not coercible to the selected loop scalar kind",
                            node.name,
                        ));
                        continue;
                    };
                    carry_initials.push(carry_initial);
                }
                let mut carry_specs = Vec::new();
                for (carry_kind, payload_values) in &carry_specs_raw {
                    let mut payloads = Vec::new();
                    for payload_value in payload_values {
                        let Some(payload) = coerce_to_loop_scalar(
                            payload_value,
                            loop_scalar_kind,
                            &mut body,
                            &mut next_reg,
                        ) else {
                            body.push(format!(
                                "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry payloads are not coercible to the selected loop scalar kind",
                                node.name,
                            ));
                            continue;
                        };
                        payloads.push(payload);
                    }
                    carry_specs.push((carry_kind.clone(), payloads));
                }
                let scalar_ty = loop_scalar_llvm_type(loop_scalar_kind);
                let current_slot = fresh_reg(&mut next_reg);
                body.push(format!("  {current_slot} = alloca {scalar_ty}"));
                body.push(format!("  store {scalar_ty} {initial}, ptr {current_slot}"));
                let carry_slots = carry_initials
                    .iter()
                    .map(|carry_initial| {
                        let carry_slot = fresh_reg(&mut next_reg);
                        body.push(format!("  {carry_slot} = alloca {scalar_ty}"));
                        body.push(format!(
                            "  store {scalar_ty} {carry_initial}, ptr {carry_slot}"
                        ));
                        carry_slot
                    })
                    .collect::<Vec<_>>();
                let loop_cond = fresh_block(&mut next_block, &format!("{loop_block_prefix}_cond"));
                let loop_body = fresh_block(&mut next_block, &format!("{loop_block_prefix}_body"));
                let loop_exit = fresh_block(&mut next_block, &format!("{loop_block_prefix}_exit"));
                body.push(format!("  br label %{loop_cond}"));
                body.push(format!("{loop_cond}:"));
                let current = fresh_reg(&mut next_reg);
                body.push(format!(
                    "  {current} = load {scalar_ty}, ptr {current_slot}"
                ));
                let cmp = emit_loop_compare(
                    &mut body,
                    &mut next_reg,
                    loop_scalar_kind,
                    cmp_kind,
                    &current,
                    &limit,
                )
                .map_err(|error| {
                    format!(
                        "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                        node.name,
                    )
                })?;
                body.push(format!(
                    "  br i1 {cmp}, label %{loop_body}, label %{loop_exit}"
                ));
                body.push(format!("{loop_body}:"));
                let next_current = emit_loop_numeric_op(
                    &mut body,
                    &mut next_reg,
                    loop_scalar_kind,
                    step_kind,
                    &current,
                    &step,
                )
                .map_err(|error| {
                    format!(
                        "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                        node.name,
                    )
                })?;
                let mut current_carries = Vec::new();
                for carry_slot in &carry_slots {
                    let carry_before = fresh_reg(&mut next_reg);
                    body.push(format!(
                        "  {carry_before} = load {scalar_ty}, ptr {carry_slot}"
                    ));
                    current_carries.push(carry_before);
                }
                let mut next_carries = Vec::new();
                for (index, ((carry_kind, raw_payloads), (_, payloads))) in
                    carry_specs_raw.iter().zip(carry_specs.iter()).enumerate()
                {
                    let (source, op) = if carry_kind == "add_current" {
                        (next_current.clone(), "add")
                    } else if carry_kind == "add_prev_current" {
                        (current.clone(), "add")
                    } else if carry_kind == "mul_current" {
                        (next_current.clone(), "mul")
                    } else if carry_kind == "mul_prev_current" {
                        (current.clone(), "mul")
                    } else if let Some((source, op)) = try_resolve_loop_carry_read_source(
                        &mut body,
                        &mut next_reg,
                        loop_scalar_kind,
                        carry_kind,
                        raw_payloads,
                        payloads,
                        &current,
                        &next_current,
                        &current_carries,
                        &next_carries,
                        &node.name,
                        loop_instruction,
                    )? {
                        (source, op)
                    } else if let Some(rest) = carry_kind.strip_prefix("add_prev_carry") {
                        let source_index = rest.parse::<usize>().map_err(|_| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (
                            current_carries.get(source_index).cloned().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?,
                            "add",
                        )
                    } else if let Some(rest) = carry_kind.strip_prefix("mul_prev_carry") {
                        let source_index = rest.parse::<usize>().map_err(|_| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (
                            current_carries.get(source_index).cloned().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?,
                            "mul",
                        )
                    } else if let Some(rest) = carry_kind.strip_prefix("add_carry") {
                        let source_index = rest.parse::<usize>().map_err(|_| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (
                            next_carries.get(source_index).cloned().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?,
                            "add",
                        )
                    } else if let Some(rest) = carry_kind.strip_prefix("mul_carry") {
                        let source_index = rest.parse::<usize>().map_err(|_| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (
                            next_carries.get(source_index).cloned().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?,
                            "mul",
                        )
                    } else if let Some((source, op)) = try_resolve_loop_carry_scaled_source(
                        &mut body,
                        &mut next_reg,
                        loop_scalar_kind,
                        carry_kind,
                        payloads,
                        &current,
                        &next_current,
                        &current_carries,
                        &next_carries,
                        &node.name,
                        loop_instruction,
                    )? {
                        (source, op)
                    } else {
                        return Err(format!(
                            "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                            node.name,
                        ));
                    };
                    let reg = emit_loop_numeric_op(
                        &mut body,
                        &mut next_reg,
                        loop_scalar_kind,
                        op,
                        &current_carries[index],
                        &source,
                    )
                    .map_err(|error| {
                        format!(
                            "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                            node.name,
                        )
                    })?;
                    next_carries.push(reg);
                }
                body.push(format!(
                    "  store {scalar_ty} {next_current}, ptr {current_slot}"
                ));
                for (carry_slot, next_carry) in carry_slots.iter().zip(next_carries.iter()) {
                    body.push(format!(
                        "  store {scalar_ty} {next_carry}, ptr {carry_slot}"
                    ));
                }
                body.push(format!("  br label %{loop_cond}"));
                body.push(format!("{loop_exit}:"));
                let final_current = fresh_reg(&mut next_reg);
                body.push(format!(
                    "  {final_current} = load {scalar_ty}, ptr {current_slot}"
                ));
                let final_carries = carry_slots
                    .iter()
                    .map(|carry_slot| {
                        let final_carry = fresh_reg(&mut next_reg);
                        body.push(format!(
                            "  {final_carry} = load {scalar_ty}, ptr {carry_slot}"
                        ));
                        final_carry
                    })
                    .collect::<Vec<_>>();
                insert_scalar_loop_chain_result(
                    &mut body,
                    &mut next_reg,
                    registers,
                    &node.name,
                    loop_scalar_kind,
                    final_current,
                    final_carries,
                    last_cpu_value,
                );
    }};
}
