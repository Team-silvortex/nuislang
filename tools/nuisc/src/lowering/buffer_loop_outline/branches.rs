use super::*;

pub(super) fn outline_branches(
    body: Vec<NirStmt>,
    scope: &mut Scope,
    names: &mut BTreeSet<String>,
    helpers: &mut Vec<NirFunction>,
    guarded: &mut BTreeSet<String>,
    catalog: &ScalarHelpers,
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
                    validate_effects(&arm, &mut scope.clone(), &mut inputs, catalog)
                        .expect("validated buffer branch");
                    let params = captured_params(inputs, scope);
                    let name = fresh_name("__nuis_buffer_branch", names);
                    let mut arm_body = vec![NirStmt::If {
                        condition: NirExpr::Binary {
                            op: NirBinaryOp::Ne,
                            lhs: Box::new(NirExpr::Var(predicate.clone())),
                            rhs: Box::new(NirExpr::Bool(selected)),
                        },
                        then_body: vec![NirStmt::Return(Some(NirExpr::Int(0)))],
                        else_body: vec![],
                    }];
                    arm_body.extend(outline_branches(
                        arm,
                        &mut scope.clone(),
                        names,
                        helpers,
                        guarded,
                        catalog,
                    ));
                    arm_body.push(NirStmt::Return(Some(NirExpr::Int(0))));
                    // Only existing values cross this boundary, never branch-local reads/math.
                    outlined.push(NirStmt::Expr(NirExpr::Call {
                        callee: name.clone(),
                        args: params
                            .iter()
                            .map(|param| NirExpr::Var(param.name.clone()))
                            .collect(),
                    }));
                    guarded.insert(name.clone());
                    helpers.push(helper(name, params, arm_body));
                }
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

fn fresh_name(prefix: &str, names: &mut BTreeSet<String>) -> String {
    for suffix in 0.. {
        let name = format!("{prefix}_{suffix}");
        if names.insert(name.clone()) {
            return name;
        }
    }
    unreachable!("unbounded helper name space")
}

fn collect_bindings(body: &[NirStmt], names: &mut BTreeSet<String>) {
    for stmt in body {
        match stmt {
            NirStmt::Let { name, .. } => {
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
            _ => {}
        }
    }
}
