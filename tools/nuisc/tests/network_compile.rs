use std::path::Path;

use nuis_semantics::model::{NirExpr, NirStmt};

fn compiled_project(path: &str) -> nuisc::pipeline::PipelineArtifacts {
    nuisc::pipeline::compile_project(Path::new(path))
        .unwrap_or_else(|error| panic!("project `{path}` should compile: {error}"))
}

#[test]
fn compiles_httpish_protocol_recipe_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_protocol_recipe_demo",
    );
    nuisc::pipeline::compile_project(project).expect("httpish protocol project should compile");
}

#[test]
fn compiles_http_request_recipe_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_request_recipe_demo",
    );
    nuisc::pipeline::compile_project(project).expect("http request project should compile");
}

#[test]
fn compiles_http_client_exchange_recipe_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_client_exchange_recipe_demo",
    );
    nuisc::pipeline::compile_project(project).expect("http client exchange project should compile");
}

#[test]
fn compiles_http_client_lane_recipe_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_client_lane_recipe_demo",
    );
    nuisc::pipeline::compile_project(project).expect("http client lane project should compile");
}

#[test]
fn compiles_http_service_lane_recipe_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_service_lane_recipe_demo",
    );
    nuisc::pipeline::compile_project(project).expect("http service lane project should compile");
}

#[test]
fn compiles_httpish_client_session_packet_recipe_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_client_session_packet_recipe_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("httpish client session packet project should compile");
}

#[test]
fn compiles_httpish_service_session_packet_recipe_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_service_session_packet_recipe_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("httpish service session packet project should compile");
}

#[test]
fn compiles_httpish_header_session_recipe_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_header_session_recipe_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("httpish header session project should compile");
}

#[test]
fn compiles_httpish_header_service_session_recipe_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_header_service_session_recipe_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("httpish header service session project should compile");
}

#[test]
fn compiles_httpish_exchange_contract_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_exchange_contract_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("httpish exchange contract project should compile");
}

#[test]
fn compiles_httpish_exchange_contract_service_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_exchange_contract_service_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("httpish exchange contract service project should compile");
}

#[test]
fn compiles_httpish_exchange_blocks_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_exchange_blocks_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("httpish exchange blocks project should compile");
}

#[test]
fn compiles_httpish_exchange_blocks_service_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_exchange_blocks_service_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("httpish exchange blocks service project should compile");
}

#[test]
fn compiles_network_host_handle_runtime_probe_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_host_handle_runtime_probe_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("network host handle runtime probe project should compile");
}

#[test]
fn compiles_http_client_runtime_probe_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_client_runtime_probe_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("http client runtime probe project should compile");
}

#[test]
fn compiles_tcp_socket_runtime_probe_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_tcp_socket_runtime_probe_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("tcp socket runtime probe project should compile");
}

#[test]
fn compiles_tcp_send_runtime_probe_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_tcp_send_runtime_probe_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("tcp send runtime probe project should compile");
}

#[test]
fn compiles_http_status_runtime_probe_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_status_runtime_probe_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("http status runtime probe project should compile");
}

#[test]
fn compiles_network_loopback_runtime_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_loopback_runtime_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("network loopback runtime project should compile");
}

#[test]
fn compiles_network_host_open_surface_runtime_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_host_open_surface_runtime_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("network host open surface runtime project should compile");
}

#[test]
fn compiles_net_session_recipe_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_session_recipe_demo",
    );
    nuisc::pipeline::compile_project(project).expect("net session recipe project should compile");
}

#[test]
fn compiles_net_loop_control_recipe_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_loop_control_recipe_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("net loop control recipe project should compile");
}

#[test]
fn compiles_net_session_loop_control_recipe_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_session_loop_control_recipe_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("net session loop control recipe project should compile");
}

#[test]
fn compiles_net_http_session_loop_bridge_recipe_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_session_loop_bridge_recipe_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("net http session loop bridge recipe project should compile");
}

#[test]
fn lowers_http_client_exchange_recipe_project_with_expected_summary_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_client_exchange_recipe_demo",
    );

    let capture = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_net_http_client_exchange_summary")
        .unwrap();
    assert!(matches!(
        capture.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "NetHttpClientExchangeSummary"
    ));

    let summarize = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "summarize_net_http_client_exchange_recipe")
        .unwrap();
    assert!(matches!(
        summarize.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "i64"
    ));
    assert!(matches!(
        summarize.body.first(),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "summary" && callee == "capture_net_http_client_exchange_summary"
    ));
}

