use super::super::super::super::tests::{reject_module, rewrite, source};
use super::*;
use crate::frontend::parse_nuis_module;

fn scope() -> Scope {
    ["local", "child", "first", "saved"]
        .map(|s| (s.into(), scalar_type("bool")))
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

fn proof(yes: &str, no: &str, suffix: &str, budget: &mut nested::Budget) -> Option<nested::Proof> {
    super::super::prove(
        &NirExpr::Var("first".into()),
        &statements(yes),
        &statements(no),
        &statements(suffix),
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

fn budget() -> nested::Budget {
    nested::Budget {
        nodes: 64,
        expressions: 256,
        logical_edges: 32,
    }
}

#[test]
fn return_nested_prefix_mixed_logical_effectful_scalar_selections_keep_current_conditions_both_orders_and_retention(
) {
    for tree in [
        "if decide(saved) { let child = child && decide(first); if child { let local = child || saved; let child = local && first; } else {let local = first;} } else {if first {} else {let child = saved;let local = child;}}",
        "if saved {let local = child || decide(first); if local {let child = local && saved;let local = child || first;} else {let child = first;}} else {if decide(child) {let local = first;let child = saved;}}",
        "if saved {let scoped = decide(child); if scoped {let child = first;let local = child;} else {let local = child;}} else {if first {let local = saved;let child = first;}}",
    ] {
        for inverted in [false, true] {
            let selected = format!("let local = saved; if first {{let child = local; {tree}
                const seal: bool = child || decide(local); let local = local && seal;
                let ignored = token(2); }} return local && decide(saved);");
            let (yes, no) = if inverted {("return saved;", selected.as_str())} else {(selected.as_str(), "return saved;")};
            let (module, names) = rewrite(&format!("if decide(gate) {{ {yes} }} else {{ {no} }}"));
            assert!(!names.is_empty(), "{tree}");
            crate::nir_verify::verify_nir_module(&module).unwrap();
            let observer = module.functions.iter().find(|f| f.name == "observe").unwrap();
            for statement in &observer.body {
                if let NirStmt::Let {value: NirExpr::Call {callee, ..}, ..} = statement {
                    let helper = module.functions.iter().find(|f| &f.name == callee).unwrap();
                    assert!(!helper.params.iter().any(|p| ["local", "child", "scoped", "seal", "ignored"].contains(&p.name.as_str())));
                }
            }
        }
    }
}

#[test]
fn return_nested_prefix_mixed_logical_effectful_scalar_selections_reject_nested_scope_leaks_seals_extra_writes_and_sequences(
) {
    for tree in [
        "if saved {if first {let local = first;let child = saved;let saved = first;}} let local = child;",
        "if saved {const child: bool = first || false; if first {let local = child;let child = saved;}} let local = child;",
        "if saved {const fresh: bool = child || first; if first {let fresh = saved;let local = first;let child = saved;}} let local = child;",
        "if saved {if first {let local = first;let child = saved;let collision = token(2);}} const collision: bool = child || first; let local = child;",
        "if saved {let collision = token(2);if first {let local = first;let child = saved;}} const collision: bool = child || first; let local = child;",
        "if saved {if first {let local = first;let child = saved;}} if child {let local = child;let child = first;let saved = first;} let local = child;",
        "if saved {if first {let local = first;let child = saved;let saved = first;}let local = child;} let local = child;",
        "if saved {if boolean(child || first) {let local = first;let child = saved;}} let local = child;",
        "if saved {if first {let local = first;let child = saved;print(96);}} let local = child;",
        "if saved {if first {let local = first;let child = saved;return child;}} let local = child;",
    ] {
        reject_module(parse_nuis_module(&source(&format!("if decide(gate) {{let local = saved;
            if first {{let child = local; {tree} }} return local && first;}} else {{return saved;}}"))).unwrap());
    }
    let mut yes = statements("if first {let local = first;let child = saved;let secret = first;}");
    let mut tail = statements("let local = child;");
    if let NirStmt::Let { value, .. } = &mut tail[0] {
        *value = NirExpr::Var("secret".into());
    }
    assert!(super::super::prove(
        &NirExpr::Bool(true),
        &yes,
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
    // A nested-only branch name also cannot become a sibling condition input.
    if let NirStmt::If { else_body, .. } = &mut yes[0] {
        *else_body = statements("if first {let local = first;let child = saved;}");
        if let NirStmt::If { then_body, .. } = &mut else_body[0] {
            if let NirStmt::Let { value, .. } = &mut then_body[1] {
                *value = NirExpr::Var("secret".into());
            }
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
fn return_nested_prefix_mixed_logical_effectful_scalar_selections_charge_original_tree_and_shared_tail_once(
) {
    let yes = "let child = first; if decide(child) {let local = child || first;let child = local && saved;} else {let local = saved;}";
    for (nodes, expressions, logical_edges, accepted) in [
        (7, 16, 3, false),
        (8, 15, 3, false),
        (8, 16, 2, false),
        (8, 16, 3, true),
    ] {
        let mut b = nested::Budget {
            nodes,
            expressions,
            logical_edges,
        };
        assert_eq!(
            proof(
                yes,
                "",
                "let observed = decide(child);let local = local && observed;",
                &mut b
            )
            .is_some(),
            accepted
        );
        if accepted {
            assert_eq!((b.nodes, b.expressions, b.logical_edges), (0, 0, 0));
        }
    }
}

fn full(levels: usize, logical: bool) -> String {
    if levels == 0 {
        if logical {
            "let local = local || first;let child = child && saved;".into()
        } else {
            "let local = first;let child = saved;".into()
        }
    } else {
        let inner = full(levels - 1, logical);
        format!("if first {{ {inner} }} else {{ {inner} }}")
    }
}

fn balanced(names: &[String], op: &str) -> String {
    if names.len() == 1 {
        return names[0].clone();
    }
    let mid = names.len() / 2;
    format!(
        "({}) {op} ({})",
        balanced(&names[..mid], op),
        balanced(&names[mid..], op)
    )
}

#[test]
fn return_nested_prefix_mixed_logical_effectful_scalar_selections_keep_exact_nodes_expressions_edges_depth_and_prefix_limits(
) {
    for count in [15, 16] {
        let yes = format!("{}{}", "let local = local;".repeat(count), full(4, false));
        assert_eq!(
            proof(&yes, "", "let local = child;", &mut budget()).is_some(),
            count == 15
        );
    }
    for extra in [false, true] {
        let tree = full(4, true);
        let tree = if extra {
            tree.replacen("local || first", "(local || first) || saved", 1)
        } else {
            tree
        };
        assert_eq!(
            proof(&tree, "", "let local = child;", &mut budget()).is_some(),
            !extra
        );
        let leaves = vec!["first".into(); 126];
        let value = balanced(&leaves, "==");
        let value = if extra { format!("!({value})") } else { value };
        let tree = format!("if first {{let local = {value};let child = false;}}");
        assert_eq!(
            proof(
                &tree,
                "",
                "let unused = false;let local = child;",
                &mut budget()
            )
            .is_some(),
            !extra
        );
    }
    for count in [7, 8] {
        let mut tree = "let local = first;let child = saved;".to_owned();
        for _ in 0..count {
            tree = format!("if first {{ {tree} }}");
        }
        assert_eq!(
            proof(&tree, "", "let local = child;", &mut budget()).is_some(),
            count == 7
        );
    }
    for count in [16, 17] {
        let tree = format!(
            "{}if first {{let local = first;let child = saved;}}",
            "let local = local;".repeat(count)
        );
        let mut b = nested::Budget {
            nodes: 128,
            expressions: 512,
            logical_edges: 64,
        };
        assert_eq!(
            proof(&tree, "", "let local = child;", &mut b).is_some(),
            count == 16
        );
    }
}

#[test]
fn return_nested_prefix_mixed_logical_effectful_scalar_selections_capture_old_reads_and_bound_union_without_permission_versions(
) {
    for (yes,no,tail,old_child) in [
        ("if first {let local = saved;let child = first;} else {let child = first;let local = saved;}","let local = saved;let child = first;","let local = child;",false),
        ("if first {let local = saved;let child = first;}","","let local = child;",true),
        ("let local = child;if first {let child = saved;let local = first;}","let child = first;let local = saved;","let local = child;",true),
        ("if first {let local = saved;let child = first;}","","let child = saved;let local = child;",false),
    ] {
        let p = proof(yes,no,tail,&mut budget()).unwrap();
        assert_eq!(p.inputs.contains("child"),old_child);
        assert!(!p.inputs.contains("local"));
    }
    for count in [30, 31] {
        let params = (0..count)
            .map(|i| format!(", p{i}: bool"))
            .collect::<String>();
        let tail = balanced(
            &(0..count).map(|i| format!("p{i}")).collect::<Vec<_>>(),
            "&&",
        );
        let text=source(&format!("if decide(gate) {{let local = false;if first {{let child = false;
            if first {{if first {{let local = first;let child = false;}} else {{let local = first;let child = false;}}}} else {{let local = first;let child = false;}}
            let local = ({tail}) && child;}} else {{let local = false;}} return local && true;}} else {{return false;}}"))
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

#[test]
fn return_nested_prefix_mixed_logical_effectful_scalar_selections_copy_flat_recipes_only_at_original_leaves(
) {
    fn counts(value: &SelectedValue) -> (usize, usize) {
        match value {
            SelectedValue::Nested(plan) | SelectedValue::StagedNested(_,plan) => {
                assert_eq!(plan.destination,Some(("local".into(),false)));
                let (a,b)=counts(&plan.yes);let(c,d)=counts(&plan.no);(a+c,b+d)
            }
            SelectedValue::Region(statements,_) => (1,statements.iter().filter(|s|matches!(s,regions::Statement::Scalar(NirStmt::Let {name,..}) if name=="tail")).count()),
            _ => panic!("unexpected transport"),
        }
    }
    let mut b = budget();
    let p = proof(&full(4, false), "", &"let tail = child;".repeat(16), &mut b).unwrap();
    assert_eq!(b.nodes, 0);
    assert_eq!(counts(&p.value), (17, 17 * 16));
    // The source has 16 tree leaves plus the root's retained arm, not 2^16
    // continuation combinations. No selection recipes enter copy_flat.
    assert!(proof(
        &full(4, false),
        "",
        &format!(
            "{}if first {{let local = child;}}",
            "let tail = child;".repeat(15)
        ),
        &mut budget()
    )
    .is_none());
}
