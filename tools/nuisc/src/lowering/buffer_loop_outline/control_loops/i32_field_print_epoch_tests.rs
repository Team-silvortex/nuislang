use super::*;

fn module() -> NirModule {
    crate::frontend::parse_nuis_module("mod cpu Main {
        struct Pair{small:i32,total:i64,flag:bool,gain:f32} struct Outer{inner:Pair}
        fn observe(value:i32,d:i64)->i64{if value<i32_from_i64(0){return -1+0/d;}if value==i32_from_i64(0){return 0/d;}return 1+0/d;}
        fn wide(value:i64)->i64{return value;}
        fn narrow(value:i32)->i32{return value;}
        fn record(value:i32)->Pair{return Pair{small:value,total:0,flag:true,gain:1.0};}
        fn whole(value:Outer)->i64{return value.inner.total;}
        fn impure(value:i32)->i64{print(9);return 0;}
        fn main()->i64{return 0;}
    }").unwrap()
}

fn value(name: &str, field: &str) -> NirExpr {
    crate::lowering::scalar_record_shape::source_value(name, &["inner".into(), field.into()])
}

fn call(callee: &str, args: Vec<NirExpr>) -> NirExpr {
    NirExpr::Call {
        callee: callee.into(),
        args,
    }
}

fn printed(value: NirExpr) -> NirExpr {
    call("observe", vec![value, NirExpr::Int(1)])
}

#[test]
fn i32_field_print_epoch_requires_exact_completed_parameter_and_leaf_kinds() {
    let module = module();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    assert_eq!(
        scalar_helpers::pure_i64_parameter_type("observe", 0, &catalog),
        Some(scalar_type("i32"))
    );
    assert_eq!(
        scalar_helpers::pure_i64_parameter_type("observe", 1, &catalog),
        Some(scalar_type("i64"))
    );
    for (name, index) in [
        ("observe", 2),
        ("narrow", 0),
        ("record", 0),
        ("impure", 0),
        ("missing", 0),
    ] {
        assert!(scalar_helpers::pure_i64_parameter_type(name, index, &catalog).is_none());
    }
    let scope = Scope::from([("live".into(), scalar_type("Outer"))]);
    let changed = BTreeSet::from(["live".into()]);
    let epoch = Epoch::enter(&scope, &changed, &changed, &layouts);
    assert_eq!(
        epoch.argument(
            &value("live", "small"),
            &scope,
            &layouts,
            &scalar_type("i32")
        ),
        Some(scalar_type("i32"))
    );
    assert!(epoch
        .field(&value("live", "small"), &scope, &layouts)
        .is_none());
    for (field, expected) in [
        ("small", "i64"),
        ("total", "i32"),
        ("flag", "i32"),
        ("gain", "i32"),
    ] {
        assert!(epoch
            .argument(
                &value("live", field),
                &scope,
                &layouts,
                &scalar_type(expected)
            )
            .is_none());
    }
    let mut qualified = scalar_type("i32");
    qualified.is_ref = true;
    assert!(epoch
        .argument(&value("live", "small"), &scope, &layouts, &qualified)
        .is_none());
    let mut stale = layouts.clone();
    stale.get_mut("Pair").unwrap()[0].1 = scalar_type("i64");
    assert!(epoch
        .argument(&value("live", "small"), &scope, &stale, &scalar_type("i32"))
        .is_none());
    let mut stale = scope.clone();
    stale.get_mut("live").unwrap().is_optional = true;
    assert!(epoch
        .argument(
            &value("live", "small"),
            &stale,
            &layouts,
            &scalar_type("i32")
        )
        .is_none());
}

