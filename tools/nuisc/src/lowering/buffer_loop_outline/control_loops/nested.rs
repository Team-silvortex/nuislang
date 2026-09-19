use super::*;

#[path = "nested_conditions.rs"]
mod conditions;

pub(super) fn present(body: &[NirStmt], scope: &Scope) -> bool {
    // Fallible arithmetic must execute in the selected iteration, never as a
    // captured metadata operand. Reuse the shared non-speculation analysis.
    if speculation::block_has_checked_arithmetic(body, &BTreeSet::new())
        || scalar_helpers::contains_calls(body)
        || control_values::has_aggregate_expressions(body)
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

pub(in crate::lowering::buffer_loop_outline) fn outline(
    module: &mut NirModule,
    names: &mut BTreeSet<String>,
    helpers: &mut Vec<NirFunction>,
    guarded: &mut BTreeSet<String>,
    control_catalog: &ScalarHelpers,
    layouts: &control_values::FlatLayouts,
) {
    for function in &mut module.functions {
        if !control_catalog.contains_key(&function.name) {
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
}

impl Builder<'_> {
    fn block(&mut self, body: &mut [NirStmt], mut scope: Scope) {
        for stmt in body {
            match stmt {
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
                NirStmt::While { body, .. } if present(body, &scope) => {
                    self.iteration(body, &scope)
                }
                _ => {}
            }
        }
    }

    fn iteration(&mut self, body: &mut Vec<NirStmt>, scope: &Scope) {
        let mut original = std::mem::take(body).into_iter();
        let step = original.next().expect("admitted induction step");
        let NirStmt::Let {
            name: induction, ..
        } = &step
        else {
            unreachable!("admitted induction binding")
        };
        let effects = original.collect::<Vec<_>>();
        // A state field belongs to a binding, not to a statement. Repeated
        // writes and different branch write sets return each carry exactly once.
        let writes = sequences::carry_names(&effects);
        let carries = writes
            .iter()
            .filter(|name| scope.contains_key(*name))
            .cloned()
            .collect::<Vec<_>>();
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
        let params = captured_params(inputs, scope);
        let aggregate = (carries.len() > 1)
            .then(|| scalar_carries::state_type(&carries, self.names, self.structs));
        let returned = scalar_carries::value(&carries, aggregate.as_ref());
        let name = branches::fresh_name("__nuis_scalar_iteration", self.names);
        // The driver still owns induction and preflights its complete bound. The
        // helper advances a private parameter copy so decisions observe the
        // source's step-first value, without updating the outer index twice.
        let first_branch = self.helpers.len();
        let mut helper_body = vec![step.clone()];
        helper_body.extend(branches::outline_effects(
            effects,
            &mut scope.clone(),
            self.names,
            self.helpers,
            self.guarded,
            types,
            &mutations,
            self.structs,
            &mut BTreeMap::new(),
        ));
        helper_body.push(NirStmt::Return(Some(returned)));
        let call = NirExpr::Call {
            callee: name.clone(),
            args: params
                .iter()
                .map(|p| NirExpr::Var(p.name.clone()))
                .collect(),
        };
        let mut function = helper(name, params, helper_body);
        if let Some(ty) = aggregate {
            let mut bindings = scope.keys().cloned().collect();
            branches::collect_bindings(&function.body, &mut bindings);
            let temporary = branches::fresh_name("__nuis_nested_state", &mut bindings);
            *body = scalar_carries::projected_call(temporary, &ty, &carries, call);
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
    }
}
