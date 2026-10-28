use super::*;

pub(super) fn project_link_accepts_draw_only_shader_bridge_at_nir_level() {
    let surface_shader = standard_surface_shader_profile_module(false);
    let project = project_with_modules(vec![
        (
            "main.ns",
            r#"
            use data FabricPlane;
            use shader SurfaceShader;

            mod cpu Main {
              fn main() {
                let color: i64 = shader_profile_color_seed("SurfaceShader", 10, 0);
                let speed: i64 = shader_profile_speed_seed("SurfaceShader", 0, 1, 20);
                let radius: i64 = shader_profile_radius_seed("SurfaceShader", 30, 0);
                let packet: SurfaceShaderPacket =
                  shader_profile_packet("SurfaceShader", color, speed, radius);
                data_profile_bind_core("FabricPlane");
                let handles: HandleTable<FabricPlaneBindings> =
                  data_profile_handle_table("FabricPlane");
                let gpu_packet: Window<SurfaceShaderPacket> =
                  data_profile_send_uplink("FabricPlane", packet);
                let pass_result: ShaderResult<Pass> =
                  shader_result(shader_profile_begin_pass("SurfaceShader"));
                let draw_result: ShaderResult<Frame> = shader_result(
                  shader_profile_draw_instanced(
                    "SurfaceShader",
                    shader_value(pass_result),
                    gpu_packet
                  )
                );
                let host_frame: Window<Frame> =
                  data_profile_send_downlink("FabricPlane", shader_value(draw_result));
                cpu_present_frame(host_frame);
              }
            }
            "#,
        ),
        ("surface_shader.ns", surface_shader.as_str()),
        (
            "fabric_plane.ns",
            r#"
            mod data FabricPlane {
              fn profile() {
                const bind_core: i64 = 0;
                const handle_table: i64 = 1;
                const window_offset: i64 = 0;
                const uplink_len: i64 = 1;
                const downlink_len: i64 = 1;
                let cpu_to_shader: Marker<CpuToShader> = data_marker("cpu_to_shader");
                let shader_to_cpu: Marker<ShaderToCpu> = data_marker("shader_to_cpu");
                let uplink_pipe: Marker<UplinkPipe> = data_marker("uplink_pipe");
                let downlink_pipe: Marker<DownlinkPipe> = data_marker("downlink_pipe");
                let uplink_pipe_class: Marker<UplinkPipeClass> = data_marker("uplink_pipe_class");
                let downlink_pipe_class: Marker<DownlinkPipeClass> = data_marker("downlink_pipe_class");
                let uplink_payload_class: Marker<PayloadClassWindow> = data_marker("uplink_payload_class");
                let downlink_payload_class: Marker<PayloadClassWindow> = data_marker("downlink_payload_class");
                let uplink_payload_shape: Marker<PayloadShapeWindowSurfaceShaderPacket> = data_marker("uplink_payload_shape");
                let downlink_payload_shape: Marker<PayloadShapeWindowFrame> = data_marker("downlink_payload_shape");
                let uplink_window_policy: Marker<UplinkWindowPolicy> = data_marker("uplink_window_policy");
                let downlink_window_policy: Marker<DownlinkWindowPolicy> = data_marker("downlink_window_policy");
              }
            }
            "#,
        ),
    ]);
    let mut project = project;
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "shader.SurfaceShader".to_owned(),
        via: Some("data.FabricPlane".to_owned()),
    }];

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    validate_project_links_against_nir(&project, &nir).unwrap();
}

pub(super) fn infers_project_route_payload_type_from_entry_with_shared_cpu_helper() {
    let project = shader_task_async_shapes_project(
        r#"
        use cpu ShaderTaskAsyncShapes;
        use data FabricPlane;
        use shader SurfaceShader;

        mod cpu Main {
          fn main() {
            let packet: SurfaceShaderPacket =
              ShaderTaskAsyncShapes.make_packet();
            let gpu_packet: Window<SurfaceShaderPacket> =
              data_profile_send_uplink("FabricPlane", packet);
          }
        }
        "#,
        false,
        r#"
        mod data FabricPlane {
          fn profile() {
            let uplink_window: Window<SurfaceShaderPacket> =
              data_receive_uplink_window("SurfaceShader");
          }
        }
        "#,
        r#"
        mod cpu ShaderTaskAsyncShapes {
          pub fn make_packet() -> SurfaceShaderPacket {
            return shader_profile_packet("SurfaceShader", 1, 2, 3);
          }
        }
        "#,
    );

    let payload = infer_project_route_payload_type(&project, "cpu.Main", "FabricPlane", true)
        .unwrap()
        .expect("expected payload contract");
    assert_eq!(payload.render(), "Window<SurfaceShaderPacket>");
}

