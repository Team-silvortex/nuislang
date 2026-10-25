use super::*;

fn module() -> NirModule {
    crate::frontend::parse_nuis_module("mod cpu Main {
        struct Pair{small:f64,total:i64,wide:f32} struct Outer{inner:Pair}
        fn observe(value:f64,d:i64)->i64{if value<0.0{return -1+0/d;}if value==0.0{return 0/d;}if value!=value{return 2+0/d;}return 1+0/d;}
        fn wide(value:f32)->i64{return 0;}
        fn narrow(value:f64)->f64{return value;}
        fn record(value:f64)->Pair{return Pair{small:value,total:0,wide:1.0};}
        fn whole(value:Outer)->i64{return value.inner.total;}
        fn impure(value:f64)->i64{print(9);return 0;}
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
fn f64_field_print_epoch_requires_exact_parameter_leaf_and_layout() {
    let module = module();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    assert_eq!(
        scalar_helpers::pure_i64_parameter_type("observe", 0, &catalog),
        Some(scalar_type("f64"))
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
            &scalar_type("f64")
        ),
        Some(scalar_type("f64"))
    );
    assert!(epoch
        .field(&value("live", "small"), &scope, &layouts)
        .is_none());
    for (field, expected) in [
        ("small", "i32"),
        ("small", "f32"),
        ("total", "f64"),
        ("wide", "f64"),
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
    let mut qualified = scalar_type("f64");
    qualified.is_ref = true;
    assert!(epoch
        .argument(&value("live", "small"), &scope, &layouts, &qualified)
        .is_none());
    let mut stale = layouts.clone();
    stale.get_mut("Pair").unwrap()[0].1 = scalar_type("f32");
    assert!(epoch
        .argument(&value("live", "small"), &scope, &stale, &scalar_type("f64"))
        .is_none());
    let mut stale = scope.clone();
    stale.get_mut("live").unwrap().is_ref = true;
    assert!(epoch
        .argument(
            &value("live", "small"),
            &stale,
            &layouts,
            &scalar_type("f64")
        )
        .is_none());
}

#[test]
fn f64_field_print_epoch_keeps_word_cast_record_and_ordinary_reads_closed() {
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
            NirExpr::PackF64Word(Box::new(value("live", "small"))),
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
            &scalar_type("f64")
        )
        .is_none());
    }
}

fn source(body: &str, selected: bool, divisor: i64) -> String {
    format!("mod cpu Main{{struct Pair{{small:f64,total:i64}} struct Outer{{inner:Pair}} @noinline fn observe(value:f64,d:i64)->i64{{if value<0.0{{return -1+0/d;}}if value==0.0{{return 0/d;}}if value!=value{{return 2+0/d;}}return 1+0/d;}} @noinline fn walk(seed:f64,selected:bool,divisor:i64)->f64{{let mut index=0;let mut live=Outer{{inner:Pair{{small:seed,total:0}}}};while index<2{{{body}}}return live.inner.small;}}fn main()->f64{{return walk(1.5,{selected},{divisor});}}}}")
}

#[test]
fn f64_field_print_epoch_source_keeps_conditions_compounds_and_ancestors_closed() {
    for body in [
        "print(observe(live.inner.small,1));let saved=live.inner.small;live=live;index=index+1;",
        "print(observe(live.inner.small,1));if live.inner.small<0.0{print(1);}live=live;index=index+1;",
        "let saved=observe(live.inner.small,1);live=live;index=index+1;",
        "print(observe(-live.inner.small,1));live=live;index=index+1;",
        "let mut child=0;while child<1{print(observe(live.inner.small,1));live=live;child=child+1;}index=index+1;",
        "print(observe(live.inner.small,1));let mut child=0;while child<1{print(observe(live.inner.small,1));live=live;child=child+1;}index=index+1;",
    ] {
        let module=crate::frontend::parse_nuis_module(&source(body,false,0)).unwrap();let layouts=control_values::layouts(&module);
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

fn classify(bits: u64) -> i64 {
    let value = f64::from_bits(bits);
    if value < 0.0 {
        -1
    } else if value == 0.0 {
        0
    } else if value.is_nan() {
        2
    } else {
        1
    }
}

#[test]
fn f64_field_print_epoch_raw_nir_keeps_ieee_versions_and_skips_poison() {
    for bits in [
        0u64,
        0x8000000000000000,
        0x3ff8000000000000,
        0xc004000000000000,
        1,
        0x800fffffffffffff,
        0x7ff0000000000000,
        0xfff0000000000000,
        0x7ff812345678abcd,
        0xfff0000000000001,
        0x3ff000000000005e,
        0x3ff000000000005f,
    ] {
        for leading in [false, true] {
            for selected in [false, true] {
                let effect="if selected{print(observe(live.inner.small,divisor));}print(observe(live.inner.small,1));live=Outer{inner:Pair{small:-live.inner.small,total:0}};print(observe(live.inner.small,1));";
                let body = if leading {
                    format!("index=index+1;{effect}")
                } else {
                    format!("{effect}index=index+1;")
                };
                let mut module = crate::frontend::parse_nuis_module(&source(
                    &body,
                    selected,
                    if selected { 1 } else { 0 },
                ))
                .unwrap();
                let main = module
                    .functions
                    .iter_mut()
                    .find(|f| f.name == "main")
                    .unwrap();
                let NirStmt::Return(Some(NirExpr::Call { args, .. })) = &mut main.body[0] else {
                    panic!("raw source entry");
                };
                args[0] = NirExpr::UnpackF64Word(Box::new(NirExpr::Int(bits as i64)));
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
                let mut current = bits;
                let mut expected = Vec::new();
                for _ in 0..2 {
                    if selected {
                        expected.push(classify(current));
                    }
                    expected.push(classify(current));
                    current ^= 0x8000000000000000;
                    expected.push(classify(current));
                }
                assert_eq!(prints(&trace.events), expected);
                yir_lower_llvm::emit_module(&yir).unwrap();
            }
        }
    }
}

#[test]
fn f64_field_print_epoch_raw_nir_zero_trips_skip_but_selected_calls_fail() {
    let body = "if selected{print(observe(live.inner.small,divisor));}live=live;index=index+1;";
    for zero in [false, true] {
        let text =
            source(body, true, 0).replace("index<2", if zero { "index<0" } else { "index<2" });
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
