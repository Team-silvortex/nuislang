use std::collections::BTreeMap;

use yir_core::Node;

use super::{fresh_reg, value_ref::coerce_to_i64, variant_select::emit_select_value, LlvmValueRef};

pub(crate) fn lower_cpu_select_node(
    node: &Node,
    body: &mut Vec<String>,
    registers: &mut BTreeMap<String, LlvmValueRef>,
    next_reg: &mut usize,
    last_cpu_value: &mut Option<String>,
) -> Result<bool, String> {
    if node.op.module != "cpu" || node.op.instruction != "select" {
        return Ok(false);
    }

    let cond_value = registers.get(&node.op.args[0]).cloned();
    let then_value = registers.get(&node.op.args[1]).cloned();
    let else_value = registers.get(&node.op.args[2]).cloned();
    let (Some(cond_value), Some(then_value), Some(else_value)) =
        (cond_value, then_value, else_value)
    else {
        body.push(format!(
            "  ; deferred lowering for cpu.select `{}` because one or more inputs are outside the current CPU LLVM slice",
            node.name
        ));
        return Ok(true);
    };

    let Some(cond) = coerce_to_i64(&cond_value, body, next_reg) else {
        body.push(format!(
            "  ; deferred lowering for cpu.select `{}` because its condition is not coercible to i64",
            node.name
        ));
        return Ok(true);
    };
    let cond_bool = fresh_reg(next_reg);
    body.push(format!("  {cond_bool} = icmp ne i64 {cond}, 0"));

    let Some(selected) = emit_select_value(&cond_bool, &then_value, &else_value, body, next_reg)
    else {
        body.push(format!(
            "  ; deferred lowering for cpu.select `{}` because its branch values are not select-compatible in the current CPU LLVM slice",
            node.name
        ));
        return Ok(true);
    };
    registers.insert(node.name.clone(), selected.clone());
    if let Some(as_i64) = coerce_to_i64(&selected, body, next_reg) {
        *last_cpu_value = Some(as_i64);
    }
    Ok(true)
}
