use super::*;

#[path = "return_effect_prefix_bounds.rs"]
mod bounds;

#[path = "return_effect_storage.rs"]
mod storage;
pub(super) use storage::verify as verify_shared;

#[cfg(test)]
#[path = "return_effect_prefix_tests.rs"]
mod tests;

// This certifies only independent return storage, not arbitrary effectful
// optimization. It does not invoke the rewriter or grant call/purity authority.
pub(super) fn verify_independent(
    function: &NirFunction,
    plan: &Prepared,
    layouts: &control_values::CarryLayouts,
) -> Option<()> {
    let mut budget = bounds::Budget::new();
    let mut names = function.params.iter().map(|p| p.name.clone()).collect();
    bounds::preflight(&function.body, &mut budget, Some(&mut names))?;
    bounds::preflight(&plan.body, &mut budget, None)?;
    let result = function.return_type.as_ref()?;
    if !control_values::supported_type(result, layouts) {
        return None;
    }
    let [flag, payload, tail @ ..] = plan.body.as_slice() else {
        return None;
    };
    let NirStmt::Let {
        name: payload_name,
        ty: Some(ty),
        value,
    } = payload
    else {
        return None;
    };
    let flag_name = plan.signal.name();
    if !plan.structs.is_empty()
        || ty != result
        || value != &control_values::zero_value(result, layouts)
        || !assigned(flag, flag_name, &scalar_type("i64"), &NirExpr::Int(0))
        || payload_name == flag_name
        || names.contains(flag_name)
        || names.contains(payload_name)
    {
        return None;
    }
    let proof = Proof {
        flag: flag_name,
        payload: payload_name,
        result,
    };
    proof.block(&function.body, tail, false)?.then_some(())
}

fn assigned(stmt: &NirStmt, name: &str, ty: &NirTypeRef, value: &NirExpr) -> bool {
    matches!(stmt, NirStmt::Let { name: n, ty: Some(t), value: v }
        if n == name && t == ty && v == value)
}

struct Proof<'a> {
    flag: &'a str,
    payload: &'a str,
    result: &'a NirTypeRef,
}

impl Proof<'_> {
    fn block(&self, source: &[NirStmt], rewritten: &[NirStmt], in_loop: bool) -> Option<bool> {
        let mut output = rewritten.iter();
        let mut returns = false;
        for (index, stmt) in source.iter().enumerate() {
            match stmt {
                NirStmt::Return(Some(value)) if in_loop => {
                    // A failure in any argument/check/call must precede both
                    // pending publication and loop evacuation. No truthiness test.
                    if index + 1 != source.len()
                        || !assigned(output.next()?, self.payload, self.result, value)
                        || !assigned(
                            output.next()?,
                            self.flag,
                            &scalar_type("i64"),
                            &NirExpr::Int(1),
                        )
                        || output.next()? != &NirStmt::Break
                    {
                        return None;
                    }
                    returns = true;
                }
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    let NirStmt::If {
                        condition: observed,
                        then_body: left,
                        else_body: right,
                    } = output.next()?
                    else {
                        return None;
                    };
                    if condition != observed {
                        return None;
                    }
                    // Visit both arms even when one already contains a return.
                    let left = self.block(then_body, left, in_loop)?;
                    let right = self.block(else_body, right, in_loop)?;
                    returns |= left || right;
                }
                NirStmt::While { condition, body } => {
                    let NirStmt::While {
                        condition: observed,
                        body: rewritten,
                    } = output.next()?
                    else {
                        return None;
                    };
                    if condition != observed {
                        return None;
                    }
                    let before = induction::parse(condition, body)?;
                    let after = induction::parse(observed, rewritten)?;
                    if before.leading != after.leading || before.step != after.step {
                        return None;
                    }
                    let child_returns = self.block(before.effects, after.effects, true)?;
                    if child_returns {
                        self.propagation(output.next()?, in_loop)?;
                        returns = true;
                    }
                }
                // Exact statement identity preserves ordinary writes, prints,
                // callees, ordered arguments, fallible work and local exits.
                _ if output.next()? == stmt => {}
                _ => return None,
            }
        }
        output.next().is_none().then_some(returns)
    }

    fn propagation(&self, stmt: &NirStmt, in_loop: bool) -> Option<()> {
        let NirStmt::If {
            condition:
                NirExpr::Binary {
                    op: NirBinaryOp::Eq,
                    lhs,
                    rhs,
                },
            then_body,
            else_body,
        } = stmt
        else {
            return None;
        };
        if lhs.as_ref() != &NirExpr::Var(self.flag.into())
            || rhs.as_ref() != &NirExpr::Int(1)
            || !else_body.is_empty()
            || then_body.len() != 1
        {
            return None;
        }
        match &then_body[0] {
            NirStmt::Break if in_loop => Some(()),
            NirStmt::Return(Some(NirExpr::Var(name))) if !in_loop && name == self.payload => {
                Some(())
            }
            _ => None,
        }
    }
}
