use super::*;

const SIGNAL: &str = "pending";

fn publish() -> NirStmt {
    NirStmt::Let {
        name: SIGNAL.into(),
        ty: Some(scalar_type("i64")),
        value: NirExpr::Int(1),
    }
}

fn branch(body: Vec<NirStmt>) -> NirStmt {
    NirStmt::If {
        condition: NirExpr::Bool(true),
        then_body: body,
        else_body: vec![],
    }
}

fn child(body: Vec<NirStmt>) -> NirStmt {
    NirStmt::While {
        condition: NirExpr::Bool(true),
        body,
    }
}

#[test]
fn return_signal_requires_canonical_publication_before_every_owned_break() {
    assert!(can_share(&[publish(), NirStmt::Break], SIGNAL));
    assert!(can_share(
        &[branch(vec![publish(), NirStmt::Break])],
        SIGNAL
    ));
    for body in [
        vec![],
        vec![NirStmt::Break],
        vec![publish()],
        vec![publish(), NirStmt::Continue],
        vec![publish(), NirStmt::Expr(NirExpr::Int(0)), NirStmt::Break],
        vec![branch(vec![publish(), NirStmt::Break]), NirStmt::Break],
        vec![publish(), NirStmt::Break, NirStmt::Expr(NirExpr::Int(0))],
    ] {
        assert!(!can_share(&body, SIGNAL), "{body:?}");
    }
    for (ty, value) in [
        (Some(scalar_type("i64")), 0),
        (None, 1),
        (Some(scalar_type("i32")), 1),
    ] {
        let stmt = NirStmt::Let {
            name: SIGNAL.into(),
            ty,
            value: NirExpr::Int(value),
        };
        assert!(!can_share(&[stmt, NirStmt::Break], SIGNAL));
    }
    assert!(!can_share(
        &[
            NirStmt::Const {
                name: SIGNAL.into(),
                ty: scalar_type("i64"),
                value: NirExpr::Int(1)
            },
            NirStmt::Break,
        ],
        SIGNAL
    ));
}

#[test]
fn return_signal_keeps_child_breaks_local_and_continues_independent() {
    let propagation = guard(SIGNAL.into());
    assert!(can_share(
        &[
            child(vec![branch(vec![publish(), NirStmt::Break])]),
            propagation.clone(),
        ],
        SIGNAL
    ));
    assert!(can_share(
        &[
            child(vec![NirStmt::Break]),
            branch(vec![NirStmt::Continue]),
            publish(),
            NirStmt::Break,
        ],
        SIGNAL
    ));
    assert!(!can_share(
        &[child(vec![publish(), NirStmt::Break])],
        SIGNAL
    ));
    assert!(!can_share(&[child(vec![NirStmt::Break])], SIGNAL));
    assert!(!can_share(
        &[
            child(vec![publish(), NirStmt::Continue]),
            propagation.clone()
        ],
        SIGNAL
    ));
    let NirStmt::If { condition, .. } = propagation else {
        unreachable!()
    };
    for (then_body, else_body) in [
        (vec![NirStmt::Continue], vec![]),
        (vec![NirStmt::Break], vec![NirStmt::Break]),
        (vec![publish(), NirStmt::Break], vec![]),
    ] {
        assert!(!can_share(
            &[NirStmt::If {
                condition: condition.clone(),
                then_body,
                else_body
            }],
            SIGNAL
        ));
    }
}

#[test]
fn return_signal_rejects_hidden_reads_and_unknown_syntax() {
    let read = NirExpr::Var(SIGNAL.into());
    for value in [
        read.clone(),
        NirExpr::Text("unsupported scalar profile".into()),
        NirExpr::Await(Box::new(NirExpr::Int(0))),
        NirExpr::CastI64ToI32(Box::new(read.clone())),
        NirExpr::FieldAccess {
            base: Box::new(read.clone()),
            field: "value".into(),
        },
        NirExpr::Call {
            callee: "consume".into(),
            args: vec![read.clone()],
        },
        NirExpr::StructLiteral {
            type_name: "Payload".into(),
            type_args: vec![],
            fields: vec![("value".into(), read.clone())],
        },
        NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(NirExpr::Int(0)),
            rhs: Box::new(read),
        },
    ] {
        assert!(!can_share(
            &[NirStmt::Expr(value), publish(), NirStmt::Break],
            SIGNAL
        ));
    }
    assert!(!can_share(
        &[
            NirStmt::Return(Some(NirExpr::Int(0))),
            publish(),
            NirStmt::Break
        ],
        SIGNAL
    ));
    assert!(!can_share(
        &[NirStmt::Continue, publish(), NirStmt::Break],
        SIGNAL
    ));
}

#[test]
fn return_signal_bounds_both_work_and_depth_without_granting_partial_proof() {
    let body = [publish(), NirStmt::Break];
    assert_eq!(
        Proof {
            signal: SIGNAL,
            remaining: 1
        }
        .block(&body, true, 0),
        None
    );
    assert_eq!(
        Proof {
            signal: SIGNAL,
            remaining: 2
        }
        .block(&body, true, 0),
        Some(true)
    );
    let mut deep = body.to_vec();
    for _ in 0..MAX_DEPTH {
        deep = vec![branch(deep)];
    }
    assert!(!can_share(&deep, SIGNAL));
    let mut wide = vec![NirStmt::Expr(NirExpr::Int(0)); MAX_WORK / 2];
    wide.extend(body);
    assert!(!can_share(&wide, SIGNAL));
}
