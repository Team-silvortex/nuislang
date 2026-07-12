use std::collections::BTreeMap;

use yir_core::Node;

use super::{
    fresh_reg,
    value_ref::{get_f32, get_f64, get_i32, get_i64},
    KnownFacts, LlvmValueRef,
};

pub(crate) fn lower_cpu_scalar_order_node(
    node: &Node,
    body: &mut Vec<String>,
    registers: &mut BTreeMap<String, LlvmValueRef>,
    facts: &mut KnownFacts,
    next_reg: &mut usize,
    last_cpu_value: &mut Option<String>,
) -> Result<bool, String> {
    if node.op.module != "cpu" {
        return Ok(false);
    }

    match node.op.instruction.as_str() {
        "lt" => {
            if let (Some(lhs), Some(rhs)) = (
                get_f64(registers, &node.op.args[0]),
                get_f64(registers, &node.op.args[1]),
            ) {
                let cmp = fresh_reg(next_reg);
                body.push(format!("  {cmp} = fcmp olt double {lhs}, {rhs}"));
                let widened = fresh_reg(next_reg);
                body.push(format!("  {widened} = zext i1 {cmp} to i64"));
                registers.insert(
                    node.name.clone(),
                    LlvmValueRef::Bool {
                        i1: cmp.clone(),
                        i64: widened.clone(),
                    },
                );
                *last_cpu_value = Some(widened);
            } else if let (Some(lhs), Some(rhs)) = (
                get_f32(registers, &node.op.args[0]),
                get_f32(registers, &node.op.args[1]),
            ) {
                let cmp = fresh_reg(next_reg);
                body.push(format!("  {cmp} = fcmp olt float {lhs}, {rhs}"));
                let widened = fresh_reg(next_reg);
                body.push(format!("  {widened} = zext i1 {cmp} to i64"));
                registers.insert(
                    node.name.clone(),
                    LlvmValueRef::Bool {
                        i1: cmp.clone(),
                        i64: widened.clone(),
                    },
                );
                *last_cpu_value = Some(widened);
            } else if let (Some(lhs), Some(rhs)) = (
                get_i64(registers, &node.op.args[0]),
                get_i64(registers, &node.op.args[1]),
            ) {
                let cmp = fresh_reg(next_reg);
                body.push(format!("  {cmp} = icmp slt i64 {lhs}, {rhs}"));
                let reg = fresh_reg(next_reg);
                body.push(format!("  {reg} = zext i1 {cmp} to i64"));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                record_known_i64_comparison(node, facts, |lhs, rhs| lhs < rhs);
                *last_cpu_value = Some(reg);
            } else if let (Some(lhs), Some(rhs)) = (
                get_i32(registers, &node.op.args[0]),
                get_i32(registers, &node.op.args[1]),
            ) {
                let cmp = fresh_reg(next_reg);
                body.push(format!("  {cmp} = icmp slt i32 {lhs}, {rhs}"));
                let widened = fresh_reg(next_reg);
                body.push(format!("  {widened} = zext i1 {cmp} to i64"));
                registers.insert(
                    node.name.clone(),
                    LlvmValueRef::Bool {
                        i1: cmp.clone(),
                        i64: widened.clone(),
                    },
                );
                *last_cpu_value = Some(widened);
            } else {
                body.push(format!(
                        "  ; deferred lowering for cpu.lt `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            }
        }
        "lt_i32" => {
            let (Some(lhs), Some(rhs)) = (
                get_i32(registers, &node.op.args[0]),
                get_i32(registers, &node.op.args[1]),
            ) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.lt_i32 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            };
            let cmp = fresh_reg(next_reg);
            body.push(format!("  {cmp} = icmp slt i32 {lhs}, {rhs}"));
            let widened = fresh_reg(next_reg);
            body.push(format!("  {widened} = zext i1 {cmp} to i64"));
            registers.insert(
                node.name.clone(),
                LlvmValueRef::Bool {
                    i1: cmp.clone(),
                    i64: widened.clone(),
                },
            );
            *last_cpu_value = Some(widened);
        }
        "lt_f32" => {
            let (Some(lhs), Some(rhs)) = (
                get_f32(registers, &node.op.args[0]),
                get_f32(registers, &node.op.args[1]),
            ) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.lt_f32 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            };
            let cmp = fresh_reg(next_reg);
            body.push(format!("  {cmp} = fcmp olt float {lhs}, {rhs}"));
            let widened = fresh_reg(next_reg);
            body.push(format!("  {widened} = zext i1 {cmp} to i64"));
            registers.insert(
                node.name.clone(),
                LlvmValueRef::Bool {
                    i1: cmp.clone(),
                    i64: widened.clone(),
                },
            );
            *last_cpu_value = Some(widened);
        }
        "lt_f64" => {
            let (Some(lhs), Some(rhs)) = (
                get_f64(registers, &node.op.args[0]),
                get_f64(registers, &node.op.args[1]),
            ) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.lt_f64 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            };
            let cmp = fresh_reg(next_reg);
            body.push(format!("  {cmp} = fcmp olt double {lhs}, {rhs}"));
            let widened = fresh_reg(next_reg);
            body.push(format!("  {widened} = zext i1 {cmp} to i64"));
            registers.insert(
                node.name.clone(),
                LlvmValueRef::Bool {
                    i1: cmp.clone(),
                    i64: widened.clone(),
                },
            );
            *last_cpu_value = Some(widened);
        }
        "gt" => {
            if let (Some(lhs), Some(rhs)) = (
                get_f64(registers, &node.op.args[0]),
                get_f64(registers, &node.op.args[1]),
            ) {
                let cmp = fresh_reg(next_reg);
                body.push(format!("  {cmp} = fcmp ogt double {lhs}, {rhs}"));
                let widened = fresh_reg(next_reg);
                body.push(format!("  {widened} = zext i1 {cmp} to i64"));
                registers.insert(
                    node.name.clone(),
                    LlvmValueRef::Bool {
                        i1: cmp.clone(),
                        i64: widened.clone(),
                    },
                );
                *last_cpu_value = Some(widened);
            } else if let (Some(lhs), Some(rhs)) = (
                get_f32(registers, &node.op.args[0]),
                get_f32(registers, &node.op.args[1]),
            ) {
                let cmp = fresh_reg(next_reg);
                body.push(format!("  {cmp} = fcmp ogt float {lhs}, {rhs}"));
                let widened = fresh_reg(next_reg);
                body.push(format!("  {widened} = zext i1 {cmp} to i64"));
                registers.insert(
                    node.name.clone(),
                    LlvmValueRef::Bool {
                        i1: cmp.clone(),
                        i64: widened.clone(),
                    },
                );
                *last_cpu_value = Some(widened);
            } else if let (Some(lhs), Some(rhs)) = (
                get_i64(registers, &node.op.args[0]),
                get_i64(registers, &node.op.args[1]),
            ) {
                let cmp = fresh_reg(next_reg);
                body.push(format!("  {cmp} = icmp sgt i64 {lhs}, {rhs}"));
                let reg = fresh_reg(next_reg);
                body.push(format!("  {reg} = zext i1 {cmp} to i64"));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                record_known_i64_comparison(node, facts, |lhs, rhs| lhs > rhs);
                *last_cpu_value = Some(reg);
            } else if let (Some(lhs), Some(rhs)) = (
                get_i32(registers, &node.op.args[0]),
                get_i32(registers, &node.op.args[1]),
            ) {
                let cmp = fresh_reg(next_reg);
                body.push(format!("  {cmp} = icmp sgt i32 {lhs}, {rhs}"));
                let widened = fresh_reg(next_reg);
                body.push(format!("  {widened} = zext i1 {cmp} to i64"));
                registers.insert(
                    node.name.clone(),
                    LlvmValueRef::Bool {
                        i1: cmp.clone(),
                        i64: widened.clone(),
                    },
                );
                *last_cpu_value = Some(widened);
            } else {
                body.push(format!(
                        "  ; deferred lowering for cpu.gt `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            }
        }
        "gt_i32" => {
            let (Some(lhs), Some(rhs)) = (
                get_i32(registers, &node.op.args[0]),
                get_i32(registers, &node.op.args[1]),
            ) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.gt_i32 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            };
            let cmp = fresh_reg(next_reg);
            body.push(format!("  {cmp} = icmp sgt i32 {lhs}, {rhs}"));
            let widened = fresh_reg(next_reg);
            body.push(format!("  {widened} = zext i1 {cmp} to i64"));
            registers.insert(
                node.name.clone(),
                LlvmValueRef::Bool {
                    i1: cmp.clone(),
                    i64: widened.clone(),
                },
            );
            *last_cpu_value = Some(widened);
        }
        "gt_f32" => {
            let (Some(lhs), Some(rhs)) = (
                get_f32(registers, &node.op.args[0]),
                get_f32(registers, &node.op.args[1]),
            ) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.gt_f32 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            };
            let cmp = fresh_reg(next_reg);
            body.push(format!("  {cmp} = fcmp ogt float {lhs}, {rhs}"));
            let widened = fresh_reg(next_reg);
            body.push(format!("  {widened} = zext i1 {cmp} to i64"));
            registers.insert(
                node.name.clone(),
                LlvmValueRef::Bool {
                    i1: cmp.clone(),
                    i64: widened.clone(),
                },
            );
            *last_cpu_value = Some(widened);
        }
        "gt_f64" => {
            let (Some(lhs), Some(rhs)) = (
                get_f64(registers, &node.op.args[0]),
                get_f64(registers, &node.op.args[1]),
            ) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.gt_f64 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            };
            let cmp = fresh_reg(next_reg);
            body.push(format!("  {cmp} = fcmp ogt double {lhs}, {rhs}"));
            let widened = fresh_reg(next_reg);
            body.push(format!("  {widened} = zext i1 {cmp} to i64"));
            registers.insert(
                node.name.clone(),
                LlvmValueRef::Bool {
                    i1: cmp.clone(),
                    i64: widened.clone(),
                },
            );
            *last_cpu_value = Some(widened);
        }
        "le" => {
            if let (Some(lhs), Some(rhs)) = (
                get_f64(registers, &node.op.args[0]),
                get_f64(registers, &node.op.args[1]),
            ) {
                let cmp = fresh_reg(next_reg);
                body.push(format!("  {cmp} = fcmp ole double {lhs}, {rhs}"));
                let widened = fresh_reg(next_reg);
                body.push(format!("  {widened} = zext i1 {cmp} to i64"));
                registers.insert(
                    node.name.clone(),
                    LlvmValueRef::Bool {
                        i1: cmp.clone(),
                        i64: widened.clone(),
                    },
                );
                *last_cpu_value = Some(widened);
            } else if let (Some(lhs), Some(rhs)) = (
                get_f32(registers, &node.op.args[0]),
                get_f32(registers, &node.op.args[1]),
            ) {
                let cmp = fresh_reg(next_reg);
                body.push(format!("  {cmp} = fcmp ole float {lhs}, {rhs}"));
                let widened = fresh_reg(next_reg);
                body.push(format!("  {widened} = zext i1 {cmp} to i64"));
                registers.insert(
                    node.name.clone(),
                    LlvmValueRef::Bool {
                        i1: cmp.clone(),
                        i64: widened.clone(),
                    },
                );
                *last_cpu_value = Some(widened);
            } else if let (Some(lhs), Some(rhs)) = (
                get_i64(registers, &node.op.args[0]),
                get_i64(registers, &node.op.args[1]),
            ) {
                let cmp = fresh_reg(next_reg);
                body.push(format!("  {cmp} = icmp sle i64 {lhs}, {rhs}"));
                let reg = fresh_reg(next_reg);
                body.push(format!("  {reg} = zext i1 {cmp} to i64"));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                record_known_i64_comparison(node, facts, |lhs, rhs| lhs <= rhs);
                *last_cpu_value = Some(reg);
            } else if let (Some(lhs), Some(rhs)) = (
                get_i32(registers, &node.op.args[0]),
                get_i32(registers, &node.op.args[1]),
            ) {
                let cmp = fresh_reg(next_reg);
                body.push(format!("  {cmp} = icmp sle i32 {lhs}, {rhs}"));
                let widened = fresh_reg(next_reg);
                body.push(format!("  {widened} = zext i1 {cmp} to i64"));
                registers.insert(
                    node.name.clone(),
                    LlvmValueRef::Bool {
                        i1: cmp.clone(),
                        i64: widened.clone(),
                    },
                );
                *last_cpu_value = Some(widened);
            } else {
                body.push(format!(
                        "  ; deferred lowering for cpu.le `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            }
        }
        "ge" => {
            if let (Some(lhs), Some(rhs)) = (
                get_f64(registers, &node.op.args[0]),
                get_f64(registers, &node.op.args[1]),
            ) {
                let cmp = fresh_reg(next_reg);
                body.push(format!("  {cmp} = fcmp oge double {lhs}, {rhs}"));
                let widened = fresh_reg(next_reg);
                body.push(format!("  {widened} = zext i1 {cmp} to i64"));
                registers.insert(
                    node.name.clone(),
                    LlvmValueRef::Bool {
                        i1: cmp.clone(),
                        i64: widened.clone(),
                    },
                );
                *last_cpu_value = Some(widened);
            } else if let (Some(lhs), Some(rhs)) = (
                get_f32(registers, &node.op.args[0]),
                get_f32(registers, &node.op.args[1]),
            ) {
                let cmp = fresh_reg(next_reg);
                body.push(format!("  {cmp} = fcmp oge float {lhs}, {rhs}"));
                let widened = fresh_reg(next_reg);
                body.push(format!("  {widened} = zext i1 {cmp} to i64"));
                registers.insert(
                    node.name.clone(),
                    LlvmValueRef::Bool {
                        i1: cmp.clone(),
                        i64: widened.clone(),
                    },
                );
                *last_cpu_value = Some(widened);
            } else if let (Some(lhs), Some(rhs)) = (
                get_i64(registers, &node.op.args[0]),
                get_i64(registers, &node.op.args[1]),
            ) {
                let cmp = fresh_reg(next_reg);
                body.push(format!("  {cmp} = icmp sge i64 {lhs}, {rhs}"));
                let reg = fresh_reg(next_reg);
                body.push(format!("  {reg} = zext i1 {cmp} to i64"));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                record_known_i64_comparison(node, facts, |lhs, rhs| lhs >= rhs);
                *last_cpu_value = Some(reg);
            } else if let (Some(lhs), Some(rhs)) = (
                get_i32(registers, &node.op.args[0]),
                get_i32(registers, &node.op.args[1]),
            ) {
                let cmp = fresh_reg(next_reg);
                body.push(format!("  {cmp} = icmp sge i32 {lhs}, {rhs}"));
                let widened = fresh_reg(next_reg);
                body.push(format!("  {widened} = zext i1 {cmp} to i64"));
                registers.insert(
                    node.name.clone(),
                    LlvmValueRef::Bool {
                        i1: cmp.clone(),
                        i64: widened.clone(),
                    },
                );
                *last_cpu_value = Some(widened);
            } else {
                body.push(format!(
                        "  ; deferred lowering for cpu.ge `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            }
        }
        _ => return Ok(false),
    }

    Ok(true)
}

fn record_known_i64_comparison(
    node: &Node,
    facts: &mut KnownFacts,
    compare: impl FnOnce(i64, i64) -> bool,
) {
    let (Some(lhs), Some(rhs)) = (
        facts.get_i64(&node.op.args[0]),
        facts.get_i64(&node.op.args[1]),
    ) else {
        return;
    };
    facts.record_bool(node.name.clone(), compare(lhs, rhs));
}
