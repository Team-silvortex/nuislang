use super::*;

pub(super) fn lowers_real_shader_result_project_with_expected_result_shape() {
    let artifacts =
        compiled_domain_project("../../examples/projects/domains/shader_result_profile_demo");

    let main = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();

    for name in ["gpu_packet_draw", "gpu_packet_render", "host_frame"] {
        assert!(main.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let { name: stmt_name, .. } if stmt_name == name
            )
        }));
    }

    for name in ["pass_result", "draw_result", "frame_result"] {
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

    assert!(main.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name: stmt_name,
                value: NirExpr::ShaderPassReady(_),
                ..
            } if stmt_name == "pass_ready"
        )
    }));
    for name in ["draw_ready", "frame_ready"] {
        assert!(main.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let {
                    name: stmt_name,
                    value: NirExpr::ShaderFrameReady(_),
                    ..
                } if stmt_name == name
            )
        }));
    }

    assert!(main
        .body
        .iter()
        .any(|stmt| { matches!(stmt, NirStmt::Expr(NirExpr::CpuPresentFrame(_))) }));
}

pub(super) fn lowers_real_shader_draw_render_project_with_expected_dual_result_shape() {
    let artifacts =
        compiled_domain_project("../../examples/projects/domains/shader_draw_render_profile_demo");

    let main = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();

    for name in [
        "gpu_packet_draw",
        "gpu_packet_render",
        "host_draw_preview",
        "host_frame",
    ] {
        assert!(main.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let { name: stmt_name, .. } if stmt_name == name
            )
        }));
    }

    for name in ["pass_result", "draw_result", "render_result"] {
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

    assert!(main.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name: stmt_name,
                value: NirExpr::ShaderPassReady(_),
                ..
            } if stmt_name == "pass_ready"
        )
    }));
    for name in ["draw_ready", "render_ready"] {
        assert!(main.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let {
                    name: stmt_name,
                    value: NirExpr::ShaderFrameReady(_),
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

pub(super) fn lowers_real_pixelmagic_packet_bridge_project_with_expected_bridge_shape() {
    let artifacts =
        compiled_domain_project("../../examples/projects/domains/pixelmagic_packet_bridge_demo");

    let main = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();

    for name in [
        "op_kind",
        "image_packet_total",
        "lowered",
        "gpu_packet",
        "frame",
        "host_frame",
    ] {
        assert!(main.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let { name: stmt_name, .. } if stmt_name == name
            )
        }));
    }
    assert!(main
        .body
        .iter()
        .any(|stmt| matches!(stmt, NirStmt::Expr(NirExpr::CpuPresentFrame(_)))));

    let source = inline_wgsl_source_named(&artifacts, "pixelmagic_packet_bridge");
    assert!(source.contains("@vertex"), "{source}");
    assert!(source.contains("@fragment"), "{source}");
    assert!(!source.contains("stage vertex"), "{source}");
    assert!(!source.contains("stage fragment"), "{source}");
}

pub(super) fn lowers_real_pixelmagic_render_project_with_expected_render_shape() {
    let artifacts =
        compiled_domain_project("../../examples/projects/domains/pixelmagic_render_demo");

    let main = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();

    for name in [
        "op_kind",
        "image_packet_total",
        "lowered",
        "gpu_packet",
        "render_result",
        "render_ready",
        "host_frame",
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
                name: stmt_name,
                value: NirExpr::ShaderResult { .. },
                ..
            } if stmt_name == "render_result"
        )
    }));
    assert!(main.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name: stmt_name,
                value: NirExpr::ShaderFrameReady(_),
                ..
            } if stmt_name == "render_ready"
        )
    }));
    assert!(main
        .body
        .iter()
        .any(|stmt| matches!(stmt, NirStmt::Expr(NirExpr::CpuPresentFrame(_)))));

    let source = inline_wgsl_source_named(&artifacts, "pixelmagic_render_demo");
    assert!(source.contains("@vertex"), "{source}");
    assert!(source.contains("@fragment"), "{source}");
    assert!(!source.contains("stage vertex"), "{source}");
    assert!(!source.contains("stage fragment"), "{source}");
}

pub(super) fn lowers_real_pixelmagic_texture_resource_project_with_expected_resource_shape() {
    let artifacts =
        compiled_domain_project("../../examples/projects/domains/pixelmagic_texture_resource_demo");

    let main = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();

    for name in [
        "image_resource",
        "resource_set",
        "resource_state",
        "lowered",
        "gpu_packet",
        "render_result",
        "host_frame",
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
                name: stmt_name,
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } if stmt_name == "resource_state" && type_name == "NovaResourceSetState"
        )
    }));
    assert!(main.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name: stmt_name,
                value: NirExpr::ShaderResult { .. },
                ..
            } if stmt_name == "render_result"
        )
    }));
    assert!(main
        .body
        .iter()
        .any(|stmt| matches!(stmt, NirStmt::Expr(NirExpr::CpuPresentFrame(_)))));

    let source = inline_wgsl_source_named(&artifacts, "pixelmagic_texture_resource_demo");
    assert!(source.contains("@vertex"), "{source}");
    assert!(source.contains("@fragment"), "{source}");
    assert!(!source.contains("stage vertex"), "{source}");
    assert!(!source.contains("stage fragment"), "{source}");
}

pub(super) fn lowers_real_pixelmagic_pipeline_project_with_expected_pipeline_shape() {
    let artifacts =
        compiled_domain_project("../../examples/projects/domains/pixelmagic_pipeline_demo");

    let main = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();

    for name in [
        "sample_kind",
        "output_kind",
        "image_packet_total",
        "shader_pipeline_total",
        "resource_set",
        "resource_state",
        "lowered",
        "gpu_packet",
        "render_result",
        "render_ready",
        "host_frame",
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
                name: stmt_name,
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } if stmt_name == "resource_state" && type_name == "NovaResourceSetState"
        )
    }));
    assert!(main.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name: stmt_name,
                value: NirExpr::ShaderResult { .. },
                ..
            } if stmt_name == "render_result"
        )
    }));
    assert!(main.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name: stmt_name,
                value: NirExpr::ShaderFrameReady(_),
                ..
            } if stmt_name == "render_ready"
        )
    }));
    assert!(main
        .body
        .iter()
        .any(|stmt| matches!(stmt, NirStmt::Expr(NirExpr::CpuPresentFrame(_)))));

    let source = inline_wgsl_source_named(&artifacts, "pixelmagic_pipeline_demo");
    assert!(source.contains("@vertex"), "{source}");
    assert!(source.contains("@fragment"), "{source}");
    assert!(!source.contains("stage vertex"), "{source}");
    assert!(!source.contains("stage fragment"), "{source}");
}
