use super::*;

pub(super) fn lower_if_pair(
    condition_name: String,
    then_body: &[NirStmt],
    else_body: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<LoweredIfOutcome, String> {
    if then_body.is_empty() && else_body.is_empty() {
        return Ok(LoweredIfOutcome::Continued);
    }

    if then_body.len() != 1 || else_body.len() != 1 {
        if else_body.is_empty() {
            if let Some(then_branch) = prepare_terminal_branch(then_body, &state.pure_helpers) {
                match then_branch {
                    PreparedTerminalBranch::Return(value) => {
                        let lowered = lower_expr(&value, state, bindings)?;
                        lower_guard_return(condition_name, lowered, state);
                    }
                    PreparedTerminalBranch::PrintReturn { print, returned } => {
                        let print_name = lower_expr(&print, state, bindings)?;
                        let return_name = lower_expr(&returned, state, bindings)?;
                        lower_guard_print_return(condition_name, print_name, return_name, state);
                    }
                }
                return Ok(LoweredIfOutcome::Continued);
            }
        }
        if let (Some(then_branch), Some(else_branch)) = (
            prepare_terminal_branch(then_body, &state.pure_helpers),
            prepare_terminal_branch(else_body, &state.pure_helpers),
        ) {
            match (then_branch, else_branch) {
                (
                    PreparedTerminalBranch::PrintReturn {
                        print: then_print,
                        returned: then_return,
                    },
                    PreparedTerminalBranch::PrintReturn {
                        print: else_print,
                        returned: else_return,
                    },
                ) => {
                    let then_print_name = lower_expr(&then_print, state, bindings)?;
                    let then_return_name = lower_expr(&then_return, state, bindings)?;
                    let else_print_name = lower_expr(&else_print, state, bindings)?;
                    let else_return_name = lower_expr(&else_return, state, bindings)?;
                    lower_branch_print_return(
                        condition_name,
                        then_print_name,
                        then_return_name,
                        else_print_name,
                        else_return_name,
                        state,
                    );
                    return Ok(LoweredIfOutcome::Continued);
                }
                (PreparedTerminalBranch::Return(lhs), PreparedTerminalBranch::Return(rhs)) => {
                    let lhs_name = lower_expr(&lhs, state, bindings)?;
                    let rhs_name = lower_expr(&rhs, state, bindings)?;
                    let selected = lower_select(condition_name, lhs_name, rhs_name, state)?;
                    return Ok(LoweredIfOutcome::Returned(selected));
                }
                _ => {}
            }
        }
        if let (Some(lhs), Some(rhs)) = (
            lower_return_if_chain(then_body, state, bindings)?,
            lower_return_if_chain(else_body, state, bindings)?,
        ) {
            let selected = lower_select(condition_name, lhs, rhs, state)?;
            return Ok(LoweredIfOutcome::Returned(selected));
        }
        let pure_helpers = state.pure_helpers.clone();
        if let (Some((lhs_name, lhs_value)), Some((rhs_name, rhs_value))) = (
            lower_binding_if_chain(then_body, state, bindings, &pure_helpers)?,
            lower_binding_if_chain(else_body, state, bindings, &pure_helpers)?,
        ) {
            if lhs_name == rhs_name {
                let selected = lower_select(condition_name, lhs_value, rhs_value, state)?;
                return Ok(LoweredIfOutcome::Bind {
                    name: lhs_name,
                    value: selected,
                });
            }
        }
        if let Some(lowered) = lower_binding_if_chain_with_shared_context(
            &condition_name,
            then_body,
            else_body,
            state,
            bindings,
        )? {
            return Ok(lowered);
        }
        return Err(
            "minimal nuisc lowering currently only supports `if` as matching `print`, matching `let/const`, `return <expr>`, or small terminal branches like `print(...); return ...`"
                .to_owned(),
        );
    }

    if let (Some(lhs), Some(rhs)) = (
        lower_return_if_chain(then_body, state, bindings)?,
        lower_return_if_chain(else_body, state, bindings)?,
    ) {
        let selected = lower_select(condition_name, lhs, rhs, state)?;
        return Ok(LoweredIfOutcome::Returned(selected));
    }

    let pure_helpers = state.pure_helpers.clone();
    if let (Some((lhs_name, lhs_value)), Some((rhs_name, rhs_value))) = (
        lower_binding_if_chain(then_body, state, bindings, &pure_helpers)?,
        lower_binding_if_chain(else_body, state, bindings, &pure_helpers)?,
    ) {
        if lhs_name == rhs_name {
            let selected = lower_select(condition_name, lhs_value, rhs_value, state)?;
            return Ok(LoweredIfOutcome::Bind {
                name: lhs_name,
                value: selected,
            });
        }
    }

    match (&then_body[0], &else_body[0]) {
        (NirStmt::Print(lhs), NirStmt::Print(rhs)) => {
            let lhs_name = lower_expr(lhs, state, bindings)?;
            let rhs_name = lower_expr(rhs, state, bindings)?;
            let selected = lower_select(condition_name, lhs_name, rhs_name, state)?;
            let print_name = format!("print_{}", state.print_counter);
            state.print_counter += 1;
            state.yir.nodes.push(Node {
                name: print_name.clone(),
                resource: "cpu0".to_owned(),
                op: Operation {
                    module: "cpu".to_owned(),
                    instruction: "print".to_owned(),
                    args: vec![selected.clone()],
                },
            });
            push_dep_edges(state, &selected, &print_name);
            state.yir.edges.push(Edge {
                kind: EdgeKind::Effect,
                from: selected,
                to: print_name,
            });
            Ok(LoweredIfOutcome::Printed)
        }
        (
            NirStmt::Let {
                name: lhs_name,
                value: lhs_value,
                ..
            },
            NirStmt::Let {
                name: rhs_name,
                value: rhs_value,
                ..
            },
        )
        | (
            NirStmt::Const {
                name: lhs_name,
                value: lhs_value,
                ..
            },
            NirStmt::Const {
                name: rhs_name,
                value: rhs_value,
                ..
            },
        ) if lhs_name == rhs_name => {
            let lhs_value = lower_expr(lhs_value, state, bindings)?;
            let rhs_value = lower_expr(rhs_value, state, bindings)?;
            let selected = lower_select(condition_name, lhs_value, rhs_value, state)?;
            Ok(LoweredIfOutcome::Bind {
                name: lhs_name.clone(),
                value: selected,
            })
        }
        (NirStmt::Return(Some(lhs)), NirStmt::Return(Some(rhs))) => {
            let lhs_name = lower_expr(lhs, state, bindings)?;
            let rhs_name = lower_expr(rhs, state, bindings)?;
            let selected = lower_select(condition_name, lhs_name, rhs_name, state)?;
            Ok(LoweredIfOutcome::Returned(selected))
        }
        _ => Err(
            "minimal nuisc lowering currently only supports `if` branches as matching `print`, matching `let/const`, or `return <expr>`"
                .to_owned(),
        ),
    }
}

fn lower_return_if_chain(
    stmts: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<Option<String>, String> {
    match stmts {
        [NirStmt::Return(Some(value))] => Ok(Some(lower_expr(value, state, bindings)?)),
        [NirStmt::If {
            condition,
            then_body,
            else_body,
        }] => {
            let condition_name = lower_expr(condition, state, bindings)?;
            let Some(lhs) = lower_return_if_chain(then_body, state, bindings)? else {
                return Ok(None);
            };
            let Some(rhs) = lower_return_if_chain(else_body, state, bindings)? else {
                return Ok(None);
            };
            Ok(Some(lower_select(condition_name, lhs, rhs, state)?))
        }
        _ => Ok(None),
    }
}

fn lower_binding_if_chain(
    stmts: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
    pure_helpers: &BTreeSet<String>,
) -> Result<Option<(String, String)>, String> {
    match stmts {
        [NirStmt::Let { name, value, .. }] | [NirStmt::Const { name, value, .. }] => {
            Ok(Some((name.clone(), lower_expr(value, state, bindings)?)))
        }
        [binding @ (NirStmt::Let { .. } | NirStmt::Const { .. }), tail @ ..] => {
            let Some((name, value)) = extract_pure_branch_binding(binding, pure_helpers) else {
                return Ok(None);
            };
            let substituted: Vec<NirStmt> = tail
                .iter()
                .map(|stmt| {
                    super::loop_purity::substitute_stmt_bindings(
                        stmt,
                        &[(name.clone(), value.clone())],
                    )
                })
                .collect();
            lower_binding_if_chain(&substituted, state, bindings, pure_helpers)
        }
        [NirStmt::If {
            condition,
            then_body,
            else_body,
        }] => {
            let condition_name = lower_expr(condition, state, bindings)?;
            let Some((lhs_name, lhs_value)) =
                lower_binding_if_chain(then_body, state, bindings, pure_helpers)?
            else {
                return Ok(None);
            };
            let Some((rhs_name, rhs_value)) =
                lower_binding_if_chain(else_body, state, bindings, pure_helpers)?
            else {
                return Ok(None);
            };
            if lhs_name != rhs_name {
                return Ok(None);
            }
            Ok(Some((
                lhs_name,
                lower_select(condition_name, lhs_value, rhs_value, state)?,
            )))
        }
        _ => Ok(None),
    }
}

fn lower_binding_if_chain_with_shared_context(
    condition_name: &str,
    then_body: &[NirStmt],
    else_body: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<Option<LoweredIfOutcome>, String> {
    let (shared_prefix, then_core, else_core, shared_suffix) =
        split_shared_branch_context(then_body, else_body);
    if shared_prefix.is_empty() && shared_suffix.is_empty() {
        return Ok(None);
    }

    let pure_helpers = state.pure_helpers.clone();
    let mut local_bindings = bindings.clone();
    super::body_lowering::lower_linear_stmts(shared_prefix, state, &mut local_bindings)?;

    let Some((lhs_name, lhs_value)) =
        lower_binding_if_chain(then_core, state, &local_bindings, &pure_helpers)?
    else {
        return Ok(None);
    };
    let Some((rhs_name, rhs_value)) =
        lower_binding_if_chain(else_core, state, &local_bindings, &pure_helpers)?
    else {
        return Ok(None);
    };
    if lhs_name != rhs_name {
        return Ok(None);
    }

    let selected = lower_select(condition_name.to_owned(), lhs_value, rhs_value, state)?;
    local_bindings.insert(lhs_name.clone(), selected.clone());

    let outcome_name = if shared_suffix.is_empty() {
        lhs_name
    } else {
        let suffix_last_bound =
            super::body_lowering::lower_linear_stmts(shared_suffix, state, &mut local_bindings)?;
        suffix_last_bound.unwrap_or(lhs_name)
    };
    let Some(outcome_value) = local_bindings.get(&outcome_name).cloned() else {
        return Err(format!(
            "minimal nuisc lowering expected shared branch binding `{outcome_name}` to be available after shared suffix lowering"
        ));
    };

    Ok(Some(LoweredIfOutcome::Bind {
        name: outcome_name,
        value: outcome_value,
    }))
}

fn split_shared_branch_context<'a>(
    then_body: &'a [NirStmt],
    else_body: &'a [NirStmt],
) -> (&'a [NirStmt], &'a [NirStmt], &'a [NirStmt], &'a [NirStmt]) {
    let shared_prefix_len = then_body
        .iter()
        .zip(else_body.iter())
        .take_while(|(lhs, rhs)| lhs == rhs)
        .count();

    let then_remaining = &then_body[shared_prefix_len..];
    let else_remaining = &else_body[shared_prefix_len..];

    let max_shared_suffix_len = then_remaining.len().min(else_remaining.len());
    let mut shared_suffix_len = 0usize;
    while shared_suffix_len < max_shared_suffix_len
        && then_remaining[then_remaining.len() - 1 - shared_suffix_len]
            == else_remaining[else_remaining.len() - 1 - shared_suffix_len]
    {
        shared_suffix_len += 1;
    }

    let then_core_end = then_remaining.len().saturating_sub(shared_suffix_len);
    let else_core_end = else_remaining.len().saturating_sub(shared_suffix_len);

    (
        &then_body[..shared_prefix_len],
        &then_remaining[..then_core_end],
        &else_remaining[..else_core_end],
        &then_remaining[then_core_end..],
    )
}
