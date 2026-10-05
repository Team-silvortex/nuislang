use super::super::tests::{execute, outline_test, source as simple_source};
use super::*;

#[path = "conditional_returns_terminal_native_tests.rs"]
mod native;

fn body(kind: &str, op: &str) -> String {
    match kind {
        "leaf" => format!("if nested {{ let current = divisor + 0; return gate {op} helper(produce(current)); }} else {{ const skipped: bool = false; return skipped; }}"),
        "local" => format!("const selected: bool = nested; let current = divisor + 0; if selected {{ return gate {op} helper(produce(current)); }} else {{ return false; }}"),
        "packet" => format!("let packet = produce(divisor); if nested {{ return gate {op} helper(packet); }} else {{ return false; }}"),
        "hygiene" => format!("if nested {{ let __nuis_return_condition_0: bool = gate; return __nuis_return_condition_0 {op} helper(produce(divisor)); }} else {{ const __nuis_return_condition_0: bool = false; return __nuis_return_condition_0; }}"),
        _ => unreachable!(),
    }
}

fn source(
    shape: &str,
    kind: &str,
    op: &str,
    outer: bool,
    nested: bool,
    gate: bool,
    divisor: i64,
    early: bool,
) -> String {
    let arm = body(kind, op);
    simple_source(shape, op, outer, gate, divisor, early)
        .replace("early: bool) -> bool", "early: bool, nested: bool) -> bool")
        .replace(
            &format!("event({outer}, {gate}, {divisor}, {early})"),
            &format!("event({outer}, {gate}, {divisor}, {early}, {nested})"),
        )
        .replace(&format!("return gate {op} helper(produce(divisor));"), &arm)
        .replace(
            "else { return helper(produce(divisor)); }",
            &format!("else {{ {arm} }}"),
        )
}

