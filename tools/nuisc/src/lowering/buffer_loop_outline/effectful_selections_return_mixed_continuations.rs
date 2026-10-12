use super::*;

#[cfg(test)]
#[path = "effectful_selections_return_mixed_continuation_tests.rs"]
mod tests;

#[path = "effectful_selections_return_mixed_continuation_tree.rs"]
mod tree;

#[path = "effectful_selections_return_mixed_sequences.rs"]
mod sequences;

fn flat(body: &[NirStmt], limit: usize) -> bool {
    body.len() <= limit
        && body
            .iter()
            .all(|s| matches!(s, NirStmt::Let { .. } | NirStmt::Const { .. }))
}

fn fresh_names(body: &[NirStmt], scope: &Scope) -> BTreeSet<String> {
    body.iter()
        .filter_map(|s| match s {
            NirStmt::Let { name, .. } | NirStmt::Const { name, .. }
                if !scope.contains_key(name) =>
            {
                Some(name.clone())
            }
            _ => None,
        })
        .collect()
}

// Copy only a proved flat tail, never a selection tree or another continuation.
fn copy_flat(statements: &[regions::Statement]) -> Option<Vec<regions::Statement>> {
    statements
        .iter()
        .map(|s| {
            Some(match s {
                regions::Statement::Scalar(s) => regions::Statement::Scalar(s.clone()),
                regions::Statement::Logical(s, p) => {
                    regions::Statement::Logical(s.clone(), p.clone())
                }
                regions::Statement::Selection(_) | regions::Statement::PrivatePair(_) => {
                    return None
                }
            })
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
pub(in super::super) fn prove(
    condition: &NirExpr,
    yes: &[NirStmt],
    no: &[NirStmt],
    suffix: &[NirStmt],
    scope: &Scope,
    owner: &Target<'_>,
    child: &Child,
    signatures: &Signatures,
    budget: &mut nested::Budget,
    depth: usize,
) -> Option<nested::Proof> {
    if transport::numeric(scope.get(&child.0)?)
        || numeric_owner(scope.get(owner.name())?)
        || suffix.iter().any(|s| matches!(s, NirStmt::If { .. }))
        || sequences::continued(yes, no)?
    {
        return sequences::prove(
            condition, yes, no, suffix, scope, owner, child, signatures, budget, depth,
        );
    }
    if depth >= 8 || suffix.is_empty() || !flat(suffix, 16) {
        return None;
    }
    // Preflight the original tree, not a recursively copied continuation tree.
    let fresh = fresh_names(suffix, scope);
    let mut remaining = 64;
    tree::shape(yes, scope, &fresh, &mut remaining, depth + 1)?;
    tree::shape(no, scope, &fresh, &mut remaining, depth + 1)?;
    nested::consume_node(budget)?;
    let (condition, inputs, calls) = predicates::prove(condition, scope, signatures, budget)?;
    // Prove the shared tail once in its lexical scope. Recipes are replayed
    // after branch writes, so arguments bind to each branch's current versions.
    let mut tail =
        super::super::super::stage_authorized(&[], scope, signatures, budget, depth, Some(owner))?;
    for statement in suffix {
        super::append(
            statement, scope, &mut tail, owner, child, signatures, budget, depth,
        )?;
    }
    let yes = tree::arm(
        yes,
        &tail,
        scope,
        owner,
        child,
        signatures,
        budget,
        depth + 1,
    )?;
    let no = tree::arm(
        no,
        &tail,
        scope,
        owner,
        child,
        signatures,
        budget,
        depth + 1,
    )?;
    nested::join(condition, inputs, calls, yes, no, scope)
}
