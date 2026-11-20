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

fn literal(call: NirExpr) -> NirExpr {
    let NirExpr::Call { args, .. } = call else {
        unreachable!()
    };
    NirExpr::StructLiteral {
        type_name: "Packet".into(),
        type_args: vec![],
        fields: vec![
            ("right".into(), args[2].clone()),
            ("left".into(), args[0].clone()),
            ("flag".into(), args[1].clone()),
        ],
    }
}

#[test]
fn resource_predicate_record_literals_preserve_nominal_bindings_and_source_order() {
    for constant in [false, true] {
        for nested in [false, true] {
            let (template, mut context, scope, live, call) = fixture();
            let value = literal(call);
            let NirExpr::StructLiteral { fields, .. } = &value else {
                unreachable!()
            };
            let original_fields = fields.clone();
            let (kind, value) = if nested {
                (
                    ty("Envelope"),
                    NirExpr::StructLiteral {
                        type_name: "Envelope".into(),
                        type_args: vec![],
                        fields: vec![("packet".into(), value)],
                    },
                )
            } else {
                (ty("Packet"), value)
            };
            let statement = if constant {
                NirStmt::Const {
                    name: "local".into(),
                    ty: kind.clone(),
                    value: value.clone(),
                }
            } else {
                NirStmt::Let {
                    name: "local".into(),
                    ty: Some(kind.clone()),
                    value: value.clone(),
                }
            };
            context
                .reserved
                .insert("__nuis_resource_predicate_left".into());
            let output = split(
                &statement,
                &kind,
                &value,
                &template,
                &scope,
                &live,
                &mut context,
            )
            .unwrap();
            assert_eq!(output.len(), 4);
            let mut names = BTreeSet::new();
            for (index, s) in output[..3].iter().enumerate() {
                let NirStmt::Let {
                    name,
                    ty: Some(kind),
                    value,
                } = s
                else {
                    unreachable!()
                };
                assert_ne!(name, "__nuis_resource_predicate_left");
                assert!(names.insert(name.clone()));
                assert_eq!(kind, &ty(if index == 2 { "bool" } else { "i64" }));
                assert_eq!(
                    value, &original_fields[index].1,
                    "lazy children must stay intact"
                );
            }
            let mut expected = statement.clone();
            let (NirStmt::Let { value: target, .. } | NirStmt::Const { value: target, .. }) =
                &mut expected
            else {
                unreachable!()
            };
            let NirExpr::StructLiteral { fields, .. } = target else {
                unreachable!()
            };
            let fields = if nested {
                let NirExpr::StructLiteral { fields, .. } = &mut fields[0].1 else {
                    unreachable!()
                };
                fields
            } else {
                fields
            };
            for (field, statement) in fields.iter_mut().zip(&output[..3]) {
                let NirStmt::Let { name, .. } = statement else {
                    unreachable!()
                };
                field.1 = NirExpr::Var(name.clone());
            }
            assert_eq!(output.last(), Some(&expected));
            assert!(context.pending.is_empty());
        }
    }
}

fn rejected(kind: NirTypeRef, value: NirExpr) {
    let (template, mut context, scope, live, _) = fixture();
    context.reserved.insert("existing".into());
    let before = context.reserved.clone();
    let statement = NirStmt::Let {
        name: "local".into(),
        ty: Some(kind.clone()),
        value: value.clone(),
    };
    assert!(split(
        &statement,
        &kind,
        &value,
        &template,
        &scope,
        &live,
        &mut context
    )
    .is_none());
    assert_eq!(context.reserved, before);
    assert!(context.pending.is_empty());
    assert!(context.outlines.functions.is_empty());
    assert!(context.outlines.owners.is_empty());
}

#[test]
fn resource_predicate_record_literals_reject_whole_constructor_drift_before_names() {
    let (_, _, _, _, call) = fixture();
    let original = literal(call);
    for variant in 0..11 {
        let mut value = original.clone();
        let NirExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } = &mut value
        else {
            unreachable!()
        };
        match variant {
            0 => *type_name = "Other".into(),
            1 => type_args.push(ty("i64")),
            2 => {
                fields.pop();
            }
            3 => fields.push(fields[0].clone()),
            4 => fields[1].0 = fields[0].0.clone(),
            5 => fields[1].0 = "unknown".into(),
            6 => fields[0].1 = NirExpr::Bool(false),
            7 => fields[0].1 = NirExpr::Var("buffer".into()),
            8 => fields[0].1 = NirExpr::Move(Box::new(NirExpr::Var("saved".into()))),
            9 => {
                fields[0].1 = NirExpr::Call {
                    callee: "unknown".into(),
                    args: vec![],
                }
            }
            10 => {
                fields[2].1 = NirExpr::Binary {
                    op: NirBinaryOp::And,
                    lhs: Box::new(NirExpr::Bool(false)),
                    rhs: Box::new(NirExpr::Binary {
                        op: NirBinaryOp::Gt,
                        lhs: Box::new(NirExpr::BytesLen(Box::new(NirExpr::Var("dead".into())))),
                        rhs: Box::new(NirExpr::Int(0)),
                    }),
                }
            }
            _ => unreachable!(),
        }
        rejected(ty("Packet"), value);
    }
    for kind in [
        ty("Other"),
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
        rejected(kind, original.clone());
    }
    let nested = NirExpr::StructLiteral {
        type_name: "Envelope".into(),
        type_args: vec![],
        fields: vec![(
            "packet".into(),
            NirExpr::StructLiteral {
                type_name: "Other".into(),
                type_args: vec![],
                fields: match original {
                    NirExpr::StructLiteral { fields, .. } => fields,
                    _ => unreachable!(),
                },
            },
        )],
    };
    rejected(ty("Envelope"), nested);
}

#[test]
fn resource_predicate_record_literals_keep_shared_budgets_and_ready_routes() {
    let (_, _, _, _, call) = fixture();
    for variant in 0..4 {
        let mut value = literal(call.clone());
        let NirExpr::StructLiteral { fields, .. } = &mut value else {
            unreachable!()
        };
        match variant {
            0 => {
                let mut tree = NirExpr::Int(1);
                for _ in 0..10 {
                    tree = NirExpr::Binary {
                        op: NirBinaryOp::Add,
                        lhs: Box::new(tree.clone()),
                        rhs: Box::new(tree),
                    };
                }
                assert!(bounded(&tree).is_some());
                fields[0].1 = tree.clone();
                fields[1].1 = tree;
                assert!(
                    bounded_tree(&value, true).is_none(),
                    "one whole-constructor work budget"
                );
            }
            1 => {
                for _ in 0..64 {
                    fields[0].1 = NirExpr::Binary {
                        op: NirBinaryOp::Add,
                        lhs: Box::new(fields[0].1.clone()),
                        rhs: Box::new(NirExpr::Int(1)),
                    };
                }
            }
            2 => {
                for _ in 0..33 {
                    fields[2].1 = NirExpr::Binary {
                        op: NirBinaryOp::And,
                        lhs: Box::new(NirExpr::Bool(true)),
                        rhs: Box::new(fields[2].1.clone()),
                    };
                }
            }
            3 => fields[2].1 = NirExpr::Bool(true),
            _ => unreachable!(),
        }
        rejected(ty("Packet"), value);
    }
}
