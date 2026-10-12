use super::super::super::tests::{reject_module, rewrite, source};
use super::*;
use crate::frontend::parse_nuis_module;

#[test]
fn return_prefix_mixed_logical_effectful_scalar_selections_keep_both_orders_and_private_current_versions(
) {
    for updates in [
        "if decide(saved) { let local = child || decide(first); let child = local && saved; } else { let child = child && first; let local = child || saved; }",
        "if decide(saved) { let child = child || decide(first); let local = child && saved; } else { let local = child && first; let child = local || saved; }",
        "if decide(saved) { let local = first; } else { let child = child || decide(first); }",
        "if decide(saved) { let child = first; } else { let local = child && decide(first); }",
    ] {
        for inverted in [false, true] {
            let selected = format!("let local = saved; if first {{ let child = local; {updates}
                let observed = decide(child); const seal: bool = observed || first;
                let local = local && seal; let ignored = token(2);
            }} return local && decide(saved);");
            let (yes,no) = if inverted { ("return saved;",selected.as_str()) } else { (selected.as_str(),"return saved;") };
            let (module,names) = rewrite(&format!("if decide(gate) {{ {yes} }} else {{ {no} }}"));
            assert!(!names.is_empty(), "{updates}");
            crate::nir_verify::verify_nir_module(&module).unwrap();
            let observer = module.functions.iter().find(|f| f.name == "observe").unwrap();
            for statement in &observer.body {
                if let NirStmt::Let {value:NirExpr::Call {callee,..},..} = statement {
                    let helper = module.functions.iter().find(|f| &f.name == callee).unwrap();
                    assert!(!helper.params.iter().any(|p| ["local","child","observed","seal","ignored"].contains(&p.name.as_str())));
                }
            }
        }
    }
    let (module, names) = rewrite(
        "if decide(gate) { let local = saved;
        if first { let child = local; if saved {
            if decide(first) { let local = child || first; let child = local && saved; }
            else { let child = first; let local = child; }
            let local = local && decide(child);
        } else { let child = first; let local = child; } }
        return local && first; } else { return saved; }",
    );
    assert!(!names.is_empty());
    crate::nir_verify::verify_nir_module(&module).unwrap();
}

