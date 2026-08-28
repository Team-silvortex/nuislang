use std::{
    cmp::Reverse,
    collections::{BTreeMap, BinaryHeap},
};

use yir_core::{EdgeKind, YirModule};

use super::extern_abi::is_cpu_extern_call_instruction;

pub(crate) fn topological_order(module: &YirModule) -> Result<Vec<String>, String> {
    let mut adjacency = BTreeMap::<&str, Vec<&str>>::new();
    let mut indegree = BTreeMap::<&str, usize>::new();
    let node_positions = module
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (node.name.as_str(), index))
        .collect::<BTreeMap<_, _>>();

    for node in &module.nodes {
        adjacency.entry(node.name.as_str()).or_default();
        indegree.entry(node.name.as_str()).or_insert(0);
    }

    for edge in &module.edges {
        match edge.kind {
            EdgeKind::Dep
            | EdgeKind::Effect
            | EdgeKind::Lifetime
            | EdgeKind::CrossDomainExchange => {
                adjacency
                    .entry(edge.from.as_str())
                    .or_default()
                    .push(edge.to.as_str());
                *indegree.entry(edge.to.as_str()).or_insert(0) += 1;
            }
        }
    }

    let mut last_cpu_extern_on_resource = BTreeMap::<&str, &str>::new();
    for node in &module.nodes {
        if node.op.module == "cpu" && is_cpu_extern_call_instruction(&node.op.instruction) {
            if let Some(previous) =
                last_cpu_extern_on_resource.insert(node.resource.as_str(), node.name.as_str())
            {
                adjacency
                    .entry(previous)
                    .or_default()
                    .push(node.name.as_str());
                *indegree.entry(node.name.as_str()).or_insert(0) += 1;
            }
        }
    }

    let mut last_cpu_node_on_lane = BTreeMap::<(&str, &str), &str>::new();
    for node in &module.nodes {
        if node.op.module != "cpu" {
            continue;
        }
        let lane = module
            .node_lanes
            .get(&node.name)
            .map(String::as_str)
            .unwrap_or("main");
        if matches!(lane, "profile" | "contract") {
            continue;
        }
        let key = (node.resource.as_str(), lane);
        if let Some(previous) = last_cpu_node_on_lane.insert(key, node.name.as_str()) {
            adjacency
                .entry(previous)
                .or_default()
                .push(node.name.as_str());
            *indegree.entry(node.name.as_str()).or_insert(0) += 1;
        }
    }

    let mut ready = indegree
        .iter()
        .filter_map(|(name, degree)| {
            (*degree == 0).then_some(Reverse((node_positions[*name], *name)))
        })
        .collect::<BinaryHeap<_>>();

    let mut order = Vec::with_capacity(module.nodes.len());
    while let Some(Reverse((_, node))) = ready.pop() {
        order.push(node.to_owned());
        if let Some(targets) = adjacency.get(node) {
            for target in targets {
                if let Some(degree) = indegree.get_mut(target) {
                    *degree -= 1;
                    if *degree == 0 {
                        ready.push(Reverse((node_positions[*target], *target)));
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
        return Err(format!(
            "graph contains a cycle across YIR edges; unresolved_nodes=[{unresolved}]"
        ));
    }

    Ok(order)
}

#[cfg(test)]
mod tests {
    use super::*;
    use yir_core::{Edge, Node, Operation};

    fn node(name: &str) -> Node {
        Node {
            name: name.to_owned(),
            resource: "data0".to_owned(),
            op: Operation {
                module: "data".to_owned(),
                instruction: "value".to_owned(),
                args: Vec::new(),
            },
        }
    }

    #[test]
    fn newly_ready_earlier_node_keeps_source_order_priority() {
        let mut module = YirModule::new("0.1");
        module.nodes = vec![node("unblocked"), node("source"), node("independent")];
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: "source".to_owned(),
            to: "unblocked".to_owned(),
        });

        assert_eq!(
            topological_order(&module).unwrap(),
            ["source", "unblocked", "independent"]
        );
    }
}
