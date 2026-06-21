use super::*;

pub(super) fn extract_pure_branch_binding(
    stmt: &NirStmt,
    pure_helpers: &BTreeSet<String>,
) -> Option<(String, NirExpr)> {
    let (name, value) = match stmt {
        NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. } => {
            (name.clone(), value.clone())
        }
        _ => return None,
    };
    if !is_terminal_branch_pure_expr(&value, pure_helpers) {
        return None;
    }
    Some((name, value))
}

pub(super) fn is_terminal_branch_pure_expr(
    expr: &NirExpr,
    pure_helpers: &BTreeSet<String>,
) -> bool {
    match expr {
        NirExpr::Call { callee, args } => {
            (pure_helpers.contains(callee) || is_branch_safe_observer_call(callee))
                && args
                    .iter()
                    .all(|arg| is_terminal_branch_pure_expr(arg, pure_helpers))
        }
        NirExpr::CpuTaskCompleted(inner)
        | NirExpr::CpuTaskTimedOut(inner)
        | NirExpr::CpuTaskCancelled(inner)
        | NirExpr::CpuTaskValue(inner)
        | NirExpr::CpuMutexValue(inner)
        | NirExpr::NetworkConfigReady(inner)
        | NirExpr::NetworkSendReady(inner)
        | NirExpr::NetworkRecvReady(inner)
        | NirExpr::NetworkAcceptReady(inner)
        | NirExpr::NetworkValue(inner)
        | NirExpr::CastI64ToI32(inner)
        | NirExpr::CastI32ToI64(inner)
        | NirExpr::CastI64ToBool(inner)
        | NirExpr::CastBoolToI64(inner)
        | NirExpr::CastI64ToF32(inner)
        | NirExpr::CastF32ToI64(inner)
        | NirExpr::CastI64ToF64(inner)
        | NirExpr::CastF64ToI64(inner) => is_terminal_branch_pure_expr(inner, pure_helpers),
        NirExpr::MethodCall { .. } => false,
        NirExpr::Await(_) | NirExpr::Instantiate { .. } => false,
        NirExpr::StructLiteral { fields, .. } => fields
            .iter()
            .all(|(_, value)| is_terminal_branch_pure_expr(value, pure_helpers)),
        NirExpr::FieldAccess { base, .. } => is_terminal_branch_pure_expr(base, pure_helpers),
        NirExpr::Binary { lhs, rhs, .. } => {
            is_terminal_branch_pure_expr(lhs, pure_helpers)
                && is_terminal_branch_pure_expr(rhs, pure_helpers)
        }
        _ => matches!(
            nir_expr_effect_class(expr),
            NirExprEffectClass::Pure
                | NirExprEffectClass::LocalReadOnly
                | NirExprEffectClass::HostReadOnly
                | NirExprEffectClass::DomainReadOnly
        ),
    }
}

fn is_branch_safe_observer_call(callee: &str) -> bool {
    matches!(
        callee,
        "task_completed"
            | "task_timed_out"
            | "task_cancelled"
            | "task_value"
            | "network_config_ready"
            | "network_send_ready"
            | "network_recv_ready"
            | "network_connect_ready"
            | "network_accept_ready"
            | "network_closed"
            | "network_value"
    )
}

pub(super) fn collect_pure_helper_functions(module: &NirModule) -> BTreeSet<String> {
    let function_map = module
        .functions
        .iter()
        .map(|function| (function.name.as_str(), function))
        .collect::<BTreeMap<_, _>>();
    let mut memo = BTreeMap::<String, bool>::new();
    let mut visiting = BTreeSet::<String>::new();
    module
        .functions
        .iter()
        .filter(|function| function.name != "main")
        .filter(|function| {
            is_pure_helper_function(function, &function_map, &mut memo, &mut visiting)
        })
        .map(|function| function.name.clone())
        .collect()
}

pub(super) fn collect_inlineable_pure_helper_exprs(
    module: &NirModule,
) -> BTreeMap<String, InlineablePureHelper> {
    let function_map = module
        .functions
        .iter()
        .map(|function| (function.name.as_str(), function))
        .collect::<BTreeMap<_, _>>();
    let mut memo = BTreeMap::<String, Option<InlineablePureHelper>>::new();
    let mut visiting = BTreeSet::<String>::new();
    module
        .functions
        .iter()
        .filter_map(|function| {
            extract_inlineable_pure_helper(function, &function_map, &mut memo, &mut visiting)
                .map(|helper| (function.name.clone(), helper))
        })
        .collect()
}

