use std::collections::BTreeMap;

use super::{
    call_return::{
        can_emit_typed_return_from_value, cpu_scalar_kind_llvm_type, emit_typed_return_from_value,
    },
    extern_abi::{
        is_builtin_host_ffi_symbol, lower_dynamic_extern_arg, lower_i64_extern_arg,
        render_extern_call,
    },
    fresh_block, fresh_reg,
    value_ref::coerce_to_i64,
    CpuCallScalarKind, LlvmValueRef,
};
use yir_core::Node;

struct GuardHostCall {
    result_alias: Option<String>,
    abi: String,
    symbol: String,
    args: Vec<String>,
}

enum GuardHostReturn {
    Value(String),
    WriteFlushExitCode {
        write_name: String,
        flush_name: String,
        offset: i64,
    },
}

pub(super) fn lower_guard_host_call_return(
    node: &Node,
    registers: &BTreeMap<String, LlvmValueRef>,
    body: &mut Vec<String>,
    next_reg: &mut usize,
    next_block: &mut usize,
    function_return_kind: CpuCallScalarKind,
) -> bool {
    let cond_value = registers.get(&node.op.args[0]).cloned();
    let Some((returned, calls)) = parse_guard_host_call_return(node) else {
        body.push(format!(
            "  ; deferred lowering for cpu.guard_host_call_return `{}` because its call chain encoding is malformed",
            node.name
        ));
        return false;
    };
    let return_value = match &returned {
        GuardHostReturn::Value(return_name) => registers.get(return_name).cloned(),
        GuardHostReturn::WriteFlushExitCode { .. } => None,
    };
    let Some(cond_value) = cond_value else {
        body.push(format!("  ; deferred lowering for cpu.guard_host_call_return `{}` because condition is outside the current CPU LLVM slice", node.name));
        return false;
    };
    let Some(cond) = coerce_to_i64(&cond_value, body, next_reg) else {
        body.push(format!("  ; deferred lowering for cpu.guard_host_call_return `{}` because its condition is not coercible to i64", node.name));
        return false;
    };
    if let Some(return_value) = &return_value {
        if !can_emit_typed_return_from_value(function_return_kind, return_value) {
            body.push(format!("  ; deferred lowering for cpu.guard_host_call_return `{}` because its return value is not coercible to {}", node.name, cpu_scalar_kind_llvm_type(function_return_kind)));
            return false;
        }
    }
    let Some(lowered_calls) = lower_guard_host_calls(&calls, registers, body, next_reg, node)
    else {
        return false;
    };
    let cond_bool = fresh_reg(next_reg);
    body.push(format!("  {cond_bool} = icmp ne i64 {cond}, 0"));
    let then_label = fresh_block(next_block, "guard_host_call_return_then");
    let cont_label = fresh_block(next_block, "guard_host_call_return_cont");
    body.push(format!(
        "  br i1 {cond_bool}, label %{then_label}, label %{cont_label}"
    ));
    body.push(format!("{then_label}:"));
    let mut call_results = BTreeMap::new();
    for (alias, call) in lowered_calls {
        let call_reg = fresh_reg(next_reg);
        body.push(format!("  {call_reg} = {call}"));
        if let Some(alias) = alias {
            call_results.insert(alias, call_reg);
        }
    }
    if !emit_guard_host_return(
        body,
        next_reg,
        function_return_kind,
        &returned,
        return_value.as_ref(),
        &call_results,
        node,
    ) {
        return false;
    }
    body.push(format!("{cont_label}:"));
    true
}

