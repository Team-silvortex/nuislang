use super::*;
use std::rc::Rc;

#[cfg(test)]
#[path = "capture_call_result_views_tests.rs"]
mod call_result_tests;
#[cfg(test)]
#[path = "capture_evaluated_record_views_tests.rs"]
mod evaluated_tests;
#[cfg(test)]
#[path = "capture_inline_record_args_tests.rs"]
mod inline_record_args_tests;
#[cfg(test)]
#[path = "capture_materialized_record_args_tests.rs"]
mod materialized_record_args_tests;
#[cfg(test)]
#[path = "capture_stored_projection_tests.rs"]
mod stored_projection_tests;
#[cfg(test)]
#[path = "capture_record_views_tests.rs"]
mod tests;

struct View {
    value: NirExpr,
    cost: usize,
}

type Views = BTreeMap<String, Rc<View>>;

#[derive(Clone)]
struct ScopeViews {
    views: Views,
    ready: Scope,
}

struct Evaluated<'a> {
    catalog: &'a ScalarHelpers,
    controls: &'a BTreeSet<String>,
    transport_types: &'a BTreeSet<String>,
    call_results: bool,
}

// Resolve fields of total input reconstructions, not arbitrary computed records.
// Whole escapes retain their binding; the separate dead-record pass removes a
// snapshot only after all of its observations have become ready input paths.
pub(super) fn normalize(function: &mut NirFunction, layouts: &impl ValueLayouts) {
    if let Some(body) = normalized(function, layouts, 65_536) {
        function.body = body;
    }
}

// A separate proof may read immutable scalars after their RHS has executed.
// Their definitions remain in place, even when an unused record copy disappears.
pub(super) fn normalize_evaluated(
    function: &mut NirFunction,
    layouts: &impl ValueLayouts,
    catalog: &ScalarHelpers,
    controls: &BTreeSet<String>,
    transport_types: &BTreeSet<String>,
) {
    let evaluated = Evaluated {
        catalog,
        controls,
        transport_types,
        call_results: false,
    };
    if let Some(body) = normalized_mode(function, layouts, Some(evaluated), 65_536) {
        function.body = body;
    }
}

// A completed pure-value call owns its full result. Only later total reads and
// copies may be projected; the call binding never becomes an erasable view.
pub(super) fn normalize_call_results(
    function: &mut NirFunction,
    layouts: &impl ValueLayouts,
    catalog: &ScalarHelpers,
    controls: &BTreeSet<String>,
    transport_types: &BTreeSet<String>,
) {
    let evaluated = Evaluated {
        catalog,
        controls,
        transport_types,
        call_results: true,
    };
    if let Some(body) = normalized_mode(function, layouts, Some(evaluated), 65_536) {
        function.body = body;
    }
}

fn normalized(
    function: &NirFunction,
    layouts: &impl ValueLayouts,
    remaining: usize,
) -> Option<Vec<NirStmt>> {
    normalized_mode(function, layouts, None, remaining)
}

