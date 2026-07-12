use super::packet_helpers::{find_packet_field, scalar_to_color_key};
use yir_core::StructValue;

pub(crate) struct BallPacketSceneRuntimeFields {
    pub(crate) visibility_cluster_slot: i64,
    pub(crate) visibility_visible_nodes: i64,
    pub(crate) visibility_occlusion_mode: i64,
    pub(crate) visibility_distance_band: i64,
    pub(crate) visibility_mask: i64,
    pub(crate) cull_cluster_slot: i64,
    pub(crate) cull_kept_nodes: i64,
    pub(crate) cull_mode: i64,
    pub(crate) cull_lod_band: i64,
    pub(crate) cull_mask: i64,
    pub(crate) lod_cluster_slot: i64,
    pub(crate) lod_level_count: i64,
    pub(crate) lod_active_level: i64,
    pub(crate) lod_switch_distance: i64,
    pub(crate) lod_bias: i64,
    pub(crate) streaming_cluster_slot: i64,
    pub(crate) streaming_resident_levels: i64,
    pub(crate) streaming_prefetch_mode: i64,
    pub(crate) streaming_evict_budget: i64,
    pub(crate) streaming_channel: i64,
    pub(crate) residency_cluster_slot: i64,
    pub(crate) residency_committed_levels: i64,
    pub(crate) residency_mode: i64,
    pub(crate) residency_spill_budget: i64,
    pub(crate) residency_mask: i64,
    pub(crate) eviction_cluster_slot: i64,
    pub(crate) eviction_levels: i64,
    pub(crate) eviction_mode: i64,
    pub(crate) eviction_reclaim_budget: i64,
    pub(crate) eviction_mask: i64,
    pub(crate) prefetch_cluster_slot: i64,
    pub(crate) prefetch_requested_levels: i64,
    pub(crate) prefetch_window: i64,
    pub(crate) prefetch_warm_budget: i64,
    pub(crate) prefetch_mask: i64,
    pub(crate) budget_cluster_slot: i64,
    pub(crate) budget_total: i64,
    pub(crate) budget_used: i64,
    pub(crate) budget_headroom: i64,
    pub(crate) budget_policy: i64,
    pub(crate) pressure_cluster_slot: i64,
    pub(crate) pressure_level: i64,
    pub(crate) pressure_saturation: i64,
    pub(crate) pressure_throttled: i64,
    pub(crate) pressure_mask: i64,
    pub(crate) thermal_cluster_slot: i64,
    pub(crate) thermal_level: i64,
    pub(crate) thermal_cooling_mode: i64,
    pub(crate) thermal_throttled: i64,
    pub(crate) thermal_mask: i64,
    pub(crate) power_cluster_slot: i64,
    pub(crate) power_level: i64,
    pub(crate) power_source_mode: i64,
    pub(crate) power_capped: i64,
    pub(crate) power_mask: i64,
    pub(crate) latency_cluster_slot: i64,
    pub(crate) latency_frame: i64,
    pub(crate) latency_input: i64,
    pub(crate) latency_jitter: i64,
    pub(crate) latency_mask: i64,
    pub(crate) frame_pacing_cluster_slot: i64,
    pub(crate) frame_pacing_cadence: i64,
    pub(crate) frame_pacing_variance: i64,
    pub(crate) frame_pacing_vsync_mode: i64,
    pub(crate) frame_pacing_mask: i64,
    pub(crate) frame_variance_cluster_slot: i64,
    pub(crate) frame_variance_frame: i64,
    pub(crate) frame_variance_input: i64,
    pub(crate) frame_variance_burst: i64,
    pub(crate) frame_variance_mask: i64,
    pub(crate) jank_cluster_slot: i64,
    pub(crate) jank_spikes: i64,
    pub(crate) jank_severity: i64,
    pub(crate) jank_recovery: i64,
    pub(crate) jank_mask: i64,
}

