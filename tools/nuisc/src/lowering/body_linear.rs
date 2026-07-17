use super::*;

use crate::lowering::if_lowering::stmts_contain_conditional_effect_primitive;

pub(in crate::lowering) fn lower_linear_stmts(
    stmts: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &mut BTreeMap<String, String>,
) -> Result<Option<String>, String> {
    let mut last_bound_name = None;
    for stmt in stmts {
        match stmt {
            NirStmt::Let { name, value, .. } => {
                let lowered = lower_expr(value, state, bindings)?;
                chain_nonpure_expr_stmt(value, &lowered, state);
                bindings.insert(name.clone(), lowered);
                last_bound_name = Some(name.clone());
            }
            NirStmt::Const { name, value, .. } => {
                let lowered = lower_expr(value, state, bindings)?;
                chain_nonpure_expr_stmt(value, &lowered, state);
                bindings.insert(name.clone(), lowered);
                last_bound_name = Some(name.clone());
            }
            NirStmt::Expr(expr) => {
                let lowered = lower_expr(expr, state, bindings)?;
                chain_nonpure_expr_stmt(expr, &lowered, state);
                last_bound_name = None;
            }
            _ => {
                return Err(
                    "minimal nuisc lowering currently only supports shared branch context made of straight-line `let`, `const`, or expression statements"
                        .to_owned(),
                )
            }
        }
    }
    Ok(last_bound_name)
}

pub(in crate::lowering) fn lower_inline_stmts(
    stmts: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &mut BTreeMap<String, String>,
    const_bindings: &mut BTreeMap<String, NirExpr>,
) -> Result<Option<String>, String> {
    let mut index = 0;
    while index < stmts.len() {
        let stmt = &stmts[index];
        match stmt {
            NirStmt::Let { name, value, .. } => {
                let lowered = lower_expr(value, state, bindings)?;
                chain_nonpure_expr_stmt(value, &lowered, state);
                bindings.insert(name.clone(), lowered);
                refresh_const_binding(const_bindings, name, value);
            }
            NirStmt::Const { name, value, .. } => {
                let lowered = lower_expr(value, state, bindings)?;
                chain_nonpure_expr_stmt(value, &lowered, state);
                bindings.insert(name.clone(), lowered);
                refresh_const_binding(const_bindings, name, value);
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
                    to: print_name.clone(),
                });
                chain_statement_effect(state, &print_name);
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
                    to: await_name.clone(),
                });
                chain_statement_effect(state, &await_name);
            }
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                if let Some(returned) = lower_early_return_continuation_if(
                    condition,
                    then_body,
                    else_body,
                    &stmts[index + 1..],
                    state,
                    bindings,
                    const_bindings,
                )? {
                    return Ok(Some(returned));
                }
                if let Some(returned) = lower_if_stmt(
                    condition,
                    then_body,
                    else_body,
                    state,
                    bindings,
                    const_bindings,
                )? {
                    return Ok(Some(returned));
                }
            }
            NirStmt::While { condition, body } => {
                if let Some(returned) =
                    lower_while_stmt(condition, body, state, bindings, const_bindings)?
                {
                    return Ok(Some(returned));
                }
            }
            NirStmt::Break => return Err(unsupported_loop_control_stmt_message("break")),
            NirStmt::Continue => return Err(unsupported_loop_control_stmt_message("continue")),
            NirStmt::Expr(expr) => {
                let lowered = lower_expr(expr, state, bindings)?;
                chain_nonpure_expr_stmt(expr, &lowered, state);
            }
            NirStmt::Return(value) => {
                let returned = match value {
                    Some(value) => Some(lower_expr(value, state, bindings)?),
                    None => None,
                };
                return Ok(returned);
            }
        }
        index += 1;
    }

    Ok(None)
}

fn lower_early_return_continuation_if(
    condition: &NirExpr,
    then_body: &[NirStmt],
    else_body: &[NirStmt],
    tail: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
    const_bindings: &BTreeMap<String, NirExpr>,
) -> Result<Option<String>, String> {
    if tail.is_empty()
        || stmts_contain_conditional_effect_primitive(else_body)
        || stmts_contain_conditional_effect_primitive(tail)
    {
        return Ok(None);
    }
    let Some(PreparedTerminalBranch::Return(early_return)) =
        prepare_terminal_branch(then_body, &state.pure_helpers)
    else {
        return Ok(None);
    };
    if else_body.is_empty()
        || !else_body
            .iter()
            .any(|stmt| matches!(stmt, NirStmt::Let { .. } | NirStmt::Const { .. }))
        || !else_body.iter().all(|stmt| {
            matches!(
                stmt,
                NirStmt::Let { .. } | NirStmt::Const { .. } | NirStmt::Expr(_)
            )
        })
    {
        return Ok(None);
    }

    let condition_name = lower_expr(condition, state, bindings)?;
    let early_return_name = lower_expr(&early_return, state, bindings)?;
    let mut local_bindings = bindings.clone();
    let Some(_) = lower_linear_stmts(else_body, state, &mut local_bindings)? else {
        return Ok(None);
    };
    let mut local_const_bindings = const_bindings.clone();
    let Some(tail_return_name) =
        lower_inline_stmts(tail, state, &mut local_bindings, &mut local_const_bindings)?
    else {
        return Ok(None);
    };
    Ok(Some(lower_select(
        condition_name,
        early_return_name,
        tail_return_name,
        state,
    )?))
}
