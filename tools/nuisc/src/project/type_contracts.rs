use std::collections::BTreeMap;

use nuis_semantics::model::NirTypeRef;
use yir_core::{Operation, YirModule};

use super::data_bridge_directions::data_bridge_directions;
use super::ProjectAbiResolution;
use super::{
    build_project_link_bridge_contract, collect_profile_int_bindings, ensure_project_resource,
    infer_data_handle_table_schema, infer_shader_packet_contract, merge_project_payload_contract,
    payload_class_marker_name, payload_shape_marker_name,
    profile_apply::resolve_registered_abi_target, profile_apply::target_config_tokens_for_domain,
    push_profile_node, resolve_project_profile_target_name, sanitize_ident, LoadedProject,
};

pub(super) fn materialize_project_type_contract_nodes(
    project: &LoadedProject,
    abi_resolution: &ProjectAbiResolution,
    module: &mut YirModule,
) -> Result<(), String> {
    ensure_project_resource(module, "cpu0", "cpu.arm64");
    super::materialize_project_bridge_contract_nodes(project, module)?;
    materialize_project_abi_summary_nodes(abi_resolution, module)?;

    for project_module in &project.modules {
        match project_module.ast.domain.as_str() {
            "data" => {
                materialize_data_type_contract_nodes(project, &project_module.ast.unit, module)?
            }
            "shader" => materialize_shader_type_contract_nodes(
                project,
                &project_module.ast.unit,
                abi_resolution,
                module,
            )?,
            "kernel" => materialize_kernel_type_contract_nodes(
                project,
                &project_module.ast.unit,
                abi_resolution,
                module,
            )?,
            "network" => materialize_network_type_contract_nodes(
                &project_module.ast.unit,
                abi_resolution,
                module,
            )?,
            _ => {}
        }
    }

    materialize_project_abi_graph_summary_node(abi_resolution, module);

    Ok(())
}

fn materialize_project_abi_summary_nodes(
    abi_resolution: &ProjectAbiResolution,
    module: &mut YirModule,
) -> Result<(), String> {
    for requirement in &abi_resolution.requirements {
        let mode = if abi_resolution.explicit {
            "explicit"
        } else {
            "auto"
        };
        let target = resolve_registered_abi_target(&requirement.domain, Some(abi_resolution))
            .ok()
            .flatten();
        let arch = target
            .as_ref()
            .map(|target| target.machine_arch.clone())
            .unwrap_or_else(|| "unknown".to_owned());
        let os = target
            .as_ref()
            .map(|target| target.machine_os.clone())
            .unwrap_or_else(|| "unknown".to_owned());
        let object = target
            .as_ref()
            .map(|target| target.object_format.clone())
            .unwrap_or_else(|| "unknown".to_owned());
        let calling = target
            .as_ref()
            .map(|target| target.calling_abi.clone())
            .unwrap_or_else(|| "unknown".to_owned());
        let backend = target
            .as_ref()
            .and_then(|target| target.backend_family.clone())
            .unwrap_or_else(|| "none".to_owned());
        let vendor = target
            .as_ref()
            .and_then(|target| target.vendor.clone())
            .unwrap_or_else(|| "none".to_owned());
        let device = target
            .as_ref()
            .and_then(|target| target.device_class.clone())
            .unwrap_or_else(|| "none".to_owned());
        let value = format!(
            "mode=symbol:{mode};abi=symbol:{};arch=symbol:{};os=symbol:{};object=symbol:{};calling=symbol:{};backend=symbol:{};vendor=symbol:{};device=symbol:{}",
            requirement.abi,
            arch,
            os,
            object,
            calling,
            backend,
            vendor,
            device
        );
        let entry_name = format!(
            "project_abi_{}_selection_entry",
            sanitize_ident(&requirement.domain)
        );
        let contract_name = format!(
            "project_abi_{}_selection_summary_type",
            sanitize_ident(&requirement.domain)
        );
        push_profile_text_node(module, entry_name.clone(), value.clone());
        push_profile_text_node(module, contract_name.clone(), value);
        connect_project_contract_node(module, &contract_name, &entry_name);
    }
    Ok(())
}

