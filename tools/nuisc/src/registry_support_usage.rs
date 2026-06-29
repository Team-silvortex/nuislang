use std::collections::BTreeSet;

use crate::data_markers::{
    all_downlink_directional_marker_slots, all_sync_marker_slots,
    all_uplink_directional_marker_slots, data_common_marker_slots, data_marker_surface,
};
use crate::nir_walk::walk_child_exprs;
use nuis_semantics::model::{NirExpr, NirModule, NirStmt};

pub(crate) fn covered_profile_slots(
    domain_family: &str,
    matched_support_surface: &[String],
    matched_support_profile_slots: &[String],
) -> Vec<String> {
    let mut covered = matched_support_profile_slots
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    for surface in matched_support_surface {
        for slot in implied_slots_for_surface(domain_family, surface) {
            covered.insert(slot.to_string());
        }
    }
    covered.into_iter().collect::<Vec<_>>()
}

pub(crate) fn collect_resource_usage_hints(
    module: &NirModule,
    domain_family: &str,
    resources: &mut BTreeSet<String>,
) {
    for function in &module.functions {
        for stmt in &function.body {
            collect_resource_usage_hints_stmt(stmt, domain_family, resources);
        }
    }
}

fn collect_resource_usage_hints_stmt(
    stmt: &NirStmt,
    domain_family: &str,
    resources: &mut BTreeSet<String>,
) {
    match stmt {
        NirStmt::Let { value, .. }
        | NirStmt::Const { value, .. }
        | NirStmt::Print(value)
        | NirStmt::Await(value)
        | NirStmt::Expr(value) => {
            collect_resource_usage_hints_expr(value, domain_family, resources)
        }
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            collect_resource_usage_hints_expr(condition, domain_family, resources);
            for stmt in then_body {
                collect_resource_usage_hints_stmt(stmt, domain_family, resources);
            }
            for stmt in else_body {
                collect_resource_usage_hints_stmt(stmt, domain_family, resources);
            }
        }
        NirStmt::While { condition, body } => {
            collect_resource_usage_hints_expr(condition, domain_family, resources);
            for stmt in body {
                collect_resource_usage_hints_stmt(stmt, domain_family, resources);
            }
        }
        NirStmt::Return(Some(value)) => {
            collect_resource_usage_hints_expr(value, domain_family, resources);
        }
        NirStmt::Return(None) | NirStmt::Break | NirStmt::Continue => {}
    }
}

fn collect_resource_usage_hints_expr(
    expr: &NirExpr,
    domain_family: &str,
    resources: &mut BTreeSet<String>,
) {
    if domain_family == "shader" {
        match expr {
            NirExpr::ShaderBinding {
                kind,
                layout,
                profile_contract,
                ..
            } => {
                resources.insert(format!("shader.binding.{kind}"));
                if let Some(layout) = layout {
                    resources.insert(format!("shader.binding.layout.{layout}"));
                }
                if let Some(profile_contract) = profile_contract {
                    resources.insert(format!("shader.binding.contract.{profile_contract}"));
                }
            }
            NirExpr::ShaderBindSet { .. } => {
                resources.insert("shader.binding.set".to_owned());
            }
            NirExpr::ShaderTexture2d { .. } => {
                resources.insert("shader.resource.texture2d".to_owned());
            }
            NirExpr::ShaderSampler { .. } => {
                resources.insert("shader.resource.sampler".to_owned());
            }
            _ => {}
        }
    }

    walk_child_exprs(expr, &mut |child| {
        collect_resource_usage_hints_expr(child, domain_family, resources);
    });
}

