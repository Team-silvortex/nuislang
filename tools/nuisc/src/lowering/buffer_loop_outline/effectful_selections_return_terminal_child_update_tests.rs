use super::tests::{reject_module, rewrite, source};
use super::*;
use crate::frontend::parse_nuis_module;

#[test]
fn return_terminal_child_update_logical_effectful_scalar_selections_keep_child_private_and_current_ancestor_through_both_polarities(
) {
    for update in [
        "let local = first; let child = local; if saved { let child = child || decide(first); }",
        "let local = first; let child = local; if saved {} else { let child = child && decide(first); }",
        "let local = local || decide(first); let child = decide(local); const seal: bool = child || saved; let ignored = token(1); if seal { let child = child && decide(first); } else { let child = child || decide(saved); }",
        "let child = local; if saved { let local = local || decide(first); } if decide(child) { let child = first; }",
        "let local = first; let child = local; if saved { let child = child || decide(first); const seal: bool = child || false; let ignored = token(1); }",
        "let local = first; let child = local; if saved { let child = child || decide(first); if decide(child) {} else {} }",
        "let local = first; let child = local; if saved { let child = first; let grandchild = child; if decide(grandchild) { let grandchild = grandchild || decide(saved); } }",
    ] {
        for inverted in [false, true] {
            let selected = format!("let local = saved; if first {{ {update} }}
                const frozen: bool = local || false;
                if frozen {{ return local && decide(first); }} else {{ return local || saved; }}");
            let (yes,no) = if inverted { ("return saved;",selected.as_str()) }
                else { (selected.as_str(),"return saved;") };
            let (module,generated) = rewrite(&format!("if gate {{ {yes} }} else {{ {no} }}"));
            assert!(!generated.is_empty(),"{update}");
            crate::nir_verify::verify_nir_module(&module).unwrap();
            let observer=module.functions.iter().find(|f|f.name=="observe").unwrap();
            assert_eq!(observer.body[observer.body.len()-2],NirStmt::Print(NirExpr::Int(80)));
            for stmt in &observer.body {
                if let NirStmt::Let {value:NirExpr::Call{callee,..},..}=stmt {
                    let helper=module.functions.iter().find(|f| &f.name==callee).unwrap();
                    assert!(!helper.params.iter().any(|p|["local","child","grandchild","seal","ignored","frozen"].contains(&p.name.as_str())));
                }
            }
            assert!(module.functions.iter().filter(|f|generated.contains(&f.name))
                .all(|f|f.params.len()<=32 && matches!(f.body.first(),Some(NirStmt::If{..}))));
        }
    }
}

#[test]
fn return_terminal_child_update_logical_effectful_scalar_selections_capture_old_reads_and_retained_parent_not_child_exports(
) {
    for (prefix, old_required) in [
        ("let local = first; let child = local;", false),
        ("let child = local; let local = first;", true),
        ("if saved { let local = first; } let child = local;", true),
        (
            "let before = decide(local); let local = first; let child = local;",
            true,
        ),
    ] {
        let module = parse_nuis_module(&source(&format!(
            "if gate {{ let local = saved;
            if first {{ {prefix} if saved {{ let child = child || decide(first); }} }} else {{ let local = first; }}
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
        assert_eq!(proof.name, "local");
        assert_eq!(proof.inputs.contains("local"), old_required);
        assert!(!proof
            .inputs
            .iter()
            .any(|n| ["child", "before"].contains(&n.as_str())));
        let SelectedValue::Nested(plan) = proof.value else {
            unreachable!()
        };
        assert_eq!(plan.params.iter().any(|p| p.name == "local"), old_required);
        assert!(!plan.params.iter().any(|p| p.name == "child"));
        assert_eq!(plan.destination, Some(("local".into(), false)));
    }
}

#[test]
fn return_terminal_child_update_logical_effectful_scalar_selections_reject_no_ancestor_write_incoming_sealed_dual_targets_hidden_work_and_late_leaks(
) {
    for body in [
        "let child = first; if saved { let child = decide(first); }",
        "let local = first; if saved { let saved = first; }",
        "let local = first; if saved { let other = first; }",
        "let local = first; let child = first; let sibling = first; if saved { let child = first; let local = first; let sibling = first; }",
        "let local = first; let child = first; let sibling = first; if saved { let child = first; } else { let local = first; let sibling = first; }",
        "let local = first; let child = first; let second = first; if saved { let child = first; } else { let second = first; }",
        "let local = first; const child: bool = first || false; if saved { let child = first; }",
        "let local = first; let child = 1; if saved { let child = token(2); }",
        "let local = first; let child = first; if saved { const child: bool = first || false; }",
        "let local = first; let child = first; if saved { let child = boolean(child || first); }",
        "let local = first; let child = first; if saved { let child = first; print(96); }",
        "let local = first; let child = first; if saved { return child; }",
        "let local = first; let child = first; if saved { while first { let child = first; } }",
    ] {
        reject_module(parse_nuis_module(&source(&format!("if decide(gate) {{
            let local = saved; let other = first; if first {{ {body} }}
            return local && first; }} else {{ return saved; }}"))).unwrap());
    }
    let original = parse_nuis_module(&source(
        "if gate { let local = saved;
        if first { let local = first; let child = first; if saved { let child = decide(first); } }
        else { let local = saved; } return local && first; } else { return saved; }",
    ))
    .unwrap();
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
fn return_terminal_child_update_logical_effectful_scalar_selections_charge_original_child_work_once_without_parent_transport_work(
) {
    for inverted in [false, true] {
        let arm="let local = first; let child = local; if decide(child) { let child = child || decide(saved); }";
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
        let signatures = BTreeMap::from([(
            "decide".into(),
            (vec![scalar_type("bool")], scalar_type("bool")),
        )]);
        for (nodes, expressions, logical_edges, accepted) in [
            (8, 16, 2, false),
            (9, 15, 2, false),
            (9, 16, 1, false),
            (9, 16, 2, true),
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
fn return_terminal_child_update_logical_effectful_scalar_selections_keep_prefix_nodes_edges_depth_and_readonly_classification_bounds(
) {
    for writes in [15, 16] {
        let prefix = "let local = local;".repeat(writes);
        assert_eq!(
            !rewrite(&format!(
                "if decide(gate) {{ let local = saved;
            if first {{ {prefix} let child = first; if saved {{ let child = first; }} }}
            return local && first; }} else {{ return saved; }}"
            ))
            .1
            .is_empty(),
            writes == 15
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
    for writes in [12, 13] {
        let first = "let local = local;".repeat(12);
        let second = "let local = local;".repeat(writes);
        assert_eq!(
            !rewrite(&format!(
                "if decide(gate) {{ let local = saved;
            if first {{ {first} let child = first; if saved {{ let child = first; }} }}
            if first {{ {second} let child = first; if saved {{ let child = first; }} }}
            {} }} else {{ {} }}",
                full(3),
                full(3)
            ))
            .1
            .is_empty(),
            writes == 12
        );
    }
    for extra in [false, true] {
        let yes = (0..14)
            .map(|n| {
                if extra && n == 13 {
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
            if first {{ {yes} let child = first; if saved {{ let child = child || first; }} }}
            else {{ {no} let child = first; if saved {{ let child = child || first; }} }}
            return local && first; }} else {{ return saved; }}"
            ))
            .1
            .is_empty(),
            !extra
        );
    }
    for levels in [5, 6] {
        let mut child="let local = saved; if first { let local = first; let child = first; if saved { let child = first; } } return local && first;".to_string();
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
    let scope = BTreeMap::from([("local".into(), scalar_type("bool"))]);
    let staged = regions::Staging {
        local: BTreeMap::from([
            ("local".into(), scalar_type("bool")),
            ("child".into(), scalar_type("bool")),
        ]),
        defined: BTreeSet::from(["child".into()]),
        inputs: BTreeSet::new(),
        calls: BTreeSet::new(),
        constants: BTreeSet::new(),
        statements: vec![],
    };
    let write = NirStmt::Let {
        name: "child".into(),
        ty: None,
        value: NirExpr::Bool(false),
    };
    assert_eq!(
        terminal_private_child(
            &vec![write.clone(); 64],
            &[],
            &scope,
            &staged,
            &Target("local"),
            1
        ),
        Some(true)
    );
    assert!(
        terminal_private_child(&vec![write; 65], &[], &scope, &staged, &Target("local"), 1)
            .is_none()
    );
    let entering = staged.local.clone();
    let write = NirStmt::Let {
        name: "child".into(),
        ty: None,
        value: NirExpr::Bool(false),
    };
    assert_eq!(
        terminal_private_child(&[write], &[], &entering, &staged, &Target("local"), 1),
        Some(false)
    );
}

#[test]
fn return_terminal_child_update_logical_effectful_scalar_selections_share_complete_external_capture_union(
) {
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
        let value = balanced(&(0..count).map(|n| format!("p{n}")).collect::<Vec<_>>());
        let text=source(&format!("if decide(gate) {{ let local = false;
            if first {{ let local = false; let child = false; if first {{ let child = ({value}); }} else {{ let child = false; }} }} else {{ let local = false; }}
            return local && true; }} else {{ return false; }}"))
            .replace("saved: bool)",&format!("saved: bool{params})"));
        let mut module = parse_nuis_module(&text).unwrap();
        let before = module.clone();
        let mut names: BTreeSet<_> = module.functions.iter().map(|f| f.name.clone()).collect();
        let before_names = names.clone();
        assert_eq!(!outline(&mut module, &mut names).is_empty(), count == 30);
        if count == 31 {
            assert_eq!(module, before);
            assert_eq!(names, before_names);
        }
    }
}
