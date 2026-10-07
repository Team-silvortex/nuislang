use super::super::tests::{execute, outline_test, source as simple_source};
use super::*;

#[path = "conditional_returns_suffix_native_tests.rs"]
mod native;

fn arm(kind: &str, op: &str) -> String {
    match kind {
        "complete" | "partial" => format!("let packet = produce(2); if gate {{ return gate {op} helper(produce(divisor)); }} else {{ let local = helper(packet); }} let tail = helper(produce(suffix)); {}", if kind == "complete" { "return false;" } else { "" }),
        "computed" => format!("let packet = produce(2); if helper(produce(divisor)) {{ return gate {op} helper(produce(divisor)); }} else {{ let local = helper(packet); }} let tail = helper(produce(suffix));"),
        "sequence" => "if gate { return false; } if nested { return helper(produce(divisor)); } let tail = helper(produce(suffix));".into(),
        "diamond" => "if gate { let left = helper(produce(divisor)); } else { let right = helper(produce(2)); } let after = helper(produce(suffix)); if nested { return after; } else { let unused = 10 / suffix; }".into(),
        _ => unreachable!(),
    }
}

#[allow(clippy::too_many_arguments)]
fn source(
    shape: &str,
    kind: &str,
    op: &str,
    outer: bool,
    gate: bool,
    nested: bool,
    divisor: i64,
    suffix: i64,
    early: bool,
) -> String {
    let body = arm(kind, op);
    let branch = match shape {
        "then" => format!("if outer {{ {body} }}"),
        "else" => format!("if outer {{ }} else {{ {body} }}"),
        "both" => format!("if outer {{ {body} }} else {{ {body} }}"),
        _ => unreachable!(),
    };
    simple_source("then", op, outer, gate, divisor, early)
        .replace(
            "early: bool) -> bool",
            "early: bool, nested: bool, suffix: i64) -> bool",
        )
        .replace(
            &format!("event({outer}, {gate}, {divisor}, {early})"),
            &format!("event({outer}, {gate}, {divisor}, {early}, {nested}, {suffix})"),
        )
        .replace(
            &format!("if outer {{ return gate {op} helper(produce(divisor)); }}"),
            &branch,
        )
}

fn outcome(
    kind: &str,
    op: &str,
    gate: bool,
    nested: bool,
    divisor: i64,
    suffix: i64,
) -> (bool, bool, bool, usize) {
    match kind {
        "complete" | "partial" => {
            let rhs = gate && op == "&&";
            (
                gate || kind == "complete",
                gate && if rhs { divisor > 0 } else { true },
                rhs && divisor == 0 || !gate && suffix == 0,
                if gate { usize::from(rhs) } else { 2 },
            )
        }
        "computed" => {
            let selected = divisor > 0;
            let rhs = selected && if op == "&&" { gate } else { !gate };
            (
                selected,
                selected && if rhs { divisor > 0 } else { gate },
                divisor == 0 || !selected && suffix == 0,
                1 + if selected { usize::from(rhs) } else { 2 },
            )
        }
        "sequence" => (
            gate || nested,
            !gate && nested && divisor > 0,
            !gate && if nested { divisor == 0 } else { suffix == 0 },
            usize::from(!gate),
        ),
        "diamond" => (
            nested,
            nested && suffix > 0,
            gate && divisor == 0 || suffix == 0,
            2,
        ),
        _ => unreachable!(),
    }
}

#[test]
fn conditional_return_suffixes_run_only_continuing_paths_and_preserve_selected_checks() {
    let mut cases = 0;
    for kind in ["complete", "partial", "computed", "sequence", "diamond"] {
        for shape in ["then", "else", "both"] {
            for op in ["&&", "||"] {
                for outer in [false, true] {
                    for (gate, nested, divisor, suffix) in [
                        (true, false, 0, 0),
                        (false, true, 2, 0),
                        (false, false, -2, 2),
                        (true, true, 2, -2),
                        (false, true, -2, -2),
                        (true, false, 2, 0),
                    ] {
                        let entered = match shape {
                            "then" => outer,
                            "else" => !outer,
                            "both" => true,
                            _ => unreachable!(),
                        };
                        let (returned, value, failed, calls) =
                            outcome(kind, op, gate, nested, divisor, suffix);
                        let result = if entered && value { 11 } else { 19 };
                        let prints = if entered && returned {
                            vec![99, result]
                        } else {
                            vec![99, 77, result]
                        };
                        execute(
                            &source(shape, kind, op, outer, gate, nested, divisor, suffix, false),
                            (!(entered && failed)).then_some(result),
                            &prints,
                            if entered { calls } else { 0 },
                        );
                        cases += 1;
                    }
                }
            }
        }
    }
    assert_eq!(cases, 360);
}

