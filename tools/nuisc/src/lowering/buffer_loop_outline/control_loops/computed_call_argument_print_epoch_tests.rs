use super::*;

fn binary(op: NirBinaryOp, lhs: NirExpr, rhs: NirExpr) -> NirExpr {
    NirExpr::Binary {
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    }
}
fn argument(flag: NirExpr, guard: NirExpr) -> NirExpr {
    call(
        "observe",
        vec![call(
            "mixed",
            vec![
                projected("wide"),
                flag,
                projected("tag"),
                guard,
                projected("small"),
            ],
        )],
    )
}
fn admitted(value: &NirExpr, child: bool, ready: bool) -> Option<NirTypeRef> {
    let module = calls_module();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    let locals = Locals {
        scope: scope(),
        writable: BTreeSet::from(["live".into()]),
        available: BTreeSet::new(),
    };
    let changed = locals.writable.clone();
    let epoch = if child {
        Epoch::enter_child(
            &locals.scope,
            &changed,
            &changed,
            if ready { &changed } else { &locals.available },
            &layouts,
        )
    } else {
        Epoch::enter(&locals.scope, &changed, &changed, &layouts)
    };
    assert!(expression(value, &locals, &changed, None, &catalog, &layouts, 0).is_none());
    expression_with_fields(
        value,
        &locals,
        &changed,
        None,
        &catalog,
        &layouts,
        0,
        Some(&epoch),
    )
}

#[test]
fn computed_call_argument_print_epoch_reuses_exact_integer_and_boolean_types() {
    for op in [
        NirBinaryOp::Add,
        NirBinaryOp::Sub,
        NirBinaryOp::Mul,
        NirBinaryOp::Div,
        NirBinaryOp::Rem,
        NirBinaryOp::Xor,
    ] {
        let guard = binary(op, projected("total"), NirExpr::Int(1));
        for cmp in [
            NirBinaryOp::Eq,
            NirBinaryOp::Ne,
            NirBinaryOp::Lt,
            NirBinaryOp::Le,
            NirBinaryOp::Gt,
            NirBinaryOp::Ge,
        ] {
            for kind in ["total", "tag"] {
                let flag = binary(
                    NirBinaryOp::Or,
                    binary(
                        NirBinaryOp::And,
                        binary(cmp, projected(kind), projected(kind)),
                        projected("flag"),
                    ),
                    NirExpr::Bool(false),
                );
                let value = argument(flag, guard.clone());
                assert_eq!(admitted(&value, false, true), Some(scalar_type("i64")));
                assert_eq!(admitted(&value, true, true), Some(scalar_type("i64")));
                assert!(admitted(&value, true, false).is_none());
            }
        }
    }
    for (flag, guard) in [
        (
            projected("flag"),
            binary(NirBinaryOp::Add, projected("tag"), projected("tag")),
        ),
        (
            binary(NirBinaryOp::Eq, projected("tag"), projected("total")),
            projected("total"),
        ),
        (
            binary(NirBinaryOp::Xor, projected("flag"), NirExpr::Int(1)),
            projected("total"),
        ),
        (projected("total"), projected("total")),
        (projected("flag"), call("wrong", vec![projected("wide")])),
    ] {
        assert!(admitted(&argument(flag, guard), false, true).is_none());
    }
}

#[test]
fn computed_call_argument_print_epoch_certifies_fallible_rhs_without_widening_ordinary_reads() {
    let division = binary(
        NirBinaryOp::Gt,
        binary(NirBinaryOp::Div, projected("total"), NirExpr::Int(0)),
        NirExpr::Int(1),
    );
    for op in [NirBinaryOp::And, NirBinaryOp::Or] {
        // The left side is always reached; the right side retains its own scalar capture proof.
        assert!(admitted(
            &argument(
                binary(op, division.clone(), projected("flag")),
                NirExpr::Int(1)
            ),
            false,
            true
        )
        .is_some());
        assert!(admitted(
            &argument(
                binary(op, projected("flag"), division.clone()),
                NirExpr::Int(1)
            ),
            false,
            true
        )
        .is_some());
    }
    let nested = binary(
        NirBinaryOp::Or,
        projected("flag"),
        binary(NirBinaryOp::And, NirExpr::Bool(false), division),
    );
    assert!(admitted(&argument(nested, NirExpr::Int(1)), false, true).is_some());
}

