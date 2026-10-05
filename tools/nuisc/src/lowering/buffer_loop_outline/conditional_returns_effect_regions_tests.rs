use super::*;

#[path = "conditional_returns_effect_regions_fixtures.rs"]
mod fixtures;
use fixtures::{events, simple_expected, simple_source, staged_expected, staged_source};
#[path = "conditional_returns_effect_regions_budget_tests.rs"]
mod budget;
#[path = "conditional_returns_effect_logical_tests.rs"]
mod logical;
#[path = "conditional_returns_effect_regions_native_tests.rs"]
mod native;

#[test]
fn conditional_return_staged_initializers_preserve_selected_work_order_and_exits() {
    let mut cases = 0;
    for kind in ["division", "remainder", "call", "field"] {
        for mode in ["return", "suffix", "zero"] {
            for entry in ["atom", "nested"] {
                for shape in ["then", "else", "both"] {
                    for (index, input) in INPUTS.into_iter().enumerate() {
                        let stamp = [0, 0, 2, -2, 2, -2, 0, 0][index];
                        let text = staged_source(kind, mode, entry, shape, input, stamp);
                        let (result, prints, calls) =
                            staged_expected(kind, mode, entry, shape, input, stamp);
                        execute(&text, result, &prints, calls);
                        cases += 1;
                    }
                }
            }
        }
    }
    for mode in ["partial", "continuation", "inferred", "const", "complete"] {
        for input in INPUTS {
            let text = staged_source("remainder", mode, "computed", "both", input, 2);
            let (result, prints, calls) =
                staged_expected("remainder", mode, "computed", "both", input, 2);
            execute(&text, result, &prints, calls);
            cases += 1;
        }
    }
    assert_eq!(cases, 616);
}

