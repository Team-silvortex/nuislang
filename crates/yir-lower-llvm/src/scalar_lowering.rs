use std::collections::BTreeMap;

use yir_core::Node;

use super::{
    fresh_reg,
    value_ref::{get_f32, get_f64, get_i32, get_i64},
    LlvmValueRef,
};

pub(crate) fn lower_cpu_scalar_node(
    node: &Node,
    body: &mut Vec<String>,
    registers: &mut BTreeMap<String, LlvmValueRef>,
    known_i64_values: &mut BTreeMap<String, i64>,
    next_reg: &mut usize,
    last_cpu_value: &mut Option<String>,
) -> Result<bool, String> {
    if node.op.module != "cpu" {
        return Ok(false);
    }

    match node.op.instruction.as_str() {
        "add" => {
            if let (Some(lhs), Some(rhs)) = (
                get_f64(registers, &node.op.args[0]),
                get_f64(registers, &node.op.args[1]),
            ) {
                let reg = fresh_reg(next_reg);
                body.push(format!("  {reg} = fadd double {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::F64(reg.clone()));
                let widened = fresh_reg(next_reg);
                body.push(format!("  {widened} = fptosi double {reg} to i64"));
                *last_cpu_value = Some(widened);
            } else if let (Some(lhs), Some(rhs)) = (
                get_f32(registers, &node.op.args[0]),
                get_f32(registers, &node.op.args[1]),
            ) {
                let reg = fresh_reg(next_reg);
                body.push(format!("  {reg} = fadd float {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::F32(reg.clone()));
                let widened = fresh_reg(next_reg);
                body.push(format!("  {widened} = fptosi float {reg} to i64"));
                *last_cpu_value = Some(widened);
            } else if let (Some(lhs), Some(rhs)) = (
                get_i64(registers, &node.op.args[0]),
                get_i64(registers, &node.op.args[1]),
            ) {
                let reg = fresh_reg(next_reg);
                body.push(format!("  {reg} = add i64 {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                record_known_i64_binary_op(node, known_i64_values, i64::checked_add);
                *last_cpu_value = Some(reg);
            } else if let (Some(lhs), Some(rhs)) = (
                get_i32(registers, &node.op.args[0]),
                get_i32(registers, &node.op.args[1]),
            ) {
                let reg = fresh_reg(next_reg);
                body.push(format!("  {reg} = add i32 {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::I32(reg.clone()));
                let widened = fresh_reg(next_reg);
                body.push(format!("  {widened} = sext i32 {reg} to i64"));
                *last_cpu_value = Some(widened);
            } else {
                body.push(format!(
                        "  ; deferred lowering for cpu.add `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            }
        }
        "add_i32" => {
            let (Some(lhs), Some(rhs)) = (
                get_i32(registers, &node.op.args[0]),
                get_i32(registers, &node.op.args[1]),
            ) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.add_i32 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            };
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = add i32 {lhs}, {rhs}"));
            registers.insert(node.name.clone(), LlvmValueRef::I32(reg.clone()));
            let widened = fresh_reg(next_reg);
            body.push(format!("  {widened} = sext i32 {reg} to i64"));
            *last_cpu_value = Some(widened);
        }
        "add_f32" => {
            let (Some(lhs), Some(rhs)) = (
                get_f32(registers, &node.op.args[0]),
                get_f32(registers, &node.op.args[1]),
            ) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.add_f32 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            };
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = fadd float {lhs}, {rhs}"));
            registers.insert(node.name.clone(), LlvmValueRef::F32(reg.clone()));
            let widened = fresh_reg(next_reg);
            body.push(format!("  {widened} = fptosi float {reg} to i64"));
            *last_cpu_value = Some(widened);
        }
        "add_f64" => {
            let (Some(lhs), Some(rhs)) = (
                get_f64(registers, &node.op.args[0]),
                get_f64(registers, &node.op.args[1]),
            ) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.add_f64 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            };
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = fadd double {lhs}, {rhs}"));
            registers.insert(node.name.clone(), LlvmValueRef::F64(reg.clone()));
            let widened = fresh_reg(next_reg);
            body.push(format!("  {widened} = fptosi double {reg} to i64"));
            *last_cpu_value = Some(widened);
        }
        "sub" => {
            if let (Some(lhs), Some(rhs)) = (
                get_f64(registers, &node.op.args[0]),
                get_f64(registers, &node.op.args[1]),
            ) {
                let reg = fresh_reg(next_reg);
                body.push(format!("  {reg} = fsub double {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::F64(reg.clone()));
                let widened = fresh_reg(next_reg);
                body.push(format!("  {widened} = fptosi double {reg} to i64"));
                *last_cpu_value = Some(widened);
            } else if let (Some(lhs), Some(rhs)) = (
                get_f32(registers, &node.op.args[0]),
                get_f32(registers, &node.op.args[1]),
            ) {
                let reg = fresh_reg(next_reg);
                body.push(format!("  {reg} = fsub float {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::F32(reg.clone()));
                let widened = fresh_reg(next_reg);
                body.push(format!("  {widened} = fptosi float {reg} to i64"));
                *last_cpu_value = Some(widened);
            } else if let (Some(lhs), Some(rhs)) = (
                get_i64(registers, &node.op.args[0]),
                get_i64(registers, &node.op.args[1]),
            ) {
                let reg = fresh_reg(next_reg);
                body.push(format!("  {reg} = sub i64 {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                record_known_i64_binary_op(node, known_i64_values, i64::checked_sub);
                *last_cpu_value = Some(reg);
            } else if let (Some(lhs), Some(rhs)) = (
                get_i32(registers, &node.op.args[0]),
                get_i32(registers, &node.op.args[1]),
            ) {
                let reg = fresh_reg(next_reg);
                body.push(format!("  {reg} = sub i32 {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::I32(reg.clone()));
                let widened = fresh_reg(next_reg);
                body.push(format!("  {widened} = sext i32 {reg} to i64"));
                *last_cpu_value = Some(widened);
            } else {
                body.push(format!(
                        "  ; deferred lowering for cpu.sub `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            }
        }
        "sub_i32" => {
            let (Some(lhs), Some(rhs)) = (
                get_i32(registers, &node.op.args[0]),
                get_i32(registers, &node.op.args[1]),
            ) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.sub_i32 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            };
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = sub i32 {lhs}, {rhs}"));
            registers.insert(node.name.clone(), LlvmValueRef::I32(reg.clone()));
            let widened = fresh_reg(next_reg);
            body.push(format!("  {widened} = sext i32 {reg} to i64"));
            *last_cpu_value = Some(widened);
        }
        "sub_f32" => {
            let (Some(lhs), Some(rhs)) = (
                get_f32(registers, &node.op.args[0]),
                get_f32(registers, &node.op.args[1]),
            ) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.sub_f32 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            };
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = fsub float {lhs}, {rhs}"));
            registers.insert(node.name.clone(), LlvmValueRef::F32(reg.clone()));
            let widened = fresh_reg(next_reg);
            body.push(format!("  {widened} = fptosi float {reg} to i64"));
            *last_cpu_value = Some(widened);
        }
        "sub_f64" => {
            let (Some(lhs), Some(rhs)) = (
                get_f64(registers, &node.op.args[0]),
                get_f64(registers, &node.op.args[1]),
            ) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.sub_f64 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            };
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = fsub double {lhs}, {rhs}"));
            registers.insert(node.name.clone(), LlvmValueRef::F64(reg.clone()));
            let widened = fresh_reg(next_reg);
            body.push(format!("  {widened} = fptosi double {reg} to i64"));
            *last_cpu_value = Some(widened);
        }
        "mul" => {
            if let (Some(lhs), Some(rhs)) = (
                get_f64(registers, &node.op.args[0]),
                get_f64(registers, &node.op.args[1]),
            ) {
                let reg = fresh_reg(next_reg);
                body.push(format!("  {reg} = fmul double {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::F64(reg.clone()));
                let widened = fresh_reg(next_reg);
                body.push(format!("  {widened} = fptosi double {reg} to i64"));
                *last_cpu_value = Some(widened);
            } else if let (Some(lhs), Some(rhs)) = (
                get_f32(registers, &node.op.args[0]),
                get_f32(registers, &node.op.args[1]),
            ) {
                let reg = fresh_reg(next_reg);
                body.push(format!("  {reg} = fmul float {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::F32(reg.clone()));
                let widened = fresh_reg(next_reg);
                body.push(format!("  {widened} = fptosi float {reg} to i64"));
                *last_cpu_value = Some(widened);
            } else if let (Some(lhs), Some(rhs)) = (
                get_i64(registers, &node.op.args[0]),
                get_i64(registers, &node.op.args[1]),
            ) {
                let reg = fresh_reg(next_reg);
                body.push(format!("  {reg} = mul i64 {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                record_known_i64_binary_op(node, known_i64_values, i64::checked_mul);
                *last_cpu_value = Some(reg);
            } else if let (Some(lhs), Some(rhs)) = (
                get_i32(registers, &node.op.args[0]),
                get_i32(registers, &node.op.args[1]),
            ) {
                let reg = fresh_reg(next_reg);
                body.push(format!("  {reg} = mul i32 {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::I32(reg.clone()));
                let widened = fresh_reg(next_reg);
                body.push(format!("  {widened} = sext i32 {reg} to i64"));
                *last_cpu_value = Some(widened);
            } else {
                body.push(format!(
                        "  ; deferred lowering for cpu.mul `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            }
        }
        "mul_i32" => {
            let (Some(lhs), Some(rhs)) = (
                get_i32(registers, &node.op.args[0]),
                get_i32(registers, &node.op.args[1]),
            ) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.mul_i32 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            };
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = mul i32 {lhs}, {rhs}"));
            registers.insert(node.name.clone(), LlvmValueRef::I32(reg.clone()));
            let widened = fresh_reg(next_reg);
            body.push(format!("  {widened} = sext i32 {reg} to i64"));
            *last_cpu_value = Some(widened);
        }
        "mul_f32" => {
            let (Some(lhs), Some(rhs)) = (
                get_f32(registers, &node.op.args[0]),
                get_f32(registers, &node.op.args[1]),
            ) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.mul_f32 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            };
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = fmul float {lhs}, {rhs}"));
            registers.insert(node.name.clone(), LlvmValueRef::F32(reg.clone()));
            let widened = fresh_reg(next_reg);
            body.push(format!("  {widened} = fptosi float {reg} to i64"));
            *last_cpu_value = Some(widened);
        }
        "mul_f64" => {
            let (Some(lhs), Some(rhs)) = (
                get_f64(registers, &node.op.args[0]),
                get_f64(registers, &node.op.args[1]),
            ) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.mul_f64 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            };
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = fmul double {lhs}, {rhs}"));
            registers.insert(node.name.clone(), LlvmValueRef::F64(reg.clone()));
            let widened = fresh_reg(next_reg);
            body.push(format!("  {widened} = fptosi double {reg} to i64"));
            *last_cpu_value = Some(widened);
        }
        "div" => {
            if let (Some(lhs), Some(rhs)) = (
                get_f64(registers, &node.op.args[0]),
                get_f64(registers, &node.op.args[1]),
            ) {
                let reg = fresh_reg(next_reg);
                body.push(format!("  {reg} = fdiv double {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::F64(reg.clone()));
                let widened = fresh_reg(next_reg);
                body.push(format!("  {widened} = fptosi double {reg} to i64"));
                *last_cpu_value = Some(widened);
            } else if let (Some(lhs), Some(rhs)) = (
                get_f32(registers, &node.op.args[0]),
                get_f32(registers, &node.op.args[1]),
            ) {
                let reg = fresh_reg(next_reg);
                body.push(format!("  {reg} = fdiv float {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::F32(reg.clone()));
                let widened = fresh_reg(next_reg);
                body.push(format!("  {widened} = fptosi float {reg} to i64"));
                *last_cpu_value = Some(widened);
            } else if let (Some(lhs), Some(rhs)) = (
                get_i64(registers, &node.op.args[0]),
                get_i64(registers, &node.op.args[1]),
            ) {
                let reg = fresh_reg(next_reg);
                body.push(format!("  {reg} = sdiv i64 {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                *last_cpu_value = Some(reg);
            } else if let (Some(lhs), Some(rhs)) = (
                get_i32(registers, &node.op.args[0]),
                get_i32(registers, &node.op.args[1]),
            ) {
                let reg = fresh_reg(next_reg);
                body.push(format!("  {reg} = sdiv i32 {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::I32(reg.clone()));
                let widened = fresh_reg(next_reg);
                body.push(format!("  {widened} = sext i32 {reg} to i64"));
                *last_cpu_value = Some(widened);
            } else {
                body.push(format!(
                        "  ; deferred lowering for cpu.div `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            }
        }
        "div_i32" => {
            let (Some(lhs), Some(rhs)) = (
                get_i32(registers, &node.op.args[0]),
                get_i32(registers, &node.op.args[1]),
            ) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.div_i32 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            };
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = sdiv i32 {lhs}, {rhs}"));
            registers.insert(node.name.clone(), LlvmValueRef::I32(reg.clone()));
            let widened = fresh_reg(next_reg);
            body.push(format!("  {widened} = sext i32 {reg} to i64"));
            *last_cpu_value = Some(widened);
        }
        "div_f32" => {
            let (Some(lhs), Some(rhs)) = (
                get_f32(registers, &node.op.args[0]),
                get_f32(registers, &node.op.args[1]),
            ) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.div_f32 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            };
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = fdiv float {lhs}, {rhs}"));
            registers.insert(node.name.clone(), LlvmValueRef::F32(reg.clone()));
            let widened = fresh_reg(next_reg);
            body.push(format!("  {widened} = fptosi float {reg} to i64"));
            *last_cpu_value = Some(widened);
        }
        "div_f64" => {
            let (Some(lhs), Some(rhs)) = (
                get_f64(registers, &node.op.args[0]),
                get_f64(registers, &node.op.args[1]),
            ) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.div_f64 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            };
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = fdiv double {lhs}, {rhs}"));
            registers.insert(node.name.clone(), LlvmValueRef::F64(reg.clone()));
            let widened = fresh_reg(next_reg);
            body.push(format!("  {widened} = fptosi double {reg} to i64"));
            *last_cpu_value = Some(widened);
        }
        "rem" => {
            if let (Some(lhs), Some(rhs)) = (
                get_i64(registers, &node.op.args[0]),
                get_i64(registers, &node.op.args[1]),
            ) {
                let reg = fresh_reg(next_reg);
                body.push(format!("  {reg} = srem i64 {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                *last_cpu_value = Some(reg);
            } else if let (Some(lhs), Some(rhs)) = (
                get_i32(registers, &node.op.args[0]),
                get_i32(registers, &node.op.args[1]),
            ) {
                let reg = fresh_reg(next_reg);
                body.push(format!("  {reg} = srem i32 {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::I32(reg.clone()));
                let widened = fresh_reg(next_reg);
                body.push(format!("  {widened} = sext i32 {reg} to i64"));
                *last_cpu_value = Some(widened);
            } else {
                body.push(format!(
                        "  ; deferred lowering for cpu.rem `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            }
        }
        "madd" => {
            let (Some(lhs), Some(rhs), Some(acc)) = (
                get_i64(registers, &node.op.args[0]),
                get_i64(registers, &node.op.args[1]),
                get_i64(registers, &node.op.args[2]),
            ) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.madd `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            };
            let mul = fresh_reg(next_reg);
            body.push(format!("  {mul} = mul i64 {lhs}, {rhs}"));
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = add i64 {mul}, {acc}"));
            registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
            *last_cpu_value = Some(reg);
        }
        _ => return Ok(false),
    }

    Ok(true)
}

fn record_known_i64_binary_op(
    node: &Node,
    known_i64_values: &mut BTreeMap<String, i64>,
    op: impl FnOnce(i64, i64) -> Option<i64>,
) {
    let (Some(lhs), Some(rhs)) = (
        known_i64_values.get(&node.op.args[0]).copied(),
        known_i64_values.get(&node.op.args[1]).copied(),
    ) else {
        return;
    };
    if let Some(value) = op(lhs, rhs) {
        known_i64_values.insert(node.name.clone(), value);
    }
}
