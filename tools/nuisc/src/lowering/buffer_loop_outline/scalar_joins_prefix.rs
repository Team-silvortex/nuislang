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
    plan(stmt, scope, suffix, catalog, layouts, false)
}

// Pure discovery never opts into statement effects, even with an effect catalog.
pub(super) fn declaration_with_prints<'a>(
    stmt: &'a NirStmt,
    scope: &Scope,
    suffix: &[NirStmt],
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
) -> Option<Declaration<'a>> {
    plan(stmt, scope, suffix, catalog, layouts, true)
}

fn plan<'a>(
    stmt: &'a NirStmt,
    scope: &Scope,
    suffix: &[NirStmt],
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
    prints: bool,
) -> Option<Declaration<'a>> {
    let NirStmt::If {
        condition,
        then_body,
        else_body,
    } = stmt
    else {
        return None;
    };
    let record_result = matches!(then_body.last(), Some(NirStmt::Let { ty: Some(ty), .. }) if !layouts.scalar(&ty.name));
    if then_body.len() == 1 && else_body.len() == 1 && (!prints || !record_result) {
        return None;
    }
    let bounded = if prints {
        let roots = super::nested_prefix::initializer_roots(condition, then_body, else_body)?;
        conditional_values::prefix::computed_initializer_expression_roots(roots, layouts)
    } else {
        let (roots, _) =
            super::nested_prefix::prefix_roots(condition, then_body, else_body, false, true)?;
        conditional_values::prefix::computed_expression_roots(roots)
    };
    if !bounded {
        return None;
    }
    if prints && !super::nested_prefix::guarded_records_bounded(then_body, else_body, layouts) {
        return None;
    }
    pair(
        stmt,
        scope,
        suffix,
        catalog,
        layouts,
        prints,
        &BTreeSet::new(),
    )
}

fn pair<'a>(
    stmt: &'a NirStmt,
    scope: &Scope,
    suffix: &[NirStmt],
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
    prints: bool,
    blocked: &BTreeSet<String>,
) -> Option<Declaration<'a>> {
    let NirStmt::If {
        condition,
        then_body,
        else_body,
    } = stmt
    else {
        return None;
    };
    if then_body.is_empty() || else_body.is_empty() || then_body.len() > 32 || else_body.len() > 32
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
        || blocked.contains(name)
        || (!prints && !layouts.scalar(&ty.name))
        || !control_values::supported_type(ty, layouts)
        || !super::observed(name, suffix)?
    {
        return None;
    }
    let mut blocked = blocked.clone();
    blocked.insert(name.clone());
    if control_values::value_type(condition, scope, catalog, layouts)? != scalar_type("bool")
        || !arm(then_body, name, scope, catalog, layouts, prints, &blocked)
        || !arm(else_body, name, scope, catalog, layouts, prints, &blocked)
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
    prints: bool,
    blocked: &BTreeSet<String>,
) -> bool {
    let mut inner = scope.clone();
    for (index, stmt) in body.iter().enumerate() {
        if let NirStmt::Print(value) = stmt {
            if !prints || iteration_prints::admit(value, &inner, catalog, layouts).is_none() {
                return false;
            }
            continue;
        }
        if prints && matches!(stmt, NirStmt::If { .. }) {
            let Some(join) = pair(
                stmt,
                &inner,
                &body[index + 1..],
                catalog,
                layouts,
                prints,
                blocked,
            ) else {
                return false;
            };
            inner.insert(join.name.to_owned(), join.ty.clone());
            continue;
        }
        let (name, declared, value) = match stmt {
            NirStmt::Let { name, ty, value } => (name, ty.as_ref(), value),
            NirStmt::Const { name, ty, value } => (name, Some(ty), value),
            _ => return false,
        };
        if inner.contains_key(name)
            || (index + 1 < body.len() && name == destination)
            || (blocked.contains(name) && name != destination)
        {
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
