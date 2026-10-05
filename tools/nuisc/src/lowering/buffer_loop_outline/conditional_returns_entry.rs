use super::*;

#[cfg(test)]
#[path = "conditional_computed_gates_tests.rs"]
mod computed_tests;
#[cfg(test)]
#[path = "conditional_returns_entry_logical_tests.rs"]
mod logical_tests;
#[cfg(test)]
#[path = "conditional_returns_entry_tests.rs"]
mod tests;

pub(super) fn admitted(
    condition: &NirExpr,
    scope: &Scope,
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
) -> bool {
    // Preflight precedes recursive typing/cloning. This independent source-site
    // proof does not widen logical return roots or branch/capture contracts.
    (conditional_values::prefix::expression(condition)
        || single_edge(condition, scope)
        || computed_edge(condition))
        && control_values::value_type(condition, scope, catalog, layouts)
            == Some(scalar_type("bool"))
}

fn computed_edge(condition: &NirExpr) -> bool {
    // Source entries and separately validated return arms share computed roots.
    // Generic expression roots retain the atom-only mode; captures stay separate.
    matches!(
        condition,
        NirExpr::Binary {
            op: NirBinaryOp::And | NirBinaryOp::Or,
            ..
        }
    ) && conditional_values::prefix::computed_logical_root(condition)
}

fn single_edge(condition: &NirExpr, scope: &Scope) -> bool {
    let NirExpr::Binary {
        op: NirBinaryOp::And | NirBinaryOp::Or,
        lhs,
        ..
    } = condition
    else {
        return false;
    };
    // The saved parent bool is a logical value root. The shared value route
    // guards its complete RHS; this grants no logical internal-arm authority.
    predicate(lhs, scope)
        && conditional_values::prefix::expression_roots(vec![(condition, 0, true)])
}

pub(super) fn has_work(condition: &NirExpr) -> bool {
    let mut pending = vec![condition];
    while let Some(value) = pending.pop() {
        match value {
            NirExpr::Call { .. }
            | NirExpr::Binary {
                op: NirBinaryOp::Div | NirBinaryOp::Rem,
                ..
            } => return true,
            NirExpr::Binary { lhs, rhs, .. } => pending.extend([lhs.as_ref(), rhs.as_ref()]),
            NirExpr::StructLiteral { fields, .. } => {
                pending.extend(fields.iter().map(|(_, value)| value));
            }
            NirExpr::FieldAccess { base, .. }
            | NirExpr::CastI64ToI32(base)
            | NirExpr::CastI32ToI64(base)
            | NirExpr::PackF32Word(base)
            | NirExpr::UnpackF32Word(base)
            | NirExpr::PackF64Word(base)
            | NirExpr::UnpackF64Word(base) => pending.push(base),
            _ => {}
        }
    }
    false
}
