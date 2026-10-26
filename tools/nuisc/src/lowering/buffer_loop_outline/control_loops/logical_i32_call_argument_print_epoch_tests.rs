use super::*;

fn argument32(flag: NirExpr) -> NirExpr {
    call("choose32", vec![flag, narrow(projected("tag"))])
}
fn admitted32(flag: NirExpr) -> NirExpr {
    in_rhs(binary(NirBinaryOp::Ne, argument32(flag), projected("tag")))
}

#[test]
fn logical_i32_call_argument_print_epoch_proves_scoped_bool_parameters_and_original_reads() {
    for reversed in [false, true] {
        let mut module = i32_module();
        if reversed {
            module.functions.reverse();
        }
        let value = admitted32(logical_flag());
        assert!(shape(&value, &module));
        assert!(check(&value, &module, false, true).is_some());
        assert!(check(&value, &module, true, true).is_some());
        assert!(check(&value, &module, true, false).is_none());
        let comparison = binary(
            NirBinaryOp::Ne,
            argument32(logical_flag()),
            projected("tag"),
        );
        for outside in [
            selector(comparison.clone()),
            selector(binary(NirBinaryOp::And, comparison, projected("flag"))),
        ] {
            assert!(!shape(&outside, &module));
            assert!(check(&outside, &module, false, true).is_none());
        }
        assert!(!conditional_values::prefix::expression(&value));
        assert!(!conditional_values::prefix::computed_logical_root(&value));
    }
}

#[test]
fn logical_i32_call_argument_print_epoch_cannot_launder_comparison_only_results() {
    let module = i32_module();
    let nested = argument32(logical_flag());
    let escaped = call(
        "all",
        vec![
            projected("flag"),
            nested.clone(),
            projected("total"),
            projected("small"),
            projected("wide"),
        ],
    );
    assert!(check(&in_rhs(escaped), &module, false, true).is_none());
    for escaped in [
        binary(NirBinaryOp::Add, nested.clone(), projected("tag")),
        NirExpr::CastI32ToI64(Box::new(nested.clone())),
    ] {
        assert!(check(
            &in_rhs(binary(NirBinaryOp::Eq, escaped, projected("tag"))),
            &module,
            false,
            true
        )
        .is_none());
    }
    let forwarded = in_rhs(binary(NirBinaryOp::Ge, narrow(nested), projected("tag")));
    assert!(shape(&forwarded, &module));
    assert!(check(&forwarded, &module, false, true).is_some());
}

#[test]
fn logical_i32_call_argument_print_epoch_requires_exact_positions_and_complete_dependencies() {
    let module = i32_module();
    let good = vec![
        logical_flag(),
        projected("tag"),
        projected("total"),
        projected("small"),
        projected("wide"),
    ];
    let make = |args| {
        in_rhs(binary(
            NirBinaryOp::Lt,
            call("all32", args),
            projected("tag"),
        ))
    };
    assert!(shape(&make(good.clone()), &module));
    assert!(check(&make(good.clone()), &module, false, true).is_some());
    for index in 0..good.len() {
        let mut bad = good.clone();
        bad[index] = NirExpr::Var("missing".into());
        assert!(check(&make(bad), &module, false, true).is_none());
    }
    let wrong = in_rhs(binary(
        NirBinaryOp::Eq,
        call("choose32", vec![projected("tag"), logical_flag()]),
        projected("tag"),
    ));
    assert!(!shape(&wrong, &module));
    assert!(check(&wrong, &module, false, true).is_none());
    for callee in [
        "record32",
        "noisy32",
        "transitive32",
        "cycle32a",
        "cycle32b",
        "missing",
    ] {
        let value = in_rhs(binary(
            NirBinaryOp::Eq,
            call(callee, vec![logical_flag()]),
            projected("tag"),
        ));
        assert!(!shape(&value, &module), "{callee}");
        assert!(check(&value, &module, false, true).is_none());
    }
}

#[test]
fn logical_i32_call_argument_print_epoch_keeps_nested_i32_argument_provenance() {
    let module = i32_module();
    let inner = binary(
        NirBinaryOp::Lt,
        argument32(logical_flag()),
        projected("tag"),
    );
    let value = admitted32(binary(NirBinaryOp::Or, projected("flag"), inner));
    assert!(shape(&value, &module));
    assert!(check(&value, &module, false, true).is_some());
    let bad = admitted32(binary(
        NirBinaryOp::And,
        NirExpr::Bool(false),
        call("transitive32", vec![projected("tag")]),
    ));
    assert!(check(&bad, &module, false, true).is_none());
    let qualified = scalar_type("i32");
    let mut qualified_module = module.clone();
    let function = qualified_module
        .functions
        .iter_mut()
        .find(|f| f.name == "choose32")
        .unwrap();
    function.return_type = Some(NirTypeRef {
        is_ref: true,
        ..qualified
    });
    assert!(!shape(&value, &qualified_module));
    assert!(check(&value, &qualified_module, false, true).is_none());
}

