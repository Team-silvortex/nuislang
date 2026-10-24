use super::*;

// These values require no call, projection, allocation or fallible arithmetic.
// Unknown expressions stay behind their existing control boundary.
pub(super) fn atom_type(value: &NirExpr, scope: &Scope) -> Option<NirTypeRef> {
    let ty = match value {
        NirExpr::Var(name) => scope.get(name)?.clone(),
        NirExpr::Bool(_) => scalar_type("bool"),
        NirExpr::Int(_) => scalar_type("i64"),
        NirExpr::F32(_) => scalar_type("f32"),
        NirExpr::F64(_) => scalar_type("f64"),
        _ => return None,
    };
    ["bool", "i32", "i64", "f32", "f64"]
        .iter()
        .any(|name| ty == scalar_type(name))
        .then_some(ty)
}

pub(super) fn total_predicate(expr: &NirExpr, scope: &Scope) -> bool {
    let mut pending = vec![expr];
    while let Some(expr) = pending.pop() {
        match expr {
            NirExpr::Binary {
                op: NirBinaryOp::And | NirBinaryOp::Or,
                lhs,
                rhs,
            } => pending.extend([lhs.as_ref(), rhs.as_ref()]),
            NirExpr::Binary {
                op:
                    NirBinaryOp::Eq
                    | NirBinaryOp::Ne
                    | NirBinaryOp::Lt
                    | NirBinaryOp::Le
                    | NirBinaryOp::Gt
                    | NirBinaryOp::Ge,
                lhs,
                rhs,
            } => {
                let Some(ty) = atom_type(lhs, scope) else {
                    return false;
                };
                if atom_type(rhs, scope).as_ref() != Some(&ty) {
                    return false;
                }
            }
            _ if atom_type(expr, scope) == Some(scalar_type("bool")) => {}
            _ => return false,
        }
    }
    true
}

pub(super) fn updates(
    arm: &[NirStmt],
    scope: &Scope,
    predicate: &str,
    selected: bool,
) -> Option<Vec<NirStmt>> {
    let mut output = Vec::new();
    for stmt in arm {
        let NirStmt::Let { name, value, .. } = stmt else {
            return None;
        };
        let ty = scope.get(name)?;
        if atom_type(value, scope).as_ref() != Some(ty) {
            return None;
        }
        let update = NirStmt::Let {
            name: name.clone(),
            ty: Some(ty.clone()),
            value: value.clone(),
        };
        let keep = NirStmt::Let {
            name: name.clone(),
            ty: Some(ty.clone()),
            value: NirExpr::Var(name.clone()),
        };
        // Keep source order: a later write observes the preceding selected write.
        // Both arms are ready values; the existing scalar select needs no helper.
        let (then_body, else_body) = if selected {
            (vec![update], vec![keep])
        } else {
            (vec![keep], vec![update])
        };
        output.push(NirStmt::If {
            condition: NirExpr::Var(predicate.to_owned()),
            then_body,
            else_body,
        });
    }
    Some(output)
}

#[cfg(test)]
#[path = "ready_values_tests.rs"]
mod tests;
