use super::simplify_nir_module;
use nuis_semantics::model::{
    NirAnnotation, NirBinaryOp, NirExpr, NirFunction, NirModule, NirParam, NirStmt, NirTypeRef,
    NirVisibility,
};

fn i64_type() -> NirTypeRef {
    NirTypeRef {
        name: "i64".to_owned(),
        generic_args: vec![],
        is_optional: false,
        is_ref: false,
    }
}

fn sample_module(body: Vec<NirStmt>) -> NirModule {
    NirModule {
        annotations: vec![],
        uses: vec![],
        domain: "cpu".to_owned(),
        unit: "Main".to_owned(),
        externs: vec![],
        extern_interfaces: vec![],
        consts: vec![],
        type_aliases: vec![],
        structs: vec![],
        enums: vec![],
        traits: vec![],
        impls: vec![],
        functions: vec![NirFunction {
            name: "main".to_owned(),
            annotations: vec![],
            visibility: NirVisibility::Private,
            test_name: None,
            test_ignored: false,
            test_should_fail: false,
            test_reason: None,
            test_timeout_ms: None,
            test_clock_domain: None,
            test_clock_policy: None,
            benchmark_name: None,
            benchmark_warmup_iters: None,
            benchmark_measure_iters: None,
            benchmark_timeout_ms: None,
            benchmark_clock_domain: None,
            benchmark_clock_policy: None,
            is_async: false,
            generic_params: vec![],
            where_bounds: vec![],
            params: vec![],
            return_type: None,
            body,
        }],
    }
}

fn annotation(name: &str) -> NirAnnotation {
    NirAnnotation {
        name: name.to_owned(),
        args: vec![],
    }
}

#[test]
fn folds_integer_binary_constants() {
    let mut module = sample_module(vec![NirStmt::Return(Some(NirExpr::Binary {
        op: NirBinaryOp::Add,
        lhs: Box::new(NirExpr::Int(2)),
        rhs: Box::new(NirExpr::Int(3)),
    }))]);
    let changed = simplify_nir_module(&mut module);
    assert!(changed);
    assert_eq!(
        module.functions[0].body,
        vec![NirStmt::Return(Some(NirExpr::Int(5)))]
    );
}

#[test]
fn folds_integer_comparison_constants() {
    let mut module = sample_module(vec![NirStmt::Return(Some(NirExpr::Binary {
        op: NirBinaryOp::Lt,
        lhs: Box::new(NirExpr::Int(2)),
        rhs: Box::new(NirExpr::Int(5)),
    }))]);
    let changed = simplify_nir_module(&mut module);
    assert!(changed);
    assert_eq!(
        module.functions[0].body,
        vec![NirStmt::Return(Some(NirExpr::Int(1)))]
    );
}

#[test]
fn normalizes_if_true_into_then_branch() {
    let mut module = sample_module(vec![NirStmt::If {
        condition: NirExpr::Bool(true),
        then_body: vec![NirStmt::Return(Some(NirExpr::Int(1)))],
        else_body: vec![NirStmt::Return(Some(NirExpr::Int(0)))],
    }]);
    let changed = simplify_nir_module(&mut module);
    assert!(changed);
    assert_eq!(
        module.functions[0].body,
        vec![NirStmt::Return(Some(NirExpr::Int(1)))]
    );
}

#[test]
fn folds_is_null_of_null() {
    let mut module = sample_module(vec![NirStmt::Return(Some(NirExpr::IsNull(Box::new(
        NirExpr::Null,
    ))))]);
    let changed = simplify_nir_module(&mut module);
    assert!(changed);
    assert_eq!(
        module.functions[0].body,
        vec![NirStmt::Return(Some(NirExpr::Bool(true)))]
    );
}

#[test]
fn propagates_literal_bindings_into_later_expressions() {
    let mut module = sample_module(vec![
        NirStmt::Let {
            name: "base".to_owned(),
            ty: None,
            value: NirExpr::Int(2),
        },
        NirStmt::Return(Some(NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(NirExpr::Var("base".to_owned())),
            rhs: Box::new(NirExpr::Int(3)),
        })),
    ]);
    let changed = simplify_nir_module(&mut module);
    assert!(changed);
    assert_eq!(
        module.functions[0].body,
        vec![NirStmt::Return(Some(NirExpr::Int(5)))]
    );
}

