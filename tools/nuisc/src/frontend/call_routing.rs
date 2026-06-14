use std::collections::BTreeMap;

use super::call_helpers::ensure_ref_like;
use super::data_builtins::lower_data_builtin_call;
use super::data_profile_builtins::lower_data_profile_builtin_call;
use super::expr_lowering::{lower_expr, lower_nested_expr_with_async_and_consts};
use super::kernel_builtins::lower_kernel_builtin_call;
use super::metadata::ModuleConstValue;
use super::network_builtins::lower_network_builtin_call;
use super::nova_builtins::lower_nova_builtin_call;
use super::shader_builtins::lower_shader_builtin_call;
use super::task_builtins::lower_task_builtin_call;
use super::{
    bool_type, compatible_types, i64_type, infer_nir_expr_type, ref_type, AstExpr,
    FunctionSignature, NirExpr, NirStructDef, NirTypeRef,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_routed_call_or_core_builtin(
    callee: &str,
    generic_args: &[nuis_semantics::model::AstTypeRef],
    args: &[AstExpr],
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    expected: Option<&NirTypeRef>,
) -> Result<Option<NirExpr>, String> {
    if let Some(task_builtin) = lower_task_builtin_call(
        callee,
        args,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
    )? {
        return Ok(Some(task_builtin));
    }
    if let Some(data_builtin) = lower_data_builtin_call(
        callee,
        args,
        current_domain,
        bindings,
        module_consts,
        signatures,
        struct_table,
        expected,
    )? {
        return Ok(Some(data_builtin));
    }
    if let Some(data_profile_builtin) = lower_data_profile_builtin_call(
        callee,
        args,
        current_domain,
        bindings,
        signatures,
        struct_table,
        expected,
    )? {
        return Ok(Some(data_profile_builtin));
    }
    if let Some(shader_builtin) = lower_shader_builtin_call(
        callee,
        args,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
    )? {
        return Ok(Some(shader_builtin));
    }
    if let Some(network_builtin) = lower_network_builtin_call(
        callee,
        args,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
    )? {
        return Ok(Some(network_builtin));
    }
    if let Some(kernel_builtin) = lower_kernel_builtin_call(
        callee,
        args,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
    )? {
        return Ok(Some(kernel_builtin));
    }
    if let Some(nova_builtin) = lower_nova_builtin_call(
        callee,
        args,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
    )? {
        return Ok(Some(nova_builtin));
    }

    let expr = match callee {
        "i32_from_i64" => {
            let [value] = args else {
                return Err("i32_from_i64(...) expects exactly one argument".to_owned());
            };
            let lowered = lower_nested_expr_with_async_and_consts(
                value,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let lowered_ty = infer_nir_expr_type(&lowered, bindings, signatures, struct_table)
                .ok_or_else(|| {
                    "i32_from_i64(...) requires an explicit integer input type".to_owned()
                })?;
            if lowered_ty != i64_type() {
                return Err(format!(
                    "i32_from_i64(...) expects `i64`, found `{}`",
                    lowered_ty.render()
                ));
            }
            NirExpr::CastI64ToI32(Box::new(lowered))
        }
        "null" => {
            if !args.is_empty() {
                return Err("null() expects 0 args".to_owned());
            }
            if let Some(expected) = expected {
                if !expected.is_ref {
                    return Err("null() currently requires an expected `ref` type".to_owned());
                }
            }
            NirExpr::Null
        }
        "borrow" => {
            let [value] = args else {
                return Err("borrow(...) expects 1 arg".to_owned());
            };
            let lowered = lower_expr(
                value,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            ensure_ref_like("borrow", &lowered, bindings, signatures, struct_table)?;
            NirExpr::Borrow(Box::new(lowered))
        }
        "borrow_end" => {
            let [value] = args else {
                return Err("borrow_end(...) expects 1 arg".to_owned());
            };
            let lowered = lower_expr(
                value,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            ensure_ref_like("borrow_end", &lowered, bindings, signatures, struct_table)?;
            NirExpr::BorrowEnd(Box::new(lowered))
        }
        "move" => {
            let [value] = args else {
                return Err("move(...) expects 1 arg".to_owned());
            };
            let lowered = lower_expr(
                value,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            ensure_ref_like("move", &lowered, bindings, signatures, struct_table)?;
            NirExpr::Move(Box::new(lowered))
        }
        "alloc_node" => {
            let [value, next] = args else {
                return Err("alloc_node(...) expects 2 args".to_owned());
            };
            let lowered_value = lower_expr(
                value,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let lowered_next = lower_expr(
                next,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&ref_type("Node")),
            )?;
            NirExpr::AllocNode {
                value: Box::new(lowered_value),
                next: Box::new(lowered_next),
            }
        }
        "alloc_buffer" => {
            let [len, fill] = args else {
                return Err("alloc_buffer(...) expects 2 args".to_owned());
            };
            NirExpr::AllocBuffer {
                len: Box::new(lower_expr(
                    len,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&i64_type()),
                )?),
                fill: Box::new(lower_expr(
                    fill,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&i64_type()),
                )?),
            }
        }
        "load_value" => {
            let [ptr] = args else {
                return Err("load_value(...) expects 1 arg".to_owned());
            };
            let lowered = lower_expr(
                ptr,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&ref_type("Node")),
            )?;
            NirExpr::LoadValue(Box::new(lowered))
        }
        "load_next" => {
            let [ptr] = args else {
                return Err("load_next(...) expects 1 arg".to_owned());
            };
            let lowered = lower_expr(
                ptr,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&ref_type("Node")),
            )?;
            NirExpr::LoadNext(Box::new(lowered))
        }
        "buffer_len" => {
            let [ptr] = args else {
                return Err("buffer_len(...) expects 1 arg".to_owned());
            };
            let lowered = lower_expr(
                ptr,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&ref_type("Buffer")),
            )?;
            NirExpr::BufferLen(Box::new(lowered))
        }
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
        "host_buffer_handle" => {
            let [buffer] = args else {
                return Err("host_buffer_handle(...) expects 1 arg".to_owned());
            };
            let lowered = lower_expr(
                buffer,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&ref_type("Buffer")),
            )?;
            NirExpr::HostBufferHandle(Box::new(lowered))
        }
        "load_at" => {
            let [target, index] = args else {
                return Err("load_at(...) expects 2 args".to_owned());
            };
            let (buffer, base_index, payload_ty) = lower_slice_or_buffer_access_target(
                target,
                current_domain,
                bindings,
                signatures,
                struct_table,
            )?;
            let raw = NirExpr::LoadAt {
                buffer: Box::new(buffer),
                index: Box::new(NirExpr::Binary {
                    op: nuis_semantics::model::NirBinaryOp::Add,
                    lhs: Box::new(base_index),
                    rhs: Box::new(lower_expr(
                        index,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )?),
                }),
            };
            match payload_ty.name.as_str() {
                "i64" => raw,
                "i32" => NirExpr::CastI64ToI32(Box::new(raw)),
                "bool" => NirExpr::CastI64ToBool(Box::new(raw)),
                "f32" => NirExpr::CastI64ToF32(Box::new(raw)),
                "f64" => NirExpr::CastI64ToF64(Box::new(raw)),
                _ => {
                    return Err(format!(
                        "slice element loads currently support only `i64`, `i32`, `bool`, `f32`, and `f64`, found `Slice<{}>`",
                        payload_ty.render()
                    ))
                }
            }
        }
        "store_value" => {
            let [target, value] = args else {
                return Err("store_value(...) expects 2 args".to_owned());
            };
            NirExpr::StoreValue {
                target: Box::new(lower_expr(
                    target,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&ref_type("Node")),
                )?),
                value: Box::new(lower_expr(
                    value,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&i64_type()),
                )?),
            }
        }
        "store_next" => {
            let [target, next] = args else {
                return Err("store_next(...) expects 2 args".to_owned());
            };
            NirExpr::StoreNext {
                target: Box::new(lower_expr(
                    target,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&ref_type("Node")),
                )?),
                next: Box::new(lower_expr(
                    next,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&ref_type("Node")),
                )?),
            }
        }
        "store_at" => {
            let [target, index, value] = args else {
                return Err("store_at(...) expects 3 args".to_owned());
            };
            let (buffer, base_index, payload_ty) = lower_slice_or_buffer_access_target(
                target,
                current_domain,
                bindings,
                signatures,
                struct_table,
            )?;
            let lowered_value = lower_expr(
                value,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&payload_ty),
            )?;
            let stored_value = match payload_ty.name.as_str() {
                "i64" => lowered_value,
                "i32" => NirExpr::CastI32ToI64(Box::new(lowered_value)),
                "bool" => NirExpr::CastBoolToI64(Box::new(lowered_value)),
                "f32" => NirExpr::CastF32ToI64(Box::new(lowered_value)),
                "f64" => NirExpr::CastF64ToI64(Box::new(lowered_value)),
                _ => {
                    return Err(format!(
                        "slice element stores currently support only `i64`, `i32`, `bool`, `f32`, and `f64`, found `Slice<{}>`",
                        payload_ty.render()
                    ))
                }
            };
            NirExpr::StoreAt {
                buffer: Box::new(buffer),
                index: Box::new(NirExpr::Binary {
                    op: nuis_semantics::model::NirBinaryOp::Add,
                    lhs: Box::new(base_index),
                    rhs: Box::new(lower_expr(
                        index,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )?),
                }),
                value: Box::new(stored_value),
            }
        }
        "cpu_bind_core" => {
            let [core] = args else {
                return Err("cpu_bind_core(...) expects 1 arg".to_owned());
            };
            let AstExpr::Int(core_index) = core else {
                return Err("cpu_bind_core(...) currently expects an integer literal".to_owned());
            };
            NirExpr::CpuBindCore(*core_index)
        }
        "cpu_window" => {
            let [width, height, title] = args else {
                return Err("cpu_window(...) expects 3 args".to_owned());
            };
            let AstExpr::Int(width) = width else {
                return Err("cpu_window(...) width must be an integer literal".to_owned());
            };
            let AstExpr::Int(height) = height else {
                return Err("cpu_window(...) height must be an integer literal".to_owned());
            };
            let AstExpr::Text(title) = title else {
                return Err("cpu_window(...) title must be a string literal".to_owned());
            };
            NirExpr::CpuWindow {
                width: *width,
                height: *height,
                title: title.clone(),
            }
        }
        "cpu_input_i64" => match args {
            [channel, default] | [channel, default, ..] => {
                let AstExpr::Text(channel) = channel else {
                    return Err("cpu_input_i64(...) channel must be a string literal".to_owned());
                };
                let AstExpr::Int(default) = default else {
                    return Err("cpu_input_i64(...) default must be an integer literal".to_owned());
                };
                let (min, max, step) = match args {
                    [_, _, min, max, step] => {
                        let AstExpr::Int(min) = min else {
                            return Err(
                                "cpu_input_i64(...) min must be an integer literal".to_owned()
                            );
                        };
                        let AstExpr::Int(max) = max else {
                            return Err(
                                "cpu_input_i64(...) max must be an integer literal".to_owned()
                            );
                        };
                        let AstExpr::Int(step) = step else {
                            return Err(
                                "cpu_input_i64(...) step must be an integer literal".to_owned()
                            );
                        };
                        (Some(*min), Some(*max), Some(*step))
                    }
                    [_, _] => (None, None, None),
                    _ => return Err("cpu_input_i64(...) expects 2 args or 5 args".to_owned()),
                };
                NirExpr::CpuInputI64 {
                    channel: channel.clone(),
                    default: *default,
                    min,
                    max,
                    step,
                }
            }
            _ => return Err("cpu_input_i64(...) expects 2 args or 5 args".to_owned()),
        },
        "cpu_tick_i64" => {
            let [start, step] = args else {
                return Err("cpu_tick_i64(...) expects 2 args".to_owned());
            };
            let AstExpr::Int(start) = start else {
                return Err("cpu_tick_i64(...) start must be an integer literal".to_owned());
            };
            let AstExpr::Int(step) = step else {
                return Err("cpu_tick_i64(...) step must be an integer literal".to_owned());
            };
            NirExpr::CpuTickI64 {
                start: *start,
                step: *step,
            }
        }
        "cpu_present_frame" => {
            let [frame] = args else {
                return Err("cpu_present_frame(...) expects 1 arg".to_owned());
            };
            NirExpr::CpuPresentFrame(Box::new(lower_expr(
                frame,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?))
        }
        _ => return Ok(None),
    };

    Ok(Some(expr))
}

