use super::*;

#[path = "conditional_returns_print_aliases_fixtures.rs"]
mod fixtures;
use fixtures::{alias_expected, alias_source};
#[path = "conditional_returns_print_aliases_budget_tests.rs"]
mod budget;
#[path = "conditional_returns_print_aliases_native_tests.rs"]
mod native;

#[test]
fn conditional_return_print_aliases_preserve_interleaved_work_order_and_exits() {
    let mut cases = 0;
    for kind in ["atom", "division", "call", "field"] {
        for mode in ["return", "suffix", "zero"] {
            for entry in ["atom", "nested"] {
                for shape in ["then", "else", "both"] {
                    for (index, input) in INPUTS.into_iter().enumerate() {
                        let stamp = [0, 0, 2, -2, 2, -2, 0, 0][index];
                        let text = alias_source(kind, mode, entry, shape, input, stamp);
                        let (result, prints, calls) =
                            alias_expected(kind, mode, entry, shape, input, stamp);
                        execute(&text, result, &prints, calls);
                        cases += 1;
                    }
                }
            }
        }
    }
    for mode in ["partial", "continuation", "inferred", "const", "complete"] {
        for input in INPUTS {
            let text = alias_source("remainder", mode, "computed", "both", input, 2);
            let (result, prints, calls) =
                alias_expected("remainder", mode, "computed", "both", input, 2);
            execute(&text, result, &prints, calls);
            cases += 1;
        }
    }
    assert_eq!(cases, 616);
}

#[test]
fn conditional_return_print_aliases_keep_captures_original_and_install_idempotent() {
    for shape in ["then", "else", "both"] {
        let text = alias_source("call", "return", "computed", shape, INPUTS[2], 2);
        let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
        let before = module.clone();
        let generated = outline_test(&mut module);
        assert_eq!(generated.len(), if shape == "both" { 3 } else { 2 });
        for function in module
            .functions
            .iter()
            .filter(|f| generated.contains(&f.name))
        {
            let body = format!("{:?}", function.body);
            assert!(
                !body.contains("Print(")
                    && !body.contains("_alias")
                    && !body.contains("prefix_stamp")
            );
            let captures = function
                .params
                .iter()
                .skip(1)
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>();
            assert_eq!(
                captures,
                if function.name.starts_with("__nuis_conditional_print_value") {
                    vec!["stamp"]
                } else {
                    vec!["gate", "left", "right"]
                }
            );
        }
        assert!(!format!("{:?}", event(&mut module).body).contains("_alias"));
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
    }
}

#[test]
fn conditional_return_print_aliases_preserve_literals_sibling_scopes_and_current_binding() {
    for outer in [false, true] {
        let mut input = INPUTS[2];
        input.0 = outer;
        let text = alias_source("atom", "return", "atom", "both", input, 2)
            .replacen(
                "let prefix_stamp: i64 = stamp;",
                "let prefix_stamp: i64 = 0;",
                1,
            )
            .replace(
                "let gate_alias: bool = gate;",
                "let gate_alias: bool = false;",
            );
        execute(
            &text,
            None,
            if outer {
                &[99, 88, 0, 89]
            } else {
                &[99, 66, 2, 67]
            },
            2,
        );
    }
    let text = alias_source("division", "return", "atom", "then", INPUTS[2], 2)
        .replace("print(99);", "print(99); let stamp: i64 = stamp + 2;");
    execute(&text, Some(11), &[99, 88, 25, 89, 11], 1);
}

#[test]
fn conditional_return_print_aliases_validate_original_scopes_and_veto_atomic() {
    let text = alias_source("division", "return", "atom", "both", INPUTS[2], 2);
    for mutation in [
        "duplicate",
        "parent-shadow",
        "type",
        "missing",
        "self",
        "forward",
        "record",
        "internal-print",
        "early-return",
        "print-before-alias",
    ] {
        // Invalid second arm must not install helpers for a valid first arm.
        let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
        let NirStmt::If { else_body, .. } = &mut event(&mut module).body[2] else {
            panic!()
        };
        let NirStmt::Let { name, ty, value } = &mut else_body[0] else {
            panic!()
        };
        match mutation {
            "parent-shadow" => *name = "stamp".into(),
            "type" => *ty = Some(scalar_type("bool")),
            "missing" | "self" | "forward" => {
                *value = NirExpr::Var(
                    match mutation {
                        "missing" => "missing",
                        "self" => "prefix_stamp",
                        _ => "stamp_alias",
                    }
                    .into(),
                )
            }
            "record" => {
                *value = NirExpr::Call {
                    callee: "produce".into(),
                    args: vec![NirExpr::Var("stamp".into())],
                }
            }
            "duplicate" => else_body.insert(1, else_body[0].clone()),
            "internal-print" => else_body.insert(
                0,
                NirStmt::If {
                    condition: NirExpr::Var("gate".into()),
                    then_body: vec![NirStmt::Print(NirExpr::Int(1))],
                    else_body: vec![],
                },
            ),
            "early-return" => else_body.insert(0, NirStmt::Return(Some(NirExpr::Bool(true)))),
            "print-before-alias" => {
                else_body.insert(0, NirStmt::Print(NirExpr::Var("prefix_stamp".into())))
            }
            _ => unreachable!(),
        }
        let before = module.clone();
        assert!(outline_test(&mut module).is_empty(), "{mutation}");
        assert_eq!(module, before, "{mutation}");
    }
    for mutation in [
        "tail-shadow",
        "branch-shadow",
        "borrow",
        "optional",
        "generic",
        "bool-print",
        "no-return",
        "print-only-work",
    ] {
        let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
        let parent = event(&mut module);
        if matches!(mutation, "borrow" | "optional" | "generic") {
            match mutation {
                "borrow" => parent.params[6].ty.is_ref = true,
                "optional" => parent.params[6].ty.is_optional = true,
                _ => parent.params[6].ty.generic_args.push(scalar_type("i64")),
            }
        } else {
            let NirStmt::If {
                then_body,
                else_body,
                ..
            } = &mut parent.body[2]
            else {
                panic!()
            };
            let alias = NirStmt::Let {
                name: "left_alias".into(),
                ty: None,
                value: NirExpr::Int(1),
            };
            match mutation {
                "tail-shadow" => then_body.insert(then_body.len() - 1, alias),
                "branch-shadow" => then_body.insert(
                    then_body.len() - 1,
                    NirStmt::If {
                        condition: NirExpr::Var("gate".into()),
                        then_body: vec![alias],
                        else_body: vec![],
                    },
                ),
                "bool-print" => then_body[3] = NirStmt::Print(NirExpr::Var("gate".into())),
                "no-return" => {
                    then_body.pop();
                    else_body.pop();
                }
                "print-only-work" => {
                    *then_body.last_mut().unwrap() = NirStmt::Return(Some(NirExpr::Bool(true)));
                    *else_body.last_mut().unwrap() = NirStmt::Return(Some(NirExpr::Bool(false)));
                }
                _ => unreachable!(),
            }
        }
        let before = module.clone();
        assert!(outline_test(&mut module).is_empty(), "{mutation}");
        assert_eq!(module, before, "{mutation}");
    }
}
