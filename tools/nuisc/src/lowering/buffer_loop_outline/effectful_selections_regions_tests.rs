use super::*;
use crate::frontend::parse_nuis_module;

fn source(body: &str, params: &str) -> String {
    format!(
        "mod cpu Main {{
        fn work(v: i64) -> i64 {{ print(70); return v; }}
        fn pure(v: i64) -> i64 {{ return v; }}
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
fn sequential_effectful_scalar_regions_capture_rhs_versions_and_keep_private_staging() {
    let body = "if gate {
        let staged = work(saved); let saved = saved + staged;
        let staged = work(saved); let saved = saved + staged;
    }";
    let (module, generated) = rewrite(&source(body, ""));
    assert_eq!(generated.len(), 2);
    let live = module
        .functions
        .iter()
        .find(|f| generated.contains(&f.name) && f.body.len() > 2)
        .unwrap();
    assert_eq!(
        live.params
            .iter()
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>(),
        [live.params[0].name.as_str(), "saved"]
    );
    assert!(matches!(live.body.first(), Some(NirStmt::If { .. })));
    assert!(
        matches!(&live.body[1], NirStmt::Let { name, value: NirExpr::Call { callee, .. }, .. }
        if name == "staged" && callee == "work")
    );
    assert!(
        matches!(&live.body[3], NirStmt::Let { value: NirExpr::Call { args, .. }, .. }
        if args == &[NirExpr::Var("saved".into())])
    );
    assert!(
        matches!(live.body.last(), Some(NirStmt::Return(Some(NirExpr::Var(name)))) if name == "saved")
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
    // An initial read of a same-name private binding still captures its input.
    let (module, _) = rewrite(&source(
        "if gate { let staged = work(other); let staged = work(staged); let saved = staged; }",
        "",
    ));
    let live = module
        .functions
        .iter()
        .find(|f| f.name.starts_with("__nuis_effect_call_arm_") && f.body.len() > 2)
        .unwrap();
    assert_eq!(live.params[1].name, "other");
}

#[test]
fn sequential_effectful_scalar_regions_preserve_nested_masks_exact_types_and_hygiene() {
    for kind in ["bool", "i32", "i64", "f32", "f64"] {
        for else_arm in [false, true] {
            let region = format!("let __nuis_effect_call_value_0: {kind} = work(saved); let saved: {kind} = work(__nuis_effect_call_value_0);");
            let body = if else_arm {
                format!("if gate {{}} else {{ if inner {{ {region} }} }}")
            } else {
                format!("if gate {{ if inner {{ {region} }} }}")
            };
            let source = format!("mod cpu Main {{
                fn __nuis_effect_call_arm_0() -> i64 {{ return 0; }}
                fn work(v: {kind}) -> {kind} {{ print(70); return v; }}
                fn observe(gate: bool, inner: bool, saved: {kind}) -> {kind} {{ {body} return saved; }}
                fn main() -> i64 {{ return 0; }} }}");
            let (module, generated) = rewrite(&source);
            assert_eq!(generated.len(), 4, "{kind}/{else_arm}");
            assert!(!generated.contains("__nuis_effect_call_arm_0"));
            for helper in module
                .functions
                .iter()
                .filter(|f| generated.contains(&f.name))
            {
                assert_eq!(helper.return_type, Some(scalar_type(kind)));
                assert!(matches!(helper.body.first(), Some(NirStmt::If { .. })));
                assert!(!helper
                    .params
                    .iter()
                    .any(|p| p.name == "__nuis_effect_call_value_0"));
            }
        }
    }
}

#[test]
fn sequential_effectful_scalar_regions_reject_exports_other_writes_exits_and_unproved_work() {
    for region in [
        "let staged = work(saved); let fresh = staged;",
        "let other = work(other); let saved = other;",
        "let staged = work(saved); return staged;",
        "const staged: i64 = work(saved); let saved = staged;",
        "print(70); let saved = work(saved);",
        "let other = work(other); if inner { let saved = other; }",
        "let staged = work(saved); let staged = true; let saved = work(saved);",
        "let staged = pure(saved); let saved = pure(staged);",
    ] {
        let mut module =
            parse_nuis_module(&source(&format!("if gate {{ {region} }}"), "")).unwrap();
        let before = module.clone();
        let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
        assert!(outline(&mut module, &mut names).is_empty(), "{region}");
        assert_eq!(module, before);
    }
    let original = parse_nuis_module(&source(
        "if gate { let staged = work(saved); let saved = staged; }",
        "",
    ))
    .unwrap();
    for mutation in ["async", "reference", "optional", "generic", "resource"] {
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
            _ => unreachable!(),
        }
        let before = module.clone();
        let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
        assert!(outline(&mut module, &mut names).is_empty(), "{mutation}");
        assert_eq!(module, before);
    }
}

