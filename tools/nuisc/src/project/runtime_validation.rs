use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use nuis_semantics::model::{NirExpr, NirModule, NirStmt};
use yir_core::{EdgeKind, Node, Operation, OperationDomainFamily, SemanticOp, YirModule};

use super::abi::required_abi_surfaces_for_domain;
use super::bridge_contracts::required_project_link_stage_contract;
use super::data_validation::{validate_data_profile_for_link, validate_data_profile_token_types};
use super::kernel_validation::{
    nir_uses_kernel_profile_batch_lanes, nir_uses_kernel_profile_bind_core,
    nir_uses_kernel_profile_queue_depth, validate_kernel_profile_for_link,
};
use super::network_validation::{
    validate_network_profile_for_link, validate_network_profile_slot_contract,
};
use super::profile_refs::{
    resolve_project_profile_refs, stitch_data_profile_edges, stitch_shader_profile_edges,
};
use super::profile_targets::has_xfer_segment;
use super::profile_usage::{
    nir_uses_cpu_extern_call,
    nir_uses_data_profile_handle_table, nir_uses_data_profile_send_downlink,
    nir_uses_data_profile_send_uplink, nir_uses_network_profile_bind_core,
    nir_uses_network_profile_endpoint_kind, nir_uses_network_profile_slot,
    nir_uses_shader_profile_color_seed, nir_uses_shader_profile_draw_instanced,
    nir_uses_shader_profile_packet, nir_uses_shader_profile_radius_seed,
    nir_uses_shader_profile_render, nir_uses_shader_profile_speed_seed,
};
use super::shader_validation::validate_shader_profile_for_link;
use super::support_contracts::{require_declared_support_surface, support_surface_for_domain};
use super::{
    apply_support_module_profile, materialize_project_type_contract_nodes, resolve_project_abi,
    sanitize_ident, split_domain_unit, LoadedProject,
};

pub fn apply_project_links_to_yir(
    project: &LoadedProject,
    module: &mut YirModule,
) -> Result<(), String> {
    let mut required = BTreeSet::new();
    for link in &project.manifest.links {
        if !link.from.starts_with("cpu.") {
            continue;
        }
        let (target_domain, target_unit) = split_domain_unit(&link.to)?;
        required.insert((target_domain, target_unit));
        if let Some(via) = &link.via {
            let (via_domain, via_unit) = split_domain_unit(via)?;
            required.insert((via_domain, via_unit));
        }
    }

    for (domain, unit) in required {
        let exists = module.nodes.iter().any(|node| {
            node.op.is_cpu_semantic_op(SemanticOp::CpuInstantiateUnit)
                && node.op.args.first().map(String::as_str) == Some(domain.as_str())
                && node.op.args.get(1).map(String::as_str) == Some(unit.as_str())
        });
        if exists {
            continue;
        }
        let name = format!(
            "project_link_instantiate_{}_{}",
            sanitize_ident(&domain),
            sanitize_ident(&unit)
        );
        module.nodes.push(Node {
            name,
            resource: "cpu0".to_owned(),
            op: Operation {
                module: "cpu".to_owned(),
                instruction: "instantiate_unit".to_owned(),
                args: vec![domain, unit],
            },
        });
    }

    crate::lowering::assign_default_lanes(module);
    crate::lowering::materialize_registered_scheduler_contract_nodes(module);
    crate::lowering::assign_default_lanes(module);
    Ok(())
}

pub fn apply_project_support_modules_to_yir(
    project: &LoadedProject,
    module: &mut YirModule,
) -> Result<(), String> {
    for project_module in &project.modules {
        if project_module.path == project.entry_path {
            continue;
        }
        apply_support_module_profile(&project_module.ast, module)?;
    }
    materialize_project_type_contract_nodes(project, module)?;
    resolve_project_profile_refs(module)?;
    stitch_shader_profile_edges(module);
    stitch_data_profile_edges(module);
    crate::lowering::assign_default_lanes(module);
    crate::lowering::materialize_registered_scheduler_contract_nodes(module);
    crate::lowering::assign_default_lanes(module);
    Ok(())
}

