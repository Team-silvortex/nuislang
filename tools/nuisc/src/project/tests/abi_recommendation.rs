use super::*;

#[test]
fn project_link_stage_contract_accepts_cpu_to_shader_over_data() {
    let contract = required_project_link_stage_contract(
        "cpu.Main",
        "shader.SurfaceShader",
        "data.FabricPlane",
    )
    .unwrap();

    assert_eq!(
        contract.uplink,
        NirResultStage::Data(NirDataFlowState::Windowed)
    );
    assert_eq!(
        contract.downlink,
        NirResultStage::Data(NirDataFlowState::Windowed)
    );
}

#[test]
fn project_link_stage_contract_accepts_cpu_to_network_over_data() {
    let contract =
        required_project_link_stage_contract("cpu.Main", "network.NetworkUnit", "data.FabricPlane")
            .unwrap();

    assert_eq!(
        contract.uplink,
        NirResultStage::Data(NirDataFlowState::Windowed)
    );
    assert_eq!(
        contract.downlink,
        NirResultStage::Data(NirDataFlowState::Windowed)
    );
}

#[test]
fn materializes_shader_and_network_resources_from_project_abi_targets() {
    let mut project = project_with_modules(vec![
        (
            "surface_shader.ns",
            r#"
            mod shader SurfaceShader {
              fn profile() {
                let profile_target: Target = shader_target("rgba8_unorm", 160, 120);
              }
            }
            "#,
        ),
        (
            "network_unit.ns",
            r#"
            mod network NetworkUnit {
              fn profile() {}
            }
            "#,
        ),
    ]);
    project.manifest.abi_requirements = vec![
        ProjectAbiRequirement {
            domain: "shader".to_owned(),
            abi: "shader.metal.msl2_4".to_owned(),
        },
        ProjectAbiRequirement {
            domain: "network".to_owned(),
            abi: "network.socket.macos.arm64.v1".to_owned(),
        },
    ];

    let mut yir = YirModule::new("0.1");
    apply_project_support_modules_to_yir(&project, &mut yir).unwrap();

    assert!(yir
        .resources
        .iter()
        .any(|resource| resource.name == "shader0" && resource.kind.raw == "shader.metal"));
    assert!(yir
        .resources
        .iter()
        .any(|resource| resource.name == "network0" && resource.kind.raw == "network.urlsession"));
    assert!(yir.nodes.iter().any(|node| {
        node.name == "project_profile_shader_SurfaceShader_shader_target_config_auto"
            && node.op.module == "shader"
            && node.op.instruction == "target_config"
            && node.op.args == vec!["arm64".to_owned(), "metal".to_owned(), "1".to_owned()]
    }));
    assert!(yir.nodes.iter().any(|node| {
        node.name == "project_profile_network_NetworkUnit_network_target_config_auto"
            && node.op.module == "network"
            && node.op.instruction == "target_config"
            && node.op.args == vec!["arm64".to_owned(), "urlsession".to_owned(), "1".to_owned()]
    }));
    assert!(yir.nodes.iter().any(|node| {
        node.name == "project_profile_shader_SurfaceShader_abi_selection_contract_type"
            && node.op.module == "cpu"
            && node.op.instruction == "text"
            && node.op.args
                == vec!["mode=symbol:explicit;abi=symbol:shader.metal.msl2_4;arch=symbol:arm64;runtime=symbol:metal;lane_width=i64:1".to_owned()]
    }));
    assert!(yir.nodes.iter().any(|node| {
        node.name == "project_profile_network_NetworkUnit_abi_selection_contract_type"
            && node.op.module == "cpu"
            && node.op.instruction == "text"
            && node.op.args
                == vec!["mode=symbol:explicit;abi=symbol:network.socket.macos.arm64.v1;arch=symbol:arm64;runtime=symbol:urlsession;lane_width=i64:1".to_owned()]
    }));
}

#[test]
fn materializes_auto_abi_selection_contract_for_recommended_shader_target() {
    let project = project_with_modules(vec![(
        "surface_shader.ns",
        r#"
        mod shader SurfaceShader {
          fn profile() {
            let profile_target: Target = shader_target("rgba8_unorm", 160, 120);
          }
        }
        "#,
    )]);

    let resolution = resolve_project_abi(&project).unwrap();
    let shader_abi = resolution
        .requirements
        .iter()
        .find(|item| item.domain == "shader")
        .unwrap()
        .abi
        .clone();
    let mut yir = YirModule::new("0.1");
    apply_project_support_modules_to_yir(&project, &mut yir).unwrap();

    let contract = yir
        .nodes
        .iter()
        .find(|node| {
            node.name == "project_profile_shader_SurfaceShader_abi_selection_contract_type"
        })
        .unwrap();
    let value = contract.op.args.first().unwrap();
    assert!(value.starts_with("mode=symbol:auto;"));
    assert!(value.contains(&format!("abi=symbol:{shader_abi};")));
}

#[test]
fn materializes_project_abi_summary_entries_for_cpu_and_data() {
    let mut project = project_with_modules(vec![
        (
            "main.ns",
            r#"
            use data FabricPlane;

            mod cpu Main {
              fn main() -> i64 {
                return 1;
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
                  data_handle_table("host=cpu0");
              }
            }
            "#,
        ),
    ]);
    project.manifest.abi_requirements = vec![
        ProjectAbiRequirement {
            domain: "cpu".to_owned(),
            abi: "cpu.arm64.apple_aapcs64".to_owned(),
        },
        ProjectAbiRequirement {
            domain: "data".to_owned(),
            abi: "data.fabric.host-match.v1".to_owned(),
        },
    ];

    let mut yir = YirModule::new("0.1");
    apply_project_support_modules_to_yir(&project, &mut yir).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.name == "project_abi_cpu_selection_entry"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.name == "project_abi_cpu_selection_summary_type"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.name == "project_abi_data_selection_entry"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.name == "project_abi_data_selection_summary_type"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.name == "project_abi_graph_summary_type"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.name == "project_abi_graph_summary_entry"));
    assert_eq!(
        yir.node_lanes
            .get("project_abi_cpu_selection_entry")
            .map(String::as_str),
        Some("contract")
    );
    assert_eq!(
        yir.node_lanes
            .get("project_abi_cpu_selection_summary_type")
            .map(String::as_str),
        Some("contract")
    );
    assert_eq!(
        yir.node_lanes
            .get("project_abi_graph_summary_entry")
            .map(String::as_str),
        Some("contract")
    );
    assert_eq!(
        yir.node_lanes
            .get("project_abi_graph_summary_type")
            .map(String::as_str),
        Some("contract")
    );
}

