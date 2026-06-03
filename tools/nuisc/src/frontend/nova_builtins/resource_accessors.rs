use std::collections::BTreeMap;

use nuis_semantics::model::{AstExpr, NirExpr, NirStructDef, NirTypeRef};

use super::super::{lower_expr, named_type, FunctionSignature, ModuleConstValue};

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_nova_resource_accessor_builtin_call(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
    _current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    _module_consts: &BTreeMap<String, ModuleConstValue>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<Option<NirExpr>, String> {
    let Some((expected_type, field_name)) = resource_state_accessor_target(callee) else {
        return Ok(None);
    };
    let [state] = args else {
        return Err(format!("{callee}(...) expects 1 arg"));
    };
    let state = lower_expr(
        state,
        current_domain,
        bindings,
        signatures,
        struct_table,
        Some(&named_type(expected_type)),
    )?;
    Ok(Some(NirExpr::FieldAccess {
        base: Box::new(state),
        field: field_name.to_owned(),
    }))
}

fn resource_state_accessor_target(callee: &str) -> Option<(&'static str, &'static str)> {
    Some(match callee {
        "nova_visibility_state_cluster" => ("NovaVisibilityState", "cluster_slot"),
        "nova_visibility_state_visible" => ("NovaVisibilityState", "visible_nodes"),
        "nova_visibility_state_occlusion" => ("NovaVisibilityState", "occlusion_mode"),
        "nova_visibility_state_distance" => ("NovaVisibilityState", "distance_band"),
        "nova_visibility_state_mask" => ("NovaVisibilityState", "mask"),
        "nova_cull_state_cluster" => ("NovaCullState", "cluster_slot"),
        "nova_cull_state_kept" => ("NovaCullState", "kept_nodes"),
        "nova_cull_state_mode" => ("NovaCullState", "cull_mode"),
        "nova_cull_state_lod" => ("NovaCullState", "lod_band"),
        "nova_cull_state_mask" => ("NovaCullState", "mask"),
        "nova_lod_state_cluster" => ("NovaLodState", "cluster_slot"),
        "nova_lod_state_levels" => ("NovaLodState", "level_count"),
        "nova_lod_state_active" => ("NovaLodState", "active_level"),
        "nova_lod_state_switch_distance" => ("NovaLodState", "switch_distance"),
        "nova_lod_state_bias" => ("NovaLodState", "bias"),
        "nova_streaming_state_cluster" => ("NovaStreamingState", "cluster_slot"),
        "nova_streaming_state_resident" => ("NovaStreamingState", "resident_levels"),
        "nova_streaming_state_prefetch" => ("NovaStreamingState", "prefetch_mode"),
        "nova_streaming_state_evict_budget" => ("NovaStreamingState", "evict_budget"),
        "nova_streaming_state_channel" => ("NovaStreamingState", "channel"),
        "nova_residency_state_cluster" => ("NovaResidencyState", "cluster_slot"),
        "nova_residency_state_committed" => ("NovaResidencyState", "committed_levels"),
        "nova_residency_state_mode" => ("NovaResidencyState", "residency_mode"),
        "nova_residency_state_spill_budget" => ("NovaResidencyState", "spill_budget"),
        "nova_residency_state_mask" => ("NovaResidencyState", "residency_mask"),
        "nova_eviction_state_cluster" => ("NovaEvictionState", "cluster_slot"),
        "nova_eviction_state_evicted" => ("NovaEvictionState", "evicted_levels"),
        "nova_eviction_state_mode" => ("NovaEvictionState", "eviction_mode"),
        "nova_eviction_state_reclaim_budget" => ("NovaEvictionState", "reclaim_budget"),
        "nova_eviction_state_mask" => ("NovaEvictionState", "eviction_mask"),
        "nova_prefetch_state_cluster" => ("NovaPrefetchState", "cluster_slot"),
        "nova_prefetch_state_requested" => ("NovaPrefetchState", "requested_levels"),
        "nova_prefetch_state_window" => ("NovaPrefetchState", "prefetch_window"),
        "nova_prefetch_state_warm_budget" => ("NovaPrefetchState", "warm_budget"),
        "nova_prefetch_state_mask" => ("NovaPrefetchState", "prefetch_mask"),
        "nova_budget_state_cluster" => ("NovaBudgetState", "cluster_slot"),
        "nova_budget_state_total" => ("NovaBudgetState", "total_budget"),
        "nova_budget_state_used" => ("NovaBudgetState", "used_budget"),
        "nova_budget_state_headroom" => ("NovaBudgetState", "headroom"),
        "nova_budget_state_policy" => ("NovaBudgetState", "budget_policy"),
        "nova_pressure_state_cluster" => ("NovaPressureState", "cluster_slot"),
        "nova_pressure_state_level" => ("NovaPressureState", "pressure_level"),
        "nova_pressure_state_saturation" => ("NovaPressureState", "saturation"),
        "nova_pressure_state_throttled" => ("NovaPressureState", "throttled"),
        "nova_pressure_state_mask" => ("NovaPressureState", "pressure_mask"),
        "nova_thermal_state_cluster" => ("NovaThermalState", "cluster_slot"),
        "nova_thermal_state_level" => ("NovaThermalState", "thermal_level"),
        "nova_thermal_state_cooling" => ("NovaThermalState", "cooling_mode"),
        "nova_thermal_state_throttled" => ("NovaThermalState", "throttled"),
        "nova_thermal_state_mask" => ("NovaThermalState", "thermal_mask"),
        "nova_power_state_cluster" => ("NovaPowerState", "cluster_slot"),
        "nova_power_state_level" => ("NovaPowerState", "power_level"),
        "nova_power_state_source" => ("NovaPowerState", "source_mode"),
        "nova_power_state_capped" => ("NovaPowerState", "capped"),
        "nova_power_state_mask" => ("NovaPowerState", "power_mask"),
        "nova_latency_state_cluster" => ("NovaLatencyState", "cluster_slot"),
        "nova_latency_state_frame" => ("NovaLatencyState", "frame_latency"),
        "nova_latency_state_input" => ("NovaLatencyState", "input_latency"),
        "nova_latency_state_jitter" => ("NovaLatencyState", "jitter"),
        "nova_latency_state_mask" => ("NovaLatencyState", "latency_mask"),
        "nova_frame_pacing_state_cluster" => ("NovaFramePacingState", "cluster_slot"),
        "nova_frame_pacing_state_cadence" => ("NovaFramePacingState", "cadence"),
        "nova_frame_pacing_state_variance" => ("NovaFramePacingState", "variance"),
        "nova_frame_pacing_state_vsync" => ("NovaFramePacingState", "vsync_mode"),
        "nova_frame_pacing_state_mask" => ("NovaFramePacingState", "pacing_mask"),
        "nova_jank_state_cluster" => ("NovaJankState", "cluster_slot"),
        "nova_jank_state_spikes" => ("NovaJankState", "spikes"),
        "nova_jank_state_severity" => ("NovaJankState", "severity"),
        "nova_jank_state_recovery" => ("NovaJankState", "recovery"),
        "nova_jank_state_mask" => ("NovaJankState", "jank_mask"),
        "nova_frame_variance_state_cluster" => ("NovaFrameVarianceState", "cluster_slot"),
        "nova_frame_variance_state_frame" => ("NovaFrameVarianceState", "frame_variance"),
        "nova_frame_variance_state_input" => ("NovaFrameVarianceState", "input_variance"),
        "nova_frame_variance_state_burst" => ("NovaFrameVarianceState", "burst_mode"),
        "nova_frame_variance_state_mask" => ("NovaFrameVarianceState", "variance_mask"),
        _ => return None,
    })
}
