use super::*;

pub(super) fn lower_data_profile_ref_expr(
    expr: &NirExpr,
    state: &mut LoweringState<'_>,
) -> Option<Result<String, String>> {
    match expr {
        NirExpr::DataProfileBindCoreRef { unit } => {
            Some(lower_project_profile_ref(state, "data", unit, "bind_core"))
        }
        NirExpr::DataProfileWindowOffsetRef { unit } => Some(lower_project_profile_ref(
            state,
            "data",
            unit,
            "window_offset",
        )),
        NirExpr::DataProfileUplinkLenRef { unit } => {
            Some(lower_project_profile_ref(state, "data", unit, "uplink_len"))
        }
        NirExpr::DataProfileDownlinkLenRef { unit } => Some(lower_project_profile_ref(
            state,
            "data",
            unit,
            "downlink_len",
        )),
        NirExpr::DataProfileHandleTableRef { unit } => Some(lower_project_profile_ref(
            state,
            "data",
            unit,
            "handle_table",
        )),
        NirExpr::DataProfileMarkerRef { unit, tag } => Some(lower_project_profile_ref(
            state,
            "data",
            unit,
            &format!("marker:{tag}"),
        )),
        _ => None,
    }
}
