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
    assert!(json.contains("\"imports_library_count\":15"));
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
    let summary = ProjectMetadataSummary {
        source_kind: "build-manifest".to_owned(),
        project_name: Some("demo".to_owned()),
        project_root: Some("/tmp/demo".to_owned()),
        manifest_path: Some("/tmp/demo/nuis.toml".to_owned()),
        build_manifest_path: Some("/tmp/demo/build/nuis.build.manifest.toml".to_owned()),
        artifact_path: Some("/tmp/demo/build/nuis.compiled.artifact".to_owned()),
        docs_index_path: Some("/tmp/demo/build/nuis.project.docs.txt".to_owned()),
        docs_module_count: 4,
        docs_documented_module_count: 3,
        docs_documented_item_count: 12,
        imports_index_path: Some("/tmp/demo/build/nuis.project.imports.txt".to_owned()),
        imports_library_count: 6,
        imports_visible_library_count: 5,
        imports_visible_module_count: 7,
        imports_documented_visible_module_count: 4,
        imports_documented_visible_item_count: 10,
        galaxy_index_path: Some("/tmp/demo/build/nuis.project.galaxy.txt".to_owned()),
        galaxy_count: 3,
        documented_galaxy_count: 2,
        documented_galaxy_library_module_count: 5,
        documented_galaxy_item_count: 10,
        host_ffi_index_path: Some("/tmp/demo/build/nuis.project.host_ffi.txt".to_owned()),
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
    assert!(paths.contains("project_root=/tmp/demo"));
    assert!(paths.contains("manifest_path=/tmp/demo/nuis.toml"));
    assert!(paths.contains("build_manifest_path=/tmp/demo/build/nuis.build.manifest.toml"));
    assert!(paths.contains("artifact_path=/tmp/demo/build/nuis.compiled.artifact"));
    assert!(paths.contains("docs_index_path=/tmp/demo/build/nuis.project.docs.txt"));
    assert!(paths.contains("imports_index_path=/tmp/demo/build/nuis.project.imports.txt"));
    assert!(paths.contains("galaxy_index_path=/tmp/demo/build/nuis.project.galaxy.txt"));
    assert!(paths.contains("host_ffi_index_path=/tmp/demo/build/nuis.project.host_ffi.txt"));
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
        },
        bridge_registry_path: None,
        host_bridge_plan_index_path: None,
        lowering_plan_index_path: None,
        lowering_plan_index_source: "unavailable".to_owned(),
        host_ffi: linker::LinkPlanHostFfiFootprint {
            index_path: None,
            symbol_count: 0,
            policy_count: 0,
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

    assert_eq!(lines.len(), 7);
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
    assert!(lines[3].contains("final_stage=host-native-link"));
    assert!(lines[3].contains("driver=clang"));
    assert!(lines[4].contains("summary_execution: hetero_domains=1"));
    assert!(lines[4].contains("network(target=urlsession.socket-io phases=4 events=4)"));
    assert_eq!(lines[5], "summary_execution_issues: <none>");
    assert_eq!(
        lines[6],
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

#[test]
fn execution_inspect_issues_flag_missing_target_and_phase_mismatch() {
    let overview = ExecutionInspectOverview {
        heterogeneous_domains: 2,
        domains: vec![
            ExecutionInspectDomainOverview {
                domain_family: "network".to_owned(),
                selected_lowering_target: None,
                phase_count: 0,
                event_count: 0,
                resource_keys: vec![],
                output_handles: vec![],
            },
            ExecutionInspectDomainOverview {
                domain_family: "shader".to_owned(),
                selected_lowering_target: Some("metal.apple-silicon-gpu".to_owned()),
                phase_count: 4,
                event_count: 3,
                resource_keys: vec!["shader_buffer".to_owned()],
                output_handles: vec![],
            },
        ],
    };

    let issues = execution_inspect_issues(&overview);

    assert_eq!(
        issues,
        vec![
            ExecutionInspectIssue {
                domain_family: "network".to_owned(),
                issue: "missing_target".to_owned(),
            },
            ExecutionInspectIssue {
                domain_family: "network".to_owned(),
                issue: "zero_phases".to_owned(),
            },
            ExecutionInspectIssue {
                domain_family: "network".to_owned(),
                issue: "missing_network_request_packet".to_owned(),
            },
            ExecutionInspectIssue {
                domain_family: "network".to_owned(),
                issue: "missing_network_active_response".to_owned(),
            },
            ExecutionInspectIssue {
                domain_family: "network".to_owned(),
                issue: "missing_network_response_handle".to_owned(),
            },
            ExecutionInspectIssue {
                domain_family: "shader".to_owned(),
                issue: "phase_event_mismatch(4->3)".to_owned(),
            },
            ExecutionInspectIssue {
                domain_family: "shader".to_owned(),
                issue: "missing_shader_frame_target".to_owned(),
            },
            ExecutionInspectIssue {
                domain_family: "shader".to_owned(),
                issue: "missing_shader_draw_handle".to_owned(),
            },
        ]
    );
}

#[test]
fn compile_command_writes_end_to_end_project_outputs() {
    let project_name = "compile_command_smoke";
    let project_root = write_temp_project_fixture(
        project_name,
        r#"
name = "compile_command_smoke"
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
    let output_dir = temp_dir("compile_command_outputs");
    let output_stem = project_name.to_owned();

    run(CommandKind::Compile {
        input: project_root.clone(),
        output_dir: output_dir.clone(),
        verbose_cache: false,
        cpu_abi: None,
        target: None,
        packaging_mode: None,
    })
    .unwrap();

    for path in [
        output_dir.join(format!("{output_stem}.ast.txt")),
        output_dir.join(format!("{output_stem}.nir.txt")),
        output_dir.join(format!("{output_stem}.yir")),
        output_dir.join(format!("{output_stem}.ll")),
        output_dir.join(&output_stem),
        output_dir.join("nuis.doc-index.json"),
        output_dir.join("nuis.build.manifest.toml"),
        output_dir.join("nuis.executable.envelope.toml"),
        output_dir.join("nuis.compiled.artifact"),
        output_dir.join("nuis.project.toml"),
        output_dir.join("nuis.project.plan.txt"),
        output_dir.join("nuis.project.organization.txt"),
        output_dir.join("nuis.project.exchange.txt"),
        output_dir.join("nuis.project.modules.txt"),
        output_dir.join("nuis.project.docs.txt"),
        output_dir.join("nuis.project.imports.txt"),
        output_dir.join("nuis.project.galaxy.txt"),
        output_dir.join("nuis.project.links.txt"),
        output_dir.join("nuis.project.packet.txt"),
        output_dir.join("nuis.project.host_ffi.txt"),
        output_dir.join("nuis.project.abi.txt"),
    ] {
        assert!(path.exists(), "expected output `{}`", path.display());
    }

    let manifest_path = output_dir.join("nuis.build.manifest.toml");
    let manifest_text = fs::read_to_string(&manifest_path).unwrap();
    assert!(manifest_text.contains("manifest_schema = \"nuis-build-manifest-v1\""));
    assert!(manifest_text.contains("packaging_mode = \"native-cpu-llvm\""));
    assert!(manifest_text.contains("loaded_nustar = [\"official.cpu\"]"));
    assert!(manifest_text.contains("doc_index_path = "));
    assert!(manifest_text.contains("doc_index_module_count = 1"));
    assert!(manifest_text.contains("doc_index_documented_item_count = 0"));
    assert!(manifest_text.contains("[[domain_build_unit]]"));
    assert!(manifest_text.contains(&format!("name = \"{project_name}\"")));
    assert!(manifest_text.contains("manifest_copy = "));
    assert!(manifest_text.contains("plan_index = "));
    assert!(manifest_text.contains("organization_index = "));
    assert!(manifest_text.contains("exchange_index = "));
    assert!(manifest_text.contains("modules_index = "));
    assert!(manifest_text.contains("docs_index = "));
    assert!(manifest_text.contains("docs_module_count = 1"));
    assert!(manifest_text.contains("docs_documented_module_count = 0"));
    assert!(manifest_text.contains("docs_documented_item_count = 0"));
    assert!(manifest_text.contains("imports_index = "));
    assert!(manifest_text.contains("imports_library_count = 0"));
    assert!(manifest_text.contains("imports_visible_library_count = 0"));
    assert!(manifest_text.contains("imports_visible_module_count = 1"));
    assert!(manifest_text.contains("imports_documented_visible_module_count = 0"));
    assert!(manifest_text.contains("imports_documented_visible_item_count = 0"));
    assert!(manifest_text.contains("galaxy_index = "));
    assert!(manifest_text.contains("galaxy_count = 0"));
    assert!(manifest_text.contains("documented_galaxy_count = 0"));
    assert!(manifest_text.contains("documented_galaxy_library_module_count = 0"));
    assert!(manifest_text.contains("documented_galaxy_item_count = 0"));
    assert!(manifest_text.contains("links_index = "));
    assert!(manifest_text.contains("packet_index = "));
    assert!(manifest_text.contains("host_ffi_index = "));
    assert!(manifest_text.contains("abi_index = "));

    let manifest_report = aot::verify_build_manifest(&manifest_path).unwrap();
    assert!(manifest_report
        .doc_index_path
        .as_deref()
        .is_some_and(|path| path.ends_with("nuis.doc-index.json")));
    assert_eq!(manifest_report.doc_index_module_count, 1);
    assert_eq!(manifest_report.doc_index_documented_item_count, 0);
    assert_eq!(manifest_report.doc_index_checked, 1);
    assert_eq!(manifest_report.project_docs_module_count, 1);
    assert_eq!(manifest_report.project_docs_documented_module_count, 0);
    assert_eq!(manifest_report.project_docs_documented_item_count, 0);
    assert_eq!(manifest_report.project_imports_library_count, 0);
    assert_eq!(manifest_report.project_imports_visible_library_count, 0);
    assert_eq!(manifest_report.project_imports_visible_module_count, 1);
    assert_eq!(
        manifest_report.project_imports_documented_visible_module_count,
        0
    );
    assert_eq!(
        manifest_report.project_imports_documented_visible_item_count,
        0
    );
    assert_eq!(manifest_report.project_galaxy_count, 0);
    assert_eq!(manifest_report.project_documented_galaxy_count, 0);
    assert_eq!(
        manifest_report.project_documented_galaxy_library_module_count,
        0
    );
    assert_eq!(manifest_report.project_documented_galaxy_item_count, 0);
    assert_eq!(
        manifest_report.envelope_schema,
        "nuis-executable-envelope-v1"
    );
    assert_eq!(manifest_report.artifact_schema, "nuis-compiled-artifact-v1");
    assert_eq!(manifest_report.artifact_binary_name, output_stem);
    assert!(Path::new(&manifest_report.envelope_path).exists());
    assert!(Path::new(&manifest_report.artifact_path).exists());
    assert!(manifest_report.project_metadata_checked >= 2);

    let artifact_report =
        aot::verify_nuis_compiled_artifact(output_dir.join("nuis.compiled.artifact").as_path())
            .unwrap();
    assert_eq!(artifact_report.binary_name, output_stem);
    assert_eq!(artifact_report.packaging_mode, "native-cpu-llvm");
    assert!(artifact_report.lifecycle_contract_consistent);
    assert!(artifact_report.artifact_roundtrip_verified);
}

#[test]
fn compile_command_reuses_cached_project_outputs_without_recompiling() {
    let project_name = "compile_command_cache_hit_smoke";
    let project_root = write_temp_project_fixture(
        project_name,
        r#"
name = "compile_command_cache_hit_smoke"
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
    let output_dir = temp_dir("compile_command_cache_hit_outputs");

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
    let first_report = aot::verify_build_manifest(&manifest_path).unwrap();
    assert_eq!(first_report.compile_cache_status.as_deref(), Some("miss"));
    assert_eq!(first_report.loaded_nustar, vec!["official.cpu".to_owned()]);

    run(CommandKind::Compile {
        input: project_root,
        output_dir: output_dir.clone(),
        verbose_cache: false,
        cpu_abi: None,
        target: None,
        packaging_mode: None,
    })
    .unwrap();

    let second_report = aot::verify_build_manifest(&manifest_path).unwrap();
    assert_eq!(second_report.compile_cache_status.as_deref(), Some("hit"));
    assert_eq!(second_report.loaded_nustar, vec!["official.cpu".to_owned()]);
    assert_eq!(second_report.packaging_mode, "native-cpu-llvm");
    assert!(Path::new(&second_report.artifact_path).exists());
}

#[test]
fn compile_command_writes_host_file_ffi_project_outputs() {
    let project_name = "compile_command_host_file_smoke";
    let project_root = write_temp_project_fixture(
        project_name,
        r#"
name = "compile_command_host_file_smoke"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
        .trim_start(),
        r#"
            mod cpu Main {
              extern "c" fn host_file_open(path_handle: i64, flags: i64) -> i64;
              extern "c" fn host_file_read(file_handle: i64, buffer_handle: i64, len: i64) -> i64;
              extern "c" fn host_file_write(file_handle: i64, text_handle: i64) -> i64;
              extern "c" fn host_file_close(file_handle: i64) -> i64;
              extern "c" fn host_path_copy(src_handle: i64, dst_handle: i64) -> i64;
              extern "c" fn host_fs_exists(path_handle: i64) -> i64;

              fn main() -> i64 {
                let handle: i64 = host_file_open(2103, 1);
                let backing: ref Buffer = alloc_buffer(8, 0);
                host_file_read(handle, host_buffer_handle(backing), 8);
                host_file_write(handle, 777);
                host_file_close(handle);
                host_path_copy(2103, 2109);
                host_fs_exists(2109);
                return 0;
              }
            }
            "#,
    );
    let output_dir = temp_dir("compile_command_host_file_outputs");
    let output_stem = project_name.to_owned();

    run(CommandKind::Compile {
        input: project_root.clone(),
        output_dir: output_dir.clone(),
        verbose_cache: false,
        cpu_abi: None,
        target: None,
        packaging_mode: None,
    })
    .unwrap();

    for path in [
        output_dir.join(format!("{output_stem}.ll")),
        output_dir.join(&output_stem),
        output_dir.join("nuis.build.manifest.toml"),
        output_dir.join("nuis.compiled.artifact"),
        output_dir.join("nuis.project.host_ffi.txt"),
    ] {
        assert!(path.exists(), "expected output `{}`", path.display());
    }

    let manifest_text = fs::read_to_string(output_dir.join("nuis.build.manifest.toml")).unwrap();
    assert!(manifest_text.contains("host_ffi_index = "));

    let host_ffi_text = fs::read_to_string(output_dir.join("nuis.project.host_ffi.txt")).unwrap();
    assert!(host_ffi_text.contains("host_file_open"));
    assert!(host_ffi_text.contains("host_file_read"));
    assert!(host_ffi_text.contains("host_file_write"));
    assert!(host_ffi_text.contains("host_file_close"));
    assert!(host_ffi_text.contains("host_path_copy"));
    assert!(host_ffi_text.contains("host_fs_exists"));

    let artifact_report =
        aot::verify_nuis_compiled_artifact(output_dir.join("nuis.compiled.artifact").as_path())
            .unwrap();
    assert_eq!(artifact_report.binary_name, output_stem);
    assert_eq!(artifact_report.packaging_mode, "native-cpu-llvm");
    assert!(artifact_report.lifecycle_contract_consistent);
    assert!(artifact_report.artifact_roundtrip_verified);

    let status = Command::new(output_dir.join(&output_stem))
        .status()
        .expect("expected compiled binary to launch");
    assert!(
        status.success(),
        "expected compiled binary to exit successfully"
    );
}

#[test]
fn compile_command_writes_benchmark_report_file_tooling_outputs() {
    let project_root = PathBuf::from("../../examples/projects/tooling/benchmark_report_file_demo");
    let output_dir = temp_dir("compile_command_benchmark_report_file_outputs");
    let output_stem = "benchmark_report_file_demo".to_owned();

    run(CommandKind::Compile {
        input: project_root,
        output_dir: output_dir.clone(),
        verbose_cache: false,
        cpu_abi: None,
        target: None,
        packaging_mode: None,
    })
    .unwrap();

    for path in [
        output_dir.join(format!("{output_stem}.ll")),
        output_dir.join(&output_stem),
        output_dir.join("nuis.build.manifest.toml"),
        output_dir.join("nuis.compiled.artifact"),
        output_dir.join("nuis.project.host_ffi.txt"),
        output_dir.join("nuis.project.plan.txt"),
    ] {
        assert!(path.exists(), "expected output `{}`", path.display());
    }

    let manifest_path = output_dir.join("nuis.build.manifest.toml");
    let manifest_text = fs::read_to_string(&manifest_path).unwrap();
    assert!(manifest_text.contains("name = \"benchmark_report_file_demo\""));
    assert!(manifest_text.contains("packaging_mode = \"native-cpu-llvm\""));
    assert!(manifest_text.contains("host_ffi_index = "));

    let manifest_report = aot::verify_build_manifest(&manifest_path).unwrap();
    assert_eq!(manifest_report.artifact_binary_name, output_stem);
    assert_eq!(manifest_report.artifact_schema, "nuis-compiled-artifact-v1");
    assert!(manifest_report.project_metadata_checked >= 6);

    let host_ffi_text = fs::read_to_string(output_dir.join("nuis.project.host_ffi.txt")).unwrap();
    assert!(host_ffi_text.contains("host_monotonic_time_ns"));
    assert!(host_ffi_text.contains("host_sleep_ns"));
    assert!(host_ffi_text.contains("host_file_open"));
    assert!(host_ffi_text.contains("host_file_write"));
    assert!(host_ffi_text.contains("host_file_close"));
    assert!(host_ffi_text.contains("host_temp_file_handle"));

    let artifact_report =
        aot::verify_nuis_compiled_artifact(output_dir.join("nuis.compiled.artifact").as_path())
            .unwrap();
    assert_eq!(artifact_report.binary_name, output_stem);
    assert_eq!(artifact_report.packaging_mode, "native-cpu-llvm");
    assert!(artifact_report.lifecycle_contract_consistent);
    assert!(artifact_report.artifact_roundtrip_verified);

    let status = Command::new(output_dir.join(&output_stem))
        .status()
        .expect("expected compiled benchmark report binary to launch");
    assert!(
        status.success(),
        "expected compiled benchmark report binary to exit successfully"
    );
}

#[test]
fn compile_command_writes_hetero_proxy_benchmark_host_ffi_policy_outputs() {
    let source =
        fs::read_to_string("../../examples/projects/tooling/hetero_proxy_benchmark_demo/main.ns")
            .unwrap();
    let project_root = write_temp_project_fixture(
        "hetero_proxy_benchmark_demo",
        r#"
name = "hetero_proxy_benchmark_demo"
entry = "main.ns"
modules = ["main.ns"]
galaxy = ["std=workspace"]
"#
        .trim_start(),
        &source,
    );
    let output_dir = temp_dir("compile_command_hetero_proxy_benchmark_outputs");
    let output_stem = "hetero_proxy_benchmark_demo".to_owned();

    run(CommandKind::Compile {
        input: project_root,
        output_dir: output_dir.clone(),
        verbose_cache: false,
        cpu_abi: None,
        target: None,
        packaging_mode: None,
    })
    .unwrap();

    for path in [
        output_dir.join(format!("{output_stem}.ll")),
        output_dir.join(&output_stem),
        output_dir.join("nuis.build.manifest.toml"),
        output_dir.join("nuis.compiled.artifact"),
        output_dir.join("nuis.project.host_ffi.txt"),
        output_dir.join("nuis.project.plan.txt"),
    ] {
        assert!(path.exists(), "expected output `{}`", path.display());
    }

    let manifest_path = output_dir.join("nuis.build.manifest.toml");
    let manifest_text = fs::read_to_string(&manifest_path).unwrap();
    assert!(manifest_text.contains("name = \"hetero_proxy_benchmark_demo\""));
    assert!(manifest_text.contains("packaging_mode = \"native-cpu-llvm\""));
    assert!(manifest_text.contains("host_ffi_index = "));

    let host_ffi_text = fs::read_to_string(output_dir.join("nuis.project.host_ffi.txt")).unwrap();
    assert!(host_ffi_text.contains("host_monotonic_time_ns"));
    assert!(host_ffi_text.contains("host_sleep_ns"));
    assert!(host_ffi_text.contains("signature_pattern=i64()"));
    assert!(host_ffi_text.contains("signature_pattern=i64(i64)"));
    assert!(host_ffi_text.contains("signature_hash=fnv1a64:"));
    assert!(host_ffi_text.contains("policy=signature-whitelist-required"));

    let manifest_report = aot::verify_build_manifest(&manifest_path).unwrap();
    assert_eq!(manifest_report.artifact_binary_name, output_stem);
    assert_eq!(manifest_report.artifact_schema, "nuis-compiled-artifact-v1");
    assert!(manifest_report.project_metadata_checked >= 6);
    let verify_manifest_json = verify_build_manifest_json(&manifest_path, &manifest_report);
    assert!(verify_manifest_json.contains("\"project_host_ffi_index\":"));
    assert!(verify_manifest_json.contains("\"project_host_ffi_symbol_count\":2"));
    assert!(verify_manifest_json.contains("\"project_host_ffi_policy_count\":2"));
    let artifact = load_nuis_compiled_artifact(&manifest_path).unwrap();
    let link_plan = linker::build_link_plan(&manifest_report, &artifact);
    let link_plan_json = linker::render_link_plan_json(&link_plan);
    assert!(link_plan_json.contains("\"host_ffi_symbol_count\":2"));
    assert!(link_plan_json.contains("\"host_ffi_policy_count\":2"));
    assert!(link_plan_json.contains("\"host_ffi_policy\":\"signature-whitelist-required\""));
    assert!(link_plan_json.contains("\"host_ffi_validation_checked\":2"));
    assert!(link_plan_json.contains("\"host_ffi_validation_valid\":true"));
    assert!(link_plan_json.contains("\"host_ffi_link_allowed\":true"));
    assert!(link_plan_json.contains("\"host_ffi_validation_issues\":[]"));
    assert!(link_plan_json.contains("\"host_ffi_validation_notes\":[]"));
    assert!(link_plan_json.contains("\"host_ffi_abi_groups\":[{"));
    assert!(link_plan_json.contains("\"abi\":\"c\""));
    assert!(link_plan_json.contains("\"symbols\":[\"host_monotonic_time_ns:i64()\""));
    assert!(link_plan_json.contains("\"validation\":{\"checked\":2,\"valid\":true"));
    assert!(link_plan_json.contains("\"entries\":[{\"symbol\":\"host_monotonic_time_ns\""));
    assert!(link_plan_json.contains("\"host_ffi_entries\":[{"));
    assert!(link_plan_json.contains("\"symbol\":\"host_monotonic_time_ns\""));
    assert!(link_plan_json.contains("\"symbol\":\"host_sleep_ns\""));
    assert!(link_plan_json.contains("\"signature_pattern\":\"i64(i64)\""));

    let artifact_report =
        aot::verify_nuis_compiled_artifact(output_dir.join("nuis.compiled.artifact").as_path())
            .unwrap();
    assert_eq!(artifact_report.binary_name, output_stem);
    assert_eq!(artifact_report.packaging_mode, "native-cpu-llvm");
    assert!(artifact_report.lifecycle_contract_consistent);
    assert!(artifact_report.artifact_roundtrip_verified);

    let status = Command::new(output_dir.join(&output_stem))
        .status()
        .expect("expected compiled hetero proxy benchmark binary to launch");
    assert!(
        status.success(),
        "expected compiled hetero proxy benchmark binary to exit successfully"
    );
}

#[test]
fn verify_build_manifest_rejects_drifted_host_ffi_signature_hash() {
    let project_root = write_temp_project_fixture(
        "drifted_host_ffi_signature_hash",
        r#"
name = "drifted_host_ffi_signature_hash"
entry = "main.ns"
modules = ["main.ns"]
"#
        .trim_start(),
        r#"
mod cpu Main {
  extern "c" fn host_monotonic_time_ns() -> i64;
  extern "c" fn host_sleep_ns(duration_ns: i64) -> i64;

  fn main() -> i64 {
    let started: i64 = host_monotonic_time_ns();
    let slept: i64 = host_sleep_ns(1);
    let ended: i64 = host_monotonic_time_ns();
    if slept < 0 {
      return 1;
    }
    if ended < started {
      return 1;
    }
    return 0;
  }
}
"#,
    );
    let output_dir = temp_dir("compile_command_hetero_proxy_benchmark_drift_outputs");

    run(CommandKind::Compile {
        input: project_root,
        output_dir: output_dir.clone(),
        verbose_cache: false,
        cpu_abi: None,
        target: None,
        packaging_mode: None,
    })
    .unwrap();

    let host_ffi_path = output_dir.join("nuis.project.host_ffi.txt");
    let host_ffi_text = fs::read_to_string(&host_ffi_path).unwrap();
    let damaged = host_ffi_text.replacen("signature_hash=fnv1a64:", "signature_hash=fnv1a64:0", 1);
    fs::write(&host_ffi_path, damaged).unwrap();

    let manifest_path = output_dir.join("nuis.build.manifest.toml");
    let error = match aot::verify_build_manifest(&manifest_path) {
        Ok(_) => panic!("expected drifted host ffi signature hash to be rejected"),
        Err(error) => error,
    };
    assert!(error.contains("project host_ffi index"));
    assert!(error.contains("signature hash mismatch"));
}

#[test]
fn benchmark_inventory_collects_declared_benchmarks() {
    let artifacts = pipeline::compile_source(
            r#"
            mod cpu Main {
              benchmark("sum_loop", warmup_iters=4, measure_iters=32, timeout_ms=25, clock_domain="global", clock_policy="bridge")
              async fn sum_loop() -> i64 {
                return 1;
              }

              fn main() -> i64 {
                return 1;
              }
            }
            "#,
        )
        .unwrap();

    let entries = collect_benchmark_inventory(&artifacts);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].symbol, "cpu::Main::sum_loop");
    assert_eq!(entries[0].label, "sum_loop");
    assert!(entries[0].is_async);
    assert_eq!(entries[0].return_type, "i64");
    assert_eq!(entries[0].warmup_iters, Some(4));
    assert_eq!(entries[0].measure_iters, Some(32));
    assert_eq!(entries[0].timeout_ms, Some(25));
    assert_eq!(entries[0].clock_domain.as_deref(), Some("global"));
    assert_eq!(entries[0].clock_policy.as_deref(), Some("bridge"));
}

#[test]
fn inspect_benchmarks_json_exposes_metadata() {
    let artifacts = pipeline::compile_source(
        r#"
            mod cpu Main {
              benchmark("sum_loop", measure_iters=32)
              fn sum_loop() -> i64 {
                return 1;
              }

              fn main() -> i64 {
                return sum_loop();
              }
            }
            "#,
    )
    .unwrap();

    let json = inspect_benchmarks_json(Path::new("main.ns"), &artifacts);
    assert!(json.contains("\"kind\":\"nuis_benchmark_inventory\""));
    assert!(json.contains("\"input\":\"main.ns\""));
    assert!(json.contains("\"benchmark_count\":1"));
    assert!(json.contains("\"symbol\":\"cpu::Main::sum_loop\""));
    assert!(json.contains("\"label\":\"sum_loop\""));
    assert!(json.contains("\"measure_iters\":32"));
}

#[test]
fn inspect_docs_json_exposes_documented_items() {
    let ast = frontend::parse_nuis_ast(
        r#"
            /// module docs
            mod cpu Docs {
              /// function docs
              fn answer() -> i32 {
                42
              }
            }
            "#,
    )
    .unwrap();

    let indexes = vec![frontend::extract_ast_doc_index(&ast)];
    let json = inspect_docs_json(Path::new("main.ns"), &indexes);
    assert!(json.contains("\"kind\":\"nuis_doc_index\""));
    assert!(json.contains("\"input\":\"main.ns\""));
    assert!(json.contains("\"module_count\":1"));
    assert!(json.contains("\"documented_item_count\":2"));
    assert!(json.contains("\"module_path\":\"cpu.Docs\""));
    assert!(json.contains("\"kind\":\"module\""));
    assert!(json.contains("\"path\":\"cpu.Docs\""));
    assert!(json.contains("\"docs\":[\"module docs\"]"));
    assert!(json.contains("\"signature\":\"mod cpu Docs\""));
    assert!(json.contains("\"kind\":\"function\""));
    assert!(json.contains("\"path\":\"cpu.Docs::answer\""));
    assert!(json.contains("\"docs\":[\"function docs\"]"));
    assert!(json.contains("\"signature\":\"fn answer() -> i32\""));
}

#[test]
fn collect_doc_indexes_reads_single_source_input() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("nuis_doc_index_{nonce}.ns"));
    std::fs::write(
        &path,
        r#"
            /// module docs
            mod cpu Docs {
              /// value docs
              const ANSWER: i32 = 42;
            }
            "#,
    )
    .unwrap();

    let indexes = collect_doc_indexes(&path).unwrap();
    let _ = std::fs::remove_file(&path);

    assert_eq!(indexes.len(), 1);
    assert_eq!(indexes[0].module_path, "cpu.Docs");
    assert_eq!(indexes[0].items.len(), 2);
    assert_eq!(indexes[0].items[0].path, "cpu.Docs");
    assert_eq!(indexes[0].items[1].path, "cpu.Docs::ANSWER");
}

