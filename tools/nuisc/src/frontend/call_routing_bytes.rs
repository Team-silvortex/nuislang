use std::collections::BTreeMap;

use super::call_routing_slice_helpers::{lower_byte_slice_parts, lower_host_compare_or_copy_call};
use super::expr_lowering::lower_expr;
use super::{i64_type, AstExpr, FunctionSignature, NirExpr, NirStructDef, NirTypeRef};

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_byte_builtin(
    callee: &str,
    generic_args: &[nuis_semantics::model::AstTypeRef],
    args: &[AstExpr],
    current_domain: &str,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<Option<NirExpr>, String> {
    if let Some(split_builtin) = super::call_routing_byte_splits::lower_byte_split_builtin(
        callee,
        generic_args,
        args,
        current_domain,
        bindings,
        signatures,
        struct_table,
    )? {
        return Ok(Some(split_builtin));
    }

    let expr = match callee {
        "fillbytes" => {
            let [target, value] = args else {
                return Err("fillbytes(...) expects 2 args".to_owned());
            };
            if !generic_args.is_empty() {
                return Err("fillbytes(...) does not accept explicit generic args".to_owned());
            }
            if current_domain != "cpu" {
                return Err(
                    "fillbytes(...) is currently only allowed inside `mod cpu <unit>`".to_owned(),
                );
            }
            let (buffer, start, len) =
                lower_byte_slice_parts(target, current_domain, bindings, signatures, struct_table)?;
            let lowered_value = lower_expr(
                value,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            NirExpr::CpuExternCall {
                abi: "c".to_owned(),
                interface: None,
                callee: "host_fill_bytes".to_owned(),
                args: vec![
                    NirExpr::HostBufferHandle(Box::new(buffer)),
                    start,
                    len,
                    lowered_value,
                ],
            }
        }
        "bytes_fill" => {
            let [target, value] = args else {
                return Err("bytes_fill(...) expects 2 args".to_owned());
            };
            if !generic_args.is_empty() {
                return Err("bytes_fill(...) does not accept explicit generic args".to_owned());
            }
            if current_domain != "cpu" {
                return Err(
                    "bytes_fill(...) is currently only allowed inside `mod cpu <unit>`".to_owned(),
                );
            }
            let (buffer, start, len) =
                lower_byte_slice_parts(target, current_domain, bindings, signatures, struct_table)?;
            let lowered_value = lower_expr(
                value,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            NirExpr::CpuExternCall {
                abi: "c".to_owned(),
                interface: None,
                callee: "host_fill_bytes".to_owned(),
                args: vec![
                    NirExpr::HostBufferHandle(Box::new(buffer)),
                    start,
                    len,
                    lowered_value,
                ],
            }
        }
        "copybytes" => {
            let [dst, src] = args else {
                return Err("copybytes(...) expects 2 args".to_owned());
            };
            if !generic_args.is_empty() {
                return Err("copybytes(...) does not accept explicit generic args".to_owned());
            }
            if current_domain != "cpu" {
                return Err(
                    "copybytes(...) is currently only allowed inside `mod cpu <unit>`".to_owned(),
                );
            }
            let (dst_buffer, dst_start, dst_len) =
                lower_byte_slice_parts(dst, current_domain, bindings, signatures, struct_table)?;
            let (src_buffer, src_start, src_len) =
                lower_byte_slice_parts(src, current_domain, bindings, signatures, struct_table)?;
            NirExpr::CpuExternCall {
                abi: "c".to_owned(),
                interface: None,
                callee: "host_copy_bytes".to_owned(),
                args: vec![
                    NirExpr::HostBufferHandle(Box::new(dst_buffer)),
                    dst_start,
                    dst_len,
                    NirExpr::HostBufferHandle(Box::new(src_buffer)),
                    src_start,
                    src_len,
                ],
            }
        }
        "bytes_copy_from" => {
            let [dst, src] = args else {
                return Err("bytes_copy_from(...) expects 2 args".to_owned());
            };
            if !generic_args.is_empty() {
                return Err("bytes_copy_from(...) does not accept explicit generic args".to_owned());
            }
            if current_domain != "cpu" {
                return Err(
                    "bytes_copy_from(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let (dst_buffer, dst_start, dst_len) =
                lower_byte_slice_parts(dst, current_domain, bindings, signatures, struct_table)?;
            let (src_buffer, src_start, src_len) =
                lower_byte_slice_parts(src, current_domain, bindings, signatures, struct_table)?;
            lower_host_compare_or_copy_call(
                "host_copy_bytes",
                dst_buffer,
                dst_start,
                dst_len,
                src_buffer,
                src_start,
                src_len,
            )
        }
        "comparebytes" => {
            let [lhs, rhs] = args else {
                return Err("comparebytes(...) expects 2 args".to_owned());
            };
            if !generic_args.is_empty() {
                return Err("comparebytes(...) does not accept explicit generic args".to_owned());
            }
            if current_domain != "cpu" {
                return Err(
                    "comparebytes(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let (lhs_buffer, lhs_start, lhs_len) =
                lower_byte_slice_parts(lhs, current_domain, bindings, signatures, struct_table)?;
            let (rhs_buffer, rhs_start, rhs_len) =
                lower_byte_slice_parts(rhs, current_domain, bindings, signatures, struct_table)?;
            NirExpr::CpuExternCall {
                abi: "c".to_owned(),
                interface: None,
                callee: "host_compare_bytes".to_owned(),
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
        "bytes_compare" => {
            let [lhs, rhs] = args else {
                return Err("bytes_compare(...) expects 2 args".to_owned());
            };
            if !generic_args.is_empty() {
                return Err("bytes_compare(...) does not accept explicit generic args".to_owned());
            }
            if current_domain != "cpu" {
                return Err(
                    "bytes_compare(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let (lhs_buffer, lhs_start, lhs_len) =
                lower_byte_slice_parts(lhs, current_domain, bindings, signatures, struct_table)?;
            let (rhs_buffer, rhs_start, rhs_len) =
                lower_byte_slice_parts(rhs, current_domain, bindings, signatures, struct_table)?;
            lower_host_compare_or_copy_call(
                "host_compare_bytes",
                lhs_buffer,
                lhs_start,
                lhs_len,
                rhs_buffer,
                rhs_start,
                rhs_len,
            )
        }
        "bytes_eq" => {
            let [lhs, rhs] = args else {
                return Err("bytes_eq(...) expects 2 args".to_owned());
            };
            if !generic_args.is_empty() {
                return Err("bytes_eq(...) does not accept explicit generic args".to_owned());
            }
            if current_domain != "cpu" {
                return Err(
                    "bytes_eq(...) is currently only allowed inside `mod cpu <unit>`".to_owned(),
                );
            }
            let (lhs_buffer, lhs_start, lhs_len) =
                lower_byte_slice_parts(lhs, current_domain, bindings, signatures, struct_table)?;
            let (rhs_buffer, rhs_start, rhs_len) =
                lower_byte_slice_parts(rhs, current_domain, bindings, signatures, struct_table)?;
            NirExpr::Binary {
                op: nuis_semantics::model::NirBinaryOp::Eq,
                lhs: Box::new(lower_host_compare_or_copy_call(
                    "host_compare_bytes",
                    lhs_buffer,
                    lhs_start,
                    lhs_len,
                    rhs_buffer,
                    rhs_start,
                    rhs_len,
                )),
                rhs: Box::new(NirExpr::Int(0)),
            }
        }
        "bytes_starts_with" => {
            let [base, prefix] = args else {
                return Err("bytes_starts_with(...) expects 2 args".to_owned());
            };
            if !generic_args.is_empty() {
                return Err(
                    "bytes_starts_with(...) does not accept explicit generic args".to_owned(),
                );
            }
            if current_domain != "cpu" {
                return Err(
                    "bytes_starts_with(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let (base_buffer, base_start, base_len) =
                lower_byte_slice_parts(base, current_domain, bindings, signatures, struct_table)?;
            let (prefix_buffer, prefix_start, prefix_len) =
                lower_byte_slice_parts(prefix, current_domain, bindings, signatures, struct_table)?;
            NirExpr::Binary {
                op: nuis_semantics::model::NirBinaryOp::And,
                lhs: Box::new(NirExpr::Binary {
                    op: nuis_semantics::model::NirBinaryOp::Ge,
                    lhs: Box::new(base_len.clone()),
                    rhs: Box::new(prefix_len.clone()),
                }),
                rhs: Box::new(NirExpr::Binary {
                    op: nuis_semantics::model::NirBinaryOp::Eq,
                    lhs: Box::new(lower_host_compare_or_copy_call(
                        "host_compare_bytes",
                        base_buffer,
                        base_start,
                        prefix_len.clone(),
                        prefix_buffer,
                        prefix_start,
                        prefix_len,
                    )),
                    rhs: Box::new(NirExpr::Int(0)),
                }),
            }
        }
        "bytes_ends_with" => {
            let [base, suffix] = args else {
                return Err("bytes_ends_with(...) expects 2 args".to_owned());
            };
            if !generic_args.is_empty() {
                return Err("bytes_ends_with(...) does not accept explicit generic args".to_owned());
            }
            if current_domain != "cpu" {
                return Err(
                    "bytes_ends_with(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let (base_buffer, base_start, base_len) =
                lower_byte_slice_parts(base, current_domain, bindings, signatures, struct_table)?;
            let (suffix_buffer, suffix_start, suffix_len) =
                lower_byte_slice_parts(suffix, current_domain, bindings, signatures, struct_table)?;
            let suffix_base_start = NirExpr::Binary {
                op: nuis_semantics::model::NirBinaryOp::Add,
                lhs: Box::new(base_start),
                rhs: Box::new(NirExpr::Binary {
                    op: nuis_semantics::model::NirBinaryOp::Sub,
                    lhs: Box::new(base_len.clone()),
                    rhs: Box::new(suffix_len.clone()),
                }),
            };
            NirExpr::Binary {
                op: nuis_semantics::model::NirBinaryOp::And,
                lhs: Box::new(NirExpr::Binary {
                    op: nuis_semantics::model::NirBinaryOp::Ge,
                    lhs: Box::new(base_len),
                    rhs: Box::new(suffix_len.clone()),
                }),
                rhs: Box::new(NirExpr::Binary {
                    op: nuis_semantics::model::NirBinaryOp::Eq,
                    lhs: Box::new(lower_host_compare_or_copy_call(
                        "host_compare_bytes",
                        base_buffer,
                        suffix_base_start,
                        suffix_len.clone(),
                        suffix_buffer,
                        suffix_start,
                        suffix_len,
                    )),
                    rhs: Box::new(NirExpr::Int(0)),
                }),
            }
        }
        "bytes_find_byte" => {
            let [base, needle] = args else {
                return Err("bytes_find_byte(...) expects 2 args".to_owned());
            };
            if !generic_args.is_empty() {
                return Err("bytes_find_byte(...) does not accept explicit generic args".to_owned());
            }
            if current_domain != "cpu" {
                return Err(
                    "bytes_find_byte(...) is currently only allowed inside `mod cpu <unit>`"
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
            }
        }
        "bytes_find_text" => {
            let [base, needle] = args else {
                return Err("bytes_find_text(...) expects 2 args".to_owned());
            };
            if !generic_args.is_empty() {
                return Err("bytes_find_text(...) does not accept explicit generic args".to_owned());
            }
            if current_domain != "cpu" {
                return Err(
                    "bytes_find_text(...) is currently only allowed inside `mod cpu <unit>`"
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
            }
        }
        "bytes_contains_byte" => {
            let [base, needle] = args else {
                return Err("bytes_contains_byte(...) expects 2 args".to_owned());
            };
            if !generic_args.is_empty() {
                return Err(
                    "bytes_contains_byte(...) does not accept explicit generic args".to_owned(),
                );
            }
            if current_domain != "cpu" {
                return Err(
                    "bytes_contains_byte(...) is currently only allowed inside `mod cpu <unit>`"
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
            NirExpr::Binary {
                op: nuis_semantics::model::NirBinaryOp::Ne,
                lhs: Box::new(NirExpr::CpuExternCall {
                    abi: "c".to_owned(),
                    interface: None,
                    callee: "host_buffer_find_byte".to_owned(),
                    args: vec![
                        NirExpr::HostBufferHandle(Box::new(buffer)),
                        start,
                        len,
                        lowered_needle,
                    ],
                }),
                rhs: Box::new(NirExpr::Int(-1)),
            }
        }
        "bytes_contains_text" => {
            let [base, needle] = args else {
                return Err("bytes_contains_text(...) expects 2 args".to_owned());
            };
            if !generic_args.is_empty() {
                return Err(
                    "bytes_contains_text(...) does not accept explicit generic args".to_owned(),
                );
            }
            if current_domain != "cpu" {
                return Err(
                    "bytes_contains_text(...) is currently only allowed inside `mod cpu <unit>`"
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
            NirExpr::Binary {
                op: nuis_semantics::model::NirBinaryOp::Ne,
                lhs: Box::new(NirExpr::CpuExternCall {
                    abi: "c".to_owned(),
                    interface: None,
                    callee: "host_buffer_find_text".to_owned(),
                    args: vec![
                        NirExpr::HostBufferHandle(Box::new(buffer)),
                        start,
                        len,
                        lowered_needle,
                    ],
                }),
                rhs: Box::new(NirExpr::Int(-1)),
            }
        }
        "bytes_find_line_end" => {
            let [base] = args else {
                return Err("bytes_find_line_end(...) expects 1 arg".to_owned());
            };
            if !generic_args.is_empty() {
                return Err(
                    "bytes_find_line_end(...) does not accept explicit generic args".to_owned(),
                );
            }
            if current_domain != "cpu" {
                return Err(
                    "bytes_find_line_end(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let (buffer, start, len) =
                lower_byte_slice_parts(base, current_domain, bindings, signatures, struct_table)?;
            NirExpr::CpuExternCall {
                abi: "c".to_owned(),
                interface: None,
                callee: "host_buffer_find_line_end".to_owned(),
                args: vec![NirExpr::HostBufferHandle(Box::new(buffer)), start, len],
            }
        }
        "bytes_trim_line_end" => {
            let [base] = args else {
                return Err("bytes_trim_line_end(...) expects 1 arg".to_owned());
            };
            if !generic_args.is_empty() {
                return Err(
                    "bytes_trim_line_end(...) does not accept explicit generic args".to_owned(),
                );
            }
            if current_domain != "cpu" {
                return Err(
                    "bytes_trim_line_end(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let (buffer, start, len) =
                lower_byte_slice_parts(base, current_domain, bindings, signatures, struct_table)?;
            NirExpr::CpuExternCall {
                abi: "c".to_owned(),
                interface: None,
                callee: "host_buffer_trim_line_end".to_owned(),
                args: vec![NirExpr::HostBufferHandle(Box::new(buffer)), start, len],
            }
        }
        _ => return Ok(None),
    };
    Ok(Some(expr))
}
