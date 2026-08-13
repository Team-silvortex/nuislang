use crate::frontend::is_host_execution_domain;

use nuis_semantics::model::{AstExpr, NirBinaryOp, NirExpr};

use crate::frontend::{i64_type, lower_expr, ref_type};

use super::DirectCallLoweringContext;

pub(super) fn lower_text_call(
    callee: &str,
    args: &[AstExpr],
    context: DirectCallLoweringContext<'_>,
) -> Result<Option<NirExpr>, String> {
    let DirectCallLoweringContext {
        current_domain,
        bindings,
        signatures,
        struct_table,
    } = context;
    match callee {
        "owned_utf8_len" => {
            if !is_host_execution_domain(current_domain) {
                return Err(
                    "owned_utf8_len(...) requires a host execution module (`mod cpu` or `mod cffi`)"
                        .to_owned(),
                );
            }
            let [text] = args else {
                return Err("owned_utf8_len(...) expects 1 arg".to_owned());
            };
            let lowered = lower_expr(
                text,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&ref_type("String")),
            )?;
            Ok(Some(NirExpr::BufferLen(Box::new(lowered))))
        }
        "owned_utf8_byte_at" => {
            if !is_host_execution_domain(current_domain) {
                return Err(
                    "owned_utf8_byte_at(...) requires a host execution module (`mod cpu` or `mod cffi`)"
                        .to_owned(),
                );
            }
            let [text, index] = args else {
                return Err("owned_utf8_byte_at(...) expects 2 args".to_owned());
            };
            let text = lower_expr(
                text,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&ref_type("String")),
            )?;
            let index = lower_expr(
                index,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(Some(NirExpr::LoadAt {
                buffer: Box::new(text),
                index: Box::new(index),
            }))
        }
        "deserialize_text_equals" => {
            if !is_host_execution_domain(current_domain) {
                return Err(
                    "deserialize_text_equals(...) requires a host execution module (`mod cpu` or `mod cffi`)"
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
            if !is_host_execution_domain(current_domain) {
                return Err(
                    "deserialize_text_starts_with(...) requires a host execution module (`mod cpu` or `mod cffi`)"
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
            if !is_host_execution_domain(current_domain) {
                return Err(
                    "deserialize_text_contains(...) requires a host execution module (`mod cpu` or `mod cffi`)"
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
            if !is_host_execution_domain(current_domain) {
                return Err(
                    "deserialize_text_ends_with(...) requires a host execution module (`mod cpu` or `mod cffi`)"
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
