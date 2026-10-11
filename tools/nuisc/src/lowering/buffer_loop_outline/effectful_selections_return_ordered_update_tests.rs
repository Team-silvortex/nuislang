use super::tests::{reject_module, rewrite, source};
use super::*;
use crate::frontend::parse_nuis_module;

#[test]
fn return_ordered_update_logical_effectful_scalar_selections_keep_source_order_current_versions_private_children_and_retention(
) {
    for update in [
        "if first { let local = local || decide(first); let local = local && saved; }",
        "if first {} else { let local = local && decide(first); let local = local || saved; }",
        "if first { if saved { let local = local || decide(first); }
            let child = local && first; if child { let local = local || saved; }
            else { let local = local && saved; } const seal: bool = child || false;
            let ignored = token(1); let local = local || decide(seal); }",
        "if first { let local = first; if saved {} else { let local = local && decide(first); }
            let child = local; if saved { let child = child || first; }
            if child { let local = local || decide(child); } } else { let local = saved; }",
    ] {
        for inverted in [false, true] {
            let selected = format!(
                "let local = saved; {update} const frozen: bool = local || false;
                if frozen {{ return local && decide(first); }} else {{ return local || saved; }}"
            );
            let (yes, no) = if inverted {
                ("return saved;", selected.as_str())
            } else {
                (selected.as_str(), "return saved;")
            };
            let (module, generated) = rewrite(&format!("if gate {{ {yes} }} else {{ {no} }}"));
            assert!(!generated.is_empty(), "{update}");
            crate::nir_verify::verify_nir_module(&module).unwrap();
            let observer = module
                .functions
                .iter()
                .find(|f| f.name == "observe")
                .unwrap();
            assert_eq!(
                observer.body[observer.body.len() - 2],
                NirStmt::Print(NirExpr::Int(80))
            );
            assert!(module
                .functions
                .iter()
                .filter(|f| generated.contains(&f.name))
                .all(
                    |f| f.params.len() <= 32 && matches!(f.body.first(), Some(NirStmt::If { .. }))
                ));
            for statement in &observer.body {
                if let NirStmt::Let {
                    value: NirExpr::Call { callee, .. },
                    ..
                } = statement
                {
                    let helper = module.functions.iter().find(|f| &f.name == callee).unwrap();
                    assert!(!helper
                        .params
                        .iter()
                        .any(|p| ["local", "child", "seal", "ignored", "frozen"]
                            .contains(&p.name.as_str())));
                }
            }
        }
    }
}

#[test]
fn return_ordered_update_logical_effectful_scalar_selections_separate_delegated_authority_from_source_defined_versions(
) {
    for (yes,no,old_required) in [
        ("let local = local || decide(first); let local = local && saved;","let local = saved;",true),
        ("if saved { let local = local || decide(first); } let local = local && first;","let local = saved;",true),
        ("let before = decide(local); let local = first; let local = local && saved;","let local = saved;",true),
        ("let local = first; if saved { let local = local || decide(first); } let local = local && saved;","let local = saved;",false),
    ] {
        let module = parse_nuis_module(&source(&format!("if gate {{ let local = saved; if first {{ {yes} }} else {{ {no} }} return local && first; }} else {{ return saved; }}"))).unwrap();
        let observer = module.functions.iter().find(|f|f.name == "observe").unwrap();
        let scope = observer.params.iter().map(|p|(p.name.clone(),p.ty.clone())).collect();
        let signatures = BTreeMap::from([("decide".into(),(vec![scalar_type("bool")],scalar_type("bool")))]);
        let NirStmt::If{then_body,..} = &observer.body[1] else {unreachable!()};
        let mut budget = nested::Budget{nodes:64,expressions:256,logical_edges:32};
        let staged = super::super::stage(&then_body[..1],&scope,&signatures,&mut budget,1).unwrap();
        let NirStmt::If{condition,then_body:yes,else_body:no} = &then_body[1] else {unreachable!()};
        let proof = prove(condition,yes,no,&staged,&signatures,&mut budget,1,None).unwrap();
        assert_eq!(proof.inputs.contains("local"),old_required);
        assert!(!proof.inputs.contains("before"));
        let SelectedValue::Nested(plan) = proof.value else {unreachable!()};
        assert_eq!(plan.params.iter().any(|p|p.name == "local"),old_required);
        assert_eq!(plan.destination,Some(("local".into(),false)));
    }
    // An arbitrary entering bool cannot obtain the private-owner capability.
    let module = parse_nuis_module(&source("if gate { if first { let saved = first; let saved = decide(first); } return saved && first; } else { return saved; }")).unwrap();
    reject_module(module);
}

