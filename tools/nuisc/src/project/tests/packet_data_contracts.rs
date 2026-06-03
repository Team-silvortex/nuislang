use super::*;

#[test]
fn renders_project_packet_index_from_packet_annotated_structs() {
    let project = project_with_modules(vec![(
        "main.ns",
        r#"
        mod cpu Main {
          @packet
          struct Packet {
            @packet_field
            lhs: i64,
            rhs: bool,
          }

          fn main() -> i64 { return 1; }
        }
        "#,
    )]);
    let rendered = render_project_packet_index(&project);
    assert!(rendered.contains(
        "main.ns\tcpu.Main.Packet\tfields=2\tpacket_fields=1\tpacket_control_fields=0\tpacket_shape=scalar-only\tfield_kinds=scalar=2\tfield_roles=payload=2\tpacket_encode_shape=fixed-payload\tpayload_bytes=8\tpayload_layout=lhs:i64@0+8"
    ));
    assert!(rendered.contains(
        "\tindex=0\tlhs\ti64\tkind=scalar\trole=payload\tpacket_slot=payload\twire_kind=i64\tfixed_width=8\tpacket_field=true\tpacket_control_field=false"
    ));
    assert!(rendered.contains(
        "\tindex=1\trhs\tbool\tkind=scalar\trole=payload\tpacket_slot=none\twire_kind=none\tfixed_width=1\tpacket_field=false\tpacket_control_field=false"
    ));
}

#[test]
fn renders_packet_shape_metadata_for_container_bearing_packets() {
    let project = project_with_modules(vec![(
        "main.ns",
        r#"
        mod cpu Main {
          @packet
          struct Packet {
            @packet_field
            payload: Window<i64>,
            tag: Marker<Tag>,
          }

          fn main() -> i64 { return 1; }
        }
        "#,
    )]);
    let rendered = render_project_packet_index(&project);
    assert!(rendered.contains("packet_shape=carrier-mixed"));
    assert!(rendered.contains("field_kinds=container=1, marker=1"));
    assert!(rendered.contains("field_roles=control-plane=1, payload=1"));
    assert!(rendered.contains("packet_encode_shape=dynamic-payload"));
    assert!(rendered.contains("payload_bytes=dynamic"));
    assert!(rendered.contains(
        "\tindex=0\tpayload\tWindow<i64>\tkind=container\trole=payload\tpacket_slot=payload\twire_kind=container\tfixed_width=dynamic\tpacket_field=true\tpacket_control_field=false"
    ));
    assert!(rendered.contains(
        "\tindex=1\ttag\tMarker<Tag>\tkind=marker\trole=control-plane\tpacket_slot=none\twire_kind=none\tfixed_width=dynamic\tpacket_field=false\tpacket_control_field=false"
    ));
}

#[test]
fn accepts_project_packet_struct_with_explicit_control_field() {
    let project = project_with_modules(vec![(
        "main.ns",
        r#"
        mod cpu Main {
          @packet
          struct Packet {
            @packet_field
            payload: i64,
            @packet_control_field
            tag: Marker<Tag>,
          }

          fn main() -> i64 { return 1; }
        }
        "#,
    )]);
    validate_project_modules(&project.modules).unwrap();
    let rendered = render_project_packet_index(&project);
    assert!(rendered.contains("packet_control_fields=1"));
    assert!(rendered.contains("packet_encode_shape=fixed-payload+control"));
    assert!(rendered.contains("payload_layout=payload:i64@0+8"));
    assert!(rendered.contains(
        "\tindex=1\ttag\tMarker<Tag>\tkind=marker\trole=control-plane\tpacket_slot=control\twire_kind=control-plane\tfixed_width=dynamic\tpacket_field=false\tpacket_control_field=true"
    ));
}

#[test]
fn renders_packet_payload_layout_for_multiple_fixed_scalars() {
    let project = project_with_modules(vec![(
        "main.ns",
        r#"
        mod cpu Main {
          @packet
          struct Packet {
            @packet_field
            lhs: i32,
            @packet_field
            rhs: bool,
            @packet_field
            bias: i64,
          }

          fn main() -> i64 { return 1; }
        }
        "#,
    )]);
    let rendered = render_project_packet_index(&project);
    assert!(rendered.contains("packet_encode_shape=fixed-payload"));
    assert!(rendered.contains("payload_bytes=13"));
    assert!(rendered.contains("payload_layout=lhs:i32@0+4, rhs:bool@4+1, bias:i64@5+8"));
}

#[test]
fn rejects_project_packet_struct_with_marker_field() {
    let project = project_with_modules(vec![(
        "main.ns",
        r#"
        mod cpu Main {
          @packet
          struct Packet {
            @packet_field
            payload: Marker<Tag>,
          }

          fn main() -> i64 { return 1; }
        }
        "#,
    )]);
    let error = validate_project_modules(&project.modules).unwrap_err();
    assert!(error.contains(
        "annotation `@packet_field` currently only supports payload-role fields (kind=marker, role=control-plane)"
    ));
}

