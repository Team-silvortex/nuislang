use std::collections::BTreeMap;

use super::call_helpers::ensure_ref_like;
use super::data_builtins::{lower_data_builtin_call, DataBuiltinInput};
use super::data_profile_builtins::{lower_data_profile_builtin_call, DataProfileBuiltinInput};
use super::expr_lowering::lower_expr;
use super::kernel_builtins::{lower_kernel_builtin_call, KernelBuiltinInput};
use super::metadata::ModuleConstValue;
use super::network_builtins::{lower_network_builtin_call, NetworkBuiltinInput};
use super::nova_builtins::{lower_nova_builtin_call, NovaBuiltinInput};
use super::shader_builtins::{lower_shader_builtin_call, ShaderBuiltinInput};
use super::task_builtins::{lower_task_builtin_call, TaskBuiltinInput};
use super::{
    i64_type, infer_nir_expr_type, lower_nested_expr_with_async_and_consts, ref_type, AstExpr,
    FunctionSignature, NestedExprWithConstsInput, NirExpr, NirStructDef, NirTypeRef,
};

pub(super) struct RoutedCallLoweringInput<'a> {
    pub(super) callee: &'a str,
    pub(super) generic_args: &'a [nuis_semantics::model::AstTypeRef],
    pub(super) args: &'a [AstExpr],
    pub(super) current_domain: &'a str,
    pub(super) current_function_is_async: bool,
    pub(super) bindings: &'a BTreeMap<String, NirTypeRef>,
    pub(super) module_consts: &'a BTreeMap<String, ModuleConstValue>,
    pub(super) signatures: &'a BTreeMap<String, FunctionSignature>,
    pub(super) struct_table: &'a BTreeMap<String, NirStructDef>,
    pub(super) expected: Option<&'a NirTypeRef>,
}

pub(super) fn lower_routed_call_or_core_builtin(
    input: RoutedCallLoweringInput<'_>,
) -> Result<Option<NirExpr>, String> {
    let RoutedCallLoweringInput {
        callee,
        generic_args,
        args,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
        expected,
    } = input;
    if let Some(task_builtin) = lower_task_builtin_call(TaskBuiltinInput {
        callee,
        args,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
    })? {
        return Ok(Some(task_builtin));
    }
    if let Some(data_builtin) = lower_data_builtin_call(DataBuiltinInput {
        callee,
        args,
        current_domain,
        bindings,
        module_consts,
        signatures,
        struct_table,
        expected,
    })? {
        return Ok(Some(data_builtin));
    }
    if let Some(data_profile_builtin) = lower_data_profile_builtin_call(DataProfileBuiltinInput {
        callee,
        args,
        current_domain,
        bindings,
        signatures,
        struct_table,
        expected,
    })? {
        return Ok(Some(data_profile_builtin));
    }
    if let Some(shader_builtin) = lower_shader_builtin_call(ShaderBuiltinInput {
        callee,
        args,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
    })? {
        return Ok(Some(shader_builtin));
    }
    if let Some(network_builtin) = lower_network_builtin_call(NetworkBuiltinInput {
        callee,
        args,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
    })? {
        return Ok(Some(network_builtin));
    }
    if let Some(kernel_builtin) = lower_kernel_builtin_call(KernelBuiltinInput {
        callee,
        args,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
    })? {
        return Ok(Some(kernel_builtin));
    }
    if let Some(nova_builtin) = lower_nova_builtin_call(NovaBuiltinInput {
        callee,
        args,
        current_domain,
        bindings,
        signatures,
        struct_table,
    })? {
        return Ok(Some(nova_builtin));
    }

    if let Some(slice_builtin) = super::call_routing_slices::lower_slice_or_byte_builtin(
        super::call_routing_slices::SliceCallRoutingInput {
            callee,
            generic_args,
            args,
            current_domain,
            bindings,
            signatures,
            struct_table,
        },
    )? {
        return Ok(Some(slice_builtin));
    }

    let expr = match callee {
        "i32_from_i64" => {
            let [value] = args else {
                return Err("i32_from_i64(...) expects exactly one argument".to_owned());
            };
            let lowered = lower_nested_expr_with_async_and_consts(NestedExprWithConstsInput {
                expr: value,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
                expected: Some(&i64_type()),
            })?;
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
            let (buffer, base_index, payload_ty) =
                super::call_routing_slice_helpers::lower_slice_or_buffer_access_target(
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
            let (buffer, base_index, payload_ty) =
                super::call_routing_slice_helpers::lower_slice_or_buffer_access_target(
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
