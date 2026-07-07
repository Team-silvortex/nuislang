use super::*;

pub(in crate::lowering) fn substitute_branch_binding(
    expr: &NirExpr,
    binding_name: &str,
    binding_value: &NirExpr,
) -> NirExpr {
    match expr {
        NirExpr::Var(name) if name == binding_name => binding_value.clone(),
        NirExpr::Await(inner) => NirExpr::Await(Box::new(substitute_branch_binding(
            inner,
            binding_name,
            binding_value,
        ))),
        NirExpr::Borrow(inner) => NirExpr::Borrow(Box::new(substitute_branch_binding(
            inner,
            binding_name,
            binding_value,
        ))),
        NirExpr::BorrowEnd(inner) => NirExpr::BorrowEnd(Box::new(substitute_branch_binding(
            inner,
            binding_name,
            binding_value,
        ))),
        NirExpr::HostBufferHandle(inner) => NirExpr::HostBufferHandle(Box::new(
            substitute_branch_binding(inner, binding_name, binding_value),
        )),
        NirExpr::Move(inner) => NirExpr::Move(Box::new(substitute_branch_binding(
            inner,
            binding_name,
            binding_value,
        ))),
        NirExpr::CastI64ToI32(inner) => NirExpr::CastI64ToI32(Box::new(substitute_branch_binding(
            inner,
            binding_name,
            binding_value,
        ))),
        NirExpr::CastI32ToI64(inner) => NirExpr::CastI32ToI64(Box::new(substitute_branch_binding(
            inner,
            binding_name,
            binding_value,
        ))),
        NirExpr::CastI64ToBool(inner) => NirExpr::CastI64ToBool(Box::new(
            substitute_branch_binding(inner, binding_name, binding_value),
        )),
        NirExpr::CastBoolToI64(inner) => NirExpr::CastBoolToI64(Box::new(
            substitute_branch_binding(inner, binding_name, binding_value),
        )),
        NirExpr::CastI64ToF32(inner) => NirExpr::CastI64ToF32(Box::new(substitute_branch_binding(
            inner,
            binding_name,
            binding_value,
        ))),
        NirExpr::CastF32ToI64(inner) => NirExpr::CastF32ToI64(Box::new(substitute_branch_binding(
            inner,
            binding_name,
            binding_value,
        ))),
        NirExpr::CastI64ToF64(inner) => NirExpr::CastI64ToF64(Box::new(substitute_branch_binding(
            inner,
            binding_name,
            binding_value,
        ))),
        NirExpr::CastF64ToI64(inner) => NirExpr::CastF64ToI64(Box::new(substitute_branch_binding(
            inner,
            binding_name,
            binding_value,
        ))),
        NirExpr::Call { callee, args } => NirExpr::Call {
            callee: callee.clone(),
            args: args
                .iter()
                .map(|arg| substitute_branch_binding(arg, binding_name, binding_value))
                .collect(),
        },
        NirExpr::CpuExternCall {
            abi,
            callee,
            interface,
            args,
        } => NirExpr::CpuExternCall {
            abi: abi.clone(),
            callee: callee.clone(),
            interface: interface.clone(),
            args: args
                .iter()
                .map(|arg| substitute_branch_binding(arg, binding_name, binding_value))
                .collect(),
        },
        NirExpr::MethodCall {
            receiver,
            method,
            args,
        } => NirExpr::MethodCall {
            receiver: Box::new(substitute_branch_binding(
                receiver,
                binding_name,
                binding_value,
            )),
            method: method.clone(),
            args: args
                .iter()
                .map(|arg| substitute_branch_binding(arg, binding_name, binding_value))
                .collect(),
        },
        NirExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } => NirExpr::StructLiteral {
            type_name: type_name.clone(),
            type_args: type_args.clone(),
            fields: fields
                .iter()
                .map(|(field, value)| {
                    (
                        field.clone(),
                        substitute_branch_binding(value, binding_name, binding_value),
                    )
                })
                .collect(),
        },
        NirExpr::FieldAccess { base, field } => NirExpr::FieldAccess {
            base: Box::new(substitute_branch_binding(base, binding_name, binding_value)),
            field: field.clone(),
        },
        NirExpr::Binary { op, lhs, rhs } => NirExpr::Binary {
            op: *op,
            lhs: Box::new(substitute_branch_binding(lhs, binding_name, binding_value)),
            rhs: Box::new(substitute_branch_binding(rhs, binding_name, binding_value)),
        },
        NirExpr::LoadValue(inner) => NirExpr::LoadValue(Box::new(substitute_branch_binding(
            inner,
            binding_name,
            binding_value,
        ))),
        NirExpr::LoadNext(inner) => NirExpr::LoadNext(Box::new(substitute_branch_binding(
            inner,
            binding_name,
            binding_value,
        ))),
        NirExpr::BufferLen(inner) => NirExpr::BufferLen(Box::new(substitute_branch_binding(
            inner,
            binding_name,
            binding_value,
        ))),
        NirExpr::LoadAt { buffer, index } => NirExpr::LoadAt {
            buffer: Box::new(substitute_branch_binding(
                buffer,
                binding_name,
                binding_value,
            )),
            index: Box::new(substitute_branch_binding(
                index,
                binding_name,
                binding_value,
            )),
        },
        NirExpr::DataReadWindow { window, index } => NirExpr::DataReadWindow {
            window: Box::new(substitute_branch_binding(
                window,
                binding_name,
                binding_value,
            )),
            index: Box::new(substitute_branch_binding(
                index,
                binding_name,
                binding_value,
            )),
        },
        NirExpr::StoreValue { target, value } => NirExpr::StoreValue {
            target: Box::new(substitute_branch_binding(
                target,
                binding_name,
                binding_value,
            )),
            value: Box::new(substitute_branch_binding(
                value,
                binding_name,
                binding_value,
            )),
        },
        NirExpr::StoreNext { target, next } => NirExpr::StoreNext {
            target: Box::new(substitute_branch_binding(
                target,
                binding_name,
                binding_value,
            )),
            next: Box::new(substitute_branch_binding(next, binding_name, binding_value)),
        },
        NirExpr::StoreAt {
            buffer,
            index,
            value,
        } => NirExpr::StoreAt {
            buffer: Box::new(substitute_branch_binding(
                buffer,
                binding_name,
                binding_value,
            )),
            index: Box::new(substitute_branch_binding(
                index,
                binding_name,
                binding_value,
            )),
            value: Box::new(substitute_branch_binding(
                value,
                binding_name,
                binding_value,
            )),
        },
        NirExpr::DataWriteWindow {
            window,
            index,
            value,
        } => NirExpr::DataWriteWindow {
            window: Box::new(substitute_branch_binding(
                window,
                binding_name,
                binding_value,
            )),
            index: Box::new(substitute_branch_binding(
                index,
                binding_name,
                binding_value,
            )),
            value: Box::new(substitute_branch_binding(
                value,
                binding_name,
                binding_value,
            )),
        },
        NirExpr::AllocNode { value, next } => NirExpr::AllocNode {
            value: Box::new(substitute_branch_binding(
                value,
                binding_name,
                binding_value,
            )),
            next: Box::new(substitute_branch_binding(next, binding_name, binding_value)),
        },
        NirExpr::AllocBuffer { len, fill } => NirExpr::AllocBuffer {
            len: Box::new(substitute_branch_binding(len, binding_name, binding_value)),
            fill: Box::new(substitute_branch_binding(fill, binding_name, binding_value)),
        },
        NirExpr::NetworkResult { value, state } => NirExpr::NetworkResult {
            value: Box::new(substitute_branch_binding(
                value,
                binding_name,
                binding_value,
            )),
            state: *state,
        },
        NirExpr::NetworkConfigReady(inner) => NirExpr::NetworkConfigReady(Box::new(
            substitute_branch_binding(inner, binding_name, binding_value),
        )),
        NirExpr::NetworkSendReady(inner) => NirExpr::NetworkSendReady(Box::new(
            substitute_branch_binding(inner, binding_name, binding_value),
        )),
        NirExpr::NetworkRecvReady(inner) => NirExpr::NetworkRecvReady(Box::new(
            substitute_branch_binding(inner, binding_name, binding_value),
        )),
        NirExpr::NetworkAcceptReady(inner) => NirExpr::NetworkAcceptReady(Box::new(
            substitute_branch_binding(inner, binding_name, binding_value),
        )),
        NirExpr::NetworkValue(inner) => NirExpr::NetworkValue(Box::new(substitute_branch_binding(
            inner,
            binding_name,
            binding_value,
        ))),
        NirExpr::DataResult { value, state } => NirExpr::DataResult {
            value: Box::new(substitute_branch_binding(
                value,
                binding_name,
                binding_value,
            )),
            state: *state,
        },
        NirExpr::DataReady(inner) => NirExpr::DataReady(Box::new(substitute_branch_binding(
            inner,
            binding_name,
            binding_value,
        ))),
        NirExpr::DataMoved(inner) => NirExpr::DataMoved(Box::new(substitute_branch_binding(
            inner,
            binding_name,
            binding_value,
        ))),
        NirExpr::DataWindowed(inner) => NirExpr::DataWindowed(Box::new(substitute_branch_binding(
            inner,
            binding_name,
            binding_value,
        ))),
        NirExpr::DataValue(inner) => NirExpr::DataValue(Box::new(substitute_branch_binding(
            inner,
            binding_name,
            binding_value,
        ))),
        NirExpr::KernelResult { value, state } => NirExpr::KernelResult {
            value: Box::new(substitute_branch_binding(
                value,
                binding_name,
                binding_value,
            )),
            state: *state,
        },
        NirExpr::KernelConfigReady(inner) => NirExpr::KernelConfigReady(Box::new(
            substitute_branch_binding(inner, binding_name, binding_value),
        )),
        NirExpr::KernelValue(inner) => NirExpr::KernelValue(Box::new(substitute_branch_binding(
            inner,
            binding_name,
            binding_value,
        ))),
        NirExpr::ShaderResult { value, state } => NirExpr::ShaderResult {
            value: Box::new(substitute_branch_binding(
                value,
                binding_name,
                binding_value,
            )),
            state: *state,
        },
        NirExpr::ShaderPassReady(inner) => NirExpr::ShaderPassReady(Box::new(
            substitute_branch_binding(inner, binding_name, binding_value),
        )),
        NirExpr::ShaderFrameReady(inner) => NirExpr::ShaderFrameReady(Box::new(
            substitute_branch_binding(inner, binding_name, binding_value),
        )),
        NirExpr::ShaderValue(inner) => NirExpr::ShaderValue(Box::new(substitute_branch_binding(
            inner,
            binding_name,
            binding_value,
        ))),
        _ => expr.clone(),
    }
}

