use super::*;
use nuis_semantics::model::{NirExpr, NirStmt};
use std::{collections::BTreeMap, fs, path::PathBuf};
use yir_core::EdgeKind;

fn darwin_x86_64_shader_project_abis() -> Vec<ProjectAbiRequirement> {
    vec![
        ProjectAbiRequirement {
            domain: "cpu".to_owned(),
            abi: "cpu.x86_64.apple_sysv64".to_owned(),
        },
        ProjectAbiRequirement {
            domain: "shader".to_owned(),
            abi: "shader.metal.x86_64.msl2_4".to_owned(),
        },
        ProjectAbiRequirement {
            domain: "data".to_owned(),
            abi: "data.fabric.macos.x86_64.v1".to_owned(),
        },
    ]
}

fn compiled_domain_project(path: &str) -> crate::pipeline::PipelineArtifacts {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path);
    crate::pipeline::compile_source_path(&root).unwrap()
}

fn inline_wgsl_source_named<'a>(
    artifacts: &'a crate::pipeline::PipelineArtifacts,
    shader_name: &str,
) -> &'a str {
    artifacts
        .yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "shader"
                && node.op.instruction == "inline_wgsl"
                && node.op.args.first().map(String::as_str) == Some(shader_name)
        })
        .and_then(|node| node.op.args.get(1).map(String::as_str))
        .unwrap_or_else(|| panic!("missing inline_wgsl source for {shader_name}"))
}

fn write_temp_shader_data_project(
    name: &str,
    entry_source: &str,
    extra_modules: Vec<(&str, &str)>,
    links: &[&str],
) -> PathBuf {
    let mut manifest = r#"
name = "shader_data_test"
version = "0.1.0"
entry = "main.ns"
modules = ["main.ns", "surface_shader.ns", "fabric_plane.ns"]
abi = [
  "cpu=cpu.arm64.apple_aapcs64",
  "shader=shader.metal.msl2_4",
  "data=data.fabric.macos.arm64.v1",
]
"#
    .trim_start()
    .to_owned();
    test_support::append_manifest_links(&mut manifest, links);
    test_support::write_temp_project_fixture(name, &manifest, entry_source, extra_modules)
}

fn standard_surface_shader_profile_module(include_data_use: bool) -> String {
    standard_surface_shader_profile_module_with_packet_field_count(include_data_use, 3)
}

fn standard_surface_shader_profile_module_with_packet_field_count(
    include_data_use: bool,
    packet_field_count: i64,
) -> String {
    let use_data = if include_data_use {
        "use data FabricPlane;\n\n"
    } else {
        ""
    };
    format!(
        r#"{use_data}mod shader SurfaceShader {{
  fn profile() {{
    const vertex_count: i64 = 4;
    const instance_count: i64 = 1;
    const packet_color_slot: i64 = 0;
    const packet_speed_slot: i64 = 1;
    const packet_radius_slot: i64 = 2;
    const packet_tag: i64 = 17;
    const material_mode: i64 = 2;
    const pass_kind: i64 = 1;
    const packet_field_count: i64 = {packet_field_count};
    let profile_target: Target = shader_target("rgba8_unorm", 160, 120);
    let profile_view: Viewport = shader_viewport(160, 120);
    let profile_pipe: Pipeline = shader_pipeline("std_shader_packet_bridge", "triangle_strip");
    let profile_wgsl: ShaderModule = shader_inline_wgsl("std_shader_packet_bridge", wgsl {{
      struct VsOut {{
        @builtin(position) pos: vec4<f32>,
        @location(0) uv: vec2<f32>,
      }};

      @vertex
      fn vs_main(@builtin(vertex_index) vid: u32) -> VsOut {{
        var out: VsOut;
        let x: f32 = f32((vid << 1u) & 2u);
        let y: f32 = f32(vid & 2u);
        out.pos = vec4<f32>(x * 2.0 - 1.0, y * -2.0 + 1.0, 0.0, 1.0);
        out.uv = vec2<f32>(x, y);
        return out;
      }}

      @fragment
      fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {{
        let color = vec3<f32>(0.18 + uv.x * 0.22, 0.28 + uv.y * 0.18, 0.86);
        return vec4<f32>(color, 1.0);
      }}
    }});
  }}
}}
"#
    )
}

