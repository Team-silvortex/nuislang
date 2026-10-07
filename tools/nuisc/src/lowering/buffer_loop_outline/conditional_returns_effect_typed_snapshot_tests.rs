use super::*;
#[path = "conditional_returns_effect_typed_snapshot_fixtures.rs"]
mod fixtures;
use fixtures::{expected, observed, source, INPUTS};
#[path = "conditional_returns_effect_typed_snapshot_native_tests.rs"]
mod native;

fn neutral(ty: &str) -> NirExpr {
    match ty {
        "i32" => NirExpr::CastI64ToI32(Box::new(NirExpr::Int(0))),
        "f32" => NirExpr::F32("0.0".into()),
        "f64" => NirExpr::F64("0.0".into()),
        _ => unreachable!(),
    }
}

#[test]
fn conditional_return_typed_snapshots_preserve_selected_work_data_and_real_exits() {
    let mut sources = BTreeSet::new();
    for ty in ["i32", "f32", "f64"] {
        for shape in ["computed", "shared", "literal"] {
            for constant in [false, true] {
                for partial in [false, true] {
                    for input in INPUTS {
                        let text = source(ty, shape, constant, partial, input);
                        let (result, prints) = expected(partial, input);
                        execute(&text, result, &prints, 0);
                        sources.insert(text);
                    }
                }
            }
        }
    }
    assert_eq!(sources.len(), 288);
}

