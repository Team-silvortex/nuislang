use std::collections::BTreeMap;

use yir_core::{EdgeKind, Node, Resource, YirModule};

use crate::scheduler_lane_contracts::{
    verify_scheduler_bridge_capability_contract_text, verify_scheduler_clock_contract_text,
    verify_scheduler_lane_capability_contract_text, verify_scheduler_lane_contract_text,
    verify_scheduler_result_lane_contract_text,
};
use crate::scheduler_observer_contracts::{
    verify_scheduler_observer_branch_class_contract_text,
    verify_scheduler_observer_role_variant_contract_text,
    verify_scheduler_observer_scope_class_contract_text,
    verify_scheduler_observer_source_class_contract_text,
    verify_scheduler_observer_stage_class_contract_text,
    verify_scheduler_result_capability_contract_text,
    verify_scheduler_summary_capability_contract_text,
    verify_scheduler_summary_class_contract_text,
};

pub(crate) fn verify_scheduler_contract_nodes(
    module: &YirModule,
    resources: &BTreeMap<String, &Resource>,
    nodes: &BTreeMap<String, &Node>,
) -> Result<(), String> {
    for node in &module.nodes {
        if node.op.module != "cpu" || node.op.instruction != "text" {
            continue;
        }
        let Some(contract) = classify_scheduler_contract_node(node.name.as_str()) else {
            continue;
        };
        let value = node
            .op
            .args
            .first()
            .map(|value| value.trim())
            .ok_or_else(|| {
                format!(
                    "scheduler contract node `{}` must carry a canonical text payload",
                    node.name
                )
            })?;
        if value.is_empty() {
            return Err(format!(
                "scheduler contract node `{}` must carry a non-empty canonical text payload",
                node.name
            ));
        }
        let targets = module
            .edges
            .iter()
            .filter(|edge| {
                edge.from == node.name
                    && matches!(edge.kind, EdgeKind::Dep | EdgeKind::CrossDomainExchange)
            })
            .map(|edge| edge.to.as_str())
            .collect::<Vec<_>>();
        if targets.is_empty() {
            return Err(format!(
                "scheduler contract node `{}` requires at least one dep/xfer edge into its domain anchor",
                node.name
            ));
        }
        for target_name in &targets {
            let target = nodes.get(*target_name).copied().ok_or_else(|| {
                format!(
                    "scheduler contract node `{}` references unknown target `{}`",
                    node.name, target_name
                )
            })?;
            let target_resource = resources.get(&target.resource).copied().ok_or_else(|| {
                format!(
                    "scheduler contract node `{}` references target `{}` with unknown resource `{}`",
                    node.name, target.name, target.resource
                )
            })?;
            if target_resource.kind.family() != contract.family {
                return Err(format!(
                    "scheduler contract node `{}` is declared for `{}`, but targets `{}` on `{}`",
                    node.name,
                    contract.family,
                    target.name,
                    target_resource.kind.family()
                ));
            }
        }

        match contract.kind {
            SchedulerContractKind::LanePolicy => {
                verify_scheduler_lane_contract_text(node.name.as_str(), contract.family, value)?
            }
            SchedulerContractKind::LaneCapability => {
                verify_scheduler_lane_capability_contract_text(
                    nodes,
                    node.name.as_str(),
                    contract.family,
                    value,
                )?
            }
            SchedulerContractKind::BridgeCapability => {
                verify_scheduler_bridge_capability_contract_text(
                    nodes,
                    node.name.as_str(),
                    contract.family,
                    value,
                )?
            }
            SchedulerContractKind::Clock => {
                verify_scheduler_clock_contract_text(node.name.as_str(), contract.family, value)?
            }
            SchedulerContractKind::ResultLane => verify_scheduler_result_lane_contract_text(
                nodes,
                node.name.as_str(),
                contract.family,
                value,
            )?,
            SchedulerContractKind::ResultCapability => {
                verify_scheduler_result_capability_contract_text(
                    nodes,
                    node.name.as_str(),
                    contract.family,
                    value,
                )?
            }
            SchedulerContractKind::ObserverRoleVariant => {
                verify_scheduler_observer_role_variant_contract_text(
                    nodes,
                    node.name.as_str(),
                    contract.family,
                    value,
                )?
            }
            SchedulerContractKind::SummaryCapability => {
                verify_scheduler_summary_capability_contract_text(
                    nodes,
                    node.name.as_str(),
                    contract.family,
                    value,
                )?
            }
            SchedulerContractKind::SummaryClass => verify_scheduler_summary_class_contract_text(
                nodes,
                node.name.as_str(),
                contract.family,
                value,
            )?,
            SchedulerContractKind::ObserverSourceClass => {
                verify_scheduler_observer_source_class_contract_text(
                    nodes,
                    node.name.as_str(),
                    contract.family,
                    value,
                )?
            }
            SchedulerContractKind::ObserverStageClass => {
                verify_scheduler_observer_stage_class_contract_text(
                    nodes,
                    node.name.as_str(),
                    contract.family,
                    value,
                )?
            }
            SchedulerContractKind::ObserverScopeClass => {
                verify_scheduler_observer_scope_class_contract_text(
                    nodes,
                    node.name.as_str(),
                    contract.family,
                    value,
                )?
            }
            SchedulerContractKind::ObserverBranchClass => {
                verify_scheduler_observer_branch_class_contract_text(
                    nodes,
                    node.name.as_str(),
                    contract.family,
                    value,
                )?
            }
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum SchedulerContractKind {
    LanePolicy,
    LaneCapability,
    BridgeCapability,
    Clock,
    ResultLane,
    ResultCapability,
    ObserverRoleVariant,
    SummaryCapability,
    SummaryClass,
    ObserverSourceClass,
    ObserverStageClass,
    ObserverScopeClass,
    ObserverBranchClass,
}

struct SchedulerContract<'a> {
    family: &'a str,
    kind: SchedulerContractKind,
}

fn classify_scheduler_contract_node(name: &str) -> Option<SchedulerContract<'_>> {
    if let Some(family) = name
        .strip_prefix("scheduler_contract_")
        .and_then(|suffix| suffix.strip_suffix("_lane_policy_type"))
    {
        return Some(SchedulerContract {
            family,
            kind: SchedulerContractKind::LanePolicy,
        });
    }
    if let Some(family) = name
        .strip_prefix("scheduler_contract_")
        .and_then(|suffix| suffix.strip_suffix("_lane_capability_type"))
    {
        return Some(SchedulerContract {
            family,
            kind: SchedulerContractKind::LaneCapability,
        });
    }
    if let Some(family) = name
        .strip_prefix("scheduler_contract_")
        .and_then(|suffix| suffix.strip_suffix("_bridge_capability_type"))
    {
        return Some(SchedulerContract {
            family,
            kind: SchedulerContractKind::BridgeCapability,
        });
    }
    if let Some(family) = name
        .strip_prefix("scheduler_contract_")
        .and_then(|suffix| suffix.strip_suffix("_clock_type"))
    {
        return Some(SchedulerContract {
            family,
            kind: SchedulerContractKind::Clock,
        });
    }
    if let Some(family) = name
        .strip_prefix("scheduler_contract_")
        .and_then(|suffix| suffix.strip_suffix("_result_lane_type"))
    {
        return Some(SchedulerContract {
            family,
            kind: SchedulerContractKind::ResultLane,
        });
    }
    if let Some(family) = name
        .strip_prefix("scheduler_contract_")
        .and_then(|suffix| suffix.strip_suffix("_result_capability_type"))
    {
        return Some(SchedulerContract {
            family,
            kind: SchedulerContractKind::ResultCapability,
        });
    }
    if let Some(family) = name
        .strip_prefix("scheduler_contract_")
        .and_then(|suffix| suffix.strip_suffix("_observer_role_variant_type"))
    {
        return Some(SchedulerContract {
            family,
            kind: SchedulerContractKind::ObserverRoleVariant,
        });
    }
    if let Some(family) = name
        .strip_prefix("scheduler_contract_")
        .and_then(|suffix| suffix.strip_suffix("_summary_capability_type"))
    {
        return Some(SchedulerContract {
            family,
            kind: SchedulerContractKind::SummaryCapability,
        });
    }
    if let Some(family) = name
        .strip_prefix("scheduler_contract_")
        .and_then(|suffix| suffix.strip_suffix("_summary_class_type"))
    {
        return Some(SchedulerContract {
            family,
            kind: SchedulerContractKind::SummaryClass,
        });
    }
    if let Some(family) = name
        .strip_prefix("scheduler_contract_")
        .and_then(|suffix| suffix.strip_suffix("_observer_source_class_type"))
    {
        return Some(SchedulerContract {
            family,
            kind: SchedulerContractKind::ObserverSourceClass,
        });
    }
    if let Some(family) = name
        .strip_prefix("scheduler_contract_")
        .and_then(|suffix| suffix.strip_suffix("_observer_stage_class_type"))
    {
        return Some(SchedulerContract {
            family,
            kind: SchedulerContractKind::ObserverStageClass,
        });
    }
    if let Some(family) = name
        .strip_prefix("scheduler_contract_")
        .and_then(|suffix| suffix.strip_suffix("_observer_scope_class_type"))
    {
        return Some(SchedulerContract {
            family,
            kind: SchedulerContractKind::ObserverScopeClass,
        });
    }
    if let Some(family) = name
        .strip_prefix("scheduler_contract_")
        .and_then(|suffix| suffix.strip_suffix("_observer_branch_class_type"))
    {
        return Some(SchedulerContract {
            family,
            kind: SchedulerContractKind::ObserverBranchClass,
        });
    }
    None
}