#[test]
fn return_prefix_mixed_logical_effectful_scalar_selections_reject_scope_collisions_hidden_work_and_recursive_copy(
) {
    for arm in [
        "if saved { let local = first; let child = saved; let saved = first; } let local = child;",
        "if saved { let local = first; let child = saved; const child: bool = first || false; } let local = child;",
        "if saved { let local = first; let child = saved; let shadow = token(2); } const shadow: bool = child || first; let local = child;",
        "if saved { if first { let local = first; let child = saved; let saved = first; } let local = child; } let local = child;",
        "if saved { let local = first; let child = saved; } if child { if first { let local = child; let saved = first; } }",
        "if saved { let local = first; let child = saved; } if first { if child { let local = child; let child = saved; let saved = first; } } let local = child;",
        "if saved { let local = first; let child = saved; } let local = boolean(child || first);",
        "if saved { let local = first; let child = saved; } let saved = child;",
        "if saved { let local = first; let child = saved; } print(96); let local = child;",
        "if saved { let local = first; let child = saved; } return child;",
        "if saved { let local = first; let child = saved; } while child { let local = child; }",
    ] {
        reject_module(parse_nuis_module(&source(&format!("if decide(gate) {{ let local = saved;
            if first {{ let child = local; {arm} }} return local && first; }} else {{ return saved; }}"))).unwrap());
    }
    let mut module = parse_nuis_module(&source("if decide(gate) { let local = saved;
        if first { let child = local; if saved { let local = first; let child = saved; let secret = first; }
            let local = child; } return local && first; } else { return saved; }")).unwrap();
    let observer = module
        .functions
        .iter_mut()
        .find(|f| f.name == "observe")
        .unwrap();
    let NirStmt::If { then_body, .. } = &mut observer.body[1] else {
        unreachable!()
    };
    let NirStmt::If { then_body, .. } = &mut then_body[1] else {
        unreachable!()
    };
    let NirStmt::Let { value, .. } = &mut then_body[2] else {
        unreachable!()
    };
    *value = NirExpr::Var("secret".into());
    reject_module(module);
}

fn statements(text: &str) -> Vec<NirStmt> {
    parse_nuis_module(&source(&format!(
        "let local = false; let child = false; {text}"
    )))
    .unwrap()
    .functions
    .into_iter()
    .find(|f| f.name == "observe")
    .unwrap()
    .body[3..]
        .to_vec()
}

#[test]
fn return_prefix_mixed_logical_effectful_scalar_selections_capture_old_reads_not_permission_or_discarded_child(
) {
    let scope = BTreeMap::from(
        ["local", "child", "first", "saved"].map(|s| (s.into(), scalar_type("bool"))),
    );
    for (yes, no, tail, old_child) in [
        (
            "let local = first; let child = saved;",
            "let child = first; let local = saved;",
            "let local = child;",
            false,
        ),
        (
            "let local = first; let child = saved;",
            "",
            "let local = child;",
            true,
        ),
        (
            "let local = child; let child = first;",
            "let child = first; let local = saved;",
            "let local = child;",
            true,
        ),
        (
            "let local = first; let child = saved;",
            "",
            "let child = first; let local = child;",
            false,
        ),
    ] {
        let trim = |text| {
            let mut v = statements(text);
            v.truncate(v.len() - 2);
            v
        };
        let mut budget = nested::Budget {
            nodes: 64,
            expressions: 256,
            logical_edges: 32,
        };
        let proof = prove(
            &NirExpr::Var("first".into()),
            &trim(yes),
            &trim(no),
            &trim(tail),
            &scope,
            &Target("local"),
            &Child("child".into()),
            &BTreeMap::new(),
            &mut budget,
            1,
        )
        .unwrap();
        assert_eq!(proof.inputs.contains("child"), old_child);
        assert!(!proof.inputs.contains("local"));
        let SelectedValue::Nested(plan) = proof.value else {
            unreachable!()
        };
        assert_eq!(plan.destination, Some(("local".into(), false)));
        assert_eq!(plan.params.iter().any(|p| p.name == "child"), old_child);
    }
}

#[test]
fn return_prefix_mixed_logical_effectful_scalar_selections_charge_shared_tail_once_without_new_transport_work(
) {
    for inverted in [false, true] {
        let body="let child = local; if decide(child) { let local = child || decide(saved); let child = local && first; } let local = child || local;";
        let update = if inverted {
            format!("if first {{}} else {{ {body} }}")
        } else {
            format!("if first {{ {body} }}")
        };
        let module=parse_nuis_module(&source(&format!("if decide(gate) {{ let local = saved; {update} return local && first; }} else {{ return saved; }}"))).unwrap();
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
        let NirStmt::If {
            condition,
            then_body,
            else_body,
        } = &observer.body[1]
        else {
            unreachable!()
        };
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
            let proof = super::super::super::super::prove(
                condition,
                then_body,
                else_body,
                &scope,
                &signatures,
                &BTreeSet::new(),
                &mut budget,
            );
            assert_eq!(proof.is_some(), accepted);
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
fn return_prefix_mixed_logical_effectful_scalar_selections_keep_original_prefix_and_flat_copy_limits(
) {
    for count in [15, 16] {
        let tail = "let local = child;".repeat(count);
        assert_eq!(
            !rewrite(&format!(
                "if decide(gate) {{ let local = saved; if first {{
            let child = local; if saved {{ let local = first; let child = saved; }} {tail}
        }} return local && first; }} else {{return saved;}}"
            ))
            .1
            .is_empty(),
            count == 15
        );
    }
    let scope =
        BTreeMap::from(["local", "child", "first"].map(|s| (s.into(), scalar_type("bool"))));
    let let_stmt = |name: &str| NirStmt::Let {
        name: name.into(),
        ty: None,
        value: NirExpr::Bool(false),
    };
    for count in [16, 17] {
        let tail = vec![let_stmt("local"); count];
        let mut budget = nested::Budget {
            nodes: 64,
            expressions: 256,
            logical_edges: 32,
        };
        assert_eq!(
            prove(
                &NirExpr::Var("first".into()),
                &[let_stmt("local"), let_stmt("child")],
                &[],
                &tail,
                &scope,
                &Target("local"),
                &Child("child".into()),
                &BTreeMap::new(),
                &mut budget,
                1
            )
            .is_some(),
            count == 16
        );
    }
    for count in [17, 18] {
        let mut branch = vec![let_stmt("child"); count - 1];
        branch.push(let_stmt("local"));
        let mut budget = nested::Budget {
            nodes: 64,
            expressions: 256,
            logical_edges: 32,
        };
        assert_eq!(
            prove(
                &NirExpr::Var("first".into()),
                &branch,
                &[],
                &[let_stmt("local")],
                &scope,
                &Target("local"),
                &Child("child".into()),
                &BTreeMap::new(),
                &mut budget,
                1
            )
            .is_some(),
            count == 17
        );
    }
    assert!(copy_flat(&[regions::Statement::Selection(Box::new(Plan {
        condition: predicates::Predicate::Scalar(NirExpr::Bool(true)),
        yes: SelectedValue::Expression(NirExpr::Bool(true)),
        no: SelectedValue::Expression(NirExpr::Bool(false)),
        destination: Some(("local".into(), false)),
        ty: scalar_type("bool"),
        params: vec![],
    }))])
    .is_none());
}

#[test]
fn return_prefix_mixed_logical_effectful_scalar_selections_share_nodes_edges_depth_and_capture_union(
) {
    fn full(depth: usize) -> String {
        if depth == 0 {
            "return saved && first;".into()
        } else {
            let child = full(depth - 1);
            format!("if gate {{ {child} }} else {{ {child} }}")
        }
    }
    for count in [10, 11] {
        let a = "let local = local;".repeat(10);
        let b = "let local = local;".repeat(count);
        assert_eq!(!rewrite(&format!("if decide(gate) {{ let local = saved;
            if first {{ {a} let child = local; if saved {{let local = first;let child = saved;}} let local = child; }}
            if first {{ {b} let child = local; if saved {{let local = first;let child = saved;}} let local = child; }}
            {} }} else {{ {} }}",full(3),full(3))).1.is_empty(),count==10);
    }
    for extra in [false, true] {
        let a = (0..13)
            .map(|i| {
                if extra && i == 12 {
                    "let local = (local || first) || saved;"
                } else {
                    "let local = local || first;"
                }
            })
            .collect::<String>();
        let b = "let local = local || first;".repeat(14);
        assert_eq!(!rewrite(&format!("if decide(gate) {{ let local = saved;
            if first {{ {a} let child = local; if saved {{let local = local || first;let child = child || first;}} let local = child; }}
            else {{ {b} let child = local; if saved {{let local = local || first;let child = child || first;}} let local = child; }}
            return local && first; }} else {{return saved;}}")).1.is_empty(),!extra);
    }
    for levels in [5, 6] {
        let mut arm="let local = saved; if first {let child = local; if saved {let local = first;let child = saved;} let local = child;} return local && first;".to_string();
        for _ in 0..levels {
            arm = format!("if first {{ {arm} }} else {{return saved;}}");
        }
        assert_eq!(
            !rewrite(&format!(
                "if decide(gate) {{ {arm} }} else {{return saved;}}"
            ))
            .1
            .is_empty(),
            levels == 5
        );
    }
    fn balanced(names: &[String]) -> String {
        if names.len() == 1 {
            names[0].clone()
        } else {
            let mid = names.len() / 2;
            format!(
                "({}) && ({})",
                balanced(&names[..mid]),
                balanced(&names[mid..])
            )
        }
    }
    for count in [30, 31] {
        let params = (0..count)
            .map(|i| format!(", p{i}: bool"))
            .collect::<String>();
        let value = balanced(&(0..count).map(|i| format!("p{i}")).collect::<Vec<_>>());
        let text=source(&format!("if decide(gate) {{let local = false; if first {{let child = false;
            if first {{let local = first;let child = false;}} else {{let child = false;let local = first;}}
            let local = ({value}) && child; }} else {{let local = false;}} return local && true;}} else {{return false;}}"))
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
