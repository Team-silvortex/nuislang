use super::*;

const COMPARISONS: [(NirBinaryOp, &str); 6] = [
    (NirBinaryOp::Eq, "=="),
    (NirBinaryOp::Ne, "!="),
    (NirBinaryOp::Lt, "<"),
    (NirBinaryOp::Le, "<="),
    (NirBinaryOp::Gt, ">"),
    (NirBinaryOp::Ge, ">="),
];

#[test]
fn float_compare_argument_print_epoch_proves_six_exact_bool_results_and_scoped_reads() {
    let module = module();
    for field in ["small", "wide"] {
        for (op, _) in COMPARISONS {
            let comparison = binary(op, projected(field), projected(field));
            for flag in [
                comparison.clone(),
                binary(NirBinaryOp::And, projected("flag"), comparison.clone()),
                binary(
                    NirBinaryOp::Or,
                    projected("flag"),
                    auxiliary("accept", comparison),
                ),
            ] {
                let value = selector(flag);
                assert!(shape(&value, &module));
                assert_eq!(
                    check(&value, &module, false, true),
                    Some(scalar_type("i64"))
                );
                assert!(check(&value, &module, true, true).is_some());
                assert!(check(&value, &module, true, false).is_none());
            }
        }
    }
}

#[test]
fn float_compare_argument_print_epoch_rejects_mixed_cast_effect_and_missing_operands() {
    let module = module();
    for (op, _) in COMPARISONS {
        for bad in [
            projected("small"),
            projected("total"),
            NirExpr::Bool(false),
            projected("missing"),
            NirExpr::Var("live".into()),
            call("effect", vec![projected("wide")]),
            call("missing", vec![]),
            NirExpr::CastI64ToF64(Box::new(projected("total"))),
        ] {
            for comparison in [
                binary(op, projected("wide"), bad.clone()),
                binary(op, bad, projected("wide")),
            ] {
                let value = selector(binary(NirBinaryOp::And, NirExpr::Bool(false), comparison));
                assert!(check(&value, &module, false, true).is_none());
            }
        }
        let value = call(
            "observe",
            vec![call(
                "choose",
                vec![
                    binary(op, projected("wide"), projected("wide")),
                    NirExpr::Bool(true),
                    NirExpr::Int(1),
                ],
            )],
        );
        assert!(check(&value, &module, false, true).is_none());
    }
}

