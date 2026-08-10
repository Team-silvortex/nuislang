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
                ready_delay: 1,
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
            if lifecycle == TaskLifecycleState::Failed {
                return Err(format!("task `{label}` failed before join"));
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
            if lifecycle == TaskLifecycleState::Failed {
                return Err(format!("thread `{label}` failed before join"));
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
            let ready_delay = task.ready_delay;
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
                ready_delay,
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
        "task_failed" => {
            let result = state.expect_task_result(&node.op.args[0])?;
            Ok(Value::Bool(result.state == TaskLifecycleState::Failed))
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
            let ready_delay = task.ready_delay;
            let lifecycle = task.state;
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
                ready_delay,
                state: lifecycle,
            }))
        }
        "ready_after" => {
            let task = state.expect_task(&node.op.args[0])?;
            let label = task.label.clone();
            let result = (*task.result).clone();
            let limit = task.limit;
            let lifecycle = task.state;
            let ready_delay = state.expect_int(&node.op.args[1])?.max(0);
            state.push_resource_event(
                resource,
                format!(
                    "effect cpu.ready_after @{} [{}]: {} <= {}",
                    node.resource, resource.kind.raw, label, ready_delay
                ),
            );
            Ok(Value::Task(yir_core::TaskHandle {
                label,
                result: Box::new(result),
                limit,
                ready_delay,
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
        "mutex_share" => {
            let mutex = state.expect_mutex(&node.op.args[0])?;
            let label = mutex.label.clone();
            let value = mutex.value.clone();
            let permit_cardinality = state.expect_int(&node.op.args[1])?;
            if !(1..=64).contains(&permit_cardinality) {
                return Err(format!(
                    "shared mutex `{label}` permit cardinality `{permit_cardinality}` is outside `1..=64`"
                ));
            }
            if state.closed_shared_mutexes.contains(&label) {
                return Err(format!(
                    "shared mutex `{label}` cannot be reopened after close"
                ));
            }
            state
                .shared_mutex_values
                .insert(label.clone(), (*value).clone());
            state
                .shared_mutex_release_epochs
                .entry(label.clone())
                .or_insert(0);
            state
                .shared_mutex_permit_cardinalities
                .insert(label.clone(), permit_cardinality);
            state.push_resource_event(
                resource,
                format!(
                    "effect cpu.mutex_share @{} [{}]: {} permit_cardinality={permit_cardinality}",
                    node.resource, resource.kind.raw, label,
                ),
            );
            Ok(Value::Mutex(yir_core::MutexHandle { label, value }))
        }
        "mutex_shared_close" => {
            let mutex = state.expect_mutex(&node.op.args[0])?;
            let label = mutex.label.clone();
            if state.closed_shared_mutexes.contains(&label) {
                return Err(format!("shared mutex `{label}` is already closed"));
            }
            if state.active_mutex_leases.contains(&label) {
                return Err(format!(
                    "shared mutex `{label}` cannot close while a lease is active"
                ));
            }
            let revoked = state
                .live_mutex_permits
                .iter()
                .filter(|(permit_label, _)| permit_label == &label)
                .count();
            state
                .live_mutex_permits
                .retain(|(permit_label, _)| permit_label != &label);
            state.closed_shared_mutexes.insert(label.clone());
            let release_epoch = state
                .shared_mutex_release_epochs
                .get(&label)
                .copied()
                .unwrap_or(0)
                .checked_add(1)
                .ok_or_else(|| format!("shared mutex `{label}` release epoch overflow"))?;
            state
                .shared_mutex_release_epochs
                .insert(label.clone(), release_epoch);
            state.push_resource_event(
                resource,
                format!(
                    "effect cpu.mutex_shared_close @{} [{}]: {} revoked={revoked} release_epoch={release_epoch}",
                    node.resource, resource.kind.raw, label,
                ),
            );
            state.shared_mutex_values.remove(&label);
            state.shared_mutex_release_epochs.remove(&label);
            state.shared_mutex_permit_cardinalities.remove(&label);
            Ok(Value::Int(revoked as i64))
        }
        "mutex_permit" => {
            let mutex = state.expect_mutex(&node.op.args[0])?;
            let label = mutex.label.clone();
            if state.closed_shared_mutexes.contains(&label) {
                return Err(format!(
                    "shared mutex `{label}` cannot issue permit after close"
                ));
            }
            let permit_cardinality = state
                .shared_mutex_permit_cardinalities
                .get(&label)
                .copied()
                .ok_or_else(|| {
                    format!("shared mutex `{label}` has no static permit cardinality")
                })?;
            let value = state
                .shared_mutex_values
                .get(&label)
                .cloned()
                .map(Box::new)
                .unwrap_or_else(|| mutex.value.clone());
            let lane = state.expect_int(&node.op.args[1])?;
            if lane < 0 || lane >= permit_cardinality {
                return Err(format!(
                    "mutex permit lane `{lane}` is outside configured range `0..{permit_cardinality}`"
                ));
            }
            if !state.live_mutex_permits.insert((label.clone(), lane)) {
                return Err(format!(
                    "shared mutex `{label}` already issued live permit lane `{lane}`"
                ));
            }
            state.push_resource_event(
                resource,
                format!(
                    "effect cpu.mutex_permit @{} [{}]: {} lane={} permit_cardinality={permit_cardinality}",
                    node.resource, resource.kind.raw, label, lane,
                ),
            );
            Ok(Value::MutexPermit(yir_core::MutexPermitHandle {
                label,
                lane,
                value,
            }))
        }
        "mutex_permit_lock" => {
            let permit = state.expect_mutex_permit(&node.op.args[0])?;
            let label = permit.label.clone();
            let lane = permit.lane;
            let fallback_value = permit.value.clone();
            if state.closed_shared_mutexes.contains(&label) {
                return Err(format!(
                    "mutex permit `{label}:{lane}` was revoked by shared close"
                ));
            }
            if state.active_mutex_leases.contains(&label) {
                return Err(format!(
                    "shared mutex `{label}` already has an active lease"
                ));
            }
            if !state.live_mutex_permits.remove(&(label.clone(), lane)) {
                return Err(format!(
                    "mutex permit `{label}:{lane}` is stale or consumed"
                ));
            }
            let value = state
                .shared_mutex_values
                .get(&label)
                .cloned()
                .map(Box::new)
                .unwrap_or(fallback_value);
            state.active_mutex_leases.insert(label.clone());
            state.push_resource_event(
                resource,
                format!(
                    "effect cpu.mutex_permit_lock @{} [{}]: {} lane={}",
                    node.resource, resource.kind.raw, label, lane
                ),
            );
            Ok(Value::MutexGuard(yir_core::MutexGuardHandle {
                label,
                value,
            }))
        }
        "mutex_lease_value" => {
            let lease = state.expect_mutex_guard(&node.op.args[0])?;
            let label = lease.label.clone();
            if !state.active_mutex_leases.contains(&label) {
                return Err(format!(
                    "mutex lease `{label}` is stale or already released"
                ));
            }
            state
                .shared_mutex_values
                .get(&label)
                .cloned()
                .ok_or_else(|| format!("shared mutex `{label}` has no published value"))
        }
        "mutex_lease_replace" => {
            let lease = state.expect_mutex_guard(&node.op.args[0])?;
            let label = lease.label.clone();
            if !state.active_mutex_leases.contains(&label) {
                return Err(format!(
                    "mutex lease `{label}` is stale or already released"
                ));
            }
            let replacement = Value::Int(state.expect_int(&node.op.args[1])?);
            let old = state
                .shared_mutex_values
                .insert(label.clone(), replacement.clone())
                .ok_or_else(|| format!("shared mutex `{label}` has no published value"))?;
            let release_epoch = state
                .shared_mutex_release_epochs
                .get(&label)
                .copied()
                .unwrap_or(0)
                .checked_add(1)
                .ok_or_else(|| format!("shared mutex `{label}` release epoch overflow"))?;
            state
                .shared_mutex_release_epochs
                .insert(label.clone(), release_epoch);
            state.push_resource_event(
                resource,
                format!(
                    "effect cpu.mutex_lease_replace @{} [{}]: {} old={} new={} release_epoch={release_epoch}",
                    node.resource, resource.kind.raw, label, old, replacement,
                ),
            );
            Ok(old)
        }
        "mutex_lease_unlock" => {
            let lease = state.expect_mutex_guard(&node.op.args[0])?;
            let label = lease.label.clone();
            if !state.active_mutex_leases.remove(&label) {
                return Err(format!(
                    "mutex lease `{label}` is stale or already released"
                ));
            }
            let release_epoch = state
                .shared_mutex_release_epochs
                .get(&label)
                .copied()
                .unwrap_or(0)
                .checked_add(1)
                .ok_or_else(|| format!("shared mutex `{label}` release epoch overflow"))?;
            state
                .shared_mutex_release_epochs
                .insert(label.clone(), release_epoch);
            state.push_resource_event(
                resource,
                format!(
                    "effect cpu.mutex_lease_unlock @{} [{}]: {} release_epoch={release_epoch}",
                    node.resource, resource.kind.raw, label,
                ),
            );
            Ok(Value::Int(1))
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
