use super::*;

fn module() -> NirModule {
    crate::frontend::parse_nuis_module(include_str!(
        "../../../tests/native_application_bridge/computed_scalar_join.ns"
    ))
    .unwrap()
}

#[test]
fn computed_scalar_join_requires_both_completed_dependencies_and_is_idempotent() {
    let mut module = module();
    let layouts = control_values::TypedLayouts::collect(&module);
    let pure = scalar_helpers::collect_capture_values(&module, &layouts);
    let mut reordered = module.clone();
    reordered.functions.reverse();
    let reordered_pure = scalar_helpers::collect_capture_values(&reordered, &layouts);
    let mut reordered_names = reordered.functions.iter().map(|f| f.name.clone()).collect();
    assert_eq!(
        conditional_values::outline_declarations(
            &mut reordered,
            &reordered_pure,
            &layouts,
            &mut reordered_names
        )
        .len(),
        1
    );
    let f = module
        .functions
        .iter()
        .find(|f| f.name == "packet")
        .unwrap();
    let scope = f
        .params
        .iter()
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect::<Scope>();
    assert!(guarded_declaration(&f.body[1], &scope, &f.body[2..], &pure, &layouts).is_none());
    assert!(computed_declaration(&f.body[1], &scope, &f.body[2..], &pure, &layouts).is_some());
    for dependency in ["decide", "quotient"] {
        let mut denied = pure.clone();
        denied.remove(dependency);
        assert!(
            computed_declaration(&f.body[1], &scope, &f.body[2..], &denied, &layouts).is_none(),
            "{dependency}"
        );
    }
    let mut effectful = module.clone();
    effectful
        .functions
        .iter_mut()
        .find(|f| f.name == "decide")
        .unwrap()
        .body
        .insert(0, NirStmt::Print(NirExpr::Int(84)));
    let effectful_pure = scalar_helpers::collect_capture_values(&effectful, &layouts);
    assert!(!effectful_pure.contains_key("decide"));
    assert!(
        computed_declaration(&f.body[1], &scope, &f.body[2..], &effectful_pure, &layouts).is_none()
    );
    let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
    let generated =
        conditional_values::outline_declarations(&mut module, &pure, &layouts, &mut names);
    assert_eq!(generated.len(), 1);
    let carries = control_values::layouts(&module);
    assert!(scalar_helpers::collect_iteration_calls(&module, &carries).contains_key("packet"));
    assert!(!scalar_helpers::collect_capture_values(&module, &layouts).contains_key("packet"));
    let before = module.clone();
    assert!(
        conditional_values::outline_declarations(&mut module, &pure, &layouts, &mut names)
            .is_empty()
    );
    assert_eq!(module, before);
    for dependency in ["decide", "quotient"] {
        let mut denied = module.clone();
        denied.functions.retain(|f| f.name != dependency);
        assert!(!scalar_helpers::collect_iteration_calls(&denied, &carries).contains_key("packet"));
    }
}

#[test]
fn computed_scalar_join_shares_comparison_kinds_without_widening_result_types() {
    for kind in ["bool", "i32", "i64", "f32", "f64"] {
        let source = format!("mod cpu Main {{ fn pick(a: {kind}, b: {kind}) -> i64 {{ let value: i64 = if a != b {{ 10 }} else {{ 20 }}; return value; }} }}");
        let module = crate::frontend::parse_nuis_module(&source).unwrap();
        let layouts = control_values::TypedLayouts::collect(&module);
        let pure = scalar_helpers::collect_capture_values(&module, &layouts);
        let f = &module.functions[0];
        let scope = f
            .params
            .iter()
            .map(|p| (p.name.clone(), p.ty.clone()))
            .collect();
        assert!(
            computed_declaration(&f.body[0], &scope, &f.body[1..], &pure, &layouts).is_some(),
            "{kind}"
        );
        let mut stmt = f.body[0].clone();
        let NirStmt::If {
            then_body,
            else_body,
            ..
        } = &mut stmt
        else {
            panic!()
        };
        for body in [then_body, else_body] {
            let NirStmt::Let { ty, .. } = &mut body[0] else {
                panic!()
            };
            *ty = Some(scalar_type("f64"));
        }
        assert!(computed_declaration(&stmt, &scope, &f.body[1..], &pure, &layouts).is_none());
    }
}