fn materialize_project_abi_graph_summary_node(
    abi_resolution: &ProjectAbiResolution,
    module: &mut YirModule,
) {
    let domains = abi_resolution
        .requirements
        .iter()
        .map(|item| item.domain.as_str())
        .collect::<Vec<_>>();
    let has_cpu_summary = domains.iter().any(|domain| *domain == "cpu");
    let has_data_summary = domains.iter().any(|domain| *domain == "data");
    let has_kernel_target = module.nodes.iter().any(|node| {
        node.name.ends_with("_kernel_target_config_auto")
            && node.op.module == "kernel"
            && node.op.instruction == "target_config"
    });
    let has_shader_target = module.nodes.iter().any(|node| {
        node.name.ends_with("_shader_target_config_auto")
            && node.op.module == "shader"
            && node.op.instruction == "target_config"
    });
    let has_network_target = module.nodes.iter().any(|node| {
        node.name.ends_with("_network_target_config_auto")
            && node.op.module == "network"
            && node.op.instruction == "target_config"
    });
    let mode = if abi_resolution.explicit {
        "explicit"
    } else {
        "auto"
    };
    let value = format!(
        "mode=symbol:{mode};domains=symbol:{};cpu_summary=symbol:{};data_summary=symbol:{};kernel_target=symbol:{};shader_target=symbol:{};network_target=symbol:{}",
        domains.join(","),
        if has_cpu_summary { "present" } else { "absent" },
        if has_data_summary { "present" } else { "absent" },
        if has_kernel_target { "present" } else { "absent" },
        if has_shader_target { "present" } else { "absent" },
        if has_network_target { "present" } else { "absent" }
    );
    let entry_name = "project_abi_graph_summary_entry".to_owned();
    let contract_name = "project_abi_graph_summary_type".to_owned();
    push_profile_text_node(module, entry_name.clone(), value.clone());
    push_profile_text_node(module, contract_name.clone(), value);
    connect_project_contract_node(module, &contract_name, &entry_name);
}

pub(super) fn push_profile_text_node(module: &mut YirModule, name: String, value: String) {
    push_profile_node(
        module,
        name,
        "cpu0",
        Operation {
            module: "cpu".to_owned(),
            instruction: "text".to_owned(),
            args: vec![value],
        },
    );
}

pub(super) fn connect_project_contract_node(module: &mut YirModule, from: &str, to: &str) {
    let resource_families = module
        .resources
        .iter()
        .map(|resource| (resource.name.clone(), resource.kind.family().to_owned()))
        .collect::<BTreeMap<_, _>>();
    let node_resources = module
        .nodes
        .iter()
        .map(|node| (node.name.clone(), node.resource.clone()))
        .collect::<BTreeMap<_, _>>();
    super::push_project_dependency_edge_if_missing(
        module,
        &resource_families,
        &node_resources,
        from,
        to,
    );
}

fn materialize_data_type_contract_nodes(
    project: &LoadedProject,
    unit: &str,
    module: &mut YirModule,
) -> Result<(), String> {
    let mut payloads = [None, None];
    for link in &project.manifest.links {
        let Some(via) = &link.via else {
            continue;
        };
        let (via_domain, via_unit) = super::split_domain_unit(via)?;
        if via_domain != "data" || via_unit != unit {
            continue;
        }
        let bridge = build_project_link_bridge_contract(project, &link.from, &link.to, via)?;
        for direction in data_bridge_directions() {
            if let Some(ty) = bridge.clone().into_payload(direction.is_uplink) {
                let slot = &mut payloads[direction.index];
                *slot = Some(merge_project_payload_contract(
                    slot.take(),
                    ty,
                    "data",
                    unit,
                    direction.name,
                )?);
            }
        }
    }

    for direction in data_bridge_directions() {
        if let Some(ty) = payloads[direction.index].as_ref() {
            let class_node = format!(
                "project_profile_data_{}_{}_payload_class_type",
                sanitize_ident(unit),
                direction.name,
            );
            let shape_node = format!(
                "project_profile_data_{}_{}_payload_shape_type",
                sanitize_ident(unit),
                direction.name,
            );
            push_profile_text_node(module, class_node.clone(), payload_class_marker_name(ty));
            push_profile_text_node(module, shape_node.clone(), payload_shape_marker_name(ty));
            connect_project_contract_node(
                module,
                &class_node,
                &resolve_project_profile_target_name("data", unit, direction.payload_class_marker),
            );
            connect_project_contract_node(
                module,
                &shape_node,
                &resolve_project_profile_target_name("data", unit, direction.payload_shape_marker),
            );
        }
    }

    if let Some(schema) = infer_data_handle_table_schema(project, unit)? {
        let schema_node = format!(
            "project_profile_data_{}_handle_table_schema_type",
            sanitize_ident(unit)
        );
        push_profile_text_node(module, schema_node.clone(), schema);
        connect_project_contract_node(
            module,
            &schema_node,
            &resolve_project_profile_target_name("data", unit, "handle_table"),
        );
    }

    Ok(())
}

