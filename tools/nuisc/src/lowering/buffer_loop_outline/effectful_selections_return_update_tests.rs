use super::*;
use crate::frontend::parse_nuis_module;

pub(super) fn source(body: &str) -> String {
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

pub(super) fn rewrite(body: &str) -> (NirModule, BTreeSet<String>) {
    let mut module = parse_nuis_module(&source(body)).unwrap();
    let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
    let generated = outline(&mut module, &mut names);
    (module, generated)
}

pub(super) fn reject_module(mut module: NirModule) {
    let before = module.clone();
    let mut names: BTreeSet<_> = module.functions.iter().map(|f| f.name.clone()).collect();
    let before_names = names.clone();
    assert!(outline(&mut module, &mut names).is_empty());
    assert_eq!(module, before);
    assert_eq!(names, before_names);
}

#[test]
fn return_update_logical_effectful_scalar_selections_keep_private_current_retained_versions_seals_and_guarded_order(
) {
    let prefix = "let local = saved; let other = first;
        if decide(local) { let local = local && first; } else { let local = local || first; }
        if local { let other = other || saved; }
        if other {} else { let local = local && saved; }
        const sealed: bool = local || false; let unused = token(1);";
    for tail in [
        "return sealed && decide(other);",
        "if sealed { return local && decide(other); } else { return local || other; }",
    ] {
        for inverted in [false, true] {
            let selected = format!("{prefix} {tail}");
            let (yes, no) = if inverted {
                ("return saved;", selected.as_str())
            } else {
                (selected.as_str(), "return saved;")
            };
            let (module, generated) = rewrite(&format!("if gate {{ {yes} }} else {{ {no} }}"));
            assert!(!generated.is_empty());
            crate::nir_verify::verify_nir_module(&module).unwrap();
            let observer = module
                .functions
                .iter()
                .find(|f| f.name == "observe")
                .unwrap();
            assert!(matches!(
                observer.body[observer.body.len() - 3],
                NirStmt::Return(Some(NirExpr::Var(_)))
            ));
            assert_eq!(
                observer.body[observer.body.len() - 2],
                NirStmt::Print(NirExpr::Int(80))
            );
            for stmt in &observer.body {
                if let NirStmt::Let {
                    value: NirExpr::Call { callee, .. },
                    ..
                } = stmt
                {
                    let arm = module.functions.iter().find(|f| &f.name == callee).unwrap();
                    assert!(
                        !arm.params
                            .iter()
                            .any(|p| ["local", "other", "sealed", "unused"]
                                .contains(&p.name.as_str()))
                    );
                }
            }
            let arm = module
                .functions
                .iter()
                .find(|f| {
                    f.body
                        .iter()
                        .any(|s| matches!(s, NirStmt::Const { name, .. } if name == "sealed"))
                })
                .unwrap();
            assert!(matches!(arm.body[0], NirStmt::If { .. }));
            let seal = arm
                .body
                .iter()
                .position(|s| matches!(s, NirStmt::Const { name, .. } if name == "sealed"))
                .unwrap();
            assert_eq!(
                arm.body[..seal]
                    .iter()
                    .filter(|s| matches!(s, NirStmt::If { .. }))
                    .count(),
                4
            );
            assert!(matches!(arm.body[seal+1], NirStmt::Let { ref name, .. } if name == "unused"));
            assert!(module
                .functions
                .iter()
                .filter(|f| generated.contains(&f.name))
                .all(
                    |f| f.params.len() <= 32 && matches!(f.body.first(), Some(NirStmt::If { .. }))
                ));
        }
    }
    // Only the update condition needs to be effectful; unused private updates still run.
    assert!(!rewrite("if gate { let unused = saved; if decide(first) { let unused = first; } return saved && first; } else { return saved; }").1.is_empty());
}

#[test]
fn return_update_logical_effectful_scalar_selections_reject_incoming_sealed_nonbool_hidden_nested_and_exit_writes_atomically(
) {
    for prefix in [
        "if first { let saved = decide(first); }",
        "if first { let local = decide(first); }",
        "let local = saved; if first { let other = first; }",
        "let local = saved; let other = first; if first { let local = first; } else { let other = saved; }",
        "const local: bool = saved || false; if first { let local = first; }",
        "const local: bool = saved || false; if first { let local = local && first; }",
        "let local = 1; if first { let local = token(2); }",
        "let local = saved; if first { const local: bool = first || false; }",
        "let local = saved; if first { let local: i64 = 1; }",
        "let local = saved; if first { let local = boolean(local || first); }",
        "let local = saved; if first { let local = first; let saved = decide(first); }",
        "let local = saved; if first { return local; }",
        "let local = saved; if first { while saved { let local = first; } }",
        "let local = saved; if first {} else {}",
        "let local = saved; if first { print(96); }",
    ] {
        reject_module(parse_nuis_module(&source(&format!("if decide(gate) {{ {prefix} return saved && first; }} else {{ return saved; }}"))).unwrap());
    }
    // Final unused private work is now admitted; an entering write is still not.
    assert!(!rewrite(
        "if decide(gate) { let local = saved;
        if first { let local = first; let unused = token(1); }
        return saved && first; } else { return saved; }"
    )
    .1
    .is_empty());
    for body in [
        "if gate { let local = saved; if first { let local = decide(first); } if first { if saved { let local = first; } return local && first; } else { return local; } } else { return saved; }",
        "if gate { let local = saved; if first { let local = first; } return local && first; } else { return saved; }",
        "if gate { let local = saved; if decide(first) { let local = first; } return local; } else { return saved; }",
    ] { reject_module(parse_nuis_module(&source(body)).unwrap()); }
    assert!(parse_nuis_module(&source("if gate { let local = saved; if first { let local = decide(first); } return local && first; } else { return local; }")).is_err());
    let original = parse_nuis_module(&source("if gate { let local = saved; if decide(first) { let local = local || first; } return local && first; } else { return saved; }")).unwrap();
    for mutation in ["borrowed", "optional", "generic", "async", "late-leak"] {
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
            let NirStmt::If {
                then_body,
                else_body,
                ..
            } = &mut observer.body[1]
            else {
                unreachable!()
            };
            if mutation == "late-leak" {
                else_body[0] = NirStmt::Return(Some(NirExpr::Var("local".into())));
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
        }
        reject_module(module);
    }
}

