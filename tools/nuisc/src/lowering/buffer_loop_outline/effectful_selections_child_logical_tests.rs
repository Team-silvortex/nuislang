use super::*;
use crate::frontend::parse_nuis_module;

fn source(body: &str) -> String {
    format!(
        "mod cpu Main {{
        fn __nuis_effect_call_predicate_0() -> bool {{ return true; }}
        fn decide(v: bool) -> bool {{ print(98); return v; }}
        fn boolean(v: bool) -> bool {{ return v; }}
        fn observe(gate: bool, first: bool, second: bool, saved: bool) -> bool {{
            {body} return saved;
        }} fn main() -> i64 {{ return 0; }} }}"
    )
}

fn rewrite(body: &str) -> (NirModule, BTreeSet<String>) {
    let mut module = parse_nuis_module(&source(body)).unwrap();
    let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
    let generated = outline(&mut module, &mut names);
    (module, generated)
}

fn reject_module(mut module: NirModule) {
    let before = module.clone();
    let mut names: BTreeSet<_> = module.functions.iter().map(|f| f.name.clone()).collect();
    let before_names = names.clone();
    assert!(outline(&mut module, &mut names).is_empty());
    assert_eq!(module, before);
    assert_eq!(names, before_names);
}

#[test]
fn child_logical_effectful_scalar_selections_keep_original_declarations_polarities_and_preceding_captures(
) {
    for root in [
        "saved && decide(saved)",
        "saved || decide(saved)",
        "decide(saved) && (saved || decide(saved))",
    ] {
        for outer_else in [false, true] {
            for inner_else in [false, true] {
                let update = format!("let saved: bool = {root};");
                let child = if inner_else {
                    format!("if first {{}} else {{ {update} }}")
                } else {
                    format!("if first {{ {update} }}")
                };
                let body = if outer_else {
                    format!("if gate {{}} else {{ {child} }}")
                } else {
                    format!("if gate {{ {child} }}")
                };
                let (module, generated) = rewrite(&body);
                assert_eq!(
                    generated.len(),
                    if root.starts_with("decide") { 6 } else { 5 }
                );
                assert!(!generated.contains("__nuis_effect_call_predicate_0"));
                crate::nir_verify::verify_nir_module(&module).unwrap();
                let mut roots = 0;
                for function in module
                    .functions
                    .iter()
                    .filter(|f| generated.contains(&f.name))
                {
                    assert_eq!(function.return_type, Some(scalar_type("bool")));
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
                        if let NirStmt::Let { name, ty, value } = statement {
                            if name == "saved" {
                                roots += 1;
                                assert!(index > 0);
                                assert_eq!(ty, &Some(scalar_type("bool")));
                                assert!(function.params.iter().any(|p| p.name == "saved"));
                                let call = match value {
                                    NirExpr::Binary {
                                        op: NirBinaryOp::Eq,
                                        lhs,
                                        rhs,
                                    } => {
                                        assert_eq!(rhs.as_ref(), &NirExpr::Bool(false));
                                        lhs.as_ref()
                                    }
                                    _ => value,
                                };
                                let NirExpr::Call { callee, args } = call else {
                                    panic!("missing predicate plan")
                                };
                                assert!(callee.starts_with("__nuis_effect_call_predicate_"));
                                assert!(args.iter().any(
                                    |arg| matches!(arg, NirExpr::Var(name) if name == "saved")
                                ));
                            }
                        }
                    }
                }
                assert_eq!(roots, 1);
            }
        }
    }
    let (module, generated) = rewrite(
        "if gate { let wanted = saved;
        if first { let saved = saved && decide(wanted); }
        else { let saved = saved || decide(wanted); }
        let wanted = saved;
        if second {} else { let saved = decide(wanted) && (saved || decide(saved)); } }",
    );
    assert_eq!(generated.len(), 10);
    crate::nir_verify::verify_nir_module(&module).unwrap();
}

