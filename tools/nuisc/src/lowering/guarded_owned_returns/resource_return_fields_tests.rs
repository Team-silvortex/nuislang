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

fn lazy() -> NirExpr {
    NirExpr::Call {
        callee: "word".into(),
        args: vec![binary(
            NirBinaryOp::And,
            NirExpr::Bool(false),
            binary(
                NirBinaryOp::Gt,
                NirExpr::BytesLen(Box::new(NirExpr::Var("saved".into()))),
                NirExpr::Int(0),
            ),
        )],
    }
}

fn record(fields: Vec<(&str, NirExpr)>) -> NirExpr {
    NirExpr::StructLiteral {
        type_name: "State".into(),
        type_args: vec![],
        fields: fields
            .into_iter()
            .map(|(name, value)| (name.into(), value))
            .collect(),
    }
}

fn moved() -> NirExpr {
    NirExpr::Move(Box::new(NirExpr::Var("saved".into())))
}

fn fixture() -> (NirFunction, OutlineContext, Vec<NirParam>, BTreeSet<String>) {
    let module = crate::pipeline::compile_source(
        "mod cpu Main { fn word(flag: bool) -> i64 { if flag { return 1; } return 0; } fn main() -> i64 { return 0; } }",
    ).unwrap().nir;
    let mut template = module
        .functions
        .iter()
        .find(|f| f.name == "main")
        .unwrap()
        .clone();
    template.return_type = Some(ty("State"));
    template.body.insert(
        0,
        NirStmt::Let {
            name: "__nuis_resource_predicate_left".into(),
            ty: Some(ty("i64")),
            value: NirExpr::Int(0),
        },
    );
    let mut buffer = ty("Buffer");
    buffer.is_ref = true;
    let scope = vec![NirParam {
        name: "pixels".into(),
        ty: buffer,
    }];
    let context = OutlineContext {
        reserved: BTreeSet::new(),
        pending: VecDeque::new(),
        roots: 1,
        outlines: Outlines::default(),
        calls: PureValueCalls::collect(&module),
        return_layouts: BTreeMap::from([(
            "State".into(),
            MixedResourceLayout::parse("State{a:i64;b:i64;data:Bytes}").unwrap(),
        )]),
    };
    (template, context, scope, BTreeSet::from(["saved".into()]))
}

#[test]
fn resource_predicate_return_fields_stage_all_fields_in_source_order() {
    let copy = NirExpr::CopyBufferOwned(Box::new(NirExpr::Var("pixels".into())));
    for value in [
        record(vec![
            ("a", lazy()),
            ("data", moved()),
            ("b", NirExpr::Int(7)),
        ]),
        record(vec![("data", copy), ("b", lazy()), ("a", NirExpr::Int(9))]),
    ] {
        let (template, mut context, scope, live) = fixture();
        let rewritten = split(&value, &template, &scope, &live, &mut context).unwrap();
        let NirExpr::StructLiteral { fields, .. } = &value else {
            unreachable!()
        };
        assert_eq!(rewritten.len(), 4);
        let mut names = BTreeSet::new();
        let mut ready = vec![];
        for ((field, original), stmt) in fields.iter().zip(&rewritten) {
            let NirStmt::Let {
                name,
                ty: Some(kind),
                value,
            } = stmt
            else {
                panic!("missing snapshot")
            };
            assert_eq!(value, original);
            assert_ne!(name, "__nuis_resource_predicate_left");
            assert!(names.insert(name));
            assert_eq!(kind.name, if field == "data" { "Bytes" } else { "i64" });
            let value = NirExpr::Var(name.clone());
            ready.push((
                field.as_str(),
                if field == "data" {
                    NirExpr::Move(Box::new(value))
                } else {
                    value
                },
            ));
        }
        assert_eq!(
            rewritten.last(),
            Some(&NirStmt::Return(Some(record(ready))))
        );
        assert!(context.pending.is_empty() && context.outlines.owners.is_empty());
    }
}

