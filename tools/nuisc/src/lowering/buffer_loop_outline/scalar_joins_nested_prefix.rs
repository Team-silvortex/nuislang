use super::*;
use std::borrow::Cow;

#[path = "scalar_joins_discarded_records.rs"]
mod discarded_records;
#[path = "scalar_joins_logical_fields.rs"]
mod logical_fields;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::lowering::buffer_loop_outline) enum JoinKind {
    Scalars,
    Records,
    DiscardedRecords,
    OuterRecords,
    LogicalFields,
    Continuation,
}

impl JoinKind {
    fn discarded_records(self) -> bool {
        matches!(
            self,
            Self::DiscardedRecords | Self::OuterRecords | Self::LogicalFields | Self::Continuation
        )
    }
}

pub(in crate::lowering::buffer_loop_outline) struct Join<'a> {
    pub condition: &'a NirExpr,
    pub yes: Arm<'a>,
    pub no: Arm<'a>,
}

pub(in crate::lowering::buffer_loop_outline) struct Arm<'a> {
    pub name: &'a str,
    pub ty: &'a NirTypeRef,
    pub value: Cow<'a, NirExpr>,
    pub locals: Vec<Local<'a>>,
    pub inputs: BTreeSet<String>,
}

pub(in crate::lowering::buffer_loop_outline) enum Local<'a> {
    Binding { stmt: &'a NirStmt, ty: NirTypeRef },
    Join { join: Box<Join<'a>>, export: bool },
    Condition(&'a NirExpr),
}

// The caller first bounds the complete function, including all synthetic neutrals.
pub(in crate::lowering::buffer_loop_outline) fn continuation_declaration<'a>(
    stmt: &'a NirStmt,
    scope: &Scope,
    suffix: &[NirStmt],
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
) -> Option<Join<'a>> {
    declaration_plan(
        stmt,
        scope,
        suffix,
        catalog,
        layouts,
        JoinKind::Continuation,
    )
}

pub(in crate::lowering::buffer_loop_outline) fn continuation_neutrals_bounded(
    body: &[NirStmt],
    layouts: &impl control_values::ValueLayouts,
) -> bool {
    discarded_records::function_bounded(body, layouts)
}

// Bound the complete original tree before building scopes, captures or helpers.
pub(in crate::lowering::buffer_loop_outline) fn declaration<'a>(
    stmt: &'a NirStmt,
    scope: &Scope,
    suffix: &[NirStmt],
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
) -> Option<Join<'a>> {
    declaration_plan(stmt, scope, suffix, catalog, layouts, JoinKind::Scalars)
}

pub(in crate::lowering::buffer_loop_outline) fn record_declaration<'a>(
    stmt: &'a NirStmt,
    scope: &Scope,
    suffix: &[NirStmt],
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
) -> Option<Join<'a>> {
    let plan = declaration_plan(stmt, scope, suffix, catalog, layouts, JoinKind::Records)?;
    has_record_join(&plan, layouts).then_some(plan)
}

pub(in crate::lowering::buffer_loop_outline) fn discarded_record_declaration<'a>(
    stmt: &'a NirStmt,
    scope: &Scope,
    suffix: &[NirStmt],
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
) -> Option<Join<'a>> {
    let plan = declaration_plan(
        stmt,
        scope,
        suffix,
        catalog,
        layouts,
        JoinKind::DiscardedRecords,
    )?;
    has_discarded_record(&plan, layouts).then_some(plan)
}

pub(in crate::lowering::buffer_loop_outline) fn outer_record_declaration<'a>(
    stmt: &'a NirStmt,
    scope: &Scope,
    suffix: &[NirStmt],
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
) -> Option<Join<'a>> {
    declaration_plan(
        stmt,
        scope,
        suffix,
        catalog,
        layouts,
        JoinKind::OuterRecords,
    )
}

pub(in crate::lowering::buffer_loop_outline) fn logical_field_declaration<'a>(
    stmt: &'a NirStmt,
    scope: &Scope,
    suffix: &[NirStmt],
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
) -> Option<Join<'a>> {
    let plan = declaration_plan(
        stmt,
        scope,
        suffix,
        catalog,
        layouts,
        JoinKind::LogicalFields,
    )?;
    logical_fields::present(&plan).then_some(plan)
}

