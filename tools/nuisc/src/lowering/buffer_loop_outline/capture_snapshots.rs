use super::*;

#[cfg(test)]
#[path = "capture_snapshots_tests.rs"]
mod tests;

#[path = "capture_snapshot_scopes.rs"]
mod scopes;

// Binding hygiene runs first. Writes may stay in their non-loop owner or a
// returning child scope. A fallthrough join/backedge still needs its own contract.
pub(super) fn normalize(function: &mut NirFunction, layouts: &impl ValueLayouts) {
    if !walk::supported(&function.body) {
        return;
    }
    let candidates = snapshot_candidates(function, layouts);
    if candidates.is_empty() {
        return;
    }
    let mut reserved = function.params.iter().map(|p| p.name.clone()).collect();
    branches::collect_bindings(&function.body, &mut reserved);
    walk::visit(&function.body, |expr| {
        if let NirExpr::Var(name) = expr {
            reserved.insert(name.clone());
        }
        true
    });
    let mut pending = vec![(&mut function.body, BTreeMap::new())];
    while let Some((body, mut versions)) = pending.pop() {
        for stmt in body {
            match stmt {
                NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. } => {
                    // Initializers read the previous version, including self-rebinding.
                    rewrite(value, &versions);
                    if candidates.contains(name) {
                        let version =
                            branches::fresh_name("__nuis_capture_snapshot", &mut reserved);
                        versions.insert(name.clone(), version.clone());
                        *name = version;
                    }
                }
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    rewrite(condition, &versions);
                    pending.push((else_body, versions.clone()));
                    pending.push((then_body, versions.clone()));
                }
                NirStmt::While { condition, body } => {
                    rewrite(condition, &versions);
                    pending.push((body, versions.clone()));
                }
                NirStmt::Print(value)
                | NirStmt::Expr(value)
                | NirStmt::Await(value)
                | NirStmt::Return(Some(value)) => rewrite(value, &versions),
                NirStmt::Break | NirStmt::Continue | NirStmt::Return(None) => {}
            }
        }
    }
}

struct Writes {
    scope: usize,
    ty: Option<NirTypeRef>,
    count: usize,
    versionable: bool,
}

fn snapshot_candidates(function: &NirFunction, layouts: &impl ValueLayouts) -> BTreeSet<String> {
    let parameters = function
        .params
        .iter()
        .map(|p| p.name.clone())
        .collect::<BTreeSet<_>>();
    let mut writes = function
        .params
        .iter()
        .map(|p| {
            (
                p.name.clone(),
                Writes {
                    scope: 0,
                    ty: Some(p.ty.clone()),
                    count: 0,
                    versionable: true,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    for (scope, info) in scopes::collect(&function.body).iter().enumerate() {
        for stmt in info.body {
            match stmt {
                NirStmt::Let { .. } | NirStmt::Const { .. } => {
                    let (name, ty) = match stmt {
                        NirStmt::Let { name, ty, .. } => (name, ty.as_ref()),
                        NirStmt::Const { name, ty, .. } => (name, Some(ty)),
                        _ => unreachable!(),
                    };
                    let entry = writes.entry(name.clone()).or_insert_with(|| Writes {
                        scope,
                        ty: ty.cloned(),
                        count: 0,
                        versionable: true,
                    });
                    entry.count += 1;
                    entry.versionable &= !info.in_loop
                        && (entry.scope == scope || info.returns)
                        && entry.ty.as_ref() == ty;
                }
                _ => {}
            }
        }
    }
    writes
        .into_iter()
        .filter(|(name, entry)| {
            entry.versionable
                && (entry.count > 1 || (entry.count > 0 && parameters.contains(name)))
                && entry.ty.as_ref().is_some_and(|ty| {
                    !layouts.scalar(&ty.name) && control_values::supported_type(ty, layouts)
                })
        })
        .map(|(name, _)| name)
        .collect()
}

fn rewrite(expr: &mut NirExpr, versions: &BTreeMap<String, String>) {
    walk::rewrite_expr(expr, |expr| {
        if let NirExpr::Var(name) = expr {
            if let Some(version) = versions.get(name) {
                *name = version.clone();
            }
        }
    });
}