#[test]
fn resource_predicate_return_fields_reject_invalid_whole_records_before_names() {
    let mut bad_type = record(vec![
        ("a", lazy()),
        ("b", NirExpr::Int(1)),
        ("data", moved()),
    ]);
    if let NirExpr::StructLiteral { type_name, .. } = &mut bad_type {
        *type_name = "Other".into();
    }
    for value in [
        bad_type,
        record(vec![
            ("data", moved()),
            ("a", lazy()),
            ("b", NirExpr::Int(1)),
        ]),
        record(vec![
            ("a", lazy()),
            ("a", NirExpr::Int(1)),
            ("data", moved()),
        ]),
        record(vec![
            ("a", lazy()),
            ("b", NirExpr::Bool(false)),
            ("data", moved()),
        ]),
        record(vec![("a", lazy()), ("b", NirExpr::Int(1))]),
        record(vec![
            ("a", lazy()),
            (
                "b",
                NirExpr::Call {
                    callee: "write".into(),
                    args: vec![],
                },
            ),
            ("data", moved()),
        ]),
        record(vec![
            ("a", lazy()),
            ("b", NirExpr::Int(1)),
            ("data", NirExpr::Var("saved".into())),
        ]),
    ] {
        let (template, mut context, scope, live) = fixture();
        assert!(split(&value, &template, &scope, &live, &mut context).is_none());
        assert!(
            context.reserved.is_empty()
                && context.pending.is_empty()
                && context.outlines.owners.is_empty()
        );
    }
}

#[test]
fn resource_predicate_return_fields_share_original_aggregate_budgets() {
    let mut logical = NirExpr::Bool(false);
    for _ in 0..17 {
        logical = binary(NirBinaryOp::And, logical, NirExpr::Bool(true));
    }
    let call = NirExpr::Call {
        callee: "word".into(),
        args: vec![logical],
    };
    let mut wide = NirExpr::Int(1);
    for _ in 0..11 {
        wide = binary(NirBinaryOp::Add, wide.clone(), wide);
    }
    let mut deep = lazy();
    for _ in 0..64 {
        deep = binary(NirBinaryOp::Add, deep, NirExpr::Int(0));
    }
    for (a, b) in [
        (call.clone(), call),
        (wide, lazy()),
        (deep, NirExpr::Int(0)),
    ] {
        let (template, mut context, scope, live) = fixture();
        assert!(split(
            &record(vec![("a", a), ("b", b), ("data", moved())]),
            &template,
            &scope,
            &live,
            &mut context
        )
        .is_none());
        assert!(context.reserved.is_empty());
    }
}

#[test]
fn resource_predicate_return_fields_leave_ready_records_and_unknown_layouts_unchanged() {
    let (template, mut context, scope, live) = fixture();
    let ready = record(vec![
        ("a", NirExpr::Int(1)),
        ("b", NirExpr::Int(2)),
        ("data", moved()),
    ]);
    assert!(split(&ready, &template, &scope, &live, &mut context).is_none());
    context.return_layouts.clear();
    assert!(split(
        &record(vec![
            ("a", lazy()),
            ("b", NirExpr::Int(2)),
            ("data", moved())
        ]),
        &template,
        &scope,
        &live,
        &mut context
    )
    .is_none());
    assert!(context.reserved.is_empty());
}

#[test]
fn resource_predicate_return_fields_validate_checked_wrapper_identity_and_type_arguments() {
    let (mut template, mut context, scope, live) = fixture();
    let mut outcome = ty("Outcome");
    outcome.generic_args = vec![ty("State"), ty("i64")];
    let layout = MixedResourceLayout::parse_outcome("__nuis_variant_union__Outcome{tag:i64;Outcome.Ok:Outcome.Ok{value:State{a:i64;b:i64;data:Bytes}};Outcome.Err:Outcome.Err{value:i64}}", "State", "Outcome").unwrap();
    context.return_layouts.insert(outcome.render(), layout);
    template.return_type = Some(outcome.clone());
    let payload = record(vec![
        ("a", lazy()),
        ("b", NirExpr::Int(2)),
        ("data", moved()),
    ]);
    for (name, type_args, field) in [
        ("Result.Ok", outcome.generic_args.clone(), "value"),
        ("Outcome.Err", outcome.generic_args.clone(), "value"),
        ("Outcome.Ok", vec![], "value"),
        ("Outcome.Ok", vec![ty("State"), ty("bool")], "value"),
        ("Outcome.Ok", outcome.generic_args.clone(), "other"),
    ] {
        let value = NirExpr::StructLiteral {
            type_name: name.into(),
            type_args,
            fields: vec![(field.into(), payload.clone())],
        };
        assert!(split(&value, &template, &scope, &live, &mut context).is_none());
        assert!(context.reserved.is_empty());
    }
    let value = NirExpr::StructLiteral {
        type_name: "Outcome.Ok".into(),
        type_args: outcome.generic_args,
        fields: vec![("value".into(), payload)],
    };
    let prefix = split(&value, &template, &scope, &live, &mut context).unwrap();
    let Some(NirStmt::Return(Some(NirExpr::StructLiteral {
        type_name,
        type_args,
        ..
    }))) = prefix.last()
    else {
        panic!("checked wrapper must survive")
    };
    assert_eq!(type_name, "Outcome.Ok");
    assert_eq!(type_args, &template.return_type.unwrap().generic_args);
}