#[test]
fn computed_scalar_join_evaluates_only_one_original_predicate_as_argument_zero() {
    let mut module = module();
    let layouts = control_values::TypedLayouts::collect(&module);
    let pure = scalar_helpers::collect_capture_values(&module, &layouts);
    let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
    let generated =
        conditional_values::outline_declarations(&mut module, &pure, &layouts, &mut names);
    let f = module
        .functions
        .iter()
        .find(|f| f.name == "packet")
        .unwrap();
    let NirStmt::Let {
        value: NirExpr::Call { callee, args },
        ..
    } = &f.body[1]
    else {
        panic!()
    };
    assert!(generated.contains(callee));
    assert!(
        matches!(&args[0], NirExpr::Call { callee, args } if callee == "decide" && args.len() == 2)
    );
    assert!(args[1..].iter().all(|arg| matches!(arg, NirExpr::Var(_))));
    let selected = module.functions.iter().find(|f| &f.name == callee).unwrap();
    let [NirStmt::If {
        condition: NirExpr::Var(_),
        then_body,
        else_body,
    }] = selected.body.as_slice()
    else {
        panic!()
    };
    assert!(
        matches!(then_body.as_slice(), [NirStmt::Return(Some(NirExpr::Call { callee, .. }))] if callee == "quotient")
    );
    assert!(matches!(
        else_body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Binary {
            op: NirBinaryOp::Rem,
            ..
        }))]
    ));
}

#[test]
fn computed_scalar_join_keeps_effects_logic_types_scope_and_shared_budget_fail_closed() {
    let module = module();
    let layouts = control_values::TypedLayouts::collect(&module);
    let pure = scalar_helpers::collect_capture_values(&module, &layouts);
    let f = module
        .functions
        .iter()
        .find(|f| f.name == "packet")
        .unwrap();
    let scope = f
        .params
        .iter()
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect::<Scope>();
    for mutation in 0..8 {
        let mut stmt = f.body[1].clone();
        let mut scope = scope.clone();
        let NirStmt::If {
            condition,
            then_body,
            else_body,
        } = &mut stmt
        else {
            panic!()
        };
        match mutation {
            0 => {
                scope.insert("selected".into(), scalar_type("i64"));
            }
            1 => *condition = NirExpr::Int(1),
            2 => {
                *condition = NirExpr::Call {
                    callee: "packet".into(),
                    args: vec![],
                }
            }
            3 => {
                *condition = NirExpr::Binary {
                    op: NirBinaryOp::And,
                    lhs: Box::new(NirExpr::Bool(true)),
                    rhs: Box::new(condition.clone()),
                }
            }
            4 => then_body.insert(0, NirStmt::Print(NirExpr::Int(88))),
            5 => else_body.clear(),
            6 => {
                for _ in 0..64 {
                    *condition = NirExpr::Binary {
                        op: NirBinaryOp::Eq,
                        lhs: Box::new(condition.clone()),
                        rhs: Box::new(NirExpr::Bool(true)),
                    };
                }
            }
            7 => {
                let mut tree = NirExpr::Int(1);
                for _ in 0..10 {
                    tree = NirExpr::Binary {
                        op: NirBinaryOp::Add,
                        lhs: Box::new(tree.clone()),
                        rhs: Box::new(tree),
                    };
                }
                *condition = NirExpr::Binary {
                    op: NirBinaryOp::Eq,
                    lhs: Box::new(tree.clone()),
                    rhs: Box::new(tree.clone()),
                };
                let NirStmt::Let { value, .. } = &mut then_body[0] else {
                    panic!()
                };
                *value = tree;
            }
            _ => unreachable!(),
        }
        assert!(
            computed_declaration(&stmt, &scope, &f.body[2..], &pure, &layouts).is_none(),
            "{mutation}"
        );
    }
}
