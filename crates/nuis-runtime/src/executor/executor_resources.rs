use std::collections::{BTreeMap, BTreeSet};

use crate::RuntimeRole;

use super::{
    ExecutionPhaseAction, ExecutionPhaseBinding, ExecutionPhaseContext, ExecutionPhaseOutcome,
    ExecutionResourceBinding, ExecutionResourceKind, ExecutionStateSnapshot,
};

pub(super) fn default_phase_action(ctx: &ExecutionPhaseContext<'_>) -> ExecutionPhaseAction {
    ExecutionPhaseAction {
        kind: format!("phase.{}", ctx.phase),
        input_handles: default_input_handles(ctx),
        resolved_inputs: Vec::new(),
        output_handles: default_output_handles(ctx),
        resource_bindings: default_resource_bindings(ctx),
        resolved_resources: Vec::new(),
        scheduler_keys: vec![
            ctx.scheduler_binding.to_owned(),
            ctx.domain_family.to_owned(),
            ctx.phase.to_owned(),
        ],
        adapter_hint: None,
    }
}

pub(super) fn materialize_phase_action(
    binding: &ExecutionPhaseBinding,
    state: &ExecutionStateSnapshot,
) -> ExecutionPhaseAction {
    let mut action = binding.action.clone();
    let slot_map = state
        .handle_slots
        .iter()
        .map(|slot| (slot.key.as_str(), slot.value.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut known = BTreeSet::new();
    action.input_handles.retain(|key| known.insert(key.clone()));
    action.resolved_inputs = action
        .input_handles
        .iter()
        .filter_map(|key| {
            slot_map
                .get(key.as_str())
                .map(|value| ExecutionResourceBinding {
                    key: key.clone(),
                    kind: slot_resource_kind(key),
                    capability_label: Some(slot_resource_capability_label(key)),
                    value: (*value).to_owned(),
                })
        })
        .collect();
    action.resolved_resources = action
        .resource_bindings
        .iter()
        .map(|binding| ExecutionResourceBinding {
            key: binding.key.clone(),
            kind: binding.kind.clone(),
            capability_label: binding.capability_label.clone(),
            value: resolve_resource_binding_value(&binding.value, &slot_map),
        })
        .collect();
    action
}

fn resolve_resource_binding_value(value: &str, slot_map: &BTreeMap<&str, &str>) -> String {
    if let Some(slot_key) = value.strip_prefix("slot:") {
        slot_map
            .get(slot_key)
            .map(|resolved| (*resolved).to_owned())
            .unwrap_or_else(|| format!("unresolved:{slot_key}"))
    } else {
        value.to_owned()
    }
}

pub(super) fn default_phase_outcome(
    action: &ExecutionPhaseAction,
    binding: &ExecutionPhaseBinding,
) -> ExecutionPhaseOutcome {
    ExecutionPhaseOutcome {
        status: "mock-complete".to_owned(),
        produced_handles: action.output_handles.clone(),
        produced_slots: action
            .output_handles
            .iter()
            .map(|key| ExecutionResourceBinding {
                key: key.clone(),
                kind: slot_resource_kind(key),
                capability_label: Some(slot_resource_capability_label(key)),
                value: format!("mock://{}/{}", binding.phase, key),
            })
            .collect(),
        notes: vec![
            format!("phase={} completed in mock runtime", binding.phase),
            format!("scheduler={}", binding.scheduler_binding),
        ],
    }
}

pub(super) fn apply_phase_outcome(
    state: &mut ExecutionStateSnapshot,
    outcome: &ExecutionPhaseOutcome,
) {
    let mut known = state
        .available_handles
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    for handle in &outcome.produced_handles {
        if known.insert(handle.clone()) {
            state.available_handles.push(handle.clone());
        }
    }
    let mut slot_map = state
        .handle_slots
        .iter()
        .map(|slot| (slot.key.clone(), slot.value.clone()))
        .collect::<BTreeMap<_, _>>();
    for slot in &outcome.produced_slots {
        slot_map.insert(slot.key.clone(), slot.value.clone());
    }
    state.handle_slots = slot_map
        .into_iter()
        .map(|(key, value)| ExecutionResourceBinding {
            kind: slot_resource_kind(&key),
            capability_label: Some(slot_resource_capability_label(&key)),
            key,
            value,
        })
        .collect();
}

pub(super) fn slot_resource_kind(key: &str) -> ExecutionResourceKind {
    if key.ends_with(".handle") {
        ExecutionResourceKind::Handle
    } else if key.ends_with(".packet") || key.contains("packet") {
        ExecutionResourceKind::Packet
    } else if key.ends_with(".response") || key.contains("response") {
        ExecutionResourceKind::Response
    } else if key.ends_with(".buffer") || key.contains("buffer") {
        ExecutionResourceKind::Buffer
    } else if key.contains("scheduler") {
        ExecutionResourceKind::Scheduler
    } else if key.contains("bridge") {
        ExecutionResourceKind::Bridge
    } else {
        ExecutionResourceKind::Slot
    }
}

pub(super) fn slot_resource_capability_label(key: &str) -> String {
    if key.ends_with(".handle") {
        format!("cap.handle.{key}")
    } else if key.ends_with(".packet") || key.contains("packet") {
        format!("cap.packet.{key}")
    } else if key.ends_with(".buffer") || key.contains("buffer") {
        format!("cap.buffer.{key}")
    } else if key.ends_with(".response") || key.contains("response") || key.ends_with(".target") {
        format!("cap.response.{key}")
    } else {
        format!("cap.slot.{key}")
    }
}

pub(super) fn domain_resource_capability_label(
    domain_family: &str,
    selected_lowering_target: Option<&str>,
    key: &str,
    kind: &ExecutionResourceKind,
) -> String {
    let scope = capability_scope(domain_family, selected_lowering_target);
    match (domain_family, kind) {
        ("network", ExecutionResourceKind::Packet) => format!("cap.{scope}.packet.{key}"),
        ("network", ExecutionResourceKind::Response) => format!("cap.{scope}.response.{key}"),
        ("network", ExecutionResourceKind::Handle) => format!("cap.{scope}.handle.{key}"),
        ("kernel", ExecutionResourceKind::Buffer) => format!("cap.{scope}.buffer.{key}"),
        ("kernel", ExecutionResourceKind::Handle) => format!("cap.{scope}.dispatch.{key}"),
        ("shader", ExecutionResourceKind::Buffer) => format!("cap.{scope}.buffer.{key}"),
        ("shader", ExecutionResourceKind::Handle) => format!("cap.{scope}.draw.{key}"),
        ("shader", ExecutionResourceKind::Response) => format!("cap.{scope}.frame.{key}"),
        (_, ExecutionResourceKind::Bridge) => format!("cap.{scope}.bridge.{key}"),
        (_, ExecutionResourceKind::Scheduler) => format!("cap.{scope}.scheduler.{key}"),
        (_, ExecutionResourceKind::Metadata) => format!("cap.{scope}.meta.{key}"),
        _ => format!("cap.{scope}.{key}"),
    }
}

fn capability_scope(domain_family: &str, selected_lowering_target: Option<&str>) -> String {
    if let Some(target) = selected_lowering_target {
        let slug = target.replace(['.', '-'], "_");
        format!("{domain_family}.{slug}")
    } else {
        domain_family.to_owned()
    }
}

fn default_input_handles(ctx: &ExecutionPhaseContext<'_>) -> Vec<String> {
    match (ctx.domain_family, ctx.phase) {
        ("network", "bind") => vec!["authority.text".to_owned()],
        ("network", "submit") => vec!["session.handle".to_owned(), "request.packet".to_owned()],
        ("network", "wait") => vec!["task.handle".to_owned()],
        ("network", "finalize") => vec!["response.handle".to_owned()],
        ("kernel", "bind") => vec!["kernel.buffer".to_owned(), "queue.slot".to_owned()],
        ("kernel", "submit") => vec!["kernel.buffer".to_owned(), "dispatch.grid".to_owned()],
        ("kernel", "wait") => vec!["dispatch.handle".to_owned()],
        ("kernel", "finalize") => vec!["result.buffer".to_owned()],
        ("shader", "bind") => vec!["shader.buffer".to_owned(), "frame.target".to_owned()],
        ("shader", "submit") => vec!["shader.buffer".to_owned(), "draw.list".to_owned()],
        ("shader", "wait") => vec!["draw.handle".to_owned()],
        ("shader", "finalize") => vec!["frame.target".to_owned()],
        (_, "bind") => vec!["bridge.surface".to_owned()],
        (_, "submit") => vec!["phase.submit".to_owned()],
        (_, "wait") => vec!["phase.wait".to_owned()],
        (_, "finalize") => vec!["phase.finalize".to_owned()],
        _ => vec!["phase.input".to_owned()],
    }
}

fn default_output_handles(ctx: &ExecutionPhaseContext<'_>) -> Vec<String> {
    match (ctx.domain_family, ctx.phase) {
        ("network", "bind") => vec!["session.handle".to_owned()],
        ("network", "submit") => vec!["task.handle".to_owned()],
        ("network", "wait") => vec!["response.handle".to_owned()],
        ("network", "finalize") => vec!["status.code".to_owned()],
        ("kernel", "bind") => vec!["kernel.buffer".to_owned()],
        ("kernel", "submit") => vec!["dispatch.handle".to_owned()],
        ("kernel", "wait") => vec!["result.buffer".to_owned()],
        ("kernel", "finalize") => vec!["completion.fence".to_owned()],
        ("shader", "bind") => vec!["shader.buffer".to_owned()],
        ("shader", "submit") => vec!["draw.handle".to_owned()],
        ("shader", "wait") => vec!["frame.target".to_owned()],
        ("shader", "finalize") => vec!["present.fence".to_owned()],
        (_, "bind") => vec!["phase.bind".to_owned()],
        (_, "submit") => vec!["phase.submit".to_owned()],
        (_, "wait") => vec!["phase.wait".to_owned()],
        (_, "finalize") => vec!["phase.finalize".to_owned()],
        _ => vec!["phase.output".to_owned()],
    }
}

fn default_resource_bindings(ctx: &ExecutionPhaseContext<'_>) -> Vec<ExecutionResourceBinding> {
    let mut bindings = vec![
        ExecutionResourceBinding {
            key: "bridge_surface".to_owned(),
            kind: ExecutionResourceKind::Bridge,
            capability_label: Some(domain_resource_capability_label(
                ctx.domain_family,
                ctx.selected_lowering_target,
                "bridge_surface",
                &ExecutionResourceKind::Bridge,
            )),
            value: ctx.bridge_surface.to_owned(),
        },
        ExecutionResourceBinding {
            key: "scheduler_binding".to_owned(),
            kind: ExecutionResourceKind::Scheduler,
            capability_label: Some(domain_resource_capability_label(
                ctx.domain_family,
                ctx.selected_lowering_target,
                "scheduler_binding",
                &ExecutionResourceKind::Scheduler,
            )),
            value: ctx.scheduler_binding.to_owned(),
        },
        ExecutionResourceBinding {
            key: "backend_summary".to_owned(),
            kind: ExecutionResourceKind::Metadata,
            capability_label: Some(domain_resource_capability_label(
                ctx.domain_family,
                ctx.selected_lowering_target,
                "backend_summary",
                &ExecutionResourceKind::Metadata,
            )),
            value: ctx.backend_summary.to_owned(),
        },
    ];
    if let Some(ir_sidecar_summary) = ctx.ir_sidecar_summary {
        bindings.push(ExecutionResourceBinding {
            key: "lowering_capabilities".to_owned(),
            kind: ExecutionResourceKind::Metadata,
            capability_label: Some(domain_resource_capability_label(
                ctx.domain_family,
                ctx.selected_lowering_target,
                "lowering_capabilities",
                &ExecutionResourceKind::Metadata,
            )),
            value: ir_sidecar_summary.to_owned(),
        });
    }
    if let Some(clock_summary) = ctx.clock_summary {
        bindings.push(ExecutionResourceBinding {
            key: "clock_protocol".to_owned(),
            kind: ExecutionResourceKind::Metadata,
            capability_label: Some(domain_resource_capability_label(
                ctx.domain_family,
                ctx.selected_lowering_target,
                "clock_protocol",
                &ExecutionResourceKind::Metadata,
            )),
            value: clock_summary.to_owned(),
        });
    }
    match ctx.domain_family {
        "network" => {
            bindings.push(ExecutionResourceBinding {
                key: "active_session".to_owned(),
                kind: ExecutionResourceKind::Handle,
                capability_label: Some(domain_resource_capability_label(
                    ctx.domain_family,
                    ctx.selected_lowering_target,
                    "active_session",
                    &ExecutionResourceKind::Handle,
                )),
                value: "slot:session.handle".to_owned(),
            });
            bindings.push(ExecutionResourceBinding {
                key: "request_packet".to_owned(),
                kind: ExecutionResourceKind::Packet,
                capability_label: Some(domain_resource_capability_label(
                    ctx.domain_family,
                    ctx.selected_lowering_target,
                    "request_packet",
                    &ExecutionResourceKind::Packet,
                )),
                value: "slot:request.packet".to_owned(),
            });
            bindings.push(ExecutionResourceBinding {
                key: "active_response".to_owned(),
                kind: ExecutionResourceKind::Response,
                capability_label: Some(domain_resource_capability_label(
                    ctx.domain_family,
                    ctx.selected_lowering_target,
                    "active_response",
                    &ExecutionResourceKind::Response,
                )),
                value: "slot:response.handle".to_owned(),
            });
        }
        "kernel" => {
            bindings.push(ExecutionResourceBinding {
                key: "kernel_buffer".to_owned(),
                kind: ExecutionResourceKind::Buffer,
                capability_label: Some(domain_resource_capability_label(
                    ctx.domain_family,
                    ctx.selected_lowering_target,
                    "kernel_buffer",
                    &ExecutionResourceKind::Buffer,
                )),
                value: "slot:kernel.buffer".to_owned(),
            });
            bindings.push(ExecutionResourceBinding {
                key: "dispatch_handle".to_owned(),
                kind: ExecutionResourceKind::Handle,
                capability_label: Some(domain_resource_capability_label(
                    ctx.domain_family,
                    ctx.selected_lowering_target,
                    "dispatch_handle",
                    &ExecutionResourceKind::Handle,
                )),
                value: "slot:dispatch.handle".to_owned(),
            });
            bindings.push(ExecutionResourceBinding {
                key: "result_buffer".to_owned(),
                kind: ExecutionResourceKind::Buffer,
                capability_label: Some(domain_resource_capability_label(
                    ctx.domain_family,
                    ctx.selected_lowering_target,
                    "result_buffer",
                    &ExecutionResourceKind::Buffer,
                )),
                value: "slot:result.buffer".to_owned(),
            });
        }
        "shader" => {
            bindings.push(ExecutionResourceBinding {
                key: "shader_buffer".to_owned(),
                kind: ExecutionResourceKind::Buffer,
                capability_label: Some(domain_resource_capability_label(
                    ctx.domain_family,
                    ctx.selected_lowering_target,
                    "shader_buffer",
                    &ExecutionResourceKind::Buffer,
                )),
                value: "slot:shader.buffer".to_owned(),
            });
            bindings.push(ExecutionResourceBinding {
                key: "draw_handle".to_owned(),
                kind: ExecutionResourceKind::Handle,
                capability_label: Some(domain_resource_capability_label(
                    ctx.domain_family,
                    ctx.selected_lowering_target,
                    "draw_handle",
                    &ExecutionResourceKind::Handle,
                )),
                value: "slot:draw.handle".to_owned(),
            });
            bindings.push(ExecutionResourceBinding {
                key: "frame_target".to_owned(),
                kind: ExecutionResourceKind::Response,
                capability_label: Some(domain_resource_capability_label(
                    ctx.domain_family,
                    ctx.selected_lowering_target,
                    "frame_target",
                    &ExecutionResourceKind::Response,
                )),
                value: "slot:frame.target".to_owned(),
            });
        }
        _ => {}
    }
    bindings
}

pub(super) fn phase_role(phase: &str) -> RuntimeRole {
    match phase {
        "bind" => RuntimeRole::Bind,
        "submit" | "wait" | "finalize" => RuntimeRole::Execute,
        _ => RuntimeRole::Execute,
    }
}
