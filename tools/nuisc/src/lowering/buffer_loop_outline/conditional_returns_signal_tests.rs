use super::super::tests::{execute, outline_test, source as simple_source};
use super::*;

#[path = "conditional_returns_signal_native_tests.rs"]
mod native;

fn arm(kind: &str) -> String {
    let (prefix, condition) = match kind {
        "call" => ("", "helper(produce(divisor))"),
        "alias" => ("let selected = helper(produce(divisor));", "selected"),
        "field" => ("let packet = produce(divisor);", "packet.value > 0"),
        "inline-field" => ("", "produce(divisor).value > 0"),
        "division" => ("", "10 / divisor > 0"),
        "nested" => ("", "(divisor > 0) == true"),
        _ => unreachable!(),
    };
    format!("{prefix} if {condition} {{ return gate && helper(produce(divisor)); }} else {{ let ignored = 10 / divisor; }}")
}

fn source(shape: &str, kind: &str, outer: bool, gate: bool, divisor: i64, early: bool) -> String {
    let arm = arm(kind);
    let branch = match shape {
        "then" => format!("if outer {{ {arm} }}"),
        "else" => format!("if outer {{ }} else {{ {arm} }}"),
        "both" => format!("if outer {{ {arm} }} else {{ {arm} }}"),
        _ => unreachable!(),
    };
    simple_source("then", "&&", outer, gate, divisor, early).replace(
        "if outer { return gate && helper(produce(divisor)); }",
        &branch,
    )
}

#[test]
fn conditional_return_stored_signals_evaluate_predicates_once_before_exits_or_continuations() {
    let mut cases = 0;
    for kind in [
        "call",
        "alias",
        "field",
        "inline-field",
        "division",
        "nested",
    ] {
        for shape in ["then", "else", "both"] {
            for outer in [false, true] {
                for early in [false, true] {
                    for (gate, divisor) in [(false, 0), (false, 2), (true, 2), (true, -2)] {
                        let entered = !early
                            && (shape == "both" || if shape == "then" { outer } else { !outer });
                        let selected = divisor > 0;
                        let returned = entered && selected;
                        let trapped = entered && divisor == 0;
                        let rhs = returned && gate;
                        let result = if rhs { 11 } else { 19 };
                        let mut prints = Vec::new();
                        if !early {
                            prints.push(99);
                            if !returned {
                                prints.push(77);
                            }
                        }
                        prints.push(result);
                        execute(
                            &source(shape, kind, outer, gate, divisor, early),
                            (!trapped).then_some(result),
                            &prints,
                            usize::from(entered && matches!(kind, "call" | "alias"))
                                + usize::from(rhs),
                        );
                        cases += 1;
                    }
                }
            }
        }
    }
    assert_eq!(cases, 288);
}

fn integer_source(shape: &str, outer: bool, divisor: i64) -> String {
    let arm = "if helper(produce(divisor)) { return 10 / divisor - 5; } else { let ignored = 20 / divisor; }";
    let branch = match shape {
        "then" => format!("if outer {{ {arm} }}"),
        "else" => format!("if outer {{ }} else {{ {arm} }}"),
        "both" => format!("if outer {{ {arm} }} else {{ {arm} }}"),
        _ => unreachable!(),
    };
    format!("mod cpu Main {{
        struct Packet {{ unused: i64, value: i64 }}
        @noinline fn produce(divisor: i64) -> Packet {{ return Packet {{ unused: 10 / divisor, value: divisor }}; }}
        @noinline fn helper(packet: Packet) -> bool {{ return packet.value > 0; }}
        @noinline fn event(outer: bool, divisor: i64) -> i64 {{ print(99); {branch} print(77); return 19; }}
        fn main() -> i64 {{ let result = event({outer}, {divisor}); print(result); return result; }}
    }}")
}

