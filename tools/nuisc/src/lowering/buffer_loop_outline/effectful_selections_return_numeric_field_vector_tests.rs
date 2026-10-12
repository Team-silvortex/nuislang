use super::tests::{let_var, reject, rewrite, scope, source, stage};
use super::*;

fn vector_source(ty: &str, body: &str) -> String {
    source(ty, body).replace("let b=float_seed;", "let b=float_seed;let c=seed;")
}
fn vector_scope(ty: &str) -> Scope {
    let mut s = scope(ty);
    s.insert("c".into(), scalar_type("i64"));
    s
}
fn vector_stage(s: &Scope) -> regions::Staging {
    let mut staged = stage(s);
    staged.defined.insert("c".into());
    staged
}
fn budget() -> nested::Budget {
    nested::Budget {
        nodes: 64,
        expressions: 256,
        logical_edges: 32,
    }
}

#[test]
fn return_numeric_field_vector_admits_three_children_in_original_and_retained_orders() {
    for ty in ["f32", "f64"] {
        for selected in [
            "if saved {let local=relay(local);let a=small(a);let b=floating(b);let c=relay(c);}else{let c=relay(c);let b=floating(b);let a=small(a);let local=relay(local);}",
            "if saved {let a=small(a);let b=floating(b);let c=relay(c);}else{let c=relay(c);let b=floating(b);let a=small(a);}let local=relay(local);",
            "if saved {let a=small(a);let c=relay(c);}else{let b=floating(b);}if c==seed {let c=relay(c);let b=floating(b);let a=small(a);}let local=relay(local);",
            "if saved {let c=relay(c);let a=small(a);let b=floating(b);if a==int_seed {let b=floating(b);let c=relay(c);}}else{}let local=relay(local);let ignored=relay(local);",
        ] {
            let (module,names)=rewrite(&vector_source(ty,selected));
            assert!(!names.is_empty(),"{selected}");
            crate::nir_verify::verify_nir_module(&module).unwrap();
            assert!(module.structs.iter().any(|s|s.name.starts_with("__nuis_effect_fields_value")&&s.fields.len()==4));
            let observe=module.functions.iter().find(|f|f.name=="observe").unwrap();
            for statement in &observe.body {
                if let NirStmt::Let{value:NirExpr::Call{callee,..},..}=statement {
                    let helper=module.functions.iter().find(|f|&f.name==callee).unwrap();
                    assert_eq!(helper.return_type,Some(scalar_type("bool")));
                    assert!(!helper.params.iter().any(|p|["local","a","b","c","ignored"].contains(&p.name.as_str())));
                }
            }
        }
    }
}

#[test]
fn return_numeric_field_vector_retains_authority_scope_types_and_atomic_rejection() {
    for ty in ["f32", "f64"] {
        let s = vector_scope(ty);
        let owner = Target("local");
        let mut entering = s.clone();
        for n in ["a", "b", "c"] {
            entering.remove(n);
        }
        let yes = [
            let_var("a", "int_seed"),
            let_var("b", "float_seed"),
            let_var("c", "seed"),
        ];
        let children = classify(&yes, &[], &entering, &vector_stage(&s), &owner, 0)
            .unwrap()
            .unwrap();
        assert_eq!(children.0, ["a", "b", "c"]);
        for n in ["a", "b", "c"] {
            let mut incoming = entering.clone();
            incoming.insert(n.into(), s[n].clone());
            assert!(classify(&yes, &[], &incoming, &vector_stage(&s), &owner, 0).is_none());
            let mut undefined = vector_stage(&s);
            undefined.defined.remove(n);
            assert!(classify(&yes, &[], &entering, &undefined, &owner, 0).is_none());
        }
        for n in ["local", "a", "b", "c"] {
            let mut sealed = vector_stage(&s);
            sealed.constants.insert(n.into());
            assert!(classify(&yes, &[], &entering, &sealed, &owner, 0).is_none());
            for qualifier in ["ref", "optional", "generic", "bool", "String", "u32"] {
                let mut bad = s.clone();
                let t = bad.get_mut(n).unwrap();
                match qualifier {
                    "ref" => t.is_ref = true,
                    "optional" => t.is_optional = true,
                    "generic" => t.generic_args.push(scalar_type("i64")),
                    other => *t = scalar_type(other),
                }
                if qualifier == "bool" && n != "local" {
                    assert!(classify(&yes, &[], &entering, &vector_stage(&bad), &owner, 0)
                        .unwrap()
                        .is_some());
                    continue;
                }
                assert!(
                    classify(&yes, &[], &entering, &vector_stage(&bad), &owner, 0).is_none(),
                    "{n}/{qualifier}"
                );
            }
        }
        for body in [
            "if saved {let a=small(a);let b=floating(b);let c=relay(c);let seed=local;}let local=relay(local);",
            "const c:i64=c;if saved {let a=small(a);let b=floating(b);let c=relay(c);}let local=relay(local);",
            "if saved {let a=small(a);let b=floating(b);let c=relay(c);print(80);}let local=relay(local);",
            "if saved {let a=small(a);let b=floating(b);let c=relay(c);}let saved=first;let local=relay(local);",
            "if saved {let a=small(a);let b=floating(b);let c=relay(c);return saved;}let local=relay(local);",
        ] {reject(&vector_source(ty,body));}
        let root = vector_source(
            ty,
            "if saved {let a=small(a);let b=floating(b);let c=relay(c);}let local=relay(local);",
        )
        .replace("let local=seed;", "")
        .replace("local", "seed");
        reject(&root);
    }
}