fn normalized_mode(
    function: &NirFunction,
    layouts: &impl ValueLayouts,
    evaluated: Option<Evaluated<'_>>,
    mut remaining: usize,
) -> Option<Vec<NirStmt>> {
    let mut writes = BTreeMap::<&str, usize>::new();
    let mut blocks = vec![(function.body.as_slice(), 0)];
    let mut expressions = Vec::new();
    while let Some((body, depth)) = blocks.pop() {
        if depth >= 64 {
            return None;
        }
        for stmt in body {
            charge(&mut remaining, 1)?;
            match stmt {
                NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. } => {
                    *writes.entry(name).or_default() += 1;
                    expressions.push((value, 0));
                }
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    expressions.push((condition, 0));
                    blocks.extend([
                        (then_body.as_slice(), depth + 1),
                        (else_body.as_slice(), depth + 1),
                    ]);
                }
                NirStmt::While { condition, body } => {
                    expressions.push((condition, 0));
                    blocks.push((body, depth + 1));
                }
                NirStmt::Print(value)
                | NirStmt::Expr(value)
                | NirStmt::Await(value)
                | NirStmt::Return(Some(value)) => expressions.push((value, 0)),
                NirStmt::Break | NirStmt::Continue | NirStmt::Return(None) => {}
            }
        }
    }
    // Preflight before cloning, including expression depth and every statement.
    while let Some((expr, depth)) = expressions.pop() {
        if depth >= 64 || !walk::supported_expr(expr) {
            return None;
        }
        charge(&mut remaining, 1)?;
        crate::nir_walk::walk_child_exprs(expr, &mut |child| expressions.push((child, depth + 1)));
    }
    if evaluated.is_some() {
        charge(&mut remaining, function.params.len())?;
    }
    let parameters = function
        .params
        .iter()
        .map(|p| p.name.as_str())
        .collect::<BTreeSet<_>>();
    if parameters.len() != function.params.len() {
        return None;
    }
    let inputs = function
        .params
        .iter()
        .filter(|p| {
            !writes.contains_key(p.name.as_str()) && control_values::supported_type(&p.ty, layouts)
        })
        .map(|p| (p.name.as_str(), &p.ty))
        .collect::<BTreeMap<_, _>>();
    let ready = if evaluated.is_some() {
        inputs
            .iter()
            .map(|(name, ty)| ((*name).to_owned(), (*ty).clone()))
            .collect()
    } else {
        Scope::new()
    };
    let mut context = Context {
        writes,
        parameters,
        inputs,
        layouts,
        remaining,
        evaluated,
        erasable: BTreeSet::new(),
    };
    let mut result = function.body.clone();
    let mut pending = vec![(
        &mut result,
        ScopeViews {
            views: Views::new(),
            ready,
        },
    )];
    while let Some((body, mut scope)) = pending.pop() {
        for stmt in body {
            charge(&mut context.remaining, 1)?;
            match stmt {
                NirStmt::Let { name, ty, value } => {
                    context.binding(name, ty.as_ref(), value, &mut scope)?;
                }
                NirStmt::Const { name, ty, value } => {
                    context.binding(name, Some(ty), value, &mut scope)?;
                }
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    context.rewrite(condition, &scope)?;
                    charge(
                        &mut context.remaining,
                        (scope.views.len() + scope.ready.len()) * 2,
                    )?;
                    pending.extend([(then_body, scope.clone()), (else_body, scope.clone())]);
                }
                NirStmt::While { condition, body } => {
                    context.rewrite(condition, &scope)?;
                    charge(
                        &mut context.remaining,
                        scope.views.len() + scope.ready.len(),
                    )?;
                    pending.push((body, scope.clone()));
                }
                NirStmt::Print(value)
                | NirStmt::Expr(value)
                | NirStmt::Await(value)
                | NirStmt::Return(Some(value)) => context.rewrite(value, &scope)?,
                NirStmt::Break | NirStmt::Continue | NirStmt::Return(None) => {}
            }
        }
    }
    if context.evaluated.is_some() {
        let mut reads = BTreeSet::new();
        walk::visit(&result, |expr| {
            if charge(&mut context.remaining, 1).is_none() {
                return false;
            }
            if let NirExpr::Var(name) = expr {
                reads.insert(name.clone());
            }
            true
        });
        charge(&mut context.remaining, 0)?;
        let mut pending = vec![&mut result];
        while let Some(body) = pending.pop() {
            body.retain(|stmt| match stmt {
                NirStmt::Let { name, .. } | NirStmt::Const { name, .. } => {
                    !context.erasable.contains(name) || reads.contains(name)
                }
                _ => true,
            });
            for stmt in body {
                charge(&mut context.remaining, 1)?;
                match stmt {
                    NirStmt::If {
                        then_body,
                        else_body,
                        ..
                    } => pending.extend([then_body, else_body]),
                    NirStmt::While { body, .. } => pending.push(body),
                    _ => {}
                }
            }
        }
        scalar_aliases::validate_expansion(&result, &mut context.remaining)?;
    }
    (context.remaining > 0).then_some(result)
}