fn slice_payload_type(ty: &NirTypeRef) -> Option<NirTypeRef> {
    (ty.name == "Slice" && !ty.is_ref && !ty.is_optional && ty.generic_args.len() == 1)
        .then(|| ty.generic_args[0].clone())
}

fn lower_optional_explicit_slice_payload_type(
    generic_args: &[nuis_semantics::model::AstTypeRef],
) -> Result<Option<NirTypeRef>, String> {
    match generic_args {
        [] => Ok(None),
        [payload] => Ok(Some(super::lower_type_ref(payload))),
        _ => Err("slice/subslice builtins accept at most 1 explicit generic arg".to_owned()),
    }
}

fn lower_slice_payload_type(
    generic_args: &[nuis_semantics::model::AstTypeRef],
) -> Result<NirTypeRef, String> {
    let explicit = lower_optional_explicit_slice_payload_type(generic_args)?;
    let payload = explicit.unwrap_or_else(i64_type);
    if payload != i64_type()
        && payload != super::i32_type()
        && payload != bool_type()
        && payload != super::f32_type()
        && payload != super::f64_type()
    {
        return Err(format!(
            "slice<...>(...) currently supports only `Slice<i64>`, `Slice<i32>`, `Slice<bool>`, `Slice<f32>`, and `Slice<f64>`, found `Slice<{}>`",
            payload.render()
        ));
    }
    Ok(payload)
}