#[test]
fn conditional_return_suffixes_keep_earlier_parent_exits_and_current_scalar_versions() {
    let mut cases = 0;
    for kind in ["complete", "partial", "computed", "sequence", "diamond"] {
        for shape in ["then", "else", "both"] {
            for op in ["&&", "||"] {
                execute(
                    &source(shape, kind, op, true, true, true, 0, 0, true),
                    Some(19),
                    &[19],
                    0,
                );
                cases += 1;
            }
        }
    }
    for (gate, divisor, suffix, expected, prints, calls) in [
        (true, 0, -2, Some(11), vec![99, 11], 1),
        (false, 0, 0, Some(19), vec![99, 77, 19], 2),
        (true, -4, -2, Some(19), vec![99, 19], 1),
        (false, 0, -2, None, vec![], 0),
    ] {
        let source = source(
            "then", "partial", "&&", true, gate, false, divisor, suffix, false,
        )
        .replace(
            "print(99);",
            "let divisor = divisor + 2; let suffix = suffix + 2; print(99);",
        );
        execute(&source, expected, &prints, calls);
        cases += 1;
    }
    assert_eq!(cases, 34);
}

fn integer_source(outer: bool, gate: bool, nested: bool, divisor: i64, suffix: i64) -> String {
    format!("mod cpu Main {{ @noinline fn event(outer: bool, gate: bool, nested: bool, divisor: i64, suffix: i64) -> i64 {{ print(99); if outer {{ if gate {{ return 0; }} let tail = 10 / suffix; if nested {{ return 10 / divisor - 5; }} let ignored = 20 / divisor; }} print(77); return 19; }} fn main() -> i64 {{ let result = event({outer}, {gate}, {nested}, {divisor}, {suffix}); print(result); return result; }} }}")
}

#[test]
fn conditional_return_suffixes_keep_real_zero_exits_separate_from_parent_continuations() {
    let mut cases = 0;
    for outer in [false, true] {
        for gate in [false, true] {
            for nested in [false, true] {
                for divisor in [0, 2, -2] {
                    for suffix in [0, 2] {
                        let returned = outer && (gate || nested);
                        let failed = outer && !gate && (suffix == 0 || divisor == 0);
                        let result = if outer && gate {
                            0
                        } else if outer && nested && divisor != 0 {
                            10 / divisor - 5
                        } else {
                            19
                        };
                        let prints = if returned {
                            vec![99, result]
                        } else {
                            vec![99, 77, result]
                        };
                        execute(
                            &integer_source(outer, gate, nested, divisor, suffix),
                            (!failed).then_some(result),
                            &prints,
                            0,
                        );
                        cases += 1;
                    }
                }
            }
        }
    }
    assert_eq!(cases, 48);
}

#[test]
fn conditional_return_suffixes_preserve_original_conditions_parent_tail_and_helper_hygiene() {
    for kind in ["partial", "computed", "sequence", "diamond"] {
        let source = source("both", kind, "&&", true, true, true, 2, 2, false)
            .replace("print(99);", "let __nuis_return_gate_0 = gate; print(99);")
            .replace(
                &arm(kind, "&&"),
                &format!("let __nuis_return_condition_0 = gate; {}", arm(kind, "&&")),
            );
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
        let helper = module
            .functions
            .iter()
            .find(|f| generated.contains(&f.name))
            .unwrap();
        assert_ne!(helper.params[0].name, "__nuis_return_condition_0");
        let NirStmt::If {
            then_body,
            else_body,
            ..
        } = &helper.body[0]
        else {
            panic!()
        };
        for arm in [then_body, else_body] {
            // The normalized helper always ends in a branch, not an eager tail.
            assert!(matches!(arm.last(), Some(NirStmt::If { .. })));
            let mut bindings = BTreeSet::new();
            branches::collect_bindings(arm, &mut bindings);
            assert!(!bindings.contains(&helper.params[0].name));
        }
        crate::nir_verify::verify_nir_module(&module).unwrap();
        let once = module.clone();
        assert!(outline_test(&mut module).is_empty());
        assert_eq!(module, once);
    }
}

