use super::super::super::super::super::tests::{reject_module, rewrite, source};
use super::*;
use crate::frontend::parse_nuis_module;

fn scope() -> Scope {
    ["local", "child", "first", "saved"]
        .map(|n| (n.into(), scalar_type("bool")))
        .into()
}

fn statements(text: &str) -> Vec<NirStmt> {
    let module = parse_nuis_module(&source(&format!(
        "let local = false;let child = false;{text}"
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

fn proof(yes: &str, no: &str, tail: &str, b: &mut nested::Budget) -> Option<nested::Proof> {
    super::super::prove(
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
        b,
        0,
    )
}

#[test]
fn return_nested_sequential_mixed_logical_effectful_scalar_selections_keep_branch_current_versions_both_orders_polarities_and_retention(
) {
    for nested in [
        "if decide(saved) {let current = child; if current {let child = child || decide(first);let local = child && saved;} else {let local = first;} if local {let child = local && first;} let local = child || saved;} else {if child {let local = child;let child = first;} let local = child;}",
        "if saved {if child {let local = child || decide(first);let child = local && saved;} if local {} else {let child = saved;} let local = child;} else {if first {} else {let child = first;let local = child;} let child = child || saved;}",
    ] {
        for tail in ["let local = local && decide(child);", "if child {let local = local && decide(child);let child = first;} const seal: bool = child || saved;let local = local && seal;"] {
            for inverted in [false,true] {
                let selected = format!("let local = saved;if first {{let child = local;{nested}{tail}}} return local && decide(saved);");
                let (yes,no) = if inverted {("return saved;",selected.as_str())} else {(selected.as_str(),"return saved;")};
                let (module,names) = rewrite(&format!("if decide(gate) {{{yes}}} else {{{no}}}"));
                assert!(!names.is_empty(),"{nested}");
                crate::nir_verify::verify_nir_module(&module).unwrap();
                let observer=module.functions.iter().find(|f|f.name=="observe").unwrap();
                for stmt in &observer.body {
                    if let NirStmt::Let {value:NirExpr::Call {callee,..},..}=stmt {
                        let helper=module.functions.iter().find(|f|&f.name==callee).unwrap();
                        assert_eq!(helper.return_type,Some(scalar_type("bool")));
                        assert!(!helper.params.iter().any(|p|["local","child","current","seal"].contains(&p.name.as_str())));
                    }
                }
            }
        }
    }
}

#[test]
fn return_nested_sequential_mixed_logical_effectful_scalar_selections_reject_lexical_leaks_collisions_seals_extra_writes_and_exits_atomically(
) {
    for body in [
        "if saved {if child {let local = first;let child = saved;let saved = first;} let local = child;} let local = child;",
        "if saved {const sealed: bool = child || first;if child {let sealed = saved;let local = first;let child = saved;} let local = child;} let local = child;",
        "if saved {if child {let local = first;let child = saved;let clash = first;} const clash: bool = child || first;let local = child;} let local = child;",
        "if saved {if child {let local = first;let child = saved;let clash = first;} let local = child;} const clash: bool = child || first;let local = child;",
        "if saved {if boolean(child || first) {let local = first;let child = saved;} let local = child;} let local = child;",
        "if saved {if child {let local = first;let child = saved;print(96);} let local = child;} let local = child;",
        "if saved {if child {let local = first;let child = saved;return child;} let local = child;} let local = child;",
    ] {
        reject_module(parse_nuis_module(&source(&format!("if decide(gate) {{let local = saved;if first {{let child = local;{body}}} return local && first;}} else {{return saved;}}"))).unwrap());
    }
    // Fabricated NIR cannot bypass lexical proof by referring to branch locals.
    let mut yes = statements(
        "if child {let secret = first;let local = first;let child = saved;} let local = child;",
    );
    if let NirStmt::Let { value, .. } = &mut yes[1] {
        *value = NirExpr::Var("secret".into());
    }
    assert!(super::super::prove(
        &NirExpr::Bool(true),
        &yes,
        &[],
        &statements("let local = child;"),
        &scope(),
        &Target("local"),
        &Child("child".into()),
        &BTreeMap::new(),
        &mut budget(),
        0
    )
    .is_none());
    let mut yes=statements("if child {let secret = first;let local = first;let child = saved;} else {let local = child;} let local = child;");
    if let NirStmt::If { else_body, .. } = &mut yes[0] {
        if let NirStmt::Let { value, .. } = &mut else_body[0] {
            *value = NirExpr::Var("secret".into());
        }
    }
    assert!(super::super::prove(
        &NirExpr::Bool(true),
        &yes,
        &[],
        &statements("let local = child;"),
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
fn return_nested_sequential_mixed_logical_effectful_scalar_selections_charge_each_original_node_expression_and_edge_once(
) {
    for (nodes, expressions, accepted) in [(8, 8, true), (7, 8, false), (8, 7, false)] {
        let mut b = nested::Budget {
            nodes,
            expressions,
            logical_edges: 0,
        };
        let p = proof(
            "if child {let child = first;let local = child;} let local = child;",
            "",
            "if local {let child = saved;} let local = child;",
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
                "if child {let child = first;let local = child && saved;} let local = child;",
                "",
                "if local {let child = saved;} let local = child || first;",
                &mut b
            )
            .is_some(),
            edges == 2
        );
    }
}

#[test]
fn return_nested_sequential_mixed_logical_effectful_scalar_selections_preflight_complete_forest_and_keep_depth_body_and_node_limits(
) {
    for count in [64, 65] {
        let arm =
            "let local = first;let child = saved;".to_owned() + &"let fresh = first;".repeat(14);
        let yes = format!("if child {{{arm}}} else {{{arm}}} let local = child;");
        let no = "let local = first;".repeat(16);
        let tail = format!(
            "if local {{{}}} let local = child;",
            "let child = saved;".repeat(count - 53)
        );
        assert_eq!(
            proof(&yes, &no, &tail, &mut budget()).is_some(),
            count == 64
        );
    }
    for count in [16, 17] {
        let yes = format!(
            "if child {{{}}} let local = child;",
            "let local = first;let child = saved;".to_owned()
                + &"let fresh = first;".repeat(count - 2)
        );
        assert_eq!(
            proof(&yes, "", "let local = child;", &mut budget()).is_some(),
            count == 16
        );
    }
    for count in [7, 8] {
        let mut yes = "let local = first;let child = saved;".to_owned();
        for _ in 0..count {
            yes = format!("if child {{{yes}}} let local = child;");
        }
        assert_eq!(
            proof(&yes, "", "let local = child;", &mut budget()).is_some(),
            count == 7
        );
    }
    assert!(continued(
        &statements("if child {let local = first;} let local = child;"),
        &[]
    )
    .unwrap());
    assert!(!continued(
        &statements("let local = first;if child {let local = first;}"),
        &[]
    )
    .unwrap());
    assert!(continued(&statements(&"let local = first;".repeat(65)), &[]).is_none());
}

#[test]
fn return_nested_sequential_mixed_logical_effectful_scalar_selections_bound_nested_capture_union_and_keep_retained_old_versions(
) {
    let p = proof(
        "if child {let local = first;let child = saved;} let local = child;",
        "",
        "if local {let child = first;} let local = child;",
        &mut budget(),
    )
    .unwrap();
    assert!(p.inputs.contains("local") && p.inputs.contains("child"));
    for count in [31, 32] {
        let names = ["local", "child", "first", "saved"]
            .map(str::to_owned)
            .into_iter()
            .chain((0..count - 4).map(|i| format!("p{i}")))
            .collect::<Vec<_>>();
        let mut s = scope();
        s.extend(names.iter().map(|n| (n.clone(), scalar_type("bool"))));
        let mut yes =
            statements("if child {let local = first;let child = saved;} let local = child;");
        if let NirStmt::If { then_body, .. } = &mut yes[0] {
            if let NirStmt::Let { value, .. } = &mut then_body[0] {
                *value = NirExpr::Call {
                    callee: "all".into(),
                    args: names.iter().map(|n| NirExpr::Var(n.clone())).collect(),
                };
            }
        }
        let signatures = BTreeMap::from([(
            "all".into(),
            (vec![scalar_type("bool"); count], scalar_type("bool")),
        )]);
        assert_eq!(
            super::super::prove(
                &NirExpr::Bool(true),
                &yes,
                &[],
                &statements("let local = child;"),
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
fn return_nested_sequential_mixed_logical_effectful_scalar_selections_install_two_helpers_per_original_selection_without_tail_copying(
) {
    fn fork(levels: usize) -> String {
        if levels == 0 {
            return "let child = first;let local = child;".into();
        }
        format!(
            "if child {{{}}} else {{{}}} let local = child;",
            fork(levels - 1),
            fork(levels - 1)
        )
    }
    for levels in [1, 2, 3] {
        let p = proof(
            &fork(levels),
            "",
            "if local {let child = saved;} let local = child;",
            &mut budget(),
        )
        .unwrap();
        let SelectedValue::Region(statements, _) = p.value else {
            unreachable!()
        };
        assert_eq!(statements.len(), 3);
        assert!(copy_flat(&statements).is_none());
        let mut helpers = Vec::new();
        regions::install(
            statements,
            &mut BTreeSet::new(),
            &mut BTreeSet::new(),
            &mut helpers,
            &mut Vec::new(),
        );
        let selections = (1usize << levels) + 1;
        assert_eq!(helpers.len(), 2 * selections);
        assert!(helpers
            .iter()
            .all(|f| f.return_type == Some(scalar_type("i64"))
                && matches!(f.body.first(), Some(NirStmt::If { .. }))));
    }
}
