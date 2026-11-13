use super::*;

// Nonterminal nested arms are completed with shared continuations before their
// helpers are lowered. No arbitrary loops or mutable resource joins are inferred.
pub(super) fn terminal_return_arm(body: &[NirStmt], depth: usize) -> Result<bool, String> {
    // Check the depth budget before recursively classifying terminal pairs.
    Ok(block(body, depth)? && ends_in_return(body))
}

pub(super) fn ends_in_return(body: &[NirStmt]) -> bool {
    match body.last() {
        Some(NirStmt::Return(Some(_))) => true,
        Some(NirStmt::If {
            then_body,
            else_body,
            ..
        }) => ends_in_return(then_body) && ends_in_return(else_body),
        _ => false,
    }
}

pub(super) fn block(body: &[NirStmt], depth: usize) -> Result<bool, String> {
    if depth > MAX_RETURN_HELPER_DEPTH {
        return Err("resource return helpers exceed the source closure depth bound".into());
    }
    for (index, statement) in body.iter().enumerate() {
        match statement {
            NirStmt::Let { .. } | NirStmt::Const { .. } | NirStmt::Expr(_) => {}
            NirStmt::Return(Some(_)) if index + 1 == body.len() => {}
            NirStmt::If {
                then_body,
                else_body,
                ..
            } => {
                if !block(then_body, depth + 1)? || !block(else_body, depth + 1)? {
                    return Ok(false);
                }
            }
            _ => return Ok(false),
        }
    }
    Ok(true)
}

pub(super) fn partition(body: &mut Vec<NirStmt>, live: &BTreeSet<String>) -> Option<OwnerEdges> {
    if body.iter().any(|s| matches!(s, NirStmt::If { .. })) {
        // Do not hoist path-local drops over inner conditions. The parent helper
        // owns all incoming Bytes and its children partition them independently.
        Some(OwnerEdges {
            cleanup: vec![],
            transfers: live.iter().cloned().collect(),
        })
    } else {
        owners::partition(body, live)
    }
}
