use super::*;

#[path = "conditional_returns_print_values_fixtures.rs"]
mod fixtures;
pub(super) use fixtures::{computed_expected, computed_source};
#[path = "conditional_returns_print_values_budget_tests.rs"]
mod budget;
#[path = "conditional_returns_print_values_native_tests.rs"]
mod native;

#[test]
fn conditional_return_computed_prints_preserve_selected_work_order_and_exits() {
    let mut cases = 0;
    for kind in ["division", "remainder", "call", "field"] {
        for mode in ["return", "suffix", "zero"] {
            for entry in ["atom", "nested"] {
                for shape in ["then", "else", "both"] {
                    for (index, input) in INPUTS.into_iter().enumerate() {
                        let stamp = [0, 0, 2, -2, 2, -2, 0, 0][index];
                        let text = computed_source(kind, mode, entry, shape, input, stamp);
                        let (result, prints, calls) =
                            computed_expected(kind, mode, entry, shape, input, stamp);
                        execute(&text, result, &prints, calls);
                        cases += 1;
                    }
                }
            }
        }
    }
    assert_eq!(cases, 576);
}

#[test]
fn conditional_return_computed_prints_keep_pure_helpers_minimal_and_install_atomic() {
    for shape in ["then", "else", "both"] {
        let text = computed_source("call", "return", "computed", shape, INPUTS[2], 2);
        let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
        let before = module.clone();
        let generated = outline_test(&mut module);
        assert_eq!(generated.len(), if shape == "both" { 3 } else { 2 });
        for function in module
            .functions
            .iter()
            .filter(|f| generated.contains(&f.name))
        {
            assert!(!format!("{:?}", function.body).contains("Print("));
            let captures = function
                .params
                .iter()
                .skip(1)
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>();
            if function.name.starts_with("__nuis_conditional_print_value") {
                assert_eq!(captures, ["stamp"]);
                let NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } = &function.body[0]
                else {
                    panic!()
                };
                assert_eq!(condition, &NirExpr::Var(function.params[0].name.clone()));
                assert_eq!(
                    then_body,
                    &[NirStmt::Return(Some(NirExpr::Call {
                        callee: "observe".into(),
                        args: vec![NirExpr::Var("stamp".into())]
                    }))]
                );
                assert_eq!(else_body, &[NirStmt::Return(Some(NirExpr::Int(0)))]);
            } else {
                assert_eq!(captures, ["gate", "left", "right"]);
            }
        }
        for name in ["produce", "helper", "observe"] {
            assert_eq!(
                module.functions.iter().find(|f| f.name == name),
                before.functions.iter().find(|f| f.name == name)
            );
        }
        crate::nir_verify::verify_nir_module(&module).unwrap();
        let once = module.clone();
        assert!(outline_test(&mut module).is_empty());
        assert_eq!(module, once);
    }
    let text = computed_source("division", "return", "atom", "both", INPUTS[2], 2);
    for mutation in [
        "borrow",
        "optional",
        "generic",
        "record",
        "i32",
        "bool",
        "unbound",
        "callee-effect",
        "logical-argument",
        "print-only-work",
    ] {
        let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
        if mutation == "logical-argument" {
            module.functions.push(helper(
                "project".into(),
                vec![NirParam {
                    name: "gate".into(),
                    ty: scalar_type("bool"),
                }],
                vec![NirStmt::Return(Some(NirExpr::Int(1)))],
            ));
        }
        if mutation == "callee-effect" {
            module
                .functions
                .iter_mut()
                .find(|f| f.name == "produce")
                .unwrap()
                .body
                .insert(0, NirStmt::Print(NirExpr::Int(1)));
        } else {
            let parent = event(&mut module);
            match mutation {
                "borrow" => parent.params[6].ty.is_ref = true,
                "optional" => parent.params[6].ty.is_optional = true,
                "generic" => parent.params[6].ty.generic_args.push(scalar_type("i64")),
                "record" | "i32" | "bool" => {
                    parent.params[6].ty = scalar_type(if mutation == "record" {
                        "Packet"
                    } else {
                        mutation
                    })
                }
                _ => {
                    let NirStmt::If {
                        then_body,
                        else_body,
                        ..
                    } = &mut parent.body[2]
                    else {
                        panic!()
                    };
                    match mutation {
                        "unbound" => else_body[1] = NirStmt::Print(NirExpr::Var("missing".into())),
                        "logical-argument" => {
                            else_body[1] = NirStmt::Print(NirExpr::Call {
                                callee: "project".into(),
                                args: vec![NirExpr::Binary {
                                    op: NirBinaryOp::And,
                                    lhs: Box::new(NirExpr::Var("gate".into())),
                                    rhs: Box::new(NirExpr::Bool(true)),
                                }],
                            })
                        }
                        "print-only-work" => {
                            *then_body.last_mut().unwrap() =
                                NirStmt::Return(Some(NirExpr::Bool(true)));
                            *else_body.last_mut().unwrap() =
                                NirStmt::Return(Some(NirExpr::Bool(false)));
                        }
                        _ => unreachable!(),
                    }
                }
            }
        }
        if mutation == "logical-argument" {
            let layouts = control_values::TypedLayouts::collect(&module);
            let carries = control_values::layouts(&module);
            let control = scalar_helpers::collect_with_layouts(&module, &carries);
            let catalog = scalar_helpers::collect_typed_values(&module, &layouts, &control);
            let parent = module.functions.iter().find(|f| f.name == "event").unwrap();
            let scope = parent
                .params
                .iter()
                .map(|p| (p.name.clone(), p.ty.clone()))
                .collect();
            let NirStmt::If { else_body, .. } = &parent.body[2] else {
                panic!()
            };
            let NirStmt::Print(value) = &else_body[1] else {
                panic!()
            };
            assert_eq!(
                control_values::value_type(value, &scope, &catalog, &layouts),
                Some(scalar_type("i64"))
            );
        }
        let before = module.clone();
        assert!(outline_test(&mut module).is_empty(), "{mutation}");
        assert_eq!(module, before, "{mutation}");
    }
}