#[test]
fn computed_call_argument_print_epoch_scopes_logical_shape_and_shared_edge_budget() {
    let module = calls_module();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    let mut tree = projected("flag");
    for count in 1..=34 {
        tree = binary(NirBinaryOp::And, tree, NirExpr::Bool(true));
        let value = argument(tree.clone(), NirExpr::Int(1));
        assert!(!conditional_values::prefix::expression(&value));
        assert_eq!(
            conditional_values::prefix::computed_call_expression_roots(
                vec![(&value, 0, false)],
                &catalog
            ),
            count <= 32
        );
    }
    for callee in ["observe", "id64", "effect", "transit", "missing"] {
        let value = call(callee, vec![tree.clone()]);
        assert!(!conditional_values::prefix::computed_call_expression_roots(
            vec![(&value, 0, false)],
            &catalog
        ));
    }
}

#[test]
fn computed_call_argument_print_epoch_ready_float_fallback_keeps_rejected_reads_closed() {
    for op in [NirBinaryOp::And, NirBinaryOp::Or] {
        let rhs = binary(
            NirBinaryOp::Gt,
            binary(
                NirBinaryOp::Div,
                projected("total"),
                call("wrong", vec![NirExpr::F64("1.0".into())]),
            ),
            NirExpr::Int(0),
        );
        let selector = call(
            "choose",
            vec![
                NirExpr::F64("1.0".into()),
                binary(op, NirExpr::Bool(true), rhs),
                NirExpr::Int(1),
            ],
        );
        assert!(admitted(&call("observe", vec![selector.clone()]), false, true).is_none());
        assert!(admitted(&NirExpr::PackF64Word(Box::new(selector)), false, true).is_none());
    }
}

#[test]
fn computed_call_argument_print_epoch_bounds_work_depth_and_original_leaf_order() {
    let module = calls_module();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    let expected = scalar_type("f64");
    let mut guard = projected("total");
    for count in 1..=67 {
        guard = binary(NirBinaryOp::Add, guard, NirExpr::Int(1));
        let NirExpr::Call { args, .. } = argument(projected("flag"), guard.clone()) else {
            panic!()
        };
        let result = binary_float::validate(&args[0], &expected, 0, &catalog, |value, _| {
            control_values::value_type(value, &scope(), &catalog, &layouts)
        });
        assert_eq!(result.is_some(), count + 3 <= 64);
    }
    let mut tree = NirExpr::Int(1);
    for _ in 0..12 {
        tree = binary(NirBinaryOp::Add, tree.clone(), tree);
    }
    assert!(admitted(&argument(projected("flag"), tree), false, true).is_none());
    let flag = binary(
        NirBinaryOp::And,
        binary(NirBinaryOp::Gt, projected("total"), NirExpr::Int(0)),
        projected("flag"),
    );
    let NirExpr::Call { args, .. } = argument(flag, NirExpr::Int(1)) else {
        panic!()
    };
    let mut reads = Vec::new();
    assert!(
        binary_float::validate(&args[0], &expected, 0, &catalog, |value, _| {
            if let NirExpr::FieldAccess { field, .. } = value {
                reads.push(field.clone());
            }
            control_values::value_type(value, &scope(), &catalog, &layouts)
        })
        .is_some()
    );
    assert_eq!(reads, ["wide", "total", "flag", "tag", "small"]);
}

fn program(
    ty: &str,
    selected: bool,
    first: i64,
    second: i64,
    leading: bool,
    skipped: bool,
) -> String {
    let print="print(observe(choose(live.inner.value+seed,(live.inner.tag/first>=50&&live.inner.flag)||live.inner.tag==99,second+(live.inner.tag-live.inner.tag))));";
    let print = if skipped {
        format!("if selected{{{print}}}")
    } else {
        print.into()
    };
    let effect=format!("{print}live=Outer{{inner:Pair{{value:-live.inner.value,tag:live.inner.tag+1,flag:selected}}}};");
    let body = if leading {
        format!("index=index+1;{effect}")
    } else {
        format!("{effect}index=index+1;")
    };
    format!("mod cpu Main{{struct Pair{{value:{ty},tag:i64,flag:bool}}struct Outer{{inner:Pair}}@noinline fn choose(value:{ty},pick:bool,guard:i64)->{ty}{{let checked=0/guard;if pick{{return value;}}return -value;}}@noinline fn observe(value:{ty})->i64{{if value<0.0{{return -1;}}if value==0.0{{return 0;}}if value!=value{{return 2;}}return 1;}}@noinline fn walk(seed:{ty},selected:bool,first:i64,second:i64)->{ty}{{let mut index=0;let mut live=Outer{{inner:Pair{{value:seed,tag:100,flag:selected}}}};while index<2{{{body}}}return live.inner.value;}}fn main()->{ty}{{return walk(1.5,{selected},{first},{second});}}}}")
}

