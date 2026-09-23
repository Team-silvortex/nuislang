use super::*;
use control_values::ValueLayouts;

pub(super) fn collect(
    module: &NirModule,
    generated: &BTreeSet<String>,
    layouts: &impl ValueLayouts,
) -> BTreeMap<String, direct_calls::CapturePlan> {
    // Scoped-call metadata supplies induction/carry arguments on every trip.
    // Even a generated value helper must retain that independent signature.
    let eligible = generated.iter().map(String::as_str).collect();
    let scoped = scoped_loop_lowering::collect_scoped_call_targets(module, &eligible);
    module
        .functions
        .iter()
        .filter(|function| generated.contains(&function.name) && !scoped.contains(&function.name))
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
            direct_calls::CapturePlan::new(leaves).map(|plan| (function.name.clone(), plan))
        })
        .collect()
}
