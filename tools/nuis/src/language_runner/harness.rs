use super::*;

pub(super) fn build_test_harness_module(ast: &AstModule, test_function: &AstFunction) -> AstModule {
    let mut harness = ast.clone();
    harness.functions.retain(|function| function.name != "main");
    harness
        .functions
        .push(build_test_main_function(test_function));
    harness
}

pub(super) fn build_benchmark_harness_module(
    ast: &AstModule,
    benchmark_function: &AstFunction,
    iterations: i64,
) -> Result<AstModule, String> {
    let mut harness = ast.clone();
    harness.functions.retain(|function| function.name != "main");
    ensure_benchmark_timing_externs(&mut harness);
    harness
        .functions
        .push(build_benchmark_loop_function(benchmark_function));
    harness
        .functions
        .push(build_benchmark_elapsed_text_function());
    harness.functions.push(build_benchmark_main_function(
        benchmark_function,
        iterations,
    ));
    Ok(harness)
}

fn build_test_main_function(test_function: &AstFunction) -> AstFunction {
    #[rustfmt::skip]
    let test_call = AstExpr::Call {
        callee: test_function.name.clone(), generic_args: vec![], args: vec![],
    };
    let body = match test_function.return_type.as_ref() {
        Some(return_type) if return_type.name == "bool" && !return_type.is_ref => {
            let value_expr = if test_function.is_async {
                AstExpr::Await(Box::new(test_call))
            } else {
                test_call
            };
            vec![
                AstStmt::Let {
                    mutable: false,
                    name: "passed".to_owned(),
                    ty: Some(bool_type_ref()),
                    value: value_expr,
                },
                AstStmt::If {
                    condition: AstExpr::Var("passed".to_owned()),
                    then_body: vec![AstStmt::Return(Some(AstExpr::Int(1)))],
                    else_body: vec![AstStmt::Return(Some(AstExpr::Int(0)))],
                },
            ]
        }
        _ => {
            let value_expr = if test_function.is_async {
                AstExpr::Await(Box::new(test_call))
            } else {
                test_call
            };
            vec![
                AstStmt::Let {
                    mutable: false,
                    name: "status".to_owned(),
                    ty: Some(i64_type_ref()),
                    value: value_expr,
                },
                AstStmt::Return(Some(AstExpr::Var("status".to_owned()))),
            ]
        }
    };
    AstFunction {
        name: "main".to_owned(),
        visibility: nuis_semantics::model::AstVisibility::Private,
        attributes: vec![],
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
        is_async: test_function.is_async,
        generic_params: vec![],
        where_bounds: vec![],
        params: vec![],
        return_type: Some(i64_type_ref()),
        body,
    }
}

fn build_benchmark_loop_function(benchmark_function: &AstFunction) -> AstFunction {
    let helper_name = benchmark_loop_function_name();
    let side_effect_name = "benchmark_side_effect".to_owned();
    let remaining_name = "benchmark_remaining".to_owned();
    let benchmark_return_type = benchmark_function
        .return_type
        .clone()
        .unwrap_or_else(i64_type_ref);
    let recursive_call = AstExpr::Call {
        callee: helper_name.clone(),
        generic_args: vec![],
        args: vec![AstExpr::Binary {
            op: nuis_semantics::model::AstBinaryOp::Sub,
            lhs: Box::new(AstExpr::Var(remaining_name.clone())),
            rhs: Box::new(AstExpr::Int(1)),
        }],
    };
    let recurse_expr = if benchmark_function.is_async {
        AstExpr::Await(Box::new(recursive_call))
    } else {
        recursive_call
    };
    AstFunction {
        name: helper_name,
        visibility: AstVisibility::Private,
        attributes: vec![],
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
        is_async: benchmark_function.is_async,
        generic_params: vec![],
        where_bounds: vec![],
        params: vec![nuis_semantics::model::AstParam {
            name: remaining_name.clone(),
            ty: i64_type_ref(),
        }],
        return_type: Some(i64_type_ref()),
        body: vec![
            AstStmt::If {
                condition: AstExpr::Binary {
                    op: nuis_semantics::model::AstBinaryOp::Le,
                    lhs: Box::new(AstExpr::Var(remaining_name.clone())),
                    rhs: Box::new(AstExpr::Int(0)),
                },
                then_body: vec![AstStmt::Return(Some(AstExpr::Int(0)))],
                else_body: vec![],
            },
            AstStmt::Let {
                mutable: false,
                name: side_effect_name,
                ty: Some(benchmark_return_type),
                value: benchmark_call_expr(benchmark_function),
            },
            AstStmt::Return(Some(recurse_expr)),
        ],
    }
}

