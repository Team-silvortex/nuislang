use super::{
    main_test_support::empty_link_plan, nsld_emit_final_executable_image_dry_run_report,
    nsld_final_executable_image_dry_run_report, nsld_prepare_report,
};
use std::{env, fs, path::Path};

#[test]
fn final_executable_image_dry_run_reports_backend_artifact_payload_roles() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-image-backend-payload-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    let kernel_payload = dir.join("kernel.payload.bin");
    let kernel_bridge = dir.join("kernel.bridge.stub");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    fs::write(&kernel_payload, b"kernel-payload").unwrap();
    fs::write(&kernel_bridge, b"kernel-bridge").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();
    plan.final_stage.output_path = dir.join("nuis-app.nsb").display().to_string();
    plan.domain_units.push(nuisc::linker::LinkPlanDomainUnit {
        kind: "heterogeneous".to_owned(),
        package_id: "official.kernel".to_owned(),
        domain_family: "kernel".to_owned(),
        abi: None,
        machine_arch: None,
        machine_os: None,
        backend_family: Some("aarch64".to_owned()),
        vendor: Some("apple".to_owned()),
        device_class: Some("cpu".to_owned()),
        target_device: Some("apple-silicon-cpu".to_owned()),
        ir_format: Some("yir-kernel".to_owned()),
        dispatch_abi: None,
        backend_priority: Some(10),
        verification: None,
        selected_lowering_target: Some("aarch64.apple-silicon-cpu".to_owned()),
        contract_family: "nustar.kernel".to_owned(),
        packaging_role: "heterogeneous-domain".to_owned(),
        artifact_stub_path: None,
        artifact_stub_inline: None,
        artifact_payload_path: None,
        artifact_bridge_stub_path: Some(kernel_bridge.display().to_string()),
        artifact_ir_sidecar_path: None,
        artifact_bridge_stub_inline: None,
        artifact_payload_blob_path: Some(kernel_payload.display().to_string()),
        artifact_payload_blob_bytes: Some(14),
        artifact_payload_format: Some("nuis-kernel-payload-v1".to_owned()),
        artifact_payload_blob_inline: None,
    });

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let report = nsld_final_executable_image_dry_run_report(Path::new("manifest.toml"), &plan);
    let emit =
        nsld_emit_final_executable_image_dry_run_report(Path::new("manifest.toml"), &plan).unwrap();
    let report_json = super::json::nsld_final_executable_image_dry_run_report_json(&report);
    let report_source = fs::read_to_string(&emit.output_path).unwrap();
    fs::remove_dir_all(dir).unwrap();

    assert!(report.image_ready, "{:?}", report.blockers);
    assert_eq!(report.backend_artifact_payload_count, 1);
    assert_eq!(report.backend_artifact_payload_present_count, 1);
    assert_eq!(report.backend_artifact_payload_role_status, "ready");
    assert_eq!(
        report.backend_artifact_payload_ids,
        vec!["payload0005.backend-artifact".to_owned()]
    );
    assert_eq!(
        report.backend_artifact_payload_kinds,
        vec!["nustar-backend-artifact:kernel:aarch64:apple-silicon-cpu".to_owned()]
    );
    assert_eq!(report.backend_artifact_payload_first_missing, None);
    assert!(report_json.contains("\"backend_artifact_payload_count\":1"));
    assert!(report_json.contains("\"backend_artifact_payload_present_count\":1"));
    assert!(report_json.contains("\"backend_artifact_payload_role_status\":\"ready\""));
    assert!(
        report_json.contains("\"backend_artifact_payload_ids\":[\"payload0005.backend-artifact\"]")
    );
    assert!(report_json.contains(
        "\"backend_artifact_payload_kinds\":[\"nustar-backend-artifact:kernel:aarch64:apple-silicon-cpu\"]"
    ));
    assert!(report_json.contains("\"backend_artifact_payload_first_missing\":null"));
    assert!(report_source.contains("backend_artifact_payload_count = 1"));
    assert!(report_source.contains("backend_artifact_payload_role_status = \"ready\""));
    assert!(report_source.contains("payload0005.backend-artifact"));
    assert!(report_source.contains("nustar-backend-artifact:kernel:aarch64:apple-silicon-cpu"));
}
