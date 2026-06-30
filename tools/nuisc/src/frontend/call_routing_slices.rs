use std::collections::BTreeMap;

use super::call_routing_slice_helpers::{
    lower_optional_explicit_slice_payload_type, lower_slice_payload_type, slice_payload_type,
};
use super::expr_lowering::lower_expr;
use super::{
    compatible_types, i64_type, infer_nir_expr_type, ref_type, AstExpr, FunctionSignature, NirExpr,
    NirStructDef, NirTypeRef,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_slice_or_byte_builtin(
    callee: &str,
    generic_args: &[nuis_semantics::model::AstTypeRef],
    args: &[AstExpr],
    current_domain: &str,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<Option<NirExpr>, String> {
    if let Some(byte_builtin) = super::call_routing_bytes::lower_byte_builtin(
        callee,
        generic_args,
        args,
        current_domain,
        bindings,
        signatures,
        struct_table,
    )? {
        return Ok(Some(byte_builtin));
    }

    let expr = match callee {
        "slice" => {
            let [buffer, start, len] = args else {
                return Err("slice(...) expects 3 args".to_owned());
            };
            let slice_payload_ty = lower_slice_payload_type(generic_args)?;
            let lowered_buffer = lower_expr(
                buffer,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&ref_type("Buffer")),
            )?;
            let lowered_start = lower_expr(
                start,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let lowered_len = lower_expr(
                len,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            NirExpr::StructLiteral {
                type_name: "Slice".to_owned(),
                type_args: vec![slice_payload_ty],
                fields: vec![
                    ("buffer".to_owned(), lowered_buffer),
                    ("start".to_owned(), lowered_start),
                    ("len".to_owned(), lowered_len),
                ],
            }
        }
        "bytes" => {
            let [buffer, start, len] = args else {
                return Err("bytes(...) expects 3 args".to_owned());
            };
            if !generic_args.is_empty() {
                return Err("bytes(...) does not accept explicit generic args".to_owned());
            }
            let lowered_buffer = lower_expr(
                buffer,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&ref_type("Buffer")),
            )?;
            let lowered_start = lower_expr(
                start,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let lowered_len = lower_expr(
                len,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            NirExpr::StructLiteral {
                type_name: "Slice".to_owned(),
                type_args: vec![i64_type()],
                fields: vec![
                    ("buffer".to_owned(), lowered_buffer),
                    ("start".to_owned(), lowered_start),
                    ("len".to_owned(), lowered_len),
                ],
            }
        }
        "subslice" => {
            let [base, offset, len] = args else {
                return Err("subslice(...) expects 3 args".to_owned());
            };
            let lowered_base = lower_expr(
                base,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            let lowered_base_ty =
                infer_nir_expr_type(&lowered_base, bindings, signatures, struct_table).ok_or_else(
                    || "subslice(...) requires a typed `Slice<...>` input".to_owned(),
                )?;
            let slice_payload_ty = slice_payload_type(&lowered_base_ty).ok_or_else(|| {
                format!(
                    "subslice(...) expects `Slice<...>`, found `{}`",
                    lowered_base_ty.render()
                )
            })?;
            if let Some(explicit_ty) = lower_optional_explicit_slice_payload_type(generic_args)? {
                if !compatible_types(&explicit_ty, &slice_payload_ty) {
                    return Err(format!(
                        "subslice<...>(...) payload `{}` does not match input slice payload `{}`",
                        explicit_ty.render(),
                        slice_payload_ty.render()
                    ));
                }
            }
            let lowered_offset = lower_expr(
                offset,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let lowered_len = lower_expr(
                len,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            NirExpr::StructLiteral {
                type_name: "Slice".to_owned(),
                type_args: vec![slice_payload_ty],
                fields: vec![
                    (
                        "buffer".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(lowered_base.clone()),
                            field: "buffer".to_owned(),
                        },
                    ),
                    (
                        "start".to_owned(),
                        NirExpr::Binary {
                            op: nuis_semantics::model::NirBinaryOp::Add,
                            lhs: Box::new(NirExpr::FieldAccess {
                                base: Box::new(lowered_base),
                                field: "start".to_owned(),
                            }),
                            rhs: Box::new(lowered_offset),
                        },
                    ),
                    ("len".to_owned(), lowered_len),
                ],
            }
        }
        "subbytes" => {
            let [base, offset, len] = args else {
                return Err("subbytes(...) expects 3 args".to_owned());
            };
            if !generic_args.is_empty() {
                return Err("subbytes(...) does not accept explicit generic args".to_owned());
            }
            let lowered_base = lower_expr(
                base,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            let lowered_base_ty =
                infer_nir_expr_type(&lowered_base, bindings, signatures, struct_table).ok_or_else(
                    || "subbytes(...) requires a typed `Slice<i64>` input".to_owned(),
                )?;
            let slice_payload_ty = slice_payload_type(&lowered_base_ty).ok_or_else(|| {
                format!(
                    "subbytes(...) expects `Slice<i64>`, found `{}`",
                    lowered_base_ty.render()
                )
            })?;
            if slice_payload_ty != i64_type() {
                return Err(format!(
                    "subbytes(...) expects `Slice<i64>`, found `Slice<{}>`",
                    slice_payload_ty.render()
                ));
            }
            let lowered_offset = lower_expr(
                offset,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let lowered_len = lower_expr(
                len,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            NirExpr::StructLiteral {
                type_name: "Slice".to_owned(),
                type_args: vec![i64_type()],
                fields: vec![
                    (
                        "buffer".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(lowered_base.clone()),
                            field: "buffer".to_owned(),
                        },
                    ),
                    (
                        "start".to_owned(),
                        NirExpr::Binary {
                            op: nuis_semantics::model::NirBinaryOp::Add,
                            lhs: Box::new(NirExpr::FieldAccess {
                                base: Box::new(lowered_base),
                                field: "start".to_owned(),
                            }),
                            rhs: Box::new(lowered_offset),
                        },
                    ),
                    ("len".to_owned(), lowered_len),
                ],
            }
        }
        "slice_len" => {
            let [base] = args else {
                return Err("slice_len(...) expects 1 arg".to_owned());
            };
            let lowered = lower_expr(
                base,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            let lowered_ty = infer_nir_expr_type(&lowered, bindings, signatures, struct_table)
                .ok_or_else(|| "slice_len(...) requires a typed `Slice<...>` input".to_owned())?;
            if slice_payload_type(&lowered_ty).is_none() {
                return Err(format!(
                    "slice_len(...) expects `Slice<...>`, found `{}`",
                    lowered_ty.render()
                ));
            }
            NirExpr::FieldAccess {
                base: Box::new(lowered),
                field: "len".to_owned(),
            }
        }
        "slice_start" => {
            let [base] = args else {
                return Err("slice_start(...) expects 1 arg".to_owned());
            };
            let lowered = lower_expr(
                base,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            let lowered_ty = infer_nir_expr_type(&lowered, bindings, signatures, struct_table)
                .ok_or_else(|| "slice_start(...) requires a typed `Slice<...>` input".to_owned())?;
            if slice_payload_type(&lowered_ty).is_none() {
                return Err(format!(
                    "slice_start(...) expects `Slice<...>`, found `{}`",
                    lowered_ty.render()
                ));
            }
            NirExpr::FieldAccess {
                base: Box::new(lowered),
                field: "start".to_owned(),
            }
        }
        "slice_buffer" => {
            let [base] = args else {
                return Err("slice_buffer(...) expects 1 arg".to_owned());
            };
            let lowered = lower_expr(
                base,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            let lowered_ty = infer_nir_expr_type(&lowered, bindings, signatures, struct_table)
                .ok_or_else(|| {
                    "slice_buffer(...) requires a typed `Slice<...>` input".to_owned()
                })?;
            if slice_payload_type(&lowered_ty).is_none() {
                return Err(format!(
                    "slice_buffer(...) expects `Slice<...>`, found `{}`",
                    lowered_ty.render()
                ));
            }
            NirExpr::FieldAccess {
                base: Box::new(lowered),
                field: "buffer".to_owned(),
            }
        }
        _ => return Ok(None),
    };
    Ok(Some(expr))
}
