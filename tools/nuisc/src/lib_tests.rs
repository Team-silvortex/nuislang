use super::*;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("nuisc_{label}_{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_temp_project_fixture(name: &str, manifest: &str, entry_source: &str) -> PathBuf {
    let root = temp_dir(name);
    fs::write(root.join("nuis.toml"), manifest).unwrap();
    fs::write(root.join("main.ns"), entry_source).unwrap();
    root
}

#[test]
fn domain_contract_json_exposes_grouped_contract_sections() {
    let contract =
        registry::load_domain_contract_for_domain(Path::new(NUSTAR_REGISTRY_ROOT), "network")
            .expect("expected network domain contract");
    let json = registry::domain_contract_json(&contract);

    assert!(json.contains("\"contract_schema\":\"nustar-domain-contract-v1\""));
    assert!(json.contains("\"contract\":{"));
    assert!(json.contains("\"schema\":\"nustar-domain-contract-v1\""));
    assert!(json.contains("\"groups\":[\"package_identity\""));
    assert!(json.contains("\"package_identity\":{"));
    assert!(json.contains("\"loader_contract\":{"));
    assert!(json.contains("\"abi_contract\":{"));
    assert!(json.contains("\"host_bridge_contract\":{"));
    assert!(json.contains("\"runtime_capability_contract\":{"));
    assert!(json.contains("\"capability_tags\":[\"io-reactor\""));
    assert!(json.contains("\"scheduler_contract\":{"));
    assert!(json.contains("\"std_net_extension\":{"));
    assert!(json.contains("\"domain\":\"network\""));
}

#[test]
fn domain_registration_json_exposes_registration_section() {
    let registration = registry::load_registered_domains(Path::new(NUSTAR_REGISTRY_ROOT))
        .expect("expected registered domains")
        .into_iter()
        .find(|item| item.domain_family == "network")
        .expect("expected network registration");
    let json = registry::domain_registration_json(&registration);

    assert!(json.contains("\"registration\":{"));
    assert!(json.contains("\"manifest_path\":"));
    assert!(json.contains("\"entry_crate\":"));
    assert!(json.contains("\"ast_entry\":"));
    assert!(json.contains("\"nir_entry\":"));
    assert!(json.contains("\"yir_lowering_entry\":"));
    assert!(json.contains("\"part_verify_entry\":"));
    assert!(json.contains("\"ast_surface\":["));
    assert!(json.contains("\"nir_surface\":["));
    assert!(json.contains("\"ops\":["));
}

#[test]
fn domain_build_contract_summary_json_exposes_grouped_sections() {
    let manifest = registry::load_manifest_for_domain(Path::new(NUSTAR_REGISTRY_ROOT), "network")
        .expect("expected network manifest");
    let json =
        domain_build_contract_summary_json(&registry::domain_build_contract_summary(&manifest));

    assert!(json.contains("\"lowering\":{"));
    assert!(json.contains("\"backend\":{"));
    assert!(json.contains("\"bridge\":{"));
    assert!(json.contains("\"host_bridge\":{"));
    assert!(json.contains("\"lane_policy\":\"dispatch-lanes.io-bound\""));
    assert!(json.contains("\"bridge_entry\":\"nuis.network.bridge.dispatch.v1\""));
    assert!(json.contains("\"transport_model\":\"client-session\""));
    assert!(json.contains("\"phase_order\":[\"bind\",\"submit\",\"wait\",\"finalize\"]"));
    assert!(json.contains("\"bridge_plan_begin\":true"));
    assert!(json.contains("\"bridge_plan_end\":true"));
}

#[test]
fn domain_registry_json_includes_effective_build_contract() {
    let registration = registry::load_registered_domains(Path::new(NUSTAR_REGISTRY_ROOT))
        .expect("expected registered domains")
        .into_iter()
        .find(|item| item.domain_family == "network")
        .expect("expected network registration");
    let manifest = registry::load_manifest_for_domain(Path::new(NUSTAR_REGISTRY_ROOT), "network")
        .expect("expected network manifest");
    let json = domain_registry_json(&registration, &manifest);

    assert!(json.contains("\"registration\":{"));
    assert!(json.contains("\"build_contract\":{"));
    assert!(json.contains("\"backend\":{"));
    assert!(json.contains("\"host_bridge\":{"));
    assert!(json.contains("\"scheduler_binding\":\"network-poll-bridge\""));
    assert!(json.contains("\"host_ffi_surface\":\"socket,urlsession\""));
}

