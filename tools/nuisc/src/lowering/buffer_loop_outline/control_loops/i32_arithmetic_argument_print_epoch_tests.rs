use super::*;

const ARITHMETIC: [(NirBinaryOp, &str); 3] = [
    (NirBinaryOp::Add, "+"),
    (NirBinaryOp::Sub, "-"),
    (NirBinaryOp::Mul, "*"),
];
fn wrapped(op: NirBinaryOp, left: NirExpr, right: NirExpr) -> NirExpr {
    NirExpr::CastI64ToI32(Box::new(binary(
        op,
        NirExpr::CastI32ToI64(Box::new(left)),
        NirExpr::CastI32ToI64(Box::new(right)),
    )))
}
fn compared(value: NirExpr) -> NirExpr {
    in_rhs(binary(NirBinaryOp::Ne, value, projected("tag")))
}

#[test]
fn i32_arithmetic_argument_print_epoch_accepts_only_typed_lazy_comparisons() {
    for reversed in [false, true] {
        let mut module = i32_module();
        if reversed {
            module.functions.reverse();
        }
        for (op, _) in ARITHMETIC {
            let sum = wrapped(op, narrow(projected("tag")), projected("tag"));
            for (cmp, _) in COMPARISONS {
                for (left, right) in [
                    (sum.clone(), projected("tag")),
                    (projected("tag"), sum.clone()),
                ] {
                    let value = in_rhs(binary(cmp, left, right));
                    assert!(shape(&value, &module));
                    assert!(check(&value, &module, false, true).is_some());
                    assert!(check(&value, &module, true, true).is_some());
                    assert!(check(&value, &module, true, false).is_none());
                }
            }
            let ordinary = selector(binary(NirBinaryOp::Ne, sum.clone(), projected("tag")));
            assert!(check(&ordinary, &module, false, true).is_none());
            assert!(check(&compared(narrow(sum)), &module, false, true).is_some());
        }
    }
}

#[test]
fn i32_arithmetic_argument_print_epoch_rejects_forged_conversions_and_inputs() {
    let module = i32_module();
    for op in [NirBinaryOp::Div, NirBinaryOp::Rem, NirBinaryOp::Xor] {
        assert!(check(
            &compared(wrapped(op, projected("tag"), projected("tag"))),
            &module,
            false,
            true
        )
        .is_none());
    }
    let extend = |value| NirExpr::CastI32ToI64(Box::new(value));
    for malformed in [
        NirExpr::CastI64ToI32(Box::new(projected("total"))),
        extend(projected("tag")),
        NirExpr::CastI64ToI32(Box::new(binary(
            NirBinaryOp::Add,
            extend(projected("tag")),
            projected("total"),
        ))),
        NirExpr::CastI64ToI32(Box::new(binary(
            NirBinaryOp::Add,
            projected("total"),
            extend(projected("tag")),
        ))),
        binary(NirBinaryOp::Add, projected("tag"), projected("tag")),
    ] {
        assert!(check(&compared(malformed), &module, false, true).is_none());
    }
    for bad in [
        projected("total"),
        projected("small"),
        projected("wide"),
        projected("flag"),
        NirExpr::Var("live".into()),
        NirExpr::Var("missing".into()),
        call("transitive32", vec![projected("tag")]),
        call("cycle32a", vec![projected("tag")]),
        call("record32", vec![NirExpr::Var("live".into())]),
    ] {
        for value in [
            wrapped(NirBinaryOp::Add, bad.clone(), projected("tag")),
            wrapped(NirBinaryOp::Mul, projected("tag"), bad),
        ] {
            assert!(check(&compared(value), &module, false, true).is_none());
        }
    }
}