#[test]
fn prunes_dead_scalar_binding_after_constant_propagation() {
    let mut module = sample_module(vec![
        NirStmt::Let {
            name: "base".to_owned(),
            ty: None,
            value: NirExpr::Int(2),
        },
        NirStmt::Print(NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(NirExpr::Var("base".to_owned())),
            rhs: Box::new(NirExpr::Int(3)),
        }),
    ]);
    let changed = simplify_nir_module(&mut module);
    assert!(changed);
    assert_eq!(
        module.functions[0].body,
        vec![NirStmt::Print(NirExpr::Int(5))]
    );
}

#[test]
fn does_not_propagate_outer_literal_into_while_condition_or_body() {
    let mut module = sample_module(vec![
        NirStmt::Let {
            name: "value".to_owned(),
            ty: None,
            value: NirExpr::Int(0),
        },
        NirStmt::While {
            condition: NirExpr::Binary {
                op: NirBinaryOp::Lt,
                lhs: Box::new(NirExpr::Var("value".to_owned())),
                rhs: Box::new(NirExpr::Int(4)),
            },
            body: vec![NirStmt::Let {
                name: "value".to_owned(),
                ty: None,
                value: NirExpr::Binary {
                    op: NirBinaryOp::Add,
                    lhs: Box::new(NirExpr::Var("value".to_owned())),
                    rhs: Box::new(NirExpr::Int(1)),
                },
            }],
        },
        NirStmt::Print(NirExpr::Var("value".to_owned())),
        NirStmt::Return(Some(NirExpr::Var("value".to_owned()))),
    ]);
    let _changed = simplify_nir_module(&mut module);
    let NirStmt::While { condition, body } = &module.functions[0].body[1] else {
        panic!("expected while statement to remain in place");
    };
    assert_eq!(
        condition,
        &NirExpr::Binary {
            op: NirBinaryOp::Lt,
            lhs: Box::new(NirExpr::Var("value".to_owned())),
            rhs: Box::new(NirExpr::Int(4)),
        }
    );
    let NirStmt::Let { value, .. } = &body[0] else {
        panic!("expected loop body binding");
    };
    assert_eq!(
        value,
        &NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(NirExpr::Var("value".to_owned())),
            rhs: Box::new(NirExpr::Int(1)),
        }
    );
    assert_eq!(
        module.functions[0].body[2],
        NirStmt::Print(NirExpr::Var("value".to_owned()))
    );
    assert_eq!(
        module.functions[0].body[3],
        NirStmt::Return(Some(NirExpr::Var("value".to_owned())))
    );
}

#[test]
fn keeps_dead_binding_with_side_effectful_value() {
    let mut module = sample_module(vec![NirStmt::Let {
        name: "task".to_owned(),
        ty: None,
        value: NirExpr::CpuExternCall {
            abi: "c".to_owned(),
            interface: None,
            callee: "host_side_effect".to_owned(),
            args: vec![],
        },
    }]);
    let changed = simplify_nir_module(&mut module);
    assert!(!changed);
    assert_eq!(module.functions[0].body.len(), 1);
}