#[test]
fn lowers_http_request_recipe_project_with_expected_request_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_request_recipe_demo",
    );

    let capture = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_net_http_request_summary")
        .unwrap();
    assert!(matches!(
        capture.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "NetHttpRequestSummary"
    ));
    for (name, callee) in [
        ("open_handle", "host_network_open_tcp_stream"),
        ("close_value", "host_network_close_owned"),
    ] {
        assert!(capture.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let {
                    name: stmt_name,
                    value: NirExpr::CpuExternCall { callee: stmt_callee, .. },
                    ..
                } if stmt_name == name && stmt_callee == callee
            )
        }));
    }
    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                value: NirExpr::NetworkResult { .. },
                ..
            } if name == "send_result"
        )
    }));
    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name: stmt_name,
                value: NirExpr::Call { callee: stmt_callee, .. },
                ..
            } if stmt_name == "send_value" && stmt_callee == "encode_value"
        )
    }));
    for field_name in [
        "method_code",
        "request_line_bytes",
        "request_header_bytes",
        "request_body_bytes",
        "send_ready",
        "request_value",
    ] {
        assert!(capture.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Return(Some(NirExpr::StructLiteral { fields, .. }))
                    if fields.iter().any(|(field, _)| field == field_name)
            )
        }));
    }

    let summarize = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "summarize_net_http_request_recipe")
        .unwrap();
    assert!(matches!(
        summarize.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "i64"
    ));
    assert!(matches!(
        summarize.body.first(),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "summary" && callee == "capture_net_http_request_summary"
    ));
}

#[test]
fn lowers_httpish_client_session_packet_recipe_project_with_expected_packet_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_client_session_packet_recipe_demo",
    );

    let capture = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_net_httpish_client_session_packet_summary")
        .unwrap();
    assert!(matches!(
        capture.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "NetHttpishClientSessionPacketSummary"
    ));
    for (name, callee) in [
        ("request_plan", "build_request_plan"),
        ("primary_result", "run_primary_fetch"),
        ("fallback_result", "run_fallback_fetch"),
        ("packet", "stage_response_packet"),
    ] {
        assert!(capture.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let {
                    name: stmt_name,
                    value: NirExpr::Call { callee: stmt_callee, .. },
                    ..
                } if stmt_name == name && stmt_callee == callee
            )
        }));
    }
    for field_name in ["status_code", "body_code", "staged_total", "packet_value"] {
        assert!(capture.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Return(Some(NirExpr::StructLiteral { fields, .. }))
                    if fields.iter().any(|(field, _)| field == field_name)
            )
        }));
    }

    let stage = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "stage_response_packet")
        .unwrap();
    assert!(matches!(
        stage.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "NetHttpishResponsePacket"
    ));
    assert!(stage.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Return(Some(NirExpr::StructLiteral { fields, .. }))
                if fields.iter().any(|(field, _)| field == "staged_total")
        )
    }));

    let summarize = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "summarize_net_httpish_client_session_packet_recipe")
        .unwrap();
    assert!(matches!(
        summarize.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "i64"
    ));
    assert!(matches!(
        summarize.body.first(),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "summary"
            && callee == "capture_net_httpish_client_session_packet_summary"
    ));
}

#[test]
fn lowers_httpish_service_session_packet_recipe_project_with_expected_packet_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_service_session_packet_recipe_demo",
    );

    let capture = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_net_httpish_service_session_packet_summary")
        .unwrap();
    assert!(matches!(
        capture.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "NetHttpishServiceSessionPacketSummary"
    ));
    for (name, callee) in [
        ("accept_plan", "build_accept_plan"),
        ("primary_result", "run_primary_recv"),
        ("fallback_result", "run_fallback_recv"),
        ("packet", "stage_service_packet"),
    ] {
        assert!(capture.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let {
                    name: stmt_name,
                    value: NirExpr::Call { callee: stmt_callee, .. },
                    ..
                } if stmt_name == name && stmt_callee == callee
            )
        }));
    }
    for field_name in ["status_code", "body_code", "staged_total", "packet_value"] {
        assert!(capture.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Return(Some(NirExpr::StructLiteral { fields, .. }))
                    if fields.iter().any(|(field, _)| field == field_name)
            )
        }));
    }

    let stage = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "stage_service_packet")
        .unwrap();
    assert!(matches!(
        stage.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "NetHttpishServicePacket"
    ));
    assert!(stage.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Return(Some(NirExpr::StructLiteral { fields, .. }))
                if fields.iter().any(|(field, _)| field == "staged_total")
        )
    }));

    let summarize = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "summarize_net_httpish_service_session_packet_recipe")
        .unwrap();
    assert!(matches!(
        summarize.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "i64"
    ));
    assert!(matches!(
        summarize.body.first(),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "summary"
            && callee == "capture_net_httpish_service_session_packet_summary"
    ));
}

