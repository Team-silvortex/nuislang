use super::tests::{let_var, reject, rewrite, scope, source, stage};
use super::*;

fn siblings_source(ty: &str, body: &str) -> String {
    source(ty, body).replace(
        "let b=float_seed;",
        "let b=float_seed;let c=saved;let d=first;",
    )
}
fn siblings_scope(ty: &str) -> Scope {
    let mut s = scope(ty);
    for n in ["c", "d"] {
        s.insert(n.into(), scalar_type("bool"));
    }
    s
}
fn siblings_stage(s: &Scope) -> regions::Staging {
    let mut result = stage(s);
    result.defined.extend(["c".into(), "d".into()]);
    result
}
fn budget() -> nested::Budget {
    nested::Budget {
        nodes: 64,
        expressions: 256,
        logical_edges: 32,
    }
}
fn logical(name: &str, leaves: usize) -> NirExpr {
    if leaves == 1 {
        return NirExpr::Var(name.into());
    }
    NirExpr::Binary {
        op: if leaves % 2 == 0 {
            NirBinaryOp::And
        } else {
            NirBinaryOp::Or
        },
        lhs: Box::new(logical(name, leaves / 2)),
        rhs: Box::new(logical(name, leaves - leaves / 2)),
    }
}

#[test]
fn return_two_bool_field_vector_accepts_current_sibling_reads_and_retention() {
    for ty in ["f32", "f64"] {
        for body in [
            "if saved {let a=small(a);let b=floating(b);let c=decide(c!=true);let d=d&&decide(c);}else{let d=decide(d!=true);let c=c||decide(d);let b=floating(b);let a=small(a);}let local=relay(local);",
            "if saved {let c=decide(c!=true);let d=d&&decide(c);let a=small(a);if d {let b=floating(b);let c=decide(c!=true);}}else{let b=floating(b);let d=decide(d!=true);}let local=relay(local);",
            "if saved {let a=small(a);let b=floating(b);let c=decide(c!=true);let d=d&&decide(c);}else{}if c {let d=decide(d!=true);let local=relay(local);}const seal:bool=d||false;",
            "if saved {let d=decide(d!=true);let c=c||decide(d);let b=floating(b);let a=small(a);}else{let c=decide(c!=true);}let d=d&&decide(c);let local=relay(local);",
        ] {
            let (module,names)=rewrite(&siblings_source(ty,body));
            assert!(!names.is_empty(),"{body}");
            crate::nir_verify::verify_nir_module(&module).unwrap();
            assert!(module.structs.iter().any(|s|s.name.starts_with("__nuis_effect_fields_value")
                &&s.fields.iter().map(|f|f.ty.clone()).collect::<Vec<_>>()
                ==[scalar_type("i64"),scalar_type("i32"),scalar_type(ty),scalar_type("bool"),scalar_type("bool")]));
            let observe=module.functions.iter().find(|f|f.name=="observe").unwrap();
            for statement in &observe.body {
                if let NirStmt::Let{value:NirExpr::Call{callee,..},..}=statement {
                    let helper=module.functions.iter().find(|f|&f.name==callee).unwrap();
                    assert_eq!(helper.return_type,Some(scalar_type("bool")));
                    assert!(!helper.params.iter().any(|p|["local","a","b","c","d"].contains(&p.name.as_str())));
                }
            }
        }
    }
}

#[test]
fn return_two_bool_field_vector_keeps_each_bool_private_mutable_and_exact() {
    for ty in ["f32", "f64"] {
        let s = siblings_scope(ty);
        let owner = Target("local");
        let yes = [
            let_var("a", "int_seed"),
            let_var("b", "float_seed"),
            let_var("c", "saved"),
            let_var("d", "first"),
        ];
        let children = classify(&yes, &[], &Scope::new(), &siblings_stage(&s), &owner, 0)
            .unwrap()
            .unwrap();
        assert_eq!(children.0, ["a", "b", "c", "d"]);
        for child in ["c", "d"] {
            let entering = Scope::from([(child.into(), scalar_type("bool"))]);
            assert!(classify(&yes, &[], &entering, &siblings_stage(&s), &owner, 0).is_none());
            let mut undefined = siblings_stage(&s);
            undefined.defined.remove(child);
            assert!(classify(&yes, &[], &Scope::new(), &undefined, &owner, 0).is_none());
            let mut sealed = siblings_stage(&s);
            sealed.constants.insert(child.into());
            assert!(classify(&yes, &[], &Scope::new(), &sealed, &owner, 0).is_none());
            for kind in ["ref", "optional", "generic", "String", "u32"] {
                let mut bad = s.clone();
                let t = bad.get_mut(child).unwrap();
                match kind {
                    "ref" => t.is_ref = true,
                    "optional" => t.is_optional = true,
                    "generic" => t.generic_args.push(scalar_type("bool")),
                    other => *t = scalar_type(other),
                }
                assert!(
                    classify(&yes, &[], &Scope::new(), &siblings_stage(&bad), &owner, 0).is_none()
                );
            }
        }
        let mut bool_owner = s.clone();
        bool_owner.insert("local".into(), scalar_type("bool"));
        assert!(classify(
            &yes,
            &[],
            &Scope::new(),
            &siblings_stage(&bool_owner),
            &owner,
            0
        )
        .is_none());
        for body in [
            "const d:bool=d||false;if saved {let a=small(a);let b=floating(b);let c=decide(c);let d=decide(c);}let local=relay(local);",
            "if saved {let a=small(a);let b=floating(b);let c=decide(c);let d=decide(c);let saved=d;}let local=relay(local);",
            "if saved {let a=small(a);let b=floating(b);let c=decide(c);let d=decide(c);return d;}let local=relay(local);",
            "if saved {let a=small(a);let b=floating(b);let c=decide(c);let d=decide(c);print(78);}let local=relay(local);",
            "if saved {let a=small(a);let b=floating(b);let c=decide(c);let d:i64=relay(seed);}let local=relay(local);",
        ] {reject(&siblings_source(ty,body));}
        let root=siblings_source(ty,"if saved {let a=small(a);let b=floating(b);let c=decide(c);let d=decide(c);}let local=relay(local);")
            .replace("let local=seed;","").replace("local","seed");
        reject(&root);
    }
}

