use super::*;

fn i32_module() -> NirModule {
    let mut module = module();
    let extra = crate::frontend::parse_nuis_module(
        "mod cpu More {
        struct Pair{small:f32,wide:f64,total:i64,flag:bool,tag:i32}struct Outer{inner:Pair}
        fn all32(flag:bool,tag:i32,total:i64,small:f32,wide:f64)->i32{return tag;}
        fn choose32(flag:bool,tag:i32)->i32{return tag;}
        fn record32(value:Outer)->i32{return value.inner.tag;}
        fn noisy32(value:i32)->i32{print(9);return value;}
        fn transitive32(value:i32)->i32{return noisy32(value);}
        fn cycle32a(value:i32)->i32{return cycle32b(value);}
        fn cycle32b(value:i32)->i32{return cycle32a(value);}
    }",
    )
    .unwrap();
    module.functions.extend(extra.functions);
    module
}

fn narrow(value: NirExpr) -> NirExpr {
    call("narrow", vec![value])
}
fn in_rhs(value: NirExpr) -> NirExpr {
    selector(binary(NirBinaryOp::And, projected("flag"), value))
}

#[test]
fn i32_call_argument_print_epoch_requires_exact_comparison_result_and_lazy_scope() {
    for reversed in [false, true] {
        let mut module = i32_module();
        if reversed {
            module.functions.reverse();
        }
        for (op, _) in COMPARISONS {
            for (left, right) in [
                (narrow(projected("tag")), projected("tag")),
                (projected("tag"), narrow(narrow(projected("tag")))),
            ] {
                let comparison = binary(op, left, right);
                let value = in_rhs(comparison.clone());
                assert!(shape(&value, &module));
                assert!(check(&value, &module, false, true).is_some());
                assert!(check(&value, &module, true, true).is_some());
                assert!(check(&value, &module, true, false).is_none());
                assert!(check(&selector(comparison.clone()), &module, false, true).is_none());
                assert!(check(
                    &selector(binary(NirBinaryOp::And, comparison, projected("flag"))),
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
fn i32_call_argument_print_epoch_keeps_new_results_comparison_only() {
    let module = i32_module();
    for value in [
        call(
            "all",
            vec![
                projected("flag"),
                narrow(projected("tag")),
                projected("total"),
                projected("small"),
                projected("wide"),
            ],
        ),
        binary(
            NirBinaryOp::Eq,
            binary(NirBinaryOp::Add, narrow(projected("tag")), projected("tag")),
            projected("tag"),
        ),
        binary(
            NirBinaryOp::Eq,
            NirExpr::CastI32ToI64(Box::new(narrow(projected("tag")))),
            projected("total"),
        ),
        binary(
            NirBinaryOp::Eq,
            narrow(projected("tag")),
            projected("total"),
        ),
        binary(
            NirBinaryOp::Eq,
            narrow(projected("tag")),
            projected("small"),
        ),
    ] {
        assert!(check(&in_rhs(value), &module, false, true).is_none());
    }
    // Existing field arguments keep their independent permission.
    let existing = call(
        "all",
        vec![
            projected("flag"),
            projected("tag"),
            projected("total"),
            projected("small"),
            projected("wide"),
        ],
    );
    assert!(check(&in_rhs(existing), &module, false, true).is_some());
}

#[test]
fn i32_call_argument_print_epoch_certifies_every_parameter_and_dependency() {
    let module = i32_module();
    let params = vec![
        projected("flag"),
        projected("tag"),
        projected("total"),
        projected("small"),
        projected("wide"),
    ];
    let make = |args| {
        in_rhs(binary(
            NirBinaryOp::Le,
            call("all32", args),
            projected("tag"),
        ))
    };
    assert!(check(&make(params.clone()), &module, false, true).is_some());
    for index in 0..params.len() {
        let mut bad = params.clone();
        bad[index] = NirExpr::Var("missing".into());
        assert!(check(&make(bad), &module, false, true).is_none());
    }
    for name in [
        "record32",
        "noisy32",
        "transitive32",
        "cycle32a",
        "cycle32b",
        "missing",
    ] {
        assert!(
            check(
                &in_rhs(binary(
                    NirBinaryOp::Eq,
                    call(name, vec![projected("tag")]),
                    projected("tag")
                )),
                &module,
                false,
                true
            )
            .is_none(),
            "{name}"
        );
    }
    for args in [
        vec![],
        vec![projected("tag"), projected("tag")],
        vec![projected("total")],
    ] {
        assert!(check(
            &in_rhs(binary(
                NirBinaryOp::Eq,
                call("narrow", args),
                projected("tag")
            )),
            &module,
            false,
            true
        )
        .is_none());
    }
}

#[test]
fn i32_call_argument_print_epoch_retains_scoped_logical_parameter_boundary() {
    let module = i32_module();
    let value = selector(binary(
        NirBinaryOp::Eq,
        call("choose32", vec![logical_flag(), projected("tag")]),
        projected("tag"),
    ));
    assert!(!shape(&value, &module));
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    assert!(!conditional_values::prefix::computed_call_expression_roots(
        vec![(&value, 0, false)],
        &catalog
    ));
}

#[test]
fn i32_call_argument_print_epoch_shares_capture_depth_and_read_order_bounds() {
    let mut module = i32_module();
    for count in [31, 32] {
        let params = (0..count - 1)
            .map(|i| format!(",p{i}:i64"))
            .collect::<String>();
        let extra = crate::frontend::parse_nuis_module(&format!(
            "mod cpu Batch{{fn batch{count}(tag:i32{params})->i32{{return tag;}}}}"
        ))
        .unwrap();
        module.functions.extend(extra.functions);
    }
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    let mut frame = scope();
    for i in 0..31 {
        frame.insert(format!("v{i}"), scalar_type("i64"));
    }
    let valid = |value: &NirExpr| {
        binary_float::validate(value, &scalar_type("f64"), 0, &catalog, |leaf, _| {
            control_values::value_type(leaf, &frame, &catalog, &layouts)
        })
        .is_some()
    };
    for count in [31, 32] {
        let mut args = vec![projected("tag")];
        args.extend((0..count - 1).map(|i| NirExpr::Var(format!("v{i}"))));
        let value = call(
            "choose",
            vec![
                projected("wide"),
                binary(
                    NirBinaryOp::And,
                    projected("flag"),
                    binary(
                        NirBinaryOp::Gt,
                        call(&format!("batch{count}"), args),
                        projected("tag"),
                    ),
                ),
                NirExpr::Int(1),
            ],
        );
        assert_eq!(valid(&value), count == 31);
    }
    let value = call(
        "choose",
        vec![
            projected("wide"),
            binary(
                NirBinaryOp::And,
                projected("flag"),
                binary(
                    NirBinaryOp::Ne,
                    narrow(projected("tag")),
                    narrow(projected("tag")),
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
    assert_eq!(reads, ["wide", "flag", "tag", "tag"]);
    let mut deep = projected("tag");
    for _ in 0..66 {
        deep = narrow(deep);
    }
    assert!(check(
        &in_rhs(binary(NirBinaryOp::Eq, deep, projected("tag"))),
        &module,
        false,
        true
    )
    .is_none());
}

fn program32(
    ty: &str,
    gate: bool,
    op: &str,
    logic: &str,
    first: i64,
    second: i64,
    left: i32,
    right: i32,
) -> String {
    format!("mod cpu Main{{struct Pair{{value:{ty},code:i32,flag:bool}}struct Outer{{inner:Pair}}
        @noinline fn checked32(value:i32,guard:i64)->i32{{let checked=0/guard;return value;}}
        @noinline fn relay32(value:i32,guard:i64)->i32{{return checked32(value,guard);}}
        @noinline fn choose(value:{ty},pick:bool)->{ty}{{if pick{{return value;}}return -value;}}
        @noinline fn observe(value:{ty})->i64{{if value<0.0{{return -1;}}if value==0.0{{return 0;}}if value!=value{{return 2;}}return 1;}}
        @noinline fn walk(seed:{ty},selected:bool,first:i64,second:i64,left:i32,right:i32)->{ty}{{
            let mut index=0;let mut live=Outer{{inner:Pair{{value:seed,code:left,flag:selected}}}};
            while index<2{{print(observe(-choose(live.inner.value+seed,live.inner.flag{logic}(relay32(checked32(live.inner.code,first),second){op}right))));
            live=Outer{{inner:Pair{{value:-live.inner.value,code:right,flag:selected}}}};index=index+1;}}return live.inner.value;}}
        fn main()->{ty}{{return walk(1.5,{gate},{first},{second},i32_from_i64({left}),i32_from_i64({right}));}}}}")
}

#[test]
fn i32_call_argument_print_epoch_source_keeps_signed_results_and_skipped_faults() {
    for ty in ["f32", "f64"] {
        for (_, op) in COMPARISONS {
            for (left, right) in [(i32::MIN, i32::MAX), (i32::MAX, i32::MIN), (-1, 0), (0, 0)] {
                for (logic, gate) in [("&&", true), ("&&", false), ("||", true), ("||", false)] {
                    let selected = if logic == "&&" { gate } else { !gate };
                    for (first, second) in [(1, 1), (0, 1), (1, 0)] {
                        let module = crate::frontend::parse_nuis_module(&program32(
                            ty, gate, op, logic, first, second, left, right,
                        ))
                        .unwrap();
                        let yir = crate::lowering::bootstrap::lower_nir_to_yir_builtin_cpu(&module)
                            .unwrap();
                        let result = yir_runtime_host::execute_module_source_with_registry(
                            &crate::render::render_yir(&yir),
                            &yir_verify::default_registry(),
                        );
                        if selected && (first == 0 || second == 0) {
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
                        let expected = [left, right]
                            .into_iter()
                            .map(|code| {
                                if !selected {
                                    return gate;
                                }
                                let a = i64::from(code);
                                let b = i64::from(right);
                                match op {
                                    "==" => a == b,
                                    "!=" => a != b,
                                    "<" => a < b,
                                    "<=" => a <= b,
                                    ">" => a > b,
                                    ">=" => a >= b,
                                    _ => unreachable!(),
                                }
                            })
                            .collect::<Vec<_>>();
                        assert_eq!(picks, expected, "{ty} {logic} {gate} {left} {right} {op}");
                        for (callee, count) in [("checked32", 4), ("relay32", 2)] {
                            assert_eq!(
                                events
                                    .iter()
                                    .filter(|e| e.contains("cpu.call")
                                        && e.contains(&format!("] {callee}(")))
                                    .count(),
                                if selected { count } else { 0 }
                            );
                        }
                        assert_eq!(prints(&events).len(), 2);
                        yir_lower_llvm::emit_module(&yir).unwrap();
                    }
                }
            }
        }
    }
}

#[path = "logical_i32_call_argument_print_epoch_tests.rs"]
mod logical_arguments;
