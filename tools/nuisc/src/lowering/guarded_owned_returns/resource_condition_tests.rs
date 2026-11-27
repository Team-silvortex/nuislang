use super::*;

fn binary(op: NirBinaryOp, lhs: NirExpr, rhs: NirExpr) -> NirExpr {
    NirExpr::Binary {
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    }
}

fn fixture() -> (
    NirFunction,
    OutlineContext,
    Vec<NirParam>,
    BTreeSet<String>,
    NirExpr,
) {
    let module = crate::pipeline::compile_source("mod cpu Main { fn main() -> i64 { return 0; } }")
        .unwrap()
        .nir;
    let mut template = module.functions[0].clone();
    template.body.insert(
        0,
        NirStmt::Let {
            name: "__nuis_resource_predicate_left".into(),
            ty: None,
            value: NirExpr::Int(0),
        },
    );
    let scope = vec![NirParam {
        name: "flag".into(),
        ty: NirTypeRef {
            name: "bool".into(),
            is_ref: false,
            is_optional: false,
            generic_args: vec![],
        },
    }];
    let value = binary(
        NirBinaryOp::And,
        NirExpr::Var("flag".into()),
        binary(
            NirBinaryOp::Gt,
            NirExpr::BytesLen(Box::new(NirExpr::Var("saved".into()))),
            NirExpr::Int(0),
        ),
    );
    let context = OutlineContext {
        reserved: BTreeSet::new(),
        pending: VecDeque::new(),
        roots: 1,
        outlines: Outlines::default(),
        calls: PureValueCalls::collect(&module),
        return_layouts: BTreeMap::new(),
    };
    (
        template,
        context,
        scope,
        BTreeSet::from(["saved".into()]),
        value,
    )
}

#[test]
fn resource_predicate_inline_condition_requeues_one_intact_bool_and_original_arms() {
    let (template, mut context, scope, live, condition) = fixture();
    context
        .reserved
        .insert("__nuis_resource_predicate_left_".into());
    let statement = NirStmt::If {
        condition: condition.clone(),
        then_body: vec![NirStmt::Return(Some(NirExpr::Int(7)))],
        else_body: vec![NirStmt::Return(Some(NirExpr::Int(9)))],
    };
    let mut suffix = VecDeque::from(template.body.clone());
    let original = suffix.clone();
    let replacement = split(
        &statement,
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
    let [NirStmt::Let {
        name,
        ty: Some(ty),
        value,
    }, branch] = replacement.as_slice()
    else {
        panic!("expected exactly one predicate snapshot and one original branch")
    };
    assert_eq!(name, "__nuis_resource_predicate_left__");
    assert_eq!(ty, &scope[0].ty);
    assert_eq!(value, &condition);
    let mut expected = statement.clone();
    let NirStmt::If { condition, .. } = &mut expected else {
        unreachable!()
    };
    *condition = NirExpr::Var(name.clone());
    assert_eq!(branch, &expected);
    assert_eq!(suffix, original);
    assert!(context.pending.is_empty() && context.outlines.owners.is_empty());
    let mut ready_scope = scope.clone();
    ready_scope.push(NirParam {
        name: name.clone(),
        ty: ty.clone(),
    });
    assert!(
        split(
            branch,
            &template,
            &mut suffix,
            &ready_scope,
            &live,
            1,
            &mut context,
            &Default::default()
        )
        .unwrap()
        .is_none(),
        "a ready condition must not be snapshotted again"
    );
}

#[test]
fn resource_predicate_inline_condition_proof_rejects_invalid_trees_atomically() {
    let (template, _, scope, live, value) = fixture();
    let mut deep = value.clone();
    for _ in 0..64 {
        deep = binary(NirBinaryOp::Eq, deep, NirExpr::Bool(false));
    }
    let mut wide = value.clone();
    for _ in 0..33 {
        wide = binary(NirBinaryOp::And, wide, NirExpr::Bool(true));
    }
    let dead = binary(
        NirBinaryOp::And,
        NirExpr::Bool(false),
        binary(
            NirBinaryOp::Gt,
            NirExpr::BytesLen(Box::new(NirExpr::Var("dead".into()))),
            NirExpr::Int(0),
        ),
    );
    for condition in [
        binary(NirBinaryOp::Eq, value.clone(), NirExpr::Int(1)),
        binary(NirBinaryOp::And, NirExpr::Int(1), value.clone()),
        binary(
            NirBinaryOp::Or,
            value,
            NirExpr::Call {
                callee: "unknown_effect".into(),
                args: vec![],
            },
        ),
        dead,
        deep,
        wide,
    ] {
        let (_, mut context, _, _, _) = fixture();
        let statement = NirStmt::If {
            condition,
            then_body: vec![],
            else_body: vec![],
        };
        let mut suffix = VecDeque::from(template.body.clone());
        let original = suffix.clone();
        assert!(split(
            &statement,
            &template,
            &mut suffix,
            &scope,
            &live,
            1,
            &mut context,
            &Default::default()
        )
        .unwrap()
        .is_none());
        assert_eq!(suffix, original);
        assert!(
            context.reserved.is_empty()
                && context.pending.is_empty()
                && context.outlines.owners.is_empty()
        );
    }
}

#[test]
fn resource_predicate_inline_conditions_keep_pure_and_ready_routes_unchanged() {
    let (template, mut context, scope, live, _) = fixture();
    for condition in [
        NirExpr::Var("flag".into()),
        binary(
            NirBinaryOp::And,
            NirExpr::Var("flag".into()),
            NirExpr::Bool(true),
        ),
        binary(
            NirBinaryOp::Gt,
            NirExpr::BytesLen(Box::new(NirExpr::Var("saved".into()))),
            NirExpr::Int(0),
        ),
    ] {
        let statement = NirStmt::If {
            condition,
            then_body: vec![],
            else_body: vec![],
        };
        let mut suffix = VecDeque::from(template.body.clone());
        assert!(split(
            &statement,
            &template,
            &mut suffix,
            &scope,
            &live,
            1,
            &mut context,
            &Default::default()
        )
        .unwrap()
        .is_none());
        assert!(context.reserved.is_empty());
    }
}
