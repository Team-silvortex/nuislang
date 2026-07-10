use std::collections::BTreeMap;

use yir_core::Node;

use super::{fresh_reg, value_ref::get_i64, LlvmValueRef};

pub(crate) fn lower_cpu_bitwise_node(
    node: &Node,
    body: &mut Vec<String>,
    registers: &mut BTreeMap<String, LlvmValueRef>,
    next_reg: &mut usize,
    last_cpu_value: &mut Option<String>,
) -> Result<bool, String> {
    if node.op.module != "cpu" {
        return Ok(false);
    }

    if node.op.instruction == "not" {
        let Some(input) = get_i64(registers, &node.op.args[0]) else {
            body.push(format!(
                "  ; deferred lowering for cpu.not `{}` because its input is outside the current CPU LLVM slice",
                node.name
            ));
            return Ok(true);
        };
        let reg = fresh_reg(next_reg);
        body.push(format!("  {reg} = xor i64 {input}, -1"));
        registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
        *last_cpu_value = Some(reg);
        return Ok(true);
    }

    let llvm_op = match node.op.instruction.as_str() {
        "and" => "and",
        "or" => "or",
        "xor" => "xor",
        "shl" => "shl",
        "shr" => "ashr",
        _ => return Ok(false),
    };

    let (Some(lhs), Some(rhs)) = (
        get_i64(registers, &node.op.args[0]),
        get_i64(registers, &node.op.args[1]),
    ) else {
        body.push(format!(
            "  ; deferred lowering for cpu.{} `{}` because one or more inputs are outside the current CPU LLVM slice",
            node.op.instruction, node.name
        ));
        return Ok(true);
    };
    let reg = fresh_reg(next_reg);
    body.push(format!("  {reg} = {llvm_op} i64 {lhs}, {rhs}"));
    registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
    *last_cpu_value = Some(reg);
    Ok(true)
}
