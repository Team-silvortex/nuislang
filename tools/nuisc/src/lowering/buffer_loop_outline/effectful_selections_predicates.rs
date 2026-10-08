use super::*;

#[cfg(test)]
#[path = "effectful_selections_predicates_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "effectful_selections_computed_predicates_tests.rs"]
mod computed_tests;

#[cfg(test)]
#[path = "effectful_selections_logical_trees_tests.rs"]
mod tree_tests;

pub(super) enum Predicate {
    Scalar(NirExpr),
    Logical {
        left: Box<Predicate>,
        right: Box<Predicate>,
        selected: bool,
        params: Vec<NirParam>,
    },
}

pub(super) fn prove(
    condition: &NirExpr,
    scope: &Scope,
    signatures: &Signatures,
    budget: &mut nested::Budget,
) -> Option<(Predicate, BTreeSet<String>, BTreeSet<String>)> {
    prove_at_depth(condition, scope, signatures, budget, 0)
}

fn prove_at_depth(
    condition: &NirExpr,
    scope: &Scope,
    signatures: &Signatures,
    budget: &mut nested::Budget,
    depth: usize,
) -> Option<(Predicate, BTreeSet<String>, BTreeSet<String>)> {
    if depth >= 32 {
        return None;
    }
    let NirExpr::Binary {
        op: op @ (NirBinaryOp::And | NirBinaryOp::Or),
        lhs,
        rhs,
    } = condition
    else {
        let (ty, inputs, calls) =
            nested::inspect_at_depth(condition, scope, signatures, budget, depth)?;
        return (ty == scalar_type("bool"))
            .then(|| (Predicate::Scalar(condition.clone()), inputs, calls));
    };
    // All direct logical children share the original region's bounds. Logical
    // nodes hidden inside ordinary leaves still fail scalar inspection.
    budget.expressions = budget.expressions.checked_sub(1)?;
    budget.logical_edges = budget.logical_edges.checked_sub(1)?;
    let (left, left_inputs, left_calls) =
        prove_at_depth(lhs, scope, signatures, budget, depth + 1)?;
    let (right, right_inputs, mut calls) =
        prove_at_depth(rhs, scope, signatures, budget, depth + 1)?;
    if right_inputs.len() > 31 {
        return None;
    }
    let params = right_inputs
        .iter()
        .map(|name| {
            Some(NirParam {
                name: name.clone(),
                ty: scope.get(name)?.clone(),
            })
        })
        .collect::<Option<Vec<_>>>()?;
    calls.extend(left_calls);
    Some((
        Predicate::Logical {
            left: Box::new(left),
            right: Box::new(right),
            selected: *op == NirBinaryOp::And,
            params,
        },
        left_inputs.union(&right_inputs).cloned().collect(),
        calls,
    ))
}

pub(super) fn install(
    predicate: Predicate,
    names: &mut BTreeSet<String>,
    bindings: &mut BTreeSet<String>,
    helpers: &mut Vec<NirFunction>,
) -> NirExpr {
    let Predicate::Logical {
        left,
        right,
        selected,
        mut params,
    } = predicate
    else {
        let Predicate::Scalar(condition) = predicate else {
            unreachable!()
        };
        return condition;
    };
    let left = install(*left, names, bindings, helpers);
    let right = install(*right, names, bindings, helpers);
    let gate = branches::fresh_name("__nuis_effect_predicate_gate", bindings);
    let name = branches::fresh_name("__nuis_effect_call_predicate", names);
    let mut args = vec![left];
    args.extend(params.iter().map(|p| NirExpr::Var(p.name.clone())));
    params.insert(
        0,
        NirParam {
            name: gate.clone(),
            ty: scalar_type("bool"),
        },
    );
    // Guarded helpers prepare only typed zero seeds. Encode OR's selected
    // result as its complement, then invert the total call result outside.
    let complement = |value| NirExpr::Binary {
        op: NirBinaryOp::Eq,
        lhs: Box::new(value),
        rhs: Box::new(NirExpr::Bool(false)),
    };
    let result = if selected { right } else { complement(right) };
    // No RHS call or fallible argument appears at the eager call site. The
    // ancestor's guard runs first, then this helper's short-circuit guard.
    let mut function = helper(
        name.clone(),
        params,
        vec![
            NirStmt::If {
                condition: NirExpr::Binary {
                    op: NirBinaryOp::Ne,
                    lhs: Box::new(NirExpr::Var(gate)),
                    rhs: Box::new(NirExpr::Bool(selected)),
                },
                then_body: vec![NirStmt::Return(Some(NirExpr::Bool(false)))],
                else_body: vec![],
            },
            NirStmt::Return(Some(result)),
        ],
    );
    function.return_type = Some(scalar_type("bool"));
    helpers.push(function);
    let call = NirExpr::Call { callee: name, args };
    if selected {
        call
    } else {
        complement(call)
    }
}
