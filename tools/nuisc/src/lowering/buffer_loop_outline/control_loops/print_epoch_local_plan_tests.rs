use super::*;

fn module(reversed: bool) -> NirModule {
    let mut module = crate::frontend::parse_nuis_module(
        "mod cpu Main {
        struct Record { code:i32, total:i64, other:i32 }
        struct Outer { inner:Record }
        fn keep64(value:i64)->i64{return value;}
        fn checked32(value:i32,guard:i64)->i32{let checked=0/guard;return value;}
        fn checked64(value:i64,guard:i64)->i64{let checked=0/guard;return value;}
        fn bool64(value:i64)->bool{return value>0;}
        fn float64(value:i64)->f64{return 0.5;}
        fn narrow64(value:i64)->i32{return i32_from_i64(value);}
        fn effect(value:i32)->i32{print(1);return value;}
        fn transitive(value:i32)->i32{return effect(value);}
        fn cycle_a(value:i32)->i32{return cycle_b(value);}
        fn cycle_b(value:i32)->i32{return cycle_a(value);}
        fn choose32(value:f32,pick:bool)->f32{if pick{return value;}return -value;}
        fn choose64(value:f64,pick:bool)->f64{if pick{return value;}return -value;}
        fn observe32(value:f32)->i64{return 1;}
        fn observe64(value:f64)->i64{return 1;}
        fn main()->i64{return 0;}
    }",
    )
    .unwrap();
    if reversed {
        module.functions.reverse();
    }
    module
}

fn scope(ty: &str) -> Scope {
    let mut scope = Scope::from([
        ("live".into(), scalar_type("Record")),
        ("ready".into(), scalar_type(ty)),
        ("gate".into(), scalar_type("bool")),
        ("guard".into(), scalar_type("i64")),
    ]);
    for i in 0..28 {
        scope.insert(format!("v{i}"), scalar_type("i64"));
    }
    scope
}
fn var(name: &str) -> NirExpr {
    NirExpr::Var(name.into())
}
fn field(name: &str) -> NirExpr {
    NirExpr::FieldAccess {
        base: Box::new(var("live")),
        field: name.into(),
    }
}
fn wide(value: NirExpr) -> NirExpr {
    NirExpr::CastI32ToI64(Box::new(value))
}
fn binary(op: NirBinaryOp, lhs: NirExpr, rhs: NirExpr) -> NirExpr {
    NirExpr::Binary {
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    }
}
fn call(callee: &str, args: Vec<NirExpr>) -> NirExpr {
    NirExpr::Call {
        callee: callee.into(),
        args,
    }
}
fn initializer() -> NirExpr {
    binary(NirBinaryOp::Xor, wide(field("code")), NirExpr::Int(1))
}
fn consumer(ty: &str, value: NirExpr) -> NirStmt {
    let suffix = if ty == "f32" { "32" } else { "64" };
    NirStmt::Print(call(
        &format!("observe{suffix}"),
        vec![call(
            &format!("choose{suffix}"),
            vec![
                var("ready"),
                binary(
                    NirBinaryOp::And,
                    var("gate"),
                    binary(
                        NirBinaryOp::Ne,
                        call("keep64", vec![value]),
                        NirExpr::Int(0),
                    ),
                ),
            ],
        )],
    ))
}
fn body(ty: &str, value: NirExpr) -> Vec<NirStmt> {
    vec![
        NirStmt::Let {
            name: "word".into(),
            ty: Some(scalar_type("i64")),
            value,
        },
        consumer(ty, var("word")),
    ]
}
fn ready(value: &NirExpr, scope: &Scope) -> Option<NirTypeRef> {
    match value {
        NirExpr::Int(_) => Some(scalar_type("i64")),
        NirExpr::Bool(_) => Some(scalar_type("bool")),
        NirExpr::Var(name) => scope.get(name).cloned(),
        _ => None,
    }
}
fn changed() -> BTreeSet<String> {
    BTreeSet::from(["live".into()])
}

