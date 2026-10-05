use super::super::tests::{execute, outline_test};
use super::*;

#[path = "conditional_returns_computed_arm_native_tests.rs"]
mod native;

type Input = (bool, bool, i64, i64, i64, bool);

const INPUTS: [Input; 8] = [
    (true, false, -2, 0, 2, false),
    (true, true, 2, 0, 0, false),
    (false, false, 0, 0, 0, false),
    (true, true, 2, 2, 0, false),
    (false, true, -2, -2, 2, false),
    (true, false, 0, 2, 2, false),
    (true, true, 0, 0, 0, true),
    (false, false, 2, -2, 2, false),
];

fn source(shape: &str, mode: &str, kind: &str, op: &str, input: Input) -> String {
    let (outer, gate, left, right, tail, early) = input;
    let (lhs, rhs) = match kind {
        "call" => ("helper(produce(left))", "helper(produce(right))"),
        "field" => ("produce(left).value > 0", "10 / right > 0"),
        "division" => ("10 / left > 0", "produce(right).value > 0"),
        "comparison" => ("left > 0", "right > 0"),
        _ => unreachable!(),
    };
    let value = format!("({lhs}) {op} ({rhs})");
    let body = match mode {
        "complete" => format!("if {value} {{ return true; }} else {{ return false; }}"),
        "partial" => format!("if {value} {{ return false; }} else {{ let checked = 20 / tail; }}"),
        "suffix" => format!("if {value} {{ return false; }} let checked = 20 / tail;"),
        "let" => format!("let current: bool = {value}; return current;"),
        "inferred" => format!("let current = {value}; return current;"),
        "const" => format!("const current: bool = {value}; if current {{ return false; }} else {{ let checked = 20 / tail; }}"),
        "return" => format!("return {value};"),
        "continuation" => format!("if gate {{ return false; }} else {{ let ignored = {value}; let checked = 20 / tail; }}"),
        "suffix-binding" => format!("if gate {{ return false; }} let ignored = {value}; let checked = 20 / tail;"),
        _ => unreachable!(),
    };
    let branch = match shape {
        "then" => format!("if outer {{ {body} }}"),
        "else" => format!("if outer {{ }} else {{ {body} }}"),
        "both" => format!("if outer {{ {body} }} else {{ {body} }}"),
        _ => unreachable!(),
    };
    format!(
        "mod cpu Main {{
        struct Packet {{ unused: i64, value: i64 }}
        @noinline fn produce(input: i64) -> Packet {{ return Packet {{ unused: 10 / input, value: input }}; }}
        @noinline fn helper(packet: Packet) -> bool {{ return packet.value > 0; }}
        @noinline fn event(outer: bool, gate: bool, left: i64, right: i64, tail: i64, early: bool) -> bool {{
            if early {{ return false; }} print(99); {branch} print(77); return false;
        }}
        fn main() -> i64 {{
            let result = event({outer}, {gate}, {left}, {right}, {tail}, {early});
            if result {{ print(11); return 11; }} print(19); return 19;
        }}
    }}"
    )
}

fn entered(shape: &str, input: Input) -> bool {
    !input.5 && (shape == "both" || if shape == "then" { input.0 } else { !input.0 })
}

fn selected(op: &str, left: i64) -> bool {
    if op == "&&" {
        left > 0
    } else {
        left <= 0
    }
}

fn decision(op: &str, left: i64, right: i64) -> bool {
    if selected(op, left) {
        right > 0
    } else {
        left > 0
    }
}

fn check(shape: &str, mode: &str, kind: &str, op: &str, input: Input) {
    let (_, gate, left, right, tail, early) = input;
    let entered = entered(shape, input);
    let continuation = matches!(mode, "continuation" | "suffix-binding");
    let evaluated = entered && (!continuation || !gate);
    let rhs = evaluated && selected(op, left);
    let complete = matches!(mode, "complete" | "let" | "inferred" | "return");
    let value = decision(op, left, right);
    let returned = entered
        && if continuation {
            gate
        } else {
            complete || value
        };
    let failed = kind != "comparison" && (evaluated && left == 0 || rhs && right == 0)
        || entered && !returned && tail == 0;
    let result = if entered && complete && value { 11 } else { 19 };
    let prints = if early {
        vec![19]
    } else if returned {
        vec![99, result]
    } else {
        vec![99, 77, result]
    };
    execute(
        &source(shape, mode, kind, op, input),
        (!failed).then_some(result),
        &prints,
        usize::from(kind == "call" && evaluated) + usize::from(kind == "call" && rhs),
    );
}