#[test]
fn conditional_return_terminal_trees_preserve_three_guards_prefix_checks_and_actual_exits() {
    for shape in ["then", "else", "both"] {
        for kind in ["leaf", "local", "packet"] {
            for outer in [false, true] {
                for nested in [false, true] {
                    for early in [false, true] {
                        for (op, gate, divisor) in [
                            ("&&", false, 0),
                            ("||", true, 0),
                            ("&&", true, 2),
                            ("||", false, -2),
                            ("&&", true, 0),
                            ("||", false, 0),
                        ] {
                            let entered = !early
                                && (shape == "both"
                                    || if shape == "then" { outer } else { !outer });
                            let rhs = entered && nested && if op == "&&" { gate } else { !gate };
                            let trapped = divisor == 0 && (rhs || (entered && kind == "packet"));
                            let value = entered && nested && if rhs { divisor > 0 } else { gate };
                            let result = if value { 11 } else { 19 };
                            let mut prints = Vec::new();
                            if !early {
                                prints.push(99);
                                if !entered {
                                    prints.push(77);
                                }
                            }
                            prints.push(result);
                            execute(
                                &source(shape, kind, op, outer, nested, gate, divisor, early),
                                (!trapped).then_some(result),
                                &prints,
                                usize::from(rhs),
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn conditional_return_terminal_trees_preserve_selected_i64_leaf_checks() {
    for shape in ["then", "else", "both"] {
        for declared in ["let current", "const current: i64"] {
            for outer in [false, true] {
                for nested in [false, true] {
                    for divisor in [0, 2, -2] {
                        let arm = format!("if nested {{ {declared} = 10 / divisor; return current; }} else {{ return 23; }}");
                        let branch = match shape {
                            "then" => format!("if outer {{ {arm} }}"),
                            "else" => format!("if outer {{ }} else {{ {arm} }}"),
                            "both" => format!("if outer {{ {arm} }} else {{ {arm} }}"),
                            _ => unreachable!(),
                        };
                        let source = format!("mod cpu Main {{ @noinline fn event(outer: bool, nested: bool, divisor: i64) -> i64 {{ print(99); {branch} print(77); return 19; }} fn main() -> i64 {{ return event({outer}, {nested}, {divisor}); }} }}");
                        let entered =
                            shape == "both" || if shape == "then" { outer } else { !outer };
                        let expected = if entered && nested && divisor == 0 {
                            None
                        } else if !entered {
                            Some(19)
                        } else if nested {
                            Some(10 / divisor)
                        } else {
                            Some(23)
                        };
                        execute(
                            &source,
                            expected,
                            if entered { &[99] } else { &[99, 77] },
                            0,
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn conditional_return_terminal_trees_keep_complete_arms_scalar_captures_hygiene_and_idempotence() {
    for shape in ["then", "else", "both"] {
        for kind in ["leaf", "local", "packet", "hygiene"] {
            let mut module = crate::frontend::parse_nuis_module(&source(
                shape, kind, "&&", true, true, true, 2, false,
            ))
            .unwrap();
            let before = module.clone();
            let generated = outline_test(&mut module);
            assert_eq!(generated.len(), 1, "{shape}/{kind}");
            let original = before.functions.iter().find(|f| f.name == "event").unwrap();
            let after = module.functions.iter().find(|f| f.name == "event").unwrap();
            assert_eq!(&after.body[..2], &original.body[..2]);
            assert_eq!(
                &after.body[after.body.len() - 2..],
                &original.body[original.body.len() - 2..]
            );
            let helper = module
                .functions
                .iter()
                .find(|f| generated.contains(&f.name))
                .unwrap();
            let NirStmt::If {
                then_body,
                else_body,
                ..
            } = &original.body[2]
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
            for (old, new) in [(then_body, yes), (else_body, no)] {
                if !old.is_empty() {
                    assert_eq!(old, new);
                }
                let mut bindings = BTreeSet::new();
                branches::collect_bindings(new, &mut bindings);
                assert!(!bindings.contains(&helper.params[0].name));
            }
            assert_eq!(
                helper
                    .params
                    .iter()
                    .skip(1)
                    .map(|p| p.name.as_str())
                    .collect::<Vec<_>>(),
                ["divisor", "gate", "nested"]
            );
            for function in &before.functions {
                if function.name != "event" {
                    assert_eq!(
                        module
                            .functions
                            .iter()
                            .find(|f| f.name == function.name)
                            .unwrap(),
                        function
                    );
                }
            }
            crate::nir_verify::verify_nir_module(&module).unwrap();
            let once = module.clone();
            assert!(outline_test(&mut module).is_empty());
            assert_eq!(module, once);
        }
    }
}

#[test]
fn conditional_return_terminal_trees_retain_fallthrough_effect_rebind_scope_kind_and_capture_vetoes(
) {
    let original = source("then", "leaf", "&&", true, true, true, 2, false);
    let selected = body("leaf", "&&");
    let admitted = original.replace(
        &selected,
        "let selected: bool = nested; if selected { return helper(produce(divisor)); }",
    );
    let compiled = crate::pipeline::compile_source(&admitted).unwrap();
    yir_lower_llvm::emit_module(&compiled.yir).unwrap();
    let admitted = original.replace(
        &selected,
        "if nested == true { return helper(produce(divisor)); } else { return false; }",
    );
    let compiled = crate::pipeline::compile_source(&admitted).unwrap();
    yir_lower_llvm::emit_module(&compiled.yir).unwrap();
    let admitted = admitted.replace("= nested;", "= nested == true;");
    let compiled = crate::pipeline::compile_source(&admitted).unwrap();
    yir_lower_llvm::emit_module(&compiled.yir).unwrap();
    for arm in [
        "let selected: bool = helper(produce(divisor)); if selected { return helper(produce(divisor)); }",
    ] {
        let admitted = original.replace(&selected, arm);
        let compiled = crate::pipeline::compile_source(&admitted).unwrap();
        yir_lower_llvm::emit_module(&compiled.yir).unwrap();
    }
    let admitted = original.replace(
        &selected,
        "if nested { let selected = gate && helper(produce(divisor)); return selected; } else { return false; }",
    );
    execute(&admitted, Some(11), &[99, 11], 1);
    let admitted = original.replace(
        &selected,
        "if nested { return gate && (gate || helper(produce(divisor))); } else { return false; }",
    );
    execute(&admitted, Some(11), &[99, 11], 0);
    for arm in [
        "if nested { return helper(produce(divisor)); } else { print(88); return false; }",
        "if nested { let gate: bool = true; return helper(produce(divisor)); } else { return false; }",
        "if nested { let current = divisor; let current = divisor; return helper(produce(current)); } else { return false; }",
        "if nested { while gate { return helper(produce(divisor)); } return false; } else { return false; }",
        "if nested { return helper(produce(divisor)); } else { let selected = gate; print(88); return selected; }",
        "if nested { return helper(produce(divisor)); } else { return false; } return false;",
        "if nested { let current = divisor; return helper(produce(current)); } else { return helper(produce(current)); }",
    ] {
        let mut module = match crate::frontend::parse_nuis_module(&original.replace(&selected, arm)) {
            Ok(module) => module,
            Err(error) => {
                assert!(arm.ends_with("else { return helper(produce(current)); }"), "{arm}: {error}");
                continue;
            }
        };
        let before = module.clone();
        assert!(outline_test(&mut module).is_empty(), "{arm}");
        assert_eq!(module, before, "{arm}");
    }
    for mutation in [
        "borrow",
        "optional",
        "generic",
        "effect",
        "aggregate",
        "declared",
        "result",
        "async",
    ] {
        let mut module = crate::frontend::parse_nuis_module(&source(
            "then", "packet", "&&", true, true, true, 2, false,
        ))
        .unwrap();
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
            match mutation {
                "borrow" => event.params[4].ty.is_ref = true,
                "optional" => event.params[4].ty.is_optional = true,
                "generic" => event.params[4].ty.generic_args.push(scalar_type("bool")),
                "result" => event.return_type = Some(scalar_type("i64")),
                "async" => event.is_async = true,
                "aggregate" | "declared" => {
                    let NirStmt::If { then_body, .. } = &mut event.body[2] else {
                        panic!()
                    };
                    let NirStmt::Let { value, ty, .. } = &mut then_body[0] else {
                        panic!()
                    };
                    if mutation == "aggregate" {
                        *value = NirExpr::Var("original_packet".into());
                        event.params.push(NirParam {
                            name: "original_packet".into(),
                            ty: scalar_type("Packet"),
                        });
                    } else {
                        *ty = Some(scalar_type("bool"));
                    }
                }
                _ => unreachable!(),
            }
        }
        let before = module.clone();
        assert!(outline_test(&mut module).is_empty(), "{mutation}");
        assert_eq!(module, before, "{mutation}");
    }
}

#[test]
fn conditional_return_terminal_trees_share_statement_expression_and_depth_budgets() {
    for count in [29, 30] {
        let locals = (0..count)
            .map(|i| format!("let fresh{i} = divisor + {i};"))
            .collect::<String>();
        let mut module = crate::frontend::parse_nuis_module(&source("then", "leaf", "&&", true, true, true, 2, false).replace(&body("leaf", "&&"), &format!("{locals} if nested {{ return helper(produce(divisor)); }} else {{ return false; }}"))).unwrap();
        let before = module.clone();
        assert_eq!(outline_test(&mut module).is_empty(), count == 30);
        if count == 30 {
            assert_eq!(module, before);
        }
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
    let binding = |name: &str| NirStmt::Let {
        name: name.into(),
        ty: Some(scalar_type("i64")),
        value: NirExpr::Int(1),
    };
    let branch = |value| NirStmt::If {
        condition: NirExpr::Bool(true),
        then_body: vec![NirStmt::Return(Some(value))],
        else_body: vec![NirStmt::Return(Some(tree(10)))],
    };
    let exact = vec![binding("one"), branch(tree(10))];
    assert!(bounded(&exact));
    let mut excess = exact.clone();
    excess.insert(0, binding("two"));
    assert!(!bounded(&excess));
    let mut linear = NirExpr::Int(1);
    for _ in 0..64 {
        linear = NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(NirExpr::Int(1)),
            rhs: Box::new(linear),
        };
    }
    assert!(!bounded(&[branch(linear)]));
}

#[test]
fn conditional_return_terminal_trees_keep_deep_lexical_paths_and_sibling_local_identity() {
    let original = source("then", "leaf", "&&", true, true, true, 2, false);
    let mut arm = "if nested { let current = divisor; return gate && helper(produce(current)); } else { const current: bool = false; return current; }".to_owned();
    for _ in 0..3 {
        let source = original.replace(&body("leaf", "&&"), &arm);
        execute(&source, Some(11), &[99, 11], 1);
        arm = format!("if nested {{ {arm} }} else {{ let current = false; return current; }}");
    }
}
