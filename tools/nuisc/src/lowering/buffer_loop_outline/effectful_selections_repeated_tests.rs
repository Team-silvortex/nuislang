use super::*;
use crate::frontend::parse_nuis_module;

fn source(body: &str, params: &str) -> String {
    format!(
        "mod cpu Main {{
        fn work(v: i64) -> i64 {{ print(70); return v; }}
        fn decide(gate: bool, staged: i64, saved: i64) -> bool {{ print(98); return gate; }}
        fn observe(gate: bool, first: bool, second: bool, saved: i64, other: i64{params}) -> i64 {{
            {body} return saved;
        }} fn main() -> i64 {{ return 0; }} }}"
    )
}

fn rewrite(text: &str) -> (NirModule, BTreeSet<String>) {
    let mut module = parse_nuis_module(text).unwrap();
    let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
    let generated = outline(&mut module, &mut names);
    crate::nir_verify::verify_nir_module(&module).unwrap();
    (module, generated)
}

const SIMPLE: &str = "if gate {
    if first { let saved = work(saved); }
    let staged = work(saved);
    if second { let saved = work(staged); }
    let saved = work(saved); }";

#[test]
fn repeated_effectful_scalar_selections_keep_current_merges_and_private_stage_versions() {
    let (module, generated) = rewrite(&source(
        "if gate { let staged = work(saved); let saved = saved + staged;
        if decide(first, staged, saved) { let saved = work(staged); }
        let staged = work(saved + staged); let saved = saved + staged;
        if decide(second, staged, saved) {} else { let saved = work(staged); }
        let staged = work(saved + staged); let saved = work(staged + other); }",
        "",
    ));
    assert_eq!(generated.len(), 6);
    let carrier = module
        .functions
        .iter()
        .find(|f| {
            generated.contains(&f.name) && f.body.iter().filter(|stmt| matches!(stmt,
            NirStmt::Let { value: NirExpr::Call { callee, .. }, .. } if callee == "decide"
        )).count() == 2
        })
        .unwrap();
    assert_eq!(
        carrier
            .params
            .iter()
            .skip(1)
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>(),
        ["first", "other", "saved", "second"]
    );
    let merges = carrier
        .body
        .iter()
        .enumerate()
        .filter_map(|(index, stmt)| {
            matches!(stmt, NirStmt::If { then_body, else_body, .. }
            if !then_body.is_empty() && !else_body.is_empty())
            .then_some(index)
        })
        .collect::<Vec<_>>();
    assert_eq!(merges.len(), 2);
    for index in &merges {
        assert!(matches!(&carrier.body[index + 1], NirStmt::Let {
            name, value: NirExpr::Call { callee, args }, .. }
            if name == "staged" && callee == "work" && args == &[NirExpr::Binary {
                op: NirBinaryOp::Add, lhs: Box::new(NirExpr::Var("saved".into())),
                rhs: Box::new(NirExpr::Var("staged".into())) }]));
    }
    assert!(matches!(&carrier.body[merges[0] + 3], NirStmt::Let {
        value: NirExpr::Call { callee, args }, .. }
        if callee == "decide" && args == &[
            NirExpr::Var("second".into()), NirExpr::Var("staged".into()), NirExpr::Var("saved".into())]));
    let observer = module
        .functions
        .iter()
        .find(|f| f.name == "observe")
        .unwrap();
    assert!(!observer.body.iter().any(|stmt| matches!(stmt,
        NirStmt::Let { name, .. } if name == "staged")));
    let (_, generated) = rewrite(&source(
        "if gate { if first { let saved = saved + 1; } let staged = saved;
        if second { let saved = staged + 1; } let saved = work(saved); }",
        "",
    ));
    assert_eq!(generated.len(), 6);
}