#[test]
fn conditional_return_computed_arms_preserve_selected_conditions_binding_roots_and_real_exits() {
    let mut cases = 0;
    for shape in ["then", "else", "both"] {
        for mode in ["complete", "partial", "suffix", "let", "const", "return"] {
            for kind in ["call", "field", "division"] {
                for op in ["&&", "||"] {
                    for input in INPUTS {
                        check(shape, mode, kind, op, input);
                        cases += 1;
                    }
                }
            }
        }
    }
    assert_eq!(cases, 864);
}

#[test]
fn conditional_return_computed_arms_keep_continuation_bindings_inferred_roots_and_total_comparisons(
) {
    let mut cases = 0;
    for mode in ["continuation", "suffix-binding", "inferred"] {
        for kind in ["call", "field", "division", "comparison"] {
            for op in ["&&", "||"] {
                for input in INPUTS {
                    check("both", mode, kind, op, input);
                    cases += 1;
                }
            }
        }
    }
    assert_eq!(cases, 192);
}

fn prefixed_source(kind: &str, op: &str, input: Input) -> String {
    let prefix = match kind {
        "record" => "let packet = produce(left);",
        "checked" => "let prefix_checked = 10 / left; let packet = produce(left);",
        "comparison" => "let packet = produce(left); const current: bool = gate == true;",
        _ => unreachable!(),
    };
    let lhs = if kind == "comparison" {
        "current == gate"
    } else {
        "helper(packet)"
    };
    source("then", "partial", "call", op, input).replace(
        &format!("if (helper(produce(left))) {op} (helper(produce(right)))"),
        &format!("{prefix} if ({lhs}) {op} helper(produce(right))"),
    )
}

