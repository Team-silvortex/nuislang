use super::*;

const MAX_WORK: usize = 65_536;
const MAX_DEPTH: usize = 64;

// An over-approximation of observation, not dead-code elimination. Even reads
// after a redefinition or in an untaken arm count. Unknown syntax or exhausted
// work invalidates the whole local proof and keeps index recovery conservative.
pub(super) struct Plan {
    last: BTreeMap<String, usize>,
    remaining: usize,
}

impl Plan {
    pub(super) fn new(body: &[NirStmt], continuation: Option<&BTreeSet<String>>) -> Option<Self> {
        let continuation = continuation?;
        let remaining = MAX_WORK.checked_sub(continuation.len())?;
        let mut plan = Self {
            last: continuation
                .iter()
                .map(|name| (name.clone(), usize::MAX))
                .collect(),
            remaining,
        };
        for (index, stmt) in body.iter().enumerate() {
            plan.statement(stmt, index, 0)?;
        }
        Some(plan)
    }

    pub(super) fn after(&mut self, index: usize) -> Option<BTreeSet<String>> {
        // Bound repeated suffix materialization as well as the initial scan.
        self.remaining = self.remaining.checked_sub(self.last.len())?;
        Some(
            self.last
                .iter()
                .filter(|(_, last)| **last > index)
                .map(|(name, _)| name.clone())
                .collect(),
        )
    }

    fn step(&mut self, depth: usize) -> Option<()> {
        if depth >= MAX_DEPTH {
            return None;
        }
        self.remaining = self.remaining.checked_sub(1)?;
        Some(())
    }

    fn statement(&mut self, stmt: &NirStmt, index: usize, depth: usize) -> Option<()> {
        self.step(depth)?;
        match stmt {
            NirStmt::Let { value, .. }
            | NirStmt::Const { value, .. }
            | NirStmt::Expr(value)
            | NirStmt::Return(Some(value)) => self.expression(value, index, depth + 1),
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                self.expression(condition, index, depth + 1)?;
                for stmt in then_body.iter().chain(else_body) {
                    self.statement(stmt, index, depth + 1)?;
                }
                Some(())
            }
            NirStmt::While { condition, body } => {
                self.expression(condition, index, depth + 1)?;
                for stmt in body {
                    self.statement(stmt, index, depth + 1)?;
                }
                Some(())
            }
            NirStmt::Break | NirStmt::Continue | NirStmt::Return(None) => Some(()),
            _ => None,
        }
    }

    fn expression(&mut self, expr: &NirExpr, index: usize, depth: usize) -> Option<()> {
        self.step(depth)?;
        match expr {
            NirExpr::Var(name) => {
                self.last
                    .entry(name.clone())
                    .and_modify(|last| *last = (*last).max(index))
                    .or_insert(index);
            }
            NirExpr::Binary { lhs, rhs, .. } => {
                self.expression(lhs, index, depth + 1)?;
                self.expression(rhs, index, depth + 1)?;
            }
            NirExpr::Call { args, .. } => {
                for arg in args {
                    self.expression(arg, index, depth + 1)?;
                }
            }
            NirExpr::StructLiteral { fields, .. } => {
                for (_, value) in fields {
                    self.expression(value, index, depth + 1)?;
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
            | NirExpr::UnpackF32Word(base) => self.expression(base, index, depth + 1)?,
            NirExpr::Int(_) | NirExpr::Bool(_) | NirExpr::F32(_) | NirExpr::F64(_) => {}
            _ => return None,
        }
        Some(())
    }
}

#[cfg(test)]
#[path = "continuation_reads_tests.rs"]
mod tests;
