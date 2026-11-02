use super::*;

#[path = "print_epoch_local_scope.rs"]
mod scope_scan;

pub(super) struct Plan<'a> {
    epoch: &'a Epoch,
    body: &'a [NirStmt],
    scope: &'a Scope,
    layouts: &'a control_values::CarryLayouts,
    catalog: &'a ScalarHelpers,
    binding: &'a str,
    definition: usize,
    consumer: usize,
    origins: BTreeMap<Vec<String>, NirTypeRef>,
    // Diagnostic facts: source admission preserves both statements regardless.
    #[allow(dead_code)]
    initializer_may_fail: bool,
    #[allow(dead_code)]
    consumer_may_fail: bool,
    work: usize,
    captures: BTreeSet<Vec<String>>,
}

impl Plan<'_> {
    fn matches(
        &self,
        body: &[NirStmt],
        definition: usize,
        epoch: &Epoch,
        scope: &Scope,
        layouts: &control_values::CarryLayouts,
        catalog: &ScalarHelpers,
    ) -> bool {
        std::ptr::eq(self.body, body)
            && std::ptr::eq(self.epoch, epoch)
            && std::ptr::eq(self.scope, scope)
            && std::ptr::eq(self.layouts, layouts)
            && std::ptr::eq(self.catalog, catalog)
            && self.definition == definition
            && self.consumer == definition.saturating_add(1)
            && !self.origins.is_empty()
            && self.work <= 4096
            && self.captures.len() <= 31
    }
}

pub(super) fn admit_source(
    body: &[NirStmt],
    definition: usize,
    epoch: &Epoch,
    scope: &Scope,
    layouts: &control_values::CarryLayouts,
    catalog: &ScalarHelpers,
    ready: impl FnMut(&NirExpr, usize) -> Option<NirTypeRef>,
) -> Option<usize> {
    let NirStmt::Let { value, .. } = body.get(definition)? else { return None; };
    // Initializers are eager ordinary-shape roots, not a new source of logical
    // roots outside the function-wide print budget.
    if !conditional_values::prefix::expression(value) { return None; }
    let certificate = plan(body, definition, epoch, scope, layouts, catalog, ready)?;
    if !certificate.matches(body, definition, epoch, scope, layouts, catalog) {
        return None;
    }
    let NirStmt::Print(value) = &body[certificate.consumer] else {
        return None;
    };
    // Type-check the consumer without publishing a binding into ordinary Locals.
    let mut print_scope = scope.clone();
    print_scope.insert(certificate.binding.to_owned(), scalar_type("i64"));
    iteration_prints::admit(value, &print_scope, catalog, layouts)?;
    Some(certificate.consumer)
}

// This certificate is borrowed analysis data, not a ready value or lowering grant.
pub(super) fn plan<'a>(
    body: &'a [NirStmt],
    definition: usize,
    epoch: &'a Epoch,
    scope: &'a Scope,
    layouts: &'a control_values::CarryLayouts,
    catalog: &'a ScalarHelpers,
    mut ready: impl FnMut(&NirExpr, usize) -> Option<NirTypeRef>,
) -> Option<Plan<'a>> {
    if body.len() > 4096 {
        return None;
    }
    let consumer = definition.checked_add(1)?;
    let NirStmt::Let { name, ty, value } = body.get(definition)? else {
        return None;
    };
    if scope.contains_key(name) || ty.as_ref().is_some_and(|ty| ty != &scalar_type("i64")) {
        return None;
    }
    // Only literal markers may precede this first root-entry profile's
    // definition. They cannot overwrite an original entry field.
    if !body[..definition]
        .iter()
        .all(|stmt| matches!(stmt, NirStmt::Print(NirExpr::Int(_))))
    {
        return None;
    }
    let mut scan_work = 4096usize.checked_sub(definition.checked_mul(2)?)?;
    let NirStmt::Print(NirExpr::Call { callee, args }) = body.get(consumer)? else {
        return None;
    };
    scope_scan::excludes(&body[consumer.checked_add(1)?..], name, &mut scan_work)?;
    let (params, result) = scalar_helpers::pure_scalar_signature(callee, catalog)?;
    if result != &scalar_type("i64")
        || params.len() != 1
        || args.len() != 1
        || !matches!(params[0].name.as_str(), "f32" | "f64")
    {
        return None;
    }
    // One Let, one Print and its outer observer call share the expression budget.
    let mut work = 4096usize.checked_sub(3)?;
    let mut origins = BTreeMap::new();
    let initializer = prove(value, 0, true, &mut work, catalog, None, |value, depth| {
        if matches!(value, NirExpr::Var(input) if input == name) {
            return None;
        }
        let ty = leaf(
            value,
            depth,
            epoch,
            scope,
            layouts,
            &mut ready,
            &mut origins,
        )?;
        matches!(ty.name.as_str(), "i32" | "i64").then_some(ty)
    })?;
    if initializer.ty != scalar_type("i64") || !initializer.comparison_only || origins.is_empty() {
        return None;
    }
    let mut binding = Binding {
        name,
        proof: &initializer,
        uses: 0,
    };
    let consumed = prove(
        &args[0],
        1,
        false,
        &mut work,
        catalog,
        Some(&mut binding),
        |value, depth| {
            leaf(
                value,
                depth,
                epoch,
                scope,
                layouts,
                &mut ready,
                &mut origins,
            )
        },
    )?;
    if binding.uses != 1
        || consumed.ty != params[0]
        || consumed.comparison_only
        || consumed.captures.len() > 31
    {
        return None;
    }
    Some(Plan {
        epoch,
        body,
        scope,
        layouts,
        catalog,
        binding: name,
        definition,
        consumer,
        origins,
        initializer_may_fail: !initializer.total,
        consumer_may_fail: !consumed.total,
        work: 4096 - work,
        captures: consumed.captures,
    })
}

fn leaf(
    value: &NirExpr,
    depth: usize,
    epoch: &Epoch,
    scope: &Scope,
    layouts: &control_values::CarryLayouts,
    ready: &mut impl FnMut(&NirExpr, usize) -> Option<NirTypeRef>,
    origins: &mut BTreeMap<Vec<String>, NirTypeRef>,
) -> Option<NirTypeRef> {
    if !matches!(value, NirExpr::FieldAccess { .. }) {
        let expected = match value {
            NirExpr::Var(name) => scope.get(name)?.clone(),
            NirExpr::Int(_) => scalar_type("i64"),
            NirExpr::Bool(_) => scalar_type("bool"),
            NirExpr::F32(_) => scalar_type("f32"),
            NirExpr::F64(_) => scalar_type("f64"),
            _ => return None,
        };
        return ready(value, depth).filter(|actual| actual == &expected);
    }
    // An uncertified projection cannot regain authority through the ready callback.
    let ty = epoch.fields.project_type(value, scope, layouts)?;
    let mut base = value;
    let mut path = Vec::new();
    while let NirExpr::FieldAccess {
        base: parent,
        field,
    } = base
    {
        if path.len() >= 64 {
            return None;
        }
        path.push(field.clone());
        base = parent;
    }
    let NirExpr::Var(root) = base else {
        return None;
    };
    path.push(root.clone());
    path.reverse();
    origins.insert(path, ty.clone());
    Some(ty)
}

#[cfg(test)]
#[path = "print_epoch_local_plan_tests.rs"]
mod tests;
