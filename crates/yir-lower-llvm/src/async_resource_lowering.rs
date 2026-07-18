use yir_core::Node;

use super::{
    facts::propagate_known_facts,
    fresh_reg,
    value_ref::{coerce_to_i64, get_mutex, get_mutex_guard, get_task, get_task_result, get_thread},
    CpuCallScalarKind, LlvmLoweringState, LlvmValueRef, MutexGuardLlvmValueRef, MutexLlvmValueRef,
    StructLlvmValueRef, TaskLlvmValueRef, TaskResultLlvmValueRef, TaskThunkArgument,
    ThreadLlvmValueRef,
};

pub(crate) fn lower_cpu_async_resource_node(node: &Node, state: &mut LlvmLoweringState) -> bool {
    match node.op.instruction.as_str() {
        "async_call" => {
            if node.op.args[1..]
                .iter()
                .any(|arg| !state.registers.contains_key(arg))
            {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.async_call `{}` because one or more args are outside the current CPU LLVM slice",
                    node.name
                ));
                return true;
            }
            state
                .registers
                .insert(node.name.clone(), LlvmValueRef::Void);
            true
        }
        "await" => {
            let Some(value_ref) = state.registers.get(&node.op.args[0]).cloned() else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.await `{}` because its input is outside the current CPU LLVM slice",
                    node.name
                ));
                return true;
            };
            state.registers.insert(node.name.clone(), value_ref.clone());
            if let Some(as_i64) = coerce_to_i64(&value_ref, &mut state.body, &mut state.next_reg) {
                state.last_cpu_value = Some(as_i64);
            }
            propagate_known_facts(&node.op.args[0], &node.name, &mut state.facts);
            propagate_value_field_facts(&node.op.args[0], &node.name, &value_ref, &mut state.facts);
            true
        }
        "spawn_task" => {
            let Some(value_ref) = state.registers.get(&node.op.args[1]).cloned() else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.spawn_task `{}` because its value is outside the current CPU LLVM slice",
                    node.name
                ));
                return true;
            };
            let runtime_handle = match &value_ref {
                LlvmValueRef::DeferredTaskThunkScalar {
                    callee, arguments, ..
                } => {
                    let context = emit_scalar_task_context(arguments, state);
                    let handle = fresh_reg(&mut state.next_reg);
                    state.body.push(format!(
                        "  {handle} = call i64 @nuis_scheduler_task_spawn_invoker_i64_v1(ptr @nuis_task_invoker_{callee}, ptr {context})"
                    ));
                    Some(handle)
                }
                _ => {
                    coerce_to_i64(&value_ref, &mut state.body, &mut state.next_reg).map(|payload| {
                        let handle = fresh_reg(&mut state.next_reg);
                        state.body.push(format!(
                            "  {handle} = call i64 @nuis_scheduler_task_spawn_i64_v1(i64 {payload})"
                        ));
                        handle
                    })
                }
            };
            state.registers.insert(
                node.name.clone(),
                LlvmValueRef::Task(TaskLlvmValueRef {
                    runtime_handle,
                    value: Box::new(value_ref.clone()),
                }),
            );
            propagate_known_facts(&node.op.args[1], &node.name, &mut state.facts);
            propagate_value_field_facts(&node.op.args[1], &node.name, &value_ref, &mut state.facts);
            true
        }
        "spawn_thread" | "thread_spawn" => {
            let Some(value_ref) = state.registers.get(&node.op.args[1]).cloned() else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.{} `{}` because its value is outside the current CPU LLVM slice",
                    node.op.instruction, node.name
                ));
                return true;
            };
            state.registers.insert(
                node.name.clone(),
                LlvmValueRef::Thread(ThreadLlvmValueRef {
                    value: Box::new(value_ref),
                }),
            );
            propagate_known_facts(&node.op.args[1], &node.name, &mut state.facts);
            true
        }
        "cancel" => {
            let Some(task) = get_task(&state.registers, &node.op.args[0]).cloned() else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.cancel `{}` because its task is outside the current CPU LLVM slice",
                    node.name
                ));
                return true;
            };
            if let Some(handle) = task.runtime_handle.as_ref() {
                state.body.push(format!(
                    "  call void @nuis_scheduler_task_cancel_v1(i64 {handle})"
                ));
            }
            state.registers.insert(
                node.name.clone(),
                LlvmValueRef::Task(TaskLlvmValueRef {
                    runtime_handle: task.runtime_handle,
                    value: task.value,
                }),
            );
            propagate_known_facts(&node.op.args[0], &node.name, &mut state.facts);
            true
        }
        "timeout" => {
            let Some(task) = get_task(&state.registers, &node.op.args[0]).cloned() else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.timeout `{}` because its task is outside the current CPU LLVM slice",
                    node.name
                ));
                return true;
            };
            let Some(duration_ref) = state.registers.get(&node.op.args[1]).cloned() else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.timeout `{}` because its duration is outside the current CPU LLVM slice",
                    node.name
                ));
                return true;
            };
            let Some(duration) = coerce_to_i64(&duration_ref, &mut state.body, &mut state.next_reg)
            else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.timeout `{}` because its duration is not i64-compatible",
                    node.name
                ));
                return true;
            };
            if let Some(handle) = task.runtime_handle.as_ref() {
                state.body.push(format!(
                    "  call void @nuis_scheduler_task_timeout_v1(i64 {handle}, i64 {duration})"
                ));
            }
            state.registers.insert(
                node.name.clone(),
                LlvmValueRef::Task(TaskLlvmValueRef {
                    runtime_handle: task.runtime_handle,
                    value: task.value.clone(),
                }),
            );
            propagate_known_facts(&node.op.args[0], &node.name, &mut state.facts);
            true
        }
        "ready_after" => {
            let Some(task) = get_task(&state.registers, &node.op.args[0]).cloned() else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.ready_after `{}` because its task is outside the current CPU LLVM slice",
                    node.name
                ));
                return true;
            };
            let Some(delay_ref) = state.registers.get(&node.op.args[1]).cloned() else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.ready_after `{}` because its delay is outside the current CPU LLVM slice",
                    node.name
                ));
                return true;
            };
            let Some(delay) = coerce_to_i64(&delay_ref, &mut state.body, &mut state.next_reg)
            else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.ready_after `{}` because its delay is not i64-compatible",
                    node.name
                ));
                return true;
            };
            if let Some(handle) = task.runtime_handle.as_ref() {
                state.body.push(format!(
                    "  call void @nuis_scheduler_task_ready_after_v1(i64 {handle}, i64 {delay})"
                ));
            }
            state.registers.insert(
                node.name.clone(),
                LlvmValueRef::Task(TaskLlvmValueRef {
                    runtime_handle: task.runtime_handle,
                    value: task.value,
                }),
            );
            propagate_known_facts(&node.op.args[0], &node.name, &mut state.facts);
            true
        }
        "join_result" => {
            let Some(task) = get_task(&state.registers, &node.op.args[0]).cloned() else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.join_result `{}` because its task is outside the current CPU LLVM slice",
                    node.name
                ));
                return true;
            };
            let runtime_state = task.runtime_handle.as_ref().map(|handle| {
                let result_state = fresh_reg(&mut state.next_reg);
                state.body.push(format!(
                    "  {result_state} = call i64 @nuis_scheduler_task_join_state_v1(i64 {handle})"
                ));
                result_state
            });
            state.registers.insert(
                node.name.clone(),
                LlvmValueRef::TaskResult(TaskResultLlvmValueRef {
                    state: "completed".to_owned(),
                    runtime_state,
                    runtime_handle: task.runtime_handle,
                    value: Some(task.value),
                }),
            );
            propagate_known_facts(&node.op.args[0], &node.name, &mut state.facts);
            true
        }
        "thread_join_result" => {
            let Some(thread) = get_thread(&state.registers, &node.op.args[0]).cloned() else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.thread_join_result `{}` because its thread is outside the current CPU LLVM slice",
                    node.name
                ));
                return true;
            };
            state.registers.insert(
                node.name.clone(),
                LlvmValueRef::TaskResult(TaskResultLlvmValueRef {
                    state: "completed".to_owned(),
                    runtime_state: None,
                    runtime_handle: None,
                    value: Some(thread.value),
                }),
            );
            propagate_known_facts(&node.op.args[0], &node.name, &mut state.facts);
            true
        }
        "join" => {
            let Some(task) = get_task(&state.registers, &node.op.args[0]).cloned() else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.join `{}` because its task is outside the current CPU LLVM slice",
                    node.name
                ));
                return true;
            };
            let value_ref = (*task.value).clone();
            state.registers.insert(node.name.clone(), value_ref.clone());
            if let Some(as_i64) = coerce_to_i64(&value_ref, &mut state.body, &mut state.next_reg) {
                state.last_cpu_value = Some(as_i64);
            }
            propagate_known_facts(&node.op.args[0], &node.name, &mut state.facts);
            true
        }
        "thread_join" => {
            let Some(thread) = get_thread(&state.registers, &node.op.args[0]).cloned() else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.thread_join `{}` because its thread is outside the current CPU LLVM slice",
                    node.name
                ));
                return true;
            };
            let value_ref = (*thread.value).clone();
            state.registers.insert(node.name.clone(), value_ref.clone());
            if let Some(as_i64) = coerce_to_i64(&value_ref, &mut state.body, &mut state.next_reg) {
                state.last_cpu_value = Some(as_i64);
            }
            propagate_known_facts(&node.op.args[0], &node.name, &mut state.facts);
            true
        }
        "task_completed" | "task_timed_out" | "task_cancelled" => {
            let Some(result) = get_task_result(&state.registers, &node.op.args[0]).cloned() else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.{} `{}` because its result is outside the current CPU LLVM slice",
                    node.op.instruction, node.name
                ));
                return true;
            };
            let wanted_state = match node.op.instruction.as_str() {
                "task_completed" => 1,
                "task_timed_out" => 2,
                "task_cancelled" => 3,
                _ => unreachable!(),
            };
            let (i1, known) = if let Some(runtime_state) = result.runtime_state {
                let compared = fresh_reg(&mut state.next_reg);
                state.body.push(format!(
                    "  {compared} = icmp eq i64 {runtime_state}, {wanted_state}"
                ));
                (compared, None)
            } else {
                let value = match node.op.instruction.as_str() {
                    "task_completed" if result.state == "completed" => true,
                    "task_timed_out" if result.state == "timed_out" => true,
                    "task_cancelled" if result.state == "cancelled" => true,
                    _ => false,
                };
                (value.to_string(), Some(value))
            };
            let widened = fresh_reg(&mut state.next_reg);
            state
                .body
                .push(format!("  {widened} = zext i1 {i1} to i64"));
            state.registers.insert(
                node.name.clone(),
                LlvmValueRef::Bool {
                    i1: i1.clone(),
                    i64: widened.clone(),
                },
            );
            if let Some(known) = known {
                state.facts.record_bool(node.name.clone(), known);
            }
            state.last_cpu_value = Some(widened);
            true
        }
        "task_value" => {
            let Some(result) = get_task_result(&state.registers, &node.op.args[0]).cloned() else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.task_value `{}` because its result is outside the current CPU LLVM slice",
                    node.name
                ));
                return true;
            };
            let runtime_handle = result.runtime_handle;
            let Some(mut value_ref) = result.value.map(|value| *value) else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.task_value `{}` because its result carries no payload",
                    node.name
                ));
                return true;
            };
            if let (Some(handle), LlvmValueRef::DeferredTaskThunkScalar { return_kind, .. }) =
                (runtime_handle.as_ref(), &value_ref)
            {
                let payload = fresh_reg(&mut state.next_reg);
                state.body.push(format!(
                    "  {payload} = call i64 @nuis_scheduler_task_value_i64_v1(i64 {handle})"
                ));
                value_ref = unpack_task_payload(payload, *return_kind, state);
            } else if let (Some(handle), LlvmValueRef::I64(_)) =
                (runtime_handle.as_ref(), &value_ref)
            {
                let payload = fresh_reg(&mut state.next_reg);
                state.body.push(format!(
                    "  {payload} = call i64 @nuis_scheduler_task_value_i64_v1(i64 {handle})"
                ));
                value_ref = LlvmValueRef::I64(payload);
            }
            state.registers.insert(node.name.clone(), value_ref.clone());
            if let Some(as_i64) = coerce_to_i64(&value_ref, &mut state.body, &mut state.next_reg) {
                state.last_cpu_value = Some(as_i64);
            }
            propagate_known_facts(&node.op.args[0], &node.name, &mut state.facts);
            true
        }
        "mutex_new" => {
            let Some(value_ref) = state.registers.get(&node.op.args[0]).cloned() else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.mutex_new `{}` because its value is outside the current CPU LLVM slice",
                    node.name
                ));
                return true;
            };
            state.registers.insert(
                node.name.clone(),
                LlvmValueRef::Mutex(MutexLlvmValueRef {
                    value: Box::new(value_ref),
                }),
            );
            propagate_known_facts(&node.op.args[0], &node.name, &mut state.facts);
            true
        }
        "mutex_lock" => {
            let Some(mutex) = get_mutex(&state.registers, &node.op.args[0]).cloned() else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.mutex_lock `{}` because its mutex is outside the current CPU LLVM slice",
                    node.name
                ));
                return true;
            };
            state.registers.insert(
                node.name.clone(),
                LlvmValueRef::MutexGuard(MutexGuardLlvmValueRef { value: mutex.value }),
            );
            propagate_known_facts(&node.op.args[0], &node.name, &mut state.facts);
            true
        }
        "mutex_unlock" => {
            let Some(guard) = get_mutex_guard(&state.registers, &node.op.args[0]).cloned() else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.mutex_unlock `{}` because its guard is outside the current CPU LLVM slice",
                    node.name
                ));
                return true;
            };
            state.registers.insert(
                node.name.clone(),
                LlvmValueRef::Mutex(MutexLlvmValueRef { value: guard.value }),
            );
            propagate_known_facts(&node.op.args[0], &node.name, &mut state.facts);
            true
        }
        "mutex_value" => {
            let Some(guard) = get_mutex_guard(&state.registers, &node.op.args[0]).cloned() else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.mutex_value `{}` because its guard is outside the current CPU LLVM slice",
                    node.name
                ));
                return true;
            };
            let value_ref = (*guard.value).clone();
            state.registers.insert(node.name.clone(), value_ref.clone());
            if let Some(as_i64) = coerce_to_i64(&value_ref, &mut state.body, &mut state.next_reg) {
                state.last_cpu_value = Some(as_i64);
            }
            propagate_known_facts(&node.op.args[0], &node.name, &mut state.facts);
            true
        }
        _ => false,
    }
}

