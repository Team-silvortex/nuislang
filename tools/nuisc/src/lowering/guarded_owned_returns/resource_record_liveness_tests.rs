use super::super::super::record_tests::fixture;
use super::*;

fn ty(name: &str) -> NirTypeRef {
    NirTypeRef {
        name: name.into(),
        is_ref: false,
        is_optional: false,
        generic_args: vec![],
    }
}

fn declaration(name: &str, kind: &str, value: NirExpr, constant: bool) -> NirStmt {
    if constant {
        NirStmt::Const {
            name: name.into(),
            ty: ty(kind),
            value,
        }
    } else {
        NirStmt::Let {
            name: name.into(),
            ty: Some(ty(kind)),
            value,
        }
    }
}

fn literal(kind: &str, fields: Vec<(&str, NirExpr)>) -> NirExpr {
    NirExpr::StructLiteral {
        type_name: kind.into(),
        type_args: vec![],
        fields: fields.into_iter().map(|(n, v)| (n.into(), v)).collect(),
    }
}

fn projection(base: NirExpr, name: &str) -> NirExpr {
    NirExpr::FieldAccess {
        base: Box::new(base),
        field: name.into(),
    }
}

fn value(statement: &NirStmt) -> &NirExpr {
    match statement {
        NirStmt::Let { value, .. } | NirStmt::Const { value, .. } => value,
        _ => unreachable!(),
    }
}

#[test]
fn resource_predicate_record_liveness_hoists_only_completed_leaves_before_eager_and_lazy_fields() {
    for constant in [false, true] {
        let (template, mut context, scope, live, call) = fixture();
        let mut locals = LocalRecords::default();
        locals.advance(
            &declaration("prior", "Packet", call.clone(), false),
            &context.calls,
        );
        let NirExpr::Call { args, .. } = &call else {
            unreachable!()
        };
        let source = declaration(
            "local",
            "Bundle",
            literal(
                "Bundle",
                vec![
                    ("before", args[0].clone()),
                    ("after", args[1].clone()),
                    ("packet", NirExpr::Var("prior".into())),
                ],
            ),
            constant,
        );
        context
            .reserved
            .insert("__nuis_resource_predicate_left".into());
        let out = locals
            .split(&source, &template, &scope, &live, &mut context)
            .unwrap();
        assert_eq!(out.len(), 6);
        let mut names = vec![];
        for (index, field, kind) in [(0, "left", "i64"), (1, "flag", "bool"), (2, "right", "i64")] {
            let NirStmt::Let {
                name,
                ty: Some(t),
                value,
            } = &out[index]
            else {
                unreachable!()
            };
            assert_eq!(t, &ty(kind));
            assert_eq!(*value, projection(NirExpr::Var("prior".into()), field));
            assert_ne!(name, "__nuis_resource_predicate_left");
            assert!(!names.contains(name));
            names.push(name.clone());
        }
        assert_eq!(value(&out[3]), &args[0]);
        assert_eq!(value(&out[4]), &args[1]);
        assert_eq!(matches!(out[5], NirStmt::Const { .. }), constant);
        let NirExpr::StructLiteral { fields, .. } = value(&out[5]) else {
            unreachable!()
        };
        let NirExpr::StructLiteral { fields, .. } = &fields[2].1 else {
            unreachable!()
        };
        for (i, (_, v)) in fields.iter().enumerate() {
            assert_eq!(*v, NirExpr::Var(names[i].clone()));
        }
        assert!(context.pending.is_empty() && context.outlines.owners.is_empty());
    }
}

#[test]
fn resource_predicate_record_liveness_deduplicates_overlapping_nominal_paths() {
    let (template, mut context, scope, live, call) = fixture();
    let mut locals = LocalRecords::default();
    locals.advance(
        &declaration("outer", "Envelope", call.clone(), false),
        &context.calls,
    );
    let NirExpr::Call { args, .. } = call else {
        unreachable!()
    };
    let source = declaration(
        "local",
        "Double",
        literal(
            "Double",
            vec![
                ("gate", args[1].clone()),
                ("first", NirExpr::Var("outer".into())),
                ("second", projection(NirExpr::Var("outer".into()), "packet")),
            ],
        ),
        false,
    );
    let out = locals
        .split(&source, &template, &scope, &live, &mut context)
        .unwrap();
    assert_eq!(
        out.len(),
        5,
        "three shared scalar projections, one lazy field, one constructor"
    );
    for (i, field) in ["left", "flag", "right"].iter().enumerate() {
        assert_eq!(
            *value(&out[i]),
            projection(projection(NirExpr::Var("outer".into()), "packet"), field)
        );
    }
    assert_eq!(value(&out[3]), &args[1]);
    let NirExpr::StructLiteral { fields, .. } = value(&out[4]) else {
        unreachable!()
    };
    let NirExpr::StructLiteral { fields: outer, .. } = &fields[1].1 else {
        unreachable!()
    };
    assert_eq!(outer[0].1, fields[2].1);
}

#[test]
fn resource_predicate_record_liveness_never_hoists_a_new_helper_call_or_its_arguments() {
    let (template, mut context, scope, live, call) = fixture();
    let mut locals = LocalRecords::default();
    locals.advance(
        &declaration("prior", "Packet", call.clone(), false),
        &context.calls,
    );
    let NirExpr::Call { args, .. } = &call else {
        unreachable!()
    };
    let source = declaration(
        "local",
        "Pair",
        literal(
            "Pair",
            vec![
                ("gate", args[1].clone()),
                ("first", call.clone()),
                ("second", NirExpr::Var("prior".into())),
            ],
        ),
        false,
    );
    let out = locals
        .split(&source, &template, &scope, &live, &mut context)
        .unwrap();
    assert_eq!(out.len(), 9);
    for stmt in &out[..3] {
        assert!(matches!(value(stmt), NirExpr::FieldAccess { .. }));
    }
    assert_eq!(value(&out[3]), &args[1]);
    assert_eq!(
        value(&out[4]),
        &call,
        "keep eager reads, checked arithmetic and lazy arguments intact"
    );
    assert_eq!(out.iter().filter(|s| value(s) == &call).count(), 1);
    assert!(context.pending.is_empty());
}
