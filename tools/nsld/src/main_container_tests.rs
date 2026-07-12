use super::{
    main_container_domain_assertions::assert_matching_kernel_contract,
    main_test_support::empty_link_plan, nsld_check_report, nsld_container_plan_report,
    nsld_container_report, nsld_emit_container_report, nsld_prepare_report,
    nsld_verify_container_plan_report, nsld_verify_container_report,
};
use nuisc::linker::LinkPlanHeteroNode;
use std::{env, fs, path::Path};

#[test]
fn verify_container_plan_accepts_matching_emitted_plan() {
    let dir = env::temp_dir().join(format!("nsld-container-plan-verify-{}", std::process::id()));
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
        target_device: None,
        ir_format: None,
        dispatch_abi: None,
        backend_priority: None,
        verification: None,
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

    let report = nsld_verify_container_plan_report(Path::new("manifest.toml"), &plan);
    fs::remove_dir_all(dir).unwrap();

    assert!(report.valid);
    assert!(report.issues.is_empty());
    assert_eq!(report.actual_section_count, Some(6));
    assert_eq!(
        report.actual_container_layout_hash,
        Some(prepare.container_layout_hash)
    );
}

#[test]
fn verify_container_tracks_kernel_domain_contract() {
    let dir = env::temp_dir().join(format!(
        "nsld-container-kernel-domain-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let sidecar_path = dir.join("kernel.sidecar.toml");
    let sidecar_source = r#"
schema = "nuis-kernel-ir-sidecar-v1"
[lowering_capabilities]
capability_owner = "kernel-nustar"
frontend_ir = "nuis-yir.kernel"
native_ir = "coreml"
dispatch_lowering = "kernel-host-reference-dispatch"
validation_contracts = ["glm.resource-lifetime"]
"#;
    fs::write(&sidecar_path, sidecar_source).unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();
    plan.domain_units.push(nuisc::linker::LinkPlanDomainUnit {
        kind: "heterogeneous".to_owned(),
        package_id: "official.kernel".to_owned(),
        domain_family: "kernel".to_owned(),
        abi: None,
        machine_arch: None,
        machine_os: None,
        backend_family: Some("coreml".to_owned()),
        vendor: None,
        device_class: None,
        target_device: None,
        ir_format: None,
        dispatch_abi: None,
        backend_priority: None,
        verification: None,
        selected_lowering_target: Some("apple-ane.coreml".to_owned()),
        contract_family: "nustar.kernel".to_owned(),
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
        timestamp: "t0001.kernel".to_owned(),
        domain_family: "kernel".to_owned(),
        package_id: "official.kernel".to_owned(),
        lifecycle_hook: "on_hetero_submission_progress".to_owned(),
        wait_on: vec!["t0000.main".to_owned()],
        emits: vec![
            "t0001.kernel.submit".to_owned(),
            "t0001.kernel.complete".to_owned(),
            "t0001.kernel.data_commit".to_owned(),
        ],
        link_input: sidecar_path.display().to_string(),
        c_world_wrapper: false,
    });
    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let report = nsld_verify_container_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::nsld_container_verify_report_json(&report);
    let check_report = nsld_check_report(Path::new("manifest.toml"), &plan);
    let check_json = super::json::check_report_json(&check_report);
    fs::remove_dir_all(dir).unwrap();

    assert!(report.valid, "{:?}", report.issues);
    assert_matching_kernel_contract(&report);
    assert!(report_json.contains("\"expected_kernel_section_present\":true"));
    assert!(report_json.contains("\"actual_kernel_section_present\":true"));
    assert!(report_json.contains(
        "\"expected_kernel_loader_symbol_id\":\"sym0001.hetero-node.kernel.official.kernel\""
    ));
    assert!(report_json.contains("\"actual_kernel_relocation_id\":\"rel0001.hetero-node\""));
    assert!(check_report.container_kernel_section_present);
    assert_eq!(
        check_report.container_kernel_section_id.as_deref(),
        Some("sec0004.kernel-lowering-sidecar-input")
    );
    assert!(check_report.container_kernel_loader_symbol_present);
    assert_eq!(
        check_report.container_kernel_loader_symbol_id.as_deref(),
        Some("sym0001.hetero-node.kernel.official.kernel")
    );
    assert!(check_report.container_kernel_relocation_present);
    assert_eq!(
        check_report.container_kernel_relocation_id.as_deref(),
        Some("rel0001.hetero-node")
    );
    assert!(check_json.contains("\"container_kernel_section_present\":true"));
    assert!(check_json
        .contains("\"container_kernel_section_id\":\"sec0004.kernel-lowering-sidecar-input\""));
    assert!(check_json.contains(
        "\"container_kernel_loader_symbol_id\":\"sym0001.hetero-node.kernel.official.kernel\""
    ));
    assert!(check_json.contains("\"container_kernel_relocation_id\":\"rel0001.hetero-node\""));
}

#[test]
fn emit_container_reports_metadata_table_hash() {
    let dir = env::temp_dir().join(format!("nsld-container-emit-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    let report = nsld_emit_container_report(Path::new("manifest.toml"), &plan).unwrap();
    let container_source = fs::read_to_string(&report.output_path).unwrap();
    let preview = nsld_container_report(Path::new("manifest.toml"), &plan);
    let preview_json = super::json::nsld_container_report_json(&preview);
    let emit_json = super::json::nsld_container_emit_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(report.metadata_table_hash.starts_with("0x"));
    assert!(container_source.contains(&format!(
        "metadata_table_hash = \"{}\"",
        report.metadata_table_hash
    )));
    assert!(preview_json.contains("\"metadata_table_hash\":\"0x"));
    assert!(preview_json.contains("\"container_section_table_hash\":\"0x"));
    assert!(preview_json.contains("\"loader_symbol_table_hash\":\"0x"));
    assert!(preview_json.contains("\"relocation_table_hash\":\"0x"));
    assert!(preview_json.contains("\"compatibility_domain_table_hash\":\"0x"));
    assert!(
        preview_json.contains("\"compatibility_domain_summary\":{\"count\":1,\"table_hash\":\"0x")
    );
    assert!(preview_json.contains("\"domain_kind\":\"cffi-host-compat\""));
    assert!(preview_json.contains("\"paradigm\":\"classic-von-neumann-host\""));
    assert!(preview_json.contains("\"external_import_table_hash\":\"0x"));
    assert!(emit_json.contains("\"metadata_table_hash\":\"0x"));
}

#[test]
fn container_plan_blocks_invalid_native_object_output() {
    let dir = env::temp_dir().join(format!(
        "nsld-container-invalid-object-output-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    fs::write(dir.join("nuis.nsld.mach-o"), b"drifted-object").unwrap();
    let plan_report = nsld_container_plan_report(Path::new("manifest.toml"), &plan);
    let container_report = nsld_container_report(Path::new("manifest.toml"), &plan);
    fs::remove_dir_all(dir).unwrap();

    assert!(!plan_report.ready);
    assert!(plan_report
        .blockers
        .iter()
        .any(|blocker| blocker.contains("object-output:object_output_hash mismatch")));
    assert!(plan_report
        .sections
        .iter()
        .all(|section| section.section_kind != "native-object-output"));
    assert_eq!(container_report.loader_readiness, "blocked");
    assert!(container_report
        .loader_blockers
        .iter()
        .any(|blocker| blocker.contains("object-output:object_output_hash mismatch")));
}