pub(in crate::lowering) fn substitute_prepared_terminal_branch(
    branch: PreparedTerminalBranch,
    binding_name: &str,
    binding_value: &NirExpr,
) -> PreparedTerminalBranch {
    match branch {
        PreparedTerminalBranch::Return(returned) => PreparedTerminalBranch::Return(
            substitute_branch_binding(&returned, binding_name, binding_value),
        ),
        PreparedTerminalBranch::PrintReturn { print, returned } => {
            PreparedTerminalBranch::PrintReturn {
                print: substitute_branch_binding(&print, binding_name, binding_value),
                returned: substitute_branch_binding(&returned, binding_name, binding_value),
            }
        }
        PreparedTerminalBranch::HostCallReturn { calls, returned } => {
            PreparedTerminalBranch::HostCallReturn {
                calls: calls
                    .into_iter()
                    .map(|call| PreparedHostCall {
                        result_name: call.result_name,
                        abi: call.abi,
                        callee: call.callee,
                        args: call
                            .args
                            .iter()
                            .map(|arg| substitute_branch_binding(arg, binding_name, binding_value))
                            .collect(),
                    })
                    .collect(),
                returned: substitute_prepared_host_call_return(
                    returned,
                    binding_name,
                    binding_value,
                ),
            }
        }
    }
}

