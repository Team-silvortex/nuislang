use super::super::tests::{execute, outline_test, source as simple_source};
use super::*;

#[path = "conditional_returns_logical_roots_native_tests.rs"]
mod native;

fn source(
    shape: &str,
    mode: &str,
    kind: &str,
    op: &str,
    input: (bool, bool, i64, i64, bool),
) -> String {
    let (outer, gate, divisor, tail, early) = input;
    let rhs = match kind {
        "call" => "helper(produce(divisor))",
        "field" => "produce(divisor).value > 0",
        "division" => "10 / divisor > 0",
        _ => unreachable!(),
    };
    let logical = format!("gate {op} {rhs}");
    let body = match mode {
        "complete" => format!("if {logical} {{ return true; }} else {{ return false; }}"),
        "partial" => format!("if {logical} {{ return false; }} else {{ let ignored = 20 / tail; }}"),
        "suffix" => format!("if {logical} {{ return false; }} let ignored = 20 / tail;"),
        "let" => format!("let current: bool = {logical}; return current;"),
        "const" => format!("const current: bool = {logical}; if current {{ return false; }} else {{ let ignored = 20 / tail; }}"),
        "continuation" => format!("if gate {{ return false; }} else {{ let ignored = {logical}; }}"),
        "suffix-binding" => format!("if gate {{ return false; }} let ignored = {logical};"),
        _ => unreachable!(),
    };
    let old = match shape {
        "then" => "if outer { return gate && helper(produce(divisor)); }",
        "else" => "if outer { } else { return gate && helper(produce(divisor)); }",
        "both" => "if outer { return gate && helper(produce(divisor)); } else { return helper(produce(divisor)); }",
        _ => unreachable!(),
    };
    let branch = match shape {
        "then" => format!("if outer {{ {body} }}"),
        "else" => format!("if outer {{ }} else {{ {body} }}"),
        "both" => format!("if outer {{ {body} }} else {{ {body} }}"),
        _ => unreachable!(),
    };
    simple_source(shape, "&&", outer, gate, divisor, early)
        .replace("early: bool) -> bool", "early: bool, tail: i64) -> bool")
        .replace(
            &format!("event({outer}, {gate}, {divisor}, {early})"),
            &format!("event({outer}, {gate}, {divisor}, {early}, {tail})"),
        )
        .replace(old, &branch)
}

fn selected(op: &str, gate: bool) -> bool {
    if op == "&&" {
        gate
    } else {
        !gate
    }
}

fn decision(op: &str, gate: bool, value: i64) -> bool {
    if selected(op, gate) {
        value > 0
    } else {
        gate
    }
}

#[test]
fn conditional_return_logical_roots_preserve_conditions_bindings_selected_checks_and_exits() {
    let mut cases = 0;
    for shape in ["then", "else", "both"] {
        for mode in ["complete", "partial", "suffix", "let", "const"] {
            for kind in ["call", "field", "division"] {
                for op in ["&&", "||"] {
                    for input in [
                        (true, false, 0, 2, false),
                        (true, true, 0, 0, false),
                        (false, false, 0, 0, false),
                        (true, true, 2, 0, false),
                        (false, false, -2, 2, false),
                        (true, false, -2, 0, false),
                        (true, true, 0, 0, true),
                        (false, false, 0, 0, true),
                    ] {
                        let (outer, gate, divisor, tail, early) = input;
                        let entered = !early
                            && (shape == "both" || if shape == "then" { outer } else { !outer });
                        let rhs = entered && selected(op, gate);
                        let condition = decision(op, gate, divisor);
                        let complete = matches!(mode, "complete" | "let");
                        let returned = entered && (complete || condition);
                        let failed =
                            rhs && divisor == 0 || entered && !complete && !condition && tail == 0;
                        let result = if entered && complete && condition {
                            11
                        } else {
                            19
                        };
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
                            usize::from(rhs && kind == "call"),
                        );
                        cases += 1;
                    }
                }
            }
        }
    }
    assert_eq!(cases, 720);
}