#[test]
fn rejects_project_packet_control_field_on_payload_role_field() {
    let project = project_with_modules(vec![(
        "main.ns",
        r#"
        mod cpu Main {
          @packet
          struct Packet {
            @packet_field
            payload: i64,
            @packet_control_field
            extra: bool,
          }

          fn main() -> i64 { return 1; }
        }
        "#,
    )]);
    let error = validate_project_modules(&project.modules).unwrap_err();
    assert!(error.contains(
        "annotation `@packet_control_field` currently only supports control-plane-role fields (kind=scalar, role=payload)"
    ));
}

#[test]
fn rejects_project_packet_struct_without_packet_fields() {
    let project = project_with_modules(vec![(
        "main.ns",
        r#"
        mod cpu Main {
          @packet
          struct Packet {
            payload: i64,
          }

          fn main() -> i64 { return 1; }
        }
        "#,
    )]);
    let error = validate_project_modules(&project.modules).unwrap_err();
    assert!(error.contains("requires at least one `@packet_field`"));
}

#[test]
fn accepts_typed_data_profile_tokens_for_project_link() {
    let project = project_with_modules(vec![
        (
            "main.ns",
            r#"
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
              }
            }
            "#,
        ),
    ]);

    validate_data_profile_token_types(
        &project,
        "cpu.Main",
        "shader.SurfaceShader",
        "data.FabricPlane",
    )
    .unwrap();
}

#[test]
fn validates_data_profile_token_types_through_local_type_aliases() {
    let project = project_with_modules(vec![
        (
            "main.ns",
            r#"
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
              type FabricBindingsTable = HandleTable<FabricBindings>;
              type CpuToShaderMarker = Marker<CpuToShader>;
              type ShaderToCpuMarker = Marker<ShaderToCpu>;
              type UplinkPipeMarker = Marker<UplinkPipe>;
              type DownlinkPipeMarker = Marker<DownlinkPipe>;
              type UplinkPipeClassMarker = Marker<UplinkPipeClass>;
              type DownlinkPipeClassMarker = Marker<DownlinkPipeClass>;
              type PayloadClassMarker = Marker<PayloadClassWindow>;
              type UplinkPayloadShapeMarker = Marker<PayloadShapeWindowSurfaceShaderPacket>;
              type DownlinkPayloadShapeMarker = Marker<PayloadShapeWindowFrame>;
              type UplinkWindowPolicyMarker = Marker<UplinkWindowPolicy>;
              type DownlinkWindowPolicyMarker = Marker<DownlinkWindowPolicy>;

              fn profile() {
                let profile_handles: FabricBindingsTable =
                  data_handle_table("host=cpu0", "render=shader0");
                let cpu_to_shader: CpuToShaderMarker = data_marker("cpu_to_shader");
                let shader_to_cpu: ShaderToCpuMarker = data_marker("shader_to_cpu");
                let uplink_pipe: UplinkPipeMarker = data_marker("uplink_pipe");
                let downlink_pipe: DownlinkPipeMarker = data_marker("downlink_pipe");
                let uplink_pipe_class: UplinkPipeClassMarker = data_marker("uplink_pipe_class");
                let downlink_pipe_class: DownlinkPipeClassMarker = data_marker("downlink_pipe_class");
                let uplink_payload_class: PayloadClassMarker = data_marker("uplink_payload_class");
                let downlink_payload_class: PayloadClassMarker = data_marker("downlink_payload_class");
                let uplink_payload_shape: UplinkPayloadShapeMarker = data_marker("uplink_payload_shape");
                let downlink_payload_shape: DownlinkPayloadShapeMarker = data_marker("downlink_payload_shape");
                let uplink_window_policy: UplinkWindowPolicyMarker = data_marker("uplink_window_policy");
                let downlink_window_policy: DownlinkWindowPolicyMarker = data_marker("downlink_window_policy");
              }
            }
            "#,
        ),
    ]);

    validate_data_profile_token_types(
        &project,
        "cpu.Main",
        "shader.SurfaceShader",
        "data.FabricPlane",
    )
    .unwrap();
}

#[test]
fn rejects_untyped_data_profile_marker_for_project_link() {
    let project = project_with_modules(vec![
        (
            "main.ns",
            r#"
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
                let profile_handles: HandleTable<FabricBindings> =
                  data_handle_table("host=cpu0", "render=shader0");
                let cpu_to_shader: Marker = data_marker("cpu_to_shader");
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

    let error = validate_data_profile_token_types(
        &project,
        "cpu.Main",
        "shader.SurfaceShader",
        "data.FabricPlane",
    )
    .unwrap_err();

    assert!(error.contains("typed form `Marker<...>`"));
}

#[test]
fn rejects_missing_window_policy_marker_for_windowed_bridge() {
    let project = project_with_modules(vec![
        (
            "main.ns",
            r#"
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
                let downlink_window_policy: Marker<DownlinkWindowPolicy> = data_marker("downlink_window_policy");
              }
            }
            "#,
        ),
    ]);

    let error = validate_data_profile_token_types(
        &project,
        "cpu.Main",
        "shader.SurfaceShader",
        "data.FabricPlane",
    )
    .unwrap_err();

    assert!(error.contains("uplink_window_policy"));
}
