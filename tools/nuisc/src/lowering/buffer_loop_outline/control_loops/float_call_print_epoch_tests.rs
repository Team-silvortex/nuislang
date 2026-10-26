use super::*;

fn calls_module() -> NirModule {
    crate::frontend::parse_nuis_module("mod cpu Main {
        struct Pair{small:f32,wide:f64,total:i64,flag:bool,tag:i32} struct Outer{inner:Pair}
        fn id32(value:f32)->f32{return value;}
        fn id64(value:f64)->f64{return value;}
        fn choose(value:f64,flag:bool,guard:i64)->f64{return value;}
        fn observe(value:f64)->i64{return 0;}
        fn mixed(value:f64,flag:bool,tag:i32,guard:i64,narrow:f32)->f64{let checked=0/guard;return value;}
        fn record(value:Outer)->f64{return value.inner.wide;}
        fn wrong(value:f64)->i64{print(9);return 0;}
        fn effect(value:f64)->f64{print(9);return value;}
        fn transit(value:f64)->f64{return effect(value);}
        fn cycle_a(value:f64)->f64{return cycle_b(value);}
        fn cycle_b(value:f64)->f64{return cycle_a(value);}
        fn main()->i64{return 0;}
    }").unwrap()
}

fn projected(name: &str) -> NirExpr {
    crate::lowering::scalar_record_shape::source_value("live", &["inner".into(), name.into()])
}
fn scope() -> Scope {
    Scope::from([("live".into(), scalar_type("Outer"))])
}
fn mixed() -> NirExpr {
    call(
        "mixed",
        vec![
            projected("wide"),
            projected("flag"),
            projected("tag"),
            projected("total"),
            projected("small"),
        ],
    )
}

#[test]
fn float_call_print_epoch_requires_complete_pure_exact_scalar_signatures() {
    let module = calls_module();
    let layouts = control_values::layouts(&module);
    for reverse in [false, true] {
        let mut module = module.clone();
        if reverse {
            module.functions.reverse();
            module.structs.reverse();
        }
        let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
        assert_eq!(
            scalar_helpers::pure_float_parameters("mixed", &scalar_type("f64"), &catalog),
            Some(
                [
                    scalar_type("f64"),
                    scalar_type("bool"),
                    scalar_type("i32"),
                    scalar_type("i64"),
                    scalar_type("f32")
                ]
                .as_slice()
            )
        );
        for callee in [
            "record", "wrong", "effect", "transit", "cycle_a", "cycle_b", "missing",
        ] {
            assert!(
                scalar_helpers::pure_float_parameters(callee, &scalar_type("f64"), &catalog)
                    .is_none(),
                "{callee}"
            );
        }
        assert!(
            scalar_helpers::pure_float_parameters("id32", &scalar_type("f64"), &catalog).is_none()
        );
        let mut qualified = scalar_type("f64");
        qualified.is_ref = true;
        assert!(scalar_helpers::pure_float_parameters("id64", &qualified, &catalog).is_none());
    }
}

