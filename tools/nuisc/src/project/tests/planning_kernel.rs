use super::*;
use std::fs;

#[test]
fn accepts_local_auxiliary_cpu_units_in_projects() {
    let project = project_with_modules(vec![
        (
            "main.ns",
            r#"
            use cpu TaskHelpers;

            mod cpu Main {
              fn main() -> i64 {
                return TaskHelpers.pick(7);
              }
            }
            "#,
        ),
        (
            "task_helpers.ns",
            r#"
            mod cpu TaskHelpers {
              fn pick(seed: i64) -> i64 {
                return seed + 1;
              }
            }
            "#,
        ),
    ]);

    validate_project_modules(&project.modules).unwrap();
    validate_project_unit_bindings(&project.modules).unwrap();
    validate_project_uses(&project.modules, &project.resolved_galaxies).unwrap();
}

#[test]
fn network_profile_slot_targets_use_stable_names() {
    assert_eq!(
        resolve_project_profile_target_name("network", "NetworkUnit", "bind_core"),
        "project_profile_network_NetworkUnit_bind_core"
    );
    assert_eq!(
        resolve_project_profile_target_name("network", "NetworkUnit", "endpoint_kind"),
        "project_profile_network_NetworkUnit_endpoint_kind"
    );
    assert_eq!(
        resolve_project_profile_target_name("network", "NetworkUnit", "transport_family"),
        "project_profile_network_NetworkUnit_transport_family"
    );
    assert_eq!(
        resolve_project_profile_target_name("network", "NetworkUnit", "connect_timeout_ms"),
        "project_profile_network_NetworkUnit_connect_timeout_ms"
    );
    assert_eq!(
        resolve_project_profile_target_name("network", "NetworkUnit", "retry_budget"),
        "project_profile_network_NetworkUnit_retry_budget"
    );
    assert_eq!(
        resolve_project_profile_target_name("network", "NetworkUnit", "stream_window"),
        "project_profile_network_NetworkUnit_stream_window"
    );
    assert_eq!(
        resolve_project_profile_target_name("network", "NetworkUnit", "recv_window"),
        "project_profile_network_NetworkUnit_recv_window"
    );
    assert_eq!(
        resolve_project_profile_target_name("network", "NetworkUnit", "send_window"),
        "project_profile_network_NetworkUnit_send_window"
    );
    assert_eq!(
        resolve_project_profile_target_name("network", "NetworkUnit", "protocol_kind"),
        "project_profile_network_NetworkUnit_protocol_kind"
    );
    assert_eq!(
        resolve_project_profile_target_name("network", "NetworkUnit", "protocol_version"),
        "project_profile_network_NetworkUnit_protocol_version"
    );
    assert_eq!(
        resolve_project_profile_target_name("network", "NetworkUnit", "protocol_header_bytes"),
        "project_profile_network_NetworkUnit_protocol_header_bytes"
    );
}

#[test]
fn organizes_project_modules_domains_and_links() {
    let mut project = project_with_modules(vec![
        (
            "main.ns",
            r#"
            mod cpu Main {
              fn main() -> i64 { return 1; }
            }
            "#,
        ),
        (
            "network_unit.ns",
            r#"
            mod network NetworkUnit {
              fn ping() -> i64 { return 1; }
            }
            "#,
        ),
    ]);
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "network.NetworkUnit".to_owned(),
        via: Some("data.FabricPlane".to_owned()),
    }];

    let organization = organize_project(&project);
    assert_eq!(organization.entry, "main.ns");
    assert_eq!(
        organization.domains,
        vec!["cpu".to_owned(), "data".to_owned(), "network".to_owned()]
    );
    assert_eq!(organization.modules.len(), 2);
    assert!(organization.modules.iter().any(|module| module.is_entry));
    assert_eq!(organization.links.len(), 1);
    assert_eq!(organization.links[0].from, "cpu.Main");
    assert_eq!(organization.links[0].to, "network.NetworkUnit");
    assert_eq!(
        organization.links[0].via.as_deref(),
        Some("data.FabricPlane")
    );
}

