use std::collections::{BTreeMap, BTreeSet};

use yir_core::{ExecutionState, Node, Resource, Value, YirModule};
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
    let nodes_by_name = module
        .nodes
        .iter()
        .map(|node| (node.name.clone(), node))
        .collect::<BTreeMap<_, _>>();

    let mut state = ExecutionState::default();
    let mut lane_steps = BTreeMap::<String, Vec<String>>::new();
    let mut delayed = BTreeMap::<String, String>::new();

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
        let lane_name = module
            .node_lanes
            .get(&node.name)
            .map(|lane| format!("{}@{}", node.resource, lane))
            .unwrap_or_else(|| resource.kind.family().to_owned());
        state.current_lane = Some(lane_name.clone());

        lane_steps
            .entry(lane_name.clone())
            .or_default()
            .push(format!(
                "{} @{} -> {}",
                node.op.full_name(),
                node.resource,
                node.name
            ));
        if node.op.module == "cpu" && node.op.instruction == "select" {
            match execute_lazy_select(node, &mut state, &mut delayed, &nodes_by_name)? {
                LazySelectOutcome::Handled(value) => {
                    state.values.insert(node.name.clone(), value);
                    continue;
                }
                LazySelectOutcome::UseRegisteredExecutor => {}
            }
        }
        if let Some((input, reason)) = first_delayed_input(node, &delayed) {
            delayed.insert(
                node.name.clone(),
                format!("depends on delayed `{input}`: {reason}"),
            );
            continue;
        }
        match module_impl.execute(node, resource, &mut state) {
            Ok(value) => {
                state.values.insert(node.name.clone(), value);
            }
            Err(error) if is_delayable_variant_error(node, &error) => {
                delayed.insert(node.name.clone(), error);
            }
            Err(error) => return Err(error),
        }
    }

    if let Some((name, error)) = delayed.iter().next() {
        return Err(format!(
            "node `{name}` was never selected by a lazy branch: {error}"
        ));
    }

    Ok(ExecutionTrace {
        events: state.events,
        lane_events: state.lane_events,
        lane_steps,
        values: state.values,
    })
}

enum LazySelectOutcome {
    Handled(Value),
    UseRegisteredExecutor,
}

fn execute_lazy_select(
    node: &Node,
    state: &mut ExecutionState,
    delayed: &mut BTreeMap<String, String>,
    nodes_by_name: &BTreeMap<String, &Node>,
) -> Result<LazySelectOutcome, String> {
    if node.op.args.len() != 3 {
        return Ok(LazySelectOutcome::UseRegisteredExecutor);
    }
    let cond = match state.expect_value(&node.op.args[0])? {
        Value::Bool(value) => *value,
        Value::Int(value) => *value != 0,
        other => {
            return Err(format!(
                "node `{}` expects bool or i64 select condition, got {}",
                node.name, other
            ))
        }
    };
    let selected = if cond {
        node.op.args[1].as_str()
    } else {
        node.op.args[2].as_str()
    };
    let unselected = if cond {
        node.op.args[2].as_str()
    } else {
        node.op.args[1].as_str()
    };
    if !delayed.contains_key(selected) && !delayed.contains_key(unselected) {
        return Ok(LazySelectOutcome::UseRegisteredExecutor);
    }
    let Some(value) = state.values.get(selected).cloned() else {
        if let Some(error) = delayed.get(selected) {
            return Err(format!(
                "node `{}` selected delayed branch `{selected}`: {error}",
                node.name
            ));
        }
        return Err(format!("missing value for `{selected}`"));
    };
    clear_delayed_dependency_closure(unselected, delayed, nodes_by_name);
    Ok(LazySelectOutcome::Handled(value))
}

fn first_delayed_input<'a>(
    node: &'a Node,
    delayed: &'a BTreeMap<String, String>,
) -> Option<(&'a str, &'a str)> {
    node.op.args.iter().find_map(|arg| {
        let value_name = arg.split_once('=').map_or(arg.as_str(), |(_, value)| value);
        delayed
            .get(value_name)
            .map(|reason| (value_name, reason.as_str()))
    })
}

fn is_delayable_variant_error(node: &Node, error: &str) -> bool {
    node.op.module == "cpu"
        && node.op.instruction == "variant_field"
        && error.contains("expects variant `")
}

