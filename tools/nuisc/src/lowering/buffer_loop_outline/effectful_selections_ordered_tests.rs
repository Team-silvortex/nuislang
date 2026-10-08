use super::*;
use crate::frontend::parse_nuis_module;

fn source(body: &str, params: &str) -> String {
    format!("mod cpu Main {{
        fn work(v: i64) -> i64 {{ print(70); return v; }}
        fn decide(gate: bool, value: i64) -> bool {{ print(98); return gate; }}
        fn observe(gate: bool, first: bool, second: bool, third: bool, saved: i64, other: i64{params}) -> i64 {{
            {body} return saved; }} fn main() -> i64 {{ return 0; }} }}")
}

fn rewrite(text: &str) -> (NirModule, BTreeSet<String>) {
    let mut module = parse_nuis_module(text).unwrap();
    let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
    let generated = outline(&mut module, &mut names);
    crate::nir_verify::verify_nir_module(&module).unwrap();
    (module, generated)
}

const THREE: &str = "if gate {
    if first { let saved = work(saved); }
    if second { let saved = work(saved); }
    if third { let saved = work(saved); } }";

#[test]
fn ordered_effectful_scalar_selections_merge_adjacent_children_before_later_predicates_and_final_results(
) {
    let (module, generated) = rewrite(&source(
        "if gate { let staged = work(saved);
        if first { let saved = work(staged); }
        if decide(second, saved) { let saved = work(saved + staged); }
        let staged = work(saved);
        if decide(third, saved) {} else { let saved = work(staged); } }",
        "",
    ));
    assert_eq!(generated.len(), 8);
    let carrier = module
        .functions
        .iter()
        .find(|f| {
            generated.contains(&f.name)
                && f.body
                    .iter()
                    .filter(|s| {
                        matches!(s, NirStmt::Let { value: NirExpr::Call { callee, .. }, .. }
            if callee == "decide")
                    })
                    .count()
                    == 2
        })
        .unwrap();
    assert_eq!(
        carrier
            .params
            .iter()
            .skip(1)
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>(),
        ["first", "saved", "second", "third"]
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
    assert_eq!(merges.len(), 3);
    assert!(matches!(&carrier.body[merges[0] + 1], NirStmt::Let {
        value: NirExpr::Call { callee, args }, .. }
        if callee == "decide" && args == &[NirExpr::Var("second".into()), NirExpr::Var("saved".into())]));
    assert!(matches!(&carrier.body[merges[1] + 1], NirStmt::Let {
        name, value: NirExpr::Call { callee, args }, .. }
        if name == "staged" && callee == "work" && args == &[NirExpr::Var("saved".into())]));
    assert!(
        matches!(&carrier.body[merges[2] + 1], NirStmt::Return(Some(NirExpr::Var(name))) if name == "saved")
    );
    for body in [
        "if gate { if first { let saved = work(saved); } if second { let saved = work(saved); } let saved = work(saved); }",
        "if gate { if first { let saved = work(saved); } let staged = work(saved); if second { let saved = work(saved); } }",
        "if gate { if first { let saved = work(saved); } let staged = work(saved); if second { let saved = work(saved); } let staged = work(saved); if second { let saved = work(saved); } let saved = work(saved); }",
        "if gate { if first { let saved = saved + 1; } if second { let saved = saved + 1; } let saved = work(saved); }",
    ] {
        assert!(!rewrite(&source(body, "")).1.is_empty(), "{body}");
    }
}

