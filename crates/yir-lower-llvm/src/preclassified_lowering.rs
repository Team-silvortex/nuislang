use yir_core::Node;

use super::{
    facts::propagate_known_facts,
    fresh_global, fresh_reg, llvm_c_string_bytes,
    value_ref::{coerce_to_i64, get_network_result, get_ptr, get_struct},
    variant_select::{emit_variant_is_value, variant_field_value, variant_parent_name},
    KnownFacts, LlvmLoweringState, LlvmValueRef, NetworkResultLlvmValueRef, StructLlvmValueRef,
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
            state.facts.record_bool(node.name.clone(), value == "true");
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
            if let Ok(value) = node.op.args[0].parse::<i32>() {
                state.facts.record_i64(node.name.clone(), i64::from(value));
            }
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
            if let Ok(value) = node.op.args[0].parse::<i64>() {
                state.facts.record_i64(node.name.clone(), value);
            }
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
                let field_fact_key = KnownFacts::struct_field_key(&node.name, field_name.trim());
                propagate_known_facts(value_name.trim(), &field_fact_key, &mut state.facts);
                fields.push((field_name.trim().to_owned(), value_ref));
            }
            state.registers.insert(
                node.name.clone(),
                LlvmValueRef::Struct(StructLlvmValueRef { type_name, fields }),
            );
            if variant_parent_name(&node.op.args[0]).is_some() {
                state
                    .facts
                    .record_variant_type(node.name.clone(), node.op.args[0].clone());
            }
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
            let field_fact_key = KnownFacts::struct_field_key(&node.op.args[0], field_name);
            propagate_known_facts(&field_fact_key, &node.name, &mut state.facts);
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
            match &value_ref {
                LlvmValueRef::Struct(struct_value) => {
                    state
                        .facts
                        .record_bool(node.name.clone(), struct_value.type_name == *variant_name);
                }
                LlvmValueRef::VariantUnion(union) => {
                    if let Ok(tag) = union.tag_i64.parse::<i64>() {
                        state.facts.record_bool(
                            node.name.clone(),
                            tag == super::variant_select::variant_tag_value(variant_name),
                        );
                    } else if let Some(active_variant) =
                        state.facts.get_variant_type(&node.op.args[0])
                    {
                        state
                            .facts
                            .record_bool(node.name.clone(), active_variant == variant_name);
                    }
                }
                _ => {}
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
                if is_wrong_concrete_variant_access(&value_ref, variant_name) {
                    state.delayed_registers.insert(
                        node.name.clone(),
                        format!(
                            "cpu.variant_field `{}` waits for lazy select because `{}` is not active on `{}`",
                            node.name, variant_name, node.op.args[0]
                        ),
                    );
                    return true;
                }
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
            let field_fact_key = KnownFacts::struct_field_key(&node.op.args[0], field_name);
            propagate_known_facts(&field_fact_key, &node.name, &mut state.facts);
            true
        }
        "async_value" => {
            let Some(value_ref) = state.registers.get(&node.op.args[0]).cloned() else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.async_value `{}` because its input is outside the current CPU LLVM slice",
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
        _ => false,
    }
}

fn is_wrong_concrete_variant_access(value_ref: &LlvmValueRef, variant_name: &str) -> bool {
    let LlvmValueRef::Struct(struct_value) = value_ref else {
        return false;
    };
    if struct_value.type_name == variant_name {
        return false;
    }
    variant_parent_name(&struct_value.type_name)
        .zip(variant_parent_name(variant_name))
        .is_some_and(|(actual_parent, expected_parent)| actual_parent == expected_parent)
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
            propagate_known_facts(&node.op.args[0], &node.name, &mut state.facts);
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
                    i1: i1.clone(),
                    i64: widened.clone(),
                },
            );
            state.facts.record_bool(node.name.clone(), i1 == "true");
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
            propagate_known_facts(&node.op.args[0], &node.name, &mut state.facts);
            true
        }
        _ => false,
    }
}