fn prefixed_source(kind: &str, op: &str, gate: bool, prefix: i64, divisor: i64) -> String {
    let (binding, rhs) = match kind {
        "alias" => (
            "let prefix_checked = 10 / prefix; let selected: bool = gate;",
            "helper(produce(divisor))",
        ),
        "computed" => (
            "let selected = helper(produce(prefix));",
            "helper(produce(divisor))",
        ),
        "record" => (
            "let packet = produce(prefix); let selected = gate;",
            "helper(packet)",
        ),
        _ => unreachable!(),
    };
    source(
        "then",
        "partial",
        "call",
        op,
        (true, gate, divisor, 2, false),
    )
    .replace("tail: i64) -> bool", "tail: i64, prefix: i64) -> bool")
    .replace(
        &format!("event(true, {gate}, {divisor}, false, 2)"),
        &format!("event(true, {gate}, {divisor}, false, 2, {prefix})"),
    )
    .replace(
        &format!("if gate {op} helper(produce(divisor))"),
        &format!("{binding} if selected {op} {rhs}"),
    )
}

#[test]
fn conditional_return_logical_roots_keep_earlier_local_checks_and_continuation_bindings() {
    let mut cases = 0;
    for kind in ["alias", "computed", "record"] {
        for op in ["&&", "||"] {
            for gate in [false, true] {
                for prefix in [-2, 0, 2] {
                    for divisor in [-2, 0, 2] {
                        let left = if kind == "computed" { prefix > 0 } else { gate };
                        let rhs = selected(op, left);
                        let returned =
                            decision(op, left, if kind == "record" { prefix } else { divisor });
                        let failed = prefix == 0 || rhs && kind != "record" && divisor == 0;
                        execute(
                            &prefixed_source(kind, op, gate, prefix, divisor),
                            (!failed).then_some(19),
                            if returned { &[99, 19] } else { &[99, 77, 19] },
                            usize::from(kind == "computed") + usize::from(rhs),
                        );
                        cases += 1;
                    }
                }
            }
        }
    }
    for mode in ["continuation", "suffix-binding"] {
        for op in ["&&", "||"] {
            for gate in [false, true] {
                for divisor in [-2, 0, 2] {
                    for tail in [-2, 0, 2] {
                        let rhs = !gate && selected(op, gate);
                        execute(
                            &source("then", mode, "call", op, (true, gate, divisor, tail, false)),
                            (!(rhs && divisor == 0)).then_some(19),
                            if gate { &[99, 19] } else { &[99, 77, 19] },
                            usize::from(rhs),
                        );
                        cases += 1;
                    }
                }
            }
        }
    }
    assert_eq!(cases, 180);
}

fn integer_source(
    outer_op: &str,
    op: &str,
    outer: bool,
    gate: bool,
    divisor: i64,
    tail: i64,
) -> String {
    format!("mod cpu Main {{
        struct Packet {{ unused: i64, value: i64 }}
        @noinline fn produce(value: i64) -> Packet {{ return Packet {{ unused: 10 / value, value: value }}; }}
        @noinline fn helper(packet: Packet) -> bool {{ return packet.value > 0; }}
        @noinline fn event(outer: bool, gate: bool, divisor: i64, tail: i64, entry: i64) -> i64 {{
            print(99); if outer {outer_op} helper(produce(entry)) {{
                if gate {op} helper(produce(divisor)) {{ return 0; }}
            }} print(77); return 20 / tail;
        }}
        fn main() -> i64 {{ let result = event({outer}, {gate}, {divisor}, {tail}, 2); print(result); return result; }}
    }}")
}

#[test]
fn conditional_return_logical_roots_compose_outer_entries_and_real_zero_parent_exits() {
    let mut cases = 0;
    for outer_op in ["&&", "||"] {
        for op in ["&&", "||"] {
            for outer in [false, true] {
                for gate in [false, true] {
                    for divisor in [-2, 0, 2] {
                        for tail in [0, 2] {
                            let entered = outer_op == "||" || outer;
                            let rhs = entered && selected(op, gate);
                            let returned = entered && decision(op, gate, divisor);
                            let failed = rhs && divisor == 0 || !returned && tail == 0;
                            let result = if returned { 0 } else { 10 };
                            execute(
                                &integer_source(outer_op, op, outer, gate, divisor, tail),
                                (!failed).then_some(result),
                                if returned { &[99, 0] } else { &[99, 77, 10] },
                                usize::from(selected(outer_op, outer)) + usize::from(rhs),
                            );
                            cases += 1;
                        }
                    }
                }
            }
        }
    }
    assert_eq!(cases, 96);
}

