use super::{
    fnv1a64_hex, main_test_support::empty_link_plan, nsld_container_report, nsld_prepare_report,
    nsld_verify_container_report,
};
use crate::main_container_verify_tamper::tampered_container_source;
use nuisc::linker::LinkPlanHeteroNode;
use std::{env, fs, path::Path};

#[test]
fn verify_container_accepts_matching_emitted_container() {
    let dir = env::temp_dir().join(format!("nsld-container-verify-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let sidecar_path = dir.join("shader.sidecar.toml");
    let sidecar_source = r#"
schema = "nuis-shader-ir-sidecar-v1"
[lowering_capabilities]
capability_owner = "shader-nustar"
frontend_ir = "nuis-yir.shader"
native_ir = "msl2.4"
dispatch_lowering = "command-encoder-draw-dispatch"
validation_contracts = ["glm.resource-lifetime"]
"#;
    fs::write(&sidecar_path, sidecar_source).unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();
    plan.domain_units.push(nuisc::linker::LinkPlanDomainUnit {
        kind: "heterogeneous".to_owned(),
        package_id: "official.shader".to_owned(),
        domain_family: "shader".to_owned(),
        abi: None,
        machine_arch: None,
        machine_os: None,
        backend_family: Some("metal".to_owned()),
        vendor: None,
        device_class: None,
        selected_lowering_target: Some("metal.apple-silicon-gpu".to_owned()),
        contract_family: "nustar.shader".to_owned(),
        packaging_role: "hetero-contract".to_owned(),
        artifact_stub_path: None,
        artifact_stub_inline: None,
        artifact_payload_path: None,
        artifact_bridge_stub_path: None,
        artifact_ir_sidecar_path: Some(sidecar_path.display().to_string()),
        artifact_bridge_stub_inline: None,
        artifact_payload_blob_path: None,
        artifact_payload_blob_bytes: None,
        artifact_payload_format: None,
        artifact_payload_blob_inline: None,
    });
    plan.hetero_calculate.nodes.push(LinkPlanHeteroNode {
        index: 0,
        timestamp: "t0001.shader".to_owned(),
        domain_family: "shader".to_owned(),
        package_id: "official.shader".to_owned(),
        lifecycle_hook: "on_hetero_submission_progress".to_owned(),
        wait_on: vec!["t0000.main".to_owned()],
        emits: vec![
            "t0001.shader.submit".to_owned(),
            "t0001.shader.complete".to_owned(),
            "t0001.shader.data_commit".to_owned(),
        ],
        link_input: sidecar_path.display().to_string(),
        c_world_wrapper: true,
    });
    let prepare = nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let container_source = fs::read_to_string(&prepare.container_path).unwrap();
    let payload_bytes = fs::read(&prepare.container_payload_path).unwrap();
    let preview = nsld_container_report(Path::new("manifest.toml"), &plan);

    let report = nsld_verify_container_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::nsld_container_verify_report_json(&report);

    assert_eq!(preview.loader_symbols.len(), 3);
    assert_eq!(preview.relocations.len(), 3);
    assert!(preview
        .loader_symbols
        .iter()
        .any(|symbol| symbol.symbol_kind == "native-object-output"));
    assert_eq!(payload_bytes.len(), prepare.payload_size_bytes);
    assert_eq!(fnv1a64_hex(&payload_bytes), prepare.payload_hash);
    assert!(container_source.contains("offset = 0"));
    assert!(container_source.contains("size_bytes = 17"));
    assert!(container_source.contains("loader_readiness = \"host-assisted\""));
    assert!(container_source.contains("external-import:final-stage-driver:clang"));
    assert!(container_source.contains("external-import:clang-target:"));
    assert!(container_source.contains("external-import:c-world-policy:wrapped"));
    assert!(container_source.contains("loader_entry_kind = \"lifecycle-bootstrap\""));
    assert!(container_source.contains("loader_entry_symbol = \"main\""));
    assert!(container_source.contains("loader_entry_section_id = \"sec0000.compiled-artifact\""));
    assert!(container_source.contains("loader_symbol_count = 3"));
    assert!(container_source.contains("loader_symbol_table_hash = \"0x"));
    assert!(container_source.contains("[[loader_symbol]]"));
    assert!(container_source.contains("symbol_id = \"sym0000.loader-entry\""));
    assert!(container_source.contains("symbol_name = \"main\""));
    assert!(container_source.contains("lifecycle_hook = \"on_lifecycle_bootstrap\""));
    assert!(container_source.contains("section_id = \"sec0000.compiled-artifact\""));
    assert!(container_source.contains("symbol_id = \"sym0001.hetero-node.shader.official.shader\""));
    assert!(container_source.contains("symbol_kind = \"hetero-node-dispatch\""));
    assert!(container_source.contains("symbol_name = \"t0001.shader\""));
    assert!(container_source.contains("lifecycle_hook = \"on_hetero_submission_progress\""));
    assert!(container_source.contains("symbol_id = \"sym0002.native-object-output\""));
    assert!(container_source.contains("symbol_kind = \"native-object-output\""));
    assert!(container_source.contains("symbol_name = \"__nuis_native_object\""));
    assert!(container_source.contains("lifecycle_hook = \"on_cffi_native_object\""));
    assert!(container_source.contains("relocation_count = 3"));
    assert!(container_source.contains("relocation_table_hash = \"0x"));
    assert!(container_source.contains("[[relocation]]"));
    assert!(container_source.contains("relocation_id = \"rel0000.lifecycle-entry\""));
    assert!(container_source.contains("relocation_kind = \"lifecycle-entry-binding\""));
    assert!(container_source.contains("source_section_id = \"sec0000.compiled-artifact\""));
    assert!(container_source.contains("target_symbol_id = \"sym0000.loader-entry\""));
    assert!(container_source.contains("relocation_id = \"rel0001.hetero-node\""));
    assert!(container_source.contains("relocation_kind = \"hetero-node-binding\""));
    assert!(container_source
        .contains("target_symbol_id = \"sym0001.hetero-node.shader.official.shader\""));
    assert!(container_source.contains("relocation_id = \"rel0002.native-object\""));
    assert!(container_source.contains("relocation_kind = \"native-object-binding\""));
    assert!(container_source.contains("target_symbol_id = \"sym0002.native-object-output\""));
    assert!(container_source.contains("compatibility_domain_count = 1"));
    assert!(container_source.contains("compatibility_domain_table_hash = \"0x"));
    assert!(container_source.contains("[[compatibility_domain]]"));
    assert!(container_source.contains("domain_id = \"compat0000.cffi-von-neumann\""));
    assert!(container_source.contains("domain_kind = \"cffi-host-compat\""));
    assert!(container_source.contains("paradigm = \"classic-von-neumann-host\""));
    assert!(container_source.contains("lifecycle_hook = \"on_cffi_native_object\""));
    assert!(container_source.contains("abi_family = \"mach-o\""));
    assert!(container_source.contains("wrapper_policy = \"wrapped\""));
    assert!(container_source.contains("external_import_count = 3"));
    assert!(container_source.contains("external_import_table_hash = \"0x"));
    assert!(container_source.contains("[[external_import]]"));
    assert!(container_source.contains("import_kind = \"final-stage-driver\""));
    assert!(container_source.contains("import_kind = \"clang-target\""));
    assert!(container_source.contains("import_kind = \"c-world-policy\""));
    assert!(container_source.contains("payload_size_bytes = "));
    assert!(container_source.contains("payload_hash = \"0x"));
    assert!(container_source.contains("container_section_table_hash = \"0x"));
    assert!(container_source.contains("metadata_table_hash = \"0x"));
    assert!(container_source.contains("section_kind = \"native-object-output\""));
    assert!(report.valid);
    assert!(report.issues.is_empty());
    assert_eq!(report.actual_section_count, Some(6));
    assert_eq!(
        report.actual_container_section_table_hash.as_deref(),
        Some(report.expected_container_section_table_hash.as_str())
    );
    assert_eq!(
        report.actual_container_layout_hash,
        Some(prepare.container_layout_hash)
    );
    assert_eq!(report.actual_container_hash, Some(prepare.container_hash));
    assert_eq!(
        report.actual_metadata_table_hash.as_deref(),
        Some(report.expected_metadata_table_hash.as_str())
    );
    assert_eq!(report.expected_loader_readiness, "host-assisted");
    assert_eq!(
        report.actual_loader_readiness.as_deref(),
        Some("host-assisted")
    );
    assert_eq!(report.expected_loader_entry_kind, "lifecycle-bootstrap");
    assert_eq!(
        report.actual_loader_entry_kind.as_deref(),
        Some("lifecycle-bootstrap")
    );
    assert_eq!(report.expected_loader_entry_symbol, "main");
    assert_eq!(report.actual_loader_entry_symbol.as_deref(), Some("main"));
    assert_eq!(
        report.expected_loader_entry_section_id,
        "sec0000.compiled-artifact"
    );
    assert_eq!(
        report.actual_loader_entry_section_id.as_deref(),
        Some("sec0000.compiled-artifact")
    );
    assert_eq!(report.expected_external_import_count, 3);
    assert_eq!(report.actual_external_import_count, Some(3));
    assert_eq!(report.expected_loader_symbol_count, 3);
    assert_eq!(report.actual_loader_symbol_count, Some(3));
    assert_eq!(
        report.actual_loader_symbol_table_hash.as_deref(),
        Some(report.expected_loader_symbol_table_hash.as_str())
    );
    assert_eq!(report.expected_loader_symbol_id, "sym0000.loader-entry");
    assert_eq!(
        report.actual_loader_symbol_id.as_deref(),
        Some("sym0000.loader-entry")
    );
    assert_eq!(report.expected_loader_symbol_kind, "lifecycle-bootstrap");
    assert_eq!(
        report.actual_loader_symbol_kind.as_deref(),
        Some("lifecycle-bootstrap")
    );
    assert_eq!(report.expected_loader_symbol_name, "main");
    assert_eq!(report.actual_loader_symbol_name.as_deref(), Some("main"));
    assert_eq!(
        report.expected_loader_symbol_section_id,
        "sec0000.compiled-artifact"
    );
    assert_eq!(
        report.actual_loader_symbol_section_id.as_deref(),
        Some("sec0000.compiled-artifact")
    );
    assert_eq!(report.expected_relocation_count, 3);
    assert_eq!(report.actual_relocation_count, Some(3));
    assert_eq!(
        report.actual_relocation_table_hash.as_deref(),
        Some(report.expected_relocation_table_hash.as_str())
    );
    assert_eq!(report.expected_relocation_id, "rel0000.lifecycle-entry");
    assert_eq!(
        report.actual_relocation_id.as_deref(),
        Some("rel0000.lifecycle-entry")
    );
    assert_eq!(report.expected_relocation_kind, "lifecycle-entry-binding");
    assert_eq!(
        report.actual_relocation_kind.as_deref(),
        Some("lifecycle-entry-binding")
    );
    assert_eq!(
        report.expected_relocation_source_section_id,
        "sec0000.compiled-artifact"
    );
    assert_eq!(
        report.actual_relocation_source_section_id.as_deref(),
        Some("sec0000.compiled-artifact")
    );
    assert_eq!(report.expected_relocation_source_offset, 0);
    assert_eq!(report.actual_relocation_source_offset, Some(0));
    assert_eq!(
        report.expected_relocation_target_symbol_id,
        "sym0000.loader-entry"
    );
    assert_eq!(
        report.actual_relocation_target_symbol_id.as_deref(),
        Some("sym0000.loader-entry")
    );
    assert_eq!(report.expected_relocation_addend, 0);
    assert_eq!(report.actual_relocation_addend, Some(0));
    assert_eq!(report.expected_compatibility_domain_count, 1);
    assert_eq!(report.actual_compatibility_domain_count, Some(1));
    assert_eq!(
        report.actual_compatibility_domain_table_hash.as_deref(),
        Some(report.expected_compatibility_domain_table_hash.as_str())
    );
    assert_eq!(
        report.expected_compatibility_domain_id,
        "compat0000.cffi-von-neumann"
    );
    assert_eq!(
        report.actual_compatibility_domain_id.as_deref(),
        Some("compat0000.cffi-von-neumann")
    );
    assert_eq!(
        report.expected_compatibility_domain_kind,
        "cffi-host-compat"
    );
    assert_eq!(
        report.actual_compatibility_domain_kind.as_deref(),
        Some("cffi-host-compat")
    );
    assert_eq!(
        report.expected_compatibility_domain_paradigm,
        "classic-von-neumann-host"
    );
    assert_eq!(
        report.actual_compatibility_domain_paradigm.as_deref(),
        Some("classic-von-neumann-host")
    );
    assert_eq!(
        report.expected_compatibility_domain_lifecycle_hook,
        "on_cffi_native_object"
    );
    assert_eq!(
        report.actual_compatibility_domain_lifecycle_hook.as_deref(),
        Some("on_cffi_native_object")
    );
    assert_eq!(report.expected_compatibility_domain_abi_family, "mach-o");
    assert_eq!(
        report.actual_compatibility_domain_abi_family.as_deref(),
        Some("mach-o")
    );
    assert_eq!(
        report.expected_compatibility_domain_wrapper_policy,
        "wrapped"
    );
    assert_eq!(
        report.actual_compatibility_domain_wrapper_policy.as_deref(),
        Some("wrapped")
    );
    assert!(report.expected_compatibility_domain_required);
    assert_eq!(report.actual_compatibility_domain_required, Some(true));
    assert_eq!(
        report.actual_external_import_table_hash.as_deref(),
        Some(report.expected_external_import_table_hash.as_str())
    );
    assert_eq!(
        report.expected_external_import_id,
        "imp0000.final-stage-driver"
    );
    assert_eq!(
        report.actual_external_import_id.as_deref(),
        Some("imp0000.final-stage-driver")
    );
    assert_eq!(report.expected_external_import_kind, "final-stage-driver");
    assert_eq!(
        report.actual_external_import_kind.as_deref(),
        Some("final-stage-driver")
    );
    assert_eq!(report.expected_external_import_name, "clang");
    assert_eq!(report.actual_external_import_name.as_deref(), Some("clang"));
    assert_eq!(report.expected_external_import_provider, "host-toolchain");
    assert_eq!(
        report.actual_external_import_provider.as_deref(),
        Some("host-toolchain")
    );
    assert!(report.expected_external_import_required);
    assert_eq!(report.actual_external_import_required, Some(true));
    assert!(report.expected_native_object_section_present);
    assert_eq!(
        report.expected_native_object_section_id,
        "sec0005.native-object-output"
    );
    assert!(report.actual_native_object_section_present);
    assert_eq!(
        report.actual_native_object_section_id.as_deref(),
        Some("sec0005.native-object-output")
    );
    assert!(report.expected_native_object_loader_symbol_present);
    assert_eq!(
        report.expected_native_object_loader_symbol_id,
        "sym0002.native-object-output"
    );
    assert!(report.actual_native_object_loader_symbol_present);
    assert_eq!(
        report.actual_native_object_loader_symbol_id.as_deref(),
        Some("sym0002.native-object-output")
    );
    assert!(report.expected_native_object_relocation_present);
    assert_eq!(
        report.expected_native_object_relocation_id,
        "rel0002.native-object"
    );
    assert!(report.actual_native_object_relocation_present);
    assert_eq!(
        report.actual_native_object_relocation_id.as_deref(),
        Some("rel0002.native-object")
    );
    assert!(report_json.contains("\"expected_native_object_section_present\":true"));
    assert!(report_json.contains("\"actual_native_object_section_present\":true"));
    assert!(report_json.contains("\"expected_compatibility_domain_count\":1"));
    assert!(report_json.contains("\"actual_compatibility_domain_count\":1"));
    assert!(report_json
        .contains("\"expected_compatibility_domain_paradigm\":\"classic-von-neumann-host\""));
    assert!(report_json
        .contains("\"actual_compatibility_domain_lifecycle_hook\":\"on_cffi_native_object\""));
    assert!(report_json
        .contains("\"expected_compatibility_domain_summary\":{\"count\":1,\"table_hash\":\"0x"));
    assert!(report_json
        .contains("\"actual_compatibility_domain_summary\":{\"count\":1,\"table_hash\":\"0x"));
    assert!(report_json
        .contains("\"expected_native_object_loader_symbol_id\":\"sym0002.native-object-output\""));
    assert!(
        report_json.contains("\"actual_native_object_relocation_id\":\"rel0002.native-object\"")
    );

    fs::write(
        &prepare.container_path,
        tampered_container_source(&container_source, &report),
    )
    .unwrap();
    let tampered_report = nsld_verify_container_report(Path::new("manifest.toml"), &plan);
    let tampered_json = super::json::nsld_container_verify_report_json(&tampered_report);
    assert!(!tampered_report.valid);
    assert!(tampered_json.contains("\"container_section_issues\":["));
    assert!(tampered_json.contains("\"loader_symbol_issues\":["));
    assert!(tampered_json.contains("\"relocation_issues\":["));
    assert!(tampered_json.contains("\"compatibility_domain_issues\":["));
    assert!(tampered_json.contains("\"external_import_issues\":["));
    assert_eq!(
        tampered_report.actual_loader_readiness.as_deref(),
        Some("self-contained")
    );
    assert_eq!(
        tampered_report
            .actual_container_section_table_hash
            .as_deref(),
        Some("0x0000000000000000")
    );
    assert_eq!(
        tampered_report.actual_metadata_table_hash.as_deref(),
        Some("0x0000000000000000")
    );
    assert!(tampered_report
        .container_section_issues
        .iter()
        .any(|issue| issue.starts_with("container_section[1].section_id mismatch")));
    assert_eq!(
        tampered_report.actual_loader_entry_kind.as_deref(),
        Some("manual-entry")
    );
    assert_eq!(
        tampered_report.actual_loader_entry_symbol.as_deref(),
        Some("alt")
    );
    assert_eq!(
        tampered_report.actual_loader_entry_section_id.as_deref(),
        Some("sec9999.missing")
    );
    assert_eq!(tampered_report.actual_external_import_count, Some(0));
    assert_eq!(tampered_report.actual_loader_symbol_count, Some(0));
    assert_eq!(
        tampered_report.actual_loader_symbol_table_hash.as_deref(),
        Some("0x0000000000000000")
    );
    assert_eq!(
        tampered_report.actual_loader_symbol_id.as_deref(),
        Some("sym9999.manual")
    );
    assert_eq!(
        tampered_report.actual_loader_symbol_kind.as_deref(),
        Some("manual-symbol")
    );
    assert_eq!(
        tampered_report.actual_loader_symbol_name.as_deref(),
        Some("alt")
    );
    assert_eq!(
        tampered_report.actual_loader_symbol_section_id.as_deref(),
        Some("sec9999.missing")
    );
    assert!(tampered_report
        .loader_symbol_issues
        .iter()
        .any(|issue| issue.starts_with("loader_symbol[1].symbol_name mismatch")));
    assert_eq!(tampered_report.actual_relocation_count, Some(4));
    assert_eq!(
        tampered_report.actual_relocation_table_hash.as_deref(),
        Some("0x0000000000000000")
    );
    assert_eq!(
        tampered_report.actual_relocation_id.as_deref(),
        Some("rel9999.manual")
    );
    assert_eq!(
        tampered_report.actual_relocation_kind.as_deref(),
        Some("manual-relocation")
    );
    assert_eq!(
        tampered_report
            .actual_relocation_source_section_id
            .as_deref(),
        Some("sec9999.missing")
    );
    assert_eq!(tampered_report.actual_relocation_source_offset, Some(7));
    assert_eq!(
        tampered_report
            .actual_relocation_target_symbol_id
            .as_deref(),
        Some("sym9999.manual")
    );
    assert_eq!(tampered_report.actual_relocation_addend, Some(-4));
    assert!(tampered_report
        .relocation_issues
        .iter()
        .any(|issue| issue.starts_with("relocation[1].target_symbol_id mismatch")));
    assert_eq!(tampered_report.actual_compatibility_domain_count, Some(2));
    assert_eq!(
        tampered_report
            .actual_compatibility_domain_table_hash
            .as_deref(),
        Some("0x0000000000000000")
    );
    assert_eq!(
        tampered_report.actual_compatibility_domain_id.as_deref(),
        Some("compat9999.manual")
    );
    assert_eq!(
        tampered_report.actual_compatibility_domain_kind.as_deref(),
        Some("manual-compat")
    );
    assert_eq!(
        tampered_report
            .actual_compatibility_domain_paradigm
            .as_deref(),
        Some("manual-host")
    );
    assert_eq!(
        tampered_report
            .actual_compatibility_domain_lifecycle_hook
            .as_deref(),
        Some("on_manual_compat")
    );
    assert_eq!(
        tampered_report
            .actual_compatibility_domain_abi_family
            .as_deref(),
        Some("manual-object")
    );
    assert_eq!(
        tampered_report
            .actual_compatibility_domain_wrapper_policy
            .as_deref(),
        Some("manual-wrapper")
    );
    assert_eq!(
        tampered_report.actual_compatibility_domain_required,
        Some(false)
    );
    assert!(tampered_report
        .compatibility_domain_issues
        .iter()
        .any(|issue| issue.starts_with("compatibility_domain[0].domain_id mismatch")));
    assert!(tampered_report
        .compatibility_domain_issues
        .iter()
        .any(|issue| issue.starts_with("compatibility_domain[0].paradigm mismatch")));
    assert_eq!(
        tampered_report.actual_external_import_table_hash.as_deref(),
        Some("0x0000000000000000")
    );
    assert_eq!(
        tampered_report.actual_external_import_id.as_deref(),
        Some("imp9999.manual")
    );
    assert_eq!(
        tampered_report.actual_external_import_kind.as_deref(),
        Some("manual-driver")
    );
    assert!(tampered_report
        .external_import_issues
        .iter()
        .any(|issue| issue.starts_with("external_import[1].import_kind mismatch")));
    assert_eq!(
        tampered_report.actual_external_import_name.as_deref(),
        Some("manual-clang")
    );
    assert_eq!(
        tampered_report.actual_external_import_provider.as_deref(),
        Some("manual-provider")
    );
    assert_eq!(tampered_report.actual_external_import_required, Some(false));
    assert!(tampered_report
        .issues
        .iter()
        .any(|issue| issue.starts_with("loader_readiness mismatch")));
    assert!(tampered_report
        .issues
        .iter()
        .any(|issue| issue.starts_with("container_section_table_hash mismatch")));
    assert!(tampered_report
        .issues
        .iter()
        .any(|issue| issue.starts_with("metadata_table_hash mismatch")));
    assert!(tampered_report
        .issues
        .iter()
        .any(|issue| issue.starts_with("loader_entry_kind mismatch")));
    assert!(tampered_report
        .issues
        .iter()
        .any(|issue| issue.starts_with("loader_entry_symbol mismatch")));
    assert!(tampered_report
        .issues
        .iter()
        .any(|issue| issue.starts_with("loader_entry_section_id mismatch")));
    assert!(tampered_report
        .issues
        .iter()
        .any(|issue| issue.starts_with("loader_symbol_count mismatch")));
    assert!(tampered_report
        .issues
        .iter()
        .any(|issue| issue.starts_with("loader_symbol_table_hash mismatch")));
    assert!(tampered_report
        .issues
        .iter()
        .any(|issue| issue.starts_with("loader_symbol_id mismatch")));
    assert!(tampered_report
        .issues
        .iter()
        .any(|issue| issue.starts_with("loader_symbol_kind mismatch")));
    assert!(tampered_report
        .issues
        .iter()
        .any(|issue| issue.starts_with("loader_symbol_name mismatch")));
    assert!(tampered_report
        .issues
        .iter()
        .any(|issue| issue.starts_with("loader_symbol_section_id mismatch")));
    assert!(tampered_report
        .issues
        .iter()
        .any(|issue| issue.starts_with("relocation_count mismatch")));
    assert!(tampered_report
        .issues
        .iter()
        .any(|issue| issue.starts_with("relocation_table_hash mismatch")));
    assert!(tampered_report
        .issues
        .iter()
        .any(|issue| issue.starts_with("relocation_id mismatch")));
    assert!(tampered_report
        .issues
        .iter()
        .any(|issue| issue.starts_with("relocation_kind mismatch")));
    assert!(tampered_report
        .issues
        .iter()
        .any(|issue| issue.starts_with("relocation_source_section_id mismatch")));
    assert!(tampered_report
        .issues
        .iter()
        .any(|issue| issue.starts_with("relocation_source_offset mismatch")));
    assert!(tampered_report
        .issues
        .iter()
        .any(|issue| issue.starts_with("relocation_target_symbol_id mismatch")));
    assert!(tampered_report
        .issues
        .iter()
        .any(|issue| issue.starts_with("relocation_addend mismatch")));
    assert!(tampered_report
        .issues
        .iter()
        .any(|issue| issue.starts_with("compatibility_domain_count mismatch")));
    assert!(tampered_report
        .issues
        .iter()
        .any(|issue| issue.starts_with("compatibility_domain_table_hash mismatch")));
    assert!(tampered_report
        .issues
        .iter()
        .any(|issue| issue.starts_with("compatibility_domain_id mismatch")));
    assert!(tampered_report
        .issues
        .iter()
        .any(|issue| issue.starts_with("compatibility_domain_kind mismatch")));
    assert!(tampered_report
        .issues
        .iter()
        .any(|issue| issue.starts_with("compatibility_domain_paradigm mismatch")));
    assert!(tampered_report
        .issues
        .iter()
        .any(|issue| issue.starts_with("compatibility_domain_lifecycle_hook mismatch")));
    assert!(tampered_report
        .issues
        .iter()
        .any(|issue| issue.starts_with("compatibility_domain_abi_family mismatch")));
    assert!(tampered_report
        .issues
        .iter()
        .any(|issue| issue.starts_with("compatibility_domain_wrapper_policy mismatch")));
    assert!(tampered_report
        .issues
        .iter()
        .any(|issue| issue.starts_with("compatibility_domain_required mismatch")));
    assert!(tampered_report
        .issues
        .iter()
        .any(|issue| issue.starts_with("external_import_count mismatch")));
    assert!(tampered_report
        .issues
        .iter()
        .any(|issue| issue.starts_with("external_import_table_hash mismatch")));
    assert!(tampered_report
        .issues
        .iter()
        .any(|issue| issue.starts_with("external_import_id mismatch")));
    assert!(tampered_report
        .issues
        .iter()
        .any(|issue| issue.starts_with("external_import_kind mismatch")));
    assert!(tampered_report
        .issues
        .iter()
        .any(|issue| issue.starts_with("external_import_name mismatch")));
    assert!(tampered_report
        .issues
        .iter()
        .any(|issue| issue.starts_with("external_import_provider mismatch")));
    assert!(tampered_report
        .issues
        .iter()
        .any(|issue| issue.starts_with("external_import_required mismatch")));
    fs::write(&prepare.container_path, container_source).unwrap();

    let mut corrupted_payload = payload_bytes;
    corrupted_payload[0] ^= 0xff;
    fs::write(&prepare.container_payload_path, corrupted_payload).unwrap();
    let corrupted_report = nsld_verify_container_report(Path::new("manifest.toml"), &plan);
    assert!(!corrupted_report.valid);
    assert!(corrupted_report
        .issues
        .iter()
        .any(|issue| issue.starts_with("payload_file_hash mismatch")));
    assert!(corrupted_report
        .section_range_issues
        .iter()
        .any(|issue| issue.starts_with("section_payload_hash mismatch")));
    fs::remove_dir_all(dir).unwrap();
}