#[test]
fn conditional_return_computed_prints_preserve_each_call_and_current_parent_binding() {
    for outer in [false, true] {
        let mut input = INPUTS[2];
        input.0 = outer;
        let text = computed_source("call", "return", "atom", "then", input, 2).replace(
            "print(observe(stamp));",
            "print(observe(stamp)); print(observe(stamp)); print(observe(stamp + 1));",
        );
        execute(
            &text,
            Some(if outer { 11 } else { 19 }),
            if outer {
                &[99, 88, 2, 2, 3, 89, 11]
            } else {
                &[99, 77, 19]
            },
            usize::from(outer),
        );
        let compiled = crate::pipeline::compile_source(&text).unwrap();
        for reversed in [false, true] {
            let mut yir = compiled.yir.clone();
            if reversed {
                yir.nodes.reverse();
                yir.edges.reverse();
                yir.functions.reverse();
                for function in &mut yir.functions {
                    function.body_nodes.reverse();
                }
            }
            let trace = yir_runtime_host::execute_module_source_with_registry(
                &crate::render::render_yir(&yir),
                &yir_verify::default_registry(),
            )
            .unwrap();
            let calls = trace
                .events
                .iter()
                .enumerate()
                .filter(|(_, e)| e.contains("cpu.call_i64") && e.contains("] observe("))
                .collect::<Vec<_>>();
            assert_eq!(calls.len(), if outer { 3 } else { 0 }, "{:?}", trace.events);
            if outer {
                let first = calls[0].0;
                let second = calls[1].0;
                assert!(
                    trace.events[first + 1..second]
                        .iter()
                        .any(|e| e.contains("cpu.guard_print") && e.ends_with("then print 2")),
                    "{:?}",
                    trace.events
                );
            }
        }
    }
    let text = computed_source("division", "return", "atom", "then", INPUTS[2], 2)
        .replace("print(99);", "print(99); let stamp: i64 = stamp + 1;");
    execute(&text, Some(11), &[99, 88, 33, 89, 11], 1);
    for mode in ["partial", "continuation"] {
        let text = computed_source("call", mode, "computed", "both", INPUTS[2], 2);
        let (result, prints, calls) =
            computed_expected("call", mode, "computed", "both", INPUTS[2], 2);
        execute(&text, result, &prints, calls);
    }
}

#[test]
fn conditional_return_computed_prints_keep_selected_overflow_and_fresh_names() {
    for op in ["/", "%"] {
        for outer in [false, true] {
            let mut input = INPUTS[2];
            input.0 = outer;
            let text = computed_source("division", "return", "atom", "then", input, -1).replace(
                "100 / stamp",
                &format!("(-9223372036854775807 - 1) {op} stamp"),
            );
            let compiled = crate::pipeline::compile_source(&text).unwrap();
            for reversed in [false, true] {
                let mut yir = compiled.yir.clone();
                if reversed {
                    yir.nodes.reverse();
                    yir.edges.reverse();
                    yir.functions.reverse();
                    for function in &mut yir.functions {
                        function.body_nodes.reverse();
                    }
                }
                let result = yir_runtime_host::execute_module_source_with_registry(
                    &crate::render::render_yir(&yir),
                    &yir_verify::default_registry(),
                );
                if outer {
                    let error = result.unwrap_err();
                    assert!(error.contains("overflow"), "{error}");
                } else {
                    assert!(result.is_ok());
                }
            }
            yir_lower_llvm::emit_module(&compiled.yir).unwrap();
            if !outer {
                execute(&text, Some(19), &[99, 77, 19], 0);
            }
        }
    }
    let text = computed_source("call", "return", "atom", "then", INPUTS[2], 2)
        .replace("stamp", "__nuis_print_condition_0")
        .replace(
            "print(99);",
            "print(99); let __nuis_print_value_0: i64 = 1;",
        );
    execute(&text, Some(11), &[99, 88, 2, 89, 11], 1);
    let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
    outline_test(&mut module);
    let helper = module
        .functions
        .iter()
        .find(|f| f.name.starts_with("__nuis_conditional_print_value"))
        .unwrap();
    assert_ne!(helper.params[0].name, "__nuis_print_condition_0");
    assert_eq!(helper.params[1].name, "__nuis_print_condition_0");
    assert!(event(&mut module)
        .body
        .iter()
        .any(|stmt| matches!(stmt, NirStmt::Let { name, .. } if name == "__nuis_print_value_1")));
    crate::nir_verify::verify_nir_module(&module).unwrap();
}