#[test]
fn conditional_return_logical_roots_keep_original_helper_roots_private_signals_and_hygiene() {
    for mode in [
        "complete",
        "partial",
        "suffix",
        "let",
        "const",
        "continuation",
        "suffix-binding",
    ] {
        let source = source("both", mode, "call", "&&", (true, false, 2, 2, false))
            .replace("print(99);", "let __nuis_return_gate_0 = gate; print(99);")
            .replace(
                "if outer {",
                "if outer { let __nuis_return_condition_0 = gate;",
            )
            .replace(
                "else { if gate",
                "else { let __nuis_return_condition_0 = gate; if gate",
            );
        let mut module = crate::frontend::parse_nuis_module(&source).unwrap();
        let before = module.clone();
        let generated = outline_test(&mut module);
        assert_eq!(generated.len(), 1, "{mode}");
        let old = before.functions.iter().find(|f| f.name == "event").unwrap();
        let new = module.functions.iter().find(|f| f.name == "event").unwrap();
        assert_eq!(&new.body[..3], &old.body[..3]);
        assert_eq!(
            &new.body[new.body.len() - 2..],
            &old.body[old.body.len() - 2..]
        );
        let helper = module
            .functions
            .iter()
            .find(|f| generated.contains(&f.name))
            .unwrap();
        assert_ne!(helper.params[0].name, "__nuis_return_condition_0");
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
        assert!(outline_test(&mut module).is_empty(), "{mode}");
        assert_eq!(module, once);
        let compiled = crate::pipeline::compile_source(&source).unwrap();
        assert!(compiled
            .yir
            .functions
            .iter()
            .any(|f| f.name.contains("__nuis_conditional_value")));
    }
}

