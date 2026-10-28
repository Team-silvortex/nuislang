use super::*;

const CHECKED: [(NirBinaryOp, &str); 2] = [(NirBinaryOp::Div, "/"), (NirBinaryOp::Rem, "%")];
fn compared(value: NirExpr) -> NirExpr {
    in_rhs(binary(NirBinaryOp::Ne, value, projected("tag")))
}

fn float_argument(value: NirExpr) -> NirExpr {
    let NirExpr::Call { mut args, .. } = compared(value) else {
        unreachable!()
    };
    args.remove(0)
}

#[test]
fn i32_division_argument_print_epoch_requires_exact_lazy_operands_and_read_permission() {
    for reversed in [false, true] {
        let mut module = i32_module();
        if reversed {
            module.functions.reverse();
        }
        for (op, _) in CHECKED {
            let result = binary(op, narrow(projected("tag")), projected("tag"));
            for (cmp, _) in COMPARISONS {
                for (left, right) in [
                    (result.clone(), projected("tag")),
                    (projected("tag"), result.clone()),
                ] {
                    let value = in_rhs(binary(cmp, left, right));
                    assert!(shape(&value, &module));
                    assert!(check(&value, &module, false, true).is_some());
                    assert!(check(&value, &module, true, true).is_some());
                    assert!(check(&value, &module, true, false).is_none());
                }
            }
            assert!(check(
                &selector(binary(NirBinaryOp::Ne, result.clone(), projected("tag"))),
                &module,
                false,
                true
            )
            .is_none());
            assert!(check(&compared(narrow(result)), &module, false, true).is_some());
        }
    }
}

#[test]
fn i32_division_argument_print_epoch_rejects_mixed_effectful_and_forged_wide_values() {
    let module = i32_module();
    for (op, _) in CHECKED {
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
                binary(op, bad.clone(), projected("tag")),
                binary(op, projected("tag"), bad),
            ] {
                assert!(check(&compared(value), &module, false, true).is_none());
            }
        }
        let widened = NirExpr::CastI64ToI32(Box::new(binary(
            op,
            NirExpr::CastI32ToI64(Box::new(projected("tag"))),
            NirExpr::CastI32ToI64(Box::new(projected("tag"))),
        )));
        assert!(check(&compared(widened), &module, false, true).is_none());
    }
}

#[test]
fn i32_division_argument_print_epoch_keeps_results_comparison_only() {
    let module = i32_module();
    for (op, _) in CHECKED {
        let value = binary(op, narrow(projected("tag")), projected("tag"));
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
                NirExpr::CastI32ToI64(Box::new(NirExpr::CastI32ToI64(Box::new(value.clone())))),
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
        let wrapped = NirExpr::CastI64ToI32(Box::new(binary(
            NirBinaryOp::Add,
            NirExpr::CastI32ToI64(Box::new(value)),
            NirExpr::CastI32ToI64(Box::new(projected("tag"))),
        )));
        assert!(check(&compared(narrow(wrapped)), &module, false, true).is_some());
    }
}

fn balanced(operations: usize) -> NirExpr {
    if operations == 0 {
        return NirExpr::Var("code".into());
    }
    let left = (operations - 1) / 2;
    binary(
        NirBinaryOp::Div,
        balanced(left),
        balanced(operations - 1 - left),
    )
}

#[test]
fn i32_division_argument_print_epoch_retains_capture_work_depth_and_logical_bounds() {
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
    frame.insert("code".into(), scalar_type("i32"));
    for i in 0..30 {
        frame.insert(format!("v{i}"), scalar_type("i64"));
    }
    for count in [31, 32] {
        let mut args = vec![projected("flag"), projected("tag")];
        args.extend((0..count - 2).map(|i| NirExpr::Var(format!("v{i}"))));
        let value = float_argument(binary(
            NirBinaryOp::Rem,
            projected("tag"),
            call(&format!("batch{count}"), args),
        ));
        assert!(shape(&value, &module));
        assert_eq!(
            binary_float::validate(&value, &scalar_type("f64"), 0, &catalog, |leaf, _| {
                control_values::value_type(leaf, &frame, &catalog, &layouts)
            })
            .is_some(),
            count == 31
        );
    }
    let value = float_argument(binary(
        NirBinaryOp::Div,
        narrow(projected("tag")),
        projected("tag"),
    ));
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
    for count in [32, 33] {
        assert_eq!(
            conditional_values::prefix::computed_call_expression_roots(
                vec![(&value, 0, false); count],
                &catalog
            ),
            count == 32
        );
    }
    for calls in [0, 1] {
        // 2044 binary nodes + 2045 leaves + 7 outer nodes + 0/1 calls = 4096/4097.
        let mut quotient = balanced(2044);
        for _ in 0..calls {
            quotient = narrow(quotient);
        }
        let root = call(
            "choose",
            vec![
                NirExpr::F64("1.0".into()),
                binary(
                    NirBinaryOp::And,
                    NirExpr::Bool(true),
                    binary(NirBinaryOp::Eq, quotient, NirExpr::Var("code".into())),
                ),
                NirExpr::Int(1),
            ],
        );
        assert_eq!(
            binary_float::validate(&root, &scalar_type("f64"), 0, &catalog, |leaf, _| {
                control_values::value_type(leaf, &frame, &catalog, &layouts)
            })
            .is_some(),
            calls == 0
        );
    }
    let mut deep = projected("tag");
    for _ in 0..65 {
        deep = binary(NirBinaryOp::Rem, deep, projected("tag"));
    }
    assert!(check(&compared(deep), &module, false, true).is_none());
}

