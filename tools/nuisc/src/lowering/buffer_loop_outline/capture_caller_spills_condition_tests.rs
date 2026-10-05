use super::*;

const CALL: &str = "helper(checked(divisor), ARGUMENT, 20 / divisor, flag)";
pub(super) const BODY: &str = "if guard == 0 { return 0; }
    let result = 0;
    if helper(checked(divisor), ARGUMENT, 20 / divisor, flag) {
        let result = 22 / yes;
    } else { let result = 38 / no; }
    return result;";

pub(super) fn source(argument: &str, body: &str) -> String {
    format!(
        "mod cpu Main {{ {RECORDS}
        @noinline fn checked(value: i64) -> i64 {{ return 10 / value; }}
        @noinline fn produce(state: State, divisor: i64) -> State {{ return {CONSTRUCTOR}; }}
        @noinline fn helper(before: i64, state: State, after: i64, flag: bool) -> bool {{
            if flag {{ return before + state.a.x + after > 25; }}
            return before + state.b.y + after > 35;
        }} @noinline fn entry(state: State, divisor: i64, unused: i64, guard: i64,
            flag: bool, yes: i64, no: i64) -> i64 {{ {} }} }}",
        body.replace("ARGUMENT", argument)
    )
}

fn fixture(argument: &str, body: &str) -> NirModule {
    crate::frontend::parse_nuis_module(&source(argument, body)).unwrap()
}

fn assert_condition(body: &[NirStmt], original: &NirStmt) {
    let NirStmt::If {
        condition: NirExpr::Call { args: before, .. },
        then_body: old_then,
        else_body: old_else,
    } = original
    else {
        panic!()
    };
    assert_eq!(body.len(), 5);
    let mut names = Vec::new();
    for ((stmt, before), kind) in body[..4]
        .iter()
        .zip(before)
        .zip(["i64", "State", "i64", "bool"])
    {
        let NirStmt::Let { name, ty, value } = stmt else {
            panic!()
        };
        assert_eq!(value, before);
        assert_eq!(ty.as_ref(), Some(&scalar_type(kind)));
        names.push(name);
    }
    let NirStmt::If {
        condition: NirExpr::Call { callee, args },
        then_body,
        else_body,
    } = &body[4]
    else {
        panic!()
    };
    assert_eq!(callee, "helper");
    assert_eq!(args.len(), 5);
    assert_eq!(args[0], NirExpr::Var(names[0].clone()));
    assert_eq!(access(&args[1]).unwrap(), [names[1].as_str(), "a", "x"]);
    assert_eq!(access(&args[2]).unwrap(), [names[1].as_str(), "b", "y"]);
    assert_eq!(args[3], NirExpr::Var(names[2].clone()));
    assert_eq!(args[4], NirExpr::Var(names[3].clone()));
    assert_eq!(then_body, old_then);
    assert_eq!(else_body, old_else);
}

#[test]
fn caller_spills_condition_roots_preserve_full_order_branches_and_mixed_callers() {
    for argument in ["produce(state, unused)", CONSTRUCTOR] {
        for nested in [false, true] {
            for reverse in [false, true] {
                let body = if nested {
                    format!("if flag {{ {BODY} }} else {{ return 0; }}")
                } else {
                    BODY.into()
                };
                let mut module = fixture(argument, &body);
                let original = function(&module, "entry").body.clone();
                for name in ["bound", "returned"] {
                    let mut caller = function(
                        &fixture(
                            argument,
                            &format!("let result = {CALL}; if result {{ return 11; }} return 19;"),
                        ),
                        "entry",
                    )
                    .clone();
                    if name == "returned" {
                        let NirStmt::Let { value, .. } = &caller.body[0] else {
                            panic!()
                        };
                        let value = value.clone();
                        caller.return_type = Some(scalar_type("bool"));
                        caller.body = vec![NirStmt::Return(Some(value))];
                    }
                    caller.name = name.into();
                    module.functions.push(caller);
                }
                if reverse {
                    module.functions.reverse();
                    module.structs.reverse();
                }
                crate::nir_verify::verify_nir_module(&module).unwrap();
                assert!(run(&mut module));
                crate::nir_verify::verify_nir_module(&module).unwrap();
                assert_eq!(function(&module, "helper").params.len(), 5);
                let mut after = function(&module, "entry").body.as_slice();
                let mut before = original.as_slice();
                if nested {
                    let NirStmt::If {
                        condition,
                        then_body,
                        else_body,
                    } = &after[0]
                    else {
                        panic!()
                    };
                    let NirStmt::If {
                        condition: old_condition,
                        then_body: old_then,
                        else_body: old_else,
                    } = &before[0]
                    else {
                        panic!()
                    };
                    assert_eq!(condition, old_condition);
                    assert_eq!(else_body, old_else);
                    after = then_body;
                    before = old_then;
                }
                assert_eq!(after[0], before[0]);
                assert_eq!(after[1], before[1]);
                assert_eq!(after.last(), before.last());
                assert_condition(&after[2..after.len() - 1], &before[2]);
                assert!(matches!(
                    function(&module, "returned").body.last(),
                    Some(NirStmt::Return(Some(NirExpr::Call { .. })))
                ));
                let before = module.clone();
                assert!(!run(&mut module));
                assert_eq!(module, before);
            }
        }
    }
}

