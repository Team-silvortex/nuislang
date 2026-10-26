use super::*;

fn module() -> NirModule {
    let mut module = helpers_module();
    let extra = crate::frontend::parse_nuis_module("mod cpu Args {
        fn accept(pick:bool,value:i64,divisor:i64)->bool{let checked=quotient(value,divisor);return pick==(checked>0);}
        fn weight(pick:bool,value:i64,divisor:i64)->i64{let checked=quotient(value,divisor);if pick{return checked;}return 0;}
        fn quotient(value:i64,divisor:i64)->i64{return value/divisor;}
    }").unwrap();
    module
        .functions
        .extend(extra.functions.into_iter().filter(|f| f.name != "quotient"));
    module
}

fn auxiliary(name: &str, flag: NirExpr) -> NirExpr {
    let call = call(name, vec![flag, projected("total"), NirExpr::Int(1)]);
    if name == "weight" {
        binary(NirBinaryOp::Gt, call, NirExpr::Int(0))
    } else {
        call
    }
}

fn logical_flag() -> NirExpr {
    binary(NirBinaryOp::And, projected("flag"), positive())
}

fn authorized(name: &str) -> NirExpr {
    selector(binary(
        NirBinaryOp::And,
        projected("flag"),
        auxiliary(name, logical_flag()),
    ))
}

fn shape(value: &NirExpr, module: &NirModule) -> bool {
    let layouts = control_values::layouts(module);
    let catalog = scalar_helpers::collect_iteration_calls(module, &layouts);
    iteration_prints::bounded(&[NirStmt::Print(value.clone())], &catalog)
}

#[test]
fn logical_helper_argument_print_epoch_requires_lazy_scope_and_original_reads() {
    for reversed in [false, true] {
        let mut module = module();
        if reversed {
            module.functions.reverse();
        }
        for name in ["accept", "weight"] {
            let value = authorized(name);
            assert!(shape(&value, &module));
            assert!(check(&value, &module, false, true).is_some());
            assert!(check(&value, &module, true, true).is_some());
            assert!(check(&value, &module, true, false).is_none());
            for value in [
                selector(auxiliary(name, logical_flag())),
                selector(binary(
                    NirBinaryOp::And,
                    auxiliary(name, logical_flag()),
                    projected("flag"),
                )),
            ] {
                assert!(!shape(&value, &module));
                assert!(check(&value, &module, false, true).is_none());
            }
            assert!(!conditional_values::prefix::expression(&value));
            assert!(!conditional_values::prefix::computed_logical_root(&value));
        }
    }
}

#[test]
fn logical_helper_argument_print_epoch_keeps_exact_parameters_and_effect_vetoes() {
    let module = module();
    for args in [
        vec![],
        vec![logical_flag(), projected("total")],
        vec![projected("total"), logical_flag(), NirExpr::Int(1)],
        vec![logical_flag(), projected("small"), NirExpr::Int(1)],
        vec![
            logical_flag(),
            projected("total"),
            NirExpr::Var("missing".into()),
        ],
        vec![logical_flag(), NirExpr::Var("live".into()), NirExpr::Int(1)],
    ] {
        let value = selector(binary(
            NirBinaryOp::And,
            projected("flag"),
            call("accept", args),
        ));
        assert!(check(&value, &module, false, true).is_none());
    }
    for name in [
        "missing",
        "noisy",
        "transitive",
        "record_bool",
        "cycle_one",
        "narrow",
    ] {
        let value = selector(binary(
            NirBinaryOp::And,
            projected("flag"),
            call(name, vec![logical_flag()]),
        ));
        assert!(!shape(&value, &module), "{name}");
        assert!(check(&value, &module, false, true).is_none());
    }
    let bad = selector(binary(
        NirBinaryOp::And,
        projected("flag"),
        call(
            "accept",
            vec![
                binary(
                    NirBinaryOp::And,
                    NirExpr::Bool(false),
                    call("transitive", vec![projected("total")]),
                ),
                projected("total"),
                NirExpr::Int(1),
            ],
        ),
    ));
    assert!(check(&bad, &module, false, true).is_none());
}