#[test]
fn i32_field_print_epoch_lends_no_cast_record_or_ordinary_readiness() {
    let module = module();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    let mut locals = Locals {
        scope: Scope::from([("live".into(), scalar_type("Outer"))]),
        writable: BTreeSet::from(["live".into()]),
        available: BTreeSet::new(),
    };
    let changed = locals.writable.clone();
    for child in [false, true] {
        let epoch = if child {
            Epoch::enter_child(&locals.scope, &changed, &changed, &changed, &layouts)
        } else {
            Epoch::enter(&locals.scope, &changed, &changed, &layouts)
        };
        let valid = printed(value("live", "small"));
        assert!(block(
            &[NirStmt::Print(valid.clone())],
            &mut locals,
            &changed,
            &catalog,
            &layouts,
            validation::LiteralPrints::Scalars,
            &epoch,
            0
        )
        .is_some());
        assert!(locals.available.is_empty());
        assert!(expression(&valid, &locals, &changed, None, &catalog, &layouts, 0).is_none());
        assert!(expression_with_fields(
            &valid,
            &locals,
            &changed,
            None,
            &catalog,
            &layouts,
            64,
            Some(&epoch)
        )
        .is_none());
        let already_permitted = printed(NirExpr::CastI64ToI32(Box::new(value("live", "total"))));
        assert!(block(
            &[NirStmt::Print(already_permitted)],
            &mut locals,
            &changed,
            &catalog,
            &layouts,
            validation::LiteralPrints::Scalars,
            &epoch,
            0
        )
        .is_some());
        for mode in [
            validation::LiteralPrints::Allow,
            validation::LiteralPrints::Reject,
        ] {
            assert!(block(
                &[NirStmt::Print(valid.clone())],
                &mut locals,
                &changed,
                &catalog,
                &layouts,
                mode,
                &epoch,
                0
            )
            .is_none());
        }
        for invalid in [
            value("live", "small"),
            NirExpr::CastI32ToI64(Box::new(value("live", "small"))),
            printed(call("narrow", vec![value("live", "small")])),
            call("wide", vec![value("live", "small")]),
            call("observe", vec![value("live", "small")]),
            call(
                "observe",
                vec![
                    value("live", "small"),
                    call("whole", vec![NirExpr::Var("live".into())]),
                ],
            ),
            call("impure", vec![value("live", "small")]),
            NirExpr::FieldAccess {
                base: Box::new(call("record", vec![value("live", "small")])),
                field: "total".into(),
            },
        ] {
            assert!(
                block(
                    &[NirStmt::Print(invalid.clone())],
                    &mut locals,
                    &changed,
                    &catalog,
                    &layouts,
                    validation::LiteralPrints::Scalars,
                    &epoch,
                    0
                )
                .is_none(),
                "{invalid:?}"
            );
        }
        assert!(locals.available.is_empty());
        assert!(Epoch::enter_child(
            &locals.scope,
            &changed,
            &changed,
            &locals.available,
            &layouts
        )
        .argument(
            &value("live", "small"),
            &locals.scope,
            &layouts,
            &scalar_type("i32")
        )
        .is_none());
    }
}

fn source(body: &str, seed: i64, selected: bool, divisor: i64) -> String {
    format!("mod cpu Main{{struct Pair{{small:i32,total:i64}} struct Outer{{inner:Pair}} @noinline fn observe(value:i32,d:i64)->i64{{if value<i32_from_i64(0){{return -1+0/d;}}if value==i32_from_i64(0){{return 0/d;}}return 1+0/d;}} @noinline fn walk(seed:i64,selected:bool,divisor:i64)->i32{{let mut index=0;let mut live=Outer{{inner:Pair{{small:i32_from_i64(seed),total:0}}}};while index<2{{{body}}}return live.inner.small;}}fn main()->i32{{return walk({seed},{selected},{divisor});}}}}")
}

