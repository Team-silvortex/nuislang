use super::*;

fn xor_value(op: NirBinaryOp) -> NirExpr {
    binary(op, logical_flag(), positive())
}

#[test]
fn bool_xor_argument_print_epoch_keeps_exact_types_epochs_and_profiles() {
    for reversed in [false, true] {
        let mut module = i32_module();
        if reversed {
            module.functions.reverse();
        }
        for op in [NirBinaryOp::Xor] {
            let predicate = xor_value(op);
            let value = in_rhs(predicate.clone());
            assert!(shape(&value, &module));
            assert!(check(&value, &module, false, true).is_some());
            assert!(check(&value, &module, true, true).is_some());
            assert!(check(&value, &module, true, false).is_none());
            assert!(!conditional_values::prefix::expression(&predicate));
            assert!(!conditional_values::prefix::computed_logical_root(
                &predicate
            ));
            // XOR must not grant its RHS the call permission of AND/OR.
            assert!(check(&selector(predicate), &module, false, true).is_none());
            let logical = binary(op, logical_flag(), logical_flag());
            assert!(shape(&selector(logical), &module));
        }
    }
}

#[test]
fn bool_xor_argument_print_epoch_rejects_mixed_effect_missing_and_provenance_escapes() {
    let module = i32_module();
    for op in [NirBinaryOp::Xor] {
        for bad in [
            projected("total"),
            projected("small"),
            projected("wide"),
            narrow(projected("tag")),
            NirExpr::Var("missing".into()),
            NirExpr::Var("live".into()),
            call("transitive32", vec![projected("tag")]),
            call("record_bool", vec![NirExpr::Var("live".into())]),
            call("missing", vec![]),
        ] {
            for value in [
                binary(op, logical_flag(), bad.clone()),
                binary(op, bad, logical_flag()),
            ] {
                assert!(check(&in_rhs(value), &module, false, true).is_none());
            }
        }
        let comparison = binary(op, logical_flag(), logical_flag());
        let consumed = admitted32(comparison);
        assert!(shape(&consumed, &module));
        assert!(check(&consumed, &module, false, true).is_some());
        let escaped = call(
            "all",
            vec![
                NirExpr::Bool(true),
                argument32(logical_flag()),
                projected("total"),
                projected("small"),
                projected("wide"),
            ],
        );
        assert!(check(
            &in_rhs(binary(op, logical_flag(), escaped)),
            &module,
            false,
            true
        )
        .is_none());
    }
}

#[test]
fn bool_xor_argument_print_epoch_visits_both_operands_in_source_order() {
    let module = i32_module();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    for op in [NirBinaryOp::Xor] {
        let value = call(
            "choose",
            vec![
                projected("wide"),
                binary(NirBinaryOp::And, projected("flag"), xor_value(op)),
                NirExpr::Int(1),
            ],
        );
        let mut reads = Vec::new();
        assert!(
            binary_float::validate(&value, &scalar_type("f64"), 0, &catalog, |leaf, _| {
                if let NirExpr::FieldAccess { field, .. } = leaf {
                    reads.push(field.clone());
                }
                control_values::value_type(leaf, &scope(), &catalog, &layouts)
            })
            .is_some()
        );
        assert_eq!(reads, ["wide", "flag", "flag", "total", "total"]);
    }
}

#[test]
fn bool_xor_argument_print_epoch_checks_rhs_even_after_constant_lhs() {
    let module = i32_module();
    let layouts = control_values::layouts(&module);
    for op in [NirBinaryOp::Xor] {
        for lhs in [false, true] {
            let comparison = binary(op, NirExpr::Bool(lhs), logical_flag());
            let value = in_rhs(comparison.clone());
            assert!(check(&value, &module, false, true).is_some());
            assert!(check(&value, &module, true, false).is_none());
            let record = NirExpr::StructLiteral {
                type_name: "Pair".into(),
                type_args: vec![],
                fields: vec![("flag".into(), comparison)],
            };
            assert!(
                !conditional_values::prefix::computed_record_expression_roots(
                    vec![(&record, 0, true)],
                    &layouts
                )
            );
            for rhs in [NirExpr::Var("missing".into()), call("missing", vec![])] {
                assert!(check(
                    &in_rhs(binary(op, NirExpr::Bool(lhs), rhs)),
                    &module,
                    false,
                    true
                )
                .is_none());
            }
        }
    }
}