#[test]
fn ordered_effectful_scalar_selections_preserve_shape_polarity_exact_types_and_name_hygiene() {
    for kind in ["bool", "i32", "i64", "f32", "f64"] {
        for shape in ["adjacent-final", "mixed-suffix", "paired-final"] {
            for mask in 0..4 {
                let leaf = format!("let saved: {kind} = work(saved);");
                let child = |gate: &str| {
                    if shape == "paired-final" {
                        format!("if {gate} {{ {leaf} }} else {{ {leaf} }}")
                    } else if mask & 1 != 0 {
                        format!("if {gate} {{}} else {{ {leaf} }}")
                    } else {
                        format!("if {gate} {{ {leaf} }}")
                    }
                };
                let body = if shape == "mixed-suffix" {
                    format!(
                        "let __nuis_effect_call_value_0 = work(saved); {}
                        let saved: {kind} = work(saved); {} {} {leaf}",
                        child("first"),
                        child("second"),
                        child("third")
                    )
                } else {
                    format!("{} {} {}", child("first"), child("second"), child("third"))
                };
                let body = if mask & 2 != 0 {
                    format!("if gate {{}} else {{ {body} }}")
                } else {
                    format!("if gate {{ {body} }}")
                };
                let text = format!("mod cpu Main {{
                    fn __nuis_effect_call_arm_0() -> i64 {{ return 0; }}
                    fn work(v: {kind}) -> {kind} {{ print(70); return v; }}
                    fn observe(gate: bool, first: bool, second: bool, third: bool, saved: {kind}) -> {kind} {{
                        {body} return saved; }} fn main() -> i64 {{ return 0; }} }}");
                let (module, generated) = rewrite(&text);
                assert_eq!(generated.len(), 8, "{kind}/{shape}/{mask}");
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
fn ordered_effectful_scalar_selections_reject_other_targets_exports_exits_resources_and_private_escapes_atomically(
) {
    for body in [
        "if first { let saved = work(saved); } if second { let other = work(saved); }",
        "let staged = work(saved); if first { let staged = work(staged); } if second { let staged = work(staged); }",
        "if first { let saved = work(saved); } if second { let saved = work(saved); } let fresh = work(saved);",
        "if first { let saved = work(saved); } let other = work(saved); if second { let saved = work(saved); }",
        "if first { let saved = work(saved); } const saved: i64 = work(saved); if second { let saved = work(saved); }",
        "if first { let saved = work(saved); } return saved; if second { let saved = work(saved); }",
        "if first { let saved = work(saved); } print(70); if second { let saved = work(saved); }",
        "if first { let saved = work(saved); } while second { let saved = work(saved); } if third { let saved = work(saved); }",
        "if first { let saved = work(saved); } if second { let saved = true; }",
    ] {
        let mut module = parse_nuis_module(&source(&format!("if gate {{ {body} }}"), "")).unwrap();
        let before = module.clone();
        let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
        assert!(outline(&mut module, &mut names).is_empty(), "{body}");
        assert_eq!(module, before);
    }
    for mutation in [
        "escape",
        "async",
        "reference",
        "optional",
        "generic",
        "resource",
        "pure",
    ] {
        let mut module = parse_nuis_module(&source(THREE, "")).unwrap();
        if mutation == "escape" {
            let observer = module
                .functions
                .iter_mut()
                .find(|f| f.name == "observe")
                .unwrap();
            let NirStmt::If { then_body, .. } = &mut observer.body[0] else {
                unreachable!()
            };
            let NirStmt::If {
                then_body: child, ..
            } = &mut then_body[0]
            else {
                unreachable!()
            };
            child.insert(
                0,
                NirStmt::Let {
                    name: "child_private".into(),
                    ty: Some(scalar_type("i64")),
                    value: NirExpr::Var("saved".into()),
                },
            );
            let NirStmt::If { condition, .. } = &mut then_body[1] else {
                unreachable!()
            };
            *condition = NirExpr::Call {
                callee: "decide".into(),
                args: vec![
                    NirExpr::Var("second".into()),
                    NirExpr::Var("child_private".into()),
                ],
            };
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
fn ordered_effectful_scalar_selections_share_child_count_stage_capture_depth_node_and_expression_limits(
) {
    for count in [20, 21] {
        let body = "if first { let saved = work(saved); }".repeat(count);
        let (_, generated) = rewrite(&source(&format!("if gate {{ {body} }}"), ""));
        assert_eq!(generated.len(), if count == 20 { 42 } else { 0 });
    }
    for count in [16, 17] {
        let stage = "let staged = work(saved);".repeat(count);
        for body in [format!("{stage} if first {{ let saved = work(saved); }} if second {{ let saved = work(saved); }}"),
            format!("if first {{ let saved = work(saved); }} {stage} if second {{ let saved = work(saved); }}")] {
            assert_eq!(rewrite(&source(&format!("if gate {{ {body} }}"), "")).1.len(), if count == 16 { 6 } else { 0 });
        }
    }
    for depth in [8, 9] {
        let mut body = THREE.to_owned();
        for _ in 0..depth - 2 {
            body = format!("if gate {{ {body} }}");
        }
        assert_eq!(
            rewrite(&source(&body, "")).1.len(),
            if depth == 8 { 20 } else { 0 }
        );
    }
    for count in [27, 28] {
        let params = (0..count)
            .map(|i| format!(", v{i}: i64"))
            .collect::<String>();
        let sum = (0..count)
            .map(|i| format!("v{i}"))
            .collect::<Vec<_>>()
            .join(" + ");
        let body = THREE.replacen("work(saved)", &format!("work(saved + ({sum}))"), 1);
        assert_eq!(
            rewrite(&source(&body, &params)).1.len(),
            if count == 27 { 8 } else { 0 }
        );
    }
    let module = parse_nuis_module(&source(THREE, "")).unwrap();
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
        (10, 256, false),
        (11, 256, true),
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
    let mut budget = nested::Budget {
        nodes: 1,
        expressions: 256,
        logical_edges: 32,
    };
    assert!(prove(then_body, &scope, &signatures, &mut budget, 1).is_none());
    assert_eq!(budget.nodes, 1);
}
