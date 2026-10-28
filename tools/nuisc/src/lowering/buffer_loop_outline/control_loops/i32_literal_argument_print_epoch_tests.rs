use super::*;

fn literal(value: i64) -> NirExpr {
    NirExpr::CastI64ToI32(Box::new(NirExpr::Int(value)))
}
fn negative(magnitude: i64) -> NirExpr {
    NirExpr::CastI64ToI32(Box::new(binary(
        NirBinaryOp::Sub,
        NirExpr::Int(0),
        NirExpr::Int(magnitude),
    )))
}
fn compared(value: NirExpr) -> NirExpr {
    in_rhs(binary(NirBinaryOp::Ne, value, projected("tag")))
}
fn float_argument(value: NirExpr) -> NirExpr {
    let NirExpr::Call { mut args, .. } = compared(value) else {
        unreachable!()
    };
    args.remove(0)
}
fn wrapped(op: NirBinaryOp, left: NirExpr, right: NirExpr) -> NirExpr {
    NirExpr::CastI64ToI32(Box::new(binary(
        op,
        NirExpr::CastI32ToI64(Box::new(left)),
        NirExpr::CastI32ToI64(Box::new(right)),
    )))
}

#[test]
fn i32_literal_argument_print_epoch_accepts_exact_in_range_lazy_comparisons() {
    for reversed in [false, true] {
        let mut module = i32_module();
        if reversed {
            module.functions.reverse();
        }
        for value in [i64::from(i32::MIN), -1, 0, 1, 107, i64::from(i32::MAX)] {
            for (cmp, _) in COMPARISONS {
                for (left, right) in [
                    (literal(value), projected("tag")),
                    (projected("tag"), literal(value)),
                ] {
                    let root = in_rhs(binary(cmp, left, right));
                    assert!(shape(&root, &module));
                    assert!(check(&root, &module, false, true).is_some());
                    assert!(check(&root, &module, true, true).is_some());
                    assert!(check(&root, &module, true, false).is_none());
                }
            }
            assert!(check(&compared(narrow(literal(value))), &module, false, true).is_some());
        }
        for magnitude in [0, 1, 2147483648] {
            assert!(check(&compared(negative(magnitude)), &module, false, true).is_some());
        }
    }
}

#[test]
fn i32_literal_argument_print_epoch_rejects_out_of_range_and_nonliteral_casts() {
    let module = i32_module();
    for magnitude in [-1, i64::MIN, 2147483649, i64::MAX] {
        assert!(check(&compared(negative(magnitude)), &module, false, true).is_none());
    }
    for value in [
        i64::MIN,
        i64::from(i32::MIN) - 1,
        i64::from(i32::MAX) + 1,
        i64::MAX,
    ] {
        assert!(check(&compared(literal(value)), &module, false, true).is_none());
    }
    for value in [
        projected("total"),
        projected("small"),
        projected("wide"),
        projected("flag"),
        NirExpr::Var("missing".into()),
        binary(NirBinaryOp::Add, NirExpr::Int(1), NirExpr::Int(1)),
        binary(NirBinaryOp::Sub, NirExpr::Int(1), NirExpr::Int(1)),
        binary(NirBinaryOp::Sub, NirExpr::Int(0), projected("total")),
        binary(
            NirBinaryOp::Sub,
            NirExpr::Int(0),
            binary(NirBinaryOp::Sub, NirExpr::Int(0), NirExpr::Int(1)),
        ),
        call("quotient", vec![NirExpr::Int(1), NirExpr::Int(1)]),
        NirExpr::CastI32ToI64(Box::new(literal(1))),
    ] {
        let cast = NirExpr::CastI64ToI32(Box::new(value));
        assert!(check(&compared(cast), &module, false, true).is_none());
    }
    for name in ["transitive32", "cycle32a", "record32", "missing"] {
        assert!(check(
            &compared(call(name, vec![literal(1)])),
            &module,
            false,
            true
        )
        .is_none());
    }
}

#[test]
fn i32_literal_argument_print_epoch_retains_scope_and_comparison_only_provenance() {
    let module = i32_module();
    assert!(check(
        &selector(binary(NirBinaryOp::Eq, literal(1), projected("tag"))),
        &module,
        false,
        true
    )
    .is_none());
    for value in [
        call(
            "all",
            vec![
                projected("flag"),
                literal(1),
                projected("total"),
                projected("small"),
                projected("wide"),
            ],
        ),
        binary(
            NirBinaryOp::Eq,
            NirExpr::CastI32ToI64(Box::new(NirExpr::CastI32ToI64(Box::new(literal(1))))),
            projected("total"),
        ),
        binary(NirBinaryOp::Eq, literal(1), NirExpr::Int(1)),
        binary(
            NirBinaryOp::Eq,
            binary(NirBinaryOp::Add, literal(1), projected("tag")),
            projected("tag"),
        ),
    ] {
        assert!(check(&in_rhs(value), &module, false, true).is_none());
    }
    for op in [
        NirBinaryOp::Add,
        NirBinaryOp::Sub,
        NirBinaryOp::Mul,
        NirBinaryOp::Div,
        NirBinaryOp::Rem,
    ] {
        let value = if matches!(op, NirBinaryOp::Div | NirBinaryOp::Rem) {
            binary(op, narrow(projected("tag")), literal(1))
        } else {
            wrapped(op, narrow(projected("tag")), literal(1))
        };
        assert!(check(&compared(narrow(value)), &module, false, true).is_some());
    }
}