struct Context<'a, L> {
    writes: BTreeMap<&'a str, usize>,
    parameters: BTreeSet<&'a str>,
    inputs: BTreeMap<&'a str, &'a NirTypeRef>,
    layouts: &'a L,
    remaining: usize,
    evaluated: Option<Evaluated<'a>>,
    erasable: BTreeSet<String>,
}

impl<L: ValueLayouts> Context<'_, L> {
    fn binding(
        &mut self,
        name: &str,
        declared: Option<&NirTypeRef>,
        value: &mut NirExpr,
        scope: &mut ScopeViews,
    ) -> Option<()> {
        self.rewrite(value, scope)?;
        if self.writes.get(name) != Some(&1)
            || self.parameters.contains(name)
            || self
                .evaluated
                .as_ref()
                .is_some_and(|e| e.controls.contains(name))
        {
            scope.views.remove(name);
            scope.ready.remove(name);
            return Some(());
        }
        charge(&mut self.remaining, scope.ready.len())?;
        let ready = scope
            .ready
            .iter()
            .map(|(name, ty)| (name.as_str(), ty))
            .collect();
        if let Some(evaluated) = self.evaluated.as_ref().filter(|e| {
            e.call_results
                && matches!(value, NirExpr::FieldAccess { .. })
                && access(value).is_none()
        }) {
            if let Some(ty) = completed_projection_type(
                value,
                &ready,
                evaluated.catalog,
                evaluated.transport_types,
                self.layouts,
                &mut self.remaining,
                0,
            )
            .filter(|ty| {
                control_values::supported_type(ty, self.layouts)
                    && declared.is_none_or(|d| d == ty)
                    && !evaluated.transport_types.contains(&ty.name)
            }) {
                // Only this stored binding becomes ready. The full call and
                // every unselected result field remain evaluated exactly once.
                charge(&mut self.remaining, 1)?;
                scope.ready.insert(name.to_owned(), ty);
                scope.views.remove(name);
                return Some(());
            }
        }
        // A direct record constructor cannot become a scalar/call-result root.
        // Its total-view proof below is separate from call-argument typing.
        if let Some(evaluated) = self
            .evaluated
            .as_ref()
            .filter(|_| !matches!(value, NirExpr::StructLiteral { .. }))
        {
            // Type inference grants only a read of this binding, never authority
            // to inline, repeat, move or delete the expression that produced it.
            if let Some(ty) = evaluated_scalar_type(
                value,
                &ready,
                evaluated.catalog,
                self.layouts,
                &mut self.remaining,
                0,
            )
            .filter(|ty| {
                control_values::supported_type(ty, self.layouts)
                    && declared.is_none_or(|d| d == ty)
                    && (self.layouts.scalar(&ty.name)
                        || (evaluated.call_results
                            && matches!(value, NirExpr::Call { .. })
                            && !evaluated.transport_types.contains(&ty.name)))
            }) {
                charge(&mut self.remaining, 1)?;
                scope.ready.insert(name.to_owned(), ty);
                scope.views.remove(name);
                // Do not add a call result to erasable, even when never read.
                return Some(());
            }
        }
        // Record aliases inherit only total ready values or proven constructors,
        // not opaque locals or changing loop-carried versions.
        let alias = if let NirExpr::Var(source) = value {
            scope.views.get(source).cloned()
        } else {
            None
        };
        if let Some(view) = &alias {
            charge(&mut self.remaining, view.cost)?;
        }
        let candidate = alias.as_ref().map_or(&*value, |view| &view.value);
        let before = self.remaining;
        let inputs = if self.evaluated.is_some() {
            &ready
        } else {
            &self.inputs
        };
        let ty = dead_records::ready_type(candidate, inputs, self.layouts, &mut self.remaining, 0);
        if let Some(ty) = ty.filter(|ty| {
            !self.layouts.scalar(&ty.name)
                && declared.is_none_or(|d| d == ty)
                && !self
                    .evaluated
                    .as_ref()
                    .is_some_and(|e| e.transport_types.contains(&ty.name))
        }) {
            let cost = before - self.remaining;
            charge(&mut self.remaining, cost)?;
            let view = Rc::new(View {
                value: candidate.clone(),
                cost,
            });
            if alias.is_some() {
                *value = view.value.clone();
            }
            scope.views.insert(name.to_owned(), view);
            self.erasable.insert(name.to_owned());
            if self.evaluated.as_ref().is_some_and(|e| e.call_results) {
                // Whole uses retain the materialized binding through the read
                // scan; its exact kind can type a later unchanged call operand.
                charge(&mut self.remaining, 1)?;
                scope.ready.insert(name.to_owned(), ty);
            }
        } else {
            scope.views.remove(name);
            if let Some(evaluated) = self
                .evaluated
                .as_ref()
                .filter(|e| e.call_results && matches!(value, NirExpr::StructLiteral { .. }))
            {
                // A checked constructor has already run at this binding. Grant
                // reads of its stored value, never constructor elision/replay.
                if let Some(ty) = evaluated_scalar_type(
                    value,
                    &ready,
                    evaluated.catalog,
                    self.layouts,
                    &mut self.remaining,
                    0,
                )
                .filter(|ty| {
                    !self.layouts.scalar(&ty.name)
                        && control_values::supported_type(ty, self.layouts)
                        && declared.is_none_or(|d| d == ty)
                        && !evaluated.transport_types.contains(&ty.name)
                }) {
                    charge(&mut self.remaining, 1)?;
                    scope.ready.insert(name.to_owned(), ty);
                    // Non-total constructors are never added to erasable.
                }
            }
        }
        (self.remaining > 0).then_some(())
    }

    fn rewrite(&mut self, value: &mut NirExpr, scope: &ScopeViews) -> Option<()> {
        charge(&mut self.remaining, scope.ready.len())?;
        let ready = scope
            .ready
            .iter()
            .map(|(name, ty)| (name.as_str(), ty))
            .collect();
        let inputs = if self.evaluated.is_some() {
            &ready
        } else {
            &self.inputs
        };
        walk::rewrite_expr(value, |expr| {
            if charge(&mut self.remaining, 1).is_none() {
                return;
            }
            let Some(path) = access(expr) else {
                return;
            };
            if charge(&mut self.remaining, path.len()).is_none() || path.len() < 2 {
                return;
            }
            let Some(view) = scope.views.get(&path[0]) else {
                return;
            };
            if charge(&mut self.remaining, view.cost).is_none() {
                return;
            }
            let Some(projected) = select(&view.value, &path[1..]) else {
                return;
            };
            if dead_records::ready_type(&projected, inputs, self.layouts, &mut self.remaining, 0)
                .is_some()
            {
                *expr = projected;
            }
        });
        (self.remaining > 0).then_some(())
    }
}

