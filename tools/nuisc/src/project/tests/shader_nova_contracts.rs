use super::*;
use std::path::PathBuf;

#[test]
fn validates_shader_packet_contract_from_cpu_usage() {
    let project = project_with_modules(vec![
        (
            "main.ns",
            r#"
            mod cpu Main {
              fn main() {
                let packet: SurfaceShaderPacket =
                  shader_profile_packet("SurfaceShader", 1, 2, 3);
              }
            }
            "#,
        ),
        (
            "surface_shader.ns",
            r#"
            mod shader SurfaceShader {
              fn profile() {
                const vertex_count: i64 = 4;
                const instance_count: i64 = 1;
                const packet_color_slot: i64 = 0;
                const packet_speed_slot: i64 = 1;
                const packet_radius_slot: i64 = 2;
                const packet_tag: i64 = 17;
                const material_mode: i64 = 2;
                const pass_kind: i64 = 1;
                const packet_field_count: i64 = 3;
                let profile_target: Target = shader_target("rgba8_unorm", 160, 120);
                let profile_view: Viewport = shader_viewport(160, 120);
                let profile_pipe: Pipeline = shader_pipeline("lit_sphere", "triangle_strip");
                let profile_wgsl: ShaderModule = shader_inline_wgsl("lit_sphere", "stub");
              }
            }
            "#,
        ),
    ]);

    validate_shader_packet_contract(&project, "SurfaceShader").unwrap();
}