#[test]
fn lowers_httpish_header_session_recipe_project_with_expected_session_packet_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_header_session_recipe_demo",
    );

    let packet = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "stage_header_session_packet")
        .unwrap();
    assert!(matches!(
        packet.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "NetHttpishHeaderSessionPacket"
    ));
    assert!(packet.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                value: NirExpr::AllocBuffer { .. },
                ..
            } if name == "scratch"
        )
    }));
    assert!(packet
        .body
        .iter()
        .any(|stmt| { matches!(stmt, NirStmt::Expr(NirExpr::StoreAt { .. })) }));
    assert!(packet.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                value: NirExpr::LoadAt { .. },
                ..
            } if name == "staged_status"
                || name == "staged_request_header"
                || name == "staged_response_header"
                || name == "staged_body"
                || name == "staged_retry"
        )
    }));
    assert!(packet
        .body
        .iter()
        .any(|stmt| matches!(stmt, NirStmt::Expr(NirExpr::Free(_)))));

    let capture = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_net_httpish_header_session_summary")
        .unwrap();
    assert!(matches!(
        capture.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "NetHttpishHeaderSessionSummary"
    ));
    for (name, callee) in [
        ("request_plan", "build_request_plan"),
        ("request_headers", "build_request_headers"),
        ("send_result", "send_session_headers"),
        ("status_result", "recv_session_status"),
        ("recv_result", "recv_session_body"),
        ("packet", "stage_header_session_packet"),
    ] {
        assert!(capture.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let {
                    name: stmt_name,
                    value: NirExpr::Call { callee: stmt_callee, .. },
                    ..
                } if stmt_name == name && stmt_callee == callee
            )
        }));
    }

    let summarize = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "summarize_net_httpish_header_session_recipe")
        .unwrap();
    assert!(matches!(
        summarize.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "i64"
    ));
    assert!(matches!(
        summarize.body.first(),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "summary" && callee == "capture_net_httpish_header_session_summary"
    ));
}

#[test]
fn lowers_http_client_lane_recipe_project_with_expected_client_lane_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_client_lane_recipe_demo",
    );

    let capture = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_net_http_client_lane_summary")
        .unwrap();
    assert!(matches!(
        capture.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "NetHttpClientLaneSummary"
    ));
    for (name, callee) in [
        ("open_handle", "host_network_open_tcp_stream"),
        ("close_value", "host_network_close_owned"),
    ] {
        assert!(capture.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let {
                    name: stmt_name,
                    value: NirExpr::CpuExternCall { callee: stmt_callee, .. },
                    ..
                } if stmt_name == name && stmt_callee == callee
            )
        }));
    }
    for name in ["send_result", "status_result", "recv_result"] {
        assert!(capture.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let {
                    name: stmt_name,
                    value: NirExpr::NetworkResult { .. },
                    ..
                } if stmt_name == name
            )
        }));
    }
    for (name, callee) in [
        ("status_code", "encode_value"),
        ("send_value", "encode_value"),
        ("recv_value", "encode_value"),
    ] {
        assert!(capture.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let {
                    name: stmt_name,
                    value: NirExpr::Call { callee: stmt_callee, .. },
                    ..
                } if stmt_name == name && stmt_callee == callee
            )
        }));
    }
    for field_name in ["send_ready", "recv_ready", "lane_value"] {
        assert!(capture.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Return(Some(NirExpr::StructLiteral { fields, .. }))
                    if fields.iter().any(|(field, _)| field == field_name)
            )
        }));
    }

    let summarize = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "summarize_net_http_client_lane_recipe")
        .unwrap();
    assert!(matches!(
        summarize.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "i64"
    ));
    assert!(matches!(
        summarize.body.first(),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "summary" && callee == "capture_net_http_client_lane_summary"
    ));
}

