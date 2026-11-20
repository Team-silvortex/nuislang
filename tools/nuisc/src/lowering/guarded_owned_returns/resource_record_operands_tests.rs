use super::super::super::record_tests::fixture;
use super::*;

pub(super) fn ty(name: &str) -> NirTypeRef {
    NirTypeRef {
        name: name.into(),
        is_ref: false,
        is_optional: false,
        generic_args: vec![],
    }
}

pub(super) fn field(path: &str) -> NirExpr {
    let mut names = path.split('.');
    let root = NirExpr::Var(names.next().unwrap().into());
    names.fold(root, |base, name| NirExpr::FieldAccess {
        base: Box::new(base),
        field: name.into(),
    })
}

pub(super) fn binary(op: NirBinaryOp, lhs: NirExpr, rhs: NirExpr) -> NirExpr {
    NirExpr::Binary {
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    }
}

pub(super) fn binding(name: &str, kind: &str, value: NirExpr) -> NirStmt {
    NirStmt::Let {
        name: name.into(),
        ty: Some(ty(kind)),
        value,
    }
}

fn source(gate: NirExpr) -> NirStmt {
    binding(
        "local",
        "Double",
        NirExpr::StructLiteral {
            type_name: "Double".into(),
            type_args: vec![],
            fields: vec![
                ("gate".into(), gate),
                ("first".into(), field("prior")),
                ("second".into(), field("prior.packet")),
            ],
        },
    )
}

pub(super) fn predicate() -> NirExpr {
    binary(
        NirBinaryOp::And,
        field("prior.packet.flag"),
        binary(
            NirBinaryOp::Eq,
            NirExpr::LoadAt {
                buffer: Box::new(field("buffer")),
                index: Box::new(field("prior.packet.left")),
            },
            field("prior.packet.right"),
        ),
    )
}

pub(super) fn restore(value: &NirExpr, snapshots: &BTreeMap<String, NirExpr>) -> NirExpr {
    if let NirExpr::Var(name) = value {
        if let Some(value) = snapshots.get(name) {
            return value.clone();
        }
    }
    children(value, |child| restore(child, snapshots))
}

#[test]
fn resource_predicate_record_operands_share_exact_scalar_snapshots_with_late_record_paths() {
    for constant in [false, true] {
        let (template, mut context, scope, live, _) = fixture();
        let mut locals = LocalRecords::default();
        locals.advance(
            &binding("prior", "Envelope", NirExpr::Int(0)),
            &context.calls,
        );
        let mut source = source(predicate());
        if constant {
            let NirStmt::Let {
                name,
                ty: Some(ty),
                value,
            } = source
            else {
                unreachable!()
            };
            source = NirStmt::Const { name, ty, value };
        }
        let original = source.clone();
        context
            .reserved
            .insert("__nuis_resource_predicate_left".into());
        let out = locals
            .split(&source, &template, &scope, &live, &mut context)
            .unwrap();
        assert_eq!(source, original);
        assert_eq!(out.len(), 5, "three shared leaves, lazy gate, constructor");
        let mut snapshots = BTreeMap::new();
        for (stmt, path, kind) in [
            (&out[0], "prior.packet.flag", "bool"),
            (&out[1], "prior.packet.left", "i64"),
            (&out[2], "prior.packet.right", "i64"),
        ] {
            let NirStmt::Let {
                name,
                ty: Some(t),
                value,
            } = stmt
            else {
                unreachable!()
            };
            assert_eq!(t, &ty(kind));
            assert_eq!(value, &field(path));
            assert_ne!(name, "__nuis_resource_predicate_left");
            assert!(snapshots.insert(name.clone(), value.clone()).is_none());
        }
        let NirStmt::Let { value, .. } = &out[3] else {
            unreachable!()
        };
        assert_eq!(
            restore(value, &snapshots),
            predicate(),
            "read stays inside the selected RHS"
        );
        assert_eq!(matches!(out[4], NirStmt::Const { .. }), constant);
        assert!(context.pending.is_empty() && context.outlines.owners.is_empty());
    }
}

