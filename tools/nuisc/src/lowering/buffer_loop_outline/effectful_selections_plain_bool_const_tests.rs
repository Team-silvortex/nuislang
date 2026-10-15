use super::*;
use crate::frontend::parse_nuis_module;

fn source(body: &str) -> String {
    format!("mod cpu Main {{
        fn decide(v: bool) -> bool {{ print(98); return v; }}
        fn boolean(v: bool) -> bool {{ return v; }}
        fn token(v: i64) -> i64 {{ print(97); return v; }}
        fn observe(gate: bool, first: bool, saved: bool) -> bool {{ {body} print(80); return false; }}
        fn main() -> i64 {{ return 0; }} }}")
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
fn return_plain_const_literals_names_calls_and_selected_seals_share_existing_proof() {
    for body in [
        "if gate { const literal: bool = true; const copied: bool = first; const sealed: bool = decide(copied); if sealed { return literal; } else { const leaf: bool = decide(saved); return leaf; } } else { const leaf: bool = false; return leaf; }",
        "if gate { const unused: bool = decide(saved); if first { return true; } else { return false; } } else { return saved; }",
        "if gate { if first { const unused: bool = decide(saved); return true; } else { return false; } } else { return saved; }",
    ] {
        let (module, names) = rewrite(&source(body));
        assert!(!names.is_empty(), "{body}");
        crate::nir_verify::verify_nir_module(&module).unwrap();
        assert!(module.functions.iter().filter(|f| names.contains(&f.name)).all(|f| matches!(f.body.first(), Some(NirStmt::If { .. })) && f.params.len() <= 32));
        assert!(module.functions.iter().filter(|f| names.contains(&f.name)).any(|f| f.body.iter().any(|s| matches!(s, NirStmt::Const { ty, .. } if ty == &scalar_type("bool")))));
        let observe = module.functions.iter().find(|f| f.name == "observe").unwrap();
        assert!(!observe.body.iter().any(|s| matches!(s, NirStmt::Const { .. })));
    }
    // The old rejection was solely the nonlogical bool const initializer.
    let text = "mod cpu Main { fn work(v: i64) -> i64 { print(70); return v; }
        fn observe(gate: bool, first: bool, saved: i64) -> i64 {
            if gate { const local: bool = first; let saved = work(saved); } return saved;
        } fn main() -> i64 { return 0; } }";
    let (module, names) = rewrite(text);
    assert!(!names.is_empty());
    crate::nir_verify::verify_nir_module(&module).unwrap();
}

#[test]
fn return_plain_const_type_seal_scope_hidden_effect_and_late_vetoes_are_atomic() {
    for body in [
        "if gate { const sealed: bool = decide(first); let sealed = saved; return sealed; } else { return false; }",
        "if gate { let sealed = first; const sealed: bool = decide(saved); return sealed; } else { return false; }",
        "if gate { const sealed: bool = decide(first); if first { let sealed = saved; return sealed; } else { return false; } } else { return false; }",
        "if gate { const saved: bool = decide(first); return saved; } else { return false; }",
        "if gate { const sealed: i64 = token(1); return true; } else { return false; }",
        "if gate { const sealed: bool = boolean(saved || decide(first)); return sealed; } else { return false; }",
        "if gate { const sealed: bool = boolean(first); return sealed; } else { return false; }",
    ] { reject(parse_nuis_module(&source(body)).unwrap()); }
    let original = parse_nuis_module(&source("if gate { const sealed: bool = decide(first); if first { return sealed; } else { return saved; } } else { return false; }")).unwrap();
    for mutation in [
        "int", "missing", "void", "leak", "ref", "optional", "generic", "type", "async",
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
            let NirStmt::If {
                then_body,
                else_body,
                ..
            } = &mut f.body[0]
            else {
                panic!()
            };
            if matches!(mutation, "ref" | "optional" | "generic" | "type") {
                let NirStmt::Const { ty, .. } = &mut then_body[0] else {
                    panic!()
                };
                match mutation {
                    "ref" => ty.is_ref = true,
                    "optional" => ty.is_optional = true,
                    "generic" => ty.generic_args.push(scalar_type("bool")),
                    "type" => *ty = scalar_type("i64"),
                    _ => unreachable!(),
                }
            } else {
                else_body[0] = NirStmt::Return(match mutation {
                    "int" => Some(NirExpr::Int(1)),
                    "missing" => Some(NirExpr::Var("absent".into())),
                    "leak" => Some(NirExpr::Var("sealed".into())),
                    "void" => None,
                    _ => unreachable!(),
                });
            }
        }
        reject(module);
    }
}

fn constant(name: &str, value: NirExpr) -> NirStmt {
    NirStmt::Const {
        name: name.into(),
        ty: scalar_type("bool"),
        value,
    }
}

fn budget(nodes: usize, expressions: usize) -> nested::Budget {
    nested::Budget {
        nodes,
        expressions,
        logical_edges: 0,
    }
}

