use super::*;

pub(in crate::lowering) fn chain_statement_effect(state: &mut LoweringState<'_>, anchor: &str) {
    if let Some(previous) = state.last_effect_anchor.as_ref() {
        if previous != anchor
            && !state.yir.edges.iter().any(|edge| {
                edge.from == *previous && edge.to == anchor && matches!(edge.kind, EdgeKind::Effect)
            })
        {
            state.yir.edges.push(Edge {
                kind: EdgeKind::Effect,
                from: previous.clone(),
                to: anchor.to_owned(),
            });
        }
    }
    state.last_effect_anchor = Some(anchor.to_owned());
}

pub(in crate::lowering) fn expr_requires_statement_anchor(expr: &NirExpr) -> bool {
    if nir_expr_effect_class(expr) != NirExprEffectClass::Pure {
        return true;
    }

    match expr {
        NirExpr::CastI64ToI32(value)
        | NirExpr::CastI32ToI64(value)
        | NirExpr::CastI64ToBool(value)
        | NirExpr::CastBoolToI64(value)
        | NirExpr::CastI64ToF32(value)
        | NirExpr::CastF32ToI64(value)
        | NirExpr::CastI64ToF64(value)
        | NirExpr::CastF64ToI64(value)
        | NirExpr::IsNull(value)
        | NirExpr::FieldAccess { base: value, .. } => expr_requires_statement_anchor(value),
        NirExpr::Binary { lhs, rhs, .. } => {
            expr_requires_statement_anchor(lhs) || expr_requires_statement_anchor(rhs)
        }
        NirExpr::StructLiteral { fields, .. } => fields
            .iter()
            .any(|(_, value)| expr_requires_statement_anchor(value)),
        NirExpr::Null
        | NirExpr::Bool(_)
        | NirExpr::Text(_)
        | NirExpr::Int(_)
        | NirExpr::F32(_)
        | NirExpr::F64(_)
        | NirExpr::Var(_) => false,
        _ => false,
    }
}

pub(in crate::lowering) fn chain_nonpure_expr_stmt(
    expr: &NirExpr,
    lowered: &str,
    state: &mut LoweringState<'_>,
) {
    if expr_requires_statement_anchor(expr) {
        chain_statement_effect(state, lowered);
    }
}

pub(in crate::lowering) fn refresh_const_binding(
    const_bindings: &mut BTreeMap<String, NirExpr>,
    name: &str,
    value: &NirExpr,
) {
    if eval_const_i64_with_env(value, const_bindings, &mut BTreeSet::new()).is_some() {
        const_bindings.insert(name.to_owned(), value.clone());
    } else {
        const_bindings.remove(name);
    }
}

pub(in crate::lowering) fn eval_const_i64_with_env(
    expr: &NirExpr,
    const_bindings: &BTreeMap<String, NirExpr>,
    visited: &mut BTreeSet<String>,
) -> Option<i64> {
    match expr {
        NirExpr::Int(value) => Some(*value),
        NirExpr::Bool(value) => Some(i64::from(*value)),
        NirExpr::Var(name) => {
            if !visited.insert(name.clone()) {
                return None;
            }
            let resolved = const_bindings
                .get(name)
                .and_then(|value| eval_const_i64_with_env(value, const_bindings, visited));
            visited.remove(name);
            resolved
        }
        NirExpr::CastI64ToI32(value)
        | NirExpr::CastI32ToI64(value)
        | NirExpr::CastBoolToI64(value)
        | NirExpr::CastF32ToI64(value)
        | NirExpr::CastF64ToI64(value) => eval_const_i64_with_env(value, const_bindings, visited),
        NirExpr::CastI64ToBool(value) => Some(i64::from(
            eval_const_i64_with_env(value, const_bindings, visited)? != 0,
        )),
        NirExpr::Binary { op, lhs, rhs } => {
            let lhs = eval_const_i64_with_env(lhs, const_bindings, visited)?;
            let rhs = eval_const_i64_with_env(rhs, const_bindings, visited)?;
            match op {
                NirBinaryOp::And => Some(i64::from(lhs != 0 && rhs != 0)),
                NirBinaryOp::Or => Some(i64::from(lhs != 0 || rhs != 0)),
                NirBinaryOp::Add => Some(lhs + rhs),
                NirBinaryOp::Sub => Some(lhs - rhs),
                NirBinaryOp::Mul => Some(lhs * rhs),
                NirBinaryOp::Div => (rhs != 0).then_some(lhs / rhs),
                NirBinaryOp::Rem => (rhs != 0).then_some(lhs % rhs),
                NirBinaryOp::Eq => Some(i64::from(lhs == rhs)),
                NirBinaryOp::Ne => Some(i64::from(lhs != rhs)),
                NirBinaryOp::Lt => Some(i64::from(lhs < rhs)),
                NirBinaryOp::Le => Some(i64::from(lhs <= rhs)),
                NirBinaryOp::Gt => Some(i64::from(lhs > rhs)),
                NirBinaryOp::Ge => Some(i64::from(lhs >= rhs)),
            }
        }
        _ => None,
    }
}

pub(in crate::lowering) fn eval_const_bool_with_env(
    expr: &NirExpr,
    const_bindings: &BTreeMap<String, NirExpr>,
) -> Option<bool> {
    match expr {
        NirExpr::Bool(value) => Some(*value),
        _ => Some(eval_const_i64_with_env(expr, const_bindings, &mut BTreeSet::new())? != 0),
    }
}
