use super::*;

#[cfg(test)]
#[path = "effectful_selections_return_prefix_tests.rs"]
mod tests;

#[path = "effectful_selections_return_updates.rs"]
mod updates;

pub(super) fn prepare(
    condition: &NirExpr,
    yes: &[NirStmt],
    no: &[NirStmt],
    scope: &Scope,
    signatures: &Signatures,
    pure: &BTreeSet<String>,
) -> Option<Plan> {
    let (yes_value, yes_prefix) = returned(yes)?;
    let (no_value, no_prefix) = returned(no)?;
    if (yes_prefix.is_empty() && no_prefix.is_empty())
        || ![yes_value, no_value].iter().any(|value| {
            matches!(
                value,
                NirExpr::Binary {
                    op: NirBinaryOp::And | NirBinaryOp::Or,
                    ..
                }
            )
        })
    {
        return None;
    }
    let mut budget = nested::Budget {
        nodes: 64,
        expressions: 256,
        logical_edges: 32,
    };
    prove(condition, yes, no, scope, signatures, pure, &mut budget)
}

fn returned(statements: &[NirStmt]) -> Option<(&NirExpr, &[NirStmt])> {
    let (NirStmt::Return(Some(value)), prefix) = statements.split_last()? else {
        return None;
    };
    (prefix.len() <= 16).then_some((value, prefix))
}

fn prove(
    condition: &NirExpr,
    yes: &[NirStmt],
    no: &[NirStmt],
    scope: &Scope,
    signatures: &Signatures,
    pure: &BTreeSet<String>,
    budget: &mut nested::Budget,
) -> Option<Plan> {
    nested::consume_node(budget)?;
    let (condition, _, calls) = predicates::prove(condition, scope, signatures, budget)?;
    let yes = body(yes, scope, signatures, budget, 1)?;
    let no = body(no, scope, signatures, budget, 1)?;
    finish(condition, calls, yes, no, scope, pure)
}

pub(super) fn stage(
    prefix: &[NirStmt],
    scope: &Scope,
    signatures: &Signatures,
    budget: &mut nested::Budget,
    depth: usize,
) -> Option<regions::Staging> {
    stage_authorized(prefix, scope, signatures, budget, depth, None)
}

fn stage_authorized(
    prefix: &[NirStmt],
    scope: &Scope,
    signatures: &Signatures,
    budget: &mut nested::Budget,
    depth: usize,
    authorized: Option<&updates::Target<'_>>,
) -> Option<regions::Staging> {
    if prefix.len() > 16 {
        return None;
    }
    if authorized.is_some_and(|target| scope.get(target.name()) != Some(&scalar_type("bool"))) {
        return None;
    }
    // Permission does not define a new version. The first RHS must still
    // capture entering values; only proven source writes advance `defined`.
    let mut staged = regions::Staging {
        local: scope.clone(),
        defined: BTreeSet::new(),
        inputs: BTreeSet::new(),
        calls: BTreeSet::new(),
        constants: BTreeSet::new(),
        statements: Vec::new(),
    };
    for statement in prefix {
        append_authorized(
            statement,
            scope,
            &mut staged,
            signatures,
            budget,
            depth,
            authorized,
        )?;
    }
    Some(staged)
}

#[allow(clippy::too_many_arguments)]
fn append_authorized(
    statement: &NirStmt,
    scope: &Scope,
    staged: &mut regions::Staging,
    signatures: &Signatures,
    budget: &mut nested::Budget,
    depth: usize,
    authorized: Option<&updates::Target<'_>>,
) -> Option<()> {
    if let NirStmt::If {
        condition,
        then_body,
        else_body,
    } = statement
    {
        let proof = updates::prove(
            condition, then_body, else_body, staged, signatures, budget, depth, authorized,
        )?;
        staged
            .inputs
            .extend(proof.inputs.difference(&staged.defined).cloned());
        staged.calls.extend(proof.calls);
        let SelectedValue::Nested(plan) = proof.value else {
            unreachable!()
        };
        staged.statements.push(regions::Statement::Selection(plan));
        return Some(());
    }
    // All entering bindings except the delegated let target stay read-only.
    let name = match statement {
        NirStmt::Let { name, .. } | NirStmt::Const { name, .. } => name,
        _ => return None,
    };
    if scope.contains_key(name)
        && (!matches!(statement, NirStmt::Let { .. })
            || !authorized.is_some_and(|target| target.name() == name))
    {
        return None;
    }
    let next = regions::stage(
        std::slice::from_ref(statement),
        &staged.local,
        &staged.constants,
        signatures,
        budget,
    )?;
    staged
        .inputs
        .extend(next.inputs.difference(&staged.defined).cloned());
    staged.calls.extend(next.calls);
    staged.defined.extend(next.defined);
    staged.local = next.local;
    staged.constants = next.constants;
    staged.statements.extend(next.statements);
    Some(())
}

pub(super) fn body(
    statements: &[NirStmt],
    scope: &Scope,
    signatures: &Signatures,
    budget: &mut nested::Budget,
    depth: usize,
) -> Option<ReturnedValue> {
    let (value, prefix) = returned(statements)?;
    let regions::Staging {
        local,
        defined,
        mut inputs,
        mut calls,
        statements: staged,
        ..
    } = stage(prefix, scope, signatures, budget, depth)?;
    nested::consume_node(budget)?;
    let (predicate, reads, effects) = predicates::prove(value, &local, signatures, budget)?;
    inputs.extend(reads.difference(&defined).cloned());
    calls.extend(effects);
    let value = if staged.is_empty() {
        SelectedValue::Predicate(predicate)
    } else {
        SelectedValue::StagedPredicate(staged, predicate)
    };
    Some(ReturnedValue {
        value,
        inputs,
        calls,
    })
}
