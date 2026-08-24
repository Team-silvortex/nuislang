use super::*;

#[test]
fn aarch64_cpu_nustar_is_independent_package_for_cpu_domain() {
    let generic_cpu = load_manifest_for_domain(Path::new("nustar-packages"), "cpu").unwrap();
    assert_eq!(generic_cpu.package_id, "official.cpu");
    assert!(generic_cpu
        .abi_profiles
        .contains(&"cpu.x86_64.sysv64".to_owned()));

    let aarch64_cpu = load_manifest(Path::new("nustar-packages"), "official.cpu.aarch64").unwrap();
    assert_eq!(aarch64_cpu.domain_family, "cpu");
    assert_eq!(aarch64_cpu.package_id, "official.cpu.aarch64");
    assert!(aarch64_cpu
        .capability_tags
        .contains(&"formal-verification-ready".to_owned()));
    assert!(aarch64_cpu
        .capability_tags
        .contains(&"aarch64-only".to_owned()));
    assert!(aarch64_cpu
        .part_verify
        .contains(&"verify.cpu.aarch64.call-frame.v1".to_owned()));
    assert!(aarch64_cpu
        .abi_profiles
        .iter()
        .all(|abi| abi.starts_with("cpu.arm64.")));
    assert!(aarch64_cpu
        .lowering_targets
        .contains(&"aarch64-proof-skeleton".to_owned()));
}

#[test]
fn cffi_nustar_owns_the_registered_host_boundary() {
    let cffi = load_manifest_for_domain(Path::new("nustar-packages"), "cffi").unwrap();
    assert_eq!(cffi.package_id, "official.cffi");
    assert_eq!(cffi.yir_lowering_entry, "cffi.yir.lowering.v1");
    assert_eq!(cffi.host_ffi_abis, ["nurs", "c", "libc", "libm"]);
    assert_eq!(cffi.linker_resolver_providers.len(), 2);
    assert_eq!(cffi.linker_symbol_versions.len(), 4);
    assert!(cffi
        .capability_tags
        .contains(&"signature-whitelist".to_owned()));
}

#[test]
fn validate_registered_domains_accepts_current_mainline_registry() {
    let issues = validate_registered_domains(Path::new("nustar-packages")).unwrap();
    assert!(issues.is_empty(), "unexpected registry issues: {issues:?}");
    ensure_registered_domains_valid(Path::new("nustar-packages")).unwrap();
}

#[test]
fn registered_provider_bundles_are_manifest_owned_and_deterministic() {
    let manifests = load_all_manifests(Path::new("nustar-packages")).unwrap();
    let mut registrations = manifests
        .iter()
        .flat_map(|manifest| provider_bundle_registrations(manifest).unwrap())
        .collect::<Vec<_>>();
    registrations.sort_by(|lhs, rhs| lhs.bundle_id.cmp(&rhs.bundle_id));

    assert_eq!(registrations.len(), 5);
    assert_eq!(
        registrations
            .iter()
            .map(|registration| registration.bundle_id.as_str())
            .collect::<Vec<_>>(),
        vec![
            "coreml.apple-ane.bundle.v1",
            "cuda.nvidia-gpu.bundle.v1",
            "data.host.bundle.v1",
            "metal.apple-silicon-gpu.bundle.v1",
            "spirv.vulkan-gpu.bundle.v1",
        ]
    );
    assert!(registrations
        .iter()
        .all(|registration| { registration.rust_const.ends_with("::PROVIDER_BUNDLE") }));
}

