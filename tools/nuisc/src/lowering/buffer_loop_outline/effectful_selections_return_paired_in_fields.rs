use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn prove(
    body: &[NirStmt],
    prefix: &[NirStmt],
    condition: &NirExpr,
    yes: Exit<'_>,
    no: Exit<'_>,
    common: &[NirStmt],
    result: &NirExpr,
    scope: &Scope,
    signatures: &Signatures,
    budget: &mut nested::Budget,
    depth: usize,
) -> Option<ReturnedValue> {
    let mut staged = stage(prefix, scope, signatures, budget, depth)?;
    // These copies classify writes only; original slices supply all staged work.
    let clean = |arm: &Exit<'_>| {
        arm.before
            .iter()
            .chain(arm.after)
            .cloned()
            .collect::<Vec<_>>()
    };
    let fields =
        updates::private_field_certificate(&clean(&yes), &clean(&no), scope, &staged, depth)?;
    nested::consume_node(budget)?;
    let (source_condition, reads, calls) =
        predicates::prove(condition, &staged.local, signatures, budget)?;
    let mut bindings = staged.local.keys().cloned().collect();
    branches::collect_bindings(body, &mut bindings);
    let gate = branches::fresh_name("__nuis_effect_field_exit_gate", &mut bindings);
    staged
        .inputs
        .extend(reads.difference(&staged.defined).cloned());
    staged.calls.extend(calls);
    staged.local.insert(gate.clone(), scalar_type("bool"));
    staged.defined.insert(gate.clone());
    staged.constants.insert(gate.clone());
    staged.statements.push(regions::Statement::Logical(
        NirStmt::Let {
            name: gate.clone(),
            ty: Some(scalar_type("bool")),
            value: NirExpr::Bool(false),
        },
        source_condition,
    ));
    fields.split(
        &gate,
        yes.before,
        no.before,
        common,
        &mut staged,
        signatures,
        budget,
        depth,
    )?;
    let mut tail = stage_authorized(&[], &staged.local, signatures, budget, depth + 1, None)?;
    fields.split(
        &gate, yes.after, no.after, common, &mut tail, signatures, budget, depth,
    )?;
    fields.tail(
        common,
        &staged.local,
        &mut tail,
        signatures,
        budget,
        depth + 1,
    )?;
    nested::consume_node(budget)?;
    let (value, reads, calls) = predicates::prove(result, &tail.local, signatures, budget)?;
    tail.inputs.extend(reads.difference(&tail.defined).cloned());
    tail.calls.extend(calls);
    let mut returned = ReturnedValue {
        value: SelectedValue::StagedPredicate(tail.statements, value),
        inputs: tail.inputs,
        calls: tail.calls,
    };
    // Reverse construction keeps the original yes check outermost. Both checks
    // are lazy behind the immutable source gate; the one tail is never copied.
    for (selected, arm) in [(false, no), (true, yes)] {
        nested::consume_node(budget)?;
        let (predicate, reads, calls) =
            predicates::prove(arm.condition, &staged.local, signatures, budget)?;
        if reads.len() > 31 {
            return None;
        }
        let params = reads
            .iter()
            .map(|name| {
                Some(NirParam {
                    name: name.clone(),
                    ty: staged.local.get(name)?.clone(),
                })
            })
            .collect::<Option<Vec<_>>>()?;
        let gated = predicates::Predicate::Logical {
            left: Box::new(predicates::Predicate::Scalar(NirExpr::Binary {
                op: NirBinaryOp::Eq,
                lhs: Box::new(NirExpr::Var(gate.clone())),
                rhs: Box::new(NirExpr::Bool(selected)),
            })),
            right: Box::new(predicate),
            selected: true,
            params,
        };
        let early = exit_prefix::prove(&arm, &fields, &staged.local, signatures, budget, depth)?;
        let inputs = reads
            .iter()
            .chain(&early.inputs)
            .chain(&returned.inputs)
            .cloned()
            .chain(std::iter::once(gate.clone()))
            .collect();
        let mut all_calls = calls;
        all_calls.extend(early.calls.iter().cloned());
        all_calls.extend(returned.calls.iter().cloned());
        let plan = super::super::super::plan(gated, early, returned, &staged.local)?;
        returned = ReturnedValue {
            value: SelectedValue::Nested(Box::new(plan)),
            inputs,
            calls: all_calls,
        };
    }
    staged
        .inputs
        .extend(returned.inputs.difference(&staged.defined).cloned());
    staged.calls.extend(returned.calls);
    let SelectedValue::Nested(plan) = returned.value else {
        return None;
    };
    Some(ReturnedValue {
        value: SelectedValue::StagedNested(staged.statements, plan),
        inputs: staged.inputs,
        calls: staged.calls,
    })
}
