use super::*;

pub(super) struct Declaration<'a> {
    pub name: &'a str,
    pub ty: &'a NirTypeRef,
    pub condition: &'a NirExpr,
    pub yes: &'a NirExpr,
    pub no: &'a NirExpr,
}

// A new typed value join is distinct from writing an existing scalar. Both
// arms contain only ready values, and an outer use must precede redeclaration.
pub(super) fn declaration<'a>(
    stmt: &'a NirStmt,
    scope: &Scope,
    suffix: &[NirStmt],
) -> Option<Declaration<'a>> {
    let join = shape(stmt, scope, suffix)?;
    let Declaration {
        ty,
        condition,
        yes,
        no,
        ..
    } = &join;
    if ready_values::atom_type(condition, scope) != Some(scalar_type("bool"))
        || ready_values::atom_type(yes, scope).as_ref() != Some(*ty)
        || ready_values::atom_type(no, scope).as_ref() != Some(*ty)
    {
        return None;
    }
    Some(join)
}

// Pure does not mean total: branch expressions stay inside the value helper.
// Bound syntax before recursive type inference and require the completed catalog.
pub(super) fn guarded_declaration<'a>(
    stmt: &'a NirStmt,
    scope: &Scope,
    suffix: &[NirStmt],
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
) -> Option<Declaration<'a>> {
    let join = shape(stmt, scope, suffix)?;
    if ready_values::atom_type(join.condition, scope) != Some(scalar_type("bool"))
        || !conditional_values::prefix::expression_roots(vec![
            (join.yes, 0, false),
            (join.no, 0, false),
        ])
        || !scalar_arms(&join, scope, catalog, layouts)
    {
        return None;
    }
    Some(join)
}

// The complete condition is evaluated once as argument zero of the value
// helper. It shares the syntax budget with both still-guarded value arms.
pub(super) fn computed_declaration<'a>(
    stmt: &'a NirStmt,
    scope: &Scope,
    suffix: &[NirStmt],
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
) -> Option<Declaration<'a>> {
    let join = shape(stmt, scope, suffix)?;
    if ready_values::atom_type(join.condition, scope).is_some()
        || !conditional_values::prefix::expression_roots(vec![
            (join.condition, 0, false),
            (join.yes, 0, false),
            (join.no, 0, false),
        ])
        || control_values::value_type(join.condition, scope, catalog, layouts)?
            != scalar_type("bool")
        || !scalar_arms(&join, scope, catalog, layouts)
    {
        return None;
    }
    Some(join)
}

fn scalar_arms(
    join: &Declaration<'_>,
    scope: &Scope,
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
) -> bool {
    layouts.scalar(&join.ty.name)
        && control_values::supported_type(join.ty, layouts)
        && control_values::value_type(join.yes, scope, catalog, layouts).as_ref() == Some(join.ty)
        && control_values::value_type(join.no, scope, catalog, layouts).as_ref() == Some(join.ty)
}

fn shape<'a>(stmt: &'a NirStmt, scope: &Scope, suffix: &[NirStmt]) -> Option<Declaration<'a>> {
    let NirStmt::If {
        condition,
        then_body,
        else_body,
    } = stmt
    else {
        return None;
    };
    let [NirStmt::Let {
        name,
        ty: Some(ty),
        value: yes,
    }] = then_body.as_slice()
    else {
        return None;
    };
    let [NirStmt::Let {
        name: other,
        ty: Some(other_ty),
        value: no,
    }] = else_body.as_slice()
    else {
        return None;
    };
    if name != other || ty != other_ty || scope.contains_key(name) || !observed(name, suffix)? {
        return None;
    }
    Some(Declaration {
        name,
        ty,
        condition,
        yes,
        no,
    })
}

fn observed(name: &str, body: &[NirStmt]) -> Option<bool> {
    let mut work = 4096usize;
    let mut blocks = vec![(body, 0usize)];
    while let Some((body, depth)) = blocks.pop() {
        if depth >= 64 {
            return None;
        }
        for stmt in body {
            work = work.checked_sub(1)?;
            let expr = match stmt {
                NirStmt::Let { value, .. }
                | NirStmt::Const { value, .. }
                | NirStmt::Print(value)
                | NirStmt::Expr(value)
                | NirStmt::Return(Some(value)) => Some(value),
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    blocks.extend([
                        (then_body.as_slice(), depth + 1),
                        (else_body.as_slice(), depth + 1),
                    ]);
                    Some(condition)
                }
                NirStmt::While { condition, body } => {
                    blocks.push((body.as_slice(), depth + 1));
                    Some(condition)
                }
                NirStmt::Return(None) | NirStmt::Break | NirStmt::Continue => None,
                _ => return None,
            };
            if let Some(expr) = expr {
                if reads(name, expr, depth, &mut work)? {
                    return Some(true);
                }
            }
            match stmt {
                // An initializer sees the preceding scope; its destination
                // shadows that identity only for subsequent statements.
                NirStmt::Let { name: bound, .. } | NirStmt::Const { name: bound, .. }
                    if bound == name =>
                {
                    break
                }
                NirStmt::Return(_) | NirStmt::Break | NirStmt::Continue => break,
                _ => {}
            }
        }
    }
    Some(false)
}

fn reads(name: &str, root: &NirExpr, depth: usize, work: &mut usize) -> Option<bool> {
    let mut pending = vec![(root, depth)];
    while let Some((expr, depth)) = pending.pop() {
        if depth >= 64 {
            return None;
        }
        *work = work.checked_sub(1)?;
        match expr {
            NirExpr::Var(value) if value == name => return Some(true),
            NirExpr::Var(_)
            | NirExpr::Int(_)
            | NirExpr::Bool(_)
            | NirExpr::F32(_)
            | NirExpr::F64(_) => {}
            NirExpr::Binary { lhs, rhs, .. } => {
                if pending.len().checked_add(2)? > *work {
                    return None;
                }
                pending.extend([(lhs.as_ref(), depth + 1), (rhs.as_ref(), depth + 1)])
            }
            NirExpr::Call { args, .. } => {
                if pending.len().checked_add(args.len())? > *work {
                    return None;
                }
                pending.extend(args.iter().map(|arg| (arg, depth + 1)))
            }
            NirExpr::StructLiteral { fields, .. } => {
                if pending.len().checked_add(fields.len())? > *work {
                    return None;
                }
                pending.extend(fields.iter().map(|(_, value)| (value, depth + 1)))
            }
            NirExpr::FieldAccess { base, .. }
            | NirExpr::CastI64ToI32(base)
            | NirExpr::CastI32ToI64(base)
            | NirExpr::CastBoolToI64(base)
            | NirExpr::CastI64ToBool(base)
            | NirExpr::PackF32Word(base)
            | NirExpr::UnpackF32Word(base)
            | NirExpr::PackF64Word(base)
            | NirExpr::UnpackF64Word(base) => pending.push((base, depth + 1)),
            _ => return None,
        }
    }
    Some(false)
}

#[cfg(test)]
#[path = "scalar_joins_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "scalar_joins_guarded_tests.rs"]
mod guarded_tests;

#[cfg(test)]
#[path = "scalar_joins_computed_tests.rs"]
mod computed_tests;
