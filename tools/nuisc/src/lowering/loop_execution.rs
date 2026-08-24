use super::*;

pub(super) fn prepare_guarded_loop_body(
    stmts: &[NirStmt],
    pure_helpers: &BTreeSet<String>,
) -> Option<PreparedLoopBody> {
    match stmts {
        [NirStmt::Break] => Some(PreparedLoopBody::Break),
        [NirStmt::Continue] => Some(PreparedLoopBody::Continue),
        [NirStmt::Print(print), NirStmt::Break] => Some(PreparedLoopBody::PrintBreak {
            print: print.clone(),
        }),
        [NirStmt::Print(print), NirStmt::Continue] => Some(PreparedLoopBody::PrintContinue {
            print: print.clone(),
        }),
        [NirStmt::Return(Some(returned))] => Some(PreparedLoopBody::Return {
            returned: returned.clone(),
        }),
        [NirStmt::Print(print), NirStmt::Return(Some(returned))] => {
            Some(PreparedLoopBody::PrintReturn {
                print: print.clone(),
                returned: returned.clone(),
            })
        }
        [NirStmt::If {
            condition,
            then_body,
            else_body,
        }] => {
            let then_prepared = prepare_guarded_loop_body(then_body, pure_helpers)?;
            let else_prepared = prepare_guarded_loop_body(else_body, pure_helpers)?;
            Some(PreparedLoopBody::Branch {
                condition: condition.clone(),
                then_body: Box::new(then_prepared),
                else_body: Box::new(else_prepared),
            })
        }
        [binding @ (NirStmt::Let { .. } | NirStmt::Const { .. }), tail @ ..] => {
            let (name, value) = extract_pure_branch_binding(binding, pure_helpers)?;
            let prepared = prepare_guarded_loop_body(tail, pure_helpers)?;
            Some(substitute_prepared_loop_body(prepared, &name, &value))
        }
        _ => None,
    }
}

