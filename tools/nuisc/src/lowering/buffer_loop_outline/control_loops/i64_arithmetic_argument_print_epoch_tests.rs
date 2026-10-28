use super::*;

const ARITHMETIC: [(NirBinaryOp, &str); 5] = [
    (NirBinaryOp::Add, "+"),
    (NirBinaryOp::Sub, "-"),
    (NirBinaryOp::Mul, "*"),
    (NirBinaryOp::Div, "/"),
    (NirBinaryOp::Rem, "%"),
];

#[test]
fn i64_arithmetic_argument_print_epoch_requires_exact_lazy_comparison_reads() {
    for reversed in [false, true] {
        let mut module = module_wide();
        if reversed {
            module.functions.reverse();
        }
        for (op, _) in ARITHMETIC {
            for (cmp, _) in COMPARISONS {
                for (left, right) in [
                    (wide(narrow(projected("tag"))), projected("total")),
                    (projected("total"), wide(projected("tag"))),
                ] {
                    let value = in_rhs(binary(cmp, binary(op, left, right), NirExpr::Int(0)));
                    assert!(shape(&value, &module));
                    assert!(check(&value, &module, false, true).is_some());
                    assert!(check(&value, &module, true, true).is_some());
                    assert!(check(&value, &module, true, false).is_none());
                }
            }
        }
    }
}

#[test]
fn i64_arithmetic_argument_print_epoch_rejects_kind_scope_and_effect_escapes() {
    let module = module_wide();
    for (op, _) in ARITHMETIC {
        for bad in [
            projected("tag"),
            projected("flag"),
            projected("small"),
            projected("wide"),
            NirExpr::Var("missing".into()),
            NirExpr::Var("live".into()),
            call("effect", vec![projected("total")]),
            call("transitive32", vec![projected("tag")]),
            call("cycle32a", vec![projected("tag")]),
        ] {
            assert!(check(
                &compared(binary(op, wide(projected("tag")), bad)),
                &module,
                false,
                true
            )
            .is_none());
        }
        let value = binary(op, wide(projected("tag")), projected("total"));
        assert!(check(
            &selector(binary(NirBinaryOp::Ne, value.clone(), NirExpr::Int(0))),
            &module,
            false,
            true
        )
        .is_none());
        assert!(check(
            &in_rhs(NirExpr::CastI64ToI32(Box::new(value.clone()))),
            &module,
            false,
            true
        )
        .is_none());
        assert!(check(
            &compared(binary(NirBinaryOp::Xor, value, NirExpr::Int(1))),
            &module,
            false,
            true
        )
        .is_none());
    }
}

#[test]
fn i64_arithmetic_argument_print_epoch_preserves_restriction_through_operations_and_calls() {
    let module = module_wide();
    let mut value = wide(narrow(projected("tag")));
    for (op, _) in ARITHMETIC {
        value = binary(op, call("keep64", vec![value]), NirExpr::Int(1));
    }
    assert!(check(
        &compared(call("keep64", vec![value.clone()])),
        &module,
        false,
        true
    )
    .is_some());
    for name in ["bool64", "float64", "narrow64"] {
        assert!(check(
            &in_rhs(call(name, vec![value.clone()])),
            &module,
            false,
            true
        )
        .is_none());
    }
    assert!(check(
        &in_rhs(binary(
            NirBinaryOp::Ne,
            call("mixed", vec![narrow(projected("tag")), value]),
            NirExpr::Int(0)
        )),
        &module,
        false,
        true
    )
    .is_none());
}

fn balanced64(operations: usize, widened: bool) -> NirExpr {
    if operations == 0 {
        return if widened {
            wide(NirExpr::Var("code".into()))
        } else {
            NirExpr::Int(1)
        };
    }
    let left = (operations - 1) / 2;
    binary(
        NirBinaryOp::Add,
        balanced64(left, widened),
        balanced64(operations - 1 - left, false),
    )
}

