use super::*;

#[cfg(test)]
#[path = "effectful_selections_nested_tests.rs"]
mod tests;

pub(super) struct Budget {
    pub(super) nodes: usize,
    pub(super) expressions: usize,
    pub(super) logical_edges: usize,
}

pub(super) struct Proof {
    pub(super) value: SelectedValue,
    pub(super) name: String,
    pub(super) ty: NirTypeRef,
    pub(super) inputs: BTreeSet<String>,
    pub(super) calls: BTreeSet<String>,
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
        .any(|body| body.len() > 1 || matches!(*body, [NirStmt::If { .. }]))
    {
        return None;
    }
    let mut budget = Budget {
        nodes: 64,
        expressions: 256,
        logical_edges: 32,
    };
    let proof = branch(condition, yes, no, scope, signatures, &mut budget, 0)?;
    // Effectful descendant conditions also need ancestor guards, even when the
    // final value is pure. Signature discovery never grants their native effects.
    if !proof.calls.iter().any(|name| !pure.contains(name)) {
        return None;
    }
    let SelectedValue::Nested(plan) = proof.value else {
        return None;
    };
    Some(*plan)
}

pub(super) fn consume_node(budget: &mut Budget) -> Option<()> {
    budget.nodes = budget.nodes.checked_sub(1)?;
    Some(())
}

pub(super) fn inspect(
    expr: &NirExpr,
    scope: &Scope,
    signatures: &Signatures,
    budget: &mut Budget,
) -> Option<(NirTypeRef, BTreeSet<String>, BTreeSet<String>)> {
    inspect_at_depth(expr, scope, signatures, budget, 0)
}

pub(super) fn inspect_at_depth(
    expr: &NirExpr,
    scope: &Scope,
    signatures: &Signatures,
    budget: &mut Budget,
    depth: usize,
) -> Option<(NirTypeRef, BTreeSet<String>, BTreeSet<String>)> {
    let mut inputs = BTreeSet::new();
    let mut calls = BTreeSet::new();
    let ty = expression(
        expr,
        scope,
        signatures,
        &mut inputs,
        &mut calls,
        &mut budget.expressions,
        depth,
    )?;
    Some((ty, inputs, calls))
}

fn leaf(
    name: &str,
    declared: Option<&NirTypeRef>,
    value: &NirExpr,
    scope: &Scope,
    signatures: &Signatures,
    budget: &mut Budget,
) -> Option<Proof> {
    consume_node(budget)?;
    let old_ty = scope.get(name)?;
    if !scalar(old_ty) || declared.is_some_and(|ty| ty != old_ty) {
        return None;
    }
    let (ty, inputs, calls) = inspect(value, scope, signatures, budget)?;
    if &ty != old_ty {
        return None;
    }
    Some(Proof {
        value: SelectedValue::Expression(value.clone()),
        name: name.to_owned(),
        ty,
        inputs,
        calls,
    })
}

fn body(
    statements: &[NirStmt],
    scope: &Scope,
    signatures: &Signatures,
    budget: &mut Budget,
    depth: usize,
) -> Option<Proof> {
    match statements {
        statements if statements.len() > 1 => {
            if statements
                .iter()
                .any(|stmt| matches!(stmt, NirStmt::If { .. }))
            {
                ordered::prove(statements, scope, signatures, budget, depth)
            } else {
                regions::prove(statements, scope, signatures, budget)
            }
        }
        [NirStmt::Let { name, ty, value }] => {
            leaf(name, ty.as_ref(), value, scope, signatures, budget)
        }
        [NirStmt::If {
            condition,
            then_body,
            else_body,
        }] => branch(
            condition, then_body, else_body, scope, signatures, budget, depth,
        ),
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn branch(
    condition: &NirExpr,
    yes: &[NirStmt],
    no: &[NirStmt],
    scope: &Scope,
    signatures: &Signatures,
    budget: &mut Budget,
    depth: usize,
) -> Option<Proof> {
    if depth >= 8 {
        return None;
    }
    consume_node(budget)?;
    let (condition, condition_inputs, condition_calls) =
        predicates::prove(condition, scope, signatures, budget)?;
    let (yes, no) = if yes.is_empty() || no.is_empty() {
        let live = body(
            if yes.is_empty() { no } else { yes },
            scope,
            signatures,
            budget,
            depth + 1,
        )?;
        let retained = leaf(
            &live.name,
            Some(&live.ty),
            &NirExpr::Var(live.name.clone()),
            scope,
            signatures,
            budget,
        )?;
        if yes.is_empty() {
            (retained, live)
        } else {
            (live, retained)
        }
    } else {
        (
            body(yes, scope, signatures, budget, depth + 1)?,
            body(no, scope, signatures, budget, depth + 1)?,
        )
    };
    if yes.name != no.name || yes.ty != no.ty {
        return None;
    }
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
    let inputs = captures.union(&condition_inputs).cloned().collect();
    let calls = yes
        .calls
        .union(&no.calls)
        .cloned()
        .chain(condition_calls)
        .collect();
    let name = yes.name;
    let ty = yes.ty;
    Some(Proof {
        value: SelectedValue::Nested(Box::new(Plan {
            condition,
            yes: yes.value,
            no: no.value,
            destination: Some((name.clone(), false)),
            ty: ty.clone(),
            params,
        })),
        name,
        ty,
        inputs,
        calls,
    })
}
