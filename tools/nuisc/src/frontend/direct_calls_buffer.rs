use std::collections::BTreeMap;

use nuis_semantics::model::{AstExpr, NirExpr, NirStructDef, NirTypeRef};

use crate::frontend::{i64_type, lower_expr, ref_type, FunctionSignature};

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_buffer_call(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<Option<NirExpr>, String> {
    match callee {
        "buffer_find_byte" => {
            if current_domain != "cpu" {
                return Err(
                    "buffer_find_byte(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let [buffer, offset, len, needle] = args else {
                return Err("buffer_find_byte(...) expects 4 args".to_owned());
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
                Some(&i64_type()),
            )?;
            Ok(Some(NirExpr::CpuExternCall {
                abi: "c".to_owned(),
                interface: None,
                callee: "host_buffer_find_byte".to_owned(),
                args: vec![
                    NirExpr::HostBufferHandle(Box::new(lowered_buffer)),
                    lowered_offset,
                    lowered_len,
                    lowered_needle,
                ],
            }))
        }
        "buffer_find_text" => {
            if current_domain != "cpu" {
                return Err(
                    "buffer_find_text(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let [buffer, offset, len, needle] = args else {
                return Err("buffer_find_text(...) expects 4 args".to_owned());
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
            Ok(Some(NirExpr::CpuExternCall {
                abi: "c".to_owned(),
                interface: None,
                callee: "host_buffer_find_text".to_owned(),
                args: vec![
                    NirExpr::HostBufferHandle(Box::new(lowered_buffer)),
                    lowered_offset,
                    lowered_len,
                    lowered_needle,
                ],
            }))
        }
        "buffer_find_line_end" => {
            if current_domain != "cpu" {
                return Err(
                    "buffer_find_line_end(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let [buffer, offset, len] = args else {
                return Err("buffer_find_line_end(...) expects 3 args".to_owned());
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
            Ok(Some(NirExpr::CpuExternCall {
                abi: "c".to_owned(),
                interface: None,
                callee: "host_buffer_find_line_end".to_owned(),
                args: vec![
                    NirExpr::HostBufferHandle(Box::new(lowered_buffer)),
                    lowered_offset,
                    lowered_len,
                ],
            }))
        }
        "buffer_trim_line_end" => {
            if current_domain != "cpu" {
                return Err(
                    "buffer_trim_line_end(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let [buffer, offset, len] = args else {
                return Err("buffer_trim_line_end(...) expects 3 args".to_owned());
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
            Ok(Some(NirExpr::CpuExternCall {
                abi: "c".to_owned(),
                interface: None,
                callee: "host_buffer_trim_line_end".to_owned(),
                args: vec![
                    NirExpr::HostBufferHandle(Box::new(lowered_buffer)),
                    lowered_offset,
                    lowered_len,
                ],
            }))
        }
        _ => Ok(None),
    }
}
