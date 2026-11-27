use super::*;

fn store(buffer: &str, index: NirExpr, value: NirExpr) -> NirStmt {
    NirStmt::Expr(NirExpr::StoreAt {
        buffer: Box::new(NirExpr::Var(buffer.into())),
        index: Box::new(index),
        value: Box::new(value),
    })
}

#[test]
fn resource_predicate_guarded_stores_share_one_leaf_with_the_return() {
    let (template, mut context, scope, live) = error_fixture();
    let (locals, leaf) = bound_index(&context);
    let read = NirExpr::LoadAt {
        buffer: Box::new(NirExpr::Var("pixels".into())),
        index: Box::new(NirExpr::Int(0)),
    };
    let suffix = |projection: NirExpr| {
        VecDeque::from([
            store(
                "pixels",
                projection.clone(),
                binary(NirBinaryOp::Add, read.clone(), projection.clone()),
            ),
            store("pixels", NirExpr::Int(0), projection.clone()),
            NirStmt::Return(Some(error_value(binary(
                NirBinaryOp::Add,
                projection,
                read.clone(),
            )))),
        ])
    };
    let mut body = suffix(leaf.clone());
    let prefix = stage_guarded_return_tail(
        &mut body,
        "ok",
        &template,
        &scope,
        &live,
        &mut context,
        &locals,
    )
    .unwrap();
    let [NirStmt::Let {
        name,
        value,
        ty: Some(kind),
    }] = prefix.as_slice()
    else {
        panic!("only the completed record field should leave the continuation")
    };
    assert_eq!((value, kind), (&leaf, &ty("i64")));
    assert_eq!(body, suffix(NirExpr::Var(name.clone())));
    let mut scope = scope;
    scope.push(NirParam {
        name: name.clone(),
        ty: kind.clone(),
    });
    let reserved = context.reserved.clone();
    assert!(stage_guarded_return_tail(
        &mut body,
        "ok",
        &template,
        &scope,
        &live,
        &mut context,
        &locals
    )
    .is_none());
    assert_eq!(reserved, context.reserved);
}

#[test]
fn resource_predicate_guarded_stores_reject_an_invalid_prefix_atomically() {
    let (template, mut context, scope, live) = error_fixture();
    let (locals, leaf) = bound_index(&context);
    for invalid in [
        store("saved", NirExpr::Int(0), leaf.clone()),
        store("pixels", NirExpr::Bool(true), leaf.clone()),
        store(
            "pixels",
            NirExpr::Int(0),
            NirExpr::BytesLen(Box::new(NirExpr::Var("missing".into()))),
        ),
        store("pixels", lazy(), NirExpr::Bool(true)),
        store("pixels", moved(), lazy()),
        store("pixels", NirExpr::Int(0), NirExpr::Bool(false)),
        NirStmt::Expr(NirExpr::DropBytes(Box::new(NirExpr::Var("saved".into())))),
    ] {
        let mut body = VecDeque::from([
            store("pixels", NirExpr::Int(0), leaf.clone()),
            invalid,
            NirStmt::Return(Some(error_value(leaf.clone()))),
        ]);
        let original = body.clone();
        assert!(stage_guarded_return_tail(
            &mut body,
            "ok",
            &template,
            &scope,
            &live,
            &mut context,
            &locals
        )
        .is_none());
        assert_eq!(body, original);
        assert!(context.reserved.is_empty());
    }
}

#[test]
fn resource_predicate_lazy_store_binds_index_then_intact_value_before_one_write() {
    let (template, mut context, scope, live) = error_fixture();
    let (locals, leaf) = bound_index(&context);
    let index = NirExpr::LoadAt {
        buffer: Box::new(NirExpr::Var("pixels".into())),
        index: Box::new(NirExpr::Int(1)),
    };
    let value = |leaf| binary(NirBinaryOp::Add, leaf, lazy());
    let stmt = store("pixels", index.clone(), value(leaf.clone()));
    let mut tail = VecDeque::from([NirStmt::Return(Some(error_value(leaf.clone())))]);
    let out = super::super::super::stores::split(
        &stmt,
        &mut tail,
        &template,
        &scope,
        &live,
        &mut context,
        &locals,
    )
    .unwrap();
    let [NirStmt::Let {
        name: snap,
        value: projection,
        ..
    }, NirStmt::Let {
        name: i, value: ix, ..
    }, NirStmt::Let {
        name: v,
        value: rhs,
        ..
    }, write] = out.as_slice()
    else {
        panic!("expected one shared projection, index, guarded value and store")
    };
    assert_eq!(projection, &leaf);
    assert_eq!(ix, &index);
    assert_eq!(rhs, &value(NirExpr::Var(snap.clone())));
    assert_eq!(
        write,
        &store("pixels", NirExpr::Var(i.clone()), NirExpr::Var(v.clone()))
    );
    assert_eq!(
        tail,
        VecDeque::from([NirStmt::Return(Some(error_value(NirExpr::Var(
            snap.clone()
        ))))])
    );
}

#[test]
fn resource_predicate_two_lazy_store_operands_keep_order_and_one_write() {
    let (template, mut context, scope, live) = error_fixture();
    let index = binary(NirBinaryOp::Div, lazy(), NirExpr::Int(9));
    let value = binary(NirBinaryOp::Add, NirExpr::Int(1), lazy());
    let statement = store("pixels", index.clone(), value.clone());
    let mut tail = VecDeque::from([NirStmt::Return(Some(error_value(NirExpr::Int(7))))]);
    let original = tail.clone();
    let out = super::super::super::stores::split(
        &statement,
        &mut tail,
        &template,
        &scope,
        &live,
        &mut context,
        &LocalRecords::default(),
    )
    .unwrap();
    let [NirStmt::Let {
        name: i,
        value: first,
        ty: Some(a),
    }, NirStmt::Let {
        name: v,
        value: second,
        ty: Some(b),
    }, write] = out.as_slice()
    else {
        panic!("expected index, value, one store");
    };
    assert_ne!(i, v);
    assert_eq!((a, b), (&ty("i64"), &ty("i64")));
    assert_eq!((first, second), (&index, &value));
    assert_eq!(
        write,
        &store("pixels", NirExpr::Var(i.clone()), NirExpr::Var(v.clone()))
    );
    assert_eq!(tail, original);
}
