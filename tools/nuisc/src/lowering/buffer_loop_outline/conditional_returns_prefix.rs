use super::*;

#[cfg(test)]
#[path = "conditional_returns_prefix_tests.rs"]
mod tests;

pub(super) struct Arm {
    pub(super) body: Vec<NirStmt>,
    pub(super) inputs: BTreeSet<String>,
}

pub(super) fn prepare(
    body: &[NirStmt],
    result: &NirTypeRef,
    scope: &Scope,
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
) -> Option<Arm> {
    // Bound all value roots before recursive inference or cloning. Fresh bool
    // bindings and the return share the existing guarded single-edge route.
    if !conditional_values::prefix::return_body(body) {
        return None;
    }
    let (returned, locals) = body.split_last()?;
    let NirStmt::Return(Some(value)) = returned else {
        return None;
    };
    let mut inner = scope.clone();
    let mut local_names = BTreeSet::new();
    let mut inputs = BTreeSet::new();
    for stmt in locals {
        let (name, declared, value) = match stmt {
            NirStmt::Let { name, ty, value } => (name, ty.as_ref(), value),
            NirStmt::Const { name, ty, value } => (name, Some(ty), value),
            _ => return None,
        };
        // Fresh locals cannot hide a capture, mutate a parent carry or reuse
        // another prefix binding. Earlier values are the only local inputs.
        if inner.contains_key(name) {
            return None;
        }
        let ty = control_values::value_type(value, &inner, catalog, layouts)?;
        if declared.is_some_and(|declared| declared != &ty) {
            return None;
        }
        control_values::collect_inputs(value, &mut inputs);
        local_names.insert(name.clone());
        inner.insert(name.clone(), ty);
    }
    if !expression(value, &inner)
        || control_values::value_type(value, &inner, catalog, layouts).as_ref() != Some(result)
    {
        return None;
    }
    control_values::collect_inputs(value, &mut inputs);
    inputs.retain(|name| !local_names.contains(name));
    if inputs
        .iter()
        .any(|name| !scope.get(name).is_some_and(scalar))
    {
        return None;
    }
    Some(Arm {
        body: body.to_vec(),
        inputs,
    })
}
