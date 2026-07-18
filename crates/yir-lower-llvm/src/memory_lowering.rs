use std::collections::BTreeMap;

use yir_core::Node;

use super::{
    fresh_reg, lower_buffer_fill,
    value_ref::{get_i64, get_ptr},
    KnownFacts, LlvmValueRef,
};

pub(crate) fn lower_cpu_memory_node(
    node: &Node,
    body: &mut Vec<String>,
    registers: &mut BTreeMap<String, LlvmValueRef>,
    buffer_lengths: &mut BTreeMap<String, String>,
    facts: &mut KnownFacts,
    next_reg: &mut usize,
    last_cpu_value: &mut Option<String>,
) -> Result<bool, String> {
    if node.op.module != "cpu" {
        return Ok(false);
    }

    match node.op.instruction.as_str() {
        "alloc_node" => {
            let (Some(value), Some(next_ptr)) = (
                get_i64(registers, &node.op.args[0]),
                get_ptr(registers, &node.op.args[1]),
            ) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.alloc_node `{}` because its inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            };
            let raw = fresh_reg(next_reg);
            body.push(format!("  {raw} = call ptr @malloc(i64 16)"));
            let value_slot = fresh_reg(next_reg);
            body.push(format!(
                "  {value_slot} = getelementptr inbounds %cpu.node, ptr {raw}, i32 0, i32 0"
            ));
            body.push(format!("  store i64 {value}, ptr {value_slot}"));
            let next_slot = fresh_reg(next_reg);
            body.push(format!(
                "  {next_slot} = getelementptr inbounds %cpu.node, ptr {raw}, i32 0, i32 1"
            ));
            body.push(format!("  store ptr {next_ptr}, ptr {next_slot}"));
            registers.insert(node.name.clone(), LlvmValueRef::Ptr(raw));
        }
        "alloc_buffer" => {
            let (Some(len), Some(fill)) = (
                get_i64(registers, &node.op.args[0]),
                get_i64(registers, &node.op.args[1]),
            ) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.alloc_buffer `{}` because its inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            };
            let len = len.to_owned();
            let fill = fill.to_owned();
            let bytes = fresh_reg(next_reg);
            body.push(format!("  {bytes} = mul i64 {len}, 8"));
            let raw = fresh_reg(next_reg);
            body.push(format!("  {raw} = call ptr @malloc(i64 {bytes})"));
            lower_buffer_fill(body, next_reg, raw.as_str(), len.as_str(), fill.as_str())?;
            registers.insert(node.name.clone(), LlvmValueRef::Ptr(raw.clone()));
            let known_len = facts
                .get_i64(&node.op.args[0])
                .map(|value| value.to_string())
                .unwrap_or(len);
            buffer_lengths.insert(node.name.clone(), known_len);
        }
        "load_value" => {
            let Some(ptr) = get_ptr(registers, &node.op.args[0]) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.load_value `{}` because its input is outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            };
            let slot = fresh_reg(next_reg);
            body.push(format!(
                "  {slot} = getelementptr inbounds %cpu.node, ptr {ptr}, i32 0, i32 0"
            ));
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = load i64, ptr {slot}"));
            registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
            *last_cpu_value = Some(reg);
        }
        "load_next" => {
            let Some(ptr) = get_ptr(registers, &node.op.args[0]) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.load_next `{}` because its input is outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            };
            let slot = fresh_reg(next_reg);
            body.push(format!(
                "  {slot} = getelementptr inbounds %cpu.node, ptr {ptr}, i32 0, i32 1"
            ));
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = load ptr, ptr {slot}"));
            registers.insert(node.name.clone(), LlvmValueRef::Ptr(reg));
            if let Some(len) = buffer_lengths.get(&node.op.args[0]).cloned() {
                buffer_lengths.insert(node.name.clone(), len);
            }
        }
        "buffer_len" => {
            let Some(len) = buffer_lengths.get(&node.op.args[0]).cloned() else {
                body.push(format!(
                        "  ; deferred lowering for cpu.buffer_len `{}` because its input buffer length is outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            };
            registers.insert(node.name.clone(), LlvmValueRef::I64(len.clone()));
            if let Ok(value) = len.parse::<i64>() {
                facts.record_i64(node.name.clone(), value);
            }
            *last_cpu_value = Some(len);
        }
        "copy_buffer_owned" => {
            let Some(ptr) = get_ptr(registers, &node.op.args[0]) else {
                body.push(format!(
                    "  ; deferred lowering for cpu.copy_buffer_owned `{}` because its input buffer is outside the current CPU LLVM slice",
                    node.name
                ));
                return Ok(true);
            };
            let Some(len) = buffer_lengths.get(&node.op.args[0]).cloned() else {
                body.push(format!(
                    "  ; deferred lowering for cpu.copy_buffer_owned `{}` because its input buffer length is outside the current CPU LLVM slice",
                    node.name
                ));
                return Ok(true);
            };
            let byte_len = fresh_reg(next_reg);
            body.push(format!("  {byte_len} = mul i64 {len}, 8"));
            let blob = fresh_reg(next_reg);
            body.push(format!(
                "  {blob} = call ptr @nuis_scheduler_owned_blob_copy_v1(ptr {ptr}, i64 {byte_len}, i64 {})",
                stable_glm_token(&node.name)
            ));
            registers.insert(node.name.clone(), LlvmValueRef::OwnedBytes { blob });
        }
        "owned_bytes_len" => {
            let Some(LlvmValueRef::OwnedBytes { blob }) = registers.get(&node.op.args[0]) else {
                body.push(format!(
                    "  ; deferred lowering for cpu.owned_bytes_len `{}` because its input is outside the current CPU LLVM slice",
                    node.name
                ));
                return Ok(true);
            };
            let len = fresh_reg(next_reg);
            body.push(format!(
                "  {len} = call i64 @nuis_scheduler_owned_blob_len_v1(ptr {blob})"
            ));
            registers.insert(node.name.clone(), LlvmValueRef::I64(len.clone()));
            *last_cpu_value = Some(len);
        }
        "drop_owned_bytes" => {
            let Some(LlvmValueRef::OwnedBytes { blob }) = registers.get(&node.op.args[0]) else {
                body.push(format!(
                    "  ; deferred lowering for cpu.drop_owned_bytes `{}` because its input is outside the current CPU LLVM slice",
                    node.name
                ));
                return Ok(true);
            };
            body.push(format!(
                "  call void @nuis_scheduler_owned_blob_drop_v1(ptr {blob})"
            ));
            registers.insert(node.name.clone(), LlvmValueRef::Void);
        }
        "load_at" => {
            let (Some(ptr), Some(index)) = (
                get_ptr(registers, &node.op.args[0]),
                get_i64(registers, &node.op.args[1]),
            ) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.load_at `{}` because its inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            };
            let slot = fresh_reg(next_reg);
            body.push(format!(
                "  {slot} = getelementptr inbounds i64, ptr {ptr}, i64 {index}"
            ));
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = load i64, ptr {slot}"));
            registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
            *last_cpu_value = Some(reg);
        }
        "store_value" => {
            let (Some(ptr), Some(value)) = (
                get_ptr(registers, &node.op.args[0]),
                get_i64(registers, &node.op.args[1]),
            ) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.store_value `{}` because its inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            };
            let slot = fresh_reg(next_reg);
            body.push(format!(
                "  {slot} = getelementptr inbounds %cpu.node, ptr {ptr}, i32 0, i32 0"
            ));
            body.push(format!("  store i64 {value}, ptr {slot}"));
            registers.insert(node.name.clone(), LlvmValueRef::Void);
        }
        "store_next" => {
            let (Some(ptr), Some(next_ptr)) = (
                get_ptr(registers, &node.op.args[0]),
                get_ptr(registers, &node.op.args[1]),
            ) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.store_next `{}` because its inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            };
            let slot = fresh_reg(next_reg);
            body.push(format!(
                "  {slot} = getelementptr inbounds %cpu.node, ptr {ptr}, i32 0, i32 1"
            ));
            body.push(format!("  store ptr {next_ptr}, ptr {slot}"));
            registers.insert(node.name.clone(), LlvmValueRef::Void);
        }
        "store_at" => {
            let (Some(ptr), Some(index), Some(value)) = (
                get_ptr(registers, &node.op.args[0]),
                get_i64(registers, &node.op.args[1]),
                get_i64(registers, &node.op.args[2]),
            ) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.store_at `{}` because its inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            };
            let slot = fresh_reg(next_reg);
            body.push(format!(
                "  {slot} = getelementptr inbounds i64, ptr {ptr}, i64 {index}"
            ));
            body.push(format!("  store i64 {value}, ptr {slot}"));
            registers.insert(node.name.clone(), LlvmValueRef::Void);
        }
        "is_null" => {
            let Some(ptr) = get_ptr(registers, &node.op.args[0]) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.is_null `{}` because its input is outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            };
            let cmp = fresh_reg(next_reg);
            body.push(format!("  {cmp} = icmp eq ptr {ptr}, null"));
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = zext i1 {cmp} to i64"));
            let known_null = ptr == "null";
            registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
            if known_null {
                facts.record_bool(node.name.clone(), true);
                facts.record_i64(node.name.clone(), 1);
            }
            *last_cpu_value = Some(reg);
        }
        "free" => {
            let Some(ptr) = get_ptr(registers, &node.op.args[0]) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.free `{}` because its input is outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            };
            body.push(format!("  call void @free(ptr {ptr})"));
            registers.insert(node.name.clone(), LlvmValueRef::Void);
        }
        _ => return Ok(false),
    }
    Ok(true)
}

fn stable_glm_token(name: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in name.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    (hash & i64::MAX as u64).max(1)
}
