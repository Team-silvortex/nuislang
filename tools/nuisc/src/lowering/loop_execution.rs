use super::*;

pub(super) fn prepare_guarded_loop_body(
    stmts: &[NirStmt],
    pure_helpers: &BTreeSet<String>,
) -> Option<PreparedLoopBody> {
    match stmts {
        [NirStmt::Break] | [NirStmt::Continue] => Some(PreparedLoopBody::ExitOnly),
        [NirStmt::Print(print), NirStmt::Break] | [NirStmt::Print(print), NirStmt::Continue] => {
            Some(PreparedLoopBody::PrintExit {
                print: print.clone(),
            })
        }
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
        PreparedLoopBody::ExitOnly => Ok(None),
        PreparedLoopBody::PrintExit { print } => {
            let print_name = lower_expr(print, state, bindings)?;
            lower_guard_print(condition_name, print_name, state);
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
                (PreparedLoopBody::ExitOnly, PreparedLoopBody::ExitOnly) => Ok(None),
                (
                    PreparedLoopBody::PrintExit { print: then_print },
                    PreparedLoopBody::PrintExit { print: else_print },
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
                    let selected_return = lower_select(
                        branch_condition,
                        then_return_name,
                        else_return_name,
                        state,
                    )?;
                    lower_guard_print_return(condition_name, selected_print, selected_return, state);
                    Ok(None)
                }
                _ => Err(
                    "guarded `while` currently supports nested `if` only when both branches share the same guarded terminal shape: `break/continue`, `print(...); break/continue;`, `return ...;`, or `print(...); return ...;`"
                        .to_owned(),
                ),
            }
        }
    }
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
