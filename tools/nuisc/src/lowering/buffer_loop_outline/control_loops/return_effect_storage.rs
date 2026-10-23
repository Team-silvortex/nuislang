use super::*;

#[cfg(test)]
#[path = "return_effect_storage_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "return_effect_storage_execution_tests.rs"]
mod execution_tests;

#[cfg(test)]
#[path = "return_effect_storage_prefix_tests.rs"]
mod prefix_tests;

#[cfg(test)]
#[path = "return_effect_storage_candidates_tests.rs"]
mod candidates_tests;

#[cfg(test)]
#[path = "return_effect_storage_reassignment_tests.rs"]
mod reassignment_tests;

#[cfg(test)]
#[path = "return_effect_storage_helper_call_tests.rs"]
mod helper_call_tests;

#[cfg(test)]
#[path = "return_effect_storage_loop_call_tests.rs"]
mod loop_call_tests;

#[cfg(test)]
#[path = "return_effect_storage_branch_call_tests.rs"]
mod branch_call_tests;

#[cfg(test)]
#[path = "return_effect_storage_bound_call_tests.rs"]
mod bound_call_tests;

#[cfg(test)]
#[path = "return_effect_storage_sibling_bound_tests.rs"]
mod sibling_bound_tests;

#[cfg(test)]
#[path = "return_effect_storage_preheader_bound_tests.rs"]
mod preheader_bound_tests;

#[cfg(test)]
#[path = "return_effect_storage_epoch_bound_tests.rs"]
mod epoch_bound_tests;

#[cfg(test)]
#[path = "return_effect_storage_snapshot_return_tests.rs"]
mod snapshot_return_tests;

#[cfg(test)]
#[path = "return_effect_storage_payload_call_tests.rs"]
mod payload_call_tests;

// A distinct certificate: independent-prefix validity alone cannot authorize
// overwriting a source binding. Both bodies must retain their own read sets.
pub(in super::super) fn verify(
    function: &NirFunction,
    independent: &Prepared,
    shared: &Prepared,
    storage: &str,
    layouts: &control_values::CarryLayouts,
    catalog: &ScalarHelpers,
) -> Option<()> {
    verify_independent(function, independent, layouts)?;
    scalar_helpers::validate_iteration_body(function, &independent.body, catalog, layouts)?;
    let mut budget = bounds::Budget::new();
    let mut names = function.params.iter().map(|p| p.name.clone()).collect();
    bounds::preflight(&function.body, &mut budget, Some(&mut names))?;
    bounds::preflight(&shared.body, &mut budget, None)?;
    eligible(function, storage, layouts, catalog)?;
    let [flag, tail @ ..] = shared.body.as_slice() else {
        return None;
    };
    if !shared.structs.is_empty()
        || shared.signal.name() != independent.signal.name()
        || names.contains(shared.signal.name())
        || !assigned(
            flag,
            shared.signal.name(),
            &scalar_type("i64"),
            &NirExpr::Int(0),
        )
    {
        return None;
    }
    Proof {
        flag: shared.signal.name(),
        payload: storage,
        result: function.return_type.as_ref()?,
    }
    .block(&function.body, tail, false)?
    .then_some(())?;
    // Reusing the record can introduce writes in a formerly read-only inner
    // loop. Do not borrow the independent body's ordered-read permission.
    scalar_helpers::validate_iteration_body(function, &shared.body, catalog, layouts)
}

fn eligible(
    function: &NirFunction,
    storage: &str,
    layouts: &control_values::CarryLayouts,
    catalog: &ScalarHelpers,
) -> Option<()> {
    let result = function.return_type.as_ref()?;
    layouts.get(&result.name)?;
    let mut scope = function
        .params
        .iter()
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect::<Scope>();
    if scope.contains_key(storage) {
        return None;
    }
    let mut initialized = false;
    for stmt in &function.body {
        let (name, declared, value, mutable) = match stmt {
            NirStmt::Let { name, ty, value } => (name, ty.as_ref(), value, true),
            NirStmt::Const { name, ty, value } => (name, Some(ty), value, false),
            // Literal prints neither define names nor change dominance. This
            // independent scan does not trust the producer's candidate search.
            NirStmt::Print(NirExpr::Int(_)) => continue,
            _ => break,
        };
        let ty = control_values::value_type(value, &scope, catalog, layouts)?;
        if declared.is_some_and(|declared| declared != &ty) {
            return None;
        }
        if name == storage {
            if !mutable || ty != *result {
                return None;
            }
            // NIR uses Let for both definition and reassignment. Only a fresh
            // first definition can establish storage; later writes keep its type.
            match scope.get(name) {
                None if !initialized => initialized = true,
                Some(previous) if initialized && previous == result => {}
                _ => return None,
            }
        }
        scope.insert(name.clone(), ty);
    }
    if !initialized {
        return None;
    }
    let mut blocks = vec![(function.body.as_slice(), false)];
    while let Some((body, in_loop)) = blocks.pop() {
        for stmt in body {
            match stmt {
                NirStmt::Let { name, .. } if in_loop && name == storage => return Some(()),
                NirStmt::While { body, .. } => blocks.push((body, true)),
                NirStmt::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    blocks.extend([
                        (then_body.as_slice(), in_loop),
                        (else_body.as_slice(), in_loop),
                    ]);
                }
                _ => {}
            }
        }
    }
    None
}