fn build_benchmark_main_function(benchmark_function: &AstFunction, iterations: i64) -> AstFunction {
    let helper_call = AstExpr::Call {
        callee: benchmark_loop_function_name(),
        generic_args: vec![],
        args: vec![AstExpr::Int(iterations)],
    };
    let return_expr = if benchmark_function.is_async {
        AstExpr::Await(Box::new(helper_call))
    } else {
        helper_call
    };
    AstFunction {
        name: "main".to_owned(),
        visibility: AstVisibility::Private,
        attributes: vec![],
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
        is_async: benchmark_function.is_async,
        generic_params: vec![],
        where_bounds: vec![],
        params: vec![],
        return_type: Some(i64_type_ref()),
        body: vec![
            AstStmt::Let {
                mutable: false,
                name: "benchmark_started_ns".to_owned(),
                ty: Some(i64_type_ref()),
                value: AstExpr::Call {
                    callee: "host_monotonic_time_ns".to_owned(),
                    generic_args: vec![],
                    args: vec![],
                },
            },
            AstStmt::Let {
                mutable: false,
                name: "benchmark_status".to_owned(),
                ty: Some(i64_type_ref()),
                value: return_expr,
            },
            AstStmt::Let {
                mutable: false,
                name: "benchmark_ended_ns".to_owned(),
                ty: Some(i64_type_ref()),
                value: AstExpr::Call {
                    callee: "host_monotonic_time_ns".to_owned(),
                    generic_args: vec![],
                    args: vec![],
                },
            },
            AstStmt::Let {
                mutable: false,
                name: "benchmark_elapsed_ns".to_owned(),
                ty: Some(i64_type_ref()),
                value: AstExpr::Binary {
                    op: nuis_semantics::model::AstBinaryOp::Sub,
                    lhs: Box::new(AstExpr::Var("benchmark_ended_ns".to_owned())),
                    rhs: Box::new(AstExpr::Var("benchmark_started_ns".to_owned())),
                },
            },
            AstStmt::Expr(AstExpr::Call {
                callee: "host_stdout_write".to_owned(),
                generic_args: vec![],
                args: vec![AstExpr::Call {
                    callee: benchmark_elapsed_text_function_name(),
                    generic_args: vec![],
                    args: vec![AstExpr::Var("benchmark_elapsed_ns".to_owned())],
                }],
            }),
            AstStmt::Return(Some(AstExpr::Int(0))),
        ],
    }
}

fn benchmark_call_expr(benchmark_function: &AstFunction) -> AstExpr {
    #[rustfmt::skip]
    let benchmark_call = AstExpr::Call {
        callee: benchmark_function.name.clone(), generic_args: vec![], args: vec![],
    };
    if benchmark_function.is_async {
        AstExpr::Await(Box::new(benchmark_call))
    } else {
        benchmark_call
    }
}

fn benchmark_loop_function_name() -> String {
    "__nuis_benchmark_loop".to_owned()
}

fn benchmark_elapsed_text_function_name() -> String {
    "__nuis_benchmark_elapsed_text".to_owned()
}

