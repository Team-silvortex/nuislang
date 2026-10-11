use super::tests::{reject_module, rewrite, source};
use super::*;
use crate::frontend::parse_nuis_module;

#[test]
fn return_trailing_update_logical_effectful_scalar_selections_keep_final_private_let_const_unused_work_and_complete_returns(
) {
    for tail in [
        "let ignored = token(1);",
        "let child = decide(local);",
        "let child = local; let child = decide(child);",
        "const seal: bool = local || decide(first);",
        "let child = local; if saved { let child = child || first; } const seal: bool = child && decide(local);",
        "let child = decide(local); const seal: bool = child || false; let ignored = token(1);",
        "if decide(local) { if saved {} else {} } else {} let ignored = token(1);",
        "if decide(local) { let child = decide(first); } else { const seal: bool = local || false; } let ignored = token(1);",
    ] {
        for inverted in [false, true] {
            let selected = format!("let local = saved;
                if first {{ if saved {{ let local = local || decide(first); }} let local = local && first; {tail} }}
                else {{ let local = first; const final_seal: bool = local || saved; }}
                if local {{ return local && decide(first); }} else {{ return local || saved; }}");
            let (yes, no) = if inverted { ("return saved;", selected.as_str()) }
                else { (selected.as_str(), "return saved;") };
            let (module, generated) = rewrite(&format!("if gate {{ {yes} }} else {{ {no} }}"));
            assert!(!generated.is_empty(), "{tail}");
            crate::nir_verify::verify_nir_module(&module).unwrap();
            let observer = module.functions.iter().find(|f| f.name == "observe").unwrap();
            assert_eq!(observer.body[observer.body.len() - 2], NirStmt::Print(NirExpr::Int(80)));
            for stmt in &observer.body {
                if let NirStmt::Let { value: NirExpr::Call { callee, .. }, .. } = stmt {
                    let helper = module.functions.iter().find(|f| &f.name == callee).unwrap();
                    assert!(!helper.params.iter().any(|p| ["local", "child", "ignored", "seal", "final_seal"].contains(&p.name.as_str())));
                }
            }
            assert!(module.functions.iter().filter(|f| generated.contains(&f.name))
                .all(|f| f.params.len() <= 32 && matches!(f.body.first(), Some(NirStmt::If { .. }))));
        }
    }
}

#[test]
fn return_trailing_update_logical_effectful_scalar_selections_keep_old_reads_definite_replacements_and_private_tail_captures(
) {
    for (body, old_required) in [
        (
            "let local = local || decide(first); let ignored = token(1);",
            true,
        ),
        ("let local = first; let child = decide(local);", false),
        (
            "let before = decide(local); let local = first; const seal: bool = local || first;",
            true,
        ),
        (
            "if saved { let local = first; } let child = decide(local);",
            true,
        ),
    ] {
        let module = parse_nuis_module(&source(&format!(
            "if gate {{ let local = saved;
            if first {{ {body} }} else {{ let local = saved; let ignored = token(1); }}
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
        assert!(!proof
            .inputs
            .iter()
            .any(|n| ["before", "child", "ignored", "seal"].contains(&n.as_str())));
        let SelectedValue::Nested(plan) = proof.value else {
            unreachable!()
        };
        assert_eq!(plan.params.iter().any(|p| p.name == "local"), old_required);
        assert_eq!(plan.destination, Some(("local".into(), false)));
    }
}

#[test]
fn return_trailing_update_logical_effectful_scalar_selections_reject_no_write_extra_targets_final_seal_mutation_hidden_work_and_late_leaks(
) {
    for body in [
        "let child = decide(local);",
        "const seal: bool = local || first;",
        "let local = first; let saved = first;",
        "let local = first; let other = first;",
        "let local = first; const local: bool = local || false;",
        "let local: i64 = 1; let ignored = token(1);",
        "let local = first; const seal: bool = first || false; let seal = first;",
        "let local = first; const seal: bool = first || false; const seal: bool = first || false;",
        "let local = first; let ignored = boolean(first || saved);",
        "let local = first; let child = first; if child { let child = first; let local = first; }",
        "let local = first; print(96);",
        "let local = first; return local;",
        "let local = first; while first { let local = saved; }",
    ] {
        reject_module(
            parse_nuis_module(&source(&format!(
                "if decide(gate) {{
            let local = saved; let other = first; if first {{ {body} }}
            return local && first; }} else {{ return saved; }}"
            )))
            .unwrap(),
        );
    }
    assert!(parse_nuis_module(&source(
        "if gate { let local = saved;
        if first { let local = first; let ignored = child; }
        return local && first; } else { return saved; }"
    ))
    .is_err());
    assert!(parse_nuis_module(&source(
        "if gate { let local = saved;
        if first { let local = first; const seal: bool = token(1); }
        return local && first; } else { return saved; }"
    ))
    .is_err());
    reject_module(parse_nuis_module(&source("if gate { const local: bool = saved || false;
        if first { let local = first; let ignored = token(1); } return local && first; } else { return saved; }")).unwrap());
    let original = parse_nuis_module(&source("if gate { let local = saved;
        if first { let local = first; let child = decide(local); }
        else { let local = saved; let ignored = token(1); } return local && first; } else { return saved; }")).unwrap();
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
            let NirStmt::Let { value, .. } = &mut else_body[1] else {
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
fn return_trailing_update_logical_effectful_scalar_selections_charge_final_original_work_once_without_invented_retention_work(
) {
    for inverted in [false, true] {
        let arm = "let local = local || decide(first); let ignored = token(1);";
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
            (6, 14, 2, false),
            (7, 13, 2, false),
            (7, 14, 1, false),
            (7, 14, 2, true),
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
fn return_trailing_update_logical_effectful_scalar_selections_keep_original_prefix_nodes_edges_depth_and_discovery_bounds(
) {
    for count in [16, 17] {
        let prefix = "let local = local;".repeat(count);
        assert_eq!(
            !rewrite(&format!(
                "if decide(gate) {{ let local = saved;
            if first {{ {prefix} let ignored = token(1); }}
            return local && first; }} else {{ return saved; }}"
            ))
            .1
            .is_empty(),
            count == 16
        );
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
        assert_eq!(
            !rewrite(&format!(
                "if decide(gate) {{ let local = saved;
            if first {{ {first} let tail = first; }} if first {{ {second} let tail = first; }}
            {} }} else {{ {} }}",
                full(3),
                full(3)
            ))
            .1
            .is_empty(),
            writes == 14
        );
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
        assert_eq!(
            !rewrite(&format!(
                "if decide(gate) {{ let local = saved;
            if first {{ {yes} let tail = first; }} else {{ {no} let tail = first; }}
            return local && first; }} else {{ return saved; }}"
            ))
            .1
            .is_empty(),
            !extra
        );
    }
    for levels in [6, 7] {
        let mut child = "let local = saved; if first { let local = first; let tail = first; } return local && first;".to_string();
        for _ in 0..levels {
            child = format!("if first {{ {child} }} else {{ return saved; }}");
        }
        assert_eq!(
            !rewrite(&format!(
                "if decide(gate) {{ {child} }} else {{ return saved; }}"
            ))
            .1
            .is_empty(),
            levels == 6
        );
    }
    let eligible = BTreeSet::from(["local".into()]);
    let fresh = NirStmt::Let {
        name: "child".into(),
        ty: None,
        value: NirExpr::Bool(false),
    };
    let seal = NirStmt::Const {
        name: "seal".into(),
        ty: scalar_type("bool"),
        value: NirExpr::Bool(false),
    };
    let mut remaining = 64;
    assert_eq!(
        target(&vec![fresh.clone(); 64], &eligible, &mut remaining, 1),
        None
    );
    assert_eq!(
        retention_targets::discover(&vec![fresh.clone(); 64], &eligible, &mut 64, 1),
        Some(None)
    );
    assert!(retention_targets::discover(&vec![fresh; 65], &eligible, &mut 64, 1).is_none());
    assert_eq!(target(&[seal], &eligible, &mut 64, 1), Some(None));
}
