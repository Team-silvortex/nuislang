use super::*;

const RETURN: &str = "return helper(checked(divisor), ARGUMENT, 20 / divisor, flag);";

fn assert_return(body: &[NirStmt], original: &NirExpr) {
    let NirExpr::Call {
        args: originals, ..
    } = original
    else {
        panic!()
    };
    assert_eq!(body.len(), 5);
    let mut names = Vec::new();
    for ((stmt, value), kind) in body[..4]
        .iter()
        .zip(originals)
        .zip(["i64", "State", "i64", "bool"])
    {
        let NirStmt::Let {
            name,
            ty,
            value: stored,
        } = stmt
        else {
            panic!()
        };
        assert_eq!(stored, value);
        assert_eq!(ty.as_ref(), Some(&scalar_type(kind)));
        names.push(name);
    }
    let NirStmt::Return(Some(NirExpr::Call { callee, args })) = &body[4] else {
        panic!()
    };
    assert_eq!(callee, "helper");
    assert_eq!(args.len(), 5);
    assert_eq!(args[0], NirExpr::Var(names[0].clone()));
    assert_eq!(access(&args[1]).unwrap(), [names[1].as_str(), "a", "x"]);
    assert_eq!(access(&args[2]).unwrap(), [names[1].as_str(), "b", "y"]);
    assert_eq!(args[3], NirExpr::Var(names[2].clone()));
    assert_eq!(args[4], NirExpr::Var(names[3].clone()));
}

#[test]
fn caller_spills_return_roots_preserve_exact_operands_branch_exit_and_mixed_callers() {
    for argument in ["produce(state, divisor)", CONSTRUCTOR] {
        for branch in [false, true] {
            for reverse in [false, true] {
                let body = if branch {
                    format!("if flag {{ {RETURN} }} else {{ {RETURN} }}")
                } else {
                    RETURN.into()
                };
                let mut module = module(argument, &body);
                let original = function(&module, "entry").clone();
                let mut bound = function(&super::module(argument, BODY), "entry").clone();
                bound.name = "bound".into();
                module.functions.push(bound);
                if reverse {
                    module.functions.reverse();
                    module.structs.reverse();
                }
                assert!(run(&mut module));
                crate::nir_verify::verify_nir_module(&module).unwrap();
                assert_eq!(function(&module, "helper").params.len(), 5);
                assert_eq!(function(&module, "entry").return_type, original.return_type);
                let body = &function(&module, "entry").body;
                if branch {
                    let NirStmt::If {
                        condition,
                        then_body,
                        else_body,
                    } = &body[0]
                    else {
                        panic!()
                    };
                    let NirStmt::If {
                        condition: before,
                        then_body: old_then,
                        else_body: old_else,
                    } = &original.body[0]
                    else {
                        panic!()
                    };
                    assert_eq!(condition, before);
                    for (after, before) in [(then_body, old_then), (else_body, old_else)] {
                        let NirStmt::Return(Some(value)) = &before[0] else {
                            panic!()
                        };
                        assert_return(after, value);
                    }
                    let names = |body: &[NirStmt]| {
                        body.iter()
                            .filter_map(|stmt| match stmt {
                                NirStmt::Let { name, .. } => Some(name.clone()),
                                _ => None,
                            })
                            .collect::<BTreeSet<_>>()
                    };
                    assert!(names(then_body).is_disjoint(&names(else_body)));
                } else {
                    let NirStmt::Return(Some(value)) = &original.body[0] else {
                        panic!()
                    };
                    assert_return(body, value);
                }
                assert_eq!(function(&module, "bound").body.len(), 6);
                let before = module.clone();
                assert!(!run(&mut module));
                assert_eq!(module, before);
            }
        }
    }
    let mut module = module("produce(state, divisor)", RETURN);
    let helper = function_mut(&mut module, "helper");
    helper.return_type = Some(scalar_type("Pair"));
    helper.body = vec![NirStmt::Return(Some(NirExpr::FieldAccess {
        base: Box::new(NirExpr::Var("state".into())),
        field: "a".into(),
    }))];
    function_mut(&mut module, "entry").return_type = Some(scalar_type("Pair"));
    let mut main = function(&module, "entry").clone();
    main.name = "main".into();
    main.params.clear();
    main.return_type = Some(scalar_type("i64"));
    main.body = vec![
        NirStmt::Let {
            name: "part".into(),
            ty: Some(scalar_type("Pair")),
            value: NirExpr::Call {
                callee: "entry".into(),
                args: vec![
                    NirExpr::StructLiteral {
                        type_name: "State".into(),
                        type_args: vec![],
                        fields: vec![
                            ("a".into(), pair(11, 13)),
                            ("b".into(), pair(17, 19)),
                            ("unused".into(), NirExpr::Int(23)),
                        ],
                    },
                    NirExpr::Int(2),
                    NirExpr::Bool(true),
                ],
            },
        },
        NirStmt::Return(Some(NirExpr::FieldAccess {
            base: Box::new(NirExpr::Var("part".into())),
            field: "y".into(),
        })),
    ];
    module.functions.push(main);
    crate::nir_verify::verify_nir_module(&module).unwrap();
    assert_eq!(execute(&module), Ok(yir_core::Value::Int(13)));
    let original = function(&module, "entry").body.clone();
    assert!(run(&mut module));
    crate::nir_verify::verify_nir_module(&module).unwrap();
    assert_eq!(execute(&module), Ok(yir_core::Value::Int(13)));
    assert_eq!(
        function(&module, "helper").return_type,
        Some(scalar_type("Pair"))
    );
    assert_eq!(
        function(&module, "entry").return_type,
        Some(scalar_type("Pair"))
    );
    assert_eq!(
        function(&module, "helper").params[1].ty,
        scalar_type("Pair")
    );
    let NirStmt::Return(Some(NirExpr::Call { args: before, .. })) = &original[0] else {
        panic!()
    };
    let body = &function(&module, "entry").body;
    for (stmt, before) in body[..4].iter().zip(before) {
        assert!(matches!(stmt, NirStmt::Let { value, .. } if value == before));
    }
    let NirStmt::Return(Some(NirExpr::Call { args, .. })) = &body[4] else {
        panic!()
    };
    let NirStmt::Let { name, .. } = &body[1] else {
        panic!()
    };
    assert_eq!(args.len(), 4);
    assert_eq!(access(&args[1]).unwrap(), [name.as_str(), "a"]);
}

