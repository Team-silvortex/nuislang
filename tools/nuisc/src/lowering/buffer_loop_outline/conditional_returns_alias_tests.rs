use super::super::tests::{execute, outline_test, source as simple_source};
use super::continuation_tests::assert_seed_after_work;
use super::*;

#[path = "conditional_returns_alias_native_tests.rs"]
mod native;

fn arm(reversed: bool, op: &str) -> String {
    let returned = format!("return gate {op} helper(produce(divisor));");
    let continued = "let packet = produce(divisor); let ignored = helper(packet);";
    let (yes, no) = if reversed {
        (continued.to_owned(), returned)
    } else {
        (returned, continued.to_owned())
    };
    format!("let first = nested; const second: bool = first; let selected: bool = second; if selected {{ {yes} }} else {{ {no} }}")
}

fn source(
    shape: &str,
    op: &str,
    outer: bool,
    nested: bool,
    gate: bool,
    divisor: i64,
    early: bool,
) -> String {
    let branch = match shape {
        "then" => format!("if outer {{ {} }}", arm(false, op)),
        "else" => format!("if outer {{ }} else {{ {} }}", arm(false, op)),
        "both" => format!(
            "if outer {{ {} }} else {{ {} }}",
            arm(false, op),
            arm(true, op)
        ),
        _ => unreachable!(),
    };
    simple_source("then", op, outer, gate, divisor, early)
        .replace("early: bool) -> bool", "early: bool, nested: bool) -> bool")
        .replace(
            &format!("event({outer}, {gate}, {divisor}, {early})"),
            &format!("event({outer}, {gate}, {divisor}, {early}, {nested})"),
        )
        .replace(
            &format!("if outer {{ return gate {op} helper(produce(divisor)); }}"),
            &branch,
        )
}

#[test]
fn conditional_return_aliases_keep_selected_calls_short_circuit_and_actual_exits() {
    let mut cases = 0;
    for shape in ["then", "else", "both"] {
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
                            && match shape {
                                "then" => outer,
                                "else" => !outer,
                                "both" => true,
                                _ => unreachable!(),
                            };
                        let returned = entered
                            && if shape == "both" {
                                outer == nested
                            } else {
                                nested
                            };
                        let continuation = entered && !returned;
                        let rhs = returned && if op == "&&" { gate } else { !gate };
                        let trapped = divisor == 0 && (rhs || continuation);
                        let value = returned && if rhs { divisor > 0 } else { gate };
                        let result = if value { 11 } else { 19 };
                        let mut prints = Vec::new();
                        if !early {
                            prints.push(99);
                            if !returned {
                                prints.push(77);
                            }
                        }
                        prints.push(result);
                        execute(
                            &source(shape, op, outer, nested, gate, divisor, early),
                            (!trapped).then_some(result),
                            &prints,
                            usize::from(rhs) + usize::from(continuation),
                        );
                        cases += 1;
                    }
                }
            }
        }
    }
    assert_eq!(cases, 144);
}

#[test]
fn conditional_return_aliases_keep_literal_atoms_and_current_parent_binding_versions() {
    for literal in [false, true] {
        for outer in [false, true] {
            let source = source("then", "&&", outer, !literal, false, 2, false);
            let literal_source = source.replace(
                "let first = nested;",
                &format!("const first: bool = {literal};"),
            );
            let current_source = source.replace(
                "print(99);",
                &format!("let nested: bool = {literal}; print(99);"),
            );
            for source in [literal_source, current_source] {
                let returned = outer && literal;
                execute(
                    &source,
                    Some(19),
                    if returned { &[99, 19] } else { &[99, 77, 19] },
                    usize::from(outer && !literal),
                );
            }
        }
    }
}

fn integer_source(outer: bool, nested: bool, divisor: i64) -> String {
    format!("mod cpu Main {{ @noinline fn event(outer: bool, nested: bool, divisor: i64) -> i64 {{ print(99); if outer {{ const first: bool = nested; let selected = first; if selected {{ let current = 10 / divisor - 5; return current; }} else {{ let ignored = 10 / divisor; }} }} print(77); return 19; }} fn main() -> i64 {{ return event({outer}, {nested}, {divisor}); }} }}")
}