#[test]
fn validates_nova_panel_contract_from_struct_literal_usage() {
    let project = project_with_modules(vec![
        (
            "main.ns",
            r#"
            mod cpu Main {
              fn main() {
                let header: NovaHeaderPacket = nova_header_packet(8);
                let slider_color: NovaSliderPacket = nova_slider_packet(1);
                let slider_speed: NovaSliderPacket = nova_slider_packet(2);
                let slider_radius: NovaSliderPacket = nova_slider_packet(3);
                let sliders: NovaSliderGroupPacket =
                  nova_slider_group_packet(slider_color, slider_speed, slider_radius);
                let toggle: NovaTogglePacket = nova_toggle_packet(1);
                let progress: NovaProgressPacket = nova_progress_packet(2);
                let meter: NovaMeterPacket = nova_meter_packet(3);
                let button: NovaButtonPacket = nova_button_packet(1, 8);
                let text_input: NovaTextInputPacket = nova_text_input_packet(4, 1);
                let select: NovaSelectPacket = nova_select_packet(0, 8);
                let checkbox: NovaCheckboxPacket = nova_checkbox_packet(1, 8);
                let radio: NovaRadioPacket = nova_radio_packet(1, 4, 8);
                let textarea: NovaTextAreaPacket = nova_textarea_packet(3, 1);
                let tabs: NovaTabsPacket = nova_tabs_packet(0, 4, 8);
                let list: NovaListPacket = nova_list_packet(1, 5, 8);
                let table: NovaTablePacket = nova_table_packet(4, 3, 1);
                let tree: NovaTreePacket = nova_tree_packet(1, 6, 1, 8);
                let inspector: NovaInspectorPacket = nova_inspector_packet(1, 4, 1, 8);
                let outline: NovaOutlinePacket = nova_outline_packet(1, 6, 1, 8);
                let theme: NovaThemePacket = nova_theme_packet(8, 3, 1, 2);
                let surface: NovaSurfacePacket = nova_surface_packet(3, 2, 1, 4);
                let viewport: NovaViewportPacket = nova_viewport_packet(2, 1, 48, 18);
                let layer: NovaLayerPacket = nova_layer_packet(1, 2, 1, 0);
                let scene: NovaScenePacket = nova_scene_packet(7, 2, 3, 1);
                let camera: NovaCameraPacket = nova_camera_packet(1, 2, 12, 9);
                let material: NovaMaterialPacket = nova_material_packet(1, 8, 3, 2);
                let light: NovaLightPacket = nova_light_packet(1, 12, 9, 8);
                let mesh: NovaMeshPacket = nova_mesh_packet(1, 12, 9, 8);
                let transform: NovaTransformPacket = nova_transform_packet(12, 1, 9, 2);
                let node: NovaNodePacket = nova_node_packet(2, 1, 8, 2);
                let scene_link: NovaSceneLinkPacket = nova_scene_link_packet(2, 12, 9, 8, 1, 1);
                let instance: NovaInstancePacket = nova_instance_packet(2, 3, 2, 1, 8, 1);
                let scene_graph: NovaSceneGraphPacket = nova_scene_graph_packet(2, 6, 3, 3, 1);
                let scene_node: NovaSceneNodePacket = nova_scene_node_packet(2, 4, 5, 3, 1);
                let instance_group: NovaInstanceGroupPacket = nova_instance_group_packet(3, 4, 3, 1, 8);
                let scene_cluster: NovaSceneClusterPacket = nova_scene_cluster_packet(2, 6, 3, 8, 1);
                let visibility: NovaVisibilityPacket = nova_visibility_packet(3, 5, 1, 2, 7);
                let cull: NovaCullPacket = nova_cull_packet(3, 4, 1, 2, 7);
                let lod: NovaLodPacket = nova_lod_packet(3, 4, 1, 9, 2);
                let streaming: NovaStreamingPacket = nova_streaming_packet(3, 2, 1, 6, 2);
                let residency: NovaResidencyPacket = nova_residency_packet(3, 2, 1, 6, 7);
                let eviction: NovaEvictionPacket = nova_eviction_packet(3, 1, 1, 5, 6);
                let prefetch: NovaPrefetchPacket = nova_prefetch_packet(3, 2, 1, 5, 5);
                let budget: NovaBudgetPacket = nova_budget_packet(3, 12, 7, 5, 1);
                let pressure: NovaPressurePacket = nova_pressure_packet(3, 2, 7, 1, 6);
                let thermal: NovaThermalPacket = nova_thermal_packet(3, 2, 1, 1, 6);
                let power: NovaPowerPacket = nova_power_packet(3, 2, 1, 1, 6);
                let latency: NovaLatencyPacket = nova_latency_packet(3, 4, 2, 1, 7);
                let frame_pacing: NovaFramePacingPacket = nova_frame_pacing_packet(3, 4, 1, 1, 7);
                let frame_variance: NovaFrameVariancePacket = nova_frame_variance_packet(3, 2, 1, 4, 7);
                let jank: NovaJankPacket = nova_jank_packet(3, 2, 1, 4, 7);
                let pass: NovaPassPacket = nova_pass_packet(1, 8, 4, 2);
                let frame: NovaFramePacket = nova_frame_packet(7, 1, 1, 9);
                let target: NovaTargetPacket = nova_target_packet(1, 48, 18, 8);
                let frame_graph: NovaFrameGraphPacket = nova_frame_graph_packet(2, 1, 1, 2);
                let attachment: NovaAttachmentPacket = nova_attachment_packet(0, 8, 1, 1);
                let pass_chain: NovaPassChainPacket = nova_pass_chain_packet(2, 1, 1, 8);
                let barrier: NovaBarrierPacket = nova_barrier_packet(1, 1, 2, 8);
                let resource_set: NovaResourceSetPacket = nova_resource_set_packet(2, 1, 1, 8);
                let schedule: NovaSchedulePacket = nova_schedule_packet(2, 4, 9, 1);
                let submission: NovaSubmissionPacket = nova_submission_packet(2, 1, 1, 8);
                let queue: NovaQueuePacket = nova_queue_packet(1, 2, 9, 1);
                let semaphore: NovaSemaphorePacket = nova_semaphore_packet(1, 2, 1, 3);
                let timeline: NovaTimelinePacket = nova_timeline_packet(9, 1, 0, 3);
                let fence: NovaFencePacket = nova_fence_packet(1, 0, 3, 1);
                let signal: NovaSignalPacket = nova_signal_packet(1, 2, 3, 1);
                let event: NovaEventPacket = nova_event_packet(1, 2, 3, 1);
                let dispatch: NovaDispatchPacket = nova_dispatch_packet(1, 2, 3, 1);
                let feedback: NovaFeedbackPacket = nova_feedback_packet(1, 2, 3, 1);
                let intent: NovaIntentPacket = nova_intent_packet(1, 2, 3, 1);
                let reaction: NovaReactionPacket = nova_reaction_packet(1, 2, 3, 1);
                let outcome: NovaOutcomePacket = nova_outcome_packet(1, 2, 3, 1);
                let resolution: NovaResolutionPacket = nova_resolution_packet(1, 2, 3, 1);
                let commit: NovaCommitPacket = nova_commit_packet(1, 2, 3, 1);
                let snapshot: NovaSnapshotPacket = nova_snapshot_packet(1, 2, 3, 1);
                let checkpoint: NovaCheckpointPacket = nova_checkpoint_packet(1, 2, 3, 1);
                let focus: NovaFocusPacket = nova_focus_packet(2);
                let panel: NovaPanelPacket = nova_panel_from_parts(
                  header,
                  sliders,
                  toggle,
                  progress,
                  meter,
                  button,
                  text_input,
                  select,
                  checkbox,
                  radio,
                  textarea,
                  tabs,
                  list,
                  table,
                  tree,
                  inspector,
                  outline,
                  theme,
                  surface,
                  viewport,
                  layer,
                  scene,
                  camera,
                  material,
                  light,
                  mesh,
                  transform,
                  node,
                  scene_link,
                  instance,
                  scene_graph,
                  scene_node,
                  instance_group,
                  scene_cluster,
                  visibility,
                  cull,
                  lod,
                  streaming,
                  residency,
                  eviction,
                  prefetch,
                  budget,
                  pressure,
                  thermal,
                  power,
                  latency,
                  frame_pacing,
                  frame_variance,
                  jank,
                  pass,
                  frame,
                  target,
                  frame_graph,
                  attachment,
                  pass_chain,
                  barrier,
                  resource_set,
                  schedule,
                  submission,
                  queue,
                  semaphore,
                  timeline,
                  fence,
                  signal,
                  event,
                  dispatch,
                  feedback,
                  intent,
                  reaction,
                  outcome,
                  resolution,
                  commit,
                  snapshot,
                  checkpoint,
                  focus
                );
                print(panel);
              }
            }
            "#,
        ),
        (
            "surface_shader.ns",
            r#"
            mod shader SurfaceShader {
              fn profile() {
                const vertex_count: i64 = 4;
                const instance_count: i64 = 1;
                const slider_color_slot: i64 = 0;
                const slider_speed_slot: i64 = 1;
                const slider_radius_slot: i64 = 2;
                const header_accent_slot: i64 = 3;
                const toggle_live_slot: i64 = 4;
                const focus_slot: i64 = 5;
                const packet_tag: i64 = 17;
                const material_mode: i64 = 2;
                const pass_kind: i64 = 1;
                const packet_field_count: i64 = 6;
                let profile_target: Target = shader_target("rgba8_unorm", 160, 120);
                let profile_view: Viewport = shader_viewport(160, 120);
                let profile_pipe: Pipeline = shader_pipeline("lit_sphere", "triangle_strip");
                let profile_wgsl: ShaderModule = shader_inline_wgsl("lit_sphere", "stub");
              }
            }
            "#,
        ),
    ]);

    validate_shader_packet_contract(&project, "SurfaceShader").unwrap();
}

