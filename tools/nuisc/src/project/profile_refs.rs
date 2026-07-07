use std::collections::{BTreeMap, BTreeSet};

use yir_core::{EdgeKind, SemanticOp, YirModule};

pub(super) use super::profile_refs_data::stitch_data_profile_edges;
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

pub(super) fn push_edge_if_missing(module: &mut YirModule, kind: EdgeKind, from: &str, to: &str) {
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
