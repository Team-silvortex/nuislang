use super::*;

// Check writes independently of generic bool-export discovery. Every bound
// name except the certified numeric owner remains read-only in this tail.
fn writes(
    body: &[NirStmt],
    bound: &BTreeSet<String>,
    owner: &str,
    remaining: &mut usize,
    depth: usize,
) -> Option<()> {
    if depth >= 8 {
        return None;
    }
    let mut local = bound.clone();
    for statement in body {
        *remaining = remaining.checked_sub(1)?;
        match statement {
            NirStmt::Let { name, .. } | NirStmt::Const { name, .. } => {
                if local.contains(name)
                    && (name != owner || !matches!(statement, NirStmt::Let { .. }))
                {
                    return None;
                }
                local.insert(name.clone());
            }
            NirStmt::If {
                then_body,
                else_body,
                ..
            } => {
                writes(then_body, &local, owner, remaining, depth + 1)?;
                writes(else_body, &local, owner, remaining, depth + 1)?;
            }
            _ => return None,
        }
    }
    Some(())
}

#[allow(clippy::too_many_arguments)]
pub(in super::super) fn append(
    statement: &NirStmt,
    entering: &Scope,
    staged: &mut regions::Staging,
    owner: &str,
    signatures: &Signatures,
    budget: &mut nested::Budget,
    depth: usize,
) -> Option<()> {
    let NirStmt::If {
        condition,
        then_body,
        else_body,
    } = statement
    else {
        return append_owner(
            statement, entering, staged, owner, signatures, budget, depth,
        );
    };
    if !numeric_owner(staged.local.get(owner)?) || staged.constants.contains(owner) {
        return None;
    }
    let bound = staged.local.keys().cloned().collect();
    let mut remaining = 64;
    writes(
        std::slice::from_ref(statement),
        &bound,
        owner,
        &mut remaining,
        depth,
    )?;
    // Select this exact owner, rather than rediscovering a bool/child export.
    // The existing branch proof charges the original tree and isolates locals.
    let proof = branch(
        condition,
        then_body,
        else_body,
        owner,
        &staged.local,
        signatures,
        budget,
        depth,
    )?;
    staged
        .inputs
        .extend(proof.inputs.difference(&staged.defined).cloned());
    staged.calls.extend(proof.calls);
    let SelectedValue::Nested(plan) = proof.value else {
        return None;
    };
    staged.statements.push(regions::Statement::Selection(plan));
    Some(())
}