#[test]
fn epoch_local_plan_binds_original_fields_source_positions_and_same_epoch() {
    for reversed in [false, true] {
        let module = module(reversed);
        let layouts = control_values::layouts(&module);
        let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
        for ty in ["f32", "f64"] {
            let scope = scope(ty);
            let epoch = Epoch::enter(&scope, &changed(), &changed(), &layouts);
            let mut body = body(ty, initializer());
            body.insert(0, NirStmt::Print(NirExpr::Int(71)));
            body.push(NirStmt::Let {
                name: "live".into(),
                ty: None,
                value: var("live"),
            });
            let result = plan(&body, 1, &epoch, &scope, &layouts, &catalog, |v, _| {
                ready(v, &scope)
            })
            .unwrap();
            assert!(std::ptr::eq(result.epoch, &epoch));
            assert!(std::ptr::eq(result.body, body.as_slice()));
            assert!(std::ptr::eq(result.scope, &scope));
            assert!(std::ptr::eq(result.layouts, &layouts));
            assert!(std::ptr::eq(result.catalog, &catalog));
            assert_eq!(
                (result.binding, result.definition, result.consumer),
                ("word", 1, 2)
            );
            assert_eq!(
                result.origins,
                BTreeMap::from([(vec!["live".into(), "code".into()], scalar_type("i32"))])
            );
            assert!(!result.initializer_may_fail);
            assert!(result.consumer_may_fail);
            assert_eq!(result.work, 16);
            assert_eq!(result.captures.len(), 4);
            let mut nested_scope = scope.clone();
            nested_scope.insert("live".into(), scalar_type("Outer"));
            let nested_epoch = Epoch::enter(&nested_scope, &changed(), &changed(), &layouts);
            let nested = wide(NirExpr::FieldAccess {
                base: Box::new(field("inner")),
                field: "code".into(),
            });
            let mut nested_body = body.clone();
            let NirStmt::Let { value, .. } = &mut nested_body[1] else {
                unreachable!()
            };
            *value = binary(NirBinaryOp::Xor, nested, NirExpr::Int(1));
            let nested_plan = plan(
                &nested_body,
                1,
                &nested_epoch,
                &nested_scope,
                &layouts,
                &catalog,
                |v, _| ready(v, &nested_scope),
            )
            .unwrap();
            assert_eq!(
                nested_plan.origins,
                BTreeMap::from([(
                    vec!["live".into(), "inner".into(), "code".into()],
                    scalar_type("i32")
                )])
            );
            assert_eq!(nested_plan.work, 17);
        }
    }
}

#[test]
fn epoch_local_plan_keeps_fallible_initialization_eager_and_binding_reads_once_only() {
    let module = module(false);
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    let scope = scope("f64");
    let epoch = Epoch::enter(&scope, &changed(), &changed(), &layouts);
    let body = body(
        "f64",
        binary(
            NirBinaryOp::Xor,
            wide(call("checked32", vec![field("code"), var("guard")])),
            NirExpr::Int(1),
        ),
    );
    let result = plan(&body, 0, &epoch, &scope, &layouts, &catalog, |v, _| {
        ready(v, &scope)
    })
    .unwrap();
    assert!(result.initializer_may_fail);
    assert_eq!(result.work, 18);
    assert!(std::ptr::eq(&result.body[0], &body[0]));
    let mut duplicate = body.clone();
    duplicate[1] = consumer("f64", binary(NirBinaryOp::Xor, var("word"), var("word")));
    assert!(
        plan(&duplicate, 0, &epoch, &scope, &layouts, &catalog, |v, _| {
            ready(v, &scope)
        })
        .is_none()
    );
    let mut skipped = body.clone();
    let NirStmt::Print(NirExpr::Call { args, .. }) = &mut skipped[1] else {
        unreachable!()
    };
    let NirExpr::Call { args, .. } = &mut args[0] else {
        unreachable!()
    };
    let NirExpr::Binary { lhs, .. } = &mut args[1] else {
        unreachable!()
    };
    *lhs = Box::new(NirExpr::Bool(false));
    // Static false affects the later use, never the preceding initializer's schedule.
    let result = plan(&skipped, 0, &epoch, &scope, &layouts, &catalog, |v, _| {
        ready(v, &scope)
    })
    .unwrap();
    assert!(result.initializer_may_fail);
    assert_eq!((result.definition, result.consumer), (0, 1));
}

