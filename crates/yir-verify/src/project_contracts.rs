use std::collections::BTreeMap;

use yir_core::{EdgeKind, Node, YirModule};

use crate::project_abi_contracts::{
    verify_abi_graph_summary_text, verify_abi_selection_contract_text,
    verify_abi_selection_summary_text,
};
use crate::project_target_contracts::{
    verify_cpu_target_contract_text, verify_kernel_slot_contract_text, verify_target_contract_text,
};

pub(crate) fn verify_project_type_contract_nodes(module: &YirModule) -> Result<(), String> {
    let nodes = module
        .nodes
        .iter()
        .map(|node| (node.name.as_str(), node))
        .collect::<BTreeMap<_, _>>();

    for node in &module.nodes {
        if node.op.module != "cpu" || node.op.instruction != "text" {
            continue;
        }

        let Some(contract) = classify_project_contract_node(node.name.as_str()) else {
            continue;
        };
        let value = node
            .op
            .args
            .first()
            .map(|value| value.trim())
            .ok_or_else(|| {
                format!(
                    "project contract node `{}` must carry a canonical type payload",
                    node.name
                )
            })?;
        if value.is_empty() {
            return Err(format!(
                "project contract node `{}` must carry a non-empty canonical type payload",
                node.name
            ));
        }

        let target = nodes
            .get(contract.target.as_str())
            .copied()
            .ok_or_else(|| {
                format!(
                    "project contract node `{}` references unknown target `{}`",
                    node.name, contract.target
                )
            })?;
        let has_link = module.edges.iter().any(|edge| {
            edge.from == node.name
                && edge.to == contract.target
                && matches!(edge.kind, EdgeKind::Dep | EdgeKind::CrossDomainExchange)
        });
        if !has_link {
            return Err(format!(
                "project contract node `{}` requires dep/xfer edge into `{}`",
                node.name, contract.target
            ));
        }

        match contract.kind {
            ProjectContractKind::AbiGraphSummary => {
                verify_abi_graph_summary_text(node.name.as_str(), value, target)?;
            }
            ProjectContractKind::AbiSelectionSummary => {
                verify_abi_selection_summary_text(node.name.as_str(), value, target)?;
            }
            ProjectContractKind::DataPayloadClass | ProjectContractKind::ShaderPacketClass => {
                require_prefixed_contract_value(node.name.as_str(), value, "PayloadClass")?;
            }
            ProjectContractKind::DataPayloadShape | ProjectContractKind::ShaderPacketShape => {
                require_prefixed_contract_value(node.name.as_str(), value, "PayloadShape")?;
            }
            ProjectContractKind::DataHandleTableSchema | ProjectContractKind::ShaderPacketType => {}
            ProjectContractKind::BridgeStageContract => {
                verify_bridge_stage_contract_text(node.name.as_str(), value)?;
            }
            ProjectContractKind::BridgePayloadContract(direction) => {
                verify_bridge_payload_contract_text(
                    &nodes,
                    node.name.as_str(),
                    value,
                    target,
                    direction,
                )?;
            }
            ProjectContractKind::KernelSlotContract => {
                verify_kernel_slot_contract_text(node.name.as_str(), value, target)?;
            }
            ProjectContractKind::KernelTargetContract => {
                verify_target_contract_text(node.name.as_str(), value, target, "kernel")?;
            }
            ProjectContractKind::KernelAbiSelectionContract => {
                verify_abi_selection_contract_text(node.name.as_str(), value, target, "kernel")?;
            }
            ProjectContractKind::ShaderTargetContract => {
                verify_target_contract_text(node.name.as_str(), value, target, "shader")?;
            }
            ProjectContractKind::ShaderAbiSelectionContract => {
                verify_abi_selection_contract_text(node.name.as_str(), value, target, "shader")?;
            }
            ProjectContractKind::NetworkTargetContract => {
                verify_target_contract_text(node.name.as_str(), value, target, "network")?;
            }
            ProjectContractKind::NetworkAbiSelectionContract => {
                verify_abi_selection_contract_text(node.name.as_str(), value, target, "network")?;
            }
        }
    }

    Ok(())
}

