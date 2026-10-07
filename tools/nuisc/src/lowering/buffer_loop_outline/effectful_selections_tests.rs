use super::*;
use crate::frontend::parse_nuis_module;

const SOURCE: &str =
    include_str!("../../../tests/native_application_bridge/effectful_selected_calls.ns");

fn rewrite(source: &str) -> (NirModule, BTreeSet<String>) {
    let mut module = parse_nuis_module(source).unwrap();
    let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
    let generated = outline(&mut module, &mut names);
    crate::nir_verify::verify_nir_module(&module).unwrap();
    (module, generated)
}

#[test]
fn effectful_selections_capture_only_scalars_and_keep_work_behind_guards() {
    let (module, generated) = rewrite(SOURCE);
    assert_eq!(generated.len(), 2);
    for function in module
        .functions
        .iter()
        .filter(|f| generated.contains(&f.name))
    {
        assert_eq!(function.params[0].ty, scalar_type("bool"));
        assert!(function.params.iter().all(|p| scalar(&p.ty)));
        assert!(
            matches!(function.body.as_slice(), [NirStmt::If { then_body, else_body, .. },
            NirStmt::Return(Some(NirExpr::Call { args, .. }))]
            if matches!(then_body.as_slice(), [NirStmt::Return(Some(NirExpr::Int(0)))])
                && else_body.is_empty() && args.iter().any(|arg| matches!(arg, NirExpr::Call { .. })))
        );
    }
    let observer = module
        .functions
        .iter()
        .find(|f| f.name == "observe")
        .unwrap();
    assert_eq!(
        observer
            .body
            .iter()
            .filter(|stmt| matches!(stmt,
        NirStmt::Let { value: NirExpr::Call { callee, .. }, .. } if callee == "decide"))
            .count(),
        1
    );
    for stmt in &observer.body {
        if let NirStmt::Let {
            value: NirExpr::Call { callee, args },
            ..
        } = stmt
        {
            if generated.contains(callee) {
                assert!(args.iter().all(|arg| matches!(arg, NirExpr::Var(_))));
            }
        }
    }
}

#[test]
fn effectful_selections_support_exact_five_scalar_zeros_and_return_or_const_destinations() {
    for kind in ["bool", "i32", "i64", "f32", "f64"] {
        for shape in ["return", "let", "const"] {
            let body = if shape == "return" {
                "if gate { return work(left); } else { return work(right); }".to_owned()
            } else {
                format!("{shape} result: {kind} = if gate {{ work(left) }} else {{ work(right) }}; return result;")
            };
            let source = format!(
                "mod cpu Main {{ fn work(value: {kind}) -> {kind} {{ print(70); return value; }}
                fn observe(gate: bool, left: {kind}, right: {kind}) -> {kind} {{ {body} }}
                fn main() -> i64 {{ return 0; }} }}"
            );
            let (module, generated) = rewrite(&source);
            assert_eq!(generated.len(), 2, "{kind}/{shape}");
            assert!(module
                .functions
                .iter()
                .filter(|f| generated.contains(&f.name))
                .all(|f| f.return_type == Some(scalar_type(kind))));
        }
    }
}

#[test]
fn effectful_selections_leave_pure_and_loop_routes_and_unsupported_shapes_unchanged() {
    for body in [
        "if gate { let result: i64 = pure(value); } else { let result: i64 = pure(value); } return 0;",
        "while gate { if gate { let result: i64 = work(value); } else { let result: i64 = work(value); } } return 0;",
        "if gate { let result: i64 = work(value); } return 0;",
        "if gate { let a: i64 = work(value); } else { let b: i64 = work(value); } return 0;",
        "if gate { print(70); let result: i64 = work(value); } else { let result: i64 = work(value); } return 0;",
    ] {
        let source = format!("mod cpu Main {{ fn work(value: i64) -> i64 {{ print(70); return value; }}
            fn pure(value: i64) -> i64 {{ return value; }}
            fn observe(gate: bool, value: i64) -> i64 {{ {body} }} fn main() -> i64 {{ return 0; }} }}");
        let mut module = parse_nuis_module(&source).unwrap();
        let before = module.clone();
        let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
        assert!(outline(&mut module, &mut names).is_empty(), "{body}");
        assert_eq!(module, before);
    }
}

