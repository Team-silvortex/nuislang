use super::*;
use crate::frontend::parse_nuis_module;

fn source(kind: &str, body: &str) -> String {
    format!(
        "mod cpu Main {{
        fn __nuis_effect_call_predicate_0() -> bool {{ return true; }}
        fn work(v: {kind}) -> {kind} {{ print(70); return v; }}
        fn decide(v: {kind}) -> bool {{ print(98); return v == v; }}
        fn boolean(v: bool) -> bool {{ return v; }}
        fn observe(gate: bool, first: bool, second: bool, saved: {kind}) -> {kind} {{
            {body} return saved;
        }} fn main() -> i64 {{ return 0; }} }}"
    )
}

fn outlined(text: &str) -> (NirModule, BTreeSet<String>) {
    let mut module = parse_nuis_module(text).unwrap();
    let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
    let generated = outline(&mut module, &mut names);
    (module, generated)
}

fn rejected(text: &str) {
    let mut module = parse_nuis_module(text).unwrap();
    let before = module.clone();
    let mut names: BTreeSet<_> = module.functions.iter().map(|f| f.name.clone()).collect();
    let before_names = names.clone();
    assert!(outline(&mut module, &mut names).is_empty(), "{text}");
    assert_eq!(module, before);
    assert_eq!(names, before_names);
}

#[test]
fn update_logical_effectful_scalar_selections_keep_private_prefix_middle_suffix_types_and_hygiene()
{
    for kind in ["bool", "i32", "i64", "f32", "f64"] {
        for inverted in [false, true] {
            let stage = "let local = first;
                let local = local && decide(saved);
                if local { let saved = work(saved); }
                let local = local || decide(saved);
                if second { let saved = work(saved); }
                let local = local && second;
                let saved = work(saved);";
            let body = if inverted {
                format!("if gate {{}} else {{ {stage} }}")
            } else {
                format!("if gate {{ {stage} }}")
            };
            let (module, generated) = outlined(&source(kind, &body));
            assert_eq!(generated.len(), 9, "{kind}/{inverted}");
            assert!(!generated.contains("__nuis_effect_call_predicate_0"));
            crate::nir_verify::verify_nir_module(&module).unwrap();
            let mut updates = 0;
            let mut initializers = 0;
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
                assert!(matches!(function.body.first(), Some(NirStmt::If { .. })));
                assert_eq!(
                    function.params.len(),
                    function
                        .params
                        .iter()
                        .map(|p| &p.name)
                        .collect::<BTreeSet<_>>()
                        .len()
                );
                for (index, statement) in function.body.iter().enumerate() {
                    if let NirStmt::Let { name, value, .. } = statement {
                        if name == "local" {
                            assert!(index > 0);
                            match value {
                                NirExpr::Var(name) => {
                                    assert_eq!(name, "first");
                                    initializers += 1;
                                }
                                value => {
                                    let call = if let NirExpr::Binary {
                                        op: NirBinaryOp::Eq,
                                        lhs,
                                        rhs,
                                    } = value
                                    {
                                        assert_eq!(rhs.as_ref(), &NirExpr::Bool(false));
                                        lhs.as_ref()
                                    } else {
                                        value
                                    };
                                    let NirExpr::Call { callee, args } = call else {
                                        panic!("logical root was not installed");
                                    };
                                    assert!(callee.starts_with("__nuis_effect_call_predicate_"));
                                    assert!(
                                        matches!(args.first(), Some(NirExpr::Var(name)) if name == "local")
                                    );
                                    updates += 1;
                                }
                            }
                        }
                    }
                }
            }
            assert_eq!((initializers, updates), (1, 3));
            let observer = module
                .functions
                .iter()
                .find(|f| f.name == "observe")
                .unwrap();
            assert!(!observer
                .body
                .iter()
                .any(|s| matches!(s, NirStmt::Let { name, .. } if name == "local")));
        }
    }
}