#[test]
fn i32_field_print_epoch_source_keeps_conditions_initializers_and_ancestors_closed() {
    for body in [
        "print(observe(live.inner.small,1));let saved=live.inner.small;live=live;index=index+1;",
        "print(observe(live.inner.small,1));if live.inner.small<i32_from_i64(0){print(1);}live=live;index=index+1;",
        "let saved=observe(live.inner.small,1);live=live;index=index+1;",
        "print(observe(live.inner.small+i32_from_i64(1),1));live=live;index=index+1;",
        "let mut child=0;while child<1{print(observe(live.inner.small,1));live=live;child=child+1;}index=index+1;",
        "print(observe(live.inner.small,1));let mut child=0;while child<1{print(observe(live.inner.small,1));live=live;child=child+1;}index=index+1;",
    ] {
        let module=crate::frontend::parse_nuis_module(&source(body,0,false,0)).unwrap();let layouts=control_values::layouts(&module);
        assert!(!scalar_helpers::collect_iteration_calls(&module,&layouts).contains_key("walk"),"{body}");
    }
}

fn prints(events: &[String]) -> Vec<i64> {
    events
        .iter()
        .filter(|e| {
            e.contains("cpu.print")
                || (e.contains("cpu.guard_print") && e.contains("if true then print"))
        })
        .map(|e| e.rsplit_once(' ').unwrap().1.parse().unwrap())
        .collect()
}

#[test]
fn i32_field_print_epoch_raw_nir_preserves_wrap_sign_zero_and_old_new_versions() {
    for seed in [i32::MIN as i64, -1, 0, i32::MAX as i64, 4294967295] {
        for leading in [false, true] {
            for selected in [false, true] {
                let effect="if selected{print(observe(live.inner.small,divisor));}print(observe(live.inner.small,1));live=Outer{inner:Pair{small:live.inner.small+i32_from_i64(1),total:0}};print(observe(live.inner.small,1));";
                let body = if leading {
                    format!("index=index+1;{effect}")
                } else {
                    format!("{effect}index=index+1;")
                };
                let module = crate::frontend::parse_nuis_module(&source(
                    &body,
                    seed,
                    selected,
                    if selected { 1 } else { 0 },
                ))
                .unwrap();
                let layouts = control_values::layouts(&module);
                assert!(
                    scalar_helpers::collect_iteration_calls(&module, &layouts).contains_key("walk")
                );
                crate::nir_verify::verify_nir_module(&module).unwrap();
                let yir =
                    crate::lowering::bootstrap::lower_nir_to_yir_builtin_cpu(&module).unwrap();
                let trace = yir_runtime_host::execute_module_source_with_registry(
                    &crate::render::render_yir(&yir),
                    &yir_verify::default_registry(),
                )
                .unwrap();
                let mut value = seed as i32;
                let mut expected = Vec::new();
                for _ in 0..2 {
                    if selected {
                        expected.push(i64::from(value.signum()));
                    }
                    expected.push(i64::from(value.signum()));
                    value = value.wrapping_add(1);
                    expected.push(i64::from(value.signum()));
                }
                assert_eq!(prints(&trace.events), expected);
                yir_lower_llvm::emit_module(&yir).unwrap();
            }
        }
    }
}

#[test]
fn i32_field_print_epoch_raw_nir_zero_trips_skip_but_selected_prints_fail() {
    let body = "if selected{print(observe(live.inner.small,divisor));}live=live;index=index+1;";
    for zero in [false, true] {
        let text =
            source(body, -1, true, 0).replace("index<2", if zero { "index<0" } else { "index<2" });
        let module = crate::frontend::parse_nuis_module(&text).unwrap();
        let yir = crate::lowering::bootstrap::lower_nir_to_yir_builtin_cpu(&module).unwrap();
        let result = yir_runtime_host::execute_module_source_with_registry(
            &crate::render::render_yir(&yir),
            &yir_verify::default_registry(),
        );
        if zero {
            assert!(prints(&result.unwrap().events).is_empty());
        } else {
            assert!(result.unwrap_err().contains("zero"));
        }
        yir_lower_llvm::emit_module(&yir).unwrap();
    }
}