pub(crate) fn verify_lowering_contract_nodes(module: &YirModule) -> Result<(), String> {
    let nodes = module
        .nodes
        .iter()
        .map(|node| (node.name.as_str(), node))
        .collect::<BTreeMap<_, _>>();

    for node in &module.nodes {
        if node.op.module != "cpu" || node.op.instruction != "text" {
            continue;
        }
        if node.name != "lowering_cpu_target_contract_type" {
            continue;
        }
        let value = node
            .op
            .args
            .first()
            .map(|value| value.trim())
            .ok_or_else(|| {
                format!(
                    "lowering contract node `{}` must carry a canonical text payload",
                    node.name
                )
            })?;
        if value.is_empty() {
            return Err(format!(
                "lowering contract node `{}` must carry a non-empty canonical text payload",
                node.name
            ));
        }
        let target_name = "lowering_cpu_target_config";
        let target = nodes.get(target_name).copied().ok_or_else(|| {
            format!(
                "lowering contract node `{}` references unknown target `{target_name}`",
                node.name
            )
        })?;
        let has_link = module.edges.iter().any(|edge| {
            edge.from == node.name
                && edge.to == target_name
                && matches!(edge.kind, EdgeKind::Dep | EdgeKind::CrossDomainExchange)
        });
        if !has_link {
            return Err(format!(
                "lowering contract node `{}` requires dep/xfer edge into `{target_name}`",
                node.name
            ));
        }
        verify_cpu_target_contract_text(node.name.as_str(), value, target)?;
    }

    Ok(())
}

enum ProjectContractKind {
    AbiGraphSummary,
    AbiSelectionSummary,
    DataPayloadClass,
    DataPayloadShape,
    DataHandleTableSchema,
    ShaderPacketType,
    ShaderPacketClass,
    ShaderPacketShape,
    BridgeStageContract,
    BridgePayloadContract(BridgePayloadDirection),
    KernelSlotContract,
    KernelTargetContract,
    KernelAbiSelectionContract,
    ShaderTargetContract,
    ShaderAbiSelectionContract,
    NetworkTargetContract,
    NetworkAbiSelectionContract,
}

#[derive(Clone, Copy)]
enum BridgePayloadDirection {
    Uplink,
    Downlink,
}

struct ProjectContract<'a> {
    kind: ProjectContractKind,
    target: String,
    _unit: &'a str,
}

