use std::collections::BTreeMap;

use yir_core::{OwnedSelectScalarArg, OwnedSelectScalarCast};

use super::{
    call_lowering::lower_scalar_value_arg, value_ref::borrowed_buffer_parts,
    variant_select::variant_field_value, CpuCallScalarKind, LlvmValueRef,
};

fn owned_tree_scalar_value(
    registers: &BTreeMap<String, LlvmValueRef>,
    arg: &OwnedSelectScalarArg<'_>,
    body: &mut Vec<String>,
    next_reg: &mut usize,
) -> Option<LlvmValueRef> {
    match arg {
        OwnedSelectScalarArg::Value(name) => registers.get(*name).cloned(),
        OwnedSelectScalarArg::VariantField {
            base,
            variant,
            field,
        } => variant_field_value(registers.get(*base)?, variant, field),
        OwnedSelectScalarArg::StructField { field, base } => {
            let LlvmValueRef::Struct(value) =
                owned_tree_scalar_value(registers, base, body, next_reg)?
            else {
                return None;
            };
            value
                .fields
                .into_iter()
                .find(|(name, _)| name == field)
                .map(|(_, value)| value)
        }
        OwnedSelectScalarArg::Cast { kind, value } => {
            let value = owned_tree_scalar_value(registers, value, body, next_reg)?;
            lower_owned_tree_cast(*kind, value, body, next_reg)
        }
        OwnedSelectScalarArg::NonNull { value } => {
            let value = owned_tree_scalar_value(registers, value, body, next_reg)?;
            let LlvmValueRef::BorrowedBuffer { ptr, len } = value else {
                return None;
            };
            let proof = super::fresh_reg(next_reg);
            body.push(format!("  {proof} = icmp ne ptr {ptr}, null"));
            body.push(format!("  call void @llvm.assume(i1 {proof})"));
            Some(LlvmValueRef::BorrowedBuffer { ptr, len })
        }
        OwnedSelectScalarArg::TraversalBorrow { value } => {
            let value = owned_tree_scalar_value(registers, value, body, next_reg)?;
            let LlvmValueRef::Ptr(ptr) = value else {
                return None;
            };
            Some(LlvmValueRef::Ptr(ptr))
        }
    }
}

fn lower_owned_tree_cast(
    kind: OwnedSelectScalarCast,
    value: LlvmValueRef,
    body: &mut Vec<String>,
    next_reg: &mut usize,
) -> Option<LlvmValueRef> {
    use OwnedSelectScalarCast as Cast;
    let reg = super::fresh_reg(next_reg);
    Some(match (kind, value) {
        (Cast::I64ToI32, LlvmValueRef::I64(value)) => {
            body.push(format!("  {reg} = trunc i64 {value} to i32"));
            LlvmValueRef::I32(reg)
        }
        (Cast::I32ToI64, LlvmValueRef::I32(value)) => {
            body.push(format!("  {reg} = sext i32 {value} to i64"));
            LlvmValueRef::I64(reg)
        }
        (Cast::I64ToBool, LlvmValueRef::I64(value)) => {
            body.push(format!("  {reg} = icmp ne i64 {value}, 0"));
            let i64 = super::fresh_reg(next_reg);
            body.push(format!("  {i64} = zext i1 {reg} to i64"));
            LlvmValueRef::Bool { i1: reg, i64 }
        }
        (Cast::BoolToI64, LlvmValueRef::Bool { i1, .. }) => {
            body.push(format!("  {reg} = zext i1 {i1} to i64"));
            LlvmValueRef::I64(reg)
        }
        (Cast::I64ToF32, LlvmValueRef::I64(value)) => {
            body.push(format!("  {reg} = sitofp i64 {value} to float"));
            LlvmValueRef::F32(reg)
        }
        (Cast::F32ToI64, LlvmValueRef::F32(value)) => {
            body.push(format!("  {reg} = fptosi float {value} to i64"));
            LlvmValueRef::I64(reg)
        }
        (Cast::I64ToF64, LlvmValueRef::I64(value)) => {
            body.push(format!("  {reg} = sitofp i64 {value} to double"));
            LlvmValueRef::F64(reg)
        }
        (Cast::F64ToI64, LlvmValueRef::F64(value)) => {
            body.push(format!("  {reg} = fptosi double {value} to i64"));
            LlvmValueRef::I64(reg)
        }
        _ => return None,
    })
}

pub(crate) fn owned_tree_scalar_args_ready(
    registers: &BTreeMap<String, LlvmValueRef>,
    buffer_lengths: &BTreeMap<String, String>,
    args: &[OwnedSelectScalarArg<'_>],
    kinds: &[CpuCallScalarKind],
) -> bool {
    let mut body = Vec::new();
    let mut next_reg = 0;
    lower_owned_tree_scalar_args(
        registers,
        buffer_lengths,
        args,
        kinds,
        &mut body,
        &mut next_reg,
    )
    .is_some()
}

pub(crate) fn lower_owned_tree_scalar_args(
    registers: &BTreeMap<String, LlvmValueRef>,
    buffer_lengths: &BTreeMap<String, String>,
    args: &[OwnedSelectScalarArg<'_>],
    kinds: &[CpuCallScalarKind],
    body: &mut Vec<String>,
    next_reg: &mut usize,
) -> Option<Vec<String>> {
    args.iter()
        .zip(kinds)
        .map(|(arg, kind)| {
            if *kind == CpuCallScalarKind::BorrowedBuffer {
                let parts = match arg {
                    OwnedSelectScalarArg::Value(name) => {
                        borrowed_buffer_parts(registers, buffer_lengths, name)
                    }
                    _ => match owned_tree_scalar_value(registers, arg, body, next_reg)? {
                        LlvmValueRef::BorrowedBuffer { ptr, len } => Some((ptr, len)),
                        _ => None,
                    },
                };
                let (pointer, len) = parts?;
                return Some(format!("ptr {pointer}, i64 {len}"));
            }
            if *kind == CpuCallScalarKind::TraversalPointer {
                let LlvmValueRef::Ptr(pointer) =
                    owned_tree_scalar_value(registers, arg, body, next_reg)?
                else {
                    return None;
                };
                return Some(format!("ptr {pointer}"));
            }
            let value = owned_tree_scalar_value(registers, arg, body, next_reg)?;
            lower_scalar_value_arg(&value, kind)
        })
        .collect()
}
