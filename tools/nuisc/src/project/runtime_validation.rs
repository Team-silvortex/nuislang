use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use nuis_semantics::model::NirModule;
use yir_core::{EdgeKind, Node, Operation, OperationDomainFamily, SemanticOp, YirModule};

use super::abi::required_abi_surfaces_for_domain;
use super::bridge_contracts::required_project_link_stage_contract;
use super::data_validation::{validate_data_profile_for_link, validate_data_profile_token_types};
use super::kernel_validation::{
    nir_uses_kernel_profile_batch_lanes, nir_uses_kernel_profile_bind_core,
    nir_uses_kernel_profile_queue_depth, validate_kernel_profile_for_link,
};
use super::network_validation::validate_network_profile_for_link;
use super::profile_refs::{
    resolve_project_profile_refs, stitch_data_profile_edges, stitch_shader_profile_edges,
};
use super::profile_targets::has_xfer_segment;
use super::profile_usage::{
    nir_uses_data_profile_handle_table, nir_uses_data_profile_send_downlink,
    nir_uses_data_profile_send_uplink, nir_uses_network_profile_bind_core,
    nir_uses_network_profile_endpoint_kind, nir_uses_shader_binding_profile_contract,
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

#[path = "runtime_validation_network.rs"]
mod runtime_validation_network;
use runtime_validation_network::{
    validate_network_host_call_requirements, validate_network_profile_slot_requirements,
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