#[test]
fn bool_xor_argument_print_epoch_bounds_shared_edges_work_depth_and_capture_union() {
    let mut module = i32_module();
    for count in [31, 32] {
        let params = (0..count - 2)
            .map(|i| format!("p{i}:i64"))
            .collect::<Vec<_>>()
            .join(",");
        module.functions.extend(
            crate::frontend::parse_nuis_module(&format!(
                "mod cpu Batch{{fn batch{count}({params})->bool{{return true;}}}}"
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
        let args = (0..count - 2)
            .map(|i| NirExpr::Var(format!("v{i}")))
            .collect();
        let value = call(
            "choose",
            vec![
                projected("wide"),
                binary(
                    NirBinaryOp::And,
                    projected("flag"),
                    binary(
                        NirBinaryOp::Xor,
                        logical_flag(),
                        call(&format!("batch{count}"), args),
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
        let mut tree = projected("flag");
        for _ in 0..count {
            tree = binary(NirBinaryOp::And, tree, projected("flag"));
        }
        let value = in_rhs(binary(NirBinaryOp::Xor, tree, projected("flag")));
        assert_eq!(shape(&value, &module), count == 31);
    }
    let value = in_rhs(xor_value(NirBinaryOp::Xor));
    assert!(!conditional_values::prefix::computed_call_expression_roots(
        vec![(&value, 64, false)],
        &catalog
    ));
    for count in [4096, 4097] {
        let leaf = NirExpr::Bool(false);
        assert_eq!(
            conditional_values::prefix::computed_call_expression_roots(
                vec![(&leaf, 0, false); count],
                &catalog
            ),
            count == 4096
        );
    }
}

#[test]
fn bool_xor_argument_print_epoch_source_preserves_eager_operands_and_nested_guards() {
    for ty in ["f32", "f64"] {
        for op in ["^"] {
            for (outer, gate) in [("&&", false), ("&&", true), ("||", false), ("||", true)] {
                for left_logic in ["&&", "||"] {
                    for right_logic in ["&&", "||"] {
                        for (left, right) in [(i32::MIN, i32::MAX), (i32::MAX, i32::MIN)] {
                            for (first, second) in [(1, 1), (0, 1), (1, 0), (0, 0)] {
                                let predicate=format!("(((live.inner.code<right){left_logic}(checked32(live.inner.code,first)!=right)){op}((live.inner.code>left){right_logic}(checked32(right,second)<=live.inner.code)))");
                                let text =
                                    program32(ty, gate, "!=", outer, first, second, left, right)
                                        .replace(
                                        "(relay32(checked32(live.inner.code,first),second)!=right)",
                                        &predicate,
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
                                let entered = if outer == "&&" { gate } else { !gate };
                                let mut failed = false;
                                let mut expected = Vec::new();
                                let mut arguments = Vec::new();
                                for code in [left, right] {
                                    if !entered {
                                        expected.push(gate);
                                        continue;
                                    }
                                    let mut side =
                                        |lhs: bool,
                                         logic: &str,
                                         value: i32,
                                         guard: i64,
                                         rhs: bool|
                                         -> Option<bool> {
                                            if (logic == "&&" && lhs) || (logic == "||" && !lhs) {
                                                arguments.push(format!("{value}i32, {guard}"));
                                                if guard == 0 {
                                                    return None;
                                                }
                                                Some(rhs)
                                            } else {
                                                Some(lhs)
                                            }
                                        };
                                    let Some(a) = side(
                                        i64::from(code) < i64::from(right),
                                        left_logic,
                                        code,
                                        first,
                                        code != right,
                                    ) else {
                                        failed = true;
                                        break;
                                    };
                                    let Some(b) = side(
                                        i64::from(code) > i64::from(left),
                                        right_logic,
                                        right,
                                        second,
                                        i64::from(right) <= i64::from(code),
                                    ) else {
                                        failed = true;
                                        break;
                                    };
                                    expected.push(a ^ b);
                                }
                                if failed {
                                    assert!(result.is_err(),"{ty} {outer} {gate} {op} {left_logic} {right_logic} {left} {right} {first} {second}");
                                    continue;
                                }
                                let events = result.unwrap().events;
                                let actual = events
                                    .iter()
                                    .filter_map(|e| {
                                        e.split_once("] checked32(")
                                            .map(|(_, a)| a.trim_end_matches(')').to_owned())
                                    })
                                    .collect::<Vec<_>>();
                                assert_eq!(actual, arguments);
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
}
