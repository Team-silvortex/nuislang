use super::*;

#[path = "i64_arithmetic_argument_print_epoch_tests.rs"]
mod arithmetic;

fn wide(value: NirExpr) -> NirExpr {
    NirExpr::CastI32ToI64(Box::new(value))
}
fn module_wide() -> NirModule {
    let mut module = i32_module();
    module.functions.extend(crate::frontend::parse_nuis_module("mod cpu MoreWide{fn keep64(value:i64)->i64{return value;}fn narrow64(value:i64)->i32{return i32_from_i64(value);}fn bool64(value:i64)->bool{return value>0;}fn float64(value:i64)->f64{return 0.5;}fn converted(value:i32)->i64{return i64_from_i32(value);}fn mixed(value:i32,wide:i64)->i64{return wide;}}").unwrap().functions);
    module
}
fn compared(value: NirExpr) -> NirExpr {
    in_rhs(binary(NirBinaryOp::Ne, value, projected("total")))
}
fn float_argument(value: NirExpr) -> NirExpr {
    let NirExpr::Call { mut args, .. } = compared(value) else {
        unreachable!()
    };
    args.remove(0)
}

#[test]
fn i32_widening_argument_print_epoch_requires_exact_lazy_comparison_reads() {
    for reversed in [false, true] {
        let mut module = module_wide();
        if reversed {
            module.functions.reverse();
        }
        for (cmp, _) in COMPARISONS {
            for (left, right) in [
                (wide(narrow(projected("tag"))), projected("total")),
                (projected("total"), wide(projected("tag"))),
            ] {
                let value = in_rhs(binary(cmp, left, right));
                assert!(shape(&value, &module));
                assert!(check(&value, &module, false, true).is_some());
                assert!(check(&value, &module, true, true).is_some());
                assert!(check(&value, &module, true, false).is_none());
            }
        }
        assert!(check(
            &selector(binary(
                NirBinaryOp::Ne,
                wide(projected("tag")),
                projected("total")
            )),
            &module,
            false,
            true
        )
        .is_none());
    }
}

#[test]
fn i32_widening_argument_print_epoch_rejects_wrong_kinds_effects_and_narrowing() {
    let module = module_wide();
    for value in [
        projected("total"),
        projected("flag"),
        projected("small"),
        projected("wide"),
        NirExpr::Var("missing".into()),
        NirExpr::Var("live".into()),
        wide(projected("tag")),
        call("transitive32", vec![projected("tag")]),
        call("cycle32a", vec![projected("tag")]),
        call("record32", vec![NirExpr::Var("live".into())]),
    ] {
        assert!(check(&compared(wide(value)), &module, false, true).is_none());
    }
    let value = NirExpr::CastI64ToI32(Box::new(wide(narrow(projected("tag")))));
    assert!(check(
        &in_rhs(binary(NirBinaryOp::Ne, value, projected("tag"))),
        &module,
        false,
        true
    )
    .is_none());
    for op in [
        NirBinaryOp::Add,
        NirBinaryOp::Sub,
        NirBinaryOp::Mul,
        NirBinaryOp::Div,
        NirBinaryOp::Rem,
        NirBinaryOp::Xor,
    ] {
        assert!(check(
            &compared(binary(op, wide(projected("tag")), projected("tag"))),
            &module,
            false,
            true
        )
        .is_none());
    }
}