fn has_discarded_record(join: &Join<'_>, layouts: &impl control_values::ValueLayouts) -> bool {
    join.yes
        .locals
        .iter()
        .chain(&join.no.locals)
        .any(|local| match local {
            Local::Join { join, export } => {
                (!export && !layouts.scalar(&join.yes.ty.name))
                    || has_discarded_record(join, layouts)
            }
            _ => false,
        })
}

fn has_record_join(join: &Join<'_>, layouts: &impl control_values::ValueLayouts) -> bool {
    join.yes
        .locals
        .iter()
        .chain(&join.no.locals)
        .any(|local| match local {
            Local::Join { join, .. } => {
                !layouts.scalar(&join.yes.ty.name) || has_record_join(join, layouts)
            }
            _ => false,
        })
}

fn declaration_plan<'a>(
    stmt: &'a NirStmt,
    scope: &Scope,
    suffix: &[NirStmt],
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
    kind: JoinKind,
) -> Option<Join<'a>> {
    let NirStmt::If {
        condition,
        then_body,
        else_body,
    } = stmt
    else {
        return None;
    };
    if !syntax(condition, then_body, else_body, kind, layouts) {
        return None;
    }
    if kind.discarded_records() && !discarded_records::bounded(then_body, else_body, layouts) {
        return None;
    }
    let (name, ty, _) = result(then_body)?;
    let result_supported = match kind {
        JoinKind::Continuation => true,
        JoinKind::LogicalFields => !layouts.scalar(&ty.name),
        JoinKind::OuterRecords => !layouts.scalar(&ty.name),
        _ => layouts.scalar(&ty.name),
    };
    if !result_supported || !super::observed(name, suffix)? {
        return None;
    }
    pair(
        condition,
        then_body,
        else_body,
        scope,
        &BTreeSet::new(),
        catalog,
        layouts,
        kind,
    )
}

fn syntax(
    condition: &NirExpr,
    yes: &[NirStmt],
    no: &[NirStmt],
    kind: JoinKind,
    layouts: &impl control_values::ValueLayouts,
) -> bool {
    let require_nested = !matches!(
        kind,
        JoinKind::OuterRecords | JoinKind::LogicalFields | JoinKind::Continuation
    );
    let mut roots = vec![(condition, 0, true)];
    let mut pending = vec![(yes, 0usize), (no, 0usize)];
    let mut remaining = 128usize;
    let mut nested = false;
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
                NirStmt::Let { value, .. } | NirStmt::Const { value, .. } => {
                    roots.push((value, 0, true))
                }
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } if depth < 8 => {
                    nested = true;
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
    if matches!(kind, JoinKind::LogicalFields | JoinKind::Continuation) {
        return conditional_values::prefix::computed_record_expression_roots(roots, layouts);
    }
    (!require_nested || nested) && conditional_values::prefix::computed_expression_roots(roots)
}

fn local_pair<'a>(
    condition: &'a NirExpr,
    yes: &'a [NirStmt],
    no: &'a [NirStmt],
    scope: &Scope,
    blocked: &BTreeSet<String>,
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
    kind: JoinKind,
) -> Option<(Join<'a>, bool)> {
    if let Some(join) = pair(condition, yes, no, scope, blocked, catalog, layouts, kind) {
        return Some((join, true));
    }
    let (body, selected) = match (yes.is_empty(), no.is_empty()) {
        (false, true) => (yes, true),
        (true, false) => (no, false),
        _ => return None,
    };
    let (name, ty, _) = result(body)?;
    if scope.contains_key(name)
        || blocked.contains(name)
        || (!layouts.scalar(&ty.name) && !kind.discarded_records())
        || !control_values::supported_type(ty, layouts)
        || control_values::value_type(condition, scope, catalog, layouts)? != scalar_type("bool")
    {
        return None;
    }
    let mut blocked = blocked.clone();
    blocked.insert(name.to_owned());
    let retained = arm(body, scope, &blocked, catalog, layouts, kind)?;
    let skipped = Arm {
        name,
        ty,
        value: Cow::Owned(control_values::zero_value(ty, layouts)),
        locals: Vec::new(),
        inputs: BTreeSet::new(),
    };
    let (yes, no) = if selected {
        (retained, skipped)
    } else {
        (skipped, retained)
    };
    // Dead-binding pruning can remove a total arm. The remaining work does not
    // grant a source result identity to following statements.
    Some((Join { condition, yes, no }, false))
}

