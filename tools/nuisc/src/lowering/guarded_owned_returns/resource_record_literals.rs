use super::*;

#[cfg(test)]
#[path = "resource_record_field_values_tests.rs"]
mod field_tests;
#[path = "resource_record_liveness.rs"]
mod liveness;
#[cfg(test)]
#[path = "resource_record_local_fields_tests.rs"]
mod local_tests;
#[path = "resource_record_operands.rs"]
mod operands;
#[cfg(test)]
#[path = "resource_record_literals_tests.rs"]
mod tests;

#[derive(Default)]
pub(in crate::lowering) struct LocalRecords(BTreeMap<String, NirTypeRef>);

impl LocalRecords {
    pub(in crate::lowering) fn advance(&mut self, statement: &NirStmt, calls: &PureValueCalls) {
        match statement {
            NirStmt::Let { name, ty, .. } => self.bind(name, ty.as_ref(), calls),
            NirStmt::Const { name, ty, .. } => self.bind(name, Some(ty), calls),
            // This table proves local availability, never a record capture or phi.
            NirStmt::If { .. } | NirStmt::While { .. } => self.0.clear(),
            _ => {}
        }
    }

    fn bind(&mut self, name: &str, ty: Option<&NirTypeRef>, calls: &PureValueCalls) {
        self.0.remove(name);
        if let Some(ty) = ty.filter(|ty| calls.is_record(ty) && snapshot_layout(ty, calls)) {
            self.0.insert(name.to_owned(), ty.clone());
        }
    }

    fn value_type(&self, value: &NirExpr, calls: &PureValueCalls) -> Option<NirTypeRef> {
        match value {
            NirExpr::Var(name) => self.0.get(name).cloned(),
            NirExpr::FieldAccess { base, field } => {
                calls.field_type(&self.value_type(base, calls)?, field)
            }
            _ => None,
        }
    }

    pub(in crate::lowering) fn split(
        &self,
        statement: &NirStmt,
        template: &NirFunction,
        scope: &[NirParam],
        live: &BTreeSet<String>,
        context: &mut OutlineContext,
    ) -> Option<Vec<NirStmt>> {
        if self.0.is_empty() {
            return None;
        }
        if let NirStmt::If { condition, .. } = statement {
            let ty = NirTypeRef {
                name: "bool".into(),
                is_ref: false,
                is_optional: false,
                generic_args: vec![],
            };
            return operands::split_scalar(
                statement, &ty, condition, template, scope, live, context, self,
            );
        }
        let (ty, value) = match statement {
            NirStmt::Let {
                ty: Some(ty),
                value,
                ..
            }
            | NirStmt::Const { ty, value, .. } => (ty, value),
            _ => return None,
        };
        if capture_scalar(ty) {
            return operands::split_scalar(
                statement, ty, value, template, scope, live, context, self,
            );
        }
        split_with_locals(statement, ty, value, template, scope, live, context, self)
    }
}

pub(super) fn split(
    statement: &NirStmt,
    ty: &NirTypeRef,
    value: &NirExpr,
    template: &NirFunction,
    scope: &[NirParam],
    live: &BTreeSet<String>,
    context: &mut OutlineContext,
) -> Option<Vec<NirStmt>> {
    split_with_locals(
        statement,
        ty,
        value,
        template,
        scope,
        live,
        context,
        &LocalRecords::default(),
    )
}

fn split_with_locals(
    statement: &NirStmt,
    ty: &NirTypeRef,
    value: &NirExpr,
    template: &NirFunction,
    scope: &[NirParam],
    live: &BTreeSet<String>,
    context: &mut OutlineContext,
    locals: &LocalRecords,
) -> Option<Vec<NirStmt>> {
    if !matches!(value, NirExpr::StructLiteral { .. }) {
        return None;
    }
    // Share the original whole-tree bound, never restart the budget per field.
    bounded_tree(value, true)?;
    if !snapshot_layout(ty, &context.calls)
        || !prove(
            &operands::for_proof(value, locals, &context.calls),
            ty,
            scope,
            live,
            &context.calls,
            locals,
        )?
    {
        return None;
    }
    let (value, mut prefix, preserved) =
        liveness::prepare(value, ty, template, scope, live, context, locals);
    let value = stage(
        &value,
        ty,
        template,
        scope,
        live,
        context,
        &mut prefix,
        &preserved,
    );
    let mut declaration = statement.clone();
    let (NirStmt::Let { value: target, .. } | NirStmt::Const { value: target, .. }) =
        &mut declaration
    else {
        unreachable!()
    };
    *target = value;
    prefix.push(declaration);
    Some(prefix)
}