#[test]
fn return_ordered_update_logical_effectful_scalar_selections_reject_extra_entering_writes_seals_mixed_targets_and_late_leaks_atomically(
) {
    for update in [
        "if first { let other = saved; let local = decide(first); }",
        "if first { if saved { let other = first; } let local = decide(first); }",
        "if first { let local = first; const local: bool = first || false; let local = decide(first); }",
        "if first { let local = first; let local: i64 = 1; let local = decide(first); }",
        "if first { let local = first; let local = boolean(local || saved); }",
        "if first { let child = first; const seal: bool = child || false;
            if saved { let seal = first; } let local = decide(child); }",
        "if first { let local = first; if saved { let other = first; } else { let local = saved; } }",
        "if first { let local = first; return local; let local = decide(first); }",
        "if first { let local = first; while saved { let local = first; } let local = decide(first); }",
        "if first { let local = first; print(96); let local = decide(first); }",
        "if first { let local = first; let other = saved; if decide(saved) {} else {} }",
    ] {
        reject_module(parse_nuis_module(&source(&format!("if decide(gate) {{ let local = saved; let other = first; {update} return local && first; }} else {{ return saved; }}"))).unwrap());
    }
    reject_module(parse_nuis_module(&source("if gate { const local: bool = saved || false; if first { let local = first; let local = decide(first); } return local && first; } else { return saved; }")).unwrap());
    let original = parse_nuis_module(&source("if gate { let local = saved; if first { let local = local || decide(first); let child = local; let local = local && child; } else { let local = saved; } return local && first; } else { return saved; }")).unwrap();
    for mutation in ["borrowed", "optional", "generic", "late-leak"] {
        let mut module = original.clone();
        let observer = module
            .functions
            .iter_mut()
            .find(|f| f.name == "observe")
            .unwrap();
        let NirStmt::If { then_body, .. } = &mut observer.body[1] else {
            unreachable!()
        };
        let NirStmt::If {
            then_body,
            else_body,
            ..
        } = &mut then_body[1]
        else {
            unreachable!()
        };
        if mutation == "late-leak" {
            else_body[0] = NirStmt::Let {
                name: "local".into(),
                ty: None,
                value: NirExpr::Var("child".into()),
            };
        } else {
            let NirStmt::Let { ty, .. } = &mut then_body[0] else {
                unreachable!()
            };
            let mut kind = scalar_type("bool");
            match mutation {
                "borrowed" => kind.is_ref = true,
                "optional" => kind.is_optional = true,
                "generic" => kind.generic_args.push(scalar_type("bool")),
                _ => unreachable!(),
            }
            *ty = Some(kind);
        }
        reject_module(module);
    }
}

#[test]
fn return_ordered_update_logical_effectful_scalar_selections_charge_original_writes_conditions_and_unused_work_once(
) {
    for inverted in [false, true] {
        let selected =
            "let local = local || decide(first); if saved { let local = local && first; }
            let ignored = token(1); let local = local || first;";
        let update = if inverted {
            format!("if first {{}} else {{ {selected} }}")
        } else {
            format!("if first {{ {selected} }}")
        };
        let module = parse_nuis_module(&source(&format!("if decide(gate) {{ let local = saved; {update} return local && first; }} else {{ return saved; }}"))).unwrap();
        let observer = module
            .functions
            .iter()
            .find(|f| f.name == "observe")
            .unwrap();
        let NirStmt::If {
            condition,
            then_body,
            else_body,
        } = &observer.body[1]
        else {
            unreachable!()
        };
        let scope = observer
            .params
            .iter()
            .map(|p| (p.name.clone(), p.ty.clone()))
            .collect();
        let signatures = BTreeMap::from([
            (
                "decide".into(),
                (vec![scalar_type("bool")], scalar_type("bool")),
            ),
            (
                "token".into(),
                (vec![scalar_type("i64")], scalar_type("i64")),
            ),
        ]);
        for (nodes, expressions, logical_edges, accepted) in [
            (9, 21, 4, false),
            (10, 20, 4, false),
            (10, 21, 3, false),
            (10, 21, 4, true),
        ] {
            let mut budget = nested::Budget {
                nodes,
                expressions,
                logical_edges,
            };
            assert_eq!(
                super::super::prove(
                    condition,
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
fn return_ordered_update_logical_effectful_scalar_selections_share_prefix_original_nodes_edges_and_depth_limits(
) {
    for count in [16, 17] {
        let prefix = "let local = local;".repeat(count);
        assert_eq!(!rewrite(&format!("if decide(gate) {{ let local = saved; if first {{ {prefix} let local = first; }} return local && first; }} else {{ return saved; }}")).1.is_empty(),count==16);
    }
    fn full(depth: usize) -> String {
        if depth == 0 {
            "return saved && first;".into()
        } else {
            let child = full(depth - 1);
            format!("if gate {{ {child} }} else {{ {child} }}")
        }
    }
    for writes in [14, 15] {
        let first = "let local = local;".repeat(14);
        let second = "let local = local;".repeat(writes);
        assert_eq!(!rewrite(&format!("if decide(gate) {{ let local = saved; if first {{ {first} let local = first; }} if first {{ {second} let local = first; }} {} }} else {{ {} }}",full(3),full(3))).1.is_empty(),writes==14);
    }
    for extra in [false, true] {
        let prefix = (0..15)
            .map(|n| {
                if extra && n == 14 {
                    "let local = (local || first) || saved;"
                } else {
                    "let local = local || first;"
                }
            })
            .collect::<String>();
        let ordinary = "let local = local || first;".repeat(15);
        assert_eq!(!rewrite(&format!("if decide(gate) {{ let local = saved; if first {{ {prefix} let local = local || first; }} else {{ {ordinary} let local = first; }} return local && first; }} else {{ return saved; }}")).1.is_empty(),!extra);
    }
    for levels in [5, 6] {
        let mut child =
            "let local = saved; if first { if saved { let local = first; } let local = local || saved; } return local && first;"
                .to_string();
        for _ in 0..levels {
            child = format!("if first {{ {child} }} else {{ return saved; }}");
        }
        assert_eq!(
            !rewrite(&format!(
                "if decide(gate) {{ {child} }} else {{ return saved; }}"
            ))
            .1
            .is_empty(),
            levels == 5
        );
    }
}
