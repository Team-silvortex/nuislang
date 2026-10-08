use super::*;
use crate::frontend::parse_nuis_module;
use tests::source;

fn expressions(body: &[NirStmt], visit: &mut impl FnMut(&NirExpr)) {
    fn walk(value: &NirExpr, visit: &mut impl FnMut(&NirExpr)) {
        visit(value);
        crate::nir_walk::walk_child_exprs(value, &mut |child| walk(child, visit));
    }
    for statement in body {
        match statement {
            NirStmt::Let { value, .. }
            | NirStmt::Const { value, .. }
            | NirStmt::Return(Some(value))
            | NirStmt::Expr(value)
            | NirStmt::Print(value) => walk(value, visit),
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                walk(condition, visit);
                expressions(then_body, visit);
                expressions(else_body, visit);
            }
            _ => {}
        }
    }
}

fn budget(expressions: usize) -> nested::Budget {
    nested::Budget {
        nodes: 64,
        expressions,
        logical_edges: 32,
    }
}

fn edge(lhs: NirExpr, rhs: NirExpr) -> NirExpr {
    NirExpr::Binary {
        op: NirBinaryOp::Or,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    }
}

fn balanced(edges: usize) -> NirExpr {
    if edges == 0 {
        return NirExpr::Bool(true);
    }
    let left = (edges - 1) / 2;
    edge(balanced(left), balanced(edges - 1 - left))
}

#[test]
fn tree_logical_effectful_scalar_selections_guard_each_edge_keep_calls_once_types_and_hygiene() {
    for kind in ["bool", "i32", "i64", "f32", "f64"] {
        for condition in [
            "(decide(saved) && first) || (second && decide(saved))",
            "(decide(saved) || first) && (second || decide(saved))",
            "(first && second) || decide(saved)",
            "decide(saved) && (first || second)",
        ] {
            for inverted in [false, true] {
                let child = if inverted {
                    format!("if {condition} {{}} else {{ let saved = work(saved); }}")
                } else {
                    format!("if {condition} {{ let saved = work(saved); }}")
                };
                let mut module = parse_nuis_module(&source(
                    kind,
                    &format!("if gate {{ {child} if second {{ let saved = work(saved); }} }}"),
                ))
                .unwrap();
                let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
                let generated = outline(&mut module, &mut names);
                let edges = condition.matches("&&").count() + condition.matches("||").count();
                assert_eq!(generated.len(), 6 + edges, "{kind}/{condition}/{inverted}");
                assert!(!generated.contains("__nuis_effect_call_predicate_0"));
                crate::nir_verify::verify_nir_module(&module).unwrap();
                let mut calls = 0;
                let mut predicates = 0;
                for function in module
                    .functions
                    .iter()
                    .filter(|f| generated.contains(&f.name))
                {
                    let predicate = function.name.starts_with("__nuis_effect_call_predicate_");
                    assert_eq!(
                        function.return_type,
                        Some(scalar_type(if predicate { "bool" } else { kind }))
                    );
                    assert_eq!(
                        function.params.len(),
                        function
                            .params
                            .iter()
                            .map(|p| &p.name)
                            .collect::<BTreeSet<_>>()
                            .len()
                    );
                    if predicate {
                        predicates += 1;
                        assert_eq!(function.body.len(), 2);
                        assert!(
                            matches!(&function.body[0], NirStmt::If { then_body, else_body, .. }
                            if then_body == &[NirStmt::Return(Some(NirExpr::Bool(false)))] && else_body.is_empty())
                        );
                    }
                    expressions(&function.body, &mut |value| {
                        assert!(!matches!(
                            value,
                            NirExpr::Binary {
                                op: NirBinaryOp::And | NirBinaryOp::Or,
                                ..
                            }
                        ));
                        if let NirExpr::Call { callee, args } = value {
                            calls += usize::from(callee == "decide");
                            if callee.starts_with("__nuis_effect_call_predicate_") {
                                assert!(args.iter().skip(1).all(|a| matches!(a, NirExpr::Var(_))));
                            }
                        }
                    });
                }
                assert_eq!(predicates, edges);
                assert_eq!(calls, condition.matches("decide(").count());
            }
        }
    }
    for condition in [
        "(decide(saved / 2) && first) || second",
        "first || (decide(saved / 2) && second)",
    ] {
        let mut module = parse_nuis_module(&source("i64", &format!(
            "if gate {{ if {condition} {{ let saved = saved; }} if second {{ let saved = saved; }} }}"
        ))).unwrap();
        let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
        let generated = outline(&mut module, &mut names);
        assert_eq!(generated.len(), 8);
        crate::nir_verify::verify_nir_module(&module).unwrap();
        let mut calls = 0;
        for function in module
            .functions
            .iter()
            .filter(|f| generated.contains(&f.name))
        {
            expressions(&function.body, &mut |value| {
                if let NirExpr::Call { callee, args } = value {
                    if callee == "decide" {
                        calls += 1;
                        assert!(matches!(
                            args.as_slice(),
                            [NirExpr::Binary {
                                op: NirBinaryOp::Div,
                                ..
                            }]
                        ));
                    }
                }
            });
        }
        assert_eq!(calls, 1);
    }
}

