use std::collections::BTreeMap;

use super::expr_lowering::lower_expr;
use super::{
    bool_type, compatible_types, i64_type, infer_nir_expr_type, ref_type, AstExpr,
    FunctionSignature, NirExpr, NirStructDef, NirTypeRef,
};

pub(super) fn slice_payload_type(ty: &NirTypeRef) -> Option<NirTypeRef> {
    (ty.name == "Slice" && !ty.is_ref && !ty.is_optional && ty.generic_args.len() == 1)
        .then(|| ty.generic_args[0].clone())
}

pub(super) fn lower_optional_explicit_slice_payload_type(
    generic_args: &[nuis_semantics::model::AstTypeRef],
) -> Result<Option<NirTypeRef>, String> {
    match generic_args {
        [] => Ok(None),
        [payload] => Ok(Some(super::lower_type_ref(payload))),
        _ => Err("slice/subslice builtins accept at most 1 explicit generic arg".to_owned()),
    }
}

pub(super) fn lower_slice_payload_type(
    generic_args: &[nuis_semantics::model::AstTypeRef],
) -> Result<NirTypeRef, String> {
    let explicit = lower_optional_explicit_slice_payload_type(generic_args)?;
    let payload = explicit.unwrap_or_else(i64_type);
    if payload != i64_type()
        && payload != super::i32_type()
        && payload != bool_type()
        && payload != super::f32_type()
        && payload != super::f64_type()
    {
        return Err(format!(
            "slice<...>(...) currently supports only `Slice<i64>`, `Slice<i32>`, `Slice<bool>`, `Slice<f32>`, and `Slice<f64>`, found `Slice<{}>`",
            payload.render()
        ));
    }
    Ok(payload)
}

