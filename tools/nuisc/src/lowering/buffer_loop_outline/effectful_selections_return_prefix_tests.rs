use super::*;
use crate::frontend::parse_nuis_module;

fn source(body: &str) -> String {
    format!(
        "mod cpu Main {{
        fn decide(v: bool) -> bool {{ print(98); return v; }}
        fn boolean(v: bool) -> bool {{ return v; }}
        fn token(v: i64) -> i64 {{ print(97); return v; }}
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

fn reject(body: &str) {
    let mut module = parse_nuis_module(&source(body)).unwrap();
    let before = module.clone();
    let mut names: BTreeSet<_> = module.functions.iter().map(|f| f.name.clone()).collect();
    let before_names = names.clone();
    assert!(outline(&mut module, &mut names).is_empty(), "{body}");
    assert_eq!(module, before);
    assert_eq!(names, before_names);
}

#[test]
fn return_prefix_logical_effectful_scalar_selections_keep_selected_private_versions_seals_unused_calls_and_original_exits(
) {
    let staged = "let local = decide(saved); let local = local && first;
        const sealed: bool = local || false; let unused = token(1);
        return sealed && decide(first);";
    for opposite in [
        "return saved;",
        "let local = token(1); return false;",
        "let local = first; return local || decide(saved);",
    ] {
        for inverted in [false, true] {
            let (yes, no) = if inverted {
                (opposite, staged)
            } else {
                (staged, opposite)
            };
            let (module, generated) = rewrite(&format!("if gate {{ {yes} }} else {{ {no} }}"));
            assert!(!generated.is_empty());
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
            let tail = &observer.body[observer.body.len() - 3..];
            assert!(matches!(tail[0], NirStmt::Return(Some(NirExpr::Var(_)))));
            assert_eq!(
                tail[1..],
                [
                    NirStmt::Print(NirExpr::Int(80)),
                    NirStmt::Return(Some(NirExpr::Bool(false)))
                ]
            );
            assert!(!observer.body.iter().any(
                |s| matches!(s, NirStmt::Let { name, .. } | NirStmt::Const { name, .. }
                if ["local", "sealed", "unused"].contains(&name.as_str()))
            ));
            for arm in module
                .functions
                .iter()
                .filter(|f| f.name.starts_with("__nuis_effect_call_arm"))
            {
                assert!(matches!(arm.body.first(), Some(NirStmt::If { .. })));
                assert!(matches!(arm.body.last(), Some(NirStmt::Return(Some(_)))));
                assert!(!arm
                    .params
                    .iter()
                    .any(|p| ["local", "sealed", "unused"].contains(&p.name.as_str())));
                assert!(arm.params.len() <= 32);
            }
            let arm = module
                .functions
                .iter()
                .find(|f| {
                    f.body.iter().any(|s| {
                        matches!(s,
                NirStmt::Const { name, .. } if name == "sealed")
                    })
                })
                .unwrap();
            assert!(
                matches!(&arm.body[1], NirStmt::Let { name, value: NirExpr::Call { callee, .. }, .. }
                if name == "local" && callee == "decide")
            );
            assert!(
                matches!(&arm.body[4], NirStmt::Let { name, value: NirExpr::Call { callee, .. }, .. }
                if name == "unused" && callee == "token")
            );
        }
    }
    // A used or unused prefix call, or the condition alone, can discover effects.
    for body in [
        "if gate { let unused = token(1); return saved && first; } else { return saved; }",
        "if decide(gate) { let local = saved; return local && first; } else { return saved; }",
    ] {
        assert!(!rewrite(body).1.is_empty());
    }
}

#[test]
fn return_prefix_logical_effectful_scalar_selections_reject_outer_writes_mixed_exits_hidden_roots_cross_arm_leaks_and_late_errors_atomically(
) {
    for body in [
        "if gate { let local = first; return saved && decide(local); }",
        "if gate { let local = first; return saved && decide(local); } else { let saved = first; }",
        "if gate { let saved = first; return saved && decide(first); } else { return saved; }",
        "if gate { const saved: bool = first || false; return saved && decide(first); } else { return saved; }",
        "if gate { print(96); return saved && decide(first); } else { return saved; }",
        "if gate { let local = first; return saved && decide(local); let unused = token(1); } else { return saved; }",
        "if gate { return; return saved && decide(first); } else { return saved; }",
        "if gate { if first { let local = saved; } return saved && decide(first); } else { return saved; }",
        "if gate { while first { let local = saved; } return saved && decide(first); } else { return saved; }",
        "if gate { let local = saved; return local && decide(first); } else { return boolean(saved || first); }",
        "if gate { let local = boolean(saved || decide(first)); return saved && decide(first); } else { return saved; }",
        "if gate { const local: bool = saved || first; let local = local && first; return saved && decide(first); } else { return saved; }",
        "if gate { const local: bool = saved || first; let local = first; return saved && decide(first); } else { return saved; }",
        "if gate { const local: i64 = 1; return saved && decide(first); } else { return saved; }",
        "if gate { let local = first; return saved && boolean(local); } else { return saved; }",
    ] { reject(body); }
    for mutation in [
        "missing",
        "borrowed",
        "optional",
        "generic",
        "integer",
        "input-reference",
        "async",
        "absent",
        "cross-arm",
        "argument",
        "prefix-type",
        "type-change",
    ] {
        let mut module = parse_nuis_module(&source(
            "if gate { let local = decide(first); return saved && local; } else { let other = first; return other; }",
        ))
        .unwrap();
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
                "missing" => observer.return_type = None,
                "borrowed" => observer.return_type.as_mut().unwrap().is_ref = true,
                "optional" => observer.return_type.as_mut().unwrap().is_optional = true,
                "generic" => observer
                    .return_type
                    .as_mut()
                    .unwrap()
                    .generic_args
                    .push(scalar_type("bool")),
                "integer" => observer.return_type = Some(scalar_type("i64")),
                "input-reference" => observer.params[1].ty.is_ref = true,
                _ => {
                    let NirStmt::If { else_body, .. } = &mut observer.body[1] else {
                        unreachable!()
                    };
                    match mutation {
                        "absent" => {
                            let NirStmt::Let { value, .. } = &mut else_body[0] else {
                                unreachable!()
                            };
                            *value = NirExpr::Var("absent".into());
                        }
                        "cross-arm" => {
                            else_body[1] = NirStmt::Return(Some(NirExpr::Var("local".into())))
                        }
                        "argument" => {
                            let NirStmt::Let { value, .. } = &mut else_body[0] else {
                                unreachable!()
                            };
                            *value = NirExpr::Call {
                                callee: "decide".into(),
                                args: vec![NirExpr::Int(1)],
                            };
                        }
                        "prefix-type" => {
                            let NirStmt::Let { ty, .. } = &mut else_body[0] else {
                                unreachable!()
                            };
                            *ty = Some(scalar_type("i64"));
                        }
                        _ => else_body.insert(
                            1,
                            NirStmt::Let {
                                name: "other".into(),
                                ty: None,
                                value: NirExpr::Int(1),
                            },
                        ),
                    }
                }
            }
        }
        let before = module.clone();
        let mut names: BTreeSet<_> = module.functions.iter().map(|f| f.name.clone()).collect();
        let before_names = names.clone();
        assert!(outline(&mut module, &mut names).is_empty());
        assert_eq!(module, before);
        assert_eq!(names, before_names);
    }
}

