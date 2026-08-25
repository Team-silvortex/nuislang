macro_rules! lower_loop_post_flow_cond_chain {
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
                let (flow_expr, carry_start_index) =
                    parse_loop_flow_expr_for_llvm(&node.op.args, 5, &node.name, loop_instruction)?;
                let (Some(initial_value), Some(limit_value), Some(step_value)) =
                    (initial_value, limit_value, step_value)
                else {
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
                let Some(step) = coerce_to_i64(&step_value, &mut body, &mut next_reg) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its step value is not coercible to i64",
                        node.name,
                    ));
                    continue;
                };
                let cmp_kind = node.op.args[3].as_str();
                let step_kind = node.op.args[4].as_str();
                let Some(resolved_flow_expr) = resolve_loop_flow_expr_for_llvm(
                    &flow_expr,
                    &registers,
                    &mut body,
                    &mut next_reg,
                    &node.name,
                    loop_instruction,
                ) else {
                    continue;
                };
                let mut carry_initials = Vec::new();
                let mut carry_specs =
                    Vec::<(String, Option<String>, Vec<String>, Vec<String>)>::new();
                let mut deferred = false;
                let mut cursor = carry_start_index;
                while cursor < node.op.args.len() {
                    let Some(initial_name) = node.op.args.get(cursor) else {
                        break;
                    };
                    let Some(cond_kind) = node.op.args.get(cursor + 1) else {
                        return Err(format!(
                            "cpu.{loop_instruction} `{}` has truncated carry condition during LLVM lowering",
                            node.name,
                        ));
                    };
                    let Some(cond_rhs_name) = node.op.args.get(cursor + 2) else {
                        return Err(format!(
                            "cpu.{loop_instruction} `{}` has truncated carry condition rhs during LLVM lowering",
                            node.name,
                        ));
                    };
                    let carry_initial_value = registers.get(initial_name).cloned();
                    let Some(carry_initial_value) = carry_initial_value else {
                        body.push(format!(
                            "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry initials are outside the current CPU LLVM slice",
                            node.name,
                        ));
                        deferred = true;
                        break;
                    };
                    let Some(carry_initial) =
                        coerce_to_i64(&carry_initial_value, &mut body, &mut next_reg)
                    else {
                        body.push(format!(
                            "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry initials are not coercible to i64",
                            node.name,
                        ));
                        deferred = true;
                        break;
                    };
                    let cond_rhs = if cond_kind == "always" {
                        None
                    } else {
                        let cond_rhs_value = registers.get(cond_rhs_name).cloned();
                        let Some(cond_rhs_value) = cond_rhs_value else {
                            body.push(format!(
                                "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more condition rhs values are outside the current CPU LLVM slice",
                                node.name,
                            ));
                            deferred = true;
                            break;
                        };
                        let Some(cond_rhs) =
                            coerce_to_i64(&cond_rhs_value, &mut body, &mut next_reg)
                        else {
                            body.push(format!(
                                "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more condition rhs values are not coercible to i64",
                                node.name,
                            ));
                            deferred = true;
                            break;
                        };
                        Some(cond_rhs)
                    };
                    let then_start = cursor + 3;
                    let Some(then_kind) = node.op.args.get(then_start).cloned() else {
                        return Err(format!(
                            "cpu.{loop_instruction} `{}` has truncated then carry source during LLVM lowering",
                            node.name,
                        ));
                    };
                    let Some(then_payload_len) =
                        async_post_flow_carry_source_payload_len(&then_kind)
                    else {
                        return Err(format!(
                            "cpu.{loop_instruction} `{}` has unsupported carry kind `{then_kind}` during LLVM lowering",
                            node.name,
                        ));
                    };
                    let then_end = then_start + 1 + then_payload_len;
                    if then_end > node.op.args.len() {
                        return Err(format!(
                            "cpu.{loop_instruction} `{}` is missing payload for carry kind `{then_kind}` during LLVM lowering",
                            node.name,
                        ));
                    }
                    let mut then_source = vec![then_kind];
                    for payload_name in &node.op.args[then_start + 1..then_end] {
                        let Some(payload_value) = registers.get(payload_name).cloned() else {
                            body.push(format!(
                                "  ; deferred lowering for cpu.{loop_instruction} `{}` because carry source payload `{payload_name}` is outside the current CPU LLVM slice",
                                node.name,
                            ));
                            deferred = true;
                            break;
                        };
                        let Some(payload) =
                            coerce_to_i64(&payload_value, &mut body, &mut next_reg)
                        else {
                            body.push(format!(
                                "  ; deferred lowering for cpu.{loop_instruction} `{}` because carry source payload `{payload_name}` is not coercible to i64",
                                node.name,
                            ));
                            deferred = true;
                            break;
                        };
                        then_source.push(payload);
                    }
                    if deferred {
                        break;
                    }
                    let Some(else_kind) = node.op.args.get(then_end).cloned() else {
                        return Err(format!(
                            "cpu.{loop_instruction} `{}` has truncated else carry source during LLVM lowering",
                            node.name,
                        ));
                    };
                    let Some(else_payload_len) =
                        async_post_flow_carry_source_payload_len(&else_kind)
                    else {
                        return Err(format!(
                            "cpu.{loop_instruction} `{}` has unsupported carry kind `{else_kind}` during LLVM lowering",
                            node.name,
                        ));
                    };
                    let else_end = then_end + 1 + else_payload_len;
                    if else_end > node.op.args.len() {
                        return Err(format!(
                            "cpu.{loop_instruction} `{}` is missing payload for carry kind `{else_kind}` during LLVM lowering",
                            node.name,
                        ));
                    }
                    let mut else_source = vec![else_kind];
                    for payload_name in &node.op.args[then_end + 1..else_end] {
                        let Some(payload_value) = registers.get(payload_name).cloned() else {
                            body.push(format!(
                                "  ; deferred lowering for cpu.{loop_instruction} `{}` because carry source payload `{payload_name}` is outside the current CPU LLVM slice",
                                node.name,
                            ));
                            deferred = true;
                            break;
                        };
                        let Some(payload) =
                            coerce_to_i64(&payload_value, &mut body, &mut next_reg)
                        else {
                            body.push(format!(
                                "  ; deferred lowering for cpu.{loop_instruction} `{}` because carry source payload `{payload_name}` is not coercible to i64",
                                node.name,
                            ));
                            deferred = true;
                            break;
                        };
                        else_source.push(payload);
                    }
                    if deferred {
                        break;
                    }
                    carry_initials.push(carry_initial);
                    carry_specs.push((cond_kind.clone(), cond_rhs, then_source, else_source));
                    cursor = else_end;
                }
                if deferred {
                    continue;
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
                let loop_continue =
                    fresh_block(&mut next_block, &format!("{loop_block_prefix}_continue"));
                let loop_exit = fresh_block(&mut next_block, &format!("{loop_block_prefix}_exit"));
                body.push(format!("  br label %{loop_cond}"));
                body.push(format!("{loop_cond}:"));
                let current = fresh_reg(&mut next_reg);
                body.push(format!("  {current} = load i64, ptr {current_slot}"));
                if cmp_kind == "always" {
                    body.push(format!("  br label %{loop_body}"));
                } else {
                    let cmp = if let Some(index) = cmp_kind.strip_prefix("pattern_carry") {
                        let index = index.parse::<usize>().map_err(|_| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported compare kind `{cmp_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let carry_slot = carry_slots.get(index).ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable pattern entry carry `{cmp_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let active = fresh_reg(&mut next_reg);
                        body.push(format!("  {active} = load i64, ptr {carry_slot}"));
                        let cmp = fresh_reg(&mut next_reg);
                        body.push(format!("  {cmp} = icmp ne i64 {active}, 0"));
                        cmp
                    } else if matches!(cmp_kind, "invariant_true" | "pattern_exit") {
                        let cmp = fresh_reg(&mut next_reg);
                        body.push(format!("  {cmp} = icmp ne i64 {limit}, 0"));
                        cmp
                    } else {
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
                        let cmp = fresh_reg(&mut next_reg);
                        body.push(format!("  {cmp} = icmp {pred} i64 {current}, {limit}"));
                        cmp
                    };
                    body.push(format!(
                        "  br i1 {cmp}, label %{loop_body}, label %{loop_exit}"
                    ));
                }
                body.push(format!("{loop_body}:"));
                let next_current = match step_kind {
                    "add" => {
                        let reg = fresh_reg(&mut next_reg);
                        body.push(format!("  {reg} = add i64 {current}, {step}"));
                        reg
                    }
                    "sub" => {
                        let reg = fresh_reg(&mut next_reg);
                        body.push(format!("  {reg} = sub i64 {current}, {step}"));
                        reg
                    }
                    other => {
                        return Err(format!(
                            "cpu.{loop_instruction} `{}` has unsupported step kind `{other}` during LLVM lowering",
                            node.name,
                        ));
                    }
                };
                let mut current_carries = Vec::new();
                for carry_slot in &carry_slots {
                    let carry_before = fresh_reg(&mut next_reg);
                    body.push(format!("  {carry_before} = load i64, ptr {carry_slot}"));
                    current_carries.push(carry_before);
                }
                let mut next_carries = Vec::new();
                for (index, (cond_kind, cond_rhs, then_source_spec, else_source_spec)) in
                    carry_specs.iter().enumerate()
                {
                    let then_value = if matches!(
                        then_source_spec[0].as_str(),
                        "keep" | "keep_prev_carry"
                    ) {
                        current_carries[index].clone()
                    } else {
                        let source = resolve_source_for_async_post_flow(
                            then_source_spec,
                            &current,
                            &next_current,
                            &current_carries,
                            &next_carries,
                            &mut body,
                            &mut next_reg,
                            &node.name,
                            loop_instruction,
                        )?;
                        let reg = fresh_reg(&mut next_reg);
                        body.push(format!(
                            "  {reg} = add i64 {}, {}",
                            current_carries[index], source
                        ));
                        reg
                    };
                    let else_value = if matches!(
                        else_source_spec[0].as_str(),
                        "keep" | "keep_prev_carry"
                    ) {
                        current_carries[index].clone()
                    } else {
                        let source = resolve_source_for_async_post_flow(
                            else_source_spec,
                            &current,
                            &next_current,
                            &current_carries,
                            &next_carries,
                            &mut body,
                            &mut next_reg,
                            &node.name,
                            loop_instruction,
                        )?;
                        let reg = fresh_reg(&mut next_reg);
                        body.push(format!(
                            "  {reg} = add i64 {}, {}",
                            current_carries[index], source
                        ));
                        reg
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
                        let cond_pred = if cond_kind.ends_with("_eq") || cond_kind == "current_eq" {
                            "eq"
                        } else if cond_kind.ends_with("_ne") || cond_kind == "current_ne" {
                            "ne"
                        } else if cond_kind.ends_with("_lt") || cond_kind == "current_lt" {
                            "slt"
                        } else if cond_kind.ends_with("_le") || cond_kind == "current_le" {
                            "sle"
                        } else if cond_kind.ends_with("_gt") || cond_kind == "current_gt" {
                            "sgt"
                        } else {
                            "sge"
                        };
                        let cond_reg = fresh_reg(&mut next_reg);
                        body.push(format!("  {cond_reg} = icmp {cond_pred} i64 {lhs}, {rhs}"));
                        let select_reg = fresh_reg(&mut next_reg);
                        body.push(format!(
                            "  {select_reg} = select i1 {cond_reg}, i64 {then_value}, i64 {else_value}"
                        ));
                        select_reg
                    };
                    next_carries.push(next_carry);
                }
                let mut flow_leaves: Vec<(&ResolvedLoopControlExpr, &str)> = Vec::new();
                collect_resolved_loop_flow_leaves(&resolved_flow_expr, &mut flow_leaves);
                let condition_blocks = (0..flow_leaves.len())
                    .map(|index| {
                        if index == 0 {
                            None
                        } else {
                            Some(fresh_block(&mut next_block, "loop_post_flow_rhs"))
                        }
                    })
                    .collect::<Vec<_>>();
                for (index, (condition, action)) in flow_leaves.iter().enumerate() {
                    if let Some(block) = &condition_blocks[index] {
                        body.push(format!("{block}:"));
                    }
                    let no_match_block = condition_blocks
                        .get(index + 1)
                        .and_then(|block| block.clone())
                        .unwrap_or_else(|| loop_continue.clone());
                    let control_cond = emit_post_loop_flow_control_expr(
                        condition,
                        &next_current,
                        &next_carries,
                        &current,
                        &current_carries,
                        &mut body,
                        &mut next_reg,
                        &node.name,
                        loop_instruction,
                    )?;
                    let action_block = fresh_block(&mut next_block, "loop_post_flow_action");
                    body.push(format!(
                        "  br i1 {control_cond}, label %{action_block}, label %{no_match_block}"
                    ));
                    body.push(format!("{action_block}:"));
                    body.push(format!("  store i64 {next_current}, ptr {current_slot}"));
                    for (carry_slot, next_carry) in carry_slots.iter().zip(next_carries.iter()) {
                        body.push(format!("  store i64 {next_carry}, ptr {carry_slot}"));
                    }
                    match *action {
                        "break" => body.push(format!("  br label %{loop_exit}")),
                        "continue" if cmp_kind == "pattern_exit" => {
                            body.push(format!("  br label %{loop_exit}"))
                        }
                        "continue" => body.push(format!("  br label %{loop_cond}")),
                        other => {
                            return Err(format!(
                                "cpu.{loop_instruction} `{}` has unsupported flow action `{other}` during LLVM lowering",
                                node.name,
                            ));
                        }
                    }
                }
                body.push(format!("{loop_continue}:"));
                body.push(format!("  store i64 {next_current}, ptr {current_slot}"));
                for (carry_slot, next_carry) in carry_slots.iter().zip(next_carries.iter()) {
                    body.push(format!("  store i64 {next_carry}, ptr {carry_slot}"));
                }
                if cmp_kind == "pattern_exit" {
                    body.push(format!("  br label %{loop_exit}"));
                } else {
                    body.push(format!("  br label %{loop_cond}"));
                }
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
                let mut fields = vec![(
                    "current".to_owned(),
                    LlvmValueRef::I64(final_current.clone()),
                )];
                for (index, final_carry) in final_carries.iter().enumerate() {
                    fields.push((
                        format!("carry{index}"),
                        LlvmValueRef::I64(final_carry.clone()),
                    ));
                }
                registers.insert(
                    node.name.clone(),
                    LlvmValueRef::Struct(StructLlvmValueRef {
                        type_name: "LoopChain".to_owned(),
                        fields,
                    }),
                );
                *last_cpu_value = final_carries.last().cloned().or(Some(final_current));
    }};
}
