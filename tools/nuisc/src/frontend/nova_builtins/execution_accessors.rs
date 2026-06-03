use std::collections::BTreeMap;

use nuis_semantics::model::{AstExpr, NirExpr, NirStructDef, NirTypeRef};

use super::super::{lower_expr, named_type, FunctionSignature, ModuleConstValue};

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_nova_execution_accessor_builtin_call(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
    _current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    _module_consts: &BTreeMap<String, ModuleConstValue>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<Option<NirExpr>, String> {
    let Some((expected_type, field_name)) = execution_state_accessor_target(callee) else {
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

fn execution_state_accessor_target(callee: &str) -> Option<(&'static str, &'static str)> {
    Some(match callee {
        "nova_pass_state_stage" => ("NovaPassState", "stage"),
        "nova_pass_state_clear_mode" => ("NovaPassState", "clear_mode"),
        "nova_pass_state_sample_count" => ("NovaPassState", "sample_count"),
        "nova_pass_state_debug_view" => ("NovaPassState", "debug_view"),
        "nova_frame_state_frame_index" => ("NovaFrameState", "frame_index"),
        "nova_frame_state_present_mode" => ("NovaFrameState", "present_mode"),
        "nova_frame_state_sync_interval" => ("NovaFrameState", "sync_interval"),
        "nova_frame_state_exposure" => ("NovaFrameState", "exposure"),
        "nova_target_state_kind" => ("NovaTargetState", "kind"),
        "nova_target_state_width" => ("NovaTargetState", "width"),
        "nova_target_state_height" => ("NovaTargetState", "height"),
        "nova_target_state_multisample" => ("NovaTargetState", "multisample"),
        "nova_frame_graph_state_passes" => ("NovaFrameGraphState", "passes"),
        "nova_frame_graph_state_targets" => ("NovaFrameGraphState", "targets"),
        "nova_frame_graph_state_present_stage" => ("NovaFrameGraphState", "present_stage"),
        "nova_frame_graph_state_debug_overlay" => ("NovaFrameGraphState", "debug_overlay"),
        "nova_attachment_state_slot" => ("NovaAttachmentState", "slot"),
        "nova_attachment_state_format_kind" => ("NovaAttachmentState", "format_kind"),
        "nova_attachment_state_load_op" => ("NovaAttachmentState", "load_op"),
        "nova_attachment_state_store_op" => ("NovaAttachmentState", "store_op"),
        "nova_pass_chain_state_stages" => ("NovaPassChainState", "stages"),
        "nova_pass_chain_state_fanout" => ("NovaPassChainState", "fanout"),
        "nova_pass_chain_state_resolve_stage" => ("NovaPassChainState", "resolve_stage"),
        "nova_pass_chain_state_barrier_mode" => ("NovaPassChainState", "barrier_mode"),
        "nova_barrier_state_scope" => ("NovaBarrierState", "scope"),
        "nova_barrier_state_source_stage" => ("NovaBarrierState", "source_stage"),
        "nova_barrier_state_target_stage" => ("NovaBarrierState", "target_stage"),
        "nova_barrier_state_flush_mode" => ("NovaBarrierState", "flush_mode"),
        "nova_resource_set_state_buffers" => ("NovaResourceSetState", "buffers"),
        "nova_resource_set_state_textures" => ("NovaResourceSetState", "textures"),
        "nova_resource_set_state_samplers" => ("NovaResourceSetState", "samplers"),
        "nova_resource_set_state_residency" => ("NovaResourceSetState", "residency"),
        "nova_schedule_state_lanes" => ("NovaScheduleState", "lanes"),
        "nova_schedule_state_queue_depth" => ("NovaScheduleState", "queue_depth"),
        "nova_schedule_state_async_budget" => ("NovaScheduleState", "async_budget"),
        "nova_schedule_state_tick_mode" => ("NovaScheduleState", "tick_mode"),
        "nova_submission_state_batches" => ("NovaSubmissionState", "batches"),
        "nova_submission_state_fences" => ("NovaSubmissionState", "fences"),
        "nova_submission_state_signal_mode" => ("NovaSubmissionState", "signal_mode"),
        "nova_submission_state_present_hint" => ("NovaSubmissionState", "present_hint"),
        "nova_queue_state_kind" => ("NovaQueueState", "kind"),
        "nova_queue_state_priority" => ("NovaQueueState", "priority"),
        "nova_queue_state_budget" => ("NovaQueueState", "budget"),
        "nova_queue_state_ownership" => ("NovaQueueState", "ownership"),
        "nova_semaphore_state_wait_count" => ("NovaSemaphoreState", "wait_count"),
        "nova_semaphore_state_signal_count" => ("NovaSemaphoreState", "signal_count"),
        "nova_semaphore_state_timeline_mode" => ("NovaSemaphoreState", "timeline_mode"),
        "nova_semaphore_state_scope" => ("NovaSemaphoreState", "scope"),
        "nova_timeline_state_value" => ("NovaTimelineState", "value"),
        "nova_timeline_state_step" => ("NovaTimelineState", "step"),
        "nova_timeline_state_epoch" => ("NovaTimelineState", "epoch"),
        "nova_timeline_state_domain" => ("NovaTimelineState", "domain"),
        "nova_fence_state_signaled" => ("NovaFenceState", "signaled"),
        "nova_fence_state_epoch" => ("NovaFenceState", "epoch"),
        "nova_fence_state_scope" => ("NovaFenceState", "scope"),
        "nova_fence_state_recycle_mode" => ("NovaFenceState", "recycle_mode"),
        "nova_signal_state_kind" => ("NovaSignalState", "kind"),
        "nova_signal_state_phase" => ("NovaSignalState", "phase"),
        "nova_signal_state_fanout" => ("NovaSignalState", "fanout"),
        "nova_signal_state_ack_mode" => ("NovaSignalState", "ack_mode"),
        "nova_event_state_kind" => ("NovaEventState", "kind"),
        "nova_event_state_route" => ("NovaEventState", "route"),
        "nova_event_state_priority" => ("NovaEventState", "priority"),
        "nova_event_state_payload_mode" => ("NovaEventState", "payload_mode"),
        "nova_dispatch_state_queue_kind" => ("NovaDispatchState", "queue_kind"),
        "nova_dispatch_state_lane" => ("NovaDispatchState", "lane"),
        "nova_dispatch_state_batch" => ("NovaDispatchState", "batch"),
        "nova_dispatch_state_completion_mode" => ("NovaDispatchState", "completion_mode"),
        _ => return None,
    })
}
