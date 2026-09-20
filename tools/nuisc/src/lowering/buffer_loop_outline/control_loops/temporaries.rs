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
    catalog: &ScalarHelpers,
    layouts: &control_values::FlatLayouts,
) -> Option<()> {
    let mut locals = Locals {
        scope: scope.clone(),
        writable: writable
            .iter()
            .filter(|name| {
                scope.get(*name).is_some_and(|ty| {
                    ty != &scalar_type("bool") && control_values::supported_type(ty, layouts)
                })
            })
            .cloned()
            .collect(),
        available: BTreeSet::from([induction.to_owned()]),
    };
    block(body, &mut locals, updates, catalog, layouts, 0)
}

fn block(
    body: &[NirStmt],
    locals: &mut Locals,
    updates: &BTreeSet<String>,
    catalog: &ScalarHelpers,
    layouts: &control_values::FlatLayouts,
    depth: usize,
) -> Option<()> {
    for stmt in body {
        match stmt {
            NirStmt::Let { name, ty, value } => {
                let existing = locals.scope.get(name);
                let own = existing.map(|_| name.as_str());
                let inferred = expression(value, locals, updates, own, catalog, layouts, 0)?;
                if ty.as_ref().is_some_and(|ty| ty != &inferred) {
                    return None;
                }
                if let Some(existing) = existing {
                    // Seeded mutable flat values may cross the backedge. Only
                    // iteration-local declarations gain bool write authority.
                    if !locals.writable.contains(name)
                        || !control_values::supported_type(existing, layouts)
                        || inferred != *existing
                    {
                        return None;
                    }
                } else {
                    if control_values::supported_type(&inferred, layouts) {
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
                    || expression(condition, locals, updates, None, catalog, layouts, 0)?
                        != scalar_type("bool")
                {
                    return None;
                }
                let mut then_locals = locals.clone();
                let mut else_locals = locals.clone();
                block(
                    then_body,
                    &mut then_locals,
                    updates,
                    catalog,
                    layouts,
                    depth + 1,
                )?;
                block(
                    else_body,
                    &mut else_locals,
                    updates,
                    catalog,
                    layouts,
                    depth + 1,
                )?;
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
    catalog: &ScalarHelpers,
    layouts: &control_values::FlatLayouts,
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
                .filter(|ty| control_values::supported_type(ty, layouts))
                .cloned()
        }
        NirExpr::Call { callee, args } => {
            let types = args
                .iter()
                .map(|arg| expression(arg, locals, updates, own, catalog, layouts, depth + 1))
                .collect::<Option<Vec<_>>>()?;
            scalar_helpers::typed_call_type(callee, &types, catalog)
        }
        NirExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } => {
            let layout = layouts.get(type_name)?;
            if !type_args.is_empty() || fields.len() != layout.len() {
                return None;
            }
            let mut seen = BTreeSet::new();
            for (name, value) in fields {
                if !layout.contains(name)
                    || !seen.insert(name)
                    || expression(value, locals, updates, own, catalog, layouts, depth + 1)?
                        != scalar_type("i64")
                {
                    return None;
                }
            }
            Some(scalar_type(type_name))
        }
        NirExpr::FieldAccess { base, field } => {
            let base = expression(base, locals, updates, own, catalog, layouts, depth + 1)?;
            layouts
                .get(&base.name)?
                .contains(field)
                .then(|| scalar_type("i64"))
        }
        NirExpr::Binary { op, lhs, rhs } => {
            let left = expression(lhs, locals, updates, own, catalog, layouts, depth + 1)?;
            let right = expression(rhs, locals, updates, own, catalog, layouts, depth + 1)?;
            if left != right {
                return None;
            }
            match op {
                NirBinaryOp::Add
                | NirBinaryOp::Sub
                | NirBinaryOp::Mul
                | NirBinaryOp::Div
                | NirBinaryOp::Rem
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
                // Keep comparisons on boolean values; call arguments have
                // their logical edges normalized inside the iteration.
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
    matches!(
        value,
        NirExpr::Bool(_) | NirExpr::Var(_) | NirExpr::Call { .. }
    )
}
