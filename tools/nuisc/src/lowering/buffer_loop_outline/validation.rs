use super::*;

// Share scoped outlining without granting Buffer effects to pure-value loops.
#[derive(Clone, Copy)]
pub(super) enum EffectTypes<'a> {
    Buffer(&'a ScalarHelpers),
    Values(&'a ScalarHelpers, &'a control_values::FlatLayouts),
}

impl<'a> EffectTypes<'a> {
    pub(super) fn layouts(self) -> Option<&'a control_values::FlatLayouts> {
        match self {
            Self::Buffer(_) => None,
            Self::Values(_, layouts) => Some(layouts),
        }
    }

    pub(super) fn expression(
        self,
        value: &NirExpr,
        scope: &Scope,
        inputs: &mut BTreeSet<String>,
    ) -> Option<NirTypeRef> {
        match self {
            Self::Buffer(catalog) => scalar_expr(value, scope, inputs, true, catalog),
            Self::Values(catalog, layouts) => {
                let ty = control_values::value_type(value, scope, catalog, layouts)?;
                control_values::collect_inputs(value, inputs);
                Some(ty)
            }
        }
    }
}

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
    let mut writable: BTreeSet<String> = scope
        .iter()
        .filter_map(|(name, ty)| {
            (ty == &scalar_type("i64")
                && name != &prepared.binding_name
                && !header_inputs.contains(name)
                && !protected.contains(name))
            .then_some(name.clone())
        })
        .collect();
    let normalized = control_flow::normalize(effects, scope, step)?;
    if let Some(normalized) = &normalized {
        writable.insert(normalized.running.clone());
        writable.extend(normalized.breaking.iter().cloned());
    }
    let break_flag = normalized
        .as_ref()
        .and_then(|normalized| normalized.breaking.clone());
    let normalized_effects = normalized.map(|normalized| normalized.effects);
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
        normalized_effects.as_deref().unwrap_or(effects),
        &mut scope.clone(),
        &mut inputs,
        EffectTypes::Buffer(catalog),
        &mutations,
        &mut carries,
    )?;
    // Inner-loop locals can change, but only entry bindings escape this iteration.
    carries.retain(|name| scope.contains_key(name));
    let mut params = captured_params(inputs, scope);
    if let Some(flag) = &break_flag {
        carries.push(flag.clone());
        params.push(NirParam {
            name: flag.clone(),
            ty: scalar_type("i64"),
        });
    }
    Some(BufferLoopPlan {
        params,
        carries,
        induction: prepared.binding_name,
        header_inputs,
        mutations,
        has_store,
        normalized_effects,
        break_flag,
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
    types: EffectTypes<'_>,
    mutations: &MutationScope,
    carries: &mut Vec<String>,
) -> Option<bool> {
    let mut has_store = false;
    for stmt in effects {
        match stmt {
            NirStmt::Let { name, ty, value } => {
                let inferred = types.expression(value, locals, inputs)?;
                if ty.as_ref().is_some_and(|ty| ty != &inferred) {
                    return None;
                }
                if let Some(existing) = locals.get(name) {
                    let supported = match types {
                        EffectTypes::Buffer(_) => inferred == scalar_type("i64"),
                        EffectTypes::Values(_, layouts) => {
                            control_values::supported_type(&inferred, layouts)
                        }
                    };
                    if !mutations.writable.contains(name) || !supported || existing != &inferred {
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
                let EffectTypes::Buffer(catalog) = types else {
                    return None;
                };
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
                if types.expression(condition, locals, inputs)? != scalar_type("bool") {
                    return None;
                }
                for arm in [then_body, else_body] {
                    has_store |= validate_effects(
                        arm,
                        &mut locals.clone(),
                        inputs,
                        types,
                        mutations,
                        carries,
                    )?;
                }
            }
            NirStmt::While { condition, body } => {
                if let EffectTypes::Values(..) = types {
                    if types.expression(condition, locals, inputs)? != scalar_type("bool") {
                        return None;
                    }
                    let mut child_carries = Vec::new();
                    let iteration = control_loops::induction::parse(condition, body)?;
                    let normalized = iteration.normalize(locals)?;
                    let mut child_mutations = mutations.clone();
                    if let Some(flow) = &normalized {
                        child_mutations.writable.insert(flow.running.clone());
                        child_mutations
                            .writable
                            .extend(flow.breaking.iter().cloned());
                    }
                    let mut child_body = normalized
                        .as_ref()
                        .map_or(iteration.effects, |flow| &flow.effects)
                        .to_vec();
                    if iteration.leading {
                        child_body.insert(0, iteration.step.clone());
                    } else {
                        child_body.push(iteration.step.clone());
                    }
                    validate_effects(
                        &child_body,
                        &mut locals.clone(),
                        inputs,
                        types,
                        &child_mutations,
                        &mut child_carries,
                    )?;
                    for name in child_carries {
                        if locals.contains_key(&name) && !carries.contains(&name) {
                            carries.push(name);
                        }
                    }
                    continue;
                }
                let EffectTypes::Buffer(catalog) = types else {
                    return None;
                };
                let plan =
                    buffer_loop_params(condition, body, locals, catalog, &mutations.protected)?;
                inputs.extend(
                    plan.params
                        .into_iter()
                        .filter(|param| plan.break_flag.as_ref() != Some(&param.name))
                        .map(|param| param.name),
                );
                inputs.extend(plan.header_inputs);
                inputs.insert(plan.induction.clone());
                for name in plan
                    .carries
                    .into_iter()
                    .filter(|name| plan.break_flag.as_ref() != Some(name))
                    .chain([plan.induction])
                {
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