#[test]
fn conditional_return_staged_initializers_reuse_bool_i64_and_retain_unused_work() {
    let mut cases = 0;
    for kind in [
        "checked",
        "bool",
        "unused",
        "unused-checked",
        "unused-between",
        "chain",
    ] {
        for outer in [false, true] {
            for stamp in [0, 2, -2] {
                let text = simple_source(kind, outer, stamp);
                let (result, prints, calls) = simple_expected(kind, outer, stamp);
                execute(&text, result, &prints, calls);
                if stamp != 0 || !outer {
                    for reversed in [false, true] {
                        let events = events(&text, reversed);
                        let invocations = events
                            .iter()
                            .filter(|e| e.contains("cpu.call_i64") && e.contains("] observe("))
                            .count();
                        assert_eq!(
                            invocations,
                            usize::from(
                                outer && matches!(kind, "unused" | "unused-between" | "chain")
                            ),
                            "{events:?}"
                        );
                        if outer && kind == "chain" {
                            let position = events
                                .iter()
                                .enumerate()
                                .find(|(_, e)| e.contains("] observe("))
                                .unwrap()
                                .0;
                            let prints = events
                                .iter()
                                .enumerate()
                                .filter(|(_, e)| {
                                    e.contains("cpu.guard_print ")
                                        && e.contains("if true then print")
                                })
                                .map(|(i, _)| i)
                                .collect::<Vec<_>>();
                            assert!(prints[0] < position && position < prints[1], "{events:?}");
                        }
                    }
                }
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 36);
    let text = simple_source("checked", true, 2)
        .replace("print(99);", "print(99); let stamp: i64 = stamp + 2;");
    execute(&text, Some(11), &[99, 25, 11], 0);
    // Identical source names in sibling arms have independent types and storage.
    for outer in [true, false] {
        let text = simple_source("checked", outer, 2).replace(
            "print(value); return value > 0; }",
            "print(value); return value > 0; } else { let value = helper(produce(stamp)); print(66); return value; }",
        );
        execute(
            &text,
            Some(11),
            if outer { &[99, 50, 11] } else { &[99, 66, 11] },
            usize::from(!outer),
        );
    }
}

#[test]
fn conditional_return_staged_initializers_keep_helpers_pure_typed_and_hygienic() {
    for kind in ["call", "division", "field"] {
        let text = staged_source(kind, "return", "computed", "both", INPUTS[2], 2);
        let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
        let before = module.clone();
        let generated = outline_test(&mut module);
        assert_eq!(generated.len(), 3);
        let initializers = module
            .functions
            .iter()
            .filter(|f| f.name.starts_with("__nuis_conditional_prefix_value"))
            .collect::<Vec<_>>();
        assert_eq!(initializers.len(), 2);
        for initializer in initializers {
            assert_eq!(initializer.return_type, Some(scalar_type("i64")));
            assert_eq!(
                initializer
                    .params
                    .iter()
                    .skip(1)
                    .map(|p| p.name.as_str())
                    .collect::<Vec<_>>(),
                ["stamp"]
            );
            let NirStmt::If { else_body, .. } = &initializer.body[0] else {
                panic!()
            };
            assert_eq!(else_body, &[NirStmt::Return(Some(NirExpr::Int(0)))]);
        }
        for function in module
            .functions
            .iter()
            .filter(|f| generated.contains(&f.name))
        {
            assert!(!format!("{:?}", function.body).contains("Print("));
        }
        assert!(!module
            .functions
            .iter()
            .any(|f| f.name.starts_with("__nuis_conditional_print_value")));
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
    let text = simple_source("bool", true, 2)
        .replace("let value =", "let __nuis_prefix_value_0 =")
        .replace("return value;", "return __nuis_prefix_value_0;")
        .replace("stamp", "__nuis_prefix_condition_0")
        .replace("outer", "__nuis_effect_region_gate_0");
    execute(&text, Some(11), &[99, 88, 11], 1);
    let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
    let generated = outline_test(&mut module);
    assert_eq!(generated.len(), 2);
    let initializer = module
        .functions
        .iter()
        .find(|f| f.name.starts_with("__nuis_conditional_prefix_value"))
        .unwrap();
    assert_eq!(initializer.return_type, Some(scalar_type("bool")));
    assert_ne!(initializer.params[0].name, initializer.params[1].name);
    let NirStmt::If { else_body, .. } = &initializer.body[0] else {
        panic!()
    };
    assert_eq!(else_body, &[NirStmt::Return(Some(NirExpr::Bool(false)))]);
    assert!(format!("{:?}", event(&mut module).body).contains("__nuis_effect_region_gate_1"));
}

#[test]
fn conditional_return_staged_initializers_validate_original_scopes_and_veto_atomically() {
    for mutation in [
        "shadow",
        "duplicate",
        "type",
        "missing",
        "self",
        "forward",
        "logical-leaf",
        "record",
        "entry-private",
        "tail-shadow",
        "nested-print",
        "no-return",
        "effectful-callee",
        "borrow",
        "optional",
        "generic",
    ] {
        let text = staged_source("call", "return", "atom", "both", INPUTS[2], 2);
        let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
        if mutation == "effectful-callee" {
            module
                .functions
                .iter_mut()
                .find(|f| f.name == "observe")
                .unwrap()
                .body
                .insert(0, NirStmt::Print(NirExpr::Int(1)));
        } else {
            let parent = event(&mut module);
            if matches!(mutation, "borrow" | "optional" | "generic") {
                match mutation {
                    "borrow" => parent.params[6].ty.is_ref = true,
                    "optional" => parent.params[6].ty.is_optional = true,
                    _ => parent.params[6].ty.generic_args.push(scalar_type("i64")),
                }
            } else {
                let NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } = &mut parent.body[2]
                else {
                    panic!()
                };
                let NirStmt::Const { name, ty, value } = &mut else_body[1] else {
                    panic!()
                };
                match mutation {
                    "shadow" => *name = "stamp".into(),
                    "type" => *ty = scalar_type("bool"),
                    "missing" | "self" | "forward" => {
                        *value = NirExpr::Var(
                            match mutation {
                                "missing" => "missing",
                                "self" => "local",
                                _ => "copy",
                            }
                            .into(),
                        )
                    }
                    "record" => {
                        *value = NirExpr::Call {
                            callee: "produce".into(),
                            args: vec![NirExpr::Var("stamp".into())],
                        }
                    }
                    "logical-leaf" => {
                        *ty = scalar_type("bool");
                        *value = NirExpr::Binary {
                            op: NirBinaryOp::Eq,
                            lhs: Box::new(NirExpr::Binary {
                                op: NirBinaryOp::And,
                                lhs: Box::new(NirExpr::Bool(true)),
                                rhs: Box::new(NirExpr::Bool(false)),
                            }),
                            rhs: Box::new(NirExpr::Bool(true)),
                        };
                        else_body[2] = NirStmt::Print(NirExpr::Int(66));
                        else_body[4] = NirStmt::Print(NirExpr::Int(67));
                    }
                    "entry-private" => *condition = NirExpr::Var("__nuis_prefix_value_0".into()),
                    "duplicate" => else_body.insert(2, else_body[1].clone()),
                    "tail-shadow" => else_body.insert(
                        else_body.len() - 1,
                        NirStmt::Let {
                            name: "local".into(),
                            ty: None,
                            value: NirExpr::Int(1),
                        },
                    ),
                    "nested-print" => else_body.insert(
                        2,
                        NirStmt::If {
                            condition: NirExpr::Bool(true),
                            then_body: vec![NirStmt::Print(NirExpr::Int(1))],
                            else_body: vec![],
                        },
                    ),
                    "no-return" => {
                        then_body.pop();
                        else_body.pop();
                    }
                    _ => unreachable!(),
                }
            }
        }
        let before = module.clone();
        assert!(outline_test(&mut module).is_empty(), "{mutation}");
        assert_eq!(module, before, "{mutation}");
    }
    // A total initializer or ordinary computed print cannot grant exit-work authority.
    let text = simple_source("checked", true, 2).replace("100 / stamp", "stamp + 1");
    let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
    let before = module.clone();
    assert!(outline_test(&mut module).is_empty());
    assert_eq!(module, before);
}
