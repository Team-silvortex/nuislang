use super::*;

fn module() -> NirModule {
    crate::frontend::parse_nuis_module(
        "mod cpu Main {
        struct Pair{small:f32,wide:f64,total:i64} struct Outer{inner:Pair}
        fn narrow(value:f32,d:i64)->i64{return 0/d;}
        fn wide(value:f64,d:i64)->i64{return 0/d;}
        fn floated(value:f64)->f64{print(9);return value;}
        fn impure(value:f64)->i64{print(9);return 0;}
        fn main()->i64{return 0;}
    }",
    )
    .unwrap()
}

fn field(ty: &str) -> NirExpr {
    crate::lowering::scalar_record_shape::source_value(
        "live",
        &[
            "inner".into(),
            if ty == "f32" { "small" } else { "wide" }.into(),
        ],
    )
}

fn negate(ty: &str, leaf: NirExpr) -> NirExpr {
    let packed = if ty == "f32" {
        NirExpr::PackF32Word(Box::new(leaf))
    } else {
        NirExpr::PackF64Word(Box::new(leaf))
    };
    let word = Box::new(NirExpr::Binary {
        op: NirBinaryOp::Xor,
        lhs: Box::new(packed),
        rhs: Box::new(NirExpr::Int(if ty == "f32" {
            1_i64 << 31
        } else {
            i64::MIN
        })),
    });
    if ty == "f32" {
        NirExpr::UnpackF32Word(word)
    } else {
        NirExpr::UnpackF64Word(word)
    }
}

fn call(callee: &str, args: Vec<NirExpr>) -> NirExpr {
    NirExpr::Call {
        callee: callee.into(),
        args,
    }
}

#[test]
fn unary_float_print_epoch_requires_exact_private_sign_and_original_field() {
    let module = module();
    let layouts = control_values::layouts(&module);
    let scope = Scope::from([("live".into(), scalar_type("Outer"))]);
    let changed = BTreeSet::from(["live".into()]);
    let epoch = Epoch::enter(&scope, &changed, &changed, &layouts);
    for ty in ["f32", "f64"] {
        let expected = scalar_type(ty);
        let valid = negate(ty, field(ty));
        assert_eq!(
            epoch.unary_argument(&valid, &scope, &layouts, &expected, 59),
            Some(expected.clone())
        );
        for depth in [60, 64, usize::MAX] {
            assert!(epoch
                .unary_argument(&valid, &scope, &layouts, &expected, depth)
                .is_none());
        }
        let mut qualified = expected.clone();
        qualified.is_ref = true;
        assert!(epoch
            .unary_argument(&valid, &scope, &layouts, &qualified, 0)
            .is_none());
        let wrong = if ty == "f32" { "f64" } else { "f32" };
        let mut malformed = vec![
            negate(wrong, field(ty)),
            negate(ty, field(wrong)),
            negate(ty, negate(wrong, field(ty))),
            negate(ty, NirExpr::Var("live".into())),
            negate(
                ty,
                NirExpr::Binary {
                    op: NirBinaryOp::Add,
                    lhs: Box::new(field(ty)),
                    rhs: Box::new(field(ty)),
                },
            ),
            negate(ty, call("floated", vec![field(ty)])),
        ];
        for change in 0..4 {
            let mut candidate = valid.clone();
            let word = match &mut candidate {
                NirExpr::UnpackF32Word(word) | NirExpr::UnpackF64Word(word) => word,
                _ => unreachable!(),
            };
            let NirExpr::Binary { op, lhs, rhs } = word.as_mut() else {
                unreachable!()
            };
            match change {
                0 => *rhs = Box::new(NirExpr::Int(1)),
                1 => {
                    *rhs = Box::new(NirExpr::Int(if ty == "f32" {
                        i64::MIN
                    } else {
                        1_i64 << 31
                    }))
                }
                2 => *op = NirBinaryOp::Add,
                _ => std::mem::swap(lhs, rhs),
            }
            malformed.push(candidate);
        }
        for candidate in malformed {
            assert!(
                epoch
                    .unary_argument(&candidate, &scope, &layouts, &expected, 0)
                    .is_none(),
                "{candidate:?}"
            );
        }
        let mut stale = layouts.clone();
        stale.get_mut("Pair").unwrap()[0].1 = scalar_type("i64");
        assert!(epoch
            .unary_argument(&valid, &scope, &stale, &expected, 0)
            .is_none());
        let mut stale = scope.clone();
        stale.get_mut("live").unwrap().is_ref = true;
        assert!(epoch
            .unary_argument(&valid, &stale, &layouts, &expected, 0)
            .is_none());
    }
}

