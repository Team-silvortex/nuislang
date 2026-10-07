use super::*;
#[path = "conditional_returns_effect_partial_join_fixtures.rs"]
mod fixtures;
use fixtures::{expected, source, Input, INPUTS};
#[path = "conditional_returns_effect_partial_join_native_tests.rs"]
mod native;

#[test]
fn conditional_return_partial_effect_joins_preserve_result_presence_real_exits_and_order() {
    let mut sources = BTreeSet::new();
    for kind in ["atom", "computed", "logical"] {
        for word in [false, true] {
            for deep in [false, true] {
                for late in [false, true] {
                    for continuing in [false, true] {
                        for (index, input) in INPUTS.into_iter().enumerate() {
                            let computed = index % 2 == 1;
                            let text = source(kind, word, deep, late, computed, continuing, input);
                            let (result, prints) =
                                expected(word, deep, late, computed, continuing, input);
                            execute(&text, result, &prints, 0);
                            sources.insert(text);
                        }
                    }
                }
            }
        }
    }
    assert_eq!(sources.len(), 480);
}

#[test]
fn conditional_return_partial_effect_joins_keep_const_zero_chains_checks_and_once_only_work() {
    for word in [false, true] {
        for index in [3, 4] {
            let computed = index % 2 == 1;
            let text = source(
                "computed",
                word,
                true,
                false,
                computed,
                false,
                INPUTS[index],
            )
            .replace("let selected:", "const selected:");
            let (result, prints) = expected(word, true, false, computed, false, INPUTS[index]);
            execute(&text, result, &prints, 0);
        }
        let input = Input {
            left: 1000,
            tail: 2,
            ..INPUTS[3]
        };
        let text = source("computed", word, true, false, true, false, input);
        let (result, prints) = expected(word, true, false, true, false, input);
        execute(&text, result, &prints, 0);
    }
    let input = Input {
        tail: 0,
        ..INPUTS[3]
    };
    let text = source("computed", true, false, false, true, false, input)
        .replace("let unused = 10 / left;", "let unused = 10 / tail;");
    execute(&text, None, &[99, 50], 0);
    let text = source("computed", true, false, false, true, false, INPUTS[3])
        .replace("print(selected);", "let chained: i64 = if selected > 0 { if stop_right { print(44); return selected - 50; } let value = observe(tail); print(value); value } else { let value = observe(tail); print(value); value }; print(chained);");
    execute(&text, Some(0), &[99, 50, 44, 0], 0);
    for word in [false, true] {
        for reversed in [false, true] {
            let text = source("computed", word, true, false, true, false, INPUTS[1]);
            let trace = events(&text, reversed);
            assert_eq!(trace.iter().filter(|e| e.contains("] choose(")).count(), 2);
            assert_eq!(trace.iter().filter(|e| e.contains("] observe(")).count(), 0);
            assert_eq!(
                trace.iter().filter(|e| e.contains("] positive(")).count(),
                0
            );
            let callee = if word { "] exit_word(" } else { "] exit_bool(" };
            assert_eq!(trace.iter().filter(|e| e.contains(callee)).count(), 1);
        }
    }
    let text = source("computed", true, true, false, false, false, INPUTS[6])
        .replace("selected", "__nuis_effect_join_0")
        .replace("local", "__nuis_effect_live_0");
    execute(&text, Some(0), &[99, 0], 0);
    let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
    let before = module.clone();
    let generated = outline_test(&mut module);
    assert!(!generated.is_empty());
    for f in module
        .functions
        .iter()
        .filter(|f| generated.contains(&f.name))
    {
        assert!(!format!("{:?}", f.body).contains("Print("));
        assert!(f.params.iter().all(|p| scalar(&p.ty)));
    }
    for name in ["choose", "observe", "positive", "exit_word", "exit_bool"] {
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

#[test]
fn conditional_return_partial_effect_joins_veto_missing_results_scope_wrong_exits_and_effects_atomically(
) {
    for mutation in [
        "missing",
        "unreachable",
        "unpaired",
        "wrong-exit",
        "forward",
        "borrow",
        "constant",
        "effectful",
    ] {
        let text = source("computed", true, false, false, true, false, INPUTS[3]);
        let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
        if mutation == "effectful" {
            module
                .functions
                .iter_mut()
                .find(|f| f.name == "exit_word")
                .unwrap()
                .body
                .insert(0, NirStmt::Print(NirExpr::Int(1)));
        } else {
            let NirStmt::If { then_body, .. } = &mut event(&mut module).body[1] else {
                panic!()
            };
            let NirStmt::If {
                then_body: yes,
                else_body: no,
                ..
            } = &mut then_body[0]
            else {
                panic!()
            };
            match mutation {
                "missing" => {
                    no.pop();
                }
                "unreachable" => no.insert(0, NirStmt::Return(Some(NirExpr::Int(0)))),
                "unpaired" => {
                    let NirStmt::Let { name, .. } = no.last_mut().unwrap() else {
                        panic!()
                    };
                    *name = "different".into();
                }
                "wrong-exit" | "forward" => {
                    let NirStmt::If { then_body, .. } = &mut yes[0] else {
                        panic!()
                    };
                    *then_body.last_mut().unwrap() =
                        NirStmt::Return(Some(if mutation == "wrong-exit" {
                            NirExpr::Bool(false)
                        } else {
                            NirExpr::Var("selected".into())
                        }));
                }
                "borrow" => {
                    let NirStmt::Let { ty, .. } = no.last_mut().unwrap() else {
                        panic!()
                    };
                    ty.as_mut().unwrap().is_ref = true;
                }
                "constant" => {
                    *no.last_mut().unwrap() = NirStmt::Const {
                        name: "selected".into(),
                        ty: scalar_type("i64"),
                        value: NirExpr::Int(0),
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
fn conditional_return_partial_effect_joins_charge_complete_exit_and_result_budgets() {
    for copies in [15, 16] {
        let aliases = (0..copies)
            .map(|i| format!("let unused_{i} = left;"))
            .collect::<String>();
        let text = source("computed", true, false, false, true, false, INPUTS[3]).replace(
            "let local = observe(left);",
            &format!("{aliases} let local = observe(left);"),
        );
        let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
        let NirStmt::If { then_body, .. } = &event(&mut module).body[1] else {
            panic!()
        };
        assert_eq!(
            effects::regions::bounds::preflight(then_body, 2),
            copies == 15
        );
        let before = module.clone();
        assert_eq!(!outline_test(&mut module).is_empty(), copies == 15);
        if copies == 16 {
            assert_eq!(module, before);
        }
    }
    let text = source("computed", true, false, false, true, false, INPUTS[3]);
    let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
    let NirStmt::If { then_body, .. } = &event(&mut module).body[1] else {
        panic!()
    };
    let limit = 4096 - roots(then_body) + 1;
    for args in [limit, limit + 1] {
        let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
        let NirStmt::If { then_body, .. } = &mut event(&mut module).body[1] else {
            panic!()
        };
        let NirStmt::If { then_body: yes, .. } = &mut then_body[0] else {
            panic!()
        };
        let NirStmt::If {
            then_body: exit, ..
        } = &mut yes[0]
        else {
            panic!()
        };
        let NirStmt::Return(Some(NirExpr::Call { args: values, .. })) = exit.last_mut().unwrap()
        else {
            panic!()
        };
        *values = vec![NirExpr::Int(2); args];
        assert_eq!(
            effects::regions::bounds::preflight(then_body, 2),
            args == limit
        );
        // Both widths have invalid source arity. Bounds grant no type authority.
        let before = module.clone();
        assert!(outline_test(&mut module).is_empty());
        assert_eq!(module, before);
    }
}

fn roots(body: &[NirStmt]) -> usize {
    body.iter()
        .map(|stmt| match stmt {
            NirStmt::Let { value, .. }
            | NirStmt::Const { value, .. }
            | NirStmt::Print(value)
            | NirStmt::Return(Some(value)) => nodes(value),
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } => nodes(condition) + roots(then_body) + roots(else_body),
            _ => panic!("unexpected fixture statement"),
        })
        .sum()
}

fn nodes(value: &NirExpr) -> usize {
    match value {
        NirExpr::Var(_) | NirExpr::Int(_) | NirExpr::Bool(_) => 1,
        NirExpr::Binary { lhs, rhs, .. } => 1 + nodes(lhs) + nodes(rhs),
        NirExpr::Call { args, .. } => 1 + args.iter().map(nodes).sum::<usize>(),
        _ => panic!("unexpected fixture expression"),
    }
}
