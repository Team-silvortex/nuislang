use super::tests::{let_var, reject, rewrite, scope, source, stage};
use super::*;

fn mixed_source(ty: &str, selected: &str) -> String {
    source(ty, selected).replace("let b=float_seed;", "let b=float_seed;let c=saved;")
}
fn mixed_scope(ty: &str) -> Scope {
    let mut s = scope(ty);
    s.insert("c".into(), scalar_type("bool"));
    s
}
fn mixed_stage(s: &Scope) -> regions::Staging {
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
fn logical(leaves: usize) -> NirExpr {
    if leaves == 1 {
        return NirExpr::Var("c".into());
    }
    NirExpr::Binary {
        op: if leaves % 2 == 0 {
            NirBinaryOp::And
        } else {
            NirBinaryOp::Or
        },
        lhs: Box::new(logical(leaves / 2)),
        rhs: Box::new(logical(leaves - leaves / 2)),
    }
}

#[test]
fn return_bool_field_vector_accepts_logical_updates_retention_and_orders() {
    for ty in ["f32", "f64"] {
        for selected in [
            "if saved {let a=small(a);let b=floating(b);let c=c&&decide(c);}else{let c=c||decide(c);let b=floating(b);let a=small(a);}let local=relay(local);",
            "if saved {let local=relay(local);let c=decide(c);let a=small(a);let b=floating(b);}else{let b=floating(b);let a=small(a);}if c {let c=c&&decide(saved);let local=relay(local);}",
            "if saved {let c=c||decide(c);let a=small(a);if a==int_seed {let b=floating(b);let c=decide(c);}}else{let b=floating(b);}let local=relay(local);const seal:bool=c||false;",
            "if saved {let a=small(a);let b=floating(b);let c=decide(c);}else{}let c=c&&decide(saved);let local=relay(local);let ignored=decide(c);",
        ] {
            let (module, names) = rewrite(&mixed_source(ty, selected));
            assert!(!names.is_empty(), "{selected}");
            crate::nir_verify::verify_nir_module(&module).unwrap();
            assert!(module.structs.iter().any(|s| s.name.starts_with("__nuis_effect_fields_value")
                && s.fields.iter().map(|f|f.ty.clone()).collect::<Vec<_>>()
                    == [scalar_type("i64"),scalar_type("i32"),scalar_type(ty),scalar_type("bool")]));
            let observe = module.functions.iter().find(|f|f.name=="observe").unwrap();
            for statement in &observe.body {
                if let NirStmt::Let{value:NirExpr::Call{callee,..},..}=statement {
                    let helper=module.functions.iter().find(|f|&f.name==callee).unwrap();
                    assert_eq!(helper.return_type,Some(scalar_type("bool")));
                    assert!(!helper.params.iter().any(|p|["local","a","b","c","ignored"].contains(&p.name.as_str())));
                }
            }
        }
        let two_flags = mixed_source(ty,
            "if saved {let a=small(a);let b=decide(b);let c=c&&decide(b);}else{let c=c||decide(b);let a=small(a);}let local=relay(local);")
            .replace("let b=float_seed;", "let b=saved;");
        let (module, names) = rewrite(&two_flags);
        assert!(!names.is_empty());
        crate::nir_verify::verify_nir_module(&module).unwrap();
        assert!(module
            .structs
            .iter()
            .any(|s| s.name.starts_with("__nuis_effect_fields_value")
                && s.fields
                    .iter()
                    .filter(|f| f.ty == scalar_type("bool"))
                    .count()
                    == 2));
    }
}

#[test]
fn return_bool_field_vector_preserves_owner_authority_and_exact_child_vetoes() {
    for ty in ["f32", "f64"] {
        let s = mixed_scope(ty);
        let owner = Target("local");
        let yes = [
            let_var("a", "int_seed"),
            let_var("b", "float_seed"),
            let_var("c", "saved"),
        ];
        assert!(
            classify(&yes, &[], &Scope::new(), &mixed_stage(&s), &owner, 0)
                .unwrap()
                .is_some()
        );
        let mut entering = Scope::new();
        entering.insert("c".into(), scalar_type("bool"));
        assert!(classify(&yes, &[], &entering, &mixed_stage(&s), &owner, 0).is_none());
        let mut undefined = mixed_stage(&s);
        undefined.defined.remove("c");
        assert!(classify(&yes, &[], &Scope::new(), &undefined, &owner, 0).is_none());
        let mut sealed = mixed_stage(&s);
        sealed.constants.insert("c".into());
        assert!(classify(&yes, &[], &Scope::new(), &sealed, &owner, 0).is_none());
        for name in ["local", "c"] {
            for kind in ["ref", "optional", "generic", "String", "u32"] {
                let mut bad = s.clone();
                let t = bad.get_mut(name).unwrap();
                match kind {
                    "ref" => t.is_ref = true,
                    "optional" => t.is_optional = true,
                    "generic" => t.generic_args.push(scalar_type("bool")),
                    other => *t = scalar_type(other),
                }
                assert!(
                    classify(&yes, &[], &Scope::new(), &mixed_stage(&bad), &owner, 0).is_none()
                );
            }
        }
        let mut bool_owner = s.clone();
        bool_owner.insert("local".into(), scalar_type("bool"));
        assert!(classify(
            &yes,
            &[],
            &Scope::new(),
            &mixed_stage(&bool_owner),
            &owner,
            0
        )
        .is_none());
        assert!(transport::prepare_fields(
            predicates::Predicate::Scalar(NirExpr::Bool(true)),
            mixed_stage(&bool_owner),
            mixed_stage(&bool_owner),
            &bool_owner,
            "local",
            &["a".into(), "b".into(), "c".into()]
        )
        .is_none());
        for body in [
            "const c:bool=c;if saved {let a=small(a);let b=floating(b);let c=decide(c);}let local=relay(local);",
            "if saved {let a=small(a);let b=floating(b);let c=decide(c);let saved=c;}let local=relay(local);",
            "if saved {let a=small(a);let b=floating(b);let c=decide(c);print(81);}let local=relay(local);",
            "if saved {let a=small(a);let b=floating(b);let c=decide(c);return c;}let local=relay(local);",
            "if saved {let a=small(a);let b=floating(b);let c:i64=relay(seed);}let local=relay(local);",
        ] { reject(&mixed_source(ty,body)); }
        let root = mixed_source(
            ty,
            "if saved {let a=small(a);let b=floating(b);let c=decide(c);}let local=relay(local);",
        )
        .replace("let local=seed;", "")
        .replace("local", "seed");
        reject(&root);
    }
}

#[test]
fn return_bool_field_vector_uses_false_neutrals_and_distinct_typed_projections() {
    for ty in ["f32", "f64"] {
        let s = mixed_scope(ty);
        let mut retained = mixed_stage(&s);
        retained.defined.clear();
        let (plan, reads, _) = transport::prepare_fields(
            predicates::Predicate::Scalar(NirExpr::Bool(true)),
            retained,
            mixed_stage(&s),
            &s,
            "local",
            &["a".into(), "b".into(), "c".into()],
        )
        .unwrap();
        assert_eq!(
            reads,
            BTreeSet::from(["local".into(), "a".into(), "b".into(), "c".into()])
        );
        let mut names = BTreeSet::from(["__nuis_effect_fields_value_0".into()]);
        let mut bindings = BTreeSet::from(["__nuis_effect_call_fields_0".into()]);
        let mut helpers = vec![];
        let mut structs = vec![];
        let output =
            transport::install_fields(plan, &mut names, &mut bindings, &mut helpers, &mut structs);
        assert_eq!(structs[0].name, "__nuis_effect_fields_value_1");
        assert_eq!(structs[0].fields[3].ty, scalar_type("bool"));
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
            assert_eq!(
                fields[2],
                (
                    "child_1".into(),
                    if ty == "f32" {
                        NirExpr::F32("0.0".into())
                    } else {
                        NirExpr::F64("0.0".into())
                    }
                )
            );
        }
        for (statement, (name, field)) in output[output.len() - 4..].iter().zip([
            ("local", "owner"),
            ("a", "child_0"),
            ("b", "child_1"),
            ("c", "child_2"),
        ]) {
            assert!(
                matches!(statement,NirStmt::Let{name:n,ty:Some(t),value:NirExpr::FieldAccess{field:f,..}}
                if n==name&&t==&s[name]&&f==field)
            );
        }
        for child_ty in ["bool", "i32", "i64", "f32", "f64"] {
            let exact = scalar_type(child_ty);
            assert!(transport::private_field_scalar(&exact));
            let mut qualified = exact;
            qualified.is_ref = true;
            assert!(!transport::private_field_scalar(&qualified));
        }
    }
}