fn balanced(operations: usize) -> NirExpr {
    if operations == 0 {
        return literal(1);
    }
    let left = (operations - 1) / 2;
    binary(
        NirBinaryOp::Div,
        balanced(left),
        balanced(operations - 1 - left),
    )
}

#[test]
fn i32_literal_argument_print_epoch_counts_both_nodes_and_preserves_capture_bounds() {
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
        let mut args = vec![projected("flag"), literal(1)];
        args.extend((0..count - 2).map(|i| NirExpr::Var(format!("v{i}"))));
        let value = float_argument(binary(
            NirBinaryOp::Div,
            projected("tag"),
            call(&format!("batch{count}"), args),
        ));
        assert_eq!(
            binary_float::validate(&value, &scalar_type("f64"), 0, &catalog, |leaf, _| {
                control_values::value_type(leaf, &frame, &catalog, &layouts)
            })
            .is_some(),
            count == 31
        );
    }
    for (operations, signed, start) in [(1362, false, 1), (1361, true, 2)] {
        for calls in [start, start + 1] {
            // Each cast/literal is charged; canonical negative literals add two nodes.
            let mut value = balanced(operations);
            if signed {
                let mut first = &mut value;
                while let NirExpr::Binary { lhs, .. } = first {
                    first = lhs;
                }
                *first = negative(1);
            }
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
                calls == start
            );
        }
    }
    let value = float_argument(narrow(literal(107)));
    let mut reads = Vec::new();
    assert!(
        binary_float::validate(&value, &scalar_type("f64"), 0, &catalog, |leaf, _| {
            reads.push(match leaf {
                NirExpr::FieldAccess { field, .. } => field.clone(),
                NirExpr::Int(value) => value.to_string(),
                _ => "other".into(),
            });
            control_values::value_type(leaf, &frame, &catalog, &layouts)
        })
        .is_some()
    );
    assert_eq!(reads, ["wide", "flag", "107", "tag", "1"]);
    let root = call(
        "choose",
        vec![
            NirExpr::F64("1.0".into()),
            binary(
                NirBinaryOp::And,
                NirExpr::Bool(true),
                binary(NirBinaryOp::Eq, literal(1), literal(1)),
            ),
            NirExpr::Int(1),
        ],
    );
    for (depth, expected) in [(60, true), (61, false)] {
        assert_eq!(
            binary_float::validate(&root, &scalar_type("f64"), depth, &catalog, |leaf, _| {
                control_values::value_type(leaf, &frame, &catalog, &layouts)
            })
            .is_some(),
            expected
        );
    }
    let signed_root = call(
        "choose",
        vec![
            NirExpr::F64("1.0".into()),
            binary(
                NirBinaryOp::And,
                NirExpr::Bool(true),
                binary(NirBinaryOp::Eq, negative(1), literal(1)),
            ),
            NirExpr::Int(1),
        ],
    );
    for (depth, expected) in [(59, true), (60, false)] {
        assert_eq!(
            binary_float::validate(
                &signed_root,
                &scalar_type("f64"),
                depth,
                &catalog,
                |leaf, _| { control_values::value_type(leaf, &frame, &catalog, &layouts) }
            )
            .is_some(),
            expected
        );
    }
    assert!(
        binary_float::validate(&root, &scalar_type("f64"), 0, &catalog, |leaf, _| {
            if matches!(leaf, NirExpr::Int(_)) {
                Some(scalar_type("i32"))
            } else {
                control_values::value_type(leaf, &frame, &catalog, &layouts)
            }
        })
        .is_none()
    );
    for count in [32, 33] {
        assert_eq!(
            conditional_values::prefix::computed_call_expression_roots(
                vec![(&value, 0, false); count],
                &catalog
            ),
            count == 32
        );
    }
}

fn calls(events: &[String], callee: &str) -> Vec<String> {
    events
        .iter()
        .filter_map(|event| {
            event
                .split_once(&format!("] {callee}("))
                .map(|(_, args)| args.trim_end_matches(')').to_owned())
        })
        .collect()
}

