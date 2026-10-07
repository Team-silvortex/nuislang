use super::*;

#[path = "conditional_returns_effect_region_bounds.rs"]
pub(in super::super) mod bounds;
use bounds::{preflight, region_end};
#[path = "conditional_returns_effect_region_joins.rs"]
mod joins;
#[path = "conditional_returns_effect_region_tails.rs"]
mod tails;

struct Arm {
    steps: Vec<Step>,
    tail: Vec<NirStmt>,
    has_initializer_work: bool,
    continuation: Option<String>,
}

enum Step {
    Join(joins::Join),
    Bind {
        name: String,
        ty: NirTypeRef,
        value: NirExpr,
        captures: Vec<NirParam>,
    },
    Print {
        value: NirExpr,
        captures: Option<Vec<NirParam>>,
    },
    Exit {
        name: String,
        value: NirExpr,
        captures: Vec<NirParam>,
    },
    Branch {
        condition: String,
        value: NirExpr,
        captures: Vec<NirParam>,
        yes: Vec<Step>,
        no: Vec<Step>,
    },
}

pub(in super::super) struct RegionPlan {
    pure: Plan,
    yes: Arm,
    no: Arm,
    gate: String,
    bindings: BTreeSet<String>,
}

#[allow(clippy::too_many_arguments)]
pub(in super::super) fn prepare(
    condition: &NirExpr,
    then_body: &[NirStmt],
    else_body: &[NirStmt],
    result: &NirTypeRef,
    scope: &Scope,
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
    checked: &BTreeSet<String>,
    bindings: &BTreeSet<String>,
) -> Option<RegionPlan> {
    let yes_end = region_end(then_body)?;
    let no_end = region_end(else_body)?;
    if ![(&then_body[..yes_end]), (&else_body[..no_end])]
        .into_iter()
        .flatten()
        .any(|stmt| {
            matches!(stmt, NirStmt::If { .. })
                || matches!(stmt, NirStmt::Let { value, .. } | NirStmt::Const { value, .. }
            if !matches!(value, NirExpr::Var(_) | NirExpr::Int(_) | NirExpr::Bool(_)))
        })
        || !entry::admitted(condition, scope, catalog, layouts)
    {
        return None;
    }
    // Bound both original arms and expanded suffixes before recursive typing or
    // cloning. Generated names remain provisional until the complete plan wins.
    for (body, end) in [(then_body, yes_end), (else_body, no_end)] {
        if !preflight(body, end) {
            return None;
        }
        if !suffix::reserve_staged_prefix(&body[end..], &body[..end]) {
            return None;
        }
    }
    let mut bindings = bindings.clone();
    let gate = branches::fresh_name("__nuis_effect_region_gate", &mut bindings);
    let mut ready = scope.clone();
    let yes = prepare_arm(
        then_body,
        yes_end,
        result,
        scope,
        &mut ready,
        catalog,
        layouts,
        checked,
        &mut bindings,
    )?;
    let no = prepare_arm(
        else_body,
        no_end,
        result,
        scope,
        &mut ready,
        catalog,
        layouts,
        checked,
        &mut bindings,
    )?;
    let mut pure = super::super::prepare_staged(
        condition,
        &yes.tail,
        &no.tail,
        result,
        &ready,
        catalog,
        layouts,
        checked,
        yes.has_initializer_work || no.has_initializer_work,
    )
    .or_else(|| {
        tails::prepare(
            condition, &yes, &no, result, &ready, catalog, layouts, checked,
        )
    })?;
    if yes.continuation.is_some() || no.continuation.is_some() {
        // Wrap original source tails, not scalar seeds synthesized by another
        // return proof. An exited prefix must never execute its pure suffix.
        pure.yes = Some(continuing_tail(&yes, result, &ready, catalog, layouts));
        pure.no = Some(continuing_tail(&no, result, &ready, catalog, layouts));
        for name in [&yes.continuation, &no.continuation].into_iter().flatten() {
            pure.params.push(NirParam {
                name: name.clone(),
                ty: scalar_type("bool"),
            });
        }
        pure.params.sort_by(|a, b| a.name.cmp(&b.name));
        pure.stored_exit = true;
        pure.readiness = None;
    }
    Some(RegionPlan {
        pure,
        yes,
        no,
        gate,
        bindings,
    })
}

fn continuing_tail(
    arm: &Arm,
    result: &NirTypeRef,
    scope: &Scope,
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
) -> Vec<NirStmt> {
    let tail = suffix::prepare(&arm.tail, result, scope, catalog, layouts)
        .unwrap_or_else(|| arm.tail.clone());
    match &arm.continuation {
        Some(name) => vec![NirStmt::If {
            condition: NirExpr::Var(name.clone()),
            then_body: tail,
            else_body: vec![],
        }],
        None => tail,
    }
}

