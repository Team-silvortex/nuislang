use std::collections::BTreeMap;

use yir_core::Node;

use super::{
    fresh_reg,
    value_ref::{get_bool, get_f32, get_f64, get_i32, get_i64},
    KnownFacts, LlvmValueRef,
};

pub(crate) fn lower_cpu_cast_node(
    node: &Node,
    body: &mut Vec<String>,
    registers: &mut BTreeMap<String, LlvmValueRef>,
    facts: &mut KnownFacts,
    next_reg: &mut usize,
    last_cpu_value: &mut Option<String>,
) -> Result<bool, String> {
    if node.op.module != "cpu" || !node.op.instruction.starts_with("cast_") {
        return Ok(false);
    }

    match node.op.instruction.as_str() {
        "cast_i32_to_i64" => {
            let Some(input) = get_i32(registers, &node.op.args[0]) else {
                body.push(format!(
                    "  ; deferred lowering for cpu.cast_i32_to_i64 `{}` because its input is outside the current CPU LLVM slice",
                    node.name
                ));
                return Ok(true);
            };
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = sext i32 {input} to i64"));
            registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
            if let Some(value) = facts.get_i64(&node.op.args[0]) {
                facts.record_i64(node.name.clone(), value);
            }
            *last_cpu_value = Some(reg);
        }
        "cast_bool_to_i64" => {
            let Some(input) = get_bool(registers, &node.op.args[0]) else {
                body.push(format!(
                    "  ; deferred lowering for cpu.cast_bool_to_i64 `{}` because its input is outside the current CPU LLVM slice",
                    node.name
                ));
                return Ok(true);
            };
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = zext i1 {input} to i64"));
            registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
            if let Some(value) = facts.get_bool(&node.op.args[0]) {
                facts.record_i64(node.name.clone(), i64::from(value));
            }
            *last_cpu_value = Some(reg);
        }
        "cast_i64_to_bool" => {
            let Some(input) = get_i64(registers, &node.op.args[0]) else {
                body.push(format!(
                    "  ; deferred lowering for cpu.cast_i64_to_bool `{}` because its input is outside the current CPU LLVM slice",
                    node.name
                ));
                return Ok(true);
            };
            let i1 = fresh_reg(next_reg);
            body.push(format!("  {i1} = icmp ne i64 {input}, 0"));
            let i64 = fresh_reg(next_reg);
            body.push(format!("  {i64} = zext i1 {i1} to i64"));
            registers.insert(
                node.name.clone(),
                LlvmValueRef::Bool {
                    i1,
                    i64: i64.clone(),
                },
            );
            if let Some(value) = facts.get_i64(&node.op.args[0]) {
                facts.record_bool(node.name.clone(), value != 0);
            }
            *last_cpu_value = Some(i64);
        }
        "cast_i64_to_i32" => {
            let Some(input) = get_i64(registers, &node.op.args[0]) else {
                body.push(format!(
                    "  ; deferred lowering for cpu.cast_i64_to_i32 `{}` because its input is outside the current CPU LLVM slice",
                    node.name
                ));
                return Ok(true);
            };
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = trunc i64 {input} to i32"));
            registers.insert(node.name.clone(), LlvmValueRef::I32(reg.clone()));
            let widened = fresh_reg(next_reg);
            body.push(format!("  {widened} = sext i32 {reg} to i64"));
            if let Some(value) = facts
                .get_i64(&node.op.args[0])
                .and_then(|value| i32::try_from(value).ok())
            {
                facts.record_i64(node.name.clone(), i64::from(value));
            }
            *last_cpu_value = Some(widened);
        }
        "cast_i32_to_f32" => {
            let Some(input) = get_i32(registers, &node.op.args[0]) else {
                body.push(format!(
                    "  ; deferred lowering for cpu.cast_i32_to_f32 `{}` because its input is outside the current CPU LLVM slice",
                    node.name
                ));
                return Ok(true);
            };
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = sitofp i32 {input} to float"));
            registers.insert(node.name.clone(), LlvmValueRef::F32(reg.clone()));
            let widened = fresh_reg(next_reg);
            body.push(format!("  {widened} = fptosi float {reg} to i64"));
            if let Some(value) = facts
                .get_i64(&node.op.args[0])
                .filter(|value| value.abs() <= 16_777_216)
            {
                facts.record_i64(node.name.clone(), value);
            }
            *last_cpu_value = Some(widened);
        }
        "cast_i32_to_f64" => {
            let Some(input) = get_i32(registers, &node.op.args[0]) else {
                body.push(format!(
                    "  ; deferred lowering for cpu.cast_i32_to_f64 `{}` because its input is outside the current CPU LLVM slice",
                    node.name
                ));
                return Ok(true);
            };
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = sitofp i32 {input} to double"));
            registers.insert(node.name.clone(), LlvmValueRef::F64(reg.clone()));
            let widened = fresh_reg(next_reg);
            body.push(format!("  {widened} = fptosi double {reg} to i64"));
            if let Some(value) = facts.get_i64(&node.op.args[0]) {
                facts.record_i64(node.name.clone(), value);
            }
            *last_cpu_value = Some(widened);
        }
        "cast_f32_to_f64" => {
            let Some(input) = get_f32(registers, &node.op.args[0]) else {
                body.push(format!(
                    "  ; deferred lowering for cpu.cast_f32_to_f64 `{}` because its input is outside the current CPU LLVM slice",
                    node.name
                ));
                return Ok(true);
            };
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = fpext float {input} to double"));
            registers.insert(node.name.clone(), LlvmValueRef::F64(reg.clone()));
            let widened = fresh_reg(next_reg);
            body.push(format!("  {widened} = fptosi double {reg} to i64"));
            if let Some(value) = facts.get_i64(&node.op.args[0]) {
                facts.record_i64(node.name.clone(), value);
            }
            *last_cpu_value = Some(widened);
        }
        "cast_f64_to_f32" => {
            let Some(input) = get_f64(registers, &node.op.args[0]) else {
                body.push(format!(
                    "  ; deferred lowering for cpu.cast_f64_to_f32 `{}` because its input is outside the current CPU LLVM slice",
                    node.name
                ));
                return Ok(true);
            };
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = fptrunc double {input} to float"));
            registers.insert(node.name.clone(), LlvmValueRef::F32(reg.clone()));
            let widened = fresh_reg(next_reg);
            body.push(format!("  {widened} = fptosi float {reg} to i64"));
            if let Some(value) = facts
                .get_i64(&node.op.args[0])
                .filter(|value| value.abs() <= 16_777_216)
            {
                facts.record_i64(node.name.clone(), value);
            }
            *last_cpu_value = Some(widened);
        }
        _ => return Ok(false),
    }

    Ok(true)
}
