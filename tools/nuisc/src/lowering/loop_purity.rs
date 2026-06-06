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
        | NirExpr::NetworkConfigReady(inner)
        | NirExpr::NetworkSendReady(inner)
        | NirExpr::NetworkRecvReady(inner)
        | NirExpr::NetworkAcceptReady(inner)
        | NirExpr::NetworkValue(inner)
        | NirExpr::CastI64ToI32(inner) => is_terminal_branch_pure_expr(inner, pure_helpers),
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
    let result = !function.is_async
        && matches!(
            function.body.as_slice(),
            [NirStmt::Return(Some(expr))]
                if is_pure_helper_expr(expr, function_map, memo, visiting)
        );
    visiting.remove(&function.name);
    memo.insert(function.name.clone(), result);
    result
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
        NirExpr::CastI64ToI32(inner) => is_pure_helper_expr(inner, function_map, memo, visiting),
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
        NirExpr::Call { callee, args } => NirExpr::Call {
            callee: callee.clone(),
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
        [NirStmt::Return(Some(value))] => Some(PreparedTerminalBranch::Return(value.clone())),
        [NirStmt::Print(print), NirStmt::Return(Some(returned))] => {
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
