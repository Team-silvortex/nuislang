use super::*;
use crate::frontend::parse_nuis_module;

fn source(body: &str) -> String {
    format!(
        "mod cpu Main {{
        fn decide(v: bool) -> bool {{ print(98); return v; }}
        fn boolean(v: bool) -> bool {{ return v; }}
        fn token(v: i64) -> i64 {{ print(97); return v; }}
        fn observe(gate: bool, first: bool, saved: bool) -> bool {{ {body} print(80); return false; }}
        fn main() -> i64 {{ return 0; }} }}"
    )
}

fn rewrite(text: &str) -> (NirModule, BTreeSet<String>) {
    let mut module = parse_nuis_module(text).unwrap();
    let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
    let generated = outline(&mut module, &mut names);
    (module, generated)
}

fn reject(mut module: NirModule) {
    let before = module.clone();
    let mut names: BTreeSet<_> = module.functions.iter().map(|f| f.name.clone()).collect();
    let before_names = names.clone();
    assert!(outline(&mut module, &mut names).is_empty());
    assert_eq!(module, before);
    assert_eq!(names, before_names);
}

#[test]
fn return_plain_tree_calls_literals_names_and_effect_only_ancestors_use_one_proof() {
    for body in [
        "if gate { if decide(first) { return saved; } else { return first; } } else { return false; }",
        "if gate { let local = decide(saved) && first; if first { return local; } else { return saved; } } else { return saved; }",
        "if decide(gate) { if first { return saved; } else { return false; } } else { return true; }",
        "if gate { let unused = token(1); if first { return saved; } else { return false; } } else { return true; }",
        "if gate { if first { let unused = token(1); return saved; } else { return false; } } else { return true; }",
        "if gate { let local = first; const sealed: bool = local || false; if first { return decide(sealed); } else { return saved; } } else { if first { return false; } else { return true; } }",
    ] {
        let (module, names) = rewrite(&source(body));
        assert!(!names.is_empty(), "{body}");
        crate::nir_verify::verify_nir_module(&module).unwrap();
        for f in module.functions.iter().filter(|f| names.contains(&f.name)) {
            assert!(matches!(f.body.first(), Some(NirStmt::If { .. })));
            assert_eq!(f.return_type, Some(scalar_type("bool")));
            assert!(f.params.len() <= 32);
        }
    }
    #[allow(dead_code)]
    #[path = "../../../tests/native_application_bridge/return_plain_tree_cases.rs"]
    mod cases;
    let variants = cases::source_variants();
    assert_eq!(variants.iter().collect::<BTreeSet<_>>().len(), 4);
    for text in variants {
        let (module, names) = rewrite(&text);
        assert!(!names.is_empty());
        crate::nir_verify::verify_nir_module(&module).unwrap();
    }
}

#[test]
fn return_plain_tree_partial_cross_scope_write_seal_type_and_late_vetoes_are_atomic() {
    for body in [
        "if decide(gate) { if first { return saved; } } else { return false; }",
        "if decide(gate) { if first { return saved; } else { let local = first; } } else { return false; }",
        "if decide(gate) { if first { let saved = first; return saved; } else { return false; } } else { return false; }",
        "if decide(gate) { let local = saved; if first { let local = first; return local; } else { return false; } } else { return false; }",
        "if decide(gate) { const sealed: bool = saved || false; let sealed = first; if first { return sealed; } else { return false; } } else { return false; }",
        "if decide(gate) { if first { return saved; } else { return false; } print(80); } else { return false; }",
        "if decide(gate) { if first { return saved; } else { return boolean(saved || decide(first)); } } else { return false; }",
        "if decide(gate) { if first { return saved; } else { print(80); return false; } } else { return false; }",
        "if gate { if first { return boolean(saved); } else { return false; } } else { return false; }",
    ] {
        reject(parse_nuis_module(&source(body)).unwrap());
    }
    let original = parse_nuis_module(&source("if decide(gate) { let local = saved; if first { return local; } else { return false; } } else { return saved; }")).unwrap();
    for mutation in [
        "int",
        "missing",
        "leak",
        "void",
        "call-type",
        "ref",
        "optional",
        "generic",
        "async",
    ] {
        let mut module = original.clone();
        if mutation == "async" {
            module
                .functions
                .iter_mut()
                .find(|f| f.name == "decide")
                .unwrap()
                .is_async = true;
        } else {
            let f = module
                .functions
                .iter_mut()
                .find(|f| f.name == "observe")
                .unwrap();
            match mutation {
                "ref" => f.return_type.as_mut().unwrap().is_ref = true,
                "optional" => f.return_type.as_mut().unwrap().is_optional = true,
                "generic" => f
                    .return_type
                    .as_mut()
                    .unwrap()
                    .generic_args
                    .push(scalar_type("bool")),
                _ => {
                    let NirStmt::If { else_body, .. } = &mut f.body[0] else {
                        panic!()
                    };
                    else_body[0] = NirStmt::Return(match mutation {
                        "int" => Some(NirExpr::Int(1)),
                        "missing" => Some(NirExpr::Var("absent".into())),
                        "leak" => Some(NirExpr::Var("local".into())),
                        "void" => None,
                        "call-type" => Some(NirExpr::Call {
                            callee: "token".into(),
                            args: vec![NirExpr::Int(1)],
                        }),
                        _ => unreachable!(),
                    });
                }
            }
        }
        reject(module);
    }
}

