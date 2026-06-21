use nuis_semantics::model::{NirResultStage, NirTypeRef};
use yir_core::YirModule;

use crate::data_markers::supports_staged_data_bridge_pair;

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
        push_profile_text_node(
            module,
            stage_node.clone(),
            bridge
                .stages
                .directions()
                .into_iter()
                .map(|(direction, stage)| format!("{direction}={}", stage.render()))
                .collect::<Vec<_>>()
                .join(";"),
        );
        for direction in bridge_directions() {
            let payload_node = format!("project_link_{id}_{}_bridge_payload_type", direction.name);
            push_profile_text_node(
                module,
                payload_node.clone(),
                direction
                    .payload(&bridge)
                    .map(NirTypeRef::render)
                    .unwrap_or_else(|| "unknown".to_owned()),
            );
            connect_project_contract_node(
                module,
                &stage_node,
                &resolve_project_profile_target_name(
                    "data",
                    &via_unit,
                    direction.window_policy_marker,
                ),
            );
            connect_project_contract_node(
                module,
                &payload_node,
                &resolve_project_profile_target_name(
                    "data",
                    &via_unit,
                    direction.payload_shape_marker,
                ),
            );
        }
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

    if !supports_staged_data_bridge_pair(&from_domain, &to_domain) {
        return Err(format!(
            "current staged bridges only support cpu<->cpu, cpu<->shader, cpu<->kernel, and cpu<->network over `data.*`"
        ));
    }

    Ok(ProjectLinkStageContract::windowed_data_bridge())
}

pub(super) fn validate_project_link_stage_contract(
    from: &str,
    to: &str,
    via: &str,
    contract: ProjectLinkStageContract,
) -> Result<(), String> {
    if !contract.is_windowed_data_bridge() {
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
    let stages = required_project_link_stage_contract(from, to, via)?;
    let mut payloads = [None, None];
    for direction in bridge_directions() {
        let source = resolve_bridge_payload_source(from, to, direction.is_uplink)?;
        let payload =
            infer_project_route_payload_type(project, &source, &data_unit, direction.is_uplink)?;
        validate_bridge_stage_payload(
            direction.name,
            from,
            to,
            via,
            direction.stage(stages),
            payload.as_ref(),
        )?;
        payloads[if direction.is_uplink { 0 } else { 1 }] = payload;
    }

    Ok(ProjectLinkBridgeContract { stages, payloads })
}

#[derive(Clone, Copy)]
struct BridgeDirection {
    name: &'static str,
    is_uplink: bool,
    payload_shape_marker: &'static str,
    window_policy_marker: &'static str,
}

impl BridgeDirection {
    fn stage(self, contract: ProjectLinkStageContract) -> NirResultStage {
        if self.is_uplink {
            contract.uplink
        } else {
            contract.downlink
        }
    }

    fn payload<'a>(self, contract: &'a ProjectLinkBridgeContract) -> Option<&'a NirTypeRef> {
        contract.payload(self.is_uplink)
    }
}

fn bridge_directions() -> [BridgeDirection; 2] {
    [
        BridgeDirection {
            name: "uplink",
            is_uplink: true,
            payload_shape_marker: "marker:uplink_payload_shape",
            window_policy_marker: "marker:uplink_window_policy",
        },
        BridgeDirection {
            name: "downlink",
            is_uplink: false,
            payload_shape_marker: "marker:downlink_payload_shape",
            window_policy_marker: "marker:downlink_window_policy",
        },
    ]
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
