use super::*;

#[path = "nested_conditions.rs"]
mod conditions;

pub(super) fn present(body: &[NirStmt], scope: &Scope) -> bool {
    // Fallible arithmetic must execute in the selected iteration, never as a
    // captured metadata operand. Reuse the shared non-speculation analysis.
    if speculation::block_has_checked_arithmetic(body, &BTreeSet::new())
        || control_flow::contains_exit(body, false)
        || contains_loop(body)
        || scalar_helpers::contains_calls(body)
        || control_values::has_aggregate_expressions(body)
        || sequences::carry_names(body)
            .iter()
            .any(|name| scope.get(name).is_some_and(|ty| ty != &scalar_type("i64")))
    {
        return true;
    }
    let mut seen = BTreeSet::new();
    body.iter().any(|stmt| match stmt {
        NirStmt::Let { name, .. } => !scope.contains_key(name) || !seen.insert(name.as_str()),
        NirStmt::If {
            then_body,
            else_body,
            ..
        } => {
            let arms = then_body.iter().chain(else_body);
            let mut names = BTreeSet::new();
            for stmt in arms {
                let NirStmt::Let { name, .. } = stmt else {
                    return true;
                };
                if !scope.contains_key(name) {
                    return true;
                }
                names.insert(name.as_str());
            }
            then_body.len() > 1
                || else_body.len() > 1
                || names.len() > 1
                || names.into_iter().any(|name| !seen.insert(name))
        }
        _ => false,
    })
}

pub(in crate::lowering::buffer_loop_outline) fn outline_iteration(
    condition: &NirExpr,
    body: &mut Vec<NirStmt>,
    scope: &Scope,
    names: &mut BTreeSet<String>,
    helpers: &mut Vec<NirFunction>,
    guarded: &mut BTreeSet<String>,
    control_catalog: &ScalarHelpers,
    layouts: &control_values::FlatLayouts,
    structs: &mut Vec<NirStructDef>,
    break_controls: &mut BTreeMap<String, String>,
) -> exits::Boundary {
    // Child effects were checked as scoped expressions, not metadata predicates.
    // Keep that execution mode even for a single conditional scalar update.
    // A counter-only child needs neither a helper nor another entry debit.
    if body.len() > 1 || present(body, scope) {
        Builder {
            names,
            helpers,
            guarded,
            control_catalog,
            layouts,
            structs,
            break_controls,
        }
        .iteration(condition, body, scope)
    } else {
        exits::Boundary::default()
    }
}

pub(in crate::lowering::buffer_loop_outline) fn outline(
    module: &mut NirModule,
    names: &mut BTreeSet<String>,
    helpers: &mut Vec<NirFunction>,
    guarded: &mut BTreeSet<String>,
    control_catalog: &ScalarHelpers,
    layouts: &control_values::FlatLayouts,
    break_controls: &mut BTreeMap<String, String>,
    preserve_entry_flow: bool,
) {
    // Exit recovery introduces bindings outside the loop. Reserve future source
    // locals too, not just the values visible at the loop's entry.
    for function in &module.functions {
        names.extend(function.params.iter().map(|param| param.name.clone()));
        branches::collect_bindings(&function.body, names);
    }
    for function in &mut module.functions {
        if !control_catalog.contains_key(&function.name)
            || (preserve_entry_flow && function.name == "main")
        {
            continue;
        }
        let scope = function
            .params
            .iter()
            .map(|p| (p.name.clone(), p.ty.clone()))
            .collect();
        let mut builder = Builder {
            names,
            helpers,
            guarded,
            control_catalog,
            layouts,
            structs: &mut module.structs,
            break_controls,
        };
        builder.block(&mut function.body, scope);
    }
}

struct Builder<'a> {
    names: &'a mut BTreeSet<String>,
    helpers: &'a mut Vec<NirFunction>,
    guarded: &'a mut BTreeSet<String>,
    control_catalog: &'a ScalarHelpers,
    layouts: &'a control_values::FlatLayouts,
    structs: &'a mut Vec<NirStructDef>,
    break_controls: &'a mut BTreeMap<String, String>,
}