#[test]
fn i64_arithmetic_argument_print_epoch_charges_full_tree_depth_and_capture_union() {
    let mut module = module_wide();
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
    let make = |value| {
        call(
            "choose",
            vec![
                NirExpr::F64("1.0".into()),
                binary(
                    NirBinaryOp::And,
                    NirExpr::Bool(true),
                    binary(NirBinaryOp::Eq, value, NirExpr::Int(0)),
                ),
                NirExpr::Int(1),
            ],
        )
    };
    let prove = |value: &NirExpr, depth| {
        binary_float::validate(value, &scalar_type("f64"), depth, &catalog, |leaf, _| {
            control_values::value_type(leaf, &frame, &catalog, &layouts)
        })
        .is_some()
    };
    for calls in [1, 2] {
        // 2043 i64 operations + 2044 leaves + one extension + seven outer nodes + calls.
        let mut value = balanced64(2043, true);
        for _ in 0..calls {
            value = call("keep64", vec![value]);
        }
        assert_eq!(prove(&make(value), 0), calls == 1, "4096/4097 work");
    }
    let root = make(binary(
        NirBinaryOp::Add,
        wide(NirExpr::Var("code".into())),
        NirExpr::Int(1),
    ));
    for (depth, expected) in [(59, true), (60, false)] {
        assert_eq!(prove(&root, depth), expected);
    }
    for count in [31, 32] {
        let mut args = vec![projected("flag"), projected("tag")];
        args.extend((0..count - 2).map(|i| NirExpr::Var(format!("v{i}"))));
        let value = make(binary(
            NirBinaryOp::Mul,
            wide(call(&format!("batch{count}"), args)),
            NirExpr::Int(3),
        ));
        assert_eq!(prove(&value, 0), count == 31);
    }
    let value = float_argument(binary(
        NirBinaryOp::Sub,
        wide(narrow(projected("tag"))),
        projected("total"),
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
    assert_eq!(reads, ["wide", "flag", "tag", "total", "total"]);
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

// A mathematical signed modulo oracle independent of backend wrapping instructions.
fn wrap64(value: i128) -> i64 {
    let bits = value.rem_euclid(1i128 << 64);
    (if bits >= 1i128 << 63 {
        bits - (1i128 << 64)
    } else {
        bits
    }) as i64
}
fn arithmetic64(left: i64, right: i64, op: &str) -> Option<i64> {
    if matches!(op, "/" | "%") && (right == 0 || (left == i64::MIN && right == -1)) {
        return None;
    }
    Some(match op {
        "+" => wrap64(i128::from(left) + i128::from(right)),
        "-" => wrap64(i128::from(left) - i128::from(right)),
        "*" => wrap64(i128::from(left) * i128::from(right)),
        "/" => (i128::from(left) / i128::from(right)) as i64,
        "%" => (i128::from(left) % i128::from(right)) as i64,
        _ => unreachable!(),
    })
}
fn source_arithmetic(
    ty: &str,
    gate: bool,
    cmp: &str,
    logic: &str,
    first: i64,
    second: i64,
    left: i32,
    right: i32,
    expression: &str,
) -> String {
    source_wide(
        ty,
        gate,
        cmp,
        logic,
        first,
        second,
        left,
        right,
        "checked32(live.inner.code,first)",
    )
    .replace("i64_from_i32(checked32(live.inner.code,first))", expression)
}
fn execute_source(source: &str) -> Result<Vec<String>, String> {
    let nir = crate::frontend::parse_nuis_module(source).unwrap();
    let yir = crate::lowering::bootstrap::lower_nir_to_yir_builtin_cpu(&nir).unwrap();
    let llvm = yir_lower_llvm::emit_module(&yir).unwrap();
    assert!(llvm.contains("sext i32"));
    yir_runtime_host::execute_module_source_with_registry(
        &crate::render::render_yir(&yir),
        &yir_verify::default_registry(),
    )
    .map(|r| r.events)
}

#[test]
fn i64_arithmetic_argument_print_epoch_source_wraps_and_keeps_exact_helper_words() {
    for ty in ["f32", "f64"] {
        for (_, op) in ARITHMETIC {
            for (i, (logic, gate, left, right, offset)) in [
                ("&&", true, i32::MIN, -1, i64::MAX),
                ("||", false, i32::MAX, 1, i64::MAX),
                ("&&", false, -7, 0, i64::MAX),
                ("||", true, -7, 0, i64::MAX),
                ("&&", true, -1, 1, i64::MIN + 1),
                ("||", false, 7, -3, -11),
            ]
            .into_iter()
            .enumerate()
            {
                let cmp = COMPARISONS[i].1;
                let expression = format!("((i64_from_i32(checked32(live.inner.code,first))+({offset})){op}i64_from_i32(right))");
                let events = execute_source(&source_arithmetic(
                    ty,
                    gate,
                    cmp,
                    logic,
                    1,
                    1,
                    left,
                    right,
                    &expression,
                ))
                .unwrap();
                let entered = if logic == "&&" { gate } else { !gate };
                let expected = if entered {
                    [left, right]
                        .into_iter()
                        .map(|code| {
                            let base = wrap64(i128::from(code) + i128::from(offset));
                            arithmetic64(base, i64::from(right), op).unwrap()
                        })
                        .collect::<Vec<_>>()
                } else {
                    vec![]
                };
                let actual = expected
                    .iter()
                    .map(|value| format!("{value}, 1"))
                    .collect::<Vec<_>>();
                assert_eq!(calls(&events, "relay64"), actual);
                assert_eq!(calls(&events, "checked64"), actual);
                assert_eq!(calls(&events, "checked32").len(), expected.len());
                let picks = if entered {
                    expected
                        .iter()
                        .map(|value| match cmp {
                            "==" => *value == i64::from(right),
                            "!=" => *value != i64::from(right),
                            "<" => *value < i64::from(right),
                            "<=" => *value <= i64::from(right),
                            ">" => *value > i64::from(right),
                            ">=" => *value >= i64::from(right),
                            _ => unreachable!(),
                        })
                        .collect::<Vec<_>>()
                } else {
                    vec![gate; 2]
                };
                let actual_picks = calls(&events, "choose")
                    .iter()
                    .map(|args| args.split(", ").nth(1).unwrap().parse::<bool>().unwrap())
                    .collect::<Vec<_>>();
                assert_eq!(actual_picks, picks);
                assert_eq!(prints(&events), vec![if picks[0] { -1 } else { 1 }, 0]);
                assert_eq!(prints(&events).len(), 2);
            }
        }
    }
}

#[test]
fn i64_arithmetic_argument_print_epoch_source_checks_div_rem_before_forwarding() {
    for ty in ["f32", "f64"] {
        for op in ["/", "%"] {
            for (left, right, first, second, gate) in [
                (2, -1, 1, 1, true),
                (7, 0, 1, 1, true),
                (2, -1, 1, 1, false),
                (7, 0, 1, 1, false),
                (1, 1, 0, 1, true),
                (1, 1, 1, 0, true),
                (1, 1, 0, 1, false),
                (1, 1, 1, 0, false),
            ] {
                let expression=format!("((i64_from_i32(checked32(live.inner.code,first))*4611686018427387904){op}i64_from_i32(right))");
                let result = execute_source(&source_arithmetic(
                    ty,
                    gate,
                    "!=",
                    "&&",
                    first,
                    second,
                    left,
                    right,
                    &expression,
                ));
                assert_eq!(result.is_err(), gate);
                if let Ok(events) = result {
                    assert!(calls(&events, "checked32").is_empty());
                    assert!(calls(&events, "relay64").is_empty());
                    assert_eq!(calls(&events, "choose").len(), 2);
                }
            }
        }
    }
}
