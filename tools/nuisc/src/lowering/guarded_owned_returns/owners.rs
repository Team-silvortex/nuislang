use super::*;

// This is a source cleanup proposal, not an ownership proof. The native profile
// independently requires the exact live SSA owner set at every guarded call.
pub(super) fn advance(statement: &NirStmt, live: &mut BTreeSet<String>) {
    match statement {
        NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. } => match value {
            NirExpr::CopyBufferOwned(_) => {
                live.insert(name.clone());
            }
            NirExpr::Move(inner) => {
                if let NirExpr::Var(source) = inner.as_ref() {
                    if live.remove(source) {
                        live.insert(name.clone());
                    }
                }
            }
            _ => {}
        },
        NirStmt::Expr(NirExpr::DropBytes(inner)) => {
            if let NirExpr::Var(source) = inner.as_ref() {
                live.remove(source);
            }
        }
        _ => {}
    }
}

pub(super) fn extract_cleanup(body: &mut Vec<NirStmt>, live: &BTreeSet<String>) -> bool {
    let mut remaining = live.clone();
    for statement in body.iter() {
        if let Some(owner) = dropped_owner(statement).filter(|owner| live.contains(*owner)) {
            if !remaining.remove(owner) {
                return false;
            }
            continue;
        }
        let value = match statement {
            NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. } => {
                if live.contains(name) {
                    return false;
                }
                value
            }
            NirStmt::Expr(value) | NirStmt::Return(Some(value)) => value,
            _ => return false,
        };
        if mentions_owner(value, live) {
            return false;
        }
    }
    if !remaining.is_empty() {
        return false;
    }
    // The earlier scope-cleanup pass may put drops after the returned value's
    // temporary binding. Only owners unused by the entire helper can move to
    // the taken edge, before any helper allocation that could fail.
    body.retain(|statement| !dropped_owner(statement).is_some_and(|n| live.contains(n)));
    true
}

pub(super) fn partition(body: &mut Vec<NirStmt>, live: &BTreeSet<String>) -> Option<OwnerEdges> {
    // Referenced owners enter the helper's independent linear proof. Drop-only
    // owners remain caller cleanup, so ordinary helper failure cannot retain them.
    let mut transfers = BTreeSet::new();
    for statement in body.iter() {
        if dropped_owner(statement).is_some_and(|n| live.contains(n)) {
            continue;
        }
        let value = match statement {
            NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. } => {
                if live.contains(name) {
                    return None;
                }
                value
            }
            NirStmt::Expr(value) | NirStmt::Return(Some(value)) => value,
            _ => return None,
        };
        for owner in live {
            if mentions_owner(value, &BTreeSet::from([owner.clone()])) {
                transfers.insert(owner.clone());
            }
        }
    }
    let cleanup = live
        .difference(&transfers)
        .cloned()
        .collect::<BTreeSet<_>>();
    extract_cleanup(body, &cleanup).then(|| OwnerEdges {
        cleanup: cleanup.into_iter().collect(),
        transfers: transfers.into_iter().collect(),
    })
}

fn dropped_owner(statement: &NirStmt) -> Option<&String> {
    if let NirStmt::Expr(NirExpr::DropBytes(inner)) = statement {
        if let NirExpr::Var(source) = inner.as_ref() {
            return Some(source);
        }
    }
    None
}

fn mentions_owner(expr: &NirExpr, live: &BTreeSet<String>) -> bool {
    let mut found = matches!(expr, NirExpr::Var(name) if live.contains(name));
    crate::nir_walk::walk_child_exprs(expr, &mut |child| {
        found |= mentions_owner(child, live);
    });
    found
}