#[test]
fn organizes_project_exchange_routes() {
    let mut project = project_with_modules(vec![(
        "main.ns",
        r#"
        mod cpu Main {
          fn main() -> i64 { return 1; }
        }
        "#,
    )]);
    project.manifest.links = vec![
        ProjectLink {
            from: "cpu.Main".to_owned(),
            to: "network.NetworkUnit".to_owned(),
            via: Some("data.FabricPlane".to_owned()),
        },
        ProjectLink {
            from: "cpu.Main".to_owned(),
            to: "cpu.Observer".to_owned(),
            via: None,
        },
    ];

    let exchanges = organize_project_exchanges(&project);
    assert_eq!(exchanges.routes.len(), 2);
    assert_eq!(exchanges.routes[0].mode, "bridged");
    assert_eq!(exchanges.routes[0].class, "bridged");
    assert_eq!(
        exchanges.routes[0].domains,
        vec!["cpu".to_owned(), "network".to_owned(), "data".to_owned()]
    );
    assert_eq!(exchanges.routes[1].mode, "direct");
    assert_eq!(exchanges.routes[1].class, "local");
    assert_eq!(exchanges.routes[1].domains, vec!["cpu".to_owned()]);
}

#[test]
fn builds_project_compilation_plan_from_shared_organization_layers() {
    let mut project = project_with_modules(vec![(
        "main.ns",
        r#"
        mod cpu Main {
          fn main() -> i64 { return 1; }
        }
        "#,
    )]);
    project.manifest.name = "demo".to_owned();
    project.manifest.links = vec![ProjectLink {
        from: "cpu.Main".to_owned(),
        to: "network.NetworkUnit".to_owned(),
        via: Some("data.FabricPlane".to_owned()),
    }];

    let plan = build_project_compilation_plan(&project).unwrap();
    assert_eq!(plan.project_name, "demo");
    assert_eq!(plan.entry, "main.ns");
    assert_eq!(plan.exchanges.routes.len(), 1);
    assert_eq!(describe_project_exchange_route_classes(&plan), "bridged=1");
    assert!(plan.dependencies.is_empty());
    assert_eq!(describe_project_dependency_categories(&plan), "<none>");
    assert_eq!(plan.synthetic_input.kind, "project-name-entry");
    assert_eq!(plan.effective_input_path, PathBuf::from("./demo.ns"));
    assert!(plan.output_intents.iter().any(|item| {
        item.category == "project-metadata"
            && item.kind == "project-plan-index"
            && item.path_hint == "nuis.project.plan.txt"
    }));
    assert!(plan.output_intents.iter().any(|item| {
        item.category == "project-metadata"
            && item.kind == "project-packet-index"
            && item.path_hint == "nuis.project.packet.txt"
    }));
    assert_eq!(
        describe_project_output_intent_categories(&plan),
        "core-artifacts=1, project-metadata=8, verification-inputs=1"
    );
    assert_eq!(
        describe_project_compilation_plan(&plan),
        "entry=main.ns domains=cpu, data, network exchanges=1 abi_mode=auto-recommended"
    );
}

#[test]
fn renders_project_compilation_plan_index() {
    let mut project = project_with_modules(vec![(
        "main.ns",
        r#"
        mod cpu Main {
          fn main() -> i64 { return 1; }
        }
        "#,
    )]);
    project.manifest.name = "demo".to_owned();
    let plan = build_project_compilation_plan(&project).unwrap();
    let rendered = render_project_compilation_plan_index(&plan);
    assert!(rendered.contains("project demo"));
    assert!(rendered.contains("entry main.ns"));
    assert!(rendered.contains("domains cpu"));
    assert!(rendered.contains("exchanges 0"));
    assert!(rendered.contains("abi_mode auto-recommended"));
    assert!(rendered.contains("dependencies <none>"));
    assert!(rendered.contains("synthetic_input_kind project-name-entry"));
    assert!(
        rendered.contains("output_intents core-artifacts:build-manifest=nuis.build.manifest.toml")
    );
    assert!(rendered.contains("effective_input ./demo.ns"));
    assert!(rendered
        .contains("summary entry=main.ns domains=cpu exchanges=0 abi_mode=auto-recommended"));
}

