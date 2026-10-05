use super::*;

pub(super) fn source(gate: &str, op: &str, argument: &str) -> String {
    source_body(gate, op, argument, super::conditions::BODY)
}

fn source_body(gate: &str, op: &str, argument: &str, body: &str) -> String {
    super::conditions::source(
        argument,
        &body.replace("if helper(", &format!("if {gate} {op} helper(")),
    )
    .replace(
        "flag: bool, yes: i64, no: i64",
        "flag: bool, yes: i64, no: i64, gate: bool",
    )
}

fn fixture(gate: &str, op: &str, argument: &str, body: &str) -> NirModule {
    let text = if body == super::conditions::BODY {
        source(gate, op, argument)
    } else {
        source_body(gate, op, argument, body)
    };
    crate::frontend::parse_nuis_module(&text).unwrap()
}

fn assert_gated_body(before: &[NirStmt], after: &[NirStmt], disjunction: bool) {
    assert_eq!(after.len(), before.len() + 2);
    assert_eq!(after[0], before[0]);
    assert_eq!(after[1], before[1]);
    assert_eq!(after.last(), before.last());
    let NirStmt::If {
        condition: NirExpr::Binary { lhs, rhs, .. },
        then_body: old_then,
        else_body: old_else,
    } = &before[2]
    else {
        panic!()
    };
    let NirExpr::Call { args: old_args, .. } = rhs.as_ref() else {
        panic!()
    };
    let NirStmt::Let { name, ty, value } = &after[2] else {
        panic!()
    };
    assert_eq!(ty.as_ref(), Some(&scalar_type("bool")));
    assert_eq!(value, &NirExpr::Bool(disjunction));
    let NirStmt::If {
        condition,
        then_body,
        else_body,
    } = &after[3]
    else {
        panic!()
    };
    assert_eq!(condition, lhs.as_ref());
    let (selected, skipped) = if disjunction {
        (else_body, then_body)
    } else {
        (then_body, else_body)
    };
    assert!(skipped.is_empty());
    assert_eq!(selected.len(), 5);
    let mut bindings = Vec::new();
    for ((stmt, original), kind) in selected[..4]
        .iter()
        .zip(old_args)
        .zip(["i64", "State", "i64", "bool"])
    {
        let NirStmt::Let { name, ty, value } = stmt else {
            panic!()
        };
        assert_eq!(value, original);
        assert_eq!(ty.as_ref(), Some(&scalar_type(kind)));
        bindings.push(name);
    }
    let NirStmt::Let {
        name: assigned,
        ty,
        value: NirExpr::Call { callee, args },
    } = &selected[4]
    else {
        panic!()
    };
    assert_eq!(assigned, name);
    assert_eq!(ty.as_ref(), Some(&scalar_type("bool")));
    assert_eq!(callee, "helper");
    assert_eq!(args.len(), 5);
    assert_eq!(args[0], NirExpr::Var(bindings[0].clone()));
    assert_eq!(access(&args[1]).unwrap(), [bindings[1].as_str(), "a", "x"]);
    assert_eq!(access(&args[2]).unwrap(), [bindings[1].as_str(), "b", "y"]);
    assert_eq!(args[3], NirExpr::Var(bindings[2].clone()));
    assert_eq!(args[4], NirExpr::Var(bindings[3].clone()));
    let NirStmt::If {
        condition,
        then_body,
        else_body,
    } = &after[4]
    else {
        panic!()
    };
    assert_eq!(condition, &NirExpr::Var(name.clone()));
    assert_eq!(then_body, old_then);
    assert_eq!(else_body, old_else);
}

#[test]
fn caller_spills_gated_predicates_keep_complete_rhs_in_one_selected_arm() {
    for (op, disjunction) in [("&&", false), ("||", true)] {
        for gate in ["gate", "true", "false"] {
            for argument in ["produce(state, unused)", CONSTRUCTOR] {
                for nested in [false, true] {
                    for reverse in [false, true] {
                        let body = if nested {
                            format!(
                                "if flag {{ {} }} else {{ return 0; }}",
                                super::conditions::BODY
                            )
                        } else {
                            super::conditions::BODY.into()
                        };
                        let mut module = fixture(gate, op, argument, &body);
                        let original = function(&module, "entry").body.clone();
                        let producer = function(&module, "produce").clone();
                        let mut direct = function(&module, "entry").clone();
                        direct.name = "direct".into();
                        direct.body = function(
                            &fixture("gate", "&&", argument, super::conditions::BODY),
                            "entry",
                        )
                        .body
                        .clone();
                        let NirStmt::If { condition, .. } = &mut direct.body[2] else {
                            panic!()
                        };
                        let NirExpr::Binary { rhs, .. } = condition else {
                            panic!()
                        };
                        *condition = *rhs.clone();
                        module.functions.push(direct);
                        if reverse {
                            module.functions.reverse();
                            module.structs.reverse();
                        }
                        crate::nir_verify::verify_nir_module(&module).unwrap();
                        assert!(run(&mut module));
                        crate::nir_verify::verify_nir_module(&module).unwrap();
                        assert_eq!(function(&module, "produce"), &producer);
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
                        assert_gated_body(before, after, disjunction);
                        let before = module.clone();
                        assert!(!run(&mut module));
                        assert_eq!(module, before);
                    }
                }
            }
        }
    }
}

