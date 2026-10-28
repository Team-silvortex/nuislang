use super::*;

pub(super) fn darwin_x86_64_shader_project_abis() -> Vec<ProjectAbiRequirement> {
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

pub(super) fn compiled_domain_project(path: &str) -> crate::pipeline::PipelineArtifacts {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path);
    crate::pipeline::compile_source_path(&root).unwrap()
}

pub(super) fn inline_wgsl_source_named<'a>(
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

pub(super) fn write_temp_shader_data_project(
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

pub(super) fn standard_surface_shader_profile_module(include_data_use: bool) -> String {
    standard_surface_shader_profile_module_with_packet_field_count(include_data_use, 3)
}

pub(super) fn standard_surface_shader_profile_module_with_packet_field_count(
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

pub(super) fn standard_nova_surface_shader_profile_module() -> String {
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

pub(super) fn reverse_shader_data_bridge_entry() -> &'static str {
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

pub(super) fn shader_fabric_plane_module(include_shader_to_cpu: bool) -> String {
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

pub(super) fn reverse_shader_data_bridge_project(
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

pub(super) fn reverse_shader_data_bridge_project_with_link(
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

pub(super) fn reverse_shader_render_bridge_entry() -> &'static str {
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

pub(super) fn shader_task_async_shapes_project(
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

pub(super) fn shader_contract_fabric_plane_module(include_shader_to_cpu: bool) -> String {
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