#[test]
fn lowers_http_service_lane_recipe_project_with_expected_service_lane_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_service_lane_recipe_demo",
    );

    let capture = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_net_http_service_lane_summary")
        .unwrap();
    assert!(matches!(
        capture.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "NetHttpServiceLaneSummary"
    ));
    for (name, callee) in [
        ("listener_handle", "host_network_open_tcp_listener"),
        ("accepted_close_value", "host_network_close_owned"),
        ("listener_close_value", "host_network_close_owned"),
    ] {
        assert!(capture.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let {
                    name: stmt_name,
                    value: NirExpr::CpuExternCall { callee: stmt_callee, .. },
                    ..
                } if stmt_name == name && stmt_callee == callee
            )
        }));
    }
    for name in ["accept_result", "recv_result", "send_result"] {
        assert!(capture.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let {
                    name: stmt_name,
                    value: NirExpr::NetworkResult { .. },
                    ..
                } if stmt_name == name
            )
        }));
    }
    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                value: NirExpr::Call { callee, .. },
                ..
            } if name == "accepted_handle" && callee == "encode_value"
        )
    }));
    for field_name in ["accept_ready", "recv_ready", "send_ready"] {
        assert!(capture.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Return(Some(NirExpr::StructLiteral { fields, .. }))
                    if fields.iter().any(|(field, _)| field == field_name)
            )
        }));
    }

    let summarize = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "summarize_net_http_service_lane_recipe")
        .unwrap();
    assert!(matches!(
        summarize.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "i64"
    ));
    assert!(matches!(
        summarize.body.first(),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "summary" && callee == "capture_net_http_service_lane_summary"
    ));
}

#[test]
fn lowers_net_session_recipe_project_with_expected_async_task_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_session_recipe_demo",
    );

    for function_name in [
        "consume_network_result",
        "plan_control",
        "plan_tx",
        "plan_rx",
        "plan_session",
    ] {
        let function = artifacts
            .nir
            .functions
            .iter()
            .find(|function| function.name == function_name)
            .unwrap_or_else(|| panic!("missing function `{function_name}`"));
        assert!(function.is_async, "`{function_name}` should remain async");
        assert!(matches!(
            function.return_type.as_ref().map(|ty| ty.render()),
            Some(rendered) if rendered == "i64"
        ));
    }

    let consume = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "consume_network_result")
        .unwrap();
    assert_eq!(
        consume
            .body
            .iter()
            .filter(|stmt| matches!(stmt, NirStmt::If { .. }))
            .count(),
        3
    );
    assert!(matches!(
        consume.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Int(0))))
    ));

    let capture = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_net_session_summary")
        .unwrap();
    assert!(matches!(
        capture.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "NetSessionSummary"
    ));
    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                value: NirExpr::CpuTimeout { .. },
                ..
            } if name == "session_task"
        )
    }));
    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                value: NirExpr::CpuJoinResult(_),
                ..
            } if name == "session_result"
        )
    }));

    let summarize = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "summarize_net_session_recipe")
        .unwrap();
    assert!(matches!(
        summarize.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "i64"
    ));
    assert!(matches!(
        summarize.body.first(),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "summary" && callee == "capture_net_session_summary"
    ));
}

#[test]
fn lowers_net_loop_control_recipe_project_with_expected_loop_nodes() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_loop_control_recipe_demo",
    );

    let flow = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "fold_flow")
        .unwrap();
    assert!(matches!(
        flow.body.as_slice(),
        [
            NirStmt::Let { .. },
            NirStmt::Let { .. },
            NirStmt::While { .. },
            NirStmt::Return(Some(_))
        ]
    ));

    let post_flow = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "fold_post_flow")
        .unwrap();
    assert!(matches!(
        post_flow.body.as_slice(),
        [
            NirStmt::Let { .. },
            NirStmt::Let { .. },
            NirStmt::While { .. },
            NirStmt::Return(Some(_))
        ]
    ));

    let flow_node = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_flow_cond_chain"
        })
        .expect("expected loop_while_scalar_flow_cond_chain node");
    assert_eq!(flow_node.op.args[3], "lt");
    assert_eq!(flow_node.op.args[5], "current_eq");
    assert_eq!(flow_node.op.args[7], "continue");
    assert_eq!(flow_node.op.args[9], "current_gt");
    assert_eq!(flow_node.op.args[11], "add_current");
    assert_eq!(flow_node.op.args[12], "keep");

    let post_flow_node = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "loop_while_scalar_post_flow_cond_chain"
        })
        .expect("expected loop_while_scalar_post_flow_cond_chain node");
    assert_eq!(post_flow_node.op.args[3], "lt");
    assert_eq!(post_flow_node.op.args[5], "and");
    assert_eq!(post_flow_node.op.args[6], "carry0_gt");
    assert_eq!(post_flow_node.op.args[8], "carry0_gt");
    assert_eq!(post_flow_node.op.args[10], "break");
    assert_eq!(post_flow_node.op.args[12], "current_gt");
    assert_eq!(post_flow_node.op.args[14], "add_current");
    assert_eq!(post_flow_node.op.args[15], "keep");
}

