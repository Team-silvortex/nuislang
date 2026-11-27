use super::super::super::record_tests::fixture;
use super::tests::{binary, binding, field, predicate, restore};
use super::*;

fn branch(condition: NirExpr) -> NirStmt {
    NirStmt::If {
        condition,
        then_body: vec![NirStmt::Return(Some(NirExpr::Int(7)))],
        else_body: vec![NirStmt::Expr(NirExpr::CopyBufferOwned(Box::new(field(
            "buffer",
        ))))],
    }
}

#[test]
fn resource_predicate_record_conditions_preserve_arms_expressions_and_requeue_once() {
    let (template, mut context, mut scope, live, _) = fixture();
    let mut locals = LocalRecords::default();
    locals.advance(
        &binding("prior", "Envelope", NirExpr::Int(0)),
        &context.calls,
    );
    let condition = binary(NirBinaryOp::Xor, predicate(), field("prior.packet.flag"));
    let source = branch(condition.clone());
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
        "three shared scalar leaves and one intact branch"
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
    let mut restored = out[3].clone();
    let NirStmt::If {
        condition: actual, ..
    } = &mut restored
    else {
        unreachable!()
    };
    *actual = restore(actual, &snapshots);
    assert_eq!(
        restored, original,
        "never clone arms into lazy edges or move reads"
    );
    let reserved = context.reserved.clone();
    assert!(locals
        .split(&out[3], &template, &scope, &live, &mut context)
        .is_none());
    assert_eq!(context.reserved, reserved);
    let mut suffix = VecDeque::from(template.body.clone());
    let original_suffix = suffix.clone();
    let staged = super::super::super::split(
        &out[3],
        &template,
        &mut suffix,
        &scope,
        &live,
        1,
        &mut context,
        &Default::default(),
    )
    .unwrap()
    .unwrap();
    assert_eq!(suffix, original_suffix);
    assert_eq!(
        staged.len(),
        2,
        "existing condition route stages the bool exactly once"
    );
    assert!(context.pending.is_empty() && context.outlines.owners.is_empty());
}

#[test]
fn resource_predicate_record_conditions_reject_scope_type_owner_and_budget_drift_atomically() {
    for variant in 0..10 {
        let (template, mut context, scope, live, _) = fixture();
        let mut locals = LocalRecords::default();
        locals.advance(
            &binding("prior", "Envelope", NirExpr::Int(0)),
            &context.calls,
        );
        let mut condition = predicate();
        match variant {
            0 => condition = binary(NirBinaryOp::And, field("prior.packet.left"), condition),
            1 => condition = binary(NirBinaryOp::Or, field("prior.packet.missing"), condition),
            2 => locals.advance(&binding("prior", "Bytes", NirExpr::Int(0)), &context.calls),
            3 => locals.advance(&branch(NirExpr::Bool(false)), &context.calls),
            4 => {
                condition = binary(
                    NirBinaryOp::And,
                    field("prior.packet.flag"),
                    binary(
                        NirBinaryOp::Gt,
                        NirExpr::BytesLen(Box::new(field("dead"))),
                        NirExpr::Int(0),
                    ),
                )
            }
            5 => {
                condition = binary(
                    NirBinaryOp::Or,
                    condition,
                    NirExpr::Call {
                        callee: "unknown_effect".into(),
                        args: vec![],
                    },
                )
            }
            6 => {
                for _ in 0..33 {
                    condition = binary(NirBinaryOp::Or, condition, field("prior.packet.flag"));
                }
            }
            7 => {
                for _ in 0..64 {
                    condition = binary(NirBinaryOp::Eq, condition, NirExpr::Bool(false));
                }
            }
            8 => {
                condition = NirExpr::Call {
                    callee: "unknown".into(),
                    args: vec![condition; 4096],
                }
            }
            9 => {
                condition = binary(
                    NirBinaryOp::Add,
                    field("prior.packet.left"),
                    NirExpr::Int(1),
                )
            }
            _ => unreachable!(),
        }
        let source = branch(condition);
        let original = source.clone();
        let reserved = context.reserved.clone();
        assert!(
            locals
                .split(&source, &template, &scope, &live, &mut context)
                .is_none(),
            "{variant}"
        );
        assert_eq!(source, original);
        assert_eq!(context.reserved, reserved);
        assert!(context.pending.is_empty() && context.outlines.owners.is_empty());
    }
}

#[test]
fn resource_predicate_record_conditions_keep_ready_pure_and_unrelated_routes() {
    let (template, mut context, scope, live, _) = fixture();
    let mut locals = LocalRecords::default();
    locals.advance(
        &binding("prior", "Envelope", NirExpr::Int(0)),
        &context.calls,
    );
    for value in [
        field("prior.packet.flag"),
        binary(
            NirBinaryOp::And,
            field("prior.packet.flag"),
            NirExpr::Bool(true),
        ),
        binary(
            NirBinaryOp::And,
            field("flag"),
            binary(
                NirBinaryOp::Gt,
                NirExpr::BytesLen(Box::new(field("saved"))),
                NirExpr::Int(0),
            ),
        ),
    ] {
        assert!(locals
            .split(&branch(value), &template, &scope, &live, &mut context)
            .is_none());
        assert!(context.reserved.is_empty());
    }
}