#[test]
fn caller_spills_condition_roots_keep_nested_loop_kind_and_atomic_veto_boundaries() {
    for body in [
        "if (flag == true) && helper(checked(divisor), ARGUMENT, 20 / divisor, flag) { return 11; } return 19;",
        "if (flag == true) || helper(checked(divisor), ARGUMENT, 20 / divisor, flag) { return 11; } return 19;",
        "if helper(checked(divisor), ARGUMENT, 20 / divisor, flag) == flag { return 11; } return 19;",
        "while flag { if helper(checked(divisor), ARGUMENT, 20 / divisor, flag) { return 11; } } return 19;",
        "while helper(checked(divisor), ARGUMENT, 20 / divisor, flag) { return 11; } return 19;",
        "let state = state; if helper(checked(divisor), ARGUMENT, 20 / divisor, flag) { return 11; } return 19;",
    ] {
        for reverse in [false, true] {
            let mut module = fixture("produce(state, unused)", BODY);
            let mut bad = function(&fixture("produce(state, unused)", body), "entry").clone();
            bad.name = "unproven".into(); module.functions.push(bad);
            if reverse { module.functions.reverse(); }
            let before = module.clone();
            assert!(!run(&mut module), "{body}");
            assert_eq!(module, before);
        }
    }
    for mutation in 0..7 {
        let mut module = fixture("produce(state, unused)", BODY);
        match mutation {
            0 => function_mut(&mut module, "helper").return_type = Some(scalar_type("i64")),
            1 => {
                function_mut(&mut module, "helper")
                    .return_type
                    .as_mut()
                    .unwrap()
                    .is_ref = true
            }
            2 => {
                function_mut(&mut module, "helper")
                    .return_type
                    .as_mut()
                    .unwrap()
                    .is_optional = true
            }
            3 => function_mut(&mut module, "entry").params[0].ty.is_ref = true,
            4 => function_mut(&mut module, "produce")
                .body
                .insert(0, NirStmt::Print(NirExpr::Int(1))),
            5 => function_mut(&mut module, "entry").is_async = true,
            6 => {
                let NirStmt::If {
                    condition: NirExpr::Call { args, .. },
                    ..
                } = &mut function_mut(&mut module, "entry").body[2]
                else {
                    panic!()
                };
                args[3] = NirExpr::Int(1);
            }
            _ => unreachable!(),
        }
        let before = module.clone();
        assert!(!run(&mut module), "{mutation}");
        assert_eq!(module, before);
    }
    let mut module = fixture("produce(state, unused)", BODY);
    function_mut(&mut module, "entry").params.push(NirParam {
        name: "__nuis_caller_operand_0".into(),
        ty: scalar_type("i64"),
    });
    let layouts = control_values::TypedLayouts::collect(&module);
    let mut catalog = scalar_helpers::collect_capture_values(&module, &layouts);
    catalog.remove("helper");
    let plan = super::super::super::plan(function(&module, "helper"), None, &layouts).unwrap();
    let caller = function(&module, "entry");
    assert!(plan.predicate_result);
    for budget in [1, 2, 16, 64] {
        assert!(prepare_with_budget(
            caller,
            "helper",
            &plan,
            &layouts,
            &catalog,
            &BTreeSet::new(),
            budget
        )
        .is_none());
    }
    assert!(prepare(
        caller,
        "helper",
        &plan,
        &layouts,
        &catalog,
        &BTreeSet::from(["state".into()])
    )
    .is_none());
    let mut deep = caller.clone();
    for _ in 0..70 {
        deep.body = vec![NirStmt::If {
            condition: NirExpr::Var("flag".into()),
            then_body: deep.body,
            else_body: vec![],
        }];
    }
    assert!(prepare(&deep, "helper", &plan, &layouts, &catalog, &BTreeSet::new()).is_none());
    assert!(run(&mut module));
    crate::nir_verify::verify_nir_module(&module).unwrap();
    assert!(
        matches!(&function(&module, "entry").body[2], NirStmt::Let { name, .. } if name != "__nuis_caller_operand_0")
    );
}

pub(super) fn execute(module: &NirModule) -> Result<yir_core::Value, ()> {
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
fn caller_spills_condition_roots_preserve_early_exit_and_selected_checks_with_an_independent_oracle(
) {
    for first in 0..4 {
        for unused in 0..4 {
            for guard in 0..3 {
                for branches in 0..4 {
                    let yes = if branches & 1 == 0 { 2 } else { 0 };
                    let no = if branches & 2 == 0 { 2 } else { 0 };
                    let mut module = fixture("produce(state, unused)", BODY);
                    let mut main = function(&module, "entry").clone();
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
                            NirExpr::Int(unused),
                            NirExpr::Int(guard),
                            NirExpr::Bool(guard == 1),
                            NirExpr::Int(yes),
                            NirExpr::Int(no),
                        ],
                    }))];
                    module.functions.push(main);
                    let expected = if guard == 0 {
                        Ok(yir_core::Value::Int(0))
                    } else if first == 0 || unused == 0 {
                        Err(())
                    } else {
                        let selected = 10 / first + 20 / first + if guard == 1 { 11 } else { 19 }
                            > if guard == 1 { 25 } else { 35 };
                        let divisor = if selected { yes } else { no };
                        if divisor == 0 {
                            Err(())
                        } else {
                            Ok(yir_core::Value::Int(if selected {
                                22 / divisor
                            } else {
                                38 / divisor
                            }))
                        }
                    };
                    assert_eq!(
                        execute(&module),
                        expected,
                        "before {first}/{unused}/{guard}/{branches}"
                    );
                    let original = function(&module, "entry").body.clone();
                    assert!(run(&mut module));
                    crate::nir_verify::verify_nir_module(&module).unwrap();
                    assert_eq!(
                        execute(&module),
                        expected,
                        "after {first}/{unused}/{guard}/{branches}"
                    );
                    assert_eq!(function(&module, "entry").body[0], original[0]);
                    let body = &function(&module, "entry").body;
                    assert_eq!(body[1], original[1]);
                    assert_eq!(body.last(), original.last());
                    assert_condition(&body[2..body.len() - 1], &original[2]);
                }
            }
        }
    }
}
