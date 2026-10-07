use super::super::tests::{
    assert_selected_effect_not_pure, execute, outline_test, source as simple_source,
};
use super::*;

#[path = "conditional_returns_fallthrough_native_tests.rs"]
mod native;

fn body(kind: &str, op: &str) -> String {
    let value = format!("gate {op} helper(produce(divisor))");
    match kind {
        "then" => format!("if nested {{ return {value}; }}"),
        "else" => format!("if nested {{ }} else {{ return {value}; }}"),
        "packet" => format!("let packet = produce(divisor); if nested {{ return gate {op} helper(packet); }}"),
        "split" => format!("if nested {{ if other {{ return {value}; }} }} else {{ if other {{ }} else {{ return {value}; }} }}"),
        "hygiene" => format!("const __nuis_return_exit_0: bool = gate; if nested {{ let __nuis_return_condition_0: bool = gate; return __nuis_return_condition_0 {op} helper(produce(divisor)); }}"),
        _ => unreachable!(),
    }
}

#[allow(clippy::too_many_arguments)]
fn source(
    shape: &str,
    kind: &str,
    op: &str,
    outer: bool,
    nested: bool,
    other: bool,
    gate: bool,
    divisor: i64,
    early: bool,
) -> String {
    let arm = body(kind, op);
    simple_source(shape, op, outer, gate, divisor, early)
        .replace(
            "early: bool) -> bool",
            "early: bool, nested: bool, other: bool) -> bool",
        )
        .replace(
            &format!("event({outer}, {gate}, {divisor}, {early})"),
            &format!("event({outer}, {gate}, {divisor}, {early}, {nested}, {other})"),
        )
        .replace(&format!("return gate {op} helper(produce(divisor));"), &arm)
        .replace(
            "else { return helper(produce(divisor)); }",
            &format!("else {{ {arm} }}"),
        )
}

