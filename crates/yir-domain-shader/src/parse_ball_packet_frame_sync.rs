use super::packet_helpers::{find_packet_field, scalar_to_color_key};
use yir_core::{StructValue, Value};

pub(crate) struct BallPacketFrameSyncFields {
    pub(crate) pass_stage: i64,
    pub(crate) pass_clear_mode: i64,
    pub(crate) pass_sample_count: i64,
    pub(crate) pass_debug_view: i64,
    pub(crate) frame_index: i64,
    pub(crate) frame_present_mode: i64,
    pub(crate) frame_sync_interval: i64,
    pub(crate) frame_exposure: i64,
    pub(crate) target_kind: i64,
    pub(crate) target_width: i64,
    pub(crate) target_height: i64,
    pub(crate) target_multisample: i64,
    pub(crate) frame_graph_passes: i64,
    pub(crate) frame_graph_targets: i64,
    pub(crate) frame_graph_present_stage: i64,
    pub(crate) frame_graph_debug_overlay: i64,
    pub(crate) attachment_slot: i64,
    pub(crate) attachment_format_kind: i64,
    pub(crate) attachment_load_op: i64,
    pub(crate) attachment_store_op: i64,
    pub(crate) pass_chain_stages: i64,
    pub(crate) pass_chain_fanout: i64,
    pub(crate) pass_chain_resolve_stage: i64,
    pub(crate) pass_chain_barrier_mode: i64,
    pub(crate) barrier_scope: i64,
    pub(crate) barrier_source_stage: i64,
    pub(crate) barrier_target_stage: i64,
    pub(crate) barrier_flush_mode: i64,
    pub(crate) resource_buffers: i64,
    pub(crate) resource_textures: i64,
    pub(crate) resource_samplers: i64,
    pub(crate) resource_residency: i64,
    pub(crate) schedule_lanes: i64,
    pub(crate) schedule_queue_depth: i64,
    pub(crate) schedule_async_budget: i64,
    pub(crate) schedule_tick_mode: i64,
    pub(crate) submission_batches: i64,
    pub(crate) submission_fences: i64,
    pub(crate) submission_signal_mode: i64,
    pub(crate) submission_present_hint: i64,
    pub(crate) queue_kind: i64,
    pub(crate) queue_priority: i64,
    pub(crate) queue_budget: i64,
    pub(crate) queue_ownership: i64,
    pub(crate) semaphore_wait_count: i64,
    pub(crate) semaphore_signal_count: i64,
    pub(crate) semaphore_timeline_mode: i64,
    pub(crate) semaphore_scope: i64,
    pub(crate) timeline_value: i64,
    pub(crate) timeline_step: i64,
    pub(crate) timeline_epoch: i64,
    pub(crate) timeline_domain: i64,
    pub(crate) fence_signaled: i64,
    pub(crate) fence_epoch: i64,
    pub(crate) fence_scope: i64,
    pub(crate) fence_recycle_mode: i64,
    pub(crate) signal_kind: i64,
    pub(crate) signal_phase: i64,
    pub(crate) signal_fanout: i64,
    pub(crate) signal_ack_mode: i64,
    pub(crate) event_kind: i64,
    pub(crate) event_route: i64,
    pub(crate) event_priority: i64,
    pub(crate) event_payload_mode: i64,
    pub(crate) dispatch_queue_kind: i64,
    pub(crate) dispatch_lane: i64,
    pub(crate) dispatch_batch: i64,
    pub(crate) dispatch_completion_mode: i64,
}