pub(super) fn collect_pure_helper_blocks(module: &NirModule) -> BTreeMap<String, PureHelperBlock> {
    let function_map = module
        .functions
        .iter()
        .map(|function| (function.name.as_str(), function))
        .collect::<BTreeMap<_, _>>();
    let mut memo = BTreeMap::<String, bool>::new();
    let mut visiting = BTreeSet::<String>::new();
    module
        .functions
        .iter()
        .filter(|function| {
            is_pure_helper_function(function, &function_map, &mut memo, &mut visiting)
        })
        .map(|function| {
            (
                function.name.clone(),
                PureHelperBlock {
                    params: function
                        .params
                        .iter()
                        .map(|param| param.name.clone())
                        .collect(),
                    body: function.body.clone(),
                },
            )
        })
        .collect()
}

fn extract_inlineable_pure_helper(
    function: &NirFunction,
    function_map: &BTreeMap<&str, &NirFunction>,
    memo: &mut BTreeMap<String, Option<InlineablePureHelper>>,
    visiting: &mut BTreeSet<String>,
) -> Option<InlineablePureHelper> {
    if let Some(cached) = memo.get(&function.name) {
        return cached.clone();
    }
    if !visiting.insert(function.name.clone()) {
        return None;
    }
    let result = if function.is_async {
        None
    } else {
        extract_inlineable_pure_expr_from_block(&function.body, function_map, memo, visiting).map(
            |expr| InlineablePureHelper {
                params: function
                    .params
                    .iter()
                    .map(|param| param.name.clone())
                    .collect(),
                expr,
            },
        )
    };
    visiting.remove(&function.name);
    memo.insert(function.name.clone(), result.clone());
    result
}

fn extract_inlineable_pure_expr_from_block(
    body: &[NirStmt],
    function_map: &BTreeMap<&str, &NirFunction>,
    _memo: &mut BTreeMap<String, Option<InlineablePureHelper>>,
    visiting: &mut BTreeSet<String>,
) -> Option<NirExpr> {
    let (NirStmt::Return(Some(expr)), prefix) = body.split_last()? else {
        return None;
    };
    let mut substituted = expr.clone();
    let mut pure_memo = BTreeMap::<String, bool>::new();
    for stmt in prefix.iter().rev() {
        let (binding_name, binding_value) = match stmt {
            NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. } => {
                (name.clone(), value.clone())
            }
            _ => return None,
        };
        if !is_pure_helper_expr(&binding_value, function_map, &mut pure_memo, visiting) {
            return None;
        }
        substituted = substitute_branch_binding(&substituted, &binding_name, &binding_value);
    }
    Some(substituted)
}

pub(super) fn inline_pure_helper_calls(
    expr: &NirExpr,
    inlineable_helpers: &BTreeMap<String, InlineablePureHelper>,
) -> NirExpr {
    fn inline_expr(
        expr: &NirExpr,
        inlineable_helpers: &BTreeMap<String, InlineablePureHelper>,
        visiting: &mut BTreeSet<String>,
    ) -> NirExpr {
        match expr {
            NirExpr::Call { callee, args } => {
                let rewritten_args = args
                    .iter()
                    .map(|arg| inline_expr(arg, inlineable_helpers, visiting))
                    .collect::<Vec<_>>();
                if let Some(helper) = inlineable_helpers.get(callee) {
                    if helper.params.len() == rewritten_args.len()
                        && visiting.insert(callee.clone())
                    {
                        let mut expanded = helper.expr.clone();
                        for (param, arg) in helper.params.iter().zip(rewritten_args.iter()) {
                            expanded = substitute_branch_binding(&expanded, param, arg);
                        }
                        let rewritten = inline_expr(&expanded, inlineable_helpers, visiting);
                        visiting.remove(callee);
                        return rewritten;
                    }
                }
                NirExpr::Call {
                    callee: callee.clone(),
                    args: rewritten_args,
                }
            }
            NirExpr::Await(inner) => {
                NirExpr::Await(Box::new(inline_expr(inner, inlineable_helpers, visiting)))
            }
            NirExpr::CastI64ToI32(inner) => {
                NirExpr::CastI64ToI32(Box::new(inline_expr(inner, inlineable_helpers, visiting)))
            }
            NirExpr::CastI32ToI64(inner) => {
                NirExpr::CastI32ToI64(Box::new(inline_expr(inner, inlineable_helpers, visiting)))
            }
            NirExpr::CastI64ToBool(inner) => {
                NirExpr::CastI64ToBool(Box::new(inline_expr(inner, inlineable_helpers, visiting)))
            }
            NirExpr::CastBoolToI64(inner) => {
                NirExpr::CastBoolToI64(Box::new(inline_expr(inner, inlineable_helpers, visiting)))
            }
            NirExpr::CastI64ToF32(inner) => {
                NirExpr::CastI64ToF32(Box::new(inline_expr(inner, inlineable_helpers, visiting)))
            }
            NirExpr::CastF32ToI64(inner) => {
                NirExpr::CastF32ToI64(Box::new(inline_expr(inner, inlineable_helpers, visiting)))
            }
            NirExpr::CastI64ToF64(inner) => {
                NirExpr::CastI64ToF64(Box::new(inline_expr(inner, inlineable_helpers, visiting)))
            }
            NirExpr::CastF64ToI64(inner) => {
                NirExpr::CastF64ToI64(Box::new(inline_expr(inner, inlineable_helpers, visiting)))
            }
            NirExpr::MethodCall {
                receiver,
                method,
                args,
            } => NirExpr::MethodCall {
                receiver: Box::new(inline_expr(receiver, inlineable_helpers, visiting)),
                method: method.clone(),
                args: args
                    .iter()
                    .map(|arg| inline_expr(arg, inlineable_helpers, visiting))
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
                            inline_expr(value, inlineable_helpers, visiting),
                        )
                    })
                    .collect(),
            },
            NirExpr::FieldAccess { base, field } => NirExpr::FieldAccess {
                base: Box::new(inline_expr(base, inlineable_helpers, visiting)),
                field: field.clone(),
            },
            NirExpr::Binary { op, lhs, rhs } => NirExpr::Binary {
                op: *op,
                lhs: Box::new(inline_expr(lhs, inlineable_helpers, visiting)),
                rhs: Box::new(inline_expr(rhs, inlineable_helpers, visiting)),
            },
            _ => expr.clone(),
        }
    }

    inline_expr(expr, inlineable_helpers, &mut BTreeSet::new())
}