#[test]
fn return_prefix_logical_effectful_scalar_selections_charge_original_prefixes_returns_and_condition_once(
) {
    let module = parse_nuis_module(&source("if decide(gate) { let local = decide(saved); let local = local && true; return local && first; } else { return false; }")).unwrap();
    let function = module
        .functions
        .iter()
        .find(|f| f.name == "observe")
        .unwrap();
    let scope = function
        .params
        .iter()
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect();
    let signatures = BTreeMap::from([(
        "decide".into(),
        (vec![scalar_type("bool")], scalar_type("bool")),
    )]);
    let NirStmt::If {
        condition,
        then_body,
        else_body,
    } = &function.body[1]
    else {
        unreachable!()
    };
    for logical_condition in [false, true] {
        let condition = if logical_condition {
            NirExpr::Binary {
                op: NirBinaryOp::Or,
                lhs: Box::new(condition.clone()),
                rhs: Box::new(NirExpr::Bool(false)),
            }
        } else {
            condition.clone()
        };
        let expressions = if logical_condition { 13 } else { 11 };
        let edges = if logical_condition { 3 } else { 2 };
        for (nodes, expressions, edges, accepted) in [
            (4, expressions, edges, false),
            (5, expressions - 1, edges, false),
            (5, expressions, edges - 1, false),
            (5, expressions, edges, true),
        ] {
            let mut budget = nested::Budget {
                nodes,
                expressions,
                logical_edges: edges,
            };
            assert_eq!(
                prove(
                    &condition,
                    then_body,
                    else_body,
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
fn return_prefix_logical_effectful_scalar_selections_bound_prefix_lengths_all_original_captures_and_shared_expressions_edges_depth(
) {
    for count in [16, 17] {
        let prefix = (0..count)
            .map(|i| format!("let local{i} = saved;"))
            .collect::<String>();
        assert_eq!(
            !rewrite(&format!(
                "if gate {{ {prefix} return saved && decide(first); }} else {{ return saved; }}"
            ))
            .1
            .is_empty(),
            count == 16
        );
    }
    fn balanced(values: &[NirExpr], op: NirBinaryOp) -> NirExpr {
        if values.len() == 1 {
            return values[0].clone();
        }
        let mid = values.len() / 2;
        NirExpr::Binary {
            op,
            lhs: Box::new(balanced(&values[..mid], op)),
            rhs: Box::new(balanced(&values[mid..], op)),
        }
    }
    fn call(v: NirExpr) -> NirExpr {
        NirExpr::Call {
            callee: "decide".into(),
            args: vec![v],
        }
    }
    fn stage(v: NirExpr) -> NirStmt {
        NirStmt::Let {
            name: "local".into(),
            ty: None,
            value: v,
        }
    }
    fn logical(left: NirExpr, right: NirExpr) -> NirExpr {
        NirExpr::Binary {
            op: NirBinaryOp::And,
            lhs: Box::new(left),
            rhs: Box::new(right),
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
    let admit =
        |condition: &NirExpr, prefix: NirExpr, value: NirExpr, no: &[NirStmt], scope: &Scope| {
            prepare(
                condition,
                &[stage(prefix), NirStmt::Return(Some(value))],
                no,
                scope,
                &signatures,
                &BTreeSet::new(),
            )
            .is_some()
        };
    for count in [31, 32] {
        let values = (0..count)
            .map(|i| {
                let name = format!("v{i}");
                scope.insert(name.clone(), scalar_type("bool"));
                NirExpr::Var(name)
            })
            .collect::<Vec<_>>();
        assert_eq!(
            admit(
                &NirExpr::Var("gate".into()),
                balanced(&values[..16], NirBinaryOp::Eq),
                logical(values[0].clone(), call(values[1].clone())),
                &[
                    stage(balanced(&values[16..], NirBinaryOp::Eq)),
                    NirStmt::Return(Some(values[16].clone()))
                ],
                &scope
            ),
            count == 31
        );
    }
    for edges in [31, 32] {
        assert_eq!(
            admit(
                &NirExpr::Var("gate".into()),
                balanced(
                    &vec![call(NirExpr::Var("saved".into())); edges + 1],
                    NirBinaryOp::Or
                ),
                logical(NirExpr::Var("local".into()), NirExpr::Var("saved".into())),
                &[NirStmt::Return(Some(NirExpr::Bool(false)))],
                &scope
            ),
            edges == 31
        );
    }
    for depth in [31, 32] {
        let mut value = call(NirExpr::Var("saved".into()));
        for _ in 0..depth - 1 {
            value = logical(NirExpr::Bool(true), value);
        }
        assert_eq!(
            admit(
                &NirExpr::Var("gate".into()),
                value,
                logical(NirExpr::Var("local".into()), NirExpr::Bool(true)),
                &[NirStmt::Return(Some(NirExpr::Bool(false)))],
                &scope
            ),
            depth == 31
        );
    }
    for width in [63, 64] {
        assert_eq!(
            admit(
                &balanced(&vec![NirExpr::Bool(true); 63], NirBinaryOp::Eq),
                call(NirExpr::Var("saved".into())),
                logical(
                    call(NirExpr::Var("local".into())),
                    balanced(&vec![NirExpr::Bool(true); width], NirBinaryOp::Eq)
                ),
                &[NirStmt::Return(Some(NirExpr::Bool(false)))],
                &scope
            ),
            width == 63
        );
    }
}