pub(crate) fn parse_ball_packet_frame_sync(
    packet: &StructValue,
    op: &str,
    radius_scale: f32,
    accent: i64,
    contrast: i64,
    speed: &Value,
) -> Result<BallPacketFrameSyncFields, String> {
    let pass_stage = packet_i64(
        packet,
        op,
        &["stage"],
        "pass",
        "stage",
        contrast.rem_euclid(3),
    )?;
    let pass_clear_mode = packet_i64(packet, op, &["clear_mode"], "pass", "clear_mode", accent)?;
    let pass_sample_count = packet_i64(packet, op, &["sample_count"], "pass", "sample_count", 4)?;
    let pass_debug_view = packet_i64(
        packet,
        op,
        &["debug_view"],
        "pass",
        "debug_view",
        accent.rem_euclid(6),
    )?;
    let frame_index =
        packet_i64_with(packet, op, &["frame_index"], "frame", "frame_index", || {
            scalar_to_color_key(speed, op).unwrap_or(0)
        })?;
    let frame_present_mode = packet_i64(
        packet,
        op,
        &["present_mode"],
        "frame",
        "present_mode",
        accent.rem_euclid(3),
    )?;
    let frame_sync_interval =
        packet_i64(packet, op, &["sync_interval"], "frame", "sync_interval", 1)?;
    let frame_exposure = packet_i64(
        packet,
        op,
        &["exposure"],
        "frame",
        "exposure",
        scaled(radius_scale, 24.0),
    )?;
    let target_kind = packet_i64(
        packet,
        op,
        &["kind"],
        "target",
        "kind",
        accent.rem_euclid(3),
    )?;
    let target_width = packet_i64(packet, op, &["width"], "target", "width", 48)?;
    let target_height = packet_i64(packet, op, &["height"], "target", "height", 18)?;
    let target_multisample = packet_i64(
        packet,
        op,
        &["multisample"],
        "target",
        "multisample",
        accent,
    )?;
    let frame_graph_passes = packet_i64(packet, op, &["passes"], "frame_graph", "passes", 2)?;
    let frame_graph_targets = packet_i64(packet, op, &["targets"], "frame_graph", "targets", 1)?;
    let frame_graph_present_stage = packet_i64(
        packet,
        op,
        &["present_stage"],
        "frame_graph",
        "present_stage",
        contrast.rem_euclid(3),
    )?;
    let frame_graph_debug_overlay = packet_i64(
        packet,
        op,
        &["debug_overlay"],
        "frame_graph",
        "debug_overlay",
        accent.rem_euclid(6),
    )?;
    let attachment_slot = packet_i64(packet, op, &["slot"], "attachment", "slot", 0)?;
    let attachment_format_kind = packet_i64(
        packet,
        op,
        &["format_kind"],
        "attachment",
        "format_kind",
        accent,
    )?;
    let attachment_load_op = packet_i64(
        packet,
        op,
        &["load_op"],
        "attachment",
        "load_op",
        contrast.rem_euclid(3),
    )?;
    let attachment_store_op = packet_i64(packet, op, &["store_op"], "attachment", "store_op", 1)?;
    let pass_chain_stages = packet_i64(packet, op, &["stages"], "pass_chain", "stages", 2)?;
    let pass_chain_fanout = packet_i64(packet, op, &["fanout"], "pass_chain", "fanout", 1)?;
    let pass_chain_resolve_stage = packet_i64(
        packet,
        op,
        &["resolve_stage"],
        "pass_chain",
        "resolve_stage",
        contrast.rem_euclid(3),
    )?;
    let pass_chain_barrier_mode = packet_i64(
        packet,
        op,
        &["barrier_mode"],
        "pass_chain",
        "barrier_mode",
        accent,
    )?;
    let barrier_scope = packet_i64(packet, op, &["scope"], "barrier", "scope", 1)?;
    let barrier_source_stage = packet_i64(
        packet,
        op,
        &["source_stage"],
        "barrier",
        "source_stage",
        contrast.rem_euclid(3),
    )?;
    let barrier_target_stage =
        packet_i64(packet, op, &["target_stage"], "barrier", "target_stage", 2)?;
    let barrier_flush_mode =
        packet_i64(packet, op, &["flush_mode"], "barrier", "flush_mode", accent)?;
    let resource_buffers = packet_i64(packet, op, &["buffers"], "resource_set", "buffers", 2)?;
    let resource_textures = packet_i64(packet, op, &["textures"], "resource_set", "textures", 1)?;
    let resource_samplers = packet_i64(packet, op, &["samplers"], "resource_set", "samplers", 1)?;
    let resource_residency = packet_i64(
        packet,
        op,
        &["residency"],
        "resource_set",
        "residency",
        accent,
    )?;
    let schedule_lanes = packet_i64(packet, op, &["lanes"], "schedule", "lanes", 2)?;
    let schedule_queue_depth =
        packet_i64(packet, op, &["queue_depth"], "schedule", "queue_depth", 4)?;
    let schedule_async_budget = packet_i64(
        packet,
        op,
        &["async_budget"],
        "schedule",
        "async_budget",
        scaled(radius_scale, 24.0),
    )?;
    let schedule_tick_mode = packet_i64(
        packet,
        op,
        &["tick_mode"],
        "schedule",
        "tick_mode",
        contrast.rem_euclid(3),
    )?;
    let submission_batches = packet_i64(packet, op, &["batches"], "submission", "batches", 2)?;
    let submission_fences = packet_i64(packet, op, &["fences"], "submission", "fences", 1)?;
    let submission_signal_mode = packet_i64(
        packet,
        op,
        &["signal_mode"],
        "submission",
        "signal_mode",
        contrast.rem_euclid(3),
    )?;
    let submission_present_hint = packet_i64(
        packet,
        op,
        &["present_hint"],
        "submission",
        "present_hint",
        accent,
    )?;
    let queue_kind = packet_i64(
        packet,
        op,
        &["kind"],
        "queue",
        "kind",
        contrast.rem_euclid(3),
    )?;
    let queue_priority = packet_i64(packet, op, &["priority"], "queue", "priority", 2)?;
    let queue_budget = packet_i64(
        packet,
        op,
        &["budget"],
        "queue",
        "budget",
        scaled(radius_scale, 24.0),
    )?;
    let queue_ownership = packet_i64(packet, op, &["ownership"], "queue", "ownership", accent)?;
    let semaphore_wait_count =
        packet_i64(packet, op, &["wait_count"], "semaphore", "wait_count", 1)?;
    let semaphore_signal_count = packet_i64(
        packet,
        op,
        &["signal_count"],
        "semaphore",
        "signal_count",
        2,
    )?;
    let semaphore_timeline_mode = packet_i64(
        packet,
        op,
        &["timeline_mode"],
        "semaphore",
        "timeline_mode",
        contrast.rem_euclid(3),
    )?;
    let semaphore_scope = packet_i64(packet, op, &["scope"], "semaphore", "scope", accent)?;
    let timeline_value = packet_i64(
        packet,
        op,
        &["value"],
        "timeline",
        "value",
        scaled(radius_scale, 24.0),
    )?;
    let timeline_step = packet_i64(packet, op, &["step"], "timeline", "step", 1)?;
    let timeline_epoch = packet_i64(packet, op, &["epoch"], "timeline", "epoch", 0)?;
    let timeline_domain = packet_i64(packet, op, &["domain"], "timeline", "domain", accent)?;
    let fence_signaled = packet_i64(packet, op, &["signaled"], "fence", "signaled", 1)?;
    let fence_epoch = packet_i64(packet, op, &["epoch"], "fence", "epoch", 0)?;
    let fence_scope = packet_i64(packet, op, &["scope"], "fence", "scope", accent)?;
    let fence_recycle_mode = packet_i64(packet, op, &["recycle_mode"], "fence", "recycle_mode", 1)?;
    let signal_kind = packet_i64(
        packet,
        op,
        &["kind"],
        "signal",
        "kind",
        contrast.rem_euclid(3),
    )?;
    let signal_phase = packet_i64(packet, op, &["phase"], "signal", "phase", 2)?;
    let signal_fanout = packet_i64(packet, op, &["fanout"], "signal", "fanout", 3)?;
    let signal_ack_mode = packet_i64(packet, op, &["ack_mode"], "signal", "ack_mode", accent)?;
    let event_kind = packet_i64(
        packet,
        op,
        &["kind"],
        "event",
        "kind",
        contrast.rem_euclid(3),
    )?;
    let event_route = packet_i64(packet, op, &["route"], "event", "route", 2)?;
    let event_priority = packet_i64(packet, op, &["priority"], "event", "priority", 3)?;
    let event_payload_mode = packet_i64(
        packet,
        op,
        &["payload_mode"],
        "event",
        "payload_mode",
        accent,
    )?;
    let dispatch_queue_kind = packet_i64(
        packet,
        op,
        &["queue_kind"],
        "dispatch",
        "queue_kind",
        contrast.rem_euclid(3),
    )?;
    let dispatch_lane = packet_i64(packet, op, &["lane"], "dispatch", "lane", 2)?;
    let dispatch_batch = packet_i64(packet, op, &["batch"], "dispatch", "batch", 3)?;
    let dispatch_completion_mode = packet_i64(
        packet,
        op,
        &["completion_mode"],
        "dispatch",
        "completion_mode",
        accent,
    )?;

    Ok(BallPacketFrameSyncFields {
        pass_stage,
        pass_clear_mode,
        pass_sample_count,
        pass_debug_view,
        frame_index,
        frame_present_mode,
        frame_sync_interval,
        frame_exposure,
        target_kind,
        target_width,
        target_height,
        target_multisample,
        frame_graph_passes,
        frame_graph_targets,
        frame_graph_present_stage,
        frame_graph_debug_overlay,
        attachment_slot,
        attachment_format_kind,
        attachment_load_op,
        attachment_store_op,
        pass_chain_stages,
        pass_chain_fanout,
        pass_chain_resolve_stage,
        pass_chain_barrier_mode,
        barrier_scope,
        barrier_source_stage,
        barrier_target_stage,
        barrier_flush_mode,
        resource_buffers,
        resource_textures,
        resource_samplers,
        resource_residency,
        schedule_lanes,
        schedule_queue_depth,
        schedule_async_budget,
        schedule_tick_mode,
        submission_batches,
        submission_fences,
        submission_signal_mode,
        submission_present_hint,
        queue_kind,
        queue_priority,
        queue_budget,
        queue_ownership,
        semaphore_wait_count,
        semaphore_signal_count,
        semaphore_timeline_mode,
        semaphore_scope,
        timeline_value,
        timeline_step,
        timeline_epoch,
        timeline_domain,
        fence_signaled,
        fence_epoch,
        fence_scope,
        fence_recycle_mode,
        signal_kind,
        signal_phase,
        signal_fanout,
        signal_ack_mode,
        event_kind,
        event_route,
        event_priority,
        event_payload_mode,
        dispatch_queue_kind,
        dispatch_lane,
        dispatch_batch,
        dispatch_completion_mode,
    })
}

fn packet_i64(
    packet: &StructValue,
    op: &str,
    flat_names: &[&str],
    nested_name: &str,
    nested_field: &str,
    default: i64,
) -> Result<i64, String> {
    packet_i64_with(packet, op, flat_names, nested_name, nested_field, || {
        default
    })
}

fn packet_i64_with(
    packet: &StructValue,
    op: &str,
    flat_names: &[&str],
    nested_name: &str,
    nested_field: &str,
    default: impl FnOnce() -> i64,
) -> Result<i64, String> {
    find_packet_field(packet, flat_names, &[nested_name], &[nested_field])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()
        .map(|value| value.unwrap_or_else(default))
}

fn scaled(value: f32, factor: f32) -> i64 {
    (value * factor).round() as i64
}
