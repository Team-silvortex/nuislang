use std::collections::BTreeMap;

use yir_core::Node;

use super::{fresh_reg, value_ref::coerce_to_i64, variant_select::emit_select_value, LlvmValueRef};

pub(crate) fn lower_cpu_select_node(
    node: &Node,
    body: &mut Vec<String>,
    registers: &mut BTreeMap<String, LlvmValueRef>,
    delayed_registers: &mut BTreeMap<String, String>,
    known_bool_values: &BTreeMap<String, bool>,
    next_reg: &mut usize,
    last_cpu_value: &mut Option<String>,
) -> Result<bool, String> {
    if node.op.module != "cpu" || node.op.instruction != "select" {
        return Ok(false);
    }

    let cond_value = registers.get(&node.op.args[0]).cloned();
    let then_value = registers.get(&node.op.args[1]).cloned();
    let else_value = registers.get(&node.op.args[2]).cloned();
    let then_delayed = delayed_registers.get(&node.op.args[1]).cloned();
    let else_delayed = delayed_registers.get(&node.op.args[2]).cloned();
    if then_delayed.is_some() || else_delayed.is_some() {
        return lower_lazy_const_select(
            node,
            body,
            registers,
            delayed_registers,
            cond_value,
            then_value,
            else_value,
            then_delayed,
            else_delayed,
            known_bool_values,
            next_reg,
            last_cpu_value,
        );
    }
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

#[allow(clippy::too_many_arguments)]
fn lower_lazy_const_select(
    node: &Node,
    body: &mut Vec<String>,
    registers: &mut BTreeMap<String, LlvmValueRef>,
    delayed_registers: &mut BTreeMap<String, String>,
    cond_value: Option<LlvmValueRef>,
    then_value: Option<LlvmValueRef>,
    else_value: Option<LlvmValueRef>,
    then_delayed: Option<String>,
    else_delayed: Option<String>,
    known_bool_values: &BTreeMap<String, bool>,
    next_reg: &mut usize,
    last_cpu_value: &mut Option<String>,
) -> Result<bool, String> {
    let Some(cond_value) = cond_value else {
        body.push(format!(
            "  ; deferred lowering for cpu.select `{}` because its condition is outside the current CPU LLVM slice",
            node.name
        ));
        return Ok(true);
    };
    let Some(cond) = const_select_condition(&node.op.args[0], &cond_value, known_bool_values)
    else {
        let delayed_branches =
            delayed_select_branches(node, then_delayed.as_deref(), else_delayed.as_deref());
        body.push(format!(
            "  ; deferred lowering for cpu.select `{}` because delayed branch lowering requires a compile-time constant condition ({delayed_branches})",
            node.name
        ));
        return Ok(true);
    };
    let (selected_name, selected_value, selected_delayed, unselected_name) = if cond {
        (
            node.op.args[1].as_str(),
            then_value,
            then_delayed,
            node.op.args[2].as_str(),
        )
    } else {
        (
            node.op.args[2].as_str(),
            else_value,
            else_delayed,
            node.op.args[1].as_str(),
        )
    };
    if let Some(reason) = selected_delayed {
        body.push(format!(
            "  ; deferred lowering for cpu.select `{}` because selected branch `{selected_name}` is delayed: {reason}",
            node.name
        ));
        return Ok(true);
    }
    let Some(selected) = selected_value else {
        body.push(format!(
            "  ; deferred lowering for cpu.select `{}` because selected branch `{selected_name}` is outside the current CPU LLVM slice",
            node.name
        ));
        return Ok(true);
    };
    delayed_registers.remove(unselected_name);
    registers.insert(node.name.clone(), selected.clone());
    if let Some(as_i64) = coerce_to_i64(&selected, body, next_reg) {
        *last_cpu_value = Some(as_i64);
    }
    Ok(true)
}

fn const_select_condition(
    cond_name: &str,
    value: &LlvmValueRef,
    known_bool_values: &BTreeMap<String, bool>,
) -> Option<bool> {
    if let Some(value) = known_bool_values.get(cond_name) {
        return Some(*value);
    }
    match value {
        LlvmValueRef::Bool { i1, .. } if i1 == "true" => Some(true),
        LlvmValueRef::Bool { i1, .. } if i1 == "false" => Some(false),
        LlvmValueRef::I64(value) if value == "0" => Some(false),
        LlvmValueRef::I64(_) => None,
        _ => None,
    }
}

fn delayed_select_branches(
    node: &Node,
    then_delayed: Option<&str>,
    else_delayed: Option<&str>,
) -> String {
    let mut branches = Vec::new();
    if let Some(reason) = then_delayed {
        branches.push(format!("then `{}`: {reason}", node.op.args[1]));
    }
    if let Some(reason) = else_delayed {
        branches.push(format!("else `{}`: {reason}", node.op.args[2]));
    }
    branches.join("; ")
}
