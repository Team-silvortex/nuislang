use yir_core::Node;

use super::{
    fresh_global, fresh_reg, llvm_c_string_bytes,
    value_ref::{
        coerce_to_i64, get_mutex, get_mutex_guard, get_network_result, get_ptr, get_struct,
        get_task, get_task_result, get_thread,
    },
    variant_select::{emit_variant_is_value, variant_field_value},
    LlvmLoweringState, LlvmValueRef, MutexGuardLlvmValueRef, MutexLlvmValueRef,
    NetworkResultLlvmValueRef, StructLlvmValueRef, TaskLlvmValueRef, TaskResultLlvmValueRef,
    ThreadLlvmValueRef,
};

pub(crate) fn lower_cpu_literal_node(node: &Node, state: &mut LlvmLoweringState) -> bool {
    match node.op.instruction.as_str() {
        "text" => {
            let label = fresh_global(&mut state.next_global);
            let (bytes, len) = llvm_c_string_bytes(&node.op.args[0]);
            state.globals.push(format!(
                "{label} = private unnamed_addr constant [{len} x i8] c\"{bytes}\""
            ));
            let ptr = fresh_reg(&mut state.next_reg);
            state.body.push(format!(
                "  {ptr} = getelementptr inbounds [{len} x i8], ptr {label}, i64 0, i64 0"
            ));
            let handle = fresh_reg(&mut state.next_reg);
            state.body.push(format!(
                "  {handle} = call i64 @nuis_host_text_lift(ptr {ptr})"
            ));
            state
                .registers
                .insert(node.name.clone(), LlvmValueRef::TextHandle { ptr, handle });
            true
        }
        "const_bool" => {
            let value = match node.op.args[0].as_str() {
                "true" => "true",
                "false" => "false",
                _ => {
                    state.body.push(format!(
                        "  ; deferred lowering for cpu.const_bool `{}` because literal `{}` is invalid",
                        node.name, node.op.args[0]
                    ));
                    return true;
                }
            };
            let widened = fresh_reg(&mut state.next_reg);
            state
                .body
                .push(format!("  {widened} = zext i1 {value} to i64"));
            state.registers.insert(
                node.name.clone(),
                LlvmValueRef::Bool {
                    i1: value.to_owned(),
                    i64: widened.clone(),
                },
            );
            state.last_cpu_value = Some(widened);
            true
        }
        "const_i32" => {
            let reg = fresh_reg(&mut state.next_reg);
            state
                .body
                .push(format!("  {reg} = add i32 0, {}", node.op.args[0]));
            state
                .registers
                .insert(node.name.clone(), LlvmValueRef::I32(reg.clone()));
            let widened = fresh_reg(&mut state.next_reg);
            state
                .body
                .push(format!("  {widened} = sext i32 {reg} to i64"));
            state.last_cpu_value = Some(widened);
            true
        }
        "const" | "const_i64" => {
            let reg = fresh_reg(&mut state.next_reg);
            state
                .body
                .push(format!("  {reg} = add i64 0, {}", node.op.args[0]));
            state
                .registers
                .insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
            state.last_cpu_value = Some(reg);
            true
        }
        "const_f32" => {
            let reg = fresh_reg(&mut state.next_reg);
            state
                .body
                .push(format!("  {reg} = fadd float 0.0, {}", node.op.args[0]));
            state
                .registers
                .insert(node.name.clone(), LlvmValueRef::F32(reg.clone()));
            let widened = fresh_reg(&mut state.next_reg);
            state
                .body
                .push(format!("  {widened} = fptosi float {reg} to i64"));
            state.last_cpu_value = Some(widened);
            true
        }
        "const_f64" => {
            let reg = fresh_reg(&mut state.next_reg);
            state
                .body
                .push(format!("  {reg} = fadd double 0.0, {}", node.op.args[0]));
            state
                .registers
                .insert(node.name.clone(), LlvmValueRef::F64(reg.clone()));
            let widened = fresh_reg(&mut state.next_reg);
            state
                .body
                .push(format!("  {widened} = fptosi double {reg} to i64"));
            state.last_cpu_value = Some(widened);
            true
        }
        "null" => {
            state
                .registers
                .insert(node.name.clone(), LlvmValueRef::Ptr("null".to_owned()));
            true
        }
        _ => false,
    }
}

