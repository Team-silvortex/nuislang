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
        "loop_while_i64_effect" => {
            if node.op.args.len() < 9 {
                return Err(format!(
                    "node `{}` expects `cpu.loop_while_i64_effect <initial> <limit> <step> <cmp> <step-kind> <action-module> <action-instruction> <arity> <action-operands...>`",
                    node.name
                ));
            }
            validate_loop_compare_kind(&node.op.args[3], &node.name)?;
            validate_loop_step_kind(&node.op.args[4], &node.name)?;
            let arity = node.op.args[7].parse::<usize>().map_err(|_| {
                format!(
                    "node `{}` has invalid loop action arity `{}`",
                    node.name, node.op.args[7]
                )
            })?;
            if node.op.args.len() != 8 + arity {
                return Err(format!(
                    "node `{}` declares {arity} loop action operands but provides {}",
                    node.name,
                    node.op.args.len() - 8
                ));
            }
            let action_inputs = match (node.op.args[5].as_str(), node.op.args[6].as_str(), arity) {
                ("cpu", "owned_bytes_copy_drop", 1) => node.op.args[8..].to_vec(),
                ("cpu", "scoped_call", arity) if arity >= 1 => node.op.args[9..]
                    .iter()
                    .filter(|arg| arg.as_str() != "$current")
                    .cloned()
                    .collect(),
                (module, instruction, _) => {
                    return Err(format!(
                        "node `{}` references unregistered loop action `{module}.{instruction}`",
                        node.name
                    ));
                }
            };
            let mut effect_args = node.op.args[..3].to_vec();
            effect_args.extend(action_inputs);
            Ok(InstructionSemantics::effect(effect_args))
        }
        "loop_while_i64_effect_flow" => {
            if node.op.args.len() < 14 {
                return Err(format!(
                    "node `{}` expects `cpu.loop_while_i64_effect_flow <initial> <limit> <step> <cmp> <step-kind> <control-token-count> <control-tokens...> <carry-count> (<carry-initial> <carry-kind>)* <action-module> <action-instruction> <arity> <action-operands...>`",
                    node.name
                ));
            }
            validate_loop_compare_kind(&node.op.args[3], &node.name)?;
            validate_loop_step_kind(&node.op.args[4], &node.name)?;
            let control_count = node.op.args[5].parse::<usize>().map_err(|_| {
                format!(
                    "node `{}` has invalid effect-flow control token count `{}`",
                    node.name, node.op.args[5]
                )
            })?;
            let control_end = 6 + control_count;
            if control_count < 3 || control_end >= node.op.args.len() {
                return Err(format!(
                    "node `{}` has inconsistent effect-flow control payload length",
                    node.name
                ));
            }
            let (control_expr, after_control) =
                parse_loop_flow_expr(&node.op.args, 6, &node.name, &validate_flow_control_kind)?;
            if after_control != control_end {
                return Err(format!(
                    "node `{}` has invalid effect-flow control action payload",
                    node.name
                ));
            }
            let carry_count = node.op.args[control_end].parse::<usize>().map_err(|_| {
                format!(
                    "node `{}` has invalid effect-flow carry count `{}`",
                    node.name, node.op.args[control_end]
                )
            })?;
            let carry_offset = control_end + 1;
            let mut carry_cursor = carry_offset;
            let mut effect_carry_args = Vec::new();
            for index in 0..carry_count {
                let Some(initial) = node.op.args.get(carry_cursor) else {
                    return Err(format!(
                        "node `{}` is missing effect-flow carry {index} initial",
                        node.name
                    ));
                };
                let Some(kind) = node.op.args.get(carry_cursor + 1) else {
                    return Err(format!(
                        "node `{}` is missing effect-flow carry {index} kind",
                        node.name
                    ));
                };
                let Some(payload_len) = carry_source_payload_len(kind) else {
                    return Err(format!(
                        "node `{}` has invalid effect-flow carry kind `{}`",
                        node.name, kind
                    ));
                };
                let payload_end = carry_cursor + 2 + payload_len;
                if payload_end > node.op.args.len() {
                    return Err(format!(
                        "node `{}` is missing effect-flow carry payload for `{}`",
                        node.name, kind
                    ));
                }
                let supported = (kind == "add_current" && payload_len == 0)
                    || (matches!(kind.as_str(), "add_invariant" | "mul_invariant")
                        && payload_len == 1)
                    || (matches!(
                        kind.as_str(),
                        "add_current_plus_invariant" | "mul_current_plus_invariant"
                    ) && payload_len == 1)
                    || ((kind.starts_with("mul_scaled_") || kind.starts_with("add_scaled_"))
                        && effect_flow_kind_has_only_available_new_carries(kind, index))
                    || effect_flow_state_list_kind_is_supported(
                        kind,
                        payload_len,
                        index,
                        carry_count,
                    )
                    || (payload_len == 0
                        && kind
                            .strip_prefix("add_carry")
                            .and_then(|source| source.parse::<usize>().ok())
                            .is_some_and(|source| source < index));
                if !supported {
                    return Err(format!(
                        "node `{}` has unsupported effect-flow carry kind `{}`",
                        node.name, kind
                    ));
                }
                effect_carry_args.push(initial.clone());
                effect_carry_args
                    .extend(node.op.args[carry_cursor + 2..payload_end].iter().cloned());
                carry_cursor = payload_end;
            }
            let action_offset = carry_cursor;
            if node.op.args.len() != action_offset + 4 {
                return Err(format!(
                    "node `{}` has inconsistent effect-flow carry/action payload length",
                    node.name
                ));
            }
            match (
                node.op.args[action_offset].as_str(),
                node.op.args[action_offset + 1].as_str(),
                node.op.args[action_offset + 2].as_str(),
            ) {
                ("cpu", "owned_bytes_copy_drop", "1") => {}
                _ => {
                    return Err(format!(
                        "node `{}` references an unregistered effect-flow resource action",
                        node.name
                    ));
                }
            }
            let mut effect_args = node.op.args[..3].to_vec();
            collect_loop_flow_rhs_inputs(&control_expr, &mut effect_args);
            effect_args.extend(effect_carry_args);
            effect_args.push(node.op.args[action_offset + 3].clone());
            Ok(InstructionSemantics::effect(effect_args))
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

fn effect_flow_kind_has_only_available_new_carries(kind: &str, carry_index: usize) -> bool {
    kind.match_indices("carry").all(|(offset, _)| {
        if kind[..offset].ends_with("prev_") {
            return true;
        }
        let digits = kind[offset + 5..]
            .chars()
            .take_while(char::is_ascii_digit)
            .collect::<String>();
        !digits.is_empty()
            && digits
                .parse::<usize>()
                .is_ok_and(|source| source < carry_index)
    })
}

fn effect_flow_state_list_kind_is_supported(
    kind: &str,
    payload_len: usize,
    carry_index: usize,
    carry_count: usize,
) -> bool {
    let Some(terms) = kind
        .strip_prefix("add_")
        .or_else(|| kind.strip_prefix("mul_"))
    else {
        return false;
    };
    let terms = terms
        .strip_suffix("_plus_invariant")
        .unwrap_or(terms)
        .split("_plus_")
        .collect::<Vec<_>>();
    terms.len() >= 2
        && terms.iter().all(|term| match *term {
            "current" | "prev_current" => true,
            other if other.starts_with("prev_carry") => other[10..]
                .parse::<usize>()
                .is_ok_and(|source| source < carry_count),
            other if other.starts_with("carry") => other[5..]
                .parse::<usize>()
                .is_ok_and(|source| source < carry_index),
            _ => false,
        })
        && payload_len == usize::from(kind.ends_with("_plus_invariant"))
}
