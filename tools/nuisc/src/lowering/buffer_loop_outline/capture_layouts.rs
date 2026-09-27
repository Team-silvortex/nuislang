use super::*;
use control_values::ValueLayouts;

pub(super) fn collect(
    module: &NirModule,
    generated: &BTreeSet<String>,
    layouts: &impl ValueLayouts,
) -> BTreeMap<String, direct_calls::CapturePlan> {
    // Scoped record transport needs a complete, agreed seed map at every caller.
    let eligible = generated.iter().map(String::as_str).collect();
    let scoped = scoped_loop_lowering::collect_scoped_call_targets(module, &eligible);
    let scoped_inputs = capture_projection::scoped_record_seeds(module, &scoped);
    module
        .functions
        .iter()
        .filter(|function| generated.contains(&function.name))
        .filter_map(|function| {
            let mut pending = function.params.iter().rev().cloned().collect::<Vec<_>>();
            let mut leaves = Vec::new();
            while let Some(param) = pending.pop() {
                if !control_values::supported_type(&param.ty, layouts) {
                    return None;
                }
                if layouts.scalar(&param.ty.name) {
                    leaves.push(param);
                } else {
                    let fields = layouts.fields(&param.ty.name)?.collect::<Vec<_>>();
                    pending.extend(fields.into_iter().rev().map(|(field, ty)| NirParam {
                        name: format!("{}.{field}", param.name),
                        ty,
                    }));
                }
            }
            direct_calls::CapturePlan::for_generated(leaves, function, module)
                .filter(|plan| {
                    !scoped.contains(&function.name)
                        || scoped_inputs
                            .get(&function.name)
                            .is_some_and(|seeds| plan.supports_scoped(function, seeds))
                })
                .map(|plan| (function.name.clone(), plan))
        })
        .collect()
}