fn clear_delayed_dependency_closure(
    root: &str,
    delayed: &mut BTreeMap<String, String>,
    nodes_by_name: &BTreeMap<String, &Node>,
) {
    let mut stack = vec![root.to_owned()];
    let mut seen = BTreeSet::new();
    while let Some(name) = stack.pop() {
        if !seen.insert(name.clone()) {
            continue;
        }
        delayed.remove(&name);
        if let Some(node) = nodes_by_name.get(&name) {
            stack.extend(node.op.args.iter().map(|arg| {
                arg.split_once('=')
                    .map_or_else(|| arg.clone(), |(_, value)| value.to_owned())
            }));
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use yir_core::{Edge, EdgeKind, Operation, ResourceKind};

    fn cpu_resource() -> Resource {
        Resource {
            name: "cpu0".to_owned(),
            kind: ResourceKind::parse("cpu.main"),
        }
    }

    fn cpu_node(name: &str, instruction: &str, args: &[&str]) -> Node {
        Node {
            name: name.to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation::parse(
                &format!("cpu.{instruction}"),
                args.iter().map(|arg| (*arg).to_owned()).collect(),
            )
            .unwrap(),
        }
    }

    fn dep(from: &str, to: &str) -> Edge {
        Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: to.to_owned(),
        }
    }

    #[test]
    fn lazy_select_skips_unselected_variant_field_chain() {
        let mut module = YirModule::new("0.1");
        module.resources.push(cpu_resource());
        module.nodes.extend([
            cpu_node("cond", "const_bool", &["false"]),
            cpu_node("payload", "const_i64", &["41"]),
            cpu_node("err", "struct", &["Result.Err", "value=payload"]),
            cpu_node(
                "wrong_payload",
                "variant_field",
                &["err", "Result.Ok", "value"],
            ),
            cpu_node("one", "const_i64", &["1"]),
            cpu_node("bad_sum", "add", &["wrong_payload", "one"]),
            cpu_node("fallback", "const_i64", &["7"]),
            cpu_node("selected", "select", &["cond", "bad_sum", "fallback"]),
        ]);
        module.edges.extend([
            dep("payload", "err"),
            dep("err", "wrong_payload"),
            dep("wrong_payload", "bad_sum"),
            dep("one", "bad_sum"),
            dep("cond", "selected"),
            dep("bad_sum", "selected"),
            dep("fallback", "selected"),
        ]);

        let trace = execute_module(&module).expect("lazy select should skip bad branch");
        assert_eq!(trace.values.get("selected"), Some(&Value::Int(7)));
        assert!(!trace.values.contains_key("wrong_payload"));
        assert!(!trace.values.contains_key("bad_sum"));
    }

    #[test]
    fn unselected_variant_field_error_still_fails_without_lazy_select() {
        let mut module = YirModule::new("0.1");
        module.resources.push(cpu_resource());
        module.nodes.extend([
            cpu_node("payload", "const_i64", &["41"]),
            cpu_node("err", "struct", &["Result.Err", "value=payload"]),
            cpu_node(
                "wrong_payload",
                "variant_field",
                &["err", "Result.Ok", "value"],
            ),
        ]);
        module
            .edges
            .extend([dep("payload", "err"), dep("err", "wrong_payload")]);

        let error = execute_module(&module).expect_err("standalone wrong variant must fail");
        assert!(error.contains("wrong_payload"));
        assert!(error.contains("expects variant `Result.Ok`"));
    }

    #[test]
    fn non_lazy_select_between_variants_preserves_union() {
        let mut module = YirModule::new("0.1");
        module.resources.push(cpu_resource());
        module.nodes.extend([
            cpu_node("cond", "const_bool", &["true"]),
            cpu_node("ok_payload", "const_i64", &["7"]),
            cpu_node("err_payload", "const_i64", &["99"]),
            cpu_node("ok", "struct", &["Result.Ok", "value=ok_payload"]),
            cpu_node("err", "struct", &["Result.Err", "value=err_payload"]),
            cpu_node("selected", "select", &["cond", "ok", "err"]),
            cpu_node("selected_is_ok", "variant_is", &["selected", "Result.Ok"]),
            cpu_node(
                "selected_ok_value",
                "variant_field",
                &["selected", "Result.Ok", "value"],
            ),
            cpu_node(
                "selected_err_value",
                "variant_field",
                &["selected", "Result.Err", "value"],
            ),
        ]);
        module.edges.extend([
            dep("ok_payload", "ok"),
            dep("err_payload", "err"),
            dep("cond", "selected"),
            dep("ok", "selected"),
            dep("err", "selected"),
            dep("selected", "selected_is_ok"),
            dep("selected", "selected_ok_value"),
            dep("selected", "selected_err_value"),
        ]);

        let trace = execute_module(&module).expect("variant select should execute");
        assert_eq!(trace.values.get("selected_is_ok"), Some(&Value::Bool(true)));
        assert_eq!(trace.values.get("selected_ok_value"), Some(&Value::Int(7)));
        assert_eq!(
            trace.values.get("selected_err_value"),
            Some(&Value::Int(99))
        );
    }
}
