use super::*;

// Share completed source proofs without exporting the provisional loop catalog.
#[derive(Default)]
pub(in crate::lowering) struct PureValueCalls {
    catalog: ScalarHelpers,
    layouts: control_values::CarryLayouts,
}

impl PureValueCalls {
    pub(in crate::lowering) fn collect(module: &NirModule) -> Self {
        let layouts = control_values::layouts(module);
        let catalog = scalar_helpers::collect_capture_values(module, &layouts);
        Self { catalog, layouts }
    }

    pub(in crate::lowering) fn call_type(
        &self,
        callee: &str,
        args: &[NirTypeRef],
    ) -> Option<NirTypeRef> {
        if self.catalog.get(callee)?.may_loop {
            return None;
        }
        scalar_helpers::typed_call_type(callee, args, &self.catalog)
    }

    pub(in crate::lowering) fn field_type(
        &self,
        base: &NirTypeRef,
        field: &str,
    ) -> Option<NirTypeRef> {
        if base != &scalar_type(&base.name) {
            return None;
        }
        self.layouts
            .get(&base.name)?
            .iter()
            .find(|(name, _)| name == field)
            .map(|(_, ty)| ty.clone())
    }
}
