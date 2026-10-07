use super::*;
#[path = "conditional_returns_effect_join_fixtures.rs"]
mod fixtures;
use fixtures::{expected, source, Input, INPUTS};
#[path = "conditional_returns_effect_join_native_tests.rs"]
mod native;

#[test]
fn conditional_return_effect_joins_smoke_paired_scalar_results() {
    let text = "mod cpu Main { @noinline fn event(outer: bool, gate: bool, left: i64, right: i64) -> i64 { print(99); if outer { let selected: i64 = if gate { let local = 100 / left; print(local); local } else { let local = 100 / right; print(local); local }; print(selected); return selected; } print(77); return 19; } fn main() -> i64 { let result = event(true, true, 2, 0); print(result); return result; } }";
    execute(text, Some(50), &[99, 50, 50, 50], 0);
    let mut module = crate::frontend::parse_nuis_module(text).unwrap();
    assert!(!outline_test(&mut module).is_empty());
    assert!(format!("{:?}", event(&mut module).body).contains("__nuis_effect_join"));
}

#[test]
fn conditional_return_effect_joins_keep_selected_nested_results_effects_and_exit_identity() {
    let mut sources = BTreeSet::new();
    for kind in ["atom", "computed", "logical"] {
        for word in [false, true] {
            for deep in [false, true] {
                for early in [false, true] {
                    for input in INPUTS {
                        let text = source(kind, word, deep, early, input);
                        let (result, prints, calls) = expected(kind, word, early, input);
                        execute(&text, result, &prints, calls);
                        sources.insert(text);
                    }
                }
            }
        }
    }
    assert_eq!(sources.len(), 192);
}

#[test]
fn conditional_return_effect_joins_keep_const_zero_chains_unused_checks_and_once_only_work() {
    for word in [false, true] {
        for index in [1, 2] {
            let text = source("computed", word, true, true, INPUTS[index])
                .replace("let selected:", "const selected:");
            let (result, prints, calls) = expected("computed", word, true, INPUTS[index]);
            execute(&text, result, &prints, calls);
        }
    }
    let input = Input {
        right: 1000,
        ..INPUTS[2]
    };
    let text = source("computed", true, true, true, input);
    execute(&text, Some(0), &[99, 0, 0, 44, 0], 0);
    let text = source("computed", true, true, false, INPUTS[1])
        .replace("let unused = 10 / left;", "let unused = 10 / tail;");
    execute(&text, None, &[99, 50], 0);
    let text = source("computed", true, true, true, INPUTS[1]);
    for reversed in [false, true] {
        let trace = events(&text, reversed);
        assert_eq!(trace.iter().filter(|e| e.contains("] choose(")).count(), 1);
        assert_eq!(trace.iter().filter(|e| e.contains("] observe(")).count(), 1);
    }
    let chained = source("computed", true, true, true, INPUTS[1])
        .replace("print(selected);", "let after: i64 = if selected > 0 { let local = selected + 1; print(local); local } else { let local = 20 / tail; print(local); local }; print(after);");
    execute(&chained, Some(50), &[99, 50, 51, 51, 44, 50], 0);
    for word in [false, true] {
        let exit = if word { "0" } else { "false" };
        let text = source("computed", word, false, true, INPUTS[5]).replace(
            "if outer {",
            &format!("if outer {{ if nested {{ print(33); return {exit}; }}"),
        );
        let result = if word { 0 } else { 19 };
        execute(&text, Some(result), &[99, 33, result], 0);
        for reversed in [false, true] {
            assert!(!events(&text, reversed).iter().any(|e| {
                ["] choose(", "] observe(", "] produce(", "] helper("]
                    .iter()
                    .any(|callee| e.contains(callee))
            }));
        }
    }
}