fn standard_nova_surface_shader_profile_module() -> String {
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
    let profile_pipe: Pipeline = shader_pipeline("std_shader_packet_bridge", "triangle_strip");
    let profile_wgsl: ShaderModule = shader_inline_wgsl("std_shader_packet_bridge", wgsl {
      struct VsOut {
        @builtin(position) pos: vec4<f32>,
        @location(0) uv: vec2<f32>,
      };

      @vertex
      fn vs_main(@builtin(vertex_index) vid: u32) -> VsOut {
        var out: VsOut;
        let x: f32 = f32((vid << 1u) & 2u);
        let y: f32 = f32(vid & 2u);
        out.pos = vec4<f32>(x * 2.0 - 1.0, y * -2.0 + 1.0, 0.0, 1.0);
        out.uv = vec2<f32>(x, y);
        return out;
      }

      @fragment
      fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
        let color = vec3<f32>(0.18 + uv.x * 0.22, 0.28 + uv.y * 0.18, 0.86);
        return vec4<f32>(color, 1.0);
      }
    });
  }
}
"#
    .to_owned()
}

fn reverse_shader_data_bridge_entry() -> &'static str {
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
        print(handles);
        print(host_frame);
        return;
      }
    }
    "#
}

fn shader_fabric_plane_module(include_shader_to_cpu: bool) -> String {
    let reverse_marker = if include_shader_to_cpu {
        "            let shader_to_cpu: Marker<ShaderToCpu> = data_marker(\"shader_to_cpu\");\n"
    } else {
        ""
    };
    format!(
        r#"
        mod data FabricPlane {{
          fn profile() {{
            const window_offset: i64 = 0;
            const uplink_len: i64 = 1;
            const downlink_len: i64 = 1;
            data_bind_core(1);
            let profile_handles: HandleTable<FabricBindings> =
              data_handle_table("host=cpu0", "render=shader0");
            let cpu_to_shader: Marker<CpuToShader> = data_marker("cpu_to_shader");
{reverse_marker}            let uplink_pipe: Marker<UplinkPipe> = data_marker("uplink_pipe");
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
          }}
        }}
        "#
    )
}

fn reverse_shader_data_bridge_project(
    entry_source: &str,
    include_data_use: bool,
    fabric_plane_source: &str,
) -> LoadedProject {
    let surface_shader = standard_surface_shader_profile_module(include_data_use);
    test_support::loaded_project_fixture(
        "shader_data_test",
        vec![
            ProjectAbiRequirement {
                domain: "cpu".to_owned(),
                abi: "cpu.arm64.apple_aapcs64".to_owned(),
            },
            ProjectAbiRequirement {
                domain: "shader".to_owned(),
                abi: "shader.metal.msl2_4".to_owned(),
            },
            ProjectAbiRequirement {
                domain: "data".to_owned(),
                abi: "data.fabric.macos.arm64.v1".to_owned(),
            },
        ],
        entry_source,
        vec![
            ("surface_shader.ns", surface_shader.as_str()),
            ("fabric_plane.ns", fabric_plane_source),
        ],
    )
}

fn reverse_shader_data_bridge_project_with_link(
    entry_source: &str,
    include_data_use: bool,
    fabric_plane_source: &str,
) -> LoadedProject {
    let mut project =
        reverse_shader_data_bridge_project(entry_source, include_data_use, fabric_plane_source);
    project.manifest.links = vec![ProjectLink {
        from: "shader.SurfaceShader".to_owned(),
        to: "cpu.Main".to_owned(),
        via: Some("data.FabricPlane".to_owned()),
    }];
    project
}

fn reverse_shader_render_bridge_entry() -> &'static str {
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
        return;
      }
    }
    "#
}

fn shader_task_async_shapes_project(
    entry_source: &str,
    include_data_use: bool,
    fabric_plane_source: &str,
    helper_source: &str,
) -> LoadedProject {
    let surface_shader =
        standard_surface_shader_profile_module_with_packet_field_count(include_data_use, 3);
    test_support::loaded_project_fixture(
        "shader_task_async_test",
        vec![
            ProjectAbiRequirement {
                domain: "cpu".to_owned(),
                abi: "cpu.arm64.apple_aapcs64".to_owned(),
            },
            ProjectAbiRequirement {
                domain: "shader".to_owned(),
                abi: "shader.metal.msl2_4".to_owned(),
            },
            ProjectAbiRequirement {
                domain: "data".to_owned(),
                abi: "data.fabric.macos.arm64.v1".to_owned(),
            },
        ],
        entry_source,
        vec![
            ("surface_shader.ns", surface_shader.as_str()),
            ("fabric_plane.ns", fabric_plane_source),
            ("shader_task_async_shapes.ns", helper_source),
        ],
    )
}