fn emit_scalar_task_context(
    arguments: &[TaskThunkArgument],
    state: &mut LlvmLoweringState,
) -> String {
    if arguments.is_empty() {
        return "null".to_owned();
    }
    let context = fresh_reg(&mut state.next_reg);
    state.body.push(format!(
        "  {context} = call ptr @malloc(i64 {})",
        arguments.len() * 8
    ));
    for (index, argument) in arguments.iter().enumerate() {
        let pointer = if index == 0 {
            context.clone()
        } else {
            let pointer = fresh_reg(&mut state.next_reg);
            state.body.push(format!(
                "  {pointer} = getelementptr i8, ptr {context}, i64 {}",
                index * 8
            ));
            pointer
        };
        let packed = match argument.kind {
            CpuCallScalarKind::Bool => {
                let packed = fresh_reg(&mut state.next_reg);
                state
                    .body
                    .push(format!("  {packed} = zext i1 {} to i64", argument.value));
                packed
            }
            CpuCallScalarKind::I32 => {
                let packed = fresh_reg(&mut state.next_reg);
                state
                    .body
                    .push(format!("  {packed} = sext i32 {} to i64", argument.value));
                packed
            }
            CpuCallScalarKind::I64 => argument.value.clone(),
            CpuCallScalarKind::F32 | CpuCallScalarKind::F64 => {
                unreachable!("floating-point task arguments are not normalized yet")
            }
        };
        state
            .body
            .push(format!("  store i64 {packed}, ptr {pointer}, align 8"));
    }
    context
}

