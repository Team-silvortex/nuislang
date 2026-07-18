use super::*;

pub(super) fn lower_cpu_expr(
    expr: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Option<Result<String, String>> {
    match expr {
        NirExpr::Instantiate { domain, unit } => Some(lower_instantiate_expr(domain, unit, state)),
        NirExpr::CpuBindCore(core_index) => Some(Ok(lower_cpu_bind_core(*core_index, state))),
        NirExpr::CpuWindow {
            width,
            height,
            title,
        } => Some(Ok(lower_cpu_window(*width, *height, title, state))),
        NirExpr::CpuInputI64 {
            channel,
            default,
            min,
            max,
            step,
        } => Some(Ok(lower_cpu_input_i64(
            channel, *default, *min, *max, *step, state,
        ))),
        NirExpr::CpuTickI64 { start, step } => Some(Ok(lower_cpu_tick_i64(*start, *step, state))),
        NirExpr::CpuSpawn { callee, args } => Some(lower_cpu_spawn(callee, args, state, bindings)),
        NirExpr::CpuJoin(task) => Some(lower_cpu_join(task, state, bindings)),
        NirExpr::CpuCancel(task) => Some(lower_cpu_cancel(task, state, bindings)),
        NirExpr::CpuJoinResult(task) => Some(lower_task_result_entry_node(state, bindings, task)),
        NirExpr::CpuThreadSpawn { callee, args } => {
            Some(lower_cpu_thread_spawn(callee, args, state, bindings))
        }
        NirExpr::CpuThreadJoin(thread) => Some(lower_cpu_thread_join(thread, state, bindings)),
        NirExpr::CpuThreadJoinResult(thread) => {
            Some(lower_cpu_thread_join_result(thread, state, bindings))
        }
        NirExpr::CpuMutexNew(value) => Some(lower_cpu_mutex_new(value, state, bindings)),
        NirExpr::CpuMutexLock(mutex) => Some(lower_cpu_mutex_lock(mutex, state, bindings)),
        NirExpr::CpuMutexUnlock(guard) => Some(lower_cpu_mutex_unlock(guard, state, bindings)),
        NirExpr::CpuMutexValue(guard) => Some(lower_cpu_mutex_value(guard, state, bindings)),
        NirExpr::CpuTaskCompleted(result) => Some(lower_task_result_observer_node(
            state,
            bindings,
            result,
            YirResultRole::StateProbe,
            Some(YirResultState::Task(TaskLifecycleState::Completed)),
        )),
        NirExpr::CpuTaskTimedOut(result) => Some(lower_task_result_observer_node(
            state,
            bindings,
            result,
            YirResultRole::StateProbe,
            Some(YirResultState::Task(TaskLifecycleState::TimedOut)),
        )),
        NirExpr::CpuTaskCancelled(result) => Some(lower_task_result_observer_node(
            state,
            bindings,
            result,
            YirResultRole::StateProbe,
            Some(YirResultState::Task(TaskLifecycleState::Cancelled)),
        )),
        NirExpr::CpuTaskFailed(result) => Some(lower_task_result_observer_node(
            state,
            bindings,
            result,
            YirResultRole::StateProbe,
            Some(YirResultState::Task(TaskLifecycleState::Failed)),
        )),
        NirExpr::CpuTaskValue(result) => Some(lower_task_result_observer_node(
            state,
            bindings,
            result,
            YirResultRole::PayloadExtractor,
            None,
        )),
        NirExpr::CpuTimeout { task, limit } => {
            Some(lower_cpu_timeout(task, limit, state, bindings))
        }
        NirExpr::CpuReadyAfter { task, delay } => {
            Some(lower_cpu_ready_after(task, delay, state, bindings))
        }
        NirExpr::CpuPresentFrame(frame) => Some(lower_cpu_present_frame(frame, state, bindings)),
        NirExpr::CpuExternCall {
            abi,
            interface: _,
            callee,
            args,
        } => Some(lower_cpu_extern_call(abi, callee, args, state, bindings)),
        NirExpr::CpuExternCallI32 {
            abi,
            interface: _,
            callee,
            args,
        } => Some(lower_cpu_extern_call_i32(
            abi, callee, args, state, bindings,
        )),
        NirExpr::HostBufferHandle(value) => Some(lower_expr(value, state, bindings)),
        _ => None,
    }
}

fn lower_instantiate_expr(
    domain: &str,
    unit: &str,
    state: &mut LoweringState<'_>,
) -> Result<String, String> {
    let name = next_name(state, "instantiate_unit");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "instantiate_unit".to_owned(),
            args: vec![domain.to_owned(), unit.to_owned()],
        },
    });
    Ok(name)
}

