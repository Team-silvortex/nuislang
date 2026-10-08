use super::*;

#[cfg(test)]
#[path = "effectful_selections_continued_tests.rs"]
mod continued_tests;
#[cfg(test)]
#[path = "effectful_selections_repeated_tests.rs"]
mod repeated_tests;
#[cfg(test)]
#[path = "effectful_selections_staged_tests.rs"]
mod staged_tests;
#[cfg(test)]
#[path = "effectful_selections_ordered_tests.rs"]
mod tests;

pub(super) fn prove(
    statements: &[NirStmt],
    scope: &Scope,
    signatures: &Signatures,
    budget: &mut nested::Budget,
    depth: usize,
) -> Option<nested::Proof> {
    // Every immediate statement costs at least one shared node. Reject before
    // allocating plans for an unbounded sequence or accepting a mixed exit.
    if statements.len() > budget.nodes
        || statements.iter().any(|stmt| {
            !matches!(
                stmt,
                NirStmt::Let { .. } | NirStmt::Const { .. } | NirStmt::If { .. }
            )
        })
    {
        return None;
    }
    let mut staging = regions::Staging {
        local: scope.clone(),
        defined: BTreeSet::new(),
        inputs: BTreeSet::new(),
        calls: BTreeSet::new(),
        constants: BTreeSet::new(),
        statements: Vec::new(),
    };
    let mut target = None;
    let mut children = Vec::new();
    let mut cursor = 0;
    while let Some(offset) = statements[cursor..]
        .iter()
        .position(|stmt| matches!(stmt, NirStmt::If { .. }))
    {
        let index = cursor + offset;
        let stage = &statements[cursor..index];
        let stage = append_stage(stage, &mut staging, signatures, budget)?;
        let NirStmt::If {
            condition,
            then_body,
            else_body,
        } = &statements[index]
        else {
            return None;
        };
        // Siblings consume the same budget and selection depth. Only their
        // selected or retained target enters the next sibling's enclosing scope.
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
        if !scalar(original_type)
            || original_type != &proof.ty
            || staging.constants.contains(&proof.name)
        {
            return None;
        }
        if let Some((name, ty)) = &target {
            if name != &proof.name || ty != &proof.ty {
                return None;
            }
        } else {
            target = Some((proof.name.clone(), proof.ty.clone()));
        }
        staging
            .inputs
            .extend(proof.inputs.difference(&staging.defined).cloned());
        staging.calls.extend(proof.calls);
        staging.local.insert(proof.name.clone(), proof.ty);
        staging.defined.insert(proof.name);
        let SelectedValue::Nested(child) = proof.value else {
            return None;
        };
        children.push((stage, child));
        cursor = index + 1;
    }
    let (target, ty) = target?;
    for statement in statements {
        if let NirStmt::Let { name, .. } | NirStmt::Const { name, .. } = statement {
            if scope.contains_key(name) && name != &target {
                return None;
            }
        }
    }
    let suffix = &statements[cursor..];
    if let Some(last) = suffix.last() {
        if !matches!(last, NirStmt::Let { name, .. } if name == &target) {
            return None;
        }
    }
    let suffix = append_stage(suffix, &mut staging, signatures, budget)?;
    Some(nested::Proof {
        value: SelectedValue::Ordered(children, suffix, NirExpr::Var(target.clone())),
        name: target,
        ty,
        inputs: staging.inputs,
        calls: staging.calls,
    })
}

fn append_stage(
    statements: &[NirStmt],
    staging: &mut regions::Staging,
    signatures: &Signatures,
    budget: &mut nested::Budget,
) -> Option<Vec<regions::Statement>> {
    if statements.is_empty() {
        return Some(Vec::new());
    }
    let next = regions::stage(
        statements,
        &staging.local,
        &staging.constants,
        signatures,
        budget,
    )?;
    staging
        .inputs
        .extend(next.inputs.difference(&staging.defined).cloned());
    staging.calls.extend(next.calls);
    staging.defined.extend(next.defined);
    staging.local = next.local;
    staging.constants = next.constants;
    Some(next.statements)
}
