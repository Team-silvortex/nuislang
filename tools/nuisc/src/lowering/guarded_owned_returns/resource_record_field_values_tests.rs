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

fn bundle(call: NirExpr) -> NirExpr {
    let NirExpr::Call { args, .. } = &call else {
        unreachable!()
    };
    NirExpr::StructLiteral {
        type_name: "Bundle".into(),
        type_args: vec![],
        fields: vec![
            ("before".into(), args[0].clone()),
            ("after".into(), args[1].clone()),
            ("packet".into(), call),
        ],
    }
}

fn reject(value: NirExpr) {
    let (template, mut context, scope, live, _) = fixture();
    context.reserved.insert("existing".into());
    let before = context.reserved.clone();
    let statement = NirStmt::Let {
        name: "local".into(),
        ty: Some(ty("Bundle")),
        value: value.clone(),
    };
    assert!(split(
        &statement,
        &ty("Bundle"),
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
fn resource_predicate_record_fields_snapshot_one_call_before_later_lazy_fields() {
    for constant in [false, true] {
        for projected in [false, true] {
            let (template, mut context, scope, live, call) = fixture();
            let mut value = bundle(call);
            let NirExpr::StructLiteral { fields, .. } = &mut value else {
                unreachable!()
            };
            fields.swap(1, 2);
            if projected {
                let NirExpr::Call { callee, .. } = &mut fields[1].1 else {
                    unreachable!()
                };
                *callee = "envelope".into();
                fields[1].1 = NirExpr::FieldAccess {
                    base: Box::new(fields[1].1.clone()),
                    field: "packet".into(),
                };
            }
            let original = fields.clone();
            let statement = if constant {
                NirStmt::Const {
                    name: "local".into(),
                    ty: ty("Bundle"),
                    value: value.clone(),
                }
            } else {
                NirStmt::Let {
                    name: "local".into(),
                    ty: Some(ty("Bundle")),
                    value: value.clone(),
                }
            };
            context
                .reserved
                .insert("__nuis_resource_predicate_left".into());
            let output = split(
                &statement,
                &ty("Bundle"),
                &value,
                &template,
                &scope,
                &live,
                &mut context,
            )
            .unwrap();
            assert_eq!(output.len(), 7);
            let mut names = Vec::new();
            for statement in &output[..6] {
                let NirStmt::Let { name, .. } = statement else {
                    unreachable!()
                };
                assert_ne!(name, "__nuis_resource_predicate_left");
                assert!(!names.contains(name));
                names.push(name.clone());
            }
            for (index, source, expected) in [
                (0, &original[0].1, ty("i64")),
                (1, &original[1].1, ty("Packet")),
                (5, &original[2].1, ty("bool")),
            ] {
                let NirStmt::Let {
                    ty: Some(kind),
                    value,
                    ..
                } = &output[index]
                else {
                    unreachable!()
                };
                assert_eq!(value, source);
                assert_eq!(kind, &expected);
            }
            for (index, field, kind) in
                [(2, "left", "i64"), (3, "flag", "bool"), (4, "right", "i64")]
            {
                assert_eq!(
                    output[index],
                    NirStmt::Let {
                        name: names[index].clone(),
                        ty: Some(ty(kind)),
                        value: NirExpr::FieldAccess {
                            base: Box::new(NirExpr::Var(names[1].clone())),
                            field: field.into()
                        }
                    }
                );
            }
            let mut expected = statement.clone();
            let (NirStmt::Let { value, .. } | NirStmt::Const { value, .. }) = &mut expected else {
                unreachable!()
            };
            *value = NirExpr::StructLiteral {
                type_name: "Bundle".into(),
                type_args: vec![],
                fields: vec![
                    ("before".into(), NirExpr::Var(names[0].clone())),
                    (
                        "packet".into(),
                        NirExpr::StructLiteral {
                            type_name: "Packet".into(),
                            type_args: vec![],
                            fields: ["left", "flag", "right"]
                                .into_iter()
                                .enumerate()
                                .map(|(i, f)| (f.into(), NirExpr::Var(names[i + 2].clone())))
                                .collect(),
                        },
                    ),
                    ("after".into(), NirExpr::Var(names[5].clone())),
                ],
            };
            assert_eq!(output.last(), Some(&expected));
            assert!(context.pending.is_empty());
        }
    }
}

#[test]
fn resource_predicate_record_fields_reject_invalid_helpers_and_late_fields_atomically() {
    let (_, _, _, _, call) = fixture();
    for variant in 0..8 {
        let mut value = bundle(call.clone());
        let NirExpr::StructLiteral { fields, .. } = &mut value else {
            unreachable!()
        };
        fields.swap(1, 2);
        let NirExpr::Call { callee, args } = &mut fields[1].1 else {
            unreachable!()
        };
        match variant {
            0 => *callee = "unknown_or_effectful".into(),
            1 => *callee = "envelope".into(),
            2 => {
                args.pop();
            }
            3 => args[0] = NirExpr::Bool(false),
            4 => args[0] = NirExpr::Var("buffer".into()),
            5 => {
                args[1] = NirExpr::Binary {
                    op: NirBinaryOp::And,
                    lhs: Box::new(NirExpr::Bool(false)),
                    rhs: Box::new(NirExpr::Binary {
                        op: NirBinaryOp::Gt,
                        lhs: Box::new(NirExpr::BytesLen(Box::new(NirExpr::Var("dead".into())))),
                        rhs: Box::new(NirExpr::Int(0)),
                    }),
                }
            }
            6 => fields[2].1 = NirExpr::Int(0),
            7 => fields[1].1 = NirExpr::Var("previous_record".into()),
            _ => unreachable!(),
        }
        reject(value);
    }
}

#[test]
fn resource_predicate_record_fields_keep_original_budgets_and_ready_calls() {
    let (_, _, _, _, call) = fixture();
    for variant in 0..4 {
        let mut value = bundle(call.clone());
        let NirExpr::StructLiteral { fields, .. } = &mut value else {
            unreachable!()
        };
        let NirExpr::Call { args, .. } = &mut fields[2].1 else {
            unreachable!()
        };
        match variant {
            0 => {
                for _ in 0..64 {
                    args[0] = NirExpr::Binary {
                        op: NirBinaryOp::Add,
                        lhs: Box::new(args[0].clone()),
                        rhs: Box::new(NirExpr::Int(1)),
                    };
                }
            }
            1 => args.extend(vec![NirExpr::Int(0); 4096]),
            2 => {
                for _ in 0..32 {
                    args[1] = NirExpr::Binary {
                        op: NirBinaryOp::And,
                        lhs: Box::new(NirExpr::Bool(true)),
                        rhs: Box::new(args[1].clone()),
                    };
                }
            }
            3 => {
                *args = vec![NirExpr::Int(0), NirExpr::Bool(true), NirExpr::Int(1)];
                fields[1].1 = NirExpr::Bool(true);
            }
            _ => unreachable!(),
        }
        reject(value);
    }
}

#[test]
fn resource_predicate_record_fields_bound_complete_snapshot_layouts_before_expansion() {
    let module = crate::pipeline::compile_source("mod cpu Main { struct Leaf { value: i64 } struct Root { child: Leaf } fn main() -> i64 { return 0; } }").unwrap().nir;
    for count in [4095, 4096] {
        let mut module = module.clone();
        let definition = module
            .structs
            .iter_mut()
            .find(|s| s.name == "Leaf")
            .unwrap();
        let field = definition.fields[0].clone();
        definition.fields = (0..count)
            .map(|i| {
                let mut field = field.clone();
                field.name = format!("f{i}");
                field
            })
            .collect();
        let calls = PureValueCalls::collect(&module);
        assert_eq!(snapshot_layout(&ty("Leaf"), &calls), count == 4095);
        assert!(
            !snapshot_layout(&ty("Root"), &calls),
            "parent counts toward the same expansion budget"
        );
    }
    for kind in ["f32", "i32", "Bytes", "Buffer", "Unknown", "Root"] {
        let mut module = module.clone();
        module
            .structs
            .iter_mut()
            .find(|s| s.name == "Leaf")
            .unwrap()
            .fields[0]
            .ty = ty(kind);
        let calls = PureValueCalls::collect(&module);
        assert!(!snapshot_layout(&ty("Root"), &calls), "{kind}");
    }
}
