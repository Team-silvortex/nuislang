use super::*;
use crate::frontend::parse_nuis_module;

fn source(kind: &str, body: &str) -> String {
    format!(
        "mod cpu Main {{
            fn __nuis_effect_call_arm_0() -> bool {{ return true; }}
            fn work(v: {kind}) -> {kind} {{ print(70); return v; }}
            fn decide(v: bool) -> bool {{ print(98); return v; }}
            fn observe(gate: bool, first: bool, saved: {kind}) -> {kind} {{
                {body} return saved;
            }}
            fn main() -> i64 {{ return 0; }}
        }}"
    )
}

fn names(module: &NirModule) -> BTreeSet<String> {
    module.functions.iter().map(|f| f.name.clone()).collect()
}

fn reject(text: &str) {
    reject_module(parse_nuis_module(text).unwrap(), text);
}

fn reject_module(mut module: NirModule, context: &str) {
    let before = module.clone();
    let mut names = names(&module);
    let before_names = names.clone();
    assert!(outline(&mut module, &mut names).is_empty(), "{context}");
    assert_eq!(module, before, "{context}");
    assert_eq!(names, before_names, "{context}");
}

#[test]
fn sealed_binding_effectful_selections_veto_ordinary_writes_in_all_scalar_branches_atomically() {
    for kind in ["bool", "i32", "i64", "f32", "f64"] {
        for body in [
            "if gate { let sealed = work(saved); }",
            "if gate {} else { let sealed = work(saved); }",
            "if gate { let sealed = work(saved); } else { let sealed = work(saved); }",
            "if gate { if first { let sealed = work(saved); } else { let sealed = work(saved); } } else { let sealed = work(saved); }",
            "if gate { let sealed = work(saved); } else { if first { let sealed = work(saved); } else { let sealed = work(saved); } }",
            "if gate { const sealed: KIND = work(saved); } else { const sealed: KIND = work(saved); }",
        ] {
            reject(&source(
                kind,
                &format!("const sealed: {kind} = saved; {}", body.replace("KIND", kind)),
            ));
        }
        let text = source(
            kind,
            &format!(
                "const sealed: {kind} = saved;
            if gate {{ let saved = work(saved); }}
            else {{ if first {{ let saved = work(saved); }} else {{ let saved = work(saved); }} }}"
            ),
        );
        let mut valid = parse_nuis_module(&text).unwrap();
        let mut valid_names = names(&valid);
        assert!(!outline(&mut valid, &mut valid_names).is_empty());
        let mut late = parse_nuis_module(&text).unwrap();
        let observe = late
            .functions
            .iter_mut()
            .find(|f| f.name == "observe")
            .unwrap();
        let NirStmt::If { else_body, .. } = &mut observe.body[1] else {
            panic!()
        };
        let NirStmt::If { else_body, .. } = &mut else_body[0] else {
            panic!()
        };
        let NirStmt::Let { name, .. } = &mut else_body[0] else {
            panic!()
        };
        *name = "sealed".into();
        reject_module(late, &format!("late NIR destination mutation: {kind}"));
    }
}

#[test]
fn sealed_binding_effectful_selections_veto_literal_name_comparison_call_and_logical_rhs() {
    for value in [
        "true",
        "saved",
        "saved == first",
        "work(saved)",
        "saved && work(first)",
        "saved || work(first)",
    ] {
        for declaration in ["let sealed", "const sealed: bool"] {
            reject(&source(
                "bool",
                &format!(
                    "const sealed: bool = first;
                    if gate {{ {declaration} = {value}; }}
                    else {{ {declaration} = work(saved); }}"
                ),
            ));
        }
    }
}

#[test]
fn sealed_binding_effectful_selections_keep_installed_const_seals_without_partial_rewrites() {
    for kind in ["bool", "i32", "i64", "f32", "f64"] {
        let prefix = format!(
            "let sealed = saved;
            if gate {{ const sealed: {kind} = work(saved); }}
            else {{ const sealed: {kind} = work(saved); }}"
        );
        for suffix in [
            "if first { let sealed = work(saved); }",
            "if first { let sealed = work(saved); } else { let sealed = work(saved); }",
            "if first { const sealed: KIND = work(saved); } else { const sealed: KIND = work(saved); }",
        ] {
            let mut expected = parse_nuis_module(&source(kind, &prefix)).unwrap();
            let mut expected_names = names(&expected);
            let expected_generated = outline(&mut expected, &mut expected_names);
            assert_eq!(expected_generated.len(), 2);
            let mut module = parse_nuis_module(&source(
                kind,
                &format!("{prefix} {}", suffix.replace("KIND", kind)),
            ))
            .unwrap();
            let rejected = module.functions.iter().find(|f| f.name == "observe").unwrap()
                .body[2].clone();
            let mut actual_names = names(&module);
            let generated = outline(&mut module, &mut actual_names);
            let observe = expected.functions.iter_mut().find(|f| f.name == "observe").unwrap();
            observe.body.insert(observe.body.len() - 1, rejected);
            assert_eq!(generated, expected_generated, "{kind}: {suffix}");
            assert_eq!(actual_names, expected_names, "{kind}: {suffix}");
            assert_eq!(module, expected, "{kind}: {suffix}");
        }
    }
}

