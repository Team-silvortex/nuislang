use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn prepare(
    condition: &NirExpr,
    yes: &Arm,
    no: &Arm,
    result: &NirTypeRef,
    scope: &Scope,
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
    checked: &BTreeSet<String>,
) -> Option<Plan> {
    // A continuation slot exists only after original, exact source exits in
    // the effect prefix have proved admission. Tail seeds confer no such right.
    if yes.continuation.is_none() && no.continuation.is_none() {
        return None;
    }
    let mut inputs = BTreeSet::new();
    let mut bodies = Vec::new();
    for arm in [yes, no] {
        let body = suffix::prepare(&arm.tail, result, scope, catalog, layouts)
            .unwrap_or_else(|| arm.tail.clone());
        let proof = signal::prepare(&body, result, scope, catalog, layouts)?;
        if proof.has_return {
            return None;
        }
        inputs.extend(proof.inputs);
        bodies.push(body);
    }
    if !entry::has_work(condition)
        && ![yes, no].into_iter().any(|arm| arm.has_initializer_work)
        && !bodies.iter().any(|body| {
            speculation::block_has_checked_arithmetic(body, checked)
                || scalar_helpers::contains_calls(body)
        })
    {
        return None;
    }
    let no = bodies.pop()?;
    let yes = bodies.pop()?;
    Some(Plan {
        condition: condition.clone(),
        yes: Some(yes),
        no: Some(no),
        params: captured_params(inputs, scope),
        readiness: None,
        stored_exit: true,
    })
}
