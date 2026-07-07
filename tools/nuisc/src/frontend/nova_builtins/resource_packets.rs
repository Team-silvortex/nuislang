use nuis_semantics::model::NirExpr;

use super::packet_helpers::{build_struct_literal, lower_i64_arg_list};
use super::NovaBuiltinInput;

pub(super) fn lower_nova_resource_packet_builtin_call(
    input: NovaBuiltinInput<'_>,
) -> Result<Option<NirExpr>, String> {
    let NovaBuiltinInput {
        callee,
        args,
        current_domain,
        bindings,
        signatures,
        struct_table,
        ..
    } = input;
    let (type_name, fields) = match callee {
        "nova_visibility_packet" => (
            "NovaVisibilityPacket",
            &[
                "cluster_slot",
                "visible_nodes",
                "occlusion_mode",
                "distance_band",
                "mask",
            ][..],
        ),
        "nova_cull_packet" => (
            "NovaCullPacket",
            &[
                "cluster_slot",
                "kept_nodes",
                "cull_mode",
                "lod_band",
                "mask",
            ][..],
        ),
        "nova_lod_packet" => (
            "NovaLodPacket",
            &[
                "cluster_slot",
                "active_levels",
                "lod_policy",
                "streaming_bias",
                "lod_mask",
            ][..],
        ),
        "nova_streaming_packet" => (
            "NovaStreamingPacket",
            &[
                "cluster_slot",
                "resident_levels",
                "streaming_mode",
                "backpressure",
                "streaming_mask",
            ][..],
        ),
        "nova_residency_packet" => (
            "NovaResidencyPacket",
            &[
                "cluster_slot",
                "resident_pages",
                "evicted_pages",
                "warm_pages",
                "residency_mask",
            ][..],
        ),
        "nova_eviction_packet" => (
            "NovaEvictionPacket",
            &[
                "cluster_slot",
                "evicted_levels",
                "eviction_mode",
                "reclaim_budget",
                "eviction_mask",
            ][..],
        ),
        "nova_prefetch_packet" => (
            "NovaPrefetchPacket",
            &[
                "cluster_slot",
                "requested_levels",
                "prefetch_window",
                "warm_budget",
                "prefetch_mask",
            ][..],
        ),
        "nova_budget_packet" => (
            "NovaBudgetPacket",
            &[
                "cluster_slot",
                "total_budget",
                "used_budget",
                "headroom",
                "budget_policy",
            ][..],
        ),
        "nova_pressure_packet" => (
            "NovaPressurePacket",
            &[
                "cluster_slot",
                "pressure_level",
                "saturation",
                "throttled",
                "pressure_mask",
            ][..],
        ),
        "nova_thermal_packet" => (
            "NovaThermalPacket",
            &[
                "cluster_slot",
                "thermal_level",
                "cooling_mode",
                "throttled",
                "thermal_mask",
            ][..],
        ),
        "nova_power_packet" => (
            "NovaPowerPacket",
            &[
                "cluster_slot",
                "power_level",
                "source_mode",
                "capped",
                "power_mask",
            ][..],
        ),
        "nova_latency_packet" => (
            "NovaLatencyPacket",
            &[
                "cluster_slot",
                "frame_latency",
                "input_latency",
                "jitter",
                "latency_mask",
            ][..],
        ),
        "nova_frame_pacing_packet" => (
            "NovaFramePacingPacket",
            &[
                "cluster_slot",
                "cadence",
                "variance",
                "vsync_mode",
                "pacing_mask",
            ][..],
        ),
        "nova_jank_packet" => (
            "NovaJankPacket",
            &[
                "cluster_slot",
                "spikes",
                "severity",
                "recovery",
                "jank_mask",
            ][..],
        ),
        "nova_frame_variance_packet" => (
            "NovaFrameVariancePacket",
            &[
                "cluster_slot",
                "frame_variance",
                "input_variance",
                "burst_mode",
                "variance_mask",
            ][..],
        ),
        _ => return Ok(None),
    };

    let values = lower_i64_arg_list(
        args,
        5,
        &format!("{callee}(...) expects 5 args"),
        current_domain,
        bindings,
        signatures,
        struct_table,
    )?;
    Ok(Some(build_struct_literal(type_name, fields, values)))
}
