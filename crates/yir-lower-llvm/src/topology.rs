use std::collections::BTreeMap;

use yir_core::{EdgeKind, YirModule};

use super::extern_abi::is_cpu_extern_call_instruction;

pub(crate) fn topological_order(module: &YirModule) -> Result<Vec<String>, String> {
    let mut adjacency = BTreeMap::<String, Vec<String>>::new();
    let mut indegree = BTreeMap::<String, usize>::new();
    let node_positions = module
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (node.name.clone(), index))
        .collect::<BTreeMap<_, _>>();

    for node in &module.nodes {
        adjacency.entry(node.name.clone()).or_default();
        indegree.entry(node.name.clone()).or_insert(0);
    }

    for edge in &module.edges {
        match edge.kind {
            EdgeKind::Dep
            | EdgeKind::Effect
            | EdgeKind::Lifetime
            | EdgeKind::CrossDomainExchange => {
                adjacency
                    .entry(edge.from.clone())
                    .or_default()
                    .push(edge.to.clone());
                *indegree.entry(edge.to.clone()).or_insert(0) += 1;
            }
        }
    }

    let mut last_cpu_extern_on_resource = BTreeMap::<String, String>::new();
    for node in &module.nodes {
        if node.op.module == "cpu" && is_cpu_extern_call_instruction(&node.op.instruction) {
            if let Some(previous) =
                last_cpu_extern_on_resource.insert(node.resource.clone(), node.name.clone())
            {
                adjacency
                    .entry(previous)
                    .or_default()
                    .push(node.name.clone());
                *indegree.entry(node.name.clone()).or_insert(0) += 1;
            }
        }
    }

    let mut last_cpu_node_on_lane = BTreeMap::<(String, String), String>::new();
    for node in &module.nodes {
        if node.op.module != "cpu" {
            continue;
        }
        let lane = module
            .node_lanes
            .get(&node.name)
            .cloned()
            .unwrap_or_else(|| "main".to_owned());
        if matches!(lane.as_str(), "profile" | "contract") {
            continue;
        }
        let key = (node.resource.clone(), lane);
        if let Some(previous) = last_cpu_node_on_lane.insert(key, node.name.clone()) {
            adjacency
                .entry(previous)
                .or_default()
                .push(node.name.clone());
            *indegree.entry(node.name.clone()).or_insert(0) += 1;
        }
    }

    let mut ready = indegree
        .iter()
        .filter_map(|(name, degree)| (*degree == 0).then_some(name.clone()))
        .collect::<Vec<_>>();
    ready.sort_by_key(|name| std::cmp::Reverse(node_positions[name]));

    let mut order = Vec::with_capacity(module.nodes.len());
    while let Some(node) = ready.pop() {
        order.push(node.clone());
        if let Some(targets) = adjacency.get(&node) {
            for target in targets {
                if let Some(degree) = indegree.get_mut(target) {
                    *degree -= 1;
                    if *degree == 0 {
                        ready.push(target.clone());
                        ready.sort_by_key(|name| std::cmp::Reverse(node_positions[name]));
                    }
                }
            }
        }
    }

    if order.len() != module.nodes.len() {
        return Err("graph contains a cycle across YIR edges".to_owned());
    }

    Ok(order)
}
