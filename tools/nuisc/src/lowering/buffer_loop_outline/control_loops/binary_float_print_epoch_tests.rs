use super::*;

fn op(op: NirBinaryOp, lhs: NirExpr, rhs: NirExpr) -> NirExpr {
    NirExpr::Binary {
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    }
}
fn frame(ty: &str) -> Locals {
    Locals {
        scope: Scope::from([
            ("live".into(), scalar_type("Outer")),
            ("ready".into(), scalar_type(ty)),
        ]),
        writable: BTreeSet::from(["live".into()]),
        available: BTreeSet::new(),
    }
}
fn printed_arg(ty: &str, arg: NirExpr) -> NirExpr {
    call(
        if ty == "f32" { "narrow" } else { "wide" },
        vec![arg, NirExpr::Int(1)],
    )
}
fn atom(ty: &str, value: &str) -> NirExpr {
    if ty == "f32" {
        NirExpr::F32(value.into())
    } else {
        NirExpr::F64(value.into())
    }
}

#[test]
fn binary_float_print_epoch_checks_all_ops_and_exact_ready_leaf_types() {
    let module = module();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    for ty in ["f32", "f64"] {
        let locals = frame(ty);
        let changed = locals.writable.clone();
        let epoch = Epoch::enter(&locals.scope, &changed, &changed, &layouts);
        for operation in [
            NirBinaryOp::Add,
            NirBinaryOp::Sub,
            NirBinaryOp::Mul,
            NirBinaryOp::Div,
        ] {
            for (left, right) in [
                (field(ty), NirExpr::Var("ready".into())),
                (NirExpr::Var("ready".into()), field(ty)),
                (field(ty), field(ty)),
                (field(ty), atom(ty, "2.0")),
            ] {
                let value = printed_arg(ty, negate(ty, op(operation, negate(ty, left), right)));
                assert_eq!(
                    expression_with_fields(
                        &value,
                        &locals,
                        &changed,
                        None,
                        &catalog,
                        &layouts,
                        0,
                        Some(&epoch)
                    ),
                    Some(scalar_type("i64"))
                );
                assert!(
                    expression(&value, &locals, &changed, None, &catalog, &layouts, 0).is_none()
                );
            }
        }
        let wrong = if ty == "f32" { "f64" } else { "f32" };
        for leaf in [
            field(wrong),
            atom(wrong, "1.0"),
            NirExpr::Int(1),
            call("floated", vec![field(ty)]),
            NirExpr::CastI64ToF64(Box::new(NirExpr::Int(1))),
        ] {
            let value = printed_arg(ty, op(NirBinaryOp::Add, field(ty), leaf));
            assert!(expression_with_fields(
                &value,
                &locals,
                &changed,
                None,
                &catalog,
                &layouts,
                0,
                Some(&epoch)
            )
            .is_none());
        }
    }
}

