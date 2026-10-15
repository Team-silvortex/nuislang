use super::*;

#[cfg(test)]
#[path = "effectful_selections_return_tree_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "effectful_selections_return_ancestor_tests.rs"]
mod ancestor_tests;

#[cfg(test)]
#[path = "effectful_selections_return_plain_tree_tests.rs"]
mod plain_tests;

pub(super) fn prepare(
    condition: &NirExpr,
    yes: &[NirStmt],
    no: &[NirStmt],
    scope: &Scope,
    signatures: &Signatures,
    pure: &BTreeSet<String>,
) -> Option<Plan> {
    if ![yes, no]
        .iter()
        .any(|body| matches!(body.last(), Some(NirStmt::If { .. })))
    {
        return None;
    }
    let mut budget = nested::Budget {
        nodes: 64,
        expressions: 256,
        logical_edges: 32,
    };
    let tree = branch(condition, yes, no, scope, signatures, &mut budget, 0)?;
    // Discovery belongs to the whole tree, not each independently pure child.
    if !tree.calls.iter().any(|name| !pure.contains(name)) {
        return None;
    }
    let SelectedValue::Nested(plan) = tree.value else {
        return None;
    };
    Some(*plan)
}

#[allow(clippy::too_many_arguments)]
fn branch(
    condition: &NirExpr,
    yes: &[NirStmt],
    no: &[NirStmt],
    scope: &Scope,
    signatures: &Signatures,
    budget: &mut nested::Budget,
    depth: usize,
) -> Option<ReturnedValue> {
    if depth >= 8 {
        return None;
    }
    nested::consume_node(budget)?;
    let (condition, mut inputs, mut calls) =
        predicates::prove(condition, scope, signatures, budget)?;
    let yes = body(yes, scope, signatures, budget, depth + 1)?;
    let no = body(no, scope, signatures, budget, depth + 1)?;
    inputs.extend(yes.inputs.iter().chain(&no.inputs).cloned());
    calls.extend(yes.calls.iter().chain(&no.calls).cloned());
    let plan = plan(condition, yes, no, scope)?;
    Some(ReturnedValue {
        value: SelectedValue::Nested(Box::new(plan)),
        inputs,
        calls,
    })
}

fn body(
    statements: &[NirStmt],
    scope: &Scope,
    signatures: &Signatures,
    budget: &mut nested::Budget,
    depth: usize,
) -> Option<ReturnedValue> {
    if let Some((
        NirStmt::If {
            condition,
            then_body,
            else_body,
        },
        prefix,
    )) = statements.split_last()
    {
        let staged = prefixes::stage(prefix, scope, signatures, budget, depth)?;
        let child = branch(
            condition,
            then_body,
            else_body,
            &staged.local,
            signatures,
            budget,
            depth,
        )?;
        let SelectedValue::Nested(plan) = child.value else {
            unreachable!()
        };
        let mut inputs = staged.inputs;
        inputs.extend(child.inputs.difference(&staged.defined).cloned());
        let mut calls = staged.calls;
        calls.extend(child.calls);
        return Some(ReturnedValue {
            value: if staged.statements.is_empty() {
                SelectedValue::Nested(plan)
            } else {
                SelectedValue::StagedNested(staged.statements, plan)
            },
            inputs,
            calls,
        });
    }
    prefixes::body(statements, scope, signatures, budget, depth)
}