pub fn validate_project_links_against_yir(
    project: &LoadedProject,
    module: &YirModule,
) -> Result<(), String> {
    let resource_families = module
        .resources
        .iter()
        .map(|resource| (resource.name.as_str(), resource.kind.family().to_owned()))
        .collect::<BTreeMap<_, _>>();
    let node_families = module
        .nodes
        .iter()
        .map(|node| {
            let family = resource_families
                .get(node.resource.as_str())
                .cloned()
                .unwrap_or_else(|| node.op.module.clone());
            (node.name.as_str(), family)
        })
        .collect::<BTreeMap<_, _>>();

    for link in &project.manifest.links {
        if let Some(via) = &link.via {
            let (via_domain, _via_unit) = split_domain_unit(via)?;
            if via_domain == "data" {
                let contract = required_project_link_stage_contract(&link.from, &link.to, via)?;
                let has_fabric = module
                    .resources
                    .iter()
                    .any(|resource| resource.kind.raw == "data.fabric");
                if !has_fabric {
                    return Err(format!(
                        "project link `{}` -> `{}` via `{}` requires a `data.fabric` resource in YIR",
                        link.from, link.to, via
                    ));
                }
                let has_data_plane = module
                    .nodes
                    .iter()
                    .any(|node| node.op.is_domain_family(OperationDomainFamily::Data));
                if !has_data_plane {
                    return Err(format!(
                        "project link `{}` -> `{}` via `{}` requires at least one `data.*` node in YIR",
                        link.from, link.to, via
                    ));
                }
                let has_cross_domain_xfer = module
                    .edges
                    .iter()
                    .any(|edge| edge.kind == EdgeKind::CrossDomainExchange);
                if !has_cross_domain_xfer {
                    return Err(format!(
                        "project link `{}` -> `{}` via `{}` requires at least one `xfer` edge in YIR",
                        link.from, link.to, via
                    ));
                }
                let (from_domain, _from_unit) = split_domain_unit(&link.from)?;
                let (to_domain, _to_unit) = split_domain_unit(&link.to)?;
                let has_uplink = has_xfer_segment(module, &node_families, &from_domain, "data");
                if !has_uplink {
                    return Err(format!(
                        "project link `{}` -> `{}` via `{}` requires a `{}` -> `data` xfer segment in YIR for uplink stage `{}`",
                        link.from, link.to, via, from_domain, contract.uplink.render()
                    ));
                }
                let has_downlink = has_xfer_segment(module, &node_families, "data", &to_domain);
                if !has_downlink {
                    return Err(format!(
                        "project link `{}` -> `{}` via `{}` requires a `data` -> `{}` xfer segment in YIR for downlink stage `{}`",
                        link.from, link.to, via, to_domain, contract.downlink.render()
                    ));
                }
                validate_data_profile_token_types(project, &link.from, &link.to, via)?;
                validate_data_profile_for_link(module, &link.from, &link.to, via)?;
            }
        }

        validate_shader_profile_for_link(project, module, &link.from)?;
        validate_shader_profile_for_link(project, module, &link.to)?;
        validate_kernel_profile_for_link(project, module, &link.from)?;
        validate_kernel_profile_for_link(project, module, &link.to)?;
        validate_network_profile_for_link(project, &link.from)?;
        validate_network_profile_for_link(project, &link.to)?;
    }
    Ok(())
}

pub fn validate_project_abi_against_yir(
    project: &LoadedProject,
    module: &YirModule,
) -> Result<(), String> {
    let resolution = resolve_project_abi(project)?;
    if resolution.requirements.is_empty() {
        return Ok(());
    }
    for requirement in &resolution.requirements {
        let manifest = crate::registry::load_manifest_for_domain(
            Path::new("nustar-packages"),
            &requirement.domain,
        )?;
        crate::registry::validate_manifest_abi(&manifest, &requirement.abi)?;
        let required_surfaces = required_abi_surfaces_for_domain(project, &requirement.domain)?;
        let used_ops = crate::registry::used_ops_for_domain(module, &requirement.domain);
        crate::registry::validate_abi_capabilities(
            &manifest,
            &requirement.abi,
            &required_surfaces,
            &used_ops,
        )?;
    }
    Ok(())
}

