use super::*;

const MAX_WORK: usize = 65_536;
const MAX_DEPTH: usize = 64;

#[cfg(test)]
#[path = "return_signal_proof_tests.rs"]
mod tests;

// The caller supplies provenance from returns::prepare, never a source name.
// Entering a rewritten loop implies pending == 0: every child publication is
// immediately propagated until the function returns. Only return-owned breaks
// may therefore reuse the driver's zero-seeded canonical break bit.
pub(super) fn can_share(body: &[NirStmt], signal: &str) -> bool {
    Proof {
        signal,
        remaining: MAX_WORK,
    }
    .block(body, true, 0)
        == Some(true)
}

struct Proof<'a> {
    signal: &'a str,
    remaining: usize,
}

impl Proof<'_> {
    fn tick(&mut self, depth: usize) -> Option<()> {
        if depth >= MAX_DEPTH {
            return None;
        }
        self.remaining = self.remaining.checked_sub(1)?;
        Some(())
    }

    fn block(&mut self, body: &[NirStmt], owns_exit: bool, depth: usize) -> Option<bool> {
        let mut found = false;
        for (index, stmt) in body.iter().enumerate() {
            self.tick(depth)?;
            match stmt {
                NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. } => {
                    if name == self.signal {
                        if !self.publication(stmt) || body.get(index + 1) != Some(&NirStmt::Break) {
                            return None;
                        }
                    } else {
                        self.expression(value, depth + 1)?;
                    }
                }
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    let propagation = matches!(condition, NirExpr::Binary {
                        op: NirBinaryOp::Eq, lhs, rhs,
                    } if matches!(lhs.as_ref(), NirExpr::Var(name) if name == self.signal)
                        && rhs.as_ref() == &NirExpr::Int(1))
                        && then_body == &[NirStmt::Break]
                        && else_body.is_empty();
                    if propagation {
                        self.remaining = self.remaining.checked_sub(4)?;
                        found |= owns_exit;
                    } else {
                        self.expression(condition, depth + 1)?;
                        found |= self.block(then_body, owns_exit, depth + 1)?;
                        found |= self.block(else_body, owns_exit, depth + 1)?;
                    }
                }
                NirStmt::While { condition, body } => {
                    self.expression(condition, depth + 1)?;
                    // Child breaks do not terminate this loop, but child writes
                    // to the shared signal still need canonical publications.
                    self.block(body, false, depth + 1)?;
                }
                NirStmt::Break => {
                    if index + 1 != body.len()
                        || (owns_exit
                            && !index
                                .checked_sub(1)
                                .is_some_and(|i| self.publication(&body[i])))
                    {
                        return None;
                    }
                    found |= owns_exit;
                }
                NirStmt::Continue if index + 1 == body.len() => {}
                NirStmt::Expr(value) => self.expression(value, depth + 1)?,
                _ => return None,
            }
        }
        Some(found)
    }

    fn publication(&self, stmt: &NirStmt) -> bool {
        matches!(stmt, NirStmt::Let { name, ty: Some(ty), value: NirExpr::Int(1) }
            if name == self.signal && ty == &scalar_type("i64"))
    }

    fn expression(&mut self, expr: &NirExpr, depth: usize) -> Option<()> {
        self.tick(depth)?;
        match expr {
            NirExpr::Var(name) if name != self.signal => {}
            NirExpr::Binary { lhs, rhs, .. } => {
                self.expression(lhs, depth + 1)?;
                self.expression(rhs, depth + 1)?;
            }
            NirExpr::Call { args, .. } => {
                for arg in args {
                    self.expression(arg, depth + 1)?;
                }
            }
            NirExpr::StructLiteral { fields, .. } => {
                for (_, value) in fields {
                    self.expression(value, depth + 1)?;
                }
            }
            NirExpr::FieldAccess { base, .. }
            | NirExpr::CastI64ToI32(base)
            | NirExpr::CastI32ToI64(base)
            | NirExpr::CastBoolToI64(base)
            | NirExpr::CastI64ToBool(base)
            | NirExpr::PackF64Word(base)
            | NirExpr::UnpackF64Word(base)
            | NirExpr::PackF32Word(base)
            | NirExpr::UnpackF32Word(base) => self.expression(base, depth + 1)?,
            NirExpr::Int(_) | NirExpr::Bool(_) | NirExpr::F32(_) | NirExpr::F64(_) => {}
            _ => return None,
        }
        Some(())
    }
}