#[test]
fn return_plain_tree_exact_source_budget_needs_zero_logical_edges() {
    let module = parse_nuis_module(&source("if decide(gate) { let local = saved; if first { return local; } else { return false; } } else { return saved; }")).unwrap();
    let f = module
        .functions
        .iter()
        .find(|f| f.name == "observe")
        .unwrap();
    let scope = f
        .params
        .iter()
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect();
    let signatures = Signatures::from([(
        "decide".into(),
        (vec![scalar_type("bool")], scalar_type("bool")),
    )]);
    let NirStmt::If {
        condition,
        then_body,
        else_body,
    } = &f.body[0]
    else {
        panic!()
    };
    for (nodes, expressions, accepted) in [(5, 7, false), (6, 6, false), (6, 7, true)] {
        let mut budget = nested::Budget {
            nodes,
            expressions,
            logical_edges: 0,
        };
        assert_eq!(
            branch(
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
        if accepted {
            assert_eq!(
                (budget.nodes, budget.expressions, budget.logical_edges),
                (0, 0, 0)
            );
        }
    }
}

#[test]
fn return_plain_tree_capture_depth_and_original_node_caps_stay_bounded() {
    for count in [31, 32] {
        let scope = (0..count)
            .map(|i| (format!("v{i}"), scalar_type("bool")))
            .collect();
        let condition = NirExpr::Bool(true);
        let leaf = NirStmt::Return(Some(NirExpr::Call {
            callee: "all".into(),
            args: (0..count).map(|i| NirExpr::Var(format!("v{i}"))).collect(),
        }));
        let no = vec![NirStmt::Return(Some(NirExpr::Bool(false)))];
        let yes = vec![NirStmt::If {
            condition: NirExpr::Bool(true),
            then_body: vec![leaf],
            else_body: no.clone(),
        }];
        let signatures = Signatures::from([(
            "all".into(),
            (vec![scalar_type("bool"); count], scalar_type("bool")),
        )]);
        assert_eq!(
            prepare(&condition, &yes, &no, &scope, &signatures, &BTreeSet::new()).is_some(),
            count == 31
        );
        assert!(prepare(
            &condition,
            &yes,
            &no,
            &scope,
            &signatures,
            &BTreeSet::from(["all".into()])
        )
        .is_none());
        assert!(prepare(&condition, &no, &no, &scope, &signatures, &BTreeSet::new()).is_none());
    }
    for depth in [8, 9] {
        let mut child = "return saved;".to_owned();
        for _ in 0..depth - 1 {
            child = format!("if first {{ {child} }} else {{ return false; }}");
        }
        assert_eq!(
            !rewrite(&source(&format!(
                "if decide(gate) {{ {child} }} else {{ return true; }}"
            )))
            .1
            .is_empty(),
            depth == 8
        );
    }
    fn full(depth: usize) -> String {
        if depth == 0 {
            "return saved;".into()
        } else {
            let child = full(depth - 1);
            format!("if first {{ {child} }} else {{ {child} }}")
        }
    }
    for extra in [1, 2] {
        let prefix = (0..extra)
            .map(|i| format!("let local{i} = saved;"))
            .collect::<String>();
        let body = format!(
            "if decide(gate) {{ {prefix} {} }} else {{ {} }}",
            full(4),
            full(4)
        );
        assert_eq!(!rewrite(&source(&body)).1.is_empty(), extra == 1);
    }
}