fn implied_slots_for_surface(domain_family: &str, surface: &str) -> Vec<String> {
    let slots: &[&str] = match (domain_family, surface) {
        ("shader", "shader.profile.render.v1") => &[
            "target",
            "viewport",
            "pipeline",
            "vertex_count",
            "instance_count",
            "pass_kind",
            "packet_field_count",
        ],
        ("shader", "shader.profile.draw.v1") => &[
            "target",
            "viewport",
            "pipeline",
            "vertex_count",
            "instance_count",
            "pass_kind",
            "packet_field_count",
        ],
        ("shader", "shader.profile.seed.color.v1") => {
            &["packet_color_slot", "slider_color_slot", "material_mode"]
        }
        ("shader", "shader.profile.seed.speed.v1") => {
            &["packet_speed_slot", "slider_speed_slot", "packet_tag"]
        }
        ("shader", "shader.profile.seed.radius.v1") => &[
            "packet_radius_slot",
            "slider_radius_slot",
            "packet_field_count",
        ],
        ("shader", "shader.profile.packet.v1") => &[
            "packet_color_slot",
            "packet_speed_slot",
            "packet_radius_slot",
            "slider_color_slot",
            "slider_speed_slot",
            "slider_radius_slot",
            "header_accent_slot",
            "toggle_live_slot",
            "focus_slot",
        ],
        ("shader", "shader.profile.packet.nova.v1") => &[
            "slider_color_slot",
            "slider_speed_slot",
            "slider_radius_slot",
            "header_accent_slot",
            "toggle_live_slot",
            "focus_slot",
        ],
        ("shader", "shader.profile.target.v1") => &["target"],
        ("shader", "shader.profile.viewport.v1") => &["viewport"],
        ("shader", "shader.profile.pipeline.v1") => &["pipeline"],
        ("shader", "shader.profile.draw-budget.v1") => &["vertex_count", "instance_count"],
        ("shader", "shader.profile.packet-slots.v1") => &[
            "packet_color_slot",
            "packet_speed_slot",
            "packet_radius_slot",
            "slider_color_slot",
            "slider_speed_slot",
            "slider_radius_slot",
            "header_accent_slot",
            "toggle_live_slot",
            "focus_slot",
        ],
        ("shader", "shader.profile.packet-tag.v1") => &["packet_tag"],
        ("shader", "shader.profile.material-mode.v1") => &["material_mode"],
        ("shader", "shader.profile.pass-kind.v1") => &["pass_kind"],
        ("shader", "shader.profile.packet-field-count.v1") => &["packet_field_count"],
        ("data", "data.profile.bind-core.v1") => &["bind_core"],
        ("data", "data.profile.send.uplink.v1") => {
            let mut slots = vec!["window_offset".to_owned(), "uplink_len".to_owned()];
            slots.extend(all_uplink_directional_marker_slots());
            slots.extend(data_common_marker_slots().iter().filter_map(|slot| {
                slot.starts_with("marker:uplink_")
                    .then_some((*slot).to_owned())
            }));
            return slots;
        }
        ("data", "data.profile.send.downlink.v1") => {
            let mut slots = vec!["window_offset".to_owned(), "downlink_len".to_owned()];
            slots.extend(all_downlink_directional_marker_slots());
            slots.extend(data_common_marker_slots().iter().filter_map(|slot| {
                slot.starts_with("marker:downlink_")
                    .then_some((*slot).to_owned())
            }));
            return slots;
        }
        ("data", "data.profile.handle-table.v1") => &["handle_table"],
        ("data", "data.profile.window-layout.v1") => {
            &["window_offset", "uplink_len", "downlink_len"]
        }
        ("data", "data.profile.sync-markers.v1") => return all_sync_marker_slots(),
        ("data", "data.profile.pipe-markers.v1") => &["marker:uplink_pipe", "marker:downlink_pipe"],
        ("data", "data.profile.pipe-class.v1") => {
            &["marker:uplink_pipe_class", "marker:downlink_pipe_class"]
        }
        ("data", "data.profile.payload-class.v1") => &[
            "marker:uplink_payload_class",
            "marker:downlink_payload_class",
        ],
        ("data", "data.profile.payload-shape.v1") => &[
            "marker:uplink_payload_shape",
            "marker:downlink_payload_shape",
        ],
        ("data", "data.profile.window-policy.v1") => &[
            "marker:uplink_window_policy",
            "marker:downlink_window_policy",
        ],
        ("network", "network.profile.bind-core.v1") => &["bind_core"],
        ("network", "network.profile.connect.v1") => {
            &["remote_port", "connect_timeout_ms", "endpoint_kind"]
        }
        ("network", "network.profile.accept.v1") => &[
            "local_port",
            "read_timeout_ms",
            "write_timeout_ms",
            "endpoint_kind",
        ],
        ("network", "network.profile.send.v1") => &["send_window", "stream_window"],
        ("network", "network.profile.recv.v1") => &["recv_window", "stream_window"],
        ("network", "network.profile.close.v1") => &[],
        ("network", "network.profile.timeout.v1") => &[
            "connect_timeout_ms",
            "read_timeout_ms",
            "write_timeout_ms",
            "timeout_budget",
        ],
        ("network", "network.profile.retry.v1") => &["retry_budget"],
        ("network", "network.profile.endpoint-kind.v1") => &["endpoint_kind"],
        ("network", "network.profile.stream-window.v1") => {
            &["stream_window", "recv_window", "send_window"]
        }
        ("network", "network.profile.transport.v1") => &["transport_family"],
        ("network", "network.profile.protocol.v1") => {
            &["protocol_kind", "protocol_version", "protocol_header_bytes"]
        }
        _ => &[],
    };
    slots.iter().map(|slot| (*slot).to_owned()).collect()
}

