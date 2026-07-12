use super::packet_helpers::{find_packet_field, scalar_to_color_key};
use yir_core::{StructValue, Value};

pub(crate) struct BallPacketResponseFields {
    pub(crate) feedback_status: i64,
    pub(crate) feedback_latency: i64,
    pub(crate) feedback_retries: i64,
    pub(crate) feedback_channel: i64,
    pub(crate) intent_kind: i64,
    pub(crate) intent_target: i64,
    pub(crate) intent_urgency: i64,
    pub(crate) intent_policy: i64,
    pub(crate) reaction_kind: i64,
    pub(crate) reaction_result_slot: i64,
    pub(crate) reaction_stability: i64,
    pub(crate) reaction_echo_mode: i64,
    pub(crate) outcome_kind: i64,
    pub(crate) outcome_final_slot: i64,
    pub(crate) outcome_confidence: i64,
    pub(crate) outcome_settle_mode: i64,
    pub(crate) resolution_kind: i64,
    pub(crate) resolution_commit_slot: i64,
    pub(crate) resolution_convergence: i64,
    pub(crate) resolution_policy_mode: i64,
    pub(crate) commit_kind: i64,
    pub(crate) commit_applied_slot: i64,
    pub(crate) commit_durability: i64,
    pub(crate) commit_commit_mode: i64,
    pub(crate) snapshot_kind: i64,
    pub(crate) snapshot_source_slot: i64,
    pub(crate) snapshot_retention: i64,
    pub(crate) snapshot_replay_mode: i64,
    pub(crate) checkpoint_kind: i64,
    pub(crate) checkpoint_anchor_slot: i64,
    pub(crate) checkpoint_rollback_depth: i64,
    pub(crate) checkpoint_resume_mode: i64,
}