#[test]
fn domain_build_unit_contract_json_includes_effective_build_contract() {
    let unit = aot::BuildManifestDomainBuildUnit {
        package_id: "official.network".to_owned(),
        domain_family: "network".to_owned(),
        abi: Some("network.socket.macos.arm64.v1".to_owned()),
        machine_arch: Some("arm64".to_owned()),
        machine_os: Some("darwin".to_owned()),
        backend_family: Some("urlsession".to_owned()),
        vendor: Some("apple".to_owned()),
        device_class: Some("socket-io".to_owned()),
        target_device: Some("urlsession-stack".to_owned()),
        ir_format: Some("host-ffi-plan".to_owned()),
        dispatch_abi: Some("nuis-host-call".to_owned()),
        backend_priority: Some(700),
        verification: Some("contract-only".to_owned()),
        selected_lowering_target: Some("urlsession.socket-io".to_owned()),
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
        contract_family: "nustar.network".to_owned(),
        packaging_role: "domain-sidecar".to_owned(),
    };
    let json = domain_build_unit_contract_json(&unit);

    assert!(json.contains("\"package_id\":\"official.network\""));
    assert!(json.contains("\"domain_family\":\"network\""));
    assert!(json.contains("\"build_contract\":{"));
    assert!(json.contains("\"lane_policy\":\"dispatch-lanes.io-bound\""));
    assert!(json.contains("\"bridge_entry\":\"nuis.network.bridge.dispatch.v1\""));
}

#[test]
fn domain_build_contract_drift_check_accepts_current_registry_alignment() {
    let unit = aot::BuildManifestDomainBuildUnit {
        package_id: "official.network".to_owned(),
        domain_family: "network".to_owned(),
        abi: Some("network.socket.macos.arm64.v1".to_owned()),
        machine_arch: Some("arm64".to_owned()),
        machine_os: Some("darwin".to_owned()),
        backend_family: Some("urlsession".to_owned()),
        vendor: Some("apple".to_owned()),
        device_class: Some("socket-io".to_owned()),
        target_device: Some("urlsession-stack".to_owned()),
        ir_format: Some("host-ffi-plan".to_owned()),
        dispatch_abi: Some("nuis-host-call".to_owned()),
        backend_priority: Some(700),
        verification: Some("contract-only".to_owned()),
        selected_lowering_target: Some("urlsession.socket-io".to_owned()),
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
        contract_family: "nustar.network".to_owned(),
        packaging_role: "domain-sidecar".to_owned(),
    };
    let drift = evaluate_domain_build_contract_drift(&unit);

    assert!(drift.consistent);
    assert!(drift.issues.is_empty());
}

#[test]
fn domain_build_contract_drift_check_reports_registry_mismatch() {
    let unit = aot::BuildManifestDomainBuildUnit {
        package_id: "official.network".to_owned(),
        domain_family: "network".to_owned(),
        abi: Some("network.socket.macos.arm64.v1".to_owned()),
        machine_arch: Some("arm64".to_owned()),
        machine_os: Some("darwin".to_owned()),
        backend_family: Some("imaginary-backend".to_owned()),
        vendor: None,
        device_class: None,
        target_device: None,
        ir_format: None,
        dispatch_abi: None,
        backend_priority: None,
        verification: None,
        selected_lowering_target: Some("imaginary-target".to_owned()),
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
        contract_family: "nustar.network.drifted".to_owned(),
        packaging_role: "domain-sidecar".to_owned(),
    };
    let drift = evaluate_domain_build_contract_drift(&unit);

    assert!(!drift.consistent);
    assert!(drift
        .issues
        .iter()
        .any(|issue| issue.contains("contract_family")));
    assert!(drift
        .issues
        .iter()
        .any(|issue| issue.contains("selected_lowering_target")));
    assert!(drift
        .issues
        .iter()
        .any(|issue| issue.contains("backend_family")));
}