#[test]
fn binary_float_print_epoch_keeps_ordinary_reads_ancestors_and_stale_roots_closed() {
    let module = module();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    for ty in ["f32", "f64"] {
        let mut locals = frame(ty);
        let changed = BTreeSet::from(["live".into(), "ready".into()]);
        let value = printed_arg(
            ty,
            op(NirBinaryOp::Sub, field(ty), NirExpr::Var("ready".into())),
        );
        let epoch = Epoch::enter(&locals.scope, &locals.writable, &changed, &layouts);
        assert!(expression_with_fields(
            &value,
            &locals,
            &changed,
            None,
            &catalog,
            &layouts,
            0,
            Some(&epoch)
        )
        .is_none());
        locals.available.insert("ready".into());
        for child in [false, true] {
            let epoch = if child {
                Epoch::enter_child(
                    &locals.scope,
                    &locals.writable,
                    &changed,
                    &changed,
                    &layouts,
                )
            } else {
                Epoch::enter(&locals.scope, &locals.writable, &changed, &layouts)
            };
            assert!(block(
                &[NirStmt::Print(value.clone())],
                &mut locals,
                &changed,
                &catalog,
                &layouts,
                validation::LiteralPrints::Scalars,
                &epoch,
                0
            )
            .is_some());
            assert!(!locals.available.contains("live"));
            assert!(expression(&value, &locals, &changed, None, &catalog, &layouts, 0).is_none());
            let mut stale = layouts.clone();
            stale.get_mut("Pair").unwrap()[0].1 = scalar_type("i64");
            assert!(expression_with_fields(
                &value,
                &locals,
                &changed,
                None,
                &catalog,
                &stale,
                0,
                Some(&epoch)
            )
            .is_none());
            for invalid in [
                call(
                    "floated",
                    vec![op(NirBinaryOp::Add, field(ty), atom(ty, "1.0"))],
                ),
                printed_arg(ty, op(NirBinaryOp::Rem, field(ty), atom(ty, "1.0"))),
                call(
                    if ty == "f32" { "narrow" } else { "wide" },
                    vec![
                        atom(ty, "1.0"),
                        op(NirBinaryOp::Add, field(ty), atom(ty, "1.0")),
                    ],
                ),
            ] {
                assert!(expression_with_fields(
                    &invalid,
                    &locals,
                    &changed,
                    None,
                    &catalog,
                    &layouts,
                    0,
                    Some(&epoch)
                )
                .is_none());
            }
        }
        let child = Epoch::enter_child(
            &locals.scope,
            &locals.writable,
            &changed,
            &locals.available,
            &layouts,
        );
        assert!(expression_with_fields(
            &value,
            &locals,
            &changed,
            None,
            &catalog,
            &layouts,
            0,
            Some(&child)
        )
        .is_none());
    }
}

#[test]
fn binary_float_print_epoch_bounds_depth_work_and_checks_left_before_right() {
    for ty in ["f32", "f64"] {
        let expected = scalar_type(ty);
        for depth in [0usize, 1, 63, 64, usize::MAX] {
            let value = op(
                NirBinaryOp::Sub,
                NirExpr::Var("left".into()),
                NirExpr::Var("right".into()),
            );
            let mut visited = Vec::new();
            let result = binary_float::validate(
                &value,
                &expected,
                depth,
                &ScalarHelpers::new(),
                |leaf, _| {
                    let NirExpr::Var(name) = leaf else { panic!() };
                    visited.push(name.clone());
                    Some(expected.clone())
                },
            );
            assert_eq!(result.is_some(), depth < 64);
            if depth < 64 {
                assert_eq!(visited, ["left", "right"]);
            }
            let mut visited = Vec::new();
            assert!(binary_float::validate(
                &value,
                &expected,
                0,
                &ScalarHelpers::new(),
                |leaf, _| {
                    let NirExpr::Var(name) = leaf else { panic!() };
                    visited.push(name.clone());
                    None
                }
            )
            .is_none());
            assert_eq!(visited, ["left"]);
        }
        let mut value = atom(ty, "1.0");
        for level in 0..70 {
            value = op(NirBinaryOp::Add, value, atom(ty, "1.0"));
            assert_eq!(
                binary_float::validate(&value, &expected, 0, &ScalarHelpers::new(), |_, _| Some(
                    expected.clone()
                ))
                .is_some(),
                level < 64
            );
        }
        let mut tree = atom(ty, "1.0");
        for _ in 0..12 {
            tree = op(NirBinaryOp::Mul, tree.clone(), tree);
        }
        assert!(
            binary_float::validate(&tree, &expected, 0, &ScalarHelpers::new(), |_, _| Some(
                expected.clone()
            ))
            .is_none()
        );
    }
}

#[test]
fn binary_float_print_epoch_source_does_not_leak_into_conditions_or_initializers() {
    for ty in ["f32", "f64"] {
        for body in [
            "print(observe(live.inner.value+seed,1));let saved=live.inner.value+seed;live=live;index=index+1;",
            "if observe(live.inner.value+seed,1)==0{print(1);}live=live;index=index+1;",
            "let saved=observe(live.inner.value*seed,1);live=live;index=index+1;",
            "print(observe(identity(live.inner.value)+seed,1));live=live;index=index+1;",
            "let mut child=0;while child<1{print(observe(live.inner.value+seed,1));live=live;child=child+1;}index=index+1;",
        ] {
            let module=crate::frontend::parse_nuis_module(&source(ty,body,false,0)).unwrap();let layouts=control_values::layouts(&module);
            assert!(!scalar_helpers::collect_iteration_calls(&module,&layouts).contains_key("walk"),"{ty}: {body}");
        }
    }
}