fn build_benchmark_elapsed_text_function() -> AstFunction {
    AstFunction {
        name: benchmark_elapsed_text_function_name(),
        visibility: AstVisibility::Private,
        attributes: vec![],
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
        params: vec![AstParam {
            name: "elapsed_ns".to_owned(),
            ty: i64_type_ref(),
        }],
        return_type: Some(i64_type_ref()),
        body: vec![
            AstStmt::Let {
                mutable: false,
                name: "buffer".to_owned(),
                ty: Some(ref_buffer_type_ref()),
                value: AstExpr::Call {
                    callee: "alloc_buffer".to_owned(),
                    generic_args: vec![],
                    args: vec![AstExpr::Int(64), AstExpr::Int(0)],
                },
            },
            AstStmt::Let {
                mutable: false,
                name: "written".to_owned(),
                ty: Some(i64_type_ref()),
                value: AstExpr::Call {
                    callee: "serialize_i64_into".to_owned(),
                    generic_args: vec![],
                    args: vec![
                        AstExpr::Var("elapsed_ns".to_owned()),
                        AstExpr::Var("buffer".to_owned()),
                        AstExpr::Int(0),
                    ],
                },
            },
            AstStmt::Return(Some(AstExpr::Call {
                callee: "deserialize_text_from".to_owned(),
                generic_args: vec![],
                args: vec![
                    AstExpr::Var("buffer".to_owned()),
                    AstExpr::Int(0),
                    AstExpr::Var("written".to_owned()),
                ],
            })),
        ],
    }
}

fn ensure_benchmark_timing_externs(module: &mut AstModule) {
    ensure_benchmark_timing_extern(module, "host_monotonic_time_ns", vec![]);
    ensure_benchmark_timing_extern(
        module,
        "host_serialize_i64_into",
        vec![
            AstParam {
                name: "value".to_owned(),
                ty: i64_type_ref(),
            },
            AstParam {
                name: "buffer_handle".to_owned(),
                ty: i64_type_ref(),
            },
            AstParam {
                name: "offset".to_owned(),
                ty: i64_type_ref(),
            },
        ],
    );
    ensure_benchmark_timing_extern(
        module,
        "host_deserialize_text_from",
        vec![
            AstParam {
                name: "buffer_handle".to_owned(),
                ty: i64_type_ref(),
            },
            AstParam {
                name: "offset".to_owned(),
                ty: i64_type_ref(),
            },
            AstParam {
                name: "len".to_owned(),
                ty: i64_type_ref(),
            },
        ],
    );
    ensure_benchmark_timing_extern(
        module,
        "host_text_len",
        vec![AstParam {
            name: "text_handle".to_owned(),
            ty: i64_type_ref(),
        }],
    );
    ensure_benchmark_timing_extern(
        module,
        "host_stdout_write",
        vec![AstParam {
            name: "text_handle".to_owned(),
            ty: i64_type_ref(),
        }],
    );
}

fn ensure_benchmark_timing_extern(module: &mut AstModule, name: &str, params: Vec<AstParam>) {
    if module.externs.iter().any(|function| function.name == name) {
        return;
    }
    module.externs.push(AstExternFunction {
        visibility: AstVisibility::Private,
        abi: "c".to_owned(),
        interface: None,
        name: name.to_owned(),
        host_symbol: None,
        params,
        return_type: i64_type_ref(),
    });
}

fn i64_type_ref() -> AstTypeRef {
    AstTypeRef {
        name: "i64".to_owned(),
        generic_args: vec![],
        is_optional: false,
        is_ref: false,
    }
}

fn ref_buffer_type_ref() -> AstTypeRef {
    AstTypeRef {
        name: "Buffer".to_owned(),
        generic_args: vec![],
        is_optional: false,
        is_ref: true,
    }
}

fn bool_type_ref() -> AstTypeRef {
    AstTypeRef {
        name: "bool".to_owned(),
        generic_args: vec![],
        is_optional: false,
        is_ref: false,
    }
}

pub(super) fn temp_test_output_dir(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "nuis-test-runner-{}-{}",
        sanitize_test_label(label),
        stamp
    ))
}

fn sanitize_test_label(label: &str) -> String {
    let mut out = String::new();
    for ch in label.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
        } else {
            out.push('-');
        }
    }
    if out.is_empty() {
        "test".to_owned()
    } else {
        out
    }
}
