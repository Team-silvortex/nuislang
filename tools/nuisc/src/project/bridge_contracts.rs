use nuis_semantics::model::{NirDataFlowState, NirResultStage, NirTypeRef};
use yir_core::YirModule;

use super::type_contracts::{connect_project_contract_node, push_profile_text_node};
use super::{
    infer_project_route_payload_type, resolve_project_profile_target_name, sanitize_ident,
    split_domain_unit, LoadedProject, ProjectLinkBridgeContract, ProjectLinkStageContract,
};

pub(super) fn materialize_project_bridge_contract_nodes(
    project: &LoadedProject,
    module: &mut YirModule,
) -> Result<(), String> {
    for link in &project.manifest.links {
        let Some(via) = &link.via else {
            continue;
        };
        let (via_domain, via_unit) = split_domain_unit(via)?;
        if via_domain != "data" {
            continue;
        }
        let bridge = build_project_link_bridge_contract(project, &link.from, &link.to, via)?;
        let id = project_link_contract_id(&link.from, &link.to, via);
        let stage_node = format!("project_link_{id}_bridge_stage_type");
        let uplink_payload_node = format!("project_link_{id}_uplink_bridge_payload_type");
        let downlink_payload_node = format!("project_link_{id}_downlink_bridge_payload_type");
        push_profile_text_node(
            module,
            stage_node.clone(),
            format!(
                "uplink={};downlink={}",
                bridge.stages.uplink.render(),
                bridge.stages.downlink.render()
            ),
        );
        push_profile_text_node(
            module,
            uplink_payload_node.clone(),
            bridge
                .uplink_payload
                .as_ref()
                .map(NirTypeRef::render)
                .unwrap_or_else(|| "unknown".to_owned()),
        );
        push_profile_text_node(
            module,
            downlink_payload_node.clone(),
            bridge
                .downlink_payload
                .as_ref()
                .map(NirTypeRef::render)
                .unwrap_or_else(|| "unknown".to_owned()),
        );
        connect_project_contract_node(
            module,
            &stage_node,
            &resolve_project_profile_target_name("data", &via_unit, "marker:uplink_window_policy"),
        );
        connect_project_contract_node(
            module,
            &stage_node,
            &resolve_project_profile_target_name(
                "data",
                &via_unit,
                "marker:downlink_window_policy",
            ),
        );
        connect_project_contract_node(
            module,
            &uplink_payload_node,
            &resolve_project_profile_target_name("data", &via_unit, "marker:uplink_payload_shape"),
        );
        connect_project_contract_node(
            module,
            &downlink_payload_node,
            &resolve_project_profile_target_name(
                "data",
                &via_unit,
                "marker:downlink_payload_shape",
            ),
        );
    }
    Ok(())
}

pub(super) fn required_project_link_stage_contract(
    from: &str,
    to: &str,
    via: &str,
) -> Result<ProjectLinkStageContract, String> {
    let (from_domain, _) = split_domain_unit(from)?;
    let (to_domain, _) = split_domain_unit(to)?;
    let (via_domain, _) = split_domain_unit(via)?;
    if via_domain != "data" {
        return Err(format!(
            "mediator `{via}` is outside the current staged bridge model"
        ));
    }

    let cpu_edge = from_domain == "cpu" || to_domain == "cpu";
    let hetero_peer = matches!(
        (from_domain.as_str(), to_domain.as_str()),
        ("cpu", "shader")
            | ("shader", "cpu")
            | ("cpu", "kernel")
            | ("kernel", "cpu")
            | ("cpu", "cpu")
    );
    if !cpu_edge || !hetero_peer {
        return Err(format!(
            "current staged bridges only support cpu<->cpu, cpu<->shader, and cpu<->kernel over `data.*`"
        ));
    }

    Ok(ProjectLinkStageContract {
        uplink: NirResultStage::Data(NirDataFlowState::Windowed),
        downlink: NirResultStage::Data(NirDataFlowState::Windowed),
    })
}

pub(super) fn validate_project_link_stage_contract(
    from: &str,
    to: &str,
    via: &str,
    contract: ProjectLinkStageContract,
) -> Result<(), String> {
    if contract.uplink != NirResultStage::Data(NirDataFlowState::Windowed)
        || contract.downlink != NirResultStage::Data(NirDataFlowState::Windowed)
    {
        return Err(format!(
            "project link `{from}` -> `{to}` via `{via}` requires staged fabric bridge `uplink={}` `downlink={}`",
            contract.uplink.render(),
            contract.downlink.render()
        ));
    }
    Ok(())
}

pub(super) fn build_project_link_bridge_contract(
    project: &LoadedProject,
    from: &str,
    to: &str,
    via: &str,
) -> Result<ProjectLinkBridgeContract, String> {
    let (_, data_unit) = split_domain_unit(via)?;
    let uplink_source = resolve_bridge_payload_source(from, to, true)?;
    let downlink_source = resolve_bridge_payload_source(from, to, false)?;
    let stages = required_project_link_stage_contract(from, to, via)?;
    let uplink_payload =
        infer_project_route_payload_type(project, &uplink_source, &data_unit, true)?;
    let downlink_payload =
        infer_project_route_payload_type(project, &downlink_source, &data_unit, false)?;

    validate_bridge_stage_payload(
        "uplink",
        from,
        to,
        via,
        stages.uplink,
        uplink_payload.as_ref(),
    )?;
    validate_bridge_stage_payload(
        "downlink",
        from,
        to,
        via,
        stages.downlink,
        downlink_payload.as_ref(),
    )?;

    Ok(ProjectLinkBridgeContract {
        stages,
        uplink_payload,
        downlink_payload,
    })
}

fn project_link_contract_id(from: &str, to: &str, via: &str) -> String {
    format!(
        "{}_to_{}_via_{}",
        sanitize_ident(from),
        sanitize_ident(to),
        sanitize_ident(via)
    )
}

fn resolve_bridge_payload_source(from: &str, to: &str, uplink: bool) -> Result<String, String> {
    let (from_domain, _) = split_domain_unit(from)?;
    let (to_domain, _) = split_domain_unit(to)?;
    if from_domain == "cpu" {
        return Ok(from.to_owned());
    }
    if to_domain == "cpu" {
        return Ok(to.to_owned());
    }
    Ok(if uplink { from } else { to }.to_owned())
}

fn validate_bridge_stage_payload(
    direction: &str,
    from: &str,
    to: &str,
    via: &str,
    stage: NirResultStage,
    payload: Option<&NirTypeRef>,
) -> Result<(), String> {
    let payload = payload.ok_or_else(|| {
        format!(
            "project link `{from}` -> `{to}` via `{via}` requires a `{direction}` payload contract for stage `{}`",
            stage.render()
        )
    })?;
    stage.validate_payload(payload).map_err(|error| {
        format!(
            "project link `{from}` -> `{to}` via `{via}` has invalid `{direction}` payload contract `{}` for stage `{}`: {error}",
            payload.render(),
            stage.render()
        )
    })
}
