use super::super::super::super::tests::{reject_module, rewrite, source};
use super::*;
use crate::frontend::parse_nuis_module;

fn scope() -> Scope {
    ["local", "child", "first", "saved"]
        .map(|n| (n.into(), scalar_type("bool")))
        .into()
}

fn statements(text: &str) -> Vec<NirStmt> {
    let module = parse_nuis_module(&source(&format!(
        "let local = false; let child = false; {text}"
    )))
    .unwrap();
    let body = &module
        .functions
        .iter()
        .find(|f| f.name == "observe")
        .unwrap()
        .body;
    body[3..body.len() - 2].to_vec()
}

fn budget() -> nested::Budget {
    nested::Budget {
        nodes: 64,
        expressions: 256,
        logical_edges: 32,
    }
}

fn proof(yes: &str, no: &str, tail: &str, budget: &mut nested::Budget) -> Option<nested::Proof> {
    prove(
        &NirExpr::Var("first".into()),
        &statements(yes),
        &statements(no),
        &statements(tail),
        &scope(),
        &Target("local"),
        &Child("child".into()),
        &BTreeMap::from([(
            "decide".into(),
            (vec![scalar_type("bool")], scalar_type("bool")),
        )]),
        budget,
        0,
    )
}

#[test]
fn return_sequential_mixed_logical_effectful_scalar_selections_keep_both_current_versions_orders_and_retention(
) {
    for first in [
        "if saved {let child = child || decide(first);let local = child && first;} else {let local = child;}",
        "if saved {let local = child || decide(first);let child = local && saved;}",
    ] {
        for inverted in [false, true] {
            let selected = format!("let local = saved; if first {{let child = local; {first}
                if child {{let local = local && decide(child);let child = local || first;}}
                else {{let child = child || decide(saved);let local = child && saved;}}
                if local {{let child = first;}} const seal: bool = child || saved;
                let local = local && seal; let ignored = token(2);
            }} return local && decide(saved);");
            let (yes, no) = if inverted { ("return saved;", selected.as_str()) } else { (selected.as_str(), "return saved;") };
            let (module, names) = rewrite(&format!("if decide(gate) {{ {yes} }} else {{ {no} }}"));
            assert!(!names.is_empty(), "{first}");
            crate::nir_verify::verify_nir_module(&module).unwrap();
            let observer = module.functions.iter().find(|f| f.name == "observe").unwrap();
            for stmt in &observer.body {
                if let NirStmt::Let { value: NirExpr::Call { callee, .. }, .. } = stmt {
                    let helper = module.functions.iter().find(|f| &f.name == callee).unwrap();
                    assert!(!helper.params.iter().any(|p| ["local", "child", "seal", "ignored"].contains(&p.name.as_str())));
                    assert_eq!(helper.return_type, Some(scalar_type("bool")));
                }
            }
        }
    }
}

#[test]
fn return_sequential_mixed_logical_effectful_scalar_selections_reject_scope_seals_extra_writes_nested_sequences_and_hidden_work_atomically(
) {
    for tail in [
        "if child {let saved = child;} let local = child;",
        "if child {const child: bool = first || saved;} let local = child;",
        "const sealed: bool = child || first; if child {let sealed = saved;} let local = child;",
        "if child {if first {let local = first;let child = saved;let saved = first;}} let local = child;",
        "if boolean(child || first) {let local = first;} let local = child;",
        "if child {let secret = first;} let local = boolean(child || first);",
        "if child {let local = first;print(96);} let local = child;",
        "if child {return first;} let local = child;",
        "if child {let clash = token(2);} const clash: bool = child || first; let local = child;",
    ] {
        reject_module(
            parse_nuis_module(&source(&format!(
                "if decide(gate) {{let local = saved;
            if first {{let child = local;if saved {{let local = first;let child = saved;}} {tail}}}
            return local && first;}} else {{return saved;}}"
            )))
            .unwrap(),
        );
    }
    // Source parser scope checks are not the outliner's authorization proof.
    let mut tail = statements("if child {let local = first;} let local = child;");
    if let NirStmt::Let { value, .. } = &mut tail[1] {
        *value = NirExpr::Var("secret".into());
    }
    assert!(prove(
        &NirExpr::Bool(true),
        &statements("let local = first;let child = saved;let secret = first;"),
        &[],
        &tail,
        &scope(),
        &Target("local"),
        &Child("child".into()),
        &BTreeMap::new(),
        &mut budget(),
        0
    )
    .is_none());
}

