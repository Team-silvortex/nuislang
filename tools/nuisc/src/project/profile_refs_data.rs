use std::collections::BTreeMap;

use crate::data_markers::{directional_bridge_marker_tag, DATA_BRIDGE_HETERO_DOMAINS};
use yir_core::{EdgeKind, OperationDomainFamily, SemanticOp, YirModule};

use super::data_bridge_directions::{data_bridge_directions, DataBridgeDirection};
use super::profile_refs::{push_edge_if_missing, push_project_dependency_edge_if_missing};

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
    let plane_directions = [
        DataPlaneDirectionContext {
            direction: data_bridge_directions()[0],
            pipe_targets: data_pipe_nodes.iter().take(2).cloned().collect::<Vec<_>>(),
            windows: uplink_windows.clone(),
            len: uplink_len.clone(),
        },
        DataPlaneDirectionContext {
            direction: data_bridge_directions()[1],
            pipe_targets: data_pipe_nodes
                .iter()
                .skip(2)
                .take(2)
                .cloned()
                .collect::<Vec<_>>(),
            windows: downlink_windows.clone(),
            len: downlink_len.clone(),
        },
    ];

    for handle in &handle_tables {
        for pipe in &data_pipe_nodes {
            push_edge_if_missing(module, EdgeKind::Dep, handle, pipe);
        }
    }
    for (_domain, uplink_markers, downlink_markers) in &directional_markers {
        push_effect_edges_from_first_marker(
            module,
            uplink_markers,
            &plane_directions[0].pipe_targets,
        );
        push_effect_edges_from_first_marker(
            module,
            downlink_markers,
            &plane_directions[1].pipe_targets,
        );
    }
    for context in &plane_directions {
        let pipe_markers = collect_data_marker_nodes(module, context.direction.pipe_marker);
        let pipe_class_markers =
            collect_data_marker_nodes(module, context.direction.pipe_class_marker);
        let payload_class_markers = collect_data_marker_nodes(
            module,
            context
                .direction
                .payload_class_marker
                .trim_start_matches("marker:"),
        );
        let payload_shape_markers = collect_data_marker_nodes(
            module,
            context
                .direction
                .payload_shape_marker
                .trim_start_matches("marker:"),
        );
        let window_policy_markers = collect_data_marker_nodes(
            module,
            context
                .direction
                .window_policy_marker
                .trim_start_matches("marker:"),
        );
        push_effect_edges_from_first_marker(module, &pipe_markers, &context.pipe_targets);
        push_effect_edges_from_first_marker(module, &pipe_class_markers, &context.pipe_targets);
        push_effect_edges_from_first_marker(module, &payload_class_markers, &context.pipe_targets);
        if let Some(marker) = payload_shape_markers.first() {
            push_effect_edges(module, marker, &context.pipe_targets);
            push_effect_edges(module, marker, &context.windows);
        }
        stitch_window_binding_edges(
            module,
            &resource_families,
            &node_resources,
            WindowBindingEdges {
                windows: &context.windows,
                pipes: &context.pipe_targets,
                policy_marker: window_policy_markers.first(),
                window_offset: window_offset.as_deref(),
                window_len: context.len.as_deref(),
            },
        );
    }
    stitch_missing_window_fallback_edges(
        module,
        &resource_families,
        &node_resources,
        &plane_directions,
        window_offset.as_ref(),
    );
    stitch_domain_bridge_edges(
        module,
        &resource_families,
        &node_resources,
        &directional_markers,
        &plane_directions,
        &cpu_nodes,
    );
}

#[derive(Clone)]
struct DataPlaneDirectionContext {
    direction: DataBridgeDirection,
    pipe_targets: Vec<String>,
    windows: Vec<String>,
    len: Option<String>,
}

fn stitch_missing_window_fallback_edges(
    module: &mut YirModule,
    resource_families: &BTreeMap<String, String>,
    node_resources: &BTreeMap<String, String>,
    plane_directions: &[DataPlaneDirectionContext; 2],
    window_offset: Option<&String>,
) {
    for context in plane_directions {
        if !context.windows.is_empty() {
            continue;
        }
        if let Some(offset) = window_offset {
            push_project_dependency_edges_from_each(
                module,
                resource_families,
                node_resources,
                std::slice::from_ref(offset),
                &context.pipe_targets,
            );
        }
        if let Some(len) = &context.len {
            push_project_dependency_edges_from_each(
                module,
                resource_families,
                node_resources,
                std::slice::from_ref(len),
                &context.pipe_targets,
            );
        }
    }
}

