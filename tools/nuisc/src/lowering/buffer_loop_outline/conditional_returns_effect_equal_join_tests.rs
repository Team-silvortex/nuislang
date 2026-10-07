use super::*;
#[path = "conditional_returns_effect_equal_join_fixtures.rs"]
mod fixtures;
use fixtures::{expected, source, INPUTS};
#[path = "conditional_returns_effect_equal_join_native_tests.rs"]
mod native;

#[test]
fn conditional_return_equal_effect_joins_preserve_selected_work_exits_and_presence() {
    let mut sources = BTreeSet::new();
    for kind in ["atom", "computed", "logical"] {
        for word in [false, true] {
            for shared in [false, true] {
                for constant in [false, true] {
                    for partial in [false, true] {
                        for input in INPUTS {
                            let text = source(kind, word, shared, constant, partial, input);
                            let (result, prints) = expected(word, shared, partial, input);
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
fn conditional_return_equal_effect_joins_keep_real_capture_sets_once_only_calls_and_distinct_snapshots(
) {
    let mut cases = 0;
    for word in [false, true] {
        for shared in [false, true] {
            for partial in [false, true] {
                let text = source("computed", word, shared, false, partial, INPUTS[1]);
                let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
                let before = module.clone();
                assert!(!outline_test(&mut module).is_empty());
                let joins = module
                    .functions
                    .iter()
                    .filter(|f| f.name.starts_with("__nuis_effect_join_value"))
                    .collect::<Vec<_>>();
                assert_eq!(joins.len(), 1);
                let joined = joins[0];
                assert_eq!(joined.params.len(), if shared { 2 } else { 1 });
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
                let atom = match (word, shared) {
                    (true, true) => NirExpr::Var("shared".into()),
                    (false, true) => NirExpr::Var("flag".into()),
                    (true, false) => NirExpr::Int(0),
                    (false, false) => NirExpr::Bool(false),
                };
                assert_eq!(then_body, &[NirStmt::Return(Some(atom))]);
                for name in ["choose", "observe"] {
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
                    assert_eq!(trace.iter().filter(|e| e.contains("] choose(")).count(), 1);
                    assert_eq!(trace.iter().filter(|e| e.contains("] observe(")).count(), 1);
                }
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 8);
    for word in [false, true] {
        let atom = if word { "7" } else { "true" };
        let seed = if word { "0" } else { "false" };
        let text = source("logical", word, false, true, false, INPUTS[1])
            .replace(&format!("; {seed} }}"), &format!("; {atom} }}"))
            .replace("selected", "__nuis_effect_join_0");
        let result = if word { 7 } else { 11 };
        execute(&text, Some(result), &[99, 50, 81, result, result], 0);
    }
    let text = source("computed", true, false, false, false, INPUTS[1])
        .replace("; 0 }", "; local }")
        .replace("observe(right)", "observe(left)")
        .replace("10 / right", "10 / left");
    execute(&text, Some(50), &[99, 50, 81, 50, 50], 0);
    let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
    assert!(!outline_test(&mut module).is_empty());
    let joined = module
        .functions
        .iter()
        .find(|f| f.name.starts_with("__nuis_effect_join_value"))
        .unwrap();
    assert_eq!(joined.params.len(), 4);
    let NirStmt::If { then_body, .. } = &joined.body[0] else {
        panic!()
    };
    assert!(matches!(&then_body[0], NirStmt::If { .. }));
    crate::nir_verify::verify_nir_module(&module).unwrap();
}

#[test]
fn conditional_return_equal_effect_joins_retain_unused_checks_and_atomic_source_budgets() {
    let base = source("computed", true, true, false, false, INPUTS[1]);
    let text = base.replace("10 / left", "10 / right");
    execute(&text, None, &[99, 50], 0);
    let text = source("computed", false, false, false, false, INPUTS[1]);
    let text = text.replace("choose(gate)", "checked(gate, right)").replace("@noinline fn choose", "@noinline fn checked(value: bool, divisor: i64) -> bool { let unused = 10 / divisor; return value; } @noinline fn choose");
    execute(&text, None, &[99], 0);
    for mutation in ["wrong-type", "borrow", "unreachable", "effectful"] {
        let mut module = crate::frontend::parse_nuis_module(&base).unwrap();
        if mutation == "effectful" {
            module
                .functions
                .iter_mut()
                .find(|f| f.name == "observe")
                .unwrap()
                .body
                .insert(0, NirStmt::Print(NirExpr::Int(9)));
        } else {
            let NirStmt::If { then_body, .. } = &mut event(&mut module).body[1] else {
                panic!()
            };
            let NirStmt::If { else_body, .. } = &mut then_body[0] else {
                panic!()
            };
            let NirStmt::Let { ty, .. } = else_body.last_mut().unwrap() else {
                panic!()
            };
            match mutation {
                "wrong-type" => *ty = Some(scalar_type("bool")),
                "borrow" => ty.as_mut().unwrap().is_ref = true,
                "unreachable" => else_body.insert(0, NirStmt::Return(Some(NirExpr::Int(0)))),
                _ => unreachable!(),
            }
        }
        let before = module.clone();
        assert!(outline_test(&mut module).is_empty(), "{mutation}");
        assert_eq!(module, before, "{mutation}");
    }
    let mut accepted = false;
    let mut rejected = false;
    for copies in 0..=32 {
        let aliases = (0..copies)
            .map(|i| format!("let unused_{i} = shared;"))
            .collect::<String>();
        let text = base.replace(
            "let local = observe(right);",
            &format!("{aliases} let local = observe(right);"),
        );
        let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
        let NirStmt::If { then_body, .. } = &event(&mut module).body[1] else {
            panic!()
        };
        assert_eq!(then_body.len(), 4);
        assert!(matches!(&then_body[2], NirStmt::Print(_)));
        let fits = effects::regions::bounds::preflight(then_body, 3);
        assert_eq!(fits, copies <= 20, "copies={copies}");
        let before = module.clone();
        assert_eq!(
            !outline_test(&mut module).is_empty(),
            fits,
            "copies={copies}"
        );
        if fits {
            accepted = true;
        } else {
            rejected = true;
            assert_eq!(module, before);
        }
    }
    assert!(accepted && rejected);
}
