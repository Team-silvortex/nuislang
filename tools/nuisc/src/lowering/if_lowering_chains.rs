use super::if_lowering_effects::expr_contains_conditional_effect_primitive;
use super::*;

pub(super) fn lower_return_if_chain(
    stmts: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<Option<String>, String> {
    match stmts {
        [NirStmt::Return(Some(value))] | [NirStmt::Expr(value)] => {
            if !is_terminal_branch_pure_expr(value, &state.pure_helpers)
                || expr_contains_conditional_effect_primitive(value)
            {
                return Ok(None);
            }
            Ok(Some(lower_expr(value, state, bindings)?))
        }
        [binding @ (NirStmt::Let { .. } | NirStmt::Const { .. }), tail @ ..] => {
            let pure_helpers = state.pure_helpers.clone();
            let Some((name, value)) = extract_pure_branch_binding(binding, &pure_helpers) else {
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
            lower_return_if_chain(&substituted, state, bindings)
        }
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

pub(in crate::lowering) fn lower_guard_return_chain(
    stmts: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<Option<String>, String> {
    match stmts {
        [NirStmt::Return(Some(value))] | [NirStmt::Expr(value)] => {
            if !is_terminal_branch_pure_expr(value, &state.pure_helpers)
                || expr_contains_conditional_effect_primitive(value)
            {
                return Ok(None);
            }
            Ok(Some(lower_expr(value, state, bindings)?))
        }
        [binding @ (NirStmt::Let { .. } | NirStmt::Const { .. }), tail @ ..] => {
            let pure_helpers = state.pure_helpers.clone();
            let Some((name, value)) = extract_pure_branch_binding(binding, &pure_helpers) else {
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
            lower_guard_return_chain(&substituted, state, bindings)
        }
        [NirStmt::If {
            condition,
            then_body,
            else_body,
        }, tail @ ..]
            if else_body.is_empty() =>
        {
            if expr_contains_conditional_effect_primitive(condition) {
                return Ok(None);
            }
            let condition_name = lower_expr(condition, state, bindings)?;
            let Some(lhs) = lower_return_if_chain(then_body, state, bindings)? else {
                return Ok(None);
            };
            let Some(rhs) = lower_guard_return_chain(tail, state, bindings)? else {
                return Ok(None);
            };
            Ok(Some(lower_select(condition_name, lhs, rhs, state)?))
        }
        _ => Ok(None),
    }
}

pub(super) fn lower_binding_if_chain(
    stmts: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
    pure_helpers: &BTreeSet<String>,
) -> Result<Option<(String, String)>, String> {
    match stmts {
        [NirStmt::Let { name, value, .. }] | [NirStmt::Const { name, value, .. }] => {
            if !is_terminal_branch_pure_expr(value, pure_helpers)
                || expr_contains_conditional_effect_primitive(value)
            {
                return Ok(None);
            }
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

pub(super) fn lower_binding_if_chain_with_shared_context(
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

pub(super) fn lower_return_if_chain_with_shared_context(
    condition_name: &str,
    then_body: &[NirStmt],
    else_body: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<Option<LoweredIfOutcome>, String> {
    let (shared_prefix, then_core, else_core, shared_suffix) =
        split_shared_branch_context(then_body, else_body);
    if shared_suffix.is_empty() {
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
    local_bindings.insert(lhs_name, selected);

    let Some(returned) = lower_return_if_chain(shared_suffix, state, &local_bindings)? else {
        return Ok(None);
    };
    Ok(Some(LoweredIfOutcome::Returned(returned)))
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
