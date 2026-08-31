use super::*;

#[test]
fn inspect_project_metadata_json_reports_source_project_summaries() {
    let project_name = "inspect_project_metadata_source_json";
    let project_root = write_temp_project_fixture(
        project_name,
        r#"
name = "inspect_project_metadata_source_json"
entry = "main.ns"
modules = ["main.ns"]
galaxy = ["pixelmagic=workspace"]
"#
        .trim_start(),
        r#"
            use cpu PixelMagicContracts;

            mod cpu Main {
              fn main() -> i64 {
                return PixelMagicContracts.blur_op_kind();
              }
            }
            "#,
    );
    let summary = inspect_project_metadata(&project_root).unwrap();
    let json = inspect_project_metadata_json(&summary);
    assert!(json.contains("\"kind\":\"nuis_project_metadata\""));
    assert!(json.contains("\"source_kind\":\"project-source\""));
    assert!(json.contains("\"project_name\":\"inspect_project_metadata_source_json\""));
    assert!(json.contains("\"imports_library_count\":26"));
    assert!(json.contains("\"galaxy_count\":3"));
    assert!(json.contains("\"host_ffi_symbol_count\":0"));
    assert!(json.contains("\"host_ffi_policy_count\":0"));
}

#[test]
fn inspect_project_metadata_output_dir_reports_build_output_summary() {
    let project_root = write_temp_project_fixture(
        "inspect_project_metadata_output_dir",
        r#"
name = "inspect_project_metadata_output_dir"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
        .trim_start(),
        r#"
mod cpu Main {
  fn main() -> i64 {
    return 7;
  }
}
"#,
    );
    let output_dir = temp_dir("inspect_project_metadata_output_dir_outputs");
    run(CommandKind::Compile {
        input: project_root.clone(),
        output_dir: output_dir.clone(),
        verbose_cache: false,
        cpu_abi: None,
        target: None,
        packaging_mode: None,
    })
    .unwrap();

    let summary = inspect_project_metadata(&output_dir).unwrap();
    let manifest_path = output_dir.join("nuis.build.manifest.toml");
    let artifact_path = output_dir.join("nuis.compiled.artifact");
    assert_eq!(summary.source_kind, "build-output-dir");
    assert_eq!(
        summary.build_manifest_path.as_deref(),
        Some(manifest_path.to_string_lossy().as_ref())
    );
    assert_eq!(
        summary.artifact_path.as_deref(),
        Some(artifact_path.to_string_lossy().as_ref())
    );
}

#[test]
fn inspect_project_metadata_reports_host_ffi_footprint_for_proxy_output() {
    let project_root = PathBuf::from("../../examples/projects/tooling/hetero_proxy_benchmark_demo");
    let output_dir = temp_dir("inspect_project_metadata_hetero_proxy_outputs");

    run(CommandKind::Compile {
        input: project_root,
        output_dir: output_dir.clone(),
        verbose_cache: false,
        cpu_abi: None,
        target: None,
        packaging_mode: None,
    })
    .unwrap();

    let summary = inspect_project_metadata(&output_dir).unwrap();
    assert_eq!(summary.source_kind, "build-output-dir");
    assert_eq!(summary.host_ffi_symbol_count, 2);
    assert_eq!(summary.host_ffi_policy_count, 2);
    assert!(summary
        .host_ffi_index_path
        .as_deref()
        .is_some_and(|path| path.ends_with("nuis.project.host_ffi.txt")));

    let json = inspect_project_metadata_json(&summary);
    assert!(json.contains("\"host_ffi_symbol_count\":2"));
    assert!(json.contains("\"host_ffi_policy_count\":2"));

    let compact = render_project_metadata_compact_summary(&summary);
    assert!(compact.contains("host_ffi=2/2"));
}

