use super::*;
use crate::frontend::parse_nuis_module;

fn source(body: &str) -> String {
    format!(
        "mod cpu Main {{
        fn __nuis_effect_call_predicate_0() -> bool {{ return true; }}
        fn decide(v: bool) -> bool {{ print(98); return v; }}
        fn boolean(v: bool) -> bool {{ return v; }}
        fn observe(gate: bool, first: bool, saved: bool) -> bool {{
            print(99); {body} print(80); return false;
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
fn return_logical_effectful_scalar_selections_keep_original_returns_polarities_conditions_and_parent_effects(
) {
    for root in [
        "saved && decide(first)",
        "saved || decide(first)",
        "decide(saved) && (saved || decide(first))",
    ] {
        for opposite in ["false", "saved", "saved || decide(first)"] {
            for inverted in [false, true] {
                let (yes, no) = if inverted {
                    (opposite, root)
                } else {
                    (root, opposite)
                };
                let (module, generated) = rewrite(&format!(
                    "if gate {{ return {yes}; }} else {{ return {no}; }}"
                ));
                let edges = if root.starts_with("decide") { 2 } else { 1 }
                    + usize::from(opposite.contains("||"));
                assert_eq!(generated.len(), 2 + edges);
                assert!(!generated.contains("__nuis_effect_call_predicate_0"));
                crate::nir_verify::verify_nir_module(&module).unwrap();
                let observer = module
                    .functions
                    .iter()
                    .find(|f| f.name == "observe")
                    .unwrap();
                assert_eq!(
                    observer.body.first(),
                    Some(&NirStmt::Print(NirExpr::Int(99)))
                );
                let original_suffix = &observer.body[observer.body.len() - 2..];
                assert_eq!(
                    original_suffix,
                    &[
                        NirStmt::Print(NirExpr::Int(80)),
                        NirStmt::Return(Some(NirExpr::Bool(false)))
                    ]
                );
                assert!(matches!(
                    observer.body[observer.body.len() - 3],
                    NirStmt::Return(Some(NirExpr::Var(_)))
                ));
                for function in module
                    .functions
                    .iter()
                    .filter(|f| generated.contains(&f.name))
                {
                    assert_eq!(function.return_type, Some(scalar_type("bool")));
                    assert!(matches!(function.body.first(), Some(NirStmt::If { .. })));
                    assert!(matches!(
                        function.body.last(),
                        Some(NirStmt::Return(Some(_)))
                    ));
                    assert!(function.params.len() <= 32);
                }
            }
        }
    }
    for body in [
        "if decide(gate) { return saved && first; } else { return saved || first; }",
        "if gate || decide(first) { return saved && first; } else { return saved; }",
    ] {
        let (module, generated) = rewrite(body);
        assert!(!generated.is_empty());
        crate::nir_verify::verify_nir_module(&module).unwrap();
    }
}

#[test]
fn return_logical_effectful_scalar_selections_reject_mixed_single_nested_hidden_pure_and_invalid_types_atomically(
) {
    for body in [
        "if gate { return saved && decide(first); }",
        "if gate {} else { return saved || decide(first); }",
        "if gate { return saved && decide(first); } else { let saved = decide(first); }",
        "if gate { let saved = first; return saved && decide(saved); } else { return saved; }",
        "if gate { if first { return saved && decide(first); } else { let saved = first; } } else { return saved; }",
        "if gate { return saved && decide(first); } else { return boolean(saved || decide(first)); }",
        "if gate { return (saved && decide(first)) == true; } else { return saved; }",
        "if gate { return saved && boolean(first); } else { return saved; }",
        "while gate { if first { return saved && decide(first); } else { return saved; } }",
    ] { reject_module(parse_nuis_module(&source(body)).unwrap()); }
    let original = parse_nuis_module(&source(
        "if gate { return saved && decide(first); } else { return saved || decide(first); }",
    ))
    .unwrap();
    for mutation in [
        "missing-return",
        "return-reference",
        "return-optional",
        "return-generic",
        "return-i64",
        "input-reference",
        "missing",
        "argument",
        "void",
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
                "missing-return" => observer.return_type = None,
                "return-reference" => observer.return_type.as_mut().unwrap().is_ref = true,
                "return-optional" => observer.return_type.as_mut().unwrap().is_optional = true,
                "return-generic" => observer
                    .return_type
                    .as_mut()
                    .unwrap()
                    .generic_args
                    .push(scalar_type("bool")),
                "return-i64" => observer.return_type = Some(scalar_type("i64")),
                "input-reference" => observer.params[2].ty.is_ref = true,
                _ => {
                    let NirStmt::If { else_body, .. } = &mut observer.body[1] else {
                        unreachable!()
                    };
                    if mutation == "void" {
                        else_body[0] = NirStmt::Return(None);
                    } else {
                        let NirStmt::Return(Some(NirExpr::Binary { rhs, .. })) = &mut else_body[0]
                        else {
                            unreachable!()
                        };
                        if mutation == "missing" {
                            *rhs = Box::new(NirExpr::Var("absent".into()));
                        } else {
                            let NirExpr::Call { args, .. } = rhs.as_mut() else {
                                unreachable!()
                            };
                            args[0] = NirExpr::Int(1);
                        }
                    }
                }
            }
        }
        reject_module(module);
    }
}

#[test]
fn return_logical_effectful_scalar_selections_charge_original_returns_once_with_shared_condition_edges(
) {
    let scope = BTreeMap::from([
        ("gate".into(), scalar_type("bool")),
        ("saved".into(), scalar_type("bool")),
    ]);
    let yes = NirExpr::Binary {
        op: NirBinaryOp::And,
        lhs: Box::new(NirExpr::Var("saved".into())),
        rhs: Box::new(NirExpr::Bool(true)),
    };
    let no = NirExpr::Bool(false);
    let left = NirExpr::Call {
        callee: "condition".into(),
        args: vec![NirExpr::Var("gate".into())],
    };
    let signatures = BTreeMap::from([(
        "condition".into(),
        (vec![scalar_type("bool")], scalar_type("bool")),
    )]);
    for logical_condition in [false, true] {
        let condition = if logical_condition {
            NirExpr::Binary {
                op: NirBinaryOp::Or,
                lhs: Box::new(left.clone()),
                rhs: Box::new(NirExpr::Bool(false)),
            }
        } else {
            left.clone()
        };
        let expressions = if logical_condition { 8 } else { 6 };
        let edges = if logical_condition { 2 } else { 1 };
        for (nodes, expressions, edges, accepted) in [
            (2, expressions, edges, false),
            (3, expressions - 1, edges, false),
            (3, expressions, edges - 1, false),
            (3, expressions, edges, true),
        ] {
            let mut budget = nested::Budget {
                nodes,
                expressions,
                logical_edges: edges,
            };
            assert_eq!(
                prove(
                    &condition,
                    &yes,
                    &no,
                    &scope,
                    &signatures,
                    &BTreeSet::new(),
                    &mut budget
                )
                .is_some(),
                accepted
            );
            if accepted {
                assert_eq!(
                    (budget.nodes, budget.expressions, budget.logical_edges),
                    (0, 0, 0)
                );
            }
        }
    }
}

#[test]
fn return_logical_effectful_scalar_selections_share_complete_arm_capture_expression_edge_and_depth_bounds(
) {
    fn balanced(values: &[NirExpr], op: NirBinaryOp) -> NirExpr {
        if values.len() == 1 {
            return values[0].clone();
        }
        let middle = values.len() / 2;
        NirExpr::Binary {
            op,
            lhs: Box::new(balanced(&values[..middle], op)),
            rhs: Box::new(balanced(&values[middle..], op)),
        }
    }
    fn call(value: NirExpr) -> NirExpr {
        NirExpr::Call {
            callee: "decide".into(),
            args: vec![value],
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
    let admit = |condition: &NirExpr, yes, no, scope: &Scope| {
        prepare(
            Some(&scalar_type("bool")),
            condition,
            &[NirStmt::Return(Some(yes))],
            &[NirStmt::Return(Some(no))],
            scope,
            &signatures,
            &BTreeSet::new(),
        )
        .is_some()
    };
    for count in [31, 32] {
        let values = (0..count)
            .map(|i| {
                let name = format!("input{i}");
                scope.insert(name.clone(), scalar_type("bool"));
                call(NirExpr::Var(name))
            })
            .collect::<Vec<_>>();
        assert_eq!(
            admit(
                &NirExpr::Var("gate".into()),
                balanced(&values[..16], NirBinaryOp::Or),
                balanced(&values[16..], NirBinaryOp::Or),
                &scope
            ),
            count == 31
        );
    }
    for edges in [32, 33] {
        assert_eq!(
            admit(
                &NirExpr::Var("gate".into()),
                balanced(
                    &vec![call(NirExpr::Var("saved".into())); edges + 1],
                    NirBinaryOp::Or
                ),
                NirExpr::Bool(false),
                &scope
            ),
            edges == 32
        );
    }
    for depth in [31, 32] {
        let mut value = call(NirExpr::Var("saved".into()));
        for _ in 0..depth - 1 {
            value = NirExpr::Binary {
                op: NirBinaryOp::And,
                lhs: Box::new(NirExpr::Bool(true)),
                rhs: Box::new(value),
            };
        }
        assert_eq!(
            admit(
                &NirExpr::Var("gate".into()),
                value,
                NirExpr::Bool(false),
                &scope
            ),
            depth == 31
        );
    }
    for width in [63, 64] {
        let equal = balanced(&vec![NirExpr::Bool(true); 64], NirBinaryOp::Eq);
        let yes = NirExpr::Binary {
            op: NirBinaryOp::And,
            lhs: Box::new(call(NirExpr::Var("saved".into()))),
            rhs: Box::new(balanced(&vec![NirExpr::Bool(true); width], NirBinaryOp::Eq)),
        };
        assert_eq!(
            admit(&equal, yes, NirExpr::Bool(false), &scope),
            width == 63
        );
    }
}
