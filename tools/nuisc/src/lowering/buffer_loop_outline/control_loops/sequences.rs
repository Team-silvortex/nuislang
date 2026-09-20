use super::*;

/// Stable lexical order, with one return slot per pre-existing carry rather
/// than per write. Each branch keeps every carry it does not update.
pub(super) fn carry_names(body: &[NirStmt]) -> Vec<String> {
    let mut pending = body.iter().rev().collect::<Vec<_>>();
    let mut seen = BTreeSet::new();
    let mut names = Vec::new();
    while let Some(stmt) = pending.pop() {
        match stmt {
            NirStmt::Let { name, .. } => {
                if seen.insert(name.clone()) {
                    names.push(name.clone());
                }
            }
            NirStmt::If {
                then_body,
                else_body,
                ..
            } => {
                pending.extend(else_body.iter().rev());
                pending.extend(then_body.iter().rev());
            }
            _ => {}
        }
    }
    names
}

pub(super) fn validate(
    prepared: &PreparedCountedWhile,
    body: &[NirStmt],
    scope: &Scope,
    loop_bindings: &BTreeSet<String>,
    catalog: &ScalarHelpers,
    layouts: &control_values::FlatLayouts,
) -> Option<()> {
    let (first, effects) = body.split_first()?;
    // Reuse the strict single-step admission, including exact i64 and local
    // mutability. No branch can advance the outer induction a second time.
    if update_name(first, scope, loop_bindings)? != prepared.binding_name {
        return None;
    }
    let writes = carry_names(effects);
    let has_temporaries = writes.iter().any(|name| !scope.contains_key(name));
    let updates = writes
        .into_iter()
        .filter(|name| scope.contains_key(name))
        .collect::<BTreeSet<_>>();
    if (!has_temporaries && updates.is_empty()) || updates.contains(&prepared.binding_name) {
        return None;
    }
    for input in [&prepared.limit, &prepared.step] {
        match input {
            NirExpr::Int(_) => {}
            NirExpr::Var(name)
                if name != &prepared.binding_name
                    && !updates.contains(name)
                    && scope.get(name) == Some(&scalar_type("i64")) => {}
            _ => return None,
        }
    }
    let has_flat_carries = updates.iter().any(|name| scope[name] != scalar_type("i64"));
    let mut mutable = updates;
    mutable.insert(prepared.binding_name.clone());
    if has_temporaries
        || has_flat_carries
        || speculation::block_has_checked_arithmetic(effects, &BTreeSet::new())
        || scalar_helpers::contains_calls(effects)
        || control_values::has_aggregate_expressions(effects)
    {
        return temporaries::validate(
            effects,
            scope,
            loop_bindings,
            &mutable,
            &prepared.binding_name,
            catalog,
            layouts,
        );
    }
    validate_block(
        effects,
        scope,
        loop_bindings,
        &mutable,
        &mut BTreeSet::from([prepared.binding_name.clone()]),
        0,
    )
}

fn validate_block(
    body: &[NirStmt],
    scope: &Scope,
    loop_bindings: &BTreeSet<String>,
    updates: &BTreeSet<String>,
    available: &mut BTreeSet<String>,
    depth: usize,
) -> Option<()> {
    for stmt in body {
        match stmt {
            NirStmt::Let { name, .. } => {
                // Only a seeded mutable local can cross the iteration boundary.
                // Preserve the no-forward-sibling-read policy; once a binding
                // has been updated, later writes observe its new value.
                update_name(stmt, scope, loop_bindings)?;
                validate_update(stmt, name, scope, updates, available)?;
                available.insert(name.clone());
            }
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } if depth < 32 => {
                if then_body.is_empty() && else_body.is_empty() {
                    return None;
                }
                validate_condition(condition, scope, updates, available)?;
                let mut then_available = available.clone();
                let mut else_available = available.clone();
                validate_block(
                    then_body,
                    scope,
                    loop_bindings,
                    updates,
                    &mut then_available,
                    depth + 1,
                )?;
                validate_block(
                    else_body,
                    scope,
                    loop_bindings,
                    updates,
                    &mut else_available,
                    depth + 1,
                )?;
                // All these names already exist on entry. Untaken arms retain
                // their own input values, so the join makes both write sets
                // available without leaking arm-local bindings.
                available.extend(then_available);
                available.extend(else_available);
            }
            _ => return None,
        }
    }
    Some(())
}
