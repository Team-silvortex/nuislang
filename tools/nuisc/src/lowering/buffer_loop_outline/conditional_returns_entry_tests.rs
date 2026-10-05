use super::super::tests::{execute, outline_test, source as simple_source};
use super::*;

#[path = "conditional_returns_entry_native_tests.rs"]
mod native;

fn source(
    shape: &str,
    kind: &str,
    op: &str,
    entry: i64,
    gate: bool,
    divisor: i64,
    early: bool,
) -> String {
    let condition = match kind {
        "call" => "helper(produce(entry))",
        "field" => "produce(entry).value > 0",
        "call-compare" => "helper(produce(entry)) == true",
        "division" => "10 / entry > 0",
        "nested" => "(entry + 0 > 0) == true",
        _ => unreachable!(),
    };
    simple_source(shape, op, true, gate, divisor, early)
        .replace("early: bool) -> bool", "early: bool, entry: i64) -> bool")
        .replace(
            &format!("event(true, {gate}, {divisor}, {early})"),
            &format!("event(true, {gate}, {divisor}, {early}, {entry})"),
        )
        .replace("if outer {", &format!("if {condition} {{"))
}

#[test]
fn conditional_return_entry_predicates_evaluate_once_at_original_site_before_complete_arms() {
    let mut cases = 0;
    for kind in ["call", "field", "call-compare", "division", "nested"] {
        for shape in ["then", "else", "both"] {
            for op in ["&&", "||"] {
                for (entry, gate, divisor, early) in [
                    (2, true, 2, false),
                    (-2, true, 2, false),
                    (2, false, 0, false),
                    (-2, true, 0, false),
                    (0, true, 2, false),
                    (0, true, 0, true),
                ] {
                    let selected = entry > 0;
                    let entered = !early
                        && match shape {
                            "then" => selected,
                            "else" => !selected,
                            "both" => true,
                            _ => unreachable!(),
                        };
                    let rhs = entered
                        && (shape == "both" && !selected || if op == "&&" { gate } else { !gate });
                    let failed = !early && kind != "nested" && entry == 0 || rhs && divisor == 0;
                    let value = entered && if rhs { divisor > 0 } else { gate };
                    let result = if value { 11 } else { 19 };
                    let prints = if early {
                        vec![19]
                    } else if entered {
                        vec![99, result]
                    } else {
                        vec![99, 77, result]
                    };
                    execute(
                        &source(shape, kind, op, entry, gate, divisor, early),
                        (!failed).then_some(result),
                        &prints,
                        usize::from(!early && matches!(kind, "call" | "call-compare"))
                            + usize::from(rhs),
                    );
                    cases += 1;
                }
            }
        }
    }
    assert_eq!(cases, 180);
}

fn partial_source(stored: bool, entry: i64, gate: bool, divisor: i64) -> String {
    let source = source("then", "call", "&&", entry, gate, divisor, false);
    let condition = if stored {
        "helper(produce(divisor))"
    } else {
        "gate"
    };
    source.replace(
        "return gate && helper(produce(divisor));",
        &format!("if {condition} {{ return false; }} else {{ let ignored = 10 / divisor; }}"),
    )
}

#[test]
fn conditional_return_entry_predicates_feed_saved_bool_to_replay_and_stored_exit_routes() {
    let mut cases = 0;
    for stored in [false, true] {
        for (entry, gate, divisor) in [
            (2, true, 2),
            (2, false, 2),
            (2, false, -2),
            (-2, true, 0),
            (2, true, 0),
            (0, true, 2),
        ] {
            let returned = entry > 0 && if stored { divisor > 0 } else { gate };
            let failed = entry == 0 || entry > 0 && divisor == 0 && (stored || !gate);
            execute(
                &partial_source(stored, entry, gate, divisor),
                (!failed).then_some(19),
                if returned { &[99, 19] } else { &[99, 77, 19] },
                1 + usize::from(stored && entry > 0),
            );
            cases += 1;
        }
    }
    assert_eq!(cases, 12);
}

#[test]
fn conditional_return_entry_predicates_keep_current_parent_records_and_condition_only_work_local() {
    let mut cases = 0;
    for entry in [-4, 0] {
        for gate in [false, true] {
            let source = source("then", "call", "&&", entry, gate, 2, false)
                .replace(
                    "print(99);",
                    "let entry = entry + 2; let packet = produce(entry); print(99);",
                )
                .replace("if helper(produce(entry)) {", "if packet.value > 0 {");
            let selected = entry + 2 > 0;
            let result = if selected && gate { 11 } else { 19 };
            let prints = if selected {
                vec![99, result]
            } else {
                vec![99, 77, result]
            };
            execute(
                &source,
                Some(result),
                &prints,
                usize::from(selected && gate),
            );
            let mut module = crate::frontend::parse_nuis_module(&source).unwrap();
            let generated = outline_test(&mut module);
            assert_eq!(generated.len(), 1);
            let helper = module
                .functions
                .iter()
                .find(|f| generated.contains(&f.name))
                .unwrap();
            assert!(!helper
                .params
                .iter()
                .any(|p| p.name == "packet" || p.name == "entry"));
            cases += 1;
        }
    }
    for entry in [-2, 2, 0] {
        let source = source("both", "call", "&&", entry, true, 0, false)
            .replace("return gate && helper(produce(divisor));", "return false;")
            .replace(
                "else { return helper(produce(divisor)); }",
                "else { return false; }",
            );
        execute(&source, (entry != 0).then_some(19), &[99, 19], 1);
        cases += 1;
    }
    for divisor in [0, 2] {
        let source = format!("mod cpu Main {{ @noinline fn event(entry: i64, divisor: i64) -> i64 {{ print(99); if 10 / entry > 0 {{ return 0; }} print(77); return 10 / divisor; }} fn main() -> i64 {{ let result = event(2, {divisor}); print(result); return result; }} }}");
        execute(&source, Some(0), &[99, 0], 0);
        cases += 1;
    }
    assert_eq!(cases, 9);
}