fn lower_cpu_bind_core(core_index: i64, state: &mut LoweringState<'_>) -> String {
    let name = next_name(state, "cpu_bind_core");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "bind_core".to_owned(),
            args: vec![core_index.to_string()],
        },
    });
    name
}

fn lower_cpu_window(width: i64, height: i64, title: &str, state: &mut LoweringState<'_>) -> String {
    let name = next_name(state, "cpu_window");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "window".to_owned(),
            args: vec![width.to_string(), height.to_string(), title.to_owned()],
        },
    });
    name
}

fn lower_cpu_input_i64(
    channel: &str,
    default: i64,
    min: Option<i64>,
    max: Option<i64>,
    step: Option<i64>,
    state: &mut LoweringState<'_>,
) -> String {
    let name = next_name(state, "cpu_input_i64");
    let mut args = vec![channel.to_owned(), default.to_string()];
    if let (Some(min), Some(max), Some(step)) = (min, max, step) {
        args.push(min.to_string());
        args.push(max.to_string());
        args.push(step.to_string());
    }
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "input_i64".to_owned(),
            args,
        },
    });
    name
}

fn lower_cpu_tick_i64(start: i64, step: i64, state: &mut LoweringState<'_>) -> String {
    let name = next_name(state, "cpu_tick_i64");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "tick_i64".to_owned(),
            args: vec![start.to_string(), step.to_string()],
        },
    });
    name
}

fn lower_cpu_spawn(
    callee: &str,
    args: &[NirExpr],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let returned = lower_async_call_boundary(callee, args, state, bindings)?;
    let name = next_name(state, "cpu_spawn_task");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "spawn_task".to_owned(),
            args: vec![callee.to_owned(), returned.clone()],
        },
    });
    push_dep_edges(state, &returned, &name);
    state.yir.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: returned,
        to: name.clone(),
    });
    Ok(name)
}

fn lower_cpu_join(
    task: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let task_name = lower_expr(task, state, bindings)?;
    let name = next_name(state, "cpu_join");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "join".to_owned(),
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

fn lower_cpu_thread_spawn(
    callee: &str,
    args: &[NirExpr],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let returned = lower_async_call_boundary(callee, args, state, bindings)?;
    let name = next_name(state, "cpu_spawn_thread");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "spawn_thread".to_owned(),
            args: vec![callee.to_owned(), returned.clone()],
        },
    });
    push_dep_edges(state, &returned, &name);
    state.yir.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: returned,
        to: name.clone(),
    });
    Ok(name)
}

fn lower_cpu_thread_join(
    thread: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    super::result_nodes::lower_cpu_unary_value_effect(
        state,
        bindings,
        thread,
        "cpu_thread_join",
        "thread_join",
    )
}

fn lower_cpu_thread_join_result(
    thread: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    super::result_nodes::lower_cpu_unary_value_effect(
        state,
        bindings,
        thread,
        "cpu_thread_join_result",
        "thread_join_result",
    )
}

