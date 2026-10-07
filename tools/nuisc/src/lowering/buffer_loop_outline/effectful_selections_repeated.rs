use super::*;

#[cfg(test)]
#[path = "effectful_selections_repeated_tests.rs"]
mod tests;

pub(super) fn prove(
    statements: &[NirStmt],
    scope: &Scope,
    signatures: &Signatures,
    budget: &mut nested::Budget,
    depth: usize,
) -> Option<nested::Proof> {
    let indices = statements
        .iter()
        .enumerate()
        .filter_map(|(index, stmt)| matches!(stmt, NirStmt::If { .. }).then_some(index))
        .collect::<Vec<_>>();
    if indices.len() != 2 {
        return None;
    }
    let NirStmt::Let { name: target, .. } = statements.last()? else {
        return None;
    };
    let ty = scope.get(target)?;
    if !scalar(ty) {
        return None;
    }
    for statement in statements {
        match statement {
            NirStmt::Let { name, .. } if !scope.contains_key(name) || name == target => {}
            NirStmt::If { .. } => {}
            _ => return None,
        }
    }
    let mut staging = regions::Staging {
        local: scope.clone(),
        defined: BTreeSet::new(),
        inputs: BTreeSet::new(),
        calls: BTreeSet::new(),
    };
    let mut children = Vec::new();
    let mut cursor = 0;
    for index in indices {
        let stage = &statements[cursor..index];
        if cursor != 0 || !stage.is_empty() {
            append_stage(stage, &mut staging, signatures, budget)?;
        }
        let NirStmt::If {
            condition,
            then_body,
            else_body,
        } = &statements[index]
        else {
            return None;
        };
        // Siblings share work budgets, but do not increase selection depth.
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
        // Only the published target enters the next sibling's scope. Its
        // retained arm sees this current version, never a neutral guard result.
        staging.local.insert(proof.name.clone(), proof.ty);
        staging.defined.insert(proof.name);
        let SelectedValue::Nested(child) = proof.value else {
            return None;
        };
        children.push((stage.to_vec(), child));
        cursor = index + 1;
    }
    let suffix = &statements[cursor..];
    append_stage(suffix, &mut staging, signatures, budget)?;
    Some(nested::Proof {
        value: SelectedValue::Repeated(children, suffix.to_vec(), NirExpr::Var(target.clone())),
        name: target.clone(),
        ty: ty.clone(),
        inputs: staging.inputs,
        calls: staging.calls,
    })
}

fn append_stage(
    statements: &[NirStmt],
    staging: &mut regions::Staging,
    signatures: &Signatures,
    budget: &mut nested::Budget,
) -> Option<()> {
    let next = regions::stage(statements, &staging.local, signatures, budget)?;
    staging
        .inputs
        .extend(next.inputs.difference(&staging.defined).cloned());
    staging.calls.extend(next.calls);
    staging.defined.extend(next.defined);
    staging.local = next.local;
    Some(())
}