// A caller may use this exact kind proof only if it preserves the complete RHS
// in an evaluated binding. This grants no inline readiness or erasure authority.
pub(super) fn materialized_operand_type(
    value: &NirExpr,
    ready: &BTreeMap<&str, &NirTypeRef>,
    catalog: &ScalarHelpers,
    layouts: &impl ValueLayouts,
    remaining: &mut usize,
) -> Option<NirTypeRef> {
    evaluated_scalar_type(value, ready, catalog, layouts, remaining, 0)
}

fn evaluated_scalar_type(
    value: &NirExpr,
    ready: &BTreeMap<&str, &NirTypeRef>,
    catalog: &ScalarHelpers,
    layouts: &impl ValueLayouts,
    remaining: &mut usize,
    depth: usize,
) -> Option<NirTypeRef> {
    if depth >= 64 {
        *remaining = 0;
        return None;
    }
    charge(remaining, 1)?;
    if access(value).is_some() {
        return dead_records::ready_type(value, ready, layouts, remaining, depth);
    }
    if let NirExpr::StructLiteral {
        type_name,
        type_args,
        fields,
    } = value
    {
        if !type_args.is_empty() {
            return None;
        }
        let declared = layouts.fields(type_name)?;
        charge(remaining, declared.len())?;
        if fields.len() != declared.len() {
            return None;
        }
        let mut expected = declared
            .map(|(name, ty)| (name.to_owned(), ty))
            .collect::<BTreeMap<_, _>>();
        // This proves an argument kind, not a total/erasable constructor. All
        // fields, including checked or unused ones, stay evaluated in the call.
        for (name, field) in fields {
            if evaluated_scalar_type(field, ready, catalog, layouts, remaining, depth + 1)?
                != expected.remove(name)?
            {
                return None;
            }
        }
        return expected.is_empty().then(|| scalar_type(type_name));
    }
    let mut operand = |value: &NirExpr| {
        evaluated_scalar_type(value, ready, catalog, layouts, remaining, depth + 1)
    };
    match value {
        NirExpr::Int(_) => Some(scalar_type("i64")),
        NirExpr::Bool(_) => Some(scalar_type("bool")),
        NirExpr::F32(_) => Some(scalar_type("f32")),
        NirExpr::F64(_) => Some(scalar_type("f64")),
        NirExpr::Binary { op, lhs, rhs } => {
            control_values::binary_type(*op, operand(lhs)?, operand(rhs)?)
        }
        NirExpr::Call { callee, args } => {
            let types = args.iter().map(&mut operand).collect::<Option<Vec<_>>>()?;
            scalar_helpers::typed_call_type(callee, &types, catalog)
        }
        NirExpr::CastI64ToI32(value) => {
            (operand(value)? == scalar_type("i64")).then(|| scalar_type("i32"))
        }
        NirExpr::CastI32ToI64(value) => {
            (operand(value)? == scalar_type("i32")).then(|| scalar_type("i64"))
        }
        NirExpr::PackF32Word(value) => {
            (operand(value)? == scalar_type("f32")).then(|| scalar_type("i64"))
        }
        NirExpr::UnpackF32Word(value) => {
            (operand(value)? == scalar_type("i64")).then(|| scalar_type("f32"))
        }
        NirExpr::PackF64Word(value) => {
            (operand(value)? == scalar_type("f64")).then(|| scalar_type("i64"))
        }
        NirExpr::UnpackF64Word(value) => {
            (operand(value)? == scalar_type("i64")).then(|| scalar_type("f64"))
        }
        _ => None,
    }
}

