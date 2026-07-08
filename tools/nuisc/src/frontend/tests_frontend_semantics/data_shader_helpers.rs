use super::*;

#[test]
fn lowers_explicit_data_result_helpers() {
    let module = parse_nuis_module(
        r#"
        mod data FabricPlane {
          fn main() -> i64 {
            let pipe_result: DataResult<Pipe<i64>> = data_result(data_output_pipe(7));
            let moved: bool = data_moved(pipe_result);
            let intake: DataResult<i64> = data_result(data_input_pipe(data_output_pipe(9)));
            let ready: bool = data_ready(intake);
            let value: i64 = data_value(intake);
            return value;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        function.body.first(),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::DataResult { state, .. },
            ..
        }) if ty.render() == "DataResult<Pipe<i64>>"
            && matches!(state, NirDataFlowState::Moved)
    ));
    assert!(matches!(
        function.body.get(1),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::DataMoved(_),
            ..
        }) if ty.render() == "bool"
    ));
    assert!(matches!(
        function.body.get(2),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::DataResult { state, .. },
            ..
        }) if ty.render() == "DataResult<i64>"
            && matches!(state, NirDataFlowState::Ready)
    ));
    assert!(matches!(
        function.body.get(4),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::DataValue(_),
            ..
        }) if ty.render() == "i64"
    ));
}

#[test]
fn rejects_data_result_of_non_data_operation() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let result: DataResult<i64> = data_result(7);
            return data_value(result);
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("data_result(...) expects a direct data operation"));
}

#[test]
fn lowers_explicit_shader_result_helpers() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let pass_result: ShaderResult<Pass> = shader_result(shader_begin_pass(
              shader_target("rgba8", 16, 16),
              shader_pipeline("flat", "triangle"),
              shader_viewport(16, 16)
            ));
            let frame_result: ShaderResult<Frame> = shader_result(shader_profile_render(
              "SurfaceShader",
              shader_profile_packet("SurfaceShader", 1, 2, 3)
            ));
            let ready: bool = shader_frame_ready(frame_result);
            let frame: Frame = shader_value(frame_result);
            return 1;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        function.body.first(),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::ShaderResult { state, .. },
            ..
        }) if ty.render() == "ShaderResult<Pass>"
            && matches!(state, NirShaderFlowState::PassReady)
    ));
    assert!(matches!(
        function.body.get(1),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::ShaderResult { state, .. },
            ..
        }) if ty.render() == "ShaderResult<Frame>"
            && matches!(state, NirShaderFlowState::FrameReady)
    ));
    assert!(matches!(
        function.body.get(2),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::ShaderFrameReady(_),
            ..
        }) if ty.render() == "bool"
    ));
    assert!(matches!(
        function.body.get(3),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::ShaderValue(_),
            ..
        }) if ty.render() == "Frame"
    ));
}

#[test]
fn lowers_shader_texture_sampling_builtins() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let texture: Texture =
              shader_texture2d("r8", 2, 2, "1,2,3,4");
            let sampler: Sampler = shader_sampler("nearest", "clamp");
            let sampled: i64 = shader_sample_nearest(texture, sampler, 1, 0);
            return sampled;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        function.body.first(),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::ShaderTexture2d { width, height, .. },
            ..
        }) if ty.render() == "Texture" && *width == 2 && *height == 2
    ));
    assert!(matches!(
        function.body.get(1),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::ShaderSampler { .. },
            ..
        }) if ty.render() == "Sampler"
    ));
    assert!(matches!(
        function.body.get(2),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::ShaderSample { mode, .. },
            ..
        }) if ty.render() == "i64"
            && matches!(mode, nuis_semantics::model::NirShaderSampleMode::Nearest)
    ));
}

#[test]
fn lowers_shader_uv_sampling_builtins() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let texture: Texture =
              shader_texture2d("r8", 2, 2, "1,2,3,4");
            let sampler: Sampler = shader_sampler("linear", "clamp");
            let uv: UV = shader_uv(512, 256);
            let sampled: i64 = shader_sample_uv_linear(texture, sampler, uv);
            return sampled;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        function.body.get(2),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::ShaderUv { u, v },
            ..
        }) if ty.render() == "UV" && *u == 512 && *v == 256
    ));
    assert!(matches!(
        function.body.get(3),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::ShaderSampleUv { mode, .. },
            ..
        }) if ty.render() == "i64"
            && matches!(mode, nuis_semantics::model::NirShaderSampleUvMode::Linear)
    ));
}

