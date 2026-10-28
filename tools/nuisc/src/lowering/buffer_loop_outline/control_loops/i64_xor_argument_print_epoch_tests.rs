use super::*;

fn xor64(left: NirExpr, right: NirExpr) -> NirExpr {
    binary(NirBinaryOp::Xor, left, right)
}

#[test]
fn i64_xor_argument_print_epoch_requires_exact_lazy_comparison_reads() {
    for reversed in [false, true] {
        let mut module = module_wide();
        if reversed {
            module.functions.reverse();
        }
        for (cmp, _) in COMPARISONS {
            for value in [
                xor64(wide(narrow(projected("tag"))), projected("total")),
                xor64(projected("total"), wide(projected("tag"))),
                xor64(
                    binary(NirBinaryOp::Add, wide(projected("tag")), NirExpr::Int(1)),
                    call("keep64", vec![wide(projected("tag"))]),
                ),
            ] {
                let value = in_rhs(binary(cmp, value, NirExpr::Int(0)));
                assert!(shape(&value, &module));
                assert!(check(&value, &module, false, true).is_some());
                assert!(check(&value, &module, true, true).is_some());
                assert!(check(&value, &module, true, false).is_none());
            }
        }
    }
}

#[test]
fn i64_xor_argument_print_epoch_rejects_kind_scope_and_effect_escapes() {
    let module = module_wide();
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
        for (left, right) in [
            (wide(projected("tag")), bad.clone()),
            (bad, wide(projected("tag"))),
        ] {
            assert!(check(&compared(xor64(left, right)), &module, false, true).is_none());
        }
    }
    for value in [
        xor64(NirExpr::Int(0), wide(projected("tag"))),
        xor64(
            wide(projected("tag")),
            call("keep64", vec![projected("total")]),
        ),
    ] {
        // XOR inherits a lazy context; it must not create one for either operand.
        assert!(check(
            &selector(binary(NirBinaryOp::Ne, value.clone(), NirExpr::Int(0))),
            &module,
            false,
            true
        )
        .is_none());
        assert!(check(&in_rhs(value.clone()), &module, false, true).is_none());
        assert!(check(
            &in_rhs(NirExpr::CastI64ToI32(Box::new(value))),
            &module,
            false,
            true
        )
        .is_none());
    }
}

#[test]
fn i64_xor_argument_print_epoch_preserves_restriction_through_arithmetic_and_helpers() {
    let module = module_wide();
    let mut value = wide(narrow(projected("tag")));
    for (op, _) in ARITHMETIC {
        value = xor64(
            call("keep64", vec![binary(op, value, NirExpr::Int(1))]),
            NirExpr::Int(-1),
        );
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
        &compared(call("mixed", vec![narrow(projected("tag")), value])),
        &module,
        false,
        true
    )
    .is_none());
}

fn balanced_xor(operations: usize, widened: bool) -> NirExpr {
    if operations == 0 {
        return if widened {
            wide(NirExpr::Var("code".into()))
        } else {
            NirExpr::Int(1)
        };
    }
    let left = (operations - 1) / 2;
    xor64(
        balanced_xor(left, widened),
        balanced_xor(operations - 1 - left, false),
    )
}

#[test]
fn i64_xor_argument_print_epoch_charges_both_operands_work_depth_and_captures() {
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
        // 2043 eager XOR operations + 2044 leaves + one extension + seven outer nodes + calls.
        let mut value = balanced_xor(2043, true);
        for _ in 0..calls {
            value = call("keep64", vec![value]);
        }
        assert_eq!(prove(&make(value), 0), calls == 1, "4096/4097 work");
    }
    let root = make(xor64(wide(NirExpr::Var("code".into())), NirExpr::Int(0)));
    for (depth, expected) in [(59, true), (60, false)] {
        assert_eq!(prove(&root, depth), expected);
    }
    for count in [31, 32] {
        let mut args = vec![projected("flag"), projected("tag")];
        args.extend((0..count - 2).map(|i| NirExpr::Var(format!("v{i}"))));
        assert_eq!(
            prove(
                &make(xor64(
                    wide(call(&format!("batch{count}"), args)),
                    NirExpr::Int(0)
                )),
                0
            ),
            count == 31
        );
    }
    for swapped in [false, true] {
        let (left, right) = if swapped {
            (projected("total"), wide(narrow(projected("tag"))))
        } else {
            (wide(narrow(projected("tag"))), projected("total"))
        };
        let value = float_argument(xor64(left, right));
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
        let expected = if swapped {
            vec!["wide", "flag", "total", "tag", "total"]
        } else {
            vec!["wide", "flag", "tag", "total", "total"]
        };
        assert_eq!(reads, expected);
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
}