// Do not extend inline expression typing: only a completed local initializer
// may gain this read authority, and its call-rooted RHS is never erasable.
fn completed_projection_type(
    value: &NirExpr,
    ready: &BTreeMap<&str, &NirTypeRef>,
    catalog: &ScalarHelpers,
    transport_types: &BTreeSet<String>,
    layouts: &impl ValueLayouts,
    remaining: &mut usize,
    depth: usize,
) -> Option<NirTypeRef> {
    if depth >= 64 {
        *remaining = 0;
        return None;
    }
    charge(remaining, 1)?;
    let ty = match value {
        NirExpr::Call { .. } => {
            evaluated_scalar_type(value, ready, catalog, layouts, remaining, depth)
        }
        NirExpr::FieldAccess { base, field } => {
            let ty = completed_projection_type(
                base,
                ready,
                catalog,
                transport_types,
                layouts,
                remaining,
                depth + 1,
            )?;
            let mut fields = layouts.fields(&ty.name)?;
            charge(remaining, fields.len())?;
            let selected = fields.find(|(name, _)| name == field).map(|(_, ty)| ty);
            selected
        }
        _ => None,
    };
    // Selecting a scalar or subrecord cannot strip a protected envelope's
    // transport identity; every nominal kind along the path must be admissible.
    ty.filter(|ty| !transport_types.contains(&ty.name))
}

fn select(value: &NirExpr, path: &[String]) -> Option<NirExpr> {
    let mut selected = value;
    for (index, field) in path.iter().enumerate() {
        if let NirExpr::StructLiteral { fields, .. } = selected {
            selected = &fields.iter().find(|(name, _)| name == field)?.1;
        } else {
            return Some(path[index..].iter().fold(selected.clone(), |base, field| {
                NirExpr::FieldAccess {
                    base: Box::new(base),
                    field: field.clone(),
                }
            }));
        }
    }
    Some(selected.clone())
}

fn charge(remaining: &mut usize, cost: usize) -> Option<()> {
    if *remaining <= cost {
        *remaining = 0;
        return None;
    }
    *remaining -= cost;
    Some(())
}
