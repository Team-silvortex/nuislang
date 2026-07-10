macro_rules! lower_loop_async_flow_cond_chain {
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
                fn resolve_control_operand(
                    kind: &str,
                    next_current: &String,
                    current_carries: &Vec<String>,
                    node_name: &str,
                ) -> Result<(String, &'static str), String> {
                    match kind {
                        "current_eq" => Ok((next_current.clone(), "eq")),
                        "current_ne" => Ok((next_current.clone(), "ne")),
                        "current_lt" => Ok((next_current.clone(), "slt")),
                        "current_le" => Ok((next_current.clone(), "sle")),
                        "current_gt" => Ok((next_current.clone(), "sgt")),
                        "current_ge" => Ok((next_current.clone(), "sge")),
                        other if other.starts_with("carry") && other.ends_with("_eq") => {
                            let i = other[5..other.len() - 3].parse::<usize>().map_err(|_| format!("cpu.loop_while_scalar_async_flow_cond_chain `{}` has unsupported control kind `{other}` during LLVM lowering", node_name))?;
                            Ok((current_carries.get(i).cloned().ok_or_else(|| format!("cpu.loop_while_scalar_async_flow_cond_chain `{}` references unavailable control source `{other}` during LLVM lowering", node_name))?, "eq"))
                        }
                        other if other.starts_with("carry") && other.ends_with("_ne") => {
                            let i = other[5..other.len() - 3].parse::<usize>().map_err(|_| format!("cpu.loop_while_scalar_async_flow_cond_chain `{}` has unsupported control kind `{other}` during LLVM lowering", node_name))?;
                            Ok((current_carries.get(i).cloned().ok_or_else(|| format!("cpu.loop_while_scalar_async_flow_cond_chain `{}` references unavailable control source `{other}` during LLVM lowering", node_name))?, "ne"))
                        }
                        other if other.starts_with("carry") && other.ends_with("_lt") => {
                            let i = other[5..other.len() - 3].parse::<usize>().map_err(|_| format!("cpu.loop_while_scalar_async_flow_cond_chain `{}` has unsupported control kind `{other}` during LLVM lowering", node_name))?;
                            Ok((current_carries.get(i).cloned().ok_or_else(|| format!("cpu.loop_while_scalar_async_flow_cond_chain `{}` references unavailable control source `{other}` during LLVM lowering", node_name))?, "slt"))
                        }
                        other if other.starts_with("carry") && other.ends_with("_le") => {
                            let i = other[5..other.len() - 3].parse::<usize>().map_err(|_| format!("cpu.loop_while_scalar_async_flow_cond_chain `{}` has unsupported control kind `{other}` during LLVM lowering", node_name))?;
                            Ok((current_carries.get(i).cloned().ok_or_else(|| format!("cpu.loop_while_scalar_async_flow_cond_chain `{}` references unavailable control source `{other}` during LLVM lowering", node_name))?, "sle"))
                        }
                        other if other.starts_with("carry") && other.ends_with("_gt") => {
                            let i = other[5..other.len() - 3].parse::<usize>().map_err(|_| format!("cpu.loop_while_scalar_async_flow_cond_chain `{}` has unsupported control kind `{other}` during LLVM lowering", node_name))?;
                            Ok((current_carries.get(i).cloned().ok_or_else(|| format!("cpu.loop_while_scalar_async_flow_cond_chain `{}` references unavailable control source `{other}` during LLVM lowering", node_name))?, "sgt"))
                        }
                        other if other.starts_with("carry") && other.ends_with("_ge") => {
                            let i = other[5..other.len() - 3].parse::<usize>().map_err(|_| format!("cpu.loop_while_scalar_async_flow_cond_chain `{}` has unsupported control kind `{other}` during LLVM lowering", node_name))?;
                            Ok((current_carries.get(i).cloned().ok_or_else(|| format!("cpu.loop_while_scalar_async_flow_cond_chain `{}` references unavailable control source `{other}` during LLVM lowering", node_name))?, "sge"))
                        }
                        other => Err(format!("cpu.loop_while_scalar_async_flow_cond_chain `{}` has unsupported control kind `{other}` during LLVM lowering", node_name)),
                    }
                }
                fn eval_control_expr(
                    expr: &ResolvedLoopControlExpr,
                    next_current: &String,
                    current_carries: &Vec<String>,
                    body: &mut Vec<String>,
                    next_reg: &mut usize,
                    node_name: &str,
                ) -> Result<String, String> {
                    match expr {
                        ResolvedLoopControlExpr::Cond { kind, rhs } => {
                            let (lhs, pred) = resolve_control_operand(
                                kind,
                                next_current,
                                current_carries,
                                node_name,
                            )?;
                            let reg = fresh_reg(next_reg);
                            body.push(format!("  {reg} = icmp {pred} i64 {lhs}, {rhs}"));
                            Ok(reg)
                        }
                        ResolvedLoopControlExpr::And(lhs, rhs) => {
                            let lhs_reg = eval_control_expr(
                                lhs,
                                next_current,
                                current_carries,
                                body,
                                next_reg,
                                node_name,
                            )?;
                            let rhs_reg = eval_control_expr(
                                rhs,
                                next_current,
                                current_carries,
                                body,
                                next_reg,
                                node_name,
                            )?;
                            let reg = fresh_reg(next_reg);
                            body.push(format!("  {reg} = and i1 {lhs_reg}, {rhs_reg}"));
                            Ok(reg)
                        }
                        ResolvedLoopControlExpr::Or(lhs, rhs) => {
                            let lhs_reg = eval_control_expr(
                                lhs,
                                next_current,
                                current_carries,
                                body,
                                next_reg,
                                node_name,
                            )?;
                            let rhs_reg = eval_control_expr(
                                rhs,
                                next_current,
                                current_carries,
                                body,
                                next_reg,
                                node_name,
                            )?;
                            let reg = fresh_reg(next_reg);
                            body.push(format!("  {reg} = or i1 {lhs_reg}, {rhs_reg}"));
                            Ok(reg)
                        }
                    }
                }
                let initial_value = registers.get(&node.op.args[0]).cloned();
                let limit_value = registers.get(&node.op.args[1]).cloned();
                let (flow_expr, carry_start_index) =
                    parse_loop_flow_expr_for_llvm(&node.op.args, 4, &node.name, loop_instruction)?;
                let callee = &node.op.args[2];
                let Some(signature) = helper_signatures.get(callee) else {
                    body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because helper signature `{}` is unavailable", node.name, callee));
                    continue;
                };
                if signature.params.as_slice() != [CpuCallScalarKind::I64]
                    || signature.ret != CpuCallScalarKind::I64
                {
                    body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because helper `{}` must have signature `(i64) -> i64`", node.name, callee));
                    continue;
                }
                let (Some(initial_value), Some(limit_value)) = (initial_value, limit_value) else {
                    body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more inputs are outside the current CPU LLVM slice", node.name));
                    continue;
                };
                let Some(initial) = coerce_to_i64(&initial_value, &mut body, &mut next_reg) else {
                    body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because its initial value is not coercible to i64", node.name));
                    continue;
                };
                let Some(limit) = coerce_to_i64(&limit_value, &mut body, &mut next_reg) else {
                    body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because its limit value is not coercible to i64", node.name));
                    continue;
                };
                let cmp_kind = node.op.args[3].as_str();
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
                let mut carry_specs = Vec::new();
                let mut deferred = false;
                for chunk in node.op.args[carry_start_index..].chunks(5) {
                    let Some(carry_initial_value) = registers.get(&chunk[0]).cloned() else {
                        body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry initials are outside the current CPU LLVM slice", node.name));
                        deferred = true;
                        break;
                    };
                    let Some(carry_initial) =
                        coerce_to_i64(&carry_initial_value, &mut body, &mut next_reg)
                    else {
                        body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry initials are not coercible to i64", node.name));
                        deferred = true;
                        break;
                    };
                    let cond_rhs = if chunk[1] == "always" {
                        None
                    } else {
                        let Some(v) = registers.get(&chunk[2]).cloned() else {
                            body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more condition rhs values are outside the current CPU LLVM slice", node.name));
                            deferred = true;
                            break;
                        };
                        let Some(rhs) = coerce_to_i64(&v, &mut body, &mut next_reg) else {
                            body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more condition rhs values are not coercible to i64", node.name));
                            deferred = true;
                            break;
                        };
                        Some(rhs)
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
                    .map(|init| {
                        let s = fresh_reg(&mut next_reg);
                        body.push(format!("  {s} = alloca i64"));
                        body.push(format!("  store i64 {init}, ptr {s}"));
                        s
                    })
                    .collect::<Vec<_>>();
                let loop_cond = fresh_block(&mut next_block, &format!("{loop_block_prefix}_cond"));
                let loop_body = fresh_block(&mut next_block, &format!("{loop_block_prefix}_body"));
                let loop_update =
                    fresh_block(&mut next_block, &format!("{loop_block_prefix}_update"));
                let loop_exit = fresh_block(&mut next_block, &format!("{loop_block_prefix}_exit"));
                body.push(format!("  br label %{loop_cond}"));
                body.push(format!("{loop_cond}:"));
                let current = fresh_reg(&mut next_reg);
                body.push(format!("  {current} = load i64, ptr {current_slot}"));
                let cmp = fresh_reg(&mut next_reg);
                let pred=match cmp_kind { "eq"=>"eq","ne"=>"ne","lt"=>"slt","le"=>"sle","gt"=>"sgt","ge"=>"sge",other=>return Err(format!("cpu.{loop_instruction} `{}` has unsupported compare kind `{other}` during LLVM lowering", node.name)), };
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
                for slot in &carry_slots {
                    let r = fresh_reg(&mut next_reg);
                    body.push(format!("  {r} = load i64, ptr {slot}"));
                    current_carries.push(r);
                }
                let mut flow_leaves: Vec<(&ResolvedLoopControlExpr, &str)> = Vec::new();
                collect_resolved_loop_flow_leaves(&resolved_flow_expr, &mut flow_leaves);
                let condition_blocks = (0..flow_leaves.len())
                    .map(|index| {
                        if index == 0 {
                            None
                        } else {
                            Some(fresh_block(&mut next_block, "loop_async_flow_rhs"))
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
                        .unwrap_or_else(|| loop_update.clone());
                    let control_cond = eval_control_expr(
                        condition,
                        &next_current,
                        &current_carries,
                        &mut body,
                        &mut next_reg,
                        &node.name,
                    )?;
                    let action_block = fresh_block(&mut next_block, "loop_async_flow_action");
                    body.push(format!(
                        "  br i1 {control_cond}, label %{action_block}, label %{no_match_block}"
                    ));
                    body.push(format!("{action_block}:"));
                    body.push(format!("  store i64 {next_current}, ptr {current_slot}"));
                    match *action {
                        "break" => body.push(format!("  br label %{loop_exit}")),
                        "continue" => body.push(format!("  br label %{loop_cond}")),
                        other => {
                            return Err(format!(
                                "cpu.{loop_instruction} `{}` has unsupported flow action `{other}` during LLVM lowering",
                                node.name,
                            ));
                        }
                    }
                }
                body.push(format!("{loop_update}:"));
                let mut next_carries = Vec::new();
                for (index, (cond_kind, cond_rhs, then_kind, else_kind)) in
                    carry_specs.iter().enumerate()
                {
                    let resolve = |kind: &str,
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
                        if let Some(rest) = kind.strip_prefix("add_prev_carry") {
                            let i=rest.parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported carry kind `{kind}` during LLVM lowering", node.name))?;
                            return Ok((current_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable carry source `{kind}` during LLVM lowering", node.name))?,"add"));
                        }
                        if let Some(rest) = kind.strip_prefix("add_carry") {
                            let i=rest.parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported carry kind `{kind}` during LLVM lowering", node.name))?;
                            return Ok((next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable carry source `{kind}` during LLVM lowering", node.name))?,"add"));
                        }
                        Err(format!("cpu.{loop_instruction} `{}` has unsupported carry kind `{kind}` during LLVM lowering", node.name))
                    };
                    let then_value = {
                        let (src, op) = resolve(then_kind, &next_carries)?;
                        if matches!(op, "keep" | "keep_prev_carry") {
                            current_carries[index].clone()
                        } else {
                            let r = fresh_reg(&mut next_reg);
                            body.push(format!(
                                "  {r} = add i64 {}, {}",
                                current_carries[index], src
                            ));
                            r
                        }
                    };
                    let else_value = {
                        let (src, op) = resolve(else_kind, &next_carries)?;
                        if matches!(op, "keep" | "keep_prev_carry") {
                            current_carries[index].clone()
                        } else {
                            let r = fresh_reg(&mut next_reg);
                            body.push(format!(
                                "  {r} = add i64 {}, {}",
                                current_carries[index], src
                            ));
                            r
                        }
                    };
                    let next_carry = if cond_kind == "always" {
                        then_value
                    } else {
                        let rhs=cond_rhs.clone().ok_or_else(|| format!("cpu.{loop_instruction} `{}` is missing condition rhs during LLVM lowering", node.name))?;
                        let (lhs,pred)=match cond_kind.as_str() {
                            "current_eq"=>(next_current.clone(),"eq"), "current_ne"=>(next_current.clone(),"ne"), "current_lt"=>(next_current.clone(),"slt"), "current_le"=>(next_current.clone(),"sle"), "current_gt"=>(next_current.clone(),"sgt"), "current_ge"=>(next_current.clone(),"sge"),
                            "prev_current_eq"=>(current.clone(),"eq"), "prev_current_ne"=>(current.clone(),"ne"), "prev_current_lt"=>(current.clone(),"slt"), "prev_current_le"=>(current.clone(),"sle"), "prev_current_gt"=>(current.clone(),"sgt"), "prev_current_ge"=>(current.clone(),"sge"),
                            other if other.starts_with("prev_carry") && other.ends_with("_eq") => { let i=other[10..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (current_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"eq") }
                            other if other.starts_with("prev_carry") && other.ends_with("_ne") => { let i=other[10..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (current_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"ne") }
                            other if other.starts_with("prev_carry") && other.ends_with("_lt") => { let i=other[10..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (current_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"slt") }
                            other if other.starts_with("prev_carry") && other.ends_with("_le") => { let i=other[10..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (current_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"sle") }
                            other if other.starts_with("prev_carry") && other.ends_with("_gt") => { let i=other[10..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (current_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"sgt") }
                            other if other.starts_with("prev_carry") && other.ends_with("_ge") => { let i=other[10..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (current_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"sge") }
                            other if other.starts_with("carry") && other.ends_with("_eq") => { let i=other[5..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"eq") }
                            other if other.starts_with("carry") && other.ends_with("_ne") => { let i=other[5..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"ne") }
                            other if other.starts_with("carry") && other.ends_with("_lt") => { let i=other[5..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"slt") }
                            other if other.starts_with("carry") && other.ends_with("_le") => { let i=other[5..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"sle") }
                            other if other.starts_with("carry") && other.ends_with("_gt") => { let i=other[5..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"sgt") }
                            other if other.starts_with("carry") && other.ends_with("_ge") => { let i=other[5..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"sge") }
                            other => return Err(format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name)),
                        };
                        let c = fresh_reg(&mut next_reg);
                        body.push(format!("  {c} = icmp {pred} i64 {lhs}, {rhs}"));
                        let s = fresh_reg(&mut next_reg);
                        body.push(format!(
                            "  {s} = select i1 {c}, i64 {then_value}, i64 {else_value}"
                        ));
                        s
                    };
                    next_carries.push(next_carry);
                }
                body.push(format!("  store i64 {next_current}, ptr {current_slot}"));
                for (slot, val) in carry_slots.iter().zip(next_carries.iter()) {
                    body.push(format!("  store i64 {val}, ptr {slot}"));
                }
                body.push(format!("  br label %{loop_cond}"));
                body.push(format!("{loop_exit}:"));
                let final_current = fresh_reg(&mut next_reg);
                body.push(format!("  {final_current} = load i64, ptr {current_slot}"));
                let final_carries = carry_slots
                    .iter()
                    .map(|slot| {
                        let r = fresh_reg(&mut next_reg);
                        body.push(format!("  {r} = load i64, ptr {slot}"));
                        r
                    })
                    .collect::<Vec<_>>();
                let mut fields = vec![(
                    "current".to_owned(),
                    LlvmValueRef::I64(final_current.clone()),
                )];
                for (index, fc) in final_carries.iter().enumerate() {
                    fields.push((format!("carry{index}"), LlvmValueRef::I64(fc.clone())));
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