pub(in crate::lowering) fn substitute_prepared_loop_body(
    body: PreparedLoopBody,
    binding_name: &str,
    binding_value: &NirExpr,
) -> PreparedLoopBody {
    match body {
        PreparedLoopBody::ExitOnly => PreparedLoopBody::ExitOnly,
        PreparedLoopBody::PrintExit { print } => PreparedLoopBody::PrintExit {
            print: substitute_branch_binding(&print, binding_name, binding_value),
        },
        PreparedLoopBody::Return { returned } => PreparedLoopBody::Return {
            returned: substitute_branch_binding(&returned, binding_name, binding_value),
        },
        PreparedLoopBody::PrintReturn { print, returned } => PreparedLoopBody::PrintReturn {
            print: substitute_branch_binding(&print, binding_name, binding_value),
            returned: substitute_branch_binding(&returned, binding_name, binding_value),
        },
        PreparedLoopBody::Branch {
            condition,
            then_body,
            else_body,
        } => PreparedLoopBody::Branch {
            condition: substitute_branch_binding(&condition, binding_name, binding_value),
            then_body: Box::new(substitute_prepared_loop_body(
                *then_body,
                binding_name,
                binding_value,
            )),
            else_body: Box::new(substitute_prepared_loop_body(
                *else_body,
                binding_name,
                binding_value,
            )),
        },
    }
}