#[test]
fn tree_logical_effectful_scalar_selections_share_exact_edge_expression_and_combined_depth_bounds()
{
    for edges in [32, 33] {
        let mut limits = budget(256);
        assert_eq!(
            prove(
                &balanced(edges),
                &Scope::new(),
                &Signatures::new(),
                &mut limits
            )
            .is_some(),
            edges == 32
        );
        assert_eq!(limits.logical_edges, 0);
        assert_eq!(limits.nodes, 64);
    }
    for expressions in [64, 65] {
        let mut limits = budget(expressions);
        assert_eq!(
            prove(
                &balanced(32),
                &Scope::new(),
                &Signatures::new(),
                &mut limits
            )
            .is_some(),
            expressions == 65
        );
        assert_eq!(limits.expressions, 0);
    }
    for depth in [31, 32] {
        let mut condition = NirExpr::Bool(true);
        for _ in 0..depth {
            condition = edge(NirExpr::Bool(false), condition);
        }
        assert_eq!(
            prove(
                &condition,
                &Scope::new(),
                &Signatures::new(),
                &mut budget(256)
            )
            .is_some(),
            depth == 31
        );
    }
    for additions in [28, 29] {
        let mut value = NirExpr::Int(1);
        for _ in 0..additions {
            value = NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(value),
                rhs: Box::new(NirExpr::Int(1)),
            };
        }
        let comparison = NirExpr::Binary {
            op: NirBinaryOp::Eq,
            lhs: Box::new(value),
            rhs: Box::new(NirExpr::Int(0)),
        };
        let condition = edge(NirExpr::Bool(false), edge(comparison, NirExpr::Bool(true)));
        assert_eq!(
            prove(
                &condition,
                &Scope::new(),
                &Signatures::new(),
                &mut budget(256)
            )
            .is_some(),
            additions == 28
        );
    }
}

#[test]
fn tree_logical_effectful_scalar_selections_share_edges_across_original_sibling_regions_atomically()
{
    for count in [16, 17] {
        let children =
            "if first && (second || decide(saved)) { let saved = work(saved); }".repeat(count);
        let mut module =
            parse_nuis_module(&source("i64", &format!("if gate {{ {children} }}"))).unwrap();
        let before = module.clone();
        let mut names: BTreeSet<String> = module.functions.iter().map(|f| f.name.clone()).collect();
        let before_names = names.clone();
        let generated = outline(&mut module, &mut names);
        if count == 16 {
            assert_eq!(generated.len(), 2 + 4 * count);
            crate::nir_verify::verify_nir_module(&module).unwrap();
        } else {
            assert!(generated.is_empty());
            assert_eq!(module, before);
            assert_eq!(names, before_names);
        }
    }
}

#[test]
fn tree_logical_effectful_scalar_selections_bound_complete_rhs_capture_union_not_each_leaf() {
    for count in [31, 32] {
        let scope = (0..count)
            .map(|i| (format!("v{i}"), scalar_type("i64")))
            .collect();
        let signatures = [
            (
                "left".into(),
                (vec![scalar_type("i64"); 16], scalar_type("bool")),
            ),
            (
                "right".into(),
                (vec![scalar_type("i64"); count - 16], scalar_type("bool")),
            ),
        ]
        .into();
        let call = |name: &str, from, to| NirExpr::Call {
            callee: name.into(),
            args: (from..to).map(|i| NirExpr::Var(format!("v{i}"))).collect(),
        };
        let condition = edge(
            NirExpr::Bool(false),
            edge(call("left", 0, 16), call("right", 16, count)),
        );
        assert_eq!(
            prove(&condition, &scope, &signatures, &mut budget(256)).is_some(),
            count == 31
        );
    }
}

#[test]
fn tree_logical_effectful_scalar_selections_reject_hidden_leaves_and_late_invalid_descendants_atomically(
) {
    for condition in [
        "((first || second) == true && decide(saved)) || first",
        "first || (bool_gate(first && second) && decide(saved))",
    ] {
        let text = source("i64", &format!("if gate {{ if {condition} {{ let saved = work(saved); }} if second {{ let saved = work(saved); }} }}"))
            .replace("fn observe", "fn bool_gate(v: bool) -> bool { print(94); return v; } fn observe");
        let mut module = parse_nuis_module(&text).unwrap();
        let before = module.clone();
        let mut names: BTreeSet<String> = module.functions.iter().map(|f| f.name.clone()).collect();
        let before_names = names.clone();
        assert!(outline(&mut module, &mut names).is_empty());
        assert_eq!(module, before);
        assert_eq!(names, before_names);
    }
    for invalid in ["absent", "work", "missing"] {
        let mut module = parse_nuis_module(&source("i64", "if gate { if (first || decide(saved)) && (second || decide(saved)) { let saved = work(saved); } if second { let saved = work(saved); } }")).unwrap();
        let function = module
            .functions
            .iter_mut()
            .find(|f| f.name == "observe")
            .unwrap();
        let NirStmt::If { then_body, .. } = &mut function.body[0] else {
            unreachable!()
        };
        let NirStmt::If { condition, .. } = &mut then_body[0] else {
            unreachable!()
        };
        let NirExpr::Binary { rhs, .. } = condition else {
            unreachable!()
        };
        let NirExpr::Binary { rhs, .. } = rhs.as_mut() else {
            unreachable!()
        };
        let NirExpr::Call { callee, args } = rhs.as_mut() else {
            unreachable!()
        };
        if invalid == "missing" {
            args[0] = NirExpr::Var("missing".into());
        } else {
            *callee = invalid.into();
        }
        let before = module.clone();
        let mut names: BTreeSet<String> = module.functions.iter().map(|f| f.name.clone()).collect();
        let before_names = names.clone();
        assert!(outline(&mut module, &mut names).is_empty());
        assert_eq!(module, before);
        assert_eq!(names, before_names);
    }
}
