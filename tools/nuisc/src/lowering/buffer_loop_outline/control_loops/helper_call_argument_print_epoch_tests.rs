use super::*;

fn helpers_module() -> NirModule {
    let mut module = calls_module();
    let extra=crate::frontend::parse_nuis_module("mod cpu Extra{struct Pair{small:f32,wide:f64,total:i64,flag:bool,tag:i32}struct Outer{inner:Pair}fn quotient(value:i64,divisor:i64)->i64{return value/divisor;}fn positive(value:i64,divisor:i64)->bool{return quotient(value,divisor)>0;}fn all(flag:bool,tag:i32,total:i64,small:f32,wide:f64)->bool{return flag;}fn narrow(value:i32)->i32{return value;}fn record_bool(value:Outer)->bool{return value.inner.total>0;}fn noisy(value:i64)->i64{print(9);return value;}fn transitive(value:i64)->bool{return noisy(value)>0;}fn cycle_one(value:i64)->bool{return cycle_two(value);}fn cycle_two(value:i64)->bool{return cycle_one(value);}}").unwrap();
    module.functions.extend(extra.functions);
    module
}
fn selector(flag: NirExpr) -> NirExpr {
    call(
        "observe",
        vec![call(
            "choose",
            vec![projected("wide"), flag, NirExpr::Int(1)],
        )],
    )
}
fn check(value: &NirExpr, module: &NirModule, child: bool, ready: bool) -> Option<NirTypeRef> {
    let layouts = control_values::layouts(module);
    let catalog = scalar_helpers::collect_iteration_calls(module, &layouts);
    let locals = Locals {
        scope: scope(),
        writable: BTreeSet::from(["live".into()]),
        available: BTreeSet::new(),
    };
    let changed = locals.writable.clone();
    let epoch = if child {
        Epoch::enter_child(
            &locals.scope,
            &changed,
            &changed,
            if ready { &changed } else { &locals.available },
            &layouts,
        )
    } else {
        Epoch::enter(&locals.scope, &changed, &changed, &layouts)
    };
    assert!(expression(value, &locals, &changed, None, &catalog, &layouts, 0).is_none());
    expression_with_fields(
        value,
        &locals,
        &changed,
        None,
        &catalog,
        &layouts,
        0,
        Some(&epoch),
    )
}
fn positive() -> NirExpr {
    call(
        "positive",
        vec![
            call("quotient", vec![projected("total"), NirExpr::Int(1)]),
            NirExpr::Int(1),
        ],
    )
}

#[test]
fn helper_call_argument_print_epoch_uses_completed_exact_primitive_signatures() {
    for reversed in [false, true] {
        let mut module = helpers_module();
        if reversed {
            module.functions.reverse();
            module.structs.reverse();
        }
        let layouts = control_values::layouts(&module);
        let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
        for (callee, result) in [
            ("quotient", "i64"),
            ("positive", "bool"),
            ("narrow", "i32"),
            ("all", "bool"),
        ] {
            let (_, actual) = scalar_helpers::pure_scalar_signature(callee, &catalog).unwrap();
            assert_eq!(actual, &scalar_type(result));
        }
        for callee in [
            "record_bool",
            "noisy",
            "transitive",
            "cycle_one",
            "cycle_two",
            "missing",
        ] {
            assert!(
                scalar_helpers::pure_scalar_signature(callee, &catalog).is_none(),
                "{callee}"
            );
        }
        let params = vec![
            projected("flag"),
            projected("tag"),
            projected("total"),
            projected("small"),
            projected("wide"),
        ];
        let value = selector(binary(
            NirBinaryOp::And,
            projected("flag"),
            call("all", params.clone()),
        ));
        assert_eq!(
            check(&value, &module, false, true),
            Some(scalar_type("i64"))
        );
        for slot in 0..params.len() {
            let mut invalid = params.clone();
            invalid[slot] = NirExpr::Var("missing".into());
            assert!(check(
                &selector(binary(
                    NirBinaryOp::And,
                    projected("flag"),
                    call("all", invalid)
                )),
                &module,
                false,
                true
            )
            .is_none());
        }
    }
}

#[test]
fn helper_call_argument_print_epoch_scopes_nested_calls_to_lazy_rhs_only() {
    let module = helpers_module();
    for op in [NirBinaryOp::And, NirBinaryOp::Or] {
        let value = selector(binary(op, projected("flag"), positive()));
        assert!(check(&value, &module, false, true).is_some());
        assert!(check(&value, &module, true, true).is_some());
        assert!(check(&value, &module, true, false).is_none());
        assert!(check(
            &selector(binary(op, positive(), projected("flag"))),
            &module,
            false,
            true
        )
        .is_none());
        let nested = selector(binary(
            op,
            projected("flag"),
            binary(op, positive(), projected("flag")),
        ));
        assert!(check(&nested, &module, false, true).is_some());
    }
    assert!(check(&selector(positive()), &module, false, true).is_none());
    for bad in [
        call("positive", vec![]),
        call("positive", vec![projected("small"), NirExpr::Int(1)]),
        call("transitive", vec![projected("total")]),
        call("record_bool", vec![NirExpr::Var("live".into())]),
        binary(
            NirBinaryOp::Eq,
            call("narrow", vec![projected("tag")]),
            projected("total"),
        ),
    ] {
        assert!(check(
            &selector(binary(NirBinaryOp::And, NirExpr::Bool(false), bad)),
            &module,
            false,
            true
        )
        .is_none());
    }
}