#[test]
fn binary_float_print_epoch_raw_nir_keeps_ieee_values_and_selected_poison_lazy() {
    for ty in ["f32", "f64"] {
        let (mask, words): (u64, &[u64]) = if ty == "f32" {
            (
                1 << 31,
                &[
                    0, 0x80000000, 1, 0x7f800000, 0x7fc12345, 0xff800001, 0x3fc00000,
                ],
            )
        } else {
            (
                1 << 63,
                &[
                    0,
                    0x8000000000000000,
                    1,
                    0x7ff0000000000000,
                    0x7ff812345678abcd,
                    0xfff0000000000001,
                    0x3ff8000000000000,
                ],
            )
        };
        for &bits in words {
            for selected in [false, true] {
                for leading in [false, true] {
                    let expression = "-(((live.inner.value+2.0)-(-2.0))*2.0/2.0)";
                    let effect=format!("if selected{{print(observe({expression},divisor));}}print(observe({expression},1));live=Outer{{inner:Pair{{value:-live.inner.value,total:0}}}};print(observe({expression},1));");
                    let body = if leading {
                        format!("index=index+1;{effect}")
                    } else {
                        format!("{effect}index=index+1;")
                    };
                    let mut module = crate::frontend::parse_nuis_module(&source(
                        ty,
                        &body,
                        selected,
                        if selected { 1 } else { 0 },
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
                    let word = Box::new(NirExpr::Int(bits as i64));
                    args[0] = if ty == "f32" {
                        NirExpr::UnpackF32Word(word)
                    } else {
                        NirExpr::UnpackF64Word(word)
                    };
                    crate::nir_verify::verify_nir_module(&module).unwrap();
                    let layouts = control_values::layouts(&module);
                    assert!(scalar_helpers::collect_iteration_calls(&module, &layouts)
                        .contains_key("walk"));
                    let yir =
                        crate::lowering::bootstrap::lower_nir_to_yir_builtin_cpu(&module).unwrap();
                    let trace = yir_runtime_host::execute_module_source_with_registry(
                        &crate::render::render_yir(&yir),
                        &yir_verify::default_registry(),
                    )
                    .unwrap();
                    let class = |bits| {
                        let value = if ty == "f32" {
                            let v = f32::from_bits(bits as u32);
                            f64::from(-(((v + 2.0) - (-2.0)) * 2.0 / 2.0))
                        } else {
                            let v = f64::from_bits(bits);
                            -(((v + 2.0) - (-2.0)) * 2.0 / 2.0)
                        };
                        if value < 0.0 {
                            -1
                        } else if value == 0.0 {
                            0
                        } else if value.is_nan() {
                            2
                        } else {
                            1
                        }
                    };
                    let mut current = bits;
                    let mut expected = Vec::new();
                    for _ in 0..2 {
                        if selected {
                            expected.push(class(current));
                        }
                        expected.push(class(current));
                        current ^= mask;
                        expected.push(class(current));
                    }
                    assert_eq!(prints(&trace.events), expected);
                    yir_lower_llvm::emit_module(&yir).unwrap();
                }
            }
        }
    }
}

#[test]
fn binary_float_print_epoch_checked_division_rejects_both_zero_signs_only_when_reached() {
    for ty in ["f32", "f64"] {
        for zero in ["0.0", "-0.0"] {
            for selected in [false, true] {
                let body=format!("if selected{{print(observe(live.inner.value/{zero},1));}}live=live;index=index+1;");
                let module =
                    crate::frontend::parse_nuis_module(&source(ty, &body, selected, 1)).unwrap();
                let yir =
                    crate::lowering::bootstrap::lower_nir_to_yir_builtin_cpu(&module).unwrap();
                let result = yir_runtime_host::execute_module_source_with_registry(
                    &crate::render::render_yir(&yir),
                    &yir_verify::default_registry(),
                );
                if selected {
                    assert!(result.unwrap_err().contains("zero"));
                } else {
                    assert!(prints(&result.unwrap().events).is_empty());
                }
                assert!(yir_lower_llvm::emit_module(&yir)
                    .unwrap()
                    .contains("float_divisor_invalid_"));
            }
        }
    }
}
