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
fn return_ancestor_logical_effectful_scalar_selections_keep_guarded_current_stages_seals_unused_effects_and_private_descendant_captures(
) {
    let staged = "let local = decide(saved); let local = local && first;
        const sealed: bool = local || false; let unused = token(1);
        if decide(sealed) {
            let inner = local || false;
            if inner { let leaf = inner; return leaf && decide(sealed); } else { return false; }
        } else { return local || first; }";
    let opposite =
        "let local = token(1); if local == 1 { return (local == 1) && first; } else { return false; }";
    for inverted in [false, true] {
        let (yes, no) = if inverted {
            (opposite, staged)
        } else {
            (staged, opposite)
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
        for stmt in &observer.body {
            assert!(
                !matches!(stmt,NirStmt::Let { name,.. } | NirStmt::Const { name,.. }
                if ["local","sealed","unused","inner","leaf"].contains(&name.as_str()))
            );
            if let NirStmt::Let {
                value: NirExpr::Call { callee, .. },
                ..
            } = stmt
            {
                let helper = module.functions.iter().find(|f| &f.name == callee).unwrap();
                assert!(!helper
                    .params
                    .iter()
                    .any(|p| ["local", "sealed", "unused", "inner", "leaf"]
                        .contains(&p.name.as_str())));
            }
        }
        let staged_arm = module
            .functions
            .iter()
            .find(|f| {
                f.body
                    .iter()
                    .any(|s| matches!(s,NirStmt::Const { name,.. } if name == "sealed"))
            })
            .unwrap();
        assert!(matches!(staged_arm.body.first(), Some(NirStmt::If { .. })));
        let unused = staged_arm
            .body
            .iter()
            .position(|s| matches!(s,NirStmt::Let { name,.. } if name == "unused"))
            .unwrap();
        let child_gate = staged_arm.body.iter().position(|s| matches!(s,NirStmt::Let { name,.. } if name.starts_with("__nuis_effect_call_gate"))).unwrap();
        assert!(unused < child_gate);
        assert!(module
            .functions
            .iter()
            .filter(|f| generated.contains(&f.name))
            .any(|f| f
                .params
                .iter()
                .any(|p| p.name == "sealed" && p.ty == scalar_type("bool"))));
        assert!(module
            .functions
            .iter()
            .filter(|f| generated.contains(&f.name))
            .any(|f| f
                .params
                .iter()
                .any(|p| p.name == "local" && p.ty == scalar_type("i64"))));
        for helper in module
            .functions
            .iter()
            .filter(|f| generated.contains(&f.name))
        {
            assert!(matches!(helper.body.first(), Some(NirStmt::If { .. })));
            assert!(helper.params.len() <= 32);
            assert_eq!(helper.return_type, Some(scalar_type("bool")));
        }
    }
    // Unused ancestor work alone can discover effects; pure complete children compose.
    let (module,generated) = rewrite("if gate { let unused = token(1); if first { return saved && first; } else { return false; } } else { return saved; }");
    assert!(!generated.is_empty());
    crate::nir_verify::verify_nir_module(&module).unwrap();
}

#[test]
fn return_ancestor_logical_effectful_scalar_selections_reject_entering_writes_cross_child_leaks_hidden_work_partial_exits_and_late_invalid_types_atomically(
) {
    for body in [
        "if gate { let saved = decide(first); if first { return saved && first; } else { return false; } } else { return saved; }",
        "if gate { let local = decide(saved); if first { let local = first; return local && first; } else { return saved; } } else { return saved; }",
        "if gate { const sealed: bool = saved || first; if first { let sealed = decide(saved); return sealed && first; } else { return saved; } } else { return saved; }",
        "if gate { const sealed: bool = saved || first; let sealed = decide(saved); if first { return sealed && first; } else { return saved; } } else { return saved; }",
        "if gate { let local = boolean(saved || decide(first)); if first { return local && first; } else { return saved; } } else { return saved; }",
        "if gate { print(96); if first { return saved && decide(first); } else { return saved; } } else { return saved; }",
        "if gate { while first { let local = saved; } if first { return saved && decide(first); } else { return saved; } } else { return saved; }",
        "if gate { return saved; if first { return saved && decide(first); } else { return saved; } } else { return saved; }",
        "if gate { let local = decide(saved); if first { return local && first; } } else { return saved; }",
        "if gate { let local = decide(saved); if first { return local && first; } else { let saved = first; } } else { return saved; }",
        "if gate { let local = saved; if first { return local && first; } else { return saved; } } else { return saved; }",
        "if gate { let local = decide(saved) && first; if first { return local; } else { return saved; } } else { return saved; }",
    ] { reject_module(parse_nuis_module(&source(body)).unwrap()); }
    let original = parse_nuis_module(&source("if gate { let local = decide(saved); if first { return local && first; } else { return saved; } } else { return saved; }")).unwrap();
    for mutation in [
        "reference",
        "optional",
        "generic",
        "missing",
        "i64",
        "late-leak",
        "late-missing",
        "late-void",
        "prefix-type",
        "child-type",
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
                "reference" => observer.return_type.as_mut().unwrap().is_ref = true,
                "optional" => observer.return_type.as_mut().unwrap().is_optional = true,
                "generic" => observer
                    .return_type
                    .as_mut()
                    .unwrap()
                    .generic_args
                    .push(scalar_type("bool")),
                "missing" => observer.return_type = None,
                "i64" => observer.return_type = Some(scalar_type("i64")),
                _ => {
                    let NirStmt::If {
                        then_body,
                        else_body,
                        ..
                    } = &mut observer.body[1]
                    else {
                        unreachable!()
                    };
                    if mutation == "prefix-type" {
                        let NirStmt::Let { ty, .. } = &mut then_body[0] else {
                            unreachable!()
                        };
                        *ty = Some(scalar_type("i64"));
                    } else if mutation == "child-type" {
                        let NirStmt::If { condition, .. } = &mut then_body[1] else {
                            unreachable!()
                        };
                        *condition = NirExpr::Int(1);
                    } else {
                        else_body[0] = NirStmt::Return(match mutation {
                            "late-leak" => Some(NirExpr::Var("local".into())),
                            "late-missing" => Some(NirExpr::Var("absent".into())),
                            "late-void" => None,
                            _ => unreachable!(),
                        });
                    }
                }
            }
        }
        reject_module(module);
    }
}