#[test]
fn caller_spills_return_roots_keep_atomic_vetoes_and_unproven_versions_unchanged() {
    for bad_body in [
        "return 1 + helper(checked(divisor), ARGUMENT, 20 / divisor, flag);",
        "if helper(checked(divisor), ARGUMENT, 20 / divisor, flag) > 0 { return 1; } return 0;",
        "while flag { if divisor > 0 { return helper(checked(divisor), ARGUMENT, 20 / divisor, flag); } } return 0;",
        "let state = state; return helper(checked(divisor), ARGUMENT, 20 / divisor, flag);",
    ] {
        for reverse in [false, true] {
            let mut module = module("produce(state, divisor)", RETURN);
            let mut bad = function(&super::module("produce(state, divisor)", bad_body), "entry").clone();
            bad.name = "unproven".into();
            module.functions.push(bad);
            if reverse { module.functions.reverse(); }
            let before = module.clone();
            assert!(!run(&mut module), "{bad_body}");
            assert_eq!(module, before);
        }
    }
    for mutation in 0..4 {
        let mut module = module("produce(state, divisor)", RETURN);
        match mutation {
            0 => function_mut(&mut module, "entry").is_async = true,
            1 => function_mut(&mut module, "entry").params[0].ty.is_ref = true,
            2 => function_mut(&mut module, "produce")
                .body
                .insert(0, NirStmt::Print(NirExpr::Int(1))),
            3 => {
                let NirStmt::Return(Some(NirExpr::Call { args, .. })) =
                    &mut function_mut(&mut module, "entry").body[0]
                else {
                    panic!()
                };
                args[3] = NirExpr::Int(1);
            }
            _ => unreachable!(),
        }
        let before = module.clone();
        assert!(!run(&mut module));
        assert_eq!(module, before);
    }
    let module = module("produce(state, divisor)", RETURN);
    let layouts = control_values::TypedLayouts::collect(&module);
    let mut catalog = scalar_helpers::collect_capture_values(&module, &layouts);
    catalog.remove("helper");
    let plan = super::super::super::plan(function(&module, "helper"), None, &layouts).unwrap();
    let caller = function(&module, "entry");
    for budget in [1, 2, 16, 64] {
        assert!(prepare_with_budget(
            caller,
            "helper",
            &plan,
            &layouts,
            &catalog,
            &BTreeSet::new(),
            budget,
        )
        .is_none());
    }
    assert!(prepare(
        caller,
        "helper",
        &plan,
        &layouts,
        &catalog,
        &BTreeSet::new()
    )
    .is_some());
    assert!(prepare(
        caller,
        "helper",
        &plan,
        &layouts,
        &catalog,
        &BTreeSet::from(["state".into()]),
    )
    .is_none());
}

