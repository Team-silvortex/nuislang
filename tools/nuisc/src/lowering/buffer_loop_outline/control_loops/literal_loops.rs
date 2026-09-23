use super::*;

pub(super) fn validate(
    condition: &NirExpr,
    body: &[NirStmt],
    locals: &mut Locals,
    updates: &BTreeSet<String>,
    catalog: &ScalarHelpers,
    layouts: &control_values::FlatLayouts,
    depth: usize,
) -> Option<()> {
    let iteration = induction::parse(condition, body)?;
    let prepared = &iteration.prepared;
    if update_name(iteration.step, &locals.scope, &locals.writable)? != prepared.binding_name {
        return None;
    }
    let normalized = iteration.normalize(&locals.scope)?;
    let effects = normalized
        .as_ref()
        .map_or(iteration.effects, |flow| &flow.effects);
    let mut changed = sequences::carry_names(effects)
        .into_iter()
        .filter(|name| locals.scope.contains_key(name))
        .collect::<BTreeSet<_>>();
    if !changed.insert(prepared.binding_name.clone()) {
        return None;
    }
    for input in [&prepared.limit, &prepared.step] {
        match input {
            NirExpr::Int(_) => {}
            NirExpr::Var(name)
                if !changed.contains(name)
                    && expression(input, locals, updates, None, catalog, layouts, 0)?
                        == scalar_type("i64") => {}
            _ => return None,
        }
    }
    // Keep the parent's unavailable siblings unavailable. Each child backedge
    // then starts a fresh source-ordered write set, without gaining authority.
    let mut child = locals.clone();
    child.available.retain(|name| !changed.contains(name));
    child.available.insert(prepared.binding_name.clone());
    let mut child_updates = updates.clone();
    child_updates.extend(changed.iter().cloned());
    block(
        effects,
        &mut child,
        &child_updates,
        catalog,
        layouts,
        depth + 1,
    )?;
    // Zero trips retain initialized entry values; child-local declarations never
    // escape. Branches and loops share the same bounded source nesting depth.
    locals.available.extend(changed);
    Some(())
}
