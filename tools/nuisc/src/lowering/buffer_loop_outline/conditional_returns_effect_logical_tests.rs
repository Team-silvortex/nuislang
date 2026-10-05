use super::*;

#[path = "conditional_returns_effect_logical_fixtures.rs"]
mod fixtures;
use fixtures::{before_print_source, chain_source, expected, source, Input, INPUTS};
#[path = "conditional_returns_effect_logical_budget_tests.rs"]
mod budget;
#[path = "conditional_returns_effect_logical_native_tests.rs"]
mod native;

#[test]
fn conditional_return_logical_initializers_preserve_short_circuit_order_and_exits() {
    let mut cases = 0;
    for tree in 0..4 {
        for kind in ["call", "field", "division"] {
            for mode in ["return", "suffix", "zero"] {
                for shape in ["then", "else", "both"] {
                    for (declaration, input) in INPUTS.into_iter().enumerate() {
                        let text = source(tree, kind, mode, shape, declaration % 3, input);
                        let (result, prints, calls) = expected(tree, kind, mode, shape, input);
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
fn conditional_return_logical_initializers_reuse_snapshots_and_preserve_unused_work() {
    for input in INPUTS {
        let text = source(0, "call", "unused", "then", 2, input);
        let (result, prints, calls) = expected(0, "call", "unused", "then", input);
        execute(&text, result, &prints, calls);
        let (result, mut prints, calls) = expected(0, "call", "return", "then", input);
        if result.is_none() {
            prints.pop();
        }
        execute(&before_print_source(input), result, &prints, calls);
    }
    for gate in [false, true] {
        for left in [0, 2, -2] {
            let text = chain_source(gate, left);
            let selected_failure = gate && left == 0;
            let result = gate && left > 0;
            execute(
                &text,
                if selected_failure {
                    None
                } else {
                    Some(if result { 11 } else { 19 })
                },
                if selected_failure {
                    &[99, 88]
                } else if result {
                    &[99, 88, 89, 50, 90, 11]
                } else {
                    &[99, 88, 89, 50, 90, 19]
                },
                if selected_failure {
                    0
                } else {
                    usize::from(gate) + usize::from(result)
                },
            );
            if !selected_failure {
                for reversed in [false, true] {
                    let trace = events(&text, reversed);
                    let observe = trace
                        .iter()
                        .position(|e| e.contains("cpu.call_i64") && e.contains("] observe("))
                        .unwrap();
                    let prints = trace
                        .iter()
                        .enumerate()
                        .filter(|(_, e)| {
                            e.contains("cpu.guard_print ") && e.contains("if true then print")
                        })
                        .map(|(index, _)| index)
                        .collect::<Vec<_>>();
                    assert!(prints[1] < observe && observe < prints[2], "{trace:?}");
                    assert_eq!(trace.iter().filter(|e| e.contains("] observe(")).count(), 1);
                }
            }
        }
    }
    // Computed entry and logical initializer each run once, in source order.
    let input = Input {
        outer: true,
        gate: true,
        values: [2, 2, 2],
        tail: 2,
        early: false,
    };
    let text = source(0, "call", "return", "then", 1, input)
        .replace("if outer {", "if helper(produce(tail)) && outer {");
    execute(&text, Some(11), &[99, 88, 89, 11], 2);
}

#[test]
fn conditional_return_logical_initializers_keep_original_authority_and_atomic_vetoes() {
    let input = INPUTS[2];
    for mutation in [
        "hidden-comparison",
        "hidden-call",
        "hidden-print",
        "bool-print",
        "tail-shadow",
        "entry-private",
        "missing",
        "type",
        "no-return",
        "total-only",
    ] {
        let text = source(0, "call", "return", "both", 2, input);
        let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
        let parent = event(&mut module);
        let NirStmt::If {
            condition,
            then_body,
            else_body,
        } = &mut parent.body[2]
        else {
            panic!()
        };
        if mutation == "entry-private" {
            *condition = NirExpr::Var("__nuis_prefix_value_0".into());
        }
        let NirStmt::Const { value, ty, .. } = &mut else_body[1] else {
            panic!()
        };
        let logical = value.clone();
        match mutation {
            "hidden-comparison" => {
                *value = NirExpr::Binary {
                    op: NirBinaryOp::Eq,
                    lhs: Box::new(logical),
                    rhs: Box::new(NirExpr::Bool(true)),
                }
            }
            "hidden-call" => {
                *ty = scalar_type("i64");
                *value = NirExpr::Call {
                    callee: "to_word".into(),
                    args: vec![logical],
                };
            }
            "hidden-print" => {
                else_body[3] = NirStmt::Print(NirExpr::Call {
                    callee: "to_word".into(),
                    args: vec![logical],
                })
            }
            "bool-print" => else_body[3] = NirStmt::Print(logical),
            "missing" => {
                *value = NirExpr::Binary {
                    op: NirBinaryOp::And,
                    lhs: Box::new(NirExpr::Var("missing".into())),
                    rhs: Box::new(NirExpr::Bool(true)),
                }
            }
            "type" => *ty = scalar_type("i64"),
            "tail-shadow" => else_body.insert(
                4,
                NirStmt::Let {
                    name: "ready".into(),
                    ty: None,
                    value: NirExpr::Bool(true),
                },
            ),
            "no-return" => {
                then_body.pop();
                else_body.pop();
            }
            "total-only" => {
                for body in [then_body, else_body] {
                    let NirStmt::Const { value, .. } = &mut body[1] else {
                        panic!()
                    };
                    *value = NirExpr::Binary {
                        op: NirBinaryOp::And,
                        lhs: Box::new(NirExpr::Var("gate".into())),
                        rhs: Box::new(NirExpr::Bool(true)),
                    };
                }
            }
            "entry-private" => {}
            _ => unreachable!(),
        }
        let before = module.clone();
        assert!(outline_test(&mut module).is_empty(), "{mutation}");
        assert_eq!(module, before, "{mutation}");
    }
}

#[test]
fn conditional_return_logical_initializers_keep_helpers_pure_typed_and_idempotent() {
    let text = source(3, "call", "return", "both", 2, INPUTS[2]);
    let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
    let before = module.clone();
    let generated = outline_test(&mut module);
    assert_eq!(generated.len(), 3);
    let initializers = module
        .functions
        .iter()
        .filter(|f| f.name.starts_with("__nuis_conditional_prefix_value"))
        .collect::<Vec<_>>();
    assert_eq!(initializers.len(), 2);
    for initializer in initializers {
        assert_eq!(initializer.return_type, Some(scalar_type("bool")));
        assert_eq!(
            initializer
                .params
                .iter()
                .skip(1)
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>(),
            ["gate", "left", "middle", "right"]
        );
        let NirStmt::If { else_body, .. } = &initializer.body[0] else {
            panic!()
        };
        assert_eq!(else_body, &[NirStmt::Return(Some(NirExpr::Bool(false)))]);
    }
    for function in module
        .functions
        .iter()
        .filter(|f| generated.contains(&f.name))
    {
        assert!(!format!("{:?}", function.body).contains("Print("));
    }
    for name in ["produce", "helper", "observe", "to_word"] {
        assert_eq!(
            module.functions.iter().find(|f| f.name == name),
            before.functions.iter().find(|f| f.name == name)
        );
    }
    crate::nir_verify::verify_nir_module(&module).unwrap();
    let once = module.clone();
    assert!(outline_test(&mut module).is_empty());
    assert_eq!(module, once);
    let text = text
        .replace("ready", "__nuis_prefix_value_0")
        .replace("outer", "__nuis_effect_region_gate_0");
    let (result, prints, calls) = expected(3, "call", "return", "both", INPUTS[2]);
    execute(&text, result, &prints, calls);
}
