use super::*;

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

pub(in crate::lowering) fn normalize_pure_bool_test_expr(expr: NirExpr) -> NirExpr {
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
