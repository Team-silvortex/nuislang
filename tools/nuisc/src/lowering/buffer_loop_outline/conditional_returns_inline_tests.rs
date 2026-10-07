use super::super::tests::{
    assert_selected_effect_not_pure, execute, outline_test, source as simple_source,
};
use super::continuation_tests::assert_seed_after_work;
use super::*;

#[path = "conditional_returns_inline_native_tests.rs"]
mod native;

const OPS: [&str; 6] = ["==", "!=", "<", "<=", ">", ">="];

fn compare(op: &str, lhs: i64, rhs: i64) -> bool {
    match op {
        "==" => lhs == rhs,
        "!=" => lhs != rhs,
        "<" => lhs < rhs,
        "<=" => lhs <= rhs,
        ">" => lhs > rhs,
        ">=" => lhs >= rhs,
        _ => unreachable!(),
    }
}

fn arm(condition: &str, op: &str, reversed: bool) -> String {
    let returned = format!("return gate {op} helper(produce(divisor));");
    let continued = "let packet = produce(divisor); let ignored = helper(packet);";
    let (yes, no) = if reversed {
        (continued.to_owned(), returned)
    } else {
        (returned, continued.to_owned())
    };
    format!("let input = divisor; const zero: i64 = 0; if {condition} {{ {yes} }} else {{ {no} }}")
}