fn shader_contract_fabric_plane_module(include_shader_to_cpu: bool) -> String {
    let reverse_marker = if include_shader_to_cpu {
        "    let shader_to_cpu: Marker<ShaderToCpu> = data_marker(\"shader_to_cpu\");\n"
    } else {
        ""
    };
    format!(
        r#"
        mod data FabricPlane {{
          fn profile() {{
            const bind_core: i64 = 0;
            const handle_table: i64 = 1;
            const window_offset: i64 = 0;
            const uplink_len: i64 = 1;
            const downlink_len: i64 = 1;
            let cpu_to_shader: Marker<CpuToShader> = data_marker("cpu_to_shader");
{reverse_marker}            let uplink_pipe: Marker<UplinkPipe> = data_marker("uplink_pipe");
            let downlink_pipe: Marker<DownlinkPipe> = data_marker("downlink_pipe");
            let uplink_pipe_class: Marker<UplinkPipeClass> = data_marker("uplink_pipe_class");
            let downlink_pipe_class: Marker<DownlinkPipeClass> = data_marker("downlink_pipe_class");
            let uplink_payload_class: Marker<PayloadClassWindow> = data_marker("uplink_payload_class");
            let downlink_payload_class: Marker<PayloadClassWindow> = data_marker("downlink_payload_class");
            let uplink_payload_shape: Marker<PayloadShapeWindowSurfaceShaderPacket> = data_marker("uplink_payload_shape");
            let downlink_payload_shape: Marker<PayloadShapeWindowFrame> = data_marker("downlink_payload_shape");
            let uplink_window_policy: Marker<UplinkWindowPolicy> = data_marker("uplink_window_policy");
            let downlink_window_policy: Marker<DownlinkWindowPolicy> = data_marker("downlink_window_policy");
          }}
        }}
        "#
    )
}

#[test]
fn validates_shader_packet_contract_from_cpu_usage() {
    let surface_shader = standard_surface_shader_profile_module(false);
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
        ("surface_shader.ns", surface_shader.as_str()),
    ]);

    validate_shader_packet_contract(&project, "SurfaceShader").unwrap();
}

#[test]
fn validates_nova_panel_contract_from_struct_literal_usage() {
    let surface_shader = standard_nova_surface_shader_profile_module();
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
        ("surface_shader.ns", surface_shader.as_str()),
    ]);

    validate_shader_packet_contract(&project, "SurfaceShader").unwrap();
}

#[test]
fn compiles_real_shader_async_policy_project() {
    let artifacts =
        compiled_domain_project("../../examples/projects/domains/shader_async_policy_profile_demo");
    assert!(artifacts
        .nir
        .functions
        .iter()
        .any(|function| function.name == "ShaderTaskAsyncShapes.async_policy_summary_completed"));
}