pub(crate) fn detect_matched_support_usage(
    module: &NirModule,
    domain_family: &str,
) -> (Vec<String>, Vec<String>) {
    let mut surfaces = BTreeSet::new();
    let mut slots = BTreeSet::new();
    for function in &module.functions {
        for stmt in &function.body {
            collect_support_usage_stmt(stmt, domain_family, &mut surfaces, &mut slots);
        }
    }
    (
        surfaces.into_iter().collect::<Vec<_>>(),
        slots.into_iter().collect::<Vec<_>>(),
    )
}

fn collect_support_usage_stmt(
    stmt: &NirStmt,
    domain_family: &str,
    surfaces: &mut BTreeSet<String>,
    slots: &mut BTreeSet<String>,
) {
    match stmt {
        NirStmt::Let { value, .. }
        | NirStmt::Const { value, .. }
        | NirStmt::Print(value)
        | NirStmt::Await(value)
        | NirStmt::Expr(value) => collect_support_usage_expr(value, domain_family, surfaces, slots),
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            collect_support_usage_expr(condition, domain_family, surfaces, slots);
            for stmt in then_body {
                collect_support_usage_stmt(stmt, domain_family, surfaces, slots);
            }
            for stmt in else_body {
                collect_support_usage_stmt(stmt, domain_family, surfaces, slots);
            }
        }
        NirStmt::While { condition, body } => {
            collect_support_usage_expr(condition, domain_family, surfaces, slots);
            for stmt in body {
                collect_support_usage_stmt(stmt, domain_family, surfaces, slots);
            }
        }
        NirStmt::Break | NirStmt::Continue => {}
        NirStmt::Return(value) => {
            if let Some(value) = value {
                collect_support_usage_expr(value, domain_family, surfaces, slots);
            }
        }
    }
}