#[test]
fn conditional_return_computed_arms_retain_local_record_checks_without_parent_record_captures() {
    let mut cases = 0;
    for kind in ["record", "checked", "comparison"] {
        for op in ["&&", "||"] {
            for input in INPUTS {
                let (_, _, left, right, tail, early) = input;
                let entered = entered("then", input);
                let lhs = if kind == "comparison" { 1 } else { left };
                let rhs = entered && selected(op, lhs);
                let returned = entered && decision(op, lhs, right);
                let failed =
                    entered && left == 0 || rhs && right == 0 || entered && !returned && tail == 0;
                execute(
                    &prefixed_source(kind, op, input),
                    (!failed).then_some(19),
                    if early {
                        &[19]
                    } else if returned {
                        &[99, 19]
                    } else {
                        &[99, 77, 19]
                    },
                    usize::from(entered && kind != "comparison") + usize::from(rhs),
                );
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 48);
}

fn integer_source(op: &str, input: Input) -> String {
    source("both", "partial", "call", op, input)
        .replace("early: bool) -> bool", "early: bool) -> i64")
        .replace("return false;", "return 0;")
        .replace("print(77); return 0;", "print(77); return 20 / tail;")
        .replace(
            "if result { print(11); return 11; } print(19); return 19;",
            "print(result); return result;",
        )
}

#[test]
fn conditional_return_computed_arms_keep_zero_exits_separate_from_fallible_parent_tails() {
    let mut cases = 0;
    for op in ["&&", "||"] {
        for input in INPUTS {
            let (_, _, left, right, tail, early) = input;
            let evaluated = !early;
            let rhs = evaluated && selected(op, left);
            let returned = early || decision(op, left, right);
            let failed = evaluated && left == 0 || rhs && right == 0 || !returned && tail == 0;
            let result = if returned { 0 } else { 10 };
            execute(
                &integer_source(op, input),
                (!failed).then_some(result),
                if early {
                    &[0]
                } else if returned {
                    &[99, 0]
                } else {
                    &[99, 77, 10]
                },
                usize::from(evaluated) + usize::from(rhs),
            );
            cases += 1;
        }
    }
    assert_eq!(cases, 16);
}

#[test]
fn conditional_return_computed_arms_preserve_original_tree_private_signals_scalar_captures_and_idempotence(
) {
    for mode in [
        "complete",
        "partial",
        "suffix",
        "let",
        "const",
        "return",
        "continuation",
        "suffix-binding",
    ] {
        let text = source("both", mode, "call", "||", (true, false, 2, 0, 0, false));
        let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
        let before = module.clone();
        let generated = outline_test(&mut module);
        assert_eq!(generated.len(), 1, "{mode}");
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
        assert!(helper.params.iter().all(|p| scalar(&p.ty)));
        if matches!(mode, "partial" | "const") {
            let signal = module
                .structs
                .iter()
                .find(|s| s.name == helper.return_type.as_ref().unwrap().name)
                .unwrap();
            assert_eq!(
                signal
                    .fields
                    .iter()
                    .map(|f| f.name.as_str())
                    .collect::<Vec<_>>(),
                ["exited", "value"]
            );
        }
        crate::nir_verify::verify_nir_module(&module).unwrap();
        let once = module.clone();
        assert!(outline_test(&mut module).is_empty());
        assert_eq!(module, once);
        let compiled = crate::pipeline::compile_source(&text).unwrap();
        assert!(compiled
            .yir
            .functions
            .iter()
            .any(|f| f.name.contains("__nuis_conditional_value")));
    }
    let text = prefixed_source("record", "&&", (true, false, -2, 0, 2, false));
    let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
    let generated = outline_test(&mut module);
    assert_eq!(generated.len(), 1);
    assert!(module
        .functions
        .iter()
        .find(|f| generated.contains(&f.name))
        .unwrap()
        .params
        .iter()
        .all(|p| scalar(&p.ty)));
}

fn tree(depth: usize) -> NirExpr {
    if depth == 0 {
        NirExpr::Int(1)
    } else {
        NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(tree(depth - 1)),
            rhs: Box::new(tree(depth - 1)),
        }
    }
}

fn prune(value: &mut NirExpr) {
    let NirExpr::Binary { lhs, rhs, .. } = value else {
        panic!()
    };
    if matches!(lhs.as_ref(), NirExpr::Int(_)) && matches!(rhs.as_ref(), NirExpr::Int(_)) {
        *value = NirExpr::Int(1);
    } else {
        prune(lhs);
    }
}

#[test]
fn conditional_return_computed_arms_bound_whole_original_expanded_roots_before_cloning() {
    let base = "mod cpu Main {
        fn decide(value: i64) -> bool { return value > 0; }
        fn narrow(value: i32) -> bool { return true; }
        fn event(outer: bool, gate: bool, left: i64, right: i64) -> bool {
            print(99); if outer { if decide(left) && gate { return true; } else { return false; } } print(77); return false;
        }
    }";
    for over in [false, true] {
        let mut module = crate::frontend::parse_nuis_module(base).unwrap();
        let mut argument = tree(11);
        let NirExpr::Binary { lhs, rhs, .. } = &mut argument else {
            panic!()
        };
        prune(lhs);
        prune(rhs);
        let event = module
            .functions
            .iter_mut()
            .find(|f| f.name == "event")
            .unwrap();
        let NirStmt::If { then_body, .. } = &mut event.body[1] else {
            panic!()
        };
        let NirStmt::If {
            condition: NirExpr::Binary { lhs, .. },
            ..
        } = &mut then_body[0]
        else {
            panic!()
        };
        // 4091 argument nodes + call + logical root + RHS atom + two returns = 4096.
        **lhs = NirExpr::Call {
            callee: if over { "narrow" } else { "decide" }.into(),
            args: vec![if over {
                NirExpr::CastI64ToI32(Box::new(argument))
            } else {
                argument
            }],
        };
        assert_eq!(bounded(then_body), !over);
        let before = module.clone();
        assert_eq!(outline_test(&mut module).is_empty(), over);
        if over {
            assert_eq!(module, before);
        }
    }
    for count in [61, 62] {
        let mut module = crate::frontend::parse_nuis_module(base).unwrap();
        let mut argument = NirExpr::Var("left".into());
        for _ in 0..count {
            argument = NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(argument),
                rhs: Box::new(NirExpr::Int(1)),
            };
        }
        let event = module
            .functions
            .iter_mut()
            .find(|f| f.name == "event")
            .unwrap();
        let NirStmt::If { then_body, .. } = &mut event.body[1] else {
            panic!()
        };
        let NirStmt::If {
            condition: NirExpr::Binary { lhs, .. },
            ..
        } = &mut then_body[0]
        else {
            panic!()
        };
        **lhs = NirExpr::Call {
            callee: "decide".into(),
            args: vec![argument],
        };
        assert_eq!(bounded(then_body), count == 61);
        let before = module.clone();
        assert_eq!(outline_test(&mut module).is_empty(), count == 62);
        if count == 62 {
            assert_eq!(module, before);
        }
    }
    for count in [29, 30] {
        let locals = (0..count)
            .map(|n| format!("let local{n} = left;"))
            .collect::<String>();
        let text = base.replace("if decide(left)", &format!("{locals} if decide(left)"));
        let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
        let before = module.clone();
        assert_eq!(outline_test(&mut module).is_empty(), count == 30);
        if count == 30 {
            assert_eq!(module, before);
        }
    }
    for over in [false, true] {
        let locals = (0..13)
            .map(|n| format!("let tail{n} = decide(left) && decide(right);"))
            .collect::<String>();
        let body = format!("let first = left; {} if decide(left) && decide(right) {{ let yes = left; }} else {{ let no = right; }} {locals} return false;", if over { "let second = right;" } else { "" });
        let text = base.replace(
            "if decide(left) && gate { return true; } else { return false; }",
            &body,
        );
        let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
        let before = module.clone();
        assert_eq!(outline_test(&mut module).is_empty(), over);
        if over {
            assert_eq!(module, before);
        }
    }
    let computed = NirExpr::Binary {
        op: NirBinaryOp::And,
        lhs: Box::new(NirExpr::Binary {
            op: NirBinaryOp::Gt,
            lhs: Box::new(NirExpr::Int(1)),
            rhs: Box::new(NirExpr::Int(0)),
        }),
        rhs: Box::new(NirExpr::Bool(true)),
    };
    assert!(conditional_values::prefix::computed_expression_roots(vec![
        (&computed, 0, true)
    ]));
    assert!(!conditional_values::prefix::expression_roots(vec![(
        &computed, 0, true
    )]));
    assert!(!conditional_values::prefix::bounded(&[NirStmt::Let {
        name: "value".into(),
        ty: Some(scalar_type("bool")),
        value: computed
    }]));
}

