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
            && node.op.args
                == vec![
                    "arm64".to_owned(),
                    "metal".to_owned(),
                    "1".to_owned(),
                    "argument-buffer,device.apple-silicon-gpu,metal,msl,resource-binding,shader-ir,vendor.apple"
                        .to_owned()
                ]
    }));
    assert!(yir.nodes.iter().any(|node| {
        node.name == "project_profile_network_NetworkUnit_network_target_config_auto"
            && node.op.module == "network"
            && node.op.instruction == "target_config"
            && node.op.args
                == vec![
                    "arm64".to_owned(),
                    "urlsession".to_owned(),
                    "1".to_owned(),
                    "async-bridge,darwin-network-stack,device.socket-io,socket-transport,tls-session,urlsession,vendor.apple"
                        .to_owned()
                ]
    }));
    assert!(yir.nodes.iter().any(|node| {
        node.name == "project_profile_shader_SurfaceShader_abi_selection_contract_type"
            && node.op.module == "cpu"
            && node.op.instruction == "text"
            && node.op.args
                == vec!["mode=symbol:explicit;abi=symbol:shader.metal.msl2_4;arch=symbol:arm64;runtime=symbol:metal;vendor=symbol:apple;device=symbol:apple-silicon-gpu;lane_width=i64:1;backend_features=list:argument-buffer,device.apple-silicon-gpu,metal,msl,resource-binding,shader-ir,vendor.apple".to_owned()]
    }));
    assert!(yir.nodes.iter().any(|node| {
        node.name == "project_profile_network_NetworkUnit_abi_selection_contract_type"
            && node.op.module == "cpu"
            && node.op.instruction == "text"
            && node.op.args
                == vec!["mode=symbol:explicit;abi=symbol:network.socket.macos.arm64.v1;arch=symbol:arm64;runtime=symbol:urlsession;vendor=symbol:apple;device=symbol:socket-io;lane_width=i64:1;backend_features=list:async-bridge,darwin-network-stack,device.socket-io,socket-transport,tls-session,urlsession,vendor.apple".to_owned()]
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
    assert_eq!(views[0].vendor.as_deref(), Some("apple"));
    assert_eq!(views[0].device_class.as_deref(), Some("socket-io"));

    let lines = render_project_abi_selection_lines(&resolution);
    assert!(lines
        .iter()
        .any(|line| line == "abi: network=network.socket.macos.arm64.v1"));
    assert!(lines
        .iter()
        .any(|line| line == "  abi_target_machine: arm64-darwin"));
    assert!(lines
        .iter()
        .any(|line| line == "  abi_target_vendor: apple"));
    assert!(lines
        .iter()
        .any(|line| line == "  abi_target_device: socket-io"));
    let mut written = String::new();
    write_project_abi_selection_lines(&mut written, &resolution).unwrap();
    assert_eq!(written.lines().collect::<Vec<_>>(), lines);
    assert!(
        project_abi_selection_view_json(&views[0]).contains("\"abi_target_host_adaptive\":false")
    );
    assert!(project_abi_selection_view_json(&views[0]).contains("\"abi_target_vendor\":\"apple\""));
    assert!(
        project_abi_selection_view_json(&views[0]).contains("\"abi_target_device\":\"socket-io\"")
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
    let mut written = String::new();
    write_project_abi_selection_check_lines(&mut written, &checks[0]).unwrap();
    assert_eq!(written.lines().collect::<Vec<_>>(), lines);
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
    let mut written = String::new();
    write_project_lowering_selection_lines(&mut written, &lowering[0]).unwrap();
    assert_eq!(written.lines().collect::<Vec<_>>(), lines);
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

    assert_eq!(
        shader.selected_lowering_target.as_deref(),
        Some("metal.apple-silicon-gpu")
    );
    assert!(shader
        .registered_lowering_targets
        .iter()
        .any(|target| target == "metal"));
    assert_eq!(
        kernel.selected_lowering_target.as_deref(),
        Some("coreml.apple-ane")
    );
    assert!(kernel
        .registered_lowering_targets
        .iter()
        .any(|target| target == "coreml"));
    assert_eq!(
        network.selected_lowering_target.as_deref(),
        Some("urlsession.socket-io")
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
            let profile_wgsl: ShaderModule = shader_inline_wgsl("lit_sphere", wgsl {
              struct VsOut {
                @builtin(position) pos: vec4<f32>,
                @location(0) color: vec4<f32>,
              };

              @vertex
              fn vs_main(@builtin(vertex_index) idx: u32) -> VsOut {
                var out: VsOut;
                out.pos = vec4<f32>(0.0, 0.0, 0.0, 1.0);
                out.color = vec4<f32>(1.0, 0.0, 0.0, 1.0);
                return out;
              }

              @fragment
              fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
                return in.color;
              }
            });
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
fn normalizes_structured_wgsl_stage_blocks_before_project_yir_emission() {
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
            let profile_wgsl: ShaderModule = shader_inline_wgsl("lit_sphere", wgsl {
              struct VsOut {
                @builtin(position) pos: vec4<f32>,
                @location(0) color: vec4<f32>,
              };

              stage vertex {
                fn vs_main(@builtin(vertex_index) idx: u32) -> VsOut {
                  var out: VsOut;
                  out.pos = vec4<f32>(0.0, 0.0, 0.0, 1.0);
                  out.color = vec4<f32>(1.0, 0.0, 0.0, 1.0);
                  return out;
                }
              }

              stage fragment {
                fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
                  return in.color;
                }
              }
            });
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

    let inline_wgsl = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "shader" && node.op.instruction == "inline_wgsl")
        .expect("inline_wgsl node");
    let source = inline_wgsl.op.args.get(1).expect("inline_wgsl source");
    assert!(source.contains("@vertex"), "{source}");
    assert!(source.contains("@fragment"), "{source}");
    assert!(!source.contains("stage vertex"), "{source}");
    assert!(!source.contains("stage fragment"), "{source}");
}

#[test]
fn normalizes_compute_stage_workgroup_metadata_before_project_yir_emission() {
    let mut project = project_with_modules(vec![(
        "surface_shader.ns",
        r#"
        mod shader SurfaceShader {
          fn profile() {
            let profile_target: Target = shader_target("rgba8_unorm", 160, 120);
            let profile_view: Viewport = shader_viewport(160, 120);
            let profile_pipe: Pipeline = shader_pipeline("lit_sphere", "triangle_strip");
            let profile_wgsl: ShaderModule = shader_inline_wgsl("lit_sphere", wgsl {
              stage compute(workgroup_size(8, 4, 1)) {
                fn cs_main() {
                }
              }
            });
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

    let inline_wgsl = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "shader" && node.op.instruction == "inline_wgsl")
        .expect("inline_wgsl node");
    let source = inline_wgsl.op.args.get(1).expect("inline_wgsl source");
    assert!(source.contains("@compute"), "{source}");
    assert!(source.contains("@workgroup_size(8, 4, 1)"), "{source}");
    assert!(!source.contains("stage compute"), "{source}");
}

#[test]
fn normalizes_fragment_stage_metadata_before_project_yir_emission() {
    let mut project = project_with_modules(vec![(
        "surface_shader.ns",
        r#"
        mod shader SurfaceShader {
          fn profile() {
            let profile_target: Target = shader_target("rgba8_unorm", 160, 120);
            let profile_view: Viewport = shader_viewport(160, 120);
            let profile_pipe: Pipeline = shader_pipeline("lit_sphere", "triangle_strip");
            let profile_wgsl: ShaderModule = shader_inline_wgsl("lit_sphere", wgsl {
              stage fragment(early_depth_test) {
                fn fs_main() -> @location(0) vec4<f32> {
                  return vec4<f32>(1.0, 1.0, 1.0, 1.0);
                }
              }
            });
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

    let inline_wgsl = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "shader" && node.op.instruction == "inline_wgsl")
        .expect("inline_wgsl node");
    let source = inline_wgsl.op.args.get(1).expect("inline_wgsl source");
    assert!(source.contains("@fragment"), "{source}");
    assert!(source.contains("@early_depth_test"), "{source}");
    assert!(!source.contains("stage fragment"), "{source}");
}

#[test]
fn normalizes_binding_declarations_before_project_yir_emission() {
    let mut project = project_with_modules(vec![(
        "surface_shader.ns",
        r#"
        mod shader SurfaceShader {
          fn profile() {
            let profile_target: Target = shader_target("rgba8_unorm", 160, 120);
            let profile_view: Viewport = shader_viewport(160, 120);
            let profile_pipe: Pipeline = shader_pipeline("lit_sphere", "triangle_strip");
            let profile_wgsl: ShaderModule = shader_inline_wgsl("lit_sphere", wgsl {
              struct Globals {
                exposure: f32,
              };

              binding(0, 0) var color_sampler: sampler;
              binding(0, 1) var color_tex: texture_2d<f32>;
              binding(0, 2) var<uniform> globals: Globals;

              stage fragment {
                fn fs_main() -> @location(0) vec4<f32> {
                  return vec4<f32>(globals.exposure, 1.0, 1.0, 1.0);
                }
              }
            });
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

    let inline_wgsl = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "shader" && node.op.instruction == "inline_wgsl")
        .expect("inline_wgsl node");
    let source = inline_wgsl.op.args.get(1).expect("inline_wgsl source");
    assert!(source.contains("@group(0)"), "{source}");
    assert!(source.contains("@binding(0)"), "{source}");
    assert!(source.contains("@binding(1)"), "{source}");
    assert!(source.contains("@binding(2)"), "{source}");
    assert!(
        source.contains("var<uniform> globals: Globals;"),
        "{source}"
    );
    assert!(!source.contains("binding(0, 0)"), "{source}");
}

#[test]
fn normalizes_bare_builtin_and_location_attributes_before_project_yir_emission() {
    let mut project = project_with_modules(vec![(
        "surface_shader.ns",
        r#"
        mod shader SurfaceShader {
          fn profile() {
            let profile_target: Target = shader_target("rgba8_unorm", 160, 120);
            let profile_view: Viewport = shader_viewport(160, 120);
            let profile_pipe: Pipeline = shader_pipeline("lit_sphere", "triangle_strip");
            let profile_wgsl: ShaderModule = shader_inline_wgsl("lit_sphere", wgsl {
              struct VsOut {
                builtin(position) pos: vec4<f32>,
                location(0) uv: vec2<f32>,
              };

              stage vertex {
                fn vs_main(builtin(vertex_index) vid: u32) -> VsOut {
                  var out: VsOut;
                  out.pos = vec4<f32>(0.0, 0.0, 0.0, 1.0);
                  out.uv = vec2<f32>(f32(vid), 0.0);
                  return out;
                }
              }

              stage fragment {
                fn fs_main(location(0) uv: vec2<f32>) -> location(0) vec4<f32> {
                  return vec4<f32>(uv.x, uv.y, 1.0, 1.0);
                }
              }
            });
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

    let inline_wgsl = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "shader" && node.op.instruction == "inline_wgsl")
        .expect("inline_wgsl node");
    let source = inline_wgsl.op.args.get(1).expect("inline_wgsl source");
    assert!(
        source.contains("@builtin(position) pos: vec4<f32>,"),
        "{source}"
    );
    assert!(source.contains("@location(0) uv: vec2<f32>,"), "{source}");
    assert!(
        source.contains("fn vs_main(@builtin(vertex_index) vid: u32)"),
        "{source}"
    );
    assert!(
        source.contains("fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32>"),
        "{source}"
    );
    assert!(
        !source.contains("\n                builtin(position)"),
        "{source}"
    );
}

#[test]
fn normalizes_bare_interpolate_and_invariant_attributes_before_project_yir_emission() {
    let mut project = project_with_modules(vec![(
        "surface_shader.ns",
        r#"
        mod shader SurfaceShader {
          fn profile() {
            let profile_target: Target = shader_target("rgba8_unorm", 160, 120);
            let profile_view: Viewport = shader_viewport(160, 120);
            let profile_pipe: Pipeline = shader_pipeline("lit_sphere", "triangle_strip");
            let profile_wgsl: ShaderModule = shader_inline_wgsl("lit_sphere", wgsl {
              struct VsOut {
                invariant builtin(position) pos: vec4<f32>,
                interpolate(flat) location(0) uv: vec2<f32>,
              };

              stage fragment {
                fn fs_main(interpolate(flat) location(0) uv: vec2<f32>) -> location(0) vec4<f32> {
                  return vec4<f32>(uv.x, uv.y, 1.0, 1.0);
                }
              }
            });
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

    let inline_wgsl = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "shader" && node.op.instruction == "inline_wgsl")
        .expect("inline_wgsl node");
    let source = inline_wgsl.op.args.get(1).expect("inline_wgsl source");
    assert!(
        source.contains("@invariant @builtin(position) pos: vec4<f32>,"),
        "{source}"
    );
    assert!(
        source.contains("@interpolate(flat) @location(0) uv: vec2<f32>,"),
        "{source}"
    );
    assert!(
        source.contains(
            "fn fs_main(@interpolate(flat) @location(0) uv: vec2<f32>) -> @location(0) vec4<f32>"
        ),
        "{source}"
    );
    assert!(
        !source.contains("\n                invariant builtin(position)"),
        "{source}"
    );
}
