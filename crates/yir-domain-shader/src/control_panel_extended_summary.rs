use super::surface_primitives::put_text;
use super::BallPacket;

pub(crate) fn draw_control_panel_extended_summary(
    rows: &mut [Vec<char>],
    panel_top: usize,
    panel_right: usize,
    packet: &BallPacket,
) {
    let left = panel_right.saturating_sub(26);
    put_text(
        rows,
        left,
        panel_top + 21,
        &format!("msaa {:>2}", packet.target_multisample),
    );
    put_text(
        rows,
        left,
        panel_top + 22,
        &format!(
            "fg p{} t{} ps{}",
            packet.frame_graph_passes, packet.frame_graph_targets, packet.frame_graph_present_stage
        ),
    );
    put_text(
        rows,
        left,
        panel_top + 23,
        &format!("ovr {:>3}", packet.frame_graph_debug_overlay),
    );
    put_text(
        rows,
        left,
        panel_top + 24,
        &format!(
            "att {} f{} l{} s{}",
            packet.attachment_slot,
            packet.attachment_format_kind,
            packet.attachment_load_op,
            packet.attachment_store_op
        ),
    );
    put_text(
        rows,
        left,
        panel_top + 25,
        &format!(
            "pch s{} f{} r{}",
            packet.pass_chain_stages, packet.pass_chain_fanout, packet.pass_chain_resolve_stage
        ),
    );
    put_text(
        rows,
        left,
        panel_top + 26,
        &format!("bar {:>3}", packet.pass_chain_barrier_mode),
    );
    put_text(
        rows,
        left,
        panel_top + 27,
        &format!(
            "sync {} {}>{}",
            packet.barrier_scope, packet.barrier_source_stage, packet.barrier_target_stage
        ),
    );
    put_text(
        rows,
        left,
        panel_top + 28,
        &format!("flush {:>2}", packet.barrier_flush_mode),
    );
    put_text(
        rows,
        left,
        panel_top + 29,
        &format!(
            "rs b{} t{} s{}",
            packet.resource_buffers, packet.resource_textures, packet.resource_samplers
        ),
    );
    put_text(
        rows,
        left,
        panel_top + 30,
        &format!("res {:>3}", packet.resource_residency),
    );
    put_text(
        rows,
        left,
        panel_top + 31,
        &format!(
            "sch l{} q{}",
            packet.schedule_lanes, packet.schedule_queue_depth
        ),
    );
    put_text(
        rows,
        left,
        panel_top + 32,
        &format!(
            "ab {:>3} tm{}",
            packet.schedule_async_budget, packet.schedule_tick_mode
        ),
    );
    put_text(
        rows,
        left,
        panel_top + 33,
        &format!(
            "sub b{} f{}",
            packet.submission_batches, packet.submission_fences
        ),
    );
    put_text(
        rows,
        left,
        panel_top + 34,
        &format!(
            "sig {} ph{}",
            packet.submission_signal_mode, packet.submission_present_hint
        ),
    );
    put_text(
        rows,
        left,
        panel_top + 35,
        &format!("q k{} p{}", packet.queue_kind, packet.queue_priority),
    );
    put_text(
        rows,
        left,
        panel_top + 36,
        &format!("qb {:>3} ow{}", packet.queue_budget, packet.queue_ownership),
    );
    put_text(
        rows,
        left,
        panel_top + 37,
        &format!(
            "sem w{} s{}",
            packet.semaphore_wait_count, packet.semaphore_signal_count
        ),
    );
    put_text(
        rows,
        left,
        panel_top + 38,
        &format!(
            "tm {} sc{}",
            packet.semaphore_timeline_mode, packet.semaphore_scope
        ),
    );
    put_text(
        rows,
        left,
        panel_top + 39,
        &format!("tl v{} st{}", packet.timeline_value, packet.timeline_step),
    );
    put_text(
        rows,
        left,
        panel_top + 40,
        &format!("ep {} dm{}", packet.timeline_epoch, packet.timeline_domain),
    );
    put_text(
        rows,
        left,
        panel_top + 41,
        &format!("fn s{} e{}", packet.fence_signaled, packet.fence_epoch),
    );
    put_text(
        rows,
        left,
        panel_top + 42,
        &format!("fs {} rc{}", packet.fence_scope, packet.fence_recycle_mode),
    );
    put_text(
        rows,
        left,
        panel_top + 43,
        &format!("sg k{} ph{}", packet.signal_kind, packet.signal_phase),
    );
    put_text(
        rows,
        left,
        panel_top + 44,
        &format!("sf {} ak{}", packet.signal_fanout, packet.signal_ack_mode),
    );
    put_text(
        rows,
        left,
        panel_top + 45,
        &format!("ev k{} rt{}", packet.event_kind, packet.event_route),
    );
    put_text(
        rows,
        left,
        panel_top + 46,
        &format!(
            "ep {} pm{}",
            packet.event_priority, packet.event_payload_mode
        ),
    );
    put_text(
        rows,
        left,
        panel_top + 47,
        &format!(
            "dp q{} l{}",
            packet.dispatch_queue_kind, packet.dispatch_lane
        ),
    );
    put_text(
        rows,
        left,
        panel_top + 48,
        &format!(
            "db {} cm{}",
            packet.dispatch_batch, packet.dispatch_completion_mode
        ),
    );
    put_text(
        rows,
        left,
        panel_top + 49,
        &format!(
            "fb st{} lt{}",
            packet.feedback_status, packet.feedback_latency
        ),
    );
    put_text(
        rows,
        left,
        panel_top + 50,
        &format!(
            "fr {} ch{}",
            packet.feedback_retries, packet.feedback_channel
        ),
    );
    put_text(
        rows,
        left,
        panel_top + 51,
        &format!("in k{} tg{}", packet.intent_kind, packet.intent_target),
    );
    put_text(
        rows,
        left,
        panel_top + 52,
        &format!("iu {} pl{}", packet.intent_urgency, packet.intent_policy),
    );
    put_text(
        rows,
        left,
        panel_top + 53,
        &format!(
            "rk {} rs{}",
            packet.reaction_kind, packet.reaction_result_slot
        ),
    );
    put_text(
        rows,
        left,
        panel_top + 54,
        &format!(
            "rb {} em{}",
            packet.reaction_stability, packet.reaction_echo_mode
        ),
    );
    put_text(
        rows,
        left,
        panel_top + 55,
        &format!("ok {} fs{}", packet.outcome_kind, packet.outcome_final_slot),
    );
    put_text(
        rows,
        left,
        panel_top + 56,
        &format!(
            "oc {} sm{}",
            packet.outcome_confidence, packet.outcome_settle_mode
        ),
    );
    put_text(
        rows,
        left,
        panel_top + 57,
        &format!(
            "rs {} cs{}",
            packet.resolution_kind, packet.resolution_commit_slot
        ),
    );
    put_text(
        rows,
        left,
        panel_top + 58,
        &format!(
            "rc {} pm{}",
            packet.resolution_convergence, packet.resolution_policy_mode
        ),
    );
    put_text(
        rows,
        left,
        panel_top + 59,
        &format!("cm {} as{}", packet.commit_kind, packet.commit_applied_slot),
    );
    put_text(
        rows,
        left,
        panel_top + 60,
        &format!(
            "cd {} md{}",
            packet.commit_durability, packet.commit_commit_mode
        ),
    );
    put_text(
        rows,
        left,
        panel_top + 61,
        &format!(
            "sn {} ss{}",
            packet.snapshot_kind, packet.snapshot_source_slot
        ),
    );
    put_text(
        rows,
        left,
        panel_top + 62,
        &format!(
            "sr {} rm{}",
            packet.snapshot_retention, packet.snapshot_replay_mode
        ),
    );
    put_text(
        rows,
        left,
        panel_top + 63,
        &format!(
            "ck {} as{}",
            packet.checkpoint_kind, packet.checkpoint_anchor_slot
        ),
    );
    put_text(
        rows,
        left,
        panel_top + 64,
        &format!(
            "cr {} rm{}",
            packet.checkpoint_rollback_depth, packet.checkpoint_resume_mode
        ),
    );
}