#[test]
fn conditional_return_entry_predicates_preserve_source_site_hygiene_and_no_helper_replay() {
    for stored in [false, true] {
        let source = partial_source(stored, 2, true, 2)
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
        let NirStmt::If {
            condition: original,
            ..
        } = &old.body[3]
        else {
            panic!()
        };
        let NirStmt::Let { name, ty, value } = &new.body[3] else {
            panic!()
        };
        assert_eq!(value, original);
        assert_eq!(ty.as_ref(), Some(&scalar_type("bool")));
        assert_ne!(name, "__nuis_return_gate_0");
        let helper = module
            .functions
            .iter()
            .find(|f| generated.contains(&f.name))
            .unwrap();
        assert!(!helper.params.iter().any(|p| p.name == "entry"));
        let NirStmt::If { condition, .. } = &helper.body[0] else {
            panic!()
        };
        assert_eq!(condition, &NirExpr::Var(helper.params[0].name.clone()));
        crate::nir_verify::verify_nir_module(&module).unwrap();
        let once = module.clone();
        assert!(outline_test(&mut module).is_empty());
        assert_eq!(module, once);
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

#[test]
fn conditional_return_entry_predicates_preflight_4096_nodes_and_depth_before_recursive_typing() {
    let base = "mod cpu Main { fn decide(value: i64) -> bool { return value > 0; } fn narrow(value: i32) -> bool { return true; } fn event(entry: i64) -> i64 { print(99); if decide(entry) { return 10 / entry; } print(77); return 0; } fn main() -> i64 { return event(2); } }";
    for over in [false, true] {
        let mut module = crate::frontend::parse_nuis_module(base).unwrap();
        let event = module
            .functions
            .iter_mut()
            .find(|f| f.name == "event")
            .unwrap();
        let NirStmt::If { condition, .. } = &mut event.body[1] else {
            panic!()
        };
        *condition = NirExpr::Call {
            callee: if over { "narrow" } else { "decide" }.into(),
            args: vec![if over {
                NirExpr::CastI64ToI32(Box::new(tree(11)))
            } else {
                tree(11)
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
    for count in [62, 63] {
        let mut module = crate::frontend::parse_nuis_module(base).unwrap();
        let event = module
            .functions
            .iter_mut()
            .find(|f| f.name == "event")
            .unwrap();
        let NirStmt::If { condition, .. } = &mut event.body[1] else {
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
        *condition = NirExpr::Call {
            callee: "decide".into(),
            args: vec![value],
        };
        let before = module.clone();
        assert_eq!(outline_test(&mut module).is_empty(), count == 63);
        if count == 63 {
            assert_eq!(module, before);
        }
    }
}

#[test]
fn conditional_return_entry_predicates_retain_logical_effect_type_and_capture_vetoes() {
    let base = source("then", "call", "&&", 2, true, 2, false);
    // Keep the former single-edge rejection as independent positive evidence.
    let logical = base.replace("helper(produce(entry))", "gate && helper(produce(entry))");
    execute(&logical, Some(11), &[99, 11], 2);
    let computed = base.replace("helper(produce(entry))", "helper(produce(entry)) || gate");
    execute(&computed, Some(11), &[99, 11], 2);
    for replacement in ["entry"] {
        let source = base.replace("helper(produce(entry))", replacement);
        let mut module = crate::frontend::parse_nuis_module(&source).unwrap();
        let before = module.clone();
        assert!(outline_test(&mut module).is_empty(), "{replacement}");
        assert_eq!(module, before);
    }
    for mutation in [
        "borrow",
        "optional",
        "generic",
        "effect",
        "branch",
        "no-return",
        "capture",
        "unknown",
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
                "borrow" => event.params[4].ty.is_ref = true,
                "optional" => event.params[4].ty.is_optional = true,
                "generic" => event.params[4].ty.generic_args.push(scalar_type("i64")),
                "capture" => event.params[1].ty.is_ref = true,
                "unknown" => {
                    let NirStmt::If { condition, .. } = &mut event.body[2] else {
                        panic!()
                    };
                    *condition = NirExpr::Call {
                        callee: "unknown".into(),
                        args: vec![NirExpr::Var("entry".into())],
                    };
                }
                _ => {
                    let NirStmt::If { then_body, .. } = &mut event.body[2] else {
                        panic!()
                    };
                    if mutation == "branch" {
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