#[test]
fn resource_predicate_return_fields_preserve_terminal_cleanup_bindings_only() {
    let (template, mut context, scope, live) = fixture();
    let value = record(vec![
        ("a", lazy()),
        ("b", NirExpr::Int(0)),
        ("data", moved()),
    ]);
    let binding = NirStmt::Let {
        name: "result".into(),
        ty: Some(ty("State")),
        value,
    };
    let mut live = live;
    live.insert("spare".into());
    let suffix = VecDeque::from([
        NirStmt::Expr(NirExpr::DropBytes(Box::new(NirExpr::Var("spare".into())))),
        NirStmt::Return(Some(NirExpr::Var("result".into()))),
    ]);
    let mut nonterminal = suffix.clone();
    nonterminal.push_front(NirStmt::Expr(NirExpr::Int(1)));
    assert!(split_cleanup_binding(
        &binding,
        &template,
        &nonterminal,
        &scope,
        &live,
        &mut context
    )
    .is_none());
    assert!(context.reserved.is_empty());
    let prefix =
        split_cleanup_binding(&binding, &template, &suffix, &scope, &live, &mut context).unwrap();
    assert_eq!(prefix.len(), 4);
    assert!(
        matches!(prefix.last(), Some(NirStmt::Let { name, ty: Some(kind), .. }) if name == "result" && kind.name == "State")
    );
    assert!(!prefix
        .iter()
        .any(|s| matches!(s, NirStmt::Expr(NirExpr::DropBytes(_)))));
    assert_eq!(suffix.len(), 2);
}

fn error_fixture() -> (NirFunction, OutlineContext, Vec<NirParam>, BTreeSet<String>) {
    let (mut template, mut context, scope, live) = fixture();
    let mut outcome = ty("Outcome");
    outcome.generic_args = vec![ty("State"), ty("i64")];
    let layout = MixedResourceLayout::parse_outcome("__nuis_variant_union__Outcome{tag:i64;Outcome.Ok:Outcome.Ok{value:State{a:i64;b:i64;data:Bytes}};Outcome.Err:Outcome.Err{value:i64}}", "State", "Outcome").unwrap();
    context.return_layouts.insert(outcome.render(), layout);
    template.return_type = Some(outcome);
    (template, context, scope, live)
}

fn error_value(value: NirExpr) -> NirExpr {
    NirExpr::StructLiteral {
        type_name: "Outcome.Err".into(),
        type_args: vec![ty("State"), ty("i64")],
        fields: vec![("value".into(), value)],
    }
}

#[test]
fn resource_predicate_checked_error_arms_use_registered_identity_without_live_owners() {
    for variant in 0..7 {
        let (mut template, mut context, _, _) = error_fixture();
        let mut value = error_value(NirExpr::Int(17));
        let NirExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } = &mut value
        else {
            unreachable!()
        };
        match variant {
            0 => {}
            1 => *type_name = "Result.Err".into(),
            2 => type_args.clear(),
            3 => fields[0].0 = "other".into(),
            4 => fields.push(fields[0].clone()),
            5 => template.return_type = None,
            6 => context.return_layouts.clear(),
            _ => unreachable!(),
        }
        let body = vec![NirStmt::Return(Some(value))];
        assert_eq!(
            super::super::checked_error_arm(&body, &template, &context),
            variant == 0
        );
        assert!(context.reserved.is_empty() && context.pending.is_empty());
    }
}