#[test]
fn return_numeric_field_vector_is_type_driven_and_uses_original_width_neutrals() {
    let s = vector_scope("f32");
    let mut names = BTreeSet::from(["__nuis_effect_fields_value_0".into()]);
    let mut bindings = BTreeSet::from(["__nuis_effect_call_fields_0".into()]);
    let mut helpers = vec![];
    let mut structs = vec![];
    let children = ["a".into(), "b".into(), "c".into()];
    let mut empty = vector_stage(&s);
    empty.defined.clear();
    let (plan, reads, _) = transport::prepare_fields(
        predicates::Predicate::Scalar(NirExpr::Bool(true)),
        empty,
        vector_stage(&s),
        &s,
        "local",
        &children,
    )
    .unwrap();
    assert_eq!(
        reads,
        BTreeSet::from(["local".into(), "a".into(), "b".into(), "c".into()])
    );
    let output =
        transport::install_fields(plan, &mut names, &mut bindings, &mut helpers, &mut structs);
    let def = &structs[0];
    assert_eq!(def.name, "__nuis_effect_fields_value_1");
    assert_eq!(
        def.fields
            .iter()
            .map(|f| (f.name.as_str(), f.ty.clone()))
            .collect::<Vec<_>>(),
        [
            ("owner", scalar_type("i64")),
            ("child_0", scalar_type("i32")),
            ("child_1", scalar_type("f32")),
            ("child_2", scalar_type("i64"))
        ]
    );
    for helper in &helpers {
        let NirStmt::If { then_body, .. } = &helper.body[0] else {
            unreachable!()
        };
        let [NirStmt::Return(Some(NirExpr::StructLiteral { fields, .. }))] = then_body.as_slice()
        else {
            unreachable!()
        };
        assert_eq!(
            fields,
            &vec![
                ("owner".into(), NirExpr::Int(0)),
                (
                    "child_0".into(),
                    NirExpr::CastI64ToI32(Box::new(NirExpr::Int(0)))
                ),
                ("child_1".into(), NirExpr::F32("0.0".into())),
                ("child_2".into(), NirExpr::Int(0))
            ]
        );
    }
    assert_eq!(
        output[output.len() - 4..]
            .iter()
            .filter(|s| matches!(
                s,
                NirStmt::Let {
                    value: NirExpr::FieldAccess { .. },
                    ..
                }
            ))
            .count(),
        4
    );
    // Every exact ordered numeric quadruple prepares; this is not live backend coverage.
    for owner in ["i32", "i64", "f32", "f64"] {
        for a in ["i32", "i64", "f32", "f64"] {
            for b in ["i32", "i64", "f32", "f64"] {
                for c in ["i32", "i64", "f32", "f64"] {
                    let s = Scope::from([
                        ("local".into(), scalar_type(owner)),
                        ("a".into(), scalar_type(a)),
                        ("b".into(), scalar_type(b)),
                        ("c".into(), scalar_type(c)),
                    ]);
                    assert!(transport::prepare_fields(
                        predicates::Predicate::Scalar(NirExpr::Bool(true)),
                        vector_stage(&s),
                        vector_stage(&s),
                        &s,
                        "local",
                        &children
                    )
                    .is_some());
                }
            }
        }
    }
    // Field capacity and captures are independent: all-defined fields need no incoming capture.
    for count in [0, 1, 2, 3, 16, 31, 63, 64] {
        let mut s = Scope::from([("local".into(), scalar_type("i64"))]);
        let children = (0..count).map(|i| format!("v_{i}")).collect::<Vec<_>>();
        for n in &children {
            s.insert(n.clone(), scalar_type("i64"));
        }
        let stage = || regions::Staging {
            local: s.clone(),
            defined: s.keys().cloned().collect(),
            inputs: BTreeSet::new(),
            calls: BTreeSet::new(),
            constants: BTreeSet::new(),
            statements: vec![],
        };
        assert_eq!(
            transport::prepare_fields(
                predicates::Predicate::Scalar(NirExpr::Bool(true)),
                stage(),
                stage(),
                &s,
                "local",
                &children
            )
            .is_some(),
            (2..64).contains(&count)
        );
    }
}

