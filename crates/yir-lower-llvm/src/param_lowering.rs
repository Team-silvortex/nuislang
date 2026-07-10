use std::collections::BTreeMap;

use yir_core::Node;

use super::{value_ref::coerce_to_i64, LlvmValueRef};

pub(crate) fn lower_cpu_param_node(
    node: &Node,
    body: &mut Vec<String>,
    registers: &BTreeMap<String, LlvmValueRef>,
    next_reg: &mut usize,
    last_cpu_value: &mut Option<String>,
) -> Result<bool, String> {
    if node.op.module != "cpu" {
        return Ok(false);
    }
    if !matches!(
        node.op.instruction.as_str(),
        "param_bool" | "param_i32" | "param_i64"
    ) {
        return Ok(false);
    }

    if let Some(input) = coerce_to_i64(
        registers
            .get(&node.name)
            .expect("parameter binding should exist"),
        body,
        next_reg,
    ) {
        *last_cpu_value = Some(input);
    }
    Ok(true)
}
