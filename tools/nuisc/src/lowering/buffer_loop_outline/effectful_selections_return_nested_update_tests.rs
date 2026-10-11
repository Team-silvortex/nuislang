use super::tests::{reject_module, rewrite, source};
use super::*;
use crate::frontend::parse_nuis_module;

#[test]
fn return_nested_update_logical_effectful_scalar_selections_keep_private_stages_current_target_and_nested_retention(
) {
    for update in [
        "if first { if saved { let local = local || decide(first); } }",
        "if first {} else { if saved {} else { let local = local && decide(first); } }",
        "if first { let child = local || first; const seal: bool = child && saved;
            let ignored = token(1); if seal { let local = local && decide(child); }
            else { let local = local || decide(saved); } } else { let local = saved; }",
        "if first { let child = local; if decide(saved) { let child = child || first; }
            const seal: bool = child && saved; let ignored = token(1);
            if seal { let local = local && decide(child); } }",
        "if first { if decide(saved) {} else {} } else { let local = first; }",
    ] {
        for inverted in [false, true] {
            let selected = format!(
                "let local = saved; {update} const frozen: bool = local || false;
                if frozen {{ return local && decide(first); }} else {{ return local || first; }}"
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
            // A private staging value may be captured by its descendants only.
            for helper in module
                .functions
                .iter()
                .filter(|f| f.params.iter().any(|p| p.name == "child"))
            {
                assert!(generated.contains(&helper.name));
            }
        }
    }
}

#[test]
fn return_nested_update_logical_effectful_scalar_selections_reject_inherited_sealed_mixed_hidden_and_child_exports_atomically(
) {
    for update in [
        "if first { if saved { let saved = decide(first); } }",
        "if first { let other = first; if saved { let local = decide(first); } }",
        "if first { const local: bool = first || false; if saved { let local = first; } }",
        "if first { let child = first; if saved { let child = decide(first); } }",
        "if first { if saved { let local = first; } else { let other = saved; } }",
        "if first { const child: bool = first || false; if saved { let child = first; } let local = child; }",
        "if first { if saved { const local: bool = first || false; } }",
        "if first { if saved { let local: i64 = 1; } }",
        "if first { if saved { let local = boolean(local || first); } }",
        "if first { if saved { return local; } }",
        "if first { let unused = token(1); if saved { let local = first; } else { return false; } }",
        "if first { if saved { while first { let local = first; } } }",
        "if first { if saved { print(96); } }",
        "if first { if decide(saved) {} else {} }",
    ] {
        reject_module(parse_nuis_module(&source(&format!("if decide(gate) {{ let local = saved; let other = first; {update} return local && first; }} else {{ return saved; }}"))).unwrap());
    }
    reject_module(parse_nuis_module(&source("if gate { const local: bool = saved || false; if first { if decide(saved) { let local = first; } } return local && first; } else { return saved; }")).unwrap());
    let mut module = parse_nuis_module(&source("if gate { let local = saved; if first { let child = saved; if decide(saved) { let local = child; } } return local && first; } else { return saved; }")).unwrap();
    let observer = module
        .functions
        .iter_mut()
        .find(|f| f.name == "observe")
        .unwrap();
    let NirStmt::If { then_body, .. } = &mut observer.body[1] else {
        unreachable!()
    };
    // Even a late malformed NIR sibling read must leave the module and names intact.
    let NirStmt::If { else_body, .. } = &mut then_body[1] else {
        unreachable!()
    };
    else_body.push(NirStmt::Let {
        name: "local".into(),
        ty: None,
        value: NirExpr::Var("child".into()),
    });
    reject_module(module);
}

#[test]
fn return_nested_update_logical_effectful_scalar_selections_charge_each_original_condition_stage_and_leaf_once(
) {
    for inverted in [false, true] {
        let selected = "let ignored = token(1); if saved { let local = local || decide(first); }";
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
            (7, 15, 2, false),
            (8, 14, 2, false),
            (8, 15, 1, false),
            (8, 15, 2, true),
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
fn return_nested_update_logical_effectful_scalar_selections_share_child_prefix_depth_capture_and_discovery_limits(
) {
    for levels in [7, 8] {
        let mut update = "let local = local || first;".to_string();
        for _ in 0..levels {
            update = format!("if first {{ {update} }}");
        }
        assert_eq!(!rewrite(&format!("if decide(gate) {{ let local = saved; {update} return local && first; }} else {{ return saved; }}")).1.is_empty(),levels == 7);
    }
    for count in [16, 17] {
        let prefix = (0..count)
            .map(|n| format!("let child{n} = token(1);"))
            .collect::<String>();
        assert_eq!(!rewrite(&format!("if decide(gate) {{ let local = saved; if first {{ {prefix} if saved {{ let local = local || first; }} }} return local && first; }} else {{ return saved; }}")).1.is_empty(),count == 16);
    }
    fn balanced(values: &[String]) -> String {
        if values.len() == 1 {
            values[0].clone()
        } else {
            let mid = values.len() / 2;
            format!(
                "({}) && ({})",
                balanced(&values[..mid]),
                balanced(&values[mid..])
            )
        }
    }
    for count in [30, 31] {
        let params = (0..count)
            .map(|n| format!(", p{n}: bool"))
            .collect::<String>();
        let expression = balanced(&(0..count).map(|n| format!("p{n}")).collect::<Vec<_>>());
        let text = source(&format!("if decide(gate) {{ let local = false; if first {{ let child = {expression}; if child {{ let local = local && child; }} }} return local && true; }} else {{ return false; }}"))
            .replace("saved: bool)",&format!("saved: bool{params})"));
        let mut module = parse_nuis_module(&text).unwrap();
        let before = module.clone();
        let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
        assert_eq!(!outline(&mut module, &mut names).is_empty(), count == 30);
        if count == 31 {
            assert_eq!(module, before);
        }
    }
    let module = parse_nuis_module(&source("if gate { let local = saved; if first { if saved { let local = first; } } return local && decide(first); } else { return saved; }")).unwrap();
    let observer = module
        .functions
        .iter()
        .find(|f| f.name == "observe")
        .unwrap();
    let NirStmt::If { then_body, .. } = &observer.body[1] else {
        unreachable!()
    };
    let NirStmt::If { then_body, .. } = &then_body[1] else {
        unreachable!()
    };
    let eligible = BTreeSet::from(["local".into()]);
    assert!(target(then_body, &eligible, &mut 2, 2).is_some());
    assert!(target(then_body, &eligible, &mut 1, 2).is_none());
}