#[test]
fn conditional_return_aliases_distinguish_zero_results_and_preserve_selected_prefix_checks() {
    for outer in [false, true] {
        for nested in [false, true] {
            for divisor in [0, 2, -2] {
                let returned = outer && nested;
                let result = if returned {
                    10 / if divisor == 0 { 1 } else { divisor } - 5
                } else {
                    19
                };
                execute(
                    &integer_source(outer, nested, divisor),
                    (!(outer && divisor == 0)).then_some(result),
                    if returned { &[99] } else { &[99, 77] },
                    0,
                );
            }
        }
    }
    for outer in [false, true] {
        for nested in [false, true] {
            for divisor in [0, 2] {
                let source = source("then", "&&", outer, nested, false, divisor, false)
                    .replace("let first = nested;", "let before = produce(divisor); let observed = helper(before); let first = nested;");
                let returned = outer && nested;
                execute(
                    &source,
                    (!(outer && divisor == 0)).then_some(19),
                    if returned { &[99, 19] } else { &[99, 77, 19] },
                    usize::from(outer) + usize::from(outer && !nested),
                );
            }
        }
    }
}

#[test]
fn conditional_return_aliases_do_not_speculate_parent_suffix_after_false_returns() {
    for outer in [false, true] {
        for nested in [false, true] {
            let source = source("then", "&&", outer, nested, false, 2, false).replace(
                "print(77); return false;",
                "print(77); return helper(produce(0));",
            );
            execute(&source, (outer && nested).then_some(19), &[99, 19], 0);
        }
    }
}

#[test]
fn conditional_return_aliases_keep_nested_sibling_alias_identities_independent() {
    for outer in [false, true] {
        for nested in [false, true] {
            for gate in [false, true] {
                for divisor in [0, 2, -2] {
                    let tree = "let first = nested; if first { let selected = gate; if selected { return helper(produce(divisor)); } else { let ignored = 10 / divisor; } } else { const selected: bool = outer; if selected { return false; } else { let ignored = 10 / divisor; } }";
                    let source = source("both", "&&", outer, nested, gate, divisor, false)
                        .replace(&arm(false, "&&"), tree)
                        .replace(&arm(true, "&&"), tree);
                    let returned = (nested && gate) || (!nested && outer);
                    let called = nested && gate;
                    let result = if called && divisor > 0 { 11 } else { 19 };
                    let prints = if returned {
                        vec![99, result]
                    } else {
                        vec![99, 77, result]
                    };
                    execute(
                        &source,
                        (!(divisor == 0 && (called || !returned))).then_some(result),
                        &prints,
                        usize::from(called),
                    );
                }
            }
        }
    }
}

#[test]
fn conditional_return_aliases_flatten_readiness_only_keep_lexical_bodies_and_idempotence() {
    let source = source("both", "&&", true, true, true, 2, false);
    for prefix in [
        "let first = nested;",
        "const first: bool = true;",
        "let __nuis_return_condition_0 = nested; let first = __nuis_return_condition_0;",
        "let first = nested; const unused: bool = gate;",
    ] {
        let mut module =
            crate::frontend::parse_nuis_module(&source.replace("let first = nested;", prefix))
                .unwrap();
        let before = module.clone();
        let generated = outline_test(&mut module);
        assert_eq!(generated.len(), 1);
        let old = before.functions.iter().find(|f| f.name == "event").unwrap();
        let new = module.functions.iter().find(|f| f.name == "event").unwrap();
        assert_eq!(&new.body[..2], &old.body[..2]);
        let captures = if prefix.contains("nested") {
            vec!["divisor", "gate", "nested"]
        } else {
            vec!["divisor", "gate"]
        };
        assert_eq!(
            &new.body[new.body.len() - 2..],
            &old.body[old.body.len() - 2..]
        );
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
            captures
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
        assert_seed_after_work(then_body, yes);
        assert_seed_after_work(else_body, no);
        for stmt in &new.body {
            if let NirStmt::Let { name, value, .. } = stmt {
                if name.starts_with("__nuis_return_exit") {
                    let mut used = BTreeSet::new();
                    control_values::collect_inputs(value, &mut used);
                    assert!(
                        used.iter()
                            .all(|name| name == "nested" || name.starts_with("__nuis_return_gate")),
                        "{used:?}"
                    );
                    assert!(super::tests::total_readiness(value));
                }
            }
        }
        crate::nir_verify::verify_nir_module(&module).unwrap();
        let once = module.clone();
        assert!(outline_test(&mut module).is_empty());
        assert_eq!(module, once);
    }
}