fn classify_project_contract_node(name: &str) -> Option<ProjectContract<'_>> {
    if name == "project_abi_graph_summary_type" {
        return Some(ProjectContract {
            kind: ProjectContractKind::AbiGraphSummary,
            target: "project_abi_graph_summary_entry".to_owned(),
            _unit: "graph",
        });
    }
    if let Some(domain) = name
        .strip_prefix("project_abi_")
        .and_then(|suffix| suffix.strip_suffix("_selection_summary_type"))
    {
        return Some(ProjectContract {
            kind: ProjectContractKind::AbiSelectionSummary,
            target: format!("project_abi_{domain}_selection_entry"),
            _unit: domain,
        });
    }
    if let Some(id) = name
        .strip_prefix("project_link_")
        .and_then(|suffix| suffix.strip_suffix("_bridge_stage_type"))
    {
        return Some(ProjectContract {
            kind: ProjectContractKind::BridgeStageContract,
            target: project_link_bridge_contract_target(id, true),
            _unit: id,
        });
    }
    if let Some(id) = name
        .strip_prefix("project_link_")
        .and_then(|suffix| suffix.strip_suffix("_uplink_bridge_payload_type"))
    {
        return Some(ProjectContract {
            kind: ProjectContractKind::BridgePayloadContract(BridgePayloadDirection::Uplink),
            target: project_link_bridge_payload_contract_target(id, BridgePayloadDirection::Uplink),
            _unit: id,
        });
    }
    if let Some(id) = name
        .strip_prefix("project_link_")
        .and_then(|suffix| suffix.strip_suffix("_downlink_bridge_payload_type"))
    {
        return Some(ProjectContract {
            kind: ProjectContractKind::BridgePayloadContract(BridgePayloadDirection::Downlink),
            target: project_link_bridge_payload_contract_target(
                id,
                BridgePayloadDirection::Downlink,
            ),
            _unit: id,
        });
    }
    if let Some(unit) = name
        .strip_prefix("project_profile_data_")
        .and_then(|suffix| suffix.strip_suffix("_uplink_payload_class_type"))
    {
        return Some(ProjectContract {
            kind: ProjectContractKind::DataPayloadClass,
            target: format!("project_profile_data_{unit}_uplink_payload_class"),
            _unit: unit,
        });
    }
    if let Some(unit) = name
        .strip_prefix("project_profile_data_")
        .and_then(|suffix| suffix.strip_suffix("_uplink_payload_shape_type"))
    {
        return Some(ProjectContract {
            kind: ProjectContractKind::DataPayloadShape,
            target: format!("project_profile_data_{unit}_uplink_payload_shape"),
            _unit: unit,
        });
    }
    if let Some(unit) = name
        .strip_prefix("project_profile_data_")
        .and_then(|suffix| suffix.strip_suffix("_downlink_payload_class_type"))
    {
        return Some(ProjectContract {
            kind: ProjectContractKind::DataPayloadClass,
            target: format!("project_profile_data_{unit}_downlink_payload_class"),
            _unit: unit,
        });
    }
    if let Some(unit) = name
        .strip_prefix("project_profile_data_")
        .and_then(|suffix| suffix.strip_suffix("_downlink_payload_shape_type"))
    {
        return Some(ProjectContract {
            kind: ProjectContractKind::DataPayloadShape,
            target: format!("project_profile_data_{unit}_downlink_payload_shape"),
            _unit: unit,
        });
    }
    if let Some(unit) = name
        .strip_prefix("project_profile_data_")
        .and_then(|suffix| suffix.strip_suffix("_handle_table_schema_type"))
    {
        return Some(ProjectContract {
            kind: ProjectContractKind::DataHandleTableSchema,
            target: format!("project_profile_data_{unit}_profile_handles"),
            _unit: unit,
        });
    }
    if let Some(unit) = name
        .strip_prefix("project_profile_shader_")
        .and_then(|suffix| suffix.strip_suffix("_packet_type"))
    {
        return Some(ProjectContract {
            kind: ProjectContractKind::ShaderPacketType,
            target: format!("project_profile_shader_{unit}_packet_field_count"),
            _unit: unit,
        });
    }
    if let Some(unit) = name
        .strip_prefix("project_profile_shader_")
        .and_then(|suffix| suffix.strip_suffix("_packet_class_type"))
    {
        return Some(ProjectContract {
            kind: ProjectContractKind::ShaderPacketClass,
            target: format!("project_profile_shader_{unit}_packet_field_count"),
            _unit: unit,
        });
    }
    if let Some(unit) = name
        .strip_prefix("project_profile_shader_")
        .and_then(|suffix| suffix.strip_suffix("_packet_shape_type"))
    {
        return Some(ProjectContract {
            kind: ProjectContractKind::ShaderPacketShape,
            target: format!("project_profile_shader_{unit}_packet_field_count"),
            _unit: unit,
        });
    }
    if let Some(unit) = name
        .strip_prefix("project_profile_kernel_")
        .and_then(|suffix| suffix.strip_suffix("_slot_contract_type"))
    {
        return Some(ProjectContract {
            kind: ProjectContractKind::KernelSlotContract,
            target: format!("project_profile_kernel_{unit}_profile_entry"),
            _unit: unit,
        });
    }
    if let Some(unit) = name
        .strip_prefix("project_profile_kernel_")
        .and_then(|suffix| suffix.strip_suffix("_target_contract_type"))
    {
        return Some(ProjectContract {
            kind: ProjectContractKind::KernelTargetContract,
            target: format!("project_profile_kernel_{unit}_kernel_target_config_auto"),
            _unit: unit,
        });
    }
    if let Some(unit) = name
        .strip_prefix("project_profile_kernel_")
        .and_then(|suffix| suffix.strip_suffix("_abi_selection_contract_type"))
    {
        return Some(ProjectContract {
            kind: ProjectContractKind::KernelAbiSelectionContract,
            target: format!("project_profile_kernel_{unit}_kernel_target_config_auto"),
            _unit: unit,
        });
    }
    if let Some(unit) = name
        .strip_prefix("project_profile_shader_")
        .and_then(|suffix| suffix.strip_suffix("_target_contract_type"))
    {
        return Some(ProjectContract {
            kind: ProjectContractKind::ShaderTargetContract,
            target: format!("project_profile_shader_{unit}_shader_target_config_auto"),
            _unit: unit,
        });
    }
    if let Some(unit) = name
        .strip_prefix("project_profile_shader_")
        .and_then(|suffix| suffix.strip_suffix("_abi_selection_contract_type"))
    {
        return Some(ProjectContract {
            kind: ProjectContractKind::ShaderAbiSelectionContract,
            target: format!("project_profile_shader_{unit}_shader_target_config_auto"),
            _unit: unit,
        });
    }
    if let Some(unit) = name
        .strip_prefix("project_profile_network_")
        .and_then(|suffix| suffix.strip_suffix("_target_contract_type"))
    {
        return Some(ProjectContract {
            kind: ProjectContractKind::NetworkTargetContract,
            target: format!("project_profile_network_{unit}_network_target_config_auto"),
            _unit: unit,
        });
    }
    if let Some(unit) = name
        .strip_prefix("project_profile_network_")
        .and_then(|suffix| suffix.strip_suffix("_abi_selection_contract_type"))
    {
        return Some(ProjectContract {
            kind: ProjectContractKind::NetworkAbiSelectionContract,
            target: format!("project_profile_network_{unit}_network_target_config_auto"),
            _unit: unit,
        });
    }
    None
}