#[test]
fn conditional_return_fallthrough_trees_distinguish_false_returns_from_continuations() {
    for shape in ["then", "else", "both"] {
        for kind in ["then", "else", "packet"] {
            for outer in [false, true] {
                for nested in [false, true] {
                    for early in [false, true] {
                        for (op, gate, divisor) in [
                            ("&&", false, 0),
                            ("||", true, 0),
                            ("&&", true, 2),
                            ("||", false, -2),
                            ("&&", true, 0),
                            ("||", false, 0),
                        ] {
                            let entered = !early
                                && (shape == "both"
                                    || if shape == "then" { outer } else { !outer });
                            let returned = entered && if kind == "else" { !nested } else { nested };
                            let rhs = returned && if op == "&&" { gate } else { !gate };
                            let trapped = divisor == 0 && (rhs || (entered && kind == "packet"));
                            let value = returned && if rhs { divisor > 0 } else { gate };
                            let result = if value { 11 } else { 19 };
                            let mut prints = Vec::new();
                            if !early {
                                prints.push(99);
                                if !returned {
                                    prints.push(77);
                                }
                            }
                            prints.push(result);
                            execute(
                                &source(
                                    shape, kind, op, outer, nested, false, gate, divisor, early,
                                ),
                                (!trapped).then_some(result),
                                &prints,
                                usize::from(rhs),
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn conditional_return_fallthrough_trees_keep_independent_deep_and_outer_arm_exit_choices() {
    for shape in ["then", "else", "both"] {
        for outer in [false, true] {
            for nested in [false, true] {
                for other in [false, true] {
                    for divisor in [0, 2, -2] {
                        let entered =
                            shape == "both" || if shape == "then" { outer } else { !outer };
                        let returned = entered && nested == other;
                        let result = if returned && divisor > 0 { 11 } else { 19 };
                        let prints = if returned {
                            vec![99, result]
                        } else {
                            vec![99, 77, result]
                        };
                        execute(
                            &source(
                                shape, "split", "&&", outer, nested, other, true, divisor, false,
                            ),
                            (!(returned && divisor == 0)).then_some(result),
                            &prints,
                            usize::from(returned),
                        );
                    }
                }
            }
        }
    }
    for outer in [false, true] {
        for nested in [false, true] {
            for other in [false, true] {
                for complete in [false, true] {
                    let no = if complete {
                        "return helper(produce(divisor));"
                    } else {
                        "if other { } else { return gate && helper(produce(divisor)); }"
                    };
                    let source =
                        source("both", "then", "&&", outer, nested, other, false, 2, false)
                            .replace(
                                &format!("else {{ {} }}", body("then", "&&")),
                                &format!("else {{ {no} }}"),
                            );
                    let returned = if outer { nested } else { complete || !other };
                    let result = if !outer && complete { 11 } else { 19 };
                    let prints = if returned {
                        vec![99, result]
                    } else {
                        vec![99, 77, result]
                    };
                    execute(
                        &source,
                        Some(result),
                        &prints,
                        usize::from(!outer && complete),
                    );
                }
            }
        }
    }
}

fn integer_source(shape: &str, outer: bool, nested: bool, divisor: i64, declared: &str) -> String {
    let arm = format!("if nested {{ {declared} = 10 / divisor - 5; return current; }}");
    let branch = match shape {
        "then" => format!("if outer {{ {arm} }}"),
        "else" => format!("if outer {{ }} else {{ {arm} }}"),
        "both" => format!("if outer {{ {arm} }} else {{ {arm} }}"),
        _ => unreachable!(),
    };
    format!("mod cpu Main {{ @noinline fn event(outer: bool, nested: bool, divisor: i64) -> i64 {{ print(99); {branch} print(77); return 19; }} fn main() -> i64 {{ return event({outer}, {nested}, {divisor}); }} }}")
}

#[test]
fn conditional_return_fallthrough_trees_distinguish_zero_returns_and_keep_literal_and_current_gates(
) {
    for shape in ["then", "else", "both"] {
        for outer in [false, true] {
            for nested in [false, true] {
                for divisor in [0, 2, -2] {
                    for declared in ["let current", "const current: i64"] {
                        let returned = nested
                            && (shape == "both" || if shape == "then" { outer } else { !outer });
                        let expected = if returned && divisor == 0 {
                            None
                        } else if returned {
                            Some(10 / divisor - 5)
                        } else {
                            Some(19)
                        };
                        execute(
                            &integer_source(shape, outer, nested, divisor, declared),
                            expected,
                            if returned { &[99] } else { &[99, 77] },
                            0,
                        );
                    }
                }
            }
        }
    }
    for literal in [false, true] {
        for gate in [false, true] {
            let literal_source =
                source("then", "then", "&&", true, !literal, false, gate, 2, false)
                    .replace("if nested {", &format!("if {literal} {{"));
            let result = if literal && gate { 11 } else { 19 };
            let prints = if literal {
                vec![99, result]
            } else {
                vec![99, 77, result]
            };
            execute(
                &literal_source,
                Some(result),
                &prints,
                usize::from(literal && gate),
            );
            let source = source("then", "then", "&&", true, !literal, false, gate, 2, false)
                .replace("print(99);", "print(99); let nested = nested == false;");
            execute(&source, Some(result), &prints, usize::from(literal && gate));
        }
    }
}

fn assert_seed_only(original: &[NirStmt], seeded: &[NirStmt]) {
    if original.is_empty() {
        assert_eq!(seeded, &[NirStmt::Return(Some(NirExpr::Bool(false)))]);
        return;
    }
    assert_eq!(original.len(), seeded.len());
    assert_eq!(&original[..original.len() - 1], &seeded[..seeded.len() - 1]);
    match (original.last().unwrap(), seeded.last().unwrap()) {
        (
            NirStmt::If {
                condition,
                then_body,
                else_body,
            },
            NirStmt::If {
                condition: gate,
                then_body: yes,
                else_body: no,
            },
        ) => {
            assert_eq!(condition, gate);
            assert_seed_only(then_body, yes);
            assert_seed_only(else_body, no);
        }
        (old, new) => assert_eq!(old, new),
    }
}

pub(super) fn total_readiness(value: &NirExpr) -> bool {
    match value {
        NirExpr::Bool(_) | NirExpr::Var(_) => true,
        NirExpr::Binary {
            op: NirBinaryOp::And | NirBinaryOp::Or | NirBinaryOp::Eq,
            lhs,
            rhs,
        } => total_readiness(lhs) && total_readiness(rhs),
        _ => false,
    }
}

#[test]
fn conditional_return_fallthrough_trees_seed_only_empty_leaves_keep_parent_tail_and_hygiene() {
    for shape in ["then", "else", "both"] {
        for kind in ["then", "else", "packet", "split", "hygiene"] {
            let mut module = crate::frontend::parse_nuis_module(&source(
                shape, kind, "&&", true, true, true, true, 2, false,
            ))
            .unwrap();
            let before = module.clone();
            let generated = outline_test(&mut module);
            assert_eq!(generated.len(), 1, "{shape}/{kind}");
            let original = before.functions.iter().find(|f| f.name == "event").unwrap();
            let after = module.functions.iter().find(|f| f.name == "event").unwrap();
            assert_eq!(&after.body[..2], &original.body[..2]);
            assert_eq!(
                &after.body[after.body.len() - 2..],
                &original.body[original.body.len() - 2..]
            );
            let helper = module
                .functions
                .iter()
                .find(|f| generated.contains(&f.name))
                .unwrap();
            let NirStmt::If {
                then_body,
                else_body,
                ..
            } = &original.body[2]
            else {
                panic!()
            };
            let NirStmt::If {
                then_body: yes,
                else_body: no,
                ..
            } = &helper.body[0]
            else {
                panic!()
            };
            for (old, new) in [(then_body, yes), (else_body, no)] {
                assert_seed_only(old, new);
                let mut bindings = BTreeSet::new();
                branches::collect_bindings(new, &mut bindings);
                assert!(!bindings.contains(&helper.params[0].name));
            }
            let mut captures = vec!["divisor", "gate", "nested"];
            if kind == "split" {
                captures.push("other");
            }
            assert_eq!(
                helper
                    .params
                    .iter()
                    .skip(1)
                    .map(|p| p.name.as_str())
                    .collect::<Vec<_>>(),
                captures
            );
            let NirStmt::Let { name, value, .. } = &after.body[4] else {
                panic!()
            };
            assert!(name.starts_with("__nuis_return_exit"));
            if kind == "hygiene" {
                assert_ne!(name, "__nuis_return_exit_0");
            }
            assert!(total_readiness(value));
            let mut used = BTreeSet::new();
            control_values::collect_inputs(value, &mut used);
            assert!(used
                .iter()
                .all(|name| name.starts_with("__nuis_return_gate")
                    || name == "nested"
                    || name == "other"));
            for function in &before.functions {
                if function.name != "event" {
                    assert_eq!(
                        module
                            .functions
                            .iter()
                            .find(|f| f.name == function.name)
                            .unwrap(),
                        function
                    );
                }
            }
            crate::nir_verify::verify_nir_module(&module).unwrap();
            let once = module.clone();
            assert!(outline_test(&mut module).is_empty());
            assert_eq!(module, once);
        }
    }
}

#[test]
fn conditional_return_fallthrough_trees_veto_computed_exit_decisions_effects_rebinds_and_logical_prefixes(
) {
    let original = source("then", "then", "&&", true, true, true, true, 2, false);
    let admitted = original.replace(
        &body("then", "&&"),
        "if nested { return helper(produce(divisor)); } else { let fresh = divisor + 1; }",
    );
    let compiled = crate::pipeline::compile_source(&admitted).unwrap();
    yir_lower_llvm::emit_module(&compiled.yir).unwrap();
    let admitted = original.replace(
        &body("then", "&&"),
        "let selected: bool = nested; if selected { return helper(produce(divisor)); }",
    );
    let compiled = crate::pipeline::compile_source(&admitted).unwrap();
    yir_lower_llvm::emit_module(&compiled.yir).unwrap();
    let admitted = original.replace(
        &body("then", "&&"),
        "if nested == true { return helper(produce(divisor)); }",
    );
    let compiled = crate::pipeline::compile_source(&admitted).unwrap();
    yir_lower_llvm::emit_module(&compiled.yir).unwrap();
    for replacement in [
        "let selected: bool = helper(produce(divisor)); if selected { return helper(produce(divisor)); }",
        "if nested == helper(produce(divisor)) { return helper(produce(divisor)); }",
    ] {
        let admitted = original.replace(&body("then", "&&"), replacement);
        let compiled = crate::pipeline::compile_source(&admitted).unwrap();
        yir_lower_llvm::emit_module(&compiled.yir).unwrap();
    }
    let logical = "if nested { return helper(produce(divisor)); } else { let fresh = gate || helper(produce(divisor)); }";
    for (gate, divisor) in [(true, 0), (false, 2), (false, 0)] {
        let admitted = source(
            "then", "then", "&&", true, false, true, gate, divisor, false,
        )
        .replace(&body("then", "&&"), logical);
        execute(
            &admitted,
            (gate || divisor != 0).then_some(19),
            &[99, 77, 19],
            usize::from(!gate),
        );
    }
    for replacement in [
        "if nested { return helper(produce(divisor)); } else { print(88); }",
        "let gate: bool = true; if nested { return helper(produce(divisor)); }",
        "if nested { while gate { return helper(produce(divisor)); } }",
        "if nested { return helper(produce(divisor)); } print(88);",
        "let packet = produce(divisor); if nested { } else { }",
    ] {
        let mut module =
            crate::frontend::parse_nuis_module(&original.replace(&body("then", "&&"), replacement))
                .unwrap();
        if replacement.contains("print(88)") {
            assert_selected_effect_not_pure(&original.replace(&body("then", "&&"), replacement));
            continue;
        }
        let before = module.clone();
        assert!(outline_test(&mut module).is_empty(), "{replacement}");
        assert_eq!(module, before, "{replacement}");
    }
    for mutation in [
        "borrow",
        "optional",
        "effect",
        "aggregate",
        "declared",
        "result",
        "async",
    ] {
        let mut module = crate::frontend::parse_nuis_module(&source(
            "then", "packet", "&&", true, true, true, true, 2, false,
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
                "borrow" => event.params[4].ty.is_ref = true,
                "optional" => event.params[4].ty.is_optional = true,
                "result" => event.return_type = Some(scalar_type("i64")),
                "async" => event.is_async = true,
                "aggregate" | "declared" => {
                    let NirStmt::If { then_body, .. } = &mut event.body[2] else {
                        panic!()
                    };
                    let NirStmt::Let { value, ty, .. } = &mut then_body[0] else {
                        panic!()
                    };
                    if mutation == "aggregate" {
                        *value = NirExpr::Var("original_packet".into());
                        event.params.push(NirParam {
                            name: "original_packet".into(),
                            ty: scalar_type("Packet"),
                        });
                    } else {
                        *ty = Some(scalar_type("bool"));
                    }
                }
                _ => unreachable!(),
            }
        }
        let before = module.clone();
        assert!(outline_test(&mut module).is_empty(), "{mutation}");
        assert_eq!(module, before, "{mutation}");
    }
}

#[test]
fn conditional_return_fallthrough_trees_share_preclone_statement_expression_and_depth_budgets() {
    for count in [30, 31] {
        let locals = (0..count)
            .map(|i| format!("let fresh{i} = divisor + {i};"))
            .collect::<String>();
        let mut module = crate::frontend::parse_nuis_module(
            &source("then", "then", "&&", true, true, false, true, 2, false).replace(
                &body("then", "&&"),
                &format!("{locals} if nested {{ return helper(produce(divisor)); }}"),
            ),
        )
        .unwrap();
        let before = module.clone();
        assert_eq!(outline_test(&mut module).is_empty(), count == 31);
        if count == 31 {
            assert_eq!(module, before);
        }
    }
    for depth in [31, 32] {
        let mut arm = "return gate && helper(produce(divisor));".to_owned();
        for _ in 0..depth {
            arm = format!("if nested {{ {arm} }}");
        }
        let source = source("then", "then", "&&", true, true, false, true, 2, false)
            .replace(&body("then", "&&"), &arm);
        let mut module = crate::frontend::parse_nuis_module(&source).unwrap();
        let before = module.clone();
        assert_eq!(outline_test(&mut module).is_empty(), depth == 32);
        if depth == 32 {
            assert_eq!(module, before);
        } else {
            let compiled = crate::pipeline::compile_source(&source).unwrap();
            yir_lower_llvm::emit_module(&compiled.yir).unwrap();
        }
    }
    fn tree(depth: usize) -> NirExpr {
        if depth == 0 {
            NirExpr::Int(1)
        } else {
            binary(NirBinaryOp::Add, tree(depth - 1), tree(depth - 1))
        }
    }
    let module = crate::frontend::parse_nuis_module(&source(
        "then", "then", "&&", true, true, false, true, 2, false,
    ))
    .unwrap();
    let layouts = control_values::TypedLayouts::collect(&module);
    let carries = control_values::layouts(&module);
    let control = scalar_helpers::collect_with_layouts(&module, &carries);
    let catalog = scalar_helpers::collect_typed_values(&module, &layouts, &control);
    let binding = |name: &str, value| NirStmt::Let {
        name: name.into(),
        ty: Some(scalar_type("i64")),
        value,
    };
    let branch = |value| NirStmt::If {
        condition: NirExpr::Bool(true),
        then_body: vec![NirStmt::Return(Some(value))],
        else_body: vec![],
    };
    let exact = vec![
        binding("large", tree(10)),
        binding("one", NirExpr::Int(1)),
        branch(tree(10)),
    ];
    assert!(prepare(
        &exact,
        &scalar_type("i64"),
        &Scope::new(),
        &catalog,
        &layouts
    )
    .is_some());
    let mut excess = exact.clone();
    excess.insert(0, binding("two", NirExpr::Int(1)));
    assert!(prepare(
        &excess,
        &scalar_type("i64"),
        &Scope::new(),
        &catalog,
        &layouts
    )
    .is_none());
    let mut linear = NirExpr::Int(1);
    for _ in 0..64 {
        linear = binary(NirBinaryOp::Add, NirExpr::Int(1), linear);
    }
    assert!(prepare(
        &[branch(linear)],
        &scalar_type("i64"),
        &Scope::new(),
        &catalog,
        &layouts
    )
    .is_none());
}

#[test]
fn conditional_return_fallthrough_readiness_is_total_boolean_selection_not_value_selection() {
    fn eval(value: &NirExpr, gate: bool, yes: bool, no: bool) -> bool {
        match value {
            NirExpr::Bool(value) => *value,
            NirExpr::Var(name) => match name.as_str() {
                "gate" => gate,
                "yes" => yes,
                "no" => no,
                _ => panic!(),
            },
            NirExpr::Binary { op, lhs, rhs } => {
                let (lhs, rhs) = (eval(lhs, gate, yes, no), eval(rhs, gate, yes, no));
                match op {
                    NirBinaryOp::And => lhs && rhs,
                    NirBinaryOp::Or => lhs || rhs,
                    NirBinaryOp::Eq => lhs == rhs,
                    _ => panic!(),
                }
            }
            _ => panic!("readiness must not contain computations or calls"),
        }
    }
    for yes in [
        NirExpr::Bool(false),
        NirExpr::Bool(true),
        NirExpr::Var("yes".into()),
    ] {
        for no in [
            NirExpr::Bool(false),
            NirExpr::Bool(true),
            NirExpr::Var("no".into()),
        ] {
            let selected = select(NirExpr::Var("gate".into()), yes.clone(), no.clone());
            for gate in [false, true] {
                for yes_value in [false, true] {
                    for no_value in [false, true] {
                        assert_eq!(
                            eval(&selected, gate, yes_value, no_value),
                            eval(if gate { &yes } else { &no }, gate, yes_value, no_value)
                        );
                    }
                }
            }
        }
    }
}