#[test]
fn conditional_return_stored_signals_distinguish_real_zero_and_false_from_fallthrough() {
    for shape in ["then", "else", "both"] {
        for outer in [false, true] {
            for divisor in [0, 2, -2] {
                let entered = shape == "both" || if shape == "then" { outer } else { !outer };
                let returned = entered && divisor > 0;
                let result = if returned { 10 / divisor - 5 } else { 19 };
                let prints = if returned {
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
            }
        }
    }
    // The suffix traps only on continuation. A real false/zero exit skips it.
    for gate in [false, true] {
        execute(
            &source("then", "call", true, gate, 2, false).replace(
                "print(77); return false;",
                "print(77); return helper(produce(0));",
            ),
            Some(if gate { 11 } else { 19 }),
            &[99, if gate { 11 } else { 19 }],
            1 + usize::from(gate),
        );
    }
    execute(
        &integer_source("then", true, 2)
            .replace("print(77); return 19;", "print(77); return 10 / 0;"),
        Some(0),
        &[99, 0],
        1,
    );
    execute(
        &integer_source("then", true, -2)
            .replace("print(77); return 19;", "print(77); return 10 / 0;"),
        None,
        &[],
        0,
    );
}

#[test]
fn conditional_return_stored_signals_preserve_nested_siblings_mixed_arms_and_current_captures() {
    for outer in [false, true] {
        for gate in [false, true] {
            for divisor in [0, -4] {
                let source = source("both", "call", outer, gate, divisor, false)
                    .replace("print(99);", "print(99); let divisor = divisor + 2;")
                    .replace(
                        &format!("else {{ {} }}", arm("call")),
                        "else { if gate { return false; } else { let ignored = 10 / divisor; } }",
                    );
                let divisor = divisor + 2;
                let returned = if outer { divisor > 0 } else { gate };
                let rhs = outer && returned && gate;
                let result = if rhs { 11 } else { 19 };
                let prints = if returned {
                    vec![99, result]
                } else {
                    vec![99, 77, result]
                };
                execute(
                    &source,
                    Some(result),
                    &prints,
                    usize::from(outer) + usize::from(rhs),
                );
                let source = source.replace(
                    &arm("call"),
                    "if helper(produce(divisor)) { let selected = gate; if selected { return false; } else { let ignored = 20 / divisor; } } else { let selected = helper(produce(divisor)); if selected { return true; } else { let ignored = 30 / divisor; } }",
                );
                let returned = if outer { divisor > 0 && gate } else { gate };
                let prints = if returned {
                    vec![99, 19]
                } else {
                    vec![99, 77, 19]
                };
                execute(
                    &source,
                    Some(19),
                    &prints,
                    if outer {
                        if divisor > 0 {
                            1
                        } else {
                            2
                        }
                    } else {
                        0
                    },
                );
                let complete = source.replace(
                    "else { if gate { return false; } else { let ignored = 10 / divisor; } }",
                    "else { return false; }",
                );
                let returned = !outer || divisor > 0 && gate;
                let prints = if returned {
                    vec![99, 19]
                } else {
                    vec![99, 77, 19]
                };
                execute(
                    &complete,
                    Some(19),
                    &prints,
                    if outer {
                        if divisor > 0 {
                            1
                        } else {
                            2
                        }
                    } else {
                        0
                    },
                );
            }
        }
    }
}

#[test]
fn conditional_return_stored_signals_publish_one_private_snapshot_and_preserve_leaf_work() {
    for shape in ["then", "else", "both"] {
        let source = source(shape, "alias", true, false, 2, false)
            .replace("let selected =", "let __nuis_return_condition_0 =")
            .replace("if selected", "if __nuis_return_condition_0")
            .replace(
                "print(99);",
                "print(99); let __nuis_return_snapshot_0 = gate;",
            )
            .replace(
                "struct Packet",
                "struct __nuis_return_signal_0 { value: i64 } struct Packet",
            );
        let mut module = crate::frontend::parse_nuis_module(&source).unwrap();
        let before = module.clone();
        let generated = outline_test(&mut module);
        assert_eq!(generated.len(), 1);
        let helper = module
            .functions
            .iter()
            .find(|f| generated.contains(&f.name))
            .unwrap();
        let definition = module
            .structs
            .iter()
            .find(|s| Some(s.name.as_str()) == helper.return_type.as_ref().map(|t| t.name.as_str()))
            .unwrap();
        assert_eq!(definition.visibility, NirVisibility::Private);
        assert_ne!(definition.name, "__nuis_return_signal_0");
        assert_eq!(
            definition
                .fields
                .iter()
                .map(|f| (&*f.name, &*f.ty.name))
                .collect::<Vec<_>>(),
            [("exited", "bool"), ("value", "bool")]
        );
        assert_eq!(
            helper
                .params
                .iter()
                .skip(1)
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>(),
            ["divisor", "gate"]
        );
        let old = before.functions.iter().find(|f| f.name == "event").unwrap();
        let new = module.functions.iter().find(|f| f.name == "event").unwrap();
        assert_eq!(&new.body[..3], &old.body[..3]);
        assert_eq!(
            &new.body[new.body.len() - 2..],
            &old.body[old.body.len() - 2..]
        );
        assert_eq!(new.body.iter().filter(|s| matches!(s, NirStmt::Let { value: NirExpr::Call { callee, .. }, .. } if generated.contains(callee))).count(), 1);
        let NirStmt::If {
            then_body,
            else_body,
            ..
        } = &old.body[3]
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
        fn check(original: &[NirStmt], wrapped: &[NirStmt]) {
            for (old, new) in original.iter().zip(wrapped) {
                match (old, new) {
                    (
                        NirStmt::Return(Some(value)),
                        NirStmt::Let {
                            name,
                            value: actual,
                            ty,
                        },
                    ) => {
                        assert_eq!(actual, value);
                        assert_eq!(ty, &Some(scalar_type("bool")));
                        let Some(NirStmt::Return(Some(NirExpr::StructLiteral { fields, .. }))) =
                            wrapped.last()
                        else {
                            panic!()
                        };
                        assert_eq!(
                            fields,
                            &[
                                ("exited".into(), NirExpr::Bool(true)),
                                ("value".into(), NirExpr::Var(name.clone()))
                            ]
                        );
                    }
                    (
                        NirStmt::If {
                            condition,
                            then_body,
                            else_body,
                        },
                        NirStmt::If {
                            condition: actual,
                            then_body: yes,
                            else_body: no,
                        },
                    ) => {
                        assert_eq!(condition, actual);
                        check(then_body, yes);
                        check(else_body, no);
                    }
                    _ => assert_eq!(old, new),
                }
            }
            if !matches!(
                original.last(),
                Some(NirStmt::Return(_) | NirStmt::If { .. })
            ) {
                assert_eq!(wrapped.len(), original.len() + 1);
                assert!(
                    matches!(wrapped.last(), Some(NirStmt::Return(Some(NirExpr::StructLiteral { fields, .. }))) if fields[0].1 == NirExpr::Bool(false))
                );
            } else if matches!(original.last(), Some(NirStmt::Return(_))) {
                assert_eq!(wrapped.len(), original.len() + 1);
            } else {
                assert_eq!(wrapped.len(), original.len());
            }
        }
        check(then_body, yes);
        check(else_body, no);
        crate::nir_verify::verify_nir_module(&module).unwrap();
        let once = module.clone();
        assert!(outline_test(&mut module).is_empty());
        assert_eq!(module, once);
        let compiled = crate::pipeline::compile_source(&source).unwrap();
        yir_lower_llvm::emit_module(&compiled.yir).unwrap();
    }
}

#[test]
fn conditional_return_stored_signals_share_preflight_and_retain_effect_scope_and_capture_vetoes() {
    let base = source("then", "call", true, false, 2, false);
    for count in [30, 31] {
        let locals = (0..count)
            .map(|n| format!("let fresh{n} = divisor + {n};"))
            .collect::<String>();
        let source = base.replace(
            &arm("call"),
            &format!("{locals} if helper(produce(divisor)) {{ return false; }}"),
        );
        let mut module = crate::frontend::parse_nuis_module(&source).unwrap();
        let before = module.clone();
        assert_eq!(outline_test(&mut module).is_empty(), count == 31);
        if count == 31 {
            assert_eq!(module, before);
        } else {
            let compiled = crate::pipeline::compile_source(&source).unwrap();
            yir_lower_llvm::emit_module(&compiled.yir).unwrap();
        }
    }
    let admitted = base.replace(
        &arm("call"),
        "if helper(produce(divisor)) { return false; } let ignored = divisor;",
    );
    execute(&admitted, Some(19), &[99, 19], 1);
    for candidate in [
        "if gate && helper(produce(divisor)) { return false; }",
        "let selected = gate && helper(produce(divisor)); if selected { return false; }",
    ] {
        let admitted = base.replace(&arm("call"), candidate);
        execute(&admitted, Some(19), &[99, 77, 19], 0);
    }
    for candidate in [
        "if helper(produce(divisor)) { print(88); return false; }",
        "let divisor = 2; if helper(produce(divisor)) { return false; }",
        "if helper(produce(divisor)) { while gate { return false; } }",
        "if helper(produce(divisor)) { let ignored = divisor; }",
    ] {
        let mut module =
            crate::frontend::parse_nuis_module(&base.replace(&arm("call"), candidate)).unwrap();
        let before = module.clone();
        assert!(outline_test(&mut module).is_empty(), "{candidate}");
        assert_eq!(module, before);
    }
    for mutation in ["borrow", "optional", "wrong-condition", "impure"] {
        let mut module = crate::frontend::parse_nuis_module(&base).unwrap();
        if mutation == "impure" {
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
            if mutation == "wrong-condition" {
                let NirStmt::If { then_body, .. } = &mut event.body[2] else {
                    panic!()
                };
                let NirStmt::If { condition, .. } = &mut then_body[0] else {
                    panic!()
                };
                *condition = NirExpr::Int(1);
            } else {
                let ty = &mut event
                    .params
                    .iter_mut()
                    .find(|p| p.name == "divisor")
                    .unwrap()
                    .ty;
                if mutation == "borrow" {
                    ty.is_ref = true;
                } else {
                    ty.is_optional = true;
                }
            }
        }
        let before = module.clone();
        assert!(outline_test(&mut module).is_empty(), "{mutation}");
        assert_eq!(module, before);
    }
}

#[test]
fn conditional_return_stored_signals_bound_all_original_condition_and_leaf_nodes() {
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
        let NirStmt::If { then_body, .. } = &mut event.body[1] else {
            panic!()
        };
        *then_body = vec![NirStmt::If {
            condition: NirExpr::Call {
                callee: "decide".into(),
                args: vec![tree(10)],
            },
            then_body: vec![NirStmt::Return(Some(tree(10)))],
            else_body: vec![NirStmt::Let {
                name: "ignored".into(),
                ty: Some(scalar_type("i64")),
                value: NirExpr::Int(1),
            }],
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
    let mut module = crate::frontend::parse_nuis_module(&source).unwrap();
    let event = module
        .functions
        .iter_mut()
        .find(|f| f.name == "event")
        .unwrap();
    let NirStmt::If { then_body, .. } = &mut event.body[1] else {
        panic!()
    };
    let NirStmt::If { condition, .. } = &mut then_body[0] else {
        panic!()
    };
    let mut value = NirExpr::Int(1);
    for _ in 0..64 {
        value = NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(value),
            rhs: Box::new(NirExpr::Int(1)),
        };
    }
    *condition = NirExpr::Call {
        callee: "decide".into(),
        args: vec![value],
    };
    let before = module.clone();
    assert!(outline_test(&mut module).is_empty());
    assert_eq!(module, before);
}