#[test]
fn sealed_binding_effectful_selections_preserve_read_only_constants_and_unrelated_updates() {
    for kind in ["bool", "i32", "i64", "f32", "f64"] {
        for body in [
            "if gate { let saved = work(sealed); } else { let saved = work(saved); }",
            "if gate { const fresh: bool = first; let saved = work(sealed); }",
            "if gate { if first { let saved = work(sealed); } else { let saved = work(saved); } } else { let saved = work(sealed); }",
        ] {
            let text = source(kind, &format!("const sealed: {kind} = saved; {body}"));
            let mut module = parse_nuis_module(&text).unwrap();
            let mut names = names(&module);
            let generated = outline(&mut module, &mut names);
            assert!(!generated.is_empty(), "{text}");
            crate::nir_verify::verify_nir_module(&module).unwrap();
            assert!(module.functions.iter().filter(|f| generated.contains(&f.name))
                .all(|f| matches!(f.body.first(), Some(NirStmt::If { .. }))));
        }
    }
    let text = source("bool", "const sealed: bool = first;
        if gate { const fresh: bool = decide(sealed); if first { return fresh; } else { return sealed; } }
        else { const fresh: bool = false; return fresh; }");
    let mut module = parse_nuis_module(&text).unwrap();
    let mut names = names(&module);
    assert!(!outline(&mut module, &mut names).is_empty());
    crate::nir_verify::verify_nir_module(&module).unwrap();
}

fn binding(name: &str, value: NirExpr, constant: bool) -> NirStmt {
    if constant {
        NirStmt::Const {
            name: name.into(),
            ty: scalar_type("bool"),
            value,
        }
    } else {
        NirStmt::Let {
            name: name.into(),
            ty: None,
            value,
        }
    }
}

fn branch(then_body: Vec<NirStmt>, else_body: Vec<NirStmt>) -> NirStmt {
    NirStmt::If {
        condition: NirExpr::Bool(true),
        then_body,
        else_body,
    }
}

#[test]
fn sealed_binding_effectful_selections_preflight_depends_on_destination_not_rhs_or_type() {
    let sealed = BTreeSet::from(["sealed".into()]);
    let values = [
        NirExpr::Bool(true),
        NirExpr::Var("saved".into()),
        NirExpr::Binary {
            op: NirBinaryOp::Eq,
            lhs: Box::new(NirExpr::Var("saved".into())),
            rhs: Box::new(NirExpr::Bool(true)),
        },
        NirExpr::Call {
            callee: "unknown".into(),
            args: vec![NirExpr::Var("saved".into())],
        },
        NirExpr::Binary {
            op: NirBinaryOp::And,
            lhs: Box::new(NirExpr::Bool(true)),
            rhs: Box::new(NirExpr::Bool(false)),
        },
        NirExpr::Int(1),
    ];
    for constant in [false, true] {
        for value in &values {
            let statement = binding("sealed", value.clone(), constant);
            assert!(writes_constants(
                std::slice::from_ref(&statement),
                &[],
                &sealed
            ));
            assert!(writes_constants(
                &[],
                std::slice::from_ref(&statement),
                &sealed
            ));
            assert!(writes_constants(
                &[branch(vec![], vec![statement])],
                &[],
                &sealed
            ));
            let fresh = binding("fresh", value.clone(), constant);
            assert!(!writes_constants(&[fresh], &[], &sealed));
        }
    }
}

#[test]
fn sealed_binding_effectful_selections_preflight_keeps_exact_64_node_pending_and_depth_bound() {
    let sealed = BTreeSet::from(["sealed".into()]);
    let harmless = binding("fresh", NirExpr::Bool(true), false);
    for (count, rejected) in [(0, false), (64, false), (65, true)] {
        assert_eq!(
            writes_constants(&vec![harmless.clone(); count], &[], &sealed),
            rejected
        );
    }
    for (depth, rejected) in [(63, false), (64, true)] {
        let mut statement = harmless.clone();
        for _ in 0..depth {
            statement = branch(vec![statement], vec![]);
        }
        assert_eq!(writes_constants(&[statement], &[], &sealed), rejected);
    }
    for (right, rejected) in [(31, false), (32, true)] {
        let yes = branch(vec![harmless.clone(); 31], vec![]);
        let no = branch(vec![], vec![harmless.clone(); right]);
        assert_eq!(writes_constants(&[yes], &[no], &sealed), rejected);
    }
    assert!(!writes_constants(
        &vec![harmless; 65],
        &[],
        &BTreeSet::new()
    ));
}

#[test]
fn sealed_binding_effectful_selections_preflight_does_not_spend_expression_budget_or_follow_reads()
{
    let sealed = BTreeSet::from(["sealed".into()]);
    // Construct each over-budget expression without recursively cloning it.
    let read = || {
        let mut value = NirExpr::Var("sealed".into());
        for _ in 0..64 {
            value = NirExpr::Call {
                callee: "unknown".into(),
                args: vec![value],
            };
        }
        value
    };
    let statements = [
        binding("fresh", read(), true),
        NirStmt::If {
            condition: read(),
            then_body: vec![NirStmt::Return(Some(read()))],
            else_body: vec![],
        },
    ];
    assert!(!writes_constants(&statements, &[], &sealed));
}
