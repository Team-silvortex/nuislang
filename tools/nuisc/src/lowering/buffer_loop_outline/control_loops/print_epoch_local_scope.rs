use super::*;

enum Visit<'a> {
    Stmt(&'a NirStmt, usize),
    Expr(&'a NirExpr, usize),
}

// This is a bounded non-use proof, not another source-admission classifier.
pub(super) fn excludes(body: &[NirStmt], name: &str, work: &mut usize) -> Option<()> {
    if body.len() > *work {
        return None;
    }
    let mut pending = body
        .iter()
        .map(|stmt| Visit::Stmt(stmt, 0))
        .collect::<Vec<_>>();
    while let Some(visit) = pending.pop() {
        *work = work.checked_sub(1)?;
        match visit {
            Visit::Stmt(stmt, depth) if depth <= 32 => match stmt {
                NirStmt::Let {
                    name: bound, value, ..
                }
                | NirStmt::Const {
                    name: bound, value, ..
                } if bound != name => pending.push(Visit::Expr(value, 0)),
                NirStmt::Print(value) | NirStmt::Expr(value) | NirStmt::Return(Some(value)) => {
                    pending.push(Visit::Expr(value, 0));
                }
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    if then_body
                        .len()
                        .checked_add(else_body.len())?
                        .checked_add(1)?
                        > work.saturating_sub(pending.len())
                    {
                        return None;
                    }
                    pending.push(Visit::Expr(condition, 0));
                    pending.extend(
                        then_body
                            .iter()
                            .chain(else_body)
                            .map(|stmt| Visit::Stmt(stmt, depth + 1)),
                    );
                }
                NirStmt::While { condition, body } => {
                    if body.len().checked_add(1)? > work.saturating_sub(pending.len()) {
                        return None;
                    }
                    pending.push(Visit::Expr(condition, 0));
                    pending.extend(body.iter().map(|stmt| Visit::Stmt(stmt, depth + 1)));
                }
                NirStmt::Return(None) | NirStmt::Break | NirStmt::Continue => {}
                _ => return None,
            },
            Visit::Expr(expr, depth) if depth <= 64 => match expr {
                NirExpr::Var(value) if value != name => {}
                NirExpr::Int(_) | NirExpr::Bool(_) | NirExpr::F32(_) | NirExpr::F64(_) => {}
                NirExpr::FieldAccess { base, .. }
                | NirExpr::CastI64ToI32(base)
                | NirExpr::CastI32ToI64(base)
                | NirExpr::PackF32Word(base)
                | NirExpr::UnpackF32Word(base)
                | NirExpr::PackF64Word(base)
                | NirExpr::UnpackF64Word(base)
                | NirExpr::CastBoolToI64(base)
                | NirExpr::CastI64ToBool(base) => pending.push(Visit::Expr(base, depth + 1)),
                NirExpr::Binary { lhs, rhs, .. } => {
                    pending.push(Visit::Expr(lhs, depth + 1));
                    pending.push(Visit::Expr(rhs, depth + 1));
                }
                NirExpr::Call { callee, args } if callee != name => {
                    if args.len() > work.saturating_sub(pending.len()) {
                        return None;
                    }
                    pending.extend(args.iter().map(|arg| Visit::Expr(arg, depth + 1)));
                }
                NirExpr::StructLiteral { fields, .. } => {
                    if fields.len() > work.saturating_sub(pending.len()) {
                        return None;
                    }
                    pending.extend(
                        fields
                            .iter()
                            .map(|(_, value)| Visit::Expr(value, depth + 1)),
                    );
                }
                _ => return None,
            },
            _ => return None,
        }
    }
    Some(())
}