fn lower_cpu_cancel(
    task: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let task_name = lower_expr(task, state, bindings)?;
    let name = next_name(state, "cpu_cancel");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "cancel".to_owned(),
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

fn lower_cpu_mutex_new(
    value: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    super::result_nodes::lower_cpu_unary_value_effect(
        state,
        bindings,
        value,
        "cpu_mutex_new",
        "mutex_new",
    )
}

fn lower_cpu_mutex_lock(
    mutex: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    super::result_nodes::lower_cpu_unary_value_effect(
        state,
        bindings,
        mutex,
        "cpu_mutex_lock",
        "mutex_lock",
    )
}

fn lower_cpu_mutex_unlock(
    guard: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    super::result_nodes::lower_cpu_unary_value_effect(
        state,
        bindings,
        guard,
        "cpu_mutex_unlock",
        "mutex_unlock",
    )
}

fn lower_cpu_mutex_value(
    guard: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    super::result_nodes::lower_cpu_unary_value_effect(
        state,
        bindings,
        guard,
        "cpu_mutex_value",
        "mutex_value",
    )
}

fn lower_cpu_timeout(
    task: &NirExpr,
    limit: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let task_name = lower_expr(task, state, bindings)?;
    let limit_name = lower_expr(limit, state, bindings)?;
    let name = next_name(state, "cpu_timeout");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "timeout".to_owned(),
            args: vec![task_name.clone(), limit_name.clone()],
        },
    });
    push_dep_edges(state, &task_name, &name);
    push_dep_edges(state, &limit_name, &name);
    state.yir.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: task_name,
        to: name.clone(),
    });
    state.yir.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: limit_name,
        to: name.clone(),
    });
    Ok(name)
}

fn lower_cpu_ready_after(
    task: &NirExpr,
    delay: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let task_name = lower_expr(task, state, bindings)?;
    let delay_name = lower_expr(delay, state, bindings)?;
    let name = next_name(state, "cpu_ready_after");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "ready_after".to_owned(),
            args: vec![task_name.clone(), delay_name.clone()],
        },
    });
    push_dep_edges(state, &task_name, &name);
    push_dep_edges(state, &delay_name, &name);
    state.yir.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: task_name,
        to: name.clone(),
    });
    state.yir.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: delay_name,
        to: name.clone(),
    });
    Ok(name)
}

fn lower_cpu_present_frame(
    frame: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let frame_name = lower_expr(frame, state, bindings)?;
    let name = next_name(state, "cpu_present_frame");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "present_frame".to_owned(),
            args: vec![frame_name.clone()],
        },
    });
    push_xfer_edge(state, &frame_name, &name);
    state.yir.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: frame_name,
        to: name.clone(),
    });
    Ok(name)
}

fn lower_cpu_extern_call(
    abi: &str,
    callee: &str,
    args: &[NirExpr],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    if let Some(target_config) = &state.target_config {
        if !target_config.supports_host_ffi_abi(abi) {
            return Err(format!(
                "extern ABI `{abi}` is not supported by lowering target `{}`",
                target_config.abi
            ));
        }
    }
    let lowered_args = args
        .iter()
        .map(|arg| lower_expr(arg, state, bindings))
        .collect::<Result<Vec<_>, _>>()?;
    let name = next_name(state, "cpu_extern_call");
    let mut op_args = vec![abi.to_owned(), callee.to_owned()];
    op_args.extend(lowered_args.clone());
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "extern_call_i64".to_owned(),
            args: op_args,
        },
    });
    for arg in lowered_args {
        push_dep_edges(state, &arg, &name);
    }
    Ok(name)
}

fn lower_cpu_extern_call_i32(
    abi: &str,
    callee: &str,
    args: &[NirExpr],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    if let Some(target_config) = &state.target_config {
        if !target_config.supports_host_ffi_abi(abi) {
            return Err(format!(
                "extern ABI `{abi}` is not supported by lowering target `{}`",
                target_config.abi
            ));
        }
    }
    let lowered_args = args
        .iter()
        .map(|arg| lower_expr(arg, state, bindings))
        .collect::<Result<Vec<_>, _>>()?;
    let name = next_name(state, "cpu_extern_call_i32");
    let mut op_args = vec![abi.to_owned(), callee.to_owned()];
    op_args.extend(lowered_args.clone());
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "extern_call_i32".to_owned(),
            args: op_args,
        },
    });
    for arg in lowered_args {
        push_dep_edges(state, &arg, &name);
    }
    Ok(name)
}