#[test]
fn return_bool_field_vector_shares_logical_work_depth_and_capture_budgets() {
    for ty in ["f32", "f64"] {
        let s = mixed_scope(ty);
        let owner = Target("local");
        let base = [
            let_var("a", "int_seed"),
            let_var("b", "float_seed"),
            let_var("c", "saved"),
        ];
        let children = classify(&base, &[], &Scope::new(), &mixed_stage(&s), &owner, 0)
            .unwrap()
            .unwrap();
        let tail = [let_var("local", "seed")];
        for (nodes, expressions, edges, accepted) in [
            (5, 69, 32, true),
            (4, 69, 32, false),
            (5, 68, 32, false),
            (5, 69, 31, false),
        ] {
            let mut yes = base.to_vec();
            let NirStmt::Let { value, .. } = &mut yes[2] else {
                panic!()
            };
            *value = logical(33);
            let mut budget = nested::Budget {
                nodes,
                expressions,
                logical_edges: edges,
            };
            let outcome = prove(
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
            assert_eq!(outcome.is_some(), accepted);
            if accepted {
                assert_eq!(
                    (budget.nodes, budget.expressions, budget.logical_edges),
                    (0, 0, 0)
                );
            }
        }
        let mut yes = base.to_vec();
        let NirStmt::Let { value, .. } = &mut yes[2] else {
            panic!()
        };
        *value = logical(34);
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
            let arm = (0..count).map(|i| base[i % 3].clone()).collect::<Vec<_>>();
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
        for count in [64, 65] {
            let mut arm = base.to_vec();
            arm.extend((3..16).map(|_| let_var("local", "seed")));
            let yes = [
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
                    then_body: (0..count - 53).map(|_| let_var("c", "saved")).collect(),
                    else_body: vec![],
                },
                let_var("local", "seed"),
            ];
            assert_eq!(
                prove(
                    &NirExpr::Var("first".into()),
                    &yes,
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
        for captures in [31, 32] {
            let mut s = s.clone();
            let mut yes = mixed_stage(&s);
            yes.defined.clear();
            for i in 0..captures - 4 {
                let n = format!("extra_{i}");
                s.insert(n.clone(), scalar_type("bool"));
                yes.inputs.insert(n);
            }
            assert_eq!(
                transport::prepare_fields(
                    predicates::Predicate::Scalar(NirExpr::Bool(true)),
                    yes,
                    mixed_stage(&s),
                    &s,
                    "local",
                    &children.0
                )
                .is_some(),
                captures == 31
            );
        }
    }
}
