use std::collections::BTreeMap;

use yir_core::Node;

use super::{
    call_return::cpu_scalar_kind_llvm_type,
    fresh_reg,
    value_ref::{get_bool, get_f32, get_f64, get_i32, get_i64, get_ptr},
    CpuCallScalarKind, CpuHelperSignature, LlvmValueRef,
};

#[derive(Clone)]
pub(crate) enum LoopEffectCleanup {
    None,
    OwnedBlob(String),
}

pub(crate) fn begin_loop_effect_action(
    node: &Node,
    action_offset: usize,
    body: &mut Vec<String>,
    registers: &BTreeMap<String, LlvmValueRef>,
    buffer_lengths: &BTreeMap<String, String>,
    helper_signatures: &BTreeMap<String, CpuHelperSignature>,
    current: &str,
    next_reg: &mut usize,
) -> Result<LoopEffectCleanup, String> {
    let action_module = &node.op.args[action_offset];
    let action_instruction = &node.op.args[action_offset + 1];
    let arity = node.op.args[action_offset + 2]
        .parse::<usize>()
        .map_err(|_| {
            format!(
                "cpu.loop_while_i64_effect `{}` has invalid action arity `{}`",
                node.name,
                node.op.args[action_offset + 2]
            )
        })?;
    let action_args = &node.op.args[action_offset + 3..];
    if action_args.len() != arity {
        return Err(format!(
            "cpu.loop_while_i64_effect `{}` declares {arity} action operands but provides {}",
            node.name,
            action_args.len()
        ));
    }

    match (action_module.as_str(), action_instruction.as_str()) {
        ("cpu", "owned_bytes_copy_drop") => {
            let source_name = &action_args[0];
            let (Some(ptr), Some(len)) = (
                get_ptr(registers, source_name),
                buffer_lengths.get(source_name),
            ) else {
                return Err(format!(
                    "cpu.loop_while_i64_effect `{}` cannot resolve owned-bytes source `{source_name}`",
                    node.name
                ));
            };
            let byte_len = fresh_reg(next_reg);
            body.push(format!("  {byte_len} = mul i64 {len}, 8"));
            let blob = fresh_reg(next_reg);
            body.push(format!(
                "  {blob} = call ptr @nuis_scheduler_owned_blob_copy_v1(ptr {ptr}, i64 {byte_len}, i64 {})",
                stable_glm_token(&node.name)
            ));
            Ok(LoopEffectCleanup::OwnedBlob(blob))
        }
        ("cpu", "scoped_call") => {
            let (callee, operands) = action_args.split_first().ok_or_else(|| {
                format!(
                    "cpu.loop_while_i64_effect `{}` has a scoped call without a callee",
                    node.name
                )
            })?;
            let signature = helper_signatures.get(callee).ok_or_else(|| {
                format!(
                    "cpu.loop_while_i64_effect `{}` cannot resolve scoped helper `{callee}`",
                    node.name
                )
            })?;
            if signature.params.len() != operands.len() {
                return Err(format!(
                    "cpu.loop_while_i64_effect `{}` scoped helper `{callee}` expects {} args, found {}",
                    node.name,
                    signature.params.len(),
                    operands.len()
                ));
            }
            let lowered = operands
                .iter()
                .zip(signature.params.iter().copied())
                .map(|(operand, kind)| {
                    lower_scoped_operand(operand, kind, current, registers, buffer_lengths)
                })
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();
            body.push(format!(
                "  call {} @nuis_fn_{callee}({})",
                cpu_scalar_kind_llvm_type(signature.ret),
                lowered.join(", ")
            ));
            Ok(LoopEffectCleanup::None)
        }
        (module, instruction) => {
            Err(format!(
                "cpu.loop_while_i64_effect `{}` references unregistered loop action `{module}.{instruction}`",
                node.name
            ))
        }
    }
}

pub(crate) fn finish_loop_effect_action(cleanup: &LoopEffectCleanup, body: &mut Vec<String>) {
    match cleanup {
        LoopEffectCleanup::None => {}
        LoopEffectCleanup::OwnedBlob(blob) => body.push(format!(
            "  call void @nuis_scheduler_owned_blob_drop_v1(ptr {blob})"
        )),
    }
}

fn lower_scoped_operand(
    operand: &str,
    kind: CpuCallScalarKind,
    current: &str,
    registers: &BTreeMap<String, LlvmValueRef>,
    buffer_lengths: &BTreeMap<String, String>,
) -> Result<Vec<String>, String> {
    if kind == CpuCallScalarKind::BorrowedBuffer {
        let ptr = get_ptr(registers, operand)
            .ok_or_else(|| format!("cannot lower scoped buffer `{operand}`"))?;
        let len = buffer_lengths
            .get(operand)
            .ok_or_else(|| format!("cannot lower scoped buffer length `{operand}`"))?;
        return Ok(vec![format!("ptr {ptr}"), format!("i64 {len}")]);
    }
    let value = if operand == "$current" {
        (kind == CpuCallScalarKind::I64).then_some(current)
    } else {
        match kind {
            CpuCallScalarKind::Bool => get_bool(registers, operand),
            CpuCallScalarKind::I32 => get_i32(registers, operand),
            CpuCallScalarKind::I64 => get_i64(registers, operand),
            CpuCallScalarKind::F32 => get_f32(registers, operand),
            CpuCallScalarKind::F64 => get_f64(registers, operand),
            CpuCallScalarKind::BorrowedBuffer => get_ptr(registers, operand),
        }
    }
    .ok_or_else(|| format!("cannot lower scoped call operand `{operand}`"))?;
    Ok(vec![format!("{} {value}", cpu_scalar_kind_llvm_type(kind))])
}

fn stable_glm_token(name: &str) -> u64 {
    name.bytes().fold(0xcbf29ce484222325u64, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    })
}