#[test]
fn write_json_output_persists_payload() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("nuis_doc_index_output_{nonce}.json"));
    write_json_output(&path, "{\"kind\":\"nuis_doc_index\"}").unwrap();
    let written = std::fs::read_to_string(&path).unwrap();
    let _ = std::fs::remove_file(&path);
    assert_eq!(written, "{\"kind\":\"nuis_doc_index\"}");
}

#[test]
fn inspect_galaxy_docs_json_reports_documented_library_modules() {
    let summary = inspect_galaxy_doc_summary("pixelmagic").unwrap();
    let json = inspect_galaxy_docs_json(&summary);

    assert!(json.contains("\"kind\":\"nuis_galaxy_doc_index\""));
    assert!(json.contains("\"galaxy\":\"pixelmagic\""));
    assert!(json.contains("\"package_id\":\"nuis.pixelmagic\""));
    assert!(json.contains("\"documented_library_module_count\":"));
    assert!(json.contains("\"documented_item_count\":"));
    assert!(json.contains("\"library_module\":\"lib/image_contracts.ns\""));
    assert!(json.contains("\"module_path\":\"cpu.PixelMagicContracts\""));
}

#[test]
fn inspect_stdlib_docs_json_reports_all_official_galaxies() {
    let summary = inspect_stdlib_doc_summary().unwrap();
    let json = inspect_stdlib_docs_json(&summary);

    assert!(json.contains("\"kind\":\"nuis_stdlib_doc_index\""));
    assert!(json.contains("\"galaxy_count\":5"));
    assert!(json.contains("\"galaxy\":\"core\""));
    assert!(json.contains("\"galaxy\":\"std\""));
    assert!(json.contains("\"galaxy\":\"pixelmagic\""));
    assert!(json.contains("\"galaxy\":\"witsage\""));
    assert!(json.contains("\"galaxy\":\"ns-nova\""));
}
