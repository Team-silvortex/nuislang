use super::super::tests::{execute, outline_test, source as simple_source};
use super::*;

#[path = "conditional_returns_entry_logical_native_tests.rs"]
mod native;

fn source(shape: &str, kind: &str, op: &str, input: (bool, i64, bool, i64, bool)) -> String {
    let (outer, entry, gate, divisor, early) = input;
    let rhs = match kind {
        "call" => "helper(produce(entry))",
        "field" => "produce(entry).value > 0",
        "division" => "10 / entry > 0",
        _ => unreachable!(),
    };
    simple_source(shape, "&&", outer, gate, divisor, early)
        .replace("early: bool) -> bool", "early: bool, entry: i64) -> bool")
        .replace(
            &format!("event({outer}, {gate}, {divisor}, {early})"),
            &format!("event({outer}, {gate}, {divisor}, {early}, {entry})"),
        )
        .replace("if outer {", &format!("if outer {op} {rhs} {{"))
}

fn selected(op: &str, outer: bool) -> bool {
    if op == "&&" {
        outer
    } else {
        !outer
    }
}

fn decision(op: &str, outer: bool, entry: i64) -> bool {
    if selected(op, outer) {
        entry > 0
    } else {
        outer
    }
}

#[test]
fn conditional_return_logical_entries_preserve_selected_rhs_once_and_complete_arm_order() {
    let mut cases = 0;
    for kind in ["call", "field", "division"] {
        for shape in ["then", "else", "both"] {
            for op in ["&&", "||"] {
                for input in [
                    (false, 0, true, 0, false),
                    (true, 0, false, 0, false),
                    (true, 2, true, 2, false),
                    (false, -2, false, 0, false),
                    (true, -2, true, 0, false),
                    (false, 2, true, 0, false),
                    (true, 0, true, 0, true),
                    (false, 0, false, 2, true),
                ] {
                    let (outer, entry, gate, divisor, early) = input;
                    let condition = decision(op, outer, entry);
                    let entered = !early
                        && (shape == "both"
                            || if shape == "then" {
                                condition
                            } else {
                                !condition
                            });
                    let arm_rhs = entered && (shape == "both" && !condition || gate);
                    let failed =
                        !early && selected(op, outer) && entry == 0 || arm_rhs && divisor == 0;
                    let value = entered && if arm_rhs { divisor > 0 } else { gate };
                    let result = if value { 11 } else { 19 };
                    let prints = if early {
                        vec![19]
                    } else if entered {
                        vec![99, result]
                    } else {
                        vec![99, 77, result]
                    };
                    execute(
                        &source(shape, kind, op, input),
                        (!failed).then_some(result),
                        &prints,
                        usize::from(!early && selected(op, outer) && kind == "call")
                            + usize::from(arm_rhs),
                    );
                    cases += 1;
                }
            }
        }
    }
    assert_eq!(cases, 144);
}

fn partial_source(
    kind: &str,
    op: &str,
    outer: bool,
    entry: i64,
    gate: bool,
    divisor: i64,
) -> String {
    let body = match kind {
        "replay" => "if gate { return false; } else { let ignored = 10 / divisor; }",
        "stored" => {
            "if helper(produce(divisor)) { return false; } else { let ignored = 10 / divisor; }"
        }
        "suffix" => "if gate { return false; } let ignored = 10 / divisor;",
        _ => unreachable!(),
    };
    source("then", "call", op, (outer, entry, gate, divisor, false))
        .replace("return gate && helper(produce(divisor));", body)
}

#[test]
fn conditional_return_logical_entries_compose_replay_stored_signals_and_intermediate_suffixes() {
    let mut cases = 0;
    for kind in ["replay", "stored", "suffix"] {
        for op in ["&&", "||"] {
            for outer in [false, true] {
                for entry in [-2, 0, 2] {
                    for gate in [false, true] {
                        for divisor in [-2, 0, 2] {
                            let condition = decision(op, outer, entry);
                            let returned =
                                condition && if kind == "stored" { divisor > 0 } else { gate };
                            let failed = selected(op, outer) && entry == 0
                                || condition && divisor == 0 && (kind == "stored" || !gate);
                            execute(
                                &partial_source(kind, op, outer, entry, gate, divisor),
                                (!failed).then_some(19),
                                if returned { &[99, 19] } else { &[99, 77, 19] },
                                usize::from(selected(op, outer))
                                    + usize::from(condition && kind == "stored"),
                            );
                            cases += 1;
                        }
                    }
                }
            }
        }
    }
    assert_eq!(cases, 216);
}

