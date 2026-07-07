use nuis_semantics::model::{AstExpr, NirExpr};

use crate::frontend::{i64_type, lower_expr, ref_type};

use super::DirectCallLoweringContext;

pub(super) fn lower_http_call(
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
        "parse_header_line" => {
            if current_domain != "cpu" {
                return Err(
                    "parse_header_line(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let [buffer, offset, len, expected_name] = args else {
                return Err("parse_header_line(...) expects 4 args".to_owned());
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
            let lowered_expected_name = lower_expr(
                expected_name,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            Ok(Some(NirExpr::CpuExternCall {
                abi: "c".to_owned(),
                interface: None,
                callee: "host_parse_header_line".to_owned(),
                args: vec![
                    NirExpr::HostBufferHandle(Box::new(lowered_buffer)),
                    lowered_offset,
                    lowered_len,
                    lowered_expected_name,
                ],
            }))
        }
        "find_header_value" => {
            if current_domain != "cpu" {
                return Err(
                    "find_header_value(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let [buffer, offset, len, expected_name] = args else {
                return Err("find_header_value(...) expects 4 args".to_owned());
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
            let lowered_expected_name = lower_expr(
                expected_name,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            Ok(Some(NirExpr::CpuExternCall {
                abi: "c".to_owned(),
                interface: None,
                callee: "host_find_header_value".to_owned(),
                args: vec![
                    NirExpr::HostBufferHandle(Box::new(lowered_buffer)),
                    lowered_offset,
                    lowered_len,
                    lowered_expected_name,
                ],
            }))
        }
        "find_status_line_reason" => {
            if current_domain != "cpu" {
                return Err(
                    "find_status_line_reason(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let [buffer, offset, len] = args else {
                return Err("find_status_line_reason(...) expects 3 args".to_owned());
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
                callee: "host_find_status_line_reason".to_owned(),
                args: vec![
                    NirExpr::HostBufferHandle(Box::new(lowered_buffer)),
                    lowered_offset,
                    lowered_len,
                ],
            }))
        }
        "parse_http_response_summary" => {
            if current_domain != "cpu" {
                return Err(
                    "parse_http_response_summary(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let [buffer, offset, len] = args else {
                return Err("parse_http_response_summary(...) expects 3 args".to_owned());
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
                callee: "host_parse_http_response_summary".to_owned(),
                args: vec![
                    NirExpr::HostBufferHandle(Box::new(lowered_buffer)),
                    lowered_offset,
                    lowered_len,
                ],
            }))
        }
        "parse_http_request_summary" => {
            if current_domain != "cpu" {
                return Err(
                    "parse_http_request_summary(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let [buffer, offset, len] = args else {
                return Err("parse_http_request_summary(...) expects 3 args".to_owned());
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
                callee: "host_parse_http_request_summary".to_owned(),
                args: vec![
                    NirExpr::HostBufferHandle(Box::new(lowered_buffer)),
                    lowered_offset,
                    lowered_len,
                ],
            }))
        }
        "parse_http_roundtrip_summary" => {
            if current_domain != "cpu" {
                return Err(
                    "parse_http_roundtrip_summary(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let [request_buffer, request_offset, request_len, response_buffer, response_offset, response_len] =
                args
            else {
                return Err("parse_http_roundtrip_summary(...) expects 6 args".to_owned());
            };
            let lowered_request_buffer = lower_expr(
                request_buffer,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&ref_type("Buffer")),
            )?;
            let lowered_request_offset = lower_expr(
                request_offset,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let lowered_request_len = lower_expr(
                request_len,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let lowered_response_buffer = lower_expr(
                response_buffer,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&ref_type("Buffer")),
            )?;
            let lowered_response_offset = lower_expr(
                response_offset,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let lowered_response_len = lower_expr(
                response_len,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(Some(NirExpr::CpuExternCall {
                abi: "c".to_owned(),
                interface: None,
                callee: "host_parse_http_roundtrip_summary".to_owned(),
                args: vec![
                    NirExpr::HostBufferHandle(Box::new(lowered_request_buffer)),
                    lowered_request_offset,
                    lowered_request_len,
                    NirExpr::HostBufferHandle(Box::new(lowered_response_buffer)),
                    lowered_response_offset,
                    lowered_response_len,
                ],
            }))
        }
        _ => Ok(None),
    }
}
