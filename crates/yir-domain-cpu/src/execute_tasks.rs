use yir_core::{ExecutionState, Node, Resource, TaskLifecycleState, Value};

use crate::runtime_helpers::{task_lifecycle_state, task_lifecycle_state_for_thread};

pub(crate) fn execute_cpu_task_node(
    node: &Node,
    resource: &Resource,
    state: &mut ExecutionState,
) -> Result<Option<Value>, String> {
    let value = match node.op.instruction.as_str() {
        "spawn_task" => {
            let callee = &node.op.args[0];
            let result = state.expect_value(&node.op.args[1])?.clone();
            state.push_resource_event(
                resource,
                format!(
                    "effect cpu.spawn_task @{} [{}] {} => {}",
                    node.resource, resource.kind.raw, callee, node.name
                ),
            );
            Ok(Value::Task(yir_core::TaskHandle {
                label: format!("{callee}@{}", node.name),
                result: Box::new(result),
                limit: None,
                state: TaskLifecycleState::Pending,
            }))
        }
        "spawn_thread" | "thread_spawn" => {
            let callee = &node.op.args[0];
            let result = state.expect_value(&node.op.args[1])?.clone();
            state.push_resource_event(
                resource,
                format!(
                    "effect cpu.{} @{} [{}] {} => {}",
                    node.op.instruction, node.resource, resource.kind.raw, callee, node.name
                ),
            );
            Ok(Value::Thread(yir_core::ThreadHandle {
                label: format!("{callee}@{}", node.name),
                result: Box::new(result),
                state: TaskLifecycleState::Pending,
            }))
        }
        "join" => {
            let task = state.expect_task(&node.op.args[0])?;
            let label = task.label.clone();
            let result = (*task.result).clone();
            let lifecycle = task_lifecycle_state(task);
            if lifecycle == TaskLifecycleState::Cancelled {
                return Err(format!("task `{label}` was cancelled before join"));
            }
            if lifecycle == TaskLifecycleState::TimedOut {
                return Err(format!("task `{label}` timed out before join"));
            }
            state.push_resource_event(
                resource,
                format!(
                    "effect cpu.join @{} [{}]: {}",
                    node.resource, resource.kind.raw, label
                ),
            );
            Ok(result)
        }
        "thread_join" => {
            let thread = state.expect_thread(&node.op.args[0])?;
            let label = thread.label.clone();
            let result = (*thread.result).clone();
            let lifecycle = task_lifecycle_state_for_thread(thread);
            if lifecycle == TaskLifecycleState::Cancelled {
                return Err(format!("thread `{label}` was cancelled before join"));
            }
            if lifecycle == TaskLifecycleState::TimedOut {
                return Err(format!("thread `{label}` timed out before join"));
            }
            state.push_resource_event(
                resource,
                format!(
                    "effect cpu.thread_join @{} [{}]: {}",
                    node.resource, resource.kind.raw, label
                ),
            );
            Ok(result)
        }
        "cancel" => {
            let task = state.expect_task(&node.op.args[0])?;
            let label = task.label.clone();
            let result = (*task.result).clone();
            let limit = task.limit;
            state.push_resource_event(
                resource,
                format!(
                    "effect cpu.cancel @{} [{}]: {}",
                    node.resource, resource.kind.raw, label
                ),
            );
            Ok(Value::Task(yir_core::TaskHandle {
                label,
                result: Box::new(result),
                limit,
                state: TaskLifecycleState::Cancelled,
            }))
        }
        "join_result" => {
            let task = state.expect_task(&node.op.args[0])?;
            let label = task.label.clone();
            let lifecycle = task_lifecycle_state(task);
            let result = if lifecycle == TaskLifecycleState::Completed {
                Some(task.result.clone())
            } else {
                None
            };
            state.push_resource_event(
                resource,
                format!(
                    "effect cpu.join_result @{} [{}]: {} => {}",
                    node.resource, resource.kind.raw, label, lifecycle
                ),
            );
            Ok(Value::TaskResult(yir_core::TaskResultHandle {
                label,
                state: lifecycle,
                result,
            }))
        }
        "thread_join_result" => {
            let thread = state.expect_thread(&node.op.args[0])?;
            let label = thread.label.clone();
            let lifecycle = task_lifecycle_state_for_thread(thread);
            let result = if lifecycle == TaskLifecycleState::Completed {
                Some(thread.result.clone())
            } else {
                None
            };
            state.push_resource_event(
                resource,
                format!(
                    "effect cpu.thread_join_result @{} [{}]: {} => {}",
                    node.resource, resource.kind.raw, label, lifecycle
                ),
            );
            Ok(Value::TaskResult(yir_core::TaskResultHandle {
                label,
                state: lifecycle,
                result,
            }))
        }
        "task_completed" => {
            let result = state.expect_task_result(&node.op.args[0])?;
            Ok(Value::Bool(result.state == TaskLifecycleState::Completed))
        }
        "task_timed_out" => {
            let result = state.expect_task_result(&node.op.args[0])?;
            Ok(Value::Bool(result.state == TaskLifecycleState::TimedOut))
        }
        "task_cancelled" => {
            let result = state.expect_task_result(&node.op.args[0])?;
            Ok(Value::Bool(result.state == TaskLifecycleState::Cancelled))
        }
        "task_value" => {
            let result = state.expect_task_result(&node.op.args[0])?;
            result.result.as_deref().cloned().ok_or_else(|| {
                format!(
                    "task result `{}` has no value in state `{}`",
                    result.label, result.state
                )
            })
        }
        "timeout" => {
            let task = state.expect_task(&node.op.args[0])?;
            let label = task.label.clone();
            let result = (*task.result).clone();
            let limit = state.expect_int(&node.op.args[1])?;
            let lifecycle = task_lifecycle_state(task);
            state.push_resource_event(
                resource,
                format!(
                    "effect cpu.timeout @{} [{}]: {} <= {}",
                    node.resource, resource.kind.raw, label, limit
                ),
            );
            Ok(Value::Task(yir_core::TaskHandle {
                label,
                result: Box::new(result),
                limit: Some(limit),
                state: lifecycle,
            }))
        }
        "mutex_new" => {
            let value = state.expect_value(&node.op.args[0])?.clone();
            state.push_resource_event(
                resource,
                format!(
                    "effect cpu.mutex_new @{} [{}]: {}",
                    node.resource, resource.kind.raw, value
                ),
            );
            Ok(Value::Mutex(yir_core::MutexHandle {
                label: node.name.clone(),
                value: Box::new(value),
            }))
        }
        "mutex_lock" => {
            let mutex = state.expect_mutex(&node.op.args[0])?;
            let label = mutex.label.clone();
            let value = mutex.value.clone();
            state.push_resource_event(
                resource,
                format!(
                    "effect cpu.mutex_lock @{} [{}]: {}",
                    node.resource, resource.kind.raw, label
                ),
            );
            Ok(Value::MutexGuard(yir_core::MutexGuardHandle {
                label,
                value,
            }))
        }
        "mutex_unlock" => {
            let guard = state.expect_mutex_guard(&node.op.args[0])?;
            let label = guard.label.clone();
            let value = guard.value.clone();
            state.push_resource_event(
                resource,
                format!(
                    "effect cpu.mutex_unlock @{} [{}]: {}",
                    node.resource, resource.kind.raw, label
                ),
            );
            Ok(Value::Mutex(yir_core::MutexHandle { label, value }))
        }
        "mutex_value" => {
            let guard = state.expect_mutex_guard(&node.op.args[0])?;
            let label = guard.label.clone();
            let value = (*guard.value).clone();
            state.push_resource_event(
                resource,
                format!(
                    "effect cpu.mutex_value @{} [{}]: {}",
                    node.resource, resource.kind.raw, label
                ),
            );
            Ok(value)
        }
        "await" => {
            let value = state.expect_value(&node.op.args[0])?.clone();
            state.push_resource_event(
                resource,
                format!(
                    "effect cpu.await @{} [{}]: {}",
                    node.resource, resource.kind.raw, value
                ),
            );
            Ok(value)
        }
        "borrow_end" => {
            let pointer = state.expect_pointer(&node.op.args[0])?;
            state.push_resource_event(
                resource,
                format!(
                    "effect cpu.borrow_end @{} [{}] ptr={}",
                    node.resource,
                    resource.kind.raw,
                    pointer
                        .map(|ptr| format!("&{ptr}"))
                        .unwrap_or_else(|| "null".to_owned())
                ),
            );
            Ok(Value::Unit)
        }

        _ => return Ok(None),
    };
    value.map(Some)
}