fn result(body: &[NirStmt]) -> Option<(&str, &NirTypeRef, &NirExpr)> {
    let NirStmt::Let {
        name,
        ty: Some(ty),
        value,
    } = body.last()?
    else {
        return None;
    };
    Some((name, ty, value))
}

fn pair<'a>(
    condition: &'a NirExpr,
    yes: &'a [NirStmt],
    no: &'a [NirStmt],
    scope: &Scope,
    blocked: &BTreeSet<String>,
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
    kind: JoinKind,
) -> Option<Join<'a>> {
    let (name, ty, _) = result(yes)?;
    let (other, other_ty, _) = result(no)?;
    if name != other
        || ty != other_ty
        || scope.contains_key(name)
        || blocked.contains(name)
        || (kind == JoinKind::Scalars && !layouts.scalar(&ty.name))
        || !control_values::supported_type(ty, layouts)
        || control_values::value_type(condition, scope, catalog, layouts)? != scalar_type("bool")
    {
        return None;
    }
    let mut blocked = blocked.clone();
    blocked.insert(name.to_owned());
    Some(Join {
        condition,
        yes: arm(yes, scope, &blocked, catalog, layouts, kind)?,
        no: arm(no, scope, &blocked, catalog, layouts, kind)?,
    })
}

fn arm<'a>(
    body: &'a [NirStmt],
    scope: &Scope,
    blocked: &BTreeSet<String>,
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
    kind: JoinKind,
) -> Option<Arm<'a>> {
    let (name, ty, value) = result(body)?;
    let mut inner = scope.clone();
    let mut locals = Vec::new();
    let mut local_names = BTreeSet::new();
    let mut inputs = BTreeSet::new();
    for stmt in &body[..body.len() - 1] {
        match stmt {
            NirStmt::Let { .. } | NirStmt::Const { .. } => {
                let (name, declared, value) = match stmt {
                    NirStmt::Let { name, ty, value } => (name, ty.as_ref(), value),
                    NirStmt::Const { name, ty, value } => (name, Some(ty), value),
                    _ => unreachable!(),
                };
                if inner.contains_key(name) || blocked.contains(name) {
                    return None;
                }
                let ty = control_values::value_type(value, &inner, catalog, layouts)?;
                if !control_values::supported_type(&ty, layouts)
                    || declared.is_some_and(|d| d != &ty)
                {
                    return None;
                }
                control_values::collect_inputs(value, &mut inputs);
                inner.insert(name.clone(), ty.clone());
                local_names.insert(name.clone());
                locals.push(Local::Binding { stmt, ty });
            }
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                if then_body.is_empty() && else_body.is_empty() {
                    if control_values::value_type(condition, &inner, catalog, layouts)?
                        != scalar_type("bool")
                    {
                        return None;
                    }
                    control_values::collect_inputs(condition, &mut inputs);
                    locals.push(Local::Condition(condition));
                    continue;
                }
                let (join, export) = local_pair(
                    condition, then_body, else_body, &inner, blocked, catalog, layouts, kind,
                )?;
                control_values::collect_inputs(condition, &mut inputs);
                inputs.extend(join.yes.inputs.iter().chain(&join.no.inputs).cloned());
                if export {
                    inner.insert(join.yes.name.to_owned(), join.yes.ty.clone());
                    local_names.insert(join.yes.name.to_owned());
                }
                locals.push(Local::Join {
                    join: Box::new(join),
                    export,
                });
            }
            _ => return None,
        }
    }
    if inner.contains_key(name)
        || control_values::value_type(value, &inner, catalog, layouts).as_ref() != Some(ty)
    {
        return None;
    }
    control_values::collect_inputs(value, &mut inputs);
    inputs.retain(|name| !local_names.contains(name));
    Some(Arm {
        name,
        ty,
        value: Cow::Borrowed(value),
        locals,
        inputs,
    })
}