#[test]
fn effectful_selection_expression_proof_bounds_depth_nodes_and_exact_signatures() {
    let scope = BTreeMap::from([("value".into(), scalar_type("i64"))]);
    let signatures = BTreeMap::from([(
        "work".into(),
        (vec![scalar_type("i64")], scalar_type("i64")),
    )]);
    for argument in [NirExpr::Bool(true), NirExpr::Var("missing".into())] {
        assert!(inspect(
            &NirExpr::Call {
                callee: "work".into(),
                args: vec![argument]
            },
            &scope,
            &signatures
        )
        .is_none());
    }
    let mut deep = NirExpr::Var("value".into());
    for _ in 0..32 {
        deep = NirExpr::Call {
            callee: "work".into(),
            args: vec![deep],
        };
    }
    assert!(inspect(&deep, &scope, &signatures).is_none());
    let sum = |mut values: Vec<NirExpr>| {
        while values.len() > 1 {
            values = values
                .chunks(2)
                .map(|pair| {
                    if pair.len() == 1 {
                        pair[0].clone()
                    } else {
                        NirExpr::Binary {
                            op: NirBinaryOp::Add,
                            lhs: Box::new(pair[0].clone()),
                            rhs: Box::new(pair[1].clone()),
                        }
                    }
                })
                .collect();
        }
        values.pop().unwrap()
    };
    assert!(inspect(&sum(vec![NirExpr::Int(1); 128]), &scope, &signatures).is_some());
    assert!(inspect(&sum(vec![NirExpr::Int(1); 129]), &scope, &signatures).is_none());
}

#[test]
fn effectful_selections_keep_private_names_hygienic_and_capture_current_bindings() {
    let source = SOURCE
        .replace("value", "__nuis_effect_call_gate_0")
        .replace(
            "fn main()",
            "fn __nuis_effect_call_arm_0() -> i64 { return 0; } fn main()",
        );
    let (module, generated) = rewrite(&source);
    assert_eq!(generated.len(), 2);
    assert!(!generated.contains("__nuis_effect_call_arm_0"));
    for function in module
        .functions
        .iter()
        .filter(|f| generated.contains(&f.name))
    {
        assert_ne!(function.params[0].name, "__nuis_effect_call_gate_0");
        assert!(function
            .params
            .iter()
            .any(|p| p.name == "__nuis_effect_call_gate_0"));
        let distinct = function
            .params
            .iter()
            .map(|p| &p.name)
            .collect::<BTreeSet<_>>();
        assert_eq!(distinct.len(), function.params.len());
    }
}

#[test]
fn effectful_selections_do_not_discover_async_resource_optional_or_generic_signatures() {
    let original = parse_nuis_module(SOURCE).unwrap();
    for mutation in [
        "async",
        "reference",
        "optional",
        "resource",
        "generic",
        "return",
    ] {
        let mut module = original.clone();
        let function = module
            .functions
            .iter_mut()
            .find(|f| f.name == "yes")
            .unwrap();
        match mutation {
            "async" => function.is_async = true,
            "reference" => function.params[0].ty.is_ref = true,
            "optional" => function.params[0].ty.is_optional = true,
            "resource" => function.params[0].ty.name = "Buffer".into(),
            "generic" => function.params[0].ty.generic_args.push(scalar_type("i64")),
            "return" => function.return_type = Some(scalar_type("Buffer")),
            _ => unreachable!(),
        }
        let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
        assert!(outline(&mut module, &mut names).is_empty(), "{mutation}");
    }
}

#[test]
fn effectful_scalar_rebindings_retain_existing_values_for_both_polarities_and_exact_types() {
    for kind in ["bool", "i32", "i64", "f32", "f64"] {
        for body in [
            format!("if gate {{ let saved: {kind} = work(value); }}"),
            format!("if gate {{}} else {{ let saved: {kind} = work(value); }}"),
            "if gate { let saved = work(value); }".into(),
            "if gate {} else { let saved = work(value); }".into(),
        ] {
            let source = format!(
                "mod cpu Main {{
                fn work(value: {kind}) -> {kind} {{ print(70); return value; }}
                fn observe(gate: bool, value: {kind}, saved: {kind}) -> {kind} {{
                    {body} return saved;
                }} fn main() -> i64 {{ return 0; }} }}"
            );
            let (module, generated) = rewrite(&source);
            assert_eq!(generated.len(), 2, "{kind}/{body}");
            let arms = module
                .functions
                .iter()
                .filter(|f| generated.contains(&f.name));
            let mut retained = 0;
            let mut called = 0;
            for function in arms {
                assert_eq!(function.return_type, Some(scalar_type(kind)));
                assert!(function
                    .params
                    .iter()
                    .any(|p| p.name == "saved" && p.ty == scalar_type(kind)));
                match function.body.last().unwrap() {
                    NirStmt::Return(Some(NirExpr::Var(name))) if name == "saved" => retained += 1,
                    NirStmt::Return(Some(NirExpr::Call { callee, .. })) if callee == "work" => {
                        called += 1
                    }
                    other => panic!("unexpected arm: {other:?}"),
                }
            }
            assert_eq!((retained, called), (1, 1));
        }
    }
}

