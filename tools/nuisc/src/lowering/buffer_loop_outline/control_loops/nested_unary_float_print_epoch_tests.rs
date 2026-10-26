use super::*;

fn chain(ty: &str, count: usize, mut value: NirExpr) -> NirExpr {
    for _ in 0..count {
        value = negate(ty, value);
    }
    value
}

fn source_chain(count: usize, mut value: String) -> String {
    for _ in 0..count {
        value = format!("-({value})");
    }
    value
}

#[test]
fn nested_unary_float_print_epoch_counts_every_layer_and_original_projection() {
    let module = module();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    let locals = Locals {
        scope: Scope::from([("live".into(), scalar_type("Outer"))]),
        writable: BTreeSet::from(["live".into()]),
        available: BTreeSet::new(),
    };
    let changed = locals.writable.clone();
    let epoch = Epoch::enter(&locals.scope, &changed, &changed, &layouts);
    for ty in ["f32", "f64"] {
        for count in 0..=24 {
            let arg = chain(ty, count, field(ty));
            for depth in [0usize, 1, 31, 59, 64, usize::MAX] {
                let expected =
                    count != 0 && depth.checked_add(3 * count + 2).is_some_and(|d| d <= 64);
                assert_eq!(
                    epoch
                        .unary_argument(&arg, &locals.scope, &layouts, &scalar_type(ty), depth)
                        .is_some(),
                    expected,
                    "{ty} count={count} depth={depth}"
                );
            }
            let value = call(
                if ty == "f32" { "narrow" } else { "wide" },
                vec![arg, NirExpr::Int(1)],
            );
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
                )
                .is_some(),
                3 * count + 3 <= 64
            );
            assert!(expression(&value, &locals, &changed, None, &catalog, &layouts, 0).is_none());
        }
        assert!(locals.available.is_empty());
    }
}

#[test]
fn nested_unary_float_print_epoch_rejects_bad_operations_at_every_layer() {
    let module = module();
    let layouts = control_values::layouts(&module);
    let scope = Scope::from([("live".into(), scalar_type("Outer"))]);
    let changed = BTreeSet::from(["live".into()]);
    let epoch = Epoch::enter(&scope, &changed, &changed, &layouts);
    for ty in ["f32", "f64"] {
        let wrong = if ty == "f32" { "f64" } else { "f32" };
        for count in [2, 3, 7, 20] {
            for bad_layer in 0..count {
                for mutation in 0..4 {
                    let mut value = field(ty);
                    for layer in 0..count {
                        value = negate(
                            if layer == bad_layer && mutation == 1 {
                                wrong
                            } else {
                                ty
                            },
                            value,
                        );
                        if layer == bad_layer && mutation != 1 {
                            let word = match &mut value {
                                NirExpr::UnpackF32Word(word) | NirExpr::UnpackF64Word(word) => word,
                                _ => unreachable!(),
                            };
                            let NirExpr::Binary { op, lhs, rhs } = word.as_mut() else {
                                unreachable!()
                            };
                            match mutation {
                                0 => **rhs = NirExpr::Int(1),
                                2 => *op = NirBinaryOp::Or,
                                _ => std::mem::swap(lhs, rhs),
                            }
                        }
                    }
                    assert!(
                        epoch
                            .unary_argument(&value, &scope, &layouts, &scalar_type(ty), 0)
                            .is_none(),
                        "{ty} count={count} bad={bad_layer}/{mutation}"
                    );
                }
            }
        }
        for leaf in [
            NirExpr::Var("live".into()),
            field(wrong),
            call("floated", vec![field(ty)]),
            NirExpr::CastI64ToF64(Box::new(NirExpr::Int(0))),
        ] {
            assert!(epoch
                .unary_argument(&chain(ty, 2, leaf), &scope, &layouts, &scalar_type(ty), 0)
                .is_none());
        }
        let valid = chain(ty, 2, field(ty));
        let mut stale = layouts.clone();
        stale.get_mut("Pair").unwrap()[0].1 = scalar_type("i64");
        assert!(epoch
            .unary_argument(&valid, &scope, &stale, &scalar_type(ty), 0)
            .is_none());
    }
}