#[test]
fn validate_registered_domains_allows_duplicate_domain_but_rejects_bad_lane_target() {
    let root = temp_registry_root("registry-duplicate-domain");
    let cpu = cpu_manifest_with_host_target();
    let mut network = load_manifest_for_domain(Path::new("nustar-packages"), "network").unwrap();
    network.default_lanes.push("network.ghost=rx".to_owned());
    let entries = vec![
        NustarPackageIndexEntry {
            package_id: cpu.package_id.clone(),
            manifest: "cpu.toml".to_owned(),
            domain_family: cpu.domain_family.clone(),
        },
        NustarPackageIndexEntry {
            package_id: network.package_id.clone(),
            manifest: "network.toml".to_owned(),
            domain_family: cpu.domain_family.clone(),
        },
    ];
    write_registry_fixture(&root, &entries, &[cpu, network]);

    let issues = validate_registered_domains(&root).unwrap();
    assert!(issues
        .iter()
        .any(|issue| issue.kind == NustarRegistryIssueKind::DomainFamilyMismatch));
    assert!(issues
        .iter()
        .any(|issue| issue.kind == NustarRegistryIssueKind::LaneContractMismatch));
    let error = ensure_registered_domains_valid(&root).unwrap_err();
    assert!(error.contains("NRV005"));
    assert!(error.contains("NRV010"));
}

#[test]
fn validate_registered_domains_rejects_loader_and_op_contract_mismatch() {
    let root = temp_registry_root("registry-loader-op");
    let mut cpu = cpu_manifest_with_host_target();
    cpu.loader_abi = "wrong-loader".to_owned();
    cpu.ops.push("shader.draw".to_owned());
    let entries = vec![NustarPackageIndexEntry {
        package_id: cpu.package_id.clone(),
        manifest: "cpu.toml".to_owned(),
        domain_family: cpu.domain_family.clone(),
    }];
    write_registry_fixture(&root, &entries, &[cpu]);

    let issues = validate_registered_domains(&root).unwrap();
    assert!(issues
        .iter()
        .any(|issue| issue.kind == NustarRegistryIssueKind::LoaderContractMismatch));
    assert!(issues
        .iter()
        .any(|issue| issue.kind == NustarRegistryIssueKind::OpContractMismatch));
}

#[test]
fn validate_registered_domains_rejects_shader_backend_without_lowering_target() {
    let root = temp_registry_root("registry-shader-backend");
    let mut shader = load_manifest_for_domain(Path::new("nustar-packages"), "shader").unwrap();
    shader
        .lowering_targets
        .retain(|target| target != "cpu-fallback");
    let entries = vec![NustarPackageIndexEntry {
        package_id: shader.package_id.clone(),
        manifest: "shader.toml".to_owned(),
        domain_family: shader.domain_family.clone(),
    }];
    write_registry_fixture(&root, &entries, &[shader]);

    let issues = validate_registered_domains(&root).unwrap();
    assert!(issues.iter().any(|issue| {
        issue.kind == NustarRegistryIssueKind::DomainContractMismatch
            && issue.message.contains("cpu-fallback")
    }));
}

#[test]
fn validate_registered_domains_rejects_shader_missing_texture_profile_slot() {
    let root = temp_registry_root("registry-shader-texture-slot");
    let mut shader = load_manifest_for_domain(Path::new("nustar-packages"), "shader").unwrap();
    shader
        .support_profile_slots
        .retain(|slot| slot != "texture_format");
    let entries = vec![NustarPackageIndexEntry {
        package_id: shader.package_id.clone(),
        manifest: "shader.toml".to_owned(),
        domain_family: shader.domain_family.clone(),
    }];
    write_registry_fixture(&root, &entries, &[shader]);

    let issues = validate_registered_domains(&root).unwrap();
    assert!(issues.iter().any(|issue| {
        issue.kind == NustarRegistryIssueKind::DomainContractMismatch
            && issue.message.contains("texture_format")
    }));
}

