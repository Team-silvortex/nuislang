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

fn reject_module(mut module: NirModule) {
    let before = module.clone();
    let mut names: BTreeSet<_> = module.functions.iter().map(|f| f.name.clone()).collect();
    let before_names = names.clone();
    assert!(outline(&mut module, &mut names).is_empty());
    assert_eq!(module, before);
    assert_eq!(names, before_names);
}

#[test]
fn return_tree_logical_effectful_scalar_selections_keep_ancestor_guards_private_leaf_scopes_and_complete_original_exits(
) {
    let child = "if first || decide(saved) {
        let local = saved; let local = local && first;
        const sealed: bool = local || false; let unused = token(1);
        return sealed && decide(first);
    } else { let local = token(1); return saved || first; }";
    for opposite in ["return false;", child] {
        for inverted in [false, true] {
            let (yes, no) = if inverted {
                (opposite, child)
            } else {
                (child, opposite)
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
            assert!(!observer.body.iter().any(|s| matches!(s,
                NirStmt::Let { name, .. } | NirStmt::Const { name, .. }
                if ["local", "sealed", "unused"].contains(&name.as_str()))));
            for helper in module
                .functions
                .iter()
                .filter(|f| generated.contains(&f.name))
            {
                assert!(matches!(helper.body.first(), Some(NirStmt::If { .. })));
                assert_eq!(helper.return_type, Some(scalar_type("bool")));
                assert!(helper.params.len() <= 32);
                assert!(!helper
                    .params
                    .iter()
                    .any(|p| ["local", "sealed", "unused"].contains(&p.name.as_str())));
            }
            let child_carriers = module
                .functions
                .iter()
                .filter(|f| {
                    f.name.starts_with("__nuis_effect_call_arm")
                        && f.body.iter().skip(1).any(|stmt| {
                            matches!(stmt, NirStmt::Let { value: NirExpr::Call { callee, .. }, .. }
                        if callee.starts_with("__nuis_effect_call_arm"))
                        })
                })
                .collect::<Vec<_>>();
            assert_eq!(child_carriers.len(), if opposite == child { 2 } else { 1 });
            for carrier in child_carriers {
                assert!(carrier.body.iter().skip(1).any(|stmt| matches!(stmt,
                    NirStmt::Let { name, .. } if name.starts_with("__nuis_effect_call_gate"))));
                assert!(matches!(
                    carrier.body[carrier.body.len() - 2],
                    NirStmt::Return(Some(_))
                ));
            }
        }
    }
    // Only a descendant condition is effectful; both complete child leaves are pure.
    for body in [
        "if gate { if decide(first) { return saved && first; } else { return false; } } else { return saved; }",
        "if decide(gate) { if first { return saved && first; } else { return false; } } else { return saved; }",
        "if gate { if first { let unused = token(1); return saved && first; } else { return false; } } else { return saved; }",
    ] {
        let (module, generated) = rewrite(body);
        assert!(!generated.is_empty());
        crate::nir_verify::verify_nir_module(&module).unwrap();
    }
}

#[test]
fn return_tree_logical_effectful_scalar_selections_reject_partial_mixed_inherited_writes_exports_hidden_and_late_invalid_leaves_atomically(
) {
    for body in [
        "if gate { if first { return saved && decide(first); } } else { return saved; }",
        "if gate { if first { return saved && decide(first); } else { let saved = first; } } else { return saved; }",
        "if gate { if first { let saved = first; return saved && decide(first); } else { return saved; } } else { return saved; }",
        "if gate { let local = first; if first { let local = saved; return local && decide(first); } else { return saved; } } else { return saved; }",
        "if gate { if first { return saved && decide(first); } else { return saved; } print(96); } else { return saved; }",
        "if gate { if first { return saved && decide(first); } else { return boolean(saved || decide(first)); } } else { return saved; }",
        "if gate { if first { const local: bool = saved || first; let local = first; return saved && decide(first); } else { return saved; } } else { return saved; }",
        "if gate { if first { print(96); return saved && decide(first); } else { return saved; } } else { return saved; }",
        "if gate { if first { return saved && boolean(first); } else { return saved; } } else { return saved; }",
    ] {
        reject_module(parse_nuis_module(&source(body)).unwrap());
    }
    let original = parse_nuis_module(&source(
        "if gate { if first { let local = saved; return local && decide(first); } else { return saved; } } else { return saved; }",
    )).unwrap();
    for mutation in [
        "missing-return",
        "reference",
        "optional",
        "generic",
        "i64",
        "late-missing",
        "late-leak",
        "late-void",
        "async",
    ] {
        let mut module = original.clone();
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
                "missing-return" => observer.return_type = None,
                "reference" => observer.return_type.as_mut().unwrap().is_ref = true,
                "optional" => observer.return_type.as_mut().unwrap().is_optional = true,
                "generic" => observer
                    .return_type
                    .as_mut()
                    .unwrap()
                    .generic_args
                    .push(scalar_type("bool")),
                "i64" => observer.return_type = Some(scalar_type("i64")),
                _ => {
                    let NirStmt::If { else_body, .. } = &mut observer.body[1] else {
                        unreachable!()
                    };
                    else_body[0] = NirStmt::Return(match mutation {
                        "late-missing" => Some(NirExpr::Var("absent".into())),
                        "late-leak" => Some(NirExpr::Var("local".into())),
                        "late-void" => None,
                        _ => unreachable!(),
                    });
                }
            }
        }
        reject_module(module);
    }
}