#[test]
fn i32_widening_argument_print_epoch_retains_kind_preserving_call_provenance() {
    let module = module_wide();
    let value = wide(narrow(projected("tag")));
    assert!(check(
        &compared(call("keep64", vec![call("keep64", vec![value.clone()])])),
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
        &compared(call("converted", vec![narrow(projected("tag"))])),
        &module,
        false,
        true
    )
    .is_none());
    assert!(check(
        &compared(call("mixed", vec![narrow(projected("tag")), value.clone()])),
        &module,
        false,
        true
    )
    .is_none());
    assert!(check(
        &compared(call("keep64", vec![wide(narrow(projected("tag")))])),
        &module,
        false,
        true
    )
    .is_some());
    assert!(check(
        &in_rhs(call(
            "all",
            vec![
                projected("flag"),
                narrow(projected("tag")),
                projected("total"),
                projected("small"),
                projected("wide")
            ]
        )),
        &module,
        false,
        true
    )
    .is_none());
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
fn i32_widening_argument_print_epoch_charges_conversion_depth_work_and_capture_union() {
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
    for count in [31, 32] {
        let mut args = vec![projected("flag"), projected("tag")];
        args.extend((0..count - 2).map(|i| NirExpr::Var(format!("v{i}"))));
        let value = make(wide(call(&format!("batch{count}"), args)));
        assert_eq!(
            binary_float::validate(&value, &scalar_type("f64"), 0, &catalog, |leaf, _| {
                control_values::value_type(leaf, &frame, &catalog, &layouts)
            })
            .is_some(),
            count == 31
        );
    }
    for calls in [1, 2] {
        // 2043 operations + 2044 leaves + one extension + seven outer nodes + calls.
        let mut value = wide(balanced(2043));
        for _ in 0..calls {
            value = call("keep64", vec![value]);
        }
        assert_eq!(
            binary_float::validate(&make(value), &scalar_type("f64"), 0, &catalog, |leaf, _| {
                control_values::value_type(leaf, &frame, &catalog, &layouts)
            })
            .is_some(),
            calls == 1,
            "4096/4097 work"
        );
    }
    let root = make(wide(NirExpr::Var("code".into())));
    for (depth, expected) in [(60, true), (61, false)] {
        assert_eq!(
            binary_float::validate(&root, &scalar_type("f64"), depth, &catalog, |leaf, _| {
                control_values::value_type(leaf, &frame, &catalog, &layouts)
            })
            .is_some(),
            expected
        );
    }
    let value = float_argument(wide(narrow(projected("tag"))));
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
    assert_eq!(reads, ["wide", "flag", "tag", "total"]);
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

fn source_wide(
    ty: &str,
    gate: bool,
    cmp: &str,
    logic: &str,
    first: i64,
    second: i64,
    left: i32,
    right: i32,
    operand: &str,
) -> String {
    program32(ty,gate,cmp,logic,first,second,left,right)
    .replace("@noinline fn choose", "@noinline fn checked64(value:i64,guard:i64)->i64{let checked=0/guard;return value;} @noinline fn relay64(value:i64,guard:i64)->i64{return checked64(value,guard);} @noinline fn choose")
    .replace("relay32(checked32(live.inner.code,first),second)",&format!("relay64(i64_from_i32({operand}),second)"))
    .replace(&format!("{cmp}right"),&format!("{cmp}i64_from_i32(right)"))
}
fn calls(events: &[String], name: &str) -> Vec<String> {
    events
        .iter()
        .filter_map(|event| {
            event
                .split_once(&format!("] {name}("))
                .map(|(_, args)| args.trim_end_matches(')').to_owned())
        })
        .collect()
}
#[test]
fn i32_widening_argument_print_epoch_source_preserves_signed_values_and_skipped_faults() {
    for ty in ["f32", "f64"] {
        for (_, cmp) in COMPARISONS {
            for (logic, gate) in [("&&", false), ("&&", true), ("||", false), ("||", true)] {
                for (left, right) in [
                    (i32::MIN, i32::MAX),
                    (i32::MAX, i32::MIN),
                    (-1, 0),
                    (0, -1),
                    (107, 106),
                ] {
                    for (first, second) in [(1, 1), (0, 1), (1, 0)] {
                        let source = source_wide(
                            ty,
                            gate,
                            cmp,
                            logic,
                            first,
                            second,
                            left,
                            right,
                            "checked32(live.inner.code,first)",
                        );
                        let nir = crate::frontend::parse_nuis_module(&source).unwrap();
                        let yir =
                            crate::lowering::bootstrap::lower_nir_to_yir_builtin_cpu(&nir).unwrap();
                        assert!(yir_lower_llvm::emit_module(&yir)
                            .unwrap()
                            .contains("sext i32"));
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
                        let values = if entered {
                            [left, right]
                                .into_iter()
                                .map(|v| format!("{}i32, {first}", v))
                                .collect::<Vec<_>>()
                        } else {
                            vec![]
                        };
                        let forwarded = if entered {
                            [left, right]
                                .into_iter()
                                .map(|v| format!("{}, {second}", i64::from(v)))
                                .collect::<Vec<_>>()
                        } else {
                            vec![]
                        };
                        assert_eq!(calls(&events, "checked32"), values);
                        assert_eq!(calls(&events, "relay64"), forwarded);
                        assert_eq!(calls(&events, "checked64"), forwarded);
                        let expected = [left, right]
                            .into_iter()
                            .map(|v| {
                                if !entered {
                                    gate
                                } else {
                                    let a = i64::from(v);
                                    let b = i64::from(right);
                                    match cmp {
                                        "==" => a == b,
                                        "!=" => a != b,
                                        "<" => a < b,
                                        "<=" => a <= b,
                                        ">" => a > b,
                                        ">=" => a >= b,
                                        _ => unreachable!(),
                                    }
                                }
                            })
                            .collect::<Vec<_>>();
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
fn i32_widening_argument_print_epoch_source_keeps_checked_i32_faults_before_conversion() {
    for ty in ["f32", "f64"] {
        for token in ["/", "%"] {
            for gate in [false, true] {
                for (left, right) in [(i32::MIN, 1), (i32::MIN, -1), (-7, 3), (7, 0)] {
                    let source = source_wide(
                        ty,
                        gate,
                        "!=",
                        "&&",
                        1,
                        1,
                        left,
                        right,
                        &format!("-(checked32(live.inner.code,first){token}right)"),
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