#[test]
fn renders_project_abi_index_with_graph_summary_and_domain_details() {
    let mut project = project_with_modules(vec![
        (
            "main.ns",
            r#"
            mod cpu Main {
              fn main() -> i64 {
                return 1;
              }
            }
            "#,
        ),
        (
            "network_unit.ns",
            r#"
            mod network NetworkUnit {
              fn profile() {}
            }
            "#,
        ),
    ]);
    project.manifest.abi_requirements = vec![
        ProjectAbiRequirement {
            domain: "cpu".to_owned(),
            abi: "cpu.arm64.apple_aapcs64".to_owned(),
        },
        ProjectAbiRequirement {
            domain: "network".to_owned(),
            abi: "network.socket.macos.arm64.v1".to_owned(),
        },
    ];

    let rendered = render_project_abi_index(&project).unwrap();
    assert!(rendered.contains("# mode=explicit"));
    assert!(rendered.contains("graph\tmode=explicit\tdomains=cpu,network"));
    assert!(rendered.contains("cpu_summary=present"));
    assert!(rendered.contains("network_target=present"));
    assert!(rendered.contains("domain\tcpu\tabi=cpu.arm64.apple_aapcs64"));
    assert!(rendered.contains("domain\tnetwork\tabi=network.socket.macos.arm64.v1"));
}

#[test]
fn project_abi_selection_views_expose_registered_targets() {
    let mut project = project_with_modules(vec![(
        "network_unit.ns",
        r#"
        mod network NetworkUnit {
          fn profile() {}
        }
        "#,
    )]);
    project.manifest.abi_requirements = vec![ProjectAbiRequirement {
        domain: "network".to_owned(),
        abi: "network.socket.macos.arm64.v1".to_owned(),
    }];

    let resolution = resolve_project_abi(&project).unwrap();
    let views = project_abi_selection_views(&resolution);
    assert_eq!(views.len(), 1);
    assert_eq!(views[0].domain, "network");
    assert_eq!(views[0].machine_arch.as_deref(), Some("arm64"));
    assert_eq!(views[0].machine_os.as_deref(), Some("darwin"));

    let lines = render_project_abi_selection_lines(&resolution);
    assert!(lines
        .iter()
        .any(|line| line == "abi: network=network.socket.macos.arm64.v1"));
    assert!(lines
        .iter()
        .any(|line| line == "  abi_target_machine: arm64-darwin"));
    assert!(
        project_abi_selection_view_json(&views[0]).contains("\"abi_target_host_adaptive\":false")
    );
}

#[test]
fn project_abi_selection_checks_report_registered_recommended_abis() {
    let project = project_with_modules(vec![(
        "network_unit.ns",
        r#"
        mod network NetworkUnit {
          fn profile() {}
        }
        "#,
    )]);

    let resolution = resolve_project_abi(&project).unwrap();
    let checks = validate_project_abi_selections(&project, &resolution).unwrap();
    assert_eq!(checks.len(), 1);
    assert!(checks[0].ok);
    assert_eq!(checks[0].source, "recommended");
    assert!(checks[0].abi_registered);
    assert_eq!(checks[0].issue_count(), 0);
    assert!(checks[0].summary_line().contains("source=recommended"));
    let lines = render_project_abi_selection_check_lines(&checks[0]);
    assert!(lines.iter().any(|line| line.contains("abi_registered=yes")));
    assert!(project_abi_selection_check_json(&checks[0]).contains("\"source\":\"recommended\""));
}

#[test]
fn project_abi_selection_checks_report_missing_explicit_domain_entries() {
    let mut project = project_with_modules(vec![
        (
            "main.ns",
            r#"
            mod cpu Main {
              fn main() -> i64 {
                return 1;
              }
            }
            "#,
        ),
        (
            "network_unit.ns",
            r#"
            mod network NetworkUnit {
              fn profile() {}
            }
            "#,
        ),
    ]);
    project.manifest.abi_requirements = vec![ProjectAbiRequirement {
        domain: "cpu".to_owned(),
        abi: "cpu.arm64.apple_aapcs64".to_owned(),
    }];
    let resolution = resolve_project_abi(&project).unwrap();
    let checks = validate_project_abi_selections(&project, &resolution).unwrap();
    let network = checks
        .iter()
        .find(|check| check.domain == "network")
        .unwrap();
    assert!(!network.ok);
    assert!(network
        .issues
        .iter()
        .any(|issue| issue.kind == ProjectAbiIssueKind::MissingExplicitDomainAbi));
    assert!(network
        .issues
        .iter()
        .any(|issue| issue.kind.code() == "ABI001"));
    assert!(network
        .summary_line()
        .contains("ABI001 missing_explicit_domain_abi"));
}

#[test]
fn project_lowering_selections_expose_registered_targets_and_selected_backend() {
    let mut project = project_with_modules(vec![(
        "main.ns",
        r#"
        mod cpu Main {
          fn main() -> i64 {
            return 1;
          }
        }
        "#,
    )]);
    project.manifest.abi_requirements = vec![ProjectAbiRequirement {
        domain: "cpu".to_owned(),
        abi: "cpu.arm64.apple_aapcs64".to_owned(),
    }];

    let resolution = resolve_project_abi(&project).unwrap();
    let lowering = validate_project_lowering_selections(&resolution);
    assert_eq!(lowering.len(), 1);
    assert!(lowering[0].ok);
    assert_eq!(lowering[0].issue_count(), 0);
    assert_eq!(
        lowering[0].selected_lowering_target.as_deref(),
        Some("llvm")
    );
    assert!(lowering[0]
        .registered_lowering_targets
        .iter()
        .any(|target| target == "llvm"));
    let lines = render_project_lowering_selection_lines(&lowering[0]);
    assert!(lines.iter().any(|line| line.contains("selected=llvm")));
    assert!(lines.iter().any(|line| line.contains("issues=0")));
    assert!(lowering[0].summary_line().contains("selected=llvm"));
    assert!(project_lowering_selection_json(&lowering[0])
        .contains("\"selected_lowering_target\":\"llvm\""));
}