#[test]
fn writes_project_compilation_plan_index_matching_rendered_output() {
    let mut project = project_with_modules(vec![(
        "main.ns",
        r#"
        mod cpu Main {
          fn main() -> i64 { return 1; }
        }
        "#,
    )]);
    project.manifest.name = "demo".to_owned();
    let plan = build_project_compilation_plan(&project).unwrap();

    let rendered = render_project_compilation_plan_index(&plan);
    let mut written = String::new();
    write_project_compilation_plan_index(&mut written, &plan).unwrap();

    assert_eq!(written, rendered);
}

#[test]
fn writer_renderers_match_project_index_strings() {
    let project = test_support::loaded_project_fixture(
        "render_writer_match",
        vec![ProjectAbiRequirement {
            domain: "cpu".to_owned(),
            abi: "cpu.arm64.apple_aapcs64".to_owned(),
        }],
        r#"
        use cpu Helpers;

        mod cpu Main {
          extern "c" fn host_clock_now() -> i64;

          fn main() -> i64 {
            return Helpers.tick() + host_clock_now();
          }
        }
        "#,
        vec![(
            "helpers.ns",
            r#"
            mod cpu Helpers {
              fn tick() -> i64 {
                return 1;
              }
            }
            "#,
        )],
    );

    let rendered_org = render_project_organization_index(&project);
    let mut written_org = String::new();
    write_project_organization_index(&mut written_org, &project).unwrap();
    assert_eq!(written_org, rendered_org);

    let rendered_exchange = render_project_exchange_index(&project);
    let mut written_exchange = String::new();
    write_project_exchange_index(&mut written_exchange, &project).unwrap();
    assert_eq!(written_exchange, rendered_exchange);

    let rendered_imports = render_project_import_index(&project);
    let mut written_imports = String::new();
    write_project_import_index(&mut written_imports, &project).unwrap();
    assert_eq!(written_imports, rendered_imports);

    let rendered_host_ffi = render_project_host_ffi_index(&project);
    let mut written_host_ffi = String::new();
    write_project_host_ffi_index(&mut written_host_ffi, &project).unwrap();
    assert_eq!(written_host_ffi, rendered_host_ffi);

    let rendered_abi = render_project_abi_index(&project).unwrap();
    let mut written_abi = String::new();
    write_project_abi_index(&mut written_abi, &project).unwrap();
    assert_eq!(written_abi, rendered_abi);

    let rendered_packet = crate::project::packet::render_project_packet_index(&project);
    let mut written_packet = String::new();
    crate::project::packet::write_project_packet_index(&mut written_packet, &project).unwrap();
    assert_eq!(written_packet, rendered_packet);

    let rendered_galaxy =
        crate::stdlib_registry::render_resolved_galaxy_index(&project.resolved_galaxies);
    let mut written_galaxy = String::new();
    crate::stdlib_registry::write_resolved_galaxy_index(
        &mut written_galaxy,
        &project.resolved_galaxies,
    )
    .unwrap();
    assert_eq!(written_galaxy, rendered_galaxy);
}

