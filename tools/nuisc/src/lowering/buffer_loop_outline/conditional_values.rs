use super::*;

#[path = "conditional_values_logical.rs"]
mod logical;
#[path = "conditional_values_prefix.rs"]
pub(super) mod prefix;
#[cfg(test)]
#[path = "conditional_values_prefix_tests.rs"]
mod prefix_tests;
#[cfg(test)]
#[path = "conditional_values_roots_tests.rs"]
mod roots_tests;

// An effectful parent is not a pure helper, but its local value selection can
// still use the same guarded scalar-control contract as a standalone function.
pub(super) fn outline(
    module: &mut NirModule,
    catalog: &ScalarHelpers,
    control_roots: &BTreeSet<String>,
    layouts: &impl control_values::ValueLayouts,
    names: &mut BTreeSet<String>,
) -> BTreeSet<String> {
    let checked = speculation::collect_checked_arithmetic(module);
    let mut helpers = Vec::new();
    for function in &mut module.functions {
        // Full control roots keep their existing local-selection route, but
        // condition laziness must not depend on which route owns the function.
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
            extract_selections: !control_roots.contains(&function.name),
            predicate_returns: function.return_type.as_ref() == Some(&scalar_type("bool")),
        };
        builder.block(&mut function.body, &mut scope, false);
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
    extract_selections: bool,
    predicate_returns: bool,
}

struct Selection {
    name: String,
    ty: NirTypeRef,
    constant: bool,
    body: Vec<NirStmt>,
    inputs: BTreeSet<String>,
}

impl<L: control_values::ValueLayouts> Builder<'_, L> {
    fn block(&mut self, body: &mut [NirStmt], scope: &mut Scope, in_loop: bool) {
        for stmt in body {
            match stmt {
                NirStmt::Let { name, ty, value } => {
                    // Capture the current version before the destination is
                    // rebound; the complete RHS remains inside its guard.
                    let mut guarded_kind = None;
                    if !in_loop && ty.as_ref().is_none_or(|ty| ty == &scalar_type("bool")) {
                        if let Some(guarded) = self.gated_condition(value, scope) {
                            *value = guarded;
                            guarded_kind = Some(scalar_type("bool"));
                        }
                    }
                    // Generated calls are not in the original catalog yet.
                    let ty = ty.clone().or(guarded_kind).or_else(|| {
                        control_values::value_type(value, scope, self.catalog, self.layouts)
                    });
                    scope.remove(name);
                    if let Some(ty) = ty {
                        scope.insert(name.clone(), ty);
                    }
                }
                NirStmt::Const { name, ty, value } => {
                    if !in_loop && ty == &scalar_type("bool") {
                        if let Some(guarded) = self.gated_condition(value, scope) {
                            *value = guarded;
                        }
                    }
                    scope.insert(name.clone(), ty.clone());
                }
                NirStmt::Return(Some(value)) if !in_loop && self.predicate_returns => {
                    if let Some(guarded) = self.gated_condition(value, scope) {
                        *value = guarded;
                    }
                }
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    let atom_gate = matches!(condition, NirExpr::Binary { lhs, .. }
                        if matches!(lhs.as_ref(), NirExpr::Bool(_) | NirExpr::Var(_)));
                    // Loop conditions retain their separately proved atom gate.
                    if !in_loop
                        || (atom_gate && prefix::expression_roots(vec![(condition, 0, true)]))
                    {
                        if let Some(guarded) = self.gated_condition(condition, scope) {
                            *condition = guarded;
                        }
                    }
                    if !self.extract_selections {
                        self.block(then_body, &mut scope.clone(), in_loop);
                        self.block(else_body, &mut scope.clone(), in_loop);
                        continue;
                    }
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
                            let then_value = self.guard_selection(then_value, scope, in_loop);
                            let else_value = self.guard_selection(else_value, scope, in_loop);
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
                        let yes = self.guard_selection(yes, scope, in_loop);
                        let no = self.guard_selection(no, scope, in_loop);
                        *stmt = self.extract(condition.clone(), yes, no, scope, discarded);
                    } else {
                        self.block(then_body, &mut scope.clone(), in_loop);
                        self.block(else_body, &mut scope.clone(), in_loop);
                    }
                }
                NirStmt::While { body, .. } => self.block(body, &mut scope.clone(), true),
                _ => {}
            }
        }
    }

    fn guard_selection(&mut self, mut value: Selection, scope: &Scope, in_loop: bool) -> Selection {
        // An admitted arm becomes a return body before extraction. Normalize
        // its logical root too, without recursively extracting local selections.
        let selections = std::mem::replace(&mut self.extract_selections, false);
        let returns =
            std::mem::replace(&mut self.predicate_returns, value.ty == scalar_type("bool"));
        self.block(&mut value.body, &mut scope.clone(), in_loop);
        self.extract_selections = selections;
        self.predicate_returns = returns;
        value
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
            [locals @ .., last] if !locals.is_empty() && prefix::bounded(body) => {
                let mut inner = scope.clone();
                let mut local_names = BTreeSet::new();
                let mut inputs = BTreeSet::new();
                for stmt in locals {
                    let (name, declared, value) = match stmt {
                        NirStmt::Let { name, ty, value } => (name, ty.as_ref(), value),
                        NirStmt::Const { name, ty, value } => (name, Some(ty), value),
                        _ => return None,
                    };
                    // Intermediate work cannot mutate an outer carry or hide
                    // an earlier local. Only the final selection may rebind.
                    if inner.contains_key(name) {
                        return None;
                    }
                    let value = self.value(name, declared, value, false, &inner)?;
                    inputs.extend(value.inputs);
                    local_names.insert(name.clone());
                    inner.insert(name.clone(), value.ty);
                }
                let mut selected = match last {
                    NirStmt::Let { name, ty, value } if !local_names.contains(name) => {
                        self.value(name, ty.as_ref(), value, false, &inner)?
                    }
                    NirStmt::Const { name, ty, value } if !local_names.contains(name) => {
                        self.value(name, Some(ty), value, true, &inner)?
                    }
                    _ => return None,
                };
                inputs.extend(selected.inputs);
                inputs.retain(|name| !local_names.contains(name));
                selected.inputs = inputs;
                let mut preserved = locals.to_vec();
                preserved.extend(selected.body);
                selected.body = preserved;
                Some(selected)
            }
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
