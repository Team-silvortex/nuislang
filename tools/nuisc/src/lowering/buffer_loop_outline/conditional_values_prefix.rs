use super::*;

// Bound source/derived pure prefixes before recursive kind inference or cloning.
// This is not general multi-binding, effectful or outer-state branch lowering.
pub(in crate::lowering::buffer_loop_outline) fn bounded(body: &[NirStmt]) -> bool {
    if body.len() > 32 {
        return false;
    }
    let mut pending = Vec::new();
    for stmt in body {
        let value = match stmt {
            NirStmt::Let { value, .. } | NirStmt::Const { value, .. } => value,
            _ => return false,
        };
        pending.push((value, 0));
    }
    expressions(pending)
}

pub(in crate::lowering::buffer_loop_outline) fn expression(expr: &NirExpr) -> bool {
    expressions(vec![(expr, 0)])
}

pub(in crate::lowering::buffer_loop_outline) fn return_body(body: &[NirStmt]) -> bool {
    let [locals @ .., NirStmt::Return(Some(value))] = body else {
        return false;
    };
    if locals.is_empty() || body.len() > 32 {
        return false;
    }
    let mut pending = vec![(value, 0, true)];
    for stmt in locals {
        let value = match stmt {
            NirStmt::Let { value, .. } | NirStmt::Const { value, .. } => value,
            _ => return false,
        };
        pending.push((value, 0, true));
    }
    computed_expression_roots(pending)
}

fn expressions(pending: Vec<(&NirExpr, usize)>) -> bool {
    expression_roots(
        pending
            .into_iter()
            .map(|(expr, depth)| (expr, depth, false))
            .collect(),
    )
}

pub(in crate::lowering::buffer_loop_outline) fn expression_roots(
    pending: Vec<(&NirExpr, usize, bool)>,
) -> bool {
    roots(pending, false)
}

pub(in crate::lowering::buffer_loop_outline) fn computed_logical_root(expr: &NirExpr) -> bool {
    computed_expression_roots(vec![(expr, 0, true)])
}

// Return trees share one budget across all roots, including duplicated suffixes.
// Only direct logical children inherit permission; ordinary leaf expressions
// cannot hide logical gates in arguments, comparisons or aggregate fields.
pub(in crate::lowering::buffer_loop_outline) fn computed_expression_roots(
    pending: Vec<(&NirExpr, usize, bool)>,
) -> bool {
    roots(pending, true)
}

fn roots(mut pending: Vec<(&NirExpr, usize, bool)>, computed_gate: bool) -> bool {
    let mut work = 4096usize;
    let mut logical_edges = 32usize;
    while let Some((expr, depth, logical_root)) = pending.pop() {
        if depth >= 64 || work == 0 {
            return false;
        }
        work -= 1;
        match expr {
            NirExpr::Int(_)
            | NirExpr::Bool(_)
            | NirExpr::F32(_)
            | NirExpr::F64(_)
            | NirExpr::Var(_) => {}
            NirExpr::Binary {
                op: NirBinaryOp::And | NirBinaryOp::Or,
                lhs,
                rhs,
            } => {
                if !logical_root
                    || (!computed_gate && depth != 0)
                    || (computed_gate && logical_edges == 0)
                    || (!computed_gate
                        && !matches!(lhs.as_ref(), NirExpr::Bool(_) | NirExpr::Var(_)))
                    || pending.len() + 2 > work
                {
                    return false;
                }
                if computed_gate {
                    logical_edges -= 1;
                }
                pending.extend([
                    (lhs.as_ref(), depth + 1, computed_gate),
                    (rhs.as_ref(), depth + 1, computed_gate),
                ]);
            }
            NirExpr::Binary { lhs, rhs, .. } => {
                if pending.len() + 2 > work {
                    return false;
                }
                pending.extend([
                    (lhs.as_ref(), depth + 1, false),
                    (rhs.as_ref(), depth + 1, false),
                ]);
            }
            NirExpr::Call { args, .. } => {
                if pending.len() + args.len() > work {
                    return false;
                }
                pending.extend(args.iter().map(|arg| (arg, depth + 1, false)));
            }
            NirExpr::StructLiteral { fields, .. } => {
                if pending.len() + fields.len() > work {
                    return false;
                }
                pending.extend(fields.iter().map(|(_, value)| (value, depth + 1, false)));
            }
            NirExpr::FieldAccess { base, .. }
            | NirExpr::CastI64ToI32(base)
            | NirExpr::CastI32ToI64(base)
            | NirExpr::PackF32Word(base)
            | NirExpr::UnpackF32Word(base)
            | NirExpr::PackF64Word(base)
            | NirExpr::UnpackF64Word(base) => pending.push((base.as_ref(), depth + 1, false)),
            _ => return false,
        }
    }
    true
}