#[allow(clippy::too_many_arguments)]
fn prepare_arm(
    body: &[NirStmt],
    end: usize,
    result: &NirTypeRef,
    scope: &Scope,
    ready: &mut Scope,
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
    checked: &BTreeSet<String>,
    bindings: &mut BTreeSet<String>,
) -> Option<Arm> {
    let mut scope = scope.clone();
    let mut values = BTreeMap::<String, NirExpr>::new();
    let mut has_initializer_work = false;
    let (steps, continues) = prepare_steps(
        &body[..end],
        result,
        &mut scope,
        &mut values,
        ready,
        catalog,
        layouts,
        checked,
        bindings,
        &mut has_initializer_work,
    )?;
    if !continues && end < body.len() {
        return None;
    }
    // Source validation precedes substitution, so private ready snapshots cannot
    // hide shadowing, forward/branch-local uses or unreachable source suffixes.
    suffix::validate(&body[end..], scope, result, catalog, layouts)?;
    let mut tail = body[end..].to_vec();
    let aliases = values
        .iter()
        .map(|(name, value)| (name.clone(), value))
        .collect();
    aliases::rewrite(&mut tail, &aliases);
    let continuation = has_exit(&steps).then(|| {
        let name = branches::fresh_name("__nuis_effect_continuation", bindings);
        ready.insert(name.clone(), scalar_type("bool"));
        name
    });
    Some(Arm {
        steps,
        tail,
        has_initializer_work,
        continuation,
    })
}

#[allow(clippy::too_many_arguments)]
fn prepare_steps(
    body: &[NirStmt],
    result: &NirTypeRef,
    scope: &mut Scope,
    values: &mut BTreeMap<String, NirExpr>,
    ready: &mut Scope,
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
    checked: &BTreeSet<String>,
    bindings: &mut BTreeSet<String>,
    has_initializer_work: &mut bool,
) -> Option<(Vec<Step>, bool)> {
    let mut steps = Vec::new();
    let mut continues = true;
    for stmt in body {
        if !continues {
            return None;
        }
        match stmt {
            NirStmt::Return(Some(value)) => {
                if !expression(value, scope)
                    || control_values::value_type(value, scope, catalog, layouts).as_ref()
                        != Some(result)
                {
                    return None;
                }
                let value = rewrite_value(value, values);
                if control_values::value_type(&value, ready, catalog, layouts).as_ref()
                    != Some(result)
                {
                    return None;
                }
                let mut inputs = BTreeSet::new();
                control_values::collect_inputs(&value, &mut inputs);
                if inputs
                    .iter()
                    .any(|name| !ready.get(name).is_some_and(scalar))
                {
                    return None;
                }
                *has_initializer_work |=
                    speculation::block_has_checked_arithmetic(std::slice::from_ref(stmt), checked)
                        || scalar_helpers::contains_calls(std::slice::from_ref(stmt));
                steps.push(Step::Exit {
                    name: branches::fresh_name("__nuis_effect_exit_value", bindings),
                    value,
                    captures: captured_params(inputs, ready),
                });
                continues = false;
            }
            NirStmt::Print(value) => {
                print_values::prepare(value, scope, catalog, layouts)?;
                let value = rewrite_value(value, values);
                let captures = print_values::prepare(&value, ready, catalog, layouts)?.params;
                steps.push(Step::Print { value, captures });
            }
            NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. } => {
                if scope.contains_key(name)
                    || !conditional_values::prefix::computed_logical_root(value)
                {
                    return None;
                }
                let ty = control_values::value_type(value, scope, catalog, layouts)?;
                let declared = match stmt {
                    NirStmt::Let { ty, .. } => ty.as_ref(),
                    NirStmt::Const { ty, .. } => Some(ty),
                    _ => unreachable!(),
                };
                if !scalar(&ty) || declared.is_some_and(|declared| declared != &ty) {
                    return None;
                }
                let value = rewrite_value(value, values);
                if control_values::value_type(&value, ready, catalog, layouts).as_ref() != Some(&ty)
                {
                    return None;
                }
                let value = if matches!(value, NirExpr::Var(_) | NirExpr::Int(_) | NirExpr::Bool(_))
                {
                    value
                } else {
                    let mut inputs = BTreeSet::new();
                    control_values::collect_inputs(&value, &mut inputs);
                    if inputs
                        .iter()
                        .any(|name| !ready.get(name).is_some_and(scalar))
                    {
                        return None;
                    }
                    *has_initializer_work |=
                        speculation::block_has_checked_arithmetic(
                            std::slice::from_ref(stmt),
                            checked,
                        ) || scalar_helpers::contains_calls(std::slice::from_ref(stmt));
                    let captures = captured_params(inputs, ready);
                    let slot = branches::fresh_name("__nuis_prefix_value", bindings);
                    ready.insert(slot.clone(), ty.clone());
                    steps.push(Step::Bind {
                        name: slot.clone(),
                        ty: ty.clone(),
                        value,
                        captures,
                    });
                    NirExpr::Var(slot)
                };
                values.insert(name.clone(), value);
                scope.insert(name.clone(), ty);
            }
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                if !entry::admitted(condition, scope, catalog, layouts) {
                    return None;
                }
                let value = rewrite_value(condition, values);
                if control_values::value_type(&value, ready, catalog, layouts)
                    != Some(scalar_type("bool"))
                {
                    return None;
                }
                let mut inputs = BTreeSet::new();
                control_values::collect_inputs(&value, &mut inputs);
                if inputs
                    .iter()
                    .any(|name| !ready.get(name).is_some_and(scalar))
                {
                    return None;
                }
                *has_initializer_work |= entry::has_work(condition);
                let captures = captured_params(inputs, ready);
                let condition = branches::fresh_name("__nuis_effect_condition", bindings);
                ready.insert(condition.clone(), scalar_type("bool"));
                let mut yes_scope = scope.clone();
                let mut yes_values = values.clone();
                let mut no_scope = scope.clone();
                let mut no_values = values.clone();
                let (yes, yes_continues) = prepare_steps(
                    then_body,
                    result,
                    &mut yes_scope,
                    &mut yes_values,
                    ready,
                    catalog,
                    layouts,
                    checked,
                    bindings,
                    has_initializer_work,
                )?;
                let (no, no_continues) = prepare_steps(
                    else_body,
                    result,
                    &mut no_scope,
                    &mut no_values,
                    ready,
                    catalog,
                    layouts,
                    checked,
                    bindings,
                    has_initializer_work,
                )?;
                // Only a fresh scalar result on every continuing arm may escape. All other
                // child bindings and alias facts retain their lexical scopes.
                let joined = joins::prepare(
                    joins::Arm {
                        body: then_body,
                        scope: &yes_scope,
                        values: &yes_values,
                        continues: yes_continues,
                    },
                    joins::Arm {
                        body: else_body,
                        scope: &no_scope,
                        values: &no_values,
                        continues: no_continues,
                    },
                    scope,
                    ready,
                    &condition,
                    bindings,
                );
                continues = yes_continues || no_continues;
                steps.push(Step::Branch {
                    condition,
                    value,
                    captures,
                    yes,
                    no,
                });
                if let Some((name, joined)) = joined {
                    scope.insert(name.clone(), joined.ty.clone());
                    values.insert(name, NirExpr::Var(joined.name.clone()));
                    steps.push(Step::Join(joined));
                }
            }
            _ => return None,
        }
    }
    Some((steps, continues))
}

