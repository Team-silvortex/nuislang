use super::*;

pub(super) fn compiles_real_shader_async_policy_project() {
    let artifacts =
        compiled_domain_project("../../examples/projects/domains/shader_async_policy_profile_demo");
    assert!(artifacts
        .nir
        .functions
        .iter()
        .any(|function| function.name == "ShaderTaskAsyncShapes.async_policy_summary_completed"));
}

pub(super) fn lowers_real_shader_async_policy_project_with_expected_async_summary_shape() {
    let artifacts =
        compiled_domain_project("../../examples/projects/domains/shader_async_policy_profile_demo");

    let observe_draw = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "observe_result_ready_draw")
        .unwrap();
    assert!(observe_draw.is_async);
    assert!(matches!(
        observe_draw.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "i64"
    ));

    let observe_render = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "observe_result_ready_render")
        .unwrap();
    assert!(observe_render.is_async);
    assert!(matches!(
        observe_render.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "i64"
    ));

    let main = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    for name in [
        "draw_result_observer_task",
        "render_result_observer_task",
        "draw_result_observer_join",
        "render_result_observer_join",
    ] {
        assert!(main.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let { name: stmt_name, .. } if stmt_name == name
            )
        }));
    }
    for (name, callee) in [
        (
            "async_policy_summary_completed",
            "ShaderTaskAsyncShapes.async_policy_summary_completed",
        ),
        (
            "async_policy_summary_selected_value",
            "ShaderTaskAsyncShapes.async_policy_summary_selected_value",
        ),
    ] {
        assert!(main.body.iter().any(|stmt| {
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

    assert!(artifacts.nir.functions.iter().any(|function| {
        function.name == "ShaderTaskAsyncShapes.async_policy_summary_completed"
            && function.generic_params.is_empty()
    }));
    assert!(artifacts.nir.functions.iter().any(|function| {
        function.name == "ShaderTaskAsyncShapes.async_policy_summary_selected_value"
            && function.generic_params.is_empty()
    }));
}

pub(super) fn lowers_real_shader_async_windowed_batch_project_with_expected_batch_summary_shape() {
    let artifacts = compiled_domain_project(
        "../../examples/projects/domains/shader_async_windowed_batch_profile_demo",
    );

    let capture = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_async_windowed_batch_summary")
        .unwrap();
    assert!(matches!(
        capture.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "ShaderAsyncWindowedBatchSummary"
    ));
    for name in [
        "draw_result_observer_task_a",
        "draw_result_observer_task_b",
        "render_result_observer_task",
        "draw_result_observer_join_a",
        "draw_result_observer_join_b",
        "render_result_observer_join",
    ] {
        assert!(capture.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let { name: stmt_name, .. } if stmt_name == name
            )
        }));
    }
    for (field_name, callee) in [
        (
            "completed_count",
            "ShaderTaskAsyncShapes.async_batch_summary_completed",
        ),
        (
            "batch_value",
            "ShaderTaskAsyncShapes.async_batch_summary_value",
        ),
        (
            "preview_value",
            "ShaderTaskAsyncShapes.async_windowed_preview_summary_value",
        ),
        (
            "final_value",
            "ShaderTaskAsyncShapes.async_windowed_final_summary_value",
        ),
    ] {
        assert!(capture.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Return(Some(NirExpr::StructLiteral { fields, .. }))
                    if fields.iter().any(|(field, value)| {
                        field == field_name
                            && matches!(
                                value,
                                NirExpr::Call { callee: stmt_callee, .. } if stmt_callee == callee
                            )
                    })
            )
        }));
    }

    let main = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(main.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                value: NirExpr::Call { callee, .. },
                ..
            } if name == "async_windowed_batch_summary"
                && callee == "capture_async_windowed_batch_summary"
        )
    }));
}

