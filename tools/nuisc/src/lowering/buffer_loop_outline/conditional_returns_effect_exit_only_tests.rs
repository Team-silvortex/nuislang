use super::*;
use fixtures::{exit_only_expected as expected, exit_only_source as source};

#[path = "conditional_returns_effect_exit_only_native_tests.rs"]
mod native;

#[test]
fn conditional_return_exit_only_smoke_empty_checked_and_branch_tails() {
    for mode in ["false", "zero"] {
        for tail in ["empty", "checked", "branch"] {
            for index in [1, 2] {
                let text = source("computed", mode, false, "then", INPUTS[index], tail);
                let (result, prints, calls) =
                    expected("computed", mode, false, "then", INPUTS[index]);
                execute(&text, result, &prints, calls);
            }
        }
    }
}

#[test]
fn conditional_return_exit_only_preserves_nested_selection_and_parent_continuation() {
    let mut cases = 0;
    for kind in ["atom", "computed", "logical"] {
        for mode in ["false", "computed", "logical", "zero", "checked"] {
            for deep in [false, true] {
                for shape in ["then", "else", "both"] {
                    for (index, input) in INPUTS.into_iter().enumerate() {
                        let tail = ["empty", "checked", "branch"][index % 3];
                        let text = source(kind, mode, deep, shape, input, tail);
                        let (result, prints, calls) = expected(kind, mode, deep, shape, input);
                        execute(&text, result, &prints, calls);
                        cases += 1;
                    }
                }
            }
        }
    }
    assert_eq!(cases, 900);
}

fn all_exiting(mode: &str) -> String {
    source("computed", mode, false, "both", INPUTS[1], "empty")
        .replace(
            "if nested { print(44); return false; }",
            "print(44); return false;",
        )
        .replace("if nested { print(44); return 0; }", "print(44); return 0;")
        .replace("print(55);", "")
        .replace("print(67);", "")
        .replace("let ignored = observe(tail); print(89);", "")
}