fn stitch_domain_bridge_edges(
    module: &mut YirModule,
    resource_families: &BTreeMap<String, String>,
    node_resources: &BTreeMap<String, String>,
    directional_markers: &[(String, Vec<String>, Vec<String>)],
    plane_directions: &[DataPlaneDirectionContext; 2],
    cpu_nodes: &[String],
) {
    for (domain, uplink_markers, downlink_markers) in directional_markers {
        let Some(family) = operation_domain_family_for_name(domain) else {
            continue;
        };
        let domain_nodes = collect_domain_nodes(module, family);
        if domain_nodes.is_empty() {
            continue;
        }
        if !uplink_markers.is_empty() {
            for source in plane_directions[1]
                .windows
                .iter()
                .chain(plane_directions[1].pipe_targets.iter())
            {
                for domain_node in &domain_nodes {
                    push_project_dependency_edge_if_missing(
                        module,
                        resource_families,
                        node_resources,
                        source,
                        domain_node,
                    );
                }
            }
        }
        if !downlink_markers.is_empty() {
            for sink in plane_directions[0]
                .windows
                .iter()
                .chain(plane_directions[0].pipe_targets.iter())
            {
                for domain_node in &domain_nodes {
                    push_project_dependency_edge_if_missing(
                        module,
                        resource_families,
                        node_resources,
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
        for source in plane_directions[1]
            .windows
            .iter()
            .chain(plane_directions[1].pipe_targets.iter())
        {
            for cpu_node in cpu_nodes {
                push_project_dependency_edge_if_missing(
                    module,
                    resource_families,
                    node_resources,
                    source,
                    cpu_node,
                );
            }
        }
    }
}

fn push_effect_edges_from_first_marker(
    module: &mut YirModule,
    markers: &[String],
    targets: &[String],
) {
    if let Some(marker) = markers.first() {
        push_effect_edges(module, marker, targets);
    }
}

fn push_effect_edges(module: &mut YirModule, marker: &str, targets: &[String]) {
    for target in targets {
        push_edge_if_missing(module, EdgeKind::Effect, marker, target);
    }
}

struct WindowBindingEdges<'a> {
    windows: &'a [String],
    pipes: &'a [String],
    policy_marker: Option<&'a String>,
    window_offset: Option<&'a str>,
    window_len: Option<&'a str>,
}

fn stitch_window_binding_edges(
    module: &mut YirModule,
    resource_families: &BTreeMap<String, String>,
    node_resources: &BTreeMap<String, String>,
    edges: WindowBindingEdges<'_>,
) {
    if let Some(marker) = edges.policy_marker {
        push_effect_edges(module, marker, edges.windows);
    }
    push_project_dependency_edges_from_each(
        module,
        resource_families,
        node_resources,
        edges.windows,
        edges.pipes,
    );
    if let Some(offset) = edges.window_offset {
        push_project_dependency_edges_to_each(
            module,
            resource_families,
            node_resources,
            offset,
            edges.windows,
        );
    }
    if let Some(len) = edges.window_len {
        push_project_dependency_edges_to_each(
            module,
            resource_families,
            node_resources,
            len,
            edges.windows,
        );
    }
}

fn push_project_dependency_edges_from_each(
    module: &mut YirModule,
    resource_families: &BTreeMap<String, String>,
    node_resources: &BTreeMap<String, String>,
    from_nodes: &[String],
    to_nodes: &[String],
) {
    for from in from_nodes {
        for to in to_nodes {
            push_project_dependency_edge_if_missing(
                module,
                resource_families,
                node_resources,
                from,
                to,
            );
        }
    }
}

fn push_project_dependency_edges_to_each(
    module: &mut YirModule,
    resource_families: &BTreeMap<String, String>,
    node_resources: &BTreeMap<String, String>,
    from_node: &str,
    to_nodes: &[String],
) {
    for to in to_nodes {
        push_project_dependency_edge_if_missing(
            module,
            resource_families,
            node_resources,
            from_node,
            to,
        );
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