#[test]
fn project_metadata_render_helpers_expose_summary_and_paths() {
    let demo_root = temp_dir("project_metadata_render_helpers_paths_demo").join("demo");
    let summary = ProjectMetadataSummary {
        source_kind: "build-manifest".to_owned(),
        project_name: Some("demo".to_owned()),
        project_root: Some(demo_root.to_string_lossy().to_string()),
        manifest_path: Some(demo_root.join("nuis.toml").to_string_lossy().to_string()),
        build_manifest_path: Some(
            demo_root
                .join("build/nuis.build.manifest.toml")
                .to_string_lossy()
                .to_string(),
        ),
        artifact_path: Some(
            demo_root
                .join("build/nuis.compiled.artifact")
                .to_string_lossy()
                .to_string(),
        ),
        docs_index_path: Some(
            demo_root
                .join("build/nuis.project.docs.txt")
                .to_string_lossy()
                .to_string(),
        ),
        docs_module_count: 4,
        docs_documented_module_count: 3,
        docs_documented_item_count: 12,
        imports_index_path: Some(
            demo_root
                .join("build/nuis.project.imports.txt")
                .to_string_lossy()
                .to_string(),
        ),
        imports_library_count: 6,
        imports_visible_library_count: 5,
        imports_visible_module_count: 7,
        imports_documented_visible_module_count: 4,
        imports_documented_visible_item_count: 10,
        galaxy_index_path: Some(
            demo_root
                .join("build/nuis.project.galaxy.txt")
                .to_string_lossy()
                .to_string(),
        ),
        galaxy_count: 3,
        documented_galaxy_count: 2,
        documented_galaxy_library_module_count: 5,
        documented_galaxy_item_count: 10,
        host_ffi_index_path: Some(
            demo_root
                .join("build/nuis.project.host_ffi.txt")
                .to_string_lossy()
                .to_string(),
        ),
        host_ffi_symbol_count: 2,
        host_ffi_policy_count: 2,
    };
    let compact = render_project_metadata_compact_summary(&summary);
    assert!(compact.contains("source_kind=build-manifest"));
    assert!(compact.contains("project=demo"));
    assert!(compact.contains("docs=4/3/12"));
    assert!(compact.contains("imports=6/5/7/4/10"));
    assert!(compact.contains("galaxies=3/2/5/10"));
    assert!(compact.contains("host_ffi=2/2"));

    let paths = render_project_metadata_paths(&summary);
    assert!(paths.contains(&format!("project_root={}", demo_root.display())));
    assert!(paths.contains(&format!(
        "manifest_path={}",
        demo_root.join("nuis.toml").display()
    )));
    assert!(paths.contains(&format!(
        "build_manifest_path={}",
        demo_root.join("build/nuis.build.manifest.toml").display()
    )));
    assert!(paths.contains(&format!(
        "artifact_path={}",
        demo_root.join("build/nuis.compiled.artifact").display()
    )));
    assert!(paths.contains(&format!(
        "docs_index_path={}",
        demo_root.join("build/nuis.project.docs.txt").display()
    )));
    assert!(paths.contains(&format!(
        "imports_index_path={}",
        demo_root.join("build/nuis.project.imports.txt").display()
    )));
    assert!(paths.contains(&format!(
        "galaxy_index_path={}",
        demo_root.join("build/nuis.project.galaxy.txt").display()
    )));
    assert!(paths.contains(&format!(
        "host_ffi_index_path={}",
        demo_root.join("build/nuis.project.host_ffi.txt").display()
    )));
}

#[test]
fn repair_project_metadata_target_rejects_non_manifest_inputs() {
    let error = repair_project_metadata_target(Path::new("examples/demo")).unwrap_err();
    assert!(error.contains("usage: nuisc repair-project-metadata"));
}

#[test]
fn resolve_build_manifest_path_accepts_output_dir() {
    let output_dir = temp_dir("resolve_build_manifest_path_accepts_output_dir");
    let manifest_path = output_dir.join("nuis.build.manifest.toml");
    fs::write(&manifest_path, "schema = \"demo\"\n").unwrap();
    let resolved = resolve_build_manifest_path(&output_dir).unwrap();
    assert_eq!(resolved, manifest_path);
}

#[test]
fn repair_project_metadata_target_reports_missing_original_input() {
    let project_root = write_temp_project_fixture(
        "repair_project_metadata_missing_input",
        r#"
name = "repair_project_metadata_missing_input"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
        .trim_start(),
        r#"
mod cpu Main {
  fn main() -> i64 {
    return 1;
  }
}
"#,
    );
    let output_dir = temp_dir("repair_project_metadata_missing_input_outputs");
    run(CommandKind::Compile {
        input: project_root.clone(),
        output_dir: output_dir.clone(),
        verbose_cache: false,
        cpu_abi: None,
        target: None,
        packaging_mode: None,
    })
    .unwrap();
    fs::remove_dir_all(&project_root).unwrap();
    let manifest_path = output_dir.join("nuis.build.manifest.toml");
    let error = repair_project_metadata_target(&manifest_path).unwrap_err();
    assert!(error.contains("cannot repair project metadata"));
    assert!(error.contains("no longer exists"));
    assert!(error.contains("inspect-project-metadata"));
}