fn lower_slice_or_buffer_access_target(
    target: &AstExpr,
    current_domain: &str,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<(NirExpr, NirExpr, NirTypeRef), String> {
    let lowered = lower_expr(
        target,
        current_domain,
        bindings,
        signatures,
        struct_table,
        None,
    )?;
    let lowered_ty = infer_nir_expr_type(&lowered, bindings, signatures, struct_table)
        .ok_or_else(|| "load/store target requires an explicit buffer-like type".to_owned())?;
    if compatible_types(&ref_type("Buffer"), &lowered_ty) {
        return Ok((lowered, NirExpr::Int(0), i64_type()));
    }
    if let Some(payload_ty) = slice_payload_type(&lowered_ty) {
        return Ok((
            NirExpr::FieldAccess {
                base: Box::new(lowered.clone()),
                field: "buffer".to_owned(),
            },
            NirExpr::FieldAccess {
                base: Box::new(lowered),
                field: "start".to_owned(),
            },
            payload_ty,
        ));
    }
    Err(format!(
        "load/store target expects `ref Buffer` or `Slice<...>`, found `{}`",
        lowered_ty.render()
    ))
}

fn lower_byte_slice_parts(
    target: &AstExpr,
    current_domain: &str,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<(NirExpr, NirExpr, NirExpr), String> {
    let lowered = lower_expr(
        target,
        current_domain,
        bindings,
        signatures,
        struct_table,
        None,
    )?;
    let lowered_ty = infer_nir_expr_type(&lowered, bindings, signatures, struct_table)
        .ok_or_else(|| "byte view builtin requires a typed `Slice<i64>` input".to_owned())?;
    let payload_ty = slice_payload_type(&lowered_ty).ok_or_else(|| {
        format!(
            "byte view builtin expects `Slice<i64>`, found `{}`",
            lowered_ty.render()
        )
    })?;
    if payload_ty != i64_type() {
        return Err(format!(
            "byte view builtin expects `Slice<i64>`, found `Slice<{}>`",
            payload_ty.render()
        ));
    }
    Ok((
        NirExpr::FieldAccess {
            base: Box::new(lowered.clone()),
            field: "buffer".to_owned(),
        },
        NirExpr::FieldAccess {
            base: Box::new(lowered.clone()),
            field: "start".to_owned(),
        },
        NirExpr::FieldAccess {
            base: Box::new(lowered),
            field: "len".to_owned(),
        },
    ))
}

fn ensure_byte_slice_input(
    builtin: &str,
    lowered: &NirExpr,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<(), String> {
    let lowered_ty = infer_nir_expr_type(lowered, bindings, signatures, struct_table)
        .ok_or_else(|| format!("{builtin}(...) requires a typed `Slice<i64>` input"))?;
    let payload_ty = slice_payload_type(&lowered_ty).ok_or_else(|| {
        format!(
            "{builtin}(...) expects `Slice<i64>`, found `{}`",
            lowered_ty.render()
        )
    })?;
    if payload_ty != i64_type() {
        return Err(format!(
            "{builtin}(...) expects `Slice<i64>`, found `Slice<{}>`",
            payload_ty.render()
        ));
    }
    Ok(())
}

fn lower_host_compare_or_copy_call(
    callee: &str,
    lhs_buffer: NirExpr,
    lhs_start: NirExpr,
    lhs_len: NirExpr,
    rhs_buffer: NirExpr,
    rhs_start: NirExpr,
    rhs_len: NirExpr,
) -> NirExpr {
    NirExpr::CpuExternCall {
        abi: "c".to_owned(),
        interface: None,
        callee: callee.to_owned(),
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

fn lower_byte_split_struct(
    buffer: NirExpr,
    start: NirExpr,
    len: NirExpr,
    delimiter_len: NirExpr,
    index: NirExpr,
) -> NirExpr {
    let found = NirExpr::Binary {
        op: nuis_semantics::model::NirBinaryOp::Ne,
        lhs: Box::new(index.clone()),
        rhs: Box::new(NirExpr::Int(-1)),
    };
    let found_i64 = NirExpr::CastBoolToI64(Box::new(found.clone()));
    let missing_i64 = NirExpr::Binary {
        op: nuis_semantics::model::NirBinaryOp::Sub,
        lhs: Box::new(NirExpr::Int(1)),
        rhs: Box::new(found_i64.clone()),
    };
    let base_end = NirExpr::Binary {
        op: nuis_semantics::model::NirBinaryOp::Add,
        lhs: Box::new(start.clone()),
        rhs: Box::new(len.clone()),
    };
    let split_after_start = NirExpr::Binary {
        op: nuis_semantics::model::NirBinaryOp::Add,
        lhs: Box::new(index.clone()),
        rhs: Box::new(delimiter_len),
    };
    let before_len = NirExpr::Binary {
        op: nuis_semantics::model::NirBinaryOp::Add,
        lhs: Box::new(NirExpr::Binary {
            op: nuis_semantics::model::NirBinaryOp::Mul,
            lhs: Box::new(found_i64.clone()),
            rhs: Box::new(NirExpr::Binary {
                op: nuis_semantics::model::NirBinaryOp::Sub,
                lhs: Box::new(index.clone()),
                rhs: Box::new(start.clone()),
            }),
        }),
        rhs: Box::new(NirExpr::Binary {
            op: nuis_semantics::model::NirBinaryOp::Mul,
            lhs: Box::new(missing_i64.clone()),
            rhs: Box::new(len.clone()),
        }),
    };
    let after_start = NirExpr::Binary {
        op: nuis_semantics::model::NirBinaryOp::Add,
        lhs: Box::new(NirExpr::Binary {
            op: nuis_semantics::model::NirBinaryOp::Mul,
            lhs: Box::new(found_i64.clone()),
            rhs: Box::new(split_after_start.clone()),
        }),
        rhs: Box::new(NirExpr::Binary {
            op: nuis_semantics::model::NirBinaryOp::Mul,
            lhs: Box::new(missing_i64.clone()),
            rhs: Box::new(base_end.clone()),
        }),
    };
    let after_len = NirExpr::Binary {
        op: nuis_semantics::model::NirBinaryOp::Mul,
        lhs: Box::new(found_i64),
        rhs: Box::new(NirExpr::Binary {
            op: nuis_semantics::model::NirBinaryOp::Sub,
            lhs: Box::new(base_end),
            rhs: Box::new(split_after_start),
        }),
    };

    NirExpr::StructLiteral {
        type_name: "ByteSplit".to_owned(),
        type_args: vec![],
        fields: vec![
            ("found".to_owned(), found),
            ("index".to_owned(), index),
            (
                "before".to_owned(),
                make_byte_slice_expr(buffer.clone(), start.clone(), before_len),
            ),
            (
                "after".to_owned(),
                make_byte_slice_expr(buffer, after_start, after_len),
            ),
        ],
    }
}

fn make_byte_slice_expr(buffer: NirExpr, start: NirExpr, len: NirExpr) -> NirExpr {
    NirExpr::StructLiteral {
        type_name: "Slice".to_owned(),
        type_args: vec![i64_type()],
        fields: vec![
            ("buffer".to_owned(), buffer),
            ("start".to_owned(), start),
            ("len".to_owned(), len),
        ],
    }
}
