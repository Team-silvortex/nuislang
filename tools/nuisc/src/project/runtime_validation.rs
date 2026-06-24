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
    nir_uses_cpu_extern_call, nir_uses_data_profile_handle_table,
    nir_uses_data_profile_send_downlink, nir_uses_data_profile_send_uplink,
    nir_uses_network_profile_bind_core, nir_uses_network_profile_endpoint_kind,
    nir_uses_network_profile_slot, nir_uses_shader_binding_profile_contract,
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
    let abi_resolution = resolve_project_abi(project)?;
    for project_module in &project.modules {
        if project_module.path == project.entry_path {
            continue;
        }
        apply_support_module_profile(&project_module.ast, module, Some(&abi_resolution))?;
    }
    materialize_project_type_contract_nodes(project, &abi_resolution, module)?;
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
        validate_network_profile_for_link(project, module, &link.from)?;
        validate_network_profile_for_link(project, module, &link.to)?;
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

pub fn prune_project_topology_for_codegen(
    project: &LoadedProject,
    module: &mut YirModule,
) -> Result<(), String> {
    let _ = project;
    let resource_families = module
        .resources
        .iter()
        .map(|resource| (resource.name.clone(), resource.kind.family().to_owned()))
        .collect::<BTreeMap<_, _>>();
    let nodes = module
        .nodes
        .iter()
        .map(|node| (node.name.clone(), node))
        .collect::<BTreeMap<_, _>>();
    let node_families = module
        .nodes
        .iter()
        .map(|node| {
            let family = resource_families
                .get(&node.resource)
                .cloned()
                .unwrap_or_else(|| node.op.module.clone());
            (node.name.clone(), family)
        })
        .collect::<BTreeMap<_, _>>();

    module.edges.retain(|edge| {
        if edge.kind != EdgeKind::CrossDomainExchange {
            return true;
        }
        let from_family = node_families.get(&edge.from).map(String::as_str);
        let to_family = node_families.get(&edge.to).map(String::as_str);
        let touches_data = from_family == Some("data") || to_family == Some("data");
        if !touches_data {
            return true;
        }
        let Some(target) = nodes.get(&edge.to) else {
            return true;
        };
        target.op.args.iter().any(|arg| {
            arg == &edge.from
                || arg
                    .split_once('=')
                    .map(|(_, value)| value == edge.from)
                    .unwrap_or(false)
        })
    });
    let removable = module
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "text"
                && node.name.starts_with("project_")
        })
        .map(|node| node.name.clone())
        .collect::<BTreeSet<_>>();
    if !removable.is_empty() {
        module.nodes.retain(|node| !removable.contains(&node.name));
        module
            .edges
            .retain(|edge| !removable.contains(&edge.from) && !removable.contains(&edge.to));
    }
    Ok(())
}

