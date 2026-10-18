use super::*;

pub(super) fn validate_body(
    function: &NirFunction,
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
    loop_layouts: Option<&control_values::CarryLayouts>,
    profile: Profile,
) -> Option<()> {
    let normalized = if let Some(loop_layouts) = loop_layouts {
        match profile {
            Profile::Pure => control_loops::returns::normalize(function, loop_layouts)?,
            Profile::IterationLiteralPrints => {
                control_loops::returns::normalize_effects(function, loop_layouts)?
            }
        }
    } else {
        None
    };
    validate_profile(
        function,
        normalized.as_deref().unwrap_or(&function.body),
        catalog,
        layouts,
        loop_layouts,
        profile,
    )
}

pub(in crate::lowering::buffer_loop_outline) fn validate_control_body(
    function: &NirFunction,
    body: &[NirStmt],
    catalog: &ScalarHelpers,
    layouts: &control_values::CarryLayouts,
) -> Option<()> {
    validate_profile(
        function,
        body,
        catalog,
        layouts,
        Some(layouts),
        Profile::Pure,
    )
}

pub(in crate::lowering::buffer_loop_outline) fn validate_iteration_body(
    function: &NirFunction,
    body: &[NirStmt],
    catalog: &ScalarHelpers,
    layouts: &control_values::CarryLayouts,
) -> Option<()> {
    validate_profile(
        function,
        body,
        catalog,
        layouts,
        Some(layouts),
        Profile::IterationLiteralPrints,
    )
}

fn validate_profile(
    function: &NirFunction,
    body: &[NirStmt],
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
    loop_layouts: Option<&control_values::CarryLayouts>,
    profile: Profile,
) -> Option<()> {
    let mut locals = function
        .params
        .iter()
        .map(|param| (param.name.clone(), param.ty.clone()))
        .collect();
    validate_block(
        body,
        &mut locals,
        &mut BTreeSet::new(),
        function.return_type.as_ref()?,
        catalog,
        layouts,
        loop_layouts,
        profile,
    )?
    .then_some(())
}
