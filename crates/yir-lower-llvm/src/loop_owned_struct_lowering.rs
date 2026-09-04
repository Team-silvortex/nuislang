use std::collections::BTreeMap;

use yir_core::{parse_loop_owned_struct_carry, Node};

use super::{
    call_lowering::{parse_owned_struct_layout, unpack_immediate_owned_struct},
    call_return::cpu_scalar_kind_llvm_type,
    fresh_reg, CpuCallScalarKind, CpuHelperSignature, LlvmValueRef, StructLlvmValueRef,
};

struct OwnedStructLoopSlot {
    index: usize,
    operand: String,
    kind: CpuCallScalarKind,
    slot: String,
}

pub(crate) struct OwnedStructLoopCarry {
    result_name: String,
    template: StructLlvmValueRef,
    slots: Vec<OwnedStructLoopSlot>,
}

pub(crate) fn prepare_owned_struct_loop_carry(
    node: &Node,
    body: &mut Vec<String>,
    registers: &BTreeMap<String, LlvmValueRef>,
    helper_signatures: &BTreeMap<String, CpuHelperSignature>,
    next_reg: &mut usize,
) -> Result<Option<OwnedStructLoopCarry>, String> {
    if node.op.instruction != "loop_while_i64_effect"
        || node.op.args.get(6).map(String::as_str) != Some("scoped_call_owned_struct_return")
    {
        return Ok(None);
    }
    let callee = node
        .op
        .args
        .get(8)
        .ok_or_else(|| missing_metadata(node, "callee"))?;
    let result_name = node
        .op
        .args
        .get(9)
        .ok_or_else(|| missing_metadata(node, "result projection"))?
        .clone();
    let layout = node
        .op
        .args
        .get(10)
        .ok_or_else(|| missing_metadata(node, "owned struct layout"))?;
    let operands = node
        .op
        .args
        .get(11..)
        .ok_or_else(|| missing_metadata(node, "scoped operands"))?;
    let signature = helper_signatures.get(callee).ok_or_else(|| {
        format!(
            "cpu.loop_while_i64_effect `{}` cannot resolve aggregate helper `{callee}`",
            node.name
        )
    })?;
    if !signature.owned_struct_return {
        return Err(format!(
            "cpu.loop_while_i64_effect `{}` treats non-aggregate helper `{callee}` as an owned struct return",
            node.name
        ));
    }
    if signature.params.len() != operands.len() {
        return Err(format!(
            "cpu.loop_while_i64_effect `{}` aggregate helper `{callee}` expects {} args, found {}",
            node.name,
            signature.params.len(),
            operands.len()
        ));
    }

    let mut slots = Vec::new();
    for (operand, kind) in operands.iter().zip(signature.params.iter().copied()) {
        let Some((index, input)) = parse_loop_owned_struct_carry(operand)? else {
            continue;
        };
        let initial = scalar_value(registers.get(input), kind).ok_or_else(|| {
            format!(
                "cpu.loop_while_i64_effect `{}` cannot resolve aggregate carry leaf `{input}`",
                node.name
            )
        })?;
        let slot = fresh_reg(next_reg);
        let llvm_type = scalar_loop_type(kind).ok_or_else(|| {
            format!(
                "cpu.loop_while_i64_effect `{}` aggregate carry leaf `{input}` is not a scalar value",
                node.name
            )
        })?;
        body.push(format!("  {slot} = alloca {llvm_type}"));
        body.push(format!("  store {llvm_type} {initial}, ptr {slot}"));
        slots.push(OwnedStructLoopSlot {
            index,
            operand: operand.clone(),
            kind,
            slot,
        });
    }
    slots.sort_by_key(|slot| slot.index);
    if slots.is_empty()
        || slots
            .iter()
            .enumerate()
            .any(|(index, slot)| slot.index != index)
    {
        return Err(format!(
            "cpu.loop_while_i64_effect `{}` has an incomplete aggregate carry sequence",
            node.name
        ));
    }
    let template = parse_owned_struct_layout(layout)?;
    if scalar_leaf_count(&template) != Some(slots.len()) {
        return Err(format!(
            "cpu.loop_while_i64_effect `{}` aggregate layout has {} leaves but carries {}",
            node.name,
            scalar_leaf_count(&template).unwrap_or_default(),
            slots.len()
        ));
    }
    Ok(Some(OwnedStructLoopCarry {
        result_name,
        template,
        slots,
    }))
}

impl OwnedStructLoopCarry {
    pub(crate) fn load_operand_overrides(
        &self,
        body: &mut Vec<String>,
        next_reg: &mut usize,
    ) -> BTreeMap<String, LlvmValueRef> {
        self.slots
            .iter()
            .map(|slot| (slot.operand.clone(), load_slot(slot, body, next_reg)))
            .collect()
    }

    pub(crate) fn store_return(
        &self,
        pointer_bits: &str,
        body: &mut Vec<String>,
        next_reg: &mut usize,
    ) -> Result<(), String> {
        let returned = unpack_immediate_owned_struct(pointer_bits, &self.template, body, next_reg);
        let mut values = Vec::new();
        flatten_scalar_values(&returned, &mut values)?;
        if values.len() != self.slots.len() {
            return Err("owned struct loop return does not match its carry layout".to_owned());
        }
        for (slot, value) in self.slots.iter().zip(values) {
            let raw = scalar_value(Some(value), slot.kind)
                .ok_or_else(|| "owned struct loop return changed scalar leaf type".to_owned())?;
            body.push(format!(
                "  store {} {raw}, ptr {}",
                scalar_loop_type(slot.kind).expect("validated aggregate scalar kind"),
                slot.slot
            ));
        }
        Ok(())
    }

