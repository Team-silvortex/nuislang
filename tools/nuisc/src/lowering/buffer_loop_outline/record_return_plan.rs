use super::*;

#[path = "record_return_origins.rs"]
mod origins;

pub(super) enum Step<'a> {
    Binding {
        stmt: &'a NirStmt,
        ty: NirTypeRef,
    },
    Print(&'a NirStmt),
    Join(scalar_joins::nested_prefix::Join<'a>),
    Branch {
        condition: &'a NirExpr,
        yes: Vec<Step<'a>>,
        no: Vec<Step<'a>>,
    },
    Return(&'a NirExpr),
}

pub(super) fn prove<'a>(
    body: &'a [NirStmt],
    scope: &Scope,
    result: &NirTypeRef,
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
) -> Option<Vec<Step<'a>>> {
    if layouts.scalar(&result.name)
        || !control_values::supported_type(result, layouts)
        || !syntax(body, layouts)
        || !scalar_joins::nested_prefix::continuation_neutrals_bounded(body, layouts)
    {
        return None;
    }
    let mut lazy_return = false;
    let (steps, exits) = block(
        body,
        scope,
        &BTreeSet::new(),
        result,
        catalog,
        layouts,
        &mut lazy_return,
    )?;
    (exits && lazy_return).then_some(steps)
}

fn syntax(body: &[NirStmt], layouts: &impl control_values::ValueLayouts) -> bool {
    let mut pending = vec![(body, 0usize)];
    let mut roots = Vec::new();
    let mut remaining = 128usize;
    while let Some((body, depth)) = pending.pop() {
        if body.len() > 32 {
            return false;
        }
        for stmt in body {
            let Some(next) = remaining.checked_sub(1) else {
                return false;
            };
            remaining = next;
            match stmt {
                NirStmt::Let { value, .. }
                | NirStmt::Const { value, .. }
                | NirStmt::Return(Some(value)) => roots.push((value, 0, true)),
                NirStmt::Print(value) => roots.push((value, 0, false)),
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } if depth < 8 => {
                    roots.push((condition, 0, true));
                    pending.extend([
                        (then_body.as_slice(), depth + 1),
                        (else_body.as_slice(), depth + 1),
                    ]);
                }
                _ => return false,
            }
        }
    }
    conditional_values::prefix::computed_record_expression_roots(roots, layouts)
}

fn block<'a>(
    body: &'a [NirStmt],
    scope: &Scope,
    certified_locals: &BTreeSet<String>,
    result: &NirTypeRef,
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
    lazy_return: &mut bool,
) -> Option<(Vec<Step<'a>>, bool)> {
    let mut inner = scope.clone();
    let mut certified_locals = certified_locals.clone();
    let mut steps = Vec::new();
    let mut exits = false;
    for (index, stmt) in body.iter().enumerate() {
        // Unreachable suffixes need a separate source-boundary contract.
        if exits {
            return None;
        }
        match stmt {
            NirStmt::Let { .. } | NirStmt::Const { .. } => {
                let (name, declared, value) = match stmt {
                    NirStmt::Let { name, ty, value } => (name, ty.as_ref(), value),
                    NirStmt::Const { name, ty, value } => (name, Some(ty), value),
                    _ => unreachable!(),
                };
                if inner.contains_key(name) {
                    return None;
                }
                let ty = control_values::value_type(value, &inner, catalog, layouts)?;
                if !control_values::supported_type(&ty, layouts)
                    || declared.is_some_and(|declared| declared != &ty)
                {
                    return None;
                }
                // Only fresh exact-type copies can carry an already certified origin.
                let certified_origin = origins::value(value, &certified_locals);
                if &ty == result && certified_origin {
                    certified_locals.insert(name.clone());
                }
                inner.insert(name.clone(), ty.clone());
                steps.push(Step::Binding { stmt, ty });
            }
            NirStmt::Print(value) => {
                let ty = control_values::value_type(value, &inner, catalog, layouts)?;
                if !layouts.scalar(&ty.name) {
                    return None;
                }
                steps.push(Step::Print(stmt));
            }
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                if let Some(join) = scalar_joins::nested_prefix::continuation_declaration(
                    stmt,
                    &inner,
                    &body[index + 1..],
                    catalog,
                    layouts,
                ) {
                    if origins::join(&join, result, &certified_locals) {
                        certified_locals.insert(join.yes.name.to_owned());
                    }
                    inner.insert(join.yes.name.to_owned(), join.yes.ty.clone());
                    steps.push(Step::Join(join));
                    continue;
                }
                if control_values::value_type(condition, &inner, catalog, layouts)?
                    != scalar_type("bool")
                {
                    return None;
                }
                let (yes, yes_exits) = block(
                    then_body,
                    &inner,
                    &certified_locals,
                    result,
                    catalog,
                    layouts,
                    lazy_return,
                )?;
                let (no, no_exits) = block(
                    else_body,
                    &inner,
                    &certified_locals,
                    result,
                    catalog,
                    layouts,
                    lazy_return,
                )?;
                exits = yes_exits && no_exits;
                steps.push(Step::Branch { condition, yes, no });
            }
            NirStmt::Return(Some(value)) => {
                if control_values::value_type(value, &inner, catalog, layouts).as_ref()
                    != Some(result)
                {
                    return None;
                }
                *lazy_return |= control_values::has_logical_record_fields(value);
                *lazy_return |=
                    matches!(value, NirExpr::Var(name) if certified_locals.contains(name));
                steps.push(Step::Return(value));
                exits = true;
            }
            _ => return None,
        }
    }
    Some((steps, exits))
}
