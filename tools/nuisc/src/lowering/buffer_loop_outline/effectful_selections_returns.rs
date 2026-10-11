use super::*;

#[path = "effectful_selections_return_prefixes.rs"]
mod prefixes;

#[path = "effectful_selections_return_trees.rs"]
mod trees;

struct ReturnedValue {
    value: SelectedValue,
    inputs: BTreeSet<String>,
    calls: BTreeSet<String>,
}

#[cfg(test)]
#[path = "effectful_selections_return_logical_tests.rs"]
mod tests;

pub(super) fn prepare(
    return_type: Option<&NirTypeRef>,
    condition: &NirExpr,
    yes: &[NirStmt],
    no: &[NirStmt],
    scope: &Scope,
    signatures: &Signatures,
    pure: &BTreeSet<String>,
) -> Option<Plan> {
    if return_type != Some(&scalar_type("bool")) {
        return None;
    }
    let ([NirStmt::Return(Some(yes))], [NirStmt::Return(Some(no))]) = (yes, no) else {
        return prefixes::prepare(condition, yes, no, scope, signatures, pure)
            .or_else(|| trees::prepare(condition, yes, no, scope, signatures, pure));
    };
    if ![yes, no].iter().any(|value| {
        matches!(
            value,
            NirExpr::Binary {
                op: NirBinaryOp::And | NirBinaryOp::Or,
                ..
            }
        )
    }) {
        return None;
    }
    let mut budget = nested::Budget {
        nodes: 64,
        expressions: 256,
        logical_edges: 32,
    };
    prove(condition, yes, no, scope, signatures, pure, &mut budget)
}

fn prove(
    condition: &NirExpr,
    yes: &NirExpr,
    no: &NirExpr,
    scope: &Scope,
    signatures: &Signatures,
    pure: &BTreeSet<String>,
    budget: &mut nested::Budget,
) -> Option<Plan> {
    // Charge the original selection and its two returns, not generated helpers.
    for _ in 0..3 {
        nested::consume_node(budget)?;
    }
    let (condition, _, condition_calls) = predicates::prove(condition, scope, signatures, budget)?;
    let (yes, yes_inputs, yes_calls) = predicates::prove(yes, scope, signatures, budget)?;
    let (no, no_inputs, no_calls) = predicates::prove(no, scope, signatures, budget)?;
    finish(
        condition,
        condition_calls,
        ReturnedValue {
            value: SelectedValue::Predicate(yes),
            inputs: yes_inputs,
            calls: yes_calls,
        },
        ReturnedValue {
            value: SelectedValue::Predicate(no),
            inputs: no_inputs,
            calls: no_calls,
        },
        scope,
        pure,
    )
}

fn finish(
    condition: predicates::Predicate,
    condition_calls: BTreeSet<String>,
    yes: ReturnedValue,
    no: ReturnedValue,
    scope: &Scope,
    pure: &BTreeSet<String>,
) -> Option<Plan> {
    if !yes
        .calls
        .iter()
        .chain(&no.calls)
        .chain(&condition_calls)
        .any(|name| !pure.contains(name))
    {
        return None;
    }
    plan(condition, yes, no, scope)
}

fn plan(
    condition: predicates::Predicate,
    yes: ReturnedValue,
    no: ReturnedValue,
    scope: &Scope,
) -> Option<Plan> {
    let captures = yes
        .inputs
        .union(&no.inputs)
        .cloned()
        .collect::<BTreeSet<_>>();
    if captures.len() > 31 {
        return None;
    }
    let params = captures
        .iter()
        .map(|name| {
            let ty = scope.get(name)?;
            scalar(ty).then(|| NirParam {
                name: name.clone(),
                ty: ty.clone(),
            })
        })
        .collect::<Option<Vec<_>>>()?;
    Some(Plan {
        condition,
        yes: yes.value,
        no: no.value,
        destination: None,
        ty: scalar_type("bool"),
        params,
    })
}
