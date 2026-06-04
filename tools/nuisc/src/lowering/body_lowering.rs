use super::*;

pub(super) fn lower_function_body(
    function: &NirFunction,
    state: &mut LoweringState<'_>,
    bindings: &mut BTreeMap<String, String>,
    allow_implicit_return: bool,
) -> Result<Option<String>, String> {
    for stmt in &function.body {
        match stmt {
            NirStmt::Let { name, value, .. } => {
                let lowered = lower_expr(value, state, bindings)?;
                bindings.insert(name.clone(), lowered);
            }
            NirStmt::Const { name, value, .. } => {
                let lowered = lower_expr(value, state, bindings)?;
                bindings.insert(name.clone(), lowered);
            }
            NirStmt::Print(value) => {
                let lowered = lower_expr(value, state, bindings)?;
                let print_name = format!("print_{}", state.print_counter);
                state.print_counter += 1;
                state.yir.nodes.push(Node {
                    name: print_name.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "print".to_owned(),
                        args: vec![lowered.clone()],
                    },
                });
                push_dep_edges(state, &lowered, &print_name);
                state.yir.edges.push(Edge {
                    kind: EdgeKind::Effect,
                    from: lowered,
                    to: print_name,
                });
            }
            NirStmt::Await(value) => {
                let awaited = match value {
                    NirExpr::Call { callee, args } => {
                        lower_async_call_boundary(callee, args, state, bindings)?
                    }
                    _ => lower_expr(value, state, bindings)?,
                };
                let await_name = push_await_node(state, &awaited);
                state.yir.edges.push(Edge {
                    kind: EdgeKind::Effect,
                    from: awaited,
                    to: await_name,
                });
            }
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                if let Some(returned) =
                    lower_if_stmt(condition, then_body, else_body, state, bindings)?
                {
                    return Ok(Some(returned));
                }
            }
            NirStmt::While { condition, body } => {
                if let Some(returned) = lower_while_stmt(condition, body, state, bindings)? {
                    return Ok(Some(returned));
                }
            }
            NirStmt::Break => {
                return Err(
                    "`break` parsed successfully, but loop execution lowering is not implemented yet"
                        .to_owned(),
                );
            }
            NirStmt::Continue => {
                return Err(
                    "`continue` parsed successfully, but loop execution lowering is not implemented yet"
                        .to_owned(),
                );
            }
            NirStmt::Expr(expr) => {
                let _ = lower_expr(expr, state, bindings)?;
            }
            NirStmt::Return(value) => {
                return match value {
                    Some(value) => Ok(Some(lower_expr(value, state, bindings)?)),
                    None => Ok(None),
                };
            }
        }
    }

    if allow_implicit_return {
        Ok(None)
    } else {
        Err(format!(
            "function `{}` ended without `return` in expression-call lowering",
            function.name
        ))
    }
}

pub(super) fn lower_if_stmt(
    condition: &NirExpr,
    then_body: &[NirStmt],
    else_body: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &mut BTreeMap<String, String>,
) -> Result<Option<String>, String> {
    let condition_name = lower_expr(condition, state, bindings)?;
    let lowered = lower_if_pair(condition_name, then_body, else_body, state, bindings)?;
    match lowered {
        LoweredIfOutcome::Continued => Ok(None),
        LoweredIfOutcome::Bind { name, value } => {
            bindings.insert(name, value);
            Ok(None)
        }
        LoweredIfOutcome::Printed => Ok(None),
        LoweredIfOutcome::Returned(value) => Ok(Some(value)),
    }
}

pub(super) fn lower_while_stmt(
    condition: &NirExpr,
    body: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &mut BTreeMap<String, String>,
) -> Result<Option<String>, String> {
    if let Some(prepared) = prepare_post_flow_while(condition, body, &state.pure_helpers) {
        lower_post_flow_while(prepared, state, bindings)?;
        return Ok(None);
    }

    if let Some(prepared) = prepare_flow_while(condition, body, &state.pure_helpers) {
        lower_flow_while(prepared, state, bindings)?;
        return Ok(None);
    }

    if let Some(prepared) = prepare_chained_while(condition, body, &state.pure_helpers) {
        lower_chained_while(prepared, state, bindings)?;
        return Ok(None);
    }

    if let Some(prepared) = prepare_counted_while(condition, body, &state.pure_helpers) {
        lower_counted_while(prepared, state, bindings)?;
        return Ok(None);
    }

    let condition_name = lower_expr(condition, state, bindings)?;
    if let Some(prepared) = prepare_guarded_loop_body(body, &state.pure_helpers) {
        return lower_prepared_loop_body(condition_name, &prepared, state, bindings);
    }

    match lower_if_stmt(condition, body, &[], state, bindings) {
        Ok(Some(returned)) => return Ok(Some(returned)),
        Ok(None) => return Ok(None),
        Err(error) if error.contains("minimal nuisc lowering currently only supports `if`") => {}
        Err(error) => return Err(error),
    }

    Err(
        "minimal nuisc lowering can currently execute only guard-style `while` loops or simple counted `while` loops like `let i = 0; while i < limit { let i = i + 1; }`; general iterative loop/backedge lowering is still not implemented"
            .to_owned(),
    )
}

pub(super) fn lower_call_expr(
    callee: &str,
    args: &[NirExpr],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    if callee == "print" {
        return Err("`print(...)` is only valid as a statement".to_owned());
    }

    let function = state
        .function_map
        .get(callee)
        .copied()
        .ok_or_else(|| format!("unknown function `{callee}`"))?;

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

    state.call_stack.push(callee.to_owned());
    let returned = lower_function_body(function, state, &mut local_bindings, false)?;
    state.call_stack.pop();

    returned.ok_or_else(|| format!("function `{callee}` did not return a value"))
}

pub(super) fn lower_async_call_boundary(
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

    let mut local_bindings = BTreeMap::new();
    let mut lowered_args = Vec::new();
    for (param, arg) in function.params.iter().zip(args.iter()) {
        let lowered = lower_expr(arg, state, bindings)?;
        lowered_args.push(lowered.clone());
        local_bindings.insert(param.name.clone(), lowered);
    }

    let call_name = next_name(state, "async_call");
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
    state.yir.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: call_name,
        to: returned.clone(),
    });
    Ok(returned)
}

pub(super) fn lower_unary_cpu_expr(
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
