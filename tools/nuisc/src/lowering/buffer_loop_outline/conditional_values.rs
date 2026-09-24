use super::*;

// An effectful parent is not a pure helper, but its local value selection can
// still use the same guarded scalar-control contract as a standalone function.
pub(super) fn outline(
    module: &mut NirModule,
    catalog: &ScalarHelpers,
    control_catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
    names: &mut BTreeSet<String>,
) -> BTreeSet<String> {
    let checked = speculation::collect_checked_arithmetic(module);
    let mut helpers = Vec::new();
    for function in &mut module.functions {
        // Typed value admission alone does not provide full control lowering.
        // Keep local selection extraction unless that route is already admitted.
        if control_catalog.contains_key(&function.name) {
            continue;
        }
        let mut scope = function
            .params
            .iter()
            .map(|param| (param.name.clone(), param.ty.clone()))
            .collect::<Scope>();
        let mut bindings = scope.keys().cloned().collect();
        branches::collect_bindings(&function.body, &mut bindings);
        let mut builder = Builder {
            catalog,
            layouts,
            names,
            checked: &checked,
            helpers: &mut helpers,
            bindings,
        };
        builder.block(&mut function.body, &mut scope);
    }
    let generated = helpers
        .iter()
        .map(|function| function.name.clone())
        .collect();
    module.functions.extend(helpers);
    generated
}

struct Builder<'a, L> {
    catalog: &'a ScalarHelpers,
    layouts: &'a L,
    names: &'a mut BTreeSet<String>,
    checked: &'a BTreeSet<String>,
    helpers: &'a mut Vec<NirFunction>,
    bindings: BTreeSet<String>,
}

struct Selection {
    name: String,
    ty: NirTypeRef,
    constant: bool,
    body: Vec<NirStmt>,
    inputs: BTreeSet<String>,
}

