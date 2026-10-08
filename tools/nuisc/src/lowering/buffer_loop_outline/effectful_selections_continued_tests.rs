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
fn continued_effectful_scalar_selections_read_child_merge_before_ordered_private_suffixes() {
    let (module, generated) = rewrite(&source(
        "if gate { let staged = work(saved); let saved = saved + staged;
            if decide(inner, staged, saved) { let saved = work(staged); }
            let staged = work(saved + staged); let saved = work(staged + other); }",
        "",
    ));
    assert_eq!(generated.len(), 4);
    let carrier = module
        .functions
        .iter()
        .find(|f| {
            generated.contains(&f.name)
                && f.body.iter().any(|s| {
                    matches!(s, NirStmt::Let {
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
        ["inner", "other", "saved"]
    );
    let merge = carrier
        .body
        .iter()
        .position(|s| {
            matches!(s, NirStmt::If {
        then_body, else_body, .. } if !then_body.is_empty() && !else_body.is_empty())
        })
        .unwrap();
    assert!(matches!(&carrier.body[merge + 1], NirStmt::Let {
        name, value: NirExpr::Call { callee, args }, .. }
        if name == "staged" && callee == "work" && args == &[NirExpr::Binary {
            op: NirBinaryOp::Add, lhs: Box::new(NirExpr::Var("saved".into())),
            rhs: Box::new(NirExpr::Var("staged".into())) }]));
    assert!(matches!(&carrier.body[merge + 2], NirStmt::Let { name, .. } if name == "saved"));
    assert!(
        matches!(&carrier.body[merge + 3], NirStmt::Return(Some(NirExpr::Var(name))) if name == "saved")
    );
    let observer = module
        .functions
        .iter()
        .find(|f| f.name == "observe")
        .unwrap();
    assert!(!observer
        .body
        .iter()
        .any(|s| matches!(s, NirStmt::Let { name, .. } if name == "staged")));
    // Effectful suffixes still require guards when the child itself is pure.
    let (_, generated) = rewrite(&source(
        "if gate { if inner { let saved = saved + 1; } let saved = work(saved); }",
        "",
    ));
    assert_eq!(generated.len(), 4);
}

#[test]
fn continued_effectful_scalar_selections_keep_exact_types_optional_prefixes_polarities_and_hygiene()
{
    for kind in ["bool", "i32", "i64", "f32", "f64"] {
        for prefix in [false, true] {
            for (outer_else, child_else) in
                [(false, false), (false, true), (true, false), (true, true)]
            {
                let leaf = format!("let saved: {kind} = work(saved);");
                let child = if child_else {
                    format!("if inner {{}} else {{ {leaf} }}")
                } else {
                    format!("if inner {{ {leaf} }}")
                };
                let body = if prefix {
                    format!("let __nuis_effect_call_value_0 = work(saved); {child}
                        let saved: {kind} = work(__nuis_effect_call_value_0); let saved: {kind} = work(saved);")
                } else {
                    format!("{child} let saved: {kind} = work(saved);")
                };
                let body = if outer_else {
                    format!("if gate {{}} else {{ {body} }}")
                } else {
                    format!("if gate {{ {body} }}")
                };
                let text = format!("mod cpu Main {{
                    fn __nuis_effect_call_arm_0() -> i64 {{ return 0; }}
                    fn work(v: {kind}) -> {kind} {{ print(70); return v; }}
                    fn observe(gate: bool, inner: bool, saved: {kind}) -> {kind} {{ {body} return saved; }}
                    fn main() -> i64 {{ return 0; }} }}");
                let (module, generated) = rewrite(&text);
                assert_eq!(
                    generated.len(),
                    4,
                    "{kind}/{prefix}/{outer_else}/{child_else}"
                );
                assert!(!generated.contains("__nuis_effect_call_arm_0"));
                for f in module
                    .functions
                    .iter()
                    .filter(|f| generated.contains(&f.name))
                {
                    assert_eq!(f.return_type, Some(scalar_type(kind)));
                    assert!(matches!(f.body.first(), Some(NirStmt::If { .. })));
                    assert_eq!(
                        f.params
                            .iter()
                            .map(|p| &p.name)
                            .collect::<BTreeSet<_>>()
                            .len(),
                        f.params.len()
                    );
                }
            }
        }
    }
}

#[test]
fn continued_effectful_scalar_selections_reject_other_writes_exports_exits_resources_and_extra_children(
) {
    for body in [
        "if inner { let saved = work(saved); } let fresh = work(saved);",
        "let staged = work(saved); if inner { let staged = work(staged); } let saved = work(staged);",
        "let other = work(other); if inner { let saved = work(saved); } let saved = work(saved);",
        "if inner { let saved = work(saved); } let other = work(saved); let saved = work(saved);",
        "if inner { let saved = work(saved); } return saved;",
        "if inner { let saved = work(saved); } const saved: i64 = work(saved);",
        "if inner { let saved = work(saved); } print(70); let saved = work(saved);",
        "if inner { let saved = work(saved); } let saved = true;",
        "if inner { let saved = work(saved); } if inner { let other = work(saved); } let saved = work(saved);",
        "if inner { let saved = work(saved); } else { let other = work(other); } let saved = work(saved);",
    ] {
        let mut module = parse_nuis_module(&source(&format!("if gate {{ {body} }}"), "")).unwrap();
        let before = module.clone();
        let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
        assert!(outline(&mut module, &mut names).is_empty(), "{body}");
        assert_eq!(module, before);
    }
    let escape = source(
        "if gate { if inner { let local = work(saved); let saved = local; }
        let saved = work(local); }",
        "",
    );
    assert!(parse_nuis_module(&escape)
        .unwrap_err()
        .contains("unknown value `local`"));
    let mut escaped = parse_nuis_module(&source(
        "if gate {
        if inner { let local = work(saved); let saved = local; } let saved = work(saved); }",
        "",
    ))
    .unwrap();
    let observer = escaped
        .functions
        .iter_mut()
        .find(|f| f.name == "observe")
        .unwrap();
    let NirStmt::If { then_body, .. } = &mut observer.body[0] else {
        unreachable!();
    };
    let NirStmt::Let {
        value: NirExpr::Call { args, .. },
        ..
    } = &mut then_body[1]
    else {
        unreachable!();
    };
    args[0] = NirExpr::Var("local".into());
    let before = escaped.clone();
    let mut names = escaped.functions.iter().map(|f| f.name.clone()).collect();
    assert!(outline(&mut escaped, &mut names).is_empty());
    assert_eq!(escaped, before);
    let original = parse_nuis_module(&source(
        "if gate { if inner { let saved = work(saved); } let saved = work(saved); }",
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
fn continued_effectful_scalar_selections_share_suffix_prefix_depth_capture_and_work_limits() {
    for count in [16, 17] {
        for prefix_limit in [false, true] {
            let prefix = "let staged = work(saved);".repeat(if prefix_limit { count } else { 1 });
            let suffix = "let saved = work(saved);".repeat(if prefix_limit { 1 } else { count });
            let (_, generated) = rewrite(&source(
                &format!(
                    "if gate {{ {prefix} if inner {{ let saved = work(staged); }} {suffix} }}"
                ),
                "",
            ));
            assert_eq!(generated.len(), if count == 16 { 4 } else { 0 });
        }
    }
    for depth in [8, 9] {
        let mut body = "let saved = work(saved);".to_owned();
        for _ in 0..depth - 1 {
            body = format!("if inner {{ {body} }} let saved = work(saved);");
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
        let (_, generated) = rewrite(&source(
            &format!(
                "if gate {{ let staged = work(saved); if inner {{ let saved = work(staged); }}
                let saved = work(saved + ({sum})); }}"
            ),
            &params,
        ));
        assert_eq!(generated.len(), if count == 29 { 4 } else { 0 });
    }
    for count in [14, 15] {
        let leaf = "let saved = work(saved);".repeat(count);
        let arm = format!("if inner {{ {leaf} }} else {{ {leaf} }} let saved = work(saved);");
        let (_, generated) = rewrite(&source(
            &format!("if gate {{ {arm} }} else {{ {arm} }}"),
            "",
        ));
        // Two 15+15 child leaves plus their suffixes exceed 64 shared nodes.
        assert_eq!(generated.len(), if count == 14 { 6 } else { 0 });
    }
    let module = parse_nuis_module(&source(
        "if gate { let staged = work(saved); if inner { let saved = work(staged); }
            let saved = work(saved); }",
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
    for expressions in [9, 10] {
        let mut budget = nested::Budget {
            nodes: 64,
            expressions,
            logical_edges: 32,
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
            expressions == 10
        );
    }
    for nodes in [6, 7] {
        let mut budget = nested::Budget {
            nodes,
            expressions: 256,
            logical_edges: 32,
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
            nodes == 7
        );
    }
}