fn unpack_task_payload(
    payload: String,
    return_kind: CpuCallScalarKind,
    state: &mut LlvmLoweringState,
) -> LlvmValueRef {
    match return_kind {
        CpuCallScalarKind::Bool => {
            let i1 = fresh_reg(&mut state.next_reg);
            state
                .body
                .push(format!("  {i1} = trunc i64 {payload} to i1"));
            LlvmValueRef::Bool { i1, i64: payload }
        }
        CpuCallScalarKind::I32 => {
            let i32 = fresh_reg(&mut state.next_reg);
            state
                .body
                .push(format!("  {i32} = trunc i64 {payload} to i32"));
            LlvmValueRef::I32(i32)
        }
        CpuCallScalarKind::I64 => LlvmValueRef::I64(payload),
        CpuCallScalarKind::F32 | CpuCallScalarKind::F64 => {
            unreachable!("floating-point task payloads are not normalized yet")
        }
    }
}

fn propagate_value_field_facts(
    from: &str,
    to: &str,
    value_ref: &LlvmValueRef,
    facts: &mut super::KnownFacts,
) {
    match value_ref {
        LlvmValueRef::Struct(struct_value) => {
            propagate_struct_field_facts(from, to, struct_value, facts);
        }
        LlvmValueRef::VariantUnion(union) => {
            for struct_value in union.variants.values() {
                propagate_struct_field_facts(from, to, struct_value, facts);
            }
        }
        _ => {}
    }
}

fn propagate_struct_field_facts(
    from: &str,
    to: &str,
    struct_value: &StructLlvmValueRef,
    facts: &mut super::KnownFacts,
) {
    for (field_name, _) in &struct_value.fields {
        facts.copy_field_facts(from, to, field_name);
    }
}
