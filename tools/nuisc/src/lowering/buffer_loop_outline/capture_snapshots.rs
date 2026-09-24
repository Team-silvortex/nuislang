use super::*;

#[cfg(test)]
#[path = "capture_snapshots_tests.rs"]
mod tests;

// Give straight-line record writes distinct identities before alias discovery.
// A name written below a control boundary still needs a join/backedge contract,
// so none of its versions are rewritten here.
pub(super) fn normalize(function: &mut NirFunction, layouts: &impl ValueLayouts) {
    if !walk::supported(&function.body) {
        return;
    }
    let nested = nested_writes(&function.body);
    let parameters = function
        .params
        .iter()
        .map(|p| p.name.clone())
        .collect::<BTreeSet<_>>();
    let mut types = function
        .params
        .iter()
        .map(|p| (p.name.clone(), Some(p.ty.clone())))
        .collect::<BTreeMap<_, _>>();
    let mut writes = BTreeMap::<String, usize>::new();
    for stmt in &function.body {
        let (name, ty) = match stmt {
            NirStmt::Let { name, ty, .. } => (name, ty.as_ref()),
            NirStmt::Const { name, ty, .. } => (name, Some(ty)),
            _ => continue,
        };
        *writes.entry(name.clone()).or_default() += 1;
        let expected = types.entry(name.clone()).or_insert_with(|| ty.cloned());
        if expected.as_ref() != ty {
            *expected = None;
        }
    }
    let candidates = writes
        .iter()
        .filter(|(name, count)| {
            !nested.contains(*name)
                && (**count > 1 || parameters.contains(*name))
                && types[*name].as_ref().is_some_and(|ty| {
                    !layouts.scalar(&ty.name) && control_values::supported_type(ty, layouts)
                })
        })
        .map(|(name, _)| name.clone())
        .collect::<BTreeSet<_>>();
    if candidates.is_empty() {
        return;
    }
    let mut reserved = parameters;
    reserved.extend(writes.into_keys());
    reserved.extend(nested);
    walk::visit(&function.body, |expr| {
        if let NirExpr::Var(name) = expr {
            reserved.insert(name.clone());
        }
        true
    });
    let mut versions = BTreeMap::<String, String>::new();
    for stmt in &mut function.body {
        // Initializers read the previous version, including self-rebinding.
        // Nested reads may use it too because candidates have no nested writes.
        walk::rewrite(std::slice::from_mut(stmt), |expr| {
            if let NirExpr::Var(name) = expr {
                if let Some(version) = versions.get(name) {
                    *name = version.clone();
                }
            }
        });
        if let NirStmt::Let { name, .. } | NirStmt::Const { name, .. } = stmt {
            if candidates.contains(name) {
                let version = branches::fresh_name("__nuis_capture_snapshot", &mut reserved);
                versions.insert(name.clone(), version.clone());
                *name = version;
            }
        }
    }
}

fn nested_writes(body: &[NirStmt]) -> BTreeSet<String> {
    let mut pending = vec![(body, false)];
    let mut names = BTreeSet::new();
    while let Some((body, nested)) = pending.pop() {
        for stmt in body {
            match stmt {
                NirStmt::Let { name, .. } | NirStmt::Const { name, .. } if nested => {
                    names.insert(name.clone());
                }
                NirStmt::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    pending.extend([(then_body.as_slice(), true), (else_body.as_slice(), true)]);
                }
                NirStmt::While { body, .. } => pending.push((body, true)),
                _ => {}
            }
        }
    }
    names
}