#[test]
fn return_numeric_field_vector_preserves_original_work_forest_capture_statement_and_depth_bounds() {
    for ty in ["f32", "f64"] {
        let s = vector_scope(ty);
        let owner = Target("local");
        let yes = vec![
            let_var("a", "int_seed"),
            let_var("b", "float_seed"),
            let_var("c", "seed"),
        ];
        let children = classify(&yes, &[], &Scope::new(), &vector_stage(&s), &owner, 0)
            .unwrap()
            .unwrap();
        let tail = [let_var("local", "seed")];
        for (nodes, expressions, accepted) in [(5, 5, true), (4, 5, false), (5, 4, false)] {
            let mut budget = nested::Budget {
                nodes,
                expressions,
                logical_edges: 0,
            };
            let result = prove(
                &NirExpr::Bool(true),
                &yes,
                &[],
                &tail,
                &s,
                &owner,
                &children,
                &BTreeMap::new(),
                &mut budget,
                0,
            );
            assert_eq!(result.is_some(), accepted);
            if let Some(p) = result {
                assert_eq!(p.name, "local");
                assert!(p.inputs.contains("local"));
                assert_eq!(
                    (budget.nodes, budget.expressions, budget.logical_edges),
                    (0, 0, 0)
                );
            }
        }
        for count in [64, 65] {
            let mut arm = yes.clone();
            arm.extend((3..16).map(|_| let_var("local", "seed")));
            let forest = [
                NirStmt::If {
                    condition: NirExpr::Var("saved".into()),
                    then_body: arm.clone(),
                    else_body: arm,
                },
                let_var("local", "seed"),
            ];
            let no = (0..16)
                .map(|_| let_var("local", "seed"))
                .collect::<Vec<_>>();
            let suffix = [
                NirStmt::If {
                    condition: NirExpr::Var("saved".into()),
                    then_body: (0..count - 53).map(|_| let_var("c", "seed")).collect(),
                    else_body: vec![],
                },
                let_var("local", "seed"),
            ];
            assert_eq!(
                prove(
                    &NirExpr::Var("first".into()),
                    &forest,
                    &no,
                    &suffix,
                    &s,
                    &owner,
                    &children,
                    &BTreeMap::new(),
                    &mut budget(),
                    0
                )
                .is_some(),
                count == 64
            );
        }
        for count in [16, 17] {
            let arm = (0..count).map(|i| yes[i % 3].clone()).collect::<Vec<_>>();
            assert_eq!(
                prove(
                    &NirExpr::Bool(true),
                    &arm,
                    &[],
                    &tail,
                    &s,
                    &owner,
                    &children,
                    &BTreeMap::new(),
                    &mut budget(),
                    0
                )
                .is_some(),
                count == 16
            );
        }
        for count in [31, 32] {
            let mut s = s.clone();
            let mut stage = vector_stage(&s);
            stage.defined.clear();
            for i in 0..count - 4 {
                let n = format!("capture_{i}");
                s.insert(n.clone(), scalar_type("i64"));
                stage.inputs.insert(n);
            }
            assert_eq!(
                transport::prepare_fields(
                    predicates::Predicate::Scalar(NirExpr::Bool(true)),
                    stage,
                    vector_stage(&s),
                    &s,
                    "local",
                    &["a".into(), "b".into(), "c".into()]
                )
                .is_some(),
                count == 31
            );
        }
        for depth in [7, 8] {
            assert_eq!(
                prove(
                    &NirExpr::Bool(true),
                    &yes,
                    &[],
                    &[],
                    &s,
                    &owner,
                    &children,
                    &BTreeMap::new(),
                    &mut budget(),
                    depth
                )
                .is_some(),
                depth == 7
            );
        }
    }
}