pub fn validate_project_links_against_nir(
    project: &LoadedProject,
    module: &NirModule,
) -> Result<(), String> {
    let mut support_surface_cache = BTreeMap::<String, BTreeSet<String>>::new();
    for link in &project.manifest.links {
        let (from_domain, _from_unit) = split_domain_unit(&link.from)?;
        let (to_domain, to_unit) = split_domain_unit(&link.to)?;
        if from_domain == "cpu" && to_domain == "shader" {
            let shader_support = support_surface_for_domain(&mut support_surface_cache, "shader")?;
            require_declared_support_surface(
                &shader_support,
                "shader",
                &to_unit,
                "shader.profile.packet.v1",
            )?;
            if !nir_uses_shader_profile_packet(module, &to_unit) {
                return Err(format!(
                    "project link `{}` -> `{}` requires CPU entry to use shader_profile_packet(\"{}\", ...) at NIR level",
                    link.from, link.to, to_unit
                ));
            }
            let uses_shader_render = nir_uses_shader_profile_render(module, &to_unit);
            let uses_shader_draw = nir_uses_shader_profile_draw_instanced(module, &to_unit);
            if uses_shader_render {
                require_declared_support_surface(
                    &shader_support,
                    "shader",
                    &to_unit,
                    "shader.profile.render.v1",
                )?;
            }
            if uses_shader_draw {
                require_declared_support_surface(
                    &shader_support,
                    "shader",
                    &to_unit,
                    "shader.profile.draw.v1",
                )?;
            }
            if !uses_shader_render && !uses_shader_draw {
                return Err(format!(
                    "project link `{}` -> `{}` requires CPU entry to use shader_profile_render(\"{}\", ...) or shader_profile_draw_instanced(\"{}\", ...) at NIR level",
                    link.from, link.to, to_unit, to_unit
                ));
            }
            require_declared_support_surface(
                &shader_support,
                "shader",
                &to_unit,
                "shader.profile.seed.color.v1",
            )?;
            if !nir_uses_shader_profile_color_seed(module, &to_unit) {
                return Err(format!(
                    "project link `{}` -> `{}` requires CPU entry to use shader_profile_color_seed(\"{}\", ...) at NIR level",
                    link.from, link.to, to_unit
                ));
            }
            require_declared_support_surface(
                &shader_support,
                "shader",
                &to_unit,
                "shader.profile.seed.speed.v1",
            )?;
            if !nir_uses_shader_profile_speed_seed(module, &to_unit) {
                return Err(format!(
                    "project link `{}` -> `{}` requires CPU entry to use shader_profile_speed_seed(\"{}\", ...) at NIR level",
                    link.from, link.to, to_unit
                ));
            }
            require_declared_support_surface(
                &shader_support,
                "shader",
                &to_unit,
                "shader.profile.seed.radius.v1",
            )?;
            if !nir_uses_shader_profile_radius_seed(module, &to_unit) {
                return Err(format!(
                    "project link `{}` -> `{}` requires CPU entry to use shader_profile_radius_seed(\"{}\", ...) at NIR level",
                    link.from, link.to, to_unit
                ));
            }
        }
        if from_domain == "cpu" && to_domain == "kernel" {
            let kernel_support = support_surface_for_domain(&mut support_surface_cache, "kernel")?;
            require_declared_support_surface(
                &kernel_support,
                "kernel",
                &to_unit,
                "kernel.profile.bind-core.v1",
            )?;
            if !nir_uses_kernel_profile_bind_core(module, &to_unit) {
                return Err(format!(
                    "project link `{}` -> `{}` requires CPU entry to use kernel_profile_bind_core(\"{}\") at NIR level",
                    link.from, link.to, to_unit
                ));
            }
            require_declared_support_surface(
                &kernel_support,
                "kernel",
                &to_unit,
                "kernel.profile.queue-depth.v1",
            )?;
            if !nir_uses_kernel_profile_queue_depth(module, &to_unit) {
                return Err(format!(
                    "project link `{}` -> `{}` requires CPU entry to use kernel_profile_queue_depth(\"{}\") at NIR level",
                    link.from, link.to, to_unit
                ));
            }
            require_declared_support_surface(
                &kernel_support,
                "kernel",
                &to_unit,
                "kernel.profile.batch-lanes.v1",
            )?;
            if !nir_uses_kernel_profile_batch_lanes(module, &to_unit) {
                return Err(format!(
                    "project link `{}` -> `{}` requires CPU entry to use kernel_profile_batch_lanes(\"{}\") at NIR level",
                    link.from, link.to, to_unit
                ));
            }
        }
        if from_domain == "cpu" && to_domain == "network" {
            let network_support =
                support_surface_for_domain(&mut support_surface_cache, "network")?;
            require_declared_support_surface(
                &network_support,
                "network",
                &to_unit,
                "network.profile.bind-core.v1",
            )?;
            if !nir_uses_network_profile_bind_core(module, &to_unit) {
                return Err(format!(
                    "project link `{}` -> `{}` requires CPU entry to use network_profile_bind_core(\"{}\") at NIR level",
                    link.from, link.to, to_unit
                ));
            }
            require_declared_support_surface(
                &network_support,
                "network",
                &to_unit,
                "network.profile.endpoint-kind.v1",
            )?;
            if !nir_uses_network_profile_endpoint_kind(module, &to_unit) {
                return Err(format!(
                    "project link `{}` -> `{}` requires CPU entry to use network_profile_endpoint_kind(\"{}\") at NIR level",
                    link.from, link.to, to_unit
                ));
            }
            validate_network_profile_slot_requirements(
                project,
                module,
                &network_support,
                &link.from,
                &link.to,
                &to_unit,
            )?;
            validate_network_host_call_requirements(
                project,
                module,
                &network_support,
                &link.from,
                &link.to,
                &to_unit,
            )?;
        }
        if from_domain == "cpu" && to_domain == "data" {
            let data_support = support_surface_for_domain(&mut support_surface_cache, "data")?;
            require_declared_support_surface(
                &data_support,
                "data",
                &to_unit,
                "data.profile.handle-table.v1",
            )?;
            if !nir_uses_data_profile_handle_table(module, &to_unit) {
                return Err(format!(
                    "project link `{}` -> `{}` requires CPU entry to use data_handle_table(\"{}\", ...) at NIR level",
                    link.from, link.to, to_unit
                ));
            }
            require_declared_support_surface(
                &data_support,
                "data",
                &to_unit,
                "data.profile.send-uplink.v1",
            )?;
            if !nir_uses_data_profile_send_uplink(module, &to_unit) {
                return Err(format!(
                    "project link `{}` -> `{}` requires CPU entry to use data_send_uplink(\"{}\", ...) at NIR level",
                    link.from, link.to, to_unit
                ));
            }
            require_declared_support_surface(
                &data_support,
                "data",
                &to_unit,
                "data.profile.send-downlink.v1",
            )?;
            if !nir_uses_data_profile_send_downlink(module, &to_unit) {
                return Err(format!(
                    "project link `{}` -> `{}` requires CPU entry to use data_send_downlink(\"{}\", ...) at NIR level",
                    link.from, link.to, to_unit
                ));
            }
        }
    }
    Ok(())
}