#[test]
fn logical_i32_call_argument_print_epoch_shares_argument_edge_work_depth_and_capture_budgets() {
    let mut module = i32_module();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    let value = admitted32(logical_flag());
    for count in [16, 17] {
        assert_eq!(
            conditional_values::prefix::computed_call_expression_roots(
                vec![(&value, 0, false); count],
                &catalog
            ),
            count == 16
        );
    }
    let leaf = NirExpr::Int(1);
    for count in [4096, 4097] {
        assert_eq!(
            conditional_values::prefix::computed_call_expression_roots(
                vec![(&leaf, 0, false); count],
                &catalog
            ),
            count == 4096
        );
    }
    assert!(!conditional_values::prefix::computed_call_expression_roots(
        vec![(&value, 64, false)],
        &catalog
    ));
    for count in [31, 32] {
        let params = (0..count - 3)
            .map(|i| format!(",p{i}:i64"))
            .collect::<String>();
        let extra = crate::frontend::parse_nuis_module(&format!(
            "mod cpu Batch{{fn batch{count}(flag:bool,tag:i32{params})->i32{{return tag;}}}}"
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
        let mut args = vec![logical_flag(), projected("tag")];
        args.extend((0..count - 3).map(|i| NirExpr::Var(format!("v{i}"))));
        let value = call(
            "choose",
            vec![
                projected("wide"),
                binary(
                    NirBinaryOp::And,
                    projected("flag"),
                    binary(
                        NirBinaryOp::Eq,
                        call(&format!("batch{count}"), args),
                        projected("tag"),
                    ),
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
}

#[test]
fn logical_i32_call_argument_print_epoch_source_keeps_argument_short_circuit_and_signed_results() {
    for ty in ["f32", "f64"] {
        for (_, op) in COMPARISONS {
            for (left, right) in [(i32::MIN, i32::MAX), (i32::MAX, i32::MIN)] {
                for (outer, gate) in [("&&", true), ("&&", false), ("||", true), ("||", false)] {
                    for inner in ["&&", "||"] {
                        for (first, second) in [(1, 1), (0, 1), (1, 0)] {
                            let replacement=format!("select32((live.inner.code<right){inner}(checked32(live.inner.code,first)!=right),live.inner.code,right,second)");
                            let text=program32(ty,gate,op,outer,first,second,left,right).replace("relay32(checked32(live.inner.code,first),second)",&replacement).replace("@noinline fn walk","@noinline fn select32(pick:bool,value:i32,other:i32,guard:i64)->i32{let checked=0/guard;if pick{return value;}return other;}@noinline fn walk");
                            let module = crate::frontend::parse_nuis_module(&text).unwrap();
                            let yir =
                                crate::lowering::bootstrap::lower_nir_to_yir_builtin_cpu(&module)
                                    .unwrap();
                            let result = yir_runtime_host::execute_module_source_with_registry(
                                &crate::render::render_yir(&yir),
                                &yir_verify::default_registry(),
                            );
                            let selected = if outer == "&&" { gate } else { !gate };
                            let mut expected = Vec::new();
                            let mut bool_args = Vec::new();
                            let mut checks = 0;
                            let mut failed = false;
                            for code in [left, right] {
                                if !selected {
                                    expected.push(gate);
                                    continue;
                                }
                                let lhs = i64::from(code) < i64::from(right);
                                let reached = if inner == "&&" { lhs } else { !lhs };
                                if reached {
                                    checks += 1;
                                    if first == 0 {
                                        failed = true;
                                        break;
                                    }
                                }
                                let pick = if inner == "&&" {
                                    lhs && code != right
                                } else {
                                    lhs || code != right
                                };
                                bool_args.push(pick);
                                if second == 0 {
                                    failed = true;
                                    break;
                                }
                                let value = if pick { code } else { right };
                                let a = i64::from(value);
                                let b = i64::from(right);
                                expected.push(match op {
                                    "==" => a == b,
                                    "!=" => a != b,
                                    "<" => a < b,
                                    "<=" => a <= b,
                                    ">" => a > b,
                                    ">=" => a >= b,
                                    _ => unreachable!(),
                                });
                            }
                            if failed {
                                assert!(result.is_err());
                                continue;
                            }
                            let events = result.unwrap().events;
                            let picks = events
                                .iter()
                                .filter_map(|e| {
                                    e.split_once("] choose(").map(|(_, args)| {
                                        args.split(", ")
                                            .nth(1)
                                            .unwrap()
                                            .trim_end_matches(')')
                                            .parse::<bool>()
                                            .unwrap()
                                    })
                                })
                                .collect::<Vec<_>>();
                            assert_eq!(
                                picks, expected,
                                "{ty} {left} {right} {outer} {inner} {first} {second} {op}"
                            );
                            let actual_args = events
                                .iter()
                                .filter_map(|e| {
                                    e.split_once("] select32(").map(|(_, args)| {
                                        args.split(", ").next().unwrap().parse::<bool>().unwrap()
                                    })
                                })
                                .collect::<Vec<_>>();
                            assert_eq!(actual_args, bool_args);
                            assert_eq!(events.iter().filter(|e|e.contains("cpu.call")&&e.contains("] checked32(")).count(),checks);
                            assert_eq!(prints(&events).len(), 2);
                            yir_lower_llvm::emit_module(&yir).unwrap();
                        }
                    }
                }
            }
        }
    }
}
