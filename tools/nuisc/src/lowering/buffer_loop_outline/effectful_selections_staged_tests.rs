use super::*;
use crate::frontend::parse_nuis_module;

fn source(body: &str, params: &str) -> String {
    format!(
        "mod cpu Main {{
        fn work(v: i64) -> i64 {{ print(70); return v; }}
        fn decide(gate: bool, staged: i64, saved: i64) -> bool {{ print(98); return gate; }}
        fn observe(gate: bool, inner: bool, saved: i64, other: i64{params}) -> i64 {{
            {body} return saved;
        }} fn main() -> i64 {{ return 0; }} }}"
    )
}

fn rewrite(source: &str) -> (NirModule, BTreeSet<String>) {
    let mut module = parse_nuis_module(source).unwrap();
    let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
    let generated = outline(&mut module, &mut names);
    crate::nir_verify::verify_nir_module(&module).unwrap();
    (module, generated)
}

#[test]
fn staged_effectful_scalar_selections_capture_child_versions_after_ordered_prefixes() {
    let (module, generated) = rewrite(&source(
        "if gate {
        let staged = work(saved); let saved = saved + staged;
        let staged = work(saved);
        if decide(inner, staged, saved) { let saved = work(staged); }
    }",
        "",
    ));
    assert_eq!(generated.len(), 4);
    let carrier = module
        .functions
        .iter()
        .find(|f| {
            generated.contains(&f.name)
                && f.body.iter().any(|stmt| {
                    matches!(stmt, NirStmt::Let {
            value: NirExpr::Call { callee, .. }, .. } if callee == "decide")
                })
        })
        .unwrap();
    assert_eq!(
        carrier
            .params
            .iter()
            .skip(1)
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>(),
        ["inner", "saved"]
    );
    assert!(matches!(carrier.body.first(), Some(NirStmt::If { .. })));
    assert!(matches!(&carrier.body[1], NirStmt::Let { name, .. } if name == "staged"));
    assert!(matches!(&carrier.body[2], NirStmt::Let { name, .. } if name == "saved"));
    assert!(
        matches!(&carrier.body[4], NirStmt::Let { value: NirExpr::Call { callee, args }, .. }
        if callee == "decide" && args == &[NirExpr::Var("inner".into()), NirExpr::Var("staged".into()), NirExpr::Var("saved".into())])
    );
    assert!(module
        .functions
        .iter()
        .filter(|f| generated.contains(&f.name))
        .any(|f| f.params.iter().any(|p| p.name == "staged")));
    let observer = module
        .functions
        .iter()
        .find(|f| f.name == "observe")
        .unwrap();
    assert!(!observer
        .body
        .iter()
        .any(|stmt| matches!(stmt, NirStmt::Let { name, .. } if name == "staged")));
}

#[test]
fn staged_effectful_scalar_selections_preserve_both_retention_levels_types_and_hygiene() {
    for kind in ["bool", "i32", "i64", "f32", "f64"] {
        for (outer_else, child_else) in [(false, false), (false, true), (true, false), (true, true)]
        {
            let leaf = format!("let saved: {kind} = work(__nuis_effect_call_value_0);");
            let child = if child_else {
                format!("if inner {{}} else {{ {leaf} }}")
            } else {
                format!("if inner {{ {leaf} }}")
            };
            let prefix = format!("let saved: {kind} = work(saved); let __nuis_effect_call_value_0: {kind} = work(saved); {child}");
            let body = if outer_else {
                format!("if gate {{}} else {{ {prefix} }}")
            } else {
                format!("if gate {{ {prefix} }}")
            };
            let source = format!("mod cpu Main {{
                fn __nuis_effect_call_arm_0() -> i64 {{ return 0; }}
                fn work(v: {kind}) -> {kind} {{ print(70); return v; }}
                fn observe(gate: bool, inner: bool, saved: {kind}) -> {kind} {{ {body} return saved; }}
                fn main() -> i64 {{ return 0; }} }}");
            let (module, generated) = rewrite(&source);
            assert_eq!(generated.len(), 4, "{kind}/{outer_else}/{child_else}");
            assert!(!generated.contains("__nuis_effect_call_arm_0"));
            for function in module
                .functions
                .iter()
                .filter(|f| generated.contains(&f.name))
            {
                assert_eq!(function.return_type, Some(scalar_type(kind)));
                assert!(matches!(function.body.first(), Some(NirStmt::If { .. })));
                assert_eq!(
                    function
                        .params
                        .iter()
                        .map(|p| &p.name)
                        .collect::<BTreeSet<_>>()
                        .len(),
                    function.params.len()
                );
            }
        }
    }
}

