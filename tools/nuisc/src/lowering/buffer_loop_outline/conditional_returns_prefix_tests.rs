use super::super::tests::{execute, outline_test, source as simple_source};
use super::*;

#[path = "conditional_returns_prefix_native_tests.rs"]
mod native;

fn prefix(kind: &str, op: &str) -> String {
    match kind {
        "let" => format!("let current: i64 = divisor + 0; return gate {op} helper(produce(current));"),
        "const" => format!("const current: i64 = divisor + 0; const selected: bool = gate; return selected {op} helper(produce(current));"),
        "packet" => format!("let packet: Packet = produce(divisor); return gate {op} helper(packet);"),
        "unused" => format!("let ignored: i64 = 10 / divisor; return gate {op} helper(produce(divisor));"),
        "inferred" => format!("let selected = gate; let current = divisor + 0; const forwarded: bool = selected; return forwarded {op} helper(produce(current));"),
        "hygiene" => format!("let __nuis_return_condition_0: bool = gate; const __nuis_return_gate_0: i64 = divisor; return __nuis_return_condition_0 {op} helper(produce(__nuis_return_gate_0));"),
        _ => unreachable!(),
    }
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
    let body = prefix(kind, op);
    let selected = format!("return gate {op} helper(produce(divisor));");
    let source = simple_source(shape, op, outer, gate, divisor, early).replace(&selected, &body);
    if shape == "both" {
        source.replace(
            "else { return helper(produce(divisor)); }",
            &format!("else {{ {body} }}"),
        )
    } else {
        source
    }
}