#[test]
fn validate_registered_domains_rejects_kernel_missing_profile_slot() {
    let root = temp_registry_root("registry-kernel-slot");
    let mut kernel = load_manifest_for_domain(Path::new("nustar-packages"), "kernel").unwrap();
    kernel
        .support_profile_slots
        .retain(|slot| slot != "batch_lanes");
    let entries = vec![NustarPackageIndexEntry {
        package_id: kernel.package_id.clone(),
        manifest: "kernel.toml".to_owned(),
        domain_family: kernel.domain_family.clone(),
    }];
    write_registry_fixture(&root, &entries, &[kernel]);

    let issues = validate_registered_domains(&root).unwrap();
    assert!(issues.iter().any(|issue| {
        issue.kind == NustarRegistryIssueKind::DomainContractMismatch
            && issue.message.contains("batch_lanes")
    }));
}

#[test]
fn validate_registered_domains_rejects_network_missing_socket_lowering_target() {
    let root = temp_registry_root("registry-network-lowering");
    let mut network = load_manifest_for_domain(Path::new("nustar-packages"), "network").unwrap();
    network
        .lowering_targets
        .retain(|target| target != "socket-abi");
    let entries = vec![NustarPackageIndexEntry {
        package_id: network.package_id.clone(),
        manifest: "network.toml".to_owned(),
        domain_family: network.domain_family.clone(),
    }];
    write_registry_fixture(&root, &entries, &[network]);

    let issues = validate_registered_domains(&root).unwrap();
    assert!(issues.iter().any(|issue| {
        issue.kind == NustarRegistryIssueKind::DomainContractMismatch
            && issue.message.contains("socket-abi")
    }));
}

#[test]
fn validate_registered_domains_rejects_incomplete_host_bridge_contract() {
    let root = temp_registry_root("registry-host-bridge-missing");
    let mut network = load_manifest_for_domain(Path::new("nustar-packages"), "network").unwrap();
    network.host_bridge_phase_wait_wake = None;
    let entries = vec![NustarPackageIndexEntry {
        package_id: network.package_id.clone(),
        manifest: "network.toml".to_owned(),
        domain_family: network.domain_family.clone(),
    }];
    write_registry_fixture(&root, &entries, &[network]);

    let issues = validate_registered_domains(&root).unwrap();
    assert!(issues.iter().any(|issue| {
        issue.kind == NustarRegistryIssueKind::DomainContractMismatch
            && issue.message.contains("host_bridge_phase_wait_wake")
    }));
}

#[test]
fn validate_registered_domains_rejects_invalid_host_bridge_phase_order() {
    let root = temp_registry_root("registry-host-bridge-order");
    let mut kernel = load_manifest_for_domain(Path::new("nustar-packages"), "kernel").unwrap();
    kernel.host_bridge_phase_order = Some(vec![
        "bind".to_owned(),
        "wait".to_owned(),
        "submit".to_owned(),
        "finalize".to_owned(),
    ]);
    let entries = vec![NustarPackageIndexEntry {
        package_id: kernel.package_id.clone(),
        manifest: "kernel.toml".to_owned(),
        domain_family: kernel.domain_family.clone(),
    }];
    write_registry_fixture(&root, &entries, &[kernel]);

    let issues = validate_registered_domains(&root).unwrap();
    assert!(issues.iter().any(|issue| {
        issue.kind == NustarRegistryIssueKind::DomainContractMismatch
            && issue.message.contains("phase_order")
    }));
}

#[test]
fn ensure_project_domain_registry_valid_accepts_registered_abi() {
    let plan = test_project_plan("network", "network.socket.macos.arm64.v1");
    let checks = validate_project_domain_registry(&plan);
    assert!(checks.iter().all(|check| check.issues.is_empty()));
    let network = checks
        .iter()
        .find(|check| check.domain == "network")
        .unwrap();
    assert_eq!(network.issue_count(), 0);
    assert!(network.summary_line().contains(": ok"));
    assert!(network.abi_registered);
    ensure_project_domain_registry_valid(&plan).unwrap();
}

