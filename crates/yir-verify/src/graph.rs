use std::collections::{BinaryHeap, HashMap};

use yir_core::YirModule;

struct DenseGraph<'a> {
    names: Vec<&'a str>,
    name_to_index: HashMap<&'a str, usize>,
    adjacency: Vec<Vec<usize>>,
    indegree: Vec<usize>,
}

pub(crate) fn ensure_acyclic(module: &YirModule) -> Result<(), String> {
    let mut graph = dense_graph(module)?;
    let mut ready = graph
        .indegree
        .iter()
        .enumerate()
        .filter_map(|(index, degree)| (*degree == 0).then_some(index))
        .collect::<Vec<_>>();
    let mut visited = 0usize;

    while let Some(node_index) = ready.pop() {
        visited += 1;
        for &target_index in &graph.adjacency[node_index] {
            let degree = &mut graph.indegree[target_index];
            *degree -= 1;
            if *degree == 0 {
                ready.push(target_index);
            }
        }
    }

    if visited != module.nodes.len() {
        return Err(cycle_error(module, &graph));
    }

    Ok(())
}

pub(crate) fn topological_order(module: &YirModule) -> Result<Vec<String>, String> {
    let mut graph = dense_graph(module)?;
    let mut ready = graph
        .indegree
        .iter()
        .enumerate()
        .filter_map(|(index, degree)| (*degree == 0).then_some((graph.names[index], index)))
        .collect::<BinaryHeap<_>>();

    let mut order = Vec::with_capacity(module.nodes.len());

    while let Some((node, node_index)) = ready.pop() {
        order.push(node.to_owned());
        for &target_index in &graph.adjacency[node_index] {
            let degree = &mut graph.indegree[target_index];
            *degree -= 1;
            if *degree == 0 {
                ready.push((graph.names[target_index], target_index));
            }
        }
    }

    if order.len() != module.nodes.len() {
        return Err(cycle_error(module, &graph));
    }

    Ok(order)
}

fn dense_graph(module: &YirModule) -> Result<DenseGraph<'_>, String> {
    let mut names = Vec::with_capacity(module.nodes.len());
    let mut name_to_index = HashMap::with_capacity(module.nodes.len());
    for (index, node) in module.nodes.iter().enumerate() {
        names.push(node.name.as_str());
        if name_to_index.insert(node.name.as_str(), index).is_some() {
            return Err(format!("duplicate node `{}`", node.name));
        }
    }

    let mut adjacency = vec![Vec::new(); module.nodes.len()];
    let mut indegree = vec![0usize; module.nodes.len()];
    for edge in &module.edges {
        let source_index = name_to_index
            .get(edge.from.as_str())
            .copied()
            .ok_or_else(|| {
                format!(
                    "edge `{}` {} `{}` references unknown source node",
                    edge.kind.as_str(),
                    edge.from,
                    edge.to
                )
            })?;
        let target_index = name_to_index
            .get(edge.to.as_str())
            .copied()
            .ok_or_else(|| {
                format!(
                    "edge `{}` {} `{}` references unknown target node",
                    edge.kind.as_str(),
                    edge.from,
                    edge.to
                )
            })?;
        adjacency[source_index].push(target_index);
        indegree[target_index] += 1;
    }

    Ok(DenseGraph {
        names,
        name_to_index,
        adjacency,
        indegree,
    })
}

fn cycle_error(module: &YirModule, graph: &DenseGraph<'_>) -> String {
    let mut unresolved = graph
        .names
        .iter()
        .enumerate()
        .filter_map(|(index, name)| {
            (graph.indegree[index] > 0).then_some((*name, graph.indegree[index]))
        })
        .collect::<Vec<_>>();
    unresolved.sort_unstable_by_key(|(name, _)| *name);
    let unresolved = unresolved
        .into_iter()
        .take(12)
        .map(|(name, degree)| format!("{name}:{degree}"))
        .collect::<Vec<_>>()
        .join(", ");
    let incoming = module
        .edges
        .iter()
        .filter(|edge| {
            graph
                .name_to_index
                .get(edge.to.as_str())
                .is_some_and(|index| graph.indegree[*index] > 0)
        })
        .take(12)
        .map(|edge| format!("{}->{}/{:?}", edge.from, edge.to, edge.kind))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "graph contains a cycle across YIR edges; unresolved_nodes=[{unresolved}]; incoming_edges=[{incoming}]"
    )
}
