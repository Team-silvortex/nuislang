use super::*;

fn neg(value: NirExpr) -> NirExpr {
    NirExpr::CastI64ToI32(Box::new(binary(
        NirBinaryOp::Sub,
        NirExpr::Int(0),
        NirExpr::CastI32ToI64(Box::new(value)),
    )))
}
fn literal(value: i64) -> NirExpr {
    NirExpr::CastI64ToI32(Box::new(NirExpr::Int(value)))
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

#[test]
fn i32_negation_argument_print_epoch_accepts_exact_lazy_comparison_operands() {
    for reversed in [false, true] {
        let mut module = i32_module();
        if reversed {
            module.functions.reverse();
        }
        for (cmp, _) in COMPARISONS {
            for (left, right) in [
                (neg(narrow(projected("tag"))), projected("tag")),
                (projected("tag"), neg(neg(projected("tag")))),
            ] {
                let value = in_rhs(binary(cmp, left, right));
                assert!(shape(&value, &module));
                assert!(check(&value, &module, false, true).is_some());
                assert!(check(&value, &module, true, true).is_some());
                assert!(check(&value, &module, true, false).is_none());
            }
        }
        assert!(check(
            &compared(narrow(neg(literal(i64::from(i32::MIN))))),
            &module,
            false,
            true
        )
        .is_some());
    }
}

#[test]
fn i32_negation_argument_print_epoch_rejects_forged_zero_casts_and_unproved_inputs() {
    let module = i32_module();
    for bad in [
        projected("total"),
        projected("small"),
        projected("wide"),
        projected("flag"),
        NirExpr::Var("missing".into()),
        NirExpr::Var("live".into()),
        call("transitive32", vec![projected("tag")]),
        call("cycle32a", vec![projected("tag")]),
        call("record32", vec![NirExpr::Var("live".into())]),
    ] {
        assert!(check(&compared(neg(bad)), &module, false, true).is_none());
    }
    for bad in [
        NirExpr::CastI64ToI32(Box::new(binary(
            NirBinaryOp::Sub,
            NirExpr::Int(1),
            NirExpr::CastI32ToI64(Box::new(projected("tag"))),
        ))),
        NirExpr::CastI64ToI32(Box::new(binary(
            NirBinaryOp::Sub,
            binary(NirBinaryOp::Sub, NirExpr::Int(1), NirExpr::Int(1)),
            NirExpr::CastI32ToI64(Box::new(projected("tag"))),
        ))),
        NirExpr::CastI64ToI32(Box::new(binary(
            NirBinaryOp::Add,
            NirExpr::Int(0),
            NirExpr::CastI32ToI64(Box::new(projected("tag"))),
        ))),
        NirExpr::CastI64ToI32(Box::new(binary(
            NirBinaryOp::Sub,
            NirExpr::Int(0),
            projected("tag"),
        ))),
        NirExpr::CastI64ToI32(Box::new(projected("total"))),
    ] {
        assert!(check(&compared(bad), &module, false, true).is_none());
    }
}

#[test]
fn i32_negation_argument_print_epoch_retains_comparison_only_scope_and_composition() {
    let module = i32_module();
    let value = neg(narrow(projected("tag")));
    assert!(check(
        &selector(binary(NirBinaryOp::Ne, value.clone(), projected("tag"))),
        &module,
        false,
        true
    )
    .is_none());
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
        binary(NirBinaryOp::Eq, value.clone(), projected("total")),
    ] {
        assert!(check(&in_rhs(escaped), &module, false, true).is_none());
    }
    let sum = NirExpr::CastI64ToI32(Box::new(binary(
        NirBinaryOp::Add,
        NirExpr::CastI32ToI64(Box::new(value.clone())),
        NirExpr::CastI32ToI64(Box::new(literal(1))),
    )));
    assert!(check(&compared(neg(sum)), &module, false, true).is_some());
    for op in [NirBinaryOp::Div, NirBinaryOp::Rem] {
        assert!(check(
            &compared(neg(binary(op, value.clone(), literal(1)))),
            &module,
            false,
            true
        )
        .is_some());
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
fn i32_negation_argument_print_epoch_bounds_all_four_nodes_depth_and_captures() {
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
        let value = float_argument(neg(call(&format!("batch{count}"), args)));
        assert_eq!(
            binary_float::validate(&value, &scalar_type("f64"), 0, &catalog, |leaf, _| {
                control_values::value_type(leaf, &frame, &catalog, &layouts)
            })
            .is_some(),
            count == 31
        );
    }
    let make = |value| {
        call(
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
        )
    };
    for calls in [0, 1] {
        // 2042 binary nodes + 2043 leaves + 4 negation nodes + 7 outer nodes = 4096.
        let mut value = neg(balanced(2042));
        for _ in 0..calls {
            value = narrow(value);
        }
        assert_eq!(
            binary_float::validate(&make(value), &scalar_type("f64"), 0, &catalog, |leaf, _| {
                control_values::value_type(leaf, &frame, &catalog, &layouts)
            })
            .is_some(),
            calls == 0,
            "4096/4097 work"
        );
    }
    let root = make(neg(NirExpr::Var("code".into())));
    for (depth, expected) in [(58, true), (59, false)] {
        assert_eq!(
            binary_float::validate(&root, &scalar_type("f64"), depth, &catalog, |leaf, _| {
                control_values::value_type(leaf, &frame, &catalog, &layouts)
            })
            .is_some(),
            expected
        );
    }
    let value = float_argument(neg(narrow(projected("tag"))));
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
    assert_eq!(reads, ["wide", "flag", "tag", "tag"]);
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
fn i32_negation_argument_print_epoch_source_wraps_once_and_suppresses_skipped_faults() {
    for ty in ["f32", "f64"] {
        for (_, cmp) in COMPARISONS {
            for (logic, gate) in [("&&", false), ("&&", true), ("||", false), ("||", true)] {
                for (left, right) in [
                    (i32::MIN, 1),
                    (i32::MAX, -1),
                    (-1, 0),
                    (0, -1),
                    (1, 1),
                    (107, 106),
                ] {
                    for (first, second) in [(1, 1), (0, 1), (1, 0)] {
                        let source = program32(ty, gate, cmp, logic, first, second, left, right)
                            .replace(
                                "relay32(checked32(live.inner.code,first),second)",
                                "relay32(-checked32(live.inner.code,first),second)",
                            );
                        let nir = crate::frontend::parse_nuis_module(&source).unwrap();
                        let yir =
                            crate::lowering::bootstrap::lower_nir_to_yir_builtin_cpu(&nir).unwrap();
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
                        let mut args = Vec::new();
                        let mut forwarded = Vec::new();
                        for code in [left, right] {
                            if !entered {
                                expected.push(gate);
                                continue;
                            }
                            let word = (-i64::from(code)) & 0xffff_ffff;
                            let value = if word >= 0x8000_0000 {
                                word - 0x1_0000_0000
                            } else {
                                word
                            };
                            args.extend([
                                format!("{code}i32, {first}"),
                                format!("{value}i32, {second}"),
                            ]);
                            forwarded.push(format!("{value}i32, {second}"));
                            let b = i64::from(right);
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
                        assert_eq!(calls(&events, "checked32"), args);
                        assert_eq!(calls(&events, "relay32"), forwarded);
                        let picks = calls(&events, "choose")
                            .iter()
                            .map(|args| args.split(", ").nth(1).unwrap().parse::<bool>().unwrap())
                            .collect::<Vec<_>>();
                        assert_eq!(picks, expected);
                        assert_eq!(prints(&events).len(), 2);
                    }
                }
            }
        }
    }
}

#[test]
fn i32_negation_argument_print_epoch_source_composes_checked_division_and_minimum_wrapping() {
    for ty in ["f32", "f64"] {
        for token in ["/", "%"] {
            for gate in [false, true] {
                for (left, right) in [(i32::MIN, 1), (i32::MIN, -1), (7, 0), (-7, 3)] {
                    let source = program32(ty, gate, "!=", "&&", 1, 1, left, right).replace(
                        "relay32(checked32(live.inner.code,first),second)",
                        &format!("relay32(-(checked32(live.inner.code,first){token}right),second)"),
                    );
                    let nir = crate::frontend::parse_nuis_module(&source).unwrap();
                    let yir =
                        crate::lowering::bootstrap::lower_nir_to_yir_builtin_cpu(&nir).unwrap();
                    let result = yir_runtime_host::execute_module_source_with_registry(
                        &crate::render::render_yir(&yir),
                        &yir_verify::default_registry(),
                    );
                    assert_eq!(
                        result.is_err(),
                        gate && (right == 0 || (left == i32::MIN && right == -1))
                    );
                }
            }
        }
    }
}
