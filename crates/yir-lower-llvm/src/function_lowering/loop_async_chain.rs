macro_rules! lower_loop_async_chain {
    ($node:expr, $body:expr, $registers:expr, $next_reg:expr, $next_block:expr, $last_cpu_value:expr, $helper_signatures:expr) => {{
        let node = $node;
        let mut body = $body;
        let registers = $registers;
        let mut next_reg = $next_reg;
        let mut next_block = $next_block;
        let last_cpu_value = $last_cpu_value;
        let helper_signatures = $helper_signatures;
                let loop_instruction = canonical_loop_instruction(&node.op.instruction);
                let loop_block_prefix = canonical_loop_block_prefix(&node.op.instruction);
                let initial_value = registers.get(&node.op.args[0]).cloned();
                let limit_value = registers.get(&node.op.args[1]).cloned();
                let callee = &node.op.args[2];
                let Some(signature) = helper_signatures.get(callee) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because helper signature `{}` is unavailable",
                        node.name, callee
                    ));
                    continue;
                };
                if signature.params.as_slice() != [CpuCallScalarKind::I64]
                    || signature.ret != CpuCallScalarKind::I64
                {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because helper `{}` must have signature `(i64) -> i64`",
                        node.name, callee
                    ));
                    continue;
                }
                let (Some(initial_value), Some(limit_value)) = (initial_value, limit_value) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name,
                    ));
                    continue;
                };
                let Some(initial) = coerce_to_i64(&initial_value, &mut body, &mut next_reg) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its initial value is not coercible to i64",
                        node.name,
                    ));
                    continue;
                };
                let Some(limit) = coerce_to_i64(&limit_value, &mut body, &mut next_reg) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its limit value is not coercible to i64",
                        node.name,
                    ));
                    continue;
                };
                let cmp_kind = node.op.args[3].as_str();
                let mut carry_initials = Vec::new();
                let mut carry_specs = Vec::new();
                let mut cursor = 4usize;
                while cursor < node.op.args.len() {
                    let carry_initial_name = &node.op.args[cursor];
                    let carry_kind = &node.op.args[cursor + 1];
                    let payload_len = loop_async_chain_carry_payload_len(carry_kind);
                    let payload_names = &node.op.args[cursor + 2..cursor + 2 + payload_len];
                    let carry_initial_value = registers.get(carry_initial_name).cloned();
                    let Some(carry_initial_value) = carry_initial_value else {
                        body.push(format!(
                            "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry initials are outside the current CPU LLVM slice",
                            node.name,
                        ));
                        continue;
                    };
                    let Some(carry_initial) =
                        coerce_to_i64(&carry_initial_value, &mut body, &mut next_reg)
                    else {
                        body.push(format!(
                            "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry initials are not coercible to i64",
                            node.name,
                        ));
                        continue;
                    };
                    let mut payloads = Vec::new();
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
                        let Some(payload) = coerce_to_i64(&payload_value, &mut body, &mut next_reg)
                        else {
                            body.push(format!(
                                "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry payloads are not coercible to i64",
                                node.name,
                            ));
                            missing_payload = true;
                            break;
                        };
                        payloads.push(payload);
                    }
                    if missing_payload {
                        continue;
                    }
                    carry_initials.push(carry_initial);
                    carry_specs.push((carry_kind.clone(), payloads));
                    cursor += 2 + payload_len;
                }
                let current_slot = fresh_reg(&mut next_reg);
                body.push(format!("  {current_slot} = alloca i64"));
                body.push(format!("  store i64 {initial}, ptr {current_slot}"));
                let carry_slots = carry_initials
                    .iter()
                    .map(|carry_initial| {
                        let carry_slot = fresh_reg(&mut next_reg);
                        body.push(format!("  {carry_slot} = alloca i64"));
                        body.push(format!("  store i64 {carry_initial}, ptr {carry_slot}"));
                        carry_slot
                    })
                    .collect::<Vec<_>>();
                let loop_cond = fresh_block(&mut next_block, &format!("{loop_block_prefix}_cond"));
                let loop_body = fresh_block(&mut next_block, &format!("{loop_block_prefix}_body"));
                let loop_exit = fresh_block(&mut next_block, &format!("{loop_block_prefix}_exit"));
                body.push(format!("  br label %{loop_cond}"));
                body.push(format!("{loop_cond}:"));
                let current = fresh_reg(&mut next_reg);
                body.push(format!("  {current} = load i64, ptr {current_slot}"));
                let cmp = fresh_reg(&mut next_reg);
                let pred = match cmp_kind {
                    "eq" => "eq",
                    "ne" => "ne",
                    "lt" => "slt",
                    "le" => "sle",
                    "gt" => "sgt",
                    "ge" => "sge",
                    other => {
                        return Err(format!(
                            "cpu.{loop_instruction} `{}` has unsupported compare kind `{other}` during LLVM lowering",
                            node.name,
                        ));
                    }
                };
                body.push(format!("  {cmp} = icmp {pred} i64 {current}, {limit}"));
                body.push(format!(
                    "  br i1 {cmp}, label %{loop_body}, label %{loop_exit}"
                ));
                body.push(format!("{loop_body}:"));
                let next_current = fresh_reg(&mut next_reg);
                body.push(format!(
                    "  {next_current} = call i64 @nuis_fn_{callee}(i64 {current})"
                ));
                let mut current_carries = Vec::new();
                for carry_slot in &carry_slots {
                    let carry_before = fresh_reg(&mut next_reg);
                    body.push(format!("  {carry_before} = load i64, ptr {carry_slot}"));
                    current_carries.push(carry_before);
                }
                let mut next_carries = Vec::new();
                for (index, (carry_kind, payloads)) in carry_specs.iter().enumerate() {
                    let (source, op) = if carry_kind == "add_current" {
                        (next_current.clone(), "add")
                    } else if carry_kind == "add_prev_current" {
                        (current.clone(), "add")
                    } else if carry_kind == "mul_current" {
                        (next_current.clone(), "mul")
                    } else if carry_kind == "mul_prev_current" {
                        (current.clone(), "mul")
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
                    } else if let Some(prefix) = carry_kind
                        .strip_prefix("mul_scaled_by_")
                        .and_then(|prefix| prefix.split_once("_plus_factor_invariant_"))
                    {
                        let (factor_term, terms_part) = prefix;
                        let (terms_part, has_invariant) =
                            if let Some(terms_part) = terms_part.strip_suffix("_plus_invariant") {
                                (terms_part, true)
                            } else {
                                (terms_part, false)
                            };
                        let resolve_term = |term: &str| {
                            resolve_loop_carry_term(
                                term,
                                carry_kind,
                                &current,
                                &next_current,
                                &current_carries,
                                &next_carries,
                                &node.name,
                                loop_instruction,
                            )
                        };
                        let factor = resolve_term(factor_term)?;
                        let factor_offset = payloads.first().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` is missing carry payload for `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let factor_reg = fresh_reg(&mut next_reg);
                        body.push(format!(
                            "  {factor_reg} = add i64 {factor}, {factor_offset}"
                        ));
                        let mut terms = terms_part.split("_plus_");
                        let first = terms.next().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let mut source = resolve_term(first)?;
                        for term in terms {
                            let rhs = resolve_term(term)?;
                            let reg = fresh_reg(&mut next_reg);
                            body.push(format!("  {reg} = add i64 {source}, {rhs}"));
                            source = reg;
                        }
                        if has_invariant {
                            let offset = payloads.get(1).ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` is missing carry payload for `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            let reg = fresh_reg(&mut next_reg);
                            body.push(format!("  {reg} = add i64 {source}, {offset}"));
                            source = reg;
                        }
                        let reg = fresh_reg(&mut next_reg);
                        body.push(format!("  {reg} = mul i64 {source}, {factor_reg}"));
                        (reg, "mul")
                    } else if let Some(prefix) = carry_kind.strip_prefix("mul_scaled_by_") {
                        let (factor_term, terms_part) = prefix.split_once('_').ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let (terms_part, has_invariant) =
                            if let Some(terms_part) = terms_part.strip_suffix("_plus_invariant") {
                                (terms_part, true)
                            } else {
                                (terms_part, false)
                            };
                        let resolve_term = |term: &str| {
                            resolve_loop_carry_term(
                                term,
                                carry_kind,
                                &current,
                                &next_current,
                                &current_carries,
                                &next_carries,
                                &node.name,
                                loop_instruction,
                            )
                        };
                        let factor = resolve_term(factor_term)?;
                        let mut terms = terms_part.split("_plus_");
                        let first = terms.next().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let mut source = resolve_term(first)?;
                        for term in terms {
                            let rhs = resolve_term(term)?;
                            let reg = fresh_reg(&mut next_reg);
                            body.push(format!("  {reg} = add i64 {source}, {rhs}"));
                            source = reg;
                        }
                        if has_invariant {
                            let offset = payloads.first().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` is missing carry payload for `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            let reg = fresh_reg(&mut next_reg);
                            body.push(format!("  {reg} = add i64 {source}, {offset}"));
                            source = reg;
                        }
                        let reg = fresh_reg(&mut next_reg);
                        body.push(format!("  {reg} = mul i64 {source}, {factor}"));
                        (reg, "mul")
                    } else if carry_kind.starts_with("mul_scaled_") {
                        let (terms_part, has_invariant) = if let Some(terms_part) =
                            carry_kind.strip_prefix("mul_scaled_")
                        {
                            if let Some(terms_part) = terms_part.strip_suffix("_plus_invariant") {
                                (terms_part, true)
                            } else {
                                (terms_part, false)
                            }
                        } else {
                            unreachable!()
                        };
                        let factor = payloads.first().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` is missing carry payload for `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let resolve_term = |term: &str| {
                            resolve_loop_carry_term(
                                term,
                                carry_kind,
                                &current,
                                &next_current,
                                &current_carries,
                                &next_carries,
                                &node.name,
                                loop_instruction,
                            )
                        };
                        let mut terms = terms_part.split("_plus_");
                        let first = terms.next().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let mut source = resolve_term(first)?;
                        for term in terms {
                            let rhs = resolve_term(term)?;
                            let reg = fresh_reg(&mut next_reg);
                            body.push(format!("  {reg} = add i64 {source}, {rhs}"));
                            source = reg;
                        }
                        if has_invariant {
                            let offset = payloads.get(1).ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` is missing carry payload for `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            let reg = fresh_reg(&mut next_reg);
                            body.push(format!("  {reg} = add i64 {source}, {offset}"));
                            source = reg;
                        }
                        let reg = fresh_reg(&mut next_reg);
                        body.push(format!("  {reg} = mul i64 {source}, {factor}"));
                        (reg, "mul")
                    } else if carry_kind.starts_with("mul_") {
                        let (terms_part, has_invariant) = if let Some(terms_part) =
                            carry_kind.strip_prefix("mul_")
                        {
                            if let Some(terms_part) = terms_part.strip_suffix("_plus_invariant") {
                                (terms_part, true)
                            } else {
                                (terms_part, false)
                            }
                        } else {
                            unreachable!()
                        };
                        let resolve_term = |term: &str| {
                            resolve_loop_carry_term(
                                term,
                                carry_kind,
                                &current,
                                &next_current,
                                &current_carries,
                                &next_carries,
                                &node.name,
                                loop_instruction,
                            )
                        };
                        let mut terms = terms_part.split("_plus_");
                        let first = terms.next().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let mut source = resolve_term(first)?;
                        for term in terms {
                            let rhs = resolve_term(term)?;
                            let reg = fresh_reg(&mut next_reg);
                            body.push(format!("  {reg} = add i64 {source}, {rhs}"));
                            source = reg;
                        }
                        if has_invariant {
                            let payload = payloads.first().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` is missing carry payload for `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            let reg = fresh_reg(&mut next_reg);
                            body.push(format!("  {reg} = add i64 {source}, {payload}"));
                            source = reg;
                        }
                        (source, "mul")
                    } else {
                        return Err(format!(
                            "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                            node.name,
                        ));
                    };
                    let reg = fresh_reg(&mut next_reg);
                    body.push(format!(
                        "  {reg} = {op} i64 {}, {}",
                        current_carries[index], source
                    ));
                    next_carries.push(reg);
                }
                body.push(format!("  store i64 {next_current}, ptr {current_slot}"));
                for (carry_slot, next_carry) in carry_slots.iter().zip(next_carries.iter()) {
                    body.push(format!("  store i64 {next_carry}, ptr {carry_slot}"));
                }
                body.push(format!("  br label %{loop_cond}"));
                body.push(format!("{loop_exit}:"));
                let final_current = fresh_reg(&mut next_reg);
                body.push(format!("  {final_current} = load i64, ptr {current_slot}"));
                let final_carries = carry_slots
                    .iter()
                    .map(|carry_slot| {
                        let final_carry = fresh_reg(&mut next_reg);
                        body.push(format!("  {final_carry} = load i64, ptr {carry_slot}"));
                        final_carry
                    })
                    .collect::<Vec<_>>();
                insert_i64_loop_chain_result(
                    registers,
                    &node.name,
                    final_current,
                    final_carries,
                    last_cpu_value,
                );
    }};
}
