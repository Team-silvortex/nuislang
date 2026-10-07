use super::*;
use crate::frontend::parse_nuis_module;

fn rewrite(source: &str) -> (NirModule, BTreeSet<String>) {
    let mut module = parse_nuis_module(source).unwrap();
    let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
    let generated = outline(&mut module, &mut names);
    crate::nir_verify::verify_nir_module(&module).unwrap();
    (module, generated)
}

#[test]
fn nested_effectful_scalar_selections_capture_inner_predicates_after_ancestor_guards() {
    let source = "mod cpu Main {
        fn decide(gate: bool) -> bool { print(98); return gate; }
        fn work(value: i64) -> i64 { print(70); return value; }
        fn observe(outer: bool, inner: bool, saved: i64) -> i64 {
            if decide(outer) { if decide(inner) { let saved = work(saved); } }
            return saved;
        } fn main() -> i64 { return 0; } }";
    let (module, generated) = rewrite(source);
    assert_eq!(generated.len(), 4);
    let parent = module
        .functions
        .iter()
        .find(|f| f.name == "observe")
        .unwrap();
    assert_eq!(
        parent
            .body
            .iter()
            .filter(|stmt| matches!(stmt,
        NirStmt::Let { value: NirExpr::Call { callee, .. }, .. } if callee == "decide"))
            .count(),
        1
    );
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
    assert!(carrier
        .params
        .iter()
        .any(|p| p.name == "inner" && p.ty == scalar_type("bool")));
    assert!(
        matches!(carrier.body.first(), Some(NirStmt::If { then_body, else_body, .. })
        if matches!(then_body.as_slice(), [NirStmt::Return(Some(NirExpr::Int(0)))]) && else_body.is_empty())
    );
    assert!(
        matches!(&carrier.body[1], NirStmt::Let { value: NirExpr::Call { callee, .. }, .. } if callee == "decide")
    );
    // Predicate effects alone still require masking, even with pure leaves.
    let (_, generated) = rewrite(&source.replace("print(70); ", ""));
    assert_eq!(generated.len(), 4);
}