pub(super) fn lower_prepared_loop_body(
    condition_name: String,
    body: &PreparedLoopBody,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<Option<String>, String> {
    match body {
        PreparedLoopBody::Break => Ok(None),
        PreparedLoopBody::Continue => {
            lower_guard_loop_continue(condition_name, state);
            Ok(None)
        }
        PreparedLoopBody::PrintBreak { print } => {
            let print_name = lower_expr(print, state, bindings)?;
            lower_guard_print(condition_name, print_name, state);
            Ok(None)
        }
        PreparedLoopBody::PrintContinue { print } => {
            let print_name = lower_expr(print, state, bindings)?;
            lower_guard_loop_print_continue(condition_name, print_name, state);
            Ok(None)
        }
        PreparedLoopBody::Return { returned } => {
            let return_name = lower_expr(returned, state, bindings)?;
            lower_guard_return(condition_name, return_name, state);
            Ok(None)
        }
        PreparedLoopBody::PrintReturn { print, returned } => {
            let print_name = lower_expr(print, state, bindings)?;
            let return_name = lower_expr(returned, state, bindings)?;
            lower_guard_print_return(condition_name, print_name, return_name, state);
            Ok(None)
        }
        PreparedLoopBody::Branch {
            condition,
            then_body,
            else_body,
        } => {
            if let Some(selected) = lower_prepared_loop_return_chain(body, state, bindings)? {
                lower_guard_return(condition_name, selected, state);
                return Ok(None);
            }
            match (then_body.as_ref(), else_body.as_ref()) {
                (PreparedLoopBody::Break, PreparedLoopBody::Break) => Ok(None),
                (PreparedLoopBody::Continue, PreparedLoopBody::Continue) => {
                    lower_guard_loop_continue(condition_name, state);
                    Ok(None)
                }
                (
                    PreparedLoopBody::PrintBreak { print: then_print },
                    PreparedLoopBody::PrintBreak { print: else_print },
                ) => {
                    let branch_condition = lower_expr(condition, state, bindings)?;
                    let then_print_name = lower_expr(then_print, state, bindings)?;
                    let else_print_name = lower_expr(else_print, state, bindings)?;
                    let selected =
                        lower_select(branch_condition, then_print_name, else_print_name, state)?;
                    lower_guard_print(condition_name, selected, state);
                    Ok(None)
                }
                (
                    PreparedLoopBody::PrintContinue { print: then_print },
                    PreparedLoopBody::PrintContinue { print: else_print },
                ) => {
                    let branch_condition = lower_expr(condition, state, bindings)?;
                    let then_print_name = lower_expr(then_print, state, bindings)?;
                    let else_print_name = lower_expr(else_print, state, bindings)?;
                    let selected =
                        lower_select(branch_condition, then_print_name, else_print_name, state)?;
                    lower_guard_loop_print_continue(condition_name, selected, state);
                    Ok(None)
                }
                (
                    PreparedLoopBody::Return {
                        returned: then_return,
                    },
                    PreparedLoopBody::Return {
                        returned: else_return,
                    },
                ) => {
                    let branch_condition = lower_expr(condition, state, bindings)?;
                    let then_return_name = lower_expr(then_return, state, bindings)?;
                    let else_return_name = lower_expr(else_return, state, bindings)?;
                    let selected =
                        lower_select(branch_condition, then_return_name, else_return_name, state)?;
                    lower_guard_return(condition_name, selected, state);
                    Ok(None)
                }
                (
                    PreparedLoopBody::PrintReturn {
                        print: then_print,
                        returned: then_return,
                    },
                    PreparedLoopBody::PrintReturn {
                        print: else_print,
                        returned: else_return,
                    },
                ) => {
                    let branch_condition = lower_expr(condition, state, bindings)?;
                    let then_print_name = lower_expr(then_print, state, bindings)?;
                    let else_print_name = lower_expr(else_print, state, bindings)?;
                    let selected_print = lower_select(
                        branch_condition.clone(),
                        then_print_name,
                        else_print_name,
                        state,
                    )?;
                    let then_return_name = lower_expr(then_return, state, bindings)?;
                    let else_return_name = lower_expr(else_return, state, bindings)?;
                    let selected_return =
                        lower_select(branch_condition, then_return_name, else_return_name, state)?;
                    lower_guard_print_return(
                        condition_name,
                        selected_print,
                        selected_return,
                        state,
                    );
                    Ok(None)
                }
                _ => lower_mixed_prepared_loop_body(condition_name, body, state, bindings),
            }
        }
    }
}

fn lower_mixed_prepared_loop_body(
    condition_name: String,
    body: &PreparedLoopBody,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<Option<String>, String> {
    let condition_path = lower_loop_condition_i64(condition_name, state)?;
    lower_prepared_loop_terminal_paths(body, condition_path, state, bindings)?;
    Ok(None)
}

fn lower_prepared_loop_terminal_paths(
    body: &PreparedLoopBody,
    path_name: String,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<(), String> {
    match body {
        PreparedLoopBody::Break => {}
        PreparedLoopBody::Continue => lower_guard_loop_continue(path_name, state),
        PreparedLoopBody::PrintBreak { print } => {
            let print_name = lower_expr(print, state, bindings)?;
            lower_guard_print(path_name, print_name, state);
        }
        PreparedLoopBody::PrintContinue { print } => {
            let print_name = lower_expr(print, state, bindings)?;
            lower_guard_loop_print_continue(path_name, print_name, state);
        }
        PreparedLoopBody::Return { returned } => {
            let return_name = lower_expr(returned, state, bindings)?;
            lower_guard_return(path_name, return_name, state);
        }
        PreparedLoopBody::PrintReturn { print, returned } => {
            let print_name = lower_expr(print, state, bindings)?;
            let return_name = lower_expr(returned, state, bindings)?;
            lower_guard_print_return(path_name, print_name, return_name, state);
        }
        PreparedLoopBody::Branch {
            condition,
            then_body,
            else_body,
        } => {
            let branch_name = lower_expr(condition, state, bindings)?;
            let branch_path = lower_loop_condition_i64(branch_name, state)?;
            let then_path =
                lower_loop_path_binary("and", path_name.clone(), branch_path.clone(), state);
            let zero_name = lower_expr(&NirExpr::Int(0), state, bindings)?;
            let else_guard = lower_loop_path_binary("eq", branch_path, zero_name, state);
            let else_path = lower_loop_path_binary("and", path_name, else_guard, state);
            lower_prepared_loop_terminal_paths(then_body, then_path, state, bindings)?;
            lower_prepared_loop_terminal_paths(else_body, else_path, state, bindings)?;
        }
    }
    Ok(())
}

fn lower_loop_condition_i64(
    condition_name: String,
    state: &mut LoweringState<'_>,
) -> Result<String, String> {
    let one_name = lower_expr(&NirExpr::Int(1), state, &BTreeMap::new())?;
    let zero_name = lower_expr(&NirExpr::Int(0), state, &BTreeMap::new())?;
    lower_select(condition_name, one_name, zero_name, state)
}

fn lower_loop_path_binary(
    instruction: &str,
    lhs_name: String,
    rhs_name: String,
    state: &mut LoweringState<'_>,
) -> String {
    let name = next_name(state, instruction);
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: instruction.to_owned(),
            args: vec![lhs_name.clone(), rhs_name.clone()],
        },
    });
    push_dep_edges(state, &lhs_name, &name);
    push_dep_edges(state, &rhs_name, &name);
    name
}

fn lower_prepared_loop_return_chain(
    body: &PreparedLoopBody,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<Option<String>, String> {
    match body {
        PreparedLoopBody::Return { returned } => {
            let value = lower_expr(returned, state, bindings)?;
            Ok(Some(value))
        }
        PreparedLoopBody::Branch {
            condition,
            then_body,
            else_body,
        } => {
            let Some(then_value) = lower_prepared_loop_return_chain(then_body, state, bindings)?
            else {
                return Ok(None);
            };
            let Some(else_value) = lower_prepared_loop_return_chain(else_body, state, bindings)?
            else {
                return Ok(None);
            };
            let branch_condition = lower_expr(condition, state, bindings)?;
            let selected = lower_select(branch_condition, then_value, else_value, state)?;
            Ok(Some(selected))
        }
        _ => Ok(None),
    }
}