fn source(
    ty: &str,
    gate: bool,
    cmp: &str,
    logic: &str,
    first: i64,
    second: i64,
    left: i32,
    right: i32,
    token: &str,
) -> String {
    program32(ty, gate, cmp, logic, first, second, left, right).replace(
        "relay32(checked32(live.inner.code,first),second)",
        &format!("relay32(checked32(live.inner.code,first){token}right,second)"),
    )
}

#[test]
fn i32_division_argument_print_epoch_source_preserves_checked_signed_and_skipped_results() {
    for ty in ["f32", "f64"] {
        for (_, token) in CHECKED {
            for (_, cmp) in COMPARISONS {
                for (logic, gate) in [("&&", false), ("&&", true), ("||", false), ("||", true)] {
                    for (left, right) in [
                        (7, 3),
                        (-7, 3),
                        (7, -3),
                        (-7, -3),
                        (i32::MIN, 1),
                        (i32::MAX, -1),
                        (7, 0),
                        (i32::MIN, -1),
                    ] {
                        for (first, second) in [(1, 1), (0, 1), (1, 0)] {
                            let text =
                                source(ty, gate, cmp, logic, first, second, left, right, token);
                            let nir = crate::frontend::parse_nuis_module(&text).unwrap();
                            let yir =
                                crate::lowering::bootstrap::lower_nir_to_yir_builtin_cpu(&nir)
                                    .unwrap();
                            let llvm = yir_lower_llvm::emit_module(&yir).unwrap();
                            assert!(llvm.contains(if token == "/" {
                                "sdiv i32"
                            } else {
                                "srem i32"
                            }));
                            let result = yir_runtime_host::execute_module_source_with_registry(
                                &crate::render::render_yir(&yir),
                                &yir_verify::default_registry(),
                            );
                            let entered = if logic == "&&" { gate } else { !gate };
                            let invalid = right == 0 || (left == i32::MIN && right == -1);
                            if entered && (invalid || first == 0 || second == 0) {
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
                                let a = i64::from(code);
                                let b = i64::from(right);
                                let value = if token == "/" { a / b } else { a % b };
                                arguments.extend([
                                    format!("{code}i32, {first}"),
                                    format!("{value}i32, {second}"),
                                ]);
                                forwarded.push(format!("{value}i32, {second}"));
                                expected.push(match cmp {
                                    "==" => value == b,
                                    "!=" => value != b,
                                    "<" => value < b,
                                    "<=" => value <= b,
                                    ">" => value > b,
                                    ">=" => value >= b,
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

#[test]
fn i32_division_argument_print_epoch_unused_pure_helper_operations_still_trap_when_reached() {
    for token in ["/", "%"] {
        for (left, right) in [(7, 0), (i32::MIN, -1)] {
            for gate in [false, true] {
                let mut text = program32("f64", gate, "!=", "&&", 1, 1, left, right);
                text = text.replace("@noinline fn relay32", &format!("@noinline fn local32(value:i32,divisor:i32)->i32{{let unused=value{token}divisor;return value;}} @noinline fn relay32"));
                text = text.replace(
                    "relay32(checked32(live.inner.code,first),second)",
                    "relay32(local32(live.inner.code,right),second)",
                );
                let nir = crate::frontend::parse_nuis_module(&text).unwrap();
                let yir = crate::lowering::bootstrap::lower_nir_to_yir_builtin_cpu(&nir).unwrap();
                let result = yir_runtime_host::execute_module_source_with_registry(
                    &crate::render::render_yir(&yir),
                    &yir_verify::default_registry(),
                );
                assert_eq!(result.is_err(), gate);
            }
        }
    }
}