#[test]
fn i32_arithmetic_argument_print_epoch_retains_comparison_only_provenance() {
    let module = i32_module();
    let value = wrapped(NirBinaryOp::Add, narrow(projected("tag")), projected("tag"));
    for escaped in [
        call(
            "all",
            vec![
                projected("flag"),
                value.clone(),
                projected("total"),
                projected("small"),
                projected("wide"),
            ],
        ),
        binary(
            NirBinaryOp::Eq,
            NirExpr::CastI32ToI64(Box::new(value.clone())),
            projected("total"),
        ),
        binary(
            NirBinaryOp::Eq,
            binary(NirBinaryOp::Add, value.clone(), projected("tag")),
            projected("tag"),
        ),
    ] {
        assert!(check(&in_rhs(escaped), &module, false, true).is_none());
    }
    let nested = wrapped(
        NirBinaryOp::Mul,
        value,
        wrapped(NirBinaryOp::Sub, projected("tag"), narrow(projected("tag"))),
    );
    assert!(check(&compared(narrow(nested)), &module, false, true).is_some());
}

#[test]
fn i32_arithmetic_argument_print_epoch_checks_read_order_and_complete_capture_union() {
    let mut module = i32_module();
    for count in [31, 32] {
        let params = (0..count - 2)
            .map(|i| format!(",p{i}:i64"))
            .collect::<String>();
        module.functions.extend(
            crate::frontend::parse_nuis_module(&format!(
                "mod cpu Batch{{fn batch{count}(flag:bool,tag:i32{params})->i32{{return tag;}}}}"
            ))
            .unwrap()
            .functions,
        );
    }
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    let mut frame = scope();
    for i in 0..30 {
        frame.insert(format!("v{i}"), scalar_type("i64"));
    }
    for count in [31, 32] {
        let mut args = vec![projected("flag"), projected("tag")];
        args.extend((0..count - 2).map(|i| NirExpr::Var(format!("v{i}"))));
        let value = call(
            "choose",
            vec![
                projected("wide"),
                binary(
                    NirBinaryOp::And,
                    projected("flag"),
                    binary(
                        NirBinaryOp::Ne,
                        wrapped(
                            NirBinaryOp::Add,
                            projected("tag"),
                            call(&format!("batch{count}"), args),
                        ),
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
    let value = call(
        "choose",
        vec![
            projected("wide"),
            binary(
                NirBinaryOp::And,
                projected("flag"),
                binary(
                    NirBinaryOp::Eq,
                    wrapped(NirBinaryOp::Sub, narrow(projected("tag")), projected("tag")),
                    projected("tag"),
                ),
            ),
            NirExpr::Int(1),
        ],
    );
    let mut reads = Vec::new();
    assert!(
        binary_float::validate(&value, &scalar_type("f64"), 0, &catalog, |leaf, _| {
            if let NirExpr::FieldAccess { field, .. } = leaf {
                reads.push(field.clone());
            }
            control_values::value_type(leaf, &frame, &catalog, &layouts)
        })
        .is_some()
    );
    assert_eq!(reads, ["wide", "flag", "tag", "tag", "tag"]);
}

fn balanced(operations: usize) -> NirExpr {
    if operations == 0 {
        return NirExpr::Var("code".into());
    }
    let left = (operations - 1) / 2;
    wrapped(
        NirBinaryOp::Add,
        balanced(left),
        balanced(operations - 1 - left),
    )
}

#[test]
fn i32_arithmetic_argument_print_epoch_charges_every_conversion_node_and_depth() {
    let module = i32_module();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    let frame = Scope::from([("code".into(), scalar_type("i32"))]);
    for calls in [3, 4] {
        let mut value = balanced(817);
        for _ in 0..calls {
            value = narrow(value);
        }
        let root = call(
            "choose",
            vec![
                NirExpr::F64("1.0".into()),
                binary(
                    NirBinaryOp::And,
                    NirExpr::Bool(true),
                    binary(NirBinaryOp::Eq, value, NirExpr::Var("code".into())),
                ),
                NirExpr::Int(1),
            ],
        );
        assert_eq!(
            binary_float::validate(&root, &scalar_type("f64"), 0, &catalog, |leaf, _| {
                control_values::value_type(leaf, &frame, &catalog, &layouts)
            })
            .is_some(),
            calls == 3,
            "4096/4097 work"
        );
    }
    let value = compared(wrapped(
        NirBinaryOp::Add,
        projected("tag"),
        projected("tag"),
    ));
    for count in [32, 33] {
        assert_eq!(
            conditional_values::prefix::computed_call_expression_roots(
                vec![(&value, 0, false); count],
                &catalog
            ),
            count == 32
        );
    }
    let mut deep = projected("tag");
    for _ in 0..22 {
        deep = wrapped(NirBinaryOp::Mul, deep, projected("tag"));
    }
    assert!(check(&compared(deep), &module, false, true).is_none());
}

#[test]
fn i32_arithmetic_argument_print_epoch_source_wraps_and_orders_helper_arguments() {
    for ty in ["f32", "f64"] {
        for (arith, token) in ARITHMETIC {
            for (_, cmp) in COMPARISONS {
                for (logic, gate) in [("&&", false), ("&&", true), ("||", false), ("||", true)] {
                    for (left, right) in [
                        (i32::MAX, 1),
                        (i32::MIN, 1),
                        (i32::MIN, -1),
                        (65536, 65536),
                        (i32::MAX, i32::MIN),
                    ] {
                        for (first, second) in [(1, 1), (0, 1), (1, 0)] {
                            let text = program32(ty, gate, cmp, logic, first, second, left, right)
                                .replace(
                                    "relay32(checked32(live.inner.code,first),second)",
                                    &format!(
                                    "relay32(checked32(live.inner.code,first){token}right,second)"
                                ),
                                );
                            let nir = crate::frontend::parse_nuis_module(&text).unwrap();
                            let yir =
                                crate::lowering::bootstrap::lower_nir_to_yir_builtin_cpu(&nir)
                                    .unwrap();
                            yir_lower_llvm::emit_module(&yir).unwrap();
                            let result = yir_runtime_host::execute_module_source_with_registry(
                                &crate::render::render_yir(&yir),
                                &yir_verify::default_registry(),
                            );
                            let entered = if logic == "&&" { gate } else { !gate };
                            if entered && (first == 0 || second == 0) {
                                assert!(result.is_err());
                                continue;
                            }
                            let events = result.unwrap().events;
                            let mut expected = Vec::new();
                            let mut arguments = Vec::new();
                            let mut forwarded = Vec::new();
                            for code in [left, right] {
                                if !entered {
                                    expected.push(gate);
                                    continue;
                                }
                                let value = match arith {
                                    NirBinaryOp::Add => code.wrapping_add(right),
                                    NirBinaryOp::Sub => code.wrapping_sub(right),
                                    NirBinaryOp::Mul => code.wrapping_mul(right),
                                    _ => unreachable!(),
                                };
                                arguments.extend([
                                    format!("{code}i32, {first}"),
                                    format!("{value}i32, {second}"),
                                ]);
                                forwarded.push(format!("{value}i32, {second}"));
                                expected.push(match cmp {
                                    "==" => value == right,
                                    "!=" => value != right,
                                    "<" => value < right,
                                    "<=" => value <= right,
                                    ">" => value > right,
                                    ">=" => value >= right,
                                    _ => unreachable!(),
                                });
                            }
                            let calls = |name: &str| {
                                events
                                    .iter()
                                    .filter_map(|e| {
                                        e.split_once(&format!("] {name}("))
                                            .map(|(_, a)| a.trim_end_matches(')').to_owned())
                                    })
                                    .collect::<Vec<_>>()
                            };
                            assert_eq!(calls("checked32"), arguments);
                            assert_eq!(calls("relay32"), forwarded);
                            let picks = events
                                .iter()
                                .filter_map(|e| {
                                    e.split_once("] choose(").map(|(_, a)| {
                                        a.split(", ")
                                            .nth(1)
                                            .unwrap()
                                            .trim_end_matches(')')
                                            .parse::<bool>()
                                            .unwrap()
                                    })
                                })
                                .collect::<Vec<_>>();
                            assert_eq!(picks, expected);
                            assert_eq!(prints(&events).len(), 2);
                        }
                    }
                }
            }
        }
    }
}