#[test]
fn nested_effectful_scalar_selections_keep_exact_types_polarities_and_name_hygiene() {
    for kind in ["bool", "i32", "i64", "f32", "f64"] {
        for (outer_else, inner_else) in [(false, false), (false, true), (true, false), (true, true)]
        {
            let update = format!("let __nuis_effect_call_gate_0: {kind} = work(value);");
            let child = if inner_else {
                format!("if decide(inner) {{}} else {{ {update} }}")
            } else {
                format!("if decide(inner) {{ {update} }}")
            };
            let body = if outer_else {
                format!("if decide(outer) {{}} else {{ {child} }}")
            } else {
                format!("if decide(outer) {{ {child} }}")
            };
            let source = format!("mod cpu Main {{
                fn __nuis_effect_call_arm_0() -> i64 {{ return 0; }}
                fn decide(gate: bool) -> bool {{ print(98); return gate; }}
                fn work(value: {kind}) -> {kind} {{ print(70); return value; }}
                fn observe(outer: bool, inner: bool, value: {kind}, __nuis_effect_call_gate_0: {kind}) -> {kind} {{
                    {body} return __nuis_effect_call_gate_0;
                }} fn main() -> i64 {{ return 0; }} }}");
            let (module, generated) = rewrite(&source);
            assert_eq!(generated.len(), 4, "{kind}/{outer_else}/{inner_else}");
            assert!(!generated.contains("__nuis_effect_call_arm_0"));
            for function in module
                .functions
                .iter()
                .filter(|f| generated.contains(&f.name))
            {
                assert_eq!(function.return_type, Some(scalar_type(kind)));
                assert_ne!(function.params[0].name, "__nuis_effect_call_gate_0");
                assert!(function.params.iter().all(|p| scalar(&p.ty)));
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
fn nested_effectful_scalar_selections_reject_unproved_exports_effects_and_signatures() {
    for body in [
        "if outer { if inner { let fresh = work(saved); } else { let fresh = work(saved); } }",
        "if outer { if inner { const saved: i64 = work(saved); } }",
        "if outer { if inner { return work(saved); } }",
        "if outer { if inner { print(70); let saved = work(saved); } }",
        "if outer { if inner { let saved = work(saved); } else { let other = work(saved); } }",
        "while outer { if outer { if inner { let saved = work(saved); } } }",
        "if outer { if inner { let saved = pure(saved); } }",
    ] {
        let source = format!(
            "mod cpu Main {{ fn work(v: i64) -> i64 {{ print(70); return v; }}
            fn pure(v: i64) -> i64 {{ return v; }}
            fn observe(outer: bool, inner: bool, saved: i64, other: i64) -> i64 {{
                {body} return saved;
            }} fn main() -> i64 {{ return 0; }} }}"
        );
        let mut module = parse_nuis_module(&source).unwrap();
        let before = module.clone();
        let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
        assert!(outline(&mut module, &mut names).is_empty(), "{body}");
        assert_eq!(module, before);
    }
    let original = parse_nuis_module(
        "mod cpu Main {
        fn work(v: i64) -> i64 { print(70); return v; }
        fn observe(outer: bool, inner: bool, saved: i64) -> i64 {
            if outer { if inner { let saved = work(saved); } } return saved;
        } fn main() -> i64 { return 0; } }",
    )
    .unwrap();
    for mutation in [
        "async",
        "reference",
        "optional",
        "generic",
        "resource",
        "destination",
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
            "destination" => {
                module
                    .functions
                    .iter_mut()
                    .find(|f| f.name == "observe")
                    .unwrap()
                    .params[2]
                    .ty
                    .is_ref = true
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
fn nested_effectful_scalar_selections_bound_depth_tree_expression_and_predicate_captures() {
    let source = |body: &str, params: &str| {
        format!(
            "mod cpu Main {{
        fn work(v: i64) -> i64 {{ print(70); return v; }}
        fn observe(gate: bool, saved: i64{params}) -> i64 {{ {body} return saved; }}
        fn main() -> i64 {{ return 0; }} }}"
        )
    };
    for depth in [8, 9] {
        let mut body = "let saved = work(saved);".to_owned();
        for _ in 0..depth {
            body = format!("if gate {{ {body} }}");
        }
        let (_, generated) = rewrite(&source(&body, ""));
        assert_eq!(generated.len(), if depth == 8 { 16 } else { 0 });
    }
    for depth in [5, 6] {
        let mut body = "let saved = work(saved);".to_owned();
        for _ in 0..depth {
            body = format!("if gate {{ {body} }} else {{ {body} }}");
        }
        let (_, generated) = rewrite(&source(&body, ""));
        assert_eq!(generated.len(), if depth == 5 { 62 } else { 0 });
    }
    for count in [126, 127] {
        let mut module = parse_nuis_module(&source(
            "if gate { if gate { let saved = work(saved); } }",
            "",
        ))
        .unwrap();
        let mut values = vec![NirExpr::Int(1); count];
        while values.len() > 1 {
            values = values
                .chunks(2)
                .map(|p| {
                    if p.len() == 1 {
                        p[0].clone()
                    } else {
                        NirExpr::Binary {
                            op: NirBinaryOp::Add,
                            lhs: Box::new(p[0].clone()),
                            rhs: Box::new(p[1].clone()),
                        }
                    }
                })
                .collect();
        }
        let mut budget = Budget {
            nodes: 64,
            expressions: 2 * count - 2,
        };
        assert!(inspect(&values[0], &BTreeMap::new(), &BTreeMap::new(), &mut budget).is_none());
        let function = module
            .functions
            .iter_mut()
            .find(|f| f.name == "observe")
            .unwrap();
        let NirStmt::If { then_body, .. } = &mut function.body[0] else {
            unreachable!()
        };
        let NirStmt::If { then_body, .. } = &mut then_body[0] else {
            unreachable!()
        };
        let NirStmt::Let { value, .. } = &mut then_body[0] else {
            unreachable!()
        };
        *value = NirExpr::Call {
            callee: "work".into(),
            args: vec![values[0].clone()],
        };
        let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
        assert_eq!(
            outline(&mut module, &mut names).len(),
            if count == 126 { 4 } else { 0 }
        );
    }
    for count in [30, 31] {
        let params = (0..count)
            .map(|i| format!(", b{i}: bool"))
            .collect::<String>();
        let predicate = (0..count)
            .map(|i| format!("b{i}"))
            .reduce(|a, b| format!("({a} == {b})"))
            .unwrap();
        let body = format!("if gate {{ if {predicate} {{ let saved = work(saved); }} }}");
        let (_, generated) = rewrite(&source(&body, &params));
        assert_eq!(generated.len(), if count == 30 { 4 } else { 0 });
    }
}
