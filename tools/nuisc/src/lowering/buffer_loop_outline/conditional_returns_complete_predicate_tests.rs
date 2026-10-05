use super::super::tests::{execute, outline_test, source as simple_source};
use super::*;

#[path = "conditional_returns_complete_predicate_native_tests.rs"]
mod native;

fn arm(kind: &str, op: &str) -> String {
    let (prefix, condition) = match kind {
        "call" => ("", "helper(produce(divisor))"),
        "alias" => (
            "let selected = helper(produce(divisor));",
            "selected == true",
        ),
        "field" => ("let packet = produce(divisor);", "packet.value > 0"),
        "inline-field" => ("", "produce(divisor).value > 0"),
        "compare" => ("", "divisor > 0"),
        "nested" => ("", "(divisor > 0) == true"),
        "bool-eq" => ("", "gate == true"),
        "bool-ne" => ("", "gate != false"),
        "division" => ("", "10 / divisor > 0"),
        "remainder" => ("", "10 % divisor == 0"),
        _ => unreachable!(),
    };
    format!("{prefix} if {condition} {{ return gate {op} helper(produce(divisor)); }} else {{ return false; }}")
}

fn source(
    shape: &str,
    kind: &str,
    op: &str,
    outer: bool,
    gate: bool,
    divisor: i64,
    early: bool,
) -> String {
    let body = arm(kind, op);
    let branch = match shape {
        "then" => format!("if outer {{ {body} }}"),
        "else" => format!("if outer {{ }} else {{ {body} }}"),
        "both" => format!("if outer {{ {body} }} else {{ {body} }}"),
        _ => unreachable!(),
    };
    simple_source("then", op, outer, gate, divisor, early).replace(
        &format!("if outer {{ return gate {op} helper(produce(divisor)); }}"),
        &branch,
    )
}

fn check(
    source: &str,
    shape: &str,
    kind: &str,
    op: &str,
    outer: bool,
    gate: bool,
    divisor: i64,
    early: bool,
) {
    let entered = !early && (shape == "both" || if shape == "then" { outer } else { !outer });
    let selected = match kind {
        "bool-eq" | "bool-ne" => gate,
        "division" => divisor != 0 && 10 / divisor > 0,
        "remainder" => divisor != 0 && 10 % divisor == 0,
        _ => divisor > 0,
    };
    let rhs = entered && selected && if op == "&&" { gate } else { !gate };
    let condition_checked = matches!(
        kind,
        "call" | "alias" | "field" | "inline-field" | "division" | "remainder"
    );
    let trapped = divisor == 0 && (rhs || (entered && condition_checked));
    let value = entered && selected && if rhs { divisor > 0 } else { gate };
    let result = if value { 11 } else { 19 };
    let mut prints = Vec::new();
    if !early {
        prints.push(99);
        if !entered {
            prints.push(77);
        }
    }
    prints.push(result);
    let predicate_calls = usize::from(entered && matches!(kind, "call" | "alias"));
    execute(
        source,
        (!trapped).then_some(result),
        &prints,
        predicate_calls + usize::from(rhs),
    );
}

#[test]
fn conditional_return_complete_predicates_evaluate_calls_and_fields_once_behind_outer_guards() {
    let mut cases = 0;
    for kind in ["call", "alias", "field", "inline-field"] {
        for shape in ["then", "else", "both"] {
            for outer in [false, true] {
                for early in [false, true] {
                    for (op, gate, divisor) in [
                        ("&&", false, 0),
                        ("||", true, 0),
                        ("&&", true, 2),
                        ("||", false, -2),
                        ("&&", true, 0),
                        ("||", false, 0),
                    ] {
                        check(
                            &source(shape, kind, op, outer, gate, divisor, early),
                            shape,
                            kind,
                            op,
                            outer,
                            gate,
                            divisor,
                            early,
                        );
                        cases += 1;
                    }
                }
            }
        }
    }
    assert_eq!(cases, 288);
}

#[test]
fn conditional_return_complete_predicates_keep_computed_bool_kinds_and_nested_comparisons() {
    let mut cases = 0;
    for kind in ["compare", "nested", "bool-eq", "bool-ne"] {
        for shape in ["then", "else", "both"] {
            for outer in [false, true] {
                for (op, gate, divisor) in [
                    ("&&", true, 2),
                    ("&&", false, 0),
                    ("||", false, -2),
                    ("||", true, 0),
                ] {
                    check(
                        &source(shape, kind, op, outer, gate, divisor, false),
                        shape,
                        kind,
                        op,
                        outer,
                        gate,
                        divisor,
                        false,
                    );
                    cases += 1;
                }
            }
        }
    }
    assert_eq!(cases, 96);
}

