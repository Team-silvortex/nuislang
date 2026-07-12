use std::collections::BTreeMap;

use yir_core::Node;

use super::{
    fresh_reg,
    value_ref::{coerce_to_i64, get_f32, get_f64, get_i32},
    KnownFacts, LlvmValueRef,
};

pub(crate) fn lower_cpu_scalar_equality_node(
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
        "eq" => {
            if let (Some(lhs), Some(rhs)) = (
                get_f64(registers, &node.op.args[0]),
                get_f64(registers, &node.op.args[1]),
            ) {
                let cmp = fresh_reg(next_reg);
                body.push(format!("  {cmp} = fcmp oeq double {lhs}, {rhs}"));
                let reg = fresh_reg(next_reg);
                body.push(format!("  {reg} = zext i1 {cmp} to i64"));
                registers.insert(
                    node.name.clone(),
                    LlvmValueRef::Bool {
                        i1: cmp.clone(),
                        i64: reg.clone(),
                    },
                );
                *last_cpu_value = Some(reg);
            } else if let (Some(lhs), Some(rhs)) = (
                get_f32(registers, &node.op.args[0]),
                get_f32(registers, &node.op.args[1]),
            ) {
                let cmp = fresh_reg(next_reg);
                body.push(format!("  {cmp} = fcmp oeq float {lhs}, {rhs}"));
                let reg = fresh_reg(next_reg);
                body.push(format!("  {reg} = zext i1 {cmp} to i64"));
                registers.insert(
                    node.name.clone(),
                    LlvmValueRef::Bool {
                        i1: cmp.clone(),
                        i64: reg.clone(),
                    },
                );
                *last_cpu_value = Some(reg);
            } else {
                let lhs_value = registers.get(&node.op.args[0]).cloned();
                let rhs_value = registers.get(&node.op.args[1]).cloned();
                if let (Some(lhs), Some(rhs)) = (
                    lhs_value
                        .as_ref()
                        .and_then(|value| coerce_to_i64(value, body, next_reg)),
                    rhs_value
                        .as_ref()
                        .and_then(|value| coerce_to_i64(value, body, next_reg)),
                ) {
                    let cmp = fresh_reg(next_reg);
                    body.push(format!("  {cmp} = icmp eq i64 {lhs}, {rhs}"));
                    let reg = fresh_reg(next_reg);
                    body.push(format!("  {reg} = zext i1 {cmp} to i64"));
                    registers.insert(
                        node.name.clone(),
                        LlvmValueRef::Bool {
                            i1: cmp.clone(),
                            i64: reg.clone(),
                        },
                    );
                    record_known_i64_equality(node, facts, |lhs, rhs| lhs == rhs);
                    *last_cpu_value = Some(reg);
                } else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.eq `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    return Ok(true);
                }
            }
        }
        "eq_i32" => {
            let (Some(lhs), Some(rhs)) = (
                get_i32(registers, &node.op.args[0]),
                get_i32(registers, &node.op.args[1]),
            ) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.eq_i32 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            };
            let cmp = fresh_reg(next_reg);
            body.push(format!("  {cmp} = icmp eq i32 {lhs}, {rhs}"));
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
        "eq_f32" => {
            let (Some(lhs), Some(rhs)) = (
                get_f32(registers, &node.op.args[0]),
                get_f32(registers, &node.op.args[1]),
            ) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.eq_f32 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            };
            let cmp = fresh_reg(next_reg);
            body.push(format!("  {cmp} = fcmp oeq float {lhs}, {rhs}"));
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
        "eq_f64" => {
            let (Some(lhs), Some(rhs)) = (
                get_f64(registers, &node.op.args[0]),
                get_f64(registers, &node.op.args[1]),
            ) else {
                body.push(format!(
                        "  ; deferred lowering for cpu.eq_f64 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                return Ok(true);
            };
            let cmp = fresh_reg(next_reg);
            body.push(format!("  {cmp} = fcmp oeq double {lhs}, {rhs}"));
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
        "ne" => {
            if let (Some(lhs), Some(rhs)) = (
                get_f64(registers, &node.op.args[0]),
                get_f64(registers, &node.op.args[1]),
            ) {
                let cmp = fresh_reg(next_reg);
                body.push(format!("  {cmp} = fcmp one double {lhs}, {rhs}"));
                let reg = fresh_reg(next_reg);
                body.push(format!("  {reg} = zext i1 {cmp} to i64"));
                registers.insert(
                    node.name.clone(),
                    LlvmValueRef::Bool {
                        i1: cmp.clone(),
                        i64: reg.clone(),
                    },
                );
                *last_cpu_value = Some(reg);
            } else if let (Some(lhs), Some(rhs)) = (
                get_f32(registers, &node.op.args[0]),
                get_f32(registers, &node.op.args[1]),
            ) {
                let cmp = fresh_reg(next_reg);
                body.push(format!("  {cmp} = fcmp one float {lhs}, {rhs}"));
                let reg = fresh_reg(next_reg);
                body.push(format!("  {reg} = zext i1 {cmp} to i64"));
                registers.insert(
                    node.name.clone(),
                    LlvmValueRef::Bool {
                        i1: cmp.clone(),
                        i64: reg.clone(),
                    },
                );
                *last_cpu_value = Some(reg);
            } else {
                let lhs_value = registers.get(&node.op.args[0]).cloned();
                let rhs_value = registers.get(&node.op.args[1]).cloned();
                if let (Some(lhs), Some(rhs)) = (
                    lhs_value
                        .as_ref()
                        .and_then(|value| coerce_to_i64(value, body, next_reg)),
                    rhs_value
                        .as_ref()
                        .and_then(|value| coerce_to_i64(value, body, next_reg)),
                ) {
                    let cmp = fresh_reg(next_reg);
                    body.push(format!("  {cmp} = icmp ne i64 {lhs}, {rhs}"));
                    let reg = fresh_reg(next_reg);
                    body.push(format!("  {reg} = zext i1 {cmp} to i64"));
                    registers.insert(
                        node.name.clone(),
                        LlvmValueRef::Bool {
                            i1: cmp.clone(),
                            i64: reg.clone(),
                        },
                    );
                    record_known_i64_equality(node, facts, |lhs, rhs| lhs != rhs);
                    *last_cpu_value = Some(reg);
                } else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.ne `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    return Ok(true);
                }
            }
        }
        _ => return Ok(false),
    }

    Ok(true)
}

fn record_known_i64_equality(
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