#[test]
fn sequential_effectful_scalar_regions_bound_statements_shared_work_and_external_captures() {
    for count in [31, 32] {
        let mut values = vec!["1".to_owned(); count];
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
        let region = format!("let staged = work({}); let saved = staged;", values[0]);
        let inner = format!("if inner {{ {region} }} else {{ {region} }}");
        let (_, generated) = rewrite(&source(
            &format!("if gate {{ {inner} }} else {{ {inner} }}"),
            "",
        ));
        // 255 original expression nodes fit the shared 256-node budget;
        // increasing each leaf by two nodes cannot reset the budget per leaf.
        assert_eq!(generated.len(), if count == 31 { 6 } else { 0 });
    }
    for count in [15, 16] {
        let region = format!(
            "{}let saved = staged;",
            "let staged = work(saved);".repeat(count - 1)
        );
        let inner = format!("if inner {{ {region} }} else {{ {region} }}");
        let (_, generated) = rewrite(&source(
            &format!("if gate {{ {inner} }} else {{ {inner} }}"),
            "",
        ));
        // Three branches plus four 15-statement leaves fit 64 shared nodes;
        // 16-statement leaves do not, despite each leaf fitting its own limit.
        assert_eq!(generated.len(), if count == 15 { 6 } else { 0 });
    }
    for count in [16, 17] {
        let region = format!(
            "{}let saved = staged;",
            "let staged = work(saved);".repeat(count - 1)
        );
        let (_, generated) = rewrite(&source(&format!("if gate {{ {region} }}"), ""));
        assert_eq!(generated.len(), if count == 16 { 2 } else { 0 });
    }
    for count in [30, 31] {
        let params = (0..count)
            .map(|i| format!(", v{i}: i64"))
            .collect::<String>();
        let sum = (0..count)
            .map(|i| format!("v{i}"))
            .collect::<Vec<_>>()
            .join(" + ");
        let region = format!("let staged = work({sum}); let saved = staged;");
        let (_, generated) = rewrite(&source(&format!("if gate {{ {region} }}"), &params));
        // The retention arm adds saved to the capture set.
        assert_eq!(generated.len(), if count == 30 { 2 } else { 0 });
    }
    let mut budget = nested::Budget {
        nodes: 64,
        expressions: 3,
    };
    let statements = [
        NirStmt::Let {
            name: "staged".into(),
            ty: None,
            value: NirExpr::Call {
                callee: "work".into(),
                args: vec![NirExpr::Var("saved".into())],
            },
        },
        NirStmt::Let {
            name: "saved".into(),
            ty: None,
            value: NirExpr::Var("staged".into()),
        },
    ];
    let scope = BTreeMap::from([("saved".into(), scalar_type("i64"))]);
    let signatures = BTreeMap::from([(
        "work".into(),
        (vec![scalar_type("i64")], scalar_type("i64")),
    )]);
    assert!(prove(&statements, &scope, &signatures, &mut budget).is_some());
    budget.expressions = 2;
    assert!(prove(&statements, &scope, &signatures, &mut budget).is_none());
    budget.expressions = 256;
    budget.nodes = 1;
    assert!(prove(&statements, &scope, &signatures, &mut budget).is_none());
}
