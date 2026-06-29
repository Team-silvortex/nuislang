use super::*;
use nuis_semantics::model::{
    NirFunction, NirNetworkFlowState, NirParam, NirTypeRef, NirVisibility,
};

fn i64_type() -> NirTypeRef {
    NirTypeRef {
        name: "i64".to_owned(),
        generic_args: vec![],
        is_optional: false,
        is_ref: false,
    }
}

fn test_module(body: Vec<NirStmt>) -> NirModule {
    test_module_with_functions(vec![NirFunction {
        visibility: NirVisibility::Private,
        name: "main".to_owned(),
        annotations: vec![],
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
        body,
    }])
}

fn test_module_with_functions(functions: Vec<NirFunction>) -> NirModule {
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
        functions,
    }
}

fn network_result_i64_type() -> NirTypeRef {
    NirTypeRef {
        name: "NetworkResult".to_owned(),
        generic_args: vec![i64_type()],
        is_optional: false,
        is_ref: false,
    }
}

fn private_fn(
    name: &str,
    params: Vec<NirParam>,
    return_type: Option<NirTypeRef>,
    body: Vec<NirStmt>,
) -> NirFunction {
    NirFunction {
        visibility: NirVisibility::Private,
        name: name.to_owned(),
        annotations: vec![],
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
        params,
        return_type,
        body,
    }
}

fn open_tcp_stream_expr() -> NirExpr {
    NirExpr::CpuExternCall {
        abi: "c".to_owned(),
        interface: None,
        callee: "host_network_open_tcp_stream".to_owned(),
        args: vec![NirExpr::Int(443), NirExpr::Int(250)],
    }
}

#[test]
fn network_owned_handle_provenance_merges_matching_if_branches() {
    let module = test_module(vec![
        NirStmt::If {
            condition: NirExpr::Bool(true),
            then_body: vec![NirStmt::Let {
                name: "handle".to_owned(),
                ty: Some(i64_type()),
                value: open_tcp_stream_expr(),
            }],
            else_body: vec![NirStmt::Let {
                name: "handle".to_owned(),
                ty: Some(i64_type()),
                value: open_tcp_stream_expr(),
            }],
        },
        NirStmt::Expr(NirExpr::CpuExternCall {
            abi: "c".to_owned(),
            interface: None,
            callee: "host_network_close_owned".to_owned(),
            args: vec![NirExpr::Var("handle".to_owned())],
        }),
    ]);

    validate_network_owned_handle_provenance(&module, "cpu.Main", "network.NetworkUnit").unwrap();
}

#[test]
fn network_owned_handle_provenance_merges_matching_while_state() {
    let module = test_module(vec![
        NirStmt::Let {
            name: "handle".to_owned(),
            ty: Some(i64_type()),
            value: open_tcp_stream_expr(),
        },
        NirStmt::While {
            condition: NirExpr::Bool(true),
            body: vec![NirStmt::Let {
                name: "handle".to_owned(),
                ty: Some(i64_type()),
                value: open_tcp_stream_expr(),
            }],
        },
        NirStmt::Expr(NirExpr::CpuExternCall {
            abi: "c".to_owned(),
            interface: None,
            callee: "host_network_close_owned".to_owned(),
            args: vec![NirExpr::Var("handle".to_owned())],
        }),
    ]);

    validate_network_owned_handle_provenance(&module, "cpu.Main", "network.NetworkUnit").unwrap();
}

#[test]
fn network_owned_handle_provenance_accepts_network_result_wrapped_helper_return() {
    let module = test_module_with_functions(vec![
        private_fn(
            "open_handle_result",
            vec![],
            Some(network_result_i64_type()),
            vec![NirStmt::Return(Some(NirExpr::NetworkResult {
                value: Box::new(open_tcp_stream_expr()),
                state: NirNetworkFlowState::ConfigReady,
            }))],
        ),
        private_fn(
            "main",
            vec![],
            Some(i64_type()),
            vec![
                NirStmt::Let {
                    name: "opened".to_owned(),
                    ty: Some(network_result_i64_type()),
                    value: NirExpr::Call {
                        callee: "open_handle_result".to_owned(),
                        args: vec![],
                    },
                },
                NirStmt::Let {
                    name: "handle".to_owned(),
                    ty: Some(i64_type()),
                    value: NirExpr::NetworkValue(Box::new(NirExpr::Var("opened".to_owned()))),
                },
                NirStmt::Expr(NirExpr::CpuExternCall {
                    abi: "c".to_owned(),
                    interface: None,
                    callee: "host_network_close_owned".to_owned(),
                    args: vec![NirExpr::Var("handle".to_owned())],
                }),
            ],
        ),
    ]);

    validate_network_owned_handle_provenance(&module, "cpu.Main", "network.NetworkUnit").unwrap();
}

#[test]
fn network_owned_handle_provenance_rejects_network_result_wrapped_listener_return_for_send() {
    let module = test_module_with_functions(vec![
        private_fn(
            "open_listener_result",
            vec![],
            Some(network_result_i64_type()),
            vec![NirStmt::Return(Some(NirExpr::NetworkResult {
                value: Box::new(NirExpr::CpuExternCall {
                    abi: "c".to_owned(),
                    interface: None,
                    callee: "host_network_open_tcp_listener".to_owned(),
                    args: vec![NirExpr::Int(9000), NirExpr::Int(125), NirExpr::Int(150)],
                }),
                state: NirNetworkFlowState::ConfigReady,
            }))],
        ),
        private_fn(
            "main",
            vec![],
            Some(i64_type()),
            vec![
                NirStmt::Let {
                    name: "opened".to_owned(),
                    ty: Some(network_result_i64_type()),
                    value: NirExpr::Call {
                        callee: "open_listener_result".to_owned(),
                        args: vec![],
                    },
                },
                NirStmt::Let {
                    name: "handle".to_owned(),
                    ty: Some(i64_type()),
                    value: NirExpr::NetworkValue(Box::new(NirExpr::Var("opened".to_owned()))),
                },
                NirStmt::Expr(NirExpr::CpuExternCall {
                    abi: "c".to_owned(),
                    interface: None,
                    callee: "host_network_send_owned".to_owned(),
                    args: vec![
                        NirExpr::Var("handle".to_owned()),
                        NirExpr::Int(64),
                        NirExpr::Int(32),
                    ],
                }),
            ],
        ),
    ]);

    let err = validate_network_owned_handle_provenance(&module, "cpu.Main", "network.NetworkUnit")
        .unwrap_err();
    assert!(err.contains("host_network_send_owned"), "{err}");
    assert!(err.contains("listener-owned source"), "{err}");
}