#[test]
fn return_sequential_mixed_logical_effectful_scalar_selections_charge_original_source_once_and_keep_exact_adjacent_budgets(
) {
    for (nodes, expressions, accepted) in [(7, 7, true), (6, 7, false), (7, 6, false)] {
        let mut b = nested::Budget {
            nodes,
            expressions,
            logical_edges: 0,
        };
        let p = proof(
            "let local = first;let child = saved;",
            "",
            "if child {let child = first;let local = child;} let local = child;",
            &mut b,
        );
        assert_eq!(p.is_some(), accepted);
        if accepted {
            assert_eq!((b.nodes, b.expressions, b.logical_edges), (0, 0, 0));
        }
    }
    for edges in [1, 2] {
        let mut b = nested::Budget {
            logical_edges: edges,
            ..budget()
        };
        assert_eq!(
            proof(
                "let local = child && first;let child = saved;",
                "",
                "if child {let local = local || saved;} let local = child;",
                &mut b
            )
            .is_some(),
            edges == 2
        );
    }
    for count in [16, 17] {
        let arm = "let local = first;let child = saved;".to_owned()
            + &"let fresh = first;".repeat(count - 2);
        assert_eq!(
            proof(
                &arm,
                "",
                "if child {let local = first;} let local = child;",
                &mut budget()
            )
            .is_some(),
            count == 16
        );
        let tail =
            "if child {let local = first;}".to_owned() + &"let local = child;".repeat(count - 1);
        assert_eq!(
            proof(
                "let local = first;let child = saved;",
                "",
                &tail,
                &mut budget()
            )
            .is_some(),
            count == 16
        );
    }
    for count in [64, 65] {
        let arm =
            "let local = first;let child = saved;".to_owned() + &"let fresh = first;".repeat(14);
        let tail = format!(
            "if child {{{}}} else {{{}}} let local = child;",
            "let local = first;".repeat(15),
            "let child = saved;".repeat(count - 50)
        );
        assert_eq!(
            proof(&arm, &arm, &tail, &mut budget()).is_some(),
            count == 64
        );
    }
    for count in [32, 33] {
        let yes = format!(
            "let local = child{};let child = saved;",
            " && first".repeat(count - 16)
        );
        let tail = format!(
            "if child {{let local = local{};}} let local = child;",
            " || saved".repeat(16)
        );
        assert_eq!(proof(&yes, "", &tail, &mut budget()).is_some(), count == 32);
    }
    for count in [256, 257] {
        let mut args = vec![NirExpr::Bool(true); 31];
        for (index, arg) in args.iter_mut().enumerate() {
            for _ in 0..(7 + usize::from(index < count - 255)) {
                *arg = NirExpr::Call {
                    callee: "decide".into(),
                    args: vec![arg.clone()],
                };
            }
        }
        let mut yes = statements("let local = first;let child = saved;");
        if let NirStmt::Let { value, .. } = &mut yes[0] {
            *value = NirExpr::Call {
                callee: "wide".into(),
                args,
            };
        }
        let signatures = BTreeMap::from([
            (
                "wide".into(),
                (vec![scalar_type("bool"); 31], scalar_type("bool")),
            ),
            (
                "decide".into(),
                (vec![scalar_type("bool")], scalar_type("bool")),
            ),
        ]);
        assert_eq!(
            prove(
                &NirExpr::Var("first".into()),
                &yes,
                &[],
                &statements("if child {let child = first;let local = child;} let local = child;"),
                &scope(),
                &Target("local"),
                &Child("child".into()),
                &signatures,
                &mut budget(),
                0
            )
            .is_some(),
            count == 256
        );
    }
    for depth in [7, 8] {
        assert_eq!(
            prove(
                &NirExpr::Bool(true),
                &statements("let local = first;let child = saved;"),
                &[],
                &statements("if child {let local = first;}"),
                &scope(),
                &Target("local"),
                &Child("child".into()),
                &BTreeMap::new(),
                &mut budget(),
                depth
            )
            .is_some(),
            depth == 7
        );
    }
}