pub fn validate_project_links_against_nir(
    project: &LoadedProject,
    module: &NirModule,
) -> Result<(), String> {
    let mut support_surface_cache = BTreeMap::<String, BTreeSet<String>>::new();
    for link in &project.manifest.links {
        let (from_domain, from_unit) = split_domain_unit(&link.from)?;
        let (to_domain, to_unit) = split_domain_unit(&link.to)?;
        if let Some(via) = &link.via {
            let (via_domain, via_unit) = split_domain_unit(via)?;
            if via_domain == "data" {
                let cpu_endpoint = if from_domain == "cpu" {
                    Some(link.from.as_str())
                } else if to_domain == "cpu" {
                    Some(link.to.as_str())
                } else {
                    None
                };
                if let Some(cpu_endpoint) = cpu_endpoint {
                    validate_data_profile_nir_usage(
                        module,
                        &mut support_surface_cache,
                        cpu_endpoint,
                        via,
                        &via_unit,
                    )?;
                }
            }
        }
        if let Some(network_unit) =
            cpu_network_link_unit(&from_domain, &link.from, &to_domain, &link.to)?
        {
            let network_support =
                support_surface_for_domain(&mut support_surface_cache, "network")?;
            require_declared_support_surface(
                &network_support,
                "network",
                &network_unit,
                "network.profile.bind-core.v1",
            )?;
            if !nir_uses_network_profile_bind_core(module, &network_unit) {
                return Err(format!(
                    "project link `{}` -> `{}` requires CPU entry to use network_profile_bind_core(\"{}\") at NIR level",
                    link.from, link.to, network_unit
                ));
            }
            require_declared_support_surface(
                &network_support,
                "network",
                &network_unit,
                "network.profile.endpoint-kind.v1",
            )?;
            if !nir_uses_network_profile_endpoint_kind(module, &network_unit) {
                return Err(format!(
                    "project link `{}` -> `{}` requires CPU entry to use network_profile_endpoint_kind(\"{}\") at NIR level",
                    link.from, link.to, network_unit
                ));
            }
            validate_network_profile_slot_requirements(
                project,
                module,
                &network_support,
                &link.from,
                &link.to,
                &network_unit,
            )?;
            validate_network_host_call_requirements(
                project,
                module,
                &network_support,
                &link.from,
                &link.to,
                &network_unit,
            )?;
        }
        let shader_unit = if from_domain == "shader" && to_domain == "cpu" {
            Some(from_unit.as_str())
        } else if from_domain == "cpu" && to_domain == "shader" {
            Some(to_unit.as_str())
        } else {
            None
        };
        if let Some(shader_unit) = shader_unit {
            validate_shader_profile_nir_usage(
                module,
                &mut support_surface_cache,
                &link.from,
                &link.to,
                shader_unit,
            )?;
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
        if from_domain == "cpu" && to_domain == "data" {
            validate_data_profile_nir_usage(
                module,
                &mut support_surface_cache,
                &link.from,
                &link.to,
                &to_unit,
            )?;
        }
    }
    Ok(())
}

fn cpu_network_link_unit(
    from_domain: &str,
    from: &str,
    to_domain: &str,
    to: &str,
) -> Result<Option<String>, String> {
    if from_domain == "cpu" && to_domain == "network" {
        return split_domain_unit(to).map(|(_, unit)| Some(unit));
    }
    if from_domain == "network" && to_domain == "cpu" {
        return split_domain_unit(from).map(|(_, unit)| Some(unit));
    }
    Ok(None)
}

fn validate_data_profile_nir_usage(
    module: &NirModule,
    support_surface_cache: &mut BTreeMap<String, BTreeSet<String>>,
    from: &str,
    data_endpoint: &str,
    unit: &str,
) -> Result<(), String> {
    let data_support = support_surface_for_domain(support_surface_cache, "data")?;
    require_declared_support_surface(&data_support, "data", unit, "data.profile.handle-table.v1")?;
    if !nir_uses_data_profile_handle_table(module, unit) {
        return Err(format!(
            "project link `{from}` -> `{data_endpoint}` requires CPU entry to use data_handle_table(\"{unit}\", ...) at NIR level"
        ));
    }
    require_declared_support_surface(&data_support, "data", unit, "data.profile.send.uplink.v1")?;
    if !nir_uses_data_profile_send_uplink(module, unit) {
        return Err(format!(
            "project link `{from}` -> `{data_endpoint}` requires CPU entry to use data_send_uplink(\"{unit}\", ...) at NIR level"
        ));
    }
    require_declared_support_surface(&data_support, "data", unit, "data.profile.send.downlink.v1")?;
    if !nir_uses_data_profile_send_downlink(module, unit) {
        return Err(format!(
            "project link `{from}` -> `{data_endpoint}` requires CPU entry to use data_send_downlink(\"{unit}\", ...) at NIR level"
        ));
    }
    Ok(())
}

fn validate_shader_profile_nir_usage(
    module: &NirModule,
    support_surface_cache: &mut BTreeMap<String, BTreeSet<String>>,
    from: &str,
    to: &str,
    shader_unit: &str,
) -> Result<(), String> {
    let shader_support = support_surface_for_domain(support_surface_cache, "shader")?;
    require_declared_support_surface(
        &shader_support,
        "shader",
        shader_unit,
        "shader.profile.packet.v1",
    )?;
    if !nir_uses_shader_profile_packet(module, shader_unit) {
        return Err(format!(
            "project link `{from}` -> `{to}` requires CPU entry to use shader_profile_packet(\"{shader_unit}\", ...) at NIR level"
        ));
    }
    if nir_uses_shader_binding_profile_contract(module, "shader.profile.packet.nova.v1") {
        require_declared_support_surface(
            &shader_support,
            "shader",
            shader_unit,
            "shader.profile.packet.nova.v1",
        )?;
    }

    let uses_shader_render = nir_uses_shader_profile_render(module, shader_unit);
    let uses_shader_draw = nir_uses_shader_profile_draw_instanced(module, shader_unit);
    if uses_shader_render {
        require_declared_support_surface(
            &shader_support,
            "shader",
            shader_unit,
            "shader.profile.render.v1",
        )?;
    }
    if uses_shader_draw {
        require_declared_support_surface(
            &shader_support,
            "shader",
            shader_unit,
            "shader.profile.draw.v1",
        )?;
    }
    if !uses_shader_render && !uses_shader_draw {
        return Err(format!(
            "project link `{from}` -> `{to}` requires CPU entry to use shader_profile_render(\"{shader_unit}\", ...) or shader_profile_draw_instanced(\"{shader_unit}\", ...) at NIR level"
        ));
    }

    require_declared_support_surface(
        &shader_support,
        "shader",
        shader_unit,
        "shader.profile.seed.color.v1",
    )?;
    if !nir_uses_shader_profile_color_seed(module, shader_unit) {
        return Err(format!(
            "project link `{from}` -> `{to}` requires CPU entry to use shader_profile_color_seed(\"{shader_unit}\", ...) at NIR level"
        ));
    }
    require_declared_support_surface(
        &shader_support,
        "shader",
        shader_unit,
        "shader.profile.seed.speed.v1",
    )?;
    if !nir_uses_shader_profile_speed_seed(module, shader_unit) {
        return Err(format!(
            "project link `{from}` -> `{to}` requires CPU entry to use shader_profile_speed_seed(\"{shader_unit}\", ...) at NIR level"
        ));
    }
    require_declared_support_surface(
        &shader_support,
        "shader",
        shader_unit,
        "shader.profile.seed.radius.v1",
    )?;
    if !nir_uses_shader_profile_radius_seed(module, shader_unit) {
        return Err(format!(
            "project link `{from}` -> `{to}` requires CPU entry to use shader_profile_radius_seed(\"{shader_unit}\", ...) at NIR level"
        ));
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
        &[
            "network.profile.send.v1",
            "network.profile.stream-window.v1",
        ],
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
        &[
            "network.profile.send.v1",
            "network.profile.stream-window.v1",
        ],
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
        &[
            "network.profile.recv.v1",
            "network.profile.stream-window.v1",
        ],
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
        &[
            "network.profile.recv.v1",
            "network.profile.stream-window.v1",
        ],
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum NetworkOwnedHandleBinding {
    Concrete(NetworkOwnedHandleKind),
    Param {
        index: usize,
        requirement: NetworkOwnedHandleRequirement,
    },
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum NetworkOwnedHandleRequirement {
    OwnedAny,
    Listener,
    Transport,
    StreamTransport,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum NetworkOwnedHandleReturn {
    Concrete(NetworkOwnedHandleKind),
    ParamIndex(usize),
}

fn validate_network_owned_handle_shape(
    module: &NirModule,
    from: &str,
    to: &str,
) -> Result<(), String> {
    let uses_open_tcp_stream = nir_uses_cpu_extern_call(module, "host_network_open_tcp_stream");
    let uses_open_udp_datagram = nir_uses_cpu_extern_call(module, "host_network_open_udp_datagram");
    let uses_bind_udp_datagram = nir_uses_cpu_extern_call(module, "host_network_bind_udp_datagram");
    let uses_open_tcp_listener = nir_uses_cpu_extern_call(module, "host_network_open_tcp_listener");
    let uses_accept_owned = nir_uses_cpu_extern_call(module, "host_network_accept_owned");
    let uses_send_owned = nir_uses_cpu_extern_call(module, "host_network_send_owned");
    let uses_recv_owned = nir_uses_cpu_extern_call(module, "host_network_recv_owned");
    let uses_recv_http_status_owned =
        nir_uses_cpu_extern_call(module, "host_network_recv_http_status_owned");
    let uses_close_owned = nir_uses_cpu_extern_call(module, "host_network_close_owned");

    let has_transport_owned_source = uses_open_tcp_stream
        || uses_open_udp_datagram
        || uses_bind_udp_datagram
        || uses_accept_owned;
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
    let function_requirements = infer_network_function_handle_requirements(module)?;
    let function_return_kinds =
        infer_network_function_return_kinds(module, &function_requirements)?;
    for function in &module.functions {
        let mut bindings = BTreeMap::new();
        seed_network_param_bindings(function, &function_requirements, &mut bindings);
        validate_network_owned_handle_provenance_in_body(
            &function.body,
            from,
            to,
            &mut bindings,
            &function_requirements,
            &function_return_kinds,
        )?;
    }
    Ok(())
}

fn validate_network_owned_handle_provenance_in_body(
    body: &[NirStmt],
    from: &str,
    to: &str,
    bindings: &mut BTreeMap<String, NetworkOwnedHandleBinding>,
    function_requirements: &BTreeMap<String, Vec<Option<NetworkOwnedHandleRequirement>>>,
    function_return_kinds: &BTreeMap<String, Option<NetworkOwnedHandleReturn>>,
) -> Result<(), String> {
    for stmt in body {
        match stmt {
            NirStmt::Let { name, value, .. } => {
                if let Some(kind) =
                    infer_network_owned_handle_kind(value, bindings, function_return_kinds)
                {
                    bindings.insert(name.clone(), kind);
                } else {
                    bindings.remove(name);
                }
                validate_network_owned_handle_provenance_in_expr(
                    value,
                    from,
                    to,
                    bindings,
                    function_requirements,
                    function_return_kinds,
                )?;
            }
            NirStmt::Const { name, value, .. } => {
                if let Some(kind) =
                    infer_network_owned_handle_kind(value, bindings, function_return_kinds)
                {
                    bindings.insert(name.clone(), kind);
                } else {
                    bindings.remove(name);
                }
                validate_network_owned_handle_provenance_in_expr(
                    value,
                    from,
                    to,
                    bindings,
                    function_requirements,
                    function_return_kinds,
                )?;
            }
            NirStmt::Print(value)
            | NirStmt::Await(value)
            | NirStmt::Expr(value)
            | NirStmt::Return(Some(value)) => {
                validate_network_owned_handle_provenance_in_expr(
                    value,
                    from,
                    to,
                    bindings,
                    function_requirements,
                    function_return_kinds,
                )?;
            }
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                validate_network_owned_handle_provenance_in_expr(
                    condition,
                    from,
                    to,
                    bindings,
                    function_requirements,
                    function_return_kinds,
                )?;
                let mut then_bindings = bindings.clone();
                validate_network_owned_handle_provenance_in_body(
                    then_body,
                    from,
                    to,
                    &mut then_bindings,
                    function_requirements,
                    function_return_kinds,
                )?;
                let mut else_bindings = bindings.clone();
                validate_network_owned_handle_provenance_in_body(
                    else_body,
                    from,
                    to,
                    &mut else_bindings,
                    function_requirements,
                    function_return_kinds,
                )?;
                merge_network_owned_handle_bindings(bindings, &then_bindings, &else_bindings);
            }
            NirStmt::While { condition, body } => {
                validate_network_owned_handle_provenance_in_expr(
                    condition,
                    from,
                    to,
                    bindings,
                    function_requirements,
                    function_return_kinds,
                )?;
                let entry_bindings = bindings.clone();
                let mut loop_bindings = bindings.clone();
                validate_network_owned_handle_provenance_in_body(
                    body,
                    from,
                    to,
                    &mut loop_bindings,
                    function_requirements,
                    function_return_kinds,
                )?;
                merge_network_owned_handle_bindings(bindings, &entry_bindings, &loop_bindings);
            }
            NirStmt::Return(None) | NirStmt::Break | NirStmt::Continue => {}
        }
    }
    Ok(())
}

fn merge_network_owned_handle_bindings(
    bindings: &mut BTreeMap<String, NetworkOwnedHandleBinding>,
    then_bindings: &BTreeMap<String, NetworkOwnedHandleBinding>,
    else_bindings: &BTreeMap<String, NetworkOwnedHandleBinding>,
) {
    let merged = bindings
        .keys()
        .chain(then_bindings.keys())
        .chain(else_bindings.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    for name in merged {
        match (
            then_bindings.get(&name).copied(),
            else_bindings.get(&name).copied(),
        ) {
            (Some(lhs), Some(rhs)) if lhs == rhs => {
                bindings.insert(name, lhs);
            }
            _ => {
                bindings.remove(&name);
            }
        }
    }
}

fn validate_network_owned_handle_provenance_in_expr(
    expr: &NirExpr,
    from: &str,
    to: &str,
    bindings: &BTreeMap<String, NetworkOwnedHandleBinding>,
    function_requirements: &BTreeMap<String, Vec<Option<NetworkOwnedHandleRequirement>>>,
    function_return_kinds: &BTreeMap<String, Option<NetworkOwnedHandleReturn>>,
) -> Result<(), String> {
    match expr {
        NirExpr::CpuExternCall { callee, args, .. } => {
            validate_network_owned_handle_call(callee, args, from, to, bindings)?;
            for arg in args {
                validate_network_owned_handle_provenance_in_expr(
                    arg,
                    from,
                    to,
                    bindings,
                    function_requirements,
                    function_return_kinds,
                )?;
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
            validate_network_owned_handle_provenance_in_expr(
                inner,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        NirExpr::DataResult { value, .. }
        | NirExpr::ShaderResult { value, .. }
        | NirExpr::NetworkResult { value, .. }
        | NirExpr::KernelResult { value, .. } => {
            validate_network_owned_handle_provenance_in_expr(
                value,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        NirExpr::AllocNode { value, next } => {
            validate_network_owned_handle_provenance_in_expr(
                value,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                next,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        NirExpr::AllocBuffer { len, fill } => {
            validate_network_owned_handle_provenance_in_expr(
                len,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                fill,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        NirExpr::LoadAt { buffer, index } => {
            validate_network_owned_handle_provenance_in_expr(
                buffer,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                index,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        NirExpr::DataReadWindow { window, index } => {
            validate_network_owned_handle_provenance_in_expr(
                window,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                index,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        NirExpr::DataWriteWindow {
            window,
            index,
            value,
        } => {
            validate_network_owned_handle_provenance_in_expr(
                window,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                index,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                value,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        NirExpr::StoreValue { target, value } => {
            validate_network_owned_handle_provenance_in_expr(
                target,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                value,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        NirExpr::StoreNext { target, next } => {
            validate_network_owned_handle_provenance_in_expr(
                target,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                next,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        NirExpr::StoreAt {
            buffer,
            index,
            value,
        } => {
            validate_network_owned_handle_provenance_in_expr(
                buffer,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                index,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                value,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        NirExpr::DataCopyWindow { input, offset, len }
        | NirExpr::DataImmutableWindow { input, offset, len } => {
            validate_network_owned_handle_provenance_in_expr(
                input,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                offset,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                len,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        NirExpr::DataProfileSendUplink { input, .. }
        | NirExpr::DataProfileSendDownlink { input, .. } => {
            validate_network_owned_handle_provenance_in_expr(
                input,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        NirExpr::ShaderProfileColorSeed { base, delta, .. }
        | NirExpr::ShaderProfileRadiusSeed { base, delta, .. } => {
            validate_network_owned_handle_provenance_in_expr(
                base,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                delta,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        NirExpr::ShaderProfileSpeedSeed {
            delta, scale, base, ..
        } => {
            validate_network_owned_handle_provenance_in_expr(
                delta,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                scale,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                base,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        NirExpr::ShaderProfilePacket {
            color,
            speed,
            radius,
            ..
        } => {
            validate_network_owned_handle_provenance_in_expr(
                color,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                speed,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                radius,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        NirExpr::CpuSpawn { callee, args }
        | NirExpr::CpuThreadSpawn { callee, args }
        | NirExpr::Call { callee, args } => {
            validate_network_function_call_requirements(
                callee,
                args,
                from,
                to,
                bindings,
                function_requirements,
            )?;
            for arg in args {
                validate_network_owned_handle_provenance_in_expr(
                    arg,
                    from,
                    to,
                    bindings,
                    function_requirements,
                    function_return_kinds,
                )?;
            }
        }
        NirExpr::CpuTimeout { task, limit } => {
            validate_network_owned_handle_provenance_in_expr(
                task,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                limit,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        NirExpr::MethodCall { receiver, args, .. } => {
            validate_network_owned_handle_provenance_in_expr(
                receiver,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            for arg in args {
                validate_network_owned_handle_provenance_in_expr(
                    arg,
                    from,
                    to,
                    bindings,
                    function_requirements,
                    function_return_kinds,
                )?;
            }
        }
        NirExpr::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                validate_network_owned_handle_provenance_in_expr(
                    value,
                    from,
                    to,
                    bindings,
                    function_requirements,
                    function_return_kinds,
                )?;
            }
        }
        NirExpr::FieldAccess { base, .. } => {
            validate_network_owned_handle_provenance_in_expr(
                base,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        NirExpr::Binary { lhs, rhs, .. } => {
            validate_network_owned_handle_provenance_in_expr(
                lhs,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                rhs,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        NirExpr::ShaderBeginPass {
            target,
            pipeline,
            viewport,
        } => {
            validate_network_owned_handle_provenance_in_expr(
                target,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                pipeline,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                viewport,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        NirExpr::ShaderDrawInstanced {
            pass,
            packet,
            vertex_count,
            instance_count,
        } => {
            validate_network_owned_handle_provenance_in_expr(
                pass,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                packet,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                vertex_count,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                instance_count,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        NirExpr::ShaderProfileRender { packet, .. } => {
            validate_network_owned_handle_provenance_in_expr(
                packet,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        _ => {}
    }
    Ok(())
}

fn validate_network_function_call_requirements(
    callee: &str,
    args: &[NirExpr],
    from: &str,
    to: &str,
    bindings: &BTreeMap<String, NetworkOwnedHandleBinding>,
    function_requirements: &BTreeMap<String, Vec<Option<NetworkOwnedHandleRequirement>>>,
) -> Result<(), String> {
    let Some(requirements) = function_requirements.get(callee) else {
        return Ok(());
    };
    for (index, requirement) in requirements.iter().enumerate() {
        let Some(requirement) = requirement else {
            continue;
        };
        let arg = args.get(index);
        validate_network_call_arg_requirement(
            callee,
            index,
            arg,
            *requirement,
            from,
            to,
            bindings,
        )?;
    }
    Ok(())
}

fn validate_network_call_arg_requirement(
    callee: &str,
    index: usize,
    arg: Option<&NirExpr>,
    requirement: NetworkOwnedHandleRequirement,
    from: &str,
    to: &str,
    bindings: &BTreeMap<String, NetworkOwnedHandleBinding>,
) -> Result<(), String> {
    match requirement {
        NetworkOwnedHandleRequirement::OwnedAny => {
            let Some(arg) = arg else {
                return Ok(());
            };
            match arg {
                NirExpr::Var(name) if bindings.contains_key(name) => Ok(()),
                NirExpr::Var(name) => Err(format!(
                    "project link `{}` -> `{}` requires call `{}` arg {} to be an owned network handle variable, but `{}` does not come from an owned network open/accept path",
                    from, to, callee, index, name
                )),
                _ => Err(format!(
                    "project link `{}` -> `{}` requires call `{}` arg {} to be an owned network handle variable produced by an open/accept path",
                    from, to, callee, index
                )),
            }
        }
        NetworkOwnedHandleRequirement::Listener => validate_network_owned_handle_arg(
            callee,
            arg,
            NetworkOwnedHandleKind::Listener,
            from,
            to,
            bindings,
            "listener",
        ),
        NetworkOwnedHandleRequirement::Transport => {
            validate_network_transport_handle_arg(callee, arg, from, to, bindings)
        }
        NetworkOwnedHandleRequirement::StreamTransport => validate_network_owned_handle_arg(
            callee,
            arg,
            NetworkOwnedHandleKind::StreamTransport,
            from,
            to,
            bindings,
            "stream transport",
        ),
    }
}

fn validate_network_owned_handle_call(
    callee: &str,
    args: &[NirExpr],
    from: &str,
    to: &str,
    bindings: &BTreeMap<String, NetworkOwnedHandleBinding>,
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
        "host_network_send_owned" | "host_network_recv_owned" => {
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
    bindings: &BTreeMap<String, NetworkOwnedHandleBinding>,
    expected_label: &str,
) -> Result<(), String> {
    let Some(arg) = arg else {
        return Ok(());
    };
    match arg {
        NirExpr::Var(name) => match bindings.get(name).copied() {
            Some(NetworkOwnedHandleBinding::Concrete(kind)) if kind == expected => Ok(()),
            Some(NetworkOwnedHandleBinding::Param {
                requirement: NetworkOwnedHandleRequirement::Listener,
                ..
            }) if expected == NetworkOwnedHandleKind::Listener => Ok(()),
            Some(NetworkOwnedHandleBinding::Param {
                requirement: NetworkOwnedHandleRequirement::StreamTransport,
                ..
            }) if expected == NetworkOwnedHandleKind::StreamTransport => Ok(()),
            Some(NetworkOwnedHandleBinding::Concrete(NetworkOwnedHandleKind::DatagramTransport))
                if expected == NetworkOwnedHandleKind::StreamTransport =>
            {
                Err(format!(
                    "project link `{}` -> `{}` requires `{}` to consume a {} handle variable, but `{}` comes from a datagram-owned source",
                    from, to, callee, expected_label, name
                ))
            }
            Some(NetworkOwnedHandleBinding::Concrete(NetworkOwnedHandleKind::Listener))
            | Some(NetworkOwnedHandleBinding::Param {
                requirement: NetworkOwnedHandleRequirement::Listener,
                ..
            }) => Err(format!(
                "project link `{}` -> `{}` requires `{}` to consume a {} handle variable, but `{}` comes from a listener-owned source",
                from, to, callee, expected_label, name
            )),
            Some(NetworkOwnedHandleBinding::Concrete(NetworkOwnedHandleKind::StreamTransport))
            | Some(NetworkOwnedHandleBinding::Param {
                requirement: NetworkOwnedHandleRequirement::StreamTransport,
                ..
            }) => Err(format!(
                "project link `{}` -> `{}` requires `{}` to consume a {} handle variable, but `{}` comes from a stream-owned source",
                from, to, callee, expected_label, name
            )),
            Some(NetworkOwnedHandleBinding::Concrete(NetworkOwnedHandleKind::DatagramTransport)) => {
                Err(format!(
                    "project link `{}` -> `{}` requires `{}` to consume a {} handle variable, but `{}` comes from a datagram-owned source",
                    from, to, callee, expected_label, name
                ))
            }
            Some(NetworkOwnedHandleBinding::Param {
                requirement: NetworkOwnedHandleRequirement::Transport,
                ..
            }) => Err(format!(
                "project link `{}` -> `{}` requires `{}` to consume a {} handle variable, but `{}` only guarantees a generic transport-owned source",
                from, to, callee, expected_label, name
            )),
            Some(NetworkOwnedHandleBinding::Param {
                requirement: NetworkOwnedHandleRequirement::OwnedAny,
                ..
            }) => Err(format!(
                "project link `{}` -> `{}` requires `{}` to consume a {} handle variable, but `{}` only guarantees an owned network source",
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
    bindings: &BTreeMap<String, NetworkOwnedHandleBinding>,
) -> Result<(), String> {
    let Some(arg) = arg else {
        return Ok(());
    };
    match arg {
        NirExpr::Var(name) => match bindings.get(name).copied() {
            Some(NetworkOwnedHandleBinding::Concrete(NetworkOwnedHandleKind::StreamTransport))
            | Some(NetworkOwnedHandleBinding::Concrete(NetworkOwnedHandleKind::DatagramTransport))
            | Some(NetworkOwnedHandleBinding::Param {
                requirement: NetworkOwnedHandleRequirement::Transport,
                ..
            })
            | Some(NetworkOwnedHandleBinding::Param {
                requirement: NetworkOwnedHandleRequirement::StreamTransport,
                ..
            }) => Ok(()),
            Some(NetworkOwnedHandleBinding::Concrete(NetworkOwnedHandleKind::Listener))
            | Some(NetworkOwnedHandleBinding::Param {
                requirement: NetworkOwnedHandleRequirement::Listener,
                ..
            }) => Err(format!(
                "project link `{}` -> `{}` requires `{}` to consume a transport handle variable, but `{}` comes from a listener-owned source",
                from, to, callee, name
            )),
            Some(NetworkOwnedHandleBinding::Param {
                requirement: NetworkOwnedHandleRequirement::OwnedAny,
                ..
            }) => Err(format!(
                "project link `{}` -> `{}` requires `{}` to consume a transport handle variable, but `{}` only guarantees an owned network source",
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
    bindings: &BTreeMap<String, NetworkOwnedHandleBinding>,
    function_return_kinds: &BTreeMap<String, Option<NetworkOwnedHandleReturn>>,
) -> Option<NetworkOwnedHandleBinding> {
    match expr {
        NirExpr::CpuExternCall { callee, .. } => match callee.as_str() {
            "host_network_open_tcp_listener" => Some(NetworkOwnedHandleBinding::Concrete(
                NetworkOwnedHandleKind::Listener,
            )),
            "host_network_open_tcp_stream" => Some(NetworkOwnedHandleBinding::Concrete(
                NetworkOwnedHandleKind::StreamTransport,
            )),
            "host_network_open_udp_datagram" | "host_network_bind_udp_datagram" => Some(
                NetworkOwnedHandleBinding::Concrete(NetworkOwnedHandleKind::DatagramTransport),
            ),
            "host_network_accept_owned" => Some(NetworkOwnedHandleBinding::Concrete(
                NetworkOwnedHandleKind::StreamTransport,
            )),
            _ => None,
        },
        NirExpr::Call { callee, args } => function_return_kinds
            .get(callee)
            .copied()
            .flatten()
            .and_then(|summary| {
                resolve_network_owned_handle_return(summary, args, bindings, function_return_kinds)
            }),
        NirExpr::NetworkValue(inner) => {
            infer_network_owned_handle_kind(inner, bindings, function_return_kinds)
        }
        NirExpr::NetworkResult { value, .. } => {
            infer_network_owned_handle_kind(value, bindings, function_return_kinds)
        }
        NirExpr::Var(name) => bindings.get(name).copied(),
        _ => None,
    }
}

fn resolve_network_owned_handle_return(
    summary: NetworkOwnedHandleReturn,
    args: &[NirExpr],
    bindings: &BTreeMap<String, NetworkOwnedHandleBinding>,
    function_return_kinds: &BTreeMap<String, Option<NetworkOwnedHandleReturn>>,
) -> Option<NetworkOwnedHandleBinding> {
    match summary {
        NetworkOwnedHandleReturn::Concrete(kind) => Some(NetworkOwnedHandleBinding::Concrete(kind)),
        NetworkOwnedHandleReturn::ParamIndex(index) => args
            .get(index)
            .and_then(|arg| infer_network_owned_handle_kind(arg, bindings, function_return_kinds)),
    }
}

fn infer_network_function_handle_requirements(
    module: &NirModule,
) -> Result<BTreeMap<String, Vec<Option<NetworkOwnedHandleRequirement>>>, String> {
    let mut requirements = module
        .functions
        .iter()
        .map(|function| (function.name.clone(), vec![None; function.params.len()]))
        .collect::<BTreeMap<_, _>>();
    let mut changed = true;
    while changed {
        changed = false;
        for function in &module.functions {
            let mut next = requirements
                .get(&function.name)
                .cloned()
                .unwrap_or_else(|| vec![None; function.params.len()]);
            infer_network_param_requirements_in_body(
                &function.body,
                &function.params,
                &mut next,
                &requirements,
            )?;
            if requirements.get(&function.name) != Some(&next) {
                requirements.insert(function.name.clone(), next);
                changed = true;
            }
        }
    }
    Ok(requirements)
}

fn infer_network_function_return_kinds(
    module: &NirModule,
    function_requirements: &BTreeMap<String, Vec<Option<NetworkOwnedHandleRequirement>>>,
) -> Result<BTreeMap<String, Option<NetworkOwnedHandleReturn>>, String> {
    let mut return_kinds = module
        .functions
        .iter()
        .map(|function| (function.name.clone(), None))
        .collect::<BTreeMap<_, _>>();
    let mut changed = true;
    while changed {
        changed = false;
        for function in &module.functions {
            let mut bindings = BTreeMap::new();
            seed_network_param_bindings(function, function_requirements, &mut bindings);
            let next = infer_network_return_kind_in_body(
                &function.body,
                &mut bindings,
                function_requirements,
                &return_kinds,
            )?;
            if return_kinds.get(&function.name) != Some(&next) {
                return_kinds.insert(function.name.clone(), next);
                changed = true;
            }
        }
    }
    Ok(return_kinds)
}

fn infer_network_return_kind_in_body(
    body: &[NirStmt],
    bindings: &mut BTreeMap<String, NetworkOwnedHandleBinding>,
    function_requirements: &BTreeMap<String, Vec<Option<NetworkOwnedHandleRequirement>>>,
    function_return_kinds: &BTreeMap<String, Option<NetworkOwnedHandleReturn>>,
) -> Result<Option<NetworkOwnedHandleReturn>, String> {
    let mut return_kind = None;
    for stmt in body {
        match stmt {
            NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. } => {
                if let Some(kind) =
                    infer_network_owned_handle_kind(value, bindings, function_return_kinds)
                {
                    bindings.insert(name.clone(), kind);
                } else {
                    bindings.remove(name);
                }
            }
            NirStmt::Return(Some(value)) => {
                let current =
                    infer_network_owned_handle_kind(value, bindings, function_return_kinds)
                        .and_then(binding_to_network_owned_handle_return);
                return_kind = merge_optional_network_owned_handle_kind(return_kind, current);
            }
            NirStmt::If {
                then_body,
                else_body,
                ..
            } => {
                let mut then_bindings = bindings.clone();
                let then_kind = infer_network_return_kind_in_body(
                    then_body,
                    &mut then_bindings,
                    function_requirements,
                    function_return_kinds,
                )?;
                let mut else_bindings = bindings.clone();
                let else_kind = infer_network_return_kind_in_body(
                    else_body,
                    &mut else_bindings,
                    function_requirements,
                    function_return_kinds,
                )?;
                return_kind = merge_optional_network_owned_handle_kind(return_kind, then_kind);
                return_kind = merge_optional_network_owned_handle_kind(return_kind, else_kind);
                merge_network_owned_handle_bindings(bindings, &then_bindings, &else_bindings);
            }
            NirStmt::While { body, .. } => {
                let entry_bindings = bindings.clone();
                let mut loop_bindings = bindings.clone();
                let loop_kind = infer_network_return_kind_in_body(
                    body,
                    &mut loop_bindings,
                    function_requirements,
                    function_return_kinds,
                )?;
                return_kind = merge_optional_network_owned_handle_kind(return_kind, loop_kind);
                merge_network_owned_handle_bindings(bindings, &entry_bindings, &loop_bindings);
            }
            NirStmt::Print(_)
            | NirStmt::Await(_)
            | NirStmt::Expr(_)
            | NirStmt::Return(None)
            | NirStmt::Break
            | NirStmt::Continue => {}
        }
    }
    Ok(return_kind)
}

fn merge_optional_network_owned_handle_kind(
    lhs: Option<NetworkOwnedHandleReturn>,
    rhs: Option<NetworkOwnedHandleReturn>,
) -> Option<NetworkOwnedHandleReturn> {
    match (lhs, rhs) {
        (Some(lhs), Some(rhs)) if lhs == rhs => Some(lhs),
        (Some(_), Some(_)) => None,
        (Some(lhs), None) => Some(lhs),
        (None, Some(rhs)) => Some(rhs),
        (None, None) => None,
    }
}

fn binding_to_network_owned_handle_return(
    binding: NetworkOwnedHandleBinding,
) -> Option<NetworkOwnedHandleReturn> {
    match binding {
        NetworkOwnedHandleBinding::Concrete(kind) => Some(NetworkOwnedHandleReturn::Concrete(kind)),
        NetworkOwnedHandleBinding::Param { index, .. } => {
            Some(NetworkOwnedHandleReturn::ParamIndex(index))
        }
    }
}

fn infer_network_param_requirements_in_body(
    body: &[NirStmt],
    params: &[nuis_semantics::model::NirParam],
    requirements: &mut [Option<NetworkOwnedHandleRequirement>],
    function_requirements: &BTreeMap<String, Vec<Option<NetworkOwnedHandleRequirement>>>,
) -> Result<(), String> {
    let mut bindings = params
        .iter()
        .enumerate()
        .map(|(index, param)| (param.name.clone(), index))
        .collect::<BTreeMap<_, _>>();
    infer_network_param_requirements_with_bindings(
        body,
        requirements,
        function_requirements,
        &mut bindings,
    )
}

fn infer_network_param_requirements_with_bindings(
    body: &[NirStmt],
    requirements: &mut [Option<NetworkOwnedHandleRequirement>],
    function_requirements: &BTreeMap<String, Vec<Option<NetworkOwnedHandleRequirement>>>,
    bindings: &mut BTreeMap<String, usize>,
) -> Result<(), String> {
    for stmt in body {
        match stmt {
            NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. } => {
                if let Some(origin) = infer_network_param_origin(value, bindings) {
                    bindings.insert(name.clone(), origin);
                } else {
                    bindings.remove(name);
                }
                infer_network_param_requirements_in_expr(
                    value,
                    requirements,
                    function_requirements,
                    bindings,
                )?;
            }
            NirStmt::Print(value)
            | NirStmt::Await(value)
            | NirStmt::Expr(value)
            | NirStmt::Return(Some(value)) => infer_network_param_requirements_in_expr(
                value,
                requirements,
                function_requirements,
                bindings,
            )?,
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                infer_network_param_requirements_in_expr(
                    condition,
                    requirements,
                    function_requirements,
                    bindings,
                )?;
                let mut then_bindings = bindings.clone();
                infer_network_param_requirements_with_bindings(
                    then_body,
                    requirements,
                    function_requirements,
                    &mut then_bindings,
                )?;
                let mut else_bindings = bindings.clone();
                infer_network_param_requirements_with_bindings(
                    else_body,
                    requirements,
                    function_requirements,
                    &mut else_bindings,
                )?;
                merge_network_param_origin_bindings(bindings, &then_bindings, &else_bindings);
            }
            NirStmt::While { condition, body } => {
                infer_network_param_requirements_in_expr(
                    condition,
                    requirements,
                    function_requirements,
                    bindings,
                )?;
                let entry_bindings = bindings.clone();
                let mut loop_bindings = bindings.clone();
                infer_network_param_requirements_with_bindings(
                    body,
                    requirements,
                    function_requirements,
                    &mut loop_bindings,
                )?;
                merge_network_param_origin_bindings(bindings, &entry_bindings, &loop_bindings);
            }
            NirStmt::Return(None) | NirStmt::Break | NirStmt::Continue => {}
        }
    }
    Ok(())
}

fn infer_network_param_requirements_in_expr(
    expr: &NirExpr,
    requirements: &mut [Option<NetworkOwnedHandleRequirement>],
    function_requirements: &BTreeMap<String, Vec<Option<NetworkOwnedHandleRequirement>>>,
    bindings: &BTreeMap<String, usize>,
) -> Result<(), String> {
    match expr {
        NirExpr::CpuExternCall { callee, args, .. } => {
            infer_network_param_requirement_from_host_call(callee, args, requirements, bindings)?;
            for arg in args {
                infer_network_param_requirements_in_expr(
                    arg,
                    requirements,
                    function_requirements,
                    bindings,
                )?;
            }
        }
        NirExpr::CpuSpawn { callee, args }
        | NirExpr::CpuThreadSpawn { callee, args }
        | NirExpr::Call { callee, args } => {
            if let Some(callee_requirements) = function_requirements.get(callee) {
                for (index, arg) in args.iter().enumerate() {
                    let Some(Some(requirement)) = callee_requirements.get(index) else {
                        continue;
                    };
                    if let Some(origin) = infer_network_param_origin(arg, bindings) {
                        merge_network_param_requirement(
                            requirements,
                            origin,
                            *requirement,
                            callee,
                        )?;
                    }
                }
            }
            for arg in args {
                infer_network_param_requirements_in_expr(
                    arg,
                    requirements,
                    function_requirements,
                    bindings,
                )?;
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
        | NirExpr::CpuThreadJoin(inner)
        | NirExpr::CpuCancel(inner)
        | NirExpr::CpuJoinResult(inner)
        | NirExpr::CpuThreadJoinResult(inner)
        | NirExpr::CpuTaskCompleted(inner)
        | NirExpr::CpuTaskTimedOut(inner)
        | NirExpr::CpuTaskCancelled(inner)
        | NirExpr::CpuTaskValue(inner)
        | NirExpr::CpuMutexNew(inner)
        | NirExpr::CpuMutexLock(inner)
        | NirExpr::CpuMutexUnlock(inner)
        | NirExpr::CpuMutexValue(inner)
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
        | NirExpr::IsNull(inner) => infer_network_param_requirements_in_expr(
            inner,
            requirements,
            function_requirements,
            bindings,
        )?,
        NirExpr::DataResult { value, .. }
        | NirExpr::ShaderResult { value, .. }
        | NirExpr::NetworkResult { value, .. }
        | NirExpr::KernelResult { value, .. } => infer_network_param_requirements_in_expr(
            value,
            requirements,
            function_requirements,
            bindings,
        )?,
        NirExpr::AllocNode { value, next } => {
            infer_network_param_requirements_in_expr(
                value,
                requirements,
                function_requirements,
                bindings,
            )?;
            infer_network_param_requirements_in_expr(
                next,
                requirements,
                function_requirements,
                bindings,
            )?;
        }
        NirExpr::AllocBuffer { len, fill } => {
            infer_network_param_requirements_in_expr(
                len,
                requirements,
                function_requirements,
                bindings,
            )?;
            infer_network_param_requirements_in_expr(
                fill,
                requirements,
                function_requirements,
                bindings,
            )?;
        }
        NirExpr::LoadAt { buffer, index }
        | NirExpr::DataReadWindow {
            window: buffer,
            index,
        } => {
            infer_network_param_requirements_in_expr(
                buffer,
                requirements,
                function_requirements,
                bindings,
            )?;
            infer_network_param_requirements_in_expr(
                index,
                requirements,
                function_requirements,
                bindings,
            )?;
        }
        NirExpr::DataWriteWindow {
            window,
            index,
            value,
        }
        | NirExpr::StoreAt {
            buffer: window,
            index,
            value,
        } => {
            infer_network_param_requirements_in_expr(
                window,
                requirements,
                function_requirements,
                bindings,
            )?;
            infer_network_param_requirements_in_expr(
                index,
                requirements,
                function_requirements,
                bindings,
            )?;
            infer_network_param_requirements_in_expr(
                value,
                requirements,
                function_requirements,
                bindings,
            )?;
        }
        NirExpr::StoreValue { target, value }
        | NirExpr::StoreNext {
            target,
            next: value,
        } => {
            infer_network_param_requirements_in_expr(
                target,
                requirements,
                function_requirements,
                bindings,
            )?;
            infer_network_param_requirements_in_expr(
                value,
                requirements,
                function_requirements,
                bindings,
            )?;
        }
        NirExpr::DataCopyWindow { input, offset, len }
        | NirExpr::DataImmutableWindow { input, offset, len } => {
            infer_network_param_requirements_in_expr(
                input,
                requirements,
                function_requirements,
                bindings,
            )?;
            infer_network_param_requirements_in_expr(
                offset,
                requirements,
                function_requirements,
                bindings,
            )?;
            infer_network_param_requirements_in_expr(
                len,
                requirements,
                function_requirements,
                bindings,
            )?;
        }
        NirExpr::DataProfileSendUplink { input, .. }
        | NirExpr::DataProfileSendDownlink { input, .. }
        | NirExpr::FieldAccess { base: input, .. }
        | NirExpr::ShaderProfileRender { packet: input, .. } => {
            infer_network_param_requirements_in_expr(
                input,
                requirements,
                function_requirements,
                bindings,
            )?
        }
        NirExpr::ShaderProfileColorSeed { base, delta, .. }
        | NirExpr::ShaderProfileRadiusSeed { base, delta, .. } => {
            infer_network_param_requirements_in_expr(
                base,
                requirements,
                function_requirements,
                bindings,
            )?;
            infer_network_param_requirements_in_expr(
                delta,
                requirements,
                function_requirements,
                bindings,
            )?;
        }
        NirExpr::ShaderProfileSpeedSeed {
            delta, scale, base, ..
        } => {
            infer_network_param_requirements_in_expr(
                delta,
                requirements,
                function_requirements,
                bindings,
            )?;
            infer_network_param_requirements_in_expr(
                scale,
                requirements,
                function_requirements,
                bindings,
            )?;
            infer_network_param_requirements_in_expr(
                base,
                requirements,
                function_requirements,
                bindings,
            )?;
        }
        NirExpr::ShaderProfilePacket {
            color,
            speed,
            radius,
            ..
        } => {
            infer_network_param_requirements_in_expr(
                color,
                requirements,
                function_requirements,
                bindings,
            )?;
            infer_network_param_requirements_in_expr(
                speed,
                requirements,
                function_requirements,
                bindings,
            )?;
            infer_network_param_requirements_in_expr(
                radius,
                requirements,
                function_requirements,
                bindings,
            )?;
        }
        NirExpr::CpuTimeout { task, limit } => {
            infer_network_param_requirements_in_expr(
                task,
                requirements,
                function_requirements,
                bindings,
            )?;
            infer_network_param_requirements_in_expr(
                limit,
                requirements,
                function_requirements,
                bindings,
            )?;
        }
        NirExpr::MethodCall { receiver, args, .. } => {
            infer_network_param_requirements_in_expr(
                receiver,
                requirements,
                function_requirements,
                bindings,
            )?;
            for arg in args {
                infer_network_param_requirements_in_expr(
                    arg,
                    requirements,
                    function_requirements,
                    bindings,
                )?;
            }
        }
        NirExpr::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                infer_network_param_requirements_in_expr(
                    value,
                    requirements,
                    function_requirements,
                    bindings,
                )?;
            }
        }
        NirExpr::Binary { lhs, rhs, .. } => {
            infer_network_param_requirements_in_expr(
                lhs,
                requirements,
                function_requirements,
                bindings,
            )?;
            infer_network_param_requirements_in_expr(
                rhs,
                requirements,
                function_requirements,
                bindings,
            )?;
        }
        NirExpr::ShaderBeginPass {
            target,
            pipeline,
            viewport,
        } => {
            infer_network_param_requirements_in_expr(
                target,
                requirements,
                function_requirements,
                bindings,
            )?;
            infer_network_param_requirements_in_expr(
                pipeline,
                requirements,
                function_requirements,
                bindings,
            )?;
            infer_network_param_requirements_in_expr(
                viewport,
                requirements,
                function_requirements,
                bindings,
            )?;
        }
        NirExpr::ShaderDrawInstanced {
            pass,
            packet,
            vertex_count,
            instance_count,
        } => {
            infer_network_param_requirements_in_expr(
                pass,
                requirements,
                function_requirements,
                bindings,
            )?;
            infer_network_param_requirements_in_expr(
                packet,
                requirements,
                function_requirements,
                bindings,
            )?;
            infer_network_param_requirements_in_expr(
                vertex_count,
                requirements,
                function_requirements,
                bindings,
            )?;
            infer_network_param_requirements_in_expr(
                instance_count,
                requirements,
                function_requirements,
                bindings,
            )?;
        }
        _ => {}
    }
    Ok(())
}

fn infer_network_param_requirement_from_host_call(
    callee: &str,
    args: &[NirExpr],
    requirements: &mut [Option<NetworkOwnedHandleRequirement>],
    bindings: &BTreeMap<String, usize>,
) -> Result<(), String> {
    let requirement = match callee {
        "host_network_accept_owned" => Some(NetworkOwnedHandleRequirement::Listener),
        "host_network_send_owned" | "host_network_recv_owned" => {
            Some(NetworkOwnedHandleRequirement::Transport)
        }
        "host_network_recv_http_status_owned" => {
            Some(NetworkOwnedHandleRequirement::StreamTransport)
        }
        "host_network_close_owned" => Some(NetworkOwnedHandleRequirement::OwnedAny),
        _ => None,
    };
    let Some(requirement) = requirement else {
        return Ok(());
    };
    let Some(origin) = args
        .first()
        .and_then(|arg| infer_network_param_origin(arg, bindings))
    else {
        return Ok(());
    };
    merge_network_param_requirement(requirements, origin, requirement, callee)
}

fn merge_network_param_requirement(
    requirements: &mut [Option<NetworkOwnedHandleRequirement>],
    index: usize,
    incoming: NetworkOwnedHandleRequirement,
    context: &str,
) -> Result<(), String> {
    let slot = requirements.get_mut(index).ok_or_else(|| {
        format!(
            "network handle requirement index {} out of bounds in {}",
            index, context
        )
    })?;
    *slot = Some(match *slot {
        None => incoming,
        Some(existing) => {
            merge_network_owned_handle_requirement(existing, incoming).ok_or_else(|| {
                format!(
                    "function `{}` uses parameter {} as incompatible network handle kinds",
                    context, index
                )
            })?
        }
    });
    Ok(())
}

fn merge_network_owned_handle_requirement(
    lhs: NetworkOwnedHandleRequirement,
    rhs: NetworkOwnedHandleRequirement,
) -> Option<NetworkOwnedHandleRequirement> {
    use NetworkOwnedHandleRequirement as Req;
    match (lhs, rhs) {
        (Req::OwnedAny, other) | (other, Req::OwnedAny) => Some(other),
        (Req::Transport, Req::StreamTransport) | (Req::StreamTransport, Req::Transport) => {
            Some(Req::StreamTransport)
        }
        (lhs, rhs) if lhs == rhs => Some(lhs),
        _ => None,
    }
}

fn infer_network_param_origin(expr: &NirExpr, bindings: &BTreeMap<String, usize>) -> Option<usize> {
    match expr {
        NirExpr::Var(name) => bindings.get(name).copied(),
        NirExpr::NetworkValue(inner) => infer_network_param_origin(inner, bindings),
        NirExpr::NetworkResult { value, .. } => infer_network_param_origin(value, bindings),
        _ => None,
    }
}

fn merge_network_param_origin_bindings(
    bindings: &mut BTreeMap<String, usize>,
    then_bindings: &BTreeMap<String, usize>,
    else_bindings: &BTreeMap<String, usize>,
) {
    let merged = bindings
        .keys()
        .chain(then_bindings.keys())
        .chain(else_bindings.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    for name in merged {
        match (
            then_bindings.get(&name).copied(),
            else_bindings.get(&name).copied(),
        ) {
            (Some(lhs), Some(rhs)) if lhs == rhs => {
                bindings.insert(name, lhs);
            }
            _ => {
                bindings.remove(&name);
            }
        }
    }
}

fn seed_network_param_bindings(
    function: &nuis_semantics::model::NirFunction,
    function_requirements: &BTreeMap<String, Vec<Option<NetworkOwnedHandleRequirement>>>,
    bindings: &mut BTreeMap<String, NetworkOwnedHandleBinding>,
) {
    let Some(requirements) = function_requirements.get(&function.name) else {
        return;
    };
    for (index, param) in function.params.iter().enumerate() {
        let Some(Some(requirement)) = requirements.get(index) else {
            continue;
        };
        bindings.insert(
            param.name.clone(),
            NetworkOwnedHandleBinding::Param {
                index,
                requirement: *requirement,
            },
        );
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

#[cfg(test)]
mod tests {
    use super::*;
    use nuis_semantics::model::{
        NirFunction, NirNetworkFlowState, NirParam, NirTypeRef, NirVisibility,
    };

    fn i64_type() -> NirTypeRef {
        NirTypeRef {
            name: "i64".to_owned(),
            generic_args: vec![],
            is_optional: false,
            is_ref: false,
        }
    }

    fn test_module(body: Vec<NirStmt>) -> NirModule {
        test_module_with_functions(vec![NirFunction {
            visibility: NirVisibility::Private,
            name: "main".to_owned(),
            annotations: vec![],
            test_name: None,
            test_ignored: false,
            test_should_fail: false,
            test_reason: None,
            test_timeout_ms: None,
            test_clock_domain: None,
            test_clock_policy: None,
            benchmark_name: None,
            benchmark_warmup_iters: None,
            benchmark_measure_iters: None,
            benchmark_timeout_ms: None,
            benchmark_clock_domain: None,
            benchmark_clock_policy: None,
            is_async: false,
            generic_params: vec![],
            where_bounds: vec![],
            params: vec![],
            return_type: Some(i64_type()),
            body,
        }])
    }

    fn test_module_with_functions(functions: Vec<NirFunction>) -> NirModule {
        NirModule {
            annotations: vec![],
            uses: vec![],
            domain: "cpu".to_owned(),
            unit: "Main".to_owned(),
            externs: vec![],
            extern_interfaces: vec![],
            consts: vec![],
            type_aliases: vec![],
            structs: vec![],
            enums: vec![],
            traits: vec![],
            impls: vec![],
            functions,
        }
    }

    fn network_result_i64_type() -> NirTypeRef {
        NirTypeRef {
            name: "NetworkResult".to_owned(),
            generic_args: vec![i64_type()],
            is_optional: false,
            is_ref: false,
        }
    }

    fn private_fn(
        name: &str,
        params: Vec<NirParam>,
        return_type: Option<NirTypeRef>,
        body: Vec<NirStmt>,
    ) -> NirFunction {
        NirFunction {
            visibility: NirVisibility::Private,
            name: name.to_owned(),
            annotations: vec![],
            test_name: None,
            test_ignored: false,
            test_should_fail: false,
            test_reason: None,
            test_timeout_ms: None,
            test_clock_domain: None,
            test_clock_policy: None,
            benchmark_name: None,
            benchmark_warmup_iters: None,
            benchmark_measure_iters: None,
            benchmark_timeout_ms: None,
            benchmark_clock_domain: None,
            benchmark_clock_policy: None,
            is_async: false,
            generic_params: vec![],
            where_bounds: vec![],
            params,
            return_type,
            body,
        }
    }

    fn open_tcp_stream_expr() -> NirExpr {
        NirExpr::CpuExternCall {
            abi: "c".to_owned(),
            interface: None,
            callee: "host_network_open_tcp_stream".to_owned(),
            args: vec![NirExpr::Int(443), NirExpr::Int(250)],
        }
    }

    #[test]
    fn network_owned_handle_provenance_merges_matching_if_branches() {
        let module = test_module(vec![
            NirStmt::If {
                condition: NirExpr::Bool(true),
                then_body: vec![NirStmt::Let {
                    name: "handle".to_owned(),
                    ty: Some(i64_type()),
                    value: open_tcp_stream_expr(),
                }],
                else_body: vec![NirStmt::Let {
                    name: "handle".to_owned(),
                    ty: Some(i64_type()),
                    value: open_tcp_stream_expr(),
                }],
            },
            NirStmt::Expr(NirExpr::CpuExternCall {
                abi: "c".to_owned(),
                interface: None,
                callee: "host_network_close_owned".to_owned(),
                args: vec![NirExpr::Var("handle".to_owned())],
            }),
        ]);

        validate_network_owned_handle_provenance(&module, "cpu.Main", "network.NetworkUnit")
            .unwrap();
    }

    #[test]
    fn network_owned_handle_provenance_merges_matching_while_state() {
        let module = test_module(vec![
            NirStmt::Let {
                name: "handle".to_owned(),
                ty: Some(i64_type()),
                value: open_tcp_stream_expr(),
            },
            NirStmt::While {
                condition: NirExpr::Bool(true),
                body: vec![NirStmt::Let {
                    name: "handle".to_owned(),
                    ty: Some(i64_type()),
                    value: open_tcp_stream_expr(),
                }],
            },
            NirStmt::Expr(NirExpr::CpuExternCall {
                abi: "c".to_owned(),
                interface: None,
                callee: "host_network_close_owned".to_owned(),
                args: vec![NirExpr::Var("handle".to_owned())],
            }),
        ]);

        validate_network_owned_handle_provenance(&module, "cpu.Main", "network.NetworkUnit")
            .unwrap();
    }

    #[test]
    fn network_owned_handle_provenance_accepts_network_result_wrapped_helper_return() {
        let module = test_module_with_functions(vec![
            private_fn(
                "open_handle_result",
                vec![],
                Some(network_result_i64_type()),
                vec![NirStmt::Return(Some(NirExpr::NetworkResult {
                    value: Box::new(open_tcp_stream_expr()),
                    state: NirNetworkFlowState::ConfigReady,
                }))],
            ),
            private_fn(
                "main",
                vec![],
                Some(i64_type()),
                vec![
                    NirStmt::Let {
                        name: "opened".to_owned(),
                        ty: Some(network_result_i64_type()),
                        value: NirExpr::Call {
                            callee: "open_handle_result".to_owned(),
                            args: vec![],
                        },
                    },
                    NirStmt::Let {
                        name: "handle".to_owned(),
                        ty: Some(i64_type()),
                        value: NirExpr::NetworkValue(Box::new(NirExpr::Var("opened".to_owned()))),
                    },
                    NirStmt::Expr(NirExpr::CpuExternCall {
                        abi: "c".to_owned(),
                        interface: None,
                        callee: "host_network_close_owned".to_owned(),
                        args: vec![NirExpr::Var("handle".to_owned())],
                    }),
                ],
            ),
        ]);

        validate_network_owned_handle_provenance(&module, "cpu.Main", "network.NetworkUnit")
            .unwrap();
    }

    #[test]
    fn network_owned_handle_provenance_rejects_network_result_wrapped_listener_return_for_send() {
        let module = test_module_with_functions(vec![
            private_fn(
                "open_listener_result",
                vec![],
                Some(network_result_i64_type()),
                vec![NirStmt::Return(Some(NirExpr::NetworkResult {
                    value: Box::new(NirExpr::CpuExternCall {
                        abi: "c".to_owned(),
                        interface: None,
                        callee: "host_network_open_tcp_listener".to_owned(),
                        args: vec![NirExpr::Int(9000), NirExpr::Int(125), NirExpr::Int(150)],
                    }),
                    state: NirNetworkFlowState::ConfigReady,
                }))],
            ),
            private_fn(
                "main",
                vec![],
                Some(i64_type()),
                vec![
                    NirStmt::Let {
                        name: "opened".to_owned(),
                        ty: Some(network_result_i64_type()),
                        value: NirExpr::Call {
                            callee: "open_listener_result".to_owned(),
                            args: vec![],
                        },
                    },
                    NirStmt::Let {
                        name: "handle".to_owned(),
                        ty: Some(i64_type()),
                        value: NirExpr::NetworkValue(Box::new(NirExpr::Var("opened".to_owned()))),
                    },
                    NirStmt::Expr(NirExpr::CpuExternCall {
                        abi: "c".to_owned(),
                        interface: None,
                        callee: "host_network_send_owned".to_owned(),
                        args: vec![
                            NirExpr::Var("handle".to_owned()),
                            NirExpr::Int(64),
                            NirExpr::Int(32),
                        ],
                    }),
                ],
            ),
        ]);

        let err =
            validate_network_owned_handle_provenance(&module, "cpu.Main", "network.NetworkUnit")
                .unwrap_err();
        assert!(err.contains("host_network_send_owned"), "{err}");
        assert!(err.contains("listener-owned source"), "{err}");
    }
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
