use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
};

use yir_core::{EdgeKind, YirModule};

use super::extern_abi::is_cpu_extern_call_instruction;

pub(crate) fn topological_order(module: &YirModule) -> Result<Vec<String>, String> {
    let node_indices = module
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (node.name.as_str(), index))
        .collect::<HashMap<_, _>>();
    let mut adjacency = vec![Vec::<usize>::new(); module.nodes.len()];
    let mut indegree = vec![0usize; module.nodes.len()];

    for edge in &module.edges {
        match edge.kind {
            EdgeKind::Dep
            | EdgeKind::Effect
            | EdgeKind::Lifetime
            | EdgeKind::CrossDomainExchange => {
                let source_index = node_index(&node_indices, &edge.from, "source")?;
                let target_index = node_index(&node_indices, &edge.to, "target")?;
                adjacency[source_index].push(target_index);
                indegree[target_index] += 1;
            }
        }
    }

    // Implicit serial queues may extend the explicit graph, never reverse its paths.
    let explicit_order = sorted_indices(module, &adjacency, indegree.clone())?;
    let mut last_cpu_extern_on_resource = HashMap::<&str, usize>::new();
    for &node_index in &explicit_order {
        let node = &module.nodes[node_index];
        if node.op.module == "cpu" && is_cpu_extern_call_instruction(&node.op.instruction) {
            if let Some(previous) =
                last_cpu_extern_on_resource.insert(node.resource.as_str(), node_index)
            {
                adjacency[previous].push(node_index);
                indegree[node_index] += 1;
            }
        }
    }

    let mut last_cpu_node_on_lane = HashMap::<(&str, &str), usize>::new();
    for &node_index in &explicit_order {
        let node = &module.nodes[node_index];
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
        if let Some(previous) = last_cpu_node_on_lane.insert(key, node_index) {
            adjacency[previous].push(node_index);
            indegree[node_index] += 1;
        }
    }

    Ok(sorted_indices(module, &adjacency, indegree)?
        .into_iter()
        .map(|index| module.nodes[index].name.clone())
        .collect())
}

fn sorted_indices(
    module: &YirModule,
    adjacency: &[Vec<usize>],
    mut indegree: Vec<usize>,
) -> Result<Vec<usize>, String> {
    let mut ready = indegree
        .iter()
        .enumerate()
        .filter_map(|(index, degree)| (*degree == 0).then_some(Reverse(index)))
        .collect::<BinaryHeap<_>>();

    let mut order = Vec::with_capacity(module.nodes.len());
    while let Some(Reverse(node_index)) = ready.pop() {
        order.push(node_index);
        for &target_index in &adjacency[node_index] {
            indegree[target_index] -= 1;
            if indegree[target_index] == 0 {
                ready.push(Reverse(target_index));
            }
        }
    }

    if order.len() != module.nodes.len() {
        let mut unresolved = indegree
            .iter()
            .enumerate()
            .filter_map(|(index, degree)| {
                (*degree > 0).then_some((module.nodes[index].name.as_str(), *degree))
            })
            .collect::<Vec<_>>();
        unresolved.sort_unstable_by_key(|(name, _)| *name);
        let unresolved = unresolved
            .into_iter()
            .take(12)
            .map(|(name, degree)| format!("{name}:{degree}"))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!(
            "graph contains a cycle across YIR edges; unresolved_nodes=[{unresolved}]"
        ));
    }

    Ok(order)
}

fn node_index(
    node_indices: &HashMap<&str, usize>,
    name: &str,
    role: &str,
) -> Result<usize, String> {
    node_indices
        .get(name)
        .copied()
        .ok_or_else(|| format!("YIR edge references unknown {role} node `{name}`"))
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

    #[test]
    fn implicit_cpu_queues_respect_transitive_paths_in_every_declaration_order() {
        for names in [
            ["a", "b", "c"],
            ["a", "c", "b"],
            ["b", "a", "c"],
            ["b", "c", "a"],
            ["c", "a", "b"],
            ["c", "b", "a"],
        ] {
            for instruction in ["const_i64", "extern_call_i64"] {
                let mut module = YirModule::new("0.1");
                module.nodes = names
                    .iter()
                    .map(|name| {
                        let mut value = node(name);
                        value.op.module = "cpu".to_owned();
                        value.op.instruction = instruction.to_owned();
                        value
                    })
                    .collect();
                for name in names {
                    module.node_lanes.insert(
                        name.to_owned(),
                        if name == "b" { "other" } else { "work" }.to_owned(),
                    );
                }
                for (from, to, kind) in [("a", "b", EdgeKind::Dep), ("b", "c", EdgeKind::Lifetime)]
                {
                    module.edges.push(Edge {
                        kind,
                        from: from.to_owned(),
                        to: to.to_owned(),
                    });
                }
                assert_eq!(topological_order(&module).unwrap(), ["a", "b", "c"]);
                module.edges.push(Edge {
                    kind: EdgeKind::Effect,
                    from: "c".to_owned(),
                    to: "a".to_owned(),
                });
                assert!(topological_order(&module).is_err());
            }
        }
    }
}
