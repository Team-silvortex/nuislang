use super::*;
use crate::frontend::parse_nuis_module;

fn source(kind: &str, body: &str) -> String {
    format!(
        "mod cpu Main {{
        fn __nuis_effect_call_predicate_0() -> bool {{ return true; }}
        fn work(v: {kind}) -> {kind} {{ print(70); return v; }}
        fn decide(v: {kind}) -> bool {{ print(98); return v == v; }}
        fn boolean(v: bool) -> bool {{ return v; }}
        fn observe(gate: bool, first: bool, second: bool, saved: {kind}) -> {kind} {{
            {body} return saved;
        }} fn main() -> i64 {{ return 0; }} }}"
    )
}

fn expressions(body: &[NirStmt], visit: &mut impl FnMut(&NirExpr)) {
    fn walk(expr: &NirExpr, visit: &mut impl FnMut(&NirExpr)) {
        visit(expr);
        crate::nir_walk::walk_child_exprs(expr, &mut |child| walk(child, visit));
    }
    for stmt in body {
        match stmt {
            NirStmt::Let { value, .. }
            | NirStmt::Const { value, .. }
            | NirStmt::Return(Some(value))
            | NirStmt::Expr(value) => walk(value, visit),
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                walk(condition, visit);
                expressions(then_body, visit);
                expressions(else_body, visit);
            }
            _ => {}
        }
    }
}

#[test]
fn staging_logical_effectful_scalar_selections_keep_prefix_middle_suffix_types_private_bindings_and_hygiene(
) {
    for kind in ["bool", "i32", "i64", "f32", "f64"] {
        for constant in [false, true] {
            let bind = if constant { "const" } else { "let" };
            for ordered in [false, true] {
                for inverted in [false, true] {
                    let body = if ordered {
                        format!(
                            "{bind} local_first: bool = first && decide(saved);
                            if local_first {{ let saved = work(saved); }}
                            {bind} local_second: bool = second || decide(saved);
                            if local_second {{ let saved = work(saved); }}
                            {bind} local_final: bool = local_first && local_second;
                            let saved = work(saved);"
                        )
                    } else {
                        format!(
                            "{bind} local_first: bool = first && decide(saved);
                            let saved = work(saved);"
                        )
                    };
                    let body = if inverted {
                        format!("if gate {{}} else {{ {body} }}")
                    } else {
                        format!("if gate {{ {body} }}")
                    };
                    let mut module = parse_nuis_module(&source(kind, &body)).unwrap();
                    let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
                    let generated = outline(&mut module, &mut names);
                    assert_eq!(
                        generated.len(),
                        if ordered { 9 } else { 3 },
                        "{kind}/{constant}/{ordered}/{inverted}"
                    );
                    assert!(!generated.contains("__nuis_effect_call_predicate_0"));
                    crate::nir_verify::verify_nir_module(&module).unwrap();
                    let mut calls = 0;
                    let mut sites = 0;
                    for function in module
                        .functions
                        .iter()
                        .filter(|f| generated.contains(&f.name))
                    {
                        let predicate = function.name.starts_with("__nuis_effect_call_predicate_");
                        assert_eq!(
                            function.return_type,
                            Some(scalar_type(if predicate { "bool" } else { kind }))
                        );
                        assert!(matches!(function.body.first(), Some(NirStmt::If { .. })));
                        assert_eq!(
                            function.params.len(),
                            function
                                .params
                                .iter()
                                .map(|p| &p.name)
                                .collect::<BTreeSet<_>>()
                                .len()
                        );
                        for (index, statement) in function.body.iter().enumerate() {
                            let (name, is_const) = match statement {
                                NirStmt::Let { name, .. } => (name, false),
                                NirStmt::Const { name, .. } => (name, true),
                                _ => continue,
                            };
                            if name.starts_with("local_") {
                                sites += 1;
                                assert!(index > 0);
                                assert_eq!(is_const, constant);
                            }
                        }
                        expressions(&function.body, &mut |expr| {
                            assert!(!matches!(
                                expr,
                                NirExpr::Binary {
                                    op: NirBinaryOp::And | NirBinaryOp::Or,
                                    ..
                                }
                            ));
                            if let NirExpr::Call { callee, .. } = expr {
                                calls += usize::from(callee == "decide");
                            }
                        });
                    }
                    assert_eq!(calls, if ordered { 2 } else { 1 });
                    assert_eq!(sites, if ordered { 3 } else { 1 });
                    let observer = module
                        .functions
                        .iter()
                        .find(|f| f.name == "observe")
                        .unwrap();
                    assert!(!observer.body.iter().any(|s| matches!(s, NirStmt::Let { name, .. } | NirStmt::Const { name, .. } if name.starts_with("local_"))));
                }
            }
        }
    }
}

