use std::collections::BTreeMap;

use nuis_semantics::model::{NirDataFlowState, NirResultStage};
use yir_core::YirModule;

use super::data_bridge_directions::data_bridge_directions;
use super::support_contracts::{require_declared_support_surface, support_surface_for_domain};
use super::{
    build_project_link_bridge_contract, data_profile_required_slots_for_link,
    find_profile_call_declared_type, has_edge_to,
    payload_class_marker_name, payload_shape_marker_name, require_declared_profile_slot,
    require_marker_semantic_payload_name, require_profile_semantic_type,
    required_project_link_stage_contract, resolve_project_profile_target_name, split_domain_unit,
    support_profile_slots_for_domain, LoadedProject,
};

pub(super) fn validate_data_profile_for_link(
    module: &YirModule,
    from_endpoint: &str,
    to_endpoint: &str,
    endpoint: &str,
) -> Result<(), String> {
    let (domain, unit) = split_domain_unit(endpoint)?;
    if domain != "data" {
        return Ok(());
    }
    let contract = required_project_link_stage_contract(from_endpoint, to_endpoint, endpoint)?;
    let (from_domain, _) = split_domain_unit(from_endpoint)?;
    let (to_domain, _) = split_domain_unit(to_endpoint)?;
    let declared_support = support_surface_for_domain(&mut BTreeMap::new(), "data")?;
    let declared_slots = support_profile_slots_for_domain("data")?;
    for required_surface in [
        "data.profile.send.uplink.v1",
        "data.profile.send.downlink.v1",
        "data.profile.payload-class.v1",
        "data.profile.payload-shape.v1",
        "data.profile.window-policy.v1",
    ] {
        require_declared_support_surface(&declared_support, "data", &unit, required_surface)?;
    }

    for slot in data_profile_required_slots_for_link(&from_domain, &to_domain) {
        require_declared_profile_slot(&declared_slots, "data", &unit, &slot)?;
        let node_name = resolve_project_profile_target_name("data", &unit, &slot);
        let exists = module.nodes.iter().any(|node| node.name == node_name);
        if !exists {
            return Err(format!(
                "project data unit `data.{}` requires support profile slot `{}` in YIR",
                unit, slot
            ));
        }
    }

    let uplink_nodes = module
        .nodes
        .iter()
        .filter(|node| node.op.is_data_pipe_semantic_op())
        .take(2)
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let downlink_nodes = module
        .nodes
        .iter()
        .filter(|node| node.op.is_data_pipe_semantic_op())
        .skip(2)
        .take(2)
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let uplink_windows = module
        .nodes
        .iter()
        .filter(|node| node.op.is_data_window_semantic_op() && node.name.contains("_uplink_window"))
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let downlink_windows = module
        .nodes
        .iter()
        .filter(|node| {
            node.op.is_data_window_semantic_op() && node.name.contains("_downlink_window")
        })
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    for direction in data_bridge_directions() {
        let payload_marker =
            resolve_project_profile_target_name("data", &unit, direction.payload_class_marker);
        let shape_marker =
            resolve_project_profile_target_name("data", &unit, direction.payload_shape_marker);
        let window_policy_marker =
            resolve_project_profile_target_name("data", &unit, direction.window_policy_marker);
        let pipe_nodes = if direction.is_uplink {
            &uplink_nodes
        } else {
            &downlink_nodes
        };
        let window_nodes = if direction.is_uplink {
            &uplink_windows
        } else {
            &downlink_windows
        };
        let stage = if direction.is_uplink {
            contract.uplink
        } else {
            contract.downlink
        };
        validate_bridge_payload_marker_edges(
            module,
            &unit,
            direction.name,
            &payload_marker,
            &shape_marker,
            pipe_nodes,
            window_nodes,
        )?;
        validate_window_policy_marker_edges(
            module,
            &unit,
            direction.name,
            stage,
            &window_policy_marker,
            window_nodes,
        )?;
    }

    Ok(())
}

