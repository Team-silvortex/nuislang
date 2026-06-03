use std::collections::BTreeMap;

use nuis_semantics::model::{AstExpr, NirExpr, NirStructDef, NirTypeRef};

use super::super::{FunctionSignature, ModuleConstValue};
use super::packet_helpers::{build_struct_literal, lower_i64_arg_list};

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_nova_execution_packet_builtin_call(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
    _current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    _module_consts: &BTreeMap<String, ModuleConstValue>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<Option<NirExpr>, String> {
    let (type_name, fields) = match callee {
        "nova_pass_packet" => (
            "NovaPassPacket",
            &["stage", "clear_mode", "sample_count", "debug_view"][..],
        ),
        "nova_frame_packet" => (
            "NovaFramePacket",
            &["frame_index", "present_mode", "sync_interval", "exposure"][..],
        ),
        "nova_target_packet" => (
            "NovaTargetPacket",
            &["kind", "width", "height", "multisample"][..],
        ),
        "nova_frame_graph_packet" => (
            "NovaFrameGraphPacket",
            &["passes", "targets", "present_stage", "debug_overlay"][..],
        ),
        "nova_attachment_packet" => (
            "NovaAttachmentPacket",
            &["slot", "format_kind", "load_op", "store_op"][..],
        ),
        "nova_pass_chain_packet" => (
            "NovaPassChainPacket",
            &["stages", "fanout", "resolve_stage", "barrier_mode"][..],
        ),
        "nova_barrier_packet" => (
            "NovaBarrierPacket",
            &["scope", "source_stage", "target_stage", "flush_mode"][..],
        ),
        "nova_resource_set_packet" => (
            "NovaResourceSetPacket",
            &["buffers", "textures", "samplers", "residency"][..],
        ),
        "nova_schedule_packet" => (
            "NovaSchedulePacket",
            &["lanes", "queue_depth", "async_budget", "tick_mode"][..],
        ),
        "nova_submission_packet" => (
            "NovaSubmissionPacket",
            &["batches", "fences", "signal_mode", "present_hint"][..],
        ),
        "nova_queue_packet" => (
            "NovaQueuePacket",
            &["kind", "priority", "budget", "ownership"][..],
        ),
        "nova_semaphore_packet" => (
            "NovaSemaphorePacket",
            &["wait_count", "signal_count", "timeline_mode", "scope"][..],
        ),
        "nova_timeline_packet" => (
            "NovaTimelinePacket",
            &["value", "step", "epoch", "domain"][..],
        ),
        "nova_fence_packet" => (
            "NovaFencePacket",
            &["signaled", "epoch", "scope", "recycle_mode"][..],
        ),
        "nova_signal_packet" => (
            "NovaSignalPacket",
            &["kind", "phase", "fanout", "ack_mode"][..],
        ),
        "nova_event_packet" => (
            "NovaEventPacket",
            &["kind", "route", "priority", "payload_mode"][..],
        ),
        "nova_dispatch_packet" => (
            "NovaDispatchPacket",
            &["queue_kind", "lane", "batch", "completion_mode"][..],
        ),
        _ => return Ok(None),
    };

    let values = lower_i64_arg_list(
        args,
        4,
        &format!("{callee}(...) expects 4 args"),
        current_domain,
        bindings,
        signatures,
        struct_table,
    )?;
    Ok(Some(build_struct_literal(type_name, fields, values)))
}