#[test]
fn write_project_metadata_preserves_modules_and_links_index_output() {
    let root = test_support::write_temp_project_fixture(
        "metadata_writer_match",
        r#"
name = "metadata_writer_match"
entry = "main.ns"
modules = ["main.ns", "helpers.ns"]
links = [
  "cpu.Main -> cpu.Helpers",
]
"#
        .trim_start(),
        r#"
        use cpu Helpers;

        mod cpu Main {
          fn main() -> i64 {
            return Helpers.tick();
          }
        }
        "#,
        vec![(
            "helpers.ns",
            r#"
            mod cpu Helpers {
              fn tick() -> i64 {
                return 1;
              }
            }
            "#,
        )],
    );
    let project = load_project(root.as_path()).unwrap();
    let plan = build_project_compilation_plan(&project).unwrap();
    let output_dir = root.join("build");

    let metadata = write_project_metadata(&output_dir, &project, &plan).unwrap();
    let modules_index = fs::read_to_string(&metadata.modules_index_path).unwrap();
    let docs_index = fs::read_to_string(&metadata.docs_index_path).unwrap();
    let links_index = fs::read_to_string(&metadata.links_index_path).unwrap();

    assert!(modules_index.contains("main.ns\tmod cpu Main\tentry=true\tsource_kind=project-local"));
    assert!(modules_index
        .contains("helpers.ns\tmod cpu Helpers\tentry=false\tsource_kind=project-local"));
    assert!(docs_index.contains("module\tcpu.Main\titems=0\tsource_kind=project-local"));
    assert!(docs_index.contains("module\tcpu.Helpers\titems=0\tsource_kind=project-local"));
    assert_eq!(links_index, "cpu.Main\tcpu.Helpers\t<direct>\n");
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn categorizes_project_compilation_dependencies() {
    let mut project = project_with_modules(vec![(
        "main.ns",
        r#"
        mod cpu Main {
          fn main() -> i64 { return 1; }
        }
        "#,
    )]);
    project.manifest.galaxy_dependencies = vec![ProjectGalaxyDependency {
        name: "demo.dep".to_owned(),
        version: "1.2.3".to_owned(),
    }];

    let plan = build_project_compilation_plan(&project).unwrap();
    assert_eq!(plan.dependencies.len(), 1);
    assert_eq!(plan.dependencies[0].category, "package-registry");
    assert_eq!(plan.dependencies[0].source, "galaxy-manifest");
    assert_eq!(
        describe_project_dependency_categories(&plan),
        "package-registry=1"
    );

    let rendered = render_project_compilation_plan_index(&plan);
    assert!(rendered.contains("dependencies package-registry:demo.dep=1.2.3 (galaxy-manifest)"));
}

#[test]
fn validates_kernel_profile_slot_contract() {
    let project = project_with_modules(vec![(
        "kernel_unit.ns",
        r#"
        mod kernel KernelUnit {
          fn profile() {
            const bind_core: i64 = 2;
            const queue_depth: i64 = 8;
            const batch_lanes: i64 = 16;
            let profile_entry: Unit =
              kernel_target_config("apple_ane", "coreml", batch_lanes);
          }
        }
        "#,
    )]);

    validate_kernel_profile_slot_contract(&project, "KernelUnit").unwrap();
}

#[test]
fn rejects_kernel_profile_without_batch_lanes_wiring() {
    let project = project_with_modules(vec![(
        "kernel_unit.ns",
        r#"
        mod kernel KernelUnit {
          fn profile() {
            const bind_core: i64 = 2;
            const queue_depth: i64 = 8;
            const batch_lanes: i64 = 16;
            let profile_entry: Unit =
              kernel_target_config("apple_ane", "coreml", queue_depth);
          }
        }
        "#,
    )]);

    let error = validate_kernel_profile_slot_contract(&project, "KernelUnit").unwrap_err();
    assert!(error.contains("kernel_target_config(..., batch_lanes)"));
}

#[test]
fn validates_kernel_target_config_against_selected_abi() {
    let mut project = project_with_modules(vec![(
        "kernel_unit.ns",
        r#"
        mod kernel KernelUnit {
          fn profile() {
            const bind_core: i64 = 2;
            const queue_depth: i64 = 8;
            const batch_lanes: i64 = 16;
            let profile_entry: Unit =
              kernel_target_config("apple_ane", "coreml", batch_lanes);
          }
        }
        "#,
    )]);
    project.manifest.abi_requirements = vec![ProjectAbiRequirement {
        domain: "kernel".to_owned(),
        abi: "kernel.apple_ane.coreml.v1".to_owned(),
    }];

    validate_kernel_target_config_contract(&project, "KernelUnit").unwrap();
}

#[test]
fn rejects_kernel_target_config_that_disagrees_with_selected_abi() {
    let mut project = project_with_modules(vec![(
        "kernel_unit.ns",
        r#"
        mod kernel KernelUnit {
          fn profile() {
            const bind_core: i64 = 2;
            const queue_depth: i64 = 8;
            const batch_lanes: i64 = 16;
            let profile_entry: Unit =
              kernel_target_config("x86_64", "cpu-fallback", batch_lanes);
          }
        }
        "#,
    )]);
    project.manifest.abi_requirements = vec![ProjectAbiRequirement {
        domain: "kernel".to_owned(),
        abi: "kernel.apple_ane.coreml.v1".to_owned(),
    }];

    let error = validate_kernel_target_config_contract(&project, "KernelUnit").unwrap_err();
    assert!(error.contains("kernel_target_config(\"apple_ane\", \"coreml\", ...)"));
    assert!(error.contains("kernel.apple_ane.coreml.v1"));
}

#[test]
fn materializes_kernel_slot_contract_node_into_yir() {
    let project = project_with_modules(vec![(
        "kernel_unit.ns",
        r#"
        mod kernel KernelUnit {
          fn profile() {
            const bind_core: i64 = 2;
            const queue_depth: i64 = 8;
            const batch_lanes: i64 = 16;
            let profile_entry: Unit =
              kernel_target_config("apple_ane", "coreml", batch_lanes);
          }
        }
        "#,
    )]);

    let mut yir = YirModule::new("0.1");
    apply_project_support_modules_to_yir(&project, &mut yir).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.name == "project_profile_kernel_KernelUnit_slot_contract_type"));
}