#[test]
fn repeated_effectful_scalar_selections_preserve_exact_types_polarities_prefixes_and_hygiene() {
    for kind in ["bool", "i32", "i64", "f32", "f64"] {
        for prefix in [false, true] {
            for mask in 0..8 {
                let child = |gate: &str, inverted| {
                    let leaf = format!("let saved: {kind} = work(saved);");
                    if inverted {
                        format!("if {gate} {{}} else {{ {leaf} }}")
                    } else {
                        format!("if {gate} {{ {leaf} }}")
                    }
                };
                let prefix = if prefix {
                    "let __nuis_effect_call_value_0 = work(saved);"
                } else {
                    ""
                };
                let body = format!(
                    "{prefix} {} let staged = work(saved); {}
                    let saved: {kind} = work(saved);",
                    child("first", mask & 1 != 0),
                    child("second", mask & 2 != 0)
                );
                let body = if mask & 4 != 0 {
                    format!("if gate {{}} else {{ {body} }}")
                } else {
                    format!("if gate {{ {body} }}")
                };
                let text = format!(
                    "mod cpu Main {{
                    fn __nuis_effect_call_arm_0() -> i64 {{ return 0; }}
                    fn work(v: {kind}) -> {kind} {{ print(70); return v; }}
                    fn observe(gate: bool, first: bool, second: bool, saved: {kind}) -> {kind} {{
                        {body} return saved; }} fn main() -> i64 {{ return 0; }} }}"
                );
                let (module, generated) = rewrite(&text);
                assert_eq!(generated.len(), 6, "{kind}/{mask}");
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
}

#[test]
fn repeated_effectful_scalar_selections_reject_unproved_shapes_and_private_child_exports_atomically(
) {
    for body in [
        "if first { let saved = work(saved); } let staged = work(saved); if second { let saved = work(saved); } let fresh = work(saved);",
        "if first { let saved = work(saved); } let other = work(saved); if second { let saved = work(saved); } let saved = work(saved);",
        "if first { let staged = work(saved); } let saved = work(saved); if second { let saved = work(saved); } let saved = work(saved);",
        "if first { let saved = work(saved); } let staged = work(saved); if second { let saved = work(saved); } else { let other = work(other); } let saved = work(saved);",
        "if first { let saved = work(saved); } let staged = work(saved); if second { let saved = work(saved); } return saved;",
        "if first { let saved = work(saved); } print(70); if second { let saved = work(saved); } let saved = work(saved);",
        "if first { let saved = work(saved); } const staged: i64 = work(saved); if second { let saved = work(saved); } let saved = work(saved);",
        "if first { let saved = work(saved); } let staged = work(saved); if second { let saved = work(saved); } let saved = true;",
    ] {
        let mut module = parse_nuis_module(&source(&format!("if gate {{ {body} }}"), "")).unwrap();
        let before = module.clone();
        let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
        assert!(outline(&mut module, &mut names).is_empty(), "{body}");
        assert_eq!(module, before);
    }
    let escape = SIMPLE
        .replace(
            "let saved = work(saved); }",
            "let local = work(saved); let saved = local; }",
        )
        .replace("let staged = work(saved);", "let staged = work(local);");
    assert!(parse_nuis_module(&source(&escape, ""))
        .unwrap_err()
        .contains("unknown value `local`"));
    for mutation in [
        "escape",
        "predicate-escape",
        "async",
        "reference",
        "optional",
        "generic",
        "resource",
        "pure",
        "second-type",
    ] {
        let mut module = parse_nuis_module(&source(SIMPLE, "")).unwrap();
        if matches!(mutation, "escape" | "predicate-escape" | "second-type") {
            let observer = module
                .functions
                .iter_mut()
                .find(|f| f.name == "observe")
                .unwrap();
            let NirStmt::If { then_body, .. } = &mut observer.body[0] else {
                unreachable!()
            };
            if mutation != "second-type" {
                let NirStmt::If {
                    then_body: first_body,
                    ..
                } = &mut then_body[0]
                else {
                    unreachable!()
                };
                first_body.insert(
                    0,
                    NirStmt::Let {
                        name: "child_private".into(),
                        ty: Some(scalar_type("i64")),
                        value: NirExpr::Var("saved".into()),
                    },
                );
                if mutation == "escape" {
                    let NirStmt::Let {
                        value: NirExpr::Call { args, .. },
                        ..
                    } = &mut then_body[1]
                    else {
                        unreachable!()
                    };
                    args[0] = NirExpr::Var("child_private".into());
                } else {
                    let NirStmt::If { condition, .. } = &mut then_body[2] else {
                        unreachable!()
                    };
                    *condition = NirExpr::Call {
                        callee: "decide".into(),
                        args: vec![
                            NirExpr::Var("second".into()),
                            NirExpr::Var("child_private".into()),
                            NirExpr::Var("saved".into()),
                        ],
                    };
                }
            } else {
                let NirStmt::If { then_body, .. } = &mut then_body[2] else {
                    unreachable!()
                };
                let NirStmt::Let { ty, value, .. } = &mut then_body[0] else {
                    unreachable!()
                };
                *ty = Some(scalar_type("bool"));
                *value = NirExpr::Bool(true);
            }
        } else {
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
        }
        let before = module.clone();
        let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
        assert!(outline(&mut module, &mut names).is_empty(), "{mutation}");
        assert_eq!(module, before);
    }
}

#[test]
fn repeated_effectful_scalar_selections_share_stage_capture_depth_node_and_expression_limits() {
    for count in [16, 17] {
        for selected in 0..3 {
            let prefix = "let staged = work(saved);".repeat(if selected == 0 { count } else { 0 });
            let middle = "let staged = work(saved);".repeat(if selected == 1 { count } else { 1 });
            let suffix = "let saved = work(saved);".repeat(if selected == 2 { count } else { 1 });
            let (_, generated) = rewrite(&source(
                &format!(
                    "if gate {{ {prefix}
                if first {{ let saved = work(saved); }} {middle}
                if second {{ let saved = work(saved); }} {suffix} }}"
                ),
                "",
            ));
            assert_eq!(generated.len(), if count == 16 { 6 } else { 0 });
        }
    }
    for depth in [8, 9] {
        let mut body = SIMPLE.to_owned();
        for _ in 0..depth - 2 {
            body = format!("if gate {{ {body} }}");
        }
        let (_, generated) = rewrite(&source(&body, ""));
        assert_eq!(generated.len(), if depth == 8 { 18 } else { 0 });
    }
    for count in [28, 29] {
        let params = (0..count)
            .map(|i| format!(", v{i}: i64"))
            .collect::<String>();
        let sum = (0..count)
            .map(|i| format!("v{i}"))
            .collect::<Vec<_>>()
            .join(" + ");
        let body = SIMPLE.replace(
            "let saved = work(saved); }",
            &format!("let saved = work(saved + ({sum})); }}"),
        );
        let (_, generated) = rewrite(&source(&body, &params));
        assert_eq!(generated.len(), if count == 28 { 6 } else { 0 });
    }
    for count in [13, 14] {
        let leaf = "let saved = work(saved);".repeat(count);
        let (_, generated) = rewrite(&source(
            &format!(
                "if gate {{
            if first {{ {leaf} }} else {{ {leaf} }} let staged = work(saved);
            if second {{ {leaf} }} else {{ {leaf} }} let saved = work(saved); }}"
            ),
            "",
        ));
        assert_eq!(generated.len(), 6);
        let (_, generated) = rewrite(&source(
            &format!(
                "if gate {{
            let staged = work(saved); if first {{ {leaf} }} else {{ {leaf} }}
            let staged = work(saved); if second {{ {leaf} }} else {{ {leaf} }}
            {} }}",
                "let saved = work(saved);".repeat(6)
            ),
            "",
        ));
        assert_eq!(generated.len(), if count == 13 { 6 } else { 0 });
    }
    let module = parse_nuis_module(&source(SIMPLE, "")).unwrap();
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
        unreachable!()
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
    for (nodes, expressions, accepted) in [
        (9, 256, false),
        (10, 256, true),
        (64, 13, false),
        (64, 14, true),
    ] {
        let mut budget = nested::Budget {
            nodes,
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
            accepted
        );
    }
}