#[test]
fn float_compare_argument_print_epoch_bounds_complete_capture_union_and_original_order() {
    let module = module();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    let mut frame = scope();
    for i in 0..31 {
        frame.insert(format!("v{i}"), scalar_type("f64"));
    }
    for count in [31, 32] {
        let mut sum = projected("wide");
        for i in 0..count - 1 {
            sum = binary(NirBinaryOp::Add, sum, NirExpr::Var(format!("v{i}")));
        }
        let value = call(
            "choose",
            vec![
                projected("wide"),
                binary(
                    NirBinaryOp::And,
                    projected("flag"),
                    binary(NirBinaryOp::Ne, call("id64", vec![sum]), projected("wide")),
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
    let value = call(
        "choose",
        vec![
            projected("wide"),
            binary(
                NirBinaryOp::Lt,
                call("id64", vec![projected("wide")]),
                projected("wide"),
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
    assert_eq!(reads, ["wide", "wide", "wide"]);
    assert!(
        binary_float::validate(&value, &scalar_type("f64"), 64, &catalog, |leaf, _| {
            control_values::value_type(leaf, &frame, &catalog, &layouts)
        })
        .is_none()
    );
}

#[test]
fn float_compare_argument_print_epoch_distinguishes_total_values_from_helper_free_predicates() {
    let module = module();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    let mut frame = scope();
    for i in 0..32 {
        frame.insert(format!("v{i}"), scalar_type("f64"));
    }
    let valid = |value: &NirExpr| {
        binary_float::validate(value, &scalar_type("f64"), 0, &catalog, |leaf, _| {
            control_values::value_type(leaf, &frame, &catalog, &layouts)
        })
        .is_some()
    };
    for count in [31, 32] {
        let mut sum = projected("wide");
        for i in 0..count - 1 {
            sum = binary(NirBinaryOp::Add, sum, NirExpr::Var(format!("v{i}")));
        }
        let value = call(
            "choose",
            vec![
                projected("wide"),
                binary(
                    NirBinaryOp::And,
                    projected("flag"),
                    binary(NirBinaryOp::Le, sum, projected("wide")),
                ),
                NirExpr::Int(1),
            ],
        );
        assert_eq!(valid(&value), count == 31);
    }
    let mut predicates = (0..16)
        .map(|i| {
            binary(
                NirBinaryOp::Ne,
                NirExpr::Var(format!("v{}", i * 2)),
                NirExpr::Var(format!("v{}", i * 2 + 1)),
            )
        })
        .collect::<Vec<_>>();
    while predicates.len() > 1 {
        predicates = predicates
            .chunks(2)
            .map(|pair| binary(NirBinaryOp::And, pair[0].clone(), pair[1].clone()))
            .collect();
    }
    let rhs = predicates.pop().unwrap();
    assert!(ready_values::total_predicate(&rhs, &frame));
    let value = call(
        "choose",
        vec![
            projected("wide"),
            binary(NirBinaryOp::And, projected("flag"), rhs),
            NirExpr::Int(1),
        ],
    );
    assert!(valid(&value));
}

fn source(ty: &str, op: &str, gate: bool, divisor: i64) -> String {
    let text = program(ty, gate, divisor, 1, false, false).replace(
        "(live.inner.tag/first>=50&&live.inner.flag)||live.inner.tag==99",
        &format!("live.inner.flag&&(sample(live.inner.value,first){op}sample(seed,second))"),
    );
    text.replace("@noinline fn walk",&format!("@noinline fn sample(value:{ty},guard:i64)->{ty}{{let checked=0/guard;return value;}}@noinline fn walk")).replace("observe(choose(","observe(-choose(")
}

#[test]
fn float_compare_argument_print_epoch_raw_ieee_results_keep_operand_calls_once_only() {
    for ty in ["f32", "f64"] {
        let words: &[u64] = if ty == "f32" {
            &[
                0, 0x80000000, 1, 0x3fc00000, 0xc0200000, 0x7f800000, 0xff800000, 0x7fc12345,
                0xff800001,
            ]
        } else {
            &[
                0,
                0x8000000000000000,
                1,
                0x3ff8000000000000,
                0xc004000000000000,
                0x7ff0000000000000,
                0xfff0000000000000,
                0x7ff812345678abcd,
                0xfff0000000000001,
            ]
        };
        for (_, op) in COMPARISONS {
            for &bits in words {
                let mut module =
                    crate::frontend::parse_nuis_module(&source(ty, op, true, 1)).unwrap();
                let main = module
                    .functions
                    .iter_mut()
                    .find(|f| f.name == "main")
                    .unwrap();
                let NirStmt::Return(Some(NirExpr::Call { args, .. })) = &mut main.body[0] else {
                    panic!()
                };
                args[0] = if ty == "f32" {
                    NirExpr::UnpackF32Word(Box::new(NirExpr::Int(bits as i64)))
                } else {
                    NirExpr::UnpackF64Word(Box::new(NirExpr::Int(bits as i64)))
                };
                crate::nir_verify::verify_nir_module(&module).unwrap();
                let yir =
                    crate::lowering::bootstrap::lower_nir_to_yir_builtin_cpu(&module).unwrap();
                let events = yir_runtime_host::execute_module_source_with_registry(
                    &crate::render::render_yir(&yir),
                    &yir_verify::default_registry(),
                )
                .unwrap()
                .events;
                let seed = if ty == "f32" {
                    f64::from(f32::from_bits(bits as u32))
                } else {
                    f64::from_bits(bits)
                };
                let mut expected = Vec::new();
                let mut expected_picks = Vec::new();
                for negative in [false, true] {
                    let current = if negative { -seed } else { seed };
                    let pick = match op {
                        "==" => current == seed,
                        "!=" => current != seed,
                        "<" => current < seed,
                        "<=" => current <= seed,
                        ">" => current > seed,
                        ">=" => current >= seed,
                        _ => unreachable!(),
                    };
                    expected_picks.push(pick);
                    let sum = if ty == "f32" {
                        f64::from((current as f32) + (seed as f32))
                    } else {
                        current + seed
                    };
                    let observed = -(if pick { sum } else { -sum });
                    expected.push(if observed < 0.0 {
                        -1
                    } else if observed == 0.0 {
                        0
                    } else if observed.is_nan() {
                        2
                    } else {
                        1
                    });
                }
                assert_eq!(prints(&events), expected, "{ty} {op} {bits:x}");
                let actual_picks = events
                    .iter()
                    .filter_map(|event| {
                        event.split_once("] choose(").map(|(_, args)| {
                            args.split(", ").nth(1).unwrap().parse::<bool>().unwrap()
                        })
                    })
                    .collect::<Vec<_>>();
                assert_eq!(actual_picks, expected_picks, "{ty} {op} {bits:x}");
                assert_eq!(
                    events
                        .iter()
                        .filter(|e| e.contains("cpu.call") && e.contains("] sample("))
                        .count(),
                    4
                );
                yir_lower_llvm::emit_module(&yir).unwrap();
            }
        }
    }
}

#[test]
fn float_compare_argument_print_epoch_reaches_only_selected_operand_faults() {
    for ty in ["f32", "f64"] {
        for (_, op) in COMPARISONS {
            for gate in [false, true] {
                for (first, second) in [(0, 1), (1, 0), (0, 0)] {
                    let text = source(ty, op, gate, first).replace(
                        &format!(",{gate},{first},1)"),
                        &format!(",{gate},{first},{second})"),
                    );
                    // The selector itself stays healthy so only operand checks can fail.
                    let text = text.replace(
                        "second+(live.inner.tag-live.inner.tag)",
                        "1+(live.inner.tag-live.inner.tag)",
                    );
                    let module = crate::frontend::parse_nuis_module(&text).unwrap();
                    let yir =
                        crate::lowering::bootstrap::lower_nir_to_yir_builtin_cpu(&module).unwrap();
                    let result = yir_runtime_host::execute_module_source_with_registry(
                        &crate::render::render_yir(&yir),
                        &yir_verify::default_registry(),
                    );
                    if gate {
                        assert!(result.is_err(), "{ty} {op} {first} {second}");
                    } else {
                        assert_eq!(prints(&result.unwrap().events).len(), 2);
                    }
                }
            }
        }
    }
}

#[path = "i32_call_argument_print_epoch_tests.rs"]
mod i32_calls;