#[test]
fn project_lowering_selections_resolve_shader_kernel_and_network_targets() {
    let mut project = project_with_modules(vec![
        (
            "surface_shader.ns",
            r#"
            mod shader SurfaceShader {
              fn profile() {
                let profile_target: Target = shader_target("rgba8_unorm", 160, 120);
              }
            }
            "#,
        ),
        (
            "kernel_unit.ns",
            r#"
            mod kernel KernelUnit {
              fn profile() {
                let batch_lanes: i64 = 4;
                let profile_entry: Unit = kernel_target_config("apple_ane", "coreml", batch_lanes);
              }
            }
            "#,
        ),
        (
            "network_unit.ns",
            r#"
            mod network NetworkUnit {
              fn profile() {}
            }
            "#,
        ),
    ]);
    project.manifest.abi_requirements = vec![
        ProjectAbiRequirement {
            domain: "shader".to_owned(),
            abi: "shader.metal.msl2_4".to_owned(),
        },
        ProjectAbiRequirement {
            domain: "kernel".to_owned(),
            abi: "kernel.apple_ane.coreml.v1".to_owned(),
        },
        ProjectAbiRequirement {
            domain: "network".to_owned(),
            abi: "network.socket.macos.arm64.v1".to_owned(),
        },
    ];

    let resolution = resolve_project_abi(&project).unwrap();
    let lowering = validate_project_lowering_selections(&resolution);

    let shader = lowering
        .iter()
        .find(|item| item.domain == "shader")
        .unwrap();
    let kernel = lowering
        .iter()
        .find(|item| item.domain == "kernel")
        .unwrap();
    let network = lowering
        .iter()
        .find(|item| item.domain == "network")
        .unwrap();

    assert_eq!(shader.selected_lowering_target.as_deref(), Some("metal"));
    assert!(shader
        .registered_lowering_targets
        .iter()
        .any(|target| target == "metal"));
    assert_eq!(kernel.selected_lowering_target.as_deref(), Some("coreml"));
    assert!(kernel
        .registered_lowering_targets
        .iter()
        .any(|target| target == "coreml"));
    assert_eq!(
        network.selected_lowering_target.as_deref(),
        Some("urlsession")
    );
    assert!(network
        .registered_lowering_targets
        .iter()
        .any(|target| target == "urlsession"));
}

#[test]
fn validates_network_target_projection_against_selected_abi() {
    let mut project = project_with_modules(vec![(
        "network_unit.ns",
        r#"
        mod network NetworkUnit {
          fn profile() {
            const bind_core: i64 = 0;
            const endpoint_kind: i64 = 1;
            const local_port: i64 = 8080;
            const remote_port: i64 = 443;
            const connect_timeout_ms: i64 = 1000;
            const retry_budget: i64 = 3;
            const stream_window: i64 = 8;
            const recv_window: i64 = 8;
            const send_window: i64 = 8;
          }
        }
        "#,
    )]);
    project.manifest.abi_requirements = vec![ProjectAbiRequirement {
        domain: "network".to_owned(),
        abi: "network.socket.macos.arm64.v1".to_owned(),
    }];

    let mut yir = YirModule::new("0.1");
    apply_project_support_modules_to_yir(&project, &mut yir).unwrap();
    validate_network_target_projection(&project, &yir, "NetworkUnit").unwrap();
}

#[test]
fn rejects_network_target_projection_that_disagrees_with_selected_abi() {
    let mut project = project_with_modules(vec![(
        "network_unit.ns",
        r#"
        mod network NetworkUnit {
          fn profile() {
            const bind_core: i64 = 0;
            const endpoint_kind: i64 = 1;
            const local_port: i64 = 8080;
            const remote_port: i64 = 443;
            const connect_timeout_ms: i64 = 1000;
            const retry_budget: i64 = 3;
            const stream_window: i64 = 8;
            const recv_window: i64 = 8;
            const send_window: i64 = 8;
          }
        }
        "#,
    )]);
    project.manifest.abi_requirements = vec![ProjectAbiRequirement {
        domain: "network".to_owned(),
        abi: "network.socket.macos.arm64.v1".to_owned(),
    }];

    let mut yir = YirModule::new("0.1");
    apply_project_support_modules_to_yir(&project, &mut yir).unwrap();
    let resource = yir
        .resources
        .iter_mut()
        .find(|resource| resource.name == "network0")
        .unwrap();
    resource.kind = yir_core::ResourceKind::parse("network.winsock");

    let error = validate_network_target_projection(&project, &yir, "NetworkUnit").unwrap_err();
    assert!(error.contains("network.urlsession"));
    assert!(error.contains("network.socket.macos.arm64.v1"));
}