pub(crate) fn lower_cpu_aggregate_node(node: &Node, state: &mut LlvmLoweringState) -> bool {
    match node.op.instruction.as_str() {
        "struct" => {
            let mut fields = Vec::new();
            let type_name = node.op.args[0].clone();
            for entry in &node.op.args[1..] {
                let Some((field_name, value_name)) = entry.split_once('=') else {
                    state.body.push(format!(
                        "  ; deferred lowering for cpu.struct `{}` because field binding `{}` is invalid",
                        node.name, entry
                    ));
                    return true;
                };
                let Some(value_ref) = state.registers.get(value_name.trim()).cloned() else {
                    state.body.push(format!(
                        "  ; deferred lowering for cpu.struct `{}` because field `{}` comes from outside the current CPU LLVM slice",
                        node.name, field_name
                    ));
                    return true;
                };
                fields.push((field_name.trim().to_owned(), value_ref));
            }
            state.registers.insert(
                node.name.clone(),
                LlvmValueRef::Struct(StructLlvmValueRef { type_name, fields }),
            );
            true
        }
        "field" => {
            let Some(struct_value) = get_struct(&state.registers, &node.op.args[0]) else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.field `{}` because its source struct is outside the current CPU LLVM slice",
                    node.name
                ));
                return true;
            };
            let field_name = &node.op.args[1];
            let Some((_, field_value)) = struct_value
                .fields
                .iter()
                .find(|(name, _)| name == field_name)
            else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.field `{}` because field `{}` does not exist on `{}`",
                    node.name, field_name, struct_value.type_name
                ));
                return true;
            };
            let field_value = field_value.clone();
            state
                .registers
                .insert(node.name.clone(), field_value.clone());
            if let Some(as_i64) = coerce_to_i64(&field_value, &mut state.body, &mut state.next_reg)
            {
                state.last_cpu_value = Some(as_i64);
            }
            true
        }
        "variant_is" => {
            let Some(value_ref) = state.registers.get(&node.op.args[0]).cloned() else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.variant_is `{}` because its input is outside the current CPU LLVM slice",
                    node.name
                ));
                return true;
            };
            let variant_name = &node.op.args[1];
            let Some(bool_ref) = emit_variant_is_value(
                &value_ref,
                variant_name,
                &mut state.body,
                &mut state.next_reg,
            ) else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.variant_is `{}` because `{}` is not a variant-shaped value",
                    node.name, node.op.args[0]
                ));
                return true;
            };
            if let LlvmValueRef::Bool { i64, .. } = &bool_ref {
                state.last_cpu_value = Some(i64.clone());
            }
            state.registers.insert(node.name.clone(), bool_ref);
            true
        }
        "variant_field" => {
            let Some(value_ref) = state.registers.get(&node.op.args[0]).cloned() else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.variant_field `{}` because its input is outside the current CPU LLVM slice",
                    node.name
                ));
                return true;
            };
            let variant_name = &node.op.args[1];
            let field_name = &node.op.args[2];
            let Some(field_value) = variant_field_value(&value_ref, variant_name, field_name)
            else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.variant_field `{}` because field `{}` does not exist on variant `{}`",
                    node.name, field_name, variant_name
                ));
                return true;
            };
            state
                .registers
                .insert(node.name.clone(), field_value.clone());
            if let Some(as_i64) = coerce_to_i64(&field_value, &mut state.body, &mut state.next_reg)
            {
                state.last_cpu_value = Some(as_i64);
            }
            true
        }
        _ => false,
    }
}