fn invert_compare(op: NirBinaryOp) -> Option<NirBinaryOp> {
    match op {
        NirBinaryOp::Eq => Some(NirBinaryOp::Ne),
        NirBinaryOp::Ne => Some(NirBinaryOp::Eq),
        NirBinaryOp::Lt => Some(NirBinaryOp::Ge),
        NirBinaryOp::Le => Some(NirBinaryOp::Gt),
        NirBinaryOp::Gt => Some(NirBinaryOp::Le),
        NirBinaryOp::Ge => Some(NirBinaryOp::Lt),
        _ => None,
    }
}

pub(super) fn normalize_pure_bool_test_expr(expr: NirExpr) -> NirExpr {
    match expr {
        NirExpr::Binary {
            op: NirBinaryOp::Eq,
            lhs,
            rhs,
        } => match rhs.as_ref() {
            NirExpr::Bool(true) => *lhs,
            NirExpr::Bool(false) => match lhs.as_ref() {
                NirExpr::Binary { op, lhs, rhs } => invert_compare(*op)
                    .map(|inverted| NirExpr::Binary {
                        op: inverted,
                        lhs: lhs.clone(),
                        rhs: rhs.clone(),
                    })
                    .unwrap_or(NirExpr::Binary {
                        op: NirBinaryOp::Eq,
                        lhs: lhs.clone(),
                        rhs: rhs.clone(),
                    }),
                _ => NirExpr::Binary {
                    op: NirBinaryOp::Eq,
                    lhs,
                    rhs,
                },
            },
            _ => NirExpr::Binary {
                op: NirBinaryOp::Eq,
                lhs,
                rhs,
            },
        },
        NirExpr::Binary {
            op: NirBinaryOp::Ne,
            lhs,
            rhs,
        } => match rhs.as_ref() {
            NirExpr::Bool(false) => *lhs,
            NirExpr::Bool(true) => match lhs.as_ref() {
                NirExpr::Binary { op, lhs, rhs } => invert_compare(*op)
                    .map(|inverted| NirExpr::Binary {
                        op: inverted,
                        lhs: lhs.clone(),
                        rhs: rhs.clone(),
                    })
                    .unwrap_or(NirExpr::Binary {
                        op: NirBinaryOp::Ne,
                        lhs: lhs.clone(),
                        rhs: rhs.clone(),
                    }),
                _ => NirExpr::Binary {
                    op: NirBinaryOp::Ne,
                    lhs,
                    rhs,
                },
            },
            _ => NirExpr::Binary {
                op: NirBinaryOp::Ne,
                lhs,
                rhs,
            },
        },
        other => other,
    }
}

fn is_pure_helper_function(
    function: &NirFunction,
    function_map: &BTreeMap<&str, &NirFunction>,
    memo: &mut BTreeMap<String, bool>,
    visiting: &mut BTreeSet<String>,
) -> bool {
    if let Some(&cached) = memo.get(&function.name) {
        return cached;
    }
    if !visiting.insert(function.name.clone()) {
        return false;
    }
    let result =
        !function.is_async && is_pure_helper_block(&function.body, function_map, memo, visiting);
    visiting.remove(&function.name);
    memo.insert(function.name.clone(), result);
    result
}