#[test]
fn validates_shader_target_projection_against_selected_abi() {
    let mut project = project_with_modules(vec![(
        "surface_shader.ns",
        r#"
        mod shader SurfaceShader {
          fn profile() {
            const vertex_count: i64 = 3;
            const instance_count: i64 = 1;
            const packet_field_count: i64 = 3;
            const pass_kind: i64 = 1;
            const packet_color_slot: i64 = 0;
            const packet_speed_slot: i64 = 1;
            const packet_radius_slot: i64 = 2;
            let profile_target: Target = shader_target("rgba8_unorm", 160, 120);
            let profile_view: Viewport = shader_viewport(160, 120);
            let profile_pipe: Pipeline = shader_pipeline("lit_sphere", "triangle_strip");
            let profile_wgsl: ShaderModule = shader_inline_wgsl("lit_sphere", "struct VsOut { @builtin(position) pos: vec4<f32>, @location(0) color: vec4<f32>, }; @vertex fn vs_main(@builtin(vertex_index) idx: u32) -> VsOut { var out: VsOut; out.pos = vec4<f32>(0.0, 0.0, 0.0, 1.0); out.color = vec4<f32>(1.0, 0.0, 0.0, 1.0); return out; } @fragment fn fs_main(in: VsOut) -> @location(0) vec4<f32> { return in.color; }");
          }
        }
        "#,
    )]);
    project.manifest.abi_requirements = vec![ProjectAbiRequirement {
        domain: "shader".to_owned(),
        abi: "shader.metal.msl2_4".to_owned(),
    }];

    let mut yir = YirModule::new("0.1");
    apply_project_support_modules_to_yir(&project, &mut yir).unwrap();
    validate_shader_target_projection(&project, &yir, "SurfaceShader").unwrap();
}

#[test]
fn rejects_shader_target_projection_that_disagrees_with_selected_abi() {
    let mut project = project_with_modules(vec![(
        "surface_shader.ns",
        r#"
        mod shader SurfaceShader {
          fn profile() {
            const vertex_count: i64 = 3;
            const instance_count: i64 = 1;
            const packet_field_count: i64 = 3;
            const pass_kind: i64 = 1;
            const packet_color_slot: i64 = 0;
            const packet_speed_slot: i64 = 1;
            const packet_radius_slot: i64 = 2;
            let profile_target: Target = shader_target("rgba8_unorm", 160, 120);
            let profile_view: Viewport = shader_viewport(160, 120);
            let profile_pipe: Pipeline = shader_pipeline("lit_sphere", "triangle_strip");
            let profile_wgsl: ShaderModule = shader_inline_wgsl("lit_sphere", "struct VsOut { @builtin(position) pos: vec4<f32>, @location(0) color: vec4<f32>, }; @vertex fn vs_main(@builtin(vertex_index) idx: u32) -> VsOut { var out: VsOut; out.pos = vec4<f32>(0.0, 0.0, 0.0, 1.0); out.color = vec4<f32>(1.0, 0.0, 0.0, 1.0); return out; } @fragment fn fs_main(in: VsOut) -> @location(0) vec4<f32> { return in.color; }");
          }
        }
        "#,
    )]);
    project.manifest.abi_requirements = vec![ProjectAbiRequirement {
        domain: "shader".to_owned(),
        abi: "shader.metal.msl2_4".to_owned(),
    }];

    let mut yir = YirModule::new("0.1");
    apply_project_support_modules_to_yir(&project, &mut yir).unwrap();
    let resource = yir
        .resources
        .iter_mut()
        .find(|resource| resource.name == "shader0")
        .unwrap();
    resource.kind = yir_core::ResourceKind::parse("shader.directx");

    let error = validate_shader_target_projection(&project, &yir, "SurfaceShader").unwrap_err();
    assert!(error.contains("shader.metal"));
    assert!(error.contains("shader.metal.msl2_4"));
}

#[test]
fn rejects_shader_target_projection_without_inline_wgsl_for_metal_abi() {
    let mut project = project_with_modules(vec![(
        "surface_shader.ns",
        r#"
        mod shader SurfaceShader {
          fn profile() {
            const vertex_count: i64 = 3;
            const instance_count: i64 = 1;
            const packet_field_count: i64 = 3;
            const pass_kind: i64 = 1;
            const packet_color_slot: i64 = 0;
            const packet_speed_slot: i64 = 1;
            const packet_radius_slot: i64 = 2;
            let profile_target: Target = shader_target("rgba8_unorm", 160, 120);
            let profile_view: Viewport = shader_viewport(160, 120);
            let profile_pipe: Pipeline = shader_pipeline("lit_sphere", "triangle_strip");
          }
        }
        "#,
    )]);
    project.manifest.abi_requirements = vec![ProjectAbiRequirement {
        domain: "shader".to_owned(),
        abi: "shader.metal.msl2_4".to_owned(),
    }];

    let mut yir = YirModule::new("0.1");
    apply_project_support_modules_to_yir(&project, &mut yir).unwrap();
    let error = validate_shader_target_projection(&project, &yir, "SurfaceShader").unwrap_err();
    assert!(error.contains("requires shader_inline_wgsl"));
    assert!(error.contains("shader.metal.msl2_4"));
}

#[test]
fn rejects_shader_inline_wgsl_for_cpu_fallback_abi() {
    let mut project = project_with_modules(vec![(
        "surface_shader.ns",
        r#"
        mod shader SurfaceShader {
          fn profile() {
            const vertex_count: i64 = 3;
            const instance_count: i64 = 1;
            const packet_field_count: i64 = 3;
            const pass_kind: i64 = 1;
            const packet_color_slot: i64 = 0;
            const packet_speed_slot: i64 = 1;
            const packet_radius_slot: i64 = 2;
            let profile_target: Target = shader_target("rgba8_unorm", 160, 120);
            let profile_view: Viewport = shader_viewport(160, 120);
            let profile_pipe: Pipeline = shader_pipeline("lit_sphere", "triangle_strip");
            let profile_wgsl: ShaderModule = shader_inline_wgsl("lit_sphere", "struct VsOut { @builtin(position) pos: vec4<f32>, @location(0) color: vec4<f32>, }; @vertex fn vs_main(@builtin(vertex_index) idx: u32) -> VsOut { var out: VsOut; out.pos = vec4<f32>(0.0, 0.0, 0.0, 1.0); out.color = vec4<f32>(1.0, 0.0, 0.0, 1.0); return out; } @fragment fn fs_main(in: VsOut) -> @location(0) vec4<f32> { return in.color; }");
          }
        }
        "#,
    )]);
    project.manifest.abi_requirements = vec![ProjectAbiRequirement {
        domain: "shader".to_owned(),
        abi: "shader.render.cpu-fallback.v1".to_owned(),
    }];

    let mut yir = YirModule::new("0.1");
    apply_project_support_modules_to_yir(&project, &mut yir).unwrap();
    let error = validate_shader_target_projection(&project, &yir, "SurfaceShader").unwrap_err();
    assert!(error.contains("does not declare inline WGSL capability"));
    assert!(error.contains("shader.render.cpu-fallback.v1"));
}