#[test]
fn staging_logical_effectful_scalar_selections_discover_initializer_only_effects_and_keep_left_checks_once(
) {
    for bind in ["let", "const"] {
        let mut module = parse_nuis_module(&source("i64", &format!(
            "if gate {{ {bind} local_first: bool = decide(saved / 2) && first; let saved = saved; }}"
        ))).unwrap();
        let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
        let generated = outline(&mut module, &mut names);
        assert_eq!(generated.len(), 3);
        crate::nir_verify::verify_nir_module(&module).unwrap();
        let mut calls = 0;
        for function in module
            .functions
            .iter()
            .filter(|f| generated.contains(&f.name))
        {
            expressions(&function.body, &mut |expr| {
                if let NirExpr::Call { callee, args } = expr {
                    if callee == "decide" {
                        calls += 1;
                        assert!(matches!(
                            args.as_slice(),
                            [NirExpr::Binary {
                                op: NirBinaryOp::Div,
                                ..
                            }]
                        ));
                    }
                }
            });
        }
        assert_eq!(calls, 1);
    }
}

#[test]
fn staging_logical_effectful_scalar_selections_reject_hidden_roots_existing_writes_constants_and_late_errors_atomically(
) {
    for stage in [
        "let local = (first && decide(saved)) == true; let saved = work(saved);",
        "let local = boolean(first && decide(saved)); let saved = work(saved);",
        "let first = first && decide(saved); let saved = work(saved);",
        "const local: bool = first; let saved = work(saved);",
        "const local: bool = first && decide(saved); let local = second; let saved = work(saved);",
        "const local: bool = first && decide(saved); if local { let saved = work(saved); } let local = second; let saved = work(saved);",
        "let local = first && decide(saved); if local { let local = second; } let saved = work(saved);",
        "let local = first && decide(saved); let saved = work(saved); return saved;",
    ] {
        let mut module = parse_nuis_module(&source("i64", &format!("if gate {{ {stage} }}"))).unwrap();
        let before = module.clone();
        let mut names: BTreeSet<_> = module.functions.iter().map(|f| f.name.clone()).collect();
        let before_names = names.clone();
        assert!(outline(&mut module, &mut names).is_empty(), "{stage}");
        assert_eq!(module, before);
        assert_eq!(names, before_names);
    }
    for mutation in ["type", "missing", "argument"] {
        let mut module = parse_nuis_module(&source(
            "i64",
            "if gate {
            let local = first && decide(saved); if local { let saved = work(saved); }
            let late = second || decide(saved); let saved = work(saved); }",
        ))
        .unwrap();
        let observer = module
            .functions
            .iter_mut()
            .find(|f| f.name == "observe")
            .unwrap();
        let NirStmt::If { then_body, .. } = &mut observer.body[0] else {
            unreachable!()
        };
        let NirStmt::Let {
            ty,
            value: NirExpr::Binary { lhs, rhs, .. },
            ..
        } = &mut then_body[2]
        else {
            unreachable!()
        };
        match mutation {
            "type" => *ty = Some(scalar_type("i64")),
            "missing" => *lhs = Box::new(NirExpr::Var("absent".into())),
            "argument" => {
                let NirExpr::Call { args, .. } = rhs.as_mut() else {
                    unreachable!()
                };
                args[0] = NirExpr::Bool(true);
            }
            _ => unreachable!(),
        }
        let before = module.clone();
        let mut names: BTreeSet<_> = module.functions.iter().map(|f| f.name.clone()).collect();
        let before_names = names.clone();
        assert!(outline(&mut module, &mut names).is_empty(), "{mutation}");
        assert_eq!(module, before);
        assert_eq!(names, before_names);
    }
}

