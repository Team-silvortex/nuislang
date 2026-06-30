use std::collections::BTreeMap;

use nuis_semantics::model::{AstExpr, NirBinaryOp, NirExpr, NirStructDef, NirTypeRef};

use crate::frontend::{i64_type, lower_expr, ref_type, FunctionSignature};

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_text_call(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<Option<NirExpr>, String> {
    match callee {
        "deserialize_text_equals" => {
            if current_domain != "cpu" {
                return Err(
                    "deserialize_text_equals(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let [buffer, offset, len, expected] = args else {
                return Err("deserialize_text_equals(...) expects 4 args".to_owned());
            };
            let lowered_buffer = lower_expr(
                buffer,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&ref_type("Buffer")),
            )?;
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
            let lowered_expected = lower_expr(
                expected,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            let raw = NirExpr::CpuExternCall {
                abi: "c".to_owned(),
                interface: None,
                callee: "host_deserialize_text_equals".to_owned(),
                args: vec![
                    NirExpr::HostBufferHandle(Box::new(lowered_buffer)),
                    lowered_offset,
                    lowered_len,
                    lowered_expected,
                ],
            };
            Ok(Some(NirExpr::Binary {
                op: NirBinaryOp::Ne,
                lhs: Box::new(raw),
                rhs: Box::new(NirExpr::Int(0)),
            }))
        }
        "deserialize_text_starts_with" => {
            if current_domain != "cpu" {
                return Err(
                    "deserialize_text_starts_with(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let [buffer, offset, len, prefix] = args else {
                return Err("deserialize_text_starts_with(...) expects 4 args".to_owned());
            };
            let lowered_buffer = lower_expr(
                buffer,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&ref_type("Buffer")),
            )?;
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
            let lowered_prefix = lower_expr(
                prefix,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            let raw = NirExpr::CpuExternCall {
                abi: "c".to_owned(),
                interface: None,
                callee: "host_deserialize_text_starts_with".to_owned(),
                args: vec![
                    NirExpr::HostBufferHandle(Box::new(lowered_buffer)),
                    lowered_offset,
                    lowered_len,
                    lowered_prefix,
                ],
            };
            Ok(Some(NirExpr::Binary {
                op: NirBinaryOp::Ne,
                lhs: Box::new(raw),
                rhs: Box::new(NirExpr::Int(0)),
            }))
        }
        "deserialize_text_contains" => {
            if current_domain != "cpu" {
                return Err(
                    "deserialize_text_contains(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let [buffer, offset, len, needle] = args else {
                return Err("deserialize_text_contains(...) expects 4 args".to_owned());
            };
            let lowered_buffer = lower_expr(
                buffer,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&ref_type("Buffer")),
            )?;
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
            let lowered_needle = lower_expr(
                needle,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            let raw = NirExpr::CpuExternCall {
                abi: "c".to_owned(),
                interface: None,
                callee: "host_deserialize_text_contains".to_owned(),
                args: vec![
                    NirExpr::HostBufferHandle(Box::new(lowered_buffer)),
                    lowered_offset,
                    lowered_len,
                    lowered_needle,
                ],
            };
            Ok(Some(NirExpr::Binary {
                op: NirBinaryOp::Ne,
                lhs: Box::new(raw),
                rhs: Box::new(NirExpr::Int(0)),
            }))
        }
        "deserialize_text_ends_with" => {
            if current_domain != "cpu" {
                return Err(
                    "deserialize_text_ends_with(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let [buffer, offset, len, suffix] = args else {
                return Err("deserialize_text_ends_with(...) expects 4 args".to_owned());
            };
            let lowered_buffer = lower_expr(
                buffer,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&ref_type("Buffer")),
            )?;
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
            let lowered_suffix = lower_expr(
                suffix,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            let raw = NirExpr::CpuExternCall {
                abi: "c".to_owned(),
                interface: None,
                callee: "host_deserialize_text_ends_with".to_owned(),
                args: vec![
                    NirExpr::HostBufferHandle(Box::new(lowered_buffer)),
                    lowered_offset,
                    lowered_len,
                    lowered_suffix,
                ],
            };
            Ok(Some(NirExpr::Binary {
                op: NirBinaryOp::Ne,
                lhs: Box::new(raw),
                rhs: Box::new(NirExpr::Int(0)),
            }))
        }
        _ => Ok(None),
    }
}
