use super::*;

#[cfg(test)]
#[path = "effectful_selections_continued_tests.rs"]
mod tests;

pub(super) fn prove(
    statements: &[NirStmt],
    scope: &Scope,
    signatures: &Signatures,
    budget: &mut nested::Budget,
    depth: usize,
) -> Option<nested::Proof> {
    let index = statements
        .iter()
        .position(|stmt| matches!(stmt, NirStmt::If { .. }))?;
    let (prefix, rest) = statements.split_at(index);
    let (child, suffix) = rest.split_first()?;
    let NirStmt::Let { name: target, .. } = suffix.last()? else {
        return None;
    };
    let ty = scope.get(target)?;
    if !scalar(ty) {
        return None;
    }
    for statement in prefix.iter().chain(suffix) {
        let NirStmt::Let { name, .. } = statement else {
            return None;
        };
        if scope.contains_key(name) && name != target {
            return None;
        }
    }
    let mut staging = if prefix.is_empty() {
        regions::Staging {
            local: scope.clone(),
            defined: BTreeSet::new(),
            inputs: BTreeSet::new(),
            calls: BTreeSet::new(),
        }
    } else {
        regions::stage(prefix, scope, signatures, budget)?
    };
    let NirStmt::If {
        condition,
        then_body,
        else_body,
    } = child
    else {
        return None;
    };
    let proof = nested::branch(
        condition,
        then_body,
        else_body,
        &staging.local,
        signatures,
        budget,
        depth,
    )?;
    if &proof.name != target || &proof.ty != ty {
        return None;
    }
    staging
        .inputs
        .extend(proof.inputs.difference(&staging.defined).cloned());
    staging.calls.extend(proof.calls);
    // Suffix reads of this target consume the child's actual merged version,
    // not another ancestor capture. Child-private bindings remain unavailable.
    staging.local.insert(proof.name.clone(), proof.ty.clone());
    staging.defined.insert(proof.name);
    let tail = regions::stage(suffix, &staging.local, signatures, budget)?;
    staging
        .inputs
        .extend(tail.inputs.difference(&staging.defined).cloned());
    staging.calls.extend(tail.calls);
    let SelectedValue::Nested(child) = proof.value else {
        return None;
    };
    Some(nested::Proof {
        value: SelectedValue::Continued(
            prefix.to_vec(),
            child,
            suffix.to_vec(),
            NirExpr::Var(target.clone()),
        ),
        name: target.clone(),
        ty: ty.clone(),
        inputs: staging.inputs,
        calls: staging.calls,
    })
}
