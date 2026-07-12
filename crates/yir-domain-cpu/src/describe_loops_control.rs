use super::describe_post_control::describe_cpu_post_control_node;
use super::*;

pub(super) fn describe_cpu_loops_control_node(
    node: &Node,
) -> Result<Option<InstructionSemantics>, String> {
    let semantics = match node.op.instruction.as_str() {
        "loop_while_i64" => {
            if node.op.args.len() != 5 {
                return Err(format!(
                    "node `{}` expects `cpu.loop_while_i64 <name> <resource> <initial> <limit> <step> <cmp> <step_kind>`",
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
                "add" | "sub" => {}
                other => {
                    return Err(format!(
                        "node `{}` has invalid loop step kind `{}`",
                        node.name, other
                    ));
                }
            }
            Ok(InstructionSemantics::effect(node.op.args[..3].to_vec()))
        }
        "loop_while_i64_chain" | "loop_while_scalar_chain" => {
            if node.op.args.len() < 7 {
                return Err(format!(
                    "node `{}` expects `cpu.loop_while_scalar_chain <name> <resource> <initial> <limit> <step> <cmp> <step_kind> (<carry_initial> <carry_kind>)+`",
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
                "add" | "sub" => {}
                other => {
                    return Err(format!(
                        "node `{}` has invalid loop step kind `{}`",
                        node.name, other
                    ));
                }
            }
            let mut effect_args = node.op.args[..3].to_vec();
            let mut cursor = 5usize;
            let mut parsed_any_carry = false;
            while cursor < node.op.args.len() {
                let Some(carry_initial) = node.op.args.get(cursor) else {
                    break;
                };
                let Some(carry_kind) = node.op.args.get(cursor + 1) else {
                    return Err(format!(
                        "node `{}` expects `cpu.loop_while_scalar_chain <name> <resource> <initial> <limit> <step> <cmp> <step_kind> (<carry_initial> <carry_kind>)+`",
                        node.name
                    ));
                };
                let Some(payload_len) = carry_source_payload_len(carry_kind) else {
                    return Err(format!(
                        "node `{}` has invalid carry kind `{}`",
                        node.name, carry_kind
                    ));
                };
                let payload_end = cursor + 2 + payload_len;
                if payload_end > node.op.args.len() {
                    return Err(format!(
                        "node `{}` is missing carry payload for `{}`",
                        node.name, carry_kind
                    ));
                }
                effect_args.push(carry_initial.clone());
                effect_args.extend(node.op.args[cursor + 2..payload_end].iter().cloned());
                cursor = payload_end;
                parsed_any_carry = true;
            }
            if !parsed_any_carry || cursor != node.op.args.len() {
                return Err(format!(
                    "node `{}` expects `cpu.loop_while_scalar_chain <name> <resource> <initial> <limit> <step> <cmp> <step_kind> (<carry_initial> <carry_kind>)+`",
                    node.name
                ));
            }
            Ok(InstructionSemantics::effect(effect_args))
        }
        "loop_while_i64_async_chain" | "loop_while_scalar_async_chain" => {
            if node.op.args.len() < 6 {
                return Err(format!(
                    "node `{}` expects `cpu.loop_while_scalar_async_chain <name> <resource> <initial> <limit> <step_callee> <cmp> (<carry_initial> <carry_kind>)+`",
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
            let mut effect_args = node.op.args[..2].to_vec();
            let mut cursor = 4usize;
            let mut parsed_any_carry = false;
            while cursor < node.op.args.len() {
                let Some(carry_initial) = node.op.args.get(cursor) else {
                    break;
                };
                let Some(carry_kind) = node.op.args.get(cursor + 1) else {
                    return Err(format!(
                        "node `{}` expects `cpu.loop_while_scalar_async_chain <name> <resource> <initial> <limit> <step_callee> <cmp> (<carry_initial> <carry_kind>)+`",
                        node.name
                    ));
                };
                let Some(payload_len) = carry_source_payload_len(carry_kind) else {
                    return Err(format!(
                        "node `{}` has invalid carry kind `{}`",
                        node.name, carry_kind
                    ));
                };
                let payload_end = cursor + 2 + payload_len;
                if payload_end > node.op.args.len() {
                    return Err(format!(
                        "node `{}` is missing carry payload for `{}`",
                        node.name, carry_kind
                    ));
                }
                effect_args.push(carry_initial.clone());
                effect_args.extend(node.op.args[cursor + 2..payload_end].iter().cloned());
                cursor = payload_end;
                parsed_any_carry = true;
            }
            if !parsed_any_carry || cursor != node.op.args.len() {
                return Err(format!(
                    "node `{}` expects `cpu.loop_while_scalar_async_chain <name> <resource> <initial> <limit> <step_callee> <cmp> (<carry_initial> <carry_kind>)+`",
                    node.name
                ));
            }
            Ok(InstructionSemantics::effect(effect_args))
        }
        "loop_while_i64_async_cond_chain" | "loop_while_scalar_async_cond_chain" => {
            if node.op.args.len() < 6 {
                return Err(format!(
                    "node `{}` expects `cpu.loop_while_scalar_async_cond_chain <name> <resource> <initial> <limit> <step_callee> <cmp> (<carry_initial> <condition_kind> <condition_rhs> <then_kind> <else_kind>)+`",
                    node.name
                ));
            }
            validate_loop_compare_kind(&node.op.args[3], &node.name)?;
            let carries = parse_conditional_carries(&node.op.args, 4, &node.name, true)?;
            let mut inputs = vec![node.op.args[0].clone(), node.op.args[1].clone()];
            for carry in &carries {
                inputs.push(carry.initial.clone());
                collect_loop_condition_rhs_inputs(&carry.condition, &mut inputs);
                collect_carry_branch_source_inputs(&carry.then_source, &mut inputs);
                collect_carry_branch_source_inputs(&carry.else_source, &mut inputs);
            }
            Ok(InstructionSemantics::effect(inputs))
        }
        "loop_while_i64_cond_chain" | "loop_while_scalar_cond_chain" => {
            if node.op.args.len() < 7 {
                return Err(format!(
                    "node `{}` expects `cpu.loop_while_scalar_cond_chain <name> <resource> <initial> <limit> <step> <cmp> <step_kind> (<carry_initial> <cond_kind> <cond_rhs> <then_kind> <else_kind>)+`",
                    node.name
                ));
            }
            validate_loop_compare_kind(&node.op.args[3], &node.name)?;
            validate_loop_step_kind(&node.op.args[4], &node.name)?;
            let carries = parse_conditional_carries(&node.op.args, 5, &node.name, true)?;
            let mut inputs = node.op.args[..3].to_vec();
            for carry in &carries {
                inputs.push(carry.initial.clone());
                collect_loop_condition_rhs_inputs(&carry.condition, &mut inputs);
                collect_carry_branch_source_inputs(&carry.then_source, &mut inputs);
                collect_carry_branch_source_inputs(&carry.else_source, &mut inputs);
            }
            Ok(InstructionSemantics::effect(inputs))
        }
        "loop_while_i64_flow_chain" | "loop_while_scalar_flow_chain" => {
            if node.op.args.len() < 8 || !(node.op.args.len() - 8).is_multiple_of(2) {
                return Err(format!(
                    "node `{}` expects `cpu.loop_while_scalar_flow_chain <name> <resource> <initial> <limit> <step> <cmp> <step_kind> <control_kind> <control_rhs> <control_action> (<carry_initial> <carry_kind>)*`",
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
        "loop_while_i64_async_flow_chain" | "loop_while_scalar_async_flow_chain" => {
            if node.op.args.len() < 7 || !(node.op.args.len() - 7).is_multiple_of(2) {
                return Err(format!(
                    "node `{}` expects `cpu.loop_while_scalar_async_flow_chain <name> <resource> <initial> <limit> <step_callee> <cmp> <control_kind> <control_rhs> <control_action> (<carry_initial> <carry_kind>)*`",
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
            let mut inputs = vec![
                node.op.args[0].clone(),
                node.op.args[1].clone(),
                node.op.args[5].clone(),
            ];
            for chunk in node.op.args[7..].chunks(2) {
                inputs.push(chunk[0].clone());
            }
            Ok(InstructionSemantics::effect(inputs))
        }
        "loop_while_i64_async_flow_cond_chain" | "loop_while_scalar_async_flow_cond_chain" => {
            validate_loop_compare_kind(&node.op.args[3], &node.name)?;
            let (control_expr, carry_start_index) =
                parse_loop_flow_expr(&node.op.args, 4, &node.name, &validate_flow_control_kind)?;
            if carry_start_index < node.op.args.len()
                && !(node.op.args.len() - carry_start_index).is_multiple_of(5)
            {
                return Err(format!(
                    "node `{}` expects `cpu.loop_while_scalar_async_flow_cond_chain <name> <resource> <initial> <limit> <step_callee> <cmp> <control_flow_expr> (<carry_initial> <cond_kind> <cond_rhs> <then_kind> <else_kind>)*`",
                    node.name
                ));
            }
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
        _ => return describe_cpu_post_control_node(node),
    };
    semantics.map(Some)
}