#[test]
fn compiles_real_shader_async_policy_project() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/projects/domains/shader_async_policy_profile_demo");
    let artifacts = crate::pipeline::compile_source_path(&root).unwrap();
    assert!(artifacts
        .nir
        .functions
        .iter()
        .any(|function| function.name == "ShaderTaskAsyncShapes.async_policy_summary_completed"));
}

#[test]
fn rejects_shader_packet_field_count_mismatch() {
    let project = project_with_modules(vec![
        (
            "main.ns",
            r#"
            mod cpu Main {
              fn main() {
                let packet: SurfaceShaderPacket =
                  shader_profile_packet("SurfaceShader", 1, 2, 3);
              }
            }
            "#,
        ),
        (
            "surface_shader.ns",
            r#"
            mod shader SurfaceShader {
              fn profile() {
                const vertex_count: i64 = 4;
                const instance_count: i64 = 1;
                const packet_color_slot: i64 = 0;
                const packet_speed_slot: i64 = 1;
                const packet_radius_slot: i64 = 2;
                const packet_tag: i64 = 17;
                const material_mode: i64 = 2;
                const pass_kind: i64 = 1;
                const packet_field_count: i64 = 4;
                let profile_target: Target = shader_target("rgba8_unorm", 160, 120);
                let profile_view: Viewport = shader_viewport(160, 120);
                let profile_pipe: Pipeline = shader_pipeline("lit_sphere", "triangle_strip");
                let profile_wgsl: ShaderModule = shader_inline_wgsl("lit_sphere", "stub");
              }
            }
            "#,
        ),
    ]);

    let error = validate_shader_packet_contract(&project, "SurfaceShader").unwrap_err();
    assert!(error.contains("packet_field_count = 3"));
}

#[test]
fn nova_panel_packet_requires_nova_support_surface() {
    let contract = ShaderPacketContract {
        type_name: "NovaPanelPacket".to_owned(),
        field_count: 6,
    };
    assert_eq!(
        shader_packet_support_surface_contract(&contract),
        ["shader.profile.packet.nova.v1"]
    );
}

