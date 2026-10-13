use super::tests::{let_var, reject, rewrite, scope, source, stage};
use super::*;

fn float_source(ty: &str, body: &str) -> String {
    source(ty, body)
        .replace(
            "fn relay(v: i64) -> i64",
            &format!("fn relay(v: {ty}) -> {ty}"),
        )
        .replace("seed: i64", &format!("seed: {ty}"))
        .replace(
            "let b=float_seed;",
            "let b=float_seed;let c=saved;let d=first;",
        )
}
fn float_scope(ty: &str) -> Scope {
    let mut s = scope(ty);
    for name in ["local", "seed"] {
        s.insert(name.into(), scalar_type(ty));
    }
    for name in ["c", "d"] {
        s.insert(name.into(), scalar_type("bool"));
    }
    s
}
fn float_stage(s: &Scope) -> regions::Staging {
    let mut staged = stage(s);
    staged.defined.extend(["c".into(), "d".into()]);
    staged
}

#[test]
fn return_float_owner_field_vector_keeps_typed_root_authority_and_current_children() {
    for ty in ["f32", "f64"] {
        let body = "if saved {let a=small(a);let b=floating(b);let c=decide(c!=true);let d=d&&decide(c);}else{let d=decide(d!=true);let c=c||decide(d);let b=floating(b);let a=small(a);}let local=relay(local);";
        let (module, names) = rewrite(&float_source(ty, body));
        assert!(!names.is_empty());
        crate::nir_verify::verify_nir_module(&module).unwrap();
        assert!(module
            .structs
            .iter()
            .any(|s| s.name.starts_with("__nuis_effect_fields_value")
                && s.fields.iter().map(|f| f.ty.clone()).collect::<Vec<_>>()
                    == [
                        scalar_type(ty),
                        scalar_type("i32"),
                        scalar_type(ty),
                        scalar_type("bool"),
                        scalar_type("bool")
                    ]));
        let root = float_source(ty, body)
            .replace("let local=seed;", "")
            .replace("local", "seed");
        reject(&root);
        let s = float_scope(ty);
        let yes = [
            let_var("a", "int_seed"),
            let_var("b", "float_seed"),
            let_var("c", "saved"),
            let_var("d", "first"),
        ];
        for qualifier in ["ref", "optional", "generic", "bool", "String", "u32"] {
            let mut bad = s.clone();
            let owner = bad.get_mut("local").unwrap();
            match qualifier {
                "ref" => owner.is_ref = true,
                "optional" => owner.is_optional = true,
                "generic" => owner.generic_args.push(scalar_type(ty)),
                other => *owner = scalar_type(other),
            }
            assert!(classify(
                &yes,
                &[],
                &Scope::new(),
                &float_stage(&bad),
                &Target("local"),
                0
            )
            .is_none());
        }
        let mut sealed = float_stage(&s);
        sealed.constants.insert("local".into());
        assert!(classify(&yes, &[], &Scope::new(), &sealed, &Target("local"), 0).is_none());
    }
}

#[test]
fn return_float_owner_field_vector_keeps_original_width_neutrals_and_distinct_projections() {
    for ty in ["f32", "f64"] {
        let s = float_scope(ty);
        let mut retained = float_stage(&s);
        retained.defined.clear();
        let (plan, reads, _) = transport::prepare_fields(
            predicates::Predicate::Scalar(NirExpr::Bool(true)),
            retained,
            float_stage(&s),
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
        let mut helpers = vec![];
        let mut structs = vec![];
        let output = transport::install_fields(
            plan,
            &mut BTreeSet::new(),
            &mut BTreeSet::new(),
            &mut helpers,
            &mut structs,
        );
        let zero = if ty == "f32" {
            NirExpr::F32("0.0".into())
        } else {
            NirExpr::F64("0.0".into())
        };
        for helper in &helpers {
            let NirStmt::If { then_body, .. } = &helper.body[0] else {
                panic!()
            };
            let [NirStmt::Return(Some(NirExpr::StructLiteral { fields, .. }))] =
                then_body.as_slice()
            else {
                panic!()
            };
            assert_eq!(fields[0], ("owner".into(), zero.clone()));
            assert_eq!(fields[2], ("child_1".into(), zero.clone()));
            assert_eq!(fields[3], ("child_2".into(), NirExpr::Bool(false)));
            assert_eq!(fields[4], ("child_3".into(), NirExpr::Bool(false)));
        }
        for (statement, name) in output[output.len() - 5..]
            .iter()
            .zip(["local", "a", "b", "c", "d"])
        {
            assert!(
                matches!(statement, NirStmt::Let { name: n, ty: Some(t), value: NirExpr::FieldAccess { .. } }
                if n == name && t == &s[name])
            );
        }
    }
}