#[test]
fn helper_call_argument_print_epoch_keeps_auxiliary_logical_argument_roots_closed() {
    let module = helpers_module();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    let value = selector(call(
        "all",
        vec![
            binary(NirBinaryOp::And, projected("flag"), positive()),
            projected("tag"),
            projected("total"),
            projected("small"),
            projected("wide"),
        ],
    ));
    assert!(!iteration_prints::bounded(
        &[NirStmt::Print(value.clone())],
        &catalog
    ));
    let mut locals = Locals {
        scope: scope(),
        writable: BTreeSet::from(["live".into()]),
        available: BTreeSet::new(),
    };
    let changed = locals.writable.clone();
    let epoch = Epoch::enter(&locals.scope, &changed, &changed, &layouts);
    assert!(block(
        &[NirStmt::Print(value)],
        &mut locals,
        &changed,
        &catalog,
        &layouts,
        validation::LiteralPrints::Scalars,
        &epoch,
        0
    )
    .is_none());
}

#[test]
fn helper_call_argument_print_epoch_bounds_complete_call_capture_union_and_depth() {
    let mut module = helpers_module();
    for count in [31, 32] {
        let params = (0..count)
            .map(|i| format!("p{i}:i64"))
            .collect::<Vec<_>>()
            .join(",");
        let extra = crate::frontend::parse_nuis_module(&format!(
            "mod cpu Batch{{fn batch{count}({params})->bool{{return true;}}}}"
        ))
        .unwrap();
        module.functions.extend(extra.functions);
    }
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    let mut frame = scope();
    for i in 0..31 {
        frame.insert(format!("v{i}"), scalar_type("i64"));
    }
    for count in [31, 32] {
        let mut args = vec![projected("total")];
        args.extend((0..count - 1).map(|i| NirExpr::Var(format!("v{i}"))));
        let value = call(
            "choose",
            vec![
                projected("wide"),
                binary(
                    NirBinaryOp::And,
                    projected("flag"),
                    call(&format!("batch{count}"), args),
                ),
                NirExpr::Int(1),
            ],
        );
        assert_eq!(
            binary_float::validate(&value, &scalar_type("f64"), 0, &catalog, |leaf, _| {
                control_values::value_type(leaf, &frame, &catalog, &layouts)
            })
            .is_some(),
            count == 31
        );
    }
    let mut value = projected("total");
    for _ in 0..66 {
        value = call("quotient", vec![value, NirExpr::Int(1)]);
    }
    assert!(check(
        &selector(binary(
            NirBinaryOp::And,
            projected("flag"),
            call("positive", vec![value, NirExpr::Int(1)])
        )),
        &module,
        false,
        true
    )
    .is_none());
}

#[test]
fn helper_call_argument_print_epoch_source_keeps_skipped_calls_under_sign_wrappers() {
    for ty in ["f32", "f64"] {
        for gate in [false, true] {
            for divisor in [0, 1] {
                for op in ["&&", "||"] {
                    for negative in [false, true] {
                        let mut text=program(ty,gate,divisor,1,false,false).replace("(live.inner.tag/first>=50&&live.inner.flag)||live.inner.tag==99",&format!("live.inner.flag{op}positive(quotient(live.inner.tag,first),second)"));
                        text=text.replace("@noinline fn walk","@noinline fn quotient(value:i64,divisor:i64)->i64{return value/divisor;}@noinline fn positive(value:i64,divisor:i64)->bool{return quotient(value,divisor)>0;}@noinline fn walk");
                        if negative {
                            text = text.replace("observe(choose(", "observe(-choose(");
                        }
                        let module = crate::frontend::parse_nuis_module(&text).unwrap();
                        let layouts = control_values::layouts(&module);
                        assert!(scalar_helpers::collect_iteration_calls(&module, &layouts)
                            .contains_key("walk"));
                        let yir = crate::lowering::bootstrap::lower_nir_to_yir_builtin_cpu(&module)
                            .unwrap();
                        let result = yir_runtime_host::execute_module_source_with_registry(
                            &crate::render::render_yir(&yir),
                            &yir_verify::default_registry(),
                        );
                        let selected = if op == "&&" { gate } else { !gate };
                        if selected && divisor == 0 {
                            assert!(result.is_err());
                        } else {
                            let events = result.unwrap().events;
                            let q = events
                                .iter()
                                .filter(|e| e.contains("cpu.call") && e.contains("] quotient("))
                                .count();
                            let b = events
                                .iter()
                                .filter(|e| e.contains("cpu.call") && e.contains("] positive("))
                                .count();
                            assert_eq!(q, if selected { 4 } else { 0 });
                            assert_eq!(b, if selected { 2 } else { 0 });
                            assert_eq!(prints(&events).len(), 2);
                        }
                        yir_lower_llvm::emit_module(&yir).unwrap();
                    }
                }
            }
        }
    }
}

#[path = "logical_helper_argument_print_epoch_tests.rs"]
mod logical_arguments;