#[test]
fn ensure_project_domain_registry_valid_rejects_unknown_abi() {
    let plan = test_project_plan("network", "network.socket.unknown.v1");
    let checks = validate_project_domain_registry(&plan);
    let network = checks
        .iter()
        .find(|check| check.domain == "network")
        .unwrap();
    assert!(network
        .issues
        .iter()
        .any(|issue| issue.kind == ProjectDomainRegistryIssueKind::AbiNotRegistered));
    assert!(network
        .issues
        .iter()
        .any(|issue| issue.kind.code() == "NRG003"));
    assert!(network.summary_line().contains("NRG003 abi_not_registered"));
    let error = ensure_project_domain_registry_valid(&plan).unwrap_err();
    assert!(error.contains("project domain registry validation failed"));
    assert!(error.contains("network"));
    assert!(error.contains("network.socket.unknown.v1"));
    assert!(error.contains("NRG003"));
    assert!(error.contains("abi_not_registered"));
}

#[test]
fn binding_plan_carries_execution_skeleton_summary() {
    let plan = binding_plan_from_source(
        r#"
use shader SurfaceShader;

mod cpu Main {
  fn main() {
    print(0);
  }
}
"#,
    );
    let shader = plan
        .bindings
        .iter()
        .find(|binding| binding.domain_family == "shader")
        .expect("shader binding should exist");
    assert_eq!(
        shader.execution.skeleton_version,
        "nustar-execution-skeleton-v1"
    );
    assert_eq!(shader.execution.function_kind, "function-node");
    assert_eq!(shader.execution.graph_kind, "function-graph");
    assert_eq!(shader.execution.execution_domain, "shader");
    assert_eq!(shader.execution.contract_family, "nustar.shader");
    assert!(shader
        .execution
        .lowering_targets
        .contains(&"metal".to_owned()));
}

#[test]
fn project_domain_registry_check_renderers_expose_codes_and_issue_counts() {
    let plan = test_project_plan("network", "network.socket.unknown.v1");
    let check = validate_project_domain_registry(&plan)
        .into_iter()
        .find(|check| check.domain == "network")
        .expect("network check");
    let json = project_domain_registry_check_json(&check);
    assert!(json.contains("\"domain\":\"network\""));
    assert!(json.contains("\"code\":\"NRG003\""));
    assert!(json.contains("\"kind\":\"abi_not_registered\""));
    let lines = render_project_domain_registry_check_lines(&check);
    assert!(!lines.is_empty());
    assert!(lines[0].contains("issues=1"));
    assert!(lines
        .iter()
        .any(|line| line.contains("NRG003 abi_not_registered")));
    let mut written = String::new();
    write_project_domain_registry_check_lines(&mut written, &check).unwrap();
    assert_eq!(written.lines().collect::<Vec<_>>(), lines);
}

#[test]
fn registered_abi_target_accepts_darwin_x86_64_domain_profiles() {
    let network = load_manifest_for_domain(Path::new("nustar-packages"), "network").unwrap();
    let data = load_manifest_for_domain(Path::new("nustar-packages"), "data").unwrap();
    let shader = load_manifest_for_domain(Path::new("nustar-packages"), "shader").unwrap();

    let network_target = registered_abi_target(&network, "network.socket.macos.x86_64.v1").unwrap();
    assert_eq!(network_target.machine_arch, "x86_64");
    assert_eq!(network_target.machine_os, "darwin");
    assert_eq!(network_target.clang_target, "x86_64-apple-darwin");

    let data_target = registered_abi_target(&data, "data.fabric.macos.x86_64.v1").unwrap();
    assert_eq!(data_target.machine_arch, "x86_64");
    assert_eq!(data_target.machine_os, "darwin");
    assert_eq!(data_target.clang_target, "x86_64-apple-darwin");

    let shader_target = registered_abi_target(&shader, "shader.metal.x86_64.msl2_4").unwrap();
    assert_eq!(shader_target.machine_arch, "x86_64");
    assert_eq!(shader_target.machine_os, "darwin");
    assert_eq!(shader_target.clang_target, "x86_64-apple-darwin");
    assert_eq!(shader_target.backend_family.as_deref(), Some("metal"));
    assert_eq!(shader_target.vendor.as_deref(), Some("apple"));
    assert_eq!(
        shader_target.device_class.as_deref(),
        Some("mac-discrete-or-integrated-gpu")
    );
}

