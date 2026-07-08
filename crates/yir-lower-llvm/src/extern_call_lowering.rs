use std::collections::BTreeMap;

use yir_core::Node;

use super::{
    extern_abi::{
        is_builtin_host_ffi_symbol, lower_dynamic_extern_arg, lower_i32_extern_arg,
        lower_i64_extern_arg, render_extern_call,
    },
    fresh_reg, LlvmValueRef,
};

pub(crate) fn lower_cpu_extern_call_node(
    node: &Node,
    body: &mut Vec<String>,
    registers: &mut BTreeMap<String, LlvmValueRef>,
    next_reg: &mut usize,
    last_cpu_value: &mut Option<String>,
) -> Result<bool, String> {
    let return_ty = match (node.op.module.as_str(), node.op.instruction.as_str()) {
        ("cpu", "extern_call_i64") => "i64",
        ("cpu", "extern_call_i32") => "i32",
        _ => return Ok(false),
    };

    let abi = &node.op.args[0];
    let symbol = &node.op.args[1];
    if abi != "nurs" && abi != "c" && abi != "libc" {
        body.push(format!(
            "  ; deferred lowering for cpu.{} `{}` because ABI `{}` is not supported by the current LLVM bridge",
            node.op.instruction, node.name, abi
        ));
        return Ok(true);
    }

    let dynamic_args = abi == "libc" || !is_builtin_host_ffi_symbol(symbol);
    let lowered_args = node.op.args[2..]
        .iter()
        .map(|arg| {
            registers.get(arg).and_then(|value| {
                if dynamic_args {
                    lower_dynamic_extern_arg(value, body, next_reg)
                } else if return_ty == "i32" {
                    lower_i32_extern_arg(value, body, next_reg)
                } else {
                    lower_i64_extern_arg(value, body, next_reg)
                }
            })
        })
        .collect::<Option<Vec<_>>>();
    let Some(lowered_args) = lowered_args else {
        body.push(format!(
            "  ; deferred lowering for cpu.{} `{}` because one or more inputs are outside the current CPU LLVM slice",
            node.op.instruction, node.name
        ));
        return Ok(true);
    };

    let reg = fresh_reg(next_reg);
    let Some(call) = render_extern_call(return_ty, symbol, &lowered_args) else {
        body.push(format!(
            "  ; deferred lowering for cpu.{} `{}` because symbol `{}` has unsupported arity {}",
            node.op.instruction,
            node.name,
            symbol,
            lowered_args.len()
        ));
        return Ok(true);
    };
    body.push(format!("  {reg} = {call}"));

    if return_ty == "i64"
        && matches!(
            symbol.as_str(),
            "host_deserialize_text_from"
                | "host_parse_header_line"
                | "host_find_header_value"
                | "host_find_status_line_reason"
                | "host_parse_http_response_summary"
                | "host_parse_http_request_summary"
                | "host_parse_http_roundtrip_summary"
        )
    {
        let ptr = fresh_reg(next_reg);
        body.push(format!("  {ptr} = call ptr @nuis_host_text_ptr(i64 {reg})"));
        registers.insert(
            node.name.clone(),
            LlvmValueRef::TextHandle {
                ptr,
                handle: reg.clone(),
            },
        );
    } else if return_ty == "i32" {
        registers.insert(node.name.clone(), LlvmValueRef::I32(reg.clone()));
    } else {
        registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
    }
    *last_cpu_value = Some(reg);
    Ok(true)
}