#[test]
fn preserves_branch_local_carry_updates_inside_while() {
    let mut module = sample_module(vec![
        NirStmt::Let {
            name: "value".to_owned(),
            ty: None,
            value: NirExpr::Int(0),
        },
        NirStmt::Let {
            name: "acc".to_owned(),
            ty: None,
            value: NirExpr::Int(0),
        },
        NirStmt::While {
            condition: NirExpr::Binary {
                op: NirBinaryOp::Lt,
                lhs: Box::new(NirExpr::Var("value".to_owned())),
                rhs: Box::new(NirExpr::Int(5)),
            },
            body: vec![
                NirStmt::Let {
                    name: "value".to_owned(),
                    ty: None,
                    value: NirExpr::Binary {
                        op: NirBinaryOp::Add,
                        lhs: Box::new(NirExpr::Var("value".to_owned())),
                        rhs: Box::new(NirExpr::Int(1)),
                    },
                },
                NirStmt::If {
                    condition: NirExpr::Binary {
                        op: NirBinaryOp::Gt,
                        lhs: Box::new(NirExpr::Var("value".to_owned())),
                        rhs: Box::new(NirExpr::Int(2)),
                    },
                    then_body: vec![NirStmt::Let {
                        name: "acc".to_owned(),
                        ty: None,
                        value: NirExpr::Binary {
                            op: NirBinaryOp::Add,
                            lhs: Box::new(NirExpr::Var("acc".to_owned())),
                            rhs: Box::new(NirExpr::Var("value".to_owned())),
                        },
                    }],
                    else_body: vec![NirStmt::Let {
                        name: "acc".to_owned(),
                        ty: None,
                        value: NirExpr::Var("acc".to_owned()),
                    }],
                },
            ],
        },
        NirStmt::Return(Some(NirExpr::Var("acc".to_owned()))),
    ]);
    let _changed = simplify_nir_module(&mut module);
    let NirStmt::While { body, .. } = &module.functions[0].body[2] else {
        panic!("expected while statement to remain in place");
    };
    let NirStmt::If {
        then_body,
        else_body,
        ..
    } = &body[1]
    else {
        panic!("expected inner if statement to remain in loop body");
    };
    assert!(matches!(then_body.first(), Some(NirStmt::Let { name, .. }) if name == "acc"));
    assert!(matches!(else_body.first(), Some(NirStmt::Let { name, .. }) if name == "acc"));
}

#[test]
fn preserves_if_branch_bindings_used_after_statement() {
    let mut module = sample_module(vec![
        NirStmt::Let {
            name: "flag".to_owned(),
            ty: None,
            value: NirExpr::Var("input_flag".to_owned()),
        },
        NirStmt::If {
            condition: NirExpr::Var("flag".to_owned()),
            then_body: vec![NirStmt::Let {
                name: "overall_bonus".to_owned(),
                ty: None,
                value: NirExpr::Int(1),
            }],
            else_body: vec![NirStmt::Let {
                name: "overall_bonus".to_owned(),
                ty: None,
                value: NirExpr::Int(0),
            }],
        },
        NirStmt::Return(Some(NirExpr::Var("overall_bonus".to_owned()))),
    ]);
    let _changed = simplify_nir_module(&mut module);
    assert!(matches!(
        module.functions[0].body.first(),
        Some(NirStmt::Let { name, .. }) if name == "flag"
    ));
    let NirStmt::If {
        then_body,
        else_body,
        ..
    } = &module.functions[0].body[1]
    else {
        panic!("expected top-level if statement to remain in place");
    };
    assert!(matches!(
        then_body.first(),
        Some(NirStmt::Let { name, .. }) if name == "overall_bonus"
    ));
    assert!(matches!(
        else_body.first(),
        Some(NirStmt::Let { name, .. }) if name == "overall_bonus"
    ));
}

#[test]
fn prunes_dead_prior_rebinding_and_propagates_final_local_value() {
    let mut module = sample_module(vec![
        NirStmt::Let {
            name: "value".to_owned(),
            ty: None,
            value: NirExpr::Int(1),
        },
        NirStmt::Let {
            name: "value".to_owned(),
            ty: None,
            value: NirExpr::Int(2),
        },
        NirStmt::Return(Some(NirExpr::Var("value".to_owned()))),
    ]);
    let changed = simplify_nir_module(&mut module);
    assert!(changed);
    assert_eq!(
        module.functions[0].body,
        vec![NirStmt::Return(Some(NirExpr::Int(2)))]
    );
}