pub(super) fn lowers_real_shader_async_fallback_project_with_expected_fallback_shape() {
    let artifacts = compiled_domain_project(
        "../../examples/projects/domains/shader_async_fallback_profile_demo",
    );

    let observe_draw = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "observe_draw")
        .unwrap();
    assert!(observe_draw.is_async);
    assert!(matches!(
        observe_draw.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "i64"
    ));

    let observe_render = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "observe_render")
        .unwrap();
    assert!(observe_render.is_async);
    assert!(matches!(
        observe_render.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "i64"
    ));

    let main = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    for name in [
        "draw_task",
        "render_task",
        "draw_task_result",
        "render_task_result",
    ] {
        assert!(main.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let { name: stmt_name, .. } if stmt_name == name
            )
        }));
    }
    assert!(main.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                value: NirExpr::CpuTimeout { .. },
                ..
            } if name == "render_task"
        )
    }));
    for (name, callee) in [
        (
            "fallback_completed",
            "ShaderTaskAsyncShapes.task_fallback_completed",
        ),
        (
            "fallback_selected_value",
            "ShaderTaskAsyncShapes.task_fallback_selected_value",
        ),
    ] {
        assert!(main.body.iter().any(|stmt| {
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

    assert!(artifacts.nir.functions.iter().any(|function| {
        function.name == "ShaderTaskAsyncShapes.task_fallback_completed"
            && function.generic_params.is_empty()
    }));
    assert!(artifacts.nir.functions.iter().any(|function| {
        function.name == "ShaderTaskAsyncShapes.task_fallback_selected_value"
            && function.generic_params.is_empty()
    }));
}

pub(super) fn lowers_real_shader_async_schedule_project_with_expected_schedule_shape() {
    let artifacts = compiled_domain_project(
        "../../examples/projects/domains/shader_async_schedule_profile_demo",
    );

    let observe_draw = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "observe_draw")
        .unwrap();
    assert!(observe_draw.is_async);
    assert!(matches!(
        observe_draw.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "i64"
    ));

    let observe_render = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "observe_render")
        .unwrap();
    assert!(observe_render.is_async);
    assert!(matches!(
        observe_render.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "i64"
    ));

    let encode = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "encode_task_value")
        .unwrap();
    assert!(matches!(
        encode.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "i64"
    ));

    let main = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();

    for name in [
        "draw_task",
        "render_task",
        "draw_task_result",
        "render_task_result",
        "gpu_packet_draw_async",
        "gpu_packet_render_async",
        "gpu_packet_present",
        "host_frame",
    ] {
        assert!(main.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let { name: stmt_name, .. } if stmt_name == name
            )
        }));
    }
    for name in [
        "draw_result_async",
        "render_result_async",
        "render_result_present",
    ] {
        assert!(main.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let {
                    name: stmt_name,
                    value: NirExpr::ShaderResult { .. },
                    ..
                } if stmt_name == name
            )
        }));
    }
    for (name, callee) in [
        ("draw_task", "observe_draw"),
        ("render_task", "observe_render"),
    ] {
        assert!(main.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let {
                    name: stmt_name,
                    value: NirExpr::CpuSpawn { callee: stmt_callee, .. },
                    ..
                } if stmt_name == name && stmt_callee == callee
            )
        }));
    }
    for name in ["draw_task_result", "render_task_result"] {
        assert!(main.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let {
                    name: stmt_name,
                    value: NirExpr::CpuJoinResult(_),
                    ..
                } if stmt_name == name
            )
        }));
    }
    assert!(main
        .body
        .iter()
        .any(|stmt| matches!(stmt, NirStmt::Expr(NirExpr::CpuPresentFrame(_)))));
}

pub(super) fn lowers_real_shader_async_fanin_project_with_expected_fanin_shape() {
    let artifacts =
        compiled_domain_project("../../examples/projects/domains/shader_async_fanin_profile_demo");

    let observe_pass = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "observe_pass")
        .unwrap();
    assert!(observe_pass.is_async);
    assert!(matches!(
        observe_pass.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "i64"
    ));

    let observe_frame = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "observe_frame")
        .unwrap();
    assert!(observe_frame.is_async);
    assert!(matches!(
        observe_frame.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "i64"
    ));

    let encode = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "encode_task_value")
        .unwrap();
    assert!(matches!(
        encode.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "i64"
    ));

    let main = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();

    for name in [
        "pass_task",
        "frame_task",
        "pass_task_result",
        "frame_task_result",
        "gpu_packet_async",
        "gpu_packet_present",
        "host_frame",
    ] {
        assert!(main.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let { name: stmt_name, .. } if stmt_name == name
            )
        }));
    }
    for name in [
        "pass_result_async",
        "frame_result_async",
        "frame_result_present",
    ] {
        assert!(main.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let {
                    name: stmt_name,
                    value: NirExpr::ShaderResult { .. },
                    ..
                } if stmt_name == name
            )
        }));
    }
    for (name, callee) in [
        ("pass_task", "observe_pass"),
        ("frame_task", "observe_frame"),
    ] {
        assert!(main.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let {
                    name: stmt_name,
                    value: NirExpr::CpuSpawn { callee: stmt_callee, .. },
                    ..
                } if stmt_name == name && stmt_callee == callee
            )
        }));
    }
    for name in ["pass_task_result", "frame_task_result"] {
        assert!(main.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let {
                    name: stmt_name,
                    value: NirExpr::CpuJoinResult(_),
                    ..
                } if stmt_name == name
            )
        }));
    }
    assert!(main
        .body
        .iter()
        .any(|stmt| matches!(stmt, NirStmt::Expr(NirExpr::CpuPresentFrame(_)))));
}