pub(super) fn validate_data_profile_token_types(
    project: &LoadedProject,
    from_endpoint: &str,
    to_endpoint: &str,
    endpoint: &str,
) -> Result<(), String> {
    let (domain, unit) = split_domain_unit(endpoint)?;
    if domain != "data" {
        return Ok(());
    }
    let bridge = build_project_link_bridge_contract(project, from_endpoint, to_endpoint, endpoint)?;
    let contract = required_project_link_stage_contract(from_endpoint, to_endpoint, endpoint)?;
    let profile_module = project
        .modules
        .iter()
        .find(|module| module.ast.domain == domain && module.ast.unit == unit)
        .ok_or_else(|| format!("project is missing support module `{endpoint}`"))?;
    let Some(profile_fn) = profile_module
        .ast
        .functions
        .iter()
        .find(|function| function.name == "profile")
    else {
        return Ok(());
    };
    let (from_domain, _) = split_domain_unit(from_endpoint)?;
    let (to_domain, _) = split_domain_unit(to_endpoint)?;

    let handle_table_ty = find_profile_call_declared_type(
        &profile_fn.body,
        &profile_module.ast.type_aliases,
        "data_handle_table",
        None,
    )
    .ok_or_else(|| {
        format!(
            "project data unit `data.{}` requires typed `HandleTable<Schema>` on its data_handle_table binding",
            unit
        )
    })?;
    require_profile_semantic_type(&handle_table_ty, "HandleTable", true, &unit, "handle_table")?;

    for slot in data_profile_required_slots_for_link(&from_domain, &to_domain) {
        if !slot.starts_with("marker:") {
            continue;
        }
        let tag = slot.trim_start_matches("marker:");
        let marker_ty = find_profile_call_declared_type(
            &profile_fn.body,
            &profile_module.ast.type_aliases,
            "data_marker",
            Some(tag),
        )
        .ok_or_else(|| {
            format!(
                "project data unit `data.{}` requires typed `Marker<Tag>` binding for marker `{}`",
                unit, tag
            )
        })?;
        require_profile_semantic_type(&marker_ty, "Marker", true, &unit, tag)?;
    }

    for tag in data_bridge_core_marker_tags() {
        let marker_ty = find_profile_call_declared_type(
            &profile_fn.body,
            &profile_module.ast.type_aliases,
            "data_marker",
            Some(&tag),
        )
        .ok_or_else(|| {
            format!(
                "project data unit `data.{}` requires typed `Marker<Tag>` binding for marker `{}`",
                unit, tag
            )
        })?;
        require_profile_semantic_type(&marker_ty, "Marker", true, &unit, &tag)?;
    }
    for direction in data_bridge_directions() {
        let stage = if direction.is_uplink {
            contract.uplink
        } else {
            contract.downlink
        };
        if stage_requires_window_policy(stage) {
            let marker_ty = find_profile_call_declared_type(
                &profile_fn.body,
                &profile_module.ast.type_aliases,
                "data_marker",
                Some(direction.window_policy_marker.trim_start_matches("marker:")),
            )
            .ok_or_else(|| {
                format!(
                    "project data unit `data.{}` requires typed `Marker<Tag>` binding for marker `{}` because {} bridge stage is `{}`",
                    unit,
                    direction.window_policy_marker.trim_start_matches("marker:"),
                    direction.name,
                    stage.render()
                )
            })?;
            require_marker_semantic_payload_name(
                &marker_ty,
                direction.window_policy_payload,
                &unit,
                direction.window_policy_marker.trim_start_matches("marker:"),
            )?;
        }

        if let Some(payload_ty) = bridge.payload(direction.is_uplink) {
            let class_tag = direction.payload_class_marker.trim_start_matches("marker:");
            let marker_ty = find_profile_call_declared_type(
                &profile_fn.body,
                &profile_module.ast.type_aliases,
                "data_marker",
                Some(class_tag),
            )
            .ok_or_else(|| {
                format!(
                    "project data unit `data.{}` requires typed `Marker<Tag>` binding for marker `{}`",
                    unit, class_tag
                )
            })?;
            require_marker_semantic_payload_name(
                &marker_ty,
                &payload_class_marker_name(payload_ty),
                &unit,
                class_tag,
            )?;

            let shape_tag = direction.payload_shape_marker.trim_start_matches("marker:");
            let marker_ty = find_profile_call_declared_type(
                &profile_fn.body,
                &profile_module.ast.type_aliases,
                "data_marker",
                Some(shape_tag),
            )
            .ok_or_else(|| {
                format!(
                    "project data unit `data.{}` requires typed `Marker<Tag>` binding for marker `{}`",
                    unit, shape_tag
                )
            })?;
            require_marker_semantic_payload_name(
                &marker_ty,
                &payload_shape_marker_name(payload_ty),
                &unit,
                shape_tag,
            )?;
        }
    }

    Ok(())
}

fn stage_requires_window_policy(stage: NirResultStage) -> bool {
    stage == NirResultStage::Data(NirDataFlowState::Windowed)
}

fn validate_bridge_payload_marker_edges(
    module: &YirModule,
    unit: &str,
    direction: &str,
    payload_marker: &str,
    shape_marker: &str,
    pipe_nodes: &[String],
    window_nodes: &[String],
) -> Result<(), String> {
    if !pipe_nodes
        .iter()
        .all(|pipe| has_edge_to(module, payload_marker, pipe))
    {
        return Err(format!(
            "project data unit `data.{}` requires {} payload class to feed all {} pipe nodes",
            unit, direction, direction
        ));
    }
    if !pipe_nodes
        .iter()
        .all(|pipe| has_edge_to(module, shape_marker, pipe))
    {
        return Err(format!(
            "project data unit `data.{}` requires {} payload shape to feed all {} pipe nodes",
            unit, direction, direction
        ));
    }
    if !window_nodes
        .iter()
        .all(|window| has_edge_to(module, shape_marker, window))
    {
        return Err(format!(
            "project data unit `data.{}` requires {} payload shape to feed all {} window nodes",
            unit, direction, direction
        ));
    }
    Ok(())
}

fn validate_window_policy_marker_edges(
    module: &YirModule,
    unit: &str,
    direction: &str,
    stage: NirResultStage,
    policy_marker: &str,
    window_nodes: &[String],
) -> Result<(), String> {
    if !stage_requires_window_policy(stage) {
        return Ok(());
    }
    if !module.nodes.iter().any(|node| node.name == policy_marker) {
        return Err(format!(
            "project data unit `data.{}` requires `{}_window_policy` marker node for bridge stage `{}`",
            unit,
            direction,
            stage.render()
        ));
    }
    if !window_nodes
        .iter()
        .all(|window| has_edge_to(module, policy_marker, window))
    {
        return Err(format!(
            "project data unit `data.{}` requires {} window policy to feed all {} window nodes for bridge stage `{}`",
            unit,
            direction,
            direction,
            stage.render()
        ));
    }
    Ok(())
}

fn data_bridge_core_marker_tags() -> Vec<String> {
    [
        "uplink_pipe",
        "downlink_pipe",
        "uplink_pipe_class",
        "downlink_pipe_class",
        "uplink_payload_class",
        "downlink_payload_class",
        "uplink_payload_shape",
        "downlink_payload_shape",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}
