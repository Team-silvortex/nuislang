use std::collections::BTreeMap;

use super::{fresh_reg, LlvmValueRef, StructLlvmValueRef, VariantUnionLlvmValueRef};

pub(crate) fn variant_parent_name(type_name: &str) -> Option<&str> {
    type_name.rsplit_once('.').map(|(parent, _)| parent)
}

pub(crate) fn variant_tag_value(variant_name: &str) -> i64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in variant_name.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    (hash & 0x7fff_ffff_ffff_ffff) as i64
}

fn variant_field_from_struct(
    struct_value: &StructLlvmValueRef,
    field_name: &str,
) -> Option<LlvmValueRef> {
    struct_value
        .fields
        .iter()
        .find(|(name, _)| name == field_name)
        .map(|(_, value)| value.clone())
}

pub(crate) fn variant_field_value(
    value: &LlvmValueRef,
    variant_name: &str,
    field_name: &str,
) -> Option<LlvmValueRef> {
    match value {
        LlvmValueRef::Struct(struct_value) if struct_value.type_name == variant_name => {
            variant_field_from_struct(struct_value, field_name)
        }
        LlvmValueRef::VariantUnion(union) => union
            .variants
            .get(variant_name)
            .and_then(|variant| variant_field_from_struct(variant, field_name)),
        _ => None,
    }
}

pub(crate) fn emit_variant_is_value(
    value: &LlvmValueRef,
    variant_name: &str,
    body: &mut Vec<String>,
    next_reg: &mut usize,
) -> Option<LlvmValueRef> {
    match value {
        LlvmValueRef::Struct(struct_value) => {
            let i1 = if struct_value.type_name == variant_name {
                "true"
            } else {
                "false"
            }
            .to_owned();
            let i64 = fresh_reg(next_reg);
            body.push(format!("  {i64} = zext i1 {i1} to i64"));
            Some(LlvmValueRef::Bool { i1, i64 })
        }
        LlvmValueRef::VariantUnion(union) => {
            let expected = variant_tag_value(variant_name);
            let i1 = fresh_reg(next_reg);
            body.push(format!(
                "  {i1} = icmp eq i64 {}, {expected}",
                union.tag_i64
            ));
            let i64 = fresh_reg(next_reg);
            body.push(format!("  {i64} = zext i1 {i1} to i64"));
            Some(LlvmValueRef::Bool { i1, i64 })
        }
        _ => None,
    }
}

fn struct_as_variant_union(
    struct_value: &StructLlvmValueRef,
    tag_i64: String,
) -> Option<VariantUnionLlvmValueRef> {
    let parent_type_name = variant_parent_name(&struct_value.type_name)?.to_owned();
    let mut variants = BTreeMap::new();
    variants.insert(struct_value.type_name.clone(), struct_value.clone());
    Some(VariantUnionLlvmValueRef {
        parent_type_name,
        tag_i64,
        variants,
    })
}

fn merge_variant_maps(
    lhs: &BTreeMap<String, StructLlvmValueRef>,
    rhs: &BTreeMap<String, StructLlvmValueRef>,
) -> BTreeMap<String, StructLlvmValueRef> {
    let mut merged = lhs.clone();
    for (name, value) in rhs {
        merged.entry(name.clone()).or_insert_with(|| value.clone());
    }
    merged
}