pub(super) fn validates_project_links_against_nir_with_shared_cpu_helper_indirection() {
    let project = shader_task_async_shapes_project(
        r#"
        use cpu ShaderTaskAsyncShapes;
        use data FabricPlane;
        use shader SurfaceShader;

        mod cpu Main {
          fn main() {
            let packet: SurfaceShaderPacket =
              ShaderTaskAsyncShapes.make_packet();
            let gpu_packet: Window<SurfaceShaderPacket> =
              ShaderTaskAsyncShapes.send_packet(packet);
            let host_frame: Window<Frame> =
              ShaderTaskAsyncShapes.render_frame(gpu_packet);
            cpu_present_frame(host_frame);
          }
        }
        "#,
        false,
        r#"
        mod data FabricPlane {
          fn profile() {
            const bind_core: i64 = 0;
            const handle_table: i64 = 1;
            const window_offset: i64 = 0;
            const uplink_len: i64 = 1;
            const downlink_len: i64 = 1;
            let cpu_to_shader: Marker<CpuToShader> = data_marker("cpu_to_shader");
            let shader_to_cpu: Marker<ShaderToCpu> = data_marker("shader_to_cpu");
            let uplink_pipe: Marker<UplinkPipe> = data_marker("uplink_pipe");
            let downlink_pipe: Marker<DownlinkPipe> = data_marker("downlink_pipe");
            let uplink_pipe_class: Marker<UplinkPipeClass> = data_marker("uplink_pipe_class");
            let downlink_pipe_class: Marker<DownlinkPipeClass> = data_marker("downlink_pipe_class");
            let uplink_payload_class: Marker<PayloadClassWindow> = data_marker("uplink_payload_class");
            let downlink_payload_class: Marker<PayloadClassWindow> = data_marker("downlink_payload_class");
            let uplink_payload_shape: Marker<PayloadShapeWindowSurfaceShaderPacket> = data_marker("uplink_payload_shape");
            let downlink_payload_shape: Marker<PayloadShapeWindowFrame> = data_marker("downlink_payload_shape");
            let uplink_window_policy: Marker<UplinkWindowPolicy> = data_marker("uplink_window_policy");
            let downlink_window_policy: Marker<DownlinkWindowPolicy> = data_marker("downlink_window_policy");
          }
        }
        "#,
        r#"
        use data FabricPlane;
        use shader SurfaceShader;

        mod cpu ShaderTaskAsyncShapes {
          pub fn make_packet() -> SurfaceShaderPacket {
            let color: i64 = shader_profile_color_seed("SurfaceShader", 10, 0);
            let speed: i64 = shader_profile_speed_seed("SurfaceShader", 0, 1, 20);
            let radius: i64 = shader_profile_radius_seed("SurfaceShader", 30, 0);
            return shader_profile_packet("SurfaceShader", color, speed, radius);
          }

          pub fn send_packet(packet: SurfaceShaderPacket) -> Window<SurfaceShaderPacket> {
            data_profile_bind_core("FabricPlane");
            let handles: HandleTable<FabricPlaneBindings> =
              data_profile_handle_table("FabricPlane");
            return data_profile_send_uplink("FabricPlane", packet);
          }

          pub fn render_frame(gpu_packet: Window<SurfaceShaderPacket>) -> Window<Frame> {
            let pass_result: ShaderResult<Pass> =
              shader_result(shader_profile_begin_pass("SurfaceShader"));
            let draw_result: ShaderResult<Frame> = shader_result(
              shader_profile_draw_instanced(
                "SurfaceShader",
                shader_value(pass_result),
                gpu_packet
              )
            );
            return data_profile_send_downlink("FabricPlane", shader_value(draw_result));
          }
        }
        "#,
    );
    let mut project = project;
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "shader.SurfaceShader".to_owned(),
        via: Some("data.FabricPlane".to_owned()),
    }];

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    validate_project_links_against_nir(&project, &nir).unwrap();
}

