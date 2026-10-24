use super::*;

pub(in crate::lowering::buffer_loop_outline) struct Declaration<'a> {
    pub name: &'a str,
    pub ty: &'a NirTypeRef,
    pub condition: &'a NirExpr,
    pub yes: &'a [NirStmt],
    pub no: &'a [NirStmt],
}

// The destination is the only exported name. Every initializer and result in
// both arms shares the predicate's source budget before inference or cloning.
pub(in crate::lowering::buffer_loop_outline) fn declaration<'a>(
    stmt: &'a NirStmt,
    scope: &Scope,
    suffix: &[NirStmt],
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
) -> Option<Declaration<'a>> {
    let NirStmt::If {
        condition,
        then_body,
        else_body,
    } = stmt
    else {
        return None;
    };
    if then_body.is_empty()
        || else_body.is_empty()
        || then_body.len() > 32
        || else_body.len() > 32
        || (then_body.len() == 1 && else_body.len() == 1)
    {
        return None;
    }
    let NirStmt::Let {
        name, ty: Some(ty), ..
    } = then_body.last()?
    else {
        return None;
    };
    let NirStmt::Let {
        name: other,
        ty: Some(other_ty),
        ..
    } = else_body.last()?
    else {
        return None;
    };
    if name != other
        || ty != other_ty
        || scope.contains_key(name)
        || !layouts.scalar(&ty.name)
        || !control_values::supported_type(ty, layouts)
        || !super::observed(name, suffix)?
    {
        return None;
    }
    let mut roots = vec![(condition, 0, true)];
    for stmt in then_body.iter().chain(else_body) {
        let value = match stmt {
            NirStmt::Let { value, .. } | NirStmt::Const { value, .. } => value,
            _ => return None,
        };
        roots.push((value, 0, true));
    }
    if !conditional_values::prefix::computed_expression_roots(roots)
        || control_values::value_type(condition, scope, catalog, layouts)? != scalar_type("bool")
        || !arm(then_body, name, scope, catalog, layouts)
        || !arm(else_body, name, scope, catalog, layouts)
    {
        return None;
    }
    Some(Declaration {
        name,
        ty,
        condition,
        yes: then_body,
        no: else_body,
    })
}

fn arm(
    body: &[NirStmt],
    destination: &str,
    scope: &Scope,
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
) -> bool {
    let mut inner = scope.clone();
    for (index, stmt) in body.iter().enumerate() {
        let (name, declared, value) = match stmt {
            NirStmt::Let { name, ty, value } => (name, ty.as_ref(), value),
            NirStmt::Const { name, ty, value } => (name, Some(ty), value),
            _ => return false,
        };
        if inner.contains_key(name) || (index + 1 < body.len() && name == destination) {
            return false;
        }
        let Some(ty) = control_values::value_type(value, &inner, catalog, layouts) else {
            return false;
        };
        if !control_values::supported_type(&ty, layouts)
            || declared.is_some_and(|declared| declared != &ty)
        {
            return false;
        }
        inner.insert(name.clone(), ty);
    }
    true
}

#[cfg(test)]
#[path = "scalar_joins_prefix_tests.rs"]
mod tests;