#[test]
fn epoch_local_plan_rejects_alias_reassignment_scope_and_noncomparison_escapes() {
    let module = module(false);
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    let scope = scope("f64");
    let epoch = Epoch::enter(&scope, &changed(), &changed(), &layouts);
    let base = body("f64", initializer());
    let check = |body: &[NirStmt], at| {
        plan(body, at, &epoch, &scope, &layouts, &catalog, |v, _| {
            ready(v, &scope)
        })
        .is_some()
    };
    assert!(!check(&[], 0));
    assert!(!check(&base[..1], 0));
    assert!(!check(&base, usize::MAX));
    for tail in [
        NirStmt::Let {
            name: "alias".into(),
            ty: None,
            value: var("word"),
        },
        NirStmt::Let {
            name: "word".into(),
            ty: None,
            value: NirExpr::Int(1),
        },
        NirStmt::Return(Some(var("word"))),
        NirStmt::Print(call("keep64", vec![var("word")])),
        NirStmt::Await(NirExpr::Int(1)),
        NirStmt::If {
            condition: NirExpr::Bool(true),
            then_body: vec![NirStmt::Print(var("word"))],
            else_body: vec![],
        },
        NirStmt::While {
            condition: NirExpr::Bool(false),
            body: vec![NirStmt::Let {
                name: "word".into(),
                ty: None,
                value: NirExpr::Int(1),
            }],
        },
    ] {
        let mut bad = base.clone();
        bad.push(tail);
        assert!(!check(&bad, 0));
    }
    for value in [
        call("bool64", vec![var("word")]),
        call("float64", vec![var("word")]),
        call("narrow64", vec![var("word")]),
        NirExpr::CastI64ToI32(Box::new(var("word"))),
    ] {
        let mut bad = base.clone();
        bad[1] = consumer("f64", value);
        assert!(!check(&bad, 0));
    }
    let mut ordinary = base.clone();
    ordinary[1] = NirStmt::Print(var("word"));
    assert!(!check(&ordinary, 0));
    let mut nonlazy = base.clone();
    let NirStmt::Print(NirExpr::Call { args, .. }) = &mut nonlazy[1] else {
        unreachable!()
    };
    let NirExpr::Call { args, .. } = &mut args[0] else {
        unreachable!()
    };
    let NirExpr::Binary { rhs, .. } = &args[1] else {
        unreachable!()
    };
    args[1] = rhs.as_ref().clone();
    assert!(!check(&nonlazy, 0));
    let mut moved = base.clone();
    moved.insert(
        0,
        NirStmt::Let {
            name: "live".into(),
            ty: None,
            value: var("live"),
        },
    );
    assert!(!check(&moved, 1));
    let mut annotated = base.clone();
    let NirStmt::Let { ty, .. } = &mut annotated[0] else {
        unreachable!()
    };
    *ty = Some(scalar_type("i32"));
    assert!(!check(&annotated, 0));
}