#[test]
fn conditional_return_typed_snapshots_keep_exact_seeds_captures_and_once_only_arguments() {
    for ty in ["i32", "f32", "f64"] {
        for shape in ["computed", "shared", "literal"] {
            for input in [INPUTS[1], INPUTS[2]] {
                let text = source(ty, shape, false, false, input);
                let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
                let before = module.clone();
                assert!(!outline_test(&mut module).is_empty(), "{ty} {shape}");
                let joined = module
                    .functions
                    .iter()
                    .find(|f| f.name.starts_with("__nuis_effect_join_value"))
                    .unwrap();
                assert_eq!(joined.return_type, Some(scalar_type(ty)));
                assert_eq!(joined.params[0].ty, scalar_type("bool"));
                let NirStmt::If {
                    then_body,
                    else_body,
                    ..
                } = &joined.body[0]
                else {
                    panic!()
                };
                assert_eq!(else_body, &[NirStmt::Return(Some(neutral(ty)))]);
                if shape == "shared" || (shape == "literal" && ty != "i32") {
                    assert!(matches!(&then_body[0], NirStmt::Return(_)));
                    assert_eq!(joined.params.len(), if shape == "shared" { 2 } else { 1 });
                } else {
                    assert!(matches!(&then_body[0], NirStmt::If { .. }));
                    assert_eq!(joined.params.len(), 4);
                }
                for helper in module
                    .functions
                    .iter()
                    .filter(|f| f.name.starts_with("__nuis_conditional_prefix_value"))
                {
                    assert_eq!(helper.params[0].ty, scalar_type("bool"));
                    if helper.return_type.as_ref() == Some(&scalar_type(ty)) {
                        let NirStmt::If { else_body, .. } = &helper.body[0] else {
                            panic!()
                        };
                        assert_eq!(else_body, &[NirStmt::Return(Some(neutral(ty)))]);
                    }
                }
                for name in ["choose", "observe", "inspect"] {
                    assert_eq!(
                        module.functions.iter().find(|f| f.name == name),
                        before.functions.iter().find(|f| f.name == name)
                    );
                }
                crate::nir_verify::verify_nir_module(&module).unwrap();
                let once = module.clone();
                assert!(outline_test(&mut module).is_empty());
                assert_eq!(module, once);
                for reversed in [false, true] {
                    let trace = events(&text, reversed);
                    for name in ["choose", "observe", "inspect"] {
                        assert_eq!(
                            trace
                                .iter()
                                .filter(|e| e.contains(&format!("] {name}(")))
                                .count(),
                            1,
                            "{trace:?}"
                        );
                    }
                    assert!(
                        trace.iter().any(|e| e.ends_with(&format!(
                            "inspect({}, true)",
                            observed(ty, shape, input.gate)
                        ))),
                        "{trace:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn conditional_return_typed_snapshots_veto_original_types_resources_scopes_and_legacy_tails() {
    for ty in ["i32", "f32", "f64"] {
        let base = source(ty, "computed", false, false, INPUTS[1]);
        for mutation in [
            "declared",
            "borrow",
            "resource",
            "condition",
            "effectful",
            "unreachable",
            "legacy-tail",
        ] {
            let mut module = crate::frontend::parse_nuis_module(&base).unwrap();
            if mutation == "effectful" {
                module
                    .functions
                    .iter_mut()
                    .find(|f| f.name == "observe")
                    .unwrap()
                    .body
                    .insert(0, NirStmt::Print(NirExpr::Int(9)));
            } else {
                let NirStmt::If { then_body, .. } = &mut event(&mut module).body[1] else {
                    panic!()
                };
                if mutation == "legacy-tail" {
                    then_body.remove(1);
                    let NirStmt::Return(Some(value)) = then_body.last_mut().unwrap() else {
                        panic!()
                    };
                    *value = NirExpr::Call {
                        callee: "inspect".into(),
                        args: vec![
                            NirExpr::Var("selected".into()),
                            NirExpr::Var("nested".into()),
                        ],
                    };
                } else {
                    let NirStmt::If {
                        condition,
                        else_body,
                        ..
                    } = &mut then_body[0]
                    else {
                        panic!()
                    };
                    if mutation == "condition" {
                        *condition = NirExpr::Var("left_value".into());
                    } else if mutation == "unreachable" {
                        else_body.insert(0, NirStmt::Return(Some(NirExpr::Bool(false))));
                    } else {
                        let NirStmt::Let {
                            ty: Some(declared), ..
                        } = &mut else_body[0]
                        else {
                            panic!()
                        };
                        match mutation {
                            "declared" => *declared = scalar_type("i64"),
                            "borrow" => declared.is_ref = true,
                            "resource" => *declared = scalar_type("Buffer"),
                            _ => unreachable!(),
                        }
                    }
                }
            }
            let before = module.clone();
            assert!(outline_test(&mut module).is_empty(), "{ty} {mutation}");
            assert_eq!(module, before, "{ty} {mutation}");
        }
        for copies in 0..=32 {
            let unused = (0..copies)
                .map(|i| format!("let unused_{i} = shared;"))
                .collect::<String>();
            let text = base.replace("let local: ", &format!("{unused} let local: "));
            let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
            let NirStmt::If { then_body, .. } = &event(&mut module).body[1] else {
                panic!()
            };
            let fits = effects::regions::bounds::preflight(then_body, 3);
            let before = module.clone();
            assert_eq!(
                !outline_test(&mut module).is_empty(),
                fits,
                "{ty} copies={copies}"
            );
            if !fits {
                assert_eq!(module, before);
            }
        }
    }
}

#[test]
fn conditional_return_typed_snapshots_keep_signed_literal_identity_and_legacy_result_profile() {
    assert!(!scalar(&scalar_type("i32")));
    assert!(!scalar(&scalar_type("f32")));
    assert!(!scalar(&scalar_type("f64")));
    for ty in ["f32", "f64"] {
        let text = source(ty, "literal", true, false, INPUTS[1])
            .replace("print(70); 0.0", "print(70); -0.0");
        assert!(text.contains("print(70); -0.0"));
        let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
        let NirStmt::If { then_body, .. } = &mut event(&mut module).body[1] else {
            panic!()
        };
        let NirStmt::If { then_body, .. } = &mut then_body[0] else {
            panic!()
        };
        let (NirStmt::Let { value, .. } | NirStmt::Const { value, .. }) =
            then_body.last_mut().unwrap()
        else {
            panic!()
        };
        let negative_zero = if ty == "f32" {
            NirExpr::F32("-0.0".into())
        } else {
            NirExpr::F64("-0.0".into())
        };
        assert_eq!(*value, negative_zero);
        assert!(!outline_test(&mut module).is_empty());
        let joined = module
            .functions
            .iter()
            .find(|f| f.name.starts_with("__nuis_effect_join_value"))
            .unwrap();
        assert_eq!(joined.params.len(), 2);
        let NirStmt::If { then_body, .. } = &joined.body[0] else {
            panic!()
        };
        assert!(matches!(&then_body[0], NirStmt::If { .. }));
        crate::nir_verify::verify_nir_module(&module).unwrap();
    }
}

#[test]
fn conditional_return_typed_snapshots_keep_nonliteral_float_negation_in_selected_execution() {
    let mut cases = 0;
    for ty in ["f32", "f64"] {
        for placement in ["helper", "selected"] {
            for constant in [false, true] {
                for partial in [false, true] {
                    for input in INPUTS {
                        let base = source(ty, "computed", constant, partial, input);
                        let text = if placement == "helper" {
                            base.replace("return value + 0.5;", "return -(value + 0.5);")
                        } else {
                            base.replace("print(70); local", "print(70); -local")
                                .replace("print(71); local", "print(71); -local")
                        };
                        assert_ne!(text, base);
                        let (result, prints) = expected(partial, input);
                        execute(&text, result, &prints, 0);
                        if result.is_some() {
                            for reversed in [false, true] {
                                let trace = events(&text, reversed);
                                let called = input.outer && !(partial && input.nested);
                                for name in ["observe", "inspect"] {
                                    assert_eq!(
                                        trace
                                            .iter()
                                            .filter(|e| e.contains(&format!("] {name}(")))
                                            .count(),
                                        usize::from(called),
                                        "{trace:?}"
                                    );
                                }
                                if called {
                                    let value = if input.gate { "-3" } else { "0.75" };
                                    assert!(
                                        trace.iter().any(|e| e.ends_with(&format!(
                                            "inspect({value}{ty}, {})",
                                            input.nested
                                        ))),
                                        "{trace:?}"
                                    );
                                }
                            }
                        }
                        cases += 1;
                    }
                }
            }
        }
    }
    assert_eq!(cases, 128);
}