fn validate_network_host_call_requirements(
    project: &LoadedProject,
    module: &NirModule,
    network_support: &BTreeSet<String>,
    from: &str,
    to: &str,
    unit: &str,
) -> Result<(), String> {
    validate_network_owned_handle_shape(module, from, to)?;
    validate_network_owned_handle_provenance(module, from, to)?;
    validate_network_host_call(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "host_network_connect_probe",
        &["local_port", "remote_port", "connect_timeout_ms"],
        &["network.profile.connect.v1"],
    )?;
    validate_network_host_call(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "host_network_open_tcp_stream",
        &["remote_port", "connect_timeout_ms"],
        &["network.profile.connect.v1"],
    )?;
    validate_network_host_call(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "host_network_open_udp_datagram",
        &["local_port", "remote_port"],
        &["network.profile.connect.v1"],
    )?;
    validate_network_host_call(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "host_network_open_tcp_listener",
        &["local_port", "read_timeout_ms", "write_timeout_ms"],
        &["network.profile.accept.v1", "network.profile.timeout.v1"],
    )?;
    validate_network_host_call(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "host_network_bind_udp_datagram",
        &["local_port", "read_timeout_ms", "write_timeout_ms"],
        &["network.profile.accept.v1", "network.profile.timeout.v1"],
    )?;
    validate_network_host_call(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "host_network_accept_probe",
        &["local_port", "read_timeout_ms", "write_timeout_ms"],
        &["network.profile.accept.v1", "network.profile.timeout.v1"],
    )?;
    validate_network_host_call(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "host_network_accept_owned",
        &["read_timeout_ms", "write_timeout_ms"],
        &["network.profile.accept.v1", "network.profile.timeout.v1"],
    )?;
    validate_network_host_call(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "host_network_send_probe",
        &["stream_window", "send_window", "remote_port"],
        &["network.profile.send.v1", "network.profile.stream-window.v1"],
    )?;
    validate_network_host_call(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "host_network_send_owned",
        &["stream_window", "send_window"],
        &["network.profile.send.v1", "network.profile.stream-window.v1"],
    )?;
    validate_network_host_call(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "host_network_recv_probe",
        &["stream_window", "recv_window", "local_port"],
        &["network.profile.recv.v1", "network.profile.stream-window.v1"],
    )?;
    validate_network_host_call(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "host_network_recv_owned",
        &["stream_window", "recv_window"],
        &["network.profile.recv.v1", "network.profile.stream-window.v1"],
    )?;
    validate_network_host_call(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "host_network_recv_http_status_owned",
        &[
            "stream_window",
            "recv_window",
            "protocol_kind",
            "protocol_version",
            "protocol_header_bytes",
        ],
        &[
            "network.profile.recv.v1",
            "network.profile.stream-window.v1",
            "network.profile.protocol.v1",
        ],
    )?;
    validate_network_host_call(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "host_network_close",
        &[],
        &["network.profile.close.v1"],
    )?;
    validate_network_host_call(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "host_network_close_owned",
        &[],
        &["network.profile.close.v1"],
    )?;
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum NetworkOwnedHandleKind {
    Listener,
    StreamTransport,
    DatagramTransport,
}

fn validate_network_owned_handle_shape(
    module: &NirModule,
    from: &str,
    to: &str,
) -> Result<(), String> {
    let uses_open_tcp_stream = nir_uses_cpu_extern_call(module, "host_network_open_tcp_stream");
    let uses_open_udp_datagram = nir_uses_cpu_extern_call(module, "host_network_open_udp_datagram");
    let uses_bind_udp_datagram = nir_uses_cpu_extern_call(module, "host_network_bind_udp_datagram");
    let uses_open_tcp_listener =
        nir_uses_cpu_extern_call(module, "host_network_open_tcp_listener");
    let uses_accept_owned = nir_uses_cpu_extern_call(module, "host_network_accept_owned");
    let uses_send_owned = nir_uses_cpu_extern_call(module, "host_network_send_owned");
    let uses_recv_owned = nir_uses_cpu_extern_call(module, "host_network_recv_owned");
    let uses_recv_http_status_owned =
        nir_uses_cpu_extern_call(module, "host_network_recv_http_status_owned");
    let uses_close_owned = nir_uses_cpu_extern_call(module, "host_network_close_owned");

    let has_transport_owned_source =
        uses_open_tcp_stream || uses_open_udp_datagram || uses_bind_udp_datagram || uses_accept_owned;
    let has_any_owned_source = has_transport_owned_source || uses_open_tcp_listener;

    if uses_accept_owned && !uses_open_tcp_listener {
        return Err(format!(
            "project link `{}` -> `{}` requires CPU entry to open a listener via `host_network_open_tcp_listener(...)` before `host_network_accept_owned(...)`",
            from, to
        ));
    }
    if uses_send_owned && !has_transport_owned_source {
        return Err(format!(
            "project link `{}` -> `{}` requires CPU entry to establish an owned transport handle before `host_network_send_owned(...)`",
            from, to
        ));
    }
    if uses_recv_owned && !has_transport_owned_source {
        return Err(format!(
            "project link `{}` -> `{}` requires CPU entry to establish an owned transport handle before `host_network_recv_owned(...)`",
            from, to
        ));
    }
    if uses_recv_http_status_owned && !has_transport_owned_source {
        return Err(format!(
            "project link `{}` -> `{}` requires CPU entry to establish an owned transport handle before `host_network_recv_http_status_owned(...)`",
            from, to
        ));
    }
    if uses_close_owned && !has_any_owned_source {
        return Err(format!(
            "project link `{}` -> `{}` requires CPU entry to establish an owned network handle before `host_network_close_owned(...)`",
            from, to
        ));
    }

    Ok(())
}

fn validate_network_owned_handle_provenance(
    module: &NirModule,
    from: &str,
    to: &str,
) -> Result<(), String> {
    for function in &module.functions {
        let mut bindings = BTreeMap::new();
        validate_network_owned_handle_provenance_in_body(
            &function.body,
            from,
            to,
            &mut bindings,
        )?;
    }
    Ok(())
}

fn validate_network_owned_handle_provenance_in_body(
    body: &[NirStmt],
    from: &str,
    to: &str,
    bindings: &mut BTreeMap<String, NetworkOwnedHandleKind>,
) -> Result<(), String> {
    for stmt in body {
        match stmt {
            NirStmt::Let { name, value, .. } => {
                if let Some(kind) = infer_network_owned_handle_kind(value, bindings) {
                    bindings.insert(name.clone(), kind);
                }
                validate_network_owned_handle_provenance_in_expr(value, from, to, bindings)?;
            }
            NirStmt::Const { name, value, .. } => {
                if let Some(kind) = infer_network_owned_handle_kind(value, bindings) {
                    bindings.insert(name.clone(), kind);
                }
                validate_network_owned_handle_provenance_in_expr(value, from, to, bindings)?;
            }
            NirStmt::Print(value)
            | NirStmt::Await(value)
            | NirStmt::Expr(value)
            | NirStmt::Return(Some(value)) => {
                validate_network_owned_handle_provenance_in_expr(value, from, to, bindings)?;
            }
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                validate_network_owned_handle_provenance_in_expr(condition, from, to, bindings)?;
                let mut then_bindings = bindings.clone();
                validate_network_owned_handle_provenance_in_body(
                    then_body,
                    from,
                    to,
                    &mut then_bindings,
                )?;
                let mut else_bindings = bindings.clone();
                validate_network_owned_handle_provenance_in_body(
                    else_body,
                    from,
                    to,
                    &mut else_bindings,
                )?;
            }
            NirStmt::While { condition, body } => {
                validate_network_owned_handle_provenance_in_expr(condition, from, to, bindings)?;
                let mut loop_bindings = bindings.clone();
                validate_network_owned_handle_provenance_in_body(
                    body,
                    from,
                    to,
                    &mut loop_bindings,
                )?;
            }
            NirStmt::Return(None) | NirStmt::Break | NirStmt::Continue => {}
        }
    }
    Ok(())
}