#[test]
fn i32_literal_argument_print_epoch_source_preserves_signed_results_and_skipped_faults() {
    for ty in ["f32", "f64"] {
        for (op, token) in [
            (NirBinaryOp::Add, "+"),
            (NirBinaryOp::Sub, "-"),
            (NirBinaryOp::Mul, "*"),
            (NirBinaryOp::Div, "/"),
            (NirBinaryOp::Rem, "%"),
        ] {
            for (_, cmp) in COMPARISONS {
                for (logic, gate) in [("&&", false), ("&&", true), ("||", false), ("||", true)] {
                    for (constant, left) in [
                        (i32::MIN, 7),
                        (-1, i32::MIN),
                        (0, -7),
                        (1, i32::MAX),
                        (i32::MAX, -7),
                    ] {
                        for (first, second) in [(1, 1), (0, 1), (1, 0)] {
                            let text = program32(ty, gate, cmp, logic, first, second, left, -3).replace("relay32(checked32(live.inner.code,first),second)", &format!("relay32(checked32(live.inner.code,first){token}i32_from_i64({constant}),second)"));
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
                            let invalid = matches!(op, NirBinaryOp::Div | NirBinaryOp::Rem)
                                && (constant == 0 || (left == i32::MIN && constant == -1));
                            if entered && (invalid || first == 0 || second == 0) {
                                assert!(result.is_err());
                                continue;
                            }
                            let events = result.unwrap().events;
                            let mut expected = Vec::new();
                            let mut arguments = Vec::new();
                            let mut forwarded = Vec::new();
                            for code in [left, -3] {
                                if !entered {
                                    expected.push(gate);
                                    continue;
                                }
                                let a = i64::from(code);
                                let b = i64::from(constant);
                                let raw = match op {
                                    NirBinaryOp::Add => a + b,
                                    NirBinaryOp::Sub => a - b,
                                    NirBinaryOp::Mul => a * b,
                                    NirBinaryOp::Div => a / b,
                                    NirBinaryOp::Rem => a % b,
                                    _ => unreachable!(),
                                };
                                let word = raw & 0xffff_ffff;
                                let value = if word >= 0x8000_0000 {
                                    word - 0x1_0000_0000
                                } else {
                                    word
                                };
                                arguments.extend([
                                    format!("{code}i32, {first}"),
                                    format!("{value}i32, {second}"),
                                ]);
                                forwarded.push(format!("{value}i32, {second}"));
                                expected.push(match cmp {
                                    "==" => value == -3,
                                    "!=" => value != -3,
                                    "<" => value < -3,
                                    "<=" => value <= -3,
                                    ">" => value > -3,
                                    ">=" => value >= -3,
                                    _ => unreachable!(),
                                });
                            }
                            assert_eq!(calls(&events, "checked32"), arguments);
                            assert_eq!(calls(&events, "relay32"), forwarded);
                            let picks = calls(&events, "choose")
                                .iter()
                                .map(|args| {
                                    args.split(", ").nth(1).unwrap().parse::<bool>().unwrap()
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
fn i32_literal_argument_print_epoch_source_literal_helpers_keep_exact_arguments() {
    for ty in ["f32", "f64"] {
        for constant in [i64::from(i32::MIN) - 1, i64::from(i32::MAX) + 1] {
            let text = program32(ty, false, "!=", "&&", 1, 1, -1, 1).replace(
                "relay32(checked32(live.inner.code,first),second)",
                &format!("relay32(i32_from_i64({constant}),second)"),
            );
            let nir = crate::frontend::parse_nuis_module(&text).unwrap();
            assert!(crate::lowering::bootstrap::lower_nir_to_yir_builtin_cpu(&nir).is_err());
        }
        for gate in [false, true] {
            for constant in [i32::MIN, -1, 0, 1, 107, i32::MAX] {
                let text = program32(ty, gate, "!=", "&&", 1, 1, -1, 1).replace(
                    "relay32(checked32(live.inner.code,first),second)",
                    &format!("relay32(checked32(i32_from_i64({constant}),first),second)"),
                );
                let nir = crate::frontend::parse_nuis_module(&text).unwrap();
                let yir = crate::lowering::bootstrap::lower_nir_to_yir_builtin_cpu(&nir).unwrap();
                let events = yir_runtime_host::execute_module_source_with_registry(
                    &crate::render::render_yir(&yir),
                    &yir_verify::default_registry(),
                )
                .unwrap()
                .events;
                assert_eq!(
                    calls(&events, "checked32"),
                    if gate {
                        vec![format!("{constant}i32, 1"); 4]
                    } else {
                        vec![]
                    }
                );
                assert_eq!(
                    calls(&events, "relay32"),
                    if gate {
                        vec![format!("{constant}i32, 1"); 2]
                    } else {
                        vec![]
                    }
                );
            }
        }
    }
}