#[test]
fn nested_unary_float_print_epoch_keeps_parameter_and_entry_permissions_local() {
    let module = module();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    let mut locals = Locals {
        scope: Scope::from([("live".into(), scalar_type("Outer"))]),
        writable: BTreeSet::from(["live".into()]),
        available: BTreeSet::new(),
    };
    let changed = locals.writable.clone();
    for ty in ["f32", "f64"] {
        for count in [2, 3, 6] {
            let arg = chain(ty, count, field(ty));
            let callee = if ty == "f32" { "narrow" } else { "wide" };
            for child in [false, true] {
                let epoch = if child {
                    Epoch::enter_child(&locals.scope, &changed, &changed, &changed, &layouts)
                } else {
                    Epoch::enter(&locals.scope, &changed, &changed, &layouts)
                };
                let valid = call(callee, vec![arg.clone(), NirExpr::Int(1)]);
                assert!(block(
                    &[NirStmt::Print(valid)],
                    &mut locals,
                    &changed,
                    &catalog,
                    &layouts,
                    validation::LiteralPrints::Scalars,
                    &epoch,
                    0
                )
                .is_some());
                for invalid in [
                    call(callee, vec![NirExpr::Int(1), arg.clone()]),
                    call("floated", vec![arg.clone()]),
                    call("impure", vec![arg.clone()]),
                    if ty == "f32" {
                        NirExpr::PackF32Word(Box::new(arg.clone()))
                    } else {
                        NirExpr::PackF64Word(Box::new(arg.clone()))
                    },
                ] {
                    assert!(block(
                        &[NirStmt::Print(invalid)],
                        &mut locals,
                        &changed,
                        &catalog,
                        &layouts,
                        validation::LiteralPrints::Scalars,
                        &epoch,
                        0
                    )
                    .is_none());
                }
                assert!(locals.available.is_empty());
            }
            let child = Epoch::enter_child(
                &locals.scope,
                &changed,
                &changed,
                &locals.available,
                &layouts,
            );
            assert!(child
                .unary_argument(&arg, &locals.scope, &layouts, &scalar_type(ty), 0)
                .is_none());
            let unowned = Epoch::enter(&locals.scope, &BTreeSet::new(), &changed, &layouts);
            assert!(unowned
                .unary_argument(&arg, &locals.scope, &layouts, &scalar_type(ty), 0)
                .is_none());
        }
    }
}

#[test]
fn nested_unary_float_print_epoch_source_keeps_ordinary_reads_and_ancestors_closed() {
    for ty in ["f32", "f64"] {
        for count in [2, 3, 6] {
            let value = source_chain(count, "live.inner.value".into());
            for body in [
                format!("print(observe({value},1));let saved={value};live=live;index=index+1;"),
                format!("print(observe({value},1));if observe({value},1)==0{{print(1);}}live=live;index=index+1;"),
                format!("let saved=observe({value},1);live=live;index=index+1;"),
                format!("print(observe(identity({value}),1));live=live;index=index+1;"),
                format!("let mut child=0;while child<1{{print(observe({value},1));live=live;child=child+1;}}index=index+1;"),
            ] {
                let module=crate::frontend::parse_nuis_module(&source(ty,&body,false,0)).unwrap();
                let layouts=control_values::layouts(&module);
                assert!(!scalar_helpers::collect_iteration_calls(&module,&layouts).contains_key("walk"),"{ty}/{count}: {body}");
            }
        }
    }
}

