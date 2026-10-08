use super::*;
use crate::frontend::parse_nuis_module;

pub(super) fn source(kind: &str, body: &str) -> String {
    format!("mod cpu Main {{
        fn __nuis_effect_call_predicate_0() -> bool {{ return true; }}
        fn work(v: {kind}) -> {kind} {{ print(70); return v; }}
        fn decide(v: {kind}) -> bool {{ print(98); return v == v; }}
        fn observe(gate: bool, first: bool, second: bool, saved: {kind}, __nuis_effect_predicate_gate_0: bool) -> {kind} {{
            {body} return saved; }} fn main() -> i64 {{ return 0; }} }}")
}

#[test]
fn logical_effectful_scalar_selections_guard_complete_rhs_and_keep_actual_child_merges() {
    let mut module = parse_nuis_module(&source(
        "i64",
        "if gate {
        if first && decide(saved / 2) { let saved = work(saved); }
        if second || decide(saved / 2) {} else { let saved = work(saved); } }",
    ))
    .unwrap();
    let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
    let generated = outline(&mut module, &mut names);
    crate::nir_verify::verify_nir_module(&module).unwrap();
    assert_eq!(generated.len(), 8);
    let predicates = module
        .functions
        .iter()
        .filter(|f| {
            generated.contains(&f.name) && f.name.starts_with("__nuis_effect_call_predicate_")
        })
        .collect::<Vec<_>>();
    assert_eq!(predicates.len(), 2);
    for (function, selected) in predicates.iter().zip([true, false]) {
        assert_eq!(function.return_type, Some(scalar_type("bool")));
        assert_eq!(function.params.len(), 2);
        assert_ne!(function.params[0].name, "__nuis_effect_predicate_gate_0");
        assert!(
            matches!(&function.body[0], NirStmt::If { condition: NirExpr::Binary {
            op: NirBinaryOp::Ne, rhs, .. }, then_body, else_body }
            if rhs.as_ref() == &NirExpr::Bool(selected)
            && then_body == &[NirStmt::Return(Some(NirExpr::Bool(false)))] && else_body.is_empty())
        );
        let NirStmt::Return(Some(result)) = &function.body[1] else {
            unreachable!()
        };
        let original = if selected {
            result
        } else {
            let NirExpr::Binary {
                op: NirBinaryOp::Eq,
                lhs,
                rhs,
            } = result
            else {
                unreachable!()
            };
            assert_eq!(rhs.as_ref(), &NirExpr::Bool(false));
            lhs.as_ref()
        };
        assert!(matches!(original, NirExpr::Call { callee, args }
            if callee == "decide" && matches!(&args[0], NirExpr::Binary { op: NirBinaryOp::Div, .. })));
    }
    let carrier = module
        .functions
        .iter()
        .find(|f| {
            f.body
                .iter()
                .filter(|s| predicate_args(s).is_some())
                .count()
                == 2
        })
        .unwrap();
    assert!(matches!(carrier.body.first(), Some(NirStmt::If { .. })));
    let sites = carrier
        .body
        .iter()
        .enumerate()
        .filter_map(|(i, s)| match predicate_args(s) {
            Some(args) => {
                assert!(args
                    .iter()
                    .all(|a| matches!(a, NirExpr::Var(_) | NirExpr::Bool(_))));
                Some(i)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(
        matches!(&carrier.body[sites[1] - 1], NirStmt::If { then_body, else_body, .. }
        if !then_body.is_empty() && !else_body.is_empty())
    );
}

pub(super) fn predicate_args(statement: &NirStmt) -> Option<&[NirExpr]> {
    let NirStmt::Let { value, .. } = statement else {
        return None;
    };
    let value = match value {
        NirExpr::Binary {
            op: NirBinaryOp::Eq,
            lhs,
            rhs,
        } if rhs.as_ref() == &NirExpr::Bool(false) => lhs.as_ref(),
        value => value,
    };
    let NirExpr::Call { callee, args } = value else {
        return None;
    };
    callee
        .starts_with("__nuis_effect_call_predicate_")
        .then_some(args)
}

#[test]
fn logical_effectful_scalar_selections_preserve_owned_types_literals_polarity_and_hygiene() {
    for kind in ["bool", "i32", "i64", "f32", "f64"] {
        for op in ["&&", "||"] {
            for left in ["first", "true", "false"] {
                for inverted in [false, true] {
                    let update = "let saved = work(saved);";
                    let child = if inverted {
                        format!("if {left} {op} decide(saved) {{}} else {{ {update} }}")
                    } else {
                        format!("if {left} {op} decide(saved) {{ {update} }}")
                    };
                    let text = source(
                        kind,
                        &format!("if gate {{ {child} if second {{ {update} }} }}"),
                    );
                    let mut module = parse_nuis_module(&text).unwrap();
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
fn logical_effectful_scalar_selections_reject_unproved_operands_signatures_and_private_escapes_atomically(
) {
    for condition in [
        "(first || second) == true && decide(saved)",
        "(first && second) == false || decide(saved)",
        "first && (second || decide(saved)) == true",
    ] {
        let mut module = parse_nuis_module(&source("i64", &format!("if gate {{
            if {condition} {{ let saved = work(saved); }} if second {{ let saved = work(saved); }} }}"))).unwrap();
        let before = module.clone();
        let mut names: BTreeSet<String> = module.functions.iter().map(|f| f.name.clone()).collect();
        let before_names = names.clone();
        assert!(outline(&mut module, &mut names).is_empty(), "{condition}");
        assert_eq!(module, before);
        assert_eq!(names, before_names);
    }
    for mutation in [
        "async",
        "ref",
        "optional",
        "generic",
        "resource",
        "pure",
        "escape",
        "left_type",
        "rhs_type",
        "missing",
        "absent",
    ] {
        let mut module = parse_nuis_module(&source(
            "i64",
            "if gate {
            if first { let saved = saved; } if second && decide(saved) { let saved = saved; } }",
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
            "pure" => {
                decide.body.remove(0);
            }
            "left_type" | "rhs_type" | "missing" | "absent" => {
                let observer = module
                    .functions
                    .iter_mut()
                    .find(|f| f.name == "observe")
                    .unwrap();
                let NirStmt::If { then_body, .. } = &mut observer.body[0] else {
                    unreachable!()
                };
                let NirStmt::If { condition, .. } = &mut then_body[1] else {
                    unreachable!()
                };
                let NirExpr::Binary { lhs, rhs, .. } = condition else {
                    unreachable!()
                };
                if mutation == "left_type" {
                    *lhs = Box::new(NirExpr::Var("saved".into()));
                } else {
                    *rhs = Box::new(NirExpr::Call {
                        callee: if mutation == "rhs_type" {
                            "work"
                        } else if mutation == "absent" {
                            "absent"
                        } else {
                            "decide"
                        }
                        .into(),
                        args: vec![NirExpr::Var(
                            if mutation == "missing" {
                                "missing"
                            } else {
                                "saved"
                            }
                            .into(),
                        )],
                    });
                }
            }
            "escape" => {
                let observer = module
                    .functions
                    .iter_mut()
                    .find(|f| f.name == "observe")
                    .unwrap();
                let NirStmt::If { then_body, .. } = &mut observer.body[0] else {
                    unreachable!()
                };
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
                let NirStmt::If { condition, .. } = &mut then_body[1] else {
                    unreachable!()
                };
                let NirExpr::Binary { rhs, .. } = condition else {
                    unreachable!()
                };
                *rhs = Box::new(NirExpr::Call {
                    callee: "decide".into(),
                    args: vec![NirExpr::Var("private".into())],
                });
            }
            _ => unreachable!(),
        }
        let before = module.clone();
        let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
        assert!(outline(&mut module, &mut names).is_empty(), "{mutation}");
        assert_eq!(module, before);
    }
}

#[test]
fn logical_effectful_scalar_selections_share_expression_depth_and_rhs_capture_limits() {
    let scope = [
        ("first".into(), scalar_type("bool")),
        ("saved".into(), scalar_type("i64")),
    ]
    .into();
    let signatures = [(
        "decide".into(),
        (vec![scalar_type("i64")], scalar_type("bool")),
    )]
    .into();
    let condition = NirExpr::Binary {
        op: NirBinaryOp::And,
        lhs: Box::new(NirExpr::Var("first".into())),
        rhs: Box::new(NirExpr::Call {
            callee: "decide".into(),
            args: vec![NirExpr::Var("saved".into())],
        }),
    };
    for expressions in [3, 4] {
        let mut budget = nested::Budget {
            nodes: 1,
            expressions,
            logical_edges: 32,
        };
        assert_eq!(
            prove(&condition, &scope, &signatures, &mut budget).is_some(),
            expressions == 4
        );
        assert_eq!(budget.nodes, 1);
        assert_eq!(budget.expressions, 0);
    }
    for count in [31, 32] {
        let scope = (0..count)
            .map(|i| (format!("v{i}"), scalar_type("i64")))
            .collect();
        let signatures = [(
            "all".into(),
            (vec![scalar_type("i64"); count], scalar_type("bool")),
        )]
        .into();
        let condition = NirExpr::Binary {
            op: NirBinaryOp::Or,
            lhs: Box::new(NirExpr::Bool(false)),
            rhs: Box::new(NirExpr::Call {
                callee: "all".into(),
                args: (0..count).map(|i| NirExpr::Var(format!("v{i}"))).collect(),
            }),
        };
        assert_eq!(
            prove(
                &condition,
                &scope,
                &signatures,
                &mut nested::Budget {
                    nodes: 64,
                    expressions: 256,
                    logical_edges: 32,
                }
            )
            .is_some(),
            count == 31
        );
    }
    for additions in [29, 30] {
        let mut value = NirExpr::Int(0);
        for _ in 0..additions {
            value = NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(value),
                rhs: Box::new(NirExpr::Int(1)),
            };
        }
        let condition = NirExpr::Binary {
            op: NirBinaryOp::And,
            lhs: Box::new(NirExpr::Bool(true)),
            rhs: Box::new(NirExpr::Binary {
                op: NirBinaryOp::Eq,
                lhs: Box::new(value),
                rhs: Box::new(NirExpr::Int(0)),
            }),
        };
        assert_eq!(
            prove(
                &condition,
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
