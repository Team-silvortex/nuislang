use std::collections::{BTreeMap, BTreeSet};

use yir_core::YirModule;

pub(crate) fn ensure_acyclic(module: &YirModule) -> Result<(), String> {
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
        let unresolved = indegree
            .iter()
            .filter_map(|(name, degree)| (*degree > 0).then_some(format!("{name}:{degree}")))
            .take(12)
            .collect::<Vec<_>>()
            .join(", ");
        let incoming = module
            .edges
            .iter()
            .filter(|edge| indegree.get(edge.to.as_str()).copied().unwrap_or(0) > 0)
            .take(12)
            .map(|edge| format!("{}->{}/{:?}", edge.from, edge.to, edge.kind))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!(
            "graph contains a cycle across YIR edges; unresolved_nodes=[{unresolved}]; incoming_edges=[{incoming}]"
        ));
    }

    Ok(())
}

pub(crate) fn topological_order(module: &YirModule) -> Result<Vec<String>, String> {
    let mut adjacency = BTreeMap::<String, Vec<String>>::new();
    let mut indegree = BTreeMap::<String, usize>::new();

    for node in &module.nodes {
        adjacency.entry(node.name.clone()).or_default();
        indegree.entry(node.name.clone()).or_insert(0);
    }

    for edge in &module.edges {
        adjacency
            .entry(edge.from.clone())
            .or_default()
            .push(edge.to.clone());
        *indegree.entry(edge.to.clone()).or_insert(0) += 1;
    }

    let mut ready = indegree
        .iter()
        .filter_map(|(name, degree)| (*degree == 0).then_some(name.clone()))
        .collect::<Vec<_>>();
    ready.sort();

    let mut order = Vec::with_capacity(module.nodes.len());

    while let Some(node) = ready.pop() {
        order.push(node.clone());
        if let Some(targets) = adjacency.get(&node) {
            for target in targets {
                if let Some(degree) = indegree.get_mut(target) {
                    *degree -= 1;
                    if *degree == 0 {
                        ready.push(target.clone());
                        ready.sort();
                    }
                }
            }
        }
    }

    if order.len() != module.nodes.len() {
        let unresolved = indegree
            .iter()
            .filter_map(|(name, degree)| (*degree > 0).then_some(format!("{name}:{degree}")))
            .take(12)
            .collect::<Vec<_>>()
            .join(", ");
        let incoming = module
            .edges
            .iter()
            .filter(|edge| indegree.get(&edge.to).copied().unwrap_or(0) > 0)
            .take(12)
            .map(|edge| format!("{}->{}/{:?}", edge.from, edge.to, edge.kind))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!(
            "graph contains a cycle across YIR edges; unresolved_nodes=[{unresolved}]; incoming_edges=[{incoming}]"
        ));
    }

    Ok(order)
}

pub(crate) fn path_exists(module: &YirModule, from: &str, to: &str) -> bool {
    if from == to {
        return true;
    }

    let mut adjacency = BTreeMap::<&str, Vec<&str>>::new();
    for edge in &module.edges {
        adjacency
            .entry(edge.from.as_str())
            .or_default()
            .push(edge.to.as_str());
    }

    let mut stack = vec![from];
    let mut visited = BTreeSet::<&str>::new();

    while let Some(current) = stack.pop() {
        if !visited.insert(current) {
            continue;
        }
        if let Some(targets) = adjacency.get(current) {
            for target in targets {
                if *target == to {
                    return true;
                }
                stack.push(target);
            }
        }
    }

    false
}