impl<L: control_values::ValueLayouts> Builder<'_, L> {
    fn block(&mut self, body: &mut [NirStmt], scope: &mut Scope) {
        for stmt in body {
            match stmt {
                NirStmt::Let { name, ty, value } => {
                    let ty = ty.clone().or_else(|| {
                        control_values::value_type(value, scope, self.catalog, self.layouts)
                    });
                    scope.remove(name);
                    if let Some(ty) = ty {
                        scope.insert(name.clone(), ty);
                    }
                }
                NirStmt::Const { name, ty, .. } => {
                    scope.insert(name.clone(), ty.clone());
                }
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    if let Some((then_value, else_value)) = self.arms(then_body, else_body, scope) {
                        let guarded = [then_body.as_slice(), else_body.as_slice()]
                            .into_iter()
                            .any(|arm| {
                                speculation::block_has_checked_arithmetic(arm, self.checked)
                                    || scalar_helpers::contains_calls(arm)
                            });
                        let ty = then_value.ty.clone();
                        let name = then_value.name.clone();
                        if guarded {
                            *stmt = self.extract(
                                condition.clone(),
                                then_value,
                                else_value,
                                scope,
                                false,
                            );
                        }
                        scope.insert(name, ty);
                    } else if let Some((yes, no, discarded)) =
                        self.one_sided(then_body, else_body, scope)
                    {
                        *stmt = self.extract(condition.clone(), yes, no, scope, discarded);
                    } else {
                        self.block(then_body, &mut scope.clone());
                        self.block(else_body, &mut scope.clone());
                    }
                }
                NirStmt::While { body, .. } => self.block(body, &mut scope.clone()),
                _ => {}
            }
        }
    }

    fn one_sided(
        &self,
        then_body: &[NirStmt],
        else_body: &[NirStmt],
        scope: &Scope,
    ) -> Option<(Selection, Selection, bool)> {
        let (body, selected) = match (then_body.is_empty(), else_body.is_empty()) {
            (false, true) => (then_body, true),
            (true, false) => (else_body, false),
            _ => return None,
        };
        if !speculation::block_has_checked_arithmetic(body, self.checked)
            && !scalar_helpers::contains_calls(body)
        {
            return None;
        }
        let value = self.selection(body, scope)?;
        let mut inputs = BTreeSet::new();
        let seed = match scope.get(&value.name) {
            Some(ty) if ty == &value.ty => {
                inputs.insert(value.name.clone());
                NirExpr::Var(value.name.clone())
            }
            Some(_) => return None,
            None => control_values::zero_value(&value.ty, self.layouts),
        };
        // Dead-binding pruning can erase a total arm but not a fallible arm.
        // A branch-local result must not escape; an outer carry keeps its old seed.
        let discarded = !scope.contains_key(&value.name);
        let empty = Selection {
            name: value.name.clone(),
            ty: value.ty.clone(),
            constant: value.constant,
            body: vec![NirStmt::Return(Some(seed))],
            inputs,
        };
        Some(if selected {
            (value, empty, discarded)
        } else {
            (empty, value, discarded)
        })
    }

    fn arms(
        &self,
        then_body: &[NirStmt],
        else_body: &[NirStmt],
        scope: &Scope,
    ) -> Option<(Selection, Selection)> {
        let yes = self.selection(then_body, scope)?;
        let no = self.selection(else_body, scope)?;
        (yes.name == no.name && yes.ty == no.ty && yes.constant == no.constant).then_some((yes, no))
    }

    fn selection(&self, body: &[NirStmt], scope: &Scope) -> Option<Selection> {
        match body {
            [NirStmt::Let { name, ty, value }] => {
                self.value(name, ty.as_ref(), value, false, scope)
            }
            [NirStmt::Const { name, ty, value }] => self.value(name, Some(ty), value, true, scope),
            [NirStmt::If {
                condition,
                then_body,
                else_body,
            }] => {
                if control_values::value_type(condition, scope, self.catalog, self.layouts)?
                    != scalar_type("bool")
                {
                    return None;
                }
                let (mut yes, no) = self.arms(then_body, else_body, scope)?;
                yes.inputs.extend(no.inputs);
                control_values::collect_inputs(condition, &mut yes.inputs);
                yes.body = vec![NirStmt::If {
                    condition: condition.clone(),
                    then_body: yes.body,
                    else_body: no.body,
                }];
                Some(yes)
            }
            _ => None,
        }
    }

    fn value(
        &self,
        name: &str,
        declared: Option<&NirTypeRef>,
        value: &NirExpr,
        constant: bool,
        scope: &Scope,
    ) -> Option<Selection> {
        let ty = control_values::value_type(value, scope, self.catalog, self.layouts)?;
        if declared.is_some_and(|declared| declared != &ty) {
            return None;
        }
        let mut inputs = BTreeSet::new();
        control_values::collect_inputs(value, &mut inputs);
        Some(Selection {
            name: name.to_owned(),
            ty,
            constant,
            body: vec![NirStmt::Return(Some(value.clone()))],
            inputs,
        })
    }

    fn extract(
        &mut self,
        condition: NirExpr,
        mut yes: Selection,
        no: Selection,
        scope: &Scope,
        discarded: bool,
    ) -> NirStmt {
        let predicate = branches::fresh_name("__nuis_value_condition", &mut self.bindings);
        yes.inputs.extend(no.inputs);
        let mut params = vec![NirParam {
            name: predicate.clone(),
            ty: scalar_type("bool"),
        }];
        params.extend(captured_params(yes.inputs, scope));
        // Only the predicate and already-bound values cross this boundary.
        // Branch calls, projections and arithmetic stay inside the helper.
        let mut args = vec![condition];
        args.extend(
            params[1..]
                .iter()
                .map(|param| NirExpr::Var(param.name.clone())),
        );
        let name = branches::fresh_name("__nuis_conditional_value", self.names);
        let mut function = helper(
            name.clone(),
            params,
            vec![NirStmt::If {
                condition: NirExpr::Var(predicate),
                then_body: yes.body,
                else_body: no.body,
            }],
        );
        function.return_type = Some(yes.ty.clone());
        self.helpers.push(function);
        let value = NirExpr::Call { callee: name, args };
        if discarded {
            // Keep the admitted value statement shape for enclosing loop
            // outliners, without exporting the source branch-local binding.
            NirStmt::Let {
                name: branches::fresh_name("__nuis_discarded_value", &mut self.bindings),
                ty: Some(yes.ty),
                value,
            }
        } else if yes.constant {
            NirStmt::Const {
                name: yes.name,
                ty: yes.ty,
                value,
            }
        } else {
            NirStmt::Let {
                name: yes.name,
                ty: Some(yes.ty),
                value,
            }
        }
    }
}
