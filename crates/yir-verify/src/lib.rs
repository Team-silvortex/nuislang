use std::collections::{BTreeMap, BTreeSet};

use yir_core::{EdgeKind, FabricMod, ModRegistry, Node, Resource, ResourceKind, YirModule};

pub fn default_registry() -> ModRegistry {
    let mut registry = ModRegistry::new();
    registry.register(FabricMod);
    registry.register(yir_domain_cpu::CpuMod);
    registry.register(yir_domain_shader::ShaderMod);
    registry
}

pub fn verify_module(module: &YirModule) -> Result<(), String> {
    let registry = default_registry();
    verify_module_with_registry(module, &registry)
}

pub fn verify_module_with_registry(
    module: &YirModule,
    registry: &ModRegistry,
) -> Result<(), String> {
    if module.version.is_empty() {
        return Err("module version must not be empty".to_owned());
    }

    let mut resources = BTreeMap::<String, &Resource>::new();
    for resource in &module.resources {
        if resources.insert(resource.name.clone(), resource).is_some() {
            return Err(format!("duplicate resource `{}`", resource.name));
        }
    }

    let mut nodes = BTreeMap::<String, &Node>::new();
    for node in &module.nodes {
        if nodes.insert(node.name.clone(), node).is_some() {
            return Err(format!("duplicate node `{}`", node.name));
        }
    }

    let mut edge_index = BTreeSet::<(String, String, &'static str)>::new();
    for edge in &module.edges {
        if !nodes.contains_key(&edge.from) {
            return Err(format!(
                "edge `{}` {} `{}` references unknown source node",
                edge.kind.as_str(),
                edge.from,
                edge.to
            ));
        }

        if !nodes.contains_key(&edge.to) {
            return Err(format!(
                "edge `{}` {} `{}` references unknown target node",
                edge.kind.as_str(),
                edge.from,
                edge.to
            ));
        }

        if !edge_index.insert((edge.from.clone(), edge.to.clone(), edge.kind.as_str())) {
            return Err(format!(
                "duplicate edge `{}` {} `{}`",
                edge.kind.as_str(),
                edge.from,
                edge.to
            ));
        }
    }

    for node in &module.nodes {
        let resource = resources.get(&node.resource).copied().ok_or_else(|| {
            format!(
                "node `{}` references unknown resource `{}`",
                node.name, node.resource
            )
        })?;

        let module_impl = registry.lookup(&node.op.module).ok_or_else(|| {
            format!(
                "node `{}` references unregistered mod `{}`",
                node.name, node.op.module
            )
        })?;

        let semantics = module_impl.describe(node, resource)?;

        for dependency in semantics.dependencies {
            if dependency == node.name {
                return Err(format!("node `{}` may not depend on itself", node.name));
            }

            let source = nodes.get(&dependency).copied().ok_or_else(|| {
                format!("node `{}` depends on unknown node `{dependency}`", node.name)
            })?;

            let source_resource = resources.get(&source.resource).copied().ok_or_else(|| {
                format!(
                    "node `{}` depends on `{dependency}` with unknown resource `{}`",
                    node.name, source.resource
                )
            })?;

            let required_kind = required_dependency_edge_kind(&source_resource.kind, &resource.kind);
            let key = (dependency.clone(), node.name.clone(), required_kind.as_str());

            if !edge_index.contains(&key) {
                return Err(format!(
                    "node `{}` requires `{}` edge from `{}` to `{}`",
                    node.name,
                    required_kind.as_str(),
                    dependency,
                    node.name
                ));
            }
        }
    }

    ensure_acyclic(module)?;
    Ok(())
}

fn required_dependency_edge_kind(source: &ResourceKind, target: &ResourceKind) -> EdgeKind {
    if source.family() == target.family() {
        EdgeKind::Dep
    } else {
        EdgeKind::CrossDomainExchange
    }
}

fn ensure_acyclic(module: &YirModule) -> Result<(), String> {
    let mut adjacency = BTreeMap::<&str, Vec<&str>>::new();
    let mut indegree = BTreeMap::<&str, usize>::new();

    for node in &module.nodes {
        adjacency.entry(node.name.as_str()).or_default();
        indegree.entry(node.name.as_str()).or_insert(0);
    }

    for edge in &module.edges {
        adjacency
            .entry(edge.from.as_str())
            .or_default()
            .push(edge.to.as_str());
        *indegree.entry(edge.to.as_str()).or_insert(0) += 1;
    }

    let mut ready = indegree
        .iter()
        .filter_map(|(name, degree)| (*degree == 0).then_some(*name))
        .collect::<Vec<_>>();
    let mut visited = 0usize;

    while let Some(node) = ready.pop() {
        visited += 1;
        if let Some(targets) = adjacency.get(node) {
            for target in targets {
                if let Some(degree) = indegree.get_mut(target) {
                    *degree -= 1;
                    if *degree == 0 {
                        ready.push(target);
                    }
                }
            }
        }
    }

    if visited != module.nodes.len() {
        return Err("graph contains a cycle across YIR edges".to_owned());
    }

    Ok(())
}
