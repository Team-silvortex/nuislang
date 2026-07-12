use super::*;

pub(super) fn describe_cpu_post_control_node(
    node: &Node,
) -> Result<Option<InstructionSemantics>, String> {
    let semantics = match node.op.instruction.as_str() {
        "loop_while_i64_post_flow_chain" | "loop_while_scalar_post_flow_chain" => {
            if node.op.args.len() < 10 || !(node.op.args.len() - 8).is_multiple_of(2) {
                return Err(format!(
                    "node `{}` expects `cpu.loop_while_scalar_post_flow_chain <name> <resource> <initial> <limit> <step> <cmp> <step_kind> <control_kind> <control_rhs> <control_action> (<carry_initial> <carry_kind>)+`",
                    node.name
                ));
            }
            match node.op.args[3].as_str() {
                "eq" | "lt" | "le" | "gt" | "ge" => {}
                other => {
                    return Err(format!(
                        "node `{}` has invalid loop compare kind `{}`",
                        node.name, other
                    ));
                }
            }
            match node.op.args[4].as_str() {
                "add" | "sub" => {}
                other => {
                    return Err(format!(
                        "node `{}` has invalid loop step kind `{}`",
                        node.name, other
                    ));
                }
            }
            match node.op.args[5].as_str() {
                "current_eq" | "current_ne" | "current_lt" | "current_le" | "current_gt"
                | "current_ge" => {}
                other if other.starts_with("carry") && other.ends_with("_eq") => {
                    other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                        format!(
                            "node `{}` has invalid flow control kind `{}`",
                            node.name, other
                        )
                    })?;
                }
                other if other.starts_with("carry") && other.ends_with("_ne") => {
                    other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                        format!(
                            "node `{}` has invalid flow control kind `{}`",
                            node.name, other
                        )
                    })?;
                }
                other if other.starts_with("carry") && other.ends_with("_lt") => {
                    other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                        format!(
                            "node `{}` has invalid flow control kind `{}`",
                            node.name, other
                        )
                    })?;
                }
                other if other.starts_with("carry") && other.ends_with("_le") => {
                    other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                        format!(
                            "node `{}` has invalid flow control kind `{}`",
                            node.name, other
                        )
                    })?;
                }
                other if other.starts_with("carry") && other.ends_with("_gt") => {
                    other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                        format!(
                            "node `{}` has invalid flow control kind `{}`",
                            node.name, other
                        )
                    })?;
                }
                other if other.starts_with("carry") && other.ends_with("_ge") => {
                    other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                        format!(
                            "node `{}` has invalid flow control kind `{}`",
                            node.name, other
                        )
                    })?;
                }
                other => {
                    return Err(format!(
                        "node `{}` has invalid flow control kind `{}`",
                        node.name, other
                    ));
                }
            }
            match node.op.args[7].as_str() {
                "break" | "continue" => {}
                other => {
                    return Err(format!(
                        "node `{}` has invalid flow control action `{}`",
                        node.name, other
                    ));
                }
            }
            for carry_kind in node.op.args[9..].iter().step_by(2) {
                if carry_kind == "add_current" {
                    continue;
                }
                if let Some(index) = carry_kind.strip_prefix("add_carry") {
                    index.parse::<usize>().map_err(|_| {
                        format!(
                            "node `{}` has invalid carry kind `{}`",
                            node.name, carry_kind
                        )
                    })?;
                    continue;
                }
                return Err(format!(
                    "node `{}` has invalid carry kind `{}`",
                    node.name, carry_kind
                ));
            }
            let mut inputs = node.op.args[..3].to_vec();
            inputs.push(node.op.args[6].clone());
            for chunk in node.op.args[8..].chunks(2) {
                inputs.push(chunk[0].clone());
            }
            Ok(InstructionSemantics::effect(inputs))
        }
        "loop_while_i64_async_post_flow_chain" | "loop_while_scalar_async_post_flow_chain" => {
            if node.op.args.len() < 9 || !(node.op.args.len() - 7).is_multiple_of(2) {
                return Err(format!(
                    "node `{}` expects `cpu.loop_while_scalar_async_post_flow_chain <name> <resource> <initial> <limit> <step_callee> <cmp> <control_kind> <control_rhs> <control_action> (<carry_initial> <carry_kind>)+`",
                    node.name
                ));
            }
            match node.op.args[3].as_str() {
                "eq" | "ne" | "lt" | "le" | "gt" | "ge" => {}
                other => {
                    return Err(format!(
                        "node `{}` has invalid loop compare kind `{}`",
                        node.name, other
                    ));
                }
            }
            match node.op.args[4].as_str() {
                "current_eq" | "current_ne" | "current_lt" | "current_le" | "current_gt"
                | "current_ge" => {}
                other if other.starts_with("carry") && other.ends_with("_eq") => {
                    other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                        format!(
                            "node `{}` has invalid flow control kind `{}`",
                            node.name, other
                        )
                    })?;
                }
                other if other.starts_with("carry") && other.ends_with("_ne") => {
                    other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                        format!(
                            "node `{}` has invalid flow control kind `{}`",
                            node.name, other
                        )
                    })?;
                }
                other if other.starts_with("carry") && other.ends_with("_lt") => {
                    other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                        format!(
                            "node `{}` has invalid flow control kind `{}`",
                            node.name, other
                        )
                    })?;
                }
                other if other.starts_with("carry") && other.ends_with("_le") => {
                    other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                        format!(
                            "node `{}` has invalid flow control kind `{}`",
                            node.name, other
                        )
                    })?;
                }
                other if other.starts_with("carry") && other.ends_with("_gt") => {
                    other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                        format!(
                            "node `{}` has invalid flow control kind `{}`",
                            node.name, other
                        )
                    })?;
                }
                other if other.starts_with("carry") && other.ends_with("_ge") => {
                    other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                        format!(
                            "node `{}` has invalid flow control kind `{}`",
                            node.name, other
                        )
                    })?;
                }
                other => {
                    return Err(format!(
                        "node `{}` has invalid flow control kind `{}`",
                        node.name, other
                    ));
                }
            }
            match node.op.args[6].as_str() {
                "break" | "continue" => {}
                other => {
                    return Err(format!(
                        "node `{}` has invalid flow control action `{}`",
                        node.name, other
                    ));
                }
            }
            for carry_kind in node.op.args[8..].iter().step_by(2) {
                if carry_kind == "add_current" {
                    continue;
                }
                if let Some(index) = carry_kind.strip_prefix("add_carry") {
                    index.parse::<usize>().map_err(|_| {
                        format!(
                            "node `{}` has invalid carry kind `{}`",
                            node.name, carry_kind
                        )
                    })?;
                    continue;
                }
                return Err(format!(
                    "node `{}` has invalid carry kind `{}`",
                    node.name, carry_kind
                ));
            }
            let mut inputs = vec![node.op.args[0].clone(), node.op.args[1].clone()];
            inputs.push(node.op.args[5].clone());
            for chunk in node.op.args[7..].chunks(2) {
                inputs.push(chunk[0].clone());
            }
            Ok(InstructionSemantics::effect(inputs))
        }
        "loop_while_i64_async_post_flow_cond_chain"
        | "loop_while_scalar_async_post_flow_cond_chain" => {
            validate_loop_compare_kind(&node.op.args[3], &node.name)?;
            let (control_expr, carry_start_index) =
                parse_loop_flow_expr(&node.op.args, 4, &node.name, &validate_flow_control_kind)?;
            let carries =
                parse_conditional_carries(&node.op.args, carry_start_index, &node.name, true)?;
            let mut inputs = vec![node.op.args[0].clone(), node.op.args[1].clone()];
            collect_loop_flow_rhs_inputs(&control_expr, &mut inputs);
            for carry in &carries {
                inputs.push(carry.initial.clone());
                collect_loop_condition_rhs_inputs(&carry.condition, &mut inputs);
                collect_carry_branch_source_inputs(&carry.then_source, &mut inputs);
                collect_carry_branch_source_inputs(&carry.else_source, &mut inputs);
            }
            Ok(InstructionSemantics::effect(inputs))
        }
        "loop_while_i64_post_flow_cond_chain" | "loop_while_scalar_post_flow_cond_chain" => {
            validate_loop_compare_kind(&node.op.args[3], &node.name)?;
            validate_loop_step_kind(&node.op.args[4], &node.name)?;
            let (control_expr, carry_start_index) =
                parse_loop_flow_expr(&node.op.args, 5, &node.name, &validate_flow_control_kind)?;
            let carries =
                parse_conditional_carries(&node.op.args, carry_start_index, &node.name, true)?;
            let mut inputs = node.op.args[..3].to_vec();
            collect_loop_flow_rhs_inputs(&control_expr, &mut inputs);
            for carry in &carries {
                inputs.push(carry.initial.clone());
                collect_loop_condition_rhs_inputs(&carry.condition, &mut inputs);
                collect_carry_branch_source_inputs(&carry.then_source, &mut inputs);
                collect_carry_branch_source_inputs(&carry.else_source, &mut inputs);
            }
            Ok(InstructionSemantics::effect(inputs))
        }
        "loop_while_i64_flow_cond_chain" | "loop_while_scalar_flow_cond_chain" => {
            validate_loop_compare_kind(&node.op.args[3], &node.name)?;
            validate_loop_step_kind(&node.op.args[4], &node.name)?;
            let (control_expr, carry_start_index) =
                parse_loop_flow_expr(&node.op.args, 5, &node.name, &validate_flow_control_kind)?;
            let carries =
                parse_conditional_carries(&node.op.args, carry_start_index, &node.name, true)?;
            let mut inputs = node.op.args[..3].to_vec();
            collect_loop_flow_rhs_inputs(&control_expr, &mut inputs);
            for carry in &carries {
                inputs.push(carry.initial.clone());
                collect_loop_condition_rhs_inputs(&carry.condition, &mut inputs);
                collect_carry_branch_source_inputs(&carry.then_source, &mut inputs);
                collect_carry_branch_source_inputs(&carry.else_source, &mut inputs);
            }
            Ok(InstructionSemantics::effect(inputs))
        }
        "guard_return" => {
            if node.op.args.len() != 2 {
                return Err(format!(
                    "node `{}` expects `cpu.guard_return <name> <resource> <condition> <return>`",
                    node.name
                ));
            }

            Ok(InstructionSemantics::effect(node.op.args.clone()))
        }
        "guard_print_return" => {
            if node.op.args.len() != 3 {
                return Err(format!(
                    "node `{}` expects `cpu.guard_print_return <name> <resource> <condition> <print> <return>`",
                    node.name
                ));
            }

            Ok(InstructionSemantics::effect(node.op.args.clone()))
        }
        "guard_host_call_return" => {
            if node.op.args.len() < 4 {
                return Err(
                    "cpu.guard_host_call_return expects condition return call-chain".to_owned(),
                );
            }
            let mut inputs = vec![node.op.args[0].clone()];
            match node.op.args.get(1).map(String::as_str) {
                Some("value") => inputs.push(node.op.args[3].clone()),
                Some("write_flush_exit_code") => {}
                _ if node.op.args[2].parse::<usize>().is_ok() => {
                    inputs.push(node.op.args[1].clone())
                }
                _ => inputs.push(node.op.args[3].clone()),
            }
            Ok(InstructionSemantics::effect(inputs))
        }
        "branch_print_return" => {
            if node.op.args.len() != 5 {
                return Err(format!(
                    "node `{}` expects `cpu.branch_print_return <name> <resource> <condition> <then_print> <then_return> <else_print> <else_return>`",
                    node.name
                ));
            }
            Ok(InstructionSemantics::effect(node.op.args.clone()))
        }
        "branch_host_call_return" => {
            if node.op.args.len() < 9 {
                return Err(
                    "cpu.branch_host_call_return expects condition then-chain else-chain"
                        .to_owned(),
                );
            }
            Ok(InstructionSemantics::effect(vec![node.op.args[0].clone()]))
        }
        _ => return Ok(None),
    };
    semantics.map(Some)
}