#[test]
fn lowers_real_shader_async_policy_project_with_expected_async_summary_shape() {
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

#[test]
fn lowers_real_shader_async_windowed_batch_project_with_expected_batch_summary_shape() {
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

#[test]
fn lowers_real_shader_async_fallback_project_with_expected_fallback_shape() {
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

#[test]
fn lowers_real_shader_result_project_with_expected_result_shape() {
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

#[test]
fn lowers_real_shader_draw_render_project_with_expected_dual_result_shape() {
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

#[test]
fn lowers_real_pixelmagic_packet_bridge_project_with_expected_bridge_shape() {
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

#[test]
fn lowers_real_pixelmagic_render_project_with_expected_render_shape() {
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

#[test]
fn lowers_real_pixelmagic_texture_resource_project_with_expected_resource_shape() {
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

#[test]
fn lowers_real_pixelmagic_pipeline_project_with_expected_pipeline_shape() {
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

#[test]
fn lowers_real_shader_async_schedule_project_with_expected_schedule_shape() {
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

#[test]
fn lowers_real_shader_async_fanin_project_with_expected_fanin_shape() {
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

#[test]
fn lowers_real_shader_packet_profile_project_with_expected_packet_shape() {
    let artifacts =
        compiled_domain_project("../../examples/projects/domains/shader_packet_profile_demo");

    let main = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();

    for name in [
        "color_slot",
        "speed_slot",
        "radius_slot",
        "packet_tag",
        "material_mode",
        "pass_kind",
        "packet_field_count",
        "panel_color",
        "panel_speed",
        "panel_radius",
        "render_controls",
    ] {
        assert!(main.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let { name: stmt_name, .. } if stmt_name == name
            )
        }));
    }
}

#[test]
fn lowers_real_shader_packet_bridge_project_with_expected_bridge_shape() {
    let artifacts =
        compiled_domain_project("../../examples/projects/domains/shader_packet_bridge_demo");

    let main = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();

    for name in ["frame_packet", "host_frame"] {
        assert!(main.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let { name: stmt_name, .. } if stmt_name == name
            )
        }));
    }
    for name in ["pass_result", "frame_result"] {
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
    assert!(main.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name: stmt_name,
                value: NirExpr::ShaderFrameReady(_),
                ..
            } if stmt_name == "frame_ready"
        )
    }));
    assert!(main
        .body
        .iter()
        .any(|stmt| matches!(stmt, NirStmt::Expr(NirExpr::CpuPresentFrame(_)))));
}

#[test]
fn rejects_shader_packet_field_count_mismatch() {
    let surface_shader = standard_surface_shader_profile_module_with_packet_field_count(false, 4);
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
        ("surface_shader.ns", surface_shader.as_str()),
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

#[test]
fn infers_project_route_payload_type_from_entry_with_shared_cpu_helper() {
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

#[test]
fn validates_project_links_against_nir_with_shared_cpu_helper_indirection() {
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

#[test]
fn materializes_shader_and_data_type_contract_nodes_into_yir() {
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

#[test]
fn validates_reverse_shader_project_links_against_nir_via_data_bridge() {
    let fabric_plane = shader_contract_fabric_plane_module(true);
    let project = reverse_shader_data_bridge_project_with_link(
        reverse_shader_data_bridge_entry(),
        false,
        fabric_plane.as_str(),
    );

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    validate_project_links_against_nir(&project, &nir).unwrap();
}

#[test]
fn validates_reverse_shader_project_links_against_yir_via_data_bridge() {
    let fabric_plane = shader_fabric_plane_module(true);
    let project = reverse_shader_data_bridge_project_with_link(
        reverse_shader_data_bridge_entry(),
        true,
        fabric_plane.as_str(),
    );

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    let lowering_manifest =
        crate::registry::load_manifest_for_domain(std::path::Path::new("nustar-packages"), "cpu")
            .unwrap();
    let mut yir = crate::lowering::lower_nir_to_yir(&nir, &lowering_manifest, None).unwrap();
    apply_project_support_modules_to_yir(&project, &mut yir).unwrap();
    apply_project_links_to_yir(&project, &mut yir).unwrap();
    validate_project_links_against_yir(&project, &yir).unwrap();

    let resource_families = yir
        .resources
        .iter()
        .map(|resource| (resource.name.clone(), resource.kind.family().to_owned()))
        .collect::<BTreeMap<_, _>>();
    let node_resources = yir
        .nodes
        .iter()
        .map(|node| (node.name.clone(), node.resource.clone()))
        .collect::<BTreeMap<_, _>>();
    assert!(yir.edges.iter().any(|edge| {
        edge.kind == EdgeKind::CrossDomainExchange
            && node_resources
                .get(&edge.from)
                .and_then(|resource| resource_families.get(resource))
                .map(String::as_str)
                == Some("shader")
            && node_resources
                .get(&edge.to)
                .and_then(|resource| resource_families.get(resource))
                .map(String::as_str)
                == Some("data")
    }));
    assert!(yir.edges.iter().any(|edge| {
        edge.kind == EdgeKind::CrossDomainExchange
            && node_resources
                .get(&edge.from)
                .and_then(|resource| resource_families.get(resource))
                .map(String::as_str)
                == Some("data")
            && node_resources
                .get(&edge.to)
                .and_then(|resource| resource_families.get(resource))
                .map(String::as_str)
                == Some("cpu")
    }));
}

#[test]
fn compiles_reverse_shader_project_via_data_bridge() {
    let surface_shader = standard_surface_shader_profile_module(true);
    let fabric_plane = shader_fabric_plane_module(true);
    let root = write_temp_shader_data_project(
        "reverse_shader_via_data_bridge",
        reverse_shader_data_bridge_entry(),
        vec![
            ("surface_shader.ns", surface_shader.as_str()),
            ("fabric_plane.ns", fabric_plane.as_str()),
        ],
        &["shader.SurfaceShader -> cpu.Main via data.FabricPlane"],
    );
    let artifacts = crate::pipeline::compile_source_path(&root).unwrap();
    let _ = fs::remove_dir_all(&root);

    assert!(artifacts
        .loaded_nustar
        .iter()
        .any(|package| package == "official.shader"));
    assert!(artifacts
        .loaded_nustar
        .iter()
        .any(|package| package == "official.data"));
    assert!(artifacts
        .yir
        .nodes
        .iter()
        .any(|node| node.op.module == "shader" && node.op.instruction == "draw_instanced"));
}

#[test]
fn rejects_reverse_shader_project_via_data_bridge_when_shader_to_data_xfer_is_missing() {
    let surface_shader = standard_surface_shader_profile_module(true);
    let fabric_plane = shader_fabric_plane_module(false);
    let root = write_temp_shader_data_project(
        "reverse_shader_via_data_bridge_missing_xfer",
        reverse_shader_data_bridge_entry(),
        vec![
            ("surface_shader.ns", surface_shader.as_str()),
            ("fabric_plane.ns", fabric_plane.as_str()),
        ],
        &["shader.SurfaceShader -> cpu.Main via data.FabricPlane"],
    );
    let err = match crate::pipeline::compile_source_path(&root) {
        Ok(_) => panic!("expected reverse shader/data bridge compile to fail"),
        Err(err) => err,
    };
    let _ = fs::remove_dir_all(&root);
    assert!(err.contains("requires typed `Marker<Tag>` binding for marker `shader_to_cpu`"));
}

#[test]
fn rejects_reverse_shader_project_links_against_yir_when_shader_to_data_xfer_is_missing() {
    let fabric_plane = shader_fabric_plane_module(true);
    let project = reverse_shader_data_bridge_project_with_link(
        reverse_shader_render_bridge_entry(),
        true,
        fabric_plane.as_str(),
    );

    let mut yir = YirModule::new("0.1");
    apply_project_support_modules_to_yir(&project, &mut yir).unwrap();
    apply_project_links_to_yir(&project, &mut yir).unwrap();

    let resource_families = yir
        .resources
        .iter()
        .map(|resource| (resource.name.clone(), resource.kind.family().to_owned()))
        .collect::<BTreeMap<_, _>>();
    let node_resources = yir
        .nodes
        .iter()
        .map(|node| (node.name.clone(), node.resource.clone()))
        .collect::<BTreeMap<_, _>>();
    yir.edges.retain(|edge| {
        if edge.kind != EdgeKind::CrossDomainExchange {
            return true;
        }
        let from_family = node_resources
            .get(&edge.from)
            .and_then(|resource| resource_families.get(resource))
            .map(String::as_str);
        let to_family = node_resources
            .get(&edge.to)
            .and_then(|resource| resource_families.get(resource))
            .map(String::as_str);
        !(from_family == Some("shader") && to_family == Some("data"))
    });

    let err = validate_project_links_against_yir(&project, &yir).unwrap_err();
    assert!(err.contains("requires a `shader` -> `data` xfer segment"));
}

#[test]
fn project_support_modules_accept_darwin_x86_64_shader_and_data_abis() {
    let surface_shader = standard_surface_shader_profile_module(true);
    let fabric_plane = shader_fabric_plane_module(true);
    let project = test_support::loaded_project_fixture(
        "shader_data_darwin_x86_64",
        darwin_x86_64_shader_project_abis(),
        reverse_shader_data_bridge_entry(),
        vec![
            ("surface_shader.ns", surface_shader.as_str()),
            ("fabric_plane.ns", fabric_plane.as_str()),
        ],
    );

    let plan = build_project_compilation_plan(&project).unwrap();
    let checks = validate_project_abi_selections(&project, &plan.abi_resolution).unwrap();
    let mut yir = YirModule::new("0.1");
    apply_project_support_modules_to_yir(&project, &mut yir).unwrap();

    assert!(checks.iter().all(|check| check.ok));
    assert!(checks.iter().any(|check| {
        check.domain == "shader" && check.abi.as_deref() == Some("shader.metal.x86_64.msl2_4")
    }));
    assert!(checks.iter().any(|check| {
        check.domain == "data" && check.abi.as_deref() == Some("data.fabric.macos.x86_64.v1")
    }));
    assert!(yir.nodes.iter().any(|node| {
        node.name == "project_profile_shader_SurfaceShader_shader_target_config_auto"
            && node.op.module == "shader"
            && node.op.instruction == "target_config"
            && node.op.args
                == vec![
                    "x86_64".to_owned(),
                    "metal".to_owned(),
                    "1".to_owned(),
                    "argument-buffer,device.mac-discrete-or-integrated-gpu,metal,msl,resource-binding,shader-ir,vendor.apple"
                        .to_owned()
                ]
    }));
}