#[test]
fn project_link_accepts_draw_only_shader_bridge_at_nir_level() {
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
        (
            "surface_shader.ns",
            r#"
            mod shader SurfaceShader {
              fn profile() {
                const vertex_count: i64 = 4;
                const instance_count: i64 = 1;
                const packet_color_slot: i64 = 0;
                const packet_speed_slot: i64 = 1;
                const packet_radius_slot: i64 = 2;
                const packet_tag: i64 = 17;
                const material_mode: i64 = 2;
                const pass_kind: i64 = 1;
                const packet_field_count: i64 = 3;
                let profile_target: Target = shader_target("rgba8_unorm", 160, 120);
                let profile_view: Viewport = shader_viewport(160, 120);
                let profile_pipe: Pipeline = shader_pipeline("lit_sphere", "triangle_strip");
                let profile_wgsl: ShaderModule = shader_inline_wgsl("lit_sphere", "stub");
              }
            }
            "#,
        ),
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

#[test]
fn infers_project_route_payload_type_from_entry_with_shared_cpu_helper() {
    let project = project_with_modules(vec![
        (
            "main.ns",
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
        ),
        (
            "surface_shader.ns",
            r#"
            mod shader SurfaceShader {
              fn profile() {
                const vertex_count: i64 = 4;
                const instance_count: i64 = 1;
                const packet_color_slot: i64 = 0;
                const packet_speed_slot: i64 = 1;
                const packet_radius_slot: i64 = 2;
                const packet_field_count: i64 = 3;
                let profile_target: Target = shader_target("rgba8_unorm", 160, 120);
                let profile_view: Viewport = shader_viewport(160, 120);
                let profile_pipe: Pipeline = shader_pipeline("lit_sphere", "triangle_strip");
                let profile_wgsl: ShaderModule = shader_inline_wgsl("lit_sphere", "stub");
              }
            }
            "#,
        ),
        (
            "fabric_plane.ns",
            r#"
            mod data FabricPlane {
              fn profile() {
                let uplink_window: Window<SurfaceShaderPacket> =
                  data_receive_uplink_window("SurfaceShader");
              }
            }
            "#,
        ),
        (
            "shader_task_async_shapes.ns",
            r#"
            mod cpu ShaderTaskAsyncShapes {
              pub fn make_packet() -> SurfaceShaderPacket {
                return shader_profile_packet("SurfaceShader", 1, 2, 3);
              }
            }
            "#,
        ),
    ]);

    let payload = infer_project_route_payload_type(&project, "cpu.Main", "FabricPlane", true)
        .unwrap()
        .expect("expected payload contract");
    assert_eq!(payload.render(), "Window<SurfaceShaderPacket>");
}

#[test]
fn validates_project_links_against_nir_with_shared_cpu_helper_indirection() {
    let project = project_with_modules(vec![
        (
            "main.ns",
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
        ),
        (
            "surface_shader.ns",
            r#"
            mod shader SurfaceShader {
              fn profile() {
                const vertex_count: i64 = 4;
                const instance_count: i64 = 1;
                const packet_color_slot: i64 = 0;
                const packet_speed_slot: i64 = 1;
                const packet_radius_slot: i64 = 2;
                const packet_tag: i64 = 17;
                const material_mode: i64 = 2;
                const pass_kind: i64 = 1;
                const packet_field_count: i64 = 3;
                let profile_target: Target = shader_target("rgba8_unorm", 160, 120);
                let profile_view: Viewport = shader_viewport(160, 120);
                let profile_pipe: Pipeline = shader_pipeline("lit_sphere", "triangle_strip");
                let profile_wgsl: ShaderModule = shader_inline_wgsl("lit_sphere", "stub");
              }
            }
            "#,
        ),
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
        (
            "shader_task_async_shapes.ns",
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

#[test]
fn materializes_shader_and_data_type_contract_nodes_into_yir() {
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
        (
            "surface_shader.ns",
            r#"
            mod shader SurfaceShader {
              fn profile() {
                const vertex_count: i64 = 4;
                const instance_count: i64 = 1;
                const packet_color_slot: i64 = 0;
                const packet_speed_slot: i64 = 1;
                const packet_radius_slot: i64 = 2;
                const packet_tag: i64 = 17;
                const material_mode: i64 = 2;
                const pass_kind: i64 = 1;
                const packet_field_count: i64 = 3;
                let profile_target: Target = shader_target("rgba8_unorm", 160, 120);
                let profile_view: Viewport = shader_viewport(160, 120);
                let profile_pipe: Pipeline = shader_pipeline("lit_sphere", "triangle_strip");
                let profile_wgsl: ShaderModule = shader_inline_wgsl("lit_sphere", "stub");
              }
            }
            "#,
        ),
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
