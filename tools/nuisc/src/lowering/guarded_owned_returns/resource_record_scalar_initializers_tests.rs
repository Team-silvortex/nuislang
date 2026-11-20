use super::super::super::record_tests::fixture;
use super::tests::{binary, binding, field, predicate, restore, ty};
use super::*;

#[test]
fn resource_predicate_local_record_scalar_declarations_preserve_types_shadowing_and_requeue_once() {
    for integer in [false, true] {
        for constant in [false, true] {
            let (template, mut context, mut scope, live, _) = fixture();
            let mut locals = LocalRecords::default();
            locals.advance(
                &binding("prior", "Envelope", NirExpr::Int(0)),
                &context.calls,
            );
            let value = if integer {
                NirExpr::FieldAccess {
                    base: Box::new(NirExpr::Call {
                        callee: "packet".into(),
                        args: vec![
                            NirExpr::LoadAt {
                                buffer: Box::new(field("buffer")),
                                index: Box::new(field("prior.packet.left")),
                            },
                            predicate(),
                            binary(NirBinaryOp::Div, NirExpr::Int(100), field("denominator")),
                        ],
                    }),
                    field: "right".into(),
                }
            } else {
                binary(
                    NirBinaryOp::Xor,
                    predicate(),
                    binary(NirBinaryOp::Eq, field("prior.packet.flag"), field("flag")),
                )
            };
            let (name, kind) = if integer {
                ("denominator", "i64")
            } else {
                ("flag", "bool")
            };
            let source = if constant {
                NirStmt::Const {
                    name: name.into(),
                    ty: ty(kind),
                    value: value.clone(),
                }
            } else {
                binding(name, kind, value.clone())
            };
            let original = source.clone();
            context
                .reserved
                .insert("__nuis_resource_predicate_left".into());
            let out = locals
                .split(&source, &template, &scope, &live, &mut context)
                .unwrap();
            assert_eq!(source, original);
            assert_eq!(
                out.len(),
                4,
                "three unique field snapshots and the original declaration"
            );
            let mut snapshots = BTreeMap::new();
            for statement in &out[..3] {
                let NirStmt::Let {
                    name,
                    ty: Some(t),
                    value,
                } = statement
                else {
                    unreachable!()
                };
                assert_ne!(name, "__nuis_resource_predicate_left");
                assert_eq!(locals.value_type(value, &context.calls).as_ref(), Some(t));
                assert!(capture_scalar(t));
                assert!(snapshots.insert(name.clone(), value.clone()).is_none());
                scope.push(NirParam {
                    name: name.clone(),
                    ty: t.clone(),
                });
            }
            let (actual_name, actual_ty, actual_value) = match &out[3] {
                NirStmt::Let {
                    name,
                    ty: Some(t),
                    value,
                } if !constant => (name, t, value),
                NirStmt::Const { name, ty: t, value } if constant => (name, t, value),
                _ => panic!("declaration identity changed"),
            };
            assert_eq!(actual_name, name);
            assert_eq!(actual_ty, &ty(kind));
            assert_eq!(
                restore(actual_value, &snapshots),
                value,
                "leave reads, calls and checked arithmetic in place"
            );
            let reserved = context.reserved.clone();
            assert!(locals
                .split(&out[3], &template, &scope, &live, &mut context)
                .is_none());
            assert_eq!(context.reserved, reserved);
            assert!(context.pending.is_empty() && context.outlines.owners.is_empty());
        }
    }
}

#[test]
fn resource_predicate_local_record_scalar_rejection_is_atomic_for_type_scope_owner_and_budget_drift(
) {
    for variant in 0..15 {
        let (template, mut context, scope, live, _) = fixture();
        let mut locals = LocalRecords::default();
        locals.advance(
            &binding("prior", "Envelope", NirExpr::Int(0)),
            &context.calls,
        );
        let mut value = predicate();
        let mut kind = ty("bool");
        match variant {
            0 => kind = ty("i64"),
            1 => kind = ty("f64"),
            2 => kind.is_ref = true,
            3 => kind.is_optional = true,
            4 => kind.generic_args.push(ty("i64")),
            5 => value = binary(NirBinaryOp::And, field("prior.packet.left"), value),
            6 => value = binary(NirBinaryOp::And, field("prior.packet.missing"), value),
            7 => value = binary(NirBinaryOp::And, field("absent.packet.flag"), value),
            8 => locals.advance(&binding("prior", "Bytes", NirExpr::Int(0)), &context.calls),
            9 => locals.advance(
                &NirStmt::If {
                    condition: NirExpr::Bool(true),
                    then_body: vec![],
                    else_body: vec![],
                },
                &context.calls,
            ),
            10 => {
                value = binary(
                    NirBinaryOp::And,
                    field("prior.packet.flag"),
                    binary(
                        NirBinaryOp::Eq,
                        NirExpr::BytesLen(Box::new(field("dead"))),
                        NirExpr::Int(0),
                    ),
                )
            }
            11 => {
                value = binary(
                    NirBinaryOp::And,
                    value,
                    NirExpr::Call {
                        callee: "unknown_or_effectful".into(),
                        args: vec![field("prior.packet.flag")],
                    },
                )
            }
            12 => {
                for _ in 0..33 {
                    value = binary(NirBinaryOp::Or, field("prior.packet.flag"), value);
                }
            }
            13 => {
                for _ in 0..64 {
                    value = binary(NirBinaryOp::Eq, value, NirExpr::Bool(false));
                }
            }
            14 => {
                value = NirExpr::Call {
                    callee: "packet".into(),
                    args: vec![value; 4096],
                }
            }
            _ => unreachable!(),
        }
        let source = NirStmt::Let {
            name: "result".into(),
            ty: Some(kind),
            value,
        };
        context.reserved.insert("existing".into());
        let reserved = context.reserved.clone();
        assert!(
            locals
                .split(&source, &template, &scope, &live, &mut context)
                .is_none(),
            "variant {variant}"
        );
        assert_eq!(context.reserved, reserved);
        assert!(context.pending.is_empty() && context.outlines.owners.is_empty());
    }
}

#[test]
fn resource_predicate_local_record_scalars_leave_ready_pure_and_unrelated_routes_unchanged() {
    let (template, mut context, scope, live, call) = fixture();
    let mut locals = LocalRecords::default();
    locals.advance(
        &binding("prior", "Envelope", NirExpr::Int(0)),
        &context.calls,
    );
    let NirExpr::Call { args, .. } = call else {
        unreachable!()
    };
    for value in [
        field("prior.packet.flag"),
        binary(
            NirBinaryOp::And,
            field("prior.packet.flag"),
            NirExpr::Bool(true),
        ),
        args[1].clone(),
    ] {
        assert!(locals
            .split(
                &binding("flag", "bool", value),
                &template,
                &scope,
                &live,
                &mut context
            )
            .is_none());
        assert!(context.reserved.is_empty() && context.pending.is_empty());
    }
}
