use std::collections::BTreeMap;

use nuis_semantics::model::{AstExpr, NirExpr, NirStructDef, NirTypeRef};

use crate::frontend::{lower_expr, named_type, FunctionSignature, ModuleConstValue};

fn build_four_field_state(
    packet: NirExpr,
    state_type: &str,
    fields: [&str; 4],
) -> Result<Option<NirExpr>, String> {
    Ok(Some(NirExpr::StructLiteral {
        type_name: state_type.to_owned(),
        fields: vec![
            (
                fields[0].to_owned(),
                NirExpr::FieldAccess {
                    base: Box::new(packet.clone()),
                    field: fields[0].to_owned(),
                },
            ),
            (
                fields[1].to_owned(),
                NirExpr::FieldAccess {
                    base: Box::new(packet.clone()),
                    field: fields[1].to_owned(),
                },
            ),
            (
                fields[2].to_owned(),
                NirExpr::FieldAccess {
                    base: Box::new(packet.clone()),
                    field: fields[2].to_owned(),
                },
            ),
            (
                fields[3].to_owned(),
                NirExpr::FieldAccess {
                    base: Box::new(packet),
                    field: fields[3].to_owned(),
                },
            ),
        ],
    }))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_nova_execution_state_builtin_call(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
    _current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    _module_consts: &BTreeMap<String, ModuleConstValue>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<Option<NirExpr>, String> {
    let (packet_type, state_type, fields) = match callee {
        "nova_pass_state" => (
            "NovaPassPacket",
            "NovaPassState",
            ["stage", "clear_mode", "sample_count", "debug_view"],
        ),
        "nova_frame_state" => (
            "NovaFramePacket",
            "NovaFrameState",
            ["frame_index", "present_mode", "sync_interval", "exposure"],
        ),
        "nova_target_state" => (
            "NovaTargetPacket",
            "NovaTargetState",
            ["kind", "width", "height", "multisample"],
        ),
        "nova_frame_graph_state" => (
            "NovaFrameGraphPacket",
            "NovaFrameGraphState",
            ["passes", "targets", "present_stage", "debug_overlay"],
        ),
        "nova_attachment_state" => (
            "NovaAttachmentPacket",
            "NovaAttachmentState",
            ["slot", "format_kind", "load_op", "store_op"],
        ),
        "nova_pass_chain_state" => (
            "NovaPassChainPacket",
            "NovaPassChainState",
            ["stages", "fanout", "resolve_stage", "barrier_mode"],
        ),
        "nova_barrier_state" => (
            "NovaBarrierPacket",
            "NovaBarrierState",
            ["scope", "source_stage", "target_stage", "flush_mode"],
        ),
        "nova_resource_set_state" => (
            "NovaResourceSetPacket",
            "NovaResourceSetState",
            ["buffers", "textures", "samplers", "residency"],
        ),
        "nova_schedule_state" => (
            "NovaSchedulePacket",
            "NovaScheduleState",
            ["lanes", "queue_depth", "async_budget", "tick_mode"],
        ),
        "nova_submission_state" => (
            "NovaSubmissionPacket",
            "NovaSubmissionState",
            ["batches", "fences", "signal_mode", "present_hint"],
        ),
        "nova_queue_state" => (
            "NovaQueuePacket",
            "NovaQueueState",
            ["kind", "priority", "budget", "ownership"],
        ),
        "nova_semaphore_state" => (
            "NovaSemaphorePacket",
            "NovaSemaphoreState",
            ["wait_count", "signal_count", "timeline_mode", "scope"],
        ),
        "nova_timeline_state" => (
            "NovaTimelinePacket",
            "NovaTimelineState",
            ["value", "step", "epoch", "domain"],
        ),
        "nova_fence_state" => (
            "NovaFencePacket",
            "NovaFenceState",
            ["signaled", "epoch", "scope", "recycle_mode"],
        ),
        "nova_signal_state" => (
            "NovaSignalPacket",
            "NovaSignalState",
            ["kind", "phase", "fanout", "ack_mode"],
        ),
        "nova_event_state" => (
            "NovaEventPacket",
            "NovaEventState",
            ["kind", "route", "priority", "payload_mode"],
        ),
        "nova_dispatch_state" => (
            "NovaDispatchPacket",
            "NovaDispatchState",
            ["queue_kind", "lane", "batch", "completion_mode"],
        ),
        _ => return Ok(None),
    };

    let [packet] = args else {
        return Err(format!("{callee}(...) expects 1 arg"));
    };
    let packet = lower_expr(
        packet,
        current_domain,
        bindings,
        signatures,
        struct_table,
        Some(&named_type(packet_type)),
    )?;
    build_four_field_state(packet, state_type, fields)
}