#[test]
fn conditional_return_complete_predicates_keep_reached_division_and_remainder_checks() {
    let mut cases = 0;
    for kind in ["division", "remainder"] {
        for shape in ["then", "else", "both"] {
            for outer in [false, true] {
                for divisor in [0, 2, -2] {
                    check(
                        &source(shape, kind, "&&", outer, false, divisor, false),
                        shape,
                        kind,
                        "&&",
                        outer,
                        false,
                        divisor,
                        false,
                    );
                    cases += 1;
                }
            }
        }
    }
    assert_eq!(cases, 36);
}

fn integer_source(shape: &str, outer: bool, divisor: i64) -> String {
    let body = "if helper(produce(divisor)) { let current = 10 / divisor - 5; return current; } else { return 0; }";
    let branch = match shape {
        "then" => format!("if outer {{ {body} }}"),
        "else" => format!("if outer {{ }} else {{ {body} }}"),
        "both" => format!("if outer {{ {body} }} else {{ {body} }}"),
        _ => unreachable!(),
    };
    source("then", "call", "&&", outer, false, divisor, false)
        .replace("early: bool) -> bool", "early: bool) -> i64")
        .replace("if early { return false; }", "if early { return 0; }")
        .replace(&format!("if outer {{ {} }}", arm("call", "&&")), &branch)
        .replace("print(77); return false;", "print(77); return 19;")
        .replace(
            "if result { print(11); return 11; }\n            print(19); return 19;",
            "print(result); return result;",
        )
}