#[test]
fn epoch_local_plan_rejects_uncertified_fields_children_and_effectful_helpers() {
    let module = module(false);
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    let scope = scope("f64");
    let epoch = Epoch::enter(&scope, &changed(), &changed(), &layouts);
    for value in [
        wide(field("missing")),
        wide(field("total")),
        wide(call("effect", vec![field("code")])),
        wide(call("transitive", vec![field("code")])),
        wide(call("cycle_a", vec![field("code")])),
        binary(NirBinaryOp::Xor, wide(field("code")), var("word")),
        binary(NirBinaryOp::Xor, wide(field("code")), var("missing")),
        binary(NirBinaryOp::Add, NirExpr::Int(1), NirExpr::Int(2)),
    ] {
        let body = body("f64", value);
        assert!(
            plan(&body, 0, &epoch, &scope, &layouts, &catalog, |v, _| ready(
                v, &scope
            ))
            .is_none()
        );
    }
    let body = body("f64", initializer());
    for available in [BTreeSet::new(), changed()] {
        let child = Epoch::enter_child(&scope, &changed(), &changed(), &available, &layouts);
        let result = plan(&body, 0, &child, &scope, &layouts, &catalog, |v, _| {
            ready(v, &scope)
        });
        assert_eq!(result.is_some(), !available.is_empty());
        if let Some(result) = result {
            assert!(!std::ptr::eq(result.epoch, &epoch));
        }
    }
    let mut stale = layouts.clone();
    stale.get_mut("Record").unwrap()[0].1 = scalar_type("i64");
    assert!(
        plan(&body, 0, &epoch, &scope, &stale, &catalog, |_, _| Some(
            scalar_type("i32")
        ))
        .is_none()
    );
}

fn balanced(operations: usize, original: bool) -> NirExpr {
    if operations == 0 {
        return if original {
            wide(field("code"))
        } else {
            NirExpr::Int(1)
        };
    }
    let left = (operations - 1) / 2;
    binary(
        NirBinaryOp::Xor,
        balanced(left, original),
        balanced(operations - 1 - left, false),
    )
}

fn direct_consumer_body(value: NirExpr) -> Vec<NirStmt> {
    let mut body = body("f64", value);
    let NirStmt::Print(NirExpr::Call { args, .. }) = &mut body[1] else {
        unreachable!()
    };
    let NirExpr::Call { args, .. } = &mut args[0] else {
        unreachable!()
    };
    let NirExpr::Binary { rhs, .. } = &mut args[1] else {
        unreachable!()
    };
    let NirExpr::Binary { lhs, .. } = rhs.as_mut() else {
        unreachable!()
    };
    *lhs = Box::new(var("word"));
    body
}

#[test]
fn epoch_local_plan_shares_exact_work_depth_and_capture_budgets() {
    let module = module(false);
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    let scope = scope("f64");
    let epoch = Epoch::enter(&scope, &changed(), &changed(), &layouts);
    let check = |body: &[NirStmt]| {
        plan(body, 0, &epoch, &scope, &layouts, &catalog, |v, _| {
            ready(v, &scope)
        })
        .map(|plan| (plan.work, plan.captures.len()))
    };
    // Keep the existing conservative pending-call reservation; no such call
    // remains in the consumer when the shared counter reaches its last nodes.
    let exact = direct_consumer_body(call("keep64", vec![balanced(2041, true)]));
    assert_eq!(check(&exact).unwrap().0, 4096);
    let over = direct_consumer_body(call(
        "keep64",
        vec![call("keep64", vec![balanced(2041, true)])],
    ));
    assert!(check(&over).is_none(), "4097 shared expression work");
    for count in [62, 63] {
        let mut value = wide(field("code"));
        for _ in 0..count {
            value = binary(NirBinaryOp::Xor, value, NirExpr::Int(1));
        }
        assert_eq!(
            check(&body("f64", value)).is_some(),
            count == 62,
            "64/65 initializer depth"
        );
    }
    for count in [27, 28] {
        let mut value = initializer();
        for i in 0..count {
            value = binary(NirBinaryOp::Xor, value, var(&format!("v{i}")));
        }
        let body = body("f64", value);
        let result = check(&body);
        assert_eq!(result.is_some(), count == 27, "31/32 combined captures");
        if let Some(result) = result {
            assert_eq!(result.1, 31);
        }
    }
}

