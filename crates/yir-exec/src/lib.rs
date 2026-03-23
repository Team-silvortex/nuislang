use std::collections::BTreeMap;

use yir_core::{ExecutionState, Resource, Value, YirModule};
use yir_verify::{default_registry, verify_module_with_registry};

#[derive(Debug, Default)]
pub struct ExecutionTrace {
    pub events: Vec<String>,
    pub lane_events: BTreeMap<String, Vec<String>>,
    pub lane_steps: BTreeMap<String, Vec<String>>,
    pub values: BTreeMap<String, Value>,
}

pub fn execute_module(module: &YirModule) -> Result<ExecutionTrace, String> {
    let registry = default_registry();
    verify_module_with_registry(module, &registry)?;

    let resources = module
        .resources
        .iter()
        .map(|resource| (resource.name.clone(), resource))
        .collect::<BTreeMap<String, &Resource>>();
    let order = topological_order(module)?;

    let mut state = ExecutionState::default();
    let mut lane_steps = BTreeMap::<String, Vec<String>>::new();

    for node_name in order {
        let node = module
            .nodes
            .iter()
            .find(|node| node.name == node_name)
            .ok_or_else(|| format!("execution order references unknown node `{node_name}`"))?;
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

        lane_steps
            .entry(resource.kind.family().to_owned())
            .or_default()
            .push(format!(
                "{} @{} -> {}",
                node.op.full_name(),
                node.resource,
                node.name
            ));
        let value = module_impl.execute(node, resource, &mut state)?;
        state.values.insert(node.name.clone(), value);
    }

    Ok(ExecutionTrace {
        events: state.events,
        lane_events: state.lane_events,
        lane_steps,
        values: state.values,
    })
}

fn topological_order(module: &YirModule) -> Result<Vec<String>, String> {
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
        return Err("graph contains a cycle across YIR edges".to_owned());
    }

    Ok(order)
}
