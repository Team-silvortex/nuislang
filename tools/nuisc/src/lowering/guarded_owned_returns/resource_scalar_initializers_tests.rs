use super::*;

fn binary(op: NirBinaryOp, lhs: NirExpr, rhs: NirExpr) -> NirExpr {
    NirExpr::Binary {
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    }
}

fn ty(name: &str) -> NirTypeRef {
    NirTypeRef {
        name: name.into(),
        is_ref: false,
        is_optional: false,
        generic_args: vec![],
    }
}

fn fixture() -> (
    NirFunction,
    OutlineContext,
    Vec<NirParam>,
    BTreeSet<String>,
    NirExpr,
) {
    let module = crate::pipeline::compile_source(
        r#"mod cpu Main {
        fn combine(left: i64, flag: bool, right: i64) -> i64 {
            if flag { return left; } return right;
        }
        fn main() -> i64 { return 0; }
    }"#,
    )
    .unwrap()
    .nir;
    let template = module
        .functions
        .iter()
        .find(|f| f.name == "main")
        .unwrap()
        .clone();
    let context = OutlineContext {
        reserved: BTreeSet::new(),
        pending: VecDeque::new(),
        roots: 1,
        outlines: Outlines::default(),
        calls: PureValueCalls::collect(&module),
        return_layouts: BTreeMap::new(),
    };
    let scope = vec![
        NirParam {
            name: "buffer".into(),
            ty: NirTypeRef {
                is_ref: true,
                ..ty("Buffer")
            },
        },
        NirParam {
            name: "flag".into(),
            ty: ty("bool"),
        },
        NirParam {
            name: "choice".into(),
            ty: ty("i64"),
        },
    ];
    let logical = binary(
        NirBinaryOp::And,
        NirExpr::Var("flag".into()),
        binary(
            NirBinaryOp::Gt,
            NirExpr::BytesLen(Box::new(NirExpr::Var("saved".into()))),
            NirExpr::Int(0),
        ),
    );
    let value = NirExpr::Call {
        callee: "combine".into(),
        args: vec![
            NirExpr::LoadAt {
                buffer: Box::new(NirExpr::Var("buffer".into())),
                index: Box::new(NirExpr::Int(0)),
            },
            logical,
            binary(
                NirBinaryOp::Div,
                NirExpr::Int(100),
                NirExpr::Var("choice".into()),
            ),
        ],
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
fn resource_predicate_i64_initializers_preserve_order_types_and_lazy_subtrees() {
    for constant in [false, true] {
        let (template, mut context, scope, live, value) = fixture();
        context
            .reserved
            .insert("__nuis_resource_predicate_left".into());
        let statement = if constant {
            NirStmt::Const {
                name: "choice".into(),
                ty: ty("i64"),
                value: value.clone(),
            }
        } else {
            NirStmt::Let {
                name: "choice".into(),
                ty: Some(ty("i64")),
                value: value.clone(),
            }
        };
        let mut suffix = VecDeque::from(template.body.clone());
        let original_suffix = suffix.clone();
        let prefix = split(
            &statement,
            &template,
            &mut suffix,
            &scope,
            &live,
            1,
            &mut context,
        )
        .unwrap()
        .unwrap();
        assert_eq!(prefix.len(), 4);
        assert_eq!(suffix, original_suffix);
        let NirExpr::Call {
            args: original_args,
            ..
        } = &value
        else {
            unreachable!()
        };
        let rewritten = match prefix.last().unwrap() {
            NirStmt::Let {
                name,
                ty: Some(result),
                value,
            } if !constant => {
                assert_eq!(name, "choice");
                assert_eq!(result, &ty("i64"));
                value
            }
            NirStmt::Const {
                name,
                ty: result,
                value,
            } if constant => {
                assert_eq!(name, "choice");
                assert_eq!(result, &ty("i64"));
                value
            }
            _ => panic!("declaration kind changed"),
        };
        let NirExpr::Call { args, .. } = rewritten else {
            unreachable!()
        };
        for (index, item) in prefix[..3].iter().enumerate() {
            let NirStmt::Let {
                name,
                ty: Some(result),
                value,
            } = item
            else {
                unreachable!()
            };
            assert_ne!(name, "choice");
            assert_ne!(name, "__nuis_resource_predicate_left");
            assert_eq!(result.name, if index == 1 { "bool" } else { "i64" });
            assert_eq!(
                value, &original_args[index],
                "lazy subtree must remain intact"
            );
            assert_eq!(args[index], NirExpr::Var(name.clone()));
        }
        assert!(context.pending.is_empty() && context.outlines.owners.is_empty());
    }
}

#[test]
fn resource_predicate_i64_admission_rejects_invalid_types_before_allocating_names() {
    let (template, _, scope, live, value) = fixture();
    let mut dead = value.clone();
    let NirExpr::Call { args, .. } = &mut dead else {
        unreachable!()
    };
    args[1] = binary(
        NirBinaryOp::And,
        NirExpr::Bool(false),
        binary(
            NirBinaryOp::Gt,
            NirExpr::BytesLen(Box::new(NirExpr::Var("dead".into()))),
            NirExpr::Int(0),
        ),
    );
    let mut bad_argument = value.clone();
    let NirExpr::Call { args, .. } = &mut bad_argument else {
        unreachable!()
    };
    args[2] = NirExpr::Bool(false);
    let mut bad_helper = value.clone();
    let NirExpr::Call { callee, .. } = &mut bad_helper else {
        unreachable!()
    };
    *callee = "unknown_effectful_call".into();
    let NirExpr::Call { args, .. } = &value else {
        unreachable!()
    };
    for (result, value) in [
        (ty("bool"), value.clone()),
        (ty("i32"), value.clone()),
        (
            NirTypeRef {
                is_ref: true,
                ..ty("i64")
            },
            value.clone(),
        ),
        (
            NirTypeRef {
                is_optional: true,
                ..ty("i64")
            },
            value.clone(),
        ),
        (
            NirTypeRef {
                generic_args: vec![ty("i64")],
                ..ty("i64")
            },
            value.clone(),
        ),
        (ty("i64"), args[1].clone()),
        (ty("i64"), dead),
        (ty("i64"), bad_argument),
        (ty("i64"), bad_helper),
    ] {
        let (_, mut context, _, _, _) = fixture();
        let statement = NirStmt::Let {
            name: "choice".into(),
            ty: Some(result),
            value,
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
            &mut context
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
fn resource_predicate_i64_keeps_plain_scalar_initializers_on_the_existing_route() {
    let (template, mut context, scope, live, _) = fixture();
    let statement = NirStmt::Let {
        name: "choice".into(),
        ty: Some(ty("i64")),
        value: NirExpr::Call {
            callee: "combine".into(),
            args: vec![NirExpr::Int(1), NirExpr::Bool(true), NirExpr::Int(2)],
        },
    };
    let mut suffix = VecDeque::from(template.body.clone());
    assert!(split(
        &statement,
        &template,
        &mut suffix,
        &scope,
        &live,
        1,
        &mut context
    )
    .unwrap()
    .is_none());
    assert!(context.reserved.is_empty());
}

#[test]
fn resource_predicate_buffer_length_stages_in_order_without_walking_lazy_children() {
    let (template, mut context, scope, live, mut value) = fixture();
    let length = NirExpr::BufferLen(Box::new(NirExpr::Var("buffer".into())));
    let NirExpr::Call { args, .. } = &mut value else {
        unreachable!()
    };
    args[0] = length.clone();
    args[1] = binary(
        NirBinaryOp::And,
        NirExpr::Var("flag".into()),
        binary(NirBinaryOp::Gt, length, NirExpr::Int(0)),
    );
    let original_args = args.clone();
    assert!(needs_guard(&args[1]));
    let mut suffix = VecDeque::from(template.body.clone());
    let original_suffix = suffix.clone();
    let prefix = split(
        &NirStmt::Let {
            name: "choice".into(),
            ty: Some(ty("i64")),
            value,
        },
        &template,
        &mut suffix,
        &scope,
        &live,
        1,
        &mut context,
    )
    .unwrap()
    .unwrap();
    assert_eq!(prefix.len(), 4);
    for (index, statement) in prefix[..3].iter().enumerate() {
        let NirStmt::Let {
            ty: Some(ty),
            value,
            ..
        } = statement
        else {
            unreachable!()
        };
        assert_eq!(value, &original_args[index]);
        assert_eq!(ty.name, if index == 1 { "bool" } else { "i64" });
    }
    assert_eq!(suffix, original_suffix);
    assert!(context.pending.is_empty() && context.outlines.owners.is_empty());
}

#[test]
fn resource_predicate_buffer_length_rejects_nonborrowed_or_hidden_invalid_operands_atomically() {
    let (template, _, scope, live, _) = fixture();
    let buffer_ty = scope[0].ty.clone();
    let mut variants = vec![
        NirTypeRef {
            is_ref: false,
            ..buffer_ty.clone()
        },
        NirTypeRef {
            is_optional: true,
            ..buffer_ty.clone()
        },
        NirTypeRef {
            generic_args: vec![ty("i64")],
            ..buffer_ty.clone()
        },
        ty("Bytes"),
        ty("i64"),
        ty("bool"),
    ]
    .into_iter()
    .map(|ty| {
        let mut changed = scope.clone();
        changed[0].ty = ty;
        (changed, NirExpr::Var("buffer".into()))
    })
    .collect::<Vec<_>>();
    for operand in [
        NirExpr::Var("missing".into()),
        NirExpr::Var("saved".into()),
        NirExpr::Bool(false),
        NirExpr::Int(0),
        NirExpr::Call {
            callee: "unknown".into(),
            args: vec![],
        },
        binary(
            NirBinaryOp::And,
            NirExpr::Bool(false),
            NirExpr::Var("missing".into()),
        ),
    ] {
        variants.push((scope.clone(), operand));
    }
    for (scope, operand) in variants {
        let (_, mut context, _, _, _) = fixture();
        let value = binary(
            NirBinaryOp::And,
            NirExpr::Bool(false),
            binary(
                NirBinaryOp::Gt,
                NirExpr::BufferLen(Box::new(operand)),
                NirExpr::Int(0),
            ),
        );
        let mut suffix = VecDeque::from(template.body.clone());
        let original = suffix.clone();
        assert!(split(
            &NirStmt::Let {
                name: "choice".into(),
                ty: Some(ty("bool")),
                value,
            },
            &template,
            &mut suffix,
            &scope,
            &live,
            1,
            &mut context
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
fn resource_predicate_buffer_length_keeps_original_budgets_and_ready_route() {
    let (template, mut context, scope, live, _) = fixture();
    let length = NirExpr::BufferLen(Box::new(NirExpr::Var("buffer".into())));
    assert!(admitted(&length, &scope, &live, &context.calls, Kind::I64));
    assert!(!admitted(
        &length,
        &scope,
        &live,
        &context.calls,
        Kind::Bool
    ));
    let mut deep = length.clone();
    for _ in 0..64 {
        deep = NirExpr::BufferLen(Box::new(deep));
    }
    let wide = NirExpr::Call {
        callee: "combine".into(),
        args: vec![length.clone(); 2048],
    };
    let mut edges = binary(NirBinaryOp::Gt, length.clone(), NirExpr::Int(0));
    for _ in 0..33 {
        edges = binary(NirBinaryOp::And, NirExpr::Bool(true), edges);
    }
    for over_budget in [deep, wide, edges] {
        assert!(bounded(&over_budget).is_none());
    }
    let mut suffix = VecDeque::from(template.body.clone());
    assert!(split(
        &NirStmt::Let {
            name: "choice".into(),
            ty: Some(ty("i64")),
            value: length,
        },
        &template,
        &mut suffix,
        &scope,
        &live,
        1,
        &mut context
    )
    .unwrap()
    .is_none());
    assert!(context.reserved.is_empty());
}