#[test]
fn conditional_return_effect_joins_keep_pure_hygienic_helpers_and_idempotence() {
    let text = source("computed", true, true, true, INPUTS[1]);
    let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
    let before = module.clone();
    let generated = outline_test(&mut module);
    assert!(!generated.is_empty());
    for function in module
        .functions
        .iter()
        .filter(|f| generated.contains(&f.name))
    {
        assert!(!format!("{:?}", function.body).contains("Print("));
        assert!(function.params.iter().all(|p| scalar(&p.ty)));
    }
    for name in ["produce", "helper", "choose", "observe"] {
        assert_eq!(
            module.functions.iter().find(|f| f.name == name),
            before.functions.iter().find(|f| f.name == name)
        );
    }
    crate::nir_verify::verify_nir_module(&module).unwrap();
    let once = module.clone();
    assert!(outline_test(&mut module).is_empty());
    assert_eq!(module, once);
    let hygienic = text
        .replace("selected", "__nuis_effect_join_0")
        .replace("local", "__nuis_join_live_0");
    execute(&hygienic, Some(50), &[99, 50, 50, 44, 50], 0);
}

#[test]
fn conditional_return_effect_joins_veto_unpaired_kind_scope_resource_and_exit_results_atomically() {
    for mutation in [
        "name",
        "kind",
        "constant",
        "borrow",
        "local",
        "resource",
        "early-kind",
        "parent",
        "effectful",
    ] {
        let text = source("computed", true, false, false, INPUTS[1]);
        let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
        if mutation == "effectful" {
            module
                .functions
                .iter_mut()
                .find(|f| f.name == "observe")
                .unwrap()
                .body
                .insert(0, NirStmt::Print(NirExpr::Int(1)));
        } else {
            let body = &mut event(&mut module).body;
            if mutation == "parent" {
                body.insert(
                    1,
                    NirStmt::Let {
                        name: "selected".into(),
                        ty: Some(scalar_type("i64")),
                        value: NirExpr::Int(0),
                    },
                );
            }
            let index = if mutation == "parent" { 2 } else { 1 };
            let NirStmt::If { then_body, .. } = &mut body[index] else {
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
            let NirStmt::Let { name, ty, value } = no.last_mut().unwrap() else {
                panic!()
            };
            match mutation {
                "name" => *name = "other".into(),
                "kind" => {
                    *ty = Some(scalar_type("bool"));
                    *value = NirExpr::Bool(false);
                }
                "constant" => {
                    let last = no.last_mut().unwrap();
                    *last = NirStmt::Const {
                        name: "selected".into(),
                        ty: scalar_type("i64"),
                        value: NirExpr::Int(0),
                    };
                }
                "borrow" => ty.as_mut().unwrap().is_ref = true,
                "local" => *value = NirExpr::Var("yes_only".into()),
                "resource" => {
                    *ty = None;
                    *value = NirExpr::Call {
                        callee: "produce".into(),
                        args: vec![NirExpr::Var("right".into())],
                    };
                }
                "early-kind" => yes.insert(
                    0,
                    NirStmt::If {
                        condition: NirExpr::Var("nested".into()),
                        then_body: vec![NirStmt::Return(Some(NirExpr::Bool(false)))],
                        else_body: vec![],
                    },
                ),
                "parent" => {}
                _ => unreachable!(),
            }
        }
        let before = module.clone();
        assert!(outline_test(&mut module).is_empty(), "{mutation}");
        assert_eq!(module, before, "{mutation}");
    }
}

#[test]
fn conditional_return_effect_joins_charge_shared_source_and_expanded_budgets() {
    for copies in [21, 22] {
        let text = source("computed", true, false, false, INPUTS[1]);
        let extra = (0..copies)
            .map(|i| format!("let unused_{i} = left;"))
            .collect::<String>();
        let text = text.replace(
            "let local = observe(left);",
            &format!("{extra} let local = observe(left);"),
        );
        let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
        let NirStmt::If { then_body, .. } = &event(&mut module).body[1] else {
            panic!()
        };
        assert_eq!(
            effects::regions::bounds::preflight(then_body, 2),
            copies == 21
        );
        let before = module.clone();
        assert_eq!(!outline_test(&mut module).is_empty(), copies == 21);
        if copies == 22 {
            assert_eq!(module, before);
        }
    }
}
