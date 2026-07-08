use super::*;

pub(in crate::lowering) fn extract_pure_branch_binding(
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

pub(in crate::lowering) fn is_terminal_branch_pure_expr(
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
        NirExpr::VariantIs { base, .. } | NirExpr::VariantFieldAccess { base, .. } => {
            is_terminal_branch_pure_expr(base, pure_helpers)
        }
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