pub(crate) fn parse_ball_packet_scene_runtime(
    packet: &StructValue,
    op: &str,
    scene_cluster_instance_group_slot: i64,
    instance_group_visible_count: i64,
    scene_node_visibility: i64,
    instance_group_phase_bias: i64,
) -> Result<BallPacketSceneRuntimeFields, String> {
    let visibility_cluster_slot = field(
        packet,
        op,
        "scene_visibility",
        "cluster_slot",
        scene_cluster_instance_group_slot,
    )?;
    let visibility_visible_nodes = field(
        packet,
        op,
        "scene_visibility",
        "visible_nodes",
        instance_group_visible_count,
    )?;
    let visibility_occlusion_mode = field(
        packet,
        op,
        "scene_visibility",
        "occlusion_mode",
        scene_node_visibility,
    )?;
    let visibility_distance_band = field(
        packet,
        op,
        "scene_visibility",
        "distance_band",
        instance_group_phase_bias,
    )?;
    let visibility_mask = field(packet, op, "scene_visibility", "mask", 7)?;
    let cull_cluster_slot = field(
        packet,
        op,
        "scene_cull",
        "cluster_slot",
        visibility_cluster_slot,
    )?;
    let cull_kept_nodes = field(
        packet,
        op,
        "scene_cull",
        "kept_nodes",
        visibility_visible_nodes,
    )?;
    let cull_mode = field(
        packet,
        op,
        "scene_cull",
        "cull_mode",
        visibility_occlusion_mode,
    )?;
    let cull_lod_band = field(
        packet,
        op,
        "scene_cull",
        "lod_band",
        visibility_distance_band,
    )?;
    let cull_mask = field(packet, op, "scene_cull", "mask", visibility_mask)?;
    let lod_cluster_slot = field(packet, op, "scene_lod", "cluster_slot", cull_cluster_slot)?;
    let lod_level_count = field(packet, op, "scene_lod", "level_count", 4)?;
    let lod_active_level = field(packet, op, "scene_lod", "active_level", cull_mode)?;
    let lod_switch_distance = field(packet, op, "scene_lod", "switch_distance", cull_lod_band)?;
    let lod_bias = field(packet, op, "scene_lod", "bias", cull_mask)?;
    let streaming_cluster_slot = field(
        packet,
        op,
        "scene_streaming",
        "cluster_slot",
        lod_cluster_slot,
    )?;
    let streaming_resident_levels = field(packet, op, "scene_streaming", "resident_levels", 2)?;
    let streaming_prefetch_mode = field(
        packet,
        op,
        "scene_streaming",
        "prefetch_mode",
        lod_active_level,
    )?;
    let streaming_evict_budget = field(
        packet,
        op,
        "scene_streaming",
        "evict_budget",
        lod_switch_distance,
    )?;
    let streaming_channel = field(packet, op, "scene_streaming", "channel", lod_bias)?;
    let residency_cluster_slot = field(
        packet,
        op,
        "scene_residency",
        "cluster_slot",
        streaming_cluster_slot,
    )?;
    let residency_committed_levels = field(
        packet,
        op,
        "scene_residency",
        "committed_levels",
        streaming_resident_levels,
    )?;
    let residency_mode = field(
        packet,
        op,
        "scene_residency",
        "residency_mode",
        streaming_prefetch_mode,
    )?;
    let residency_spill_budget = field(
        packet,
        op,
        "scene_residency",
        "spill_budget",
        streaming_evict_budget,
    )?;
    let residency_mask = field(
        packet,
        op,
        "scene_residency",
        "residency_mask",
        streaming_channel,
    )?;
    let eviction_cluster_slot = field(
        packet,
        op,
        "scene_eviction",
        "cluster_slot",
        residency_cluster_slot,
    )?;
    let eviction_levels = field(
        packet,
        op,
        "scene_eviction",
        "evicted_levels",
        residency_mode,
    )?;
    let eviction_mode = field(
        packet,
        op,
        "scene_eviction",
        "eviction_mode",
        residency_mode,
    )?;
    let eviction_reclaim_budget = field(
        packet,
        op,
        "scene_eviction",
        "reclaim_budget",
        residency_spill_budget,
    )?;
    let eviction_mask = field(
        packet,
        op,
        "scene_eviction",
        "eviction_mask",
        residency_mask,
    )?;
    let prefetch_cluster_slot = field(
        packet,
        op,
        "scene_prefetch",
        "cluster_slot",
        eviction_cluster_slot,
    )?;
    let prefetch_requested_levels = field(
        packet,
        op,
        "scene_prefetch",
        "requested_levels",
        streaming_resident_levels,
    )?;
    let prefetch_window = field(
        packet,
        op,
        "scene_prefetch",
        "prefetch_window",
        streaming_prefetch_mode,
    )?;
    let prefetch_warm_budget = field(
        packet,
        op,
        "scene_prefetch",
        "warm_budget",
        eviction_reclaim_budget,
    )?;
    let prefetch_mask = field(packet, op, "scene_prefetch", "prefetch_mask", eviction_mask)?;
    let budget_cluster_slot = field(
        packet,
        op,
        "scene_budget",
        "cluster_slot",
        prefetch_cluster_slot,
    )?;
    let budget_total = field(packet, op, "scene_budget", "total_budget", 12)?;
    let budget_used = field(
        packet,
        op,
        "scene_budget",
        "used_budget",
        prefetch_warm_budget,
    )?;
    let budget_headroom = field(
        packet,
        op,
        "scene_budget",
        "headroom",
        prefetch_requested_levels,
    )?;
    let budget_policy = field(packet, op, "scene_budget", "budget_policy", prefetch_window)?;
    let pressure_cluster_slot = field(
        packet,
        op,
        "scene_pressure",
        "cluster_slot",
        budget_cluster_slot,
    )?;
    let pressure_level = field(packet, op, "scene_pressure", "pressure_level", 2)?;
    let pressure_saturation = field(packet, op, "scene_pressure", "saturation", budget_used)?;
    let pressure_throttled = field(packet, op, "scene_pressure", "throttled", budget_policy)?;
    let pressure_mask = field(
        packet,
        op,
        "scene_pressure",
        "pressure_mask",
        budget_headroom,
    )?;
    let thermal_cluster_slot = field(
        packet,
        op,
        "scene_thermal",
        "cluster_slot",
        pressure_cluster_slot,
    )?;
    let thermal_level = field(packet, op, "scene_thermal", "thermal_level", pressure_level)?;
    let thermal_cooling_mode = field(
        packet,
        op,
        "scene_thermal",
        "cooling_mode",
        pressure_throttled,
    )?;
    let thermal_throttled = field(packet, op, "scene_thermal", "throttled", pressure_throttled)?;
    let thermal_mask = field(packet, op, "scene_thermal", "thermal_mask", pressure_mask)?;
    let power_cluster_slot = field(
        packet,
        op,
        "scene_power",
        "cluster_slot",
        thermal_cluster_slot,
    )?;
    let power_level = field(packet, op, "scene_power", "power_level", thermal_level)?;
    let power_source_mode = field(
        packet,
        op,
        "scene_power",
        "source_mode",
        thermal_cooling_mode,
    )?;
    let power_capped = field(packet, op, "scene_power", "capped", thermal_throttled)?;
    let power_mask = field(packet, op, "scene_power", "power_mask", thermal_mask)?;
    let latency_cluster_slot = field(
        packet,
        op,
        "scene_latency",
        "cluster_slot",
        power_cluster_slot,
    )?;
    let latency_frame = field(packet, op, "scene_latency", "frame_latency", 4)?;
    let latency_input = field(packet, op, "scene_latency", "input_latency", 2)?;
    let latency_jitter = field(packet, op, "scene_latency", "jitter", power_capped)?;
    let latency_mask = field(packet, op, "scene_latency", "latency_mask", power_mask)?;
    let frame_pacing_cluster_slot = field(
        packet,
        op,
        "scene_frame_pacing",
        "cluster_slot",
        latency_cluster_slot,
    )?;
    let frame_pacing_cadence = field(packet, op, "scene_frame_pacing", "cadence", latency_frame)?;
    let frame_pacing_variance =
        field(packet, op, "scene_frame_pacing", "variance", latency_jitter)?;
    let frame_pacing_vsync_mode = field(
        packet,
        op,
        "scene_frame_pacing",
        "vsync_mode",
        latency_jitter,
    )?;
    let frame_pacing_mask = field(
        packet,
        op,
        "scene_frame_pacing",
        "pacing_mask",
        latency_mask,
    )?;
    let frame_variance_cluster_slot = field(
        packet,
        op,
        "scene_frame_variance",
        "cluster_slot",
        frame_pacing_cluster_slot,
    )?;
    let frame_variance_frame = field(
        packet,
        op,
        "scene_frame_variance",
        "frame_variance",
        frame_pacing_variance.max(1),
    )?;
    let frame_variance_input = field(
        packet,
        op,
        "scene_frame_variance",
        "input_variance",
        latency_input,
    )?;
    let frame_variance_burst = field(
        packet,
        op,
        "scene_frame_variance",
        "burst_mode",
        frame_pacing_cadence,
    )?;
    let frame_variance_mask = field(
        packet,
        op,
        "scene_frame_variance",
        "variance_mask",
        frame_pacing_mask,
    )?;
    let jank_cluster_slot = field(
        packet,
        op,
        "scene_jank",
        "cluster_slot",
        frame_variance_cluster_slot,
    )?;
    let jank_spikes = field(
        packet,
        op,
        "scene_jank",
        "spikes",
        1 + frame_variance_frame.rem_euclid(2),
    )?;
    let jank_severity = field(packet, op, "scene_jank", "severity", frame_variance_frame)?;
    let jank_recovery = field(packet, op, "scene_jank", "recovery", frame_variance_burst)?;
    let jank_mask = field(packet, op, "scene_jank", "jank_mask", frame_variance_mask)?;

    Ok(BallPacketSceneRuntimeFields {
        visibility_cluster_slot,
        visibility_visible_nodes,
        visibility_occlusion_mode,
        visibility_distance_band,
        visibility_mask,
        cull_cluster_slot,
        cull_kept_nodes,
        cull_mode,
        cull_lod_band,
        cull_mask,
        lod_cluster_slot,
        lod_level_count,
        lod_active_level,
        lod_switch_distance,
        lod_bias,
        streaming_cluster_slot,
        streaming_resident_levels,
        streaming_prefetch_mode,
        streaming_evict_budget,
        streaming_channel,
        residency_cluster_slot,
        residency_committed_levels,
        residency_mode,
        residency_spill_budget,
        residency_mask,
        eviction_cluster_slot,
        eviction_levels,
        eviction_mode,
        eviction_reclaim_budget,
        eviction_mask,
        prefetch_cluster_slot,
        prefetch_requested_levels,
        prefetch_window,
        prefetch_warm_budget,
        prefetch_mask,
        budget_cluster_slot,
        budget_total,
        budget_used,
        budget_headroom,
        budget_policy,
        pressure_cluster_slot,
        pressure_level,
        pressure_saturation,
        pressure_throttled,
        pressure_mask,
        thermal_cluster_slot,
        thermal_level,
        thermal_cooling_mode,
        thermal_throttled,
        thermal_mask,
        power_cluster_slot,
        power_level,
        power_source_mode,
        power_capped,
        power_mask,
        latency_cluster_slot,
        latency_frame,
        latency_input,
        latency_jitter,
        latency_mask,
        frame_pacing_cluster_slot,
        frame_pacing_cadence,
        frame_pacing_variance,
        frame_pacing_vsync_mode,
        frame_pacing_mask,
        frame_variance_cluster_slot,
        frame_variance_frame,
        frame_variance_input,
        frame_variance_burst,
        frame_variance_mask,
        jank_cluster_slot,
        jank_spikes,
        jank_severity,
        jank_recovery,
        jank_mask,
    })
}

fn field(
    packet: &StructValue,
    op: &str,
    group: &str,
    name: &str,
    default: i64,
) -> Result<i64, String> {
    find_packet_field(packet, &[name], &[group], &[name])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()
        .map(|value| value.unwrap_or(default))
}
