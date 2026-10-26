use super::*;

#[test]
fn lazy_call_argument_print_epoch_bounds_complete_rhs_capture_union() {
    let module = calls_module();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    let mut locals = Locals {
        scope: scope(),
        writable: BTreeSet::from(["live".into()]),
        available: BTreeSet::new(),
    };
    for i in 0..32 {
        locals.scope.insert(format!("v{i}"), scalar_type("i64"));
    }
    let changed = locals.writable.clone();
    let epoch = Epoch::enter(&locals.scope, &changed, &changed, &layouts);
    for count in [30, 31] {
        let mut sum = projected("total");
        for i in 0..count {
            sum = binary(NirBinaryOp::Add, sum, NirExpr::Var(format!("v{i}")));
        }
        let rhs = binary(
            NirBinaryOp::Gt,
            binary(NirBinaryOp::Div, sum, NirExpr::Var("v0".into())),
            NirExpr::Int(0),
        );
        for op in [NirBinaryOp::And, NirBinaryOp::Or] {
            let choose = call(
                "choose",
                vec![
                    projected("wide"),
                    binary(op, projected("flag"), rhs.clone()),
                    NirExpr::Int(1),
                ],
            );
            let value = call("observe", vec![choose]);
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
                count == 30
            );
            assert!(expression(&value, &locals, &changed, None, &catalog, &layouts, 0).is_none());
        }
    }
    let unknown = binary(
        NirBinaryOp::And,
        NirExpr::Bool(false),
        binary(
            NirBinaryOp::Gt,
            binary(NirBinaryOp::Div, projected("missing"), NirExpr::Int(0)),
            NirExpr::Int(0),
        ),
    );
    assert!(admitted(&argument(unknown, NirExpr::Int(1)), false, true).is_none());
}

fn source(ty: &str, gate: bool, divisor: i64, leading: bool, op: &str, negative: bool) -> String {
    let old = "(live.inner.tag/first>=50&&live.inner.flag)||live.inner.tag==99";
    let predicate = match op {
        "and" => "live.inner.flag&&(live.inner.tag/first>0)",
        "or" => "live.inner.flag||(live.inner.tag%first>0)",
        "nested" => "live.inner.flag&&((live.inner.tag/first>0)||(live.inner.tag%(first-2)>0))",
        _ => panic!(),
    };
    let mut text = program(ty, gate, divisor, 1, leading, false).replace(old, predicate);
    if negative {
        text = text.replace("observe(choose(", "observe(-choose(");
    }
    text
}

#[test]
fn lazy_call_argument_print_epoch_raw_nir_keeps_both_short_circuit_edges_under_sign_wrappers() {
    for ty in ["f32", "f64"] {
        let words: &[u64] = if ty == "f32" {
            &[
                0, 0x80000000, 1, 0x3fc00000, 0x7f800000, 0x7fc12345, 0xff800001,
            ]
        } else {
            &[
                0,
                0x8000000000000000,
                1,
                0x3ff8000000000000,
                0x7ff0000000000000,
                0x7ff812345678abcd,
                0xfff0000000000001,
            ]
        };
        for op in ["and", "or", "nested"] {
            for gate in [false, true] {
                for negative in [false, true] {
                    for leading in [false, true] {
                        for &bits in words {
                            let mut module = crate::frontend::parse_nuis_module(&source(
                                ty, gate, 2, leading, op, negative,
                            ))
                            .unwrap();
                            let main = module
                                .functions
                                .iter_mut()
                                .find(|f| f.name == "main")
                                .unwrap();
                            let NirStmt::Return(Some(NirExpr::Call { args, .. })) =
                                &mut main.body[0]
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
                            assert!(scalar_helpers::collect_iteration_calls(&module, &layouts)
                                .contains_key("walk"));
                            let mut outlined = module.clone();
                            crate::lowering::buffer_loop_outline::outline_buffer_loops(
                                &mut outlined,
                            )
                            .unwrap();
                            let predicates = outlined
                                .functions
                                .iter()
                                .filter(|f| f.name.starts_with("__nuis_scalar_predicate"))
                                .collect::<Vec<_>>();
                            assert!(!predicates.is_empty());
                            assert!(predicates.iter().all(|f| f.params.iter().all(|p| matches!(
                                p.ty.name.as_str(),
                                "bool" | "i32" | "i64" | "f32" | "f64"
                            ))));
                            let yir =
                                crate::lowering::bootstrap::lower_nir_to_yir_builtin_cpu(&module)
                                    .unwrap();
                            let trace = yir_runtime_host::execute_module_source_with_registry(
                                &crate::render::render_yir(&yir),
                                &yir_verify::default_registry(),
                            )
                            .unwrap();
                            let mut expected = Vec::new();
                            for index in 0..2 {
                                let pick = match op {
                                    "and" => gate && (100 + index) / 2 > 0,
                                    "or" => gate || (100 + index) % 2 > 0,
                                    "nested" => {
                                        gate && ((100 + index) / 2 > 0 || panic!("poison reached"))
                                    }
                                    _ => panic!(),
                                };
                                let v = if ty == "f32" {
                                    let v = f32::from_bits(bits as u32);
                                    f64::from((if index == 0 { v } else { -v }) + v)
                                } else {
                                    let v = f64::from_bits(bits);
                                    (if index == 0 { v } else { -v }) + v
                                };
                                let v = if pick { v } else { -v };
                                let v = if negative { -v } else { v };
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
            }
        }
    }
}

#[test]
fn lazy_call_argument_print_epoch_reaches_only_selected_zero_and_overflow_faults() {
    for ty in ["f32", "f64"] {
        for op in ["and", "or", "nested"] {
            for gate in [false, true] {
                for overflow in [false, true] {
                    let mut text = source(ty, gate, if overflow { -1 } else { 0 }, false, op, true);
                    if overflow {
                        text = text.replace("tag:100", "tag:(-9223372036854775807-1)");
                    }
                    let module = crate::frontend::parse_nuis_module(&text).unwrap();
                    let yir =
                        crate::lowering::bootstrap::lower_nir_to_yir_builtin_cpu(&module).unwrap();
                    let result = yir_runtime_host::execute_module_source_with_registry(
                        &crate::render::render_yir(&yir),
                        &yir_verify::default_registry(),
                    );
                    let reached = if op == "or" { !gate } else { gate };
                    if reached {
                        assert!(result.is_err(), "{ty} {op} {overflow}");
                    } else {
                        assert_eq!(prints(&result.unwrap().events).len(), 2);
                    }
                }
            }
        }
    }
}
