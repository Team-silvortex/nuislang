use super::*;

pub(in crate::lowering) fn lower_call_expr(
    callee: &str,
    args: &[NirExpr],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    if callee == "print" {
        return Err("`print(...)` is only valid as a statement".to_owned());
    }
    if callee == "__nuis_require_non_null_buffer" {
        return Err(
            "require_non_null(...) is valid only in a selected owned-helper leaf dominated by the matching non-null branch"
                .to_owned(),
        );
    }

    let function = state
        .function_map
        .get(callee)
        .copied()
        .ok_or_else(|| format!("unknown function `{callee}`"))?;

    for (param, arg) in function.params.iter().zip(args) {
        if param.ty.is_ref
            && !param.ty.is_optional
            && param.ty.name == "Node"
            && !matches!(arg, NirExpr::Borrow(_))
        {
            return Err(format!(
                "traversal pointer argument `{}` for `{callee}` requires explicit borrow(...) capability",
                param.name
            ));
        }
    }

    if state.direct_call_functions.contains(callee) {
        let lowered_args = args
            .iter()
            .map(|arg| lower_expr(arg, state, bindings))
            .collect::<Result<Vec<_>, _>>()?;
        return push_direct_call_node(function, &lowered_args, state);
    }

    if state.call_stack.iter().any(|active| active == callee) {
        return Err(format!(
            "recursive function call `{callee}` is not yet supported by minimal nuisc lowering"
        ));
    }

    if function.params.len() != args.len() {
        return Err(format!(
            "function `{callee}` expects {} args, found {}",
            function.params.len(),
            args.len()
        ));
    }

    let mut local_bindings = BTreeMap::new();
    for (param, arg) in function.params.iter().zip(args.iter()) {
        let lowered = lower_expr(arg, state, bindings)?;
        local_bindings.insert(param.name.clone(), lowered);
    }

    let caller_effect_anchor = state.last_effect_anchor.take();
    state.call_stack.push(callee.to_owned());
    let returned = lower_function_body(function, state, &mut local_bindings, false)?;
    state.call_stack.pop();
    let callee_effect_anchor = state.last_effect_anchor.take();
    state.last_effect_anchor = callee_effect_anchor.or(caller_effect_anchor);

    returned.ok_or_else(|| format!("function `{callee}` did not return a value"))
}

pub(in crate::lowering) fn lower_async_call_boundary(
    callee: &str,
    args: &[NirExpr],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let function = state
        .function_map
        .get(callee)
        .copied()
        .ok_or_else(|| format!("unknown function `{callee}`"))?;
    if !function.is_async {
        return lower_call_expr(callee, args, state, bindings);
    }
    if state.call_stack.iter().any(|active| active == callee) {
        return Err(format!(
            "recursive async function call `{callee}` is not yet supported by minimal nuisc lowering"
        ));
    }
    if function.params.len() != args.len() {
        return Err(format!(
            "function `{callee}` expects {} args, found {}",
            function.params.len(),
            args.len()
        ));
    }

    let lowered_args = args
        .iter()
        .map(|arg| lower_expr(arg, state, bindings))
        .collect::<Result<Vec<_>, _>>()?;

    if state.async_helper_functions.contains(callee) {
        let call_name = next_name(state, "async_call");
        let call_index = state.yir.nodes.len();
        let mut op_args = vec![callee.to_owned()];
        op_args.extend(lowered_args.clone());
        state.yir.nodes.push(Node {
            name: call_name.clone(),
            resource: "cpu0".to_owned(),
            op: Operation {
                module: "cpu".to_owned(),
                instruction: "async_call".to_owned(),
                args: op_args,
            },
        });
        for arg in &lowered_args {
            push_dep_edges(state, arg, &call_name);
        }

        let returned = push_direct_call_node(function, &lowered_args, state)?;
        let returned = wrap_async_value_if_needed(state, call_index, &returned);
        state.yir.edges.push(Edge {
            kind: EdgeKind::Effect,
            from: call_name.clone(),
            to: returned.clone(),
        });
        return Ok(returned);
    }

    let mut local_bindings = BTreeMap::new();
    for (param, lowered) in function.params.iter().zip(lowered_args.iter()) {
        local_bindings.insert(param.name.clone(), lowered.clone());
    }

    let call_name = next_name(state, "async_call");
    let call_index = state.yir.nodes.len();
    let mut op_args = vec![callee.to_owned()];
    op_args.extend(lowered_args.clone());
    state.yir.nodes.push(Node {
        name: call_name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "async_call".to_owned(),
            args: op_args,
        },
    });
    for arg in &lowered_args {
        push_dep_edges(state, arg, &call_name);
    }

    state.call_stack.push(callee.to_owned());
    let returned = lower_function_body(function, state, &mut local_bindings, false)?;
    state.call_stack.pop();
    let returned = returned.ok_or_else(|| format!("function `{callee}` did not return a value"))?;
    let returned = wrap_async_value_if_needed(state, call_index, &returned);
    state.yir.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: call_name.clone(),
        to: returned.clone(),
    });
    Ok(returned)
}

fn wrap_async_value_if_needed(
    state: &mut LoweringState<'_>,
    call_index: usize,
    returned: &str,
) -> String {
    let returned_index = state
        .yir
        .nodes
        .iter()
        .position(|node| node.name == returned)
        .unwrap_or(usize::MAX);
    if returned_index > call_index {
        return returned.to_owned();
    }

    let name = next_name(state, "async_value");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "async_value".to_owned(),
            args: vec![returned.to_owned()],
        },
    });
    push_dep_edges(state, returned, &name);
    name
}

pub(in crate::lowering) fn lower_unary_cpu_expr(
    instruction: &str,
    value: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let lowered = lower_expr(value, state, bindings)?;
    let name = next_name(state, instruction);
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: instruction.to_owned(),
            args: vec![lowered.clone()],
        },
    });
    push_dep_edges(state, &lowered, &name);
    Ok(name)
}