#[test]
fn epoch_local_plan_never_publishes_ordinary_availability_or_admission() {
    let module = module(false);
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    let scope = scope("f64");
    let epoch = Epoch::enter(&scope, &changed(), &changed(), &layouts);
    let body = body("f64", initializer());
    let mut locals = Locals {
        scope: scope.clone(),
        writable: changed(),
        available: BTreeSet::new(),
    };
    assert!(
        plan(&body, 0, &epoch, &scope, &layouts, &catalog, |v, depth| {
            expression(v, &locals, &changed(), None, &catalog, &layouts, depth)
        })
        .is_some()
    );
    assert!(block(
        &body,
        &mut locals,
        &changed(),
        &catalog,
        &layouts,
        validation::LiteralPrints::Scalars,
        &epoch,
        0
    )
    .is_some());
    assert_eq!(locals.scope, scope);
    assert_eq!(locals.writable, changed());
    assert!(locals.available.is_empty());
    assert!(expression(
        &var("word"),
        &locals,
        &changed(),
        None,
        &catalog,
        &layouts,
        0
    )
    .is_none());
    for (mode, depth) in [
        (validation::LiteralPrints::Allow, 0),
        (validation::LiteralPrints::Reject, 0),
        (validation::LiteralPrints::Scalars, 1),
    ] {
        assert!(block(
            &body,
            &mut locals.clone(),
            &changed(),
            &catalog,
            &layouts,
            mode,
            &epoch,
            depth
        )
        .is_none());
    }
    assert!(block(
        &body[..1],
        &mut locals,
        &changed(),
        &catalog,
        &layouts,
        validation::LiteralPrints::Scalars,
        &epoch,
        0
    )
    .is_none());
}

#[test]
fn epoch_local_plan_source_admission_requires_identical_owner_context() {
    let module = module(false);
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    let scope = scope("f64");
    let epoch = Epoch::enter(&scope, &changed(), &changed(), &layouts);
    let body = body("f64", initializer());
    let certificate = plan(&body, 0, &epoch, &scope, &layouts, &catalog, |v, _| {
        ready(v, &scope)
    })
    .unwrap();
    assert!(certificate.matches(&body, 0, &epoch, &scope, &layouts, &catalog));
    let other_epoch = Epoch::enter(&scope, &changed(), &changed(), &layouts);
    assert!(!certificate.matches(&body.clone(), 0, &epoch, &scope, &layouts, &catalog));
    assert!(!certificate.matches(&body, 1, &epoch, &scope, &layouts, &catalog));
    assert!(!certificate.matches(&body, 0, &other_epoch, &scope, &layouts, &catalog));
    assert!(!certificate.matches(&body, 0, &epoch, &scope.clone(), &layouts, &catalog));
    assert!(!certificate.matches(&body, 0, &epoch, &scope, &layouts.clone(), &catalog));
    assert!(!certificate.matches(&body, 0, &epoch, &scope, &layouts, &catalog.clone()));
}

#[test]
fn epoch_local_plan_bounds_whole_scope_nonuse_scan_without_panics() {
    let module = module(false);
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    let scope = scope("f64");
    let epoch = Epoch::enter(&scope, &changed(), &changed(), &layouts);
    for count in [2048, 2049] {
        let mut body = body("f64", initializer());
        body.extend(vec![NirStmt::Print(NirExpr::Int(1)); count]);
        assert_eq!(
            plan(&body, 0, &epoch, &scope, &layouts, &catalog, |v, _| ready(
                v, &scope
            ))
            .is_some(),
            count == 2048
        );
    }
    let mut body = body("f64", initializer());
    body.push(NirStmt::Print(call("opaque", vec![NirExpr::Int(1); 4096])));
    assert!(
        plan(&body, 0, &epoch, &scope, &layouts, &catalog, |v, _| ready(
            v, &scope
        ))
        .is_none()
    );
}