#[test]
fn return_ancestor_logical_effectful_scalar_selections_charge_original_stages_child_conditions_and_leaf_returns_once(
) {
    let module = parse_nuis_module(&source("if decide(gate) { let local = saved; const sealed: bool = local || false; if sealed { return local && first; } else { return false; } } else { return saved; }")).unwrap();
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
        (6, 12, 2, false),
        (7, 11, 2, false),
        (7, 12, 1, false),
        (7, 12, 2, true),
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
fn return_ancestor_logical_effectful_scalar_selections_bound_private_stages_unused_parent_reads_and_shared_node_edge_expression_depth_work(
) {
    for count in [16, 17] {
        let prefix = (0..count)
            .map(|n| format!("let local{n} = saved;"))
            .collect::<String>();
        assert_eq!(!rewrite(&format!("if decide(gate) {{ {prefix} if first {{ return saved && first; }} else {{ return false; }} }} else {{ return saved; }}")).1.is_empty(),count == 16);
    }
    for count in [31, 32] {
        let params = (0..count)
            .map(|n| format!(", p{n}: bool"))
            .collect::<String>();
        let value = balanced(&(0..count).map(|n| format!("p{n}")).collect::<Vec<_>>());
        let text = source(&format!("if decide(gate) {{ let unused = {value}; if p0 {{ return p0 && true; }} else {{ return false; }} }} else {{ return false; }}"))
            .replace("saved: bool)",&format!("saved: bool{params})"));
        let mut module = parse_nuis_module(&text).unwrap();
        let before = module.clone();
        let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
        assert_eq!(!outline(&mut module, &mut names).is_empty(), count == 31);
        if count == 32 {
            assert_eq!(module, before);
        }
    }
    for depth in [8, 9] {
        let mut child = "return saved && first;".to_string();
        for n in 0..depth - 1 {
            child =
                format!("let local{n} = saved; if first {{ {child} }} else {{ return false; }}");
        }
        assert_eq!(
            !rewrite(&format!(
                "if decide(gate) {{ {child} }} else {{ return saved; }}"
            ))
            .1
            .is_empty(),
            depth == 8
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
    for extra in [1, 2] {
        let prefix = (0..extra)
            .map(|n| format!("let local{n} = saved;"))
            .collect::<String>();
        let body = format!(
            "if decide(gate) {{ {prefix} {} }} else {{ {} }}",
            full(4),
            full(4)
        );
        assert_eq!(!rewrite(&body).1.is_empty(), extra == 1); // 64 versus 65 original statements.
    }
    for leaves in [32, 33] {
        let value = balanced(&vec!["saved".into(); leaves]);
        assert_eq!(!rewrite(&format!("if decide(gate) {{ let unused = {value}; if first {{ return saved && first; }} else {{ return false; }} }} else {{ return saved; }}")).1.is_empty(),leaves == 32);
    }
    for depth in [30, 31] {
        let mut value = "saved".to_string();
        for _ in 0..depth {
            value = format!("boolean({value})");
        }
        assert_eq!(!rewrite(&format!("if decide(gate) {{ let unused = {value} && first; if first {{ return saved && first; }} else {{ return false; }} }} else {{ return saved; }}")).1.is_empty(),depth == 30);
    }
    for width in [62, 63] {
        let value = balanced(&vec!["(saved == first)".into(); width]);
        let module=parse_nuis_module(&source(&format!("if decide(gate) {{ let unused = {value}; if first {{ return saved && true; }} else {{ return false; }} }} else {{ return false; }}"))).unwrap();
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
            width == 62
        );
    }
}