pub(super) fn lower_branch_host_call_return(
    node: &Node,
    registers: &BTreeMap<String, LlvmValueRef>,
    body: &mut Vec<String>,
    next_reg: &mut usize,
    next_block: &mut usize,
    function_return_kind: CpuCallScalarKind,
) -> bool {
    let cond_value = registers.get(&node.op.args[0]).cloned();
    let Some((then_returned, then_calls, cursor)) =
        parse_host_call_return_component(&node.op.args, 1)
    else {
        body.push(format!(
            "  ; deferred lowering for cpu.branch_host_call_return `{}` because its then branch encoding is malformed",
            node.name
        ));
        return false;
    };
    let Some((else_returned, else_calls, cursor)) =
        parse_host_call_return_component(&node.op.args, cursor)
    else {
        body.push(format!(
            "  ; deferred lowering for cpu.branch_host_call_return `{}` because its else branch encoding is malformed",
            node.name
        ));
        return false;
    };
    if cursor != node.op.args.len() {
        body.push(format!(
            "  ; deferred lowering for cpu.branch_host_call_return `{}` because it has trailing branch encoding args",
            node.name
        ));
        return false;
    }
    let Some(cond_value) = cond_value else {
        body.push(format!("  ; deferred lowering for cpu.branch_host_call_return `{}` because condition is outside the current CPU LLVM slice", node.name));
        return false;
    };
    let Some(cond) = coerce_to_i64(&cond_value, body, next_reg) else {
        body.push(format!("  ; deferred lowering for cpu.branch_host_call_return `{}` because its condition is not coercible to i64", node.name));
        return false;
    };
    let Some(then_calls) = lower_guard_host_calls(&then_calls, registers, body, next_reg, node)
    else {
        return false;
    };
    let Some(else_calls) = lower_guard_host_calls(&else_calls, registers, body, next_reg, node)
    else {
        return false;
    };
    let cond_bool = fresh_reg(next_reg);
    body.push(format!("  {cond_bool} = icmp ne i64 {cond}, 0"));
    let then_label = fresh_block(next_block, "branch_host_call_return_then");
    let else_label = fresh_block(next_block, "branch_host_call_return_else");
    body.push(format!(
        "  br i1 {cond_bool}, label %{then_label}, label %{else_label}"
    ));
    let then_return_value = guard_host_value_return(&then_returned, registers);
    let else_return_value = guard_host_value_return(&else_returned, registers);
    body.push(format!("{then_label}:"));
    if !emit_host_call_return_body(
        body,
        next_reg,
        function_return_kind,
        &then_returned,
        then_return_value.as_ref(),
        then_calls,
        node,
    ) {
        return false;
    }
    body.push(format!("{else_label}:"));
    emit_host_call_return_body(
        body,
        next_reg,
        function_return_kind,
        &else_returned,
        else_return_value.as_ref(),
        else_calls,
        node,
    )
}

fn parse_guard_host_call_return(node: &Node) -> Option<(GuardHostReturn, Vec<GuardHostCall>)> {
    if node.op.args.len() < 4 {
        return None;
    }
    if matches!(
        node.op.args.get(1).map(String::as_str),
        Some("value" | "write_flush_exit_code")
    ) {
        return parse_guard_host_call_return_v2(node);
    }
    if let Ok(call_count) = node.op.args[2].parse::<usize>() {
        let mut cursor = 3;
        let mut calls = Vec::with_capacity(call_count);
        for _ in 0..call_count {
            let abi = node.op.args.get(cursor)?.clone();
            let symbol = node.op.args.get(cursor + 1)?.clone();
            let arg_count = node.op.args.get(cursor + 2)?.parse::<usize>().ok()?;
            cursor += 3;
            let args = node.op.args.get(cursor..cursor + arg_count)?.to_vec();
            cursor += arg_count;
            calls.push(GuardHostCall {
                result_alias: None,
                abi,
                symbol,
                args,
            });
        }
        (cursor == node.op.args.len())
            .then(|| (GuardHostReturn::Value(node.op.args[1].clone()), calls))
    } else {
        Some((
            GuardHostReturn::Value(node.op.args[3].clone()),
            vec![GuardHostCall {
                result_alias: None,
                abi: node.op.args[1].clone(),
                symbol: node.op.args[2].clone(),
                args: node.op.args[4..].to_vec(),
            }],
        ))
    }
}

fn parse_guard_host_call_return_v2(node: &Node) -> Option<(GuardHostReturn, Vec<GuardHostCall>)> {
    let (returned, calls, cursor) = parse_host_call_return_component(&node.op.args, 1)?;
    (cursor == node.op.args.len()).then_some((returned, calls))
}

fn parse_host_call_return_component(
    args: &[String],
    start: usize,
) -> Option<(GuardHostReturn, Vec<GuardHostCall>, usize)> {
    let mode = args.get(start)?.as_str();
    let return_arg_count = args.get(start + 1)?.parse::<usize>().ok()?;
    let return_args = args.get(start + 2..start + 2 + return_arg_count)?.to_vec();
    let returned = match mode {
        "value" if return_args.len() == 1 => GuardHostReturn::Value(return_args[0].clone()),
        "write_flush_exit_code" if return_args.len() == 2 || return_args.len() == 3 => {
            GuardHostReturn::WriteFlushExitCode {
                write_name: return_args[0].clone(),
                flush_name: return_args[1].clone(),
                offset: return_args
                    .get(2)
                    .map(|value| value.parse::<i64>())
                    .transpose()
                    .ok()?
                    .unwrap_or(0),
            }
        }
        _ => return None,
    };
    let mut cursor = start + 2 + return_arg_count;
    let call_count = args.get(cursor)?.parse::<usize>().ok()?;
    cursor += 1;
    let mut calls = Vec::with_capacity(call_count);
    for _ in 0..call_count {
        let alias = args.get(cursor)?.clone();
        let abi = args.get(cursor + 1)?.clone();
        let symbol = args.get(cursor + 2)?.clone();
        let arg_count = args.get(cursor + 3)?.parse::<usize>().ok()?;
        cursor += 4;
        let args = args.get(cursor..cursor + arg_count)?.to_vec();
        cursor += arg_count;
        calls.push(GuardHostCall {
            result_alias: (alias != "_").then_some(alias),
            abi,
            symbol,
            args,
        });
    }
    Some((returned, calls, cursor))
}