#[test]
fn child_logical_effectful_scalar_selections_reject_constants_hidden_leaves_exports_other_targets_and_late_errors_atomically(
) {
    for body in [
        "if gate { let fresh = saved && decide(saved); }",
        "if gate { if first { let fresh = saved && decide(saved); } else { let fresh = saved || decide(saved); } }",
        "if gate { if first { const saved: bool = saved && decide(saved); } }",
        "const saved: bool = first; if gate { if first { let saved = saved && decide(saved); } }",
        "if gate { const local: bool = first && decide(saved); if second { let local = local || decide(saved); } }",
        "if gate { if first { return saved && decide(saved); } }",
        "if gate { if first { let saved = (saved && decide(saved)) == true; } }",
        "if gate { if first { let saved = boolean(saved && decide(saved)); } }",
        "if gate { if first { let first = first && decide(saved); } if second { let saved = saved || decide(saved); } }",
        "if gate { if first { let saved = saved && boolean(saved); } }",
        "while gate { if first { let saved = saved && decide(saved); } }",
    ] { reject_module(parse_nuis_module(&source(body)).unwrap()); }
    let original = parse_nuis_module(&source(
        "if gate {
        if first { let saved = saved && decide(saved); }
        if second { let saved = saved || decide(saved); } }",
    ))
    .unwrap();
    for mutation in [
        "declared",
        "reference",
        "optional",
        "generic",
        "i64",
        "missing",
        "argument",
        "async",
    ] {
        let mut module = original.clone();
        if mutation == "async" {
            module
                .functions
                .iter_mut()
                .find(|f| f.name == "decide")
                .unwrap()
                .is_async = true;
        } else {
            let observer = module
                .functions
                .iter_mut()
                .find(|f| f.name == "observe")
                .unwrap();
            match mutation {
                "reference" => observer.params[3].ty.is_ref = true,
                "optional" => observer.params[3].ty.is_optional = true,
                "generic" => observer.params[3].ty.generic_args.push(scalar_type("bool")),
                "i64" => observer.params[3].ty = scalar_type("i64"),
                _ => {
                    let NirStmt::If { then_body, .. } = &mut observer.body[0] else {
                        unreachable!()
                    };
                    let NirStmt::If { then_body, .. } = &mut then_body[1] else {
                        unreachable!()
                    };
                    let NirStmt::Let {
                        ty,
                        value: NirExpr::Binary { rhs, .. },
                        ..
                    } = &mut then_body[0]
                    else {
                        unreachable!()
                    };
                    match mutation {
                        "declared" => *ty = Some(scalar_type("i64")),
                        "missing" => *rhs = Box::new(NirExpr::Var("absent".into())),
                        "argument" => {
                            let NirExpr::Call { args, .. } = rhs.as_mut() else {
                                unreachable!()
                            };
                            args[0] = NirExpr::Int(1);
                        }
                        _ => unreachable!(),
                    }
                }
            }
        }
        reject_module(module);
    }
}