#[test]
fn computed_call_argument_print_epoch_raw_source_keeps_ieee_values_and_selected_failures() {
    for ty in ["f32", "f64"] {
        let words: &[u64] = if ty == "f32" {
            &[0, 0x80000000, 1, 0x3fc00000, 0x7f800000, 0x7fc12345]
        } else {
            &[
                0,
                0x8000000000000000,
                1,
                0x3ff8000000000000,
                0x7ff0000000000000,
                0x7ff812345678abcd,
            ]
        };
        for leading in [false, true] {
            for selected in [false, true] {
                for &bits in words {
                    let mut module = crate::frontend::parse_nuis_module(&program(
                        ty, selected, 2, 1, leading, false,
                    ))
                    .unwrap();
                    let main = module
                        .functions
                        .iter_mut()
                        .find(|f| f.name == "main")
                        .unwrap();
                    let NirStmt::Return(Some(NirExpr::Call { args, .. })) = &mut main.body[0]
                    else {
                        panic!()
                    };
                    let raw = Box::new(NirExpr::Int(bits as i64));
                    args[0] = if ty == "f32" {
                        NirExpr::UnpackF32Word(raw)
                    } else {
                        NirExpr::UnpackF64Word(raw)
                    };
                    crate::nir_verify::verify_nir_module(&module).unwrap();
                    let layouts = control_values::layouts(&module);
                    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
                    assert!(
                        catalog.contains_key("walk"),
                        "catalog={:?}, choose={}",
                        catalog.keys().collect::<Vec<_>>(),
                        scalar_helpers::pure_float_signature("choose", &catalog).is_some()
                    );
                    let yir =
                        crate::lowering::bootstrap::lower_nir_to_yir_builtin_cpu(&module).unwrap();
                    let trace = yir_runtime_host::execute_module_source_with_registry(
                        &crate::render::render_yir(&yir),
                        &yir_verify::default_registry(),
                    )
                    .unwrap();
                    let mut expected = Vec::new();
                    for sign in [false, true] {
                        let v = if ty == "f32" {
                            let v = f32::from_bits(bits as u32);
                            f64::from((if sign { -v } else { v }) + v)
                        } else {
                            let v = f64::from_bits(bits);
                            (if sign { -v } else { v }) + v
                        };
                        let v = if selected { v } else { -v };
                        expected.push(if v < 0.0 {
                            -1
                        } else if v == 0.0 {
                            0
                        } else if v.is_nan() {
                            2
                        } else {
                            1
                        });
                    }
                    assert_eq!(prints(&trace.events), expected);
                    yir_lower_llvm::emit_module(&yir).unwrap();
                }
            }
        }
        for (selected, first, second, skipped, zero) in [
            (true, 0, 1, false, false),
            (true, 2, 0, false, false),
            (false, 0, 0, true, false),
            (true, 0, 0, false, true),
        ] {
            let text = program(ty, selected, first, second, false, skipped)
                .replace("index<2", if zero { "index<0" } else { "index<2" });
            let module = crate::frontend::parse_nuis_module(&text).unwrap();
            let yir = crate::lowering::bootstrap::lower_nir_to_yir_builtin_cpu(&module).unwrap();
            let result = yir_runtime_host::execute_module_source_with_registry(
                &crate::render::render_yir(&yir),
                &yir_verify::default_registry(),
            );
            if skipped || zero {
                assert!(prints(&result.unwrap().events).is_empty());
            } else {
                assert!(result.unwrap_err().contains("zero"));
            }
        }
    }
}

#[path = "lazy_call_argument_print_epoch_tests.rs"]
mod lazy_arguments;

#[path = "helper_call_argument_print_epoch_tests.rs"]
mod helper_arguments;
