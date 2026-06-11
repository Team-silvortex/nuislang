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
use super::profile_refs::{
    resolve_project_profile_refs, stitch_data_profile_edges, stitch_shader_profile_edges,
};
use super::profile_targets::has_xfer_segment;
use super::profile_usage::{
    nir_uses_data_profile_handle_table, nir_uses_data_profile_send_downlink,
    nir_uses_data_profile_send_uplink, nir_uses_network_profile_bind_core,
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
