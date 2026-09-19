use super::*;

// This source profile uses scoped functions, not captured loop-metadata RHSs.
// Keep definite initialization separate from the set of returned loop carries.
#[derive(Clone)]
struct Locals {
    scope: Scope,
    writable: BTreeSet<String>,
    available: BTreeSet<String>,
}

pub(super) fn validate(
    body: &[NirStmt],
    scope: &Scope,
    writable: &BTreeSet<String>,
    updates: &BTreeSet<String>,
    induction: &str,
) -> Option<()> {
    let mut locals = Locals {
        scope: scope.clone(),
        writable: writable.clone(),
        available: BTreeSet::from([induction.to_owned()]),
    };
    block(body, &mut locals, updates, 0)
}

fn block(
    body: &[NirStmt],
    locals: &mut Locals,
    updates: &BTreeSet<String>,
    depth: usize,
) -> Option<()> {
    for stmt in body {
        match stmt {
            NirStmt::Let { name, ty, value } => {
                let existing = locals.scope.get(name);
                let own = existing.map(|_| name.as_str());
                let inferred = expression(value, locals, updates, own, 0)?;
                if ty.as_ref().is_some_and(|ty| ty != &inferred) {
                    return None;
                }
                if let Some(existing) = existing {
                    // Parameters/constants never gain write authority. Local
                    // bools are snapshots, not a new cross-branch carry ABI.
                    if !locals.writable.contains(name)
                        || existing != &scalar_type("i64")
                        || inferred != *existing
                    {
                        return None;
                    }
                } else {
                    if inferred == scalar_type("i64") {
                        locals.writable.insert(name.clone());
                    }
                    locals.scope.insert(name.clone(), inferred);
                }
                locals.available.insert(name.clone());
            }
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } if depth < 32 => {
                if (then_body.is_empty() && else_body.is_empty())
                    || expression(condition, locals, updates, None, 0)? != scalar_type("bool")
                {
                    return None;
                }
                let mut then_locals = locals.clone();
                let mut else_locals = locals.clone();
                block(then_body, &mut then_locals, updates, depth + 1)?;
                block(else_body, &mut else_locals, updates, depth + 1)?;
                // Only bindings that exist before the branch survive its join.
                // Even matching names declared in both arms remain arm-local.
                locals.available.extend(
                    then_locals
                        .available
                        .into_iter()
                        .chain(else_locals.available)
                        .filter(|name| locals.scope.contains_key(name)),
                );
            }
            _ => return None,
        }
    }
    Some(())
}

fn expression(
    value: &NirExpr,
    locals: &Locals,
    updates: &BTreeSet<String>,
    own: Option<&str>,
    depth: usize,
) -> Option<NirTypeRef> {
    if depth > 64 {
        return None;
    }
    match value {
        NirExpr::Int(_) => Some(scalar_type("i64")),
        NirExpr::Bool(_) => Some(scalar_type("bool")),
        NirExpr::Var(name) => {
            if updates.contains(name) && !locals.available.contains(name) && own != Some(name) {
                return None;
            }
            locals
                .scope
                .get(name)
                .filter(|ty| **ty == scalar_type("i64") || **ty == scalar_type("bool"))
                .cloned()
        }
        NirExpr::Binary { op, lhs, rhs } => {
            let left = expression(lhs, locals, updates, own, depth + 1)?;
            let right = expression(rhs, locals, updates, own, depth + 1)?;
            if left != right {
                return None;
            }
            match op {
                NirBinaryOp::Add | NirBinaryOp::Sub | NirBinaryOp::Mul
                    if left == scalar_type("i64") =>
                {
                    Some(left)
                }
                NirBinaryOp::Eq
                | NirBinaryOp::Ne
                | NirBinaryOp::Lt
                | NirBinaryOp::Le
                | NirBinaryOp::Gt
                | NirBinaryOp::Ge
                    if left == scalar_type("i64") =>
                {
                    Some(scalar_type("bool"))
                }
                NirBinaryOp::And | NirBinaryOp::Or if left == scalar_type("bool") => Some(left),
                // Compound bools are outlined only at logical edges. Do not
                // hide eager logical evaluation under an equality operation.
                NirBinaryOp::Eq | NirBinaryOp::Ne
                    if left == scalar_type("bool") && bool_atom(lhs) && bool_atom(rhs) =>
                {
                    Some(left)
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn bool_atom(value: &NirExpr) -> bool {
    matches!(value, NirExpr::Bool(_) | NirExpr::Var(_))
}