pub(crate) fn lower_cpu_pointer_node(node: &Node, state: &mut LlvmLoweringState) -> bool {
    match node.op.instruction.as_str() {
        "borrow" | "move_ptr" => {
            let Some(ptr) = get_ptr(&state.registers, &node.op.args[0]) else {
                state.body.push(format!(
                    "  ; deferred lowering for {} `{}` because its input is outside the current CPU LLVM slice",
                    node.op.full_name(),
                    node.name
                ));
                return true;
            };
            state
                .registers
                .insert(node.name.clone(), LlvmValueRef::Ptr(ptr.to_owned()));
            if let Some(len) = state.buffer_lengths.get(&node.op.args[0]).cloned() {
                state.buffer_lengths.insert(node.name.clone(), len);
            }
            true
        }
        "borrow_end" => {
            state
                .registers
                .insert(node.name.clone(), LlvmValueRef::Void);
            true
        }
        _ => false,
    }
}

pub(crate) fn lower_network_observer_node(node: &Node, state: &mut LlvmLoweringState) -> bool {
    match node.op.instruction.as_str() {
        "observe" => {
            let Some(value_ref) = state.registers.get(&node.op.args[0]).cloned() else {
                state.body.push(format!(
                    "  ; deferred lowering for network.observe `{}` because its input is outside the current CPU LLVM slice",
                    node.name
                ));
                return true;
            };
            state.registers.insert(
                node.name.clone(),
                LlvmValueRef::NetworkResult(NetworkResultLlvmValueRef {
                    state: node.op.args[1].clone(),
                    value: Box::new(value_ref),
                }),
            );
            true
        }
        "is_config_ready" | "is_send_ready" | "is_recv_ready" | "is_connect_ready"
        | "is_accept_ready" | "is_closed" => {
            let Some(result) = get_network_result(&state.registers, &node.op.args[0]) else {
                state.body.push(format!(
                    "  ; deferred lowering for network.{} `{}` because its result is outside the current CPU LLVM slice",
                    node.op.instruction, node.name
                ));
                return true;
            };
            let wanted_state = match node.op.instruction.as_str() {
                "is_config_ready" => "config_ready",
                "is_send_ready" => "send_ready",
                "is_recv_ready" => "recv_ready",
                "is_connect_ready" => "connect_ready",
                "is_accept_ready" => "accept_ready",
                "is_closed" => "closed",
                _ => unreachable!(),
            };
            let i1 = if result.state == wanted_state {
                "true".to_owned()
            } else {
                "false".to_owned()
            };
            let widened = fresh_reg(&mut state.next_reg);
            state
                .body
                .push(format!("  {widened} = zext i1 {i1} to i64"));
            state.registers.insert(
                node.name.clone(),
                LlvmValueRef::Bool {
                    i1,
                    i64: widened.clone(),
                },
            );
            state.last_cpu_value = Some(widened);
            true
        }
        "value" => {
            let Some(result) = get_network_result(&state.registers, &node.op.args[0]) else {
                state.body.push(format!(
                    "  ; deferred lowering for network.value `{}` because its result is outside the current CPU LLVM slice",
                    node.name
                ));
                return true;
            };
            let value_ref = (*result.value).clone();
            state.registers.insert(node.name.clone(), value_ref.clone());
            if let Some(as_i64) = coerce_to_i64(&value_ref, &mut state.body, &mut state.next_reg) {
                state.last_cpu_value = Some(as_i64);
            }
            true
        }
        _ => false,
    }
}

pub(crate) fn lower_cpu_async_resource_node(node: &Node, state: &mut LlvmLoweringState) -> bool {
    match node.op.instruction.as_str() {
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
                    value: Box::new(value_ref),
                }),
            );
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
                    i1,
                    i64: widened.clone(),
                },
            );
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
            true
        }
        _ => false,
    }
}