fn lower_guard_host_calls(
    calls: &[GuardHostCall],
    registers: &BTreeMap<String, LlvmValueRef>,
    body: &mut Vec<String>,
    next_reg: &mut usize,
    node: &Node,
) -> Option<Vec<(Option<String>, String)>> {
    calls
        .iter()
        .map(|call| lower_guard_host_call(call, registers, body, next_reg, node))
        .collect()
}

fn lower_guard_host_call(
    call: &GuardHostCall,
    registers: &BTreeMap<String, LlvmValueRef>,
    body: &mut Vec<String>,
    next_reg: &mut usize,
    node: &Node,
) -> Option<(Option<String>, String)> {
    let dynamic_args = call.abi == "libc" || !is_builtin_host_ffi_symbol(&call.symbol);
    let lowered_args = call
        .args
        .iter()
        .map(|arg| {
            registers.get(arg).and_then(|value| {
                if dynamic_args {
                    lower_dynamic_extern_arg(value, body, next_reg)
                } else {
                    lower_i64_extern_arg(value, body, next_reg)
                }
            })
        })
        .collect::<Option<Vec<_>>>();
    let Some(lowered_args) = lowered_args else {
        body.push(format!("  ; deferred lowering for cpu.guard_host_call_return `{}` because one or more host call inputs are outside the current CPU LLVM slice", node.name));
        return None;
    };
    let rendered = render_extern_call("i64", &call.symbol, &lowered_args);
    if rendered.is_none() {
        body.push(format!("  ; deferred lowering for cpu.guard_host_call_return `{}` because symbol `{}` has unsupported arity {}", node.name, call.symbol, lowered_args.len()));
    }
    rendered.map(|rendered| (call.result_alias.clone(), rendered))
}

fn guard_host_value_return(
    returned: &GuardHostReturn,
    registers: &BTreeMap<String, LlvmValueRef>,
) -> Option<LlvmValueRef> {
    match returned {
        GuardHostReturn::Value(return_name) => registers.get(return_name).cloned(),
        GuardHostReturn::WriteFlushExitCode { .. } => None,
    }
}

fn emit_host_call_return_body(
    body: &mut Vec<String>,
    next_reg: &mut usize,
    function_return_kind: CpuCallScalarKind,
    returned: &GuardHostReturn,
    return_value: Option<&LlvmValueRef>,
    lowered_calls: Vec<(Option<String>, String)>,
    node: &Node,
) -> bool {
    let mut call_results = BTreeMap::new();
    for (alias, call) in lowered_calls {
        let call_reg = fresh_reg(next_reg);
        body.push(format!("  {call_reg} = {call}"));
        if let Some(alias) = alias {
            call_results.insert(alias, call_reg);
        }
    }
    emit_guard_host_return(
        body,
        next_reg,
        function_return_kind,
        returned,
        return_value,
        &call_results,
        node,
    )
}

fn emit_guard_host_return(
    body: &mut Vec<String>,
    next_reg: &mut usize,
    function_return_kind: CpuCallScalarKind,
    returned: &GuardHostReturn,
    return_value: Option<&LlvmValueRef>,
    call_results: &BTreeMap<String, String>,
    node: &Node,
) -> bool {
    match returned {
        GuardHostReturn::Value(_) => {
            let Some(return_value) = return_value else {
                body.push(format!("  ; deferred lowering for cpu.guard_host_call_return `{}` because return value is outside the current CPU LLVM slice", node.name));
                return false;
            };
            if !emit_typed_return_from_value(body, next_reg, function_return_kind, return_value) {
                body.push(format!("  ; deferred lowering for cpu.guard_host_call_return `{}` because its return value is not coercible to {}", node.name, cpu_scalar_kind_llvm_type(function_return_kind)));
                return false;
            }
            true
        }
        GuardHostReturn::WriteFlushExitCode {
            write_name,
            flush_name,
            offset,
        } => {
            let (Some(write), Some(flush)) =
                (call_results.get(write_name), call_results.get(flush_name))
            else {
                body.push(format!("  ; deferred lowering for cpu.guard_host_call_return `{}` because write_flush_exit_code references a missing host call result", node.name));
                return false;
            };
            let write_ok = fresh_reg(next_reg);
            let flush_ok = fresh_reg(next_reg);
            let both_ok = fresh_reg(next_reg);
            let exit_code = fresh_reg(next_reg);
            body.push(format!("  {write_ok} = icmp sge i64 {write}, 0"));
            body.push(format!("  {flush_ok} = icmp sge i64 {flush}, 0"));
            body.push(format!("  {both_ok} = and i1 {write_ok}, {flush_ok}"));
            body.push(format!(
                "  {exit_code} = select i1 {both_ok}, i64 {offset}, i64 {}",
                offset + 1
            ));
            emit_typed_return_from_value(
                body,
                next_reg,
                function_return_kind,
                &LlvmValueRef::I64(exit_code),
            )
        }
    }
}