#[test]
fn return_update_logical_effectful_scalar_selections_charge_only_original_work_not_retained_transport(
) {
    for inverted in [false, true] {
        let update = if inverted {
            "if first {} else { let local = local || decide(first); }"
        } else {
            "if first { let local = local || decide(first); }"
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
        let signatures = BTreeMap::from([(
            "decide".into(),
            (vec![scalar_type("bool")], scalar_type("bool")),
        )]);
        for (nodes, expressions, logical_edges, accepted) in [
            (5, 12, 2, false),
            (6, 11, 2, false),
            (6, 12, 1, false),
            (6, 12, 2, true),
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
fn return_update_logical_effectful_scalar_selections_share_prefix_node_capture_edge_and_ancestor_depth_limits(
) {
    for updates in [15, 16] {
        let prefix = "if first { let local = local; }".repeat(updates);
        assert_eq!(!rewrite(&format!("if decide(gate) {{ let local = saved; {prefix} return local && first; }} else {{ return saved; }}")).1.is_empty(), updates == 15);
    }
    for levels in [7, 8] {
        let mut child =
            "let local = saved; if first { let local = local; } return local && first;".to_string();
        for _ in 0..levels - 1 {
            child = format!("if first {{ {child} }} else {{ return false; }}");
        }
        assert_eq!(
            !rewrite(&format!(
                "if decide(gate) {{ {child} }} else {{ return saved; }}"
            ))
            .1
            .is_empty(),
            levels == 7
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
    for paired in [2, 3] {
        let prefix = (0..15)
            .map(|n| {
                if n < paired {
                    "if first { let local = local; } else { let local = local; }"
                } else {
                    "if first { let local = local; }"
                }
            })
            .collect::<String>();
        assert_eq!(
            !rewrite(&format!(
                "if decide(gate) {{ let local = saved; {prefix} {} }} else {{ {} }}",
                full(3),
                full(3)
            ))
            .1
            .is_empty(),
            paired == 2
        );
    }
    for extra in [false, true] {
        let prefix = (0..15).map(|n| if extra && n==14 { "if first { let local = (local || false) || false; } else { let local = local || false; }" } else { "if first { let local = local || false; } else { let local = local || false; }" }).collect::<String>();
        assert_eq!(!rewrite(&format!("if decide(gate) {{ let local = saved || false; {prefix} return local && first; }} else {{ return saved; }}")).1.is_empty(),!extra);
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
    for count in [31, 32] {
        let params = (0..count)
            .map(|n| format!(", p{n}: bool"))
            .collect::<String>();
        let value = balanced(&(0..count).map(|n| format!("p{n}")).collect::<Vec<_>>());
        let text=source(&format!("if decide(gate) {{ let local = {value}; if local {{ let local = local; }} return local && true; }} else {{ return false; }}")).replace("saved: bool)",&format!("saved: bool{params})"));
        let mut module = parse_nuis_module(&text).unwrap();
        let before = module.clone();
        let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
        assert_eq!(!outline(&mut module, &mut names).is_empty(), count == 31);
        if count == 32 {
            assert_eq!(module, before);
        }
    }
}
