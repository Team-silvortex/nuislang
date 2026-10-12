use super::super::tests::{reject_module, rewrite, source};
use super::*;
use crate::frontend::parse_nuis_module;

#[test]
fn return_mixed_update_logical_effectful_scalar_selections_keep_one_ancestor_export_both_orders_polarities_and_terminal_nesting(
) {
    for selected in [
        "if saved { let local = child || decide(first); let child = local && decide(saved); }",
        "if saved { let child = child || decide(first); let local = child && decide(saved); }",
        "if saved { let local = first; } else { let child = child || decide(first); }",
        "if saved { let child = first; } else { let local = child && decide(first); }",
        "if saved { let local = child || decide(first); let child = local && saved; const seal: bool = child || first; let ignored = token(2); } else { let child = decide(first); let local = child || saved; }",
        "if saved { if first { let child = child || decide(saved); } let local = child && decide(first); } else { let local = first; let child = local || decide(saved); }",
        "if saved { let child = first; if decide(child) { let local = child || saved; let child = local && first; } else { let child = saved; } } else { let local = first; }",
    ] {
        for inverted in [false, true] {
            let arm=format!("let local = saved; if first {{ let child = local; {selected} }}
                const frozen: bool = local || false;
                if frozen {{ return local && decide(first); }} else {{ return local || saved; }}");
            let (yes,no)=if inverted {("return saved;",arm.as_str())} else {(arm.as_str(),"return saved;")};
            let (module, generated)=rewrite(&format!("if gate {{ {yes} }} else {{ {no} }}"));
            assert!(!generated.is_empty(), "{selected}");
            crate::nir_verify::verify_nir_module(&module).unwrap();
            let observer=module.functions.iter().find(|f|f.name=="observe").unwrap();
            assert_eq!(observer.body[observer.body.len()-2], NirStmt::Print(NirExpr::Int(80)));
            for stmt in &observer.body {
                if let NirStmt::Let {value:NirExpr::Call{callee,..},..}=stmt {
                    let helper=module.functions.iter().find(|f|&f.name==callee).unwrap();
                    assert!(!helper.params.iter().any(|p|["local","child","seal","ignored","frozen"].contains(&p.name.as_str())));
                }
            }
            assert!(module.functions.iter().filter(|f|generated.contains(&f.name))
                .all(|f|f.params.len()<=32 && matches!(f.body.first(),Some(NirStmt::If{..}))));
        }
    }
}

#[test]
fn return_mixed_update_logical_effectful_scalar_selections_capture_source_versions_not_permission_or_private_exports(
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
        let module=parse_nuis_module(&source(&format!("if gate {{ let local = saved;
            if first {{ {prefix} if saved {{ let local = child; let child = child || decide(first); }} else {{ let child = first; }} }} else {{ let local = first; }}
            return local && first; }} else {{ return saved; }}"))).unwrap();
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
            super::super::super::stage(&then_body[..1], &scope, &signatures, &mut budget, 1)
                .unwrap();
        let NirStmt::If {
            condition,
            then_body: yes,
            else_body: no,
        } = &then_body[1]
        else {
            unreachable!()
        };
        let proof = super::super::prove(
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
fn return_mixed_update_logical_effectful_scalar_selections_reject_extra_outward_targets_seals_hidden_work_prefix_continuations_and_late_leaks(
) {
    for body in [
        "let child = first; if saved { let child = decide(first); }",
        "let child = first; if saved { let local = first; let child = first; let saved = first; }",
        "let child = first; if saved { let local = first; let child = first; let other = first; }",
        "let child = first; let sibling = first; if saved { let local = first; let child = first; let sibling = first; }",
        "const child: bool = first || false; if saved { let local = first; let child = first; }",
        "let child = 1; let sibling = 2; if saved { let local = first; let child = token(1); let sibling = token(2); }",
        "let child = first; if saved { let local = first; const child: bool = first || false; }",
        "let child = first; if saved { let local = first; let child = boolean(child || first); }",
        "let child = first; if saved { let local = first; let child = first; print(96); }",
        "let child = first; if saved { let local = first; return child; } else { let child = first; }",
        "let child = first; if saved { let local = first; while first { let child = first; } }",
        "let child = first; if saved { if first { let local = first; let child = saved; } if child { if saved { let local = child; let saved = first; } } } else { let child = first; }",
    ] {
        reject_module(parse_nuis_module(&source(&format!("if decide(gate) {{ let local = saved; let other = first;
            if first {{ {body} }} return local && first; }} else {{ return saved; }}"))).unwrap());
    }
    let original = parse_nuis_module(&source(
        "if gate { let local = saved;
        if first { let child = first; if saved { let local = first; let child = saved; } }
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
fn return_mixed_update_logical_effectful_scalar_selections_charge_original_writes_once_without_retention_transport_work(
) {
    for inverted in [false, true] {
        let arm="let child = local; if decide(child) { let local = child || decide(saved); let child = local && first; }";
        let update = if inverted {
            format!("if first {{}} else {{ {arm} }}")
        } else {
            format!("if first {{ {arm} }}")
        };
        let module=parse_nuis_module(&source(&format!("if decide(gate) {{ let local = saved; {update} return local && first; }} else {{ return saved; }}"))).unwrap();
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
            (8, 18, 3, false),
            (9, 17, 3, false),
            (9, 18, 2, false),
            (9, 18, 3, true),
        ] {
            let mut budget = nested::Budget {
                nodes,
                expressions,
                logical_edges,
            };
            assert_eq!(
                super::super::super::prove(
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
fn return_mixed_update_logical_effectful_scalar_selections_keep_shared_prefix_nodes_edges_depth_and_discovery_limits(
) {
    for writes in [15, 16] {
        let prefix = "let local = local;".repeat(writes);
        assert_eq!(!rewrite(&format!("if decide(gate) {{ let local = saved;
            if first {{ {prefix} let child = first; if saved {{ let local = first; let child = first; }} }}
            return local && first; }} else {{ return saved; }}")).1.is_empty(),writes==15);
    }
    fn full(depth: usize) -> String {
        if depth == 0 {
            "return saved && first;".into()
        } else {
            let child = full(depth - 1);
            format!("if gate {{ {child} }} else {{ {child} }}")
        }
    }
    for writes in [11, 12] {
        let first = "let local = local;".repeat(11);
        let second = "let local = local;".repeat(writes);
        assert_eq!(!rewrite(&format!("if decide(gate) {{ let local = saved;
            if first {{ {first} let child = first; if saved {{ let local = first; let child = first; }} }}
            if first {{ {second} let child = first; if saved {{ let local = first; let child = first; }} }}
            {} }} else {{ {} }}",full(3),full(3))).1.is_empty(),writes==11);
    }
    for extra in [false, true] {
        let yes = (0..13)
            .map(|n| {
                if extra && n == 12 {
                    "let local = (local || first) || saved;"
                } else {
                    "let local = local || first;"
                }
            })
            .collect::<String>();
        let no = "let local = local || first;".repeat(14);
        assert_eq!(!rewrite(&format!("if decide(gate) {{ let local = saved;
            if first {{ {yes} let child = first; if saved {{ let local = local || first; let child = child || first; }} }}
            else {{ {no} let child = first; if saved {{ let local = local || first; let child = child || first; }} }}
            return local && first; }} else {{ return saved; }}")).1.is_empty(),!extra);
    }
    for levels in [5, 6] {
        let mut child="let local = saved; if first { let child = first; if saved { let local = first; let child = first; } } return local && first;".to_string();
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
    let write = |name: &str| NirStmt::Let {
        name: name.into(),
        ty: None,
        value: NirExpr::Bool(false),
    };
    let yes = vec![write("local"); 63];
    assert!(classify(
        &yes,
        &[write("child")],
        &scope,
        &staged,
        &Target("local"),
        1
    )
    .unwrap()
    .is_some());
    assert!(classify(
        &vec![write("local"); 64],
        &[write("child")],
        &scope,
        &staged,
        &Target("local"),
        1
    )
    .is_none());
    assert!(classify(
        &[write("local")],
        &[write("child")],
        &staged.local,
        &staged,
        &Target("local"),
        1
    )
    .is_none());
}

#[test]
fn return_mixed_update_logical_effectful_scalar_selections_share_complete_external_capture_union_and_atomic_overflow(
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
            if first {{ let child = false; if first {{ let child = ({value}); let local = first; }} else {{ let local = first; let child = false; }} }} else {{ let local = false; }}
            return local && true; }} else {{ return false; }}")).replace("saved: bool)",&format!("saved: bool{params})"));
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