#[test]
fn return_plain_const_exact_original_budget_adds_no_logical_edges_or_helpers() {
    let scope = Scope::from([
        ("input".into(), scalar_type("bool")),
        ("value".into(), scalar_type("bool")),
    ]);
    let signatures = Signatures::from([(
        "decide".into(),
        (vec![scalar_type("bool")], scalar_type("bool")),
    )]);
    let statements = vec![
        constant("lit", NirExpr::Bool(true)),
        constant("copy", NirExpr::Var("input".into())),
        constant(
            "call",
            NirExpr::Call {
                callee: "decide".into(),
                args: vec![NirExpr::Var("copy".into())],
            },
        ),
        constant(
            "compare",
            NirExpr::Binary {
                op: NirBinaryOp::Eq,
                lhs: Box::new(NirExpr::Var("value".into())),
                rhs: Box::new(NirExpr::Var("lit".into())),
            },
        ),
    ];
    for (nodes, expressions, accepted) in [(3, 7, false), (4, 6, false), (4, 7, true)] {
        let mut budget = budget(nodes, expressions);
        let result = stage(
            &statements,
            &scope,
            &BTreeSet::new(),
            &signatures,
            &mut budget,
        );
        assert_eq!(result.is_some(), accepted);
        if let Some(staged) = result {
            assert_eq!(
                (budget.nodes, budget.expressions, budget.logical_edges),
                (0, 0, 0)
            );
            assert_eq!(
                staged.inputs,
                BTreeSet::from(["input".into(), "value".into()])
            );
            assert_eq!(staged.calls, BTreeSet::from(["decide".into()]));
            assert_eq!(
                staged.constants,
                BTreeSet::from(["lit".into(), "copy".into(), "call".into(), "compare".into()])
            );
            let (mut names, mut bindings, mut helpers, mut structs) =
                (BTreeSet::new(), BTreeSet::new(), vec![], vec![]);
            assert_eq!(
                install(
                    staged.statements,
                    &mut names,
                    &mut bindings,
                    &mut helpers,
                    &mut structs
                ),
                statements
            );
            assert!(
                helpers.is_empty() && structs.is_empty() && names.is_empty() && bindings.is_empty()
            );
        }
    }
    assert!(stage(
        &statements[..1],
        &Scope::from([("lit".into(), scalar_type("bool"))]),
        &BTreeSet::new(),
        &signatures,
        &mut budget(64, 256)
    )
    .is_none());
    assert!(stage(
        &statements[..1],
        &scope,
        &BTreeSet::from(["lit".into()]),
        &signatures,
        &mut budget(64, 256)
    )
    .is_none());
}

#[test]
fn return_plain_const_capture_depth_and_prefix_limits_stay_bounded() {
    for count in [16, 17] {
        let statements = (0..count)
            .map(|i| constant(&format!("v{i}"), NirExpr::Bool(true)))
            .collect::<Vec<_>>();
        assert_eq!(
            stage(
                &statements,
                &Scope::new(),
                &BTreeSet::new(),
                &Signatures::new(),
                &mut budget(64, 256)
            )
            .is_some(),
            count == 16
        );
    }
    let signatures = Signatures::from([(
        "boolean".into(),
        (vec![scalar_type("bool")], scalar_type("bool")),
    )]);
    for depth in [31, 32] {
        let mut value = NirExpr::Bool(true);
        for _ in 0..depth {
            value = NirExpr::Call {
                callee: "boolean".into(),
                args: vec![value],
            };
        }
        assert_eq!(
            stage(
                &[constant("sealed", value)],
                &Scope::new(),
                &BTreeSet::new(),
                &signatures,
                &mut budget(64, 256)
            )
            .is_some(),
            depth == 31
        );
    }
    for depth in [8, 9] {
        let mut child = "const leaf: bool = decide(saved); return leaf;".to_owned();
        for _ in 0..depth - 1 {
            child = format!("if first {{ {child} }} else {{ return false; }}");
        }
        assert_eq!(
            !rewrite(&source(&format!(
                "if gate {{ {child} }} else {{ return true; }}"
            )))
            .1
            .is_empty(),
            depth == 8
        );
    }
    for count in [31, 32] {
        let params = (0..count)
            .map(|i| format!("v{i}: bool"))
            .collect::<Vec<_>>()
            .join(",");
        let args = (0..count)
            .map(|i| format!("v{i}"))
            .collect::<Vec<_>>()
            .join(",");
        let text = format!("mod cpu Main {{ fn all({params}) -> bool {{ print(1); return v0; }} fn observe({params}) -> bool {{ if true {{ const sealed: bool = all({args}); if true {{ return sealed; }} else {{ return false; }} }} else {{ return false; }} }} fn main() -> i64 {{ return 0; }} }}");
        let mut module = parse_nuis_module(&text).unwrap();
        let before = module.clone();
        let mut names: BTreeSet<_> = module.functions.iter().map(|f| f.name.clone()).collect();
        let before_names = names.clone();
        assert_eq!(!outline(&mut module, &mut names).is_empty(), count == 31);
        if count == 32 {
            assert_eq!(module, before);
            assert_eq!(names, before_names);
        }
    }
}