pub(super) fn lower_slice_or_buffer_access_target(
    target: &AstExpr,
    current_domain: &str,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<(NirExpr, NirExpr, NirTypeRef), String> {
    let lowered = lower_expr(
        target,
        current_domain,
        bindings,
        signatures,
        struct_table,
        None,
    )?;
    let lowered_ty = infer_nir_expr_type(&lowered, bindings, signatures, struct_table)
        .ok_or_else(|| "load/store target requires an explicit buffer-like type".to_owned())?;
    if compatible_types(&ref_type("Buffer"), &lowered_ty) {
        return Ok((lowered, NirExpr::Int(0), i64_type()));
    }
    if let Some(payload_ty) = slice_payload_type(&lowered_ty) {
        return Ok((
            NirExpr::FieldAccess {
                base: Box::new(lowered.clone()),
                field: "buffer".to_owned(),
            },
            NirExpr::FieldAccess {
                base: Box::new(lowered),
                field: "start".to_owned(),
            },
            payload_ty,
        ));
    }
    Err(format!(
        "load/store target expects `ref Buffer` or `Slice<...>`, found `{}`",
        lowered_ty.render()
    ))
}

pub(super) fn lower_byte_slice_parts(
    target: &AstExpr,
    current_domain: &str,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<(NirExpr, NirExpr, NirExpr), String> {
    let lowered = lower_expr(
        target,
        current_domain,
        bindings,
        signatures,
        struct_table,
        None,
    )?;
    let lowered_ty = infer_nir_expr_type(&lowered, bindings, signatures, struct_table)
        .ok_or_else(|| "byte view builtin requires a typed `Slice<i64>` input".to_owned())?;
    let payload_ty = slice_payload_type(&lowered_ty).ok_or_else(|| {
        format!(
            "byte view builtin expects `Slice<i64>`, found `{}`",
            lowered_ty.render()
        )
    })?;
    if payload_ty != i64_type() {
        return Err(format!(
            "byte view builtin expects `Slice<i64>`, found `Slice<{}>`",
            payload_ty.render()
        ));
    }
    Ok((
        NirExpr::FieldAccess {
            base: Box::new(lowered.clone()),
            field: "buffer".to_owned(),
        },
        NirExpr::FieldAccess {
            base: Box::new(lowered.clone()),
            field: "start".to_owned(),
        },
        NirExpr::FieldAccess {
            base: Box::new(lowered),
            field: "len".to_owned(),
        },
    ))
}

pub(super) fn ensure_byte_slice_input(
    builtin: &str,
    lowered: &NirExpr,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<(), String> {
    let lowered_ty = infer_nir_expr_type(lowered, bindings, signatures, struct_table)
        .ok_or_else(|| format!("{builtin}(...) requires a typed `Slice<i64>` input"))?;
    let payload_ty = slice_payload_type(&lowered_ty).ok_or_else(|| {
        format!(
            "{builtin}(...) expects `Slice<i64>`, found `{}`",
            lowered_ty.render()
        )
    })?;
    if payload_ty != i64_type() {
        return Err(format!(
            "{builtin}(...) expects `Slice<i64>`, found `Slice<{}>`",
            payload_ty.render()
        ));
    }
    Ok(())
}

pub(super) fn lower_host_compare_or_copy_call(
    callee: &str,
    lhs_buffer: NirExpr,
    lhs_start: NirExpr,
    lhs_len: NirExpr,
    rhs_buffer: NirExpr,
    rhs_start: NirExpr,
    rhs_len: NirExpr,
) -> NirExpr {
    NirExpr::CpuExternCall {
        abi: "c".to_owned(),
        interface: None,
        callee: callee.to_owned(),
        args: vec![
            NirExpr::HostBufferHandle(Box::new(lhs_buffer)),
            lhs_start,
            lhs_len,
            NirExpr::HostBufferHandle(Box::new(rhs_buffer)),
            rhs_start,
            rhs_len,
        ],
    }
}

pub(super) fn lower_byte_split_struct(
    buffer: NirExpr,
    start: NirExpr,
    len: NirExpr,
    delimiter_len: NirExpr,
    index: NirExpr,
) -> NirExpr {
    let found = NirExpr::Binary {
        op: nuis_semantics::model::NirBinaryOp::Ne,
        lhs: Box::new(index.clone()),
        rhs: Box::new(NirExpr::Int(-1)),
    };
    let found_i64 = NirExpr::CastBoolToI64(Box::new(found.clone()));
    let missing_i64 = NirExpr::Binary {
        op: nuis_semantics::model::NirBinaryOp::Sub,
        lhs: Box::new(NirExpr::Int(1)),
        rhs: Box::new(found_i64.clone()),
    };
    let base_end = NirExpr::Binary {
        op: nuis_semantics::model::NirBinaryOp::Add,
        lhs: Box::new(start.clone()),
        rhs: Box::new(len.clone()),
    };
    let split_after_start = NirExpr::Binary {
        op: nuis_semantics::model::NirBinaryOp::Add,
        lhs: Box::new(index.clone()),
        rhs: Box::new(delimiter_len),
    };
    let before_len = NirExpr::Binary {
        op: nuis_semantics::model::NirBinaryOp::Add,
        lhs: Box::new(NirExpr::Binary {
            op: nuis_semantics::model::NirBinaryOp::Mul,
            lhs: Box::new(found_i64.clone()),
            rhs: Box::new(NirExpr::Binary {
                op: nuis_semantics::model::NirBinaryOp::Sub,
                lhs: Box::new(index.clone()),
                rhs: Box::new(start.clone()),
            }),
        }),
        rhs: Box::new(NirExpr::Binary {
            op: nuis_semantics::model::NirBinaryOp::Mul,
            lhs: Box::new(missing_i64.clone()),
            rhs: Box::new(len.clone()),
        }),
    };
    let after_start = NirExpr::Binary {
        op: nuis_semantics::model::NirBinaryOp::Add,
        lhs: Box::new(NirExpr::Binary {
            op: nuis_semantics::model::NirBinaryOp::Mul,
            lhs: Box::new(found_i64.clone()),
            rhs: Box::new(split_after_start.clone()),
        }),
        rhs: Box::new(NirExpr::Binary {
            op: nuis_semantics::model::NirBinaryOp::Mul,
            lhs: Box::new(missing_i64.clone()),
            rhs: Box::new(base_end.clone()),
        }),
    };
    let after_len = NirExpr::Binary {
        op: nuis_semantics::model::NirBinaryOp::Mul,
        lhs: Box::new(found_i64),
        rhs: Box::new(NirExpr::Binary {
            op: nuis_semantics::model::NirBinaryOp::Sub,
            lhs: Box::new(base_end),
            rhs: Box::new(split_after_start),
        }),
    };

    NirExpr::StructLiteral {
        type_name: "ByteSplit".to_owned(),
        type_args: vec![],
        fields: vec![
            ("found".to_owned(), found),
            ("index".to_owned(), index),
            (
                "before".to_owned(),
                make_byte_slice_expr(buffer.clone(), start.clone(), before_len),
            ),
            (
                "after".to_owned(),
                make_byte_slice_expr(buffer, after_start, after_len),
            ),
        ],
    }
}

pub(super) fn make_byte_slice_expr(buffer: NirExpr, start: NirExpr, len: NirExpr) -> NirExpr {
    NirExpr::StructLiteral {
        type_name: "Slice".to_owned(),
        type_args: vec![i64_type()],
        fields: vec![
            ("buffer".to_owned(), buffer),
            ("start".to_owned(), start),
            ("len".to_owned(), len),
        ],
    }
}
