use nuis_semantics::model::{AstExpr, NirExpr};

use crate::frontend::{i64_type, lower_expr, named_type, ref_type};

use super::DirectCallLoweringContext;

pub(super) fn lower_buffer_call(
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
        "bytes_len" | "drop_bytes" => {
            if current_domain != "cpu" {
                return Err(format!(
                    "{callee}(...) is currently only allowed inside `mod cpu <unit>`"
                ));
            }
            let [bytes] = args else {
                return Err(format!("{callee}(...) expects 1 arg"));
            };
            let lowered = lower_expr(
                bytes,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("Bytes")),
            )?;
            Ok(Some(if callee == "bytes_len" {
                NirExpr::BytesLen(Box::new(lowered))
            } else {
                NirExpr::DropBytes(Box::new(lowered))
            }))
        }
        "copy_bytes" => {
            if current_domain != "cpu" {
                return Err(
                    "copy_bytes(...) is currently only allowed inside `mod cpu <unit>`".to_owned(),
                );
            }
            let [buffer] = args else {
                return Err("copy_bytes(...) expects 1 arg".to_owned());
            };
            let lowered = lower_expr(
                buffer,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&ref_type("Buffer")),
            )?;
            Ok(Some(NirExpr::CopyBufferOwned(Box::new(lowered))))
        }
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