fn validate_network_owned_handle_provenance_in_expr(
    expr: &NirExpr,
    from: &str,
    to: &str,
    bindings: &BTreeMap<String, NetworkOwnedHandleKind>,
) -> Result<(), String> {
    match expr {
        NirExpr::CpuExternCall { callee, args, .. } => {
            validate_network_owned_handle_call(callee, args, from, to, bindings)?;
            for arg in args {
                validate_network_owned_handle_provenance_in_expr(arg, from, to, bindings)?;
            }
        }
        NirExpr::Await(inner)
        | NirExpr::Borrow(inner)
        | NirExpr::BorrowEnd(inner)
        | NirExpr::Move(inner)
        | NirExpr::LoadValue(inner)
        | NirExpr::LoadNext(inner)
        | NirExpr::BufferLen(inner)
        | NirExpr::CpuJoin(inner)
        | NirExpr::CpuCancel(inner)
        | NirExpr::CpuJoinResult(inner)
        | NirExpr::CpuTaskCompleted(inner)
        | NirExpr::CpuTaskTimedOut(inner)
        | NirExpr::CpuTaskCancelled(inner)
        | NirExpr::CpuTaskValue(inner)
        | NirExpr::DataReady(inner)
        | NirExpr::DataMoved(inner)
        | NirExpr::DataWindowed(inner)
        | NirExpr::DataValue(inner)
        | NirExpr::DataFreezeWindow(inner)
        | NirExpr::ShaderPassReady(inner)
        | NirExpr::ShaderFrameReady(inner)
        | NirExpr::ShaderValue(inner)
        | NirExpr::NetworkConfigReady(inner)
        | NirExpr::NetworkSendReady(inner)
        | NirExpr::NetworkRecvReady(inner)
        | NirExpr::NetworkAcceptReady(inner)
        | NirExpr::NetworkValue(inner)
        | NirExpr::KernelConfigReady(inner)
        | NirExpr::KernelValue(inner)
        | NirExpr::DataOutputPipe(inner)
        | NirExpr::DataInputPipe(inner)
        | NirExpr::CpuPresentFrame(inner)
        | NirExpr::Free(inner)
        | NirExpr::IsNull(inner) => {
            validate_network_owned_handle_provenance_in_expr(inner, from, to, bindings)?;
        }
        NirExpr::DataResult { value, .. }
        | NirExpr::ShaderResult { value, .. }
        | NirExpr::NetworkResult { value, .. }
        | NirExpr::KernelResult { value, .. } => {
            validate_network_owned_handle_provenance_in_expr(value, from, to, bindings)?;
        }
        NirExpr::AllocNode { value, next } => {
            validate_network_owned_handle_provenance_in_expr(value, from, to, bindings)?;
            validate_network_owned_handle_provenance_in_expr(next, from, to, bindings)?;
        }
        NirExpr::AllocBuffer { len, fill } => {
            validate_network_owned_handle_provenance_in_expr(len, from, to, bindings)?;
            validate_network_owned_handle_provenance_in_expr(fill, from, to, bindings)?;
        }
        NirExpr::LoadAt { buffer, index } => {
            validate_network_owned_handle_provenance_in_expr(buffer, from, to, bindings)?;
            validate_network_owned_handle_provenance_in_expr(index, from, to, bindings)?;
        }
        NirExpr::DataReadWindow { window, index } => {
            validate_network_owned_handle_provenance_in_expr(window, from, to, bindings)?;
            validate_network_owned_handle_provenance_in_expr(index, from, to, bindings)?;
        }
        NirExpr::DataWriteWindow {
            window,
            index,
            value,
        } => {
            validate_network_owned_handle_provenance_in_expr(window, from, to, bindings)?;
            validate_network_owned_handle_provenance_in_expr(index, from, to, bindings)?;
            validate_network_owned_handle_provenance_in_expr(value, from, to, bindings)?;
        }
        NirExpr::StoreValue { target, value } => {
            validate_network_owned_handle_provenance_in_expr(target, from, to, bindings)?;
            validate_network_owned_handle_provenance_in_expr(value, from, to, bindings)?;
        }
        NirExpr::StoreNext { target, next } => {
            validate_network_owned_handle_provenance_in_expr(target, from, to, bindings)?;
            validate_network_owned_handle_provenance_in_expr(next, from, to, bindings)?;
        }
        NirExpr::StoreAt {
            buffer,
            index,
            value,
        } => {
            validate_network_owned_handle_provenance_in_expr(buffer, from, to, bindings)?;
            validate_network_owned_handle_provenance_in_expr(index, from, to, bindings)?;
            validate_network_owned_handle_provenance_in_expr(value, from, to, bindings)?;
        }
        NirExpr::DataCopyWindow { input, offset, len }
        | NirExpr::DataImmutableWindow { input, offset, len } => {
            validate_network_owned_handle_provenance_in_expr(input, from, to, bindings)?;
            validate_network_owned_handle_provenance_in_expr(offset, from, to, bindings)?;
            validate_network_owned_handle_provenance_in_expr(len, from, to, bindings)?;
        }
        NirExpr::DataProfileSendUplink { input, .. }
        | NirExpr::DataProfileSendDownlink { input, .. } => {
            validate_network_owned_handle_provenance_in_expr(input, from, to, bindings)?;
        }
        NirExpr::ShaderProfileColorSeed { base, delta, .. }
        | NirExpr::ShaderProfileRadiusSeed { base, delta, .. } => {
            validate_network_owned_handle_provenance_in_expr(base, from, to, bindings)?;
            validate_network_owned_handle_provenance_in_expr(delta, from, to, bindings)?;
        }
        NirExpr::ShaderProfileSpeedSeed {
            delta, scale, base, ..
        } => {
            validate_network_owned_handle_provenance_in_expr(delta, from, to, bindings)?;
            validate_network_owned_handle_provenance_in_expr(scale, from, to, bindings)?;
            validate_network_owned_handle_provenance_in_expr(base, from, to, bindings)?;
        }
        NirExpr::ShaderProfilePacket {
            color,
            speed,
            radius,
            ..
        } => {
            validate_network_owned_handle_provenance_in_expr(color, from, to, bindings)?;
            validate_network_owned_handle_provenance_in_expr(speed, from, to, bindings)?;
            validate_network_owned_handle_provenance_in_expr(radius, from, to, bindings)?;
        }
        NirExpr::CpuSpawn { args, .. } | NirExpr::Call { args, .. } => {
            for arg in args {
                validate_network_owned_handle_provenance_in_expr(arg, from, to, bindings)?;
            }
        }
        NirExpr::CpuTimeout { task, limit } => {
            validate_network_owned_handle_provenance_in_expr(task, from, to, bindings)?;
            validate_network_owned_handle_provenance_in_expr(limit, from, to, bindings)?;
        }
        NirExpr::MethodCall { receiver, args, .. } => {
            validate_network_owned_handle_provenance_in_expr(receiver, from, to, bindings)?;
            for arg in args {
                validate_network_owned_handle_provenance_in_expr(arg, from, to, bindings)?;
            }
        }
        NirExpr::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                validate_network_owned_handle_provenance_in_expr(value, from, to, bindings)?;
            }
        }
        NirExpr::FieldAccess { base, .. } => {
            validate_network_owned_handle_provenance_in_expr(base, from, to, bindings)?;
        }
        NirExpr::Binary { lhs, rhs, .. } => {
            validate_network_owned_handle_provenance_in_expr(lhs, from, to, bindings)?;
            validate_network_owned_handle_provenance_in_expr(rhs, from, to, bindings)?;
        }
        NirExpr::ShaderBeginPass {
            target,
            pipeline,
            viewport,
        } => {
            validate_network_owned_handle_provenance_in_expr(target, from, to, bindings)?;
            validate_network_owned_handle_provenance_in_expr(pipeline, from, to, bindings)?;
            validate_network_owned_handle_provenance_in_expr(viewport, from, to, bindings)?;
        }
        NirExpr::ShaderDrawInstanced {
            pass,
            packet,
            vertex_count,
            instance_count,
        } => {
            validate_network_owned_handle_provenance_in_expr(pass, from, to, bindings)?;
            validate_network_owned_handle_provenance_in_expr(packet, from, to, bindings)?;
            validate_network_owned_handle_provenance_in_expr(vertex_count, from, to, bindings)?;
            validate_network_owned_handle_provenance_in_expr(instance_count, from, to, bindings)?;
        }
        NirExpr::ShaderProfileRender { packet, .. } => {
            validate_network_owned_handle_provenance_in_expr(packet, from, to, bindings)?;
        }
        _ => {}
    }
    Ok(())
}

