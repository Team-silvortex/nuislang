use super::*;

pub(super) fn buffer_loop_params(
    condition: &NirExpr,
    body: &[NirStmt],
    scope: &Scope,
    catalog: &ScalarHelpers,
    protected: &BTreeSet<String>,
) -> Option<BufferLoopPlan> {
    let (step, effects) = body.split_last()?;
    let prepared = prepare_counted_while(
        condition,
        std::slice::from_ref(step),
        &BTreeSet::new(),
        &BTreeMap::new(),
        &BTreeMap::new(),
    )?;
    // Strict comparisons with unit steps cannot wrap before reaching the bound.
    if !matches!(prepared.step, NirExpr::Int(1))
        || !matches!(
            (prepared.compare, prepared.step_kind),
            (PreparedLoopCompare::Lt, PreparedLoopStepKind::Add)
                | (PreparedLoopCompare::Gt, PreparedLoopStepKind::Sub)
        )
        || scope.get(&prepared.binding_name)? != &scalar_type("i64")
        || protected.contains(&prepared.binding_name)
    {
        return None;
    }
    let mut header_inputs = BTreeSet::new();
    if scalar_expr(
        &prepared.limit,
        scope,
        &mut header_inputs,
        false,
        &ScalarHelpers::new(),
    )? != scalar_type("i64")
        || header_inputs.contains(&prepared.binding_name)
    {
        return None;
    }
    let writable = scope
        .iter()
        .filter_map(|(name, ty)| {
            (ty == &scalar_type("i64")
                && name != &prepared.binding_name
                && !header_inputs.contains(name)
                && !protected.contains(name))
            .then_some(name.clone())
        })
        .collect();
    let mut protected = protected.clone();
    protected.insert(prepared.binding_name.clone());
    protected.extend(header_inputs.iter().cloned());
    let mutations = MutationScope {
        writable,
        protected,
    };
    let mut inputs = BTreeSet::new();
    let mut carries = Vec::new();
    let has_store = validate_effects(
        effects,
        &mut scope.clone(),
        &mut inputs,
        catalog,
        &mutations,
        &mut carries,
    )?;
    // Inner-loop locals can change, but only entry bindings escape this iteration.
    carries.retain(|name| scope.contains_key(name));
    Some(BufferLoopPlan {
        params: captured_params(inputs, scope),
        carries,
        induction: prepared.binding_name,
        header_inputs,
        mutations,
        has_store,
    })
}

pub(super) fn captured_params(inputs: BTreeSet<String>, scope: &Scope) -> Vec<NirParam> {
    inputs
        .into_iter()
        .filter_map(|name| {
            scope.get(&name).map(|ty| NirParam {
                name: name.clone(),
                ty: ty.clone(),
            })
        })
        .collect()
}

pub(super) fn validate_effects(
    effects: &[NirStmt],
    locals: &mut Scope,
    inputs: &mut BTreeSet<String>,
    catalog: &ScalarHelpers,
    mutations: &MutationScope,
    carries: &mut Vec<String>,
) -> Option<bool> {
    let mut has_store = false;
    for stmt in effects {
        match stmt {
            NirStmt::Let { name, ty, value } => {
                let inferred = scalar_expr(value, locals, inputs, true, catalog)?;
                if ty.as_ref().is_some_and(|ty| ty != &inferred) {
                    return None;
                }
                if locals.contains_key(name) {
                    if !mutations.writable.contains(name) || inferred != scalar_type("i64") {
                        return None;
                    }
                    // Replacing and conditional updates still need a seed on zero trips.
                    inputs.insert(name.clone());
                    if !carries.contains(name) {
                        carries.push(name.clone());
                    }
                }
                locals.insert(name.clone(), inferred);
            }
            NirStmt::Expr(NirExpr::StoreAt {
                buffer,
                index,
                value,
            }) => {
                buffer_input(buffer, locals, inputs)?;
                if scalar_expr(index, locals, inputs, true, catalog)? != scalar_type("i64")
                    || scalar_expr(value, locals, inputs, true, catalog)? != scalar_type("i64")
                {
                    return None;
                }
                has_store = true;
            }
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                if scalar_expr(condition, locals, inputs, true, catalog)? != scalar_type("bool") {
                    return None;
                }
                for arm in [then_body, else_body] {
                    has_store |= validate_effects(
                        arm,
                        &mut locals.clone(),
                        inputs,
                        catalog,
                        mutations,
                        carries,
                    )?;
                }
            }
            NirStmt::While { condition, body } => {
                let plan =
                    buffer_loop_params(condition, body, locals, catalog, &mutations.protected)?;
                inputs.extend(plan.params.into_iter().map(|param| param.name));
                inputs.extend(plan.header_inputs);
                inputs.insert(plan.induction.clone());
                for name in plan.carries.into_iter().chain([plan.induction]) {
                    if !carries.contains(&name) {
                        carries.push(name);
                    }
                }
                has_store |= plan.has_store;
            }
            _ => return None,
        }
    }
    Some(has_store)
}
