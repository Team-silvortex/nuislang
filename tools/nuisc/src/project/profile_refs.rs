use std::collections::{BTreeMap, BTreeSet};

use crate::data_markers::{directional_bridge_marker_tag, DATA_BRIDGE_HETERO_DOMAINS};
use yir_core::{EdgeKind, OperationDomainFamily, SemanticOp, YirModule};

use super::profile_targets::resolve_project_profile_target_name;

pub(super) fn resolve_project_profile_refs(module: &mut YirModule) -> Result<(), String> {
    let replacements = module
        .nodes
        .iter()
        .filter(|node| node.op.is_cpu_semantic_op(SemanticOp::CpuProjectProfileRef))
        .map(|node| {
            let [domain, unit, slot] = node.op.args.as_slice() else {
                return Err(format!(
                    "project profile ref node `{}` expects `<domain> <unit> <slot>` args",
                    node.name
                ));
            };
            let target = resolve_project_profile_target_name(domain, unit, slot);
            if !module.nodes.iter().any(|candidate| candidate.name == target) {
                return Err(format!(
                    "project profile ref `{}` could not resolve `{}` `{}` slot `{}` into a support-module profile node",
                    node.name, domain, unit, slot
                ));
            }
            Ok((node.name.clone(), target))
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;

    if replacements.is_empty() {
        return Ok(());
    }

    let replacement_sources = replacements.keys().cloned().collect::<BTreeSet<_>>();

    for node in &mut module.nodes {
        if node.op.is_cpu_semantic_op(SemanticOp::CpuProjectProfileRef) {
            continue;
        }
        for arg in &mut node.op.args {
            if let Some(target) = replacements.get(arg) {
                *arg = target.clone();
            } else if let Some((field, value)) = arg.split_once('=') {
                if let Some(target) = replacements.get(value) {
                    *arg = format!("{field}={target}");
                }
            }
        }
    }
    module.edges.retain(|edge| {
        !replacement_sources.contains(&edge.from) && !replacement_sources.contains(&edge.to)
    });
    let replacement_targets = replacements.values().cloned().collect::<BTreeSet<_>>();
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
    let mut extra_dep_edges = Vec::new();
    for node in &module.nodes {
        if node.op.is_cpu_semantic_op(SemanticOp::CpuProjectProfileRef) {
            continue;
        }
        for arg in &node.op.args {
            let dependency = if replacement_targets.contains(arg) {
                Some(arg.as_str())
            } else if let Some((_field, value)) = arg.split_once('=') {
                replacement_targets.contains(value).then_some(value)
            } else {
                None
            };
            let Some(dependency) = dependency else {
                continue;
            };
            let edge_kind = inferred_project_dependency_edge_kind(
                &resource_families,
                &node_resources,
                dependency,
                &node.name,
            );
            let exists = module.edges.iter().any(|edge| {
                edge.kind == edge_kind && edge.from == dependency && edge.to == node.name
            });
            if !exists {
                extra_dep_edges.push(yir_core::Edge {
                    kind: edge_kind,
                    from: dependency.to_owned(),
                    to: node.name.clone(),
                });
            }
        }
    }
    module
        .nodes
        .retain(|node| !node.op.is_cpu_semantic_op(SemanticOp::CpuProjectProfileRef));
    module.edges.extend(extra_dep_edges);
    Ok(())
}

pub(super) fn stitch_shader_profile_edges(module: &mut YirModule) {
    let pass_kind_nodes = module
        .nodes
        .iter()
        .filter(|node| node.name.contains("_pass_kind"))
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let packet_field_count_nodes = module
        .nodes
        .iter()
        .filter(|node| node.name.contains("_packet_field_count"))
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let begin_pass_nodes = module
        .nodes
        .iter()
        .filter(|node| node.op.is_shader_semantic_op(SemanticOp::ShaderBeginPass))
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let draw_nodes = module
        .nodes
        .iter()
        .filter(|node| {
            node.op
                .is_shader_semantic_op(SemanticOp::ShaderDrawInstanced)
        })
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();

    for pass_kind in &pass_kind_nodes {
        for begin_pass in &begin_pass_nodes {
            push_edge_if_missing(module, EdgeKind::CrossDomainExchange, pass_kind, begin_pass);
        }
    }
    for packet_field_count in &packet_field_count_nodes {
        for draw in &draw_nodes {
            push_edge_if_missing(
                module,
                EdgeKind::CrossDomainExchange,
                packet_field_count,
                draw,
            );
        }
    }
}

pub(super) fn stitch_data_profile_edges(module: &mut YirModule) {
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
    let handle_tables = module
        .nodes
        .iter()
        .filter(|node| node.op.semantic_op() == SemanticOp::DataHandleTable)
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let uplink_pipe_markers = module
        .nodes
        .iter()
        .filter(|node| node.op.is_data_marker_tag("uplink_pipe"))
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let uplink_pipe_class_markers = module
        .nodes
        .iter()
        .filter(|node| node.op.is_data_marker_tag("uplink_pipe_class"))
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let uplink_payload_class_markers = module
        .nodes
        .iter()
        .filter(|node| node.op.is_data_marker_tag("uplink_payload_class"))
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let uplink_payload_shape_markers = module
        .nodes
        .iter()
        .filter(|node| node.op.is_data_marker_tag("uplink_payload_shape"))
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let downlink_pipe_markers = module
        .nodes
        .iter()
        .filter(|node| node.op.is_data_marker_tag("downlink_pipe"))
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let downlink_payload_class_markers = module
        .nodes
        .iter()
        .filter(|node| node.op.is_data_marker_tag("downlink_payload_class"))
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let downlink_payload_shape_markers = module
        .nodes
        .iter()
        .filter(|node| node.op.is_data_marker_tag("downlink_payload_shape"))
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let downlink_pipe_class_markers = module
        .nodes
        .iter()
        .filter(|node| node.op.is_data_marker_tag("downlink_pipe_class"))
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let uplink_window_policy_markers = module
        .nodes
        .iter()
        .filter(|node| node.op.is_data_marker_tag("uplink_window_policy"))
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let downlink_window_policy_markers = module
        .nodes
        .iter()
        .filter(|node| node.op.is_data_marker_tag("downlink_window_policy"))
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let directional_markers = DATA_BRIDGE_HETERO_DOMAINS
        .iter()
        .filter_map(|domain| {
            let uplink_tag = directional_bridge_marker_tag("cpu", domain)?;
            let downlink_tag = directional_bridge_marker_tag(domain, "cpu")?;
            let uplink_markers = collect_data_marker_nodes(module, &uplink_tag);
            let downlink_markers = collect_data_marker_nodes(module, &downlink_tag);
            Some(((*domain).to_owned(), uplink_markers, downlink_markers))
        })
        .collect::<Vec<_>>();
    let cpu_nodes = module
        .nodes
        .iter()
        .filter(|node| node.op.is_domain_family(OperationDomainFamily::Cpu))
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let data_pipe_nodes = module
        .nodes
        .iter()
        .filter(|node| node.op.is_data_pipe_semantic_op())
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
    let window_offset = module
        .nodes
        .iter()
        .find(|node| node.name.contains("_window_offset"))
        .map(|node| node.name.clone());
    let uplink_len = module
        .nodes
        .iter()
        .find(|node| node.name.contains("_uplink_len"))
        .map(|node| node.name.clone());
    let downlink_len = module
        .nodes
        .iter()
        .find(|node| node.name.contains("_downlink_len"))
        .map(|node| node.name.clone());

    for handle in &handle_tables {
        for pipe in &data_pipe_nodes {
            push_edge_if_missing(module, EdgeKind::Dep, handle, pipe);
        }
    }
    for (_domain, uplink_markers, downlink_markers) in &directional_markers {
        if let Some(marker) = uplink_markers.first() {
            for pipe in data_pipe_nodes.iter().take(2) {
                push_edge_if_missing(module, EdgeKind::Effect, marker, pipe);
            }
        }
        if let Some(marker) = downlink_markers.first() {
            for pipe in data_pipe_nodes.iter().skip(2).take(2) {
                push_edge_if_missing(module, EdgeKind::Effect, marker, pipe);
            }
        }
    }
    if let Some(marker) = uplink_pipe_markers.first() {
        for pipe in data_pipe_nodes.iter().take(2) {
            push_edge_if_missing(module, EdgeKind::Effect, marker, pipe);
        }
    }
    if let Some(marker) = uplink_pipe_class_markers.first() {
        for pipe in data_pipe_nodes.iter().take(2) {
            push_edge_if_missing(module, EdgeKind::Effect, marker, pipe);
        }
    }
    if let Some(marker) = uplink_payload_class_markers.first() {
        for pipe in data_pipe_nodes.iter().take(2) {
            push_edge_if_missing(module, EdgeKind::Effect, marker, pipe);
        }
    }
    if let Some(marker) = uplink_payload_shape_markers.first() {
        for pipe in data_pipe_nodes.iter().take(2) {
            push_edge_if_missing(module, EdgeKind::Effect, marker, pipe);
        }
        for window in &uplink_windows {
            push_edge_if_missing(module, EdgeKind::Effect, marker, window);
        }
    }
    if let Some(marker) = downlink_pipe_markers.first() {
        for pipe in data_pipe_nodes.iter().skip(2).take(2) {
            push_edge_if_missing(module, EdgeKind::Effect, marker, pipe);
        }
    }
    if let Some(marker) = downlink_pipe_class_markers.first() {
        for pipe in data_pipe_nodes.iter().skip(2).take(2) {
            push_edge_if_missing(module, EdgeKind::Effect, marker, pipe);
        }
    }
    if let Some(marker) = downlink_payload_class_markers.first() {
        for pipe in data_pipe_nodes.iter().skip(2).take(2) {
            push_edge_if_missing(module, EdgeKind::Effect, marker, pipe);
        }
    }
    if let Some(marker) = downlink_payload_shape_markers.first() {
        for pipe in data_pipe_nodes.iter().skip(2).take(2) {
            push_edge_if_missing(module, EdgeKind::Effect, marker, pipe);
        }
        for window in &downlink_windows {
            push_edge_if_missing(module, EdgeKind::Effect, marker, window);
        }
    }
    for window in &uplink_windows {
        if let Some(marker) = uplink_window_policy_markers.first() {
            push_edge_if_missing(module, EdgeKind::Effect, marker, window);
        }
        for pipe in data_pipe_nodes.iter().take(2) {
            push_project_dependency_edge_if_missing(
                module,
                &resource_families,
                &node_resources,
                window,
                pipe,
            );
        }
        if let Some(offset) = &window_offset {
            push_project_dependency_edge_if_missing(
                module,
                &resource_families,
                &node_resources,
                offset,
                window,
            );
        }
        if let Some(len) = &uplink_len {
            push_project_dependency_edge_if_missing(
                module,
                &resource_families,
                &node_resources,
                len,
                window,
            );
        }
    }
    for window in &downlink_windows {
        if let Some(marker) = downlink_window_policy_markers.first() {
            push_edge_if_missing(module, EdgeKind::Effect, marker, window);
        }
        for pipe in data_pipe_nodes.iter().skip(2).take(2) {
            push_project_dependency_edge_if_missing(
                module,
                &resource_families,
                &node_resources,
                window,
                pipe,
            );
        }
        if let Some(offset) = &window_offset {
            push_project_dependency_edge_if_missing(
                module,
                &resource_families,
                &node_resources,
                offset,
                window,
            );
        }
        if let Some(len) = &downlink_len {
            push_project_dependency_edge_if_missing(
                module,
                &resource_families,
                &node_resources,
                len,
                window,
            );
        }
    }
    if uplink_windows.is_empty() {
        if let Some(offset) = &window_offset {
            for pipe in data_pipe_nodes.iter().take(2) {
                push_project_dependency_edge_if_missing(
                    module,
                    &resource_families,
                    &node_resources,
                    offset,
                    pipe,
                );
            }
        }
        if let Some(len) = &uplink_len {
            for pipe in data_pipe_nodes.iter().take(2) {
                push_project_dependency_edge_if_missing(
                    module,
                    &resource_families,
                    &node_resources,
                    len,
                    pipe,
                );
            }
        }
    }
    if downlink_windows.is_empty() {
        if let Some(offset) = &window_offset {
            for pipe in data_pipe_nodes.iter().skip(2).take(2) {
                push_project_dependency_edge_if_missing(
                    module,
                    &resource_families,
                    &node_resources,
                    offset,
                    pipe,
                );
            }
        }
        if let Some(len) = &downlink_len {
            for pipe in data_pipe_nodes.iter().skip(2).take(2) {
                push_project_dependency_edge_if_missing(
                    module,
                    &resource_families,
                    &node_resources,
                    len,
                    pipe,
                );
            }
        }
    }
    for (domain, uplink_markers, downlink_markers) in &directional_markers {
        let Some(family) = operation_domain_family_for_name(domain) else {
            continue;
        };
        let domain_nodes = collect_domain_nodes(module, family);
        if domain_nodes.is_empty() {
            continue;
        }
        if !uplink_markers.is_empty() {
            for source in downlink_windows
                .iter()
                .chain(data_pipe_nodes.iter().skip(2).take(2))
            {
                for domain_node in &domain_nodes {
                    push_project_dependency_edge_if_missing(
                        module,
                        &resource_families,
                        &node_resources,
                        source,
                        domain_node,
                    );
                }
            }
        }
        if !downlink_markers.is_empty() {
            for sink in uplink_windows.iter().chain(data_pipe_nodes.iter().take(2)) {
                for domain_node in &domain_nodes {
                    push_project_dependency_edge_if_missing(
                        module,
                        &resource_families,
                        &node_resources,
                        domain_node,
                        sink,
                    );
                }
            }
        }
    }
    let has_to_cpu_bridge = directional_markers
        .iter()
        .any(|(_, _, downlink_markers)| !downlink_markers.is_empty());
    if has_to_cpu_bridge && !cpu_nodes.is_empty() {
        for source in downlink_windows
            .iter()
            .chain(data_pipe_nodes.iter().skip(2).take(2))
        {
            for cpu_node in &cpu_nodes {
                push_project_dependency_edge_if_missing(
                    module,
                    &resource_families,
                    &node_resources,
                    source,
                    cpu_node,
                );
            }
        }
    }
}

fn collect_data_marker_nodes(module: &YirModule, tag: &str) -> Vec<String> {
    module
        .nodes
        .iter()
        .filter(|node| node.op.is_data_marker_tag(tag))
        .map(|node| node.name.clone())
        .collect()
}

fn collect_domain_nodes(module: &YirModule, family: OperationDomainFamily) -> Vec<String> {
    module
        .nodes
        .iter()
        .filter(|node| node.op.is_domain_family(family))
        .map(|node| node.name.clone())
        .collect()
}

fn operation_domain_family_for_name(domain: &str) -> Option<OperationDomainFamily> {
    match domain {
        "shader" => Some(OperationDomainFamily::Shader),
        "kernel" => Some(OperationDomainFamily::Kernel),
        "network" => Some(OperationDomainFamily::Network),
        _ => None,
    }
}

pub(super) fn push_project_dependency_edge_if_missing(
    module: &mut YirModule,
    resource_families: &BTreeMap<String, String>,
    node_resources: &BTreeMap<String, String>,
    from: &str,
    to: &str,
) {
    let kind = inferred_project_dependency_edge_kind(resource_families, node_resources, from, to);
    push_edge_if_missing(module, kind, from, to);
}

fn inferred_project_dependency_edge_kind(
    resource_families: &BTreeMap<String, String>,
    node_resources: &BTreeMap<String, String>,
    from_node: &str,
    to_node: &str,
) -> EdgeKind {
    let from_family = node_resources
        .get(from_node)
        .and_then(|resource| resource_families.get(resource))
        .map(String::as_str);
    let to_family = node_resources
        .get(to_node)
        .and_then(|resource| resource_families.get(resource))
        .map(String::as_str);
    if from_family.is_some() && from_family == to_family {
        EdgeKind::Dep
    } else {
        EdgeKind::CrossDomainExchange
    }
}

fn push_edge_if_missing(module: &mut YirModule, kind: EdgeKind, from: &str, to: &str) {
    if module
        .edges
        .iter()
        .any(|edge| edge.kind == kind && edge.from == from && edge.to == to)
    {
        return;
    }
    module.edges.push(yir_core::Edge {
        kind,
        from: from.to_owned(),
        to: to.to_owned(),
    });
}
