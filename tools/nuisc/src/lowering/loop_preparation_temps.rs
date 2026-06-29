use super::*;

pub(super) fn is_loop_match_scrutinee_temp_binding(name: &str) -> bool {
    name.starts_with("__match_scrutinee_")
}

pub(super) fn extract_loop_match_scrutinee_temp_binding(
    stmt: &NirStmt,
    pure_helpers: &BTreeSet<String>,
) -> Option<(String, NirExpr)> {
    let (name, expr) = extract_pure_branch_binding(stmt, pure_helpers)?;
    if is_loop_match_scrutinee_temp_binding(&name) {
        Some((name, expr))
    } else {
        None
    }
}

fn extract_loop_control_temp_binding(
    stmt: &NirStmt,
    consumer_stmts: &[&NirStmt],
    pure_helpers: &BTreeSet<String>,
) -> Option<(String, NirExpr)> {
    let (name, expr) = extract_pure_branch_binding(stmt, pure_helpers)?;
    if is_loop_match_scrutinee_temp_binding(&name) {
        return Some((name, expr));
    }
    let declares_bool_temp = match stmt {
        NirStmt::Let { ty, .. } => ty.as_ref().is_some_and(|ty| ty.is_bool_scalar()),
        NirStmt::Const { ty, .. } => ty.is_bool_scalar(),
        _ => false,
    };
    if !declares_bool_temp {
        return None;
    }
    if consumer_stmts
        .iter()
        .any(|consumer| stmt_references_any_name(consumer, &BTreeSet::from([name.clone()])))
    {
        Some((name, expr))
    } else {
        None
    }
}

fn normalize_loop_control_temp_bindings(
    bindings: Vec<(String, NirExpr)>,
) -> Vec<(String, NirExpr)> {
    let mut normalized = Vec::<(String, NirExpr)>::new();
    for (name, expr) in bindings {
        let normalized_expr =
            normalized
                .iter()
                .fold(expr, |current, (binding_name, binding_expr)| {
                    substitute_branch_binding(&current, binding_name, binding_expr)
                });
        normalized.push((name, normalized_expr));
    }
    normalized
}

pub(super) fn split_temp_prefixed_loop_flow_control<'a>(
    stmts: &'a [NirStmt],
    pure_helpers: &BTreeSet<String>,
) -> Option<(Vec<(String, NirExpr)>, &'a NirStmt, &'a [NirStmt])> {
    let mut temp_bindings = Vec::<(String, NirExpr)>::new();
    for (index, stmt) in stmts.iter().enumerate() {
        let remaining = &stmts[index + 1..];
        let consumer_stmts = remaining.iter().collect::<Vec<_>>();
        if let Some((temp_name, temp_expr)) =
            extract_loop_control_temp_binding(stmt, &consumer_stmts, pure_helpers)
        {
            temp_bindings.push((temp_name, temp_expr));
            continue;
        }
        return Some((
            normalize_loop_control_temp_bindings(temp_bindings),
            stmt,
            &stmts[index + 1..],
        ));
    }
    None
}

pub(super) fn split_trailing_loop_control_temp_bindings<'a>(
    stmts: &'a [NirStmt],
    control_stmt: &'a NirStmt,
    pure_helpers: &BTreeSet<String>,
) -> Option<(&'a [NirStmt], Vec<(String, NirExpr)>)> {
    let mut accepted = Vec::<(String, NirExpr)>::new();
    let mut consumer_stmts = vec![control_stmt];
    let mut split_index = stmts.len();
    for stmt in stmts.iter().rev() {
        let Some((temp_name, temp_expr)) =
            extract_loop_control_temp_binding(stmt, &consumer_stmts, pure_helpers)
        else {
            break;
        };
        accepted.push((temp_name, temp_expr));
        consumer_stmts.push(stmt);
        split_index -= 1;
    }
    accepted.reverse();
    Some((
        &stmts[..split_index],
        normalize_loop_control_temp_bindings(accepted),
    ))
}

pub(super) fn split_temp_prefixed_loop_step_bindings<'a>(
    body: &'a [NirStmt],
    binding_name: &str,
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
) -> Option<(Vec<(String, NirExpr)>, &'a NirStmt, &'a [NirStmt])> {
    let mut temp_bindings = Vec::<(String, NirExpr)>::new();
    let prev_current = NirExpr::Var(TAIL_RECURSIVE_PREV_CURRENT_BINDING.to_owned());
    for (index, stmt) in body.iter().enumerate() {
        let (name, expr) = match stmt {
            NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. } => {
                (name.clone(), value.clone())
            }
            _ => return None,
        };
        if name == binding_name {
            return Some((
                normalize_loop_control_temp_bindings(temp_bindings),
                stmt,
                &body[index + 1..],
            ));
        }
        if !is_terminal_branch_pure_expr(&expr, pure_helpers) {
            return None;
        }
        let normalized = inline_pure_helper_calls(&expr, inlineable_pure_helpers);
        let preserved = substitute_branch_binding(&normalized, binding_name, &prev_current);
        temp_bindings.push((name, preserved));
    }
    None
}