#[test]
fn resource_predicate_record_operands_preserve_helper_wrappers_and_checked_arguments() {
    let (template, mut context, scope, live, _) = fixture();
    let mut locals = LocalRecords::default();
    locals.advance(
        &binding("prior", "Envelope", NirExpr::Int(0)),
        &context.calls,
    );
    let call = NirExpr::Call {
        callee: "packet".into(),
        args: vec![
            binary(
                NirBinaryOp::Div,
                NirExpr::Int(100),
                field("prior.packet.left"),
            ),
            binary(NirBinaryOp::Xor, predicate(), field("prior.packet.flag")),
            field("prior.packet.right"),
        ],
    };
    let source = binding(
        "local",
        "Pair",
        NirExpr::StructLiteral {
            type_name: "Pair".into(),
            type_args: vec![],
            fields: vec![
                ("gate".into(), predicate()),
                ("first".into(), call.clone()),
                ("second".into(), field("prior.packet")),
            ],
        },
    );
    let out = locals
        .split(&source, &template, &scope, &live, &mut context)
        .unwrap();
    let snapshots = out[..3]
        .iter()
        .map(|s| {
            let NirStmt::Let { name, value, .. } = s else {
                unreachable!()
            };
            (name.clone(), value.clone())
        })
        .collect();
    let NirStmt::Let { value, .. } = &out[4] else {
        unreachable!()
    };
    assert_eq!(restore(value, &snapshots), call);
    assert_eq!(out.iter().filter(|s| matches!(s, NirStmt::Let { value: NirExpr::Call { callee, .. }, .. } if callee == "packet")).count(), 1);
}

#[test]
fn resource_predicate_record_operands_reject_invalid_trees_atomically() {
    for variant in 0..10 {
        let (template, mut context, scope, live, _) = fixture();
        let mut locals = LocalRecords::default();
        locals.advance(
            &binding("prior", "Envelope", NirExpr::Int(0)),
            &context.calls,
        );
        let mut gate = predicate();
        match variant {
            0 => locals.advance(&binding("prior", "i64", NirExpr::Int(0)), &context.calls),
            1 => locals.advance(
                &NirStmt::If {
                    condition: NirExpr::Bool(true),
                    then_body: vec![],
                    else_body: vec![],
                },
                &context.calls,
            ),
            2 => gate = binary(NirBinaryOp::And, field("prior.packet.left"), predicate()),
            3 => gate = binary(NirBinaryOp::And, field("prior.packet.missing"), predicate()),
            4 => gate = binary(NirBinaryOp::And, field("unknown.packet.flag"), predicate()),
            5 => {
                gate = binary(
                    NirBinaryOp::And,
                    field("prior.packet.flag"),
                    binary(
                        NirBinaryOp::Eq,
                        NirExpr::BytesLen(Box::new(field("dead"))),
                        NirExpr::Int(0),
                    ),
                )
            }
            6 => {
                gate = binary(
                    NirBinaryOp::And,
                    predicate(),
                    NirExpr::Call {
                        callee: "effectful_or_unknown".into(),
                        args: vec![field("prior.packet.flag")],
                    },
                )
            }
            7 => {
                for _ in 0..33 {
                    gate = binary(NirBinaryOp::Or, field("prior.packet.flag"), gate);
                }
            }
            8 => {
                for _ in 0..64 {
                    gate = binary(NirBinaryOp::Eq, gate, NirExpr::Bool(false));
                }
            }
            9 => gate = field("prior.packet.flag"), // Ready constructors stay on their old route.
            _ => unreachable!(),
        }
        context.reserved.insert("existing".into());
        let reserved = context.reserved.clone();
        assert!(
            locals
                .split(&source(gate), &template, &scope, &live, &mut context)
                .is_none(),
            "variant {variant}"
        );
        assert_eq!(context.reserved, reserved);
        assert!(context.pending.is_empty() && context.outlines.owners.is_empty());
    }
}