#[test]
fn resource_predicate_error_code_requeues_one_intact_i64_and_preserves_identity() {
    let (template, mut context, scope, live) = error_fixture();
    let code = binary(NirBinaryOp::Add, NirExpr::Int(17), lazy());
    let value = error_value(code.clone());
    let prefix = split(&value, &template, &scope, &live, &mut context).unwrap();
    let [NirStmt::Let {
        name,
        ty: Some(kind),
        value,
    }, NirStmt::Return(Some(result))] = prefix.as_slice()
    else {
        panic!("one scalar snapshot and one Err")
    };
    assert_eq!(kind, &ty("i64"));
    assert_ne!(name, "__nuis_resource_predicate_left");
    assert_eq!(value, &code);
    assert_eq!(result, &error_value(NirExpr::Var(name.clone())));
    let mut scope = scope;
    scope.push(NirParam {
        name: name.clone(),
        ty: kind.clone(),
    });
    assert!(split(result, &template, &scope, &live, &mut context).is_none());
    assert!(context.pending.is_empty() && context.outlines.owners.is_empty());
}

#[test]
fn resource_predicate_error_code_rejection_is_atomic_for_kinds_identity_owners_and_budgets() {
    let mut deep = lazy();
    for _ in 0..64 {
        deep = binary(NirBinaryOp::Add, deep, NirExpr::Int(0));
    }
    let mut wide = NirExpr::Int(1);
    for _ in 0..11 {
        wide = binary(NirBinaryOp::Add, wide.clone(), wide);
    }
    let mut logical = NirExpr::Bool(false);
    for _ in 0..33 {
        logical = binary(NirBinaryOp::And, logical, NirExpr::Bool(true));
    }
    let mut invalid = vec![
        error_value(NirExpr::Bool(false)),
        error_value(moved()),
        error_value(record(vec![])),
        error_value(NirExpr::Call {
            callee: "unknown_effect".into(),
            args: vec![lazy()],
        }),
        error_value(deep),
        error_value(binary(NirBinaryOp::Add, wide, lazy())),
        error_value(NirExpr::Call {
            callee: "word".into(),
            args: vec![logical],
        }),
    ];
    for case in 0..5 {
        let mut value = error_value(lazy());
        let NirExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } = &mut value
        else {
            unreachable!()
        };
        match case {
            0 => *type_name = "Other.Err".into(),
            1 => type_args[1] = ty("bool"),
            2 => fields[0].0 = "code".into(),
            3 => fields.push(fields[0].clone()),
            _ => type_args.clear(),
        }
        invalid.push(value);
    }
    for value in invalid {
        let (template, mut context, scope, live) = error_fixture();
        assert!(split(&value, &template, &scope, &live, &mut context).is_none());
        assert!(
            context.reserved.is_empty()
                && context.pending.is_empty()
                && context.outlines.owners.is_empty()
        );
    }
    let (template, mut context, scope, _) = error_fixture();
    assert!(split(
        &error_value(lazy()),
        &template,
        &scope,
        &BTreeSet::new(),
        &mut context
    )
    .is_none());
    assert!(
        context.reserved.is_empty(),
        "even an unselected read needs a live owner"
    );
}

#[test]
fn resource_predicate_error_codes_preserve_terminal_cleanup_and_ready_routes() {
    let (template, mut context, scope, live) = error_fixture();
    for code in [
        NirExpr::Int(0),
        NirExpr::BytesLen(Box::new(NirExpr::Var("saved".into()))),
    ] {
        assert!(split(&error_value(code), &template, &scope, &live, &mut context).is_none());
    }
    assert!(context.reserved.is_empty());
    let binding = NirStmt::Let {
        name: "result".into(),
        ty: template.return_type.clone(),
        value: error_value(lazy()),
    };
    let suffix = VecDeque::from([
        NirStmt::Expr(NirExpr::DropBytes(Box::new(NirExpr::Var("saved".into())))),
        NirStmt::Return(Some(NirExpr::Var("result".into()))),
    ]);
    let original = suffix.clone();
    let prefix =
        split_cleanup_binding(&binding, &template, &suffix, &scope, &live, &mut context).unwrap();
    assert_eq!(prefix.len(), 2);
    assert!(
        matches!(prefix.last(), Some(NirStmt::Let { name, ty, .. }) if name == "result" && ty == &template.return_type)
    );
    assert_eq!(suffix, original);
    assert!(!prefix
        .iter()
        .any(|s| matches!(s, NirStmt::Expr(NirExpr::DropBytes(_)))));
}
