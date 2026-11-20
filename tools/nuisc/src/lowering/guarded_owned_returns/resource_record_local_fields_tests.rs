use super::super::record_tests::fixture;
use super::*;

fn ty(name: &str) -> NirTypeRef {
    NirTypeRef {
        name: name.into(),
        is_ref: false,
        is_optional: false,
        generic_args: vec![],
    }
}

fn binding(name: &str, kind: &str) -> NirStmt {
    NirStmt::Let {
        name: name.into(),
        ty: Some(ty(kind)),
        value: NirExpr::Int(0),
    }
}

fn literal(packet: NirExpr, call: NirExpr) -> NirStmt {
    let NirExpr::Call { args, .. } = call else {
        unreachable!()
    };
    NirStmt::Let {
        name: "local".into(),
        ty: Some(ty("Bundle")),
        value: NirExpr::StructLiteral {
            type_name: "Bundle".into(),
            type_args: vec![],
            fields: vec![
                ("before".into(), NirExpr::Int(3)),
                ("packet".into(), packet),
                ("after".into(), args[1].clone()),
            ],
        },
    }
}

#[test]
fn resource_predicate_record_locals_project_exact_visible_types_without_captures() {
    for projected in [false, true] {
        let (template, mut context, scope, live, call) = fixture();
        let mut locals = LocalRecords::default();
        locals.advance(
            &binding("prior", if projected { "Envelope" } else { "Packet" }),
            &context.calls,
        );
        let value = if projected {
            NirExpr::FieldAccess {
                base: Box::new(NirExpr::Var("prior".into())),
                field: "packet".into(),
            }
        } else {
            NirExpr::Var("prior".into())
        };
        let declaration = literal(value.clone(), call);
        let output = locals
            .split(&declaration, &template, &scope, &live, &mut context)
            .unwrap();
        assert_eq!(output.len(), 7);
        let NirStmt::Let {
            name,
            ty: Some(kind),
            value: snapshot,
        } = &output[1]
        else {
            unreachable!()
        };
        assert_eq!(kind, &ty("Packet"));
        assert_eq!(snapshot, &value);
        for (index, field, kind) in [(2, "left", "i64"), (3, "flag", "bool"), (4, "right", "i64")] {
            let NirStmt::Let {
                ty: Some(t), value, ..
            } = &output[index]
            else {
                unreachable!()
            };
            assert_eq!(t, &ty(kind));
            assert_eq!(
                *value,
                NirExpr::FieldAccess {
                    base: Box::new(NirExpr::Var(name.clone())),
                    field: field.into()
                }
            );
        }
        assert!(scope.iter().all(|p| p.name != "prior"));
        assert!(
            context.pending.is_empty()
                && context.outlines.functions.is_empty()
                && context.outlines.owners.is_empty()
        );
    }
}

#[test]
fn resource_predicate_record_locals_invalidate_rebindings_and_control_flow() {
    let (_, context, _, _, _) = fixture();
    for invalidation in [
        binding("prior", "i64"),
        binding("prior", "Bytes"),
        binding("prior", "Unknown"),
        NirStmt::Let {
            name: "prior".into(),
            ty: None,
            value: NirExpr::Int(0),
        },
        NirStmt::Const {
            name: "prior".into(),
            ty: ty("bool"),
            value: NirExpr::Bool(false),
        },
        NirStmt::If {
            condition: NirExpr::Bool(true),
            then_body: vec![],
            else_body: vec![],
        },
        NirStmt::While {
            condition: NirExpr::Bool(false),
            body: vec![],
        },
    ] {
        let mut locals = LocalRecords::default();
        locals.advance(&binding("prior", "Packet"), &context.calls);
        assert_eq!(
            locals.value_type(&NirExpr::Var("prior".into()), &context.calls),
            Some(ty("Packet"))
        );
        locals.advance(&invalidation, &context.calls);
        assert!(locals
            .value_type(&NirExpr::Var("prior".into()), &context.calls)
            .is_none());
    }
    let mut locals = LocalRecords::default();
    locals.advance(&binding("prior", "Packet"), &context.calls);
    locals.advance(&binding("prior", "Envelope"), &context.calls);
    assert_eq!(
        locals.value_type(&NirExpr::Var("prior".into()), &context.calls),
        Some(ty("Envelope"))
    );
    for kind in [
        NirTypeRef {
            is_ref: true,
            ..ty("Packet")
        },
        NirTypeRef {
            is_optional: true,
            ..ty("Packet")
        },
        NirTypeRef {
            generic_args: vec![ty("i64")],
            ..ty("Packet")
        },
    ] {
        locals.advance(
            &NirStmt::Let {
                name: "prior".into(),
                ty: Some(kind),
                value: NirExpr::Int(0),
            },
            &context.calls,
        );
        assert!(locals.0.is_empty());
    }
}

#[test]
fn resource_predicate_record_locals_reject_nominal_or_late_drift_before_names() {
    for variant in 0..6 {
        let (template, mut context, scope, live, call) = fixture();
        let mut locals = LocalRecords::default();
        locals.advance(
            &binding("prior", if variant == 0 { "Other" } else { "Packet" }),
            &context.calls,
        );
        let value = match variant {
            1 => NirExpr::Var("unavailable".into()),
            2 => NirExpr::FieldAccess {
                base: Box::new(NirExpr::Var("prior".into())),
                field: "missing".into(),
            },
            3 => NirExpr::FieldAccess {
                base: Box::new(NirExpr::Var("prior".into())),
                field: "left".into(),
            },
            _ => NirExpr::Var("prior".into()),
        };
        let mut declaration = literal(value, call);
        let NirStmt::Let {
            value: NirExpr::StructLiteral { fields, .. },
            ..
        } = &mut declaration
        else {
            unreachable!()
        };
        if variant == 4 {
            fields[2].1 = NirExpr::Int(0);
        }
        if variant == 5 {
            fields[2].1 = NirExpr::Bool(true);
        }
        fields.swap(1, 2);
        context.reserved.insert("existing".into());
        let before = context.reserved.clone();
        assert!(locals
            .split(&declaration, &template, &scope, &live, &mut context)
            .is_none());
        assert_eq!(context.reserved, before);
        assert!(context.pending.is_empty() && context.outlines.owners.is_empty());
    }
}
