use super::*;

#[path = "conditional_returns_effect_exit_fixtures.rs"]
mod fixtures;
use fixtures::{expected, source, INPUTS};
#[path = "conditional_returns_effect_exit_only_tests.rs"]
mod exit_only;
#[path = "conditional_returns_effect_exit_native_tests.rs"]
mod native;

#[test]
fn conditional_return_effect_exits_smoke_false_and_zero_skip_later_checks() {
    for mode in ["false", "zero"] {
        let text = source("atom", mode, false, "then", INPUTS[1]);
        let (result, prints, calls) = expected("atom", mode, false, "then", INPUTS[1]);
        execute(&text, result, &prints, calls);
    }
}

#[test]
fn conditional_return_effect_exits_compose_with_real_partial_tail_exits_and_continuation_checks() {
    for mode in ["false", "zero"] {
        for index in [1, 2, 3, 4] {
            let text = fixtures::partial_source(mode, index);
            let (result, prints, calls) = fixtures::partial_expected(mode, index);
            execute(&text, result, &prints, calls);
        }
    }
}

#[test]
fn conditional_return_effect_exits_preserve_nested_selection_order_and_return_identity() {
    let mut cases = 0;
    for kind in ["atom", "computed", "logical"] {
        for mode in ["false", "computed", "logical", "zero", "checked"] {
            for deep in [false, true] {
                for shape in ["then", "else", "both"] {
                    for input in INPUTS {
                        let text = source(kind, mode, deep, shape, input);
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

#[test]
fn conditional_return_effect_exits_keep_first_exit_unused_checks_and_pure_hygienic_helpers() {
    let text = source("computed", "false", true, "both", INPUTS[1]);
    let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
    let before = module.clone();
    let generated = outline_test(&mut module);
    assert!(!generated.is_empty());
    let body = format!("{:?}", event(&mut module).body);
    for marker in [
        "__nuis_effect_exit_value",
        "__nuis_effect_live",
        "__nuis_effect_continuation",
    ] {
        assert!(body.contains(marker));
    }
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
    let hygienic = text
        .replace("nested", "__nuis_effect_live_0")
        .replace("local", "__nuis_effect_exit_value_0")
        .replace("ready", "__nuis_effect_continuation_0");
    execute(&hygienic, Some(19), &[99, 88, 50, 44, 19], 2);
    for reversed in [false, true] {
        let trace = events(&text, reversed);
        assert_eq!(
            trace.iter().filter(|e| e.contains("] observe(")).count(),
            1,
            "{trace:?}"
        );
    }
    let second = source(
        "atom",
        "false",
        false,
        "then",
        fixtures::Input {
            nested: false,
            ..INPUTS[1]
        },
    )
    .replace(
        "let ignored = observe(tail);",
        "if gate { print(43); return false; } let ignored = observe(tail);",
    );
    execute(&second, Some(19), &[99, 88, 50, 55, 43, 19], 1);
    let bypass = text.replace("print(99);", "if nested { return false; } print(99);");
    execute(&bypass, Some(19), &[19], 0);
    let unused = text.replace("print(44);", "let ignored_exit = observe(tail); print(44);");
    execute(&unused, None, &[99, 88, 50], 2);
}

#[test]
fn conditional_return_effect_exits_veto_invalid_source_exits_and_unreachable_code_atomically() {
    for mutation in [
        "kind",
        "void",
        "resource",
        "missing",
        "logical-leaf",
        "after-exit",
        "both-exit",
        "no-source-exit",
    ] {
        let mut module =
            crate::frontend::parse_nuis_module(&source("atom", "false", false, "both", INPUTS[1]))
                .unwrap();
        let NirStmt::If { else_body, .. } = &mut event(&mut module).body[1] else {
            panic!()
        };
        if mutation == "no-source-exit" {
            let text =
                fixtures::exit_only_source("atom", "false", false, "both", INPUTS[1], "empty")
                    .replace("return false;", "");
            module = crate::frontend::parse_nuis_module(&text).unwrap();
        } else {
            let NirStmt::If { then_body, .. } = &mut else_body[2] else {
                panic!()
            };
            let NirStmt::If {
                then_body: exit,
                else_body: continuation,
                ..
            } = &mut then_body[2]
            else {
                panic!()
            };
            match mutation {
                "kind" => exit[1] = NirStmt::Return(Some(NirExpr::Int(0))),
                "void" => exit[1] = NirStmt::Return(None),
                "resource" => {
                    exit[1] = NirStmt::Return(Some(NirExpr::Call {
                        callee: "produce".into(),
                        args: vec![NirExpr::Var("right".into())],
                    }))
                }
                "missing" => exit[1] = NirStmt::Return(Some(NirExpr::Var("absent".into()))),
                "logical-leaf" => {
                    exit[1] = NirStmt::Return(Some(NirExpr::Binary {
                        op: NirBinaryOp::Eq,
                        lhs: Box::new(NirExpr::Binary {
                            op: NirBinaryOp::And,
                            lhs: Box::new(NirExpr::Bool(true)),
                            rhs: Box::new(NirExpr::Bool(false)),
                        }),
                        rhs: Box::new(NirExpr::Bool(true)),
                    }))
                }
                "after-exit" => exit.push(NirStmt::Print(NirExpr::Int(9))),
                "both-exit" => continuation.push(NirStmt::Return(Some(NirExpr::Bool(true)))),
                _ => unreachable!(),
            }
        }
        let before = module.clone();
        assert!(outline_test(&mut module).is_empty(), "{mutation}");
        assert_eq!(module, before, "{mutation}");
    }
}

#[test]
fn conditional_return_effect_exits_share_return_root_and_expanded_tail_budgets() {
    let logical = |count| {
        (0..count).fold(NirExpr::Bool(true), |lhs, _| NirExpr::Binary {
            op: NirBinaryOp::And,
            lhs: Box::new(lhs),
            rhs: Box::new(NirExpr::Bool(true)),
        })
    };
    for edges in [16, 17] {
        let prefix = [NirStmt::If {
            condition: logical(16),
            then_body: vec![
                NirStmt::Print(NirExpr::Int(1)),
                NirStmt::Return(Some(logical(edges))),
            ],
            else_body: vec![],
        }];
        let tail = [NirStmt::Return(Some(NirExpr::Bool(false)))];
        let original = prefix.iter().chain(&tail).cloned().collect::<Vec<_>>();
        assert_eq!(
            effects::regions::bounds::preflight(&original, 1),
            edges == 16
        );
        assert_eq!(suffix::reserve_staged_prefix(&tail, &prefix), edges == 16);
    }
    for args in [4092, 4093] {
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
        let tail = [NirStmt::Return(Some(NirExpr::Bool(false)))];
        assert_eq!(suffix::reserve_staged_prefix(&tail, &prefix), args == 4092);
    }
    for depth in [63, 64] {
        let value = (0..depth).fold(NirExpr::Int(1), |lhs, _| NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(lhs),
            rhs: Box::new(NirExpr::Int(1)),
        });
        let prefix = [NirStmt::If {
            condition: NirExpr::Bool(true),
            then_body: vec![
                NirStmt::Print(NirExpr::Int(1)),
                NirStmt::Return(Some(value)),
            ],
            else_body: vec![],
        }];
        assert_eq!(
            suffix::reserve_staged_prefix(&[NirStmt::Return(Some(NirExpr::Int(0)))], &prefix),
            depth == 63
        );
    }
    for copies in [28, 29] {
        let text = source("atom", "false", false, "then", INPUTS[1]);
        let start = text.find("print(88);").unwrap();
        let end = text[start..].find("return ready;").unwrap() + start;
        let bindings = (0..copies)
            .map(|i| format!("let unused_{i} = gate;"))
            .collect::<String>();
        let text = format!(
            "{}if helper(produce(value)) {{ {bindings} print(1); return false; }} return true;{}",
            &text[..start],
            &text[end + "return ready;".len()..]
        );
        let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
        let before = module.clone();
        assert_eq!(!outline_test(&mut module).is_empty(), copies == 28);
        if copies == 29 {
            assert_eq!(module, before);
        }
    }
}
