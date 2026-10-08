use super::*;
use crate::frontend::parse_nuis_module;
use tests::{predicate_args, source};

#[test]
fn computed_logical_effectful_scalar_selections_keep_left_checks_at_argument_zero_and_left_only_effects(
) {
    for body in [
        "if gate { if decide(saved / 2) && first { let saved = saved; } if second { let saved = saved; } }",
        "if gate { if decide(saved / 2) == first || second { let saved = saved; } if first { let saved = saved; } }",
    ] {
        let mut module = parse_nuis_module(&source("i64", body)).unwrap();
        let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
        let generated = outline(&mut module, &mut names);
        assert_eq!(generated.len(), 7, "{body}");
        crate::nir_verify::verify_nir_module(&module).unwrap();
        let carrier = module.functions.iter().find(|f| f.body.iter().any(|s| predicate_args(s).is_some())).unwrap();
        assert!(matches!(carrier.body.first(), Some(NirStmt::If { .. })));
        let sites = carrier.body.iter().filter_map(predicate_args).collect::<Vec<_>>();
        assert_eq!(sites.len(), 1);
        assert!(sites[0].iter().skip(1).all(|arg| matches!(arg, NirExpr::Var(_))));
        let left = match &sites[0][0] { NirExpr::Binary { lhs, .. } => lhs.as_ref(), value => value };
        assert!(matches!(left, NirExpr::Call { callee, args } if callee == "decide"
            && matches!(args.as_slice(), [NirExpr::Binary { op: NirBinaryOp::Div, .. }])));
        let predicate = module.functions.iter().find(|f| generated.contains(&f.name)
            && f.name.starts_with("__nuis_effect_call_predicate_")).unwrap();
        assert!(!predicate.body.iter().any(|s| matches!(s, NirStmt::Return(Some(NirExpr::Call { callee, .. })) if callee == "decide")));
    }
    let scope = [
        ("saved".into(), scalar_type("i64")),
        ("first".into(), scalar_type("bool")),
    ]
    .into();
    let signatures = [(
        "decide".into(),
        (vec![scalar_type("i64")], scalar_type("bool")),
    )]
    .into();
    let condition = NirExpr::Binary {
        op: NirBinaryOp::And,
        lhs: Box::new(NirExpr::Call {
            callee: "decide".into(),
            args: vec![NirExpr::Var("saved".into())],
        }),
        rhs: Box::new(NirExpr::Var("first".into())),
    };
    let (_, inputs, calls) = prove(
        &condition,
        &scope,
        &signatures,
        &mut nested::Budget {
            nodes: 64,
            expressions: 256,
            logical_edges: 32,
        },
    )
    .unwrap();
    assert_eq!(inputs, ["first".into(), "saved".into()].into());
    assert_eq!(calls, ["decide".into()].into());
}