#[test]
fn return_tree_logical_effectful_scalar_selections_charge_original_selections_prefixes_and_returns_once_across_pure_children(
) {
    let module = parse_nuis_module(&source(
        "if decide(gate) { if first { let local = saved; return local && true; } else { return false; } } else { return saved || false; }",
    )).unwrap();
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
        (5, 11, 2, false),
        (6, 10, 2, false),
        (6, 11, 1, false),
        (6, 11, 2, true),
    ] {
        let mut budget = nested::Budget {
            nodes,
            expressions,
            logical_edges,
        };
        assert_eq!(
            branch(
                condition,
                then_body,
                else_body,
                &scope,
                &signatures,
                &mut budget,
                0
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

fn balanced(values: &[String]) -> String {
    if values.len() == 1 {
        return values[0].clone();
    }
    let middle = values.len() / 2;
    format!(
        "({}) && ({})",
        balanced(&values[..middle]),
        balanced(&values[middle..])
    )
}

#[test]
fn return_tree_logical_effectful_scalar_selections_bound_complete_subtree_condition_captures_shared_work_and_selection_depth(
) {
    for depth in [8, 9] {
        let mut body = "return saved && decide(first);".to_string();
        for _ in 0..depth {
            body = format!("if gate {{ {body} }} else {{ return false; }}");
        }
        assert_eq!(!rewrite(&body).1.is_empty(), depth == 8);
    }
    for count in [31, 32] {
        let params = (0..count)
            .map(|n| format!(", p{n}: bool"))
            .collect::<String>();
        let leaves = (0..count - 1).map(|n| format!("p{n}")).collect::<Vec<_>>();
        let text = source(&format!("if decide(gate) {{ if p{} {{ return {}; }} else {{ return false; }} }} else {{ return false; }}", count-1, balanced(&leaves)))
            .replace("saved: bool)", &format!("saved: bool{params})"));
        let mut module = parse_nuis_module(&text).unwrap();
        let before = module.clone();
        let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
        let generated = outline(&mut module, &mut names);
        assert_eq!(!generated.is_empty(), count == 31);
        if count == 32 {
            assert_eq!(module, before);
        }
    }
    for count in [16, 17] {
        let prefix = (0..count)
            .map(|n| format!("let local{n} = saved;"))
            .collect::<String>();
        assert_eq!(!rewrite(&format!("if decide(gate) {{ if first {{ {prefix} return saved && first; }} else {{ return false; }} }} else {{ return saved; }}")).1.is_empty(), count == 16);
    }
    for leaves in [33, 34] {
        let value = balanced(&vec!["saved".into(); leaves]);
        assert_eq!(!rewrite(&format!("if decide(gate) {{ if first {{ return {value}; }} else {{ return false; }} }} else {{ return saved; }}")).1.is_empty(), leaves == 33);
    }
    for depth in [30, 31] {
        let mut value = "saved".to_string();
        for _ in 0..depth {
            value = format!("boolean({value})");
        }
        assert_eq!(!rewrite(&format!("if decide(gate) {{ if first {{ return {value} && first; }} else {{ return false; }} }} else {{ return saved; }}")).1.is_empty(), depth == 30);
    }
    fn full(depth: usize) -> String {
        if depth == 0 {
            "return saved && first;".into()
        } else {
            let child = full(depth - 1);
            format!("if gate {{ {child} }} else {{ {child} }}")
        }
    }
    for extra in [0, 1, 2] {
        let mut body = full(5);
        let prefix = (0..extra)
            .map(|n| format!("let local{n} = saved;"))
            .collect::<String>();
        body = body.replacen(
            "return saved && first;",
            &format!("{prefix} return saved && first;"),
            1,
        );
        body = body.replacen("if gate", "if decide(gate)", 1);
        assert_eq!(!rewrite(&body).1.is_empty(), extra <= 1); // 63, 64, 65 original nodes.
    }
    for width in [63, 64] {
        let value = balanced(&vec!["(saved == first)".into(); width]);
        // Logical edges exceed 32 here; prove expression bounds separately with
        // an exact shared budget, never by resetting it for the nested child.
        let module = parse_nuis_module(&source(&format!("if decide(gate) {{ if first {{ return {value}; }} else {{ return false; }} }} else {{ return false; }}"))).unwrap();
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
        let mut budget = nested::Budget {
            nodes: 64,
            expressions: 256,
            logical_edges: 128,
        };
        assert_eq!(
            branch(
                condition,
                then_body,
                else_body,
                &scope,
                &signatures,
                &mut budget,
                0
            )
            .is_some(),
            width == 63
        );
    }
}