fn has_exit(steps: &[Step]) -> bool {
    steps.iter().any(|step| match step {
        Step::Exit { .. } => true,
        Step::Branch { yes, no, .. } => has_exit(yes) || has_exit(no),
        _ => false,
    })
}

fn rewrite_value(value: &NirExpr, values: &BTreeMap<String, NirExpr>) -> NirExpr {
    let mut value = value.clone();
    let aliases = values
        .iter()
        .map(|(name, value)| (name.clone(), value))
        .collect();
    aliases::rewrite_expr(&mut value, &aliases);
    value
}

pub(in super::super) fn install(
    mut plan: RegionPlan,
    result: &NirTypeRef,
    names: &mut BTreeSet<String>,
    bindings: &mut BTreeSet<String>,
    helpers: &mut Vec<NirFunction>,
    definitions: &mut Vec<NirStructDef>,
) -> Vec<NirStmt> {
    *bindings = plan.bindings;
    let mut output = vec![NirStmt::Let {
        name: plan.gate.clone(),
        ty: Some(scalar_type("bool")),
        value: plan.pure.condition,
    }];
    let mut exits = Vec::new();
    for (arm, selected) in [(plan.yes, true), (plan.no, false)] {
        let condition = if selected {
            NirExpr::Var(plan.gate.clone())
        } else {
            NirExpr::Binary {
                op: NirBinaryOp::Eq,
                lhs: Box::new(NirExpr::Var(plan.gate.clone())),
                rhs: Box::new(NirExpr::Bool(false)),
            }
        };
        let live = install_steps(
            arm.steps,
            &condition,
            result,
            names,
            bindings,
            helpers,
            &mut output,
            &mut exits,
        );
        if let Some(name) = arm.continuation {
            output.push(NirStmt::Let {
                name,
                ty: Some(scalar_type("bool")),
                value: live,
            });
        }
    }
    for (condition, name) in exits {
        output.push(NirStmt::If {
            condition,
            then_body: vec![NirStmt::Return(Some(NirExpr::Var(name)))],
            else_body: vec![],
        });
    }
    plan.pure.condition = NirExpr::Var(plan.gate);
    output.extend(super::super::install(
        plan.pure,
        result,
        names,
        bindings,
        helpers,
        definitions,
    ));
    output
}