fn integer_source(op: &str, outer: bool, entry: i64, gate: bool, late: i64) -> String {
    format!("mod cpu Main {{
        struct Packet {{ unused: i64, value: i64 }}
        @noinline fn produce(value: i64) -> Packet {{ return Packet {{ unused: 10 / value, value: value }}; }}
        @noinline fn helper(packet: Packet) -> bool {{ return packet.value > 0; }}
        @noinline fn event(outer: bool, entry: i64, gate: bool, late: i64) -> i64 {{
            print(99); if outer {op} helper(produce(entry)) {{
                if gate {{ return 0; }} else {{ let ignored = 10 / 2; }}
            }} print(77); return 20 / late;
        }}
        fn main() -> i64 {{ let result = event({outer}, {entry}, {gate}, {late}); print(result); return result; }}
    }}")
}

#[test]
fn conditional_return_logical_entries_keep_real_zero_exits_fallible_parent_tails_and_current_gates()
{
    let mut cases = 0;
    for op in ["&&", "||"] {
        for outer in [false, true] {
            for entry in [-2, 0, 2] {
                for gate in [false, true] {
                    for late in [0, 2] {
                        let returned = decision(op, outer, entry) && gate;
                        let failed = selected(op, outer) && entry == 0 || !returned && late == 0;
                        let result = if returned { 0 } else { 10 };
                        execute(
                            &integer_source(op, outer, entry, gate, late),
                            (!failed).then_some(result),
                            if returned { &[99, 0] } else { &[99, 77, 10] },
                            usize::from(selected(op, outer)),
                        );
                        cases += 1;
                    }
                }
            }
            let source = source("then", "call", op, (outer, 2, false, 0, false))
                .replace("print(99);", "print(99); let outer = outer == false;");
            execute(
                &source,
                Some(19),
                if decision(op, !outer, 2) {
                    &[99, 19]
                } else {
                    &[99, 77, 19]
                },
                usize::from(selected(op, !outer)),
            );
            cases += 1;
        }
    }
    assert_eq!(cases, 52);
}