pub(crate) fn emit_select_value(
    cond_bool: &str,
    then_value: &LlvmValueRef,
    else_value: &LlvmValueRef,
    body: &mut Vec<String>,
    next_reg: &mut usize,
) -> Option<LlvmValueRef> {
    match (then_value, else_value) {
        (LlvmValueRef::I64(then_i64), LlvmValueRef::I64(else_i64)) => {
            let reg = fresh_reg(next_reg);
            body.push(format!(
                "  {reg} = select i1 {cond_bool}, i64 {then_i64}, i64 {else_i64}"
            ));
            Some(LlvmValueRef::I64(reg))
        }
        (LlvmValueRef::I32(then_i32), LlvmValueRef::I32(else_i32)) => {
            let reg = fresh_reg(next_reg);
            body.push(format!(
                "  {reg} = select i1 {cond_bool}, i32 {then_i32}, i32 {else_i32}"
            ));
            Some(LlvmValueRef::I32(reg))
        }
        (LlvmValueRef::F32(then_f32), LlvmValueRef::F32(else_f32)) => {
            let reg = fresh_reg(next_reg);
            body.push(format!(
                "  {reg} = select i1 {cond_bool}, float {then_f32}, float {else_f32}"
            ));
            Some(LlvmValueRef::F32(reg))
        }
        (LlvmValueRef::F64(then_f64), LlvmValueRef::F64(else_f64)) => {
            let reg = fresh_reg(next_reg);
            body.push(format!(
                "  {reg} = select i1 {cond_bool}, double {then_f64}, double {else_f64}"
            ));
            Some(LlvmValueRef::F64(reg))
        }
        (LlvmValueRef::Bool { i1: then_i1, .. }, LlvmValueRef::Bool { i1: else_i1, .. }) => {
            let i1 = fresh_reg(next_reg);
            body.push(format!(
                "  {i1} = select i1 {cond_bool}, i1 {then_i1}, i1 {else_i1}"
            ));
            let i64 = fresh_reg(next_reg);
            body.push(format!("  {i64} = zext i1 {i1} to i64"));
            Some(LlvmValueRef::Bool { i1, i64 })
        }
        (LlvmValueRef::Bool { .. }, _) | (_, LlvmValueRef::Bool { .. }) => {
            let then_i1 = coerce_to_i1_for_select(then_value, body, next_reg)?;
            let else_i1 = coerce_to_i1_for_select(else_value, body, next_reg)?;
            let i1 = fresh_reg(next_reg);
            body.push(format!(
                "  {i1} = select i1 {cond_bool}, i1 {then_i1}, i1 {else_i1}"
            ));
            let i64 = fresh_reg(next_reg);
            body.push(format!("  {i64} = zext i1 {i1} to i64"));
            Some(LlvmValueRef::Bool { i1, i64 })
        }
        (LlvmValueRef::Ptr(then_ptr), LlvmValueRef::Ptr(else_ptr)) => {
            let reg = fresh_reg(next_reg);
            body.push(format!(
                "  {reg} = select i1 {cond_bool}, ptr {then_ptr}, ptr {else_ptr}"
            ));
            Some(LlvmValueRef::Ptr(reg))
        }
        (
            LlvmValueRef::TextHandle {
                ptr: then_ptr,
                handle: then_handle,
            },
            LlvmValueRef::TextHandle {
                ptr: else_ptr,
                handle: else_handle,
            },
        ) => {
            let ptr = fresh_reg(next_reg);
            body.push(format!(
                "  {ptr} = select i1 {cond_bool}, ptr {then_ptr}, ptr {else_ptr}"
            ));
            let handle = fresh_reg(next_reg);
            body.push(format!(
                "  {handle} = select i1 {cond_bool}, i64 {then_handle}, i64 {else_handle}"
            ));
            Some(LlvmValueRef::TextHandle { ptr, handle })
        }
        (LlvmValueRef::Struct(then_struct), LlvmValueRef::Struct(else_struct)) => {
            if then_struct.type_name != else_struct.type_name {
                let then_parent = variant_parent_name(&then_struct.type_name)?;
                let else_parent = variant_parent_name(&else_struct.type_name)?;
                if then_parent != else_parent {
                    return None;
                }
                let tag_i64 = fresh_reg(next_reg);
                body.push(format!(
                    "  {tag_i64} = select i1 {cond_bool}, i64 {}, i64 {}",
                    variant_tag_value(&then_struct.type_name),
                    variant_tag_value(&else_struct.type_name)
                ));
                let then_union = struct_as_variant_union(then_struct, tag_i64.clone())?;
                let else_union = struct_as_variant_union(else_struct, tag_i64.clone())?;
                return Some(LlvmValueRef::VariantUnion(VariantUnionLlvmValueRef {
                    parent_type_name: then_parent.to_owned(),
                    tag_i64,
                    variants: merge_variant_maps(&then_union.variants, &else_union.variants),
                }));
            }
            if then_struct.fields.len() != else_struct.fields.len() {
                return None;
            }
            let mut fields = Vec::new();
            for ((then_name, then_field), (else_name, else_field)) in
                then_struct.fields.iter().zip(else_struct.fields.iter())
            {
                if then_name != else_name {
                    return None;
                }
                let selected =
                    emit_select_value(cond_bool, then_field, else_field, body, next_reg)?;
                fields.push((then_name.clone(), selected));
            }
            Some(LlvmValueRef::Struct(StructLlvmValueRef {
                type_name: then_struct.type_name.clone(),
                fields,
            }))
        }
        (LlvmValueRef::VariantUnion(then_union), LlvmValueRef::VariantUnion(else_union)) => {
            if then_union.parent_type_name != else_union.parent_type_name {
                return None;
            }
            let tag_i64 = fresh_reg(next_reg);
            body.push(format!(
                "  {tag_i64} = select i1 {cond_bool}, i64 {}, i64 {}",
                then_union.tag_i64, else_union.tag_i64
            ));
            Some(LlvmValueRef::VariantUnion(VariantUnionLlvmValueRef {
                parent_type_name: then_union.parent_type_name.clone(),
                tag_i64,
                variants: merge_variant_maps(&then_union.variants, &else_union.variants),
            }))
        }
        (LlvmValueRef::VariantUnion(union), LlvmValueRef::Struct(struct_value))
        | (LlvmValueRef::Struct(struct_value), LlvmValueRef::VariantUnion(union)) => {
            let parent = variant_parent_name(&struct_value.type_name)?;
            if parent != union.parent_type_name {
                return None;
            }
            let struct_tag = variant_tag_value(&struct_value.type_name);
            let tag_i64 = fresh_reg(next_reg);
            if matches!(then_value, LlvmValueRef::VariantUnion(_)) {
                body.push(format!(
                    "  {tag_i64} = select i1 {cond_bool}, i64 {}, i64 {struct_tag}",
                    union.tag_i64
                ));
            } else {
                body.push(format!(
                    "  {tag_i64} = select i1 {cond_bool}, i64 {struct_tag}, i64 {}",
                    union.tag_i64
                ));
            }
            let mut variants = union.variants.clone();
            variants
                .entry(struct_value.type_name.clone())
                .or_insert_with(|| struct_value.clone());
            Some(LlvmValueRef::VariantUnion(VariantUnionLlvmValueRef {
                parent_type_name: union.parent_type_name.clone(),
                tag_i64,
                variants,
            }))
        }
        _ => None,
    }
}

fn coerce_to_i1_for_select(
    value: &LlvmValueRef,
    body: &mut Vec<String>,
    next_reg: &mut usize,
) -> Option<String> {
    match value {
        LlvmValueRef::Bool { i1, .. } => Some(i1.clone()),
        LlvmValueRef::I64(value) => {
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = icmp ne i64 {value}, 0"));
            Some(reg)
        }
        LlvmValueRef::I32(value) => {
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = icmp ne i32 {value}, 0"));
            Some(reg)
        }
        _ => None,
    }
}