fn candidate(body: &str) -> String {
    simple_source("then", "&&", true, true, 2, false).replace(
        "if outer { return gate && helper(produce(divisor)); }",
        &format!("if helper(produce(divisor)) {{ {body} }}"),
    )
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

fn prune_left_leaf(value: &mut NirExpr) {
    if matches!(value, NirExpr::Binary { lhs, .. } if matches!(lhs.as_ref(), NirExpr::Int(_))) {
        *value = NirExpr::Int(1);
    } else {
        let NirExpr::Binary { lhs, .. } = value else {
            panic!()
        };
        prune_left_leaf(lhs);
    }
}

#[test]
fn conditional_return_suffixes_bound_original_and_expanded_work_before_expression_cloning() {
    for count in [30, 31] {
        let tail = (0..count)
            .map(|n| format!("let fresh{n} = divisor;"))
            .collect::<String>();
        let source = candidate(&format!(
            "if gate {{ return helper(produce(divisor)); }} {tail}"
        ));
        let mut module = crate::frontend::parse_nuis_module(&source).unwrap();
        let before = module.clone();
        assert_eq!(outline_test(&mut module).is_empty(), count == 31);
        if count == 31 {
            assert_eq!(module, before);
        } else {
            crate::nir_verify::verify_nir_module(&module).unwrap();
        }
    }
    for over in [false, true] {
        let tail = (0..12)
            .map(|n| format!("let fresh{n} = divisor;"))
            .collect::<String>();
        let source = candidate(&format!("let root = divisor; {} if gate {{ let left = divisor; }} else {{ let right = divisor; }} {tail} if root > 0 {{ return helper(produce(divisor)); }}", if over { "let extra = divisor;" } else { "" }));
        let mut module = crate::frontend::parse_nuis_module(&source).unwrap();
        let before = module.clone();
        assert_eq!(outline_test(&mut module).is_empty(), over);
        if over {
            assert_eq!(module, before);
        } else {
            crate::nir_verify::verify_nir_module(&module).unwrap();
        }
    }
    for over in [false, true] {
        let mut module = crate::frontend::parse_nuis_module(&candidate(
            "if gate { let left = divisor; } let ignored = divisor; return false;",
        ))
        .unwrap();
        let event = module
            .functions
            .iter_mut()
            .find(|f| f.name == "event")
            .unwrap();
        let NirStmt::If { then_body, .. } = &mut event.body[2] else {
            panic!()
        };
        let NirStmt::Let { value, ty, .. } = &mut then_body[1] else {
            panic!()
        };
        let mut root = tree(10);
        // Prune one two-leaf node: 2047 - 2, then one cast gives 2046.
        prune_left_leaf(&mut root);
        *value = NirExpr::CastI64ToI32(Box::new(root));
        *ty = Some(scalar_type("i32"));
        if over {
            then_body.insert(
                0,
                NirStmt::Let {
                    name: "extra".into(),
                    ty: None,
                    value: NirExpr::Int(1),
                },
            );
        }
        let before = module.clone();
        assert_eq!(outline_test(&mut module).is_empty(), over);
        if over {
            assert_eq!(module, before);
        } else {
            crate::nir_verify::verify_nir_module(&module).unwrap();
        }
    }
    for count in [61, 62] {
        let mut module = crate::frontend::parse_nuis_module(&candidate(
            "if gate { return false; } let ignored = helper(produce(divisor));",
        ))
        .unwrap();
        let event = module
            .functions
            .iter_mut()
            .find(|f| f.name == "event")
            .unwrap();
        let NirStmt::If { then_body, .. } = &mut event.body[2] else {
            panic!()
        };
        let NirStmt::Let {
            value: NirExpr::Call { args, .. },
            ..
        } = &mut then_body[1]
        else {
            panic!()
        };
        let NirExpr::Call { args, .. } = &mut args[0] else {
            panic!()
        };
        let mut value = NirExpr::Var("divisor".into());
        for _ in 0..count {
            value = NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(value),
                rhs: Box::new(NirExpr::Int(1)),
            };
        }
        args[0] = value;
        let before = module.clone();
        assert_eq!(outline_test(&mut module).is_empty(), count == 62);
        if count == 62 {
            assert_eq!(module, before);
        }
    }
}

