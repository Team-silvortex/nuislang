use std::collections::BTreeMap;

use yir_core::Node;

use super::{
    call_return::cpu_scalar_kind_llvm_type,
    fresh_reg,
    value_ref::{get_bool, get_f32, get_f64, get_i32, get_i64},
    CpuCallScalarKind, CpuHelperSignature, LlvmValueRef,
};

pub(crate) fn lower_cpu_call_node(
    node: &Node,
    body: &mut Vec<String>,
    registers: &mut BTreeMap<String, LlvmValueRef>,
    helper_signatures: &BTreeMap<String, CpuHelperSignature>,
    next_reg: &mut usize,
    last_cpu_value: &mut Option<String>,
) -> Result<bool, String> {
    if node.op.module != "cpu" {
        return Ok(false);
    }
    if !matches!(
        node.op.instruction.as_str(),
        "call_bool" | "call_i32" | "call_i64" | "call_f32" | "call_f64"
    ) {
        return Ok(false);
    }

    let callee = &node.op.args[0];
    let Some(signature) = helper_signatures.get(callee) else {
        body.push(format!(
            "  ; deferred lowering for cpu.{} `{}` because helper signature `{}` is unavailable",
            node.op.instruction, node.name, callee
        ));
        return Ok(true);
    };

    let lowered_args = node.op.args[1..]
        .iter()
        .zip(signature.params.iter())
        .map(|(arg, kind)| match kind {
            CpuCallScalarKind::Bool => get_bool(registers, arg).map(|value| format!("i1 {value}")),
            CpuCallScalarKind::I32 => get_i32(registers, arg).map(|value| format!("i32 {value}")),
            CpuCallScalarKind::I64 => get_i64(registers, arg).map(|value| format!("i64 {value}")),
            CpuCallScalarKind::F32 => get_f32(registers, arg).map(|value| format!("float {value}")),
            CpuCallScalarKind::F64 => {
                get_f64(registers, arg).map(|value| format!("double {value}"))
            }
        })
        .collect::<Option<Vec<_>>>();
    let Some(lowered_args) = lowered_args else {
        body.push(format!(
            "  ; deferred lowering for cpu.{} `{}` because one or more inputs are outside the current CPU LLVM slice",
            node.op.instruction, node.name
        ));
        return Ok(true);
    };

    let reg = fresh_reg(next_reg);
    let symbol = format!("nuis_fn_{callee}");
    let call = match lowered_args.as_slice() {
        [] => format!(
            "call {} @{symbol}()",
            cpu_scalar_kind_llvm_type(signature.ret)
        ),
        [a0] => format!(
            "call {} @{symbol}({a0})",
            cpu_scalar_kind_llvm_type(signature.ret)
        ),
        [a0, a1] => format!(
            "call {} @{symbol}({a0}, {a1})",
            cpu_scalar_kind_llvm_type(signature.ret)
        ),
        [a0, a1, a2] => format!(
            "call {} @{symbol}({a0}, {a1}, {a2})",
            cpu_scalar_kind_llvm_type(signature.ret)
        ),
        _ => {
            body.push(format!(
                "  ; deferred lowering for cpu.{} `{}` because callee `{}` has unsupported arity {}",
                node.op.instruction,
                node.name,
                callee,
                lowered_args.len()
            ));
            return Ok(true);
        }
    };
    body.push(format!("  {reg} = {call}"));

    match signature.ret {
        CpuCallScalarKind::Bool => {
            let widened = fresh_reg(next_reg);
            body.push(format!("  {widened} = zext i1 {reg} to i64"));
            registers.insert(
                node.name.clone(),
                LlvmValueRef::Bool {
                    i1: reg.clone(),
                    i64: widened.clone(),
                },
            );
            *last_cpu_value = Some(widened);
        }
        CpuCallScalarKind::I32 => {
            registers.insert(node.name.clone(), LlvmValueRef::I32(reg.clone()));
            let widened = fresh_reg(next_reg);
            body.push(format!("  {widened} = sext i32 {reg} to i64"));
            *last_cpu_value = Some(widened);
        }
        CpuCallScalarKind::I64 => {
            registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
            *last_cpu_value = Some(reg);
        }
        CpuCallScalarKind::F32 => {
            registers.insert(node.name.clone(), LlvmValueRef::F32(reg.clone()));
            let widened = fresh_reg(next_reg);
            body.push(format!("  {widened} = fpext float {reg} to double"));
            let as_i64 = fresh_reg(next_reg);
            body.push(format!("  {as_i64} = fptosi double {widened} to i64"));
            *last_cpu_value = Some(as_i64);
        }
        CpuCallScalarKind::F64 => {
            registers.insert(node.name.clone(), LlvmValueRef::F64(reg.clone()));
            let as_i64 = fresh_reg(next_reg);
            body.push(format!("  {as_i64} = fptosi double {reg} to i64"));
            *last_cpu_value = Some(as_i64);
        }
    }

    Ok(true)
}
