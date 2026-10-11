use super::*;
use crate::frontend::parse_nuis_module;

fn source(body: &str) -> String {
    format!(
        "mod cpu Main {{
        fn __nuis_effect_call_predicate_0() -> bool {{ return true; }}
        fn decide(v: bool) -> bool {{ print(98); return v; }}
        fn boolean(v: bool) -> bool {{ return v; }}
        fn observe(gate: bool, first: bool, saved: bool) -> bool {{
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
fn top_logical_effectful_scalar_selections_keep_declarations_polarities_retention_and_condition_only_effects(
) {
    for root in [
        "saved && decide(first)",
        "saved || decide(first)",
        "decide(saved) && (saved || decide(first))",
    ] {
        for opposite in [
            "",
            "let saved = first;",
            "let saved = saved || decide(first);",
        ] {
            for inverted in [false, true] {
                let update = format!("let saved: bool = {root};");
                let body = if inverted {
                    format!("if gate {{ {opposite} }} else {{ {update} }}")
                } else {
                    format!("if gate {{ {update} }} else {{ {opposite} }}")
                };
                let (module, generated) = rewrite(&body);
                let edges = if root.starts_with("decide") { 2 } else { 1 }
                    + usize::from(opposite.contains("||"));
                assert_eq!(generated.len(), 2 + edges);
                assert!(!generated.contains("__nuis_effect_call_predicate_0"));
                crate::nir_verify::verify_nir_module(&module).unwrap();
                let mut updates = 0;
                let mut retained = 0;
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
                        if let NirStmt::Let { name, ty, .. } = statement {
                            if name == "saved" {
                                updates += 1;
                                assert!(index > 0);
                                assert_eq!(ty, &Some(scalar_type("bool")));
                                assert!(function.params.iter().any(|p| p.name == "saved"));
                            }
                        }
                        retained += usize::from(matches!(statement,
                            NirStmt::Return(Some(NirExpr::Var(name))) if name == "saved"));
                    }
                }
                assert_eq!(updates, 1 + usize::from(opposite.contains("||")));
                assert!(retained >= 1);
            }
        }
    }
    for body in [
        "if decide(gate) { let saved = saved && first; }",
        "if decide(gate) {} else { let saved = saved || first; }",
        "if gate || decide(first) { let saved = saved && first; }",
    ] {
        let (module, generated) = rewrite(body);
        assert!(!generated.is_empty());
        crate::nir_verify::verify_nir_module(&module).unwrap();
    }
}

#[test]
fn top_logical_effectful_scalar_selections_reject_constants_exports_hidden_roots_other_targets_and_late_errors_atomically(
) {
    for body in [
        "if gate { let fresh = saved && decide(first); } else { let fresh = saved || decide(first); }",
        "if gate { const saved: bool = saved && decide(first); }",
        "const saved: bool = first; if gate { let saved = saved && decide(first); }",
        "if gate { return saved && decide(first); } else { let saved = saved || decide(first); }",
        "if gate { let saved = (saved && decide(first)) == true; }",
        "if gate { let saved = boolean(saved && decide(first)); }",
        "if gate { let saved = saved && decide(first); } else { let first = first || decide(saved); }",
        "if gate { let saved = saved && decide(first); } else { let saved = boolean(saved || decide(first)); }",
        "if gate { let saved = saved && boolean(first); }",
        "while gate { if first { let saved = saved && decide(first); } }",
    ] { reject_module(parse_nuis_module(&source(body)).unwrap()); }
    let original = parse_nuis_module(&source(
        "if gate { let saved = saved && decide(first); } else { let saved = saved || decide(first); }"
    )).unwrap();
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
                "reference" => observer.params[2].ty.is_ref = true,
                "optional" => observer.params[2].ty.is_optional = true,
                "generic" => observer.params[2].ty.generic_args.push(scalar_type("bool")),
                "i64" => observer.params[2].ty = scalar_type("i64"),
                _ => {
                    let NirStmt::If { else_body, .. } = &mut observer.body[0] else {
                        unreachable!()
                    };
                    let NirStmt::Let {
                        ty,
                        value: NirExpr::Binary { rhs, .. },
                        ..
                    } = &mut else_body[0]
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
fn top_logical_effectful_scalar_selections_capture_current_merges_and_keep_previously_published_constants_sealed(
) {
    let (module, generated) = rewrite(
        "let __nuis_effect_call_gate_0 = first;
        if gate { let saved = saved && decide(first); }
        if first { let saved = saved || decide(saved); } else { let saved = decide(saved); }
        if decide(gate) {} else { let saved: bool = saved && first; }",
    );
    assert_eq!(generated.len(), 9);
    crate::nir_verify::verify_nir_module(&module).unwrap();
    let observer = module
        .functions
        .iter()
        .find(|f| f.name == "observe")
        .unwrap();
    let joins = observer
        .body
        .iter()
        .filter(|statement| {
            matches!(statement,
        NirStmt::If { then_body, else_body, .. }
            if [then_body, else_body].iter().all(|body| matches!(body.as_slice(),
                [NirStmt::Let { name, value: NirExpr::Var(_), .. }] if name == "saved")))
        })
        .count();
    assert_eq!(joins, 3);
    assert_eq!(
        observer
            .body
            .iter()
            .filter(|statement| matches!(statement,
        NirStmt::Let { name, .. } if name == "__nuis_effect_call_gate_0"))
            .count(),
        1
    );
    for function in module
        .functions
        .iter()
        .filter(|f| f.name.starts_with("__nuis_effect_call_arm_"))
    {
        assert!(function.params.iter().any(|p| p.name == "saved"));
    }
    let body =
        "if gate { const saved: bool = decide(first); } else { const saved: bool = decide(saved); }
        if first { let saved = saved && decide(first); }";
    let (module, generated) = rewrite(body);
    assert_eq!(generated.len(), 2);
    let observer = module
        .functions
        .iter()
        .find(|f| f.name == "observe")
        .unwrap();
    assert!(observer.body.iter().any(|statement| matches!(statement,
        NirStmt::If { then_body, .. } if matches!(then_body.as_slice(), [NirStmt::Let { value: NirExpr::Binary { op: NirBinaryOp::And, .. }, .. }]))));
}

#[test]
fn top_logical_effectful_scalar_selections_share_edge_expression_and_complete_arm_capture_limits() {
    fn balanced(values: &[NirExpr]) -> NirExpr {
        if values.len() == 1 {
            return values[0].clone();
        }
        let middle = values.len() / 2;
        NirExpr::Binary {
            op: NirBinaryOp::Or,
            lhs: Box::new(balanced(&values[..middle])),
            rhs: Box::new(balanced(&values[middle..])),
        }
    }
    fn call(value: NirExpr) -> NirExpr {
        NirExpr::Call {
            callee: "decide".into(),
            args: vec![value],
        }
    }
    fn leaf(value: NirExpr) -> NirStmt {
        NirStmt::Let {
            name: "saved".into(),
            ty: None,
            value,
        }
    }
    let mut scope = BTreeMap::from([
        ("gate".into(), scalar_type("bool")),
        ("saved".into(), scalar_type("bool")),
    ]);
    let signatures = BTreeMap::from([(
        "decide".into(),
        (vec![scalar_type("bool")], scalar_type("bool")),
    )]);
    for edges in [32, 33] {
        let values = vec![call(NirExpr::Var("saved".into())); edges + 1];
        assert_eq!(
            prepare(
                &NirExpr::Var("gate".into()),
                &[leaf(balanced(&values))],
                &[],
                &scope,
                &signatures,
                &BTreeSet::new()
            )
            .is_some(),
            edges == 32
        );
    }
    for captures in [31, 32] {
        let values = (0..captures)
            .map(|i| {
                let name = format!("input{i}");
                scope.insert(name.clone(), scalar_type("bool"));
                call(NirExpr::Var(name))
            })
            .collect::<Vec<_>>();
        // Each RHS individually fits; admission also checks the union of both arms.
        assert_eq!(
            prepare(
                &NirExpr::Var("gate".into()),
                &[leaf(balanced(&values[..16]))],
                &[leaf(balanced(&values[16..]))],
                &scope,
                &signatures,
                &BTreeSet::new()
            )
            .is_some(),
            captures == 31
        );
    }
    for expressions in [255, 256] {
        let mut value = NirExpr::Var("saved".into());
        // Balanced equality is ordinary scalar work, not hidden logical work.
        let values = vec![NirExpr::Bool(true); 64];
        fn equality(values: &[NirExpr]) -> NirExpr {
            if values.len() == 1 {
                return values[0].clone();
            }
            let middle = values.len() / 2;
            NirExpr::Binary {
                op: NirBinaryOp::Eq,
                lhs: Box::new(equality(&values[..middle])),
                rhs: Box::new(equality(&values[middle..])),
            }
        }
        value = NirExpr::Binary {
            op: NirBinaryOp::And,
            lhs: Box::new(call(value)),
            rhs: Box::new(equality(&values[..63])),
        };
        let mut budget = Budget {
            nodes: 3,
            expressions,
            logical_edges: 1,
        };
        assert_eq!(
            branch(
                &equality(&values),
                &[leaf(value.clone())],
                &[],
                &scope,
                &signatures,
                &mut budget,
                0
            )
            .is_some(),
            expressions == 256
        );
        if expressions == 256 {
            assert_eq!(budget.expressions, 0);
        }
        assert!(prepare(
            &equality(&values),
            &[leaf(value)],
            &[],
            &scope,
            &signatures,
            &BTreeSet::new()
        )
        .is_some());
        assert!(prepare(
            &equality(&values),
            &[leaf(NirExpr::Binary {
                op: NirBinaryOp::And,
                lhs: Box::new(call(NirExpr::Var("saved".into()))),
                rhs: Box::new(equality(&values))
            })],
            &[],
            &scope,
            &signatures,
            &BTreeSet::new()
        )
        .is_none());
    }
}