#[test]
fn domain_build_unit_verification_verdict_marks_cpu_unit_consistent() {
    let unit = aot::BuildManifestDomainBuildUnit {
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
        contract_family: "nustar.cpu".to_owned(),
        packaging_role: "host-binary".to_owned(),
    };
    let report = aot::BuildManifestVerifyReport {
        schema: "nuis-build-manifest-v1".to_owned(),
        input: "main.ns".to_owned(),
        output_dir: "out".to_owned(),
        packaging_mode: "native-cpu-llvm".to_owned(),
        envelope_path: "out/nuis.executable.envelope.toml".to_owned(),
        envelope_schema: "nuis-executable-envelope-v1".to_owned(),
        envelope_package_count: 1,
        artifact_path: "out/nuis.compiled.artifact".to_owned(),
        artifact_schema: "nuis-compiled-artifact-v1".to_owned(),
        artifact_binary_name: "demo".to_owned(),
        artifact_binary_bytes: 1,
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
        execution_contracts_checked: 1,
        domain_build_unit_count: 1,
        heterogeneous_domain_count: 0,
        domain_payload_blobs_checked: 0,
        domain_payload_blob_sections_checked: 0,
        domain_payload_contract_sections_checked: 0,
        domain_payload_lowering_plans_checked: 0,
        domain_payload_backend_stubs_checked: 0,
        domain_payload_bridge_plans_checked: 0,
        domain_bridge_stubs_checked: 0,
        domain_build_units: vec![unit.clone()],
        cpu_target_abi: "cpu.arm64.apple_aapcs64".to_owned(),
        cpu_target_machine_arch: "arm64".to_owned(),
        cpu_target_machine_os: "darwin".to_owned(),
        cpu_target_object_format: "mach-o".to_owned(),
        cpu_target_calling_abi: "aapcs64-darwin".to_owned(),
        cpu_target_clang: "aarch64-apple-darwin".to_owned(),
        cpu_target_cross: false,
        loaded_nustar: vec!["official.cpu".to_owned()],
        compile_cache_status: None,
        compile_cache_key: None,
        compile_cache_root: None,
        doc_index_path: None,
        doc_index_module_count: 0,
        doc_index_documented_item_count: 0,
        doc_index_checked: 0,
        project_text_handle_rewrite_helper_hits: 0,
        project_text_handle_rewrite_local_hits: 0,
        artifact_provider_metadata: vec![],
        project_plan_index: None,
        project_docs_index: None,
        project_docs_module_count: 0,
        project_docs_documented_module_count: 0,
        project_docs_documented_item_count: 0,
        project_imports_index: None,
        project_imports_library_count: 0,
        project_imports_visible_library_count: 0,
        project_imports_visible_module_count: 0,
        project_imports_documented_visible_module_count: 0,
        project_imports_documented_visible_item_count: 0,
        project_galaxy_index: None,
        project_galaxy_count: 0,
        project_documented_galaxy_count: 0,
        project_documented_galaxy_library_module_count: 0,
        project_documented_galaxy_item_count: 0,
        project_packet_index: None,
        project_host_ffi_index: None,
        bridge_registry_path: None,
        bridge_registry_units: 0,
        bridge_registry_checked: 0,
        bridge_registry_entries_checked: 0,
        host_bridge_plan_index_path: None,
        host_bridge_plan_units: 0,
        host_bridge_plan_checked: 0,
        host_bridge_plan_entries_checked: 0,
        lowering_plan_index_path: None,
        lowering_plan_units: 0,
        lowering_plan_index_checked: 0,
        lowering_plan_entries_checked: 0,
        clock_protocol_path: None,
        clock_protocol_domains: 1,
        clock_protocol_checked: 1,
        clock_protocol_entries_checked: 1,
        hetero_calculate_plan_path: None,
        hetero_calculate_plan_units: 0,
        hetero_calculate_plan_checked: 0,
        hetero_calculate_plan_entries_checked: 0,
        artifacts_checked: 0,
        project_metadata_checked: 0,
    };
    let verdict = domain_build_unit_verification_verdict(&unit, &report);

    assert_eq!(verdict.kind, "host");
    assert!(verdict.payload_blob_ok);
    assert!(verdict.bridge_registry_ok);
    assert!(verdict.host_bridge_plan_ok);
    assert!(verdict.registry_alignment_ok);
    assert!(verdict.failure_reasons.is_empty());
    assert!(verdict.consistent);
}