// Build the XOR result bit by bit without using the implementation's XOR primitive.
fn xor_word(left: i64, right: i64) -> i64 {
    let modulus = 1i128 << 64;
    let mut left = i128::from(left).rem_euclid(modulus);
    let mut right = i128::from(right).rem_euclid(modulus);
    let mut place = 1i128;
    let mut bits = 0i128;
    for _ in 0..64 {
        if left % 2 != right % 2 {
            bits += place;
        }
        left /= 2;
        right /= 2;
        place *= 2;
    }
    (if bits >= 1i128 << 63 {
        bits - modulus
    } else {
        bits
    }) as i64
}
fn source_xor(
    ty: &str,
    gate: bool,
    logic: &str,
    cmp: &str,
    left: i32,
    right: i32,
    offset: i64,
    mask: i64,
    first: i64,
    second: i64,
    swapped: bool,
) -> String {
    let lhs = format!("(i64_from_i32(checked32(live.inner.code,first))+({offset}))");
    let rhs = format!("right64(({mask}),second)");
    let expression = if swapped {
        format!("({rhs}^{lhs})")
    } else {
        format!("({lhs}^{rhs})")
    };
    source_arithmetic(ty,gate,cmp,logic,first,second,left,right,&expression)
      .replace("@noinline fn checked64","@noinline fn right64(value:i64,guard:i64)->i64{let checked=0/guard;return value;} @noinline fn checked64")
}
// Reference events record completion; native probes separately pin callee entry order.
fn completed_calls(events: &[String]) -> Vec<&str> {
    events
        .iter()
        .filter_map(|event| {
            ["checked32", "right64", "checked64", "relay64"]
                .into_iter()
                .find(|name| event.contains(&format!("] {name}(")) && event.contains("cpu.call"))
        })
        .collect()
}
fn compare_word(left: i64, right: i64, cmp: &str) -> bool {
    match cmp {
        "==" => left == right,
        "!=" => left != right,
        "<" => left < right,
        "<=" => left <= right,
        ">" => left > right,
        ">=" => left >= right,
        _ => unreachable!(),
    }
}

#[test]
fn i64_xor_argument_print_epoch_source_keeps_full_width_words_and_comparison_choices() {
    for ty in ["f32", "f64"] {
        for (_, cmp) in COMPARISONS {
            for swapped in [false, true] {
                for (logic, gate, left, right, offset, mask) in [
                    ("&&", true, i32::MIN, i32::MAX, i64::MAX, 4294967296),
                    ("||", false, i32::MAX, i32::MIN, i64::MAX, -1),
                    ("&&", true, -1, 0, i64::MIN + 1, 0),
                    ("||", false, 0, -1, 0, i64::MAX),
                    ("&&", false, -7, 0, i64::MAX, -1),
                    ("||", true, -7, 0, i64::MAX, -1),
                ] {
                    let events = execute_source(&source_xor(
                        ty, gate, logic, cmp, left, right, offset, mask, 1, 1, swapped,
                    ))
                    .unwrap();
                    let entered = if logic == "&&" { gate } else { !gate };
                    let expected = if entered {
                        [left, right]
                            .into_iter()
                            .map(|code| {
                                xor_word(wrap64(i128::from(code) + i128::from(offset)), mask)
                            })
                            .collect::<Vec<_>>()
                    } else {
                        vec![]
                    };
                    let forwarded = expected
                        .iter()
                        .map(|word| format!("{word}, 1"))
                        .collect::<Vec<_>>();
                    assert_eq!(calls(&events, "relay64"), forwarded);
                    assert_eq!(calls(&events, "checked64"), forwarded);
                    assert_eq!(
                        calls(&events, "right64"),
                        if entered {
                            vec![format!("{mask}, 1"); 2]
                        } else {
                            vec![]
                        }
                    );
                    let picks = if entered {
                        expected
                            .iter()
                            .map(|word| compare_word(*word, i64::from(right), cmp))
                            .collect::<Vec<_>>()
                    } else {
                        vec![gate; 2]
                    };
                    let actual = calls(&events, "choose")
                        .iter()
                        .map(|args| args.split(", ").nth(1).unwrap().parse::<bool>().unwrap())
                        .collect::<Vec<_>>();
                    assert_eq!(actual, picks);
                    assert_eq!(prints(&events), vec![if picks[0] { -1 } else { 1 }, 0]);
                    let order = if swapped {
                        ["right64", "checked32", "checked64", "relay64"]
                    } else {
                        ["checked32", "right64", "checked64", "relay64"]
                    };
                    assert_eq!(
                        completed_calls(&events),
                        if entered { order.repeat(2) } else { vec![] }
                    );
                }
            }
        }
    }
}

#[test]
fn i64_xor_argument_print_epoch_source_is_eager_inside_guarded_contexts() {
    for ty in ["f32", "f64"] {
        for swapped in [false, true] {
            for (logic, gate) in [("&&", true), ("&&", false), ("||", true), ("||", false)] {
                for (first, second) in [(1, 1), (0, 1), (1, 0), (0, 0)] {
                    // Zero on either side is not a short-circuit rule for XOR.
                    let events = execute_source(&source_xor(
                        ty, gate, logic, "!=", 0, 0, 0, 0, first, second, swapped,
                    ));
                    let entered = if logic == "&&" { gate } else { !gate };
                    assert_eq!(events.is_err(), entered && (first == 0 || second == 0));
                    if let Ok(events) = events {
                        let order = if swapped {
                            ["right64", "checked32", "checked64", "relay64"]
                        } else {
                            ["checked32", "right64", "checked64", "relay64"]
                        };
                        assert_eq!(
                            completed_calls(&events),
                            if entered { order.repeat(2) } else { vec![] }
                        );
                        assert_eq!(prints(&events).len(), 2);
                    }
                }
            }
        }
    }
}