#[test]
fn materializes_default_kernel_target_config_from_project_abi() {
    let mut project = project_with_modules(vec![(
        "kernel_unit.ns",
        r#"
        mod kernel KernelUnit {
          fn profile() {
            const bind_core: i64 = 2;
          }
        }
        "#,
    )]);
    project.manifest.abi_requirements = vec![ProjectAbiRequirement {
        domain: "kernel".to_owned(),
        abi: "kernel.apple_ane.coreml.v1".to_owned(),
    }];

    let mut yir = YirModule::new("0.1");
    apply_project_support_modules_to_yir(&project, &mut yir).unwrap();

    assert!(yir
        .resources
        .iter()
        .any(|resource| resource.name == "kernel0" && resource.kind.raw == "kernel.apple"));
    assert!(yir.nodes.iter().any(|node| {
        node.name == "project_profile_kernel_KernelUnit_kernel_target_config_auto"
            && node.op.module == "kernel"
            && node.op.instruction == "target_config"
            && node.op.args
                == vec![
                    "apple_ane".to_owned(),
                    "coreml".to_owned(),
                    "1".to_owned(),
                    "ane-dispatch,buffer-table,coreml,device.apple-ane,kernel-dispatch,tensor-graph,vendor.apple"
                        .to_owned()
                ]
    }));
    assert!(yir.nodes.iter().any(|node| {
        node.name == "project_profile_kernel_KernelUnit_target_contract_type"
            && node.op.module == "cpu"
            && node.op.instruction == "text"
            && node.op.args
                == vec!["arch=symbol:apple_ane;runtime=symbol:coreml;vendor=symbol:apple;device=symbol:apple-ane;lane_width=i64:1;backend_features=list:ane-dispatch,buffer-table,coreml,device.apple-ane,kernel-dispatch,tensor-graph,vendor.apple".to_owned()]
    }));
    assert!(yir.nodes.iter().any(|node| {
        node.name == "project_profile_kernel_KernelUnit_abi_selection_contract_type"
            && node.op.module == "cpu"
            && node.op.instruction == "text"
            && node.op.args
                == vec!["mode=symbol:explicit;abi=symbol:kernel.apple_ane.coreml.v1;arch=symbol:apple_ane;runtime=symbol:coreml;vendor=symbol:apple;device=symbol:apple-ane;lane_width=i64:1;backend_features=list:ane-dispatch,buffer-table,coreml,device.apple-ane,kernel-dispatch,tensor-graph,vendor.apple".to_owned()]
    }));
}