#[test]
fn effectful_scalar_rebindings_keep_sequential_versions_and_private_guards() {
    let source =
        include_str!("../../../tests/native_application_bridge/effectful_scalar_rebindings.ns")
            .replace("selected", "__nuis_effect_call_gate_0");
    let (module, generated) = rewrite(&source);
    assert_eq!(generated.len(), 4);
    for function in module
        .functions
        .iter()
        .filter(|f| generated.contains(&f.name))
    {
        assert_ne!(function.params[0].name, "__nuis_effect_call_gate_0");
        assert!(function
            .params
            .iter()
            .any(|p| p.name == "__nuis_effect_call_gate_0"));
    }
    let observer = module
        .functions
        .iter()
        .find(|f| f.name == "observe")
        .unwrap();
    assert_eq!(
        observer
            .body
            .iter()
            .filter(|stmt| matches!(stmt,
        NirStmt::Let { value: NirExpr::Call { callee, .. }, .. } if callee == "decide"))
            .count(),
        2
    );
    let rebound_positions = observer
        .body
        .iter()
        .enumerate()
        .filter_map(|(index, stmt)| {
            matches!(stmt, NirStmt::If { then_body, else_body, .. }
            if then_body.len() == 1 && else_body.len() == 1)
            .then_some(index)
        })
        .collect::<Vec<_>>();
    assert_eq!(rebound_positions.len(), 2);
    assert!(observer.body[rebound_positions[0] + 1..rebound_positions[1]].iter().any(|stmt|
        matches!(stmt, NirStmt::Let { value: NirExpr::Call { callee, args }, .. }
            if generated.contains(callee) && args.contains(&NirExpr::Var("__nuis_effect_call_gate_0".into())))));
}

#[test]
fn effectful_scalar_rebindings_reject_fresh_constants_mismatches_and_capture_overflow() {
    for body in [
        "if gate { let fresh: i64 = work(value); } return value;",
        "if gate {} else { let fresh: i64 = work(value); } return value;",
        "if gate { const value: i64 = work(value); } return value;",
        "if gate { print(70); let value: i64 = work(value); } return value;",
        "if gate { return work(value); } return value;",
        "if gate { let value: i64 = pure(value); } return value;",
        "while gate { if gate { let value: i64 = work(value); } } return value;",
    ] {
        let source = format!("mod cpu Main {{ fn work(value: i64) -> i64 {{ print(70); return value; }}
            fn pure(value: i64) -> i64 {{ return value; }}
            fn observe(gate: bool, value: i64) -> i64 {{ {body} }} fn main() -> i64 {{ return 0; }} }}");
        let mut module = parse_nuis_module(&source).unwrap();
        let before = module.clone();
        let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
        assert!(outline(&mut module, &mut names).is_empty(), "{body}");
        assert_eq!(module, before);
    }
    let source = "mod cpu Main { fn work(value: i64) -> i64 { print(70); return value; }
        fn observe(gate: bool, value: i64) -> i64 {
            if gate { let value: i64 = work(value); } return value;
        } fn main() -> i64 { return 0; } }";
    let mismatched = source.replace("let value: i64", "let value: bool");
    assert_eq!(
        parse_nuis_module(&mismatched).unwrap_err(),
        "binding `value` expected type `bool`, found `i64`"
    );
    let mut module = parse_nuis_module(source).unwrap();
    let observer = module
        .functions
        .iter_mut()
        .find(|f| f.name == "observe")
        .unwrap();
    let NirStmt::If { then_body, .. } = &mut observer.body[0] else {
        panic!("missing conditional")
    };
    let NirStmt::Let { ty, .. } = &mut then_body[0] else {
        panic!("missing binding")
    };
    *ty = Some(scalar_type("bool"));
    let before = module.clone();
    let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
    assert!(outline(&mut module, &mut names).is_empty());
    assert_eq!(module, before);
    for count in [30, 31] {
        let params = (0..count)
            .map(|i| format!("v{i}: i64"))
            .collect::<Vec<_>>()
            .join(", ");
        let sum = (0..count)
            .map(|i| format!("v{i}"))
            .collect::<Vec<_>>()
            .join(" + ");
        let source = format!(
            "mod cpu Main {{ fn work(value: i64) -> i64 {{ print(70); return value; }}
            fn observe(gate: bool, saved: i64, {params}) -> i64 {{
                if gate {{ let saved: i64 = work({sum}); }} return saved;
            }} fn main() -> i64 {{ return 0; }} }}"
        );
        let (_, generated) = rewrite(&source);
        assert_eq!(generated.len(), if count == 30 { 2 } else { 0 });
    }
}