    pub(crate) fn finish(
        self,
        body: &mut Vec<String>,
        next_reg: &mut usize,
    ) -> Result<(String, StructLlvmValueRef), String> {
        let values = self
            .slots
            .iter()
            .map(|slot| load_slot(slot, body, next_reg))
            .collect::<Vec<_>>();
        let mut values = values.into_iter();
        let value = rebuild_struct(&self.template, &mut values)?;
        if values.next().is_some() {
            return Err("owned struct loop carry left unmatched scalar values".to_owned());
        }
        Ok((self.result_name, value))
    }
}

fn load_slot(
    slot: &OwnedStructLoopSlot,
    body: &mut Vec<String>,
    next_reg: &mut usize,
) -> LlvmValueRef {
    let llvm_type = scalar_loop_type(slot.kind).expect("validated aggregate scalar kind");
    let raw = fresh_reg(next_reg);
    body.push(format!("  {raw} = load {llvm_type}, ptr {}", slot.slot));
    if slot.kind == CpuCallScalarKind::Bool {
        let widened = fresh_reg(next_reg);
        body.push(format!("  {widened} = zext i1 {raw} to i64"));
        return LlvmValueRef::Bool {
            i1: raw,
            i64: widened,
        };
    }
    match slot.kind {
        CpuCallScalarKind::I32 => LlvmValueRef::I32(raw),
        CpuCallScalarKind::I64 => LlvmValueRef::I64(raw),
        CpuCallScalarKind::F32 => LlvmValueRef::F32(raw),
        CpuCallScalarKind::F64 => LlvmValueRef::F64(raw),
        _ => unreachable!("aggregate carry slots admit scalar values only"),
    }
}

fn scalar_value<'a>(value: Option<&'a LlvmValueRef>, kind: CpuCallScalarKind) -> Option<&'a str> {
    match (kind, value?) {
        (CpuCallScalarKind::Bool, LlvmValueRef::Bool { i1, .. }) => Some(i1),
        (CpuCallScalarKind::I32, LlvmValueRef::I32(value))
        | (CpuCallScalarKind::I64, LlvmValueRef::I64(value))
        | (CpuCallScalarKind::F32, LlvmValueRef::F32(value))
        | (CpuCallScalarKind::F64, LlvmValueRef::F64(value)) => Some(value),
        _ => None,
    }
}

fn scalar_loop_type(kind: CpuCallScalarKind) -> Option<&'static str> {
    matches!(
        kind,
        CpuCallScalarKind::Bool
            | CpuCallScalarKind::I32
            | CpuCallScalarKind::I64
            | CpuCallScalarKind::F32
            | CpuCallScalarKind::F64
    )
    .then(|| cpu_scalar_kind_llvm_type(kind))
}

fn scalar_leaf_count(value: &StructLlvmValueRef) -> Option<usize> {
    value.fields.iter().try_fold(0usize, |count, (_, field)| {
        let leaves = match field {
            LlvmValueRef::Struct(nested) => scalar_leaf_count(nested)?,
            LlvmValueRef::Bool { .. }
            | LlvmValueRef::I32(_)
            | LlvmValueRef::I64(_)
            | LlvmValueRef::F32(_)
            | LlvmValueRef::F64(_) => 1,
            _ => return None,
        };
        count.checked_add(leaves)
    })
}

fn flatten_scalar_values<'a>(
    value: &'a StructLlvmValueRef,
    values: &mut Vec<&'a LlvmValueRef>,
) -> Result<(), String> {
    for (_, field) in &value.fields {
        match field {
            LlvmValueRef::Struct(nested) => flatten_scalar_values(nested, values)?,
            LlvmValueRef::Bool { .. }
            | LlvmValueRef::I32(_)
            | LlvmValueRef::I64(_)
            | LlvmValueRef::F32(_)
            | LlvmValueRef::F64(_) => values.push(field),
            _ => return Err("owned struct loop return contains a non-scalar leaf".to_owned()),
        }
    }
    Ok(())
}

fn rebuild_struct(
    template: &StructLlvmValueRef,
    values: &mut impl Iterator<Item = LlvmValueRef>,
) -> Result<StructLlvmValueRef, String> {
    let fields = template
        .fields
        .iter()
        .map(|(name, field)| {
            let value = match field {
                LlvmValueRef::Struct(nested) => {
                    LlvmValueRef::Struct(rebuild_struct(nested, values)?)
                }
                _ => values
                    .next()
                    .ok_or_else(|| "owned struct loop carry ended before its layout".to_owned())?,
            };
            Ok((name.clone(), value))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(StructLlvmValueRef {
        type_name: template.type_name.clone(),
        fields,
    })
}

fn missing_metadata(node: &Node, field: &str) -> String {
    format!(
        "cpu.loop_while_i64_effect `{}` is missing aggregate {field}",
        node.name
    )
}