#[test]
fn logical_helper_argument_print_epoch_shares_edge_work_and_capture_budgets() {
    let mut module = module();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    let value = authorized("accept");
    for count in [16, 17] {
        assert_eq!(
            conditional_values::prefix::computed_call_expression_roots(
                vec![(&value, 0, false); count],
                &catalog
            ),
            count == 16
        );
    }
    for count in [4096, 4097] {
        let leaf = NirExpr::Int(1);
        assert_eq!(
            conditional_values::prefix::computed_call_expression_roots(
                vec![(&leaf, 0, false); count],
                &catalog
            ),
            count == 4096
        );
    }
    for count in [31, 32] {
        let params = (0..count - 2)
            .map(|i| format!(",p{i}:i64"))
            .collect::<String>();
        let extra = crate::frontend::parse_nuis_module(&format!(
            "mod cpu Batch{{fn batch{count}(flag:bool{params})->bool{{return flag;}}}}"
        ))
        .unwrap();
        module.functions.extend(extra.functions);
    }
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    let mut frame = scope();
    for i in 0..30 {
        frame.insert(format!("v{i}"), scalar_type("i64"));
    }
    for count in [31, 32] {
        let mut args = vec![logical_flag()];
        args.extend((0..count - 2).map(|i| NirExpr::Var(format!("v{i}"))));
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
        assert!(shape(&value, &module));
        assert_eq!(
            binary_float::validate(&value, &scalar_type("f64"), 0, &catalog, |leaf, _| {
                control_values::value_type(leaf, &frame, &catalog, &layouts)
            })
            .is_some(),
            count == 31
        );
    }
    assert!(!conditional_values::prefix::computed_call_expression_roots(
        vec![(&value, 64, false)],
        &catalog
    ));
}

#[test]
fn logical_helper_argument_print_epoch_source_guards_nested_argument_calls() {
    for ty in ["f32", "f64"] {
        for name in ["accept", "weight"] {
            for (outer, outer_gate) in [("&&", false), ("&&", true), ("||", false), ("||", true)] {
                for (inner, inner_gate) in
                    [("&&", false), ("&&", true), ("||", false), ("||", true)]
                {
                    for first in [0, 1] {
                        let nested = format!(
                            "{inner_gate}{inner}positive(quotient(live.inner.tag,first),1)"
                        );
                        let arg = if name == "accept" {
                            format!("accept({nested},live.inner.tag,second)")
                        } else {
                            format!("positive(weight({nested},live.inner.tag,second),1)")
                        };
                        let mut text = program(ty, outer_gate, first, 1, false, false).replace(
                            "(live.inner.tag/first>=50&&live.inner.flag)||live.inner.tag==99",
                            &format!("live.inner.flag{outer}{arg}"),
                        );
                        text=text.replace("@noinline fn walk","@noinline fn quotient(value:i64,divisor:i64)->i64{return value/divisor;}@noinline fn positive(value:i64,divisor:i64)->bool{return quotient(value,divisor)>0;}@noinline fn accept(pick:bool,value:i64,divisor:i64)->bool{let checked=quotient(value,divisor);return pick==(checked>0);}@noinline fn weight(pick:bool,value:i64,divisor:i64)->i64{let checked=quotient(value,divisor);if pick{return checked;}return 0;}@noinline fn walk").replace("observe(choose(","observe(-choose(");
                        let module = crate::frontend::parse_nuis_module(&text).unwrap();
                        let yir = crate::lowering::bootstrap::lower_nir_to_yir_builtin_cpu(&module)
                            .unwrap();
                        let result = yir_runtime_host::execute_module_source_with_registry(
                            &crate::render::render_yir(&yir),
                            &yir_verify::default_registry(),
                        );
                        let selected = if outer == "&&" {
                            outer_gate
                        } else {
                            !outer_gate
                        };
                        let inner_selected = if inner == "&&" {
                            inner_gate
                        } else {
                            !inner_gate
                        };
                        if selected && inner_selected && first == 0 {
                            assert!(result.is_err());
                        } else {
                            let events = result.unwrap().events;
                            let calls = |callee: &str| {
                                events
                                    .iter()
                                    .filter(|e| {
                                        e.contains("cpu.call")
                                            && e.contains(&format!("] {callee}("))
                                    })
                                    .count()
                            };
                            assert_eq!(calls(name), if selected { 2 } else { 0 });
                            assert_eq!(
                                calls("quotient"),
                                if selected {
                                    2 * ((if inner_selected { 2 } else { 0 })
                                        + if name == "weight" { 2 } else { 1 })
                                } else {
                                    0
                                }
                            );
                            assert_eq!(prints(&events).len(), 2);
                        }
                        yir_lower_llvm::emit_module(&yir).unwrap();
                    }
                }
            }
        }
    }
}

#[path = "float_compare_argument_print_epoch_tests.rs"]
mod float_comparisons;
