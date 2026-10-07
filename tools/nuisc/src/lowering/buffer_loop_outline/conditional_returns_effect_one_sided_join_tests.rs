use super::*;
#[path = "conditional_returns_effect_one_sided_join_fixtures.rs"]
mod fixtures;
use fixtures::{expected, source, Input, INPUTS};
#[path = "conditional_returns_effect_one_sided_join_native_tests.rs"]
mod native;

#[test]
fn conditional_return_join_captures_match_actual_single_value_helper_reads() {
    let mut cases = 0;
    for kind in ["atom", "computed", "logical"] {
        for word in [false, true] {
            for swapped in [false, true] {
                let text = source(kind, word, swapped, true, true, INPUTS[1]);
                let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
                assert!(!outline_test(&mut module).is_empty());
                let joins = module
                    .functions
                    .iter()
                    .filter(|f| f.name.starts_with("__nuis_effect_join_value"))
                    .collect::<Vec<_>>();
                assert_eq!(joins.len(), 1);
                let joined = joins[0];
                assert_eq!(joined.params.len(), 2);
                assert!(!joined
                    .params
                    .iter()
                    .any(|p| p.name.starts_with("__nuis_effect_condition")));
                let NirStmt::If {
                    condition,
                    then_body,
                    ..
                } = &joined.body[0]
                else {
                    panic!()
                };
                assert_eq!(condition, &NirExpr::Var(joined.params[0].name.clone()));
                assert_eq!(
                    then_body,
                    &[NirStmt::Return(Some(NirExpr::Var(
                        joined.params[1].name.clone()
                    )))]
                );
                crate::nir_verify::verify_nir_module(&module).unwrap();
                let once = module.clone();
                assert!(outline_test(&mut module).is_empty());
                assert_eq!(module, once);
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 12);
}

#[test]
fn conditional_return_one_sided_effect_joins_preserve_real_exits_presence_and_selected_work() {
    let mut sources = BTreeSet::new();
    for kind in ["atom", "computed", "logical"] {
        for word in [false, true] {
            for swapped in [false, true] {
                for deep in [false, true] {
                    for partial in [false, true] {
                        for input in INPUTS {
                            let text = source(kind, word, swapped, deep, partial, input);
                            let (result, prints) = expected(word, swapped, deep, partial, input);
                            execute(&text, result, &prints, 0);
                            sources.insert(text);
                        }
                    }
                }
            }
        }
    }
    assert_eq!(sources.len(), 384);
}

#[test]
fn conditional_return_one_sided_effect_joins_keep_const_zero_hygiene_and_once_only_work() {
    for word in [false, true] {
        for swapped in [false, true] {
            let input = Input {
                gate: !swapped,
                value: 1000,
                tail: 0,
                ..INPUTS[5]
            };
            let text = source("computed", word, swapped, true, false, input)
                .replace("let selected:", "const selected:");
            let (result, prints) = expected(word, swapped, true, false, input);
            execute(&text, result, &prints, 0);
        }
    }
    let text = source("computed", true, true, true, false, INPUTS[5]);
    for reversed in [false, true] {
        let trace = events(&text, reversed);
        assert_eq!(trace.iter().filter(|e| e.contains("] choose(")).count(), 2);
        assert_eq!(trace.iter().filter(|e| e.contains("] leave(")).count(), 1);
        assert_eq!(trace.iter().filter(|e| e.contains("] observe(")).count(), 0);
        assert_eq!(
            trace.iter().filter(|e| e.contains("] positive(")).count(),
            0
        );
    }
    let text = source("logical", true, false, true, false, INPUTS[5])
        .replace("selected", "__nuis_effect_join_0")
        .replace("local", "__nuis_effect_live_0");
    execute(&text, Some(0), &[99, 0, 0, 0], 0);
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
    for name in ["choose", "observe", "positive", "leave"] {
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
fn conditional_return_one_sided_effect_joins_reject_unproved_presence_exits_and_scope_atomically() {
    for mutation in [
        "missing",
        "continuing-other",
        "unreachable-exit",
        "unreachable-result",
        "wrong-exit",
        "borrow",
        "forward",
        "exit-arity",
        "both-exit",
        "effectful",
    ] {
        let text = source("computed", true, false, false, false, INPUTS[1]);
        let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
        if mutation == "effectful" {
            module
                .functions
                .iter_mut()
                .find(|f| f.name == "leave")
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
                    yes.pop();
                }
                "continuing-other" => {
                    *no.last_mut().unwrap() = NirStmt::Let {
                        name: "different".into(),
                        ty: Some(scalar_type("i64")),
                        value: NirExpr::Int(0),
                    };
                }
                "unreachable-exit" => no.push(NirStmt::Print(NirExpr::Int(1))),
                "unreachable-result" => yes.insert(0, NirStmt::Return(Some(NirExpr::Int(0)))),
                "wrong-exit" => {
                    *no.last_mut().unwrap() = NirStmt::Return(Some(NirExpr::Bool(false)))
                }
                "borrow" => {
                    let NirStmt::Let { ty, .. } = yes.last_mut().unwrap() else {
                        panic!()
                    };
                    ty.as_mut().unwrap().is_ref = true;
                }
                "forward" => {
                    *no.last_mut().unwrap() = NirStmt::Return(Some(NirExpr::Var("selected".into())))
                }
                "exit-arity" => {
                    *no.last_mut().unwrap() = NirStmt::Return(Some(NirExpr::Call {
                        callee: "leave".into(),
                        args: vec![],
                    }))
                }
                "both-exit" => *yes = vec![NirStmt::Return(Some(NirExpr::Int(0)))],
                _ => unreachable!(),
            }
        }
        let before = module.clone();
        assert!(outline_test(&mut module).is_empty(), "{mutation}");
        assert_eq!(module, before, "{mutation}");
    }
    let text = "mod cpu Main { fn main() -> i64 { if true { let child = 7; } else { return 0; } return child; } }";
    assert!(crate::frontend::parse_nuis_module(text).is_err());
}

#[test]
fn conditional_return_one_sided_effect_joins_charge_unused_and_exiting_work_before_installation() {
    let base = source("computed", true, false, false, false, INPUTS[1]);
    for copies in [23, 24] {
        let aliases = (0..copies)
            .map(|i| format!("let unused_{i} = tail;"))
            .collect::<String>();
        let text = base.replace("print(31);", &format!("{aliases} print(31);"));
        let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
        let NirStmt::If { then_body, .. } = &event(&mut module).body[1] else {
            panic!()
        };
        assert_eq!(
            effects::regions::bounds::preflight(then_body, 2),
            copies == 23
        );
        let before = module.clone();
        assert_eq!(!outline_test(&mut module).is_empty(), copies == 23);
        if copies == 24 {
            assert_eq!(module, before);
        }
    }
    let text = base.replace("let unused = 10 / value;", "let unused = 10 / tail;");
    execute(&text, None, &[99, 50], 0);
    let text = source(
        "computed",
        true,
        true,
        false,
        false,
        Input {
            gate: true,
            ..INPUTS[2]
        },
    )
    .replace(
        "print(31); return leave(tail);",
        "let checked = 10 / value; print(31); return leave(tail);",
    );
    execute(&text, None, &[99], 0);
    let text = base.replace("print(selected); return selected;", "let chained: i64 = if selected > 0 { print(45); return 0; } else { let local = observe(value); print(local); local }; print(chained); return chained;");
    execute(&text, Some(0), &[99, 50, 45, 0], 0);
}