fn execute(module: &NirModule) -> Result<yir_core::Value, ()> {
    let mut yir = crate::lowering::lower_nir_to_yir_builtin_cpu(module).unwrap();
    yir.nodes.reverse();
    yir.functions.reverse();
    for function in &mut yir.functions {
        function.body_nodes.reverse();
    }
    let trace = yir_runtime_host::execute_module_source_with_registry(
        &crate::render::render_yir(&yir),
        &yir_verify::default_registry(),
    )
    .map_err(|_| ())?;
    let main = yir
        .functions
        .iter()
        .find(|f| f.role == yir_core::YirFunctionRole::Entry)
        .unwrap();
    Ok(trace.values[&main.result.as_ref().unwrap().node].clone())
}

#[test]
fn caller_spills_return_roots_preserve_guarded_unused_checks_with_an_independent_oracle() {
    for first in 0..4 {
        for unused in 0..4 {
            for guard in 0..3 {
                let mut module = module(
                    "produce(state, divisor)",
                    &format!("if divisor == 0 {{ return 0; }} {RETURN}"),
                );
                let entry = function_mut(&mut module, "entry");
                entry.params.extend([
                    NirParam {
                        name: "unused".into(),
                        ty: scalar_type("i64"),
                    },
                    NirParam {
                        name: "guard".into(),
                        ty: scalar_type("i64"),
                    },
                ]);
                let NirStmt::If { condition, .. } = &mut entry.body[0] else {
                    panic!()
                };
                let NirExpr::Binary { lhs, .. } = condition else {
                    panic!()
                };
                *lhs = Box::new(NirExpr::Var("guard".into()));
                let NirStmt::Return(Some(NirExpr::Call { args, .. })) = &mut entry.body[1] else {
                    panic!()
                };
                let NirExpr::Call { args, .. } = &mut args[1] else {
                    panic!()
                };
                args[1] = NirExpr::Var("unused".into());
                let mut main = entry.clone();
                main.name = "main".into();
                main.params.clear();
                main.body = vec![NirStmt::Return(Some(NirExpr::Call {
                    callee: "entry".into(),
                    args: vec![
                        NirExpr::StructLiteral {
                            type_name: "State".into(),
                            type_args: vec![],
                            fields: vec![
                                ("a".into(), pair(11, 13)),
                                ("b".into(), pair(17, 19)),
                                ("unused".into(), NirExpr::Int(23)),
                            ],
                        },
                        NirExpr::Int(first),
                        NirExpr::Bool(guard == 1),
                        NirExpr::Int(unused),
                        NirExpr::Int(guard),
                    ],
                }))];
                module.functions.push(main);
                let expected = if guard == 0 {
                    Ok(yir_core::Value::Int(0))
                } else if first == 0 || unused == 0 {
                    Err(())
                } else {
                    Ok(yir_core::Value::Int(
                        10 / first + 20 / first + if guard == 1 { 11 } else { 19 },
                    ))
                };
                assert_eq!(
                    execute(&module),
                    expected,
                    "before {first}/{unused}/{guard}"
                );
                let earlier_exit = function(&module, "entry").body[0].clone();
                assert!(run(&mut module));
                crate::nir_verify::verify_nir_module(&module).unwrap();
                assert_eq!(execute(&module), expected, "after {first}/{unused}/{guard}");
                assert_eq!(function(&module, "entry").body[0], earlier_exit);
                assert!(matches!(
                    function(&module, "entry").body.last(),
                    Some(NirStmt::Return(_))
                ));
            }
        }
    }
}
