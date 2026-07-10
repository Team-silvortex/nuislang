macro_rules! lower_loop_async_cond_chain {
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
                let mut deferred = false;
                for chunk in node.op.args[4..].chunks(5) {
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
                    let cond_rhs = if chunk[1] == "always" {
                        None
                    } else {
                        let cond_rhs_value = registers.get(&chunk[2]).cloned();
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
                    carry_initials.push(carry_initial);
                    carry_specs.push((
                        chunk[1].clone(),
                        cond_rhs,
                        chunk[3].clone(),
                        chunk[4].clone(),
                    ));
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
                    other => return Err(format!(
                        "cpu.{loop_instruction} `{}` has unsupported compare kind `{other}` during LLVM lowering",
                        node.name,
                    )),
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
                let resolve_source = |kind: &str,
                                      next_current: &String,
                                      next_carries: &Vec<String>|
                 -> Result<(String, &'static str), String> {
                    if matches!(kind, "keep" | "keep_prev_carry") {
                        return Ok((String::new(), "keep"));
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
                let eval_cond = |kind: &str,
                                 rhs: &Option<String>,
                                 next_current: &String,
                                 next_carries: &Vec<String>,
                                 body: &mut Vec<String>,
                                 next_reg: &mut usize|
                 -> Result<String, String> {
                    if kind == "always" {
                        return Ok("1".to_owned());
                    }
                    let rhs = rhs.as_ref().ok_or_else(|| format!(
                        "cpu.{loop_instruction} `{}` is missing condition rhs for `{kind}` during LLVM lowering",
                        node.name,
                    ))?;
                    let (lhs, pred) = match kind {
                        "current_eq" => (next_current.clone(), "eq"),
                        "current_ne" => (next_current.clone(), "ne"),
                        "current_lt" => (next_current.clone(), "slt"),
                        "current_le" => (next_current.clone(), "sle"),
                        "current_gt" => (next_current.clone(), "sgt"),
                        "current_ge" => (next_current.clone(), "sge"),
                        "prev_current_eq" => (current.clone(), "eq"),
                        "prev_current_ne" => (current.clone(), "ne"),
                        "prev_current_lt" => (current.clone(), "slt"),
                        "prev_current_le" => (current.clone(), "sle"),
                        "prev_current_gt" => (current.clone(), "sgt"),
                        "prev_current_ge" => (current.clone(), "sge"),
                        other if other.starts_with("prev_carry") && other.ends_with("_eq") => {
                            let idx = other[10..other.len()-3].parse::<usize>().map_err(|_| format!(
                                "cpu.{loop_instruction} `{}` has unsupported condition kind `{other}` during LLVM lowering",
                                node.name,
                            ))?;
                            (current_carries.get(idx).cloned().ok_or_else(|| format!(
                                "cpu.{loop_instruction} `{}` references unavailable condition source `{other}` during LLVM lowering",
                                node.name,
                            ))?, "eq")
                        }
                        other if other.starts_with("prev_carry") && other.ends_with("_ne") => {
                            let idx = other[10..other.len()-3].parse::<usize>().map_err(|_| format!(
                                "cpu.{loop_instruction} `{}` has unsupported condition kind `{other}` during LLVM lowering",
                                node.name,
                            ))?;
                            (current_carries.get(idx).cloned().ok_or_else(|| format!(
                                "cpu.{loop_instruction} `{}` references unavailable condition source `{other}` during LLVM lowering",
                                node.name,
                            ))?, "ne")
                        }
                        other if other.starts_with("prev_carry") && other.ends_with("_lt") => {
                            let idx = other[10..other.len()-3].parse::<usize>().map_err(|_| format!(
                                "cpu.{loop_instruction} `{}` has unsupported condition kind `{other}` during LLVM lowering",
                                node.name,
                            ))?;
                            (current_carries.get(idx).cloned().ok_or_else(|| format!(
                                "cpu.{loop_instruction} `{}` references unavailable condition source `{other}` during LLVM lowering",
                                node.name,
                            ))?, "slt")
                        }
                        other if other.starts_with("prev_carry") && other.ends_with("_le") => {
                            let idx = other[10..other.len()-3].parse::<usize>().map_err(|_| format!(
                                "cpu.{loop_instruction} `{}` has unsupported condition kind `{other}` during LLVM lowering",
                                node.name,
                            ))?;
                            (current_carries.get(idx).cloned().ok_or_else(|| format!(
                                "cpu.{loop_instruction} `{}` references unavailable condition source `{other}` during LLVM lowering",
                                node.name,
                            ))?, "sle")
                        }
                        other if other.starts_with("prev_carry") && other.ends_with("_gt") => {
                            let idx = other[10..other.len()-3].parse::<usize>().map_err(|_| format!(
                                "cpu.{loop_instruction} `{}` has unsupported condition kind `{other}` during LLVM lowering",
                                node.name,
                            ))?;
                            (current_carries.get(idx).cloned().ok_or_else(|| format!(
                                "cpu.{loop_instruction} `{}` references unavailable condition source `{other}` during LLVM lowering",
                                node.name,
                            ))?, "sgt")
                        }
                        other if other.starts_with("prev_carry") && other.ends_with("_ge") => {
                            let idx = other[10..other.len()-3].parse::<usize>().map_err(|_| format!(
                                "cpu.{loop_instruction} `{}` has unsupported condition kind `{other}` during LLVM lowering",
                                node.name,
                            ))?;
                            (current_carries.get(idx).cloned().ok_or_else(|| format!(
                                "cpu.{loop_instruction} `{}` references unavailable condition source `{other}` during LLVM lowering",
                                node.name,
                            ))?, "sge")
                        }
                        other if other.starts_with("carry") && other.ends_with("_eq") => {
                            let idx = other[5..other.len()-3].parse::<usize>().map_err(|_| format!(
                                "cpu.{loop_instruction} `{}` has unsupported condition kind `{other}` during LLVM lowering",
                                node.name,
                            ))?;
                            (next_carries.get(idx).cloned().ok_or_else(|| format!(
                                "cpu.{loop_instruction} `{}` references unavailable condition source `{other}` during LLVM lowering",
                                node.name,
                            ))?, "eq")
                        }
                        other if other.starts_with("carry") && other.ends_with("_ne") => {
                            let idx = other[5..other.len()-3].parse::<usize>().map_err(|_| format!(
                                "cpu.{loop_instruction} `{}` has unsupported condition kind `{other}` during LLVM lowering",
                                node.name,
                            ))?;
                            (next_carries.get(idx).cloned().ok_or_else(|| format!(
                                "cpu.{loop_instruction} `{}` references unavailable condition source `{other}` during LLVM lowering",
                                node.name,
                            ))?, "ne")
                        }
                        other if other.starts_with("carry") && other.ends_with("_lt") => {
                            let idx = other[5..other.len()-3].parse::<usize>().map_err(|_| format!(
                                "cpu.{loop_instruction} `{}` has unsupported condition kind `{other}` during LLVM lowering",
                                node.name,
                            ))?;
                            (next_carries.get(idx).cloned().ok_or_else(|| format!(
                                "cpu.{loop_instruction} `{}` references unavailable condition source `{other}` during LLVM lowering",
                                node.name,
                            ))?, "slt")
                        }
                        other if other.starts_with("carry") && other.ends_with("_le") => {
                            let idx = other[5..other.len()-3].parse::<usize>().map_err(|_| format!(
                                "cpu.{loop_instruction} `{}` has unsupported condition kind `{other}` during LLVM lowering",
                                node.name,
                            ))?;
                            (next_carries.get(idx).cloned().ok_or_else(|| format!(
                                "cpu.{loop_instruction} `{}` references unavailable condition source `{other}` during LLVM lowering",
                                node.name,
                            ))?, "sle")
                        }
                        other if other.starts_with("carry") && other.ends_with("_gt") => {
                            let idx = other[5..other.len()-3].parse::<usize>().map_err(|_| format!(
                                "cpu.{loop_instruction} `{}` has unsupported condition kind `{other}` during LLVM lowering",
                                node.name,
                            ))?;
                            (next_carries.get(idx).cloned().ok_or_else(|| format!(
                                "cpu.{loop_instruction} `{}` references unavailable condition source `{other}` during LLVM lowering",
                                node.name,
                            ))?, "sgt")
                        }
                        other if other.starts_with("carry") && other.ends_with("_ge") => {
                            let idx = other[5..other.len()-3].parse::<usize>().map_err(|_| format!(
                                "cpu.{loop_instruction} `{}` has unsupported condition kind `{other}` during LLVM lowering",
                                node.name,
                            ))?;
                            (next_carries.get(idx).cloned().ok_or_else(|| format!(
                                "cpu.{loop_instruction} `{}` references unavailable condition source `{other}` during LLVM lowering",
                                node.name,
                            ))?, "sge")
                        }
                        other => return Err(format!(
                            "cpu.{loop_instruction} `{}` has unsupported condition kind `{other}` during LLVM lowering",
                            node.name,
                        )),
                    };
                    let reg = fresh_reg(next_reg);
                    body.push(format!("  {reg} = icmp {pred} i64 {lhs}, {rhs}"));
                    Ok(reg)
                };
                let mut next_carries = Vec::new();
                for (index, (cond_kind, cond_rhs, then_kind, else_kind)) in
                    carry_specs.iter().enumerate()
                {
                    let cond = eval_cond(
                        cond_kind,
                        cond_rhs,
                        &next_current,
                        &next_carries,
                        &mut body,
                        &mut next_reg,
                    )?;
                    let then_block =
                        fresh_block(&mut next_block, "loop_while_scalar_async_cond_then");
                    let else_block =
                        fresh_block(&mut next_block, "loop_while_scalar_async_cond_else");
                    let merge_block =
                        fresh_block(&mut next_block, "loop_while_scalar_async_cond_merge");
                    body.push(format!(
                        "  br i1 {cond}, label %{then_block}, label %{else_block}"
                    ));
                    body.push(format!("{then_block}:"));
                    let (then_source, then_op) =
                        resolve_source(then_kind, &next_current, &next_carries)?;
                    let then_value = if matches!(then_op, "keep" | "keep_prev_carry") {
                        current_carries[index].clone()
                    } else {
                        let reg = fresh_reg(&mut next_reg);
                        body.push(format!(
                            "  {reg} = {then_op} i64 {}, {}",
                            current_carries[index], then_source
                        ));
                        reg
                    };
                    body.push(format!("  br label %{merge_block}"));
                    body.push(format!("{else_block}:"));
                    let (else_source, else_op) =
                        resolve_source(else_kind, &next_current, &next_carries)?;
                    let else_value = if matches!(else_op, "keep" | "keep_prev_carry") {
                        current_carries[index].clone()
                    } else {
                        let reg = fresh_reg(&mut next_reg);
                        body.push(format!(
                            "  {reg} = {else_op} i64 {}, {}",
                            current_carries[index], else_source
                        ));
                        reg
                    };
                    body.push(format!("  br label %{merge_block}"));
                    body.push(format!("{merge_block}:"));
                    let merged = fresh_reg(&mut next_reg);
                    body.push(format!(
                        "  {merged} = phi i64 [{then_value}, %{then_block}], [{else_value}, %{else_block}]"
                    ));
                    next_carries.push(merged);
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
