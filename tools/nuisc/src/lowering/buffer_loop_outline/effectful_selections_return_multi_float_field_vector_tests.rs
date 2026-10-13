use super::tests::{let_var, reject, rewrite, scope, source, stage};
use super::*;
fn mixed_source(body: &str) -> String {
    source("f32", body)
        .replace(
            "fn observe(",
            "fn wide(v:f64,n:f32,flag:bool)->f64{print(73);return v;}fn observe(",
        )
        .replace("float_seed: f32", "float_seed: f32,wide_seed:f64")
        .replace(
            "let b=float_seed;",
            "let b=float_seed;let c=wide_seed;let d=saved;let e=first;",
        )
}
fn mixed_scope() -> Scope {
    let mut s = scope("f32");
    s.insert("c".into(), scalar_type("f64"));
    s.insert("wide_seed".into(), scalar_type("f64"));
    s.insert("d".into(), scalar_type("bool"));
    s.insert("e".into(), scalar_type("bool"));
    s
}
fn mixed_stage(s: &Scope) -> regions::Staging {
    let mut staged = stage(s);
    staged.defined.extend(["c".into(), "d".into(), "e".into()]);
    staged
}
#[test]
fn return_multi_float_field_vector_keeps_five_children_private_and_one_numeric_owner() {
    let body = "if saved {let a=small(a);let b=floating(b);let d=decide(d!=true);let e=e&&decide(d);let c=wide(c,b,d);}else{let e=decide(e!=true);let d=d||decide(e);let b=floating(b);let a=small(a);let c=wide(c,b,d);}let local=relay(local);";
    let (module, names) = rewrite(&mixed_source(body));
    assert!(!names.is_empty());
    crate::nir_verify::verify_nir_module(&module).unwrap();
    assert!(module
        .structs
        .iter()
        .any(|s| s.name.starts_with("__nuis_effect_fields_value")
            && s.fields.iter().map(|f| f.ty.clone()).collect::<Vec<_>>()
                == [
                    scalar_type("i64"),
                    scalar_type("i32"),
                    scalar_type("f32"),
                    scalar_type("f64"),
                    scalar_type("bool"),
                    scalar_type("bool")
                ]));
    reject(
        &mixed_source(body)
            .replace("let local=seed;", "")
            .replace("local", "seed"),
    );
    let s = mixed_scope();
    let yes = [
        let_var("a", "int_seed"),
        let_var("b", "float_seed"),
        let_var("c", "wide_seed"),
        let_var("d", "saved"),
        let_var("e", "first"),
    ];
    let owner = Target("local");
    let children = classify(&yes, &[], &Scope::new(), &mixed_stage(&s), &owner, 0)
        .unwrap()
        .unwrap();
    assert_eq!(children.0, ["a", "b", "c", "d", "e"]);
    for name in ["a", "b", "c", "d", "e"] {
        let entering = Scope::from([(name.into(), s[name].clone())]);
        assert!(classify(&yes, &[], &entering, &mixed_stage(&s), &owner, 0).is_none());
        let mut undefined = mixed_stage(&s);
        undefined.defined.remove(name);
        assert!(classify(&yes, &[], &Scope::new(), &undefined, &owner, 0).is_none());
        let mut sealed = mixed_stage(&s);
        sealed.constants.insert(name.into());
        assert!(classify(&yes, &[], &Scope::new(), &sealed, &owner, 0).is_none());
        for kind in ["ref", "optional", "generic", "String", "u32"] {
            let mut bad = s.clone();
            let t = bad.get_mut(name).unwrap();
            match kind {
                "ref" => t.is_ref = true,
                "optional" => t.is_optional = true,
                "generic" => t.generic_args.push(s[name].clone()),
                other => *t = scalar_type(other),
            }
            assert!(classify(&yes, &[], &Scope::new(), &mixed_stage(&bad), &owner, 0).is_none());
        }
    }
    let mut bad = s.clone();
    bad.insert("local".into(), scalar_type("bool"));
    assert!(classify(&yes, &[], &Scope::new(), &mixed_stage(&bad), &owner, 0).is_none());
}
#[test]
fn return_multi_float_field_vector_has_six_exact_neutrals_and_distinct_projections() {
    let s = mixed_scope();
    let mut retained = mixed_stage(&s);
    retained.defined.clear();
    let children = ["a".into(), "b".into(), "c".into(), "d".into(), "e".into()];
    let (plan, reads, _) = transport::prepare_fields(
        predicates::Predicate::Scalar(NirExpr::Bool(true)),
        retained,
        mixed_stage(&s),
        &s,
        "local",
        &children,
    )
    .unwrap();
    assert_eq!(
        reads,
        BTreeSet::from([
            "local".into(),
            "a".into(),
            "b".into(),
            "c".into(),
            "d".into(),
            "e".into()
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
    for helper in &helpers {
        let NirStmt::If { then_body, .. } = &helper.body[0] else {
            panic!()
        };
        let [NirStmt::Return(Some(NirExpr::StructLiteral { fields, .. }))] = then_body.as_slice()
        else {
            panic!()
        };
        assert_eq!(
            fields,
            &[
                ("owner".into(), NirExpr::Int(0)),
                (
                    "child_0".into(),
                    NirExpr::CastI64ToI32(Box::new(NirExpr::Int(0)))
                ),
                ("child_1".into(), NirExpr::F32("0.0".into())),
                ("child_2".into(), NirExpr::F64("0.0".into())),
                ("child_3".into(), NirExpr::Bool(false)),
                ("child_4".into(), NirExpr::Bool(false)),
            ]
        );
    }
    for (statement, (name, field)) in output[output.len() - 6..].iter().zip([
        ("local", "owner"),
        ("a", "child_0"),
        ("b", "child_1"),
        ("c", "child_2"),
        ("d", "child_3"),
        ("e", "child_4"),
    ]) {
        assert!(
            matches!(statement, NirStmt::Let { name: n, ty: Some(t), value: NirExpr::FieldAccess { field: f, .. } }
            if n == name && t == &s[name] && f == field)
        );
    }
    for count in [31, 32] {
        let mut extra = s.clone();
        let mut held = mixed_stage(&extra);
        held.defined.clear();
        for index in 0..count - 6 {
            let name = format!("extra_{index}");
            extra.insert(name.clone(), scalar_type("bool"));
            held.inputs.insert(name);
        }
        assert_eq!(
            transport::prepare_fields(
                predicates::Predicate::Scalar(NirExpr::Bool(true)),
                held,
                mixed_stage(&extra),
                &extra,
                "local",
                &children
            )
            .is_some(),
            count == 31
        );
    }
}
