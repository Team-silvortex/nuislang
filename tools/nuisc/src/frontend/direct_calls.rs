use std::collections::BTreeMap;

use nuis_semantics::model::{AstExpr, NirBinaryOp, NirExpr, NirStructDef, NirTypeRef};

use super::call_helpers::{ensure_call_arg_matches_param, lower_extern_call_arg_for_param};
use super::{
    ensure_ref_like, i64_type, lower_expr, lower_nested_expr_with_async, ref_type,
    FunctionSignature, ModuleConstValue,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_direct_call_builtin_or_named_call(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    _module_consts: &BTreeMap<String, ModuleConstValue>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    allow_async_calls: bool,
) -> Result<Option<NirExpr>, String> {
    match callee {
        "text_handle" => {
            if current_domain != "cpu" {
                return Err(
                    "text_handle(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let [value] = args else {
                return Err("text_handle(...) expects 1 arg".to_owned());
            };
            let lowered = lower_expr(
                value,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            Ok(Some(NirExpr::CpuExternCall {
                abi: "c".to_owned(),
                interface: None,
                callee: "host_text_handle".to_owned(),
                args: vec![lowered],
            }))
        }
        "text_len" => {
            if current_domain != "cpu" {
                return Err(
                    "text_len(...) is currently only allowed inside `mod cpu <unit>`".to_owned(),
                );
            }
            let [value] = args else {
                return Err("text_len(...) expects 1 arg".to_owned());
            };
            let lowered = lower_expr(
                value,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            Ok(Some(NirExpr::CpuExternCall {
                abi: "c".to_owned(),
                interface: None,
                callee: "host_text_len".to_owned(),
                args: vec![lowered],
            }))
        }
        "serialize_text_into" => {
            if current_domain != "cpu" {
                return Err(
                    "serialize_text_into(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let [text, buffer, offset] = args else {
                return Err("serialize_text_into(...) expects 3 args".to_owned());
            };
            let lowered_text = lower_expr(
                text,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
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
            Ok(Some(NirExpr::CpuExternCall {
                abi: "c".to_owned(),
                interface: None,
                callee: "host_serialize_text_into".to_owned(),
                args: vec![
                    lowered_text,
                    NirExpr::HostBufferHandle(Box::new(lowered_buffer)),
                    lowered_offset,
                ],
            }))
        }
        "serialize_i64_into" => {
            if current_domain != "cpu" {
                return Err(
                    "serialize_i64_into(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let [value, buffer, offset] = args else {
                return Err("serialize_i64_into(...) expects 3 args".to_owned());
            };
            let lowered_value = lower_expr(
                value,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
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
            Ok(Some(NirExpr::CpuExternCall {
                abi: "c".to_owned(),
                interface: None,
                callee: "host_serialize_i64_into".to_owned(),
                args: vec![
                    lowered_value,
                    NirExpr::HostBufferHandle(Box::new(lowered_buffer)),
                    lowered_offset,
                ],
            }))
        }
        "serialize_bool_into" => {
            if current_domain != "cpu" {
                return Err(
                    "serialize_bool_into(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let [value, buffer, offset] = args else {
                return Err("serialize_bool_into(...) expects 3 args".to_owned());
            };
            let lowered_value = lower_expr(
                value,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
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
            Ok(Some(NirExpr::CpuExternCall {
                abi: "c".to_owned(),
                interface: None,
                callee: "host_serialize_bool_into".to_owned(),
                args: vec![
                    lowered_value,
                    NirExpr::HostBufferHandle(Box::new(lowered_buffer)),
                    lowered_offset,
                ],
            }))
        }
        "serialize_byte_into" => {
            if current_domain != "cpu" {
                return Err(
                    "serialize_byte_into(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let [value, buffer, offset] = args else {
                return Err("serialize_byte_into(...) expects 3 args".to_owned());
            };
            let lowered_value = lower_expr(
                value,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
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
            Ok(Some(NirExpr::CpuExternCall {
                abi: "c".to_owned(),
                interface: None,
                callee: "host_serialize_byte_into".to_owned(),
                args: vec![
                    lowered_value,
                    NirExpr::HostBufferHandle(Box::new(lowered_buffer)),
                    lowered_offset,
                ],
            }))
        }
        "deserialize_i64_from" => {
            if current_domain != "cpu" {
                return Err(
                    "deserialize_i64_from(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let [buffer, offset, len] = args else {
                return Err("deserialize_i64_from(...) expects 3 args".to_owned());
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
                callee: "host_deserialize_i64_from".to_owned(),
                args: vec![
                    NirExpr::HostBufferHandle(Box::new(lowered_buffer)),
                    lowered_offset,
                    lowered_len,
                ],
            }))
        }
        "deserialize_bool_from" => {
            if current_domain != "cpu" {
                return Err(
                    "deserialize_bool_from(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let [buffer, offset, len] = args else {
                return Err("deserialize_bool_from(...) expects 3 args".to_owned());
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
            let raw = NirExpr::CpuExternCall {
                abi: "c".to_owned(),
                interface: None,
                callee: "host_deserialize_bool_from".to_owned(),
                args: vec![
                    NirExpr::HostBufferHandle(Box::new(lowered_buffer)),
                    lowered_offset,
                    lowered_len,
                ],
            };
            Ok(Some(NirExpr::Binary {
                op: NirBinaryOp::Ne,
                lhs: Box::new(raw),
                rhs: Box::new(NirExpr::Int(0)),
            }))
        }
        "deserialize_byte_from" => {
            if current_domain != "cpu" {
                return Err(
                    "deserialize_byte_from(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let [buffer, offset] = args else {
                return Err("deserialize_byte_from(...) expects 2 args".to_owned());
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
            Ok(Some(NirExpr::CpuExternCall {
                abi: "c".to_owned(),
                interface: None,
                callee: "host_deserialize_byte_from".to_owned(),
                args: vec![
                    NirExpr::HostBufferHandle(Box::new(lowered_buffer)),
                    lowered_offset,
                ],
            }))
        }
        "deserialize_text_from" => {
            if current_domain != "cpu" {
                return Err(
                    "deserialize_text_from(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let [buffer, offset, len] = args else {
                return Err("deserialize_text_from(...) expects 3 args".to_owned());
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
                callee: "host_deserialize_text_from".to_owned(),
                args: vec![
                    NirExpr::HostBufferHandle(Box::new(lowered_buffer)),
                    lowered_offset,
                    lowered_len,
                ],
            }))
        }
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
        "free" => {
            let [value] = args else {
                return Err("free(...) expects 1 arg".to_owned());
            };
            let lowered = lower_expr(
                value,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            ensure_ref_like("free", &lowered, bindings, signatures, struct_table)?;
            Ok(Some(NirExpr::Free(Box::new(lowered))))
        }
        "is_null" => {
            let [value] = args else {
                return Err("is_null(...) expects 1 arg".to_owned());
            };
            let lowered = lower_expr(
                value,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            ensure_ref_like("is_null", &lowered, bindings, signatures, struct_table)?;
            Ok(Some(NirExpr::IsNull(Box::new(lowered))))
        }
        _ => lower_named_call(
            callee,
            args,
            current_domain,
            current_function_is_async,
            bindings,
            signatures,
            struct_table,
            allow_async_calls,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn lower_named_call(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    allow_async_calls: bool,
) -> Result<Option<NirExpr>, String> {
    let Some(signature) = signatures.get(callee) else {
        return Ok(None);
    };
    let lowered_args = args
        .iter()
        .zip(signature.params.iter())
        .map(|(arg, expected_param)| {
            lower_nested_expr_with_async(
                arg,
                current_domain,
                current_function_is_async,
                bindings,
                signatures,
                struct_table,
                Some(expected_param),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    if signature.params.len() != lowered_args.len() {
        return Err(format!(
            "function `{callee}` expects {} args, found {}",
            signature.params.len(),
            lowered_args.len()
        ));
    }
    for (index, (arg, expected_param)) in
        lowered_args.iter().zip(signature.params.iter()).enumerate()
    {
        ensure_call_arg_matches_param(
            callee,
            index,
            arg,
            expected_param,
            bindings,
            signatures,
            struct_table,
            signature.is_extern,
        )?;
    }
    if signature.is_async {
        if !current_function_is_async {
            return Err(format!(
                "async function `{callee}` can only be called inside `async fn`"
            ));
        }
        if !allow_async_calls {
            return Err(format!(
                "async function `{callee}` must be used under `await`"
            ));
        }
    }
    if signature.is_extern {
        if current_domain != "cpu" {
            return Err(format!(
                "extern call `{callee}` is currently only allowed inside `mod cpu <unit>`"
            ));
        }
        let lowered_args = lowered_args
            .into_iter()
            .zip(signature.params.iter())
            .map(|(arg, expected_param)| lower_extern_call_arg_for_param(arg, expected_param))
            .collect();
        return Ok(Some(NirExpr::CpuExternCall {
            abi: signature.abi.clone(),
            interface: None,
            callee: signature.symbol_name.clone(),
            args: lowered_args,
        }));
    }
    Ok(Some(NirExpr::Call {
        callee: signature.symbol_name.clone(),
        args: lowered_args,
    }))
}