#[allow(clippy::too_many_arguments)]
fn install_steps(
    steps: Vec<Step>,
    condition: &NirExpr,
    result: &NirTypeRef,
    names: &mut BTreeSet<String>,
    bindings: &mut BTreeSet<String>,
    helpers: &mut Vec<NirFunction>,
    output: &mut Vec<NirStmt>,
    exits: &mut Vec<(NirExpr, String)>,
) -> NirExpr {
    let mut condition = condition.clone();
    for step in steps {
        match step {
            Step::Join(joined) => joins::install(joined, &condition, names, helpers, output),
            Step::Exit {
                name,
                value,
                captures,
            } => {
                install_steps(
                    vec![Step::Bind {
                        name: name.clone(),
                        ty: result.clone(),
                        value,
                        captures,
                    }],
                    &condition,
                    result,
                    names,
                    bindings,
                    helpers,
                    output,
                    exits,
                );
                exits.push((condition.clone(), name));
                condition = NirExpr::Bool(false);
            }
            Step::Bind {
                name,
                ty,
                value,
                captures,
            } => {
                let mut parameter_names = captures.iter().map(|p| p.name.clone()).collect();
                let gate = branches::fresh_name("__nuis_prefix_condition", &mut parameter_names);
                let mut params = vec![NirParam {
                    name: gate.clone(),
                    ty: scalar_type("bool"),
                }];
                params.extend(captures);
                let mut args = vec![condition.clone()];
                args.extend(params[1..].iter().map(|p| NirExpr::Var(p.name.clone())));
                let function = branches::fresh_name("__nuis_conditional_prefix_value", names);
                let seed = if ty == scalar_type("bool") {
                    NirExpr::Bool(false)
                } else {
                    NirExpr::Int(0)
                };
                let mut initializer = helper(
                    function.clone(),
                    params,
                    vec![NirStmt::If {
                        condition: NirExpr::Var(gate),
                        then_body: vec![NirStmt::Return(Some(value))],
                        else_body: vec![NirStmt::Return(Some(seed))],
                    }],
                );
                initializer.return_type = Some(ty.clone());
                helpers.push(initializer);
                output.push(NirStmt::Let {
                    name,
                    ty: Some(ty),
                    value: NirExpr::Call {
                        callee: function,
                        args,
                    },
                });
            }
            Step::Print { value, captures } => {
                let value = print_values::install(
                    print_values::PrintValue {
                        value: &value,
                        params: captures,
                    },
                    &condition,
                    names,
                    bindings,
                    helpers,
                    output,
                );
                output.push(NirStmt::If {
                    condition: condition.clone(),
                    then_body: vec![NirStmt::Print(value)],
                    else_body: vec![],
                });
            }
            Step::Branch {
                condition: slot,
                value,
                captures,
                yes,
                no,
            } => {
                install_steps(
                    vec![Step::Bind {
                        name: slot.clone(),
                        ty: scalar_type("bool"),
                        value,
                        captures,
                    }],
                    &condition,
                    result,
                    names,
                    bindings,
                    helpers,
                    output,
                    exits,
                );
                let mut gates = Vec::new();
                for selected in [true, false] {
                    let branch = if selected {
                        NirExpr::Var(slot.clone())
                    } else {
                        NirExpr::Binary {
                            op: NirBinaryOp::Eq,
                            lhs: Box::new(NirExpr::Var(slot.clone())),
                            rhs: Box::new(NirExpr::Bool(false)),
                        }
                    };
                    let gate = branches::fresh_name("__nuis_effect_path", bindings);
                    output.push(NirStmt::Let {
                        name: gate.clone(),
                        ty: Some(scalar_type("bool")),
                        value: NirExpr::Binary {
                            op: NirBinaryOp::And,
                            lhs: Box::new(condition.clone()),
                            rhs: Box::new(branch),
                        },
                    });
                    gates.push(NirExpr::Var(gate));
                }
                let early = has_exit(&yes) || has_exit(&no);
                let yes = install_steps(
                    yes, &gates[0], result, names, bindings, helpers, output, exits,
                );
                let no = install_steps(
                    no, &gates[1], result, names, bindings, helpers, output, exits,
                );
                if early {
                    let live = branches::fresh_name("__nuis_effect_live", bindings);
                    output.push(NirStmt::Let {
                        name: live.clone(),
                        ty: Some(scalar_type("bool")),
                        value: NirExpr::Binary {
                            op: NirBinaryOp::Or,
                            lhs: Box::new(yes),
                            rhs: Box::new(no),
                        },
                    });
                    condition = NirExpr::Var(live);
                }
            }
        }
    }
    condition
}
