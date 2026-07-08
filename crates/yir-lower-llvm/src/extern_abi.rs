use std::collections::BTreeMap;

use yir_core::{Node, YirModule};

use super::{coerce_to_i32, coerce_to_i64, LlvmValueRef};

pub(crate) fn render_dynamic_extern_decls(module: &YirModule) -> Vec<String> {
    let producer_types = module
        .nodes
        .iter()
        .map(|node| (node.name.as_str(), node_result_llvm_abi_type(node)))
        .collect::<BTreeMap<_, _>>();
    let mut declared = BTreeMap::<String, (&'static str, Vec<&'static str>)>::new();
    for node in &module.nodes {
        if node.op.module != "cpu" || !is_cpu_extern_call_instruction(&node.op.instruction) {
            continue;
        }
        if node.op.args.len() < 2 {
            continue;
        }
        let abi = node.op.args[0].as_str();
        if abi != "c" && abi != "nurs" && abi != "libc" {
            continue;
        }
        let symbol = node.op.args[1].clone();
        if is_builtin_host_ffi_symbol(&symbol) {
            continue;
        }
        let return_ty = cpu_extern_call_llvm_return_type(&node.op.instruction);
        let arg_types = node.op.args[2..]
            .iter()
            .map(|arg| producer_types.get(arg.as_str()).copied().unwrap_or("i64"))
            .collect::<Vec<_>>();
        declared.entry(symbol).or_insert((return_ty, arg_types));
    }
    declared
        .into_iter()
        .map(|(symbol, (return_ty, arg_types))| {
            let signature = if arg_types.is_empty() {
                String::new()
            } else {
                arg_types.join(", ")
            };
            format!("declare {return_ty} @{symbol}({signature})")
        })
        .collect()
}

pub(crate) fn lower_dynamic_extern_arg(
    value: &LlvmValueRef,
    body: &mut Vec<String>,
    next_reg: &mut usize,
) -> Option<String> {
    match value {
        LlvmValueRef::Ptr(value) | LlvmValueRef::TextHandle { ptr: value, .. } => {
            Some(format!("ptr {value}"))
        }
        LlvmValueRef::I32(_) => {
            coerce_to_i32(value, body, next_reg).map(|value| format!("i32 {value}"))
        }
        _ => coerce_to_i64(value, body, next_reg).map(|value| format!("i64 {value}")),
    }
}

pub(crate) fn lower_i64_extern_arg(
    value: &LlvmValueRef,
    body: &mut Vec<String>,
    next_reg: &mut usize,
) -> Option<String> {
    coerce_to_i64(value, body, next_reg).map(|value| format!("i64 {value}"))
}

pub(crate) fn lower_i32_extern_arg(
    value: &LlvmValueRef,
    body: &mut Vec<String>,
    next_reg: &mut usize,
) -> Option<String> {
    coerce_to_i32(value, body, next_reg).map(|value| format!("i32 {value}"))
}

pub(crate) fn render_extern_call(
    return_ty: &str,
    symbol: &str,
    lowered_args: &[String],
) -> Option<String> {
    if lowered_args.len() > 6 {
        return None;
    }
    Some(format!(
        "call {return_ty} @{symbol}({})",
        lowered_args.join(", ")
    ))
}

pub(crate) fn is_cpu_extern_call_instruction(instruction: &str) -> bool {
    matches!(instruction, "extern_call_i64" | "extern_call_i32")
}

pub(crate) fn cpu_extern_call_llvm_return_type(instruction: &str) -> &'static str {
    match instruction {
        "extern_call_i32" => "i32",
        _ => "i64",
    }
}

pub(crate) fn is_builtin_host_ffi_symbol(symbol: &str) -> bool {
    matches!(
        symbol,
        "malloc"
            | "free"
            | "puts"
            | "nuis_debug_print_bool"
            | "nuis_debug_print_i32"
            | "nuis_debug_print_i64"
            | "nuis_debug_print_f32"
            | "nuis_debug_print_f64"
            | "host_color_bias"
            | "host_speed_curve"
            | "host_radius_curve"
            | "host_mix_tick"
            | "HostRenderCurves__color_bias"
            | "HostRenderCurves__speed_curve"
            | "HostRenderCurves__radius_curve"
            | "HostRenderCurves__mix_tick"
            | "HostMath__speed_curve"
            | "host_argv_count"
            | "host_argv_at"
            | "host_env_has"
            | "host_env_get"
            | "host_file_open"
            | "host_file_read"
            | "host_file_write"
            | "host_file_close"
            | "host_text_handle"
            | "host_text_len"
            | "host_text_line_count"
            | "host_text_word_count"
            | "host_text_concat"
            | "host_stdout_write"
            | "host_stdout_flush"
            | "host_stderr_write"
            | "host_stderr_flush"
            | "host_serialize_i64_into"
            | "host_serialize_text_into"
            | "host_serialize_bool_into"
            | "host_serialize_byte_into"
            | "host_deserialize_i64_from"
            | "host_deserialize_bool_from"
            | "host_deserialize_byte_from"
            | "host_deserialize_text_from"
            | "host_monotonic_time_ns"
    )
}

pub(crate) fn node_result_llvm_abi_type(node: &Node) -> &'static str {
    if node.op.module != "cpu" {
        return "i64";
    }
    match node.op.instruction.as_str() {
        "text" | "null" | "borrow" | "move_ptr" | "alloc_node" | "alloc_buffer" | "load_next" => {
            "ptr"
        }
        "const_i32" | "cast_i64_to_i32" | "extern_call_i32" | "call_i32" | "param_i32" => "i32",
        _ => "i64",
    }
}