#[test]
fn project_link_stage_contract_rejects_shader_to_kernel_for_now() {
    let error = required_project_link_stage_contract(
        "shader.SurfaceShader",
        "kernel.KernelUnit",
        "data.FabricPlane",
    )
    .unwrap_err();

    assert!(error.contains("cpu<->cpu"));
    assert!(error.contains("cpu<->shader"));
    assert!(error.contains("cpu<->kernel"));
    assert!(error.contains("cpu<->network"));
}

#[test]
fn rejects_missing_bridge_payload_contract_for_windowed_link() {
    let project = project_with_modules(vec![
        (
            "main.ns",
            r#"
            mod cpu Main {
              fn main() {
                return;
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

    let error = build_project_link_bridge_contract(
        &project,
        "cpu.Main",
        "shader.SurfaceShader",
        "data.FabricPlane",
    )
    .unwrap_err();

    assert!(error.contains("payload contract"));
    assert!(error.contains("data.FabricPlane"));
}

#[test]
fn recommend_cpu_abi_profile_prefers_registered_host_target() {
    let host_arch = match std::env::consts::ARCH {
        "aarch64" => "arm64",
        other => other,
    };
    let host_os = match std::env::consts::OS {
        "macos" => "darwin",
        other => other,
    };
    let host_object = host_object_format();
    let host_calling = host_calling_abi(host_arch, host_os);
    let host_clang = match (host_arch, host_os) {
        ("arm64", "darwin") => "aarch64-apple-darwin",
        ("arm64", "linux") => "aarch64-unknown-linux-gnu",
        ("x86_64", "linux") => "x86_64-unknown-linux-gnu",
        ("x86_64", "windows") => "x86_64-pc-windows-msvc",
        _ => "x86_64-unknown-linux-gnu",
    };
    let alt = if host_arch == "arm64" && host_os == "darwin" {
        (
            "cpu.x86_64.win64",
            "x86_64",
            "windows",
            "coff",
            "win64",
            "x86_64-pc-windows-msvc",
        )
    } else {
        (
            "cpu.arm64.apple_aapcs64",
            "arm64",
            "darwin",
            "mach-o",
            "aapcs64-darwin",
            "aarch64-apple-darwin",
        )
    };
    let manifest = crate::registry::NustarPackageManifest {
        manifest_schema: "nustar-manifest-v1".to_owned(),
        package_id: "official.cpu".to_owned(),
        domain_family: "cpu".to_owned(),
        frontend: "nustar-cpu".to_owned(),
        entry_crate: "crates/yir-domain-cpu".to_owned(),
        ast_entry: "cpu.ast.bootstrap.v1".to_owned(),
        nir_entry: "cpu.nir.bootstrap.v1".to_owned(),
        yir_lowering_entry: "cpu.yir.lowering.v1".to_owned(),
        part_verify_entry: "cpu.verify.partial.v1".to_owned(),
        ast_surface: vec!["cpu.mod-ast.v1".to_owned()],
        nir_surface: vec!["nir.cpu.surface.v1".to_owned()],
        yir_lowering: vec!["yir.cpu.lowering.v1".to_owned()],
        part_verify: vec!["verify.cpu.contract.v1".to_owned()],
        binary_extension: "nustar".to_owned(),
        package_layout: "single-envelope".to_owned(),
        machine_abi_policy: "exact-match".to_owned(),
        abi_profiles: vec!["cpu.host.match".to_owned(), alt.0.to_owned()],
        abi_capabilities: vec![
            "cpu.host.match:op:cpu.*".to_owned(),
            format!("{}:op:cpu.*", alt.0),
        ],
        abi_targets: vec![
            format!(
                "cpu.host.match:arch={}|os={}|object={}|calling={}|clang={}",
                host_arch, host_os, host_object, host_calling, host_clang
            ),
            format!(
                "{}:arch={}|os={}|object={}|calling={}|clang={}",
                alt.0, alt.1, alt.2, alt.3, alt.4, alt.5
            ),
        ],
        implementation_kinds: vec!["native-stub".to_owned()],
        loader_entry: "nustar.bootstrap.v1".to_owned(),
        loader_abi: "nustar-loader-v1".to_owned(),
        host_ffi_surface: Vec::new(),
        host_ffi_abis: Vec::new(),
        host_ffi_bridge: "none".to_owned(),
        bridge_lane_policy: None,
        bridge_surface: None,
        bridge_emission_kind: None,
        bridge_entry: None,
        bridge_kind: None,
        bridge_scheduler_binding: None,
        backend_stub_kind: None,
        backend_submission_mode: None,
        backend_wake_policy: None,
        backend_transport_model: None,
        backend_request_shape: None,
        backend_response_shape: None,
        backend_dispatch_shape: None,
        backend_memory_binding: None,
        backend_resource_binding: None,
        backend_completion_model: None,
        phase_bind: None,
        phase_submit: None,
        phase_wait: None,
        phase_finalize: None,
        host_bridge_host_ffi_surface: None,
        host_bridge_handle_family: None,
        host_bridge_phase_order: None,
        host_bridge_phase_bind_inputs: None,
        host_bridge_phase_bind_outputs: None,
        host_bridge_phase_submit_inputs: None,
        host_bridge_phase_submit_outputs: None,
        host_bridge_phase_wait_inputs: None,
        host_bridge_phase_wait_outputs: None,
        host_bridge_phase_finalize_inputs: None,
        host_bridge_phase_finalize_outputs: None,
        host_bridge_phase_bind_wake: None,
        host_bridge_phase_submit_wake: None,
        host_bridge_phase_wait_wake: None,
        host_bridge_phase_finalize_wake: None,
        host_bridge_plan_begin: None,
        host_bridge_plan_end: None,
        support_surface: Vec::new(),
        support_profile_slots: Vec::new(),
        default_lanes: Vec::new(),
        clock_domain_id: "cpu.clock.host.v1".to_owned(),
        clock_kind: "host-monotonic".to_owned(),
        clock_epoch_kind: "host-epoch".to_owned(),
        clock_resolution: "cpu.tick_i64".to_owned(),
        clock_bridge_default: "global->monotonic:bridge".to_owned(),
        profiles: vec!["aot".to_owned()],
        resource_families: vec!["cpu".to_owned()],
        unit_types: vec!["Main".to_owned()],
        lowering_targets: vec!["llvm".to_owned()],
        ops: vec!["cpu.const".to_owned()],
    };

    let selected = recommend_abi_profile_for_host(&manifest).unwrap();
    assert_eq!(selected, "cpu.host.match");
}

#[test]
fn recommend_data_abi_profile_prefers_registered_host_target() {
    let host_arch = match std::env::consts::ARCH {
        "aarch64" => "arm64",
        other => other,
    };
    let host_os = match std::env::consts::OS {
        "macos" => "darwin",
        other => other,
    };
    let host_object = host_object_format();
    let host_calling = host_calling_abi(host_arch, host_os);
    let host_clang = match (host_arch, host_os) {
        ("arm64", "darwin") => "aarch64-apple-darwin",
        ("arm64", "linux") => "aarch64-unknown-linux-gnu",
        ("x86_64", "linux") => "x86_64-unknown-linux-gnu",
        ("x86_64", "windows") => "x86_64-pc-windows-msvc",
        _ => "x86_64-unknown-linux-gnu",
    };
    let manifest = crate::registry::NustarPackageManifest {
        manifest_schema: "nustar-manifest-v1".to_owned(),
        package_id: "official.data".to_owned(),
        domain_family: "data".to_owned(),
        frontend: "nustar-data".to_owned(),
        entry_crate: "crates/yir-core".to_owned(),
        ast_entry: "data.ast.bootstrap.v1".to_owned(),
        nir_entry: "data.nir.bootstrap.v1".to_owned(),
        yir_lowering_entry: "data.yir.lowering.v1".to_owned(),
        part_verify_entry: "data.verify.partial.v1".to_owned(),
        ast_surface: vec!["data.mod-ast.v1".to_owned()],
        nir_surface: vec!["nir.data.surface.v1".to_owned()],
        yir_lowering: vec!["yir.data.lowering.v1".to_owned()],
        part_verify: vec!["verify.data.contract.v1".to_owned()],
        binary_extension: "nustar".to_owned(),
        package_layout: "single-envelope".to_owned(),
        machine_abi_policy: "exact-match".to_owned(),
        abi_profiles: vec![
            "data.fabric.v1".to_owned(),
            "data.fabric.host-match.v1".to_owned(),
            "data.fabric.alt.v1".to_owned(),
        ],
        abi_capabilities: vec![
            "data.fabric.v1:surface:data.profile.*|op:data.*".to_owned(),
            "data.fabric.host-match.v1:surface:data.profile.*|op:data.*".to_owned(),
            "data.fabric.alt.v1:surface:data.profile.*|op:data.*".to_owned(),
        ],
        abi_targets: vec![
            "data.fabric.v1:arch=host|os=host|object=host|calling=host|clang=host".to_owned(),
            format!(
                "data.fabric.host-match.v1:arch={}|os={}|object={}|calling={}|clang={}",
                host_arch, host_os, host_object, host_calling, host_clang
            ),
            "data.fabric.alt.v1:arch=x86_64|os=windows|object=coff|calling=win64|clang=x86_64-pc-windows-msvc".to_owned(),
        ],
        implementation_kinds: vec!["native-stub".to_owned()],
        loader_entry: "nustar.bootstrap.v1".to_owned(),
        loader_abi: "nustar-loader-v1".to_owned(),
        host_ffi_surface: Vec::new(),
        host_ffi_abis: Vec::new(),
        host_ffi_bridge: "none".to_owned(),
        bridge_lane_policy: None,
        bridge_surface: None,
        bridge_emission_kind: None,
        bridge_entry: None,
        bridge_kind: None,
        bridge_scheduler_binding: None,
        backend_stub_kind: None,
        backend_submission_mode: None,
        backend_wake_policy: None,
        backend_transport_model: None,
        backend_request_shape: None,
        backend_response_shape: None,
        backend_dispatch_shape: None,
        backend_memory_binding: None,
        backend_resource_binding: None,
        backend_completion_model: None,
        phase_bind: None,
        phase_submit: None,
        phase_wait: None,
        phase_finalize: None,
        host_bridge_host_ffi_surface: None,
        host_bridge_handle_family: None,
        host_bridge_phase_order: None,
        host_bridge_phase_bind_inputs: None,
        host_bridge_phase_bind_outputs: None,
        host_bridge_phase_submit_inputs: None,
        host_bridge_phase_submit_outputs: None,
        host_bridge_phase_wait_inputs: None,
        host_bridge_phase_wait_outputs: None,
        host_bridge_phase_finalize_inputs: None,
        host_bridge_phase_finalize_outputs: None,
        host_bridge_phase_bind_wake: None,
        host_bridge_phase_submit_wake: None,
        host_bridge_phase_wait_wake: None,
        host_bridge_phase_finalize_wake: None,
        host_bridge_plan_begin: None,
        host_bridge_plan_end: None,
        support_surface: Vec::new(),
        support_profile_slots: Vec::new(),
        default_lanes: Vec::new(),
        clock_domain_id: "data.clock.fabric.v1".to_owned(),
        clock_kind: "fabric-monotonic".to_owned(),
        clock_epoch_kind: "fabric-epoch".to_owned(),
        clock_resolution: "fabric-window-step".to_owned(),
        clock_bridge_default: "global->fabric:bridge".to_owned(),
        profiles: vec!["aot".to_owned()],
        resource_families: vec!["data".to_owned()],
        unit_types: vec!["FabricPlane".to_owned()],
        lowering_targets: vec!["fabric-abi".to_owned()],
        ops: vec!["data.move".to_owned()],
    };

    let selected = recommend_abi_profile_for_host(&manifest).unwrap();
    assert_eq!(selected, "data.fabric.host-match.v1");
}

#[test]
fn recommend_kernel_abi_profile_prefers_registered_host_target() {
    let host_arch = match std::env::consts::ARCH {
        "aarch64" => "arm64",
        other => other,
    };
    let host_os = match std::env::consts::OS {
        "macos" => "darwin",
        other => other,
    };
    let host_object = host_object_format();
    let host_calling = host_calling_abi(host_arch, host_os);
    let host_clang = match (host_arch, host_os) {
        ("arm64", "darwin") => "aarch64-apple-darwin",
        ("arm64", "linux") => "aarch64-unknown-linux-gnu",
        ("x86_64", "linux") => "x86_64-unknown-linux-gnu",
        ("x86_64", "windows") => "x86_64-pc-windows-msvc",
        _ => "x86_64-unknown-linux-gnu",
    };
    let manifest = crate::registry::NustarPackageManifest {
        manifest_schema: "nustar-manifest-v1".to_owned(),
        package_id: "official.kernel".to_owned(),
        domain_family: "kernel".to_owned(),
        frontend: "nustar-kernel".to_owned(),
        entry_crate: "crates/yir-domain-kernel".to_owned(),
        ast_entry: "kernel.ast.bootstrap.v1".to_owned(),
        nir_entry: "kernel.nir.bootstrap.v1".to_owned(),
        yir_lowering_entry: "kernel.yir.lowering.v1".to_owned(),
        part_verify_entry: "kernel.verify.partial.v1".to_owned(),
        ast_surface: vec!["kernel.mod-ast.v1".to_owned()],
        nir_surface: vec!["nir.kernel.surface.v1".to_owned()],
        yir_lowering: vec!["yir.kernel.lowering.v1".to_owned()],
        part_verify: vec!["verify.kernel.contract.v1".to_owned()],
        binary_extension: "nustar".to_owned(),
        package_layout: "single-envelope".to_owned(),
        machine_abi_policy: "exact-match".to_owned(),
        abi_profiles: vec![
            "kernel.cpu-fallback.v1".to_owned(),
            "kernel.host-match.v1".to_owned(),
            "kernel.alt.v1".to_owned(),
        ],
        abi_capabilities: vec![
            "kernel.cpu-fallback.v1:surface:kernel.profile.*|op:kernel.*".to_owned(),
            "kernel.host-match.v1:surface:kernel.profile.*|op:kernel.*".to_owned(),
            "kernel.alt.v1:surface:kernel.profile.*|op:kernel.*".to_owned(),
        ],
        abi_targets: vec![
            "kernel.cpu-fallback.v1:arch=host|os=host|object=host|calling=host|clang=host"
                .to_owned(),
            format!(
                "kernel.host-match.v1:arch={}|os={}|object={}|calling={}|clang={}|backend=coreml",
                host_arch, host_os, host_object, host_calling, host_clang
            ),
            "kernel.alt.v1:arch=x86_64|os=windows|object=coff|calling=win64|clang=x86_64-pc-windows-msvc|backend=cpu-fallback".to_owned(),
        ],
        implementation_kinds: vec!["native-stub".to_owned()],
        loader_entry: "nustar.bootstrap.v1".to_owned(),
        loader_abi: "nustar-loader-v1".to_owned(),
        host_ffi_surface: Vec::new(),
        host_ffi_abis: Vec::new(),
        host_ffi_bridge: "none".to_owned(),
        bridge_lane_policy: None,
        bridge_surface: None,
        bridge_emission_kind: None,
        bridge_entry: None,
        bridge_kind: None,
        bridge_scheduler_binding: None,
        backend_stub_kind: None,
        backend_submission_mode: None,
        backend_wake_policy: None,
        backend_transport_model: None,
        backend_request_shape: None,
        backend_response_shape: None,
        backend_dispatch_shape: None,
        backend_memory_binding: None,
        backend_resource_binding: None,
        backend_completion_model: None,
        phase_bind: None,
        phase_submit: None,
        phase_wait: None,
        phase_finalize: None,
        host_bridge_host_ffi_surface: None,
        host_bridge_handle_family: None,
        host_bridge_phase_order: None,
        host_bridge_phase_bind_inputs: None,
        host_bridge_phase_bind_outputs: None,
        host_bridge_phase_submit_inputs: None,
        host_bridge_phase_submit_outputs: None,
        host_bridge_phase_wait_inputs: None,
        host_bridge_phase_wait_outputs: None,
        host_bridge_phase_finalize_inputs: None,
        host_bridge_phase_finalize_outputs: None,
        host_bridge_phase_bind_wake: None,
        host_bridge_phase_submit_wake: None,
        host_bridge_phase_wait_wake: None,
        host_bridge_phase_finalize_wake: None,
        host_bridge_plan_begin: None,
        host_bridge_plan_end: None,
        support_surface: Vec::new(),
        support_profile_slots: Vec::new(),
        default_lanes: Vec::new(),
        clock_domain_id: "kernel.clock.dispatch.v1".to_owned(),
        clock_kind: "dispatch-monotonic".to_owned(),
        clock_epoch_kind: "dispatch-epoch".to_owned(),
        clock_resolution: "kernel-queue-step".to_owned(),
        clock_bridge_default: "global->dispatch:bridge".to_owned(),
        profiles: vec!["aot".to_owned()],
        resource_families: vec!["kernel".to_owned()],
        unit_types: vec!["KernelUnit".to_owned()],
        lowering_targets: vec!["coreml".to_owned()],
        ops: vec!["kernel.tensor".to_owned()],
    };

    let selected = recommend_abi_profile_for_host(&manifest).unwrap();
    assert_eq!(selected, "kernel.host-match.v1");
}

#[test]
fn recommend_shader_abi_profile_prefers_registered_host_target() {
    let host_arch = match std::env::consts::ARCH {
        "aarch64" => "arm64",
        other => other,
    };
    let host_os = match std::env::consts::OS {
        "macos" => "darwin",
        other => other,
    };
    let host_object = host_object_format();
    let host_calling = host_calling_abi(host_arch, host_os);
    let host_clang = match (host_arch, host_os) {
        ("arm64", "darwin") => "aarch64-apple-darwin",
        ("arm64", "linux") => "aarch64-unknown-linux-gnu",
        ("x86_64", "linux") => "x86_64-unknown-linux-gnu",
        ("x86_64", "windows") => "x86_64-pc-windows-msvc",
        _ => "x86_64-unknown-linux-gnu",
    };
    let manifest = crate::registry::NustarPackageManifest {
        manifest_schema: "nustar-manifest-v1".to_owned(),
        package_id: "official.shader".to_owned(),
        domain_family: "shader".to_owned(),
        frontend: "nustar-shader".to_owned(),
        entry_crate: "crates/yir-domain-shader".to_owned(),
        ast_entry: "shader.ast.bootstrap.v1".to_owned(),
        nir_entry: "shader.nir.bootstrap.v1".to_owned(),
        yir_lowering_entry: "shader.yir.lowering.v1".to_owned(),
        part_verify_entry: "shader.verify.partial.v1".to_owned(),
        ast_surface: vec!["shader.mod-ast.v1".to_owned()],
        nir_surface: vec!["nir.shader.surface.v1".to_owned()],
        yir_lowering: vec!["yir.shader.lowering.v1".to_owned()],
        part_verify: vec!["verify.shader.contract.v1".to_owned()],
        binary_extension: "nustar".to_owned(),
        package_layout: "single-envelope".to_owned(),
        machine_abi_policy: "exact-match".to_owned(),
        abi_profiles: vec![
            "shader.render.cpu-fallback.v1".to_owned(),
            "shader.host-match.v1".to_owned(),
            "shader.alt.v1".to_owned(),
        ],
        abi_capabilities: vec![
            "shader.render.cpu-fallback.v1:surface:shader.profile.*|op:shader.*".to_owned(),
            "shader.host-match.v1:surface:shader.profile.*|op:shader.*".to_owned(),
            "shader.alt.v1:surface:shader.profile.*|op:shader.*".to_owned(),
        ],
        abi_targets: vec![
            "shader.render.cpu-fallback.v1:arch=host|os=host|object=host|calling=host|clang=host"
                .to_owned(),
            format!(
                "shader.host-match.v1:arch={}|os={}|object={}|calling={}|clang={}|backend=metal",
                host_arch, host_os, host_object, host_calling, host_clang
            ),
            "shader.alt.v1:arch=x86_64|os=windows|object=coff|calling=win64|clang=x86_64-pc-windows-msvc|backend=directx".to_owned(),
        ],
        implementation_kinds: vec!["native-stub".to_owned()],
        loader_entry: "nustar.bootstrap.v1".to_owned(),
        loader_abi: "nustar-loader-v1".to_owned(),
        host_ffi_surface: Vec::new(),
        host_ffi_abis: Vec::new(),
        host_ffi_bridge: "none".to_owned(),
        bridge_lane_policy: None,
        bridge_surface: None,
        bridge_emission_kind: None,
        bridge_entry: None,
        bridge_kind: None,
        bridge_scheduler_binding: None,
        backend_stub_kind: None,
        backend_submission_mode: None,
        backend_wake_policy: None,
        backend_transport_model: None,
        backend_request_shape: None,
        backend_response_shape: None,
        backend_dispatch_shape: None,
        backend_memory_binding: None,
        backend_resource_binding: None,
        backend_completion_model: None,
        phase_bind: None,
        phase_submit: None,
        phase_wait: None,
        phase_finalize: None,
        host_bridge_host_ffi_surface: None,
        host_bridge_handle_family: None,
        host_bridge_phase_order: None,
        host_bridge_phase_bind_inputs: None,
        host_bridge_phase_bind_outputs: None,
        host_bridge_phase_submit_inputs: None,
        host_bridge_phase_submit_outputs: None,
        host_bridge_phase_wait_inputs: None,
        host_bridge_phase_wait_outputs: None,
        host_bridge_phase_finalize_inputs: None,
        host_bridge_phase_finalize_outputs: None,
        host_bridge_phase_bind_wake: None,
        host_bridge_phase_submit_wake: None,
        host_bridge_phase_wait_wake: None,
        host_bridge_phase_finalize_wake: None,
        host_bridge_plan_begin: None,
        host_bridge_plan_end: None,
        support_surface: Vec::new(),
        support_profile_slots: Vec::new(),
        default_lanes: Vec::new(),
        clock_domain_id: "shader.clock.frame.v1".to_owned(),
        clock_kind: "frame-monotonic".to_owned(),
        clock_epoch_kind: "frame-epoch".to_owned(),
        clock_resolution: "render-pass-step".to_owned(),
        clock_bridge_default: "global->frame:bridge".to_owned(),
        profiles: vec!["aot".to_owned()],
        resource_families: vec!["shader".to_owned()],
        unit_types: vec!["SurfaceShader".to_owned()],
        lowering_targets: vec!["metal".to_owned()],
        ops: vec!["shader.target".to_owned()],
    };

    let selected = recommend_abi_profile_for_host(&manifest).unwrap();
    assert_eq!(selected, "shader.host-match.v1");
}

#[test]
fn parses_project_tests_array() {
    let manifest = parse_project_manifest(
        r#"
name = "sample"
entry = "main.ns"
tests = ["tests/smoke.ns", "tests/data.ns"]
"#,
        Path::new("nuis.toml"),
    )
    .unwrap();
    assert_eq!(
        manifest.tests,
        vec!["tests/smoke.ns".to_owned(), "tests/data.ns".to_owned()]
    );
}
