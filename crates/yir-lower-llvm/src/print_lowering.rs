use std::collections::BTreeMap;

use yir_core::Node;

use super::{
    fresh_reg,
    value_ref::{get_bool, get_cstr, get_f32, get_f64, get_i32, get_i64},
    LlvmValueRef,
};

pub(crate) fn lower_cpu_print_node(
    node: &Node,
    body: &mut Vec<String>,
    registers: &BTreeMap<String, LlvmValueRef>,
    next_reg: &mut usize,
    last_cpu_value: &mut Option<String>,
) -> Result<bool, String> {
    if node.op.module != "cpu" || node.op.instruction != "print" {
        return Ok(false);
    }

    if let Some(input) = get_cstr(registers, &node.op.args[0]) {
        let print_reg = fresh_reg(next_reg);
        body.push(format!("  {print_reg} = call i32 @puts(ptr {input})"));
        *last_cpu_value = Some("0".to_owned());
    } else if let Some(input) = get_i64(registers, &node.op.args[0]) {
        body.push(format!("  call void @nuis_debug_print_i64(i64 {input})"));
        *last_cpu_value = Some(input.to_owned());
    } else if let Some(input) = get_i32(registers, &node.op.args[0]) {
        body.push(format!("  call void @nuis_debug_print_i32(i32 {input})"));
        let widened = fresh_reg(next_reg);
        body.push(format!("  {widened} = sext i32 {input} to i64"));
        *last_cpu_value = Some(widened);
    } else if let Some(input) = get_bool(registers, &node.op.args[0]) {
        let widened = fresh_reg(next_reg);
        body.push(format!("  {widened} = zext i1 {input} to i32"));
        body.push(format!("  call void @nuis_debug_print_bool(i32 {widened})"));
        let widened64 = fresh_reg(next_reg);
        body.push(format!("  {widened64} = zext i1 {input} to i64"));
        *last_cpu_value = Some(widened64);
    } else if let Some(input) = get_f32(registers, &node.op.args[0]) {
        body.push(format!("  call void @nuis_debug_print_f32(float {input})"));
        let widened = fresh_reg(next_reg);
        body.push(format!("  {widened} = fptosi float {input} to i64"));
        *last_cpu_value = Some(widened);
    } else if let Some(input) = get_f64(registers, &node.op.args[0]) {
        body.push(format!("  call void @nuis_debug_print_f64(double {input})"));
        let widened = fresh_reg(next_reg);
        body.push(format!("  {widened} = fptosi double {input} to i64"));
        *last_cpu_value = Some(widened);
    } else {
        body.push(format!(
            "  ; deferred lowering for cpu.print `{}` because its input is produced outside the current CPU LLVM slice",
            node.op.args[0]
        ));
    }

    Ok(true)
}