#[test]
fn computed_logical_effectful_scalar_selections_preserve_comparison_call_types_polarity_and_hygiene(
) {
    for kind in ["bool", "i32", "i64", "f32", "f64"] {
        for op in ["&&", "||"] {
            let comparison = if matches!(kind, "f32" | "f64") {
                "first == second"
            } else {
                "saved == saved"
            };
            for left in ["decide(saved)", "decide(saved) == first", comparison] {
                for inverted in [false, true] {
                    let update = "let saved = work(saved);";
                    let child = if inverted {
                        format!("if {left} {op} decide(saved) {{}} else {{ {update} }}")
                    } else {
                        format!("if {left} {op} decide(saved) {{ {update} }}")
                    };
                    let mut module = parse_nuis_module(&source(
                        kind,
                        &format!("if gate {{ {child} if second {{ {update} }} }}"),
                    ))
                    .unwrap_or_else(|e| panic!("{kind}/{op}/{left}/{inverted}: {e}"));
                    let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
                    let generated = outline(&mut module, &mut names);
                    assert_eq!(generated.len(), 7, "{kind}/{op}/{left}/{inverted}");
                    assert!(!generated.contains("__nuis_effect_call_predicate_0"));
                    crate::nir_verify::verify_nir_module(&module).unwrap();
                    for function in module
                        .functions
                        .iter()
                        .filter(|f| generated.contains(&f.name))
                    {
                        assert_eq!(
                            function.return_type,
                            Some(scalar_type(
                                if function.name.starts_with("__nuis_effect_call_predicate_") {
                                    "bool"
                                } else {
                                    kind
                                }
                            ))
                        );
                        assert_eq!(
                            function.params.len(),
                            function
                                .params
                                .iter()
                                .map(|p| &p.name)
                                .collect::<BTreeSet<_>>()
                                .len()
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn computed_logical_effectful_scalar_selections_reject_nested_leaves_bad_left_signatures_and_private_reads_atomically(
) {
    for condition in [
        "(first || second) == true && decide(saved)",
        "decide(saved) && (first || second) == true",
        "decide(saved) == (first || second) && first",
    ] {
        let mut module = parse_nuis_module(&source("i64", &format!("if gate {{
            if {condition} {{ let saved = work(saved); }} if second {{ let saved = work(saved); }} }}"))).unwrap();
        let before = module.clone();
        let mut names: BTreeSet<String> = module.functions.iter().map(|f| f.name.clone()).collect();
        let before_names = names.clone();
        assert!(outline(&mut module, &mut names).is_empty());
        assert_eq!(module, before);
        assert_eq!(names, before_names);
    }
    for mutation in [
        "async", "ref", "optional", "generic", "resource", "result", "pure", "escape", "argument",
    ] {
        let mut module = parse_nuis_module(&source(
            "i64",
            "if gate {
            if first { let saved = saved; } if decide(saved) && second { let saved = saved; } }",
        ))
        .unwrap();
        let decide = module
            .functions
            .iter_mut()
            .find(|f| f.name == "decide")
            .unwrap();
        match mutation {
            "async" => decide.is_async = true,
            "ref" => decide.params[0].ty.is_ref = true,
            "optional" => decide.params[0].ty.is_optional = true,
            "generic" => decide.params[0].ty.generic_args.push(scalar_type("i64")),
            "resource" => decide.params[0].ty.name = "Buffer".into(),
            "result" => decide.return_type = Some(scalar_type("i64")),
            "pure" => {
                decide.body.remove(0);
            }
            "escape" | "argument" => {
                let observer = module
                    .functions
                    .iter_mut()
                    .find(|f| f.name == "observe")
                    .unwrap();
                let NirStmt::If { then_body, .. } = &mut observer.body[0] else {
                    unreachable!()
                };
                if mutation == "escape" {
                    let NirStmt::If {
                        then_body: child, ..
                    } = &mut then_body[0]
                    else {
                        unreachable!()
                    };
                    child.insert(
                        0,
                        NirStmt::Let {
                            name: "private".into(),
                            ty: None,
                            value: NirExpr::Var("saved".into()),
                        },
                    );
                }
                let NirStmt::If { condition, .. } = &mut then_body[1] else {
                    unreachable!()
                };
                let NirExpr::Binary { lhs, .. } = condition else {
                    unreachable!()
                };
                let NirExpr::Call { args, .. } = lhs.as_mut() else {
                    unreachable!()
                };
                args[0] = if mutation == "escape" {
                    NirExpr::Var("private".into())
                } else {
                    NirExpr::Binary {
                        op: NirBinaryOp::Or,
                        lhs: Box::new(NirExpr::Var("first".into())),
                        rhs: Box::new(NirExpr::Var("second".into())),
                    }
                };
            }
            _ => unreachable!(),
        }
        let before = module.clone();
        let mut names: BTreeSet<String> = module.functions.iter().map(|f| f.name.clone()).collect();
        let before_names = names.clone();
        assert!(outline(&mut module, &mut names).is_empty(), "{mutation}");
        assert_eq!(module, before);
        assert_eq!(names, before_names);
    }
}

#[test]
fn computed_logical_effectful_scalar_selections_share_both_operands_expression_depth_and_ancestor_captures(
) {
    let scope = [("saved".into(), scalar_type("i64"))].into();
    let signatures = [(
        "decide".into(),
        (vec![scalar_type("i64")], scalar_type("bool")),
    )]
    .into();
    let call = || NirExpr::Call {
        callee: "decide".into(),
        args: vec![NirExpr::Var("saved".into())],
    };
    let condition = NirExpr::Binary {
        op: NirBinaryOp::And,
        lhs: Box::new(call()),
        rhs: Box::new(call()),
    };
    for expressions in [4, 5] {
        let mut budget = nested::Budget {
            nodes: 1,
            expressions,
            logical_edges: 32,
        };
        assert_eq!(
            prove(&condition, &scope, &signatures, &mut budget).is_some(),
            expressions == 5
        );
        assert_eq!(budget.nodes, 1);
        assert_eq!(budget.expressions, 0);
    }
    for left in [false, true] {
        for additions in [29, 30] {
            let mut value = NirExpr::Int(0);
            for _ in 0..additions {
                value = NirExpr::Binary {
                    op: NirBinaryOp::Add,
                    lhs: Box::new(value),
                    rhs: Box::new(NirExpr::Int(1)),
                };
            }
            let comparison = NirExpr::Binary {
                op: NirBinaryOp::Eq,
                lhs: Box::new(value),
                rhs: Box::new(NirExpr::Int(0)),
            };
            let (lhs, rhs) = if left {
                (comparison, NirExpr::Bool(true))
            } else {
                (NirExpr::Bool(true), comparison)
            };
            assert_eq!(
                prove(
                    &NirExpr::Binary {
                        op: NirBinaryOp::Or,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs)
                    },
                    &Scope::new(),
                    &Signatures::new(),
                    &mut nested::Budget {
                        nodes: 64,
                        expressions: 256,
                        logical_edges: 32,
                    }
                )
                .is_some(),
                additions == 29
            );
        }
    }
    for count in [28, 29] {
        let extra = (0..count)
            .map(|i| format!(", p{i}: i64"))
            .collect::<String>();
        let args = (0..count)
            .map(|i| format!("p{i}"))
            .collect::<Vec<_>>()
            .join(" + ");
        let text = source("i64", &format!("if gate {{ if decide(saved + {args}) && first {{ let saved = saved; }} if second {{ let saved = saved; }} }}"))
            .replace("__nuis_effect_predicate_gate_0: bool)", &format!("__nuis_effect_predicate_gate_0: bool{extra})"));
        let mut module = parse_nuis_module(&text).unwrap();
        let before = module.clone();
        let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
        let generated = outline(&mut module, &mut names);
        assert_eq!(!generated.is_empty(), count == 28);
        if count == 29 {
            assert_eq!(module, before);
        }
    }
}
