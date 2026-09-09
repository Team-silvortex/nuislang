use super::*;

pub(super) fn outline_effects(
    body: Vec<NirStmt>,
    scope: &mut Scope,
    names: &mut BTreeSet<String>,
    helpers: &mut Vec<NirFunction>,
    guarded: &mut BTreeSet<String>,
    catalog: &ScalarHelpers,
    mutations: &MutationScope,
    structs: &mut Vec<NirStructDef>,
) -> Vec<NirStmt> {
    let mut bindings = scope.keys().cloned().collect::<BTreeSet<_>>();
    collect_bindings(&body, &mut bindings);
    let mut outlined = Vec::new();
    for stmt in body {
        match stmt {
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                let mut carries = Vec::new();
                for arm in [&then_body, &else_body] {
                    validate_effects(
                        arm,
                        &mut scope.clone(),
                        &mut BTreeSet::new(),
                        catalog,
                        mutations,
                        &mut carries,
                    )
                    .expect("validated branch state");
                }
                carries.retain(|name| scope.contains_key(name));
                let aggregate = (carries.len() > 1)
                    .then(|| scalar_carries::state_type(&carries, names, structs));
                let returned = scalar_carries::value(&carries, aggregate.as_ref());
                // Snapshot once: the selected arm may mutate a buffer read by the condition.
                let predicate = fresh_name("__nuis_buffer_condition", &mut bindings);
                outlined.push(NirStmt::Let {
                    name: predicate.clone(),
                    ty: Some(scalar_type("bool")),
                    value: condition,
                });
                scope.insert(predicate.clone(), scalar_type("bool"));
                for (selected, arm) in [(true, then_body), (false, else_body)] {
                    if arm.is_empty() {
                        continue;
                    }
                    let mut inputs = BTreeSet::from([predicate.clone()]);
                    inputs.extend(carries.iter().cloned());
                    validate_effects(
                        &arm,
                        &mut scope.clone(),
                        &mut inputs,
                        catalog,
                        mutations,
                        &mut Vec::new(),
                    )
                    .expect("validated buffer branch");
                    let params = captured_params(inputs, scope);
                    let name = fresh_name("__nuis_buffer_branch", names);
                    let mut arm_body = vec![NirStmt::If {
                        condition: NirExpr::Binary {
                            op: NirBinaryOp::Ne,
                            lhs: Box::new(NirExpr::Var(predicate.clone())),
                            rhs: Box::new(NirExpr::Bool(selected)),
                        },
                        then_body: vec![NirStmt::Return(Some(returned.clone()))],
                        else_body: vec![],
                    }];
                    arm_body.extend(outline_effects(
                        arm,
                        &mut scope.clone(),
                        names,
                        helpers,
                        guarded,
                        catalog,
                        mutations,
                        structs,
                    ));
                    arm_body.push(NirStmt::Return(Some(returned.clone())));
                    // Only existing values cross this boundary, never branch-local reads/math.
                    let call = NirExpr::Call {
                        callee: name.clone(),
                        args: params
                            .iter()
                            .map(|param| NirExpr::Var(param.name.clone()))
                            .collect(),
                    };
                    let mut function = helper(name.clone(), params, arm_body);
                    if let Some(ty) = &aggregate {
                        let temporary = fresh_name("__nuis_branch_state", &mut bindings);
                        outlined.extend(scalar_carries::projected_call(
                            temporary, ty, &carries, call,
                        ));
                        function.return_type = Some(ty.clone());
                    } else if let Some(name) = carries.first() {
                        outlined.push(NirStmt::Let {
                            name: name.clone(),
                            ty: Some(scalar_type("i64")),
                            value: call,
                        });
                    } else {
                        outlined.push(NirStmt::Expr(call));
                    }
                    guarded.insert(name.clone());
                    helpers.push(function);
                }
            }
            NirStmt::While {
                condition,
                mut body,
            } => {
                let plan =
                    buffer_loop_params(&condition, &body, scope, catalog, &mutations.protected)
                        .expect("validated nested loop");
                outline_loop(
                    &mut body, scope, names, helpers, guarded, catalog, structs, plan,
                );
                outlined.push(NirStmt::While { condition, body });
            }
            NirStmt::Let {
                ref name,
                ref value,
                ..
            } => {
                scope.insert(
                    name.clone(),
                    infer_local(value, scope, catalog).expect("validated local"),
                );
                outlined.push(stmt);
            }
            _ => outlined.push(stmt),
        }
    }
    outlined
}

pub(super) fn fresh_name(prefix: &str, names: &mut BTreeSet<String>) -> String {
    for suffix in 0.. {
        let name = format!("{prefix}_{suffix}");
        if names.insert(name.clone()) {
            return name;
        }
    }
    unreachable!("unbounded helper name space")
}

pub(super) fn collect_bindings(body: &[NirStmt], names: &mut BTreeSet<String>) {
    for stmt in body {
        match stmt {
            NirStmt::Let { name, .. } | NirStmt::Const { name, .. } => {
                names.insert(name.clone());
            }
            NirStmt::If {
                then_body,
                else_body,
                ..
            } => {
                collect_bindings(then_body, names);
                collect_bindings(else_body, names);
            }
            NirStmt::While { body, .. } => collect_bindings(body, names),
            _ => {}
        }
    }
}