fn source(
    shape: &str,
    condition: &str,
    op: &str,
    outer: bool,
    gate: bool,
    divisor: i64,
    early: bool,
) -> String {
    let branch = match shape {
        "then" => format!("if outer {{ {} }}", arm(condition, op, false)),
        "else" => format!("if outer {{ }} else {{ {} }}", arm(condition, op, false)),
        "both" => format!(
            "if outer {{ {} }} else {{ {} }}",
            arm(condition, op, false),
            arm(condition, op, true)
        ),
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
    op: &str,
    outer: bool,
    selected: bool,
    gate: bool,
    divisor: i64,
) {
    let entered = shape == "both" || if shape == "then" { outer } else { !outer };
    let returned = entered
        && if shape == "both" {
            outer == selected
        } else {
            selected
        };
    let continuation = entered && !returned;
    let rhs = returned && if op == "&&" { gate } else { !gate };
    let value = returned && if rhs { divisor > 0 } else { gate };
    let result = if value { 11 } else { 19 };
    let prints = if returned {
        vec![99, result]
    } else {
        vec![99, 77, result]
    };
    execute(
        source,
        (!(divisor == 0 && (rhs || continuation))).then_some(result),
        &prints,
        usize::from(rhs) + usize::from(continuation),
    );
}

#[test]
fn conditional_return_inline_comparisons_keep_signed_operators_selected_work_and_exits() {
    let mut cases = 0;
    for comparison in OPS {
        for shape in ["then", "else", "both"] {
            for outer in [false, true] {
                for (op, gate, divisor) in [
                    ("&&", false, 0),
                    ("||", true, 0),
                    ("&&", true, 2),
                    ("||", false, -2),
                    ("&&", true, 0),
                    ("||", false, 0),
                ] {
                    let condition = format!("input {comparison} zero");
                    check(
                        &source(shape, &condition, op, outer, gate, divisor, false),
                        shape,
                        op,
                        outer,
                        compare(comparison, divisor, 0),
                        gate,
                        divisor,
                    );
                    cases += 1;
                }
            }
        }
    }
    assert_eq!(cases, 216);
}

#[test]
fn conditional_return_inline_comparisons_keep_bool_literals_and_current_atoms() {
    let mut cases = 0;
    for (condition, comparison, literal) in [
        ("other == gate", "==", None),
        ("other != gate", "!=", None),
        ("other == true", "==", Some(true)),
        ("other != false", "!=", Some(false)),
    ] {
        for shape in ["then", "else", "both"] {
            for outer in [false, true] {
                for other in [false, true] {
                    for (op, gate, divisor) in [
                        ("&&", true, 2),
                        ("&&", false, 0),
                        ("||", false, -2),
                        ("||", true, 0),
                    ] {
                        let source = source(shape, condition, op, outer, gate, divisor, false)
                            .replace(
                                "const zero: i64 = 0;",
                                &format!("const zero: i64 = 0; const other: bool = {other};"),
                            );
                        let selected = compare(
                            comparison,
                            i64::from(other),
                            i64::from(literal.unwrap_or(gate)),
                        );
                        check(&source, shape, op, outer, selected, gate, divisor);
                        cases += 1;
                    }
                }
            }
        }
    }
    assert_eq!(cases, 192);
}

fn integer_source(comparison: &str, outer: bool, divisor: i64) -> String {
    format!("mod cpu Main {{ @noinline fn event(outer: bool, divisor: i64) -> i64 {{ print(99); if outer {{ const input: i64 = divisor; let zero = 0; if input {comparison} zero {{ let current = 10 / divisor - 5; return current; }} else {{ let ignored = 10 / divisor; }} }} print(77); return 19; }} fn main() -> i64 {{ return event({outer}, {divisor}); }} }}")
}

#[test]
fn conditional_return_inline_comparisons_keep_zero_results_extremes_and_parent_rebinding() {
    let mut cases = 0;
    for comparison in OPS {
        for outer in [false, true] {
            for divisor in [0, 2, -2] {
                let returned = outer && compare(comparison, divisor, 0);
                let result = if returned {
                    10 / if divisor == 0 { 1 } else { divisor } - 5
                } else {
                    19
                };
                execute(
                    &integer_source(comparison, outer, divisor),
                    (!(outer && divisor == 0)).then_some(result),
                    if returned { &[99] } else { &[99, 77] },
                    0,
                );
                cases += 1;
            }
        }
        for divisor in [-i64::MAX, i64::MAX] {
            check(
                &source(
                    "both",
                    &format!("zero {comparison} input"),
                    "&&",
                    true,
                    false,
                    divisor,
                    false,
                ),
                "both",
                "&&",
                true,
                compare(comparison, 0, divisor),
                false,
                divisor,
            );
            cases += 1;
        }
    }
    for comparison in [">", "<"] {
        let source = source(
            "both",
            &format!("input {comparison} zero"),
            "&&",
            true,
            false,
            0,
            false,
        )
        .replace("print(99);", "let divisor: i64 = 2; print(99);");
        check(
            &source,
            "both",
            "&&",
            true,
            compare(comparison, 2, 0),
            false,
            2,
        );
        cases += 1;
    }
    assert_eq!(cases, 50);
}

#[test]
fn conditional_return_inline_comparisons_keep_prefix_checks_early_exits_and_parent_suffixes() {
    let mut cases = 0;
    for outer in [false, true] {
        for selected in [false, true] {
            for divisor in [0, 2] {
                let source = source("then", &format!("{selected} == true"), "&&", outer, false, divisor, false)
                    .replace("let input = divisor;", "let before = produce(divisor); let observed = helper(before); let input = divisor;");
                let returned = outer && selected;
                execute(
                    &source,
                    (!(outer && divisor == 0)).then_some(19),
                    if returned { &[99, 19] } else { &[99, 77, 19] },
                    usize::from(outer) + usize::from(outer && !selected),
                );
                cases += 1;
                let source = source.replace(
                    &format!("event({outer}, false, {divisor}, false)"),
                    &format!("event({outer}, false, {divisor}, true)"),
                );
                execute(&source, Some(19), &[19], 0);
                cases += 1;
            }
            let source = source(
                "then",
                &format!("{selected} == true"),
                "&&",
                outer,
                false,
                2,
                false,
            )
            .replace(
                "print(77); return false;",
                "print(77); return helper(produce(0));",
            );
            execute(&source, (outer && selected).then_some(19), &[99, 19], 0);
            cases += 1;
        }
    }
    assert_eq!(cases, 20);
}

#[test]
fn conditional_return_inline_comparisons_isolate_sibling_atoms_and_current_bool_versions() {
    let mut cases = 0;
    for outer in [false, true] {
        for gate in [false, true] {
            for divisor in [0, 2, -2] {
                for current in [false, true] {
                    let tree = "if gate == true { let input = divisor; if input > 0 { return helper(produce(divisor)); } else { let ignored = 10 / divisor; } } else { const input: i64 = divisor; if input == 0 { return false; } else { let ignored = 10 / divisor; } }";
                    let source = source("both", "input == zero", "&&", outer, gate, divisor, false)
                        .replace(&arm("input == zero", "&&", false), tree)
                        .replace(&arm("input == zero", "&&", true), tree)
                        .replace(
                            "print(99);",
                            &format!("let gate: bool = {current}; print(99);"),
                        );
                    let returned = if current { divisor > 0 } else { divisor == 0 };
                    let called = current && divisor > 0;
                    let result = if called { 11 } else { 19 };
                    let prints = if returned {
                        vec![99, result]
                    } else {
                        vec![99, 77, result]
                    };
                    execute(
                        &source,
                        (!(current && divisor == 0)).then_some(result),
                        &prints,
                        usize::from(called),
                    );
                    cases += 1;
                }
            }
        }
    }
    assert_eq!(cases, 24);
}

#[test]
fn conditional_return_inline_comparisons_retain_bodies_captures_hygiene_and_idempotence() {
    for condition in [
        "input > zero",
        "gate == true",
        "outer != gate",
        "false == true",
    ] {
        let source = source("both", condition, "&&", true, true, 2, false).replace(
            "let input = divisor;",
            "let __nuis_return_condition_0 = divisor; let input = __nuis_return_condition_0;",
        );
        let mut module = crate::frontend::parse_nuis_module(&source).unwrap();
        let before = module.clone();
        let generated = outline_test(&mut module);
        assert_eq!(generated.len(), 1);
        let old = before.functions.iter().find(|f| f.name == "event").unwrap();
        let new = module.functions.iter().find(|f| f.name == "event").unwrap();
        assert_eq!(&new.body[..2], &old.body[..2]);
        assert_eq!(
            &new.body[new.body.len() - 2..],
            &old.body[old.body.len() - 2..]
        );
        let helper = module
            .functions
            .iter()
            .find(|f| generated.contains(&f.name))
            .unwrap();
        let mut captures = vec!["divisor", "gate"];
        if condition.contains("outer") {
            captures.push("outer");
        }
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
        for (original, seeded) in [(then_body, yes), (else_body, no)] {
            assert_seed_after_work(original, seeded);
            let mut bindings = BTreeSet::new();
            branches::collect_bindings(seeded, &mut bindings);
            assert!(!bindings.contains(&helper.params[0].name));
        }
        for stmt in &new.body {
            if let NirStmt::Let { name, value, .. } = stmt {
                if name.starts_with("__nuis_return_exit") {
                    let mut used = BTreeSet::new();
                    control_values::collect_inputs(value, &mut used);
                    assert!(
                        used.iter()
                            .all(|name| ["divisor", "gate", "outer"].contains(&name.as_str())
                                || name.starts_with("__nuis_return_gate")),
                        "{used:?}"
                    );
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
fn conditional_return_inline_comparisons_share_budgets_and_retain_non_total_vetoes() {
    let base = source("then", "input > zero", "&&", true, true, 2, false);
    let complete = base.replace(
        &arm("input > zero", "&&", false),
        "if divisor > 0 { return helper(produce(divisor)); } else { return false; }",
    );
    let compiled = crate::pipeline::compile_source(&complete).unwrap();
    yir_lower_llvm::emit_module(&compiled.yir).unwrap();
    for count in [30, 31] {
        let mut locals = "let a0: i64 = divisor;".to_owned();
        for n in 1..count {
            locals.push_str(&format!("const a{n}: i64 = a{};", n - 1));
        }
        let candidate = format!(
            "{locals} if a{} > 0 {{ return helper(produce(divisor)); }}",
            count - 1
        );
        let source = base.replace(&arm("input > zero", "&&", false), &candidate);
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
        "if helper(produce(divisor)) == true { return helper(produce(divisor)); }",
        "let selected = helper(produce(divisor)); if selected == true { return helper(produce(divisor)); }",
        "let packet = produce(divisor); if packet.value > 0 { return helper(packet); }",
        "let packet = produce(divisor); let input = packet.value; if input > 0 { return helper(packet); }",
        "if divisor / divisor == 0 { return helper(produce(divisor)); }",
        "if divisor + 0 > 0 { return helper(produce(divisor)); }",
        "let input = divisor + 0; if input > 0 { return helper(produce(divisor)); }",
        "if (divisor > 0) == true { return helper(produce(divisor)); }",
        "let decision = divisor > 0; if decision == true { return helper(produce(divisor)); }",
    ] {
        let admitted = base.replace(&arm("input > zero", "&&", false), candidate);
        let compiled = crate::pipeline::compile_source(&admitted).unwrap();
        yir_lower_llvm::emit_module(&compiled.yir).unwrap();
    }
    let admitted = base.replace(
        &arm("input > zero", "&&", false),
        "if divisor > 0 { return helper(produce(divisor)); } let ignored = divisor;",
    );
    execute(&admitted, Some(11), &[99, 11], 1);
    let admitted = base.replace(
        &arm("input > zero", "&&", false),
        "if gate && helper(produce(divisor)) { return helper(produce(divisor)); }",
    );
    execute(&admitted, Some(11), &[99, 11], 2);
    for candidate in [
        "if divisor > 0 { print(88); return helper(produce(divisor)); }",
        "let divisor = 2; if divisor > 0 { return helper(produce(divisor)); }",
        "if divisor > 0 { let ignored = 10 / divisor; } else { let ignored = 10 / divisor; }",
    ] {
        let mut module = crate::frontend::parse_nuis_module(
            &base.replace(&arm("input > zero", "&&", false), candidate),
        )
        .unwrap();
        if candidate.contains("print(88)") {
            assert_selected_effect_not_pure(
                &base.replace(&arm("input > zero", "&&", false), candidate),
            );
            continue;
        }
        let before = module.clone();
        assert!(outline_test(&mut module).is_empty(), "{candidate}");
        assert_eq!(module, before, "{candidate}");
    }
    let outer = base.replace("if outer {", "if divisor > 0 {");
    let mut module = crate::frontend::parse_nuis_module(&outer).unwrap();
    assert_eq!(outline_test(&mut module).len(), 1);
    execute(&outer, Some(11), &[99, 11], 1);
    for mutation in ["borrow", "optional", "kind", "declared", "effect"] {
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
            if mutation == "declared" {
                let NirStmt::If { then_body, .. } = &mut event.body[2] else {
                    panic!()
                };
                let NirStmt::Let { ty, .. } = &mut then_body[0] else {
                    panic!()
                };
                *ty = Some(scalar_type("bool"));
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
}