#[test]
fn update_logical_effectful_scalar_selections_capture_preceding_bool_versions_before_publication() {
    for stage in [
        "let saved = saved && decide(saved); let saved = saved || decide(saved);",
        "let saved = saved && decide(saved); if first { let saved = work(saved); }
         let saved = saved || decide(saved); if second { let saved = work(saved); }
         let saved = decide(saved) && (saved || decide(saved));",
    ] {
        for inverted in [false, true] {
            let body = if inverted {
                format!("if gate {{}} else {{ {stage} }}")
            } else {
                format!("if gate {{ {stage} }}")
            };
            let (module, generated) = outlined(&source("bool", &body));
            assert_eq!(
                generated.len(),
                if stage.contains("if first") { 10 } else { 4 }
            );
            crate::nir_verify::verify_nir_module(&module).unwrap();
            let observer = module
                .functions
                .iter()
                .find(|f| f.name == "observe")
                .unwrap();
            assert!(observer.body.iter().any(|s| matches!(s,
                NirStmt::If { then_body, else_body, .. }
                    if [then_body, else_body].iter().all(|body|
                        matches!(body.as_slice(), [NirStmt::Let { name, .. }] if name == "saved")))));
        }
    }
    let statements = [NirStmt::Let {
        name: "saved".into(),
        ty: None,
        value: NirExpr::Binary {
            op: NirBinaryOp::And,
            lhs: Box::new(NirExpr::Var("saved".into())),
            rhs: Box::new(NirExpr::Var("first".into())),
        },
    }];
    let scope = BTreeMap::from([
        ("saved".into(), scalar_type("bool")),
        ("first".into(), scalar_type("bool")),
    ]);
    let mut budget = nested::Budget {
        nodes: 64,
        expressions: 256,
        logical_edges: 32,
    };
    let staged = stage(
        &statements,
        &scope,
        &BTreeSet::new(),
        &Signatures::new(),
        &mut budget,
    )
    .unwrap();
    assert_eq!(
        staged.inputs,
        BTreeSet::from(["saved".into(), "first".into()])
    );
    assert_eq!(staged.defined, BTreeSet::from(["saved".into()]));
    assert!(staged.constants.is_empty());
    assert_eq!(
        (budget.nodes, budget.expressions, budget.logical_edges),
        (63, 253, 31)
    );
}

