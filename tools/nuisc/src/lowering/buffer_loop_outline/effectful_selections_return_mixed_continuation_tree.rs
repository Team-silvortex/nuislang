use super::*;

#[cfg(test)]
#[path = "effectful_selections_return_mixed_continuation_tree_tests.rs"]
mod tests;

pub(super) fn shape(
    body: &[NirStmt],
    tail_scope: &Scope,
    fresh_tail: &BTreeSet<String>,
    remaining: &mut usize,
    depth: usize,
) -> Option<()> {
    *remaining = remaining.checked_sub(body.len())?;
    if body.len() > 17 || !fresh_tail.is_disjoint(&fresh_names(body, tail_scope)) {
        return None;
    }
    if let Some((
        NirStmt::If {
            then_body,
            else_body,
            ..
        },
        prefix,
    )) = body.split_last()
    {
        if depth >= 8 || !flat(prefix, 16) {
            return None;
        }
        shape(then_body, tail_scope, fresh_tail, remaining, depth + 1)?;
        shape(else_body, tail_scope, fresh_tail, remaining, depth + 1)
    } else {
        flat(body, 17).then_some(())
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn arm(
    body: &[NirStmt],
    tail: &regions::Staging,
    scope: &Scope,
    owner: &Target<'_>,
    child: &Child,
    signatures: &Signatures,
    budget: &mut nested::Budget,
    depth: usize,
) -> Option<nested::Proof> {
    let terminal = match body.last() {
        Some(statement @ NirStmt::If { .. }) => Some(statement),
        _ => None,
    };
    let prefix = if terminal.is_some() {
        &body[..body.len() - 1]
    } else {
        body
    };
    let mut staged = super::super::super::super::stage_authorized(
        &[],
        scope,
        signatures,
        budget,
        depth,
        Some(owner),
    )?;
    for statement in prefix {
        super::super::append(
            statement,
            scope,
            &mut staged,
            owner,
            child,
            signatures,
            budget,
            depth,
        )?;
    }
    if let Some(NirStmt::If {
        condition,
        then_body,
        else_body,
    }) = terminal
    {
        if depth >= 8 {
            return None;
        }
        nested::consume_node(budget)?;
        let (condition, inputs, calls) =
            predicates::prove(condition, &staged.local, signatures, budget)?;
        // Each original child is proved once. Only flat recipes reach leaves;
        // no selection plan or remaining selection sequence is ever cloned.
        let yes = arm(
            then_body,
            tail,
            &staged.local,
            owner,
            child,
            signatures,
            budget,
            depth + 1,
        )?;
        let no = arm(
            else_body,
            tail,
            &staged.local,
            owner,
            child,
            signatures,
            budget,
            depth + 1,
        )?;
        let proof = nested::join(condition, inputs, calls, yes, no, &staged.local)?;
        return Some(super::super::super::finish(staged, proof));
    }
    let mut reads = tail.inputs.clone();
    if !tail.defined.contains(owner.name()) {
        reads.insert(owner.name().into());
    }
    staged
        .inputs
        .extend(reads.difference(&staged.defined).cloned());
    staged.calls.extend(tail.calls.iter().cloned());
    staged.statements.extend(copy_flat(&tail.statements)?);
    Some(nested::Proof {
        value: SelectedValue::Region(staged.statements, NirExpr::Var(owner.name().into())),
        name: owner.name().into(),
        ty: scope.get(owner.name())?.clone(),
        inputs: staged.inputs,
        calls: staged.calls,
    })
}