#[test]
fn conditional_return_complete_predicates_keep_real_zero_returns_and_skip_parent_suffixes() {
    let mut cases = 0;
    for shape in ["then", "else", "both"] {
        for outer in [false, true] {
            for divisor in [0, 2, -2] {
                let entered = shape == "both" || if shape == "then" { outer } else { !outer };
                let result = if !entered {
                    19
                } else if divisor > 0 {
                    10 / divisor - 5
                } else {
                    0
                };
                let prints = if entered {
                    vec![99, result]
                } else {
                    vec![99, 77, result]
                };
                execute(
                    &integer_source(shape, outer, divisor),
                    (!(entered && divisor == 0)).then_some(result),
                    &prints,
                    usize::from(entered),
                );
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 18);
}

#[test]
fn conditional_return_complete_predicates_keep_prefix_checks_current_versions_and_sibling_paths() {
    let mut cases = 0;
    for outer in [false, true] {
        for gate in [false, true] {
            let prefix_source = source("then", "compare", "&&", outer, gate, 2, false)
                .replace("if divisor > 0", "let ignored = produce(0); if divisor > 0");
            execute(&prefix_source, (!outer).then_some(19), &[99, 77, 19], 0);
            cases += 1;
            for divisor in [0, 2, -2] {
                let source = source("then", "bool-eq", "&&", outer, gate, divisor, false)
                    .replace("print(99);", "let gate: bool = gate == false; print(99);");
                check(
                    &source, "then", "bool-eq", "&&", outer, !gate, divisor, false,
                );
                cases += 1;
            }
        }
        for divisor in [-2, 0, 2] {
            let source = source("then", "call", "&&", outer, false, divisor, false)
                .replace("print(99);", "let divisor: i64 = divisor + 2; print(99);");
            check(
                &source,
                "then",
                "call",
                "&&",
                outer,
                false,
                divisor + 2,
                false,
            );
            cases += 1;
        }
        for divisor in [0, 2, -2] {
            check(
                &source("then", "call", "&&", outer, false, divisor, true),
                "then",
                "call",
                "&&",
                outer,
                false,
                divisor,
                true,
            );
            cases += 1;
        }
    }
    assert_eq!(cases, 28);
    let mut nested_cases = 0;
    for shape in ["then", "both"] {
        for outer in [false, true] {
            for gate in [false, true] {
                for divisor in [0, 2, -2] {
                    let tree = "if helper(produce(divisor)) { let current = divisor; if helper(produce(current)) { return gate && helper(produce(current)); } else { return false; } } else { const current: bool = false; if current == false { return current; } else { return false; } }";
                    let source = source(shape, "call", "&&", outer, gate, divisor, false)
                        .replace(&arm("call", "&&"), tree);
                    let entered = shape == "both" || outer;
                    let positive = entered && divisor > 0;
                    let result = if positive && gate { 11 } else { 19 };
                    let prints = if entered {
                        vec![99, result]
                    } else {
                        vec![99, 77, result]
                    };
                    execute(
                        &source,
                        (!(entered && divisor == 0)).then_some(result),
                        &prints,
                        usize::from(entered)
                            + usize::from(positive)
                            + usize::from(positive && gate),
                    );
                    nested_cases += 1;
                }
            }
        }
    }
    assert_eq!(nested_cases, 24);
}

#[test]
fn conditional_return_complete_predicates_retain_whole_bodies_captures_hygiene_and_idempotence() {
    for kind in ["call", "alias", "field", "inline-field"] {
        for shape in ["then", "else", "both"] {
            let source = source(shape, kind, "&&", true, true, 2, false).replace(
                "return gate &&",
                "let __nuis_return_condition_0 = gate; return __nuis_return_condition_0 &&",
            );
            let mut module = crate::frontend::parse_nuis_module(&source).unwrap();
            let before = module.clone();
            let generated = outline_test(&mut module);
            assert_eq!(generated.len(), 1, "{shape}/{kind}");
            let old = before.functions.iter().find(|f| f.name == "event").unwrap();
            let new = module.functions.iter().find(|f| f.name == "event").unwrap();
            assert_eq!(&new.body[..2], &old.body[..2]);
            assert_eq!(
                &new.body[new.body.len() - 2..],
                &old.body[old.body.len() - 2..]
            );
            assert!(!new.body.iter().any(
                |s| matches!(s, NirStmt::Let { name, .. } if name.starts_with("__nuis_return_exit"))
            ));
            let helper = module
                .functions
                .iter()
                .find(|f| generated.contains(&f.name))
                .unwrap();
            assert_eq!(
                helper
                    .params
                    .iter()
                    .skip(1)
                    .map(|p| p.name.as_str())
                    .collect::<Vec<_>>(),
                ["divisor", "gate"]
            );
            let NirStmt::If {
                then_body,
                else_body,
                ..
            } = &old.body[2]
            else {
                panic!()
            };
            let NirStmt::If {
                then_body: yes,
                else_body: no,
                ..
            } = &helper.body[0]
            else {
                panic!()
            };
            for (original, cloned) in [(then_body, yes), (else_body, no)] {
                if !original.is_empty() {
                    assert_eq!(original, cloned);
                }
                let mut bindings = BTreeSet::new();
                branches::collect_bindings(cloned, &mut bindings);
                assert!(!bindings.contains(&helper.params[0].name));
            }
            crate::nir_verify::verify_nir_module(&module).unwrap();
            let once = module.clone();
            assert!(outline_test(&mut module).is_empty());
            assert_eq!(module, once);
        }
    }
}

#[test]
fn conditional_return_complete_predicates_bound_condition_work_and_retain_partial_effect_and_capture_vetoes(
) {
    let base = source("then", "call", "&&", true, true, 2, false);
    for count in [29, 30] {
        let locals = (0..count)
            .map(|n| format!("let fresh{n} = divisor + {n};"))
            .collect::<String>();
        let source = base.replace(
            &arm("call", "&&"),
            &format!("{locals} {}", arm("call", "&&")),
        );
        let mut module = crate::frontend::parse_nuis_module(&source).unwrap();
        let before = module.clone();
        if count == 29 {
            assert_eq!(outline_test(&mut module).len(), 1);
            let compiled = crate::pipeline::compile_source(&source).unwrap();
            yir_lower_llvm::emit_module(&compiled.yir).unwrap();
        } else {
            assert!(outline_test(&mut module).is_empty());
            assert_eq!(module, before);
        }
    }
    for candidate in [
        "if helper(produce(divisor)) { return helper(produce(divisor)); }",
        "if helper(produce(divisor)) == true { return helper(produce(divisor)); } else { let ignored = divisor; }",
        "let packet = produce(divisor); if packet.value > 0 { return helper(packet); }",
        "if 10 / divisor > 0 { return helper(produce(divisor)); }",
    ] {
        let admitted = base.replace(&arm("call", "&&"), candidate);
        let compiled = crate::pipeline::compile_source(&admitted).unwrap();
        yir_lower_llvm::emit_module(&compiled.yir).unwrap();
    }
    let admitted = base.replace(
        &arm("call", "&&"),
        "if gate && helper(produce(divisor)) { return helper(produce(divisor)); } else { return false; }",
    );
    execute(&admitted, Some(11), &[99, 11], 2);
    for candidate in [
        "if helper(produce(divisor)) { print(88); return false; } else { return false; }",
        "let divisor = 2; if helper(produce(divisor)) { return false; } else { return true; }",
        "if helper(produce(divisor)) { while gate { return false; } return true; } else { return false; }",
        "if helper(produce(divisor)) { return false; } else { return true; } let ignored = divisor;",
    ] {
        let mut module = crate::frontend::parse_nuis_module(&base.replace(&arm("call", "&&"), candidate)).unwrap();
        let before = module.clone();
        assert!(outline_test(&mut module).is_empty(), "{candidate}");
        assert_eq!(module, before, "{candidate}");
    }
    let outer = base.replace("if outer {", "if helper(produce(divisor)) {");
    let mut module = crate::frontend::parse_nuis_module(&outer).unwrap();
    assert_eq!(outline_test(&mut module).len(), 1);
    execute(&outer, Some(11), &[99, 11], 3);
    for mutation in [
        "borrow",
        "optional",
        "kind",
        "condition",
        "effect",
        "nominal",
    ] {
        let mut module = crate::frontend::parse_nuis_module(&base).unwrap();
        if mutation == "effect" {
            module
                .functions
                .iter_mut()
                .find(|f| f.name == "produce")
                .unwrap()
                .body
                .insert(0, NirStmt::Print(NirExpr::Int(88)));
        } else {
            let event = module
                .functions
                .iter_mut()
                .find(|f| f.name == "event")
                .unwrap();
            if matches!(mutation, "condition" | "nominal") {
                let NirStmt::If { then_body, .. } = &mut event.body[2] else {
                    panic!()
                };
                let NirStmt::If { condition, .. } = &mut then_body[0] else {
                    panic!()
                };
                *condition = if mutation == "condition" {
                    NirExpr::Int(1)
                } else {
                    NirExpr::Call {
                        callee: "helper".into(),
                        args: vec![NirExpr::StructLiteral {
                            type_name: "Packet".into(),
                            type_args: vec![],
                            fields: vec![("value".into(), NirExpr::Int(2))],
                        }],
                    }
                };
            } else {
                let ty = &mut event
                    .params
                    .iter_mut()
                    .find(|p| p.name == "divisor")
                    .unwrap()
                    .ty;
                match mutation {
                    "borrow" => ty.is_ref = true,
                    "optional" => ty.is_optional = true,
                    "kind" => *ty = scalar_type("f64"),
                    _ => unreachable!(),
                }
            }
        }
        let before = module.clone();
        assert!(outline_test(&mut module).is_empty(), "{mutation}");
        assert_eq!(module, before, "{mutation}");
    }
    fn tree(depth: usize) -> NirExpr {
        if depth == 0 {
            NirExpr::Int(1)
        } else {
            NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(tree(depth - 1)),
                rhs: Box::new(tree(depth - 1)),
            }
        }
    }
    let source = integer_source("then", true, 2).replace(
        "@noinline fn event",
        "@noinline fn decide(value: i64) -> bool { return value > 0; } @noinline fn event",
    );
    for over in [false, true] {
        let mut module = crate::frontend::parse_nuis_module(&source).unwrap();
        let event = module
            .functions
            .iter_mut()
            .find(|f| f.name == "event")
            .unwrap();
        let NirStmt::If { then_body, .. } = &mut event.body[2] else {
            panic!()
        };
        *then_body = vec![NirStmt::If {
            condition: NirExpr::Call {
                callee: "decide".into(),
                args: vec![tree(10)],
            },
            then_body: vec![NirStmt::Return(Some(tree(10)))],
            else_body: vec![NirStmt::Return(Some(NirExpr::Int(0)))],
        }];
        if over {
            then_body.insert(
                0,
                NirStmt::Let {
                    name: "extra".into(),
                    ty: Some(scalar_type("i64")),
                    value: NirExpr::Int(1),
                },
            );
        }
        let before = module.clone();
        assert_eq!(outline_test(&mut module).is_empty(), over);
        if over {
            assert_eq!(module, before);
        } else {
            crate::nir_verify::verify_nir_module(&module).unwrap();
        }
    }
    let mut module = crate::frontend::parse_nuis_module(&base).unwrap();
    let event = module
        .functions
        .iter_mut()
        .find(|f| f.name == "event")
        .unwrap();
    let NirStmt::If { then_body, .. } = &mut event.body[2] else {
        panic!()
    };
    let NirStmt::If { condition, .. } = &mut then_body[0] else {
        panic!()
    };
    let mut value = NirExpr::Int(1);
    for _ in 0..64 {
        value = NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(NirExpr::Int(1)),
            rhs: Box::new(value),
        };
    }
    *condition = NirExpr::Binary {
        op: NirBinaryOp::Gt,
        lhs: Box::new(value),
        rhs: Box::new(NirExpr::Int(0)),
    };
    let before = module.clone();
    assert!(outline_test(&mut module).is_empty());
    assert_eq!(module, before);
}