fn validate_network_owned_handle_call(
    callee: &str,
    args: &[NirExpr],
    from: &str,
    to: &str,
    bindings: &BTreeMap<String, NetworkOwnedHandleKind>,
) -> Result<(), String> {
    match callee {
        "host_network_accept_owned" => {
            validate_network_owned_handle_arg(
                callee,
                args.first(),
                NetworkOwnedHandleKind::Listener,
                from,
                to,
                bindings,
                "listener",
            )?;
        }
        "host_network_send_owned"
        | "host_network_recv_owned" => {
            validate_network_transport_handle_arg(callee, args.first(), from, to, bindings)?;
        }
        "host_network_recv_http_status_owned" => {
            validate_network_owned_handle_arg(
                callee,
                args.first(),
                NetworkOwnedHandleKind::StreamTransport,
                from,
                to,
                bindings,
                "stream transport",
            )?;
        }
        "host_network_close_owned" => {
            let Some(arg) = args.first() else {
                return Ok(());
            };
            match arg {
                NirExpr::Var(name) if bindings.contains_key(name) => {}
                NirExpr::Var(name) => {
                    return Err(format!(
                        "project link `{}` -> `{}` requires `host_network_close_owned(...)` to consume an owned handle variable, but `{}` does not come from an owned network open/accept path",
                        from, to, name
                    ));
                }
                _ => {
                    return Err(format!(
                        "project link `{}` -> `{}` requires `host_network_close_owned(...)` to consume an owned handle variable produced by an open/accept path",
                        from, to
                    ));
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_network_owned_handle_arg(
    callee: &str,
    arg: Option<&NirExpr>,
    expected: NetworkOwnedHandleKind,
    from: &str,
    to: &str,
    bindings: &BTreeMap<String, NetworkOwnedHandleKind>,
    expected_label: &str,
) -> Result<(), String> {
    let Some(arg) = arg else {
        return Ok(());
    };
    match arg {
        NirExpr::Var(name) => match bindings.get(name).copied() {
            Some(kind) if kind == expected => Ok(()),
            Some(NetworkOwnedHandleKind::DatagramTransport)
                if expected == NetworkOwnedHandleKind::StreamTransport =>
            {
                Err(format!(
                    "project link `{}` -> `{}` requires `{}` to consume a {} handle variable, but `{}` comes from a datagram-owned source",
                    from, to, callee, expected_label, name
                ))
            }
            Some(NetworkOwnedHandleKind::Listener) => Err(format!(
                "project link `{}` -> `{}` requires `{}` to consume a {} handle variable, but `{}` comes from a listener-owned source",
                from, to, callee, expected_label, name
            )),
            Some(NetworkOwnedHandleKind::StreamTransport) => Err(format!(
                "project link `{}` -> `{}` requires `{}` to consume a {} handle variable, but `{}` comes from a stream-owned source",
                from, to, callee, expected_label, name
            )),
            Some(NetworkOwnedHandleKind::DatagramTransport) => Err(format!(
                "project link `{}` -> `{}` requires `{}` to consume a {} handle variable, but `{}` comes from a datagram-owned source",
                from, to, callee, expected_label, name
            )),
            None => Err(format!(
                "project link `{}` -> `{}` requires `{}` to consume a {} handle variable, but `{}` does not come from an owned network open/accept path",
                from, to, callee, expected_label, name
            )),
        },
        _ => Err(format!(
            "project link `{}` -> `{}` requires `{}` to consume a {} handle variable produced by an owned network open/accept path",
            from, to, callee, expected_label
        )),
    }
}

fn validate_network_transport_handle_arg(
    callee: &str,
    arg: Option<&NirExpr>,
    from: &str,
    to: &str,
    bindings: &BTreeMap<String, NetworkOwnedHandleKind>,
) -> Result<(), String> {
    let Some(arg) = arg else {
        return Ok(());
    };
    match arg {
        NirExpr::Var(name) => match bindings.get(name).copied() {
            Some(NetworkOwnedHandleKind::StreamTransport)
            | Some(NetworkOwnedHandleKind::DatagramTransport) => Ok(()),
            Some(NetworkOwnedHandleKind::Listener) => Err(format!(
                "project link `{}` -> `{}` requires `{}` to consume a transport handle variable, but `{}` comes from a listener-owned source",
                from, to, callee, name
            )),
            None => Err(format!(
                "project link `{}` -> `{}` requires `{}` to consume a transport handle variable, but `{}` does not come from an owned network open/accept path",
                from, to, callee, name
            )),
        },
        _ => Err(format!(
            "project link `{}` -> `{}` requires `{}` to consume a transport handle variable produced by an owned network open/accept path",
            from, to, callee
        )),
    }
}

fn infer_network_owned_handle_kind(
    expr: &NirExpr,
    bindings: &BTreeMap<String, NetworkOwnedHandleKind>,
) -> Option<NetworkOwnedHandleKind> {
    match expr {
        NirExpr::CpuExternCall { callee, .. } => match callee.as_str() {
            "host_network_open_tcp_listener" => Some(NetworkOwnedHandleKind::Listener),
            "host_network_open_tcp_stream" => Some(NetworkOwnedHandleKind::StreamTransport),
            "host_network_open_udp_datagram" | "host_network_bind_udp_datagram" => {
                Some(NetworkOwnedHandleKind::DatagramTransport)
            }
            "host_network_accept_owned" => Some(NetworkOwnedHandleKind::StreamTransport),
            _ => None,
        },
        NirExpr::NetworkValue(inner) => infer_network_owned_handle_kind(inner, bindings),
        NirExpr::NetworkResult { value, .. } => infer_network_owned_handle_kind(value, bindings),
        NirExpr::Var(name) => bindings.get(name).copied(),
        _ => None,
    }
}

fn validate_network_host_call(
    project: &LoadedProject,
    module: &NirModule,
    network_support: &BTreeSet<String>,
    from: &str,
    to: &str,
    unit: &str,
    host_symbol: &'static str,
    required_slots: &[&str],
    required_surfaces: &[&str],
) -> Result<(), String> {
    if !nir_uses_cpu_extern_call(module, host_symbol) {
        return Ok(());
    }
    for surface in required_surfaces {
        require_declared_support_surface(network_support, "network", unit, surface)?;
    }
    validate_network_profile_slot_contract(project, unit, required_slots)?;
    for slot in required_slots {
        if !nir_uses_network_profile_slot(module, unit, slot) {
            let builtin_name = network_profile_builtin_name(slot);
            return Err(format!(
                "project link `{}` -> `{}` requires CPU entry to route `{}` through {}(\"{}\")",
                from, to, host_symbol, builtin_name, unit
            ));
        }
    }
    Ok(())
}

fn network_profile_builtin_name(slot: &str) -> &str {
    match slot {
        "local_port" => "network_profile_local_port",
        "remote_port" => "network_profile_remote_port",
        "connect_timeout_ms" => "network_profile_connect_timeout",
        "read_timeout_ms" => "network_profile_read_timeout",
        "write_timeout_ms" => "network_profile_write_timeout",
        "stream_window" => "network_profile_stream_window",
        "recv_window" => "network_profile_recv_window",
        "send_window" => "network_profile_send_window",
        "protocol_kind" => "network_profile_protocol_kind",
        "protocol_version" => "network_profile_protocol_version",
        other => other,
    }
}

fn validate_network_profile_slot_requirements(
    project: &LoadedProject,
    module: &NirModule,
    network_support: &BTreeSet<String>,
    from: &str,
    to: &str,
    unit: &str,
) -> Result<(), String> {
    validate_network_slot_usage(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "transport_family",
        "network.profile.transport.v1",
        "network_profile_transport_family",
    )?;
    validate_network_slot_usage(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "local_port",
        "network.profile.connect.v1",
        "network_profile_local_port",
    )?;
    validate_network_slot_usage(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "remote_port",
        "network.profile.connect.v1",
        "network_profile_remote_port",
    )?;
    validate_network_slot_usage(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "connect_timeout_ms",
        "network.profile.connect.v1",
        "network_profile_connect_timeout",
    )?;
    validate_network_slot_usage(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "read_timeout_ms",
        "network.profile.timeout.v1",
        "network_profile_read_timeout",
    )?;
    validate_network_slot_usage(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "write_timeout_ms",
        "network.profile.timeout.v1",
        "network_profile_write_timeout",
    )?;
    validate_network_slot_usage(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "retry_budget",
        "network.profile.retry.v1",
        "network_profile_retry_budget",
    )?;
    validate_network_slot_usage(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "stream_window",
        "network.profile.stream-window.v1",
        "network_profile_stream_window",
    )?;
    validate_network_slot_usage(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "recv_window",
        "network.profile.recv.v1",
        "network_profile_recv_window",
    )?;
    validate_network_slot_usage(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "send_window",
        "network.profile.send.v1",
        "network_profile_send_window",
    )?;
    validate_network_slot_usage(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "protocol_kind",
        "network.profile.protocol.v1",
        "network_profile_protocol_kind",
    )?;
    validate_network_slot_usage(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "protocol_version",
        "network.profile.protocol.v1",
        "network_profile_protocol_version",
    )?;
    validate_network_slot_usage(
        project,
        module,
        network_support,
        from,
        to,
        unit,
        "protocol_header_bytes",
        "network.profile.protocol.v1",
        "network_profile_protocol_header_bytes",
    )?;
    Ok(())
}

fn validate_network_slot_usage(
    project: &LoadedProject,
    module: &NirModule,
    network_support: &BTreeSet<String>,
    from: &str,
    to: &str,
    unit: &str,
    slot: &'static str,
    required_surface: &'static str,
    builtin_name: &'static str,
) -> Result<(), String> {
    if !nir_uses_network_profile_slot(module, unit, slot) {
        return Ok(());
    }
    require_declared_support_surface(network_support, "network", unit, required_surface)?;
    validate_network_profile_slot_contract(project, unit, &[slot])?;
    let rendered = format!("{builtin_name}(\"{unit}\")");
    if !nir_uses_network_profile_slot(module, unit, slot) {
        return Err(format!(
            "project link `{}` -> `{}` requires CPU entry to use {} at NIR level",
            from, to, rendered
        ));
    }
    Ok(())
}
