use std::collections::BTreeMap;

use yir_core::Node;

use super::{
    extern_abi::{
        is_builtin_host_ffi_symbol, lower_dynamic_extern_arg, lower_i32_extern_arg,
        lower_i64_extern_arg, render_extern_call,
    },
    fresh_block, fresh_reg, LlvmValueRef,
};

pub(crate) fn lower_cpu_extern_call_node(
    node: &Node,
    body: &mut Vec<String>,
    registers: &mut BTreeMap<String, LlvmValueRef>,
    buffer_lengths: &mut BTreeMap<String, String>,
    next_reg: &mut usize,
    next_block: &mut usize,
    last_cpu_value: &mut Option<String>,
) -> Result<bool, String> {
    if node.op.module == "cpu" && node.op.instruction == "extern_call_owned_buffer" {
        return lower_owned_buffer_call(
            node,
            body,
            registers,
            buffer_lengths,
            next_reg,
            next_block,
            last_cpu_value,
        );
    }
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

fn lower_owned_buffer_call(
    node: &Node,
    body: &mut Vec<String>,
    registers: &mut BTreeMap<String, LlvmValueRef>,
    buffer_lengths: &mut BTreeMap<String, String>,
    next_reg: &mut usize,
    next_block: &mut usize,
    last_cpu_value: &mut Option<String>,
) -> Result<bool, String> {
    let contract =
        yir_core::ffi::parse_owned_buffer_return_contract(&node.op.args).map_err(|error| {
            format!(
                "node `{}` has invalid owned FFI buffer contract: {error}",
                node.name
            )
        })?;
    if contract.abi != "nurs" && contract.abi != "c" && contract.abi != "libc" {
        body.push(format!(
            "  ; deferred lowering for cpu.extern_call_owned_buffer `{}` because ABI `{}` is not supported by the current LLVM bridge",
            node.name, contract.abi
        ));
        return Ok(true);
    }
    let lowered_args = contract
        .inputs
        .iter()
        .map(|arg| {
            registers
                .get(arg)
                .and_then(|value| lower_dynamic_extern_arg(value, body, next_reg))
        })
        .collect::<Option<Vec<_>>>();
    let Some(lowered_args) = lowered_args else {
        body.push(format!(
            "  ; deferred lowering for cpu.extern_call_owned_buffer `{}` because one or more inputs are outside the current CPU LLVM slice",
            node.name
        ));
        return Ok(true);
    };
    let ptr = fresh_reg(next_reg);
    let Some(call) = render_extern_call("ptr", contract.symbol, &lowered_args) else {
        body.push(format!(
            "  ; deferred lowering for cpu.extern_call_owned_buffer `{}` because symbol `{}` has unsupported arity {}",
            node.name,
            contract.symbol,
            lowered_args.len()
        ));
        return Ok(true);
    };
    body.push(format!("  {ptr} = {call}"));
    let nonnull = fresh_reg(next_reg);
    body.push(format!("  {nonnull} = icmp ne ptr {ptr}, null"));
    let header_block = fresh_block(next_block, "owned_buffer_header");
    let null_block = fresh_block(next_block, "owned_buffer_null");
    body.push(format!(
        "  br i1 {nonnull}, label %{header_block}, label %{null_block}"
    ));
    body.push(format!("{null_block}:"));
    body.push("  call void @llvm.trap()".to_owned());
    body.push("  unreachable".to_owned());
    body.push(format!("{header_block}:"));
    let header = fresh_reg(next_reg);
    body.push(format!(
        "  {header} = getelementptr inbounds i64, ptr {ptr}, i64 -1"
    ));
    let len = fresh_reg(next_reg);
    body.push(format!("  {len} = load i64, ptr {header}"));
    let valid_len = fresh_reg(next_reg);
    body.push(format!("  {valid_len} = icmp sge i64 {len}, 0"));
    let ready_block = fresh_block(next_block, "owned_buffer_ready");
    let invalid_len_block = fresh_block(next_block, "owned_buffer_invalid_len");
    body.push(format!(
        "  br i1 {valid_len}, label %{ready_block}, label %{invalid_len_block}"
    ));
    body.push(format!("{invalid_len_block}:"));
    body.push("  call void @llvm.trap()".to_owned());
    body.push("  unreachable".to_owned());
    body.push(format!("{ready_block}:"));
    registers.insert(
        node.name.clone(),
        LlvmValueRef::OwnedExternalBuffer {
            ptr: ptr.clone(),
            len: len.clone(),
            destructor: contract.destructor_symbol.to_owned(),
        },
    );
    buffer_lengths.insert(node.name.clone(), len);
    *last_cpu_value = Some(ptr);
    Ok(true)
}
