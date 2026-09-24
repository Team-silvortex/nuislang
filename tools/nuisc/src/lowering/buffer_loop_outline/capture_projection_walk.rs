use super::*;

// Scan every expression kind, including unsupported callers, before changing a
// private signature. An unrewritable use must veto the optimization, not vanish.
pub(super) fn visit(body: &[NirStmt], mut visitor: impl FnMut(&NirExpr) -> bool) {
    let mut blocks = vec![body];
    let mut expressions = Vec::new();
    while let Some(body) = blocks.pop() {
        for stmt in body {
            match stmt {
                NirStmt::Let { value, .. }
                | NirStmt::Const { value, .. }
                | NirStmt::Print(value)
                | NirStmt::Expr(value)
                | NirStmt::Await(value)
                | NirStmt::Return(Some(value)) => expressions.push(value),
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    expressions.push(condition);
                    blocks.extend([then_body.as_slice(), else_body.as_slice()]);
                }
                NirStmt::While { condition, body } => {
                    expressions.push(condition);
                    blocks.push(body);
                }
                NirStmt::Break | NirStmt::Continue | NirStmt::Return(None) => {}
            }
        }
    }
    while let Some(expr) = expressions.pop() {
        if visitor(expr) {
            crate::nir_walk::walk_child_exprs(expr, &mut |child| expressions.push(child));
        }
    }
}

pub(super) fn supported(body: &[NirStmt]) -> bool {
    let mut valid = true;
    visit(body, |expr| {
        valid &= matches!(
            expr,
            NirExpr::Var(_)
                | NirExpr::Int(_)
                | NirExpr::Bool(_)
                | NirExpr::F32(_)
                | NirExpr::F64(_)
                | NirExpr::Binary { .. }
                | NirExpr::Call { .. }
                | NirExpr::StructLiteral { .. }
                | NirExpr::FieldAccess { .. }
                | NirExpr::CastI64ToI32(_)
                | NirExpr::CastI32ToI64(_)
                | NirExpr::CastBoolToI64(_)
                | NirExpr::CastI64ToBool(_)
        );
        true
    });
    valid
}

pub(super) fn rewrite(body: &mut [NirStmt], visitor: impl FnMut(&mut NirExpr)) {
    let mut blocks = vec![body];
    let mut expressions = Vec::new();
    while let Some(body) = blocks.pop() {
        for stmt in body {
            match stmt {
                NirStmt::Let { value, .. }
                | NirStmt::Const { value, .. }
                | NirStmt::Print(value)
                | NirStmt::Expr(value)
                | NirStmt::Await(value)
                | NirStmt::Return(Some(value)) => expressions.push(value),
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    expressions.push(condition);
                    blocks.extend([then_body.as_mut_slice(), else_body.as_mut_slice()]);
                }
                NirStmt::While { condition, body } => {
                    expressions.push(condition);
                    blocks.push(body);
                }
                NirStmt::Break | NirStmt::Continue | NirStmt::Return(None) => {}
            }
        }
    }
    rewrite_expressions(expressions, visitor);
}

pub(super) fn rewrite_expr(expr: &mut NirExpr, visitor: impl FnMut(&mut NirExpr)) {
    rewrite_expressions(vec![expr], visitor);
}

fn rewrite_expressions(mut expressions: Vec<&mut NirExpr>, mut visitor: impl FnMut(&mut NirExpr)) {
    while let Some(expr) = expressions.pop() {
        visitor(expr);
        match expr {
            NirExpr::Binary { lhs, rhs, .. } => expressions.extend([lhs.as_mut(), rhs.as_mut()]),
            NirExpr::Call { args, .. } => expressions.extend(args),
            NirExpr::StructLiteral { fields, .. } => {
                expressions.extend(fields.iter_mut().map(|(_, value)| value));
            }
            NirExpr::FieldAccess { base, .. }
            | NirExpr::CastI64ToI32(base)
            | NirExpr::CastI32ToI64(base)
            | NirExpr::CastBoolToI64(base)
            | NirExpr::CastI64ToBool(base) => expressions.push(base),
            NirExpr::Var(_)
            | NirExpr::Int(_)
            | NirExpr::Bool(_)
            | NirExpr::F32(_)
            | NirExpr::F64(_) => {}
            _ => unreachable!("private capture rewrite was preflighted"),
        }
    }
}
