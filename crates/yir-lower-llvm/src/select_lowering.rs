use std::collections::BTreeMap;

use yir_core::Node;

use super::{
    fresh_reg, value_ref::coerce_to_i64, variant_select::emit_select_value, KnownFacts,
    LlvmValueRef,
};

pub(crate) fn lower_cpu_select_node(
    node: &Node,
    body: &mut Vec<String>,
    registers: &mut BTreeMap<String, LlvmValueRef>,
    delayed_registers: &mut BTreeMap<String, String>,
    facts: &mut KnownFacts,
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
            facts,
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

    let selected = match emit_select_value(&cond_bool, &then_value, &else_value, body, next_reg) {
        Some(selected) => selected,
        None => {
            if let Some(selected_name) =
                const_select_condition(&node.op.args[0], &cond_value, facts).map(|condition| {
                    if condition {
                        node.op.args[1].as_str()
                    } else {
                        node.op.args[2].as_str()
                    }
                })
            {
                let selected = if selected_name == node.op.args[1] {
                    then_value
                } else {
                    else_value
                };
                registers.insert(node.name.clone(), selected.clone());
                record_known_selected_branch(node, selected_name, &selected, facts);
                if let Some(as_i64) = coerce_to_i64(&selected, body, next_reg) {
                    *last_cpu_value = Some(as_i64);
                }
                return Ok(true);
            }
            body.push(format!(
                "  ; deferred lowering for cpu.select `{}` because its branch values are not select-compatible in the current CPU LLVM slice",
                node.name
            ));
            return Ok(true);
        }
    };
    registers.insert(node.name.clone(), selected.clone());
    record_known_select_value(node, &cond_value, &then_value, &else_value, facts);
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
    facts: &mut KnownFacts,
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
    let Some(cond) = const_select_condition(&node.op.args[0], &cond_value, facts) else {
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
    record_known_selected_branch(node, selected_name, &selected, facts);
    if let Some(as_i64) = coerce_to_i64(&selected, body, next_reg) {
        *last_cpu_value = Some(as_i64);
    }
    Ok(true)
}

fn const_select_condition(
    cond_name: &str,
    value: &LlvmValueRef,
    facts: &KnownFacts,
) -> Option<bool> {
    if let Some(value) = facts.get_bool(cond_name) {
        return Some(value);
    }
    if let Some(value) = facts.get_i64(cond_name) {
        return Some(value != 0);
    }
    match value {
        LlvmValueRef::Bool { i1, .. } if i1 == "true" => Some(true),
        LlvmValueRef::Bool { i1, .. } if i1 == "false" => Some(false),
        LlvmValueRef::I64(value) => value.parse::<i64>().ok().map(|value| value != 0),
        _ => None,
    }
}

fn record_known_select_value(
    node: &Node,
    cond_value: &LlvmValueRef,
    then_value: &LlvmValueRef,
    else_value: &LlvmValueRef,
    facts: &mut KnownFacts,
) {
    let cond = const_select_condition(&node.op.args[0], cond_value, facts);
    if let Some(selected_name) = cond.map(|value| {
        if value {
            node.op.args[1].as_str()
        } else {
            node.op.args[2].as_str()
        }
    }) {
        let selected_value = if selected_name == node.op.args[1] {
            then_value
        } else {
            else_value
        };
        record_known_selected_branch(node, selected_name, selected_value, facts);
        return;
    }

    if let (Some(then_value), Some(else_value)) = (
        facts.get_i64(&node.op.args[1]),
        facts.get_i64(&node.op.args[2]),
    ) {
        if then_value == else_value {
            facts.record_i64(node.name.clone(), then_value);
        }
    }

    if let (Some(then_value), Some(else_value)) = (
        facts.get_bool(&node.op.args[1]),
        facts.get_bool(&node.op.args[2]),
    ) {
        if then_value == else_value {
            facts.record_bool(node.name.clone(), then_value);
        }
    }

    if let (LlvmValueRef::I64(then_i64), LlvmValueRef::I64(else_i64)) = (then_value, else_value) {
        if let (Ok(then_value), Ok(else_value)) = (then_i64.parse::<i64>(), else_i64.parse::<i64>())
        {
            if then_value == else_value {
                facts.record_i64(node.name.clone(), then_value);
            }
        }
    }
}

fn record_known_selected_branch(
    node: &Node,
    selected_name: &str,
    selected_value: &LlvmValueRef,
    facts: &mut KnownFacts,
) {
    if let Some(value) = facts.get_i64(selected_name) {
        facts.record_i64(node.name.clone(), value);
    }
    if let Some(value) = facts.get_bool(selected_name) {
        facts.record_bool(node.name.clone(), value);
    }
    if let Some(value) = facts.get_variant_type(selected_name).map(str::to_owned) {
        facts.record_variant_type(node.name.clone(), value);
    }
    if let LlvmValueRef::Struct(struct_value) = selected_value {
        for (field_name, _) in &struct_value.fields {
            let from = KnownFacts::struct_field_key(selected_name, field_name);
            let to = KnownFacts::struct_field_key(&node.name, field_name);
            if let Some(value) = facts.get_i64(&from) {
                facts.record_i64(to.clone(), value);
            }
            if let Some(value) = facts.get_bool(&from) {
                facts.record_bool(to.clone(), value);
            }
            if let Some(value) = facts.get_variant_type(&from).map(str::to_owned) {
                facts.record_variant_type(to.clone(), value);
            }
        }
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