#[test]
fn staged_effectful_scalar_selections_reject_child_exports_other_writes_suffixes_and_signatures() {
    for body in [
        "let staged = work(saved); if inner { let staged = work(staged); }",
        "let other = work(other); if inner { let saved = work(other); }",
        "let staged = work(saved); if inner { let saved = work(staged); } let other = work(saved); let saved = work(saved);",
        "const staged: i64 = work(saved); if inner { let saved = work(staged); }",
        "print(70); if inner { let saved = work(saved); }",
        "let staged = work(saved); if inner { return work(staged); }",
        "let staged = work(saved); if inner { let saved = work(staged); } else { let other = work(staged); }",
    ] {
        let mut module = parse_nuis_module(&source(&format!("if gate {{ {body} }}"), "")).unwrap();
        let before = module.clone();
        let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
        assert!(outline(&mut module, &mut names).is_empty(), "{body}");
        assert_eq!(module, before);
    }
    let original = parse_nuis_module(&source(
        "if gate { let staged = work(saved); if inner { let saved = work(staged); } }",
        "",
    ))
    .unwrap();
    for mutation in [
        "async",
        "reference",
        "optional",
        "generic",
        "resource",
        "pure",
    ] {
        let mut module = original.clone();
        let work = module
            .functions
            .iter_mut()
            .find(|f| f.name == "work")
            .unwrap();
        match mutation {
            "async" => work.is_async = true,
            "reference" => work.params[0].ty.is_ref = true,
            "optional" => work.params[0].ty.is_optional = true,
            "generic" => work.params[0].ty.generic_args.push(scalar_type("i64")),
            "resource" => work.params[0].ty.name = "Buffer".into(),
            "pure" => {
                work.body.remove(0);
            }
            _ => unreachable!(),
        }
        let before = module.clone();
        let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
        assert!(outline(&mut module, &mut names).is_empty(), "{mutation}");
        assert_eq!(module, before);
    }
}

#[test]
fn staged_effectful_scalar_selections_share_prefix_depth_node_expression_and_capture_limits() {
    for count in [15, 16] {
        let prefix = (0..16)
            .map(|i| format!("let local{i} = work(saved);"))
            .collect::<String>();
        let params = (0..count)
            .map(|i| format!(", v{i}: i64"))
            .collect::<String>();
        let mut values = (0..16)
            .map(|i| format!("local{i}"))
            .chain((0..count).map(|i| format!("v{i}")))
            .collect::<Vec<_>>();
        while values.len() > 1 {
            values = values
                .chunks(2)
                .map(|pair| {
                    if pair.len() == 1 {
                        pair[0].clone()
                    } else {
                        format!("({} + {})", pair[0], pair[1])
                    }
                })
                .collect();
        }
        let leaf = format!("let saved = work({});", values[0]);
        let body = format!("if gate {{ {prefix}if inner {{ {leaf} }} else {{ {leaf} }} }}");
        let (_, generated) = rewrite(&source(&body, &params));
        // Private prefix bindings do not count at the ancestor, but forwarding
        // 16 locals plus 16 external values exceeds the child's own limit.
        assert_eq!(generated.len(), if count == 15 { 4 } else { 0 });
    }
    for count in [16, 17] {
        let body = format!(
            "if gate {{ {}if inner {{ let saved = work(staged); }} }}",
            "let staged = work(saved);".repeat(count)
        );
        let (_, generated) = rewrite(&source(&body, ""));
        assert_eq!(generated.len(), if count == 16 { 4 } else { 0 });
    }
    for depth in [8, 9] {
        let mut body = "let saved = work(saved);".to_owned();
        for level in 0..depth - 1 {
            body = format!("let staged{level} = work(saved); if inner {{ {body} }}");
        }
        let (_, generated) = rewrite(&source(&format!("if gate {{ {body} }}"), ""));
        assert_eq!(generated.len(), if depth == 8 { 16 } else { 0 });
    }
    for count in [29, 30] {
        let params = (0..count)
            .map(|i| format!(", v{i}: i64"))
            .collect::<String>();
        let sum = (0..count)
            .map(|i| format!("v{i}"))
            .collect::<Vec<_>>()
            .join(" + ");
        let body = format!(
            "if gate {{ let staged = work({sum}); if inner {{ let saved = work(staged); }} }}"
        );
        let (_, generated) = rewrite(&source(&body, &params));
        // saved retention and inner predicate are also ancestor captures.
        assert_eq!(generated.len(), if count == 29 { 4 } else { 0 });
    }
    for expressions in [7, 8] {
        let module = parse_nuis_module(&source(
            "if gate { let staged = work(saved); if inner { let saved = work(staged); } }",
            "",
        ))
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
        } = &observer.body[0]
        else {
            unreachable!();
        };
        let scope = observer
            .params
            .iter()
            .map(|p| (p.name.clone(), p.ty.clone()))
            .collect();
        let signatures = BTreeMap::from([(
            "work".into(),
            (vec![scalar_type("i64")], scalar_type("i64")),
        )]);
        let mut budget = nested::Budget {
            nodes: 64,
            expressions,
        };
        // Root/child conditions, prefix call+arg, live call+arg and both retained
        // values consume eight expression nodes together.
        assert_eq!(
            nested::branch(
                condition,
                then_body,
                else_body,
                &scope,
                &signatures,
                &mut budget,
                0
            )
            .is_some(),
            expressions == 8
        );
        if expressions == 8 {
            for nodes in [5, 6] {
                let mut budget = nested::Budget {
                    nodes,
                    expressions: 256,
                };
                assert_eq!(
                    nested::branch(
                        condition,
                        then_body,
                        else_body,
                        &scope,
                        &signatures,
                        &mut budget,
                        0
                    )
                    .is_some(),
                    nodes == 6
                );
            }
        }
    }
}
