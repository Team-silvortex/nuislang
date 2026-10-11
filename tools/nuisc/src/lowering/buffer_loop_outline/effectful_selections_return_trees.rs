use super::*;

#[cfg(test)]
#[path = "effectful_selections_return_tree_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "effectful_selections_return_ancestor_tests.rs"]
mod ancestor_tests;

struct Tree {
    returned: ReturnedValue,
    logical_return: bool,
}

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
    if !tree.logical_return || !tree.returned.calls.iter().any(|name| !pure.contains(name)) {
        return None;
    }
    let SelectedValue::Nested(plan) = tree.returned.value else {
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
) -> Option<Tree> {
    if depth >= 8 {
        return None;
    }
    nested::consume_node(budget)?;
    let (condition, mut inputs, mut calls) =
        predicates::prove(condition, scope, signatures, budget)?;
    let yes = body(yes, scope, signatures, budget, depth + 1)?;
    let no = body(no, scope, signatures, budget, depth + 1)?;
    let logical_return = yes.logical_return || no.logical_return;
    inputs.extend(
        yes.returned
            .inputs
            .iter()
            .chain(&no.returned.inputs)
            .cloned(),
    );
    calls.extend(yes.returned.calls.iter().chain(&no.returned.calls).cloned());
    let plan = plan(condition, yes.returned, no.returned, scope)?;
    Some(Tree {
        returned: ReturnedValue {
            value: SelectedValue::Nested(Box::new(plan)),
            inputs,
            calls,
        },
        logical_return,
    })
}

fn body(
    statements: &[NirStmt],
    scope: &Scope,
    signatures: &Signatures,
    budget: &mut nested::Budget,
    depth: usize,
) -> Option<Tree> {
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
        let SelectedValue::Nested(plan) = child.returned.value else {
            unreachable!()
        };
        let mut inputs = staged.inputs;
        inputs.extend(child.returned.inputs.difference(&staged.defined).cloned());
        let mut calls = staged.calls;
        calls.extend(child.returned.calls);
        return Some(Tree {
            returned: ReturnedValue {
                value: if staged.statements.is_empty() {
                    SelectedValue::Nested(plan)
                } else {
                    SelectedValue::StagedNested(staged.statements, plan)
                },
                inputs,
                calls,
            },
            logical_return: child.logical_return,
        });
    }
    let logical_return = matches!(
        statements.last(),
        Some(NirStmt::Return(Some(NirExpr::Binary {
            op: NirBinaryOp::And | NirBinaryOp::Or,
            ..
        })))
    );
    Some(Tree {
        returned: prefixes::body(statements, scope, signatures, budget, depth)?,
        logical_return,
    })
}
