use super::*;

struct Arm {
    steps: Vec<Step>,
    tail: Vec<NirStmt>,
    has_initializer_work: bool,
}

enum Step {
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
    let yes_end = region_end(then_body);
    let no_end = region_end(else_body);
    if ![(&then_body[..yes_end]), (&else_body[..no_end])]
        .into_iter()
        .flatten()
        .any(|stmt| {
            matches!(stmt, NirStmt::Let { value, .. } | NirStmt::Const { value, .. }
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
    let pure = super::super::prepare_staged(
        condition,
        &yes.tail,
        &no.tail,
        result,
        &ready,
        catalog,
        layouts,
        checked,
        yes.has_initializer_work || no.has_initializer_work,
    )?;
    Some(RegionPlan {
        pure,
        yes,
        no,
        gate,
        bindings,
    })
}

fn region_end(body: &[NirStmt]) -> usize {
    body.iter()
        .rposition(|stmt| matches!(stmt, NirStmt::Print(_)))
        .map_or(0, |i| i + 1)
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
    let mut steps = Vec::new();
    let mut has_initializer_work = false;
    for stmt in &body[..end] {
        match stmt {
            NirStmt::Print(value) => {
                print_values::prepare(value, &scope, catalog, layouts)?;
                let value = rewrite_value(value, &values);
                let captures = print_values::prepare(&value, ready, catalog, layouts)?.params;
                steps.push(Step::Print { value, captures });
            }
            NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. } => {
                if scope.contains_key(name)
                    || !conditional_values::prefix::computed_logical_root(value)
                {
                    return None;
                }
                let ty = control_values::value_type(value, &scope, catalog, layouts)?;
                let declared = match stmt {
                    NirStmt::Let { ty, .. } => ty.as_ref(),
                    NirStmt::Const { ty, .. } => Some(ty),
                    _ => unreachable!(),
                };
                if !scalar(&ty) || declared.is_some_and(|declared| declared != &ty) {
                    return None;
                }
                let value = rewrite_value(value, &values);
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
                    has_initializer_work |=
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
            _ => return None,
        }
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
    Some(Arm {
        steps,
        tail,
        has_initializer_work,
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
        for step in arm.steps {
            match step {
                Step::Bind {
                    name,
                    ty,
                    value,
                    captures,
                } => {
                    let mut parameter_names = captures.iter().map(|p| p.name.clone()).collect();
                    let gate =
                        branches::fresh_name("__nuis_prefix_condition", &mut parameter_names);
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
                        &mut output,
                    );
                    output.push(NirStmt::If {
                        condition: condition.clone(),
                        then_body: vec![NirStmt::Print(value)],
                        else_body: vec![],
                    });
                }
            }
        }
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