#[test]
fn lowers_shader_binding_set_builtins() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let pipeline: Pipeline = shader_pipeline("blit", "triangle-strip");
            let texture: Texture =
              shader_texture2d("r8", 2, 2, "1,2,3,4");
            let sampler: Sampler = shader_sampler("linear", "clamp");
            let texture_binding: Binding = shader_texture_binding(0, texture);
            let sampler_binding: Binding = shader_sampler_binding(1, sampler);
            let bindings: BindingSet =
              shader_bind_set(pipeline, texture_binding, sampler_binding);
            print(bindings);
            return 1;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        function.body.get(3),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::ShaderBinding { kind, slot, .. },
            ..
        }) if ty.render() == "Binding" && kind == "texture_binding" && *slot == 0
    ));
    assert!(matches!(
        function.body.get(4),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::ShaderBinding { kind, slot, .. },
            ..
        }) if ty.render() == "Binding" && kind == "sampler_binding" && *slot == 1
    ));
    assert!(matches!(
        function.body.get(5),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::ShaderBindSet { bindings, .. },
            ..
        }) if ty.render() == "BindingSet" && bindings.len() == 2
    ));
}

#[test]
fn lowers_shader_buffer_binding_builtins() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let uniform_buffer: ref Buffer = alloc_buffer(64, 0);
            let storage_buffer: ref Buffer = alloc_buffer(128, 1);
            let uniform_binding: Binding = shader_uniform_binding(2, uniform_buffer);
            let storage_binding: Binding = shader_storage_binding(3, storage_buffer);
            print(uniform_binding);
            print(storage_binding);
            return 1;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        function.body.get(2),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::ShaderBinding { kind, slot, .. },
            ..
        }) if ty.render() == "Binding" && kind == "uniform_binding" && *slot == 2
    ));
    assert!(matches!(
        function.body.get(3),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::ShaderBinding { kind, slot, .. },
            ..
        }) if ty.render() == "Binding" && kind == "storage_binding" && *slot == 3
    ));
}

#[test]
fn lowers_shader_buffer_binding_layout_builtins() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let uniform_buffer: ref Buffer = alloc_buffer(64, 0);
            let storage_buffer: ref Buffer = alloc_buffer(128, 1);
            let uniform_binding: Binding =
              shader_uniform_binding_layout(2, "std140", uniform_buffer);
            let storage_binding: Binding =
              shader_storage_binding_layout(3, "std430", storage_buffer);
            print(uniform_binding);
            print(storage_binding);
            return 1;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        function.body.get(2),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::ShaderBinding { kind, slot, layout: Some(layout), .. },
            ..
        }) if ty.render() == "Binding"
            && kind == "uniform_binding"
            && *slot == 2
            && layout == "std140"
    ));
    assert!(matches!(
        function.body.get(3),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::ShaderBinding { kind, slot, layout: Some(layout), .. },
            ..
        }) if ty.render() == "Binding"
            && kind == "storage_binding"
            && *slot == 3
            && layout == "std430"
    ));
}

#[test]
fn lowers_shader_packet_binding_builtins() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let packet: NovaPanelPacket = nova_panel_packet(1, 2, 3, 4, 5, 6);
            let uniform_binding: Binding = shader_packet_uniform_binding(4, packet);
            let storage_binding: Binding = shader_packet_storage_binding(5, packet);
            print(uniform_binding);
            print(storage_binding);
            return 1;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        function.body.get(1),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::ShaderBinding {
                kind,
                slot,
                layout: Some(layout),
                profile_contract: Some(profile_contract),
                ..
            },
            ..
        }) if ty.render() == "Binding"
            && kind == "uniform_binding"
            && *slot == 4
            && layout == "std140"
            && profile_contract == "shader.profile.packet.nova.v1"
    ));
    assert!(matches!(
        function.body.get(2),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::ShaderBinding {
                kind,
                slot,
                layout: Some(layout),
                profile_contract: Some(profile_contract),
                ..
            },
            ..
        }) if ty.render() == "Binding"
            && kind == "storage_binding"
            && *slot == 5
            && layout == "std430"
            && profile_contract == "shader.profile.packet.nova.v1"
    ));
}