fn collect_support_usage_expr(
    expr: &NirExpr,
    domain_family: &str,
    surfaces: &mut BTreeSet<String>,
    slots: &mut BTreeSet<String>,
) {
    match expr {
        NirExpr::ShaderProfileTargetRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.target.v1".to_owned());
            slots.insert("target".to_owned());
        }
        NirExpr::ShaderProfileViewportRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.viewport.v1".to_owned());
            slots.insert("viewport".to_owned());
        }
        NirExpr::ShaderProfilePipelineRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.pipeline.v1".to_owned());
            slots.insert("pipeline".to_owned());
        }
        NirExpr::ShaderProfileVertexCountRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.draw-budget.v1".to_owned());
            slots.insert("vertex_count".to_owned());
        }
        NirExpr::ShaderProfileInstanceCountRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.draw-budget.v1".to_owned());
            slots.insert("instance_count".to_owned());
        }
        NirExpr::ShaderProfilePacketColorSlotRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.packet-slots.v1".to_owned());
            slots.insert("packet_color_slot".to_owned());
        }
        NirExpr::ShaderProfilePacketSpeedSlotRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.packet-slots.v1".to_owned());
            slots.insert("packet_speed_slot".to_owned());
        }
        NirExpr::ShaderProfilePacketRadiusSlotRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.packet-slots.v1".to_owned());
            slots.insert("packet_radius_slot".to_owned());
        }
        NirExpr::ShaderProfileSliderColorSlotRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.packet-slots.v1".to_owned());
            surfaces.insert("shader.profile.packet.nova.v1".to_owned());
            slots.insert("slider_color_slot".to_owned());
        }
        NirExpr::ShaderProfileSliderSpeedSlotRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.packet-slots.v1".to_owned());
            surfaces.insert("shader.profile.packet.nova.v1".to_owned());
            slots.insert("slider_speed_slot".to_owned());
        }
        NirExpr::ShaderProfileSliderRadiusSlotRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.packet-slots.v1".to_owned());
            surfaces.insert("shader.profile.packet.nova.v1".to_owned());
            slots.insert("slider_radius_slot".to_owned());
        }
        NirExpr::ShaderProfileHeaderAccentSlotRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.packet-slots.v1".to_owned());
            surfaces.insert("shader.profile.packet.nova.v1".to_owned());
            slots.insert("header_accent_slot".to_owned());
        }
        NirExpr::ShaderProfileToggleLiveSlotRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.packet-slots.v1".to_owned());
            surfaces.insert("shader.profile.packet.nova.v1".to_owned());
            slots.insert("toggle_live_slot".to_owned());
        }
        NirExpr::ShaderProfileFocusSlotRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.packet-slots.v1".to_owned());
            surfaces.insert("shader.profile.packet.nova.v1".to_owned());
            slots.insert("focus_slot".to_owned());
        }
        NirExpr::ShaderProfilePacketTagRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.packet-tag.v1".to_owned());
            slots.insert("packet_tag".to_owned());
        }
        NirExpr::ShaderProfileMaterialModeRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.material-mode.v1".to_owned());
            slots.insert("material_mode".to_owned());
        }
        NirExpr::ShaderProfilePassKindRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.pass-kind.v1".to_owned());
            slots.insert("pass_kind".to_owned());
        }
        NirExpr::ShaderProfilePacketFieldCountRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.packet-field-count.v1".to_owned());
            slots.insert("packet_field_count".to_owned());
        }
        NirExpr::ShaderProfileColorSeed { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.seed.color.v1".to_owned());
        }
        NirExpr::ShaderProfileSpeedSeed { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.seed.speed.v1".to_owned());
        }
        NirExpr::ShaderProfileRadiusSeed { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.seed.radius.v1".to_owned());
        }
        NirExpr::ShaderProfilePacket {
            packet_type_name,
            accent,
            toggle_state,
            focus_index,
            ..
        } if domain_family == "shader" => {
            surfaces.insert("shader.profile.packet.v1".to_owned());
            let is_nova_panel = packet_type_name.as_deref() == Some("NovaPanelPacket")
                || accent.is_some()
                || toggle_state.is_some()
                || focus_index.is_some();
            if is_nova_panel {
                surfaces.insert("shader.profile.packet.nova.v1".to_owned());
            }
        }
        NirExpr::ShaderBinding {
            profile_contract: Some(profile_contract),
            ..
        } if domain_family == "shader" => {
            surfaces.insert(profile_contract.clone());
        }
        NirExpr::ShaderProfileRender { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.render.v1".to_owned());
        }
        NirExpr::ShaderDrawInstanced { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.draw.v1".to_owned());
        }
        NirExpr::ShaderInlineWgsl { .. } if domain_family == "shader" => {
            surfaces.insert("shader.inline.wgsl.v1".to_owned());
        }
        NirExpr::DataProfileBindCoreRef { .. } if domain_family == "data" => {
            surfaces.insert("data.profile.bind-core.v1".to_owned());
            slots.insert("bind_core".to_owned());
        }
        NirExpr::DataProfileWindowOffsetRef { .. } if domain_family == "data" => {
            surfaces.insert("data.profile.window-layout.v1".to_owned());
            slots.insert("window_offset".to_owned());
        }
        NirExpr::DataProfileUplinkLenRef { .. } if domain_family == "data" => {
            surfaces.insert("data.profile.window-layout.v1".to_owned());
            slots.insert("uplink_len".to_owned());
        }
        NirExpr::DataProfileDownlinkLenRef { .. } if domain_family == "data" => {
            surfaces.insert("data.profile.window-layout.v1".to_owned());
            slots.insert("downlink_len".to_owned());
        }
        NirExpr::DataProfileHandleTableRef { .. } if domain_family == "data" => {
            surfaces.insert("data.profile.handle-table.v1".to_owned());
            slots.insert("handle_table".to_owned());
        }
        NirExpr::DataProfileMarkerRef { tag, .. } if domain_family == "data" => {
            surfaces.insert(data_marker_surface(tag).to_owned());
            slots.insert(format!("marker:{tag}"));
        }
        NirExpr::NetworkProfileBindCoreRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.bind-core.v1".to_owned());
            slots.insert("bind_core".to_owned());
        }
        NirExpr::NetworkProfileEndpointKindRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.endpoint-kind.v1".to_owned());
            slots.insert("endpoint_kind".to_owned());
        }
        NirExpr::NetworkProfileTransportFamilyRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.transport.v1".to_owned());
            slots.insert("transport_family".to_owned());
        }
        NirExpr::NetworkProfileLocalPortRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.accept.v1".to_owned());
            slots.insert("local_port".to_owned());
        }
        NirExpr::NetworkProfileRemotePortRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.connect.v1".to_owned());
            slots.insert("remote_port".to_owned());
        }
        NirExpr::NetworkProfileConnectTimeoutRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.timeout.v1".to_owned());
            slots.insert("connect_timeout_ms".to_owned());
        }
        NirExpr::NetworkProfileReadTimeoutRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.timeout.v1".to_owned());
            slots.insert("read_timeout_ms".to_owned());
        }
        NirExpr::NetworkProfileWriteTimeoutRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.timeout.v1".to_owned());
            slots.insert("write_timeout_ms".to_owned());
        }
        NirExpr::NetworkProfileTimeoutBudgetRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.timeout.v1".to_owned());
            slots.insert("timeout_budget".to_owned());
        }
        NirExpr::NetworkProfileRetryBudgetRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.retry.v1".to_owned());
            slots.insert("retry_budget".to_owned());
        }
        NirExpr::NetworkProfileStreamWindowRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.stream-window.v1".to_owned());
            slots.insert("stream_window".to_owned());
        }
        NirExpr::NetworkProfileRecvWindowRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.recv.v1".to_owned());
            slots.insert("recv_window".to_owned());
        }
        NirExpr::NetworkProfileSendWindowRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.send.v1".to_owned());
            slots.insert("send_window".to_owned());
        }
        NirExpr::NetworkProfileProtocolKindRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.protocol.v1".to_owned());
            slots.insert("protocol_kind".to_owned());
        }
        NirExpr::NetworkProfileProtocolVersionRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.protocol.v1".to_owned());
            slots.insert("protocol_version".to_owned());
        }
        NirExpr::NetworkProfileProtocolHeaderBytesRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.protocol.v1".to_owned());
            slots.insert("protocol_header_bytes".to_owned());
        }
        NirExpr::KernelProfileBindCoreRef { .. } if domain_family == "kernel" => {
            surfaces.insert("kernel.profile.bind-core.v1".to_owned());
            slots.insert("bind_core".to_owned());
        }
        NirExpr::KernelProfileQueueDepthRef { .. } if domain_family == "kernel" => {
            surfaces.insert("kernel.profile.queue-depth.v1".to_owned());
            slots.insert("queue_depth".to_owned());
        }
        NirExpr::KernelProfileBatchLanesRef { .. } if domain_family == "kernel" => {
            surfaces.insert("kernel.profile.batch-lanes.v1".to_owned());
            slots.insert("batch_lanes".to_owned());
        }
        NirExpr::DataProfileSendUplink { .. } if domain_family == "data" => {
            surfaces.insert("data.profile.send.uplink.v1".to_owned());
        }
        NirExpr::DataProfileSendDownlink { .. } if domain_family == "data" => {
            surfaces.insert("data.profile.send.downlink.v1".to_owned());
        }
        _ => {}
    }

    walk_child_exprs(expr, &mut |child| {
        collect_support_usage_expr(child, domain_family, surfaces, slots);
    });
}
