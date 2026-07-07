use nuis_semantics::model::{AstExpr, NirBinaryOp, NirExpr};

use crate::frontend::{i64_type, lower_expr, ref_type};

use super::DirectCallLoweringContext;

pub(super) fn lower_serialization_call(
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
        "text_handle" => {
            if current_domain != "cpu" {
                return Err(
                    "text_handle(...) is currently only allowed inside `mod cpu <unit>`".to_owned(),
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
        _ => Ok(None),
    }
}
