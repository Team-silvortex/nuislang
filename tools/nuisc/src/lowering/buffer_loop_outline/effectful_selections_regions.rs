use super::*;

#[cfg(test)]
#[path = "effectful_selections_regions_tests.rs"]
mod tests;

pub(super) struct Staging {
    pub(super) local: Scope,
    pub(super) defined: BTreeSet<String>,
    pub(super) inputs: BTreeSet<String>,
    pub(super) calls: BTreeSet<String>,
}

pub(super) fn prove(
    statements: &[NirStmt],
    scope: &Scope,
    signatures: &Signatures,
    budget: &mut nested::Budget,
) -> Option<nested::Proof> {
    if !(2..=16).contains(&statements.len()) {
        return None;
    }
    let NirStmt::Let { name: target, .. } = statements.last()? else {
        return None;
    };
    let result_type = scope.get(target)?;
    if !scalar(result_type) {
        return None;
    }
    for statement in statements {
        let NirStmt::Let { name, .. } = statement else {
            return None;
        };
        if scope.contains_key(name) && name != target {
            return None;
        }
    }
    let staged = stage(statements, scope, signatures, budget)?;
    if staged.local.get(target)? != result_type {
        return None;
    }
    Some(nested::Proof {
        value: SelectedValue::Region(statements.to_vec(), NirExpr::Var(target.clone())),
        name: target.clone(),
        ty: result_type.clone(),
        inputs: staged.inputs,
        calls: staged.calls,
    })
}

pub(super) fn stage(
    statements: &[NirStmt],
    scope: &Scope,
    signatures: &Signatures,
    budget: &mut nested::Budget,
) -> Option<Staging> {
    if !(1..=16).contains(&statements.len()) {
        return None;
    }
    let mut local = scope.clone();
    let mut defined = BTreeSet::new();
    let mut inputs = BTreeSet::new();
    let mut calls = BTreeSet::new();
    for statement in statements {
        nested::consume_node(budget)?;
        let NirStmt::Let { name, ty, value } = statement else {
            return None;
        };
        // Capture the RHS version before installing its LHS. Each caller proves
        // which outer writes may be published; private staged names never escape.
        let (inferred, reads, effects) = nested::inspect(value, &local, signatures, budget)?;
        if ty.as_ref().is_some_and(|ty| ty != &inferred)
            || local.get(name).is_some_and(|ty| ty != &inferred)
        {
            return None;
        }
        inputs.extend(reads.difference(&defined).cloned());
        calls.extend(effects);
        local.insert(name.clone(), inferred);
        defined.insert(name.clone());
    }
    Some(Staging {
        local,
        defined,
        inputs,
        calls,
    })
}