#[test]
fn staging_logical_effectful_scalar_selections_share_original_stage_child_edge_and_expression_budgets(
) {
    let signatures = BTreeMap::from([(
        "decide".into(),
        (vec![scalar_type("i64")], scalar_type("bool")),
    )]);
    let scope = BTreeMap::from([
        ("gate".into(), scalar_type("bool")),
        ("first".into(), scalar_type("bool")),
        ("saved".into(), scalar_type("i64")),
    ]);
    for expression_budget in [2, 3] {
        let statements = vec![NirStmt::Let {
            name: "local".into(),
            ty: None,
            value: NirExpr::Binary {
                op: NirBinaryOp::And,
                lhs: Box::new(NirExpr::Var("first".into())),
                rhs: Box::new(NirExpr::Bool(true)),
            },
        }];
        let mut budget = nested::Budget {
            nodes: 64,
            expressions: expression_budget,
            logical_edges: 32,
        };
        assert_eq!(
            stage(
                &statements,
                &scope,
                &BTreeSet::new(),
                &signatures,
                &mut budget
            )
            .is_some(),
            expression_budget == 3
        );
        assert_eq!(budget.nodes, 63);
        assert_eq!(budget.expressions, 0);
    }
    for count in [30, 31] {
        let stages = (0..count)
            .map(|i| {
                format!(
                    "let local{i} = first && decide(saved); if local{i} {{ let saved = saved; }}"
                )
            })
            .collect::<String>();
        // Charge the original ancestor and child nodes separately from logical edges.
        let module =
            parse_nuis_module(&source("i64", &format!("if gate && true {{ {stages} }}"))).unwrap();
        let observer = module
            .functions
            .iter()
            .find(|f| f.name == "observe")
            .unwrap();
        let NirStmt::If {
            condition,
            then_body,
            else_body,
        } = &observer.body[0]
        else {
            unreachable!()
        };
        let mut budget = nested::Budget {
            nodes: 200,
            expressions: 1000,
            logical_edges: 31,
        };
        assert_eq!(
            nested::branch(
                condition,
                then_body,
                else_body,
                &scope,
                &signatures,
                &mut budget,
                0
            )
            .is_some(),
            count == 30
        );
        assert_eq!(budget.logical_edges, 0);
    }
}

#[test]
fn staging_logical_effectful_scalar_selections_bound_complete_initializer_rhs_capture_union() {
    fn balanced(inputs: &[String]) -> NirExpr {
        if inputs.len() == 1 {
            return NirExpr::Var(inputs[0].clone());
        }
        let middle = inputs.len() / 2;
        NirExpr::Binary {
            op: NirBinaryOp::Or,
            lhs: Box::new(balanced(&inputs[..middle])),
            rhs: Box::new(balanced(&inputs[middle..])),
        }
    }
    for count in [31, 32] {
        let inputs = (0..count).map(|i| format!("input{i}")).collect::<Vec<_>>();
        let scope = inputs
            .iter()
            .map(|name| (name.clone(), scalar_type("bool")))
            .collect();
        let statements = vec![NirStmt::Const {
            name: "local".into(),
            ty: scalar_type("bool"),
            value: NirExpr::Binary {
                op: NirBinaryOp::Or,
                lhs: Box::new(NirExpr::Bool(false)),
                rhs: Box::new(balanced(&inputs)),
            },
        }];
        let mut budget = nested::Budget {
            nodes: 64,
            expressions: 256,
            logical_edges: 32,
        };
        let result = stage(
            &statements,
            &scope,
            &BTreeSet::new(),
            &Signatures::new(),
            &mut budget,
        );
        assert_eq!(result.is_some(), count == 31);
        if let Some(staged) = result {
            assert_eq!(staged.inputs.len(), 31);
            assert!(staged.constants.contains("local"));
            assert_eq!(staged.statements.len(), 1);
        }
    }
}
