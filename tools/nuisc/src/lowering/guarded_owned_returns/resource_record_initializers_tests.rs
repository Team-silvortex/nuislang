use super::*;

fn ty(name: &str) -> NirTypeRef {
    NirTypeRef {
        name: name.into(),
        is_ref: false,
        is_optional: false,
        generic_args: vec![],
    }
}

fn binary(op: NirBinaryOp, lhs: NirExpr, rhs: NirExpr) -> NirExpr {
    NirExpr::Binary {
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    }
}

pub(super) fn fixture() -> (
    NirFunction,
    OutlineContext,
    Vec<NirParam>,
    BTreeSet<String>,
    NirExpr,
) {
    let module = crate::pipeline::compile_source(
        r#"mod cpu Main {
        struct Packet { left: i64, flag: bool, right: i64 }
        struct Envelope { packet: Packet }
        struct Bundle { before: i64, packet: Packet, after: bool }
        struct Pair { gate: bool, first: Packet, second: Packet }
        struct Double { gate: bool, first: Envelope, second: Packet }
        struct Other { left: i64, flag: bool, right: i64 }
        fn packet(left: i64, flag: bool, right: i64) -> Packet {
            return Packet { left: left, flag: flag, right: right };
        }
        fn envelope(left: i64, flag: bool, right: i64) -> Envelope {
            return Envelope { packet: packet(left, flag, right) };
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
            name: "denominator".into(),
            ty: ty("i64"),
        },
    ];
    let value = NirExpr::Call {
        callee: "packet".into(),
        args: vec![
            NirExpr::LoadAt {
                buffer: Box::new(NirExpr::Var("buffer".into())),
                index: Box::new(NirExpr::Int(0)),
            },
            binary(
                NirBinaryOp::And,
                NirExpr::Var("flag".into()),
                binary(
                    NirBinaryOp::Gt,
                    NirExpr::BytesLen(Box::new(NirExpr::Var("saved".into()))),
                    NirExpr::Int(0),
                ),
            ),
            binary(
                NirBinaryOp::Div,
                NirExpr::Int(100),
                NirExpr::Var("denominator".into()),
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
fn resource_predicate_record_initializers_preserve_nominal_bindings_and_scalar_argument_order() {
    for constant in [false, true] {
        for projected in [false, true] {
            let (template, mut context, scope, live, mut value) = fixture();
            let NirExpr::Call { callee, args } = &mut value else {
                unreachable!()
            };
            let originals = args.clone();
            if projected {
                *callee = "envelope".into();
                value = NirExpr::FieldAccess {
                    base: Box::new(value),
                    field: "packet".into(),
                };
            }
            let declaration = if constant {
                NirStmt::Const {
                    name: "local".into(),
                    ty: ty("Packet"),
                    value,
                }
            } else {
                NirStmt::Let {
                    name: "local".into(),
                    ty: Some(ty("Packet")),
                    value,
                }
            };
            context
                .reserved
                .insert("__nuis_resource_predicate_left".into());
            let mut suffix = VecDeque::from(template.body.clone());
            let original_suffix = suffix.clone();
            let result = split(
                &declaration,
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
            assert_eq!(result.len(), 4);
            let value = match result.last().unwrap() {
                NirStmt::Let {
                    name,
                    ty: Some(result),
                    value,
                } if !constant => {
                    assert_eq!(name, "local");
                    assert_eq!(result, &ty("Packet"));
                    value
                }
                NirStmt::Const {
                    name,
                    ty: result,
                    value,
                } if constant => {
                    assert_eq!(name, "local");
                    assert_eq!(result, &ty("Packet"));
                    value
                }
                _ => panic!("record declaration identity changed"),
            };
            let value = if projected {
                let NirExpr::FieldAccess { base, field } = value else {
                    unreachable!()
                };
                assert_eq!(field, "packet");
                base.as_ref()
            } else {
                value
            };
            let NirExpr::Call { callee, args } = value else {
                unreachable!()
            };
            assert_eq!(callee, if projected { "envelope" } else { "packet" });
            for (index, statement) in result[..3].iter().enumerate() {
                let NirStmt::Let {
                    name,
                    ty: Some(kind),
                    value,
                } = statement
                else {
                    unreachable!()
                };
                assert_ne!(name, "__nuis_resource_predicate_left");
                assert_eq!(kind, &ty(if index == 1 { "bool" } else { "i64" }));
                assert_eq!(value, &originals[index], "lazy subtree must remain intact");
                assert_eq!(args[index], NirExpr::Var(name.clone()));
            }
            assert_eq!(suffix, original_suffix);
            assert!(context.pending.is_empty() && context.outlines.owners.is_empty());
            assert!(split(
                result.last().unwrap(),
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
        }
    }
}

fn rejected(ty: NirTypeRef, value: NirExpr) {
    let (template, mut context, scope, live, _) = fixture();
    let mut suffix = VecDeque::from(template.body.clone());
    let original = suffix.clone();
    let statement = NirStmt::Let {
        name: "local".into(),
        ty: Some(ty),
        value,
    };
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

#[test]
fn resource_predicate_record_initializers_reject_invalid_types_owners_and_calls_before_names() {
    let (_, _, _, _, value) = fixture();
    for result in [
        ty("Other"),
        ty("Missing"),
        ty("Bytes"),
        ty("bool"),
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
        rejected(result, value.clone());
    }
    for variant in 0..5 {
        let mut value = value.clone();
        let NirExpr::Call { callee, args } = &mut value else {
            unreachable!()
        };
        match variant {
            0 => *callee = "unknown_or_effectful".into(),
            1 => {
                args.pop();
            }
            2 => args[0] = NirExpr::Bool(false),
            3 => args[2] = NirExpr::Var("buffer".into()),
            4 => {
                args[1] = binary(
                    NirBinaryOp::And,
                    NirExpr::Bool(false),
                    binary(
                        NirBinaryOp::Gt,
                        NirExpr::BytesLen(Box::new(NirExpr::Var("dead".into()))),
                        NirExpr::Int(0),
                    ),
                )
            }
            _ => unreachable!(),
        }
        rejected(ty("Packet"), value);
    }
}

#[test]
fn resource_predicate_record_initializers_keep_whole_expression_budgets_and_ready_routes() {
    let (_, _, _, _, value) = fixture();
    for variant in 0..4 {
        let mut value = value.clone();
        let NirExpr::Call { args, .. } = &mut value else {
            unreachable!()
        };
        match variant {
            0 => {
                for _ in 0..64 {
                    args[0] = binary(NirBinaryOp::Add, args[0].clone(), NirExpr::Int(1));
                }
            }
            1 => args.extend(vec![NirExpr::Int(0); 4096]),
            2 => {
                for _ in 0..33 {
                    args[1] = binary(NirBinaryOp::And, NirExpr::Bool(true), args[1].clone());
                }
            }
            3 => *args = vec![NirExpr::Int(1), NirExpr::Bool(true), NirExpr::Int(2)],
            _ => unreachable!(),
        }
        rejected(ty("Packet"), value);
    }
}

#[test]
fn resource_predicate_record_catalog_never_admits_hidden_resources_or_recursive_layouts() {
    let module = crate::pipeline::compile_source(
        r#"mod cpu Main {
        struct Packet { value: i64 }
        struct Envelope { packet: Packet, unused: Packet }
        fn main() -> i64 { return 0; }
    }"#,
    )
    .unwrap()
    .nir;
    for kind in ["Bytes", "Buffer", "Envelope", "Unknown"] {
        let mut module = module.clone();
        module
            .structs
            .iter_mut()
            .find(|s| s.name == "Envelope")
            .unwrap()
            .fields[1]
            .ty = ty(kind);
        let calls = PureValueCalls::collect(&module);
        assert!(!calls.is_record(&ty("Envelope")));
        assert!(calls.is_record(&ty("Packet")));
    }
}
