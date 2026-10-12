use super::tests::{reject_module, rewrite, source};
use super::*;
use crate::frontend::parse_nuis_module;

#[test]
fn return_retention_update_logical_effectful_scalar_selections_keep_nonterminal_writes_current_conditions_private_work_and_both_polarities(
) {
    for update in [
        "if first { let local = local || decide(first); if decide(local) {} else {} }",
        "if first {} else { let local = local && decide(first); if local {} else {} }",
        "if first { if saved { let local = local || decide(first); }
            let child = local; const seal: bool = child || false; let ignored = token(1);
            let local = seal && first; if decide(local) { if saved {} else {} } else {} }",
        "if first { let child = local; if saved { let child = child || first; }
            let local = first; if saved { let local = local || decide(child); }
            if decide(local) {} else {} } else { let local = saved; if local {} else {} }",
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
            assert!(module
                .functions
                .iter()
                .filter(|f| generated.contains(&f.name))
                .all(
                    |f| f.params.len() <= 32 && matches!(f.body.first(), Some(NirStmt::If { .. }))
                ));
        }
    }
}

#[test]
fn return_retention_update_logical_effectful_scalar_selections_preserve_first_reads_replacements_and_conditional_current_captures(
) {
    for (yes, old_required) in [
        (
            "let local = local || decide(first); if local {} else {}",
            true,
        ),
        ("let local = first; if decide(local) {} else {}", false),
        (
            "let before = decide(local); let local = first; if local {} else {}",
            true,
        ),
        (
            "if saved { let local = first; } if decide(local) {} else {}",
            true,
        ),
    ] {
        let module = parse_nuis_module(&source(&format!(
            "if gate {{ let local = saved;
            if first {{ {yes} }} else {{ let local = saved; if local {{}} else {{}} }}
            return local && first; }} else {{ return saved; }}"
        )))
        .unwrap();
        let observer = module
            .functions
            .iter()
            .find(|f| f.name == "observe")
            .unwrap();
        let scope = observer
            .params
            .iter()
            .map(|p| (p.name.clone(), p.ty.clone()))
            .collect();
        let signatures = BTreeMap::from([(
            "decide".into(),
            (vec![scalar_type("bool")], scalar_type("bool")),
        )]);
        let NirStmt::If { then_body, .. } = &observer.body[1] else {
            unreachable!()
        };
        let mut budget = nested::Budget {
            nodes: 64,
            expressions: 256,
            logical_edges: 32,
        };
        let staged =
            super::super::stage(&then_body[..1], &scope, &signatures, &mut budget, 1).unwrap();
        let NirStmt::If {
            condition,
            then_body: yes,
            else_body: no,
        } = &then_body[1]
        else {
            unreachable!()
        };
        let proof = prove(
            condition,
            yes,
            no,
            &staged,
            &signatures,
            &mut budget,
            1,
            None,
        )
        .unwrap();
        assert_eq!(proof.inputs.contains("local"), old_required);
        assert!(!proof.inputs.contains("before"));
        let SelectedValue::Nested(plan) = proof.value else {
            unreachable!()
        };
        assert_eq!(plan.params.iter().any(|p| p.name == "local"), old_required);
        assert_eq!(plan.destination, Some(("local".into(), false)));
    }
}