#[test]
fn repair_project_metadata_target_resolves_manifest_to_input_and_output_dir() {
    let project_root = write_temp_project_fixture(
        "repair_project_metadata_target_resolves",
        r#"
name = "repair_project_metadata_target_resolves"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
        .trim_start(),
        r#"
mod cpu Main {
  fn main() -> i64 {
    return 1;
  }
}
"#,
    );
    let output_dir = temp_dir("repair_project_metadata_target_resolves_outputs");
    run(CommandKind::Compile {
        input: project_root.clone(),
        output_dir: output_dir.clone(),
        verbose_cache: false,
        cpu_abi: None,
        target: None,
        packaging_mode: None,
    })
    .unwrap();
    let manifest_path = output_dir.join("nuis.build.manifest.toml");
    let (resolved_input, resolved_output_dir) =
        repair_project_metadata_target(&manifest_path).unwrap();
    assert_eq!(resolved_input, project_root);
    assert_eq!(resolved_output_dir, output_dir);
}

#[test]
fn repair_project_metadata_target_accepts_output_dir() {
    let project_root = write_temp_project_fixture(
        "repair_project_metadata_target_output_dir",
        r#"
name = "repair_project_metadata_target_output_dir"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
        .trim_start(),
        r#"
mod cpu Main {
  fn main() -> i64 {
    return 1;
  }
}
"#,
    );
    let output_dir = temp_dir("repair_project_metadata_target_output_dir_outputs");
    run(CommandKind::Compile {
        input: project_root.clone(),
        output_dir: output_dir.clone(),
        verbose_cache: false,
        cpu_abi: None,
        target: None,
        packaging_mode: None,
    })
    .unwrap();
    let (resolved_input, resolved_output_dir) =
        repair_project_metadata_target(&output_dir).unwrap();
    assert_eq!(resolved_input, project_root);
    assert_eq!(resolved_output_dir, output_dir);
}