#[test]
fn update_logical_effectful_scalar_selections_reject_constants_hidden_roots_other_targets_and_late_errors_atomically(
) {
    for stage in [
        "let local = first; const local: bool = local && decide(saved); let saved = work(saved);",
        "const local: bool = first && decide(saved); let local = local || second; let saved = work(saved);",
        "const local: bool = first && decide(saved); if local { let saved = work(saved); } let local = local || second; let saved = work(saved);",
        "let first = first && decide(saved); let saved = work(saved);",
        "let local = first; let local = boolean(local && decide(saved)); let saved = work(saved);",
        "let local = first; let local = (local && decide(saved)) == true; let saved = work(saved);",
    ] { rejected(&source("i64", &format!("if gate {{ {stage} }}"))); }
    let mut module = parse_nuis_module(&source(
        "i64",
        "if gate {
        let local = first; let local = local && decide(saved);
        if local { let saved = work(saved); }
        let late = local || second; let saved = work(saved); }",
    ))
    .unwrap();
    let observer = module
        .functions
        .iter_mut()
        .find(|f| f.name == "observe")
        .unwrap();
    let NirStmt::If { then_body, .. } = &mut observer.body[0] else {
        unreachable!()
    };
    let NirStmt::Let { ty, .. } = &mut then_body[3] else {
        unreachable!()
    };
    *ty = Some(scalar_type("i64"));
    let before = module.clone();
    let mut names: BTreeSet<_> = module.functions.iter().map(|f| f.name.clone()).collect();
    let before_names = names.clone();
    assert!(outline(&mut module, &mut names).is_empty());
    assert_eq!(module, before);
    assert_eq!(names, before_names);
    rejected(&source(
        "bool",
        "const saved: bool = first;
        if gate { let saved = saved && decide(saved); let saved = work(saved); }",
    ));
    rejected(&source(
        "bool",
        "if gate {
        if first { let saved = saved && decide(saved); }
        if second { let saved = work(saved); } }",
    ));
    let text = source(
        "bool",
        "let sealed = first; if gate { const sealed: bool = decide(saved); }
        else { const sealed: bool = decide(saved); }
        if first { let sealed = sealed && decide(saved); let sealed = work(sealed); }",
    );
    let (module, generated) = outlined(&text);
    assert_eq!(generated.len(), 2);
    assert!(matches!(
        module
            .functions
            .iter()
            .find(|f| f.name == "observe")
            .unwrap()
            .body
            .last()
            .unwrap(),
        NirStmt::Return(_)
    ));
    assert!(module
        .functions
        .iter()
        .find(|f| f.name == "observe")
        .unwrap()
        .body
        .iter()
        .any(|s| matches!(s, NirStmt::If { .. })));
    let (module, generated) = outlined(&source("i64", "const sealed: bool = first;
        if gate { let local = sealed; let local = local && decide(saved); let saved = work(saved); }"));
    assert_eq!(generated.len(), 3);
    crate::nir_verify::verify_nir_module(&module).unwrap();
}

#[test]
fn update_logical_effectful_scalar_selections_keep_exact_owned_types_and_shared_bounded_proofs() {
    let statement = NirStmt::Let {
        name: "saved".into(),
        ty: None,
        value: NirExpr::Binary {
            op: NirBinaryOp::Or,
            lhs: Box::new(NirExpr::Var("saved".into())),
            rhs: Box::new(NirExpr::Bool(true)),
        },
    };
    for mutation in ["i64", "ref", "optional", "generic", "declared"] {
        let mut ty = scalar_type("bool");
        match mutation {
            "i64" => ty.name = "i64".into(),
            "ref" => ty.is_ref = true,
            "optional" => ty.is_optional = true,
            "generic" => ty.generic_args.push(scalar_type("i64")),
            _ => {}
        }
        let scope = BTreeMap::from([("saved".into(), ty)]);
        let mut statement = statement.clone();
        if mutation == "declared" {
            let NirStmt::Let { ty, .. } = &mut statement else {
                unreachable!()
            };
            *ty = Some(scalar_type("i64"));
        }
        let mut budget = nested::Budget {
            nodes: 64,
            expressions: 256,
            logical_edges: 32,
        };
        assert!(
            stage(
                &[statement],
                &scope,
                &BTreeSet::new(),
                &Signatures::new(),
                &mut budget
            )
            .is_none(),
            "{mutation}"
        );
    }
    let scope = BTreeMap::from([("saved".into(), scalar_type("bool"))]);
    for (nodes, expressions, edges, accepted) in [
        (0, 3, 1, false),
        (1, 2, 1, false),
        (1, 3, 0, false),
        (1, 3, 1, true),
    ] {
        let mut budget = nested::Budget {
            nodes,
            expressions,
            logical_edges: edges,
        };
        assert_eq!(
            stage(
                &[statement.clone()],
                &scope,
                &BTreeSet::new(),
                &Signatures::new(),
                &mut budget
            )
            .is_some(),
            accepted
        );
    }
    let sealed = BTreeSet::from(["saved".into()]);
    assert!(writes_logical_constants(&[statement.clone()], &[], &sealed));
    assert!(!writes_logical_constants(
        &[statement.clone()],
        &[],
        &BTreeSet::new()
    ));
    let mut deep = statement.clone();
    for _ in 0..64 {
        deep = NirStmt::If {
            condition: NirExpr::Bool(true),
            then_body: vec![deep],
            else_body: Vec::new(),
        };
    }
    assert!(writes_logical_constants(&[deep], &[], &sealed));
    assert!(writes_logical_constants(&vec![statement; 65], &[], &sealed));
}