#[test]
fn conditional_return_prefixes_preserve_local_work_two_guards_and_parent_exits() {
    for shape in ["then", "else", "both"] {
        for kind in ["let", "const", "packet", "unused", "inferred", "hygiene"] {
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
                        let entered = !early
                            && (shape == "both" || if shape == "then" { outer } else { !outer });
                        let rhs = entered && if op == "&&" { gate } else { !gate };
                        let trapped = divisor == 0
                            && (rhs || (entered && matches!(kind, "packet" | "unused")));
                        let value = entered && if rhs { divisor > 0 } else { gate };
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
                            &source(shape, kind, op, outer, gate, divisor, early),
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

#[test]
fn conditional_return_prefixes_preserve_i64_producers_and_unused_checks() {
    for shape in ["then", "else", "both"] {
        for declaration in ["let current", "const current: i64"] {
            for outer in [false, true] {
                for divisor in [0, 2, -2] {
                    let body = format!("let ignored: i64 = 20 / divisor; {declaration} = 10 / divisor; return current;");
                    let branch = match shape {
                        "then" => format!("if outer {{ {body} }}"),
                        "else" => format!("if outer {{ }} else {{ {body} }}"),
                        "both" => format!("if outer {{ {body} }} else {{ {body} }}"),
                        _ => unreachable!(),
                    };
                    let source = format!("mod cpu Main {{ @noinline fn event(outer: bool, divisor: i64) -> i64 {{ print(99); {branch} print(77); return 19; }} fn main() -> i64 {{ return event({outer}, {divisor}); }} }}");
                    let entered = shape == "both" || if shape == "then" { outer } else { !outer };
                    let expected = if entered && divisor == 0 {
                        None
                    } else if entered {
                        Some(10 / divisor)
                    } else {
                        Some(19)
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

#[test]
fn conditional_return_prefixes_preserve_complete_arm_identity_capture_hygiene_and_idempotence() {
    for shape in ["then", "else", "both"] {
        for kind in ["let", "const", "packet", "unused", "inferred", "hygiene"] {
            let mut module = crate::frontend::parse_nuis_module(&source(
                shape, kind, "&&", true, true, 2, false,
            ))
            .unwrap();
            let before = module.clone();
            let generated = outline_test(&mut module);
            assert_eq!(generated.len(), 1);
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
                assert!(!new.iter().any(|stmt| matches!(stmt, NirStmt::Let { name, .. } | NirStmt::Const { name, .. } if name == &helper.params[0].name)));
            }
            assert_eq!(
                helper
                    .params
                    .iter()
                    .skip(1)
                    .map(|p| p.name.as_str())
                    .collect::<Vec<_>>(),
                ["divisor", "gate"]
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
fn conditional_return_prefixes_retain_outer_rebind_effect_kind_scope_and_shape_vetoes() {
    let original = simple_source("then", "&&", true, true, 2, false);
    let selected = "if outer { return gate && helper(produce(divisor)); }";
    // Retain the formerly rejected suffix as independent execution evidence.
    let suffix =
        "let current = helper(produce(divisor)); if gate { return current; } return false;";
    for outer in [false, true] {
        for gate in [false, true] {
            for divisor in [0, 2, -2] {
                let source = simple_source("then", "&&", outer, gate, divisor, false)
                    .replace(selected, &format!("if outer {{ {suffix} }}"));
                let mut module = crate::frontend::parse_nuis_module(&source).unwrap();
                assert_eq!(outline_test(&mut module).len(), 1);
                let result = if outer && gate && divisor > 0 { 11 } else { 19 };
                let prints = if outer {
                    vec![99, result]
                } else {
                    vec![99, 77, result]
                };
                execute(
                    &source,
                    (!(outer && divisor == 0)).then_some(result),
                    &prints,
                    usize::from(outer),
                );
            }
        }
    }
    let admitted = original.replace(
        selected,
        "if outer { let current = gate && helper(produce(divisor)); return current; }",
    );
    execute(&admitted, Some(11), &[99, 11], 1);
    let admitted = original.replace(
        selected,
        "if outer { let current = divisor; return (gate == true) && helper(produce(current)); }",
    );
    let mut module = crate::frontend::parse_nuis_module(&admitted).unwrap();
    assert_eq!(outline_test(&mut module).len(), 1);
    execute(&admitted, Some(11), &[99, 11], 1);
    let admitted = original.replace(
        selected,
        "if outer { let current = divisor; return gate && (gate || helper(produce(current))); }",
    );
    execute(&admitted, Some(11), &[99, 11], 0);
    // Staged selected initialization now has a separate parent-print proof.
    let admitted = original.replace(
        selected,
        "if outer { let current = helper(produce(divisor)); print(88); return current; }",
    );
    let mut module = crate::frontend::parse_nuis_module(&admitted).unwrap();
    assert_eq!(outline_test(&mut module).len(), 2);
    execute(&admitted, Some(11), &[99, 88, 11], 1);
    for body in [
        "let gate: bool = helper(produce(divisor)); return gate;",
        "let current: i64 = divisor; let current: i64 = 10 / current; return current > 0;",
        "let current: bool = 10 / divisor; return current;",
        "const current: i64 = gate; return current > 0;",
        "let current = missing; return helper(produce(current));",
        "let first = current; let current = divisor; return helper(produce(first));",
        "let current = helper(produce(divisor)); if gate { print(88); } return current;",
        "let current = divisor; while gate { let current = current + 1; } return helper(produce(current));",
        "let current = divisor; return helper(produce(current)); print(88);",
        "let current = divisor; return;",
    ] {
        let mut module = match crate::frontend::parse_nuis_module(&original.replace(selected, &format!("if outer {{ {body} }}"))) {
            Ok(module) => module,
            Err(error) => {
                assert!(body.contains("current: bool = 10 / divisor")
                    || body.contains("current: i64 = gate")
                    || body.contains("= missing")
                    || body.contains("first = current")
                    || body.ends_with("return;"), "{body}: {error}");
                continue;
            }
        };
        let before = module.clone();
        assert!(outline_test(&mut module).is_empty(), "{body}");
        assert_eq!(module, before, "{body}");
    }
    for mutation in [
        "borrow",
        "optional",
        "generic",
        "effect",
        "aggregate-capture",
        "declared-kind",
        "declared-ref",
        "declared-optional",
        "declared-generic",
        "result-kind",
    ] {
        let mut module = crate::frontend::parse_nuis_module(&source(
            "then", "packet", "&&", true, true, 2, false,
        ))
        .unwrap();
        match mutation {
            "effect" => module
                .functions
                .iter_mut()
                .find(|f| f.name == "produce")
                .unwrap()
                .body
                .insert(0, NirStmt::Print(NirExpr::Int(88))),
            "aggregate-capture" => {
                let event = module
                    .functions
                    .iter_mut()
                    .find(|f| f.name == "event")
                    .unwrap();
                event.params.push(NirParam {
                    name: "original_packet".into(),
                    ty: scalar_type("Packet"),
                });
                let NirStmt::If { then_body, .. } = &mut event.body[2] else {
                    panic!()
                };
                let NirStmt::Let { value, .. } = &mut then_body[0] else {
                    panic!()
                };
                *value = NirExpr::Var("original_packet".into());
            }
            _ => {
                let event = module
                    .functions
                    .iter_mut()
                    .find(|f| f.name == "event")
                    .unwrap();
                match mutation {
                    "borrow" => event.params[2].ty.is_ref = true,
                    "optional" => event.params[2].ty.is_optional = true,
                    "generic" => event.params[2].ty.generic_args.push(scalar_type("i64")),
                    "result-kind" => event.return_type = Some(scalar_type("i64")),
                    "declared-kind" | "declared-ref" | "declared-optional" | "declared-generic" => {
                        let NirStmt::If { then_body, .. } = &mut event.body[2] else {
                            panic!()
                        };
                        let NirStmt::Let { ty: Some(ty), .. } = &mut then_body[0] else {
                            panic!()
                        };
                        match mutation {
                            "declared-kind" => *ty = scalar_type("bool"),
                            "declared-ref" => ty.is_ref = true,
                            "declared-optional" => ty.is_optional = true,
                            "declared-generic" => ty.generic_args.push(scalar_type("i64")),
                            _ => unreachable!(),
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }
        let before = module.clone();
        assert!(outline_test(&mut module).is_empty(), "{mutation}");
        assert_eq!(module, before, "{mutation}");
    }
}

#[test]
fn conditional_return_prefixes_bound_whole_arm_work_and_statement_depth_before_cloning() {
    let source = simple_source("then", "&&", true, true, 2, false);
    for count in [31, 32] {
        let locals = (0..count)
            .map(|i| format!("let fresh{i}: i64 = divisor + {i};"))
            .collect::<String>();
        let body = format!("if outer {{ {locals} return helper(produce(divisor)); }}");
        let mut module = crate::frontend::parse_nuis_module(&source.replace(
            "if outer { return gate && helper(produce(divisor)); }",
            &body,
        ))
        .unwrap();
        let before = module.clone();
        let generated = outline_test(&mut module);
        assert_eq!(generated.is_empty(), count == 32);
        if count == 32 {
            assert_eq!(module, before);
        }
    }
    let binding = |value| NirStmt::Let {
        name: "local".into(),
        ty: Some(scalar_type("i64")),
        value,
    };
    let returned = NirStmt::Return(Some(NirExpr::Bool(true)));
    let mut linear = NirExpr::Int(1);
    for _ in 0..64 {
        linear = NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(NirExpr::Int(1)),
            rhs: Box::new(linear),
        };
    }
    assert!(!conditional_values::prefix::return_body(&[
        binding(linear),
        returned.clone()
    ]));
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
    assert!(conditional_values::prefix::return_body(&[
        binding(tree(11)),
        returned.clone()
    ]));
    assert!(!conditional_values::prefix::return_body(&[
        binding(tree(12)),
        returned
    ]));
    let total = |count| {
        (0..count)
            .map(|i| NirStmt::Let {
                name: format!("local{i}"),
                ty: Some(scalar_type("i64")),
                value: tree(10),
            })
            .chain(std::iter::once(NirStmt::Return(Some(NirExpr::Bool(true)))))
            .collect::<Vec<_>>()
    };
    assert!(conditional_values::prefix::return_body(&total(2)));
    assert!(!conditional_values::prefix::return_body(&total(3)));
}