fn materialize_shader_type_contract_nodes(
    project: &LoadedProject,
    unit: &str,
    abi_resolution: &ProjectAbiResolution,
    module: &mut YirModule,
) -> Result<(), String> {
    if let Some(contract) = infer_shader_packet_contract(project, unit)? {
        let packet_type = NirTypeRef {
            name: contract.type_name.clone(),
            generic_args: Vec::new(),
            is_optional: false,
            is_ref: false,
        };
        let type_node = format!(
            "project_profile_shader_{}_packet_type",
            sanitize_ident(unit)
        );
        let class_node = format!(
            "project_profile_shader_{}_packet_class_type",
            sanitize_ident(unit)
        );
        let shape_node = format!(
            "project_profile_shader_{}_packet_shape_type",
            sanitize_ident(unit)
        );
        push_profile_text_node(module, type_node.clone(), contract.type_name);
        push_profile_text_node(
            module,
            class_node.clone(),
            payload_class_marker_name(&packet_type),
        );
        push_profile_text_node(
            module,
            shape_node.clone(),
            payload_shape_marker_name(&packet_type),
        );
        connect_project_contract_node(
            module,
            &type_node,
            &resolve_project_profile_target_name("shader", unit, "packet_field_count"),
        );
        connect_project_contract_node(
            module,
            &class_node,
            &resolve_project_profile_target_name("shader", unit, "packet_field_count"),
        );
        connect_project_contract_node(
            module,
            &shape_node,
            &resolve_project_profile_target_name("shader", unit, "packet_field_count"),
        );
    }
    materialize_target_config_contract_node("shader", unit, abi_resolution, module)?;
    materialize_abi_selection_contract_node("shader", unit, abi_resolution, module)?;
    Ok(())
}

fn materialize_kernel_type_contract_nodes(
    project: &LoadedProject,
    unit: &str,
    abi_resolution: &ProjectAbiResolution,
    module: &mut YirModule,
) -> Result<(), String> {
    if let Some(summary) = infer_kernel_slot_contract_summary(project, unit)? {
        let summary_node = format!(
            "project_profile_kernel_{}_slot_contract_type",
            sanitize_ident(unit)
        );
        push_profile_text_node(module, summary_node.clone(), summary);
        connect_project_contract_node(
            module,
            &summary_node,
            &format!(
                "project_profile_kernel_{}_profile_entry",
                sanitize_ident(unit)
            ),
        );
    }
    materialize_target_config_contract_node("kernel", unit, abi_resolution, module)?;
    materialize_abi_selection_contract_node("kernel", unit, abi_resolution, module)?;
    Ok(())
}

fn materialize_network_type_contract_nodes(
    unit: &str,
    abi_resolution: &ProjectAbiResolution,
    module: &mut YirModule,
) -> Result<(), String> {
    materialize_target_config_contract_node("network", unit, abi_resolution, module)?;
    materialize_abi_selection_contract_node("network", unit, abi_resolution, module)?;
    Ok(())
}

