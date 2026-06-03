use std::collections::BTreeMap;

use nuis_semantics::model::NirTypeRef;
use yir_core::{Operation, YirModule};

use super::{
    build_project_link_bridge_contract, collect_profile_int_bindings, ensure_project_resource,
    infer_data_handle_table_schema, infer_shader_packet_contract, merge_project_payload_contract,
    payload_class_marker_name, payload_shape_marker_name, push_profile_node,
    resolve_project_profile_target_name, sanitize_ident, LoadedProject,
};

pub(super) fn materialize_project_type_contract_nodes(
    project: &LoadedProject,
    module: &mut YirModule,
) -> Result<(), String> {
    ensure_project_resource(module, "cpu0", "cpu.arm64");
    super::materialize_project_bridge_contract_nodes(project, module)?;

    for project_module in &project.modules {
        match project_module.ast.domain.as_str() {
            "data" => {
                materialize_data_type_contract_nodes(project, &project_module.ast.unit, module)?
            }
            "shader" => {
                materialize_shader_type_contract_nodes(project, &project_module.ast.unit, module)?
            }
            "kernel" => {
                materialize_kernel_type_contract_nodes(project, &project_module.ast.unit, module)?
            }
            _ => {}
        }
    }

    Ok(())
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
    let mut uplink_payload: Option<NirTypeRef> = None;
    let mut downlink_payload: Option<NirTypeRef> = None;
    for link in &project.manifest.links {
        let Some(via) = &link.via else {
            continue;
        };
        let (via_domain, via_unit) = super::split_domain_unit(via)?;
        if via_domain != "data" || via_unit != unit {
            continue;
        }
        let bridge = build_project_link_bridge_contract(project, &link.from, &link.to, via)?;
        if let Some(ty) = bridge.uplink_payload {
            uplink_payload = Some(merge_project_payload_contract(
                uplink_payload.take(),
                ty,
                "data",
                unit,
                "uplink",
            )?);
        }
        if let Some(ty) = bridge.downlink_payload {
            downlink_payload = Some(merge_project_payload_contract(
                downlink_payload.take(),
                ty,
                "data",
                unit,
                "downlink",
            )?);
        }
    }

    if let Some(ty) = uplink_payload.as_ref() {
        let class_node = format!(
            "project_profile_data_{}_uplink_payload_class_type",
            sanitize_ident(unit)
        );
        let shape_node = format!(
            "project_profile_data_{}_uplink_payload_shape_type",
            sanitize_ident(unit)
        );
        push_profile_text_node(module, class_node.clone(), payload_class_marker_name(ty));
        push_profile_text_node(module, shape_node.clone(), payload_shape_marker_name(ty));
        connect_project_contract_node(
            module,
            &class_node,
            &resolve_project_profile_target_name("data", unit, "marker:uplink_payload_class"),
        );
        connect_project_contract_node(
            module,
            &shape_node,
            &resolve_project_profile_target_name("data", unit, "marker:uplink_payload_shape"),
        );
    }
    if let Some(ty) = downlink_payload.as_ref() {
        let class_node = format!(
            "project_profile_data_{}_downlink_payload_class_type",
            sanitize_ident(unit)
        );
        let shape_node = format!(
            "project_profile_data_{}_downlink_payload_shape_type",
            sanitize_ident(unit)
        );
        push_profile_text_node(module, class_node.clone(), payload_class_marker_name(ty));
        push_profile_text_node(module, shape_node.clone(), payload_shape_marker_name(ty));
        connect_project_contract_node(
            module,
            &class_node,
            &resolve_project_profile_target_name("data", unit, "marker:downlink_payload_class"),
        );
        connect_project_contract_node(
            module,
            &shape_node,
            &resolve_project_profile_target_name("data", unit, "marker:downlink_payload_shape"),
        );
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
    module: &mut YirModule,
) -> Result<(), String> {
    let Some(contract) = infer_shader_packet_contract(project, unit)? else {
        return Ok(());
    };
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
    Ok(())
}

fn materialize_kernel_type_contract_nodes(
    project: &LoadedProject,
    unit: &str,
    module: &mut YirModule,
) -> Result<(), String> {
    let Some(summary) = infer_kernel_slot_contract_summary(project, unit)? else {
        return Ok(());
    };
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