#[test]
fn conditional_return_logical_entries_store_original_value_root_without_replaying_helper_inputs() {
    for op in ["&&", "||"] {
        let source = partial_source("stored", op, true, 2, false, 2)
            .replace("print(99);", "let __nuis_return_gate_0 = gate; print(99);");
        let mut module = crate::frontend::parse_nuis_module(&source).unwrap();
        let before = module.clone();
        let generated = outline_test(&mut module);
        assert_eq!(generated.len(), 1);
        let old = before.functions.iter().find(|f| f.name == "event").unwrap();
        let new = module.functions.iter().find(|f| f.name == "event").unwrap();
        assert_eq!(&new.body[..3], &old.body[..3]);
        assert_eq!(
            &new.body[new.body.len() - 2..],
            &old.body[old.body.len() - 2..]
        );
        let NirStmt::If { condition, .. } = &old.body[3] else {
            panic!()
        };
        let NirStmt::Let { name, ty, value } = &new.body[3] else {
            panic!()
        };
        assert_ne!(name, "__nuis_return_gate_0");
        assert_eq!(ty.as_ref(), Some(&scalar_type("bool")));
        assert_eq!(value, condition);
        let helper = module
            .functions
            .iter()
            .find(|f| generated.contains(&f.name))
            .unwrap();
        assert!(!helper
            .params
            .iter()
            .any(|p| p.name == "entry" || p.name == "outer"));
        let NirStmt::If { condition, .. } = &helper.body[0] else {
            panic!()
        };
        assert_eq!(condition, &NirExpr::Var(helper.params[0].name.clone()));
        crate::nir_verify::verify_nir_module(&module).unwrap();
        let once = module.clone();
        assert!(outline_test(&mut module).is_empty());
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
fn conditional_return_logical_entries_share_whole_root_4096_node_and_depth_limits_before_typing() {
    let base = "mod cpu Main { fn decide(value: i64) -> bool { return value > 0; } fn narrow(value: i32) -> bool { return true; } fn event(gate: bool, entry: i64) -> i64 { print(99); if gate && decide(entry) { return 10 / entry; } print(77); return 0; } fn main() -> i64 { return event(true, 2); } }";
    for over in [false, true] {
        let mut module = crate::frontend::parse_nuis_module(base).unwrap();
        let event = module
            .functions
            .iter_mut()
            .find(|f| f.name == "event")
            .unwrap();
        let NirStmt::If {
            condition: NirExpr::Binary { rhs, .. },
            ..
        } = &mut event.body[1]
        else {
            panic!()
        };
        let mut value = tree(11);
        prune(&mut value);
        // 4093 argument nodes + call + logical root + gate = exactly 4096.
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
        } else {
            crate::nir_verify::verify_nir_module(&module).unwrap();
        }
    }
    for count in [61, 62] {
        let mut module = crate::frontend::parse_nuis_module(base).unwrap();
        let event = module
            .functions
            .iter_mut()
            .find(|f| f.name == "event")
            .unwrap();
        let NirStmt::If {
            condition: NirExpr::Binary { rhs, .. },
            ..
        } = &mut event.body[1]
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
}

#[test]
fn conditional_return_logical_entries_retain_nested_derived_gate_effect_type_and_capture_vetoes() {
    let base = source("then", "call", "&&", (true, 2, true, 2, false));
    for condition in [
        "helper(produce(entry)) && outer",
        "(outer == true) && helper(produce(entry))",
    ] {
        let admitted = base.replace("outer && helper(produce(entry))", condition);
        execute(&admitted, Some(11), &[99, 11], 2);
    }
    for (condition, calls) in [
        ("outer && (gate || helper(produce(entry)))", 1),
        ("outer && (helper(produce(entry)) && gate)", 2),
    ] {
        let admitted = base.replace("outer && helper(produce(entry))", condition);
        execute(&admitted, Some(11), &[99, 11], calls);
    }
    for condition in ["outer && produce(entry)"] {
        let source = base.replace("outer && helper(produce(entry))", condition);
        let mut module = match crate::frontend::parse_nuis_module(&source) {
            Ok(module) => module,
            Err(error) => {
                assert_eq!(condition, "outer && produce(entry)");
                assert!(error.contains("matching operand types"), "{error}");
                continue;
            }
        };
        let before = module.clone();
        assert!(outline_test(&mut module).is_empty(), "{condition}");
        assert_eq!(module, before);
    }
    for mutation in [
        "borrow-gate",
        "optional-gate",
        "generic-gate",
        "wrong-gate",
        "borrow-rhs",
        "effect",
        "branch-effect",
        "no-return",
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
                "borrow-gate" => event.params[0].ty.is_ref = true,
                "optional-gate" => event.params[0].ty.is_optional = true,
                "generic-gate" => event.params[0].ty.generic_args.push(scalar_type("i64")),
                "wrong-gate" => event.params[0].ty = scalar_type("i64"),
                "borrow-rhs" => event.params[4].ty.is_ref = true,
                _ => {
                    let NirStmt::If { then_body, .. } = &mut event.body[2] else {
                        panic!()
                    };
                    if mutation == "branch-effect" {
                        then_body.insert(0, NirStmt::Print(NirExpr::Int(88)));
                    } else {
                        *then_body = vec![NirStmt::Let {
                            name: "ignored".into(),
                            ty: None,
                            value: NirExpr::Int(1),
                        }];
                    }
                }
            }
        }
        let before = module.clone();
        assert!(outline_test(&mut module).is_empty(), "{mutation}");
        assert_eq!(module, before);
    }
}