#[test]
fn float_call_print_epoch_composes_nested_calls_and_checks_every_parameter_read() {
    let module = calls_module();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    let locals = Locals {
        scope: scope(),
        writable: BTreeSet::from(["live".into()]),
        available: BTreeSet::new(),
    };
    let changed = locals.writable.clone();
    let epoch = Epoch::enter(&locals.scope, &changed, &changed, &layouts);
    let value = call(
        "observe",
        vec![negate(
            "f64",
            NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(call("id64", vec![mixed()])),
                rhs: Box::new(projected("wide")),
            },
        )],
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
        ),
        Some(scalar_type("i64"))
    );
    assert!(expression(&value, &locals, &changed, None, &catalog, &layouts, 0).is_none());
    let NirExpr::Call { args, .. } = mixed() else {
        unreachable!()
    };
    for slot in 0..args.len() {
        let mut invalid = args.clone();
        invalid[slot] = NirExpr::Var("missing".into());
        let value = call("observe", vec![call("mixed", invalid)]);
        assert!(
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
            .is_none(),
            "slot {slot}"
        );
    }
    for (slot, replacement) in [
        (0, projected("small")),
        (1, projected("tag")),
        (2, projected("total")),
        (3, projected("flag")),
        (4, projected("wide")),
    ] {
        let mut invalid = args.clone();
        invalid[slot] = replacement;
        let value = call("observe", vec![call("mixed", invalid)]);
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
    for arg in [
        call("id64", vec![]),
        call("id64", vec![projected("wide"), projected("wide")]),
        call("id32", vec![projected("wide")]),
        call("effect", vec![projected("wide")]),
        call("record", vec![NirExpr::Var("live".into())]),
    ] {
        let value = call("observe", vec![arg]);
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

#[test]
fn float_call_print_epoch_preserves_child_entry_layout_and_single_print_scope() {
    let module = calls_module();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    let mut locals = Locals {
        scope: scope(),
        writable: BTreeSet::from(["live".into()]),
        available: BTreeSet::new(),
    };
    let changed = locals.writable.clone();
    let value = call("observe", vec![mixed()]);
    for child in [false, true] {
        let epoch = if child {
            Epoch::enter_child(&locals.scope, &changed, &changed, &changed, &layouts)
        } else {
            Epoch::enter(&locals.scope, &changed, &changed, &layouts)
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
        assert!(locals.available.is_empty());
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
    }
    for epoch in [
        Epoch::enter_child(
            &locals.scope,
            &changed,
            &changed,
            &locals.available,
            &layouts,
        ),
        Epoch::enter(&locals.scope, &BTreeSet::new(), &changed, &layouts),
    ] {
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

#[test]
fn float_call_print_epoch_counts_call_depth_shared_work_and_parameter_order() {
    let module = calls_module();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    let expected = scalar_type("f64");
    let mut chain = projected("wide");
    for count in 1..=66 {
        chain = call("id64", vec![chain]);
        assert_eq!(
            binary_float::validate(&chain, &expected, 0, &catalog, |_, _| Some(
                expected.clone()
            ))
            .is_some(),
            count + 2 <= 64
        );
    }
    let value = mixed();
    let mut order = Vec::new();
    assert!(
        binary_float::validate(&value, &expected, 0, &catalog, |value, _| {
            let ty = control_values::value_type(value, &scope(), &catalog, &layouts)?;
            order.push(ty.name.clone());
            Some(ty)
        })
        .is_some()
    );
    assert_eq!(order, ["f64", "bool", "i32", "i64", "f32"]);
    let mut checked = 0;
    assert!(
        binary_float::validate(&value, &expected, 0, &catalog, |_, _| {
            checked += 1;
            None
        })
        .is_none()
    );
    assert_eq!(checked, 1);
    let mut tree = call("id64", vec![NirExpr::F64("1.0".into())]);
    for _ in 0..12 {
        tree = NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(tree.clone()),
            rhs: Box::new(tree),
        };
    }
    assert!(
        binary_float::validate(&tree, &expected, 0, &catalog, |_, _| Some(expected.clone()))
            .is_none()
    );
    assert!(
        binary_float::validate(&value, &expected, usize::MAX, &catalog, |_, _| Some(
            expected.clone()
        ))
        .is_none()
    );
}

fn source(ty: &str, body: &str, selected: bool, first: i64, second: i64) -> String {
    format!("mod cpu Main{{struct Pair{{value:{ty}}} struct Outer{{inner:Pair}} @noinline fn identity(value:{ty},guard:i64)->{ty}{{let checked=0/guard;return value;}} @noinline fn scale(value:{ty},factor:{ty},guard:i64)->{ty}{{let checked=0/guard;return value*factor;}} @noinline fn observe(value:{ty})->i64{{if value<0.0{{return -1;}}if value==0.0{{return 0;}}if value!=value{{return 2;}}return 1;}} @noinline fn walk(seed:{ty},selected:bool,first:i64,second:i64)->{ty}{{let mut index=0;let mut live=Outer{{inner:Pair{{value:seed}}}};while index<2{{{body}}}return live.inner.value;}}fn main()->{ty}{{return walk(1.5,{selected},{first},{second});}}}}")
}

#[test]
fn float_call_print_epoch_source_keeps_ordinary_reads_and_ancestor_calls_closed() {
    for ty in ["f32", "f64"] {
        for body in [
            "print(observe(identity(live.inner.value,1)));let saved=identity(live.inner.value,1);live=live;index=index+1;",
            "if observe(identity(live.inner.value,1))==0{print(1);}live=live;index=index+1;",
            "let saved=observe(identity(live.inner.value,1));live=live;index=index+1;",
            "let mut child=0;while child<1{print(observe(identity(live.inner.value,1)));live=live;child=child+1;}index=index+1;",
        ] {
            let module=crate::frontend::parse_nuis_module(&source(ty,body,false,0,0)).unwrap();let layouts=control_values::layouts(&module);
            assert!(!scalar_helpers::collect_iteration_calls(&module,&layouts).contains_key("walk"),"{ty}: {body}");
        }
    }
}

#[test]
fn float_call_print_epoch_raw_nir_keeps_selected_faults_and_ieee_argument_values() {
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
            for leading in [false, true] {
                for selected in [false, true] {
                    let expr = "identity(scale(identity(live.inner.value+2.0,1),2.0,1)/2.0,1)";
                    let effect=format!("if selected{{print(observe(identity(live.inner.value,first)));}}print(observe({expr}));live=Outer{{inner:Pair{{value:-live.inner.value}}}};print(observe({expr}));");
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
                        1,
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
                    let class = |bits, computed| {
                        let value = if ty == "f32" {
                            let v = f32::from_bits(bits as u32);
                            f64::from(if computed { ((v + 2.0) * 2.0) / 2.0 } else { v })
                        } else {
                            let v = f64::from_bits(bits);
                            if computed {
                                ((v + 2.0) * 2.0) / 2.0
                            } else {
                                v
                            }
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
                            expected.push(class(current, false));
                        }
                        expected.push(class(current, true));
                        current ^= mask;
                        expected.push(class(current, true));
                    }
                    assert_eq!(prints(&trace.events), expected);
                    yir_lower_llvm::emit_module(&yir).unwrap();
                }
            }
        }
        for zero in [false, true] {
            let text=source(ty,"if selected{print(observe(scale(identity(live.inner.value,first),identity(2.0,second),1)));}live=live;index=index+1;",true,0,0).replace("index<2",if zero{"index<0"}else{"index<2"});
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
        }
    }
}

#[path = "computed_call_argument_print_epoch_tests.rs"]
mod computed_arguments;
