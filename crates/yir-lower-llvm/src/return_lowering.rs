use std::collections::BTreeMap;

use yir_core::Node;

use super::{
    fresh_reg,
    task_owned_payload::emit_flat_struct_data,
    value_ref::{coerce_to_i64, get_bool, get_f32, get_f64, get_i32, get_i64},
    LlvmValueRef,
};

pub(crate) enum ReturnLoweringOutcome {
    NotReturn,
    Deferred,
    Returned,
}

pub(crate) fn lower_cpu_return_node(
    node: &Node,
    body: &mut Vec<String>,
    registers: &BTreeMap<String, LlvmValueRef>,
    next_reg: &mut usize,
    last_cpu_value: &mut Option<String>,
) -> Result<ReturnLoweringOutcome, String> {
    if node.op.module != "cpu" {
        return Ok(ReturnLoweringOutcome::NotReturn);
    }

    match node.op.instruction.as_str() {
        "return_bool" => {
            let Some(input) = get_bool(registers, &node.op.args[0]) else {
                body.push(format!(
                    "  ; deferred lowering for cpu.return_bool `{}` because its input is outside the current CPU LLVM slice",
                    node.name
                ));
                return Ok(ReturnLoweringOutcome::Deferred);
            };
            if let Some(LlvmValueRef::Bool { i64, .. }) = registers.get(&node.op.args[0]) {
                *last_cpu_value = Some(i64.clone());
            }
            body.push(format!("  ret i1 {input}"));
        }
        "return_i32" => {
            let Some(input) = get_i32(registers, &node.op.args[0]) else {
                body.push(format!(
                    "  ; deferred lowering for cpu.return_i32 `{}` because its input is outside the current CPU LLVM slice",
                    node.name
                ));
                return Ok(ReturnLoweringOutcome::Deferred);
            };
            if let Some(as_i64) = coerce_to_i64(
                registers
                    .get(&node.op.args[0])
                    .expect("return_i32 source should exist"),
                body,
                next_reg,
            ) {
                *last_cpu_value = Some(as_i64);
            }
            body.push(format!("  ret i32 {input}"));
        }
        "return_i64" => {
            let Some(input) = get_i64(registers, &node.op.args[0]) else {
                body.push(format!(
                    "  ; deferred lowering for cpu.return_i64 `{}` because its input is outside the current CPU LLVM slice",
                    node.name
                ));
                return Ok(ReturnLoweringOutcome::Deferred);
            };
            body.push(format!("  ret i64 {input}"));
            *last_cpu_value = Some(input.to_owned());
        }
        "return_f32" => {
            let Some(input) = get_f32(registers, &node.op.args[0]) else {
                body.push(format!(
                    "  ; deferred lowering for cpu.return_f32 `{}` because its input is outside the current CPU LLVM slice",
                    node.name
                ));
                return Ok(ReturnLoweringOutcome::Deferred);
            };
            let widened = fresh_reg(next_reg);
            body.push(format!("  {widened} = fpext float {input} to double"));
            let as_i64 = fresh_reg(next_reg);
            body.push(format!("  {as_i64} = fptosi double {widened} to i64"));
            *last_cpu_value = Some(as_i64);
            body.push(format!("  ret float {input}"));
        }
        "return_f64" => {
            let Some(input) = get_f64(registers, &node.op.args[0]) else {
                body.push(format!(
                    "  ; deferred lowering for cpu.return_f64 `{}` because its input is outside the current CPU LLVM slice",
                    node.name
                ));
                return Ok(ReturnLoweringOutcome::Deferred);
            };
            let as_i64 = fresh_reg(next_reg);
            body.push(format!("  {as_i64} = fptosi double {input} to i64"));
            *last_cpu_value = Some(as_i64);
            body.push(format!("  ret double {input}"));
        }
        "return_owned_struct" => {
            let Some(LlvmValueRef::Struct(value)) = registers.get(&node.op.args[0]) else {
                body.push(format!(
                    "  ; deferred lowering for cpu.return_owned_struct `{}` because its input is not a flat struct",
                    node.name
                ));
                return Ok(ReturnLoweringOutcome::Deferred);
            };
            let mut state = super::LlvmLoweringState {
                body: std::mem::take(body),
                globals: Vec::new(),
                registers: BTreeMap::new(),
                delayed_registers: BTreeMap::new(),
                facts: super::KnownFacts::new(),
                buffer_lengths: BTreeMap::new(),
                next_reg: *next_reg,
                next_global: 0,
                next_block: 0,
                last_cpu_value: None,
                ends_with_terminal_return: false,
            };
            let Some(data) = emit_flat_struct_data(value, &mut state) else {
                *body = state.body;
                return Ok(ReturnLoweringOutcome::Deferred);
            };
            let pointer_bits = fresh_reg(&mut state.next_reg);
            state
                .body
                .push(format!("  {pointer_bits} = ptrtoint ptr {data} to i64"));
            state.body.push(format!("  ret i64 {pointer_bits}"));
            *body = state.body;
            *next_reg = state.next_reg;
            *last_cpu_value = Some(pointer_bits);
        }
        _ => return Ok(ReturnLoweringOutcome::NotReturn),
    }

    Ok(ReturnLoweringOutcome::Returned)
}