#[test]
fn conditional_return_exit_only_all_exiting_prefixes_and_once_only_hygiene() {
    for (mode, result) in [("false", 19), ("zero", 0)] {
        let text = all_exiting(mode);
        execute(&text, Some(result), &[99, 88, 50, 44, result], 2);
        let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
        let before = module.clone();
        let generated = outline_test(&mut module);
        assert!(!generated.is_empty());
        assert!(format!("{:?}", event(&mut module).body).contains("__nuis_effect_continuation"));
        for function in module
            .functions
            .iter()
            .filter(|f| generated.contains(&f.name))
        {
            assert!(!format!("{:?}", function.body).contains("Print("));
            assert!(function.params.iter().all(|p| scalar(&p.ty)));
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
        for reversed in [false, true] {
            let trace = events(&text, reversed);
            assert_eq!(trace.iter().filter(|e| e.contains("] observe(")).count(), 1);
        }
    }
    let text = source("computed", "false", false, "both", INPUTS[2], "branch")
        .replace("nested", "__nuis_effect_live_0")
        .replace("ready", "__nuis_effect_continuation_0")
        .replace("local", "__nuis_effect_exit_value_0");
    execute(&text, Some(19), &[99, 88, 50, 55, 89, 77, 19], 2);
}

#[test]
fn conditional_return_exit_only_checks_continuation_without_promoting_scalar_seeds() {
    let base = source("computed", "false", false, "then", INPUTS[2], "checked").replace(
        "let final_check = observe(tail);",
        "let final_check = observe(tail - 2);",
    );
    execute(&base, None, &[99, 88, 50, 55, 89], 2);
    let skipped = base.replace("event(true, true, false,", "event(true, true, true,");
    execute(&skipped, Some(19), &[99, 88, 50, 44, 19], 2);
    let total = "mod cpu Main { fn event(outer: bool, gate: bool) -> bool { if outer { if gate { print(1); return false; } } print(2); return false; } fn main() -> i64 { return 0; } }";
    let print_only = total.replace("print(1);", "print(10 / 2);");
    for text in [total.to_string(), print_only] {
        let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
        let before = module.clone();
        assert!(outline_test(&mut module).is_empty());
        assert_eq!(module, before);
    }
    let checked_tail = "mod cpu Main { @noinline fn event(outer: bool, gate: bool, divisor: i64) -> bool { if outer { if gate { print(1); return false; } let ignored = 20 / divisor; } print(2); return false; } fn main() -> i64 { let result = event(true, false, 2); if result { print(11); return 11; } print(19); return 19; } }";
    for (gate, divisor, result, prints) in [
        (false, 2, Some(19), vec![2, 19]),
        (false, 0, None, vec![]),
        (true, 0, Some(19), vec![1, 19]),
    ] {
        let text = checked_tail.replace(
            "event(true, false, 2)",
            &format!("event(true, {gate}, {divisor})"),
        );
        execute(&text, result, &prints, 0);
    }
}

#[test]
fn conditional_return_exit_only_vetoes_invalid_tail_scope_and_failed_second_arm_atomically() {
    for mutation in [
        "branch-local",
        "forward",
        "wrong-result",
        "no-source-exit",
        "tail-return",
    ] {
        let base = source("computed", "false", false, "both", INPUTS[2], "checked");
        let mut module = crate::frontend::parse_nuis_module(&base).unwrap();
        let NirStmt::If {
            then_body,
            else_body,
            ..
        } = &mut event(&mut module).body[1]
        else {
            panic!()
        };
        match mutation {
            "branch-local" | "forward" => {
                let name = if mutation == "branch-local" {
                    "local"
                } else {
                    "later"
                };
                let text = base.replace(
                    "let final_check = observe(tail);",
                    &format!("let final_check = observe({name});"),
                );
                assert!(crate::frontend::parse_nuis_module(&text).is_err());
                let NirStmt::Let {
                    value: NirExpr::Call { args, .. },
                    ..
                } = else_body.last_mut().unwrap()
                else {
                    panic!()
                };
                args[0] = NirExpr::Var(name.into());
                if mutation == "forward" {
                    else_body.push(NirStmt::Let {
                        name: name.into(),
                        ty: None,
                        value: NirExpr::Var("tail".into()),
                    });
                }
            }
            "wrong-result" => {
                let NirStmt::If { then_body, .. } = &mut else_body[2] else {
                    panic!()
                };
                let NirStmt::If { then_body, .. } = &mut then_body[2] else {
                    panic!()
                };
                then_body[1] = NirStmt::Return(Some(NirExpr::Int(0)));
            }
            "no-source-exit" => {
                remove_exits(then_body);
                remove_exits(else_body);
            }
            "tail-return" => {
                *else_body.last_mut().unwrap() = NirStmt::Return(Some(NirExpr::Int(0)))
            }
            _ => unreachable!(),
        }
        let before = module.clone();
        assert!(outline_test(&mut module).is_empty(), "{mutation}");
        assert_eq!(module, before, "{mutation}");
    }
}

fn remove_exits(body: &mut Vec<NirStmt>) {
    body.retain(|stmt| !matches!(stmt, NirStmt::Return(_)));
    for stmt in body {
        if let NirStmt::If {
            then_body,
            else_body,
            ..
        } = stmt
        {
            remove_exits(then_body);
            remove_exits(else_body);
        }
    }
}

#[test]
fn conditional_return_exit_only_shares_original_and_expanded_empty_tail_budgets() {
    for args in [4093, 4094] {
        let prefix = [NirStmt::If {
            condition: NirExpr::Bool(true),
            then_body: vec![
                NirStmt::Print(NirExpr::Int(1)),
                NirStmt::Return(Some(NirExpr::Call {
                    callee: "helper".into(),
                    args: vec![NirExpr::Int(1); args],
                })),
            ],
            else_body: vec![],
        }];
        assert_eq!(
            effects::regions::bounds::preflight(&prefix, 1),
            args == 4093
        );
        assert_eq!(suffix::reserve_staged_prefix(&[], &prefix), args == 4093);
    }
    for copies in [29, 30] {
        let text = source("atom", "false", false, "then", INPUTS[1], "empty");
        let start = text.find("print(88);").unwrap();
        let end = text[start..].find("print(89);").unwrap() + start + "print(89);".len();
        let bindings = (0..copies)
            .map(|i| format!("let unused_{i} = gate;"))
            .collect::<String>();
        let text = format!(
            "{}if helper(produce(value)) {{ {bindings} print(1); return false; }}{}",
            &text[..start],
            &text[end..]
        );
        let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
        let before = module.clone();
        let generated = outline_test(&mut module);
        assert_eq!(!generated.is_empty(), copies == 29);
        if copies == 30 {
            assert_eq!(module, before);
        }
    }
}