#[test]
fn inlines_annotated_pure_function_calls() {
    let mut module = NirModule {
        annotations: vec![],
        uses: vec![],
        domain: "cpu".to_owned(),
        unit: "Main".to_owned(),
        externs: vec![],
        extern_interfaces: vec![],
        consts: vec![],
        type_aliases: vec![],
        structs: vec![],
        enums: vec![],
        traits: vec![],
        impls: vec![],
        functions: vec![
            NirFunction {
                name: "add_one".to_owned(),
                annotations: vec![annotation("inline")],
                visibility: NirVisibility::Private,
                test_name: None,
                test_ignored: false,
                test_should_fail: false,
                test_reason: None,
                test_timeout_ms: None,
                test_clock_domain: None,
                test_clock_policy: None,
                benchmark_name: None,
                benchmark_warmup_iters: None,
                benchmark_measure_iters: None,
                benchmark_timeout_ms: None,
                benchmark_clock_domain: None,
                benchmark_clock_policy: None,
                is_async: false,
                generic_params: vec![],
                where_bounds: vec![],
                params: vec![NirParam {
                    name: "value".to_owned(),
                    ty: i64_type(),
                }],
                return_type: Some(i64_type()),
                body: vec![NirStmt::Return(Some(NirExpr::Binary {
                    op: NirBinaryOp::Add,
                    lhs: Box::new(NirExpr::Var("value".to_owned())),
                    rhs: Box::new(NirExpr::Int(1)),
                }))],
            },
            NirFunction {
                name: "main".to_owned(),
                annotations: vec![],
                visibility: NirVisibility::Private,
                test_name: None,
                test_ignored: false,
                test_should_fail: false,
                test_reason: None,
                test_timeout_ms: None,
                test_clock_domain: None,
                test_clock_policy: None,
                benchmark_name: None,
                benchmark_warmup_iters: None,
                benchmark_measure_iters: None,
                benchmark_timeout_ms: None,
                benchmark_clock_domain: None,
                benchmark_clock_policy: None,
                is_async: false,
                generic_params: vec![],
                where_bounds: vec![],
                params: vec![],
                return_type: Some(i64_type()),
                body: vec![NirStmt::Return(Some(NirExpr::Call {
                    callee: "add_one".to_owned(),
                    args: vec![NirExpr::Int(41)],
                }))],
            },
        ],
    };
    let changed = simplify_nir_module(&mut module);
    assert!(changed);
    assert_eq!(
        module.functions[1].body,
        vec![NirStmt::Return(Some(NirExpr::Int(42)))]
    );
}

#[test]
fn does_not_inline_noinline_annotated_function_calls() {
    let mut module = NirModule {
        annotations: vec![],
        uses: vec![],
        domain: "cpu".to_owned(),
        unit: "Main".to_owned(),
        externs: vec![],
        extern_interfaces: vec![],
        consts: vec![],
        type_aliases: vec![],
        structs: vec![],
        enums: vec![],
        traits: vec![],
        impls: vec![],
        functions: vec![
            NirFunction {
                name: "add_one".to_owned(),
                annotations: vec![annotation("inline"), annotation("noinline")],
                visibility: NirVisibility::Private,
                test_name: None,
                test_ignored: false,
                test_should_fail: false,
                test_reason: None,
                test_timeout_ms: None,
                test_clock_domain: None,
                test_clock_policy: None,
                benchmark_name: None,
                benchmark_warmup_iters: None,
                benchmark_measure_iters: None,
                benchmark_timeout_ms: None,
                benchmark_clock_domain: None,
                benchmark_clock_policy: None,
                is_async: false,
                generic_params: vec![],
                where_bounds: vec![],
                params: vec![NirParam {
                    name: "value".to_owned(),
                    ty: i64_type(),
                }],
                return_type: Some(i64_type()),
                body: vec![NirStmt::Return(Some(NirExpr::Binary {
                    op: NirBinaryOp::Add,
                    lhs: Box::new(NirExpr::Var("value".to_owned())),
                    rhs: Box::new(NirExpr::Int(1)),
                }))],
            },
            NirFunction {
                name: "main".to_owned(),
                annotations: vec![],
                visibility: NirVisibility::Private,
                test_name: None,
                test_ignored: false,
                test_should_fail: false,
                test_reason: None,
                test_timeout_ms: None,
                test_clock_domain: None,
                test_clock_policy: None,
                benchmark_name: None,
                benchmark_warmup_iters: None,
                benchmark_measure_iters: None,
                benchmark_timeout_ms: None,
                benchmark_clock_domain: None,
                benchmark_clock_policy: None,
                is_async: false,
                generic_params: vec![],
                where_bounds: vec![],
                params: vec![],
                return_type: Some(i64_type()),
                body: vec![NirStmt::Return(Some(NirExpr::Call {
                    callee: "add_one".to_owned(),
                    args: vec![NirExpr::Int(41)],
                }))],
            },
        ],
    };
    let changed = simplify_nir_module(&mut module);
    assert!(!changed);
    assert_eq!(
        module.functions[1].body,
        vec![NirStmt::Return(Some(NirExpr::Call {
            callee: "add_one".to_owned(),
            args: vec![NirExpr::Int(41)],
        }))]
    );
}