#[test]
fn child_logical_effectful_scalar_selections_charge_original_leaf_once_with_shared_condition_work()
{
    let scope = BTreeMap::from([
        ("gate".into(), scalar_type("bool")),
        ("first".into(), scalar_type("bool")),
        ("saved".into(), scalar_type("bool")),
    ]);
    let statement = NirStmt::Let {
        name: "saved".into(),
        ty: None,
        value: NirExpr::Binary {
            op: NirBinaryOp::And,
            lhs: Box::new(NirExpr::Var("saved".into())),
            rhs: Box::new(NirExpr::Var("first".into())),
        },
    };
    for logical_condition in [false, true] {
        let condition = if logical_condition {
            NirExpr::Binary {
                op: NirBinaryOp::Or,
                lhs: Box::new(NirExpr::Var("gate".into())),
                rhs: Box::new(NirExpr::Bool(false)),
            }
        } else {
            NirExpr::Var("gate".into())
        };
        let expressions = if logical_condition { 7 } else { 5 };
        let edges = if logical_condition { 2 } else { 1 };
        for (nodes, expressions, edges, accepted) in [
            (2, expressions, edges, false),
            (3, expressions - 1, edges, false),
            (3, expressions, edges - 1, false),
            (3, expressions, edges, true),
        ] {
            let mut budget = Budget {
                nodes,
                expressions,
                logical_edges: edges,
            };
            let proof = branch(
                &condition,
                &[statement.clone()],
                &[],
                &scope,
                &Signatures::new(),
                &mut budget,
                0,
            );
            assert_eq!(proof.is_some(), accepted);
            if let Some(proof) = proof {
                assert_eq!(
                    proof.inputs,
                    BTreeSet::from(["gate".into(), "first".into(), "saved".into()])
                );
                assert_eq!(
                    (budget.nodes, budget.expressions, budget.logical_edges),
                    (0, 0, 0)
                );
            }
        }
    }
}

#[test]
fn child_logical_effectful_scalar_selections_keep_capture_union_selection_depth_expression_depth_and_sibling_bounds(
) {
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
    for count in [30, 31, 32] {
        let inputs = (0..count).map(|i| format!("input{i}")).collect::<Vec<_>>();
        let mut scope = inputs
            .iter()
            .map(|name| (name.clone(), scalar_type("bool")))
            .collect::<Scope>();
        scope.extend([
            ("gate".into(), scalar_type("bool")),
            ("saved".into(), scalar_type("bool")),
        ]);
        let statement = NirStmt::Let {
            name: "saved".into(),
            ty: None,
            value: NirExpr::Binary {
                op: NirBinaryOp::And,
                lhs: Box::new(NirExpr::Var("saved".into())),
                rhs: Box::new(balanced(&inputs)),
            },
        };
        let mut budget = Budget {
            nodes: 64,
            expressions: 256,
            logical_edges: 32,
        };
        assert_eq!(
            branch(
                &NirExpr::Var("gate".into()),
                &[statement],
                &[],
                &scope,
                &Signatures::new(),
                &mut budget,
                0
            )
            .is_some(),
            count == 30
        );
    }
    let scope = BTreeMap::from([
        ("gate".into(), scalar_type("bool")),
        ("saved".into(), scalar_type("bool")),
    ]);
    for depth in [31, 32] {
        let mut value = NirExpr::Var("saved".into());
        for _ in 0..depth {
            value = NirExpr::Binary {
                op: NirBinaryOp::And,
                lhs: Box::new(NirExpr::Bool(true)),
                rhs: Box::new(value),
            };
        }
        let statement = NirStmt::Let {
            name: "saved".into(),
            ty: None,
            value,
        };
        let mut budget = Budget {
            nodes: 64,
            expressions: 256,
            logical_edges: 32,
        };
        assert_eq!(
            branch(
                &NirExpr::Var("gate".into()),
                &[statement],
                &[],
                &scope,
                &Signatures::new(),
                &mut budget,
                0
            )
            .is_some(),
            depth == 31
        );
    }
    for depth in [8, 9] {
        let mut body = "let saved = saved && decide(saved);".to_owned();
        for _ in 0..depth {
            body = format!("if gate {{ {body} }}");
        }
        let (module, generated) = rewrite(&body);
        assert_eq!(!generated.is_empty(), depth == 8);
        if depth == 8 {
            crate::nir_verify::verify_nir_module(&module).unwrap();
        }
    }
    for count in [20, 21] {
        let children = "if first { let saved = saved && decide(saved); }".repeat(count);
        let (module, generated) = rewrite(&format!("if gate {{ {children} }}"));
        assert_eq!(!generated.is_empty(), count == 20);
        if count == 20 {
            crate::nir_verify::verify_nir_module(&module).unwrap();
        }
    }
}
