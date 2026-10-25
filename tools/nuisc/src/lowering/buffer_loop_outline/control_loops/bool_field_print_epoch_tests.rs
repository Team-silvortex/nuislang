use super::*;

fn module() -> NirModule {
    crate::frontend::parse_nuis_module("mod cpu Main {
        struct Pair { total:i64, flag:bool } struct Outer { inner:Pair }
        fn observe(flag:bool,value:i64,d:i64)->i64 { if flag {return value+0/d;} else {return value+10+0/d;} }
        fn narrow(value:i64)->i64 {return value;}
        fn boolean(flag:bool)->bool {return flag;}
        fn record(flag:bool)->Pair {return Pair{total:1,flag:flag};}
        fn whole(value:Outer)->i64 {return value.inner.total;}
        fn impure(flag:bool)->i64 {print(9);return 1;}
        fn main()->i64 {return 0;}
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

fn printed(flag: NirExpr) -> NirExpr {
    call("observe", vec![flag, NirExpr::Int(7), NirExpr::Int(1)])
}

#[test]
fn bool_field_print_epoch_requires_exact_completed_pure_i64_parameter_position() {
    let module = module();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    assert!(catalog.contains_key("impure"));
    assert!(
        scalar_helpers::pure_i64_parameter_type("observe", 0, &catalog)
            .is_some_and(|ty| ty == scalar_type("bool"))
    );
    for (callee, index) in [
        ("observe", 1),
        ("observe", 3),
        ("narrow", 0),
        ("boolean", 0),
        ("record", 0),
        ("whole", 0),
        ("impure", 0),
        ("missing", 0),
    ] {
        assert!(
            !scalar_helpers::pure_i64_parameter_type(callee, index, &catalog)
                .is_some_and(|ty| ty == scalar_type("bool"))
        );
    }
    let scope = Scope::from([("live".into(), scalar_type("Outer"))]);
    let changed = BTreeSet::from(["live".into()]);
    let epoch = Epoch::enter(&scope, &changed, &changed, &layouts);
    let flag = value("live", "flag");
    assert_eq!(
        epoch.argument(&flag, &scope, &layouts, &scalar_type("bool")),
        Some(scalar_type("bool"))
    );
    assert!(epoch.field(&flag, &scope, &layouts).is_none());
    assert!(epoch
        .argument(
            &value("live", "total"),
            &scope,
            &layouts,
            &scalar_type("bool")
        )
        .is_none());
    let mut stale = layouts.clone();
    stale.get_mut("Pair").unwrap()[1].1 = scalar_type("i64");
    assert!(epoch
        .argument(&flag, &scope, &stale, &scalar_type("bool"))
        .is_none());
    let mut stale = scope.clone();
    stale.get_mut("live").unwrap().is_ref = true;
    assert!(epoch
        .argument(&flag, &stale, &layouts, &scalar_type("bool"))
        .is_none());
    assert!(Epoch::default()
        .argument(&flag, &scope, &layouts, &scalar_type("bool"))
        .is_none());
}

#[test]
fn bool_field_print_epoch_grants_one_argument_without_record_or_bool_readiness() {
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
        let valid = printed(value("live", "flag"));
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
        let compound = NirExpr::Binary {
            op: NirBinaryOp::Eq,
            lhs: Box::new(value("live", "flag")),
            rhs: Box::new(NirExpr::Bool(true)),
        };
        for invalid in [
            value("live", "flag"),
            printed(compound),
            printed(value("live", "total")),
            call("observe", vec![value("live", "flag")]),
            call("narrow", vec![value("live", "flag")]),
            call("impure", vec![value("live", "flag")]),
            call("boolean", vec![value("live", "flag")]),
            NirExpr::FieldAccess {
                base: Box::new(call("record", vec![value("live", "flag")])),
                field: "total".into(),
            },
            printed(call("boolean", vec![value("live", "flag")])),
            call("whole", vec![NirExpr::Var("live".into())]),
            call(
                "observe",
                vec![
                    value("live", "flag"),
                    call("whole", vec![NirExpr::Var("live".into())]),
                    NirExpr::Int(1),
                ],
            ),
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
            &value("live", "flag"),
            &locals.scope,
            &layouts,
            &scalar_type("bool")
        )
        .is_none());
    }
}

fn source(body: &str, selected: bool, divisor: i64) -> String {
    format!("mod cpu Main{{struct Pair{{total:i64,flag:bool}} struct Outer{{inner:Pair}} @noinline fn observe(flag:bool,value:i64,d:i64)->i64{{if flag{{return value+0/d;}}else{{return value+10+0/d;}}}} @noinline fn walk(selected:bool,divisor:i64)->i64{{let mut index=0;let mut live=Outer{{inner:Pair{{total:7,flag:true}}}};while index<2{{{body}}}return live.inner.total;}}fn main()->i64{{return walk({selected},{divisor});}}}}")
}

#[test]
fn bool_field_print_epoch_source_keeps_conditions_initializers_and_ancestors_closed() {
    for body in [
        "print(observe(live.inner.flag,7,1));if live.inner.flag{print(1);}live=live;index=index+1;",
        "print(observe(live.inner.flag,7,1));let saved=live.inner.flag;live=live;index=index+1;",
        "let saved=observe(live.inner.flag,7,1);live=live;index=index+1;",
        "print(observe(live.inner.flag==true,7,1));live=live;index=index+1;",
        "let mut child=0;while child<1{print(observe(live.inner.flag,7,1));live=live;child=child+1;}index=index+1;",
        "print(observe(live.inner.flag,7,1));let mut child=0;while child<1{print(observe(live.inner.flag,7,1));live=live;child=child+1;}index=index+1;",
    ] {
        let module=crate::frontend::parse_nuis_module(&source(body,false,0)).unwrap();
        let layouts=control_values::layouts(&module);
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
fn bool_field_print_epoch_raw_nir_keeps_true_false_old_new_and_skipped_poison() {
    for leading in [false, true] {
        for selected in [false, true] {
            let effect="if selected{print(observe(live.inner.flag,live.inner.total,divisor));}print(observe(live.inner.flag,live.inner.total,1));live=Outer{inner:Pair{total:live.inner.total+2,flag:live.inner.flag==false}};print(observe(live.inner.flag,live.inner.total,1));";
            let body = if leading {
                format!("index=index+1;{effect}")
            } else {
                format!("{effect}index=index+1;")
            };
            let module = crate::frontend::parse_nuis_module(&source(
                &body,
                selected,
                if selected { 1 } else { 0 },
            ))
            .unwrap();
            let layouts = control_values::layouts(&module);
            assert!(scalar_helpers::collect_iteration_calls(&module, &layouts).contains_key("walk"));
            crate::nir_verify::verify_nir_module(&module).unwrap();
            let yir = crate::lowering::bootstrap::lower_nir_to_yir_builtin_cpu(&module).unwrap();
            let trace = yir_runtime_host::execute_module_source_with_registry(
                &crate::render::render_yir(&yir),
                &yir_verify::default_registry(),
            )
            .unwrap();
            assert_eq!(
                prints(&trace.events),
                if selected {
                    vec![7, 7, 19, 19, 19, 11]
                } else {
                    vec![7, 19, 19, 11]
                }
            );
            yir_lower_llvm::emit_module(&yir).unwrap();
        }
    }
}

#[test]
fn bool_field_print_epoch_raw_nir_zero_trips_skip_but_selected_calls_fail() {
    let body="if selected{print(observe(live.inner.flag,live.inner.total,divisor));}live=Outer{inner:Pair{total:live.inner.total+1,flag:false}};index=index+1;";
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
