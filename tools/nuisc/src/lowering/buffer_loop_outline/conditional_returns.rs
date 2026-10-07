use super::*;

#[path = "conditional_returns_effects.rs"]
mod effects;
#[path = "conditional_returns_entry.rs"]
mod entry;
#[path = "conditional_returns_fallthrough.rs"]
mod fallthrough;
#[cfg(test)]
#[path = "conditional_logical_trees_tests.rs"]
mod nested_tests;
#[path = "conditional_returns_predicates.rs"]
mod predicates;
#[path = "conditional_returns_prefix.rs"]
mod prefix;
#[path = "conditional_returns_signal.rs"]
mod signal;
#[path = "conditional_returns_suffix.rs"]
mod suffix;
#[path = "conditional_returns_terminal.rs"]
mod terminal;
#[cfg(test)]
#[path = "conditional_returns_tests.rs"]
pub(super) mod tests;

// Keep parent effects and exits in place. Only an admitted top-level return
// computation moves behind the existing pure scalar-control helper contract.
pub(super) fn outline(
    module: &mut NirModule,
    catalog: &ScalarHelpers,
    control_roots: &BTreeSet<String>,
    layouts: &impl control_values::ValueLayouts,
    names: &mut BTreeSet<String>,
) -> BTreeSet<String> {
    let checked = speculation::collect_checked_arithmetic(module);
    let mut helpers = Vec::new();
    let mut definitions = Vec::new();
    for function in &mut module.functions {
        let Some(result) = function.return_type.as_ref().filter(|ty| scalar(ty)) else {
            continue;
        };
        if control_roots.contains(&function.name)
            || function.is_async
            || !function.generic_params.is_empty()
            || !function.where_bounds.is_empty()
        {
            continue;
        }
        let mut scope = function
            .params
            .iter()
            .map(|param| (param.name.clone(), param.ty.clone()))
            .collect::<Scope>();
        let mut bindings = scope.keys().cloned().collect();
        branches::collect_bindings(&function.body, &mut bindings);
        let mut output = Vec::new();
        for stmt in std::mem::take(&mut function.body) {
            match &stmt {
                NirStmt::Let { name, ty, value } => {
                    let inferred = ty.clone().or_else(|| {
                        expression(value, &scope)
                            .then(|| control_values::value_type(value, &scope, catalog, layouts))
                            .flatten()
                    });
                    scope.remove(name);
                    if let Some(ty) = inferred {
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
                    if let Some(plan) = prepare(
                        condition, then_body, else_body, result, &scope, catalog, layouts, &checked,
                    ) {
                        output.extend(install(
                            plan,
                            result,
                            names,
                            &mut bindings,
                            &mut helpers,
                            &mut definitions,
                        ));
                        continue;
                    }
                    if let Some(plan) = effects::prepare(
                        condition, then_body, else_body, result, &scope, catalog, layouts, &checked,
                    ) {
                        output.extend(effects::install(
                            plan,
                            result,
                            names,
                            &mut bindings,
                            &mut helpers,
                            &mut definitions,
                        ));
                        continue;
                    }
                    if let Some((yes, no)) = effects::aliases::prepare(
                        then_body, else_body, result, &scope, catalog, layouts,
                    ) {
                        if let Some(plan) = effects::prepare(
                            condition, &yes, &no, result, &scope, catalog, layouts, &checked,
                        ) {
                            output.extend(effects::install(
                                plan,
                                result,
                                names,
                                &mut bindings,
                                &mut helpers,
                                &mut definitions,
                            ));
                            continue;
                        }
                    }
                    if let Some(plan) = effects::regions::prepare(
                        condition, then_body, else_body, result, &scope, catalog, layouts,
                        &checked, &bindings,
                    ) {
                        output.extend(effects::regions::install(
                            plan,
                            result,
                            names,
                            &mut bindings,
                            &mut helpers,
                            &mut definitions,
                        ));
                        continue;
                    }
                }
                // Do not lift work out of loops or an enclosing source branch.
                _ => {}
            }
            output.push(stmt);
        }
        function.body = output;
    }
    let generated = helpers
        .iter()
        .map(|function| function.name.clone())
        .collect();
    module.functions.extend(helpers);
    module.structs.extend(definitions);
    generated
}

struct Plan {
    condition: NirExpr,
    yes: Option<Vec<NirStmt>>,
    no: Option<Vec<NirStmt>>,
    params: Vec<NirParam>,
    readiness: Option<(NirExpr, NirExpr)>,
    stored_exit: bool,
}

#[allow(clippy::too_many_arguments)]
fn prepare(
    condition: &NirExpr,
    then_body: &[NirStmt],
    else_body: &[NirStmt],
    result: &NirTypeRef,
    scope: &Scope,
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
    checked: &BTreeSet<String>,
) -> Option<Plan> {
    prepare_staged(
        condition, then_body, else_body, result, scope, catalog, layouts, checked, false,
    )
}

#[allow(clippy::too_many_arguments)]
fn prepare_staged(
    condition: &NirExpr,
    then_body: &[NirStmt],
    else_body: &[NirStmt],
    result: &NirTypeRef,
    scope: &Scope,
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
    checked: &BTreeSet<String>,
    selected_initializer_work: bool,
) -> Option<Plan> {
    if !entry::admitted(condition, scope, catalog, layouts) {
        return None;
    }
    // Normalize only independently validated source continuations. Existing
    // arm proofs still validate the bounded result and its exact captures.
    let yes_source = suffix::prepare(then_body, result, scope, catalog, layouts);
    let no_source = suffix::prepare(else_body, result, scope, catalog, layouts);
    let then_body = yes_source.as_deref().unwrap_or(then_body);
    let else_body = no_source.as_deref().unwrap_or(else_body);
    let mut inputs = BTreeSet::new();
    let mut partial = false;
    let mut has_return = false;
    let mut stored_exit = false;
    let mut arm = |body: &[NirStmt]| -> Option<(Option<Vec<NirStmt>>, NirExpr)> {
        match body {
            [] => Some((None, NirExpr::Bool(false))),
            [NirStmt::Return(Some(value))]
                if expression(value, scope)
                    && control_values::value_type(value, scope, catalog, layouts).as_ref()
                        == Some(result) =>
            {
                has_return = true;
                control_values::collect_inputs(value, &mut inputs);
                Some((Some(body.to_vec()), NirExpr::Bool(true)))
            }
            _ => {
                if let Some(arm) = prefix::prepare(body, result, scope, catalog, layouts)
                    .or_else(|| terminal::prepare(body, result, scope, catalog, layouts))
                {
                    has_return = true;
                    inputs.extend(arm.inputs);
                    Some((Some(arm.body), NirExpr::Bool(true)))
                } else {
                    partial = true;
                    if let Some(arm) = fallthrough::prepare(body, result, scope, catalog, layouts) {
                        has_return |= arm.has_return;
                        inputs.extend(arm.value.inputs);
                        Some((Some(arm.value.body), arm.ready))
                    } else {
                        let arm = signal::prepare(body, result, scope, catalog, layouts)?;
                        stored_exit = true;
                        has_return |= arm.has_return;
                        inputs.extend(arm.inputs);
                        Some((Some(body.to_vec()), NirExpr::Bool(false)))
                    }
                }
            }
        }
    };
    let (mut yes, yes_ready) = arm(then_body)?;
    let (mut no, no_ready) = arm(else_body)?;
    // Synthetic seeds must not invent source exits in a computation-only tree.
    if !has_return {
        return None;
    }
    // Only the separately proven selected region (bindings/condition snapshots) may supply this
    // work authority. Ordinary print arguments still grant no tail eligibility.
    if !entry::has_work(condition)
        && !selected_initializer_work
        && ![then_body, else_body].into_iter().any(|body| {
            speculation::block_has_checked_arithmetic(body, checked)
                || scalar_helpers::contains_calls(body)
        })
    {
        return None;
    }
    // No borrowing, resource transport, outer-state mutation or provisional
    // name capture is authorized by this local return computation proof.
    if inputs
        .iter()
        .any(|name| !scope.get(name).is_some_and(scalar))
    {
        return None;
    }
    if stored_exit {
        // Wrap original leaves, never the other proof's synthetic scalar seeds.
        yes = (!then_body.is_empty()).then(|| then_body.to_vec());
        no = (!else_body.is_empty()).then(|| else_body.to_vec());
    }
    Some(Plan {
        condition: condition.clone(),
        yes,
        no,
        params: captured_params(inputs, scope),
        readiness: partial.then_some((yes_ready, no_ready)),
        stored_exit,
    })
}

fn scalar(ty: &NirTypeRef) -> bool {
    ty == &scalar_type("bool") || ty == &scalar_type("i64")
}

fn predicate(value: &NirExpr, scope: &Scope) -> bool {
    match value {
        NirExpr::Bool(_) => true,
        NirExpr::Var(name) => scope.get(name) == Some(&scalar_type("bool")),
        _ => false,
    }
}

fn expression(value: &NirExpr, _scope: &Scope) -> bool {
    match value {
        NirExpr::Binary {
            op: NirBinaryOp::And | NirBinaryOp::Or,
            ..
        } => conditional_values::prefix::computed_logical_root(value),
        _ => conditional_values::prefix::expression(value),
    }
}

fn install(
    plan: Plan,
    result: &NirTypeRef,
    names: &mut BTreeSet<String>,
    bindings: &mut BTreeSet<String>,
    helpers: &mut Vec<NirFunction>,
    definitions: &mut Vec<NirStructDef>,
) -> Vec<NirStmt> {
    let mut parameter_names = plan.params.iter().map(|param| param.name.clone()).collect();
    for body in [&plan.yes, &plan.no].into_iter().flatten() {
        branches::collect_bindings(body, &mut parameter_names);
    }
    let predicate = branches::fresh_name("__nuis_return_condition", &mut parameter_names);
    let params = std::iter::once(NirParam {
        name: predicate.clone(),
        ty: scalar_type("bool"),
    })
    .chain(plan.params)
    .collect::<Vec<_>>();
    let seed = if result == &scalar_type("bool") {
        NirExpr::Bool(false)
    } else {
        NirExpr::Int(0)
    };
    let skipped = vec![NirStmt::Return(Some(seed))];
    let both = plan.yes.is_some() && plan.no.is_some();
    let selected = plan.yes.is_some();
    let name = branches::fresh_name("__nuis_conditional_return", names);
    let returned = if plan.stored_exit {
        let definition = signal::definition(result, names);
        let ty = scalar_type(&definition.name);
        definitions.push(definition);
        ty
    } else {
        result.clone()
    };
    let (yes, no) = if plan.stored_exit {
        // Empty outer arms are continuations, not seeded source returns.
        let yes = signal::wrap(
            plan.yes.unwrap_or_default(),
            &returned,
            result,
            &mut parameter_names,
        );
        let no = signal::wrap(
            plan.no.unwrap_or_default(),
            &returned,
            result,
            &mut parameter_names,
        );
        (yes, no)
    } else {
        (
            plan.yes.unwrap_or_else(|| skipped.clone()),
            plan.no.unwrap_or(skipped),
        )
    };
    let mut function = helper(
        name.clone(),
        params.clone(),
        vec![NirStmt::If {
            condition: NirExpr::Var(predicate),
            then_body: yes,
            else_body: no,
        }],
    );
    function.return_type = Some(returned.clone());
    helpers.push(function);
    let condition = branches::fresh_name("__nuis_return_gate", bindings);
    let mut args = vec![NirExpr::Var(condition.clone())];
    args.extend(
        params[1..]
            .iter()
            .map(|param| NirExpr::Var(param.name.clone())),
    );
    let call = NirExpr::Call { callee: name, args };
    // Evaluate the original entry once at its source site, before any helper
    // work. Helpers and exit readiness consume only the saved bool.
    let mut output = vec![NirStmt::Let {
        name: condition.clone(),
        ty: Some(scalar_type("bool")),
        value: plan.condition,
    }];
    if plan.stored_exit {
        output.extend(signal::read(call, &returned, result, bindings));
    } else if let Some((yes, no)) = plan.readiness {
        // Run the complete guarded computation first. Readiness reads only
        // total parent atoms; false/zero results never stand in for an exit flag.
        let value = branches::fresh_name("__nuis_return_value", bindings);
        let ready = branches::fresh_name("__nuis_return_exit", bindings);
        output.push(NirStmt::Let {
            name: value.clone(),
            ty: Some(result.clone()),
            value: call,
        });
        output.push(NirStmt::Let {
            name: ready.clone(),
            ty: Some(scalar_type("bool")),
            value: fallthrough::select(NirExpr::Var(condition), yes, no),
        });
        output.push(NirStmt::If {
            condition: NirExpr::Var(ready),
            then_body: vec![NirStmt::Return(Some(NirExpr::Var(value)))],
            else_body: vec![],
        });
    } else if both {
        output.push(NirStmt::Return(Some(call)));
    } else {
        let value = branches::fresh_name("__nuis_return_value", bindings);
        output.push(NirStmt::Let {
            name: value.clone(),
            ty: Some(result.clone()),
            value: call,
        });
        // The parent's return shortcut sees only ready atoms, never the
        // fallible computation. Its existing speculation barrier stays intact.
        let condition = if selected {
            NirExpr::Var(condition)
        } else {
            NirExpr::Binary {
                op: NirBinaryOp::Eq,
                lhs: Box::new(NirExpr::Var(condition)),
                rhs: Box::new(NirExpr::Bool(false)),
            }
        };
        output.push(NirStmt::If {
            condition,
            then_body: vec![NirStmt::Return(Some(NirExpr::Var(value)))],
            else_body: vec![],
        });
    }
    output
}
