use std::collections::BTreeMap;

use yir_core::{
    loop_carry_contract::ScopedRecordInput, native_scalar_session::ScalarKind,
    OwnedStructFieldLayout, OwnedStructLayout,
};

use crate::{
    call_parameters::CpuCallParameterKind, native_session::value_transport::PreparedNativeValue,
    CpuCallScalarKind, LlvmValueRef, StructLlvmValueRef,
};

#[cfg(test)]
mod tests;

/// Expand only the transport shape; induction and carry slots keep their identities.
pub(crate) fn leaves<'a>(
    operand: &'a str,
    kind: &CpuCallParameterKind,
) -> Result<Vec<(&'a str, CpuCallScalarKind)>, String> {
    match (ScopedRecordInput::parse(operand)?, kind) {
        (Some(record), CpuCallParameterKind::Record(layout)) => {
            if yir_core::parse_owned_struct_layout(record.layout.source())? != *layout.layout() {
                return Err(
                    "scoped record input disagrees with its helper parameter layout".into(),
                );
            }
            Ok(record
                .operands
                .into_iter()
                .zip(record.layout.fields())
                .map(|(input, (_, kind))| {
                    let kind = match kind {
                        ScalarKind::Bool => CpuCallScalarKind::Bool,
                        ScalarKind::I32 => CpuCallScalarKind::I32,
                        ScalarKind::I64 => CpuCallScalarKind::I64,
                        ScalarKind::F32 => CpuCallScalarKind::F32,
                        ScalarKind::F64 => CpuCallScalarKind::F64,
                    };
                    (input, kind)
                })
                .collect())
        }
        (None, CpuCallParameterKind::Scalar(kind)) => Ok(vec![(operand, *kind)]),
        _ => Err("scoped record input requires an explicit matching record parameter".into()),
    }
}

pub(crate) fn prepare(
    operand: &str,
    kind: &CpuCallParameterKind,
    current: &str,
    registers: &BTreeMap<String, LlvmValueRef>,
    overrides: &BTreeMap<String, LlvmValueRef>,
) -> Result<Option<PreparedNativeValue>, String> {
    let inputs = leaves(operand, kind)?;
    let CpuCallParameterKind::Record(layout) = kind else {
        return Ok(None);
    };
    let values = inputs
        .into_iter()
        .map(|(input, kind)| {
            let value = if input == "$current" {
                LlvmValueRef::I64(current.into())
            } else {
                overrides
                    .get(input)
                    .or_else(|| registers.get(input))
                    .cloned()
                    .ok_or_else(|| format!("missing scoped record leaf `{input}`"))?
            };
            if crate::call_lowering::lower_scalar_value_arg(&value, &kind).is_none() {
                return Err(format!(
                    "scoped record leaf `{input}` requires exact {}",
                    crate::call_return::cpu_scalar_kind_llvm_type(kind)
                ));
            }
            Ok(value)
        })
        .collect::<Result<Vec<_>, String>>()?;
    let value = rebuild(layout.layout(), &mut values.into_iter());
    layout.prepare(&LlvmValueRef::Struct(value)).map(Some)
}

fn rebuild(
    layout: &OwnedStructLayout,
    values: &mut impl Iterator<Item = LlvmValueRef>,
) -> StructLlvmValueRef {
    StructLlvmValueRef {
        type_name: layout.type_name.clone(),
        fields: layout
            .fields
            .iter()
            .map(|(name, field)| {
                let value = match field {
                    OwnedStructFieldLayout::Struct(nested) => {
                        LlvmValueRef::Struct(rebuild(nested, values))
                    }
                    OwnedStructFieldLayout::Scalar(_) => {
                        values.next().expect("checked scoped record leaf count")
                    }
                };
                (name.clone(), value)
            })
            .collect(),
    }
}