#[test]
fn network_binding_plan_detects_profile_surfaces_and_slots() {
    let source = r#"
use network NetworkUnit;

mod cpu Main {
  fn capture_network_profile_summary() -> i64 {
    let bind_core: i64 = network_profile_bind_core("NetworkUnit");
    let endpoint_kind: i64 = network_profile_endpoint_kind("NetworkUnit");
    let timeout_budget: i64 = network_profile_timeout_budget("NetworkUnit");
    let retry_budget: i64 = network_profile_retry_budget("NetworkUnit");
    let stream_window: i64 = network_profile_stream_window("NetworkUnit");
    let recv_window: i64 = network_profile_recv_window("NetworkUnit");
    let send_window: i64 = network_profile_send_window("NetworkUnit");
    return bind_core + endpoint_kind + timeout_budget + retry_budget + stream_window + recv_window + send_window;
  }

  fn main() {
    print(capture_network_profile_summary());
  }
}
"#;
    let plan = binding_plan_from_source(source);

    let binding = plan
        .bindings
        .iter()
        .find(|binding| binding.domain_family == "network")
        .expect("network binding should be present");

    for surface in [
        "network.profile.bind-core.v1",
        "network.profile.endpoint-kind.v1",
        "network.profile.timeout.v1",
        "network.profile.retry.v1",
        "network.profile.stream-window.v1",
        "network.profile.recv.v1",
        "network.profile.send.v1",
    ] {
        assert!(
            binding
                .matched_support_surface
                .iter()
                .any(|candidate| candidate == surface),
            "expected matched network surface `{surface}`"
        );
    }

    for slot in [
        "bind_core",
        "endpoint_kind",
        "timeout_budget",
        "retry_budget",
        "stream_window",
        "recv_window",
        "send_window",
    ] {
        assert!(
            binding
                .matched_support_profile_slots
                .iter()
                .any(|candidate| candidate == slot),
            "expected matched network slot `{slot}`"
        );
        assert!(
            binding
                .covered_support_profile_slots
                .iter()
                .any(|candidate| candidate == slot),
            "expected covered network slot `{slot}`"
        );
    }
    assert!(binding.capability_tags.contains(&"async-bridge".to_owned()));
}

#[test]
fn data_binding_plan_detects_profile_surfaces_and_slots() {
    let plan = binding_plan_from_source(DATA_BINDING_SOURCE);
    let binding = plan
        .bindings
        .iter()
        .find(|binding| binding.domain_family == "data")
        .expect("data binding should be present");
    for surface in ["data.profile.bind-core.v1", "data.profile.window-layout.v1"] {
        assert!(
            binding
                .matched_support_surface
                .iter()
                .any(|candidate| candidate == surface),
            "expected matched data surface `{surface}`"
        );
    }
    for slot in ["bind_core", "window_offset", "uplink_len", "downlink_len"] {
        assert!(
            binding
                .matched_support_profile_slots
                .iter()
                .any(|candidate| candidate == slot),
            "expected matched data slot `{slot}`"
        );
    }
}