fn snapshot_layout(ty: &NirTypeRef, calls: &PureValueCalls) -> bool {
    // A short helper call can return a large type DAG. Bound the entire expanded
    // constructor before allocating names, independently of native ABI admission.
    let mut work = vec![(ty, 0)];
    let mut visited = 0;
    while let Some((ty, depth)) = work.pop() {
        visited += 1;
        if visited > 4096 || depth >= 64 {
            return false;
        }
        if capture_scalar(ty) {
            continue;
        }
        let Some(fields) = calls.record_fields(ty) else {
            return false;
        };
        if fields.len() > 4096usize.saturating_sub(visited + work.len()) {
            return false;
        }
        work.extend(fields.iter().map(|(_, ty)| (ty, depth + 1)));
    }
    true
}

fn prove(
    value: &NirExpr,
    ty: &NirTypeRef,
    scope: &[NirParam],
    live: &BTreeSet<String>,
    calls: &PureValueCalls,
    locals: &LocalRecords,
) -> Option<bool> {
    if capture_scalar(ty) {
        let kind = if ty.name == "bool" {
            Kind::Bool
        } else {
            Kind::I64
        };
        (predicate_kind(value, scope, live, calls) == Some(kind)).then_some(())?;
        return Some(arguments::contains_logic(value) && admitted(value, scope, live, calls, kind));
    }
    let layout = calls.record_fields(ty)?;
    let NirExpr::StructLiteral {
        type_name,
        type_args,
        fields,
    } = value
    else {
        if locals.value_type(value, calls).as_ref() == Some(ty) {
            return Some(false);
        }
        (helper_value_type(value, scope, live, calls).as_ref() == Some(ty)).then_some(())?;
        return Some(arguments::contains_argument_logic(value));
    };
    if type_name != &ty.name || !type_args.is_empty() || fields.len() != layout.len() {
        return None;
    }
    let mut seen = BTreeSet::new();
    let mut needed = false;
    for (field, expression) in fields {
        if !seen.insert(field) {
            return None;
        }
        let (_, field_type) = layout.iter().find(|(name, _)| name == field)?;
        needed |= prove(expression, field_type, scope, live, calls, locals)?;
    }
    Some(needed)
}

fn stage(
    value: &NirExpr,
    ty: &NirTypeRef,
    template: &NirFunction,
    scope: &[NirParam],
    live: &BTreeSet<String>,
    context: &mut OutlineContext,
    prefix: &mut Vec<NirStmt>,
    preserved: &BTreeSet<String>,
) -> NirExpr {
    if matches!(value, NirExpr::Var(name) if preserved.contains(name)) {
        return value.clone();
    }
    if let NirExpr::StructLiteral {
        type_name,
        type_args,
        fields,
    } = value
    {
        return NirExpr::StructLiteral {
            type_name: type_name.clone(),
            type_args: type_args.clone(),
            fields: fields
                .iter()
                .map(|(field, value)| {
                    let field_type = context
                        .calls
                        .field_type(ty, field)
                        .expect("complete constructor proof precedes field staging");
                    (
                        field.clone(),
                        stage(
                            value,
                            &field_type,
                            template,
                            scope,
                            live,
                            context,
                            prefix,
                            preserved,
                        ),
                    )
                })
                .collect(),
        };
    }
    let result = snapshot(value, ty, template, scope, live, context, prefix);
    if capture_scalar(ty) {
        return result;
    }
    // Execute a record-valued helper once, then freeze all leaves before any
    // later field can branch. Whole records never become resource captures.
    project(&result, ty, template, scope, live, context, prefix)
}

fn project(
    value: &NirExpr,
    ty: &NirTypeRef,
    template: &NirFunction,
    scope: &[NirParam],
    live: &BTreeSet<String>,
    context: &mut OutlineContext,
    prefix: &mut Vec<NirStmt>,
) -> NirExpr {
    if capture_scalar(ty) {
        return snapshot(value, ty, template, scope, live, context, prefix);
    }
    let fields = context
        .calls
        .record_fields(ty)
        .expect("proved snapshot layout")
        .to_vec();
    NirExpr::StructLiteral {
        type_name: ty.name.clone(),
        type_args: vec![],
        fields: fields
            .into_iter()
            .map(|(field, ty)| {
                let projected = NirExpr::FieldAccess {
                    base: Box::new(value.clone()),
                    field: field.clone(),
                };
                (
                    field,
                    project(&projected, &ty, template, scope, live, context, prefix),
                )
            })
            .collect(),
    }
}

fn snapshot(
    value: &NirExpr,
    ty: &NirTypeRef,
    template: &NirFunction,
    scope: &[NirParam],
    live: &BTreeSet<String>,
    context: &mut OutlineContext,
    prefix: &mut Vec<NirStmt>,
) -> NirExpr {
    let name = fresh_left(template, scope, live, context);
    // Freeze every leaf in source order, including eager reads and traps.
    // Keep lazy subtrees intact for the existing owned-region normalizer.
    prefix.push(NirStmt::Let {
        name: name.clone(),
        ty: Some(ty.clone()),
        value: value.clone(),
    });
    NirExpr::Var(name)
}