#[test]
fn conditional_return_suffixes_reject_scope_leaks_effects_rebinding_unreachable_work_and_new_exit_authority(
) {
    for (body, calls) in [
        (
            "if gate && helper(produce(divisor)) { return false; } let ignored = divisor;",
            2,
        ),
        (
            "if gate { return false; } let ignored = gate && helper(produce(divisor));",
            1,
        ),
    ] {
        execute(&candidate(body), Some(19), &[99, 19], calls);
    }
    for body in [
        "if gate { return 0; } print(88); let ignored = helper(produce(divisor));",
        "if gate { return false; } let divisor = 2; let ignored = helper(produce(divisor));",
        "if gate { return false; } while gate { let ignored = divisor; }",
        "if gate { return false; } else { return true; } let ignored = helper(produce(divisor));",
        "if gate { let inside = divisor; } let inside = divisor; return helper(produce(inside));",
        "if gate { let left = divisor; } let ignored = helper(produce(divisor));",
    ] {
        // Unsupported source scopes and effects do not acquire exit authority.
        let source = candidate(body);
        let mut module = crate::frontend::parse_nuis_module(&source).unwrap();
        let before = module.clone();
        assert!(outline_test(&mut module).is_empty(), "{body}");
        assert_eq!(module, before);
    }
    let admitted =
        candidate("if gate { return false; } print(88); let ignored = helper(produce(divisor));");
    execute(&admitted, Some(19), &[99, 19], 1);
    execute(
        &admitted.replace(
            "event(true, true, 2, false)",
            "event(true, false, 2, false)",
        ),
        Some(19),
        &[99, 88, 77, 19],
        2,
    );
    let source = candidate("if gate { let inside = divisor; } else { let inside = divisor; } return helper(produce(divisor));");
    let mut module = crate::frontend::parse_nuis_module(&source).unwrap();
    let event = module
        .functions
        .iter_mut()
        .find(|f| f.name == "event")
        .unwrap();
    let NirStmt::If { then_body, .. } = &mut event.body[2] else {
        panic!()
    };
    let NirStmt::Return(Some(NirExpr::Call { args, .. })) = &mut then_body[1] else {
        panic!()
    };
    let NirExpr::Call { args, .. } = &mut args[0] else {
        panic!()
    };
    args[0] = NirExpr::Var("inside".into());
    let before = module.clone();
    assert!(outline_test(&mut module).is_empty());
    assert_eq!(module, before);
    for mutation in ["borrow", "optional", "kind", "impure", "declared"] {
        let mut module = crate::frontend::parse_nuis_module(&candidate(
            "if gate { return false; } let ignored = helper(produce(divisor));",
        ))
        .unwrap();
        if mutation == "impure" {
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
                "kind" => event.params[1].ty = scalar_type("i64"),
                "declared" => {
                    let NirStmt::If { then_body, .. } = &mut event.body[2] else {
                        panic!()
                    };
                    let NirStmt::Let { ty, .. } = &mut then_body[1] else {
                        panic!()
                    };
                    *ty = Some(scalar_type("i64"));
                }
                _ => unreachable!(),
            }
        }
        let before = module.clone();
        assert!(outline_test(&mut module).is_empty(), "{mutation}");
        assert_eq!(module, before);
    }
}