fn project_link_bridge_contract_target(id: &str, stage: bool) -> String {
    let marker = if stage {
        "uplink_window_policy"
    } else {
        "uplink_payload_shape"
    };
    if let Some(data_unit) = id.split("_via_data_").nth(1) {
        return format!("project_profile_data_{data_unit}_{marker}");
    }
    format!("project_link_{id}_missing_target")
}

fn project_link_bridge_payload_contract_target(
    id: &str,
    direction: BridgePayloadDirection,
) -> String {
    let marker = match direction {
        BridgePayloadDirection::Uplink => "uplink_payload_shape",
        BridgePayloadDirection::Downlink => "downlink_payload_shape",
    };
    if let Some(data_unit) = id.split("_via_data_").nth(1) {
        return format!("project_profile_data_{data_unit}_{marker}");
    }
    format!("project_link_{id}_missing_target")
}

fn require_prefixed_contract_value(
    node_name: &str,
    value: &str,
    prefix: &str,
) -> Result<(), String> {
    if !value.starts_with(prefix) {
        return Err(format!(
            "project contract node `{node_name}` must use `{prefix}...`, got `{value}`"
        ));
    }
    Ok(())
}

fn verify_bridge_stage_contract_text(node_name: &str, value: &str) -> Result<(), String> {
    let fields = parse_semicolon_kv_contract(node_name, value, "bridge stage")?;
    let uplink = fields.get("uplink").ok_or_else(|| {
        format!("project contract node `{node_name}` is missing bridge `uplink` stage")
    })?;
    let downlink = fields.get("downlink").ok_or_else(|| {
        format!("project contract node `{node_name}` is missing bridge `downlink` stage")
    })?;
    if *uplink != "windowed" || *downlink != "windowed" {
        return Err(format!(
            "project contract node `{node_name}` currently expects `uplink=windowed;downlink=windowed`, got `{value}`"
        ));
    }
    Ok(())
}

fn verify_bridge_payload_contract_text(
    nodes: &BTreeMap<&str, &Node>,
    node_name: &str,
    value: &str,
    target: &Node,
    direction: BridgePayloadDirection,
) -> Result<(), String> {
    if value.is_empty() || value == "unknown" {
        return Err(format!(
            "project contract node `{node_name}` requires non-empty bridge payload, got `{value}`"
        ));
    }
    if !value.starts_with("Window<") {
        return Err(format!(
            "project contract node `{node_name}` currently expects bridge payload to be `Window<...>`, got `{value}`"
        ));
    }
    let expected_shape = payload_shape_contract_for_bridge_payload(value).ok_or_else(|| {
        format!("project contract node `{node_name}` could not derive payload shape from `{value}`")
    })?;
    let target_shape_node_name = format!("{}_type", target.name);
    let target_shape_node = nodes
        .get(target_shape_node_name.as_str())
        .copied()
        .unwrap_or(target);
    let target_shape = target_shape_node
        .op
        .args
        .first()
        .map(|value| value.as_str())
        .ok_or_else(|| {
            format!(
                "project contract node `{node_name}` targets `{}` without payload shape text",
                target_shape_node.name
            )
        })?;
    if target_shape != expected_shape {
        let direction = match direction {
            BridgePayloadDirection::Uplink => "uplink",
            BridgePayloadDirection::Downlink => "downlink",
        };
        return Err(format!(
            "project contract node `{node_name}` has {direction} bridge payload `{value}` requiring `{expected_shape}`, but target `{}` encodes `{target_shape}`",
            target_shape_node.name
        ));
    }
    Ok(())
}

fn payload_shape_contract_for_bridge_payload(value: &str) -> Option<String> {
    let normalized = value.replace(['<', '>'], "");
    Some(format!(
        "PayloadShape{}",
        sanitize_contract_type_fragment(&normalized)
    ))
}

fn sanitize_contract_type_fragment(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
        .collect()
}

pub(crate) fn parse_semicolon_kv_contract<'a>(
    node_name: &str,
    value: &'a str,
    label: &str,
) -> Result<BTreeMap<&'a str, &'a str>, String> {
    value
        .split(';')
        .filter(|entry| !entry.trim().is_empty())
        .map(|entry| {
            let (key, raw) = entry.split_once('=').ok_or_else(|| {
                format!("project contract node `{node_name}` has invalid {label} field `{entry}`")
            })?;
            Ok((key.trim(), raw.trim()))
        })
        .collect::<Result<BTreeMap<_, _>, String>>()
}
