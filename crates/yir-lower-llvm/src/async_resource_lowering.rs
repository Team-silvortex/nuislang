use yir_core::Node;

use super::{
    facts::propagate_known_facts,
    fresh_reg,
    value_ref::{coerce_to_i64, get_mutex, get_mutex_guard, get_task, get_task_result, get_thread},
    LlvmLoweringState, LlvmValueRef, MutexGuardLlvmValueRef, MutexLlvmValueRef, StructLlvmValueRef,
    TaskLlvmValueRef, TaskResultLlvmValueRef, ThreadLlvmValueRef,
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
            state.registers.insert(
                node.name.clone(),
                LlvmValueRef::Task(TaskLlvmValueRef {
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
        "join_result" => {
            let Some(task) = get_task(&state.registers, &node.op.args[0]).cloned() else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.join_result `{}` because its task is outside the current CPU LLVM slice",
                    node.name
                ));
                return true;
            };
            state.registers.insert(
                node.name.clone(),
                LlvmValueRef::TaskResult(TaskResultLlvmValueRef {
                    state: "completed".to_owned(),
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
            let i1 = match node.op.instruction.as_str() {
                "task_completed" if result.state == "completed" => "true",
                "task_timed_out" if result.state == "timed_out" => "true",
                "task_cancelled" if result.state == "cancelled" => "true",
                _ => "false",
            }
            .to_owned();
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
            state.facts.record_bool(node.name.clone(), i1 == "true");
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
            let Some(value_ref) = result.value.map(|value| *value) else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.task_value `{}` because its result carries no payload",
                    node.name
                ));
                return true;
            };
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