fn expand(body: &mut [NirStmt], ty: &str, count: usize) {
    for stmt in body {
        match stmt {
            NirStmt::Print(NirExpr::Call { callee, args }) if callee == "observe" => {
                args[0] = chain(ty, count - 1, args[0].clone());
            }
            NirStmt::If {
                then_body,
                else_body,
                ..
            } => {
                expand(then_body, ty, count);
                expand(else_body, ty, count);
            }
            NirStmt::While { body, .. } => expand(body, ty, count),
            _ => {}
        }
    }
}

#[test]
fn nested_unary_float_print_epoch_raw_nir_preserves_even_odd_versions_and_faults() {
    for ty in ["f32", "f64"] {
        let (mask, words): (u64, &[u64]) = if ty == "f32" {
            (
                1 << 31,
                &[
                    0, 0x80000000, 1, 0x807fffff, 0x7f800000, 0x7fc12345, 0xff800001, 0x3f80005f,
                ],
            )
        } else {
            (
                1 << 63,
                &[
                    0,
                    0x8000000000000000,
                    1,
                    0x800fffffffffffff,
                    0x7ff0000000000000,
                    0x7ff812345678abcd,
                    0xfff0000000000001,
                    0x3ff000000000005f,
                ],
            )
        };
        for count in [2, 3, 6] {
            for &bits in words {
                for leading in [false, true] {
                    for selected in [false, true] {
                        let effect="if selected{print(observe(-live.inner.value,divisor));}print(observe(-live.inner.value,1));live=Outer{inner:Pair{value:-live.inner.value,total:0}};print(observe(-live.inner.value,1));";
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
                        for function in &mut module.functions {
                            expand(&mut function.body, ty, count);
                        }
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
                        let yir = crate::lowering::bootstrap::lower_nir_to_yir_builtin_cpu(&module)
                            .unwrap();
                        let trace = yir_runtime_host::execute_module_source_with_registry(
                            &crate::render::render_yir(&yir),
                            &yir_verify::default_registry(),
                        )
                        .unwrap();
                        let class = |word| {
                            let value = if ty == "f32" {
                                f64::from(f32::from_bits(word as u32))
                            } else {
                                f64::from_bits(word)
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
                        let sign = if count % 2 == 0 { 0 } else { mask };
                        let mut current = bits;
                        let mut expected = Vec::new();
                        for _ in 0..2 {
                            if selected {
                                expected.push(class(current ^ sign));
                            }
                            expected.push(class(current ^ sign));
                            current ^= mask;
                            expected.push(class(current ^ sign));
                        }
                        assert_eq!(prints(&trace.events), expected);
                        yir_lower_llvm::emit_module(&yir).unwrap();
                    }
                }
            }
            for zero in [false, true] {
                let body = format!(
                    "if selected{{print(observe({},divisor));}}live=live;index=index+1;",
                    source_chain(count, "live.inner.value".into())
                );
                let text = source(ty, &body, true, 0)
                    .replace("index<2", if zero { "index<0" } else { "index<2" });
                let module = crate::frontend::parse_nuis_module(&text).unwrap();
                let yir =
                    crate::lowering::bootstrap::lower_nir_to_yir_builtin_cpu(&module).unwrap();
                let result = yir_runtime_host::execute_module_source_with_registry(
                    &crate::render::render_yir(&yir),
                    &yir_verify::default_registry(),
                );
                if zero {
                    assert!(prints(&result.unwrap().events).is_empty());
                } else {
                    assert!(result.unwrap_err().contains("zero"));
                }
            }
        }
    }
}

#[test]
fn nested_unary_float_print_epoch_retains_shared_root_work_budget() {
    for ty in ["f32", "f64"] {
        let print = NirStmt::Print(call(
            if ty == "f32" { "narrow" } else { "wide" },
            vec![chain(ty, 4, field(ty)), NirExpr::Int(1)],
        ));
        assert!(iteration_prints::bounded(
            &vec![print.clone(); 100],
            &ScalarHelpers::new()
        ));
        assert!(!iteration_prints::bounded(
            &vec![print; 256],
            &ScalarHelpers::new()
        ));
    }
}