pub(in crate::lowering) fn substitute_stmt_bindings(
    stmt: &NirStmt,
    bindings: &[(String, NirExpr)],
) -> NirStmt {
    fn substitute_stmt_binding(
        stmt: &NirStmt,
        binding_name: &str,
        binding_value: &NirExpr,
    ) -> NirStmt {
        match stmt {
            NirStmt::Let { name, ty, value } => NirStmt::Let {
                name: name.clone(),
                ty: ty.clone(),
                value: substitute_branch_binding(value, binding_name, binding_value),
            },
            NirStmt::Const { name, ty, value } => NirStmt::Const {
                name: name.clone(),
                ty: ty.clone(),
                value: substitute_branch_binding(value, binding_name, binding_value),
            },
            NirStmt::Expr(expr) => {
                NirStmt::Expr(substitute_branch_binding(expr, binding_name, binding_value))
            }
            NirStmt::Return(Some(expr)) => NirStmt::Return(Some(substitute_branch_binding(
                expr,
                binding_name,
                binding_value,
            ))),
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } => NirStmt::If {
                condition: substitute_branch_binding(condition, binding_name, binding_value),
                then_body: then_body
                    .iter()
                    .map(|inner| substitute_stmt_binding(inner, binding_name, binding_value))
                    .collect(),
                else_body: else_body
                    .iter()
                    .map(|inner| substitute_stmt_binding(inner, binding_name, binding_value))
                    .collect(),
            },
            NirStmt::While { condition, body } => NirStmt::While {
                condition: substitute_branch_binding(condition, binding_name, binding_value),
                body: body
                    .iter()
                    .map(|inner| substitute_stmt_binding(inner, binding_name, binding_value))
                    .collect(),
            },
            _ => stmt.clone(),
        }
    }

    bindings
        .iter()
        .fold(stmt.clone(), |current, (binding_name, binding_value)| {
            substitute_stmt_binding(&current, binding_name, binding_value)
        })
}

pub(in crate::lowering) fn prepare_terminal_branch(
    stmts: &[NirStmt],
    pure_helpers: &BTreeSet<String>,
) -> Option<PreparedTerminalBranch> {
    match stmts {
        [NirStmt::Return(Some(value))] | [NirStmt::Expr(value)] => {
            if !is_terminal_branch_pure_expr(value, pure_helpers) {
                return None;
            }
            Some(PreparedTerminalBranch::Return(value.clone()))
        }
        [NirStmt::Print(print), NirStmt::Return(Some(returned))]
        | [NirStmt::Print(print), NirStmt::Expr(returned)] => {
            if !is_terminal_branch_pure_expr(print, pure_helpers)
                || !is_terminal_branch_pure_expr(returned, pure_helpers)
            {
                return None;
            }
            Some(PreparedTerminalBranch::PrintReturn {
                print: print.clone(),
                returned: returned.clone(),
            })
        }
        [calls @ .., NirStmt::Return(Some(returned)) | NirStmt::Expr(returned)]
            if !calls.is_empty() && calls.len() <= 4 =>
        {
            if !is_terminal_branch_pure_expr(returned, pure_helpers) {
                return None;
            }
            let prepared_calls = calls
                .iter()
                .map(prepare_guard_host_call_stmt)
                .collect::<Option<Vec<_>>>()?;
            let effect_bindings = prepared_calls
                .iter()
                .filter_map(|(name, _)| name.as_deref())
                .collect::<BTreeSet<_>>();
            let returned = if expr_references_names(returned, &effect_bindings) {
                prepare_host_call_computed_return(returned, &effect_bindings)?
            } else {
                PreparedHostCallReturn::Expr(returned.clone())
            };
            if matches!(returned, PreparedHostCallReturn::Expr(_))
                && expr_references_names(
                    match &returned {
                        PreparedHostCallReturn::Expr(expr) => expr,
                        PreparedHostCallReturn::WriteFlushExitCode { .. } => unreachable!(),
                    },
                    &effect_bindings,
                )
            {
                return None;
            }
            Some(PreparedTerminalBranch::HostCallReturn {
                calls: prepared_calls.into_iter().map(|(_, call)| call).collect(),
                returned,
            })
        }
        [binding @ (NirStmt::Let { .. } | NirStmt::Const { .. }), tail @ ..] => {
            let (name, value) = extract_pure_branch_binding(binding, pure_helpers)?;
            let prepared = prepare_terminal_branch(tail, pure_helpers)?;
            Some(substitute_prepared_terminal_branch(prepared, &name, &value))
        }
        _ => None,
    }
}

fn substitute_prepared_host_call_return(
    returned: PreparedHostCallReturn,
    binding_name: &str,
    binding_value: &NirExpr,
) -> PreparedHostCallReturn {
    match returned {
        PreparedHostCallReturn::Expr(expr) => PreparedHostCallReturn::Expr(
            substitute_branch_binding(&expr, binding_name, binding_value),
        ),
        PreparedHostCallReturn::WriteFlushExitCode {
            write_name,
            flush_name,
            offset,
        } => PreparedHostCallReturn::WriteFlushExitCode {
            write_name,
            flush_name,
            offset,
        },
    }
}
