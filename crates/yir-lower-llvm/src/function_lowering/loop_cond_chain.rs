macro_rules! lower_loop_cond_chain {
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
                let mut carry_specs = Vec::new();
                let mut carry_source_values = Vec::new();
                let mut deferred = false;
                let parsed_carries = match yir_domain_cpu::parse_conditional_carries(
                    &node.op.args,
                    5,
                    &node.name,
                    true,
                ) {
                    Ok(carries) => carries,
                    Err(error) => {
                        body.push(format!(
                            "  ; deferred lowering for cpu.{loop_instruction} `{}` because conditional carry metadata is invalid: {error}",
                            node.name,
                        ));
                        continue;
                    }
                };
                for carry in parsed_carries {
                    let carry_initial_value = registers.get(&carry.initial).cloned();
                    let Some(carry_initial_value) = carry_initial_value else {
                        body.push(format!(
                            "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry initials are outside the current CPU LLVM slice",
                            node.name,
                        ));
                        deferred = true;
                        break;
                    };
                    carry_initial_values.push(carry_initial_value);
                    let (cond_kind, cond_rhs_name) = match carry.condition {
                        yir_domain_cpu::LoopCondExpr::Leaf { kind, rhs } => (kind, rhs),
                        yir_domain_cpu::LoopCondExpr::Binary { .. } => {
                            body.push(format!(
                                "  ; deferred lowering for cpu.{loop_instruction} `{}` because compound conditional carries are not yet in the current CPU LLVM slice",
                                node.name,
                            ));
                            deferred = true;
                            break;
                        }
                    };
                    let cond_rhs = if let Some(cond_rhs_name) = cond_rhs_name {
                        let cond_rhs_value = registers.get(&cond_rhs_name).cloned();
                        let Some(cond_rhs_value) = cond_rhs_value else {
                            body.push(format!(
                                "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more condition rhs values are outside the current CPU LLVM slice",
                                node.name,
                            ));
                            deferred = true;
                            break;
                        };
                        Some(cond_rhs_value)
                    } else {
                        None
                    };
                    let mut lower_source = |source: yir_domain_cpu::ParsedCarryBranchSource| {
                        let mut payload_values = Vec::new();
                        for payload_name in source.payload {
                            let Some(payload_value) = registers.get(&payload_name).cloned() else {
                                return None;
                            };
                            carry_source_values.push(payload_value.clone());
                            payload_values.push(payload_value);
                        }
                        Some((source.kind, payload_values))
                    };
                    let Some(then_source) = lower_source(carry.then_source) else {
                        body.push(format!(
                            "  ; deferred lowering for cpu.{loop_instruction} `{}` because a then-carry payload is outside the current CPU LLVM slice",
                            node.name,
                        ));
                        deferred = true;
                        break;
                    };
                    let Some(else_source) = lower_source(carry.else_source) else {
                        body.push(format!(
                            "  ; deferred lowering for cpu.{loop_instruction} `{}` because an else-carry payload is outside the current CPU LLVM slice",
                            node.name,
                        ));
                        deferred = true;
                        break;
                    };
                    carry_specs.push((
                        cond_kind,
                        cond_rhs,
                        then_source,
                        else_source,
                    ));
                }
                if deferred {
                    continue;
                }
                let Some(loop_scalar_kind) = infer_loop_scalar_kind(
                    [&initial_value, &limit_value, &step_value]
                        .into_iter()
                        .chain(carry_initial_values.iter())
                        .chain(carry_source_values.iter())
                        .chain(
                            carry_specs
                                .iter()
                                .filter_map(|(_, cond_rhs, _, _)| cond_rhs.as_ref()),
                        ),
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
                        deferred = true;
                        break;
                    };
                    carry_initials.push(carry_initial);
                }
                if deferred {
                    continue;
                }
                let mut lowered_carry_specs = Vec::new();
                for (cond_kind, cond_rhs, then_source, else_source) in carry_specs {
                    let lowered_cond_rhs = if let Some(cond_rhs) = cond_rhs {
                        let Some(cond_rhs) = coerce_to_loop_scalar(
                            &cond_rhs,
                            loop_scalar_kind,
                            &mut body,
                            &mut next_reg,
                        ) else {
                            body.push(format!(
                                "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more condition rhs values are not coercible to the selected loop scalar kind",
                                node.name,
                            ));
                            deferred = true;
                            break;
                        };
                        Some(cond_rhs)
                    } else {
                        None
                    };
                    let mut lower_source_payload = |(kind, payload): (String, Vec<LlvmValueRef>)| {
                        let mut lowered = Vec::new();
                        for value in payload {
                            let value = coerce_to_loop_scalar(
                                &value,
                                loop_scalar_kind,
                                &mut body,
                                &mut next_reg,
                            )?;
                            lowered.push(value);
                        }
                        Some((kind, lowered))
                    };
                    let Some(then_source) = lower_source_payload(then_source) else {
                        deferred = true;
                        break;
                    };
                    let Some(else_source) = lower_source_payload(else_source) else {
                        deferred = true;
                        break;
                    };
                    lowered_carry_specs.push((
                        cond_kind,
                        lowered_cond_rhs,
                        then_source,
                        else_source,
                    ));
                }
                if deferred {
                    continue;
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
                let resolve_source = |source: &(String, Vec<String>),
                                      next_current: &String,
                                      next_carries: &Vec<String>|
                 -> Result<(String, &'static str), String> {
                    let (kind, payload) = source;
                    if matches!(kind.as_str(), "keep" | "keep_prev_carry") {
                        return Ok((String::new(), "keep"));
                    }
                    if matches!(kind.as_str(), "add_invariant" | "mul_invariant") {
                        let value = payload.first().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` carry kind `{kind}` is missing its invariant payload during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let value = value.clone();
                        let op = if kind == "add_invariant" { "add" } else { "mul" };
                        return Ok((value, op));
                    }
                    if kind == "add_current" {
                        return Ok((next_current.clone(), "add"));
                    }
                    if kind == "add_prev_current" {
                        return Ok((current.clone(), "add"));
                    }
                    if kind == "mul_current" {
                        return Ok((next_current.clone(), "mul"));
                    }
                    if kind == "mul_prev_current" {
                        return Ok((current.clone(), "mul"));
                    }
                    if let Some(rest) = kind.strip_prefix("add_prev_carry") {
                        let source_index = rest.parse::<usize>().map_err(|_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported carry kind `{kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                        return current_carries.get(source_index).cloned().map(|value| (value, "add")).ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` references unavailable carry source `{kind}` during LLVM lowering",
                                    node.name,
                                )
                            });
                    }
                    if let Some(rest) = kind.strip_prefix("mul_prev_carry") {
                        let source_index = rest.parse::<usize>().map_err(|_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported carry kind `{kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                        return current_carries.get(source_index).cloned().map(|value| (value, "mul")).ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` references unavailable carry source `{kind}` during LLVM lowering",
                                    node.name,
                                )
                            });
                    }
                    if let Some(rest) = kind.strip_prefix("add_carry") {
                        let source_index = rest.parse::<usize>().map_err(|_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported carry kind `{kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                        return next_carries.get(source_index).cloned().map(|value| (value, "add")).ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` references unavailable carry source `{kind}` during LLVM lowering",
                                    node.name,
                                )
                            });
                    }
                    if let Some(rest) = kind.strip_prefix("mul_carry") {
                        let source_index = rest.parse::<usize>().map_err(|_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported carry kind `{kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                        return next_carries.get(source_index).cloned().map(|value| (value, "mul")).ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` references unavailable carry source `{kind}` during LLVM lowering",
                                    node.name,
                                )
                            });
                    }
                    Err(format!(
                            "cpu.{loop_instruction} `{}` has unsupported carry kind `{kind}` during LLVM lowering",
                            node.name,
                        ))
                };
                let mut next_carries = Vec::new();
                for (index, (cond_kind, cond_rhs, then_source, else_source)) in
                    lowered_carry_specs.iter().enumerate()
                {
                    let then_value = if matches!(then_source.0.as_str(), "keep" | "keep_prev_carry") {
                        current_carries[index].clone()
                    } else {
                        let (source, op) =
                            resolve_source(then_source, &next_current, &next_carries)?;
                        emit_loop_numeric_op(
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
                        })?
                    };
                    let else_value = if matches!(else_source.0.as_str(), "keep" | "keep_prev_carry") {
                        current_carries[index].clone()
                    } else {
                        let (source, op) =
                            resolve_source(else_source, &next_current, &next_carries)?;
                        emit_loop_numeric_op(
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
                        })?
                    };
                    let next_carry = if cond_kind == "always" {
                        then_value
                    } else {
                        let lhs = if matches!(
                            cond_kind.as_str(),
                            "current_eq"
                                | "current_ne"
                                | "current_lt"
                                | "current_le"
                                | "current_gt"
                                | "current_ge"
                        ) {
                            next_current.clone()
                        } else if matches!(
                            cond_kind.as_str(),
                            "prev_current_eq"
                                | "prev_current_ne"
                                | "prev_current_lt"
                                | "prev_current_le"
                                | "prev_current_gt"
                                | "prev_current_ge"
                        ) {
                            current.clone()
                        } else if let Some(rest) = cond_kind.strip_prefix("prev_carry") {
                            let (index_text, suffix) = rest.split_once('_').ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{cond_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            let source_index = index_text.parse::<usize>().map_err(|_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{cond_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            if suffix != "eq"
                                && suffix != "ne"
                                && suffix != "lt"
                                && suffix != "le"
                                && suffix != "gt"
                                && suffix != "ge"
                            {
                                return Err(format!(
                                    "cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{cond_kind}` during LLVM lowering",
                                    node.name,
                                ));
                            }
                            current_carries.get(source_index).cloned().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` references unavailable conditional carry source `{cond_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?
                        } else if let Some(rest) = cond_kind.strip_prefix("carry") {
                            let (index_text, suffix) = rest.split_once('_').ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{cond_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            let source_index = index_text.parse::<usize>().map_err(|_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{cond_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            if suffix != "eq"
                                && suffix != "ne"
                                && suffix != "lt"
                                && suffix != "le"
                                && suffix != "gt"
                                && suffix != "ge"
                            {
                                return Err(format!(
                                    "cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{cond_kind}` during LLVM lowering",
                                    node.name,
                                ));
                            }
                            next_carries.get(source_index).cloned().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` references unavailable conditional carry source `{cond_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?
                        } else {
                            return Err(format!(
                                "cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{cond_kind}` during LLVM lowering",
                                node.name,
                            ));
                        };
                        let rhs = cond_rhs.clone().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` is missing condition rhs during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let cond_compare =
                            if cond_kind.ends_with("_eq") || cond_kind == "current_eq" {
                                "eq"
                            } else if cond_kind.ends_with("_ne") || cond_kind == "current_ne" {
                                "ne"
                            } else if cond_kind.ends_with("_lt") || cond_kind == "current_lt" {
                                "lt"
                            } else if cond_kind.ends_with("_le") || cond_kind == "current_le" {
                                "le"
                            } else if cond_kind.ends_with("_gt") || cond_kind == "current_gt" {
                                "gt"
                            } else {
                                "ge"
                            };
                        let cond_reg = emit_loop_compare(
                            &mut body,
                            &mut next_reg,
                            loop_scalar_kind,
                            cond_compare,
                            &lhs,
                            &rhs,
                        )
                        .map_err(|error| {
                            format!(
                                "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let select_reg = fresh_reg(&mut next_reg);
                        body.push(format!(
                            "  {select_reg} = select i1 {cond_reg}, {scalar_ty} {then_value}, {scalar_ty} {else_value}"
                        ));
                        select_reg
                    };
                    next_carries.push(next_carry);
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
