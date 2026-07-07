use super::*;

pub(in crate::lowering) fn prepare_guard_host_call_stmt(
    stmt: &NirStmt,
) -> Option<(Option<String>, PreparedHostCall)> {
    match stmt {
        NirStmt::Expr(NirExpr::CpuExternCall {
            abi, callee, args, ..
        }) if is_guard_host_call(callee) => Some((
            None,
            PreparedHostCall {
                result_name: None,
                abi: abi.clone(),
                callee: callee.clone(),
                args: args.clone(),
            },
        )),
        NirStmt::Expr(NirExpr::Call { callee, args }) if is_guard_host_call(callee) => Some((
            None,
            PreparedHostCall {
                result_name: None,
                abi: "c".to_owned(),
                callee: callee.clone(),
                args: args.clone(),
            },
        )),
        NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. } => {
            prepare_guard_host_call_stmt(&NirStmt::Expr(value.clone())).map(|(_, mut call)| {
                call.result_name = Some(name.clone());
                (Some(name.clone()), call)
            })
        }
        _ => None,
    }
}

pub(in crate::lowering) fn prepare_host_call_computed_return(
    expr: &NirExpr,
    effect_bindings: &BTreeSet<&str>,
) -> Option<PreparedHostCallReturn> {
    match expr {
        NirExpr::Call { callee, args }
            if is_write_flush_exit_code_call(callee) && args.len() == 2 =>
        {
            prepare_write_flush_exit_code_call(&args[0], &args[1], effect_bindings, 0)
        }
        NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs,
            rhs,
        } => match (lhs.as_ref(), rhs.as_ref()) {
            (call @ NirExpr::Call { .. }, NirExpr::Int(offset))
            | (NirExpr::Int(offset), call @ NirExpr::Call { .. }) => {
                let mut returned = prepare_host_call_computed_return(call, effect_bindings)?;
                if let PreparedHostCallReturn::WriteFlushExitCode {
                    offset: base_offset,
                    ..
                } = &mut returned
                {
                    *base_offset += *offset;
                }
                Some(returned)
            }
            _ => None,
        },
        _ => None,
    }
}

fn is_guard_host_output_call(callee: &str) -> bool {
    matches!(callee, "host_stdout_write" | "host_stderr_write")
}

fn is_guard_host_flush_call(callee: &str) -> bool {
    matches!(callee, "host_stdout_flush" | "host_stderr_flush")
}

fn is_guard_host_call(callee: &str) -> bool {
    is_guard_host_output_call(callee) || is_guard_host_flush_call(callee)
}

fn prepare_write_flush_exit_code_call(
    write: &NirExpr,
    flush: &NirExpr,
    effect_bindings: &BTreeSet<&str>,
    offset: i64,
) -> Option<PreparedHostCallReturn> {
    let (NirExpr::Var(write_name), NirExpr::Var(flush_name)) = (write, flush) else {
        return None;
    };
    if !effect_bindings.contains(write_name.as_str())
        || !effect_bindings.contains(flush_name.as_str())
    {
        return None;
    }
    Some(PreparedHostCallReturn::WriteFlushExitCode {
        write_name: write_name.clone(),
        flush_name: flush_name.clone(),
        offset,
    })
}

fn is_write_flush_exit_code_call(callee: &str) -> bool {
    matches!(
        callee,
        "write_flush_exit_code" | "StdIoContracts.write_flush_exit_code"
    )
}