fn is_pure_helper_block(
    body: &[NirStmt],
    function_map: &BTreeMap<&str, &NirFunction>,
    memo: &mut BTreeMap<String, bool>,
    visiting: &mut BTreeSet<String>,
) -> bool {
    let Some((first, tail)) = body.split_first() else {
        return false;
    };
    match first {
        NirStmt::Let { value, .. } | NirStmt::Const { value, .. } => {
            is_pure_helper_expr(value, function_map, memo, visiting)
                && is_pure_helper_block(tail, function_map, memo, visiting)
        }
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            is_pure_helper_expr(condition, function_map, memo, visiting)
                && is_pure_helper_block(then_body, function_map, memo, visiting)
                && if else_body.is_empty() {
                    is_pure_helper_block(tail, function_map, memo, visiting)
                } else {
                    tail.is_empty() && is_pure_helper_block(else_body, function_map, memo, visiting)
                }
        }
        NirStmt::Return(Some(expr)) => {
            tail.is_empty() && is_pure_helper_expr(expr, function_map, memo, visiting)
        }
        _ => false,
    }
}

fn is_pure_helper_expr(
    expr: &NirExpr,
    function_map: &BTreeMap<&str, &NirFunction>,
    memo: &mut BTreeMap<String, bool>,
    visiting: &mut BTreeSet<String>,
) -> bool {
    match expr {
        NirExpr::Call { callee, args } => {
            let Some(function) = function_map.get(callee.as_str()).copied() else {
                return false;
            };
            is_pure_helper_function(function, function_map, memo, visiting)
                && args
                    .iter()
                    .all(|arg| is_pure_helper_expr(arg, function_map, memo, visiting))
        }
        NirExpr::MethodCall { .. } => false,
        NirExpr::Await(_) | NirExpr::Instantiate { .. } => false,
        NirExpr::CastI64ToI32(inner)
        | NirExpr::CastI32ToI64(inner)
        | NirExpr::CastI64ToBool(inner)
        | NirExpr::CastBoolToI64(inner) => is_pure_helper_expr(inner, function_map, memo, visiting),
        NirExpr::CastI64ToF32(inner) | NirExpr::CastF32ToI64(inner) => {
            is_pure_helper_expr(inner, function_map, memo, visiting)
        }
        NirExpr::CastI64ToF64(inner) | NirExpr::CastF64ToI64(inner) => {
            is_pure_helper_expr(inner, function_map, memo, visiting)
        }
        NirExpr::StructLiteral { fields, .. } => fields
            .iter()
            .all(|(_, value)| is_pure_helper_expr(value, function_map, memo, visiting)),
        NirExpr::FieldAccess { base, .. } => {
            is_pure_helper_expr(base, function_map, memo, visiting)
        }
        NirExpr::Binary { lhs, rhs, .. } => {
            is_pure_helper_expr(lhs, function_map, memo, visiting)
                && is_pure_helper_expr(rhs, function_map, memo, visiting)
        }
        _ => nir_expr_effect_class(expr) == NirExprEffectClass::Pure,
    }
}

pub(super) fn substitute_branch_binding(
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
        NirExpr::NetworkResult { value, state } => NirExpr::NetworkResult {
            value: Box::new(substitute_branch_binding(value, binding_name, binding_value)),
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
            value: Box::new(substitute_branch_binding(value, binding_name, binding_value)),
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
            value: Box::new(substitute_branch_binding(value, binding_name, binding_value)),
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
            value: Box::new(substitute_branch_binding(value, binding_name, binding_value)),
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

pub(super) fn substitute_prepared_terminal_branch(
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
    }
}

pub(super) fn substitute_prepared_loop_body(
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

pub(super) fn substitute_stmt_bindings(stmt: &NirStmt, bindings: &[(String, NirExpr)]) -> NirStmt {
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

pub(super) fn prepare_terminal_branch(
    stmts: &[NirStmt],
    pure_helpers: &BTreeSet<String>,
) -> Option<PreparedTerminalBranch> {
    match stmts {
        [NirStmt::Return(Some(value))] | [NirStmt::Expr(value)] => {
            Some(PreparedTerminalBranch::Return(value.clone()))
        }
        [NirStmt::Print(print), NirStmt::Return(Some(returned))]
        | [NirStmt::Print(print), NirStmt::Expr(returned)] => {
            Some(PreparedTerminalBranch::PrintReturn {
                print: print.clone(),
                returned: returned.clone(),
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