fn tree(depth: usize) -> NirExpr {
    if depth == 0 {
        return NirExpr::Int(1);
    }
    NirExpr::Binary {
        op: NirBinaryOp::Add,
        lhs: Box::new(tree(depth - 1)),
        rhs: Box::new(tree(depth - 1)),
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
fn conditional_return_logical_roots_share_original_expanded_statement_node_and_depth_budgets() {
    let base = "mod cpu Main { fn decide(value: i64) -> bool { return value > 0; } fn narrow(value: i32) -> bool { return true; } fn event(outer: bool, gate: bool, entry: i64) -> bool { print(99); if outer { if gate && decide(entry) { return true; } else { return false; } } print(77); return false; } fn main() -> i64 { return 0; } }";
    for over in [false, true] {
        let mut module = crate::frontend::parse_nuis_module(base).unwrap();
        let event = module
            .functions
            .iter_mut()
            .find(|f| f.name == "event")
            .unwrap();
        let NirStmt::If { then_body, .. } = &mut event.body[1] else {
            panic!()
        };
        let NirStmt::If {
            condition: NirExpr::Binary { rhs, .. },
            ..
        } = &mut then_body[0]
        else {
            panic!()
        };
        let mut value = tree(11);
        let NirExpr::Binary {
            lhs, rhs: right, ..
        } = &mut value
        else {
            panic!()
        };
        prune(lhs);
        prune(right);
        // 4091 argument nodes + call + logical root + gate + two returns = 4096.
        **rhs = NirExpr::Call {
            callee: if over { "narrow" } else { "decide" }.into(),
            args: vec![if over {
                NirExpr::CastI64ToI32(Box::new(value))
            } else {
                value
            }],
        };
        let before = module.clone();
        assert_eq!(outline_test(&mut module).is_empty(), over);
        if over {
            assert_eq!(module, before);
        }
    }
    for count in [61, 62] {
        let mut module = crate::frontend::parse_nuis_module(base).unwrap();
        let event = module
            .functions
            .iter_mut()
            .find(|f| f.name == "event")
            .unwrap();
        let NirStmt::If { then_body, .. } = &mut event.body[1] else {
            panic!()
        };
        let NirStmt::If {
            condition: NirExpr::Binary { rhs, .. },
            ..
        } = &mut then_body[0]
        else {
            panic!()
        };
        let mut value = NirExpr::Var("entry".into());
        for _ in 0..count {
            value = NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(value),
                rhs: Box::new(NirExpr::Int(1)),
            };
        }
        **rhs = NirExpr::Call {
            callee: "decide".into(),
            args: vec![value],
        };
        let before = module.clone();
        assert_eq!(outline_test(&mut module).is_empty(), count == 62);
        if count == 62 {
            assert_eq!(module, before);
        }
    }
    for count in [29, 30] {
        let locals = (0..count)
            .map(|n| format!("let fresh{n} = gate;"))
            .collect::<String>();
        let source = base.replace(
            "if gate && decide(entry)",
            &format!("{locals} if gate && decide(entry)"),
        );
        let mut module = crate::frontend::parse_nuis_module(&source).unwrap();
        let before = module.clone();
        assert_eq!(outline_test(&mut module).is_empty(), count == 30);
        if count == 30 {
            assert_eq!(module, before);
        }
    }
    for over in [false, true] {
        let locals = (0..13)
            .map(|n| format!("let tail{n} = gate && decide(entry);"))
            .collect::<String>();
        let body = format!("let first = entry; {} if gate && decide(entry) {{ let left = entry; }} else {{ let right = entry; }} {locals} return false;", if over { "let second = entry;" } else { "" });
        let source = base.replace(
            "if gate && decide(entry) { return true; } else { return false; }",
            &body,
        );
        let mut module = crate::frontend::parse_nuis_module(&source).unwrap();
        let before = module.clone();
        assert_eq!(outline_test(&mut module).is_empty(), over);
        if over {
            assert_eq!(module, before);
        }
    }
}

#[test]
fn conditional_return_logical_roots_retain_nested_derived_gate_scope_effect_and_capture_vetoes() {
    let base = source("then", "partial", "call", "&&", (true, true, 2, 2, false));
    for condition in [
        "helper(produce(divisor)) && gate",
        "(gate == true) && helper(produce(divisor))",
    ] {
        let source = base.replace("gate && helper(produce(divisor))", condition);
        let mut module = crate::frontend::parse_nuis_module(&source).unwrap();
        assert_eq!(outline_test(&mut module).len(), 1, "{condition}");
        crate::nir_verify::verify_nir_module(&module).unwrap();
        execute(&source, Some(19), &[99, 19], 1);
    }
    for condition in ["gate && (gate || helper(produce(divisor)))"] {
        let source = base.replace("gate && helper(produce(divisor))", condition);
        let mut module = crate::frontend::parse_nuis_module(&source).unwrap();
        assert_eq!(outline_test(&mut module).len(), 1, "{condition}");
        crate::nir_verify::verify_nir_module(&module).unwrap();
        execute(&source, Some(19), &[99, 19], 0);
    }
    for mutation in [
        "borrow",
        "optional",
        "generic",
        "declared",
        "effect",
        "rebind",
        "aggregate",
        "no-return",
        "scope",
    ] {
        let mut module = crate::frontend::parse_nuis_module(&source(
            "then",
            "const",
            "call",
            "&&",
            (true, true, 2, 2, false),
        ))
        .unwrap();
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
                "borrow" => event.params[1].ty.is_ref = true,
                "optional" => event.params[1].ty.is_optional = true,
                "generic" => event.params[1].ty.generic_args.push(scalar_type("i64")),
                _ => {
                    let NirStmt::If { then_body, .. } = &mut event.body[2] else {
                        panic!()
                    };
                    match mutation {
                        "declared" => {
                            let NirStmt::Const { ty, .. } = &mut then_body[0] else {
                                panic!()
                            };
                            *ty = scalar_type("i64");
                        }
                        "rebind" => {
                            let NirStmt::Const { name, .. } = &mut then_body[0] else {
                                panic!()
                            };
                            *name = "gate".into();
                        }
                        "aggregate" => {
                            let NirStmt::Const {
                                value: NirExpr::Binary { rhs, .. },
                                ..
                            } = &mut then_body[0]
                            else {
                                panic!()
                            };
                            **rhs = NirExpr::Call {
                                callee: "helper".into(),
                                args: vec![NirExpr::Var("packet".into())],
                            };
                            event.params.push(NirParam {
                                name: "packet".into(),
                                ty: scalar_type("Packet"),
                            });
                        }
                        "scope" => {
                            let NirStmt::Const {
                                value: NirExpr::Binary { lhs, .. },
                                ..
                            } = &mut then_body[0]
                            else {
                                panic!()
                            };
                            **lhs = NirExpr::Var("missing".into());
                        }
                        _ => {
                            let NirStmt::If {
                                then_body: returned,
                                ..
                            } = &mut then_body[1]
                            else {
                                panic!()
                            };
                            returned.clear();
                        }
                    }
                }
            }
        }
        let before = module.clone();
        assert!(outline_test(&mut module).is_empty(), "{mutation}");
        assert_eq!(module, before);
    }
}