#[test]
fn artifact_report_summary_lines_expose_compact_overview() {
    let artifact_verify = aot::NuisCompiledArtifactVerifyReport {
        schema: "nuis-compiled-artifact-v1".to_owned(),
        artifact_container_kind: "compiled-artifact-v1".to_owned(),
        artifact_container_version: 1,
        artifact_section_count: 0,
        artifact_section_names: Vec::new(),
        artifact_section_table_valid: true,
        lowering_unit_count: 0,
        lowering_domain_families: Vec::new(),
        lowering_targets: Vec::new(),
        lowering_units: Vec::new(),
        packaging_mode: "native-cpu-llvm".to_owned(),
        binary_name: "demo".to_owned(),
        binary_bytes: 1,
        host_object_count: 0,
        host_object_ids: Vec::new(),
        host_object_roles: Vec::new(),
        host_object_formats: Vec::new(),
        host_object_bytes: Vec::new(),
        host_object_hashes: Vec::new(),
        build_manifest_bytes: 1,
        envelope_schema: "nuis-executable-envelope-v1".to_owned(),
        envelope_package_count: 1,
        lifecycle_schema: "nuis-lifecycle-contract-v1".to_owned(),
        lifecycle_bootstrap_entry: "main".to_owned(),
        lifecycle_tick_policy: "poll".to_owned(),
        lifecycle_shutdown_policy: "flush".to_owned(),
        lifecycle_yalivia_rpc: "disabled".to_owned(),
        lifecycle_hook_count: 0,
        lifecycle_hook_surface: Vec::new(),
        lifecycle_export_count: 0,
        lifecycle_export_surface: Vec::new(),
        lifecycle_runtime_capability_flags: Vec::new(),
        lifecycle_contract_consistent: true,
        lifecycle_runtime_capability_flags_consistent: true,
        execution_contracts_checked: 1,
        cpu_target_abi: "cpu.arm64.apple_aapcs64".to_owned(),
        cpu_target_machine_arch: "arm64".to_owned(),
        cpu_target_machine_os: "darwin".to_owned(),
        cpu_target_object_format: "mach-o".to_owned(),
        cpu_target_calling_abi: "aapcs64-darwin".to_owned(),
        artifact_roundtrip_verified: true,
    };
    let summary = DomainBuildVerificationSummary {
        all_units_consistent: true,
        total_units: 1,
        host_units_checked: 1,
        hetero_units_checked: 0,
        registry_drift_units: 0,
        failing_units: Vec::new(),
    };
    let link_plan = linker::LinkPlan {
        schema: linker::LINK_PLAN_SCHEMA.to_owned(),
        input: "main.ns".to_owned(),
        output_dir: "out".to_owned(),
        packaging_mode: "native-cpu-llvm".to_owned(),
        cpu_target: linker::LinkPlanCpuTarget {
            abi: "cpu.arm64.apple_aapcs64".to_owned(),
            machine_arch: "arm64".to_owned(),
            machine_os: "darwin".to_owned(),
            object_format: "mach-o".to_owned(),
            calling_abi: "aapcs64-darwin".to_owned(),
            clang_target: "aarch64-apple-darwin".to_owned(),
            cross_compile: false,
        },
        lifecycle: linker::LinkPlanLifecycle {
            bootstrap_entry: "main".to_owned(),
            tick_policy: "poll".to_owned(),
            shutdown_policy: "flush".to_owned(),
            yalivia_rpc: "disabled".to_owned(),
            hook_surface: Vec::new(),
            export_surface: Vec::new(),
            runtime_capability_flags: Vec::new(),
        },
        envelope: linker::LinkPlanEnvelope {
            schema: "nuis-executable-envelope-v1".to_owned(),
            package_count: 1,
            contract_families: vec!["nustar.cpu".to_owned()],
            domain_families: vec!["cpu".to_owned()],
            function_kind: "federated-function".to_owned(),
            graph_kind: "federated-graph".to_owned(),
            default_time_mode: "global".to_owned(),
        },
        compiled_artifact: linker::LinkPlanArtifact {
            path: "out/nuis.compiled.artifact".to_owned(),
            binary_name: "demo".to_owned(),
            binary_path: "out/demo".to_owned(),
            binary_bytes: 1,
            build_manifest_bytes: 1,
            container_kind: Some("compiled-artifact-v1".to_owned()),
            container_version: Some(1),
            section_count: Some(0),
            section_names: Vec::new(),
            section_table_valid: Some(true),
            lowering_unit_count: Some(0),
            lowering_domain_families: Vec::new(),
            lowering_targets: Vec::new(),
            lowering_units: Vec::new(),
            host_objects: Vec::new(),
        },
        bridge_registry_path: None,
        host_bridge_plan_index_path: None,
        lowering_plan_index_path: None,
        lowering_plan_index_source: "unavailable".to_owned(),
        artifact_provider_metadata: vec![],
        host_ffi: linker::LinkPlanHostFfiFootprint {
            index_path: None,
            symbol_count: 0,
            policy_count: 0,
            memory_capability_count: 0,
            policy: "signature-whitelist-required".to_owned(),
            abi_groups: Vec::new(),
            entries: Vec::new(),
            validation: linker::LinkPlanHostFfiValidationSummary {
                checked: 0,
                valid: true,
                link_allowed: true,
                issues: Vec::new(),
                notes: Vec::new(),
            },
        },
        domain_units: vec![linker::LinkPlanDomainUnit {
            kind: "host".to_owned(),
            package_id: "official.cpu".to_owned(),
            domain_family: "cpu".to_owned(),
            abi: Some("cpu.arm64.apple_aapcs64".to_owned()),
            machine_arch: Some("arm64".to_owned()),
            machine_os: Some("darwin".to_owned()),
            backend_family: Some("llvm".to_owned()),
            vendor: None,
            device_class: None,
            target_device: Some("host-cpu".to_owned()),
            ir_format: Some("llvm-bitcode".to_owned()),
            dispatch_abi: Some("nuis-host-call".to_owned()),
            backend_priority: Some(100),
            verification: Some("contract-only".to_owned()),
            selected_lowering_target: Some("llvm".to_owned()),
            contract_family: "nustar.cpu".to_owned(),
            packaging_role: "host-binary".to_owned(),
            artifact_stub_path: None,
            artifact_stub_inline: None,
            artifact_payload_path: None,
            artifact_bridge_stub_path: None,
            artifact_ir_sidecar_path: None,
            artifact_bridge_stub_inline: None,
            artifact_payload_blob_path: None,
            artifact_payload_blob_bytes: None,
            artifact_payload_format: None,
            artifact_payload_blob_inline: None,
        }],
        artifact_lowering_alignment: linker::ArtifactLoweringAlignmentSummary {
            checked: 0,
            mismatches: 0,
            consistent: true,
            checks: Vec::new(),
        },
        clock_protocol: linker::LinkPlanClockProtocol {
            schema: "nuis-clock-protocol-v1".to_owned(),
            mode: "host-lifecycle-clock".to_owned(),
            source: "test".to_owned(),
            default_time_mode: "global".to_owned(),
            lifecycle_tick_policy: "poll".to_owned(),
            domains: vec![linker::LinkPlanClockDomain {
                index: 0,
                domain_family: "cpu".to_owned(),
                package_id: "official.cpu".to_owned(),
                clock_domain_id: "cpu.clock.host.v1".to_owned(),
                clock_kind: "host-monotonic".to_owned(),
                clock_epoch_kind: "host-epoch".to_owned(),
                clock_resolution: "cpu.tick_i64".to_owned(),
                clock_bridge_default: "global->monotonic:bridge".to_owned(),
                lifecycle_hook: "on_scheduler_tick".to_owned(),
            }],
            edges: vec![linker::LinkPlanClockEdge {
                index: 0,
                from: "global.clock.root.v1".to_owned(),
                to: "cpu.clock.host.v1".to_owned(),
                relation: "global->monotonic:bridge".to_owned(),
                source: "test".to_owned(),
            }],
            validation: linker::LinkPlanClockValidationSummary {
                checked: 1,
                valid: true,
                issues: Vec::new(),
            },
        },
        hetero_calculate: linker::LinkPlanHeteroCalculate {
            schema: "nuis-hetero-calculate-link-plan-v1".to_owned(),
            mode: "host-only".to_owned(),
            static_link: true,
            lifecycle_driven: true,
            time_order_model: "timestamped-partial-order".to_owned(),
            data_order_model: "deterministic-segment-order".to_owned(),
            c_world_policy: "wrapped-ordinary-node-no-linker-fast-path".to_owned(),
            nodes: Vec::new(),
            data_segments: Vec::new(),
            validation: linker::LinkPlanHeteroValidationSummary {
                checked: 6,
                valid: true,
                issues: Vec::new(),
            },
        },
        final_stage: linker::LinkPlanFinalStage {
            kind: "host-native-link".to_owned(),
            driver: "clang".to_owned(),
            link_mode: "host-toolchain-finalize".to_owned(),
            output_path: "out/demo".to_owned(),
            inputs: vec![
                "out/nuis.compiled.artifact".to_owned(),
                "out/nuis.executable.envelope.toml".to_owned(),
            ],
            notes: vec!["demo".to_owned()],
        },
    };
    let execution_overview = ExecutionInspectOverview {
        heterogeneous_domains: 1,
        domains: vec![ExecutionInspectDomainOverview {
            domain_family: "network".to_owned(),
            selected_lowering_target: Some("urlsession.socket-io".to_owned()),
            phase_count: 4,
            event_count: 4,
            resource_keys: vec![
                "active_response".to_owned(),
                "active_session".to_owned(),
                "request_packet".to_owned(),
            ],
            output_handles: vec![
                "response.handle".to_owned(),
                "session.handle".to_owned(),
                "status.code".to_owned(),
                "task.handle".to_owned(),
            ],
        }],
    };
    let lines = artifact_report_summary_lines(
        &artifact_verify,
        &summary,
        Some(&link_plan),
        false,
        Some(&execution_overview),
        Some(&[frontend::AstDocIndex {
            module_path: "cpu.Main".to_owned(),
            items: vec![frontend::AstDocIndexItem {
                kind: "function".to_owned(),
                path: "cpu.Main::main".to_owned(),
                docs: vec!["entry docs".to_owned()],
                signature: Some("fn main() -> i64".to_owned()),
            }],
        }]),
        None,
    );

    assert_eq!(lines.len(), 8);
    assert!(lines[0].contains("artifact_roundtrip=ok"));
    assert!(lines[0].contains("lifecycle=ok"));
    assert!(lines[0].contains("runtime_flags=ok"));
    assert!(lines[0].contains("all_units_consistent=true"));
    assert!(lines[1].contains("total=1"));
    assert!(lines[1].contains("host=1"));
    assert!(lines[1].contains("hetero=0"));
    assert!(lines[1].contains("drift=0"));
    assert!(lines[1].contains("failing=<none>"));
    assert_eq!(lines[2], "summary_manifest: reconstructed=false");
    assert_eq!(lines[3], "summary_host_objects: count=0 roles=<none>");
    assert!(lines[4].contains("final_stage=host-native-link"));
    assert!(lines[4].contains("driver=clang"));
    assert!(lines[5].contains("summary_execution: hetero_domains=1"));
    assert!(lines[5].contains("network(target=urlsession.socket-io phases=4 events=4)"));
    assert_eq!(lines[6], "summary_execution_issues: <none>");
    assert_eq!(
        lines[7],
        "summary_docs: modules=1 documented_items=1 documented_modules=cpu.Main"
    );

    let v1_link_plan_json = link_plan_json(&link_plan);
    assert!(v1_link_plan_json.contains("\"artifact_lowering_alignment\":{"));
    assert!(v1_link_plan_json.contains("\"checked\":0"));
    assert!(v1_link_plan_json.contains("\"mismatches\":0"));
    assert!(v1_link_plan_json.contains("\"consistent\":true"));
    assert!(v1_link_plan_json.contains("\"hetero_calculate\":{"));
    assert!(v1_link_plan_json.contains("\"schema\":\"nuis-hetero-calculate-link-plan-v1\""));
    assert!(v1_link_plan_json.contains("\"static_link\":true"));
    assert!(v1_link_plan_json.contains("\"lifecycle_driven\":true"));
    assert!(v1_link_plan_json.contains("\"time_order_model\":\"timestamped-partial-order\""));
    assert!(v1_link_plan_json.contains("\"data_order_model\":\"deterministic-segment-order\""));
    assert!(v1_link_plan_json
        .contains("\"c_world_policy\":\"wrapped-ordinary-node-no-linker-fast-path\""));
    assert!(v1_link_plan_json.contains("\"validation\":{"));
    assert!(v1_link_plan_json.contains("\"valid\":true"));
    assert!(v1_link_plan_json.contains("\"issues\":[]"));

    let mut v2_link_plan = link_plan.clone();
    v2_link_plan.compiled_artifact.container_kind =
        Some("compiled-artifact-section-table-v2".to_owned());
    v2_link_plan.compiled_artifact.container_version = Some(2);
    v2_link_plan.compiled_artifact.section_count = Some(6);
    v2_link_plan.compiled_artifact.lowering_unit_count = Some(1);
    v2_link_plan.compiled_artifact.lowering_domain_families = vec!["cpu".to_owned()];
    v2_link_plan.compiled_artifact.lowering_targets = vec!["llvm".to_owned()];
    v2_link_plan.compiled_artifact.lowering_units =
        vec![aot::NuisCompiledArtifactLoweringUnitInspect {
            package_id: "official.cpu".to_owned(),
            domain_family: "cpu".to_owned(),
            backend_family: Some("llvm".to_owned()),
            target_device: Some("host-cpu".to_owned()),
            ir_format: Some("llvm-bitcode".to_owned()),
            dispatch_abi: Some("nuis-host-call".to_owned()),
            backend_priority: Some(100),
            verification: Some("contract-only".to_owned()),
            selected_lowering_target: Some("llvm".to_owned()),
            artifact_ir_sidecar_path: None,
            contract_family: "nustar.cpu".to_owned(),
            packaging_role: "host-binary".to_owned(),
        }];
    v2_link_plan.artifact_lowering_alignment = linker::build_artifact_lowering_alignment_summary(
        &v2_link_plan.compiled_artifact,
        &v2_link_plan.domain_units,
    );
    let v2_link_plan_json = link_plan_json(&v2_link_plan);
    assert!(v2_link_plan_json.contains("\"artifact_lowering_alignment\":{"));
    assert!(v2_link_plan_json.contains("\"checked\":1"));
    assert!(v2_link_plan_json.contains("\"mismatches\":0"));
    assert!(v2_link_plan_json.contains("\"checks\":[{"));
}
