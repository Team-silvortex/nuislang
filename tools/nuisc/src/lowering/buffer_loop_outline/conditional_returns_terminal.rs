use super::*;

#[cfg(test)]
#[path = "conditional_returns_computed_arm_tests.rs"]
mod computed_arm_tests;
#[cfg(test)]
#[path = "conditional_returns_complete_predicate_tests.rs"]
mod computed_tests;
#[cfg(test)]
#[path = "conditional_returns_logical_roots_tests.rs"]
mod logical_tests;
#[cfg(test)]
#[path = "conditional_returns_terminal_tests.rs"]
mod tests;

pub(super) fn prepare(
    body: &[NirStmt],
    result: &NirTypeRef,
    scope: &Scope,
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
) -> Option<prefix::Arm> {
    // Bound the complete return tree before recursive inference or cloning.
    // Every leaf returns; no parent continuation is copied into the helper.
    if !bounded(body) {
        return None;
    }
    let mut inputs = BTreeSet::new();
    validate(
        body,
        scope.clone(),
        scope,
        result,
        catalog,
        layouts,
        &mut inputs,
        false,
    )?;
    if inputs
        .iter()
        .any(|name| !scope.get(name).is_some_and(scalar))
    {
        return None;
    }
    Some(prefix::Arm {
        body: body.to_vec(),
        inputs,
    })
}

fn bounded(body: &[NirStmt]) -> bool {
    bounded_tree(body, false)
}

pub(super) fn partial_inputs(
    body: &[NirStmt],
    result: &NirTypeRef,
    scope: &Scope,
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
) -> Option<BTreeSet<String>> {
    // This proves bounded types, purity and captures, not parent exit readiness.
    // The caller must separately prove replayable facts or store an exit signal.
    if !bounded_tree(body, true) {
        return None;
    }
    let mut inputs = BTreeSet::new();
    validate(
        body,
        scope.clone(),
        scope,
        result,
        catalog,
        layouts,
        &mut inputs,
        true,
    )?;
    inputs
        .iter()
        .all(|name| scope.get(name).is_some_and(scalar))
        .then_some(inputs)
}

fn bounded_tree(body: &[NirStmt], allow_fallthrough: bool) -> bool {
    let mut pending = vec![(body, 0usize)];
    let mut statements = 32usize;
    let mut expressions = Vec::new();
    let mut nested = false;
    while let Some((body, depth)) = pending.pop() {
        if depth >= 64 || body.len() > statements {
            return false;
        }
        if body.is_empty() {
            if !allow_fallthrough {
                return false;
            }
            continue;
        }
        statements -= body.len();
        let Some((locals, last)) = split_tail(body, allow_fallthrough) else {
            return false;
        };
        for stmt in locals {
            let value = match stmt {
                NirStmt::Let { value, .. } | NirStmt::Const { value, .. } => value,
                _ => return false,
            };
            expressions.push((value, 0, true));
        }
        match last {
            Some(NirStmt::Return(Some(value))) => expressions.push((value, 0, true)),
            Some(NirStmt::If {
                condition,
                then_body,
                else_body,
            }) => {
                nested = true;
                expressions.push((condition, 0, true));
                pending.extend([
                    (then_body.as_slice(), depth + 1),
                    (else_body.as_slice(), depth + 1),
                ]);
            }
            None => {}
            _ => return false,
        }
    }
    (nested || allow_fallthrough)
        && conditional_values::prefix::computed_expression_roots(expressions)
}

fn split_tail(body: &[NirStmt], allow_fallthrough: bool) -> Option<(&[NirStmt], Option<&NirStmt>)> {
    // A continuation leaf is a whole fresh-binding prefix, never a suffix
    // after intermediate control flow. Preflight and validation share this split.
    match body.last() {
        Some(last @ (NirStmt::Return(Some(_)) | NirStmt::If { .. })) => {
            Some((&body[..body.len() - 1], Some(last)))
        }
        _ if allow_fallthrough => Some((body, None)),
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
fn validate(
    body: &[NirStmt],
    mut inner: Scope,
    outer: &Scope,
    result: &NirTypeRef,
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
    inputs: &mut BTreeSet<String>,
    allow_fallthrough: bool,
) -> Option<()> {
    if body.is_empty() {
        return allow_fallthrough.then_some(());
    }
    let (locals, last) = split_tail(body, allow_fallthrough)?;
    for stmt in locals {
        let (name, declared, value) = match stmt {
            NirStmt::Let { name, ty, value } => (name, ty.as_ref(), value),
            NirStmt::Const { name, ty, value } => (name, Some(ty), value),
            _ => return None,
        };
        if inner.contains_key(name) {
            return None;
        }
        let ty = control_values::value_type(value, &inner, catalog, layouts)?;
        if declared.is_some_and(|declared| declared != &ty) {
            return None;
        }
        capture(value, outer, inputs);
        inner.insert(name.clone(), ty);
    }
    match last {
        Some(NirStmt::Return(Some(value))) => {
            if !expression(value, &inner)
                || control_values::value_type(value, &inner, catalog, layouts).as_ref()
                    != Some(result)
            {
                return None;
            }
            capture(value, outer, inputs);
        }
        Some(NirStmt::If {
            condition,
            then_body,
            else_body,
        }) => {
            // Shared value-root lowering guards one logical edge at the original
            // condition site. Non-total partial predicates use stored signals.
            // Every tree evaluates its original pure condition inside selected
            // work. Partial trees separately prove replay or stored exit signals.
            let admitted = control_values::value_type(condition, &inner, catalog, layouts).as_ref()
                == Some(&scalar_type("bool"));
            if !admitted {
                return None;
            }
            capture(condition, outer, inputs);
            // Lexical arms see the same earlier values, never sibling locals.
            validate(
                then_body,
                inner.clone(),
                outer,
                result,
                catalog,
                layouts,
                inputs,
                allow_fallthrough,
            )?;
            validate(
                else_body,
                inner,
                outer,
                result,
                catalog,
                layouts,
                inputs,
                allow_fallthrough,
            )?;
        }
        None => return allow_fallthrough.then_some(()),
        _ => return None,
    }
    Some(())
}

fn capture(value: &NirExpr, outer: &Scope, inputs: &mut BTreeSet<String>) {
    let mut used = BTreeSet::new();
    control_values::collect_inputs(value, &mut used);
    inputs.extend(used.into_iter().filter(|name| outer.contains_key(name)));
}