#[test]
fn return_sequential_mixed_logical_effectful_scalar_selections_transport_all_four_pairs_with_fresh_names_and_no_traps(
) {
    fn evaluate(expr: &NirExpr, owner: bool, child: bool, carrier: i64) -> i64 {
        match expr {
            NirExpr::Int(v) => *v,
            NirExpr::Var(n) if n == "local" => i64::from(owner),
            NirExpr::Var(n) if n == "child" => i64::from(child),
            NirExpr::Var(_) => carrier,
            NirExpr::CastBoolToI64(v) => evaluate(v, owner, child, carrier),
            NirExpr::Binary { op, lhs, rhs } => {
                let a = evaluate(lhs, owner, child, carrier);
                let b = evaluate(rhs, owner, child, carrier);
                match op {
                    NirBinaryOp::Add => a + b,
                    NirBinaryOp::Mul => a * b,
                    NirBinaryOp::Eq => i64::from(a == b),
                    NirBinaryOp::Ge => i64::from(a >= b),
                    NirBinaryOp::Xor => a ^ b,
                    _ => panic!("non-closed recipe"),
                }
            }
            _ => panic!("non-closed recipe"),
        }
    }
    let p = proof(
        "let local = first;let child = saved;",
        "",
        "if child {let local = child;} let local = child;",
        &mut budget(),
    )
    .unwrap();
    let SelectedValue::Region(statements, _) = p.value else {
        unreachable!()
    };
    let mut bindings = BTreeSet::from(["__nuis_effect_call_pair_0".into()]);
    let mut helpers = Vec::new();
    let output = regions::install(
        statements,
        &mut BTreeSet::new(),
        &mut bindings,
        &mut helpers,
        &mut Vec::new(),
    );
    let packs = helpers
        .iter()
        .filter_map(|f| match f.body.last()? {
            NirStmt::Return(Some(
                value @ NirExpr::Binary {
                    op: NirBinaryOp::Add,
                    ..
                },
            )) => Some(value),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(packs.len(), 4);
    let decodes = output
        .iter()
        .filter_map(|s| match s {
            NirStmt::Let { name, value, .. } if name == "local" || name == "child" => {
                Some((name, value))
            }
            _ => None,
        })
        .take(2)
        .collect::<Vec<_>>();
    for owner in [false, true] {
        for child in [false, true] {
            let code = i64::from(owner) + 2 * i64::from(child);
            for expr in &packs {
                assert_eq!(evaluate(expr, owner, child, 0), code);
            }
            assert_eq!(evaluate(decodes[0].1, owner, child, code), i64::from(owner));
            assert_eq!(evaluate(decodes[1].1, owner, child, code), i64::from(child));
        }
    }
    assert!(!output
        .iter()
        .any(|s| matches!(s,NirStmt::Let { name,.. } if name == "__nuis_effect_call_pair_0")));
}

#[test]
fn return_sequential_mixed_logical_effectful_scalar_selections_bound_retained_capture_union_without_permission_versions(
) {
    let p = proof(
        "let local = first;let child = saved;",
        "",
        "if child {let local = child;} let local = child;",
        &mut budget(),
    )
    .unwrap();
    assert!(p.inputs.contains("local") && p.inputs.contains("child"));
    let p = proof(
        "let local = first;let child = saved;",
        "let local = saved;let child = first;",
        "if child {let local = child;} let local = child;",
        &mut budget(),
    )
    .unwrap();
    assert!(!p.inputs.contains("local") && !p.inputs.contains("child"));
    for count in [31, 32] {
        let names = (0..count).map(|i| format!("p{i}")).collect::<Vec<_>>();
        let mut s = scope();
        s.extend(names.iter().map(|n| (n.clone(), scalar_type("bool"))));
        let arms = vec![
            NirStmt::Let {
                name: "local".into(),
                ty: None,
                value: NirExpr::Call {
                    callee: "all".into(),
                    args: names.iter().map(|n| NirExpr::Var(n.clone())).collect(),
                },
            },
            NirStmt::Let {
                name: "child".into(),
                ty: None,
                value: NirExpr::Bool(true),
            },
        ];
        let signatures = BTreeMap::from([(
            "all".into(),
            (vec![scalar_type("bool"); count], scalar_type("bool")),
        )]);
        assert_eq!(
            prove(
                &NirExpr::Bool(true),
                &arms,
                &arms,
                &statements("if child {let local = child;} let local = child;"),
                &s,
                &Target("local"),
                &Child("child".into()),
                &signatures,
                &mut budget(),
                0
            )
            .is_some(),
            count == 31
        );
    }
}

#[test]
fn return_sequential_mixed_logical_effectful_scalar_selections_install_linear_plans_without_remaining_tree_copies(
) {
    for count in [1, 8, 16] {
        let tail = "if child {let child = first;let local = child;}".repeat(count);
        let p = proof(
            "let local = first;let child = saved;",
            "",
            &tail,
            &mut budget(),
        )
        .unwrap();
        let SelectedValue::Region(statements, _) = p.value else {
            unreachable!()
        };
        assert_eq!(statements.len(), count + 1);
        assert!(statements
            .iter()
            .all(|s| matches!(s, regions::Statement::PrivatePair(_))));
        assert!(copy_flat(&statements).is_none());
        let mut helpers = Vec::new();
        regions::install(
            statements,
            &mut BTreeSet::new(),
            &mut BTreeSet::new(),
            &mut helpers,
            &mut Vec::new(),
        );
        assert_eq!(helpers.len(), 2 * (count + 1));
    }
}
