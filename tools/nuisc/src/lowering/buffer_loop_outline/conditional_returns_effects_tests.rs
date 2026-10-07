use super::super::tests::{execute, outline_test};
use super::*;

#[path = "conditional_returns_effects_fixtures.rs"]
mod fixtures;
use fixtures::{event, expected, source, INPUTS};
#[path = "conditional_returns_print_aliases_tests.rs"]
mod alias_tests;
#[path = "conditional_returns_effects_budget_tests.rs"]
mod budget;
#[path = "conditional_returns_print_values_tests.rs"]
mod computed;
#[path = "conditional_returns_effects_native_tests.rs"]
mod native;
#[path = "conditional_returns_effect_regions_tests.rs"]
mod regions;

#[test]
fn conditional_return_effects_preserve_leading_prints_entry_once_and_complete_exits() {
    let mut cases = 0;
    for mode in ["return", "inferred", "const", "complete"] {
        for entry in ["atom", "computed", "nested"] {
            for prefix in ["literal", "current", "multiple"] {
                for shape in ["then", "else", "both"] {
                    for input in INPUTS {
                        let text = source(mode, entry, prefix, shape, input);
                        let (result, prints, calls) = expected(mode, entry, prefix, shape, input);
                        execute(&text, result, &prints, calls);
                        cases += 1;
                    }
                }
            }
        }
    }
    assert_eq!(cases, 864);
}

#[test]
fn conditional_return_effects_preserve_partial_suffix_and_continuation_readiness() {
    let mut cases = 0;
    for mode in ["partial", "suffix", "continuation"] {
        for entry in ["atom", "computed", "nested"] {
            for prefix in ["literal", "multiple"] {
                for shape in ["then", "else", "both"] {
                    for input in INPUTS {
                        let text = source(mode, entry, prefix, shape, input);
                        let (result, prints, calls) = expected(mode, entry, prefix, shape, input);
                        execute(&text, result, &prints, calls);
                        cases += 1;
                    }
                }
            }
        }
    }
    assert_eq!(cases, 432);
}

#[test]
fn conditional_return_effects_preserve_zero_exit_and_current_parent_print_binding() {
    let mut cases = 0;
    for entry in ["atom", "nested"] {
        for shape in ["then", "else", "both"] {
            for input in INPUTS {
                let text = source("zero", entry, "multiple", shape, input);
                let (result, prints, calls) = expected("zero", entry, "multiple", shape, input);
                execute(&text, result, &prints, calls);
                cases += 1;
            }
        }
    }
    let text = source("return", "atom", "current", "then", INPUTS[2])
        .replace("print(99);", "print(99); let stamp: i64 = stamp + 1;");
    execute(&text, Some(11), &[99, 56, 11], 1);
    assert_eq!(cases + 1, 49);
}

#[test]
fn conditional_return_effects_keep_helpers_pure_captures_minimal_and_install_idempotent() {
    for shape in ["then", "else", "both"] {
        let text = source("return", "computed", "multiple", shape, INPUTS[2]);
        let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
        let before = module.clone();
        let generated = outline_test(&mut module);
        assert_eq!(generated.len(), 1);
        let original = before.functions.iter().find(|f| f.name == "event").unwrap();
        let parent = event(&mut module);
        assert_eq!(&parent.body[..2], &original.body[..2]);
        assert_eq!(&parent.body[parent.body.len() - 2..], &original.body[3..]);
        let NirStmt::If { condition, .. } = &original.body[2] else {
            panic!()
        };
        assert_eq!(
            parent.body[2],
            NirStmt::Let {
                name: "__nuis_effect_return_gate_0".into(),
                ty: Some(scalar_type("bool")),
                value: condition.clone()
            }
        );
        let guards = parent.body.iter().filter(|stmt| matches!(stmt, NirStmt::If { then_body, else_body, .. } if then_body.len() == 1 && matches!(then_body[0], NirStmt::Print(_)) && else_body.is_empty())).count();
        assert_eq!(guards, if shape == "both" { 6 } else { 3 });
        let helper = module
            .functions
            .iter()
            .find(|f| generated.contains(&f.name))
            .unwrap();
        assert!(!format!("{:?}", helper.body).contains("Print("));
        assert_eq!(
            helper
                .params
                .iter()
                .skip(1)
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>(),
            ["gate", "left", "right"]
        );
        for name in ["produce", "helper"] {
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
}

#[test]
fn conditional_return_effects_reject_interleaving_calls_resources_and_new_capture_authority() {
    let base = source("return", "atom", "literal", "then", INPUTS[2]);
    // The former checked-print veto now has its own selected-value proof.
    let admitted = base.replace("print(88);", "print(10 / left);");
    execute(&admitted, Some(11), &[99, 5, 11], 1);
    let admitted = base.replace("print(88);", "let local = left; print(local);");
    execute(&admitted, Some(11), &[99, 2, 11], 1);
    let admitted = base.replace("print(88);", "let local = 10 / left; print(local);");
    execute(&admitted, Some(11), &[99, 5, 11], 1);
    let admitted = base.replace("print(88);", "if gate { print(88); }");
    execute(&admitted, Some(11), &[99, 88, 11], 1);
    let admitted = base.replace("print(88);", "if gate { print(88); return false; }");
    execute(&admitted, Some(19), &[99, 88, 19], 0);
    for replacement in [
        "print(helper(produce(left)));",
        "let local = produce(left); print(local.value);",
        "if gate { print(88); return 0; }",
        "while gate { print(88); }",
        "print(gate);",
    ] {
        let mut module =
            crate::frontend::parse_nuis_module(&base.replace("print(88);", replacement)).unwrap();
        let before = module.clone();
        assert!(outline_test(&mut module).is_empty(), "{replacement}");
        assert_eq!(module, before);
    }
    for mutation in [
        "borrow",
        "optional",
        "generic",
        "callee-effect",
        "unbound",
        "no-return",
    ] {
        let text = source("return", "atom", "current", "then", INPUTS[2]);
        let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
        match mutation {
            "callee-effect" => module
                .functions
                .iter_mut()
                .find(|f| f.name == "helper")
                .unwrap()
                .body
                .insert(0, NirStmt::Print(NirExpr::Int(1))),
            _ => {
                let parent = event(&mut module);
                match mutation {
                    "borrow" => parent.params[6].ty.is_ref = true,
                    "optional" => parent.params[6].ty.is_optional = true,
                    "generic" => parent.params[6].ty.generic_args.push(scalar_type("i64")),
                    "unbound" | "no-return" => {
                        let NirStmt::If { then_body, .. } = &mut parent.body[2] else {
                            panic!()
                        };
                        if mutation == "unbound" {
                            then_body[0] = NirStmt::Print(NirExpr::Var("missing".into()));
                        } else {
                            then_body.pop();
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }
        let before = module.clone();
        assert!(outline_test(&mut module).is_empty(), "{mutation}");
        assert_eq!(module, before);
    }
}