impl Builder<'_> {
    fn block(&mut self, body: &mut Vec<NirStmt>, mut scope: Scope) {
        let mut output = Vec::new();
        for mut stmt in std::mem::take(body) {
            match &mut stmt {
                NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. } => {
                    let ty = control_values::value_type(
                        value,
                        &scope,
                        self.control_catalog,
                        self.layouts,
                    )
                    .expect("admitted control binding");
                    scope.insert(name.clone(), ty);
                }
                NirStmt::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    self.block(then_body, scope.clone());
                    self.block(else_body, scope.clone());
                }
                NirStmt::While { condition, body }
                    if present(body, &scope)
                        || induction::parse(condition, body).is_some_and(|i| !i.leading) =>
                {
                    let boundary = self.iteration(condition, body, &scope);
                    output.extend(boundary.before);
                    output.push(stmt);
                    output.extend(boundary.after);
                    continue;
                }
                _ => {}
            }
            output.push(stmt);
        }
        *body = output;
    }

    fn iteration(
        &mut self,
        condition: &NirExpr,
        body: &mut Vec<NirStmt>,
        scope: &Scope,
    ) -> exits::Boundary {
        let original = std::mem::take(body);
        let iteration = induction::parse(condition, &original).expect("admitted induction step");
        let step = iteration.step.clone();
        let NirStmt::Let {
            name: induction, ..
        } = &step
        else {
            unreachable!("admitted induction binding")
        };
        let plan = exits::prepare(&iteration, scope, self.names);
        let effects = plan.effects;
        let scope = &plan.scope;
        // A state field belongs to a binding, not to a statement. Repeated
        // writes and different branch write sets return each carry exactly once.
        let writes = sequences::carry_names(&effects);
        let carries = plan.carries;
        let mut mutations = MutationScope {
            writable: writes.into_iter().collect(),
            protected: BTreeSet::new(),
        };
        mutations.writable.insert(induction.clone());
        let mut inputs = BTreeSet::new();
        let mut discovered = Vec::new();
        let types = validation::EffectTypes::Values(self.control_catalog, self.layouts);
        validate_effects(
            std::slice::from_ref(&step),
            &mut scope.clone(),
            &mut inputs,
            types,
            &mutations,
            &mut discovered,
        )
        .expect("admitted scalar induction");
        validate_effects(
            &effects,
            &mut scope.clone(),
            &mut inputs,
            types,
            &mutations,
            &mut discovered,
        )
        .expect("admitted nested decisions");
        inputs.extend(carries.iter().cloned());
        let mut params = captured_params(inputs, scope);
        let transport = scalar_carries::Plan::new(&carries, scope, Some(self.layouts));
        let aggregate = (transport.needs_struct()
            || plan.breaking.is_some()
            || carries
                .iter()
                .any(|name| scope[name] == scalar_type("bool")))
        .then(|| scalar_carries::state_type(&transport, self.names, self.structs));
        let returned = scalar_carries::value(&transport, aggregate.as_ref());
        let name = branches::fresh_name("__nuis_scalar_iteration", self.names);
        if let Some(flag) = &plan.breaking {
            self.break_controls.insert(name.clone(), flag.clone());
        }
        // The driver still owns induction and preflights its complete bound.
        // Leading loops advance a private copy before effects. Trailing loops
        // observe the entry index; only the driver's accepted backedge advances it.
        let first_branch = self.helpers.len();
        let mut helper_body = if iteration.leading {
            vec![step.clone()]
        } else {
            vec![]
        };
        helper_body.extend(branches::outline_effects(
            effects,
            &mut scope.clone(),
            self.names,
            self.helpers,
            self.guarded,
            types,
            &mutations,
            self.structs,
            self.break_controls,
        ));
        helper_body.push(NirStmt::Return(Some(returned)));
        // Backedge slots remain i64 even for bool source bindings. Decode in the
        // same iteration helper, rather than adding a second call/budget boundary.
        let mut bindings = scope.keys().cloned().collect();
        branches::collect_bindings(&helper_body, &mut bindings);
        let mut seeds = Vec::new();
        let args = params
            .iter_mut()
            .map(|param| {
                let input = NirExpr::Var(param.name.clone());
                if plan.breaking.as_ref() == Some(&param.name) {
                    NirExpr::Int(0)
                } else if param.ty == scalar_type("bool") && carries.contains(&param.name) {
                    let word = branches::fresh_name("__nuis_bool_seed", &mut bindings);
                    seeds.push(scalar_carries::binding(
                        &param.name,
                        scope,
                        NirExpr::Var(word.clone()),
                    ));
                    param.name = word;
                    param.ty = scalar_type("i64");
                    NirExpr::CastBoolToI64(Box::new(input))
                } else {
                    input
                }
            })
            .collect();
        seeds.extend(helper_body);
        let helper_body = seeds;
        let call = NirExpr::Call {
            callee: name.clone(),
            args,
        };
        let mut function = helper(name, params, helper_body);
        if let Some(ty) = aggregate {
            let mut bindings = scope.keys().cloned().collect();
            branches::collect_bindings(&function.body, &mut bindings);
            let temporary = branches::fresh_name("__nuis_nested_state", &mut bindings);
            *body = scalar_carries::projected_call(temporary, &ty, &transport, call);
            function.return_type = Some(ty);
        } else if let Some(carry) = carries.first() {
            body.push(NirStmt::Let {
                name: carry.clone(),
                ty: Some(scalar_type("i64")),
                value: call,
            });
        } else {
            // The iteration may have only local work. Its scalar result is not
            // a new state slot and must not become a seed for the next trip.
            body.push(NirStmt::Expr(call));
        }
        if let Some(flag) = plan.breaking {
            body.push(exits::guard(flag));
        }
        body.push(step);
        // The generic effect outliner has already checked these source predicates.
        // Preserve short-circuit evaluation inside the new private call boundary.
        let mut branches = self.helpers.split_off(first_branch);
        branches.push(function);
        for mut function in branches {
            conditions::outline(
                &mut function,
                self.names,
                self.helpers,
                self.guarded,
                self.control_catalog,
                self.layouts,
            );
            self.helpers.push(function);
        }
        plan.boundary
    }
}
