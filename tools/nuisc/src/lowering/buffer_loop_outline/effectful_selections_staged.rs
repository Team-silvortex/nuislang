use super::*;

#[cfg(test)]
#[path = "effectful_selections_staged_tests.rs"]
mod tests;

pub(super) fn prove(
    statements: &[NirStmt],
    scope: &Scope,
    signatures: &Signatures,
    budget: &mut nested::Budget,
    depth: usize,
) -> Option<nested::Proof> {
    let (child, prefix) = statements.split_last()?;
    let NirStmt::If {
        condition,
        then_body,
        else_body,
    } = child
    else {
        return None;
    };
    let staging = regions::stage(prefix, scope, signatures, budget)?;
    let proof = nested::branch(
        condition,
        then_body,
        else_body,
        &staging.local,
        signatures,
        budget,
        depth,
    )?;
    let original_type = scope.get(&proof.name)?;
    if !scalar(original_type) || original_type != &proof.ty {
        return None;
    }
    for statement in prefix {
        let NirStmt::Let { name, .. } = statement else {
            return None;
        };
        if scope.contains_key(name) && name != &proof.name {
            return None;
        }
    }
    // The child still captures private/new local versions, while its ancestor
    // captures only reads that occurred before those versions were defined.
    let mut inputs = staging.inputs;
    inputs.extend(proof.inputs.difference(&staging.defined).cloned());
    let calls = staging.calls.union(&proof.calls).cloned().collect();
    let SelectedValue::Nested(child) = proof.value else {
        return None;
    };
    Some(nested::Proof {
        value: SelectedValue::Staged(prefix.to_vec(), child),
        name: proof.name,
        ty: proof.ty,
        inputs,
        calls,
    })
}