#[test]
fn kernel_binding_plan_detects_profile_surfaces_and_slots() {
    let source = r#"
use kernel KernelUnit;

mod cpu Main {
  fn capture_kernel_profile_summary() -> i64 {
    let bind_core: i64 = kernel_profile_bind_core("KernelUnit");
    let queue_depth: i64 = kernel_profile_queue_depth("KernelUnit");
    let batch_lanes: i64 = kernel_profile_batch_lanes("KernelUnit");
    return bind_core + queue_depth + batch_lanes;
  }

  fn main() {
    print(capture_kernel_profile_summary());
  }
}
"#;
    let plan = binding_plan_from_source(source);
    let binding = plan
        .bindings
        .iter()
        .find(|binding| binding.domain_family == "kernel")
        .expect("kernel binding should be present");
    for surface in [
        "kernel.profile.bind-core.v1",
        "kernel.profile.queue-depth.v1",
        "kernel.profile.batch-lanes.v1",
    ] {
        assert!(
            binding
                .matched_support_surface
                .iter()
                .any(|candidate| candidate == surface),
            "expected matched kernel surface `{surface}`"
        );
    }
    for slot in ["bind_core", "queue_depth", "batch_lanes"] {
        assert!(
            binding
                .matched_support_profile_slots
                .iter()
                .any(|candidate| candidate == slot),
            "expected matched kernel slot `{slot}`"
        );
    }
}

#[test]
fn shader_binding_plan_detects_profile_surfaces_and_slots() {
    let source = r#"
use shader SurfaceShader;

mod cpu Main {
  fn capture_shader_profile_summary() -> i64 {
    let target: Target = shader_profile_target("SurfaceShader");
    let viewport: Viewport = shader_profile_viewport("SurfaceShader");
    let pipeline: Pipeline = shader_profile_pipeline("SurfaceShader");
    let vertex_count: i64 = shader_profile_vertex_count("SurfaceShader");
    let instance_count: i64 = shader_profile_instance_count("SurfaceShader");
    let _ = target;
    let _ = viewport;
    let _ = pipeline;
    return vertex_count + instance_count;
  }

  fn main() {
    print(capture_shader_profile_summary());
  }
}
"#;
    let plan = binding_plan_from_source(source);
    let binding = plan
        .bindings
        .iter()
        .find(|binding| binding.domain_family == "shader")
        .expect("shader binding should be present");
    for surface in [
        "shader.profile.target.v1",
        "shader.profile.viewport.v1",
        "shader.profile.pipeline.v1",
        "shader.profile.draw-budget.v1",
    ] {
        assert!(
            binding
                .matched_support_surface
                .iter()
                .any(|candidate| candidate == surface),
            "expected matched shader surface `{surface}`"
        );
    }
    for slot in [
        "target",
        "viewport",
        "pipeline",
        "vertex_count",
        "instance_count",
    ] {
        assert!(
            binding
                .matched_support_profile_slots
                .iter()
                .any(|candidate| candidate == slot),
            "expected matched shader slot `{slot}`"
        );
    }
}

#[test]
fn shader_binding_plan_detects_nova_packet_surface_and_covered_slots() {
    let source = r#"
use shader SurfaceShader;

mod cpu Main {
  fn main() {
    let packet: NovaPanelPacket =
      shader_profile_panel_packet("SurfaceShader", 1, 2, 3, 4, 5, 6);
    let _ = packet;
    print(0);
  }
}
"#;
    let plan = binding_plan_from_source(source);
    let binding = plan
        .bindings
        .iter()
        .find(|binding| binding.domain_family == "shader")
        .expect("shader binding should be present");
    assert!(binding
        .matched_support_surface
        .iter()
        .any(|surface| surface == "shader.profile.packet.nova.v1"));
    for slot in [
        "slider_color_slot",
        "slider_speed_slot",
        "slider_radius_slot",
        "header_accent_slot",
        "toggle_live_slot",
        "focus_slot",
    ] {
        assert!(
            binding
                .covered_support_profile_slots
                .iter()
                .any(|candidate| candidate == slot),
            "expected covered shader slot `{slot}`"
        );
    }
}