fn materialize_abi_selection_contract_node(
    domain: &str,
    unit: &str,
    abi_resolution: &ProjectAbiResolution,
    module: &mut YirModule,
) -> Result<(), String> {
    let Some(requirement) = abi_resolution
        .requirements
        .iter()
        .find(|item| item.domain == domain)
    else {
        return Ok(());
    };
    let Some(target) = resolve_registered_abi_target(domain, Some(abi_resolution))? else {
        return Ok(());
    };
    let tokens = target_config_tokens_for_domain(domain, &target);
    let target_suffix = match domain {
        "kernel" => "kernel_target_config_auto",
        "shader" => "shader_target_config_auto",
        "network" => "network_target_config_auto",
        _ => return Ok(()),
    };
    let target_name = format!(
        "project_profile_{}_{}_{}",
        sanitize_ident(domain),
        sanitize_ident(unit),
        target_suffix
    );
    if !module.nodes.iter().any(|node| node.name == target_name) {
        return Ok(());
    }
    let contract_name = format!(
        "project_profile_{}_{}_abi_selection_contract_type",
        sanitize_ident(domain),
        sanitize_ident(unit)
    );
    let mode = if abi_resolution.explicit {
        "explicit"
    } else {
        "auto"
    };
    let contract_value = format!(
        "mode=symbol:{mode};abi=symbol:{};arch=symbol:{};runtime=symbol:{};vendor=symbol:{};device=symbol:{};lane_width=i64:{};backend_features=list:{}",
        requirement.abi,
        tokens.arch,
        tokens.runtime,
        target.vendor.as_deref().unwrap_or("none"),
        target.device_class.as_deref().unwrap_or("none"),
        tokens.lane_width,
        tokens.backend_features
    );
    push_profile_text_node(module, contract_name.clone(), contract_value);
    connect_project_contract_node(module, &contract_name, &target_name);
    Ok(())
}

fn materialize_target_config_contract_node(
    domain: &str,
    unit: &str,
    abi_resolution: &ProjectAbiResolution,
    module: &mut YirModule,
) -> Result<(), String> {
    let suffix = match domain {
        "kernel" => "kernel_target_config_auto",
        "shader" => "shader_target_config_auto",
        "network" => "network_target_config_auto",
        _ => return Ok(()),
    };
    let target_name = format!(
        "project_profile_{}_{}_{}",
        sanitize_ident(domain),
        sanitize_ident(unit),
        suffix
    );
    let Some(target) = module.nodes.iter().find(|node| node.name == target_name) else {
        return Ok(());
    };
    if target.op.args.len() < 3 {
        return Ok(());
    }
    let selected_target = resolve_registered_abi_target(domain, Some(abi_resolution))?;
    let contract_name = format!(
        "project_profile_{}_{}_target_contract_type",
        sanitize_ident(domain),
        sanitize_ident(unit)
    );
    let contract_value = format!(
        "arch=symbol:{};runtime=symbol:{};vendor=symbol:{};device=symbol:{};lane_width=i64:{};backend_features=list:{}",
        target.op.args[0],
        target.op.args[1],
        selected_target
            .as_ref()
            .and_then(|target| target.vendor.as_deref())
            .unwrap_or("none"),
        selected_target
            .as_ref()
            .and_then(|target| target.device_class.as_deref())
            .unwrap_or("none"),
        target.op.args[2],
        target.op.args.get(3).map(String::as_str).unwrap_or("")
    );
    push_profile_text_node(module, contract_name.clone(), contract_value);
    connect_project_contract_node(module, &contract_name, &target_name);
    Ok(())
}

fn infer_kernel_slot_contract_summary(
    project: &LoadedProject,
    unit: &str,
) -> Result<Option<String>, String> {
    let Some(project_module) = project
        .modules
        .iter()
        .find(|module| module.ast.domain == "kernel" && module.ast.unit == unit)
    else {
        return Ok(None);
    };
    let Some(profile_fn) = project_module
        .ast
        .functions
        .iter()
        .find(|function| function.name == "profile")
    else {
        return Ok(None);
    };
    let int_bindings = collect_profile_int_bindings(&profile_fn.body);
    let Some(bind_core) = int_bindings.get("bind_core") else {
        return Ok(None);
    };
    let Some(queue_depth) = int_bindings.get("queue_depth") else {
        return Ok(None);
    };
    let Some(batch_lanes) = int_bindings.get("batch_lanes") else {
        return Ok(None);
    };
    Ok(Some(format!(
        "bind_core=i64:{bind_core};queue_depth=i64:{queue_depth};batch_lanes=i64:{batch_lanes}"
    )))
}
