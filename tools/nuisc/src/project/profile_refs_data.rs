use std::collections::{BTreeMap, BTreeSet};

use crate::data_markers::{directional_bridge_marker_tag, DATA_BRIDGE_HETERO_DOMAINS};
use yir_core::{EdgeKind, OperationDomainFamily, SemanticOp, YirModule};

use super::data_bridge_directions::{data_bridge_directions, DataBridgeDirection};
use super::profile_refs::{push_edge_if_missing, push_project_dependency_edge_if_missing};
use super::profile_targets::resolve_project_profile_target_name;

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
    let cpu_nodes = module
        .nodes
        .iter()
        .filter(|node| node.op.is_domain_family(OperationDomainFamily::Cpu))
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    for handle in handle_tables {
        let Some(unit) = data_unit_from_handle_table(&handle) else {
            continue;
        };
        stitch_data_unit_profile_edges(
            module,
            &resource_families,
            &node_resources,
            &cpu_nodes,
            &unit,
            &handle,
        );
    }
}

fn stitch_data_unit_profile_edges(
    module: &mut YirModule,
    resource_families: &BTreeMap<String, String>,
    node_resources: &BTreeMap<String, String>,
    cpu_nodes: &[String],
    unit: &str,
    handle: &str,
) {
    let unit_fragment = format!("_data_{unit}_");
    let directional_markers = DATA_BRIDGE_HETERO_DOMAINS
        .iter()
        .filter_map(|domain| {
            let uplink_tag = directional_bridge_marker_tag("cpu", domain)?;
            let downlink_tag = directional_bridge_marker_tag(domain, "cpu")?;
            Some((
                (*domain).to_owned(),
                collect_unit_data_marker_nodes(module, &uplink_tag, &unit_fragment),
                collect_unit_data_marker_nodes(module, &downlink_tag, &unit_fragment),
            ))
        })
        .collect::<Vec<_>>();
    let uplink_windows = collect_unit_profile_windows(module, &unit_fragment, "_uplink_window");
    let downlink_windows = collect_unit_profile_windows(module, &unit_fragment, "_downlink_window");
    let plane_directions = [
        DataPlaneDirectionContext {
            direction: data_bridge_directions()[0],
            pipe_targets: data_pipe_nodes_for_unit(module, unit, true),
            windows: uplink_windows,
            len: find_unit_profile_node(module, &unit_fragment, "_uplink_len"),
        },
        DataPlaneDirectionContext {
            direction: data_bridge_directions()[1],
            pipe_targets: data_pipe_nodes_for_unit(module, unit, false),
            windows: downlink_windows,
            len: find_unit_profile_node(module, &unit_fragment, "_downlink_len"),
        },
    ];
    let window_offset = find_unit_profile_node(module, &unit_fragment, "_window_offset");

    for context in &plane_directions {
        for pipe in &context.pipe_targets {
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
        let marker = |tag: &str| collect_unit_data_marker_nodes(module, tag, &unit_fragment);
        let pipe_markers = marker(context.direction.pipe_marker);
        let pipe_class_markers = marker(context.direction.pipe_class_marker);
        let payload_class_markers = marker(
            context
                .direction
                .payload_class_marker
                .trim_start_matches("marker:"),
        );
        let payload_shape_markers = marker(
            context
                .direction
                .payload_shape_marker
                .trim_start_matches("marker:"),
        );
        let window_policy_markers = marker(
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
            resource_families,
            node_resources,
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
        resource_families,
        node_resources,
        &plane_directions,
        window_offset.as_ref(),
    );
    stitch_domain_bridge_edges(
        module,
        resource_families,
        node_resources,
        &directional_markers,
        &plane_directions,
        cpu_nodes,
    );
}

#[derive(Clone)]
struct DataPlaneDirectionContext {
    direction: DataBridgeDirection,
    pipe_targets: Vec<String>,
    windows: Vec<String>,
    len: Option<String>,
}

pub(super) fn data_pipe_nodes_for_unit(
    module: &YirModule,
    unit: &str,
    is_uplink: bool,
) -> Vec<String> {
    let handle = resolve_project_profile_target_name("data", unit, "handle_table");
    let expected_window = if is_uplink {
        SemanticOp::DataImmutableWindow
    } else {
        SemanticOp::DataCopyWindow
    };
    module
        .nodes
        .iter()
        .filter(|node| node.op.is_data_pipe_semantic_op())
        .filter(|node| {
            module
                .edges
                .iter()
                .any(|edge| edge.from == handle && edge.to == node.name)
        })
        .filter(|node| node_has_semantic_ancestor(module, &node.name, expected_window))
        .map(|node| node.name.clone())
        .collect()
}

fn node_has_semantic_ancestor(module: &YirModule, node_name: &str, expected: SemanticOp) -> bool {
    fn visit(
        module: &YirModule,
        node_name: &str,
        expected: SemanticOp,
        visited: &mut BTreeSet<String>,
    ) -> bool {
        if !visited.insert(node_name.to_owned()) {
            return false;
        }
        let Some(node) = module.nodes.iter().find(|node| node.name == node_name) else {
            return false;
        };
        if node.op.semantic_op() == expected {
            return true;
        }
        node.op
            .args
            .iter()
            .any(|arg| visit(module, arg, expected, visited))
    }
    visit(module, node_name, expected, &mut BTreeSet::new())
}

fn data_unit_from_handle_table(name: &str) -> Option<String> {
    name.strip_prefix("project_profile_data_")
        .and_then(|name| name.strip_suffix("_profile_handles"))
        .map(str::to_owned)
}

fn collect_unit_profile_windows(
    module: &YirModule,
    unit_fragment: &str,
    direction_fragment: &str,
) -> Vec<String> {
    module
        .nodes
        .iter()
        .filter(|node| {
            node.op.is_data_window_semantic_op()
                && node.name.contains(unit_fragment)
                && node.name.contains(direction_fragment)
        })
        .map(|node| node.name.clone())
        .collect()
}

fn find_unit_profile_node(
    module: &YirModule,
    unit_fragment: &str,
    slot_fragment: &str,
) -> Option<String> {
    module
        .nodes
        .iter()
        .find(|node| node.name.contains(unit_fragment) && node.name.ends_with(slot_fragment))
        .map(|node| node.name.clone())
}

fn collect_unit_data_marker_nodes(
    module: &YirModule,
    tag: &str,
    unit_fragment: &str,
) -> Vec<String> {
    collect_data_marker_nodes(module, tag)
        .into_iter()
        .filter(|name| name.contains(unit_fragment))
        .collect()
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