#[test]
fn lowers_net_session_loop_control_recipe_project_with_expected_summary_and_loop_nodes() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_session_loop_control_recipe_demo",
    );

    let capture = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_net_session_loop_control_summary")
        .unwrap();
    assert!(matches!(
        capture.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "NetSessionLoopControlSummary"
    ));
    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                value: NirExpr::Call { callee, .. },
                ..
            } if name == "control_summary" && callee == "capture_net_control_session_summary"
        )
    }));
    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                value: NirExpr::CpuJoin(_),
                ..
            } if name == "primary_value" || name == "secondary_value"
        )
    }));

    let summarize = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "summarize_net_session_loop_control_recipe")
        .unwrap();
    assert!(matches!(
        summarize.body.first(),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "summary" && callee == "capture_net_session_loop_control_summary"
    ));

    assert!(artifacts.yir.nodes.iter().any(|node| {
        node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_flow_cond_chain"
    }));
    assert!(artifacts.yir.nodes.iter().any(|node| {
        node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_post_flow_cond_chain"
    }));
}

#[test]
fn lowers_net_http_session_loop_bridge_recipe_project_with_expected_bridge_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_session_loop_bridge_recipe_demo",
    );

    let packet = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_net_session_loop_bridge_packet")
        .unwrap();
    assert!(matches!(
        packet.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered)
            if rendered
                == "NetBridgePacket<NetBridgeCell<NetSessionLoopBridgeSummary>>"
    ));
    assert!(packet.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                value: NirExpr::Call { callee, .. },
                ..
            } if name == "http_session" && callee == "capture_net_http_client_session_summary"
        )
    }));
    assert!(packet.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                value: NirExpr::Call { callee, .. },
                ..
            } if name == "loop_window" && callee == "capture_loop_window"
        )
    }));

    let capture = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_net_session_loop_bridge_summary")
        .unwrap();
    assert!(matches!(
        capture.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered)
            if rendered
                == "NetBridgeEnvelope<NetBridgePacket<NetBridgeCell<NetSessionLoopBridgeSummary>>>"
    ));
    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                value: NirExpr::Call { callee, .. },
                ..
            } if name == "packet" && callee == "capture_net_session_loop_bridge_packet"
        )
    }));
    assert!(artifacts.nir.functions.iter().any(|function| {
        function.name.starts_with("wrap_bridge_cell__") && function.generic_params.is_empty()
    }));
    assert!(artifacts.nir.functions.iter().any(|function| {
        function.name.starts_with("wrap_bridge_packet__") && function.generic_params.is_empty()
    }));
    assert!(artifacts.nir.functions.iter().any(|function| {
        function.name.starts_with("wrap_bridge_envelope__") && function.generic_params.is_empty()
    }));
    assert!(artifacts.nir.functions.iter().any(|function| {
        function.name.starts_with("__hof_apply_bridge_packetized")
            && function.generic_params.is_empty()
    }));
    assert!(artifacts.nir.functions.iter().any(|function| {
        function
            .name
            .starts_with("__lambda_capture_net_session_loop_bridge_summary_")
            && function.generic_params.is_empty()
    }));

    let summarize = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "summarize_net_http_session_loop_bridge_recipe")
        .unwrap();
    assert!(matches!(
        summarize.body.first(),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "summary" && callee == "capture_net_session_loop_bridge_summary"
    ));

    assert!(artifacts.yir.nodes.iter().any(|node| {
        node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_flow_cond_chain"
    }));
    assert!(artifacts.yir.nodes.iter().any(|node| {
        node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_post_flow_cond_chain"
    }));
}