pub(crate) fn parse_ball_packet_response(
    packet: &StructValue,
    op: &str,
    radius_scale: f32,
    accent: i64,
    contrast: i64,
    speed: &Value,
) -> Result<BallPacketResponseFields, String> {
    let speed_key = || scalar_to_color_key(speed, op).unwrap_or(0);
    let small_radius = || radius_scale.round() as i64 % 4;

    let feedback_status = packet_i64_with(packet, op, &["status"], "feedback", "status", || {
        speed_key().rem_euclid(2)
    })?;
    let feedback_latency =
        packet_i64_with(packet, op, &["latency"], "feedback", "latency", speed_key)?;
    let feedback_retries = packet_i64_with(
        packet,
        op,
        &["retries"],
        "feedback",
        "retries",
        small_radius,
    )?;
    let feedback_channel = packet_i64(packet, op, &["channel"], "feedback", "channel", accent)?;
    let intent_kind = packet_i64(
        packet,
        op,
        &["kind"],
        "intent",
        "kind",
        contrast.rem_euclid(3),
    )?;
    let intent_target = packet_i64(
        packet,
        op,
        &["target_slot"],
        "intent",
        "target_slot",
        contrast,
    )?;
    let intent_urgency = packet_i64_with(packet, op, &["urgency"], "intent", "urgency", speed_key)?;
    let intent_policy = packet_i64(packet, op, &["policy"], "intent", "policy", accent)?;
    let reaction_kind = packet_i64(
        packet,
        op,
        &["kind"],
        "reaction",
        "kind",
        contrast.rem_euclid(3),
    )?;
    let reaction_result_slot = packet_i64(
        packet,
        op,
        &["result_slot"],
        "reaction",
        "result_slot",
        contrast,
    )?;
    let reaction_stability = packet_i64_with(
        packet,
        op,
        &["stability"],
        "reaction",
        "stability",
        small_radius,
    )?;
    let reaction_echo_mode =
        packet_i64(packet, op, &["echo_mode"], "reaction", "echo_mode", accent)?;
    let outcome_kind = packet_i64(
        packet,
        op,
        &["kind"],
        "outcome",
        "kind",
        contrast.rem_euclid(3),
    )?;
    let outcome_final_slot = packet_i64(
        packet,
        op,
        &["final_slot"],
        "outcome",
        "final_slot",
        contrast,
    )?;
    let outcome_confidence = packet_i64_with(
        packet,
        op,
        &["confidence"],
        "outcome",
        "confidence",
        speed_key,
    )?;
    let outcome_settle_mode = packet_i64(
        packet,
        op,
        &["settle_mode"],
        "outcome",
        "settle_mode",
        accent,
    )?;
    let resolution_kind = packet_i64(
        packet,
        op,
        &["kind"],
        "resolution",
        "kind",
        contrast.rem_euclid(3),
    )?;
    let resolution_commit_slot = packet_i64(
        packet,
        op,
        &["commit_slot"],
        "resolution",
        "commit_slot",
        contrast,
    )?;
    let resolution_convergence = packet_i64_with(
        packet,
        op,
        &["convergence"],
        "resolution",
        "convergence",
        small_radius,
    )?;
    let resolution_policy_mode = packet_i64(
        packet,
        op,
        &["policy_mode"],
        "resolution",
        "policy_mode",
        accent,
    )?;
    let commit_kind = packet_i64(
        packet,
        op,
        &["kind"],
        "commit",
        "kind",
        contrast.rem_euclid(3),
    )?;
    let commit_applied_slot = packet_i64(
        packet,
        op,
        &["applied_slot"],
        "commit",
        "applied_slot",
        contrast,
    )?;
    let commit_durability = packet_i64_with(
        packet,
        op,
        &["durability"],
        "commit",
        "durability",
        speed_key,
    )?;
    let commit_commit_mode = packet_i64(
        packet,
        op,
        &["commit_mode"],
        "commit",
        "commit_mode",
        accent,
    )?;
    let snapshot_kind = packet_i64(
        packet,
        op,
        &["kind"],
        "snapshot",
        "kind",
        contrast.rem_euclid(3),
    )?;
    let snapshot_source_slot = packet_i64(
        packet,
        op,
        &["source_slot"],
        "snapshot",
        "source_slot",
        contrast,
    )?;
    let snapshot_retention = packet_i64_with(
        packet,
        op,
        &["retention"],
        "snapshot",
        "retention",
        small_radius,
    )?;
    let snapshot_replay_mode = packet_i64(
        packet,
        op,
        &["replay_mode"],
        "snapshot",
        "replay_mode",
        accent,
    )?;
    let checkpoint_kind = packet_i64(
        packet,
        op,
        &["kind"],
        "checkpoint",
        "kind",
        contrast.rem_euclid(3),
    )?;
    let checkpoint_anchor_slot = packet_i64(
        packet,
        op,
        &["anchor_slot"],
        "checkpoint",
        "anchor_slot",
        contrast,
    )?;
    let checkpoint_rollback_depth = packet_i64_with(
        packet,
        op,
        &["rollback_depth"],
        "checkpoint",
        "rollback_depth",
        speed_key,
    )?;
    let checkpoint_resume_mode = packet_i64(
        packet,
        op,
        &["resume_mode"],
        "checkpoint",
        "resume_mode",
        accent,
    )?;

    Ok(BallPacketResponseFields {
        feedback_status,
        feedback_latency,
        feedback_retries,
        feedback_channel,
        intent_kind,
        intent_target,
        intent_urgency,
        intent_policy,
        reaction_kind,
        reaction_result_slot,
        reaction_stability,
        reaction_echo_mode,
        outcome_kind,
        outcome_final_slot,
        outcome_confidence,
        outcome_settle_mode,
        resolution_kind,
        resolution_commit_slot,
        resolution_convergence,
        resolution_policy_mode,
        commit_kind,
        commit_applied_slot,
        commit_durability,
        commit_commit_mode,
        snapshot_kind,
        snapshot_source_slot,
        snapshot_retention,
        snapshot_replay_mode,
        checkpoint_kind,
        checkpoint_anchor_slot,
        checkpoint_rollback_depth,
        checkpoint_resume_mode,
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