#[test]
fn conditional_return_aliases_share_budgets_before_alias_proof_and_leave_rejections_unchanged() {
    let base = source("then", "&&", true, true, true, 2, false);
    let admitted = base.replace(
        &arm(false, "&&"),
        "let selected: bool = nested == true; if selected { return helper(produce(divisor)); }",
    );
    let compiled = crate::pipeline::compile_source(&admitted).unwrap();
    yir_lower_llvm::emit_module(&compiled.yir).unwrap();
    for count in [30, 31] {
        let mut locals = "let a0 = nested;".to_owned();
        for n in 1..count {
            locals.push_str(&format!("const a{n}: bool = a{};", n - 1));
        }
        let replacement = format!(
            "{locals} if a{} {{ return gate && helper(produce(divisor)); }}",
            count - 1
        );
        let source = base.replace(&arm(false, "&&"), &replacement);
        let mut module = crate::frontend::parse_nuis_module(&source).unwrap();
        let before = module.clone();
        if count == 30 {
            assert_eq!(outline_test(&mut module).len(), 1);
            let compiled = crate::pipeline::compile_source(&source).unwrap();
            yir_lower_llvm::emit_module(&compiled.yir).unwrap();
        } else {
            assert!(outline_test(&mut module).is_empty());
            assert_eq!(module, before);
        }
    }
    for candidate in [
        "let current = divisor + 0; let selected: bool = current == 0; if selected { return helper(produce(divisor)); }",
        "let selected: bool = helper(produce(divisor)); if selected { return helper(produce(divisor)); }",
        "let packet = produce(divisor); let selected = packet.value > 0; if selected { return helper(packet); }",
        "let first = helper(produce(divisor)); let selected = first; if selected { return helper(produce(divisor)); }",
    ] {
        let admitted = base.replace(&arm(false, "&&"), candidate);
        let compiled = crate::pipeline::compile_source(&admitted).unwrap();
        yir_lower_llvm::emit_module(&compiled.yir).unwrap();
    }
    let admitted = base.replace(
        &arm(false, "&&"),
        "let first = nested; if first { return helper(produce(divisor)); } let ignored = divisor;",
    );
    execute(&admitted, Some(11), &[99, 11], 1);
    let admitted = base.replace(
        &arm(false, "&&"),
        "let selected = gate && helper(produce(divisor)); if selected { return helper(produce(divisor)); }",
    );
    execute(&admitted, Some(11), &[99, 11], 2);
    for candidate in [
        "let first = nested; let first = gate; if first { return helper(produce(divisor)); }",
        "let nested = gate; if nested { return helper(produce(divisor)); }",
        "let first = nested; if first { print(88); return helper(produce(divisor)); }",
        "if nested { let selected = gate; return helper(produce(divisor)); } else { if selected { return helper(produce(divisor)); } }",
    ] {
        let source = base.replace(&arm(false, "&&"), candidate);
        let mut module = match crate::frontend::parse_nuis_module(&source) {
            Ok(module) => module,
            Err(error) => {
                assert!(candidate.contains("else { if selected"));
                assert!(error.contains("unknown value `selected`"), "{error}");
                assert!(crate::pipeline::compile_source(&source).is_err());
                continue;
            }
        };
        let before = module.clone();
        assert!(outline_test(&mut module).is_empty(), "{candidate}");
        assert_eq!(module, before, "{candidate}");
    }
}
