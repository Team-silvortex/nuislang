use std::collections::BTreeMap;

use super::call_routing_slice_helpers::{
    ensure_byte_slice_input, lower_byte_slice_parts, lower_byte_split_struct,
};
use super::expr_lowering::lower_expr;
use super::{i64_type, AstExpr, FunctionSignature, NirExpr, NirStructDef, NirTypeRef};

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_byte_split_builtin(
    callee: &str,
    generic_args: &[nuis_semantics::model::AstTypeRef],
    args: &[AstExpr],
    current_domain: &str,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<Option<NirExpr>, String> {
    let expr = match callee {
        "bytes_slice_before" => {
            let [base, index] = args else {
                return Err("bytes_slice_before(...) expects 2 args".to_owned());
            };
            if !generic_args.is_empty() {
                return Err(
                    "bytes_slice_before(...) does not accept explicit generic args".to_owned(),
                );
            }
            let lowered_base = lower_expr(
                base,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            ensure_byte_slice_input(
                "bytes_slice_before",
                &lowered_base,
                bindings,
                signatures,
                struct_table,
            )?;
            let lowered_index = lower_expr(
                index,
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
                        NirExpr::FieldAccess {
                            base: Box::new(lowered_base.clone()),
                            field: "start".to_owned(),
                        },
                    ),
                    (
                        "len".to_owned(),
                        NirExpr::Binary {
                            op: nuis_semantics::model::NirBinaryOp::Sub,
                            lhs: Box::new(lowered_index),
                            rhs: Box::new(NirExpr::FieldAccess {
                                base: Box::new(lowered_base),
                                field: "start".to_owned(),
                            }),
                        },
                    ),
                ],
            }
        }
        "bytes_slice_after" => {
            let [base, index] = args else {
                return Err("bytes_slice_after(...) expects 2 args".to_owned());
            };
            if !generic_args.is_empty() {
                return Err(
                    "bytes_slice_after(...) does not accept explicit generic args".to_owned(),
                );
            }
            let lowered_base = lower_expr(
                base,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            ensure_byte_slice_input(
                "bytes_slice_after",
                &lowered_base,
                bindings,
                signatures,
                struct_table,
            )?;
            let lowered_index = lower_expr(
                index,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let after_start = NirExpr::Binary {
                op: nuis_semantics::model::NirBinaryOp::Add,
                lhs: Box::new(lowered_index),
                rhs: Box::new(NirExpr::Int(1)),
            };
            let base_end = NirExpr::Binary {
                op: nuis_semantics::model::NirBinaryOp::Add,
                lhs: Box::new(NirExpr::FieldAccess {
                    base: Box::new(lowered_base.clone()),
                    field: "start".to_owned(),
                }),
                rhs: Box::new(NirExpr::FieldAccess {
                    base: Box::new(lowered_base.clone()),
                    field: "len".to_owned(),
                }),
            };
            NirExpr::StructLiteral {
                type_name: "Slice".to_owned(),
                type_args: vec![i64_type()],
                fields: vec![
                    (
                        "buffer".to_owned(),
                        NirExpr::FieldAccess {
                            base: Box::new(lowered_base),
                            field: "buffer".to_owned(),
                        },
                    ),
                    ("start".to_owned(), after_start.clone()),
                    (
                        "len".to_owned(),
                        NirExpr::Binary {
                            op: nuis_semantics::model::NirBinaryOp::Sub,
                            lhs: Box::new(base_end),
                            rhs: Box::new(after_start),
                        },
                    ),
                ],
            }
        }
        "bytes_split_once_byte" => {
            let [base, needle] = args else {
                return Err("bytes_split_once_byte(...) expects 2 args".to_owned());
            };
            if !generic_args.is_empty() {
                return Err(
                    "bytes_split_once_byte(...) does not accept explicit generic args".to_owned(),
                );
            }
            if current_domain != "cpu" {
                return Err(
                    "bytes_split_once_byte(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let (buffer, start, len) =
                lower_byte_slice_parts(base, current_domain, bindings, signatures, struct_table)?;
            let lowered_needle = lower_expr(
                needle,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            lower_byte_split_struct(
                buffer.clone(),
                start.clone(),
                len.clone(),
                NirExpr::Int(1),
                NirExpr::CpuExternCall {
                    abi: "c".to_owned(),
                    interface: None,
                    callee: "host_buffer_find_byte".to_owned(),
                    args: vec![
                        NirExpr::HostBufferHandle(Box::new(buffer)),
                        start,
                        len,
                        lowered_needle,
                    ],
                },
            )
        }
        "bytes_split_once_text" => {
            let [base, needle] = args else {
                return Err("bytes_split_once_text(...) expects 2 args".to_owned());
            };
            if !generic_args.is_empty() {
                return Err(
                    "bytes_split_once_text(...) does not accept explicit generic args".to_owned(),
                );
            }
            if current_domain != "cpu" {
                return Err(
                    "bytes_split_once_text(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let (buffer, start, len) =
                lower_byte_slice_parts(base, current_domain, bindings, signatures, struct_table)?;
            let lowered_needle = lower_expr(
                needle,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            lower_byte_split_struct(
                buffer.clone(),
                start.clone(),
                len.clone(),
                NirExpr::CpuExternCall {
                    abi: "c".to_owned(),
                    interface: None,
                    callee: "host_text_len".to_owned(),
                    args: vec![lowered_needle.clone()],
                },
                NirExpr::CpuExternCall {
                    abi: "c".to_owned(),
                    interface: None,
                    callee: "host_buffer_find_text".to_owned(),
                    args: vec![
                        NirExpr::HostBufferHandle(Box::new(buffer)),
                        start,
                        len,
                        lowered_needle,
                    ],
                },
            )
        }
        _ => return Ok(None),
    };
    Ok(Some(expr))
}
