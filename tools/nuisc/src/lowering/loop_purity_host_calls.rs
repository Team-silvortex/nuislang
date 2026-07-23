use super::*;

pub(in crate::lowering) fn prepare_guard_host_call_stmt(
    stmt: &NirStmt,
) -> Option<(Option<String>, PreparedHostCall)> {
    match stmt {
        NirStmt::Expr(NirExpr::CpuExternCall {
            abi, callee, args, ..
        }) => Some((
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

pub(in crate::lowering) fn prepare_host_call_compare_return(
    condition: &NirExpr,
    matched: &NirExpr,
    unmatched: &NirExpr,
    pure_helpers: &BTreeSet<String>,
) -> Option<(PreparedHostCall, PreparedHostCallReturn)> {
    if !is_terminal_branch_pure_expr(matched, pure_helpers)
        || !is_terminal_branch_pure_expr(unmatched, pure_helpers)
    {
        return None;
    }
    let NirExpr::Binary { op, lhs, rhs } = condition else {
        return None;
    };
    let (call, expected, op) = match (lhs.as_ref(), rhs.as_ref()) {
        (
            NirExpr::CpuExternCall {
                abi, callee, args, ..
            },
            expected,
        ) => (
            PreparedHostCall {
                result_name: Some("__nuis_branch_host_result".to_owned()),
                abi: abi.clone(),
                callee: callee.clone(),
                args: args.clone(),
            },
            expected.clone(),
            *op,
        ),
        (
            expected,
            NirExpr::CpuExternCall {
                abi, callee, args, ..
            },
        ) => (
            PreparedHostCall {
                result_name: Some("__nuis_branch_host_result".to_owned()),
                abi: abi.clone(),
                callee: callee.clone(),
                args: args.clone(),
            },
            expected.clone(),
            reverse_comparison(*op)?,
        ),
        _ => return None,
    };
    if !is_terminal_branch_pure_expr(&expected, pure_helpers) {
        return None;
    }
    let result_name = call.result_name.clone()?;
    Some((
        call,
        PreparedHostCallReturn::CompareCallResult {
            result_name,
            op,
            expected,
            matched: matched.clone(),
            unmatched: unmatched.clone(),
        },
    ))
}

fn reverse_comparison(op: NirBinaryOp) -> Option<NirBinaryOp> {
    Some(match op {
        NirBinaryOp::Eq => NirBinaryOp::Eq,
        NirBinaryOp::Ne => NirBinaryOp::Ne,
        NirBinaryOp::Lt => NirBinaryOp::Gt,
        NirBinaryOp::Le => NirBinaryOp::Ge,
        NirBinaryOp::Gt => NirBinaryOp::Lt,
        NirBinaryOp::Ge => NirBinaryOp::Le,
        _ => return None,
    })
}

fn is_guard_host_output_call(callee: &str) -> bool {
    matches!(callee, "host_stdout_write" | "host_stderr_write")
}

fn is_guard_host_flush_call(callee: &str) -> bool {
    matches!(callee, "host_stdout_flush" | "host_stderr_flush")
}

fn is_guard_host_diag_call(callee: &str) -> bool {
    matches!(
        callee,
        "host_diag_label" | "host_diag_span" | "host_diag_emit"
    )
}

fn is_guard_host_call(callee: &str) -> bool {
    is_guard_host_output_call(callee)
        || is_guard_host_flush_call(callee)
        || is_guard_host_diag_call(callee)
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