#[test]
fn conditional_return_computed_arms_retain_nested_effect_type_scope_loop_and_capture_vetoes() {
    let base = source(
        "then",
        "complete",
        "call",
        "&&",
        (true, true, 2, 2, 2, false),
    );
    for condition in [
        "(helper(produce(left)) && gate) && helper(produce(right))",
        "helper(produce(left)) && (gate || helper(produce(right)))",
    ] {
        let text = base.replace(
            "(helper(produce(left))) && (helper(produce(right)))",
            condition,
        );
        let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
        assert_eq!(outline_test(&mut module).len(), 1, "{condition}");
        crate::nir_verify::verify_nir_module(&module).unwrap();
        execute(
            &text,
            Some(11),
            &[99, 11],
            if condition.contains("||") { 1 } else { 2 },
        );
    }
    for mutation in [
        "borrow", "optional", "generic", "effect", "rebind", "scope", "branch", "loop", "unknown",
        "rhs-kind",
    ] {
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
            match mutation {
                "borrow" => event.params[2].ty.is_ref = true,
                "optional" => event.params[2].ty.is_optional = true,
                "generic" => event.params[2].ty.generic_args.push(scalar_type("i64")),
                "unknown" | "rhs-kind" => {
                    let NirStmt::If { then_body, .. } = &mut event.body[2] else {
                        panic!()
                    };
                    let NirStmt::If {
                        condition: NirExpr::Binary { rhs, .. },
                        ..
                    } = &mut then_body[0]
                    else {
                        panic!()
                    };
                    **rhs = if mutation == "unknown" {
                        NirExpr::Call {
                            callee: "unknown".into(),
                            args: vec![NirExpr::Var("right".into())],
                        }
                    } else {
                        NirExpr::Var("right".into())
                    };
                }
                "loop" => {
                    let body = std::mem::take(&mut event.body);
                    event.body = vec![
                        NirStmt::While {
                            condition: NirExpr::Bool(false),
                            body,
                        },
                        NirStmt::Return(Some(NirExpr::Bool(false))),
                    ];
                }
                _ => {
                    let NirStmt::If { then_body, .. } = &mut event.body[2] else {
                        panic!()
                    };
                    then_body.insert(
                        0,
                        match mutation {
                            "branch" => NirStmt::Print(NirExpr::Int(88)),
                            "scope" => NirStmt::Let {
                                name: "fresh".into(),
                                ty: None,
                                value: NirExpr::Var("missing".into()),
                            },
                            _ => NirStmt::Let {
                                name: "left".into(),
                                ty: Some(scalar_type("i64")),
                                value: NirExpr::Int(2),
                            },
                        },
                    );
                }
            }
        }
        if mutation == "branch" {
            // Keep the old veto fixture as a positive leading-parent-print proof.
            assert_eq!(outline_test(&mut module).len(), 1);
            crate::nir_verify::verify_nir_module(&module).unwrap();
            continue;
        }
        let before = module.clone();
        assert!(outline_test(&mut module).is_empty(), "{mutation}");
        assert_eq!(module, before);
    }
    let parent_record = base
        .replace("print(99);", "let packet = produce(left); print(99);")
        .replace("helper(produce(left))", "helper(packet)");
    let mut module = crate::frontend::parse_nuis_module(&parent_record).unwrap();
    let before = module.clone();
    assert!(outline_test(&mut module).is_empty());
    assert_eq!(module, before);
}