pub(super) fn materializes_shader_and_data_type_contract_nodes_into_yir() {
    let surface_shader = standard_surface_shader_profile_module(false);
    let project = project_with_modules(vec![
        (
            "main.ns",
            r#"
            use data FabricPlane;
            use shader SurfaceShader;

            mod cpu Main {
              fn main() {
                let packet: SurfaceShaderPacket =
                  shader_profile_packet("SurfaceShader", 1, 2, 3);
                let gpu_packet: Window<SurfaceShaderPacket> =
                  data_profile_send_uplink("FabricPlane", packet);
                let frame: Frame = shader_profile_render("SurfaceShader", gpu_packet);
                let host_frame: Window<Frame> =
                  data_profile_send_downlink("FabricPlane", frame);
                print(host_frame);
              }
            }
            "#,
        ),
        ("surface_shader.ns", surface_shader.as_str()),
        (
            "fabric_plane.ns",
            r#"
            mod data FabricPlane {
              fn profile() {
                const window_offset: i64 = 0;
                const uplink_len: i64 = 1;
                const downlink_len: i64 = 1;
                data_bind_core(1);
                let profile_handles: HandleTable<FabricBindings> =
                  data_handle_table("host=cpu0", "render=shader0");
                let cpu_to_shader: Marker<CpuToShader> = data_marker("cpu_to_shader");
                let shader_to_cpu: Marker<ShaderToCpu> = data_marker("shader_to_cpu");
                let uplink_pipe: Marker<UplinkPipe> = data_marker("uplink_pipe");
                let downlink_pipe: Marker<DownlinkPipe> = data_marker("downlink_pipe");
                let uplink_pipe_class: Marker<UplinkPipeClass> = data_marker("uplink_pipe_class");
                let downlink_pipe_class: Marker<DownlinkPipeClass> = data_marker("downlink_pipe_class");
                let uplink_payload_class: Marker<PayloadClassWindow> = data_marker("uplink_payload_class");
                let downlink_payload_class: Marker<PayloadClassWindow> = data_marker("downlink_payload_class");
                let uplink_payload_shape: Marker<PayloadShapeWindowSurfaceShaderPacket> = data_marker("uplink_payload_shape");
                let downlink_payload_shape: Marker<PayloadShapeWindowFrame> = data_marker("downlink_payload_shape");
                let uplink_window_policy: Marker<UplinkWindowPolicy> = data_marker("uplink_window_policy");
                let downlink_window_policy: Marker<DownlinkWindowPolicy> = data_marker("downlink_window_policy");
                let uplink_window: Window<i64> =
                  data_immutable_window(window_offset, window_offset, uplink_len);
                let downlink_window: WindowMut<i64> =
                  data_copy_window(window_offset, window_offset, downlink_len);
              }
            }
            "#,
        ),
    ]);
    let mut project = project;
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "shader.SurfaceShader".to_owned(),
        via: Some("data.FabricPlane".to_owned()),
    }];

    let mut yir = YirModule::new("0.1");
    apply_project_support_modules_to_yir(&project, &mut yir).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.name == "project_profile_shader_SurfaceShader_packet_type"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.name == "project_profile_data_FabricPlane_uplink_payload_shape_type"));
    assert!(yir.nodes.iter().any(|node| {
        node.name.contains("_bridge_stage_type")
            && node
                .op
                .args
                .first()
                .is_some_and(|value| value == "uplink=windowed;downlink=windowed")
    }));
    assert!(yir.nodes.iter().any(|node| {
        node.name.contains("_uplink_bridge_payload_type")
            && node
                .op
                .args
                .first()
                .is_some_and(|value| value == "Window<SurfaceShaderPacket>")
    }));
    assert!(yir.nodes.iter().any(|node| {
        node.name.contains("_downlink_bridge_payload_type")
            && node
                .op
                .args
                .first()
                .is_some_and(|value| value == "Window<Frame>")
    }));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.name == "scheduler_contract_shader_lane_policy_type"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.name == "scheduler_contract_data_clock_type"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.name == "scheduler_contract_shader_lane_capability_type"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.name == "scheduler_contract_shader_bridge_capability_type"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.name == "scheduler_contract_shader_result_lane_type"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.name == "scheduler_contract_shader_result_capability_type"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.name == "scheduler_contract_shader_observer_role_variant_type"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.name == "scheduler_contract_shader_summary_capability_type"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.name == "scheduler_contract_shader_summary_class_type"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.name == "scheduler_contract_shader_observer_source_class_type"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.name == "scheduler_contract_shader_observer_stage_class_type"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.name == "scheduler_contract_shader_observer_scope_class_type"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.name == "scheduler_contract_shader_observer_branch_class_type"));
}