#[test]
fn verify_build_manifest_json_includes_domain_build_contracts() {
    let project_name = "verify_build_manifest_contract_json";
    let project_root = write_temp_project_fixture(
        project_name,
        r#"
name = "verify_build_manifest_contract_json"
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
    let output_dir = temp_dir("verify_build_manifest_contract_json_outputs");

    run(CommandKind::Compile {
        input: project_root,
        output_dir: output_dir.clone(),
        verbose_cache: false,
        cpu_abi: None,
        target: None,
        packaging_mode: None,
    })
    .unwrap();

    let manifest_path = output_dir.join("nuis.build.manifest.toml");
    let report = aot::verify_build_manifest(&manifest_path).unwrap();
    let json = verify_build_manifest_json(&manifest_path, &report);

    assert!(json.contains("\"domain_build_units\":["));
    assert!(json.contains("\"domain_build_contracts\":["));
    assert!(json.contains("\"domain_payload_blobs_checked\":0"));
    assert!(json.contains("\"domain_payload_blob_sections_checked\":0"));
    assert!(json.contains("\"domain_payload_lowering_plans_checked\":0"));
    assert!(json.contains("\"domain_payload_backend_stubs_checked\":0"));
    assert!(json.contains("\"domain_payload_bridge_plans_checked\":0"));
    assert!(json.contains("\"domain_bridge_stubs_checked\":0"));
    assert!(json.contains("\"bridge_registry_entries_checked\":0"));
    assert!(json.contains("\"host_bridge_plan_entries_checked\":0"));
    assert!(json.contains("\"doc_index_path\":"));
    assert!(json.contains("\"doc_index_module_count\":1"));
    assert!(json.contains("\"doc_index_documented_item_count\":0"));
    assert!(json.contains("\"doc_index_checked\":1"));
    assert!(json.contains("\"domain_build_contract_drift_checked\":"));
    assert!(json.contains("\"domain_build_contract_drift_mismatches\":0"));
    assert!(json.contains("\"domain_build_contracts_consistent\":true"));
    assert!(json.contains("\"domain_build_contract_drift\":["));
    assert!(json.contains("\"domain_build_unit_verdicts\":["));
    assert!(json.contains("\"domain_build_verification_summary\":{"));
    assert!(json.contains("\"all_units_consistent\":true"));
    assert!(json.contains("\"failing_units\":[]"));
    assert!(json.contains("\"kind\":\"host\""));
    assert!(json.contains("\"failure_reasons\":[]"));
    assert!(json.contains("\"registry_alignment_ok\":true"));
    assert!(json.contains("\"consistent\":true"));
    assert!(json.contains("\"package_id\":\"official.cpu\""));
    assert!(json.contains("\"build_contract\":{"));
}

#[test]
fn inspect_artifact_json_includes_domain_build_contracts_when_manifest_is_available() {
    let project_name = "inspect_artifact_contract_json";
    let project_root = write_temp_project_fixture(
        project_name,
        r#"
name = "inspect_artifact_contract_json"
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
    let output_dir = temp_dir("inspect_artifact_contract_json_outputs");

    run(CommandKind::Compile {
        input: project_root,
        output_dir: output_dir.clone(),
        verbose_cache: false,
        cpu_abi: None,
        target: None,
        packaging_mode: None,
    })
    .unwrap();

    let manifest_path = output_dir.join("nuis.build.manifest.toml");
    let artifact = load_nuis_compiled_artifact(&manifest_path).unwrap();
    let report = aot::verify_build_manifest(&manifest_path).unwrap();
    let container = inspect_artifact_container_for_input(&manifest_path, Some(&report))
        .unwrap()
        .unwrap();
    let json = inspect_artifact_json(&manifest_path, &artifact, Some(&container), Some(&report));

    assert!(json.contains("\"domain_build_unit_count\":"));
    assert!(json.contains("\"domain_build_units\":["));
    assert!(json.contains("\"domain_build_contracts\":["));
    assert!(json.contains("\"domain_payload_blobs_checked\":0"));
    assert!(json.contains("\"domain_payload_blob_sections_checked\":0"));
    assert!(json.contains("\"domain_payload_lowering_plans_checked\":0"));
    assert!(json.contains("\"domain_payload_backend_stubs_checked\":0"));
    assert!(json.contains("\"domain_payload_bridge_plans_checked\":0"));
    assert!(json.contains("\"domain_bridge_stubs_checked\":0"));
    assert!(json.contains("\"bridge_registry_entries_checked\":0"));
    assert!(json.contains("\"host_bridge_plan_entries_checked\":0"));
    assert!(json.contains("\"domain_build_contract_drift_checked\":"));
    assert!(json.contains("\"domain_build_contract_drift_mismatches\":0"));
    assert!(json.contains("\"domain_build_contracts_consistent\":true"));
    assert!(json.contains("\"domain_build_contract_drift\":["));
    assert!(json.contains("\"domain_build_unit_verdicts\":["));
    assert!(json.contains("\"domain_build_verification_summary\":{"));
    assert!(json.contains("\"all_units_consistent\":true"));
    assert!(json.contains("\"failing_units\":[]"));
    assert!(json.contains("\"kind\":\"host\""));
    assert!(json.contains("\"failure_reasons\":[]"));
    assert!(json.contains("\"registry_alignment_ok\":true"));
    assert!(json.contains("\"consistent\":true"));
    assert!(json.contains("\"package_id\":\"official.cpu\""));
    assert!(json.contains("\"link_plan\":{"));
    assert!(json.contains("\"final_stage_driver\":\"clang\""));
    assert!(json.contains("\"final_stage_kind\":\"host-native-link\""));
    assert!(json.contains("\"final_stage_link_mode\":\"host-toolchain-finalize\""));
    assert!(json.contains("\"artifact_container_kind\":\"compiled-artifact-section-table-v2\""));
    assert!(json.contains("\"artifact_container_version\":2"));
    assert!(json.contains("\"artifact_section_table_valid\":true"));
    assert!(json.contains("\"lowering_unit_count\":1"));
    assert!(json.contains("\"lowering_domain_families\":[\"cpu\"]"));
    assert!(json.contains("\"lowering_targets\":[\"llvm\"]"));
    assert!(json.contains("\"lowering_units\":[{"));
    assert!(json.contains("\"link_plan\":{\"schema\":\"nuis-link-plan-v1\""));
    assert!(json.contains("\"artifact_section_count\":6"));
}

#[test]
fn inspect_artifact_json_accepts_section_table_artifact_container() {
    let project_name = "inspect_artifact_v2_section_table_json";
    let project_root = write_temp_project_fixture(
        project_name,
        r#"
name = "inspect_artifact_v2_section_table_json"
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
    let output_dir = temp_dir("inspect_artifact_v2_section_table_outputs");

    run(CommandKind::Compile {
        input: project_root,
        output_dir: output_dir.clone(),
        verbose_cache: false,
        cpu_abi: None,
        target: None,
        packaging_mode: None,
    })
    .unwrap();

    let manifest_path = output_dir.join("nuis.build.manifest.toml");
    let artifact = load_nuis_compiled_artifact(&manifest_path).unwrap();
    let v2_path = output_dir.join("nuis.compiled.v2.artifact");
    let v2_bytes = aot::encode_nuis_compiled_artifact_section_table_binary(&artifact).unwrap();
    std::fs::write(&v2_path, v2_bytes).unwrap();

    let decoded = load_nuis_compiled_artifact(&v2_path).unwrap();
    let container = inspect_artifact_container_for_input(&v2_path, None)
        .unwrap()
        .unwrap();
    let json = inspect_artifact_json(&v2_path, &decoded, Some(&container), None);
    let verify_report = aot::verify_nuis_compiled_artifact(&v2_path).unwrap();
    let verify_json = verify_artifact_json(&v2_path, &verify_report);

    assert_eq!(decoded.binary_name, artifact.binary_name);
    assert!(json.contains("\"artifact_container_kind\":\"compiled-artifact-section-table-v2\""));
    assert!(json.contains("\"artifact_container_version\":2"));
    assert!(json.contains("\"artifact_section_count\":6"));
    assert!(json.contains("\"metadata_toml\""));
    assert!(json.contains("\"envelope_binary\""));
    assert!(json.contains("\"lifecycle_toml\""));
    assert!(json.contains("\"build_manifest_toml\""));
    assert!(json.contains("\"lowering_index_toml\""));
    assert!(json.contains("\"host_binary\""));
    assert!(json.contains("\"artifact_section_table_valid\":true"));
    assert!(json.contains("\"lowering_unit_count\":1"));
    assert!(json.contains("\"lowering_domain_families\":[\"cpu\"]"));
    assert!(json.contains("\"lowering_targets\":[\"llvm\"]"));
    assert!(json.contains("\"lowering_units\":[{"));
    assert!(json.contains("\"package_id\":\"official.cpu\""));
    assert!(json.contains("\"domain_family\":\"cpu\""));
    assert!(json.contains("\"selected_lowering_target\":\"llvm\""));
    assert!(
        verify_json.contains("\"artifact_container_kind\":\"compiled-artifact-section-table-v2\"")
    );
    assert!(verify_json.contains("\"artifact_container_version\":2"));
    assert!(verify_json.contains("\"artifact_section_count\":6"));
    assert!(verify_json.contains("\"lowering_unit_count\":1"));
    assert!(verify_json.contains("\"lowering_targets\":[\"llvm\"]"));
    assert!(verify_json.contains("\"lowering_units\":[{"));
    assert!(verify_json.contains("\"artifact_roundtrip_verified\":true"));
}

#[test]
fn artifact_report_json_includes_top_level_verification_summary() {
    let project_name = "artifact_report_contract_summary_json";
    let project_root = write_temp_project_fixture(
        project_name,
        r#"
name = "artifact_report_contract_summary_json"
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
    let output_dir = temp_dir("artifact_report_contract_summary_json_outputs");

    run(CommandKind::Compile {
        input: project_root,
        output_dir: output_dir.clone(),
        verbose_cache: false,
        cpu_abi: None,
        target: None,
        packaging_mode: None,
    })
    .unwrap();

    let manifest_path = output_dir.join("nuis.build.manifest.toml");
    let artifact = load_nuis_compiled_artifact(&manifest_path).unwrap();
    let artifact_verify =
        aot::verify_nuis_compiled_artifact(output_dir.join("nuis.compiled.artifact").as_path())
            .unwrap();
    let manifest_verify = aot::verify_build_manifest(&manifest_path).unwrap();
    let json = artifact_report_json(
        &manifest_path,
        &artifact,
        output_dir.join("nuis.compiled.artifact").as_path(),
        &artifact_verify,
        &manifest_path,
        &manifest_verify,
        false,
    );

    assert!(json.contains("\"domain_build_verification_summary\":{"));
    assert!(json.contains("\"all_units_consistent\":true"));
    assert!(json.contains("\"host_units_checked\":1"));
    assert!(json.contains("\"hetero_units_checked\":0"));
    assert!(json.contains("\"failing_units\":[]"));
    assert!(json.contains("\"execution_inspect\":{"));
    assert!(json.contains("\"kind\":\"nuis_execution_inspect\""));
    assert!(json.contains("\"heterogeneous_execution_domains\":0"));
    assert!(json.contains("\"execution_inspect\":{\"kind\":\"nuis_execution_inspect\""));
    assert!(json.contains("\"issues\":[]"));
    assert!(json.contains("\"project_metadata\":{"));
    assert!(json.contains("\"kind\":\"nuis_project_metadata\""));
    assert!(json.contains("\"source_kind\":\"build-manifest\""));
    assert!(json.contains("\"sections\":[]"));
    assert!(json.contains("\"doc_index\":{"));
    assert!(json.contains("\"kind\":\"nuis_doc_index\""));
    assert!(json.contains("\"module_count\":1"));
    assert!(json.contains("\"link_plan\":{"));
    assert!(json.contains("\"host_ffi\":{"));
    assert!(json.contains("\"symbol_count\":0"));
    assert!(json.contains("\"policy_count\":0"));
    assert!(json.contains("\"policy\":\"signature-whitelist-required\""));
    assert!(json.contains("\"final_stage_driver\":\"clang\""));
}

#[test]
fn benchmark_report_file_tooling_outputs_support_inspect_and_verify_json() {
    let project_root = PathBuf::from("../../examples/projects/tooling/benchmark_report_file_demo");
    let output_dir = temp_dir("benchmark_report_file_artifact_json_outputs");

    run(CommandKind::Compile {
        input: project_root,
        output_dir: output_dir.clone(),
        verbose_cache: false,
        cpu_abi: None,
        target: None,
        packaging_mode: None,
    })
    .unwrap();

    let manifest_path = output_dir.join("nuis.build.manifest.toml");
    let artifact_path = output_dir.join("nuis.compiled.artifact");
    let artifact = load_nuis_compiled_artifact(&manifest_path).unwrap();
    let manifest_verify = aot::verify_build_manifest(&manifest_path).unwrap();
    let artifact_verify = aot::verify_nuis_compiled_artifact(&artifact_path).unwrap();

    let container = inspect_artifact_container_for_input(&manifest_path, Some(&manifest_verify))
        .unwrap()
        .unwrap();
    let inspect_json = inspect_artifact_json(
        &manifest_path,
        &artifact,
        Some(&container),
        Some(&manifest_verify),
    );
    assert!(inspect_json.contains("\"kind\":\"nuis_artifact_inspect\""));
    assert!(inspect_json.contains("\"binary_name\":\"benchmark_report_file_demo\""));
    assert!(inspect_json.contains("\"packaging_mode\":\"native-cpu-llvm\""));
    assert!(
        inspect_json.contains("\"artifact_container_kind\":\"compiled-artifact-section-table-v2\"")
    );
    assert!(inspect_json.contains("\"domain_build_units\":["));
    assert!(inspect_json.contains("\"domain_build_contracts\":["));
    assert!(inspect_json.contains("\"link_plan\":{"));
    assert!(inspect_json.contains("\"artifact_container_version\":2"));
    assert!(inspect_json.contains("\"artifact_section_count\":6"));
    assert!(inspect_json.contains("\"lowering_unit_count\":1"));
    assert!(inspect_json.contains("\"final_stage_driver\":\"clang\""));

    let verify_manifest_json = verify_build_manifest_json(&manifest_path, &manifest_verify);
    assert!(verify_manifest_json.contains("\"kind\":\"nuis_build_manifest_verify\""));
    assert!(
        verify_manifest_json.contains("\"artifact_binary_name\":\"benchmark_report_file_demo\"")
    );
    assert!(verify_manifest_json.contains("\"project_metadata_checked\":"));
    assert!(verify_manifest_json.contains("\"project_host_ffi_index\":"));
    assert!(verify_manifest_json.contains("\"project_host_ffi_symbol_count\":"));
    assert!(verify_manifest_json.contains("\"project_host_ffi_policy_count\":"));
    assert!(verify_manifest_json.contains("\"domain_build_verification_summary\":{"));
    assert!(verify_manifest_json.contains("\"all_units_consistent\":true"));

    let verify_artifact_json_text = verify_artifact_json(&artifact_path, &artifact_verify);
    assert!(verify_artifact_json_text.contains("\"kind\":\"nuis_artifact_verify\""));
    assert!(verify_artifact_json_text.contains("\"binary_name\":\"benchmark_report_file_demo\""));
    assert!(verify_artifact_json_text
        .contains("\"artifact_container_kind\":\"compiled-artifact-section-table-v2\""));
    assert!(verify_artifact_json_text.contains("\"artifact_container_version\":2"));
    assert!(verify_artifact_json_text.contains("\"artifact_section_count\":6"));
    assert!(verify_artifact_json_text.contains("\"lowering_unit_count\":1"));
    assert!(verify_artifact_json_text.contains("\"artifact_roundtrip_verified\":true"));
    assert!(verify_artifact_json_text.contains("\"lifecycle_contract_consistent\":true"));

    let artifact_report = artifact_report_json(
        &manifest_path,
        &artifact,
        &artifact_path,
        &artifact_verify,
        &manifest_path,
        &manifest_verify,
        false,
    );
    assert!(artifact_report.contains("\"kind\":\"nuis_artifact_report\""));
    assert!(artifact_report.contains("\"manifest_verify_reconstructed\":false"));
    assert!(artifact_report.contains("\"execution_inspect\":{"));
    assert!(artifact_report.contains("\"kind\":\"nuis_execution_inspect\""));
    assert!(artifact_report.contains("\"sections\":[]"));
    assert!(artifact_report.contains("\"project_metadata\":{"));
    assert!(artifact_report.contains("\"kind\":\"nuis_project_metadata\""));
    assert!(artifact_report.contains("\"doc_index\":{"));
    assert!(artifact_report.contains("\"kind\":\"nuis_doc_index\""));
    assert!(artifact_report.contains("\"artifact_inspect\":{"));
    assert!(artifact_report.contains("\"artifact_verify\":{"));
    assert!(artifact_report.contains("\"manifest_verify\":{"));
    assert!(artifact_report.contains("\"binary_name\":\"benchmark_report_file_demo\""));
    assert!(artifact_report.contains("\"all_units_consistent\":true"));
}

#[path = "lib_tests_artifact.rs"]
mod artifact_tests;
#[path = "lib_tests_execution.rs"]
mod execution_tests;
