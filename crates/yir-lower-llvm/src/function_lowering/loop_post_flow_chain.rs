macro_rules! lower_loop_post_flow_chain {
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
                let control_rhs_value = registers.get(&node.op.args[6]).cloned();
                let (
                    Some(initial_value),
                    Some(limit_value),
                    Some(step_value),
                    Some(control_rhs_value),
                ) = (initial_value, limit_value, step_value, control_rhs_value)
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
                let Some(control_rhs) = coerce_to_i64(&control_rhs_value, &mut body, &mut next_reg)
                else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its control rhs is not coercible to i64",
                        node.name,
                    ));
                    continue;
                };
                let cmp_kind = node.op.args[3].as_str();
                let step_kind = node.op.args[4].as_str();
                let control_kind = node.op.args[5].as_str();
                let control_action = node.op.args[7].as_str();
                let mut carry_initials = Vec::new();
                let mut carry_kinds = Vec::new();
                let mut deferred = false;
                for chunk in node.op.args[8..].chunks(2) {
                    let carry_initial_value = registers.get(&chunk[0]).cloned();
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
                    carry_initials.push(carry_initial);
                    carry_kinds.push(chunk[1].clone());
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
                let loop_action =
                    fresh_block(&mut next_block, &format!("{loop_block_prefix}_action"));
                let loop_continue =
                    fresh_block(&mut next_block, &format!("{loop_block_prefix}_continue"));
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
                for (index, carry_kind) in carry_kinds.iter().enumerate() {
                    let source = if carry_kind == "add_current" {
                        next_current.clone()
                    } else if let Some(rest) = carry_kind.strip_prefix("add_carry") {
                        let source_index = rest.parse::<usize>().map_err(|_| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        next_carries.get(source_index).cloned().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?
                    } else {
                        return Err(format!(
                            "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                            node.name,
                        ));
                    };
                    let reg = fresh_reg(&mut next_reg);
                    body.push(format!(
                        "  {reg} = add i64 {}, {}",
                        current_carries[index], source
                    ));
                    next_carries.push(reg);
                }
                let (control_lhs, control_pred) = match control_kind {
                    "current_eq" => (next_current.clone(), "eq"),
                    "current_ne" => (next_current.clone(), "ne"),
                    "current_lt" => (next_current.clone(), "slt"),
                    "current_le" => (next_current.clone(), "sle"),
                    "current_gt" => (next_current.clone(), "sgt"),
                    "current_ge" => (next_current.clone(), "sge"),
                    other if other.starts_with("carry") && other.ends_with("_eq") => {
                        let source_index = other[5..other.len() - 3].parse::<usize>().map_err(
                            |_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                                    node.name,
                                )
                            },
                        )?;
                        let lhs = next_carries.get(source_index).cloned().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (lhs, "eq")
                    }
                    other if other.starts_with("carry") && other.ends_with("_ne") => {
                        let source_index = other[5..other.len() - 3].parse::<usize>().map_err(
                            |_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                                    node.name,
                                )
                            },
                        )?;
                        let lhs = next_carries.get(source_index).cloned().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (lhs, "ne")
                    }
                    other if other.starts_with("carry") && other.ends_with("_lt") => {
                        let source_index = other[5..other.len() - 3].parse::<usize>().map_err(
                            |_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                                    node.name,
                                )
                            },
                        )?;
                        let lhs = next_carries.get(source_index).cloned().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (lhs, "slt")
                    }
                    other if other.starts_with("carry") && other.ends_with("_le") => {
                        let source_index = other[5..other.len() - 3].parse::<usize>().map_err(
                            |_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                                    node.name,
                                )
                            },
                        )?;
                        let lhs = next_carries.get(source_index).cloned().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (lhs, "sle")
                    }
                    other if other.starts_with("carry") && other.ends_with("_gt") => {
                        let source_index = other[5..other.len() - 3].parse::<usize>().map_err(
                            |_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                                    node.name,
                                )
                            },
                        )?;
                        let lhs = next_carries.get(source_index).cloned().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (lhs, "sgt")
                    }
                    other if other.starts_with("carry") && other.ends_with("_ge") => {
                        let source_index = other[5..other.len() - 3].parse::<usize>().map_err(
                            |_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                                    node.name,
                                )
                            },
                        )?;
                        let lhs = next_carries.get(source_index).cloned().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (lhs, "sge")
                    }
                    other => {
                        return Err(format!(
                            "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                            node.name,
                        ));
                    }
                };
                let control_cond = fresh_reg(&mut next_reg);
                body.push(format!(
                    "  {control_cond} = icmp {control_pred} i64 {control_lhs}, {control_rhs}"
                ));
                body.push(format!(
                    "  br i1 {control_cond}, label %{loop_action}, label %{loop_continue}"
                ));
                body.push(format!("{loop_action}:"));
                body.push(format!("  store i64 {next_current}, ptr {current_slot}"));
                for (carry_slot, next_carry) in carry_slots.iter().zip(next_carries.iter()) {
                    body.push(format!("  store i64 {next_carry}, ptr {carry_slot}"));
                }
                match control_action {
                    "break" => body.push(format!("  br label %{loop_exit}")),
                    "continue" => body.push(format!("  br label %{loop_cond}")),
                    other => {
                        return Err(format!(
                            "cpu.{loop_instruction} `{}` has unsupported control action `{other}` during LLVM lowering",
                            node.name,
                        ));
                    }
                }
                body.push(format!("{loop_continue}:"));
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