#[test]
fn unary_float_print_epoch_retains_signature_scope_and_ordinary_read_vetoes() {
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
        let callee = if ty == "f32" { "narrow" } else { "wide" };
        let arg = negate(ty, field(ty));
        let valid = call(callee, vec![arg.clone(), NirExpr::Int(1)]);
        for child in [false, true] {
            let epoch = if child {
                Epoch::enter_child(&locals.scope, &changed, &changed, &changed, &layouts)
            } else {
                Epoch::enter(&locals.scope, &changed, &changed, &layouts)
            };
            assert!(block(
                &[NirStmt::Print(valid.clone())],
                &mut locals,
                &changed,
                &catalog,
                &layouts,
                validation::LiteralPrints::Scalars,
                &epoch,
                0
            )
            .is_some());
            assert!(expression(&valid, &locals, &changed, None, &catalog, &layouts, 0).is_none());
            for invalid in [
                call(callee, vec![arg.clone()]),
                call(callee, vec![NirExpr::Int(1), arg.clone()]),
                call(
                    if ty == "f32" { "wide" } else { "narrow" },
                    vec![arg.clone(), NirExpr::Int(1)],
                ),
                call("floated", vec![arg.clone()]),
                call("impure", vec![arg.clone()]),
                if ty == "f32" {
                    NirExpr::PackF32Word(Box::new(arg.clone()))
                } else {
                    NirExpr::PackF64Word(Box::new(arg.clone()))
                },
            ] {
                assert!(
                    block(
                        &[NirStmt::Print(invalid.clone())],
                        &mut locals,
                        &changed,
                        &catalog,
                        &layouts,
                        validation::LiteralPrints::Scalars,
                        &epoch,
                        0
                    )
                    .is_none(),
                    "{invalid:?}"
                );
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
    }
}

fn source(ty: &str, body: &str, selected: bool, divisor: i64) -> String {
    format!("mod cpu Main{{struct Pair{{value:{ty},total:i64}} struct Outer{{inner:Pair}} @noinline fn observe(value:{ty},d:i64)->i64{{if value<0.0{{return -1+0/d;}}if value==0.0{{return 0/d;}}if value!=value{{return 2+0/d;}}return 1+0/d;}} @noinline fn identity(value:{ty})->{ty}{{print(9);return value;}} @noinline fn walk(seed:{ty},selected:bool,divisor:i64)->{ty}{{let mut index=0;let mut live=Outer{{inner:Pair{{value:seed,total:0}}}};while index<2{{{body}}}return live.inner.value;}}fn main()->{ty}{{return walk(1.5,{selected},{divisor});}}}}")
}

#[test]
fn unary_float_print_epoch_source_keeps_compounds_and_ancestors_closed() {
    for ty in ["f32", "f64"] {
        for body in [
            "print(observe(-live.inner.value,1));let saved=-live.inner.value;live=live;index=index+1;",
            "print(observe(-live.inner.value,1));if -live.inner.value<0.0{print(1);}live=live;index=index+1;",
            "let saved=observe(-live.inner.value,1);live=live;index=index+1;",
            "print(observe(identity(-live.inner.value),1));live=live;index=index+1;",
            "print(observe(-identity(live.inner.value),1));live=live;index=index+1;",
            "let mut child=0;while child<1{print(observe(-live.inner.value,1));live=live;child=child+1;}index=index+1;",
        ] {
            let module=crate::frontend::parse_nuis_module(&source(ty,body,false,0)).unwrap();
            let layouts=control_values::layouts(&module);
            assert!(!scalar_helpers::collect_iteration_calls(&module,&layouts).contains_key("walk"),"{ty} {body}");
        }
    }
}

fn prints(events: &[String]) -> Vec<i64> {
    events
        .iter()
        .filter(|e| {
            e.contains("cpu.print")
                || (e.contains("cpu.guard_print") && e.contains("if true then print"))
        })
        .map(|e| e.rsplit_once(' ').unwrap().1.parse().unwrap())
        .collect()
}

#[path = "nested_unary_float_print_epoch_tests.rs"]
mod nested;

#[path = "binary_float_print_epoch_tests.rs"]
mod binary;

#[path = "float_call_print_epoch_tests.rs"]
mod float_calls;

#[test]
fn unary_float_print_epoch_raw_nir_keeps_width_sign_payload_and_source_order() {
    for ty in ["f32", "f64"] {
        let (mask, words): (u64, &[u64]) = if ty == "f32" {
            (
                1 << 31,
                &[
                    0, 0x80000000, 0x3fc00000, 0xc0200000, 1, 0x807fffff, 0x7f800000, 0xff800000,
                    0x7fc12345, 0xff800001, 0x3f80005e, 0x3f80005f,
                ],
            )
        } else {
            (
                1 << 63,
                &[
                    0,
                    0x8000000000000000,
                    0x3ff8000000000000,
                    0xc004000000000000,
                    1,
                    0x800fffffffffffff,
                    0x7ff0000000000000,
                    0xfff0000000000000,
                    0x7ff812345678abcd,
                    0xfff0000000000001,
                    0x3ff000000000005e,
                    0x3ff000000000005f,
                ],
            )
        };
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
                    let layouts = control_values::layouts(&module);
                    assert!(scalar_helpers::collect_iteration_calls(&module, &layouts)
                        .contains_key("walk"));
                    crate::nir_verify::verify_nir_module(&module).unwrap();
                    let yir =
                        crate::lowering::bootstrap::lower_nir_to_yir_builtin_cpu(&module).unwrap();
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
                    let mut current = bits;
                    let mut expected = Vec::new();
                    for _ in 0..2 {
                        if selected {
                            expected.push(class(current ^ mask));
                        }
                        expected.push(class(current ^ mask));
                        current ^= mask;
                        expected.push(class(current ^ mask));
                    }
                    assert_eq!(prints(&trace.events), expected);
                    yir_lower_llvm::emit_module(&yir).unwrap();
                }
            }
        }
    }
}

#[test]
fn unary_float_print_epoch_zero_trips_skip_but_selected_calls_fail() {
    for ty in ["f32", "f64"] {
        for zero in [false, true] {
            let text = source(
                ty,
                "if selected{print(observe(-live.inner.value,divisor));}live=live;index=index+1;",
                true,
                0,
            )
            .replace("index<2", if zero { "index<0" } else { "index<2" });
            let module = crate::frontend::parse_nuis_module(&text).unwrap();
            let yir = crate::lowering::bootstrap::lower_nir_to_yir_builtin_cpu(&module).unwrap();
            let result = yir_runtime_host::execute_module_source_with_registry(
                &crate::render::render_yir(&yir),
                &yir_verify::default_registry(),
            );
            if zero {
                assert!(prints(&result.unwrap().events).is_empty());
            } else {
                assert!(result.unwrap_err().contains("zero"));
            }
            yir_lower_llvm::emit_module(&yir).unwrap();
        }
    }
}
