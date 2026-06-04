use super::*;

pub(super) fn push_await_node(state: &mut LoweringState<'_>, awaited: &str) -> String {
    let await_name = format!("await_{}", state.await_counter);
    state.await_counter += 1;
    state.yir.nodes.push(Node {
        name: await_name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "await".to_owned(),
            args: vec![awaited.to_owned()],
        },
    });
    push_dep_edges(state, awaited, &await_name);
    await_name
}

pub(super) fn lower_cpu_unary_value_effect(
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
    input: &NirExpr,
    prefix: &str,
    instruction: &str,
) -> Result<String, String> {
    let input_name = lower_expr(input, state, bindings)?;
    let name = next_name(state, prefix);
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: instruction.to_owned(),
            args: vec![input_name.clone()],
        },
    });
    push_dep_edges(state, &input_name, &name);
    state.yir.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: input_name,
        to: name.clone(),
    });
    Ok(name)
}

pub(super) fn lower_result_observe_node(
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
    domain: ResultLoweringDomain,
    input: &NirExpr,
    prefix: &str,
    observed_state: &str,
) -> Result<String, String> {
    domain.ensure_resource(state.yir);
    let input_name = lower_expr(input, state, bindings)?;
    let name = next_name(state, prefix);
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: domain.resource_name().to_owned(),
        op: Operation {
            module: domain.module_name().to_owned(),
            instruction: "observe".to_owned(),
            args: vec![input_name.clone(), observed_state.to_owned()],
        },
    });
    push_dep_edges(state, &input_name, &name);
    Ok(name)
}

pub(super) fn lower_task_result_entry_node(
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
    input: &NirExpr,
) -> Result<String, String> {
    let task_name = lower_expr(input, state, bindings)?;
    let name = next_name(state, "cpu_join_result");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "join_result".to_owned(),
            args: vec![task_name.clone()],
        },
    });
    push_dep_edges(state, &task_name, &name);
    state.yir.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: task_name,
        to: name.clone(),
    });
    Ok(name)
}

pub(super) fn lower_task_result_observer_node(
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
    input: &NirExpr,
    role: YirResultRole,
    observed_state: Option<YirResultState>,
) -> Result<String, String> {
    let (prefix, instruction) = match (role, observed_state) {
        (YirResultRole::StateProbe, Some(YirResultState::Task(TaskLifecycleState::Completed))) => {
            ("cpu_task_completed", "task_completed")
        }
        (YirResultRole::StateProbe, Some(YirResultState::Task(TaskLifecycleState::TimedOut))) => {
            ("cpu_task_timed_out", "task_timed_out")
        }
        (YirResultRole::StateProbe, Some(YirResultState::Task(TaskLifecycleState::Cancelled))) => {
            ("cpu_task_cancelled", "task_cancelled")
        }
        (YirResultRole::PayloadExtractor, None) => ("cpu_task_value", "task_value"),
        (YirResultRole::Entry, _) => {
            return Err(
                "task result entry must lower through lower_task_result_entry_node".to_owned(),
            )
        }
        (YirResultRole::StateProbe, Some(other)) => {
            return Err(format!(
                "unsupported non-task result probe state `{other:?}` for task observer"
            ))
        }
        (YirResultRole::StateProbe, None) => {
            return Err("task state probe requires an explicit task lifecycle state".to_owned())
        }
        (YirResultRole::PayloadExtractor, Some(_)) => {
            return Err("task payload extractor must not carry an explicit result state".to_owned())
        }
    };
    lower_cpu_unary_value_effect(state, bindings, input, prefix, instruction)
}

pub(super) fn lower_result_unary_value_effect(
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
    domain: ResultLoweringDomain,
    input: &NirExpr,
    prefix: &str,
    instruction: &str,
) -> Result<String, String> {
    domain.ensure_resource(state.yir);
    let input_name = lower_expr(input, state, bindings)?;
    let name = next_name(state, prefix);
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: domain.resource_name().to_owned(),
        op: Operation {
            module: domain.module_name().to_owned(),
            instruction: instruction.to_owned(),
            args: vec![input_name.clone()],
        },
    });
    push_dep_edges(state, &input_name, &name);
    Ok(name)
}
