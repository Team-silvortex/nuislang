use std::collections::BTreeMap;

use nuis_semantics::model::{NirDataFlowState, NirResultStage};
use yir_core::YirModule;

use super::support_contracts::{require_declared_support_surface, support_surface_for_domain};
use super::{
    build_project_link_bridge_contract, data_profile_required_slots_for_link,
    data_support_surface_contract, find_profile_call_declared_type, has_edge_to,
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
    for required_surface in data_support_surface_contract() {
        require_declared_support_surface(&declared_support, "data", &unit, required_surface)?;
    }

    for slot in data_profile_required_slots_for_link(&from_domain, &to_domain) {
        require_declared_profile_slot(&declared_slots, "data", &unit, slot)?;
        let node_name = resolve_project_profile_target_name("data", &unit, slot);
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
    let uplink_payload =
        resolve_project_profile_target_name("data", &unit, "marker:uplink_payload_class");
    let downlink_payload =
        resolve_project_profile_target_name("data", &unit, "marker:downlink_payload_class");
    let uplink_shape =
        resolve_project_profile_target_name("data", &unit, "marker:uplink_payload_shape");
    let downlink_shape =
        resolve_project_profile_target_name("data", &unit, "marker:downlink_payload_shape");
    let uplink_window_policy =
        resolve_project_profile_target_name("data", &unit, "marker:uplink_window_policy");
    let downlink_window_policy =
        resolve_project_profile_target_name("data", &unit, "marker:downlink_window_policy");
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

    if !uplink_nodes
        .iter()
        .all(|pipe| has_edge_to(module, &uplink_payload, pipe))
    {
        return Err(format!(
            "project data unit `data.{}` requires uplink payload class to feed all uplink pipe nodes",
            unit
        ));
    }
    if !uplink_nodes
        .iter()
        .all(|pipe| has_edge_to(module, &uplink_shape, pipe))
    {
        return Err(format!(
            "project data unit `data.{}` requires uplink payload shape to feed all uplink pipe nodes",
            unit
        ));
    }
    if !uplink_windows
        .iter()
        .all(|window| has_edge_to(module, &uplink_shape, window))
    {
        return Err(format!(
            "project data unit `data.{}` requires uplink payload shape to feed all uplink window nodes",
            unit
        ));
    }
    if !downlink_nodes
        .iter()
        .all(|pipe| has_edge_to(module, &downlink_payload, pipe))
    {
        return Err(format!(
            "project data unit `data.{}` requires downlink payload class to feed all downlink pipe nodes",
            unit
        ));
    }
    if !downlink_nodes
        .iter()
        .all(|pipe| has_edge_to(module, &downlink_shape, pipe))
    {
        return Err(format!(
            "project data unit `data.{}` requires downlink payload shape to feed all downlink pipe nodes",
            unit
        ));
    }
    if !downlink_windows
        .iter()
        .all(|window| has_edge_to(module, &downlink_shape, window))
    {
        return Err(format!(
            "project data unit `data.{}` requires downlink payload shape to feed all downlink window nodes",
            unit
        ));
    }
    if contract.uplink == NirResultStage::Data(NirDataFlowState::Windowed) {
        if !module
            .nodes
            .iter()
            .any(|node| node.name == uplink_window_policy)
        {
            return Err(format!(
                "project data unit `data.{}` requires `uplink_window_policy` marker node for bridge stage `{}`",
                unit,
                contract.uplink.render()
            ));
        }
        if !uplink_windows
            .iter()
            .all(|window| has_edge_to(module, &uplink_window_policy, window))
        {
            return Err(format!(
                "project data unit `data.{}` requires uplink window policy to feed all uplink window nodes for bridge stage `{}`",
                unit,
                contract.uplink.render()
            ));
        }
    }
    if contract.downlink == NirResultStage::Data(NirDataFlowState::Windowed) {
        if !module
            .nodes
            .iter()
            .any(|node| node.name == downlink_window_policy)
        {
            return Err(format!(
                "project data unit `data.{}` requires `downlink_window_policy` marker node for bridge stage `{}`",
                unit,
                contract.downlink.render()
            ));
        }
        if !downlink_windows
            .iter()
            .all(|window| has_edge_to(module, &downlink_window_policy, window))
        {
            return Err(format!(
                "project data unit `data.{}` requires downlink window policy to feed all downlink window nodes for bridge stage `{}`",
                unit,
                contract.downlink.render()
            ));
        }
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

    for tag in [
        "uplink_pipe",
        "downlink_pipe",
        "uplink_pipe_class",
        "downlink_pipe_class",
        "uplink_payload_class",
        "downlink_payload_class",
        "uplink_payload_shape",
        "downlink_payload_shape",
    ] {
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
    if contract.uplink == NirResultStage::Data(NirDataFlowState::Windowed) {
        let marker_ty = find_profile_call_declared_type(
            &profile_fn.body,
            &profile_module.ast.type_aliases,
            "data_marker",
            Some("uplink_window_policy"),
        )
        .ok_or_else(|| {
            format!(
                "project data unit `data.{}` requires typed `Marker<Tag>` binding for marker `uplink_window_policy` because uplink bridge stage is `{}`",
                unit,
                contract.uplink.render()
            )
        })?;
        require_marker_semantic_payload_name(
            &marker_ty,
            "UplinkWindowPolicy",
            &unit,
            "uplink_window_policy",
        )?;
    }
    if contract.downlink == NirResultStage::Data(NirDataFlowState::Windowed) {
        let marker_ty = find_profile_call_declared_type(
            &profile_fn.body,
            &profile_module.ast.type_aliases,
            "data_marker",
            Some("downlink_window_policy"),
        )
        .ok_or_else(|| {
            format!(
                "project data unit `data.{}` requires typed `Marker<Tag>` binding for marker `downlink_window_policy` because downlink bridge stage is `{}`",
                unit,
                contract.downlink.render()
            )
        })?;
        require_marker_semantic_payload_name(
            &marker_ty,
            "DownlinkWindowPolicy",
            &unit,
            "downlink_window_policy",
        )?;
    }

    if let Some(uplink_ty) = bridge.uplink_payload.as_ref() {
        let marker_ty = find_profile_call_declared_type(
            &profile_fn.body,
            &profile_module.ast.type_aliases,
            "data_marker",
            Some("uplink_payload_class"),
        )
        .ok_or_else(|| {
            format!(
                "project data unit `data.{}` requires typed `Marker<Tag>` binding for marker `uplink_payload_class`",
                unit
            )
        })?;
        require_marker_semantic_payload_name(
            &marker_ty,
            &payload_class_marker_name(uplink_ty),
            &unit,
            "uplink_payload_class",
        )?;

        let marker_ty = find_profile_call_declared_type(
            &profile_fn.body,
            &profile_module.ast.type_aliases,
            "data_marker",
            Some("uplink_payload_shape"),
        )
        .ok_or_else(|| {
            format!(
                "project data unit `data.{}` requires typed `Marker<Tag>` binding for marker `uplink_payload_shape`",
                unit
            )
        })?;
        require_marker_semantic_payload_name(
            &marker_ty,
            &payload_shape_marker_name(uplink_ty),
            &unit,
            "uplink_payload_shape",
        )?;
    }

    if let Some(downlink_ty) = bridge.downlink_payload.as_ref() {
        let marker_ty = find_profile_call_declared_type(
            &profile_fn.body,
            &profile_module.ast.type_aliases,
            "data_marker",
            Some("downlink_payload_class"),
        )
        .ok_or_else(|| {
            format!(
                "project data unit `data.{}` requires typed `Marker<Tag>` binding for marker `downlink_payload_class`",
                unit
            )
        })?;
        require_marker_semantic_payload_name(
            &marker_ty,
            &payload_class_marker_name(downlink_ty),
            &unit,
            "downlink_payload_class",
        )?;

        let marker_ty = find_profile_call_declared_type(
            &profile_fn.body,
            &profile_module.ast.type_aliases,
            "data_marker",
            Some("downlink_payload_shape"),
        )
        .ok_or_else(|| {
            format!(
                "project data unit `data.{}` requires typed `Marker<Tag>` binding for marker `downlink_payload_shape`",
                unit
            )
        })?;
        require_marker_semantic_payload_name(
            &marker_ty,
            &payload_shape_marker_name(downlink_ty),
            &unit,
            "downlink_payload_shape",
        )?;
    }

    Ok(())
}
