use std::{collections::BTreeSet, path::Path};

use super::{
    LinkPlanClockDomain, LinkPlanClockEdge, LinkPlanClockProtocol, LinkPlanClockValidationSummary,
    LinkPlanDomainUnit, LinkPlanHeteroCalculate, LinkPlanLifecycle,
};

pub(crate) fn derive_clock_protocol(
    lifecycle: &LinkPlanLifecycle,
    default_time_mode: &str,
    domain_units: &[LinkPlanDomainUnit],
    hetero_calculate: &LinkPlanHeteroCalculate,
) -> LinkPlanClockProtocol {
    let mut domains = domain_units
        .iter()
        .enumerate()
        .map(|(index, unit)| clock_domain_for_unit(index, unit))
        .collect::<Vec<_>>();
    domains.sort_by(|left, right| {
        (
            left.domain_family.as_str(),
            left.clock_domain_id.as_str(),
            left.package_id.as_str(),
        )
            .cmp(&(
                right.domain_family.as_str(),
                right.clock_domain_id.as_str(),
                right.package_id.as_str(),
            ))
    });
    domains.dedup_by(|left, right| {
        left.domain_family == right.domain_family && left.clock_domain_id == right.clock_domain_id
    });
    for (index, domain) in domains.iter_mut().enumerate() {
        domain.index = index;
    }

    let mut edges = Vec::new();
    for domain in &domains {
        edges.push(LinkPlanClockEdge {
            index: edges.len(),
            from: "global.clock.root.v1".to_owned(),
            to: domain.clock_domain_id.clone(),
            relation: domain.clock_bridge_default.clone(),
            source: "nustar.clock_bridge_default".to_owned(),
        });
    }
    for node in &hetero_calculate.nodes {
        edges.push(LinkPlanClockEdge {
            index: edges.len(),
            from: node.wait_on.join("|"),
            to: node.timestamp.clone(),
            relation: "happens-before".to_owned(),
            source: format!("hetero.node.{}", node.index),
        });
    }
    for segment in &hetero_calculate.data_segments {
        edges.push(LinkPlanClockEdge {
            index: edges.len(),
            from: segment.wait_event.clone(),
            to: segment.commit_event.clone(),
            relation: "data-segment-commit".to_owned(),
            source: format!("hetero.data_segment.{}", segment.index),
        });
    }

    let mut plan = LinkPlanClockProtocol {
        schema: "nuis-clock-protocol-v1".to_owned(),
        mode: if hetero_calculate.nodes.is_empty() {
            "host-lifecycle-clock".to_owned()
        } else {
            "heterogeneous-lifecycle-clock".to_owned()
        },
        source: "registry+lifecycle+hetero-linker".to_owned(),
        default_time_mode: default_time_mode.to_owned(),
        lifecycle_tick_policy: lifecycle.tick_policy.clone(),
        domains,
        edges,
        validation: LinkPlanClockValidationSummary {
            checked: 0,
            valid: true,
            issues: Vec::new(),
        },
    };
    plan.validation = validate_clock_protocol(&plan, lifecycle);
    plan
}

pub(crate) fn validate_clock_protocol(
    plan: &LinkPlanClockProtocol,
    lifecycle: &LinkPlanLifecycle,
) -> LinkPlanClockValidationSummary {
    let mut checked = 0;
    let mut issues = Vec::new();

    checked += 1;
    if plan.schema != "nuis-clock-protocol-v1" {
        issues.push(format!(
            "unexpected clock protocol schema `{}`",
            plan.schema
        ));
    }
    checked += 1;
    if plan.default_time_mode.is_empty() {
        issues.push("clock protocol default_time_mode is empty".to_owned());
    }
    checked += 1;
    if !lifecycle
        .hook_surface
        .iter()
        .any(|hook| hook == "on_scheduler_tick")
    {
        issues.push("clock protocol requires lifecycle hook `on_scheduler_tick`".to_owned());
    }

    let mut domain_ids = BTreeSet::new();
    for domain in &plan.domains {
        checked += 1;
        if domain.clock_domain_id.is_empty() {
            issues.push(format!(
                "clock domain `{}` has empty clock_domain_id",
                domain.domain_family
            ));
        }
        checked += 1;
        if !domain_ids.insert(domain.clock_domain_id.clone()) {
            issues.push(format!(
                "duplicate clock domain id `{}`",
                domain.clock_domain_id
            ));
        }
        checked += 1;
        if domain.clock_kind.is_empty()
            || domain.clock_epoch_kind.is_empty()
            || domain.clock_resolution.is_empty()
            || domain.clock_bridge_default.is_empty()
        {
            issues.push(format!(
                "clock domain `{}` has incomplete clock contract",
                domain.clock_domain_id
            ));
        }
    }

    for (expected_index, edge) in plan.edges.iter().enumerate() {
        checked += 1;
        if edge.index != expected_index {
            issues.push(format!(
                "clock edge `{}` -> `{}` index mismatch: expected {}, found {}",
                edge.from, edge.to, expected_index, edge.index
            ));
        }
        checked += 1;
        if edge.from.is_empty() || edge.to.is_empty() || edge.relation.is_empty() {
            issues.push(format!(
                "clock edge `{}` -> `{}` has incomplete relation",
                edge.from, edge.to
            ));
        }
        checked += 1;
        if edge.relation == "data-segment-commit"
            && (!edge.from.ends_with(".complete") || !edge.to.ends_with(".data_commit"))
        {
            issues.push(format!(
                "clock data segment edge `{}` -> `{}` must connect complete to data_commit",
                edge.from, edge.to
            ));
        }
    }

    LinkPlanClockValidationSummary {
        checked,
        valid: issues.is_empty(),
        issues,
    }
}

fn clock_domain_for_unit(index: usize, unit: &LinkPlanDomainUnit) -> LinkPlanClockDomain {
    let manifest = crate::registry::load_manifest_for_domain(
        Path::new("nustar-packages"),
        &unit.domain_family,
    )
    .ok();
    let capability = manifest.as_ref().map(crate::registry::capability_summary);
    LinkPlanClockDomain {
        index,
        domain_family: unit.domain_family.clone(),
        package_id: unit.package_id.clone(),
        clock_domain_id: capability
            .as_ref()
            .map(|summary| summary.clock.domain_id.clone())
            .unwrap_or_else(|| format!("{}.clock.unregistered.v1", unit.domain_family)),
        clock_kind: capability
            .as_ref()
            .map(|summary| summary.clock.kind.clone())
            .unwrap_or_else(|| "unregistered".to_owned()),
        clock_epoch_kind: capability
            .as_ref()
            .map(|summary| summary.clock.epoch_kind.clone())
            .unwrap_or_else(|| "unregistered-epoch".to_owned()),
        clock_resolution: capability
            .as_ref()
            .map(|summary| summary.clock.resolution.clone())
            .unwrap_or_else(|| "unregistered-resolution".to_owned()),
        clock_bridge_default: capability
            .as_ref()
            .map(|summary| summary.clock.bridge_default.clone())
            .unwrap_or_else(|| "unregistered-bridge".to_owned()),
        lifecycle_hook: lifecycle_hook_for_domain(&unit.domain_family),
    }
}

fn lifecycle_hook_for_domain(domain_family: &str) -> String {
    match domain_family {
        "network" => "on_network_bridge_progress".to_owned(),
        "kernel" | "shader" | "data" => "on_hetero_submission_progress".to_owned(),
        _ => "on_scheduler_tick".to_owned(),
    }
}