#[test]
fn caller_spills_gated_predicates_retain_kind_gate_loop_budget_and_atomic_vetoes() {
    for condition in [
        "(gate == true) && helper(checked(divisor), ARGUMENT, 20 / divisor, flag)",
        "helper(checked(divisor), ARGUMENT, 20 / divisor, flag) && gate",
        "gate && (flag || helper(checked(divisor), ARGUMENT, 20 / divisor, flag))",
        "(gate && helper(checked(divisor), ARGUMENT, 20 / divisor, flag)) || flag",
    ] {
        let body = format!("if {condition} {{ return 11; }} return 19;");
        let mut module = fixture(
            "gate",
            "&&",
            "produce(state, unused)",
            super::conditions::BODY,
        );
        let text = super::conditions::source("produce(state, unused)", &body).replace(
            "flag: bool, yes: i64, no: i64",
            "flag: bool, yes: i64, no: i64, gate: bool",
        );
        let mut bad =
            function(&crate::frontend::parse_nuis_module(&text).unwrap(), "entry").clone();
        bad.name = "unproven".into();
        module.functions.push(bad);
        let before = module.clone();
        assert!(!run(&mut module), "{condition}");
        assert_eq!(module, before);
    }
    for mutation in 0..8 {
        let mut module = fixture(
            "gate",
            "||",
            "produce(state, unused)",
            super::conditions::BODY,
        );
        match mutation {
            0 => {
                function_mut(&mut module, "entry")
                    .params
                    .last_mut()
                    .unwrap()
                    .ty
                    .is_ref = true
            }
            1 => {
                function_mut(&mut module, "entry")
                    .params
                    .last_mut()
                    .unwrap()
                    .ty
                    .is_optional = true
            }
            2 => {
                function_mut(&mut module, "entry")
                    .params
                    .last_mut()
                    .unwrap()
                    .ty = scalar_type("i64")
            }
            3 => function_mut(&mut module, "helper").return_type = Some(scalar_type("i64")),
            4 => function_mut(&mut module, "entry").body.push(NirStmt::Let {
                name: "gate".into(),
                ty: None,
                value: NirExpr::Bool(false),
            }),
            5 => {
                let entry = function_mut(&mut module, "entry");
                entry.body = vec![NirStmt::While {
                    condition: NirExpr::Var("flag".into()),
                    body: entry.body.clone(),
                }];
            }
            6 => function_mut(&mut module, "produce")
                .body
                .insert(0, NirStmt::Print(NirExpr::Int(1))),
            7 => {
                let entry = function_mut(&mut module, "entry");
                entry.params.pop();
                entry.body.insert(
                    0,
                    NirStmt::Let {
                        name: "gate".into(),
                        ty: Some(scalar_type("bool")),
                        value: NirExpr::Bool(true),
                    },
                );
            }
            _ => unreachable!(),
        }
        let before = module.clone();
        assert!(!run(&mut module), "{mutation}");
        assert_eq!(module, before);
    }
    let mut module = fixture(
        "gate",
        "&&",
        "produce(state, unused)",
        super::conditions::BODY,
    );
    function_mut(&mut module, "entry").params.push(NirParam {
        name: "__nuis_caller_predicate_0".into(),
        ty: scalar_type("bool"),
    });
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
        &BTreeSet::from(["gate".into()])
    )
    .is_none());
    assert!(run(&mut module));
    let NirStmt::Let { name, .. } = &function(&module, "entry").body[2] else {
        panic!()
    };
    assert_ne!(name, "__nuis_caller_predicate_0");
    crate::nir_verify::verify_nir_module(&module).unwrap();
}

#[test]
fn caller_spills_gated_predicates_preserve_skipped_checks_and_selected_results_with_an_oracle() {
    for (op, disjunction) in [("&&", false), ("||", true)] {
        for first in [0, 2, 3] {
            for unused in [0, 2] {
                for guard in [0, 1] {
                    for flag in [false, true] {
                        for gate in [false, true] {
                            for branches in 0..4 {
                                let yes = if branches & 1 == 0 { 2 } else { 0 };
                                let no = if branches & 2 == 0 { 2 } else { 0 };
                                let mut module = fixture(
                                    "gate",
                                    op,
                                    "produce(state, unused)",
                                    super::conditions::BODY,
                                );
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
                                        NirExpr::Bool(flag),
                                        NirExpr::Int(yes),
                                        NirExpr::Int(no),
                                        NirExpr::Bool(gate),
                                    ],
                                }))];
                                module.functions.push(main);
                                let reached = if disjunction { !gate } else { gate };
                                let expected = if guard == 0 {
                                    Ok(yir_core::Value::Int(0))
                                } else if reached && (first == 0 || unused == 0) {
                                    Err(())
                                } else {
                                    let predicate = if reached {
                                        10 / first + 20 / first + if flag { 11 } else { 19 }
                                            > if flag { 25 } else { 35 }
                                    } else {
                                        disjunction
                                    };
                                    let divisor = if predicate { yes } else { no };
                                    if divisor == 0 {
                                        Err(())
                                    } else {
                                        Ok(yir_core::Value::Int(if predicate {
                                            22 / divisor
                                        } else {
                                            38 / divisor
                                        }))
                                    }
                                };
                                let before_module = module.clone();
                                let original = function(&module, "entry").body.clone();
                                assert!(run(&mut module));
                                crate::nir_verify::verify_nir_module(&module).unwrap();
                                assert_eq!(
                                    super::conditions::execute(&before_module),
                                    expected,
                                    "before {op}/{first}/{unused}/{guard}/{flag}/{gate}/{branches}"
                                );
                                assert_eq!(
                                    super::conditions::execute(&module),
                                    expected,
                                    "after {op}/{first}/{unused}/{guard}/{flag}/{gate}/{branches}"
                                );
                                assert_gated_body(
                                    &original,
                                    &function(&module, "entry").body,
                                    disjunction,
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}
