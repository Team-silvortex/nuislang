use super::*;

const MAX_INLINE_EXPR_NODES: usize = 4096;
const MAX_INLINE_EXPR_DEPTH: usize = 128;

fn charge_expr(
    expr: &NirExpr,
    replacement: Option<(&str, usize)>,
    remaining: &mut usize,
    depth: usize,
) -> bool {
    if depth > MAX_INLINE_EXPR_DEPTH {
        return false;
    }
    let cost = match (expr, replacement) {
        (NirExpr::Var(name), Some((binding, size))) if name == binding => size,
        _ => 1,
    };
    let Some(left) = remaining.checked_sub(cost) else {
        return false;
    };
    *remaining = left;
    let mut valid = true;
    let mut charge_child = |child: &NirExpr| {
        if valid {
            valid = charge_expr(child, replacement, remaining, depth + 1);
        }
    };
    // Count only expression shapes whose complete children are known here.
    // Unknown shapes decline this optimization rather than undercounting a subtree.
    match expr {
        NirExpr::Bool(_)
        | NirExpr::Text(_)
        | NirExpr::Int(_)
        | NirExpr::F32(_)
        | NirExpr::F64(_)
        | NirExpr::Var(_)
        | NirExpr::Null => {}
        NirExpr::CastI64ToI32(inner)
        | NirExpr::CastI32ToI64(inner)
        | NirExpr::CastI64ToBool(inner)
        | NirExpr::CastBoolToI64(inner)
        | NirExpr::CastI64ToF32(inner)
        | NirExpr::CastF32ToI64(inner)
        | NirExpr::CastI64ToF64(inner)
        | NirExpr::CastF64ToI64(inner)
        | NirExpr::CpuTaskCompleted(inner)
        | NirExpr::CpuTaskTimedOut(inner)
        | NirExpr::CpuTaskCancelled(inner)
        | NirExpr::CpuTaskFailed(inner)
        | NirExpr::CpuTaskValue(inner)
        | NirExpr::CpuMutexValue(inner)
        | NirExpr::LoadValue(inner)
        | NirExpr::LoadNext(inner)
        | NirExpr::BufferLen(inner)
        | NirExpr::BytesLen(inner)
        | NirExpr::IsNull(inner)
        | NirExpr::OwnedObjectSize(inner)
        | NirExpr::FieldAccess { base: inner, .. }
        | NirExpr::VariantIs { base: inner, .. }
        | NirExpr::VariantFieldAccess { base: inner, .. } => charge_child(inner),
        NirExpr::Call { args, .. }
        | NirExpr::CpuMutexCapability {
            op: NirMutexCapabilityOp::LeaseValue,
            args,
        } => {
            for arg in args {
                charge_child(arg);
            }
        }
        NirExpr::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                charge_child(value);
            }
        }
        NirExpr::Binary { lhs, rhs, .. }
        | NirExpr::LoadAt {
            buffer: lhs,
            index: rhs,
        }
        | NirExpr::OwnedObjectReadI64 {
            object: lhs,
            index: rhs,
        } => {
            charge_child(lhs);
            charge_child(rhs);
        }
        _ => return false,
    }
    valid
}

pub(super) fn bounded_inline_seed(expr: &NirExpr) -> Option<NirExpr> {
    let mut remaining = MAX_INLINE_EXPR_NODES;
    charge_expr(expr, None, &mut remaining, 0).then(|| expr.clone())
}

pub(super) fn bounded_inline_substitution(
    expr: &NirExpr,
    name: &str,
    value: &NirExpr,
) -> Option<NirExpr> {
    let mut remaining = MAX_INLINE_EXPR_NODES;
    if !charge_expr(value, None, &mut remaining, 0) {
        return None;
    }
    let size = MAX_INLINE_EXPR_NODES - remaining;
    remaining = MAX_INLINE_EXPR_NODES;
    if !charge_expr(expr, Some((name, size)), &mut remaining, 0) {
        return None;
    }
    // This is an optimization limit, not a source-language or direct-call limit.
    let substituted = substitute_branch_binding(expr, name, value);
    remaining = MAX_INLINE_EXPR_NODES;
    charge_expr(&substituted, None, &mut remaining, 0).then_some(substituted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_budget_counts_cast_depth_and_declines_unknown_shapes() {
        let mut expr = NirExpr::Int(1);
        for _ in 0..MAX_INLINE_EXPR_DEPTH / 2 {
            expr = NirExpr::CastI32ToI64(Box::new(NirExpr::CastI64ToI32(Box::new(expr))));
        }
        assert!(bounded_inline_seed(&expr).is_some());
        expr = NirExpr::CastI64ToI32(Box::new(expr));
        assert!(bounded_inline_seed(&expr).is_none());
        assert!(bounded_inline_seed(&NirExpr::Await(Box::new(NirExpr::Int(0)))).is_none());
    }
}