#[test]
fn return_retention_update_logical_effectful_scalar_selections_reject_fresh_or_no_write_authority_extra_targets_seals_hidden_work_and_late_leaks(
) {
    for update in [
        "if first { if decide(saved) {} else {} }",
        "if first { let child = decide(first); if child {} else {} }",
        "if first { let saved = first; if decide(saved) {} else {} }",
        "if first { let local = first; let other = saved; if decide(local) {} else {} }",
        "if first { let local = first; if saved { let other = first; } if local {} else {} }",
        "if first { const local: bool = first || false; if local {} else {} }",
        "if first { let local: i64 = 1; if first {} else {} }",
        "if first { let local = boolean(local || first); if local {} else {} }",
        "if first { let local = first; if local { return local; } else {} }",
        "if first { let local = first; while local { let local = saved; } if local {} else {} }",
        "if first { let local = first; print(96); if local {} else {} }",
        "if first { let local = first; let saved = token(1); }",
        "if first { let local = first; let child = first; let sibling = first; if child { let child = first; let local = first; let sibling = first; } }",
    ] {
        reject_module(
            parse_nuis_module(&source(&format!(
                "if decide(gate) {{ let local = saved; let other = first;
            {update} return local && first; }} else {{ return saved; }}"
            )))
            .unwrap(),
        );
    }
    reject_module(parse_nuis_module(&source("if gate { const local: bool = saved || false;
        if first { let local = first; if local {} else {} } return local && first; } else { return saved; }")).unwrap());
    let original = parse_nuis_module(&source("if gate { let local = saved;
        if first { let child = first; let local = local || decide(child); if local {} else {} }
        else { let local = saved; if local {} else {} } return local && first; } else { return saved; }")).unwrap();
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
            let NirStmt::Let { value, .. } = &mut else_body[0] else {
                unreachable!()
            };
            *value = NirExpr::Var("child".into());
        } else {
            let NirStmt::Let { ty, .. } = &mut then_body[1] else {
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
fn return_retention_update_logical_effectful_scalar_selections_charge_original_prefix_writes_unused_work_and_retention_conditions_once(
) {
    for inverted in [false, true] {
        let arm="let local = local || decide(first); let ignored = token(1); if decide(local) {} else {}";
        let update = if inverted {
            format!("if first {{}} else {{ {arm} }}")
        } else {
            format!("if first {{ {arm} }}")
        };
        let module = parse_nuis_module(&source(&format!(
            "if decide(gate) {{ let local = saved; {update}
            return local && first; }} else {{ return saved; }}"
        )))
        .unwrap();
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
            (7, 16, 2, false),
            (8, 15, 2, false),
            (8, 16, 1, false),
            (8, 16, 2, true),
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
fn return_retention_update_logical_effectful_scalar_selections_share_prefix_original_nodes_edges_depth_and_readonly_discovery_limits(
) {
    for count in [16, 17] {
        let prefix = "let local = local;".repeat(count);
        assert_eq!(!rewrite(&format!("if decide(gate) {{ let local = saved; if first {{ {prefix} if local {{}} else {{}} }}
            return local && first; }} else {{ return saved; }}")).1.is_empty(),count==16);
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
        assert_eq!(!rewrite(&format!("if decide(gate) {{ let local = saved;
            if first {{ {first} if local {{}} else {{}} }} if first {{ {second} if local {{}} else {{}} }}
            {} }} else {{ {} }}",full(3),full(3))).1.is_empty(),writes==14);
    }
    for extra in [false, true] {
        let yes = (0..16)
            .map(|n| {
                if extra && n == 15 {
                    "let local = (local || first) || saved;"
                } else {
                    "let local = local || first;"
                }
            })
            .collect::<String>();
        let no = "let local = local || first;".repeat(15);
        assert_eq!(!rewrite(&format!("if decide(gate) {{ let local = saved; if first {{ {yes} if local {{}} else {{}} }}
            else {{ {no} if local {{}} else {{}} }} return local && first; }} else {{ return saved; }}")).1.is_empty(),!extra);
    }
    for levels in [5, 6] {
        let mut child="let local = saved; if first { let local = first; if decide(local) {} else {} } return local && first;".to_string();
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
    let eligible = BTreeSet::from(["local".into()]);
    let write = NirStmt::Let {
        name: "local".into(),
        ty: None,
        value: NirExpr::Bool(false),
    };
    let mut remaining = 64;
    assert_eq!(
        retention_targets::discover(&vec![write.clone(); 64], &eligible, &mut remaining, 1),
        Some(Some("local".into()))
    );
    assert_eq!(remaining, 0);
    assert!(retention_targets::discover(&vec![write; 65], &eligible, &mut 64, 1).is_none());
    let fresh = NirStmt::Let {
        name: "child".into(),
        ty: None,
        value: NirExpr::Bool(false),
    };
    assert_eq!(
        retention_targets::discover(&[fresh], &eligible, &mut 64, 1),
        Some(None)
    );
    assert!(retention_targets::merge(Some("local".into()), Some("other".into())).is_none());
}