#[test]
fn return_two_bool_field_vector_preserves_distinct_false_neutrals_and_projections() {
    for ty in ["f32", "f64"] {
        let s = siblings_scope(ty);
        let mut retained = siblings_stage(&s);
        retained.defined.clear();
        let (plan, reads, _) = transport::prepare_fields(
            predicates::Predicate::Scalar(NirExpr::Bool(true)),
            retained,
            siblings_stage(&s),
            &s,
            "local",
            &["a".into(), "b".into(), "c".into(), "d".into()],
        )
        .unwrap();
        assert_eq!(
            reads,
            BTreeSet::from([
                "local".into(),
                "a".into(),
                "b".into(),
                "c".into(),
                "d".into()
            ])
        );
        let mut names = BTreeSet::from(["__nuis_effect_fields_value_0".into()]);
        let mut bindings = BTreeSet::from(["__nuis_effect_call_fields_0".into()]);
        let mut helpers = vec![];
        let mut structs = vec![];
        let output =
            transport::install_fields(plan, &mut names, &mut bindings, &mut helpers, &mut structs);
        assert_eq!(structs[0].name, "__nuis_effect_fields_value_1");
        for helper in &helpers {
            let NirStmt::If { then_body, .. } = &helper.body[0] else {
                panic!()
            };
            let [NirStmt::Return(Some(NirExpr::StructLiteral { fields, .. }))] =
                then_body.as_slice()
            else {
                panic!()
            };
            assert_eq!(fields[3], ("child_2".into(), NirExpr::Bool(false)));
            assert_eq!(fields[4], ("child_3".into(), NirExpr::Bool(false)));
        }
        for (statement, (name, field)) in output[output.len() - 5..].iter().zip([
            ("local", "owner"),
            ("a", "child_0"),
            ("b", "child_1"),
            ("c", "child_2"),
            ("d", "child_3"),
        ]) {
            assert!(
                matches!(statement,NirStmt::Let{name:n,ty:Some(t),value:NirExpr::FieldAccess{field:f,..}}
                if n==name&&t==&s[name]&&f==field)
            );
        }
    }
}

#[test]
fn return_two_bool_field_vector_shares_budget_across_both_logical_roots() {
    for ty in ["f32", "f64"] {
        let s = siblings_scope(ty);
        let owner = Target("local");
        let base = [
            let_var("a", "int_seed"),
            let_var("b", "float_seed"),
            let_var("c", "saved"),
            let_var("d", "first"),
        ];
        let children = classify(&base, &[], &Scope::new(), &siblings_stage(&s), &owner, 0)
            .unwrap()
            .unwrap();
        let mut yes = base.to_vec();
        if let NirStmt::Let { value, .. } = &mut yes[2] {
            *value = logical("d", 17);
        }
        if let NirStmt::Let { value, .. } = &mut yes[3] {
            *value = logical("c", 17);
        }
        let tail = [let_var("local", "seed")];
        for (nodes, expressions, edges, accepted) in [
            (6, 70, 32, true),
            (5, 70, 32, false),
            (6, 69, 32, false),
            (6, 70, 31, false),
        ] {
            let mut budget = nested::Budget {
                nodes,
                expressions,
                logical_edges: edges,
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
            if accepted {
                assert_eq!(
                    (budget.nodes, budget.expressions, budget.logical_edges),
                    (0, 0, 0)
                );
            }
        }
        if let NirStmt::Let { value, .. } = &mut yes[2] {
            *value = logical("d", 18);
        }
        assert!(prove(
            &NirExpr::Bool(true),
            &yes,
            &[],
            &tail,
            &s,
            &owner,
            &children,
            &BTreeMap::new(),
            &mut budget(),
            0
        )
        .is_none());
        for count in [16, 17] {
            let arm = (0..count).map(|i| base[i % 4].clone()).collect::<Vec<_>>();
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
        for depth in [7, 8] {
            assert_eq!(
                prove(
                    &NirExpr::Bool(true),
                    &base,
                    &[],
                    &tail,
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
        for count in [31, 32] {
            let mut s = s.clone();
            let mut retained = siblings_stage(&s);
            retained.defined.clear();
            for i in 0..count - 5 {
                let n = format!("extra_{i}");
                s.insert(n.clone(), scalar_type("bool"));
                retained.inputs.insert(n);
            }
            assert_eq!(
                transport::prepare_fields(
                    predicates::Predicate::Scalar(NirExpr::Bool(true)),
                    retained,
                    siblings_stage(&s),
                    &s,
                    "local",
                    &children.0
                )
                .is_some(),
                count == 31
            );
        }
    }
}