#[test]
fn shader_binding_plan_detects_nova_profile_slot_accessors() {
    let source = r#"
use shader SurfaceShader;

mod cpu Main {
  fn capture_shader_nova_profile_summary() -> i64 {
    let slider_color: i64 = shader_profile_slider_color_slot("SurfaceShader");
    let slider_speed: i64 = shader_profile_slider_speed_slot("SurfaceShader");
    let slider_radius: i64 = shader_profile_slider_radius_slot("SurfaceShader");
    let header_accent: i64 = shader_profile_header_accent_slot("SurfaceShader");
    let toggle_live: i64 = shader_profile_toggle_live_slot("SurfaceShader");
    let focus: i64 = shader_profile_focus_slot("SurfaceShader");
    return slider_color + slider_speed + slider_radius + header_accent + toggle_live + focus;
  }

  fn main() {
    print(capture_shader_nova_profile_summary());
  }
}
"#;
    let plan = binding_plan_from_source(source);
    let binding = plan
        .bindings
        .iter()
        .find(|binding| binding.domain_family == "shader")
        .expect("shader binding should be present");
    for surface in [
        "shader.profile.packet-slots.v1",
        "shader.profile.packet.nova.v1",
    ] {
        assert!(
            binding
                .matched_support_surface
                .iter()
                .any(|candidate| candidate == surface),
            "expected matched shader surface `{surface}`"
        );
    }
    for slot in [
        "slider_color_slot",
        "slider_speed_slot",
        "slider_radius_slot",
        "header_accent_slot",
        "toggle_live_slot",
        "focus_slot",
    ] {
        assert!(
            binding
                .matched_support_profile_slots
                .iter()
                .any(|candidate| candidate == slot),
            "expected matched shader slot `{slot}`"
        );
    }
}

#[test]
fn shader_binding_plan_detects_packet_binding_profile_contract_surface() {
    let source = r#"
use shader SurfaceShader;

mod cpu Main {
  fn main() {
    let packet: NovaPanelPacket =
      shader_profile_panel_packet("SurfaceShader", 1, 2, 3, 4, 5, 6);
    let binding: Binding = shader_packet_uniform_binding(4, packet);
    print(binding);
  }
}
"#;
    let nir = crate::frontend::parse_nuis_module(source).expect("source should lower to nir");
    let (matched_support_surface, matched_support_profile_slots) =
        detect_matched_support_usage(&nir, "shader");
    let covered_support_profile_slots = covered_profile_slots(
        "shader",
        &matched_support_surface,
        &matched_support_profile_slots,
    );
    assert!(matched_support_surface
        .iter()
        .any(|surface| surface == "shader.profile.packet.nova.v1"));
    for slot in [
        "slider_color_slot",
        "slider_speed_slot",
        "slider_radius_slot",
        "header_accent_slot",
        "toggle_live_slot",
        "focus_slot",
    ] {
        assert!(
            covered_support_profile_slots
                .iter()
                .any(|candidate| candidate == slot),
            "expected covered shader slot `{slot}`"
        );
    }
}

#[test]
fn shader_binding_plan_collects_packet_binding_resource_hints() {
    let source = r#"
use shader SurfaceShader;

mod cpu Main {
  fn main() {
    let packet: NovaPanelPacket =
      shader_profile_panel_packet("SurfaceShader", 1, 2, 3, 4, 5, 6);
    let binding: Binding = shader_packet_uniform_binding(4, packet);
    let pipeline: Pipeline = shader_profile_pipeline("SurfaceShader");
    let bindings: BindingSet = shader_bind_set(pipeline, binding);
    print(bindings);
  }
}
"#;
    let nir = crate::frontend::parse_nuis_module(source).expect("source should lower to nir");
    let mut resources = BTreeSet::new();
    collect_resource_usage_hints(&nir, "shader", &mut resources);
    for resource in [
        "shader.binding.uniform_binding",
        "shader.binding.layout.std140",
        "shader.binding.contract.shader.profile.packet.nova.v1",
        "shader.binding.set",
    ] {
        assert!(
            resources.iter().any(|candidate| candidate == resource),
            "expected matched shader resource hint `{resource}`"
        );
    }
}
