use super::{
    main_test_support::empty_link_plan, nsld_emit_final_executable_report,
    nsld_emit_final_stage_plan_report, nsld_final_executable_output_report, nsld_prepare_report,
};
use std::{env, fs, path::Path};

#[test]
fn final_executable_output_orders_nustar_dispatch_blockers_before_output_bytes() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-output-nustar-dispatch-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();
    plan.final_stage.output_path = dir.join("nuis-app.nsb").display().to_string();
    plan.domain_units.push(nuisc::linker::LinkPlanDomainUnit {
        kind: "heterogeneous".to_owned(),
        package_id: "official.ghost".to_owned(),
        domain_family: "ghost".to_owned(),
        abi: None,
        machine_arch: None,
        machine_os: None,
        backend_family: Some("ghost".to_owned()),
        vendor: None,
        device_class: None,
        target_device: None,
        ir_format: None,
        dispatch_abi: None,
        backend_priority: None,
        verification: None,
        selected_lowering_target: Some("ghost.backend".to_owned()),
        contract_family: "nustar.ghost".to_owned(),
        packaging_role: "heterogeneous-domain".to_owned(),
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
    });

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_stage_plan_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_executable_report(Path::new("manifest.toml"), &plan).unwrap();
    let report = nsld_final_executable_output_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::nsld_final_executable_output_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert_eq!(
        report.blockers.first().map(String::as_str),
        Some("nustar-dispatch:official.ghost:registry-unavailable")
    );
    assert_eq!(
        report.execution_handoff_first_blocker.as_deref(),
        Some("nustar-dispatch:official.ghost:registry-unavailable")
    );
    assert_eq!(report.backend_artifact_candidate_count, 1);
    assert_eq!(report.backend_artifact_ready_count, 0);
    assert_eq!(report.backend_artifact_selection_status, "blocked");
    assert_eq!(
        report.backend_artifact_ordered_candidates,
        vec!["ghost:ghost:none".to_owned()]
    );
    assert_eq!(
        report.backend_artifact_first_unready.as_deref(),
        Some("ghost:ghost:none")
    );
    assert_eq!(report.backend_artifact_selected_candidate, None);
    assert_eq!(
        report.backend_artifact_selection_reason,
        "all-candidates-blocked"
    );
    assert_eq!(report.backend_artifact_assembly_status, "not-applicable");
    assert_eq!(report.backend_artifact_selected_payload_path, None);
    assert!(!report.backend_artifact_selected_payload_consumed);
    assert_eq!(report.backend_artifact_assembly_first_blocker, None);
    assert!(report
        .blockers
        .iter()
        .any(|blocker| blocker == "nustar-backend-artifact:ghost:ghost:none:unready"));
    assert!(report.blockers.iter().any(|blocker| {
        blocker == "nustar-backend-artifact:ghost:ghost:none:missing:target_device"
    }));
    assert!(report
        .blockers
        .iter()
        .any(|blocker| blocker == "final-executable-output:missing"));
    assert!(report_json.contains("\"nustar-dispatch:official.ghost:registry-unavailable\""));
    assert!(report_json.contains("\"backend_artifact_candidate_count\":1"));
    assert!(report_json.contains("\"backend_artifact_ready_count\":0"));
    assert!(report_json.contains("\"backend_artifact_selection_status\":\"blocked\""));
    assert!(report_json.contains("\"backend_artifact_ordered_candidates\":[\"ghost:ghost:none\"]"));
    assert!(report_json.contains("\"backend_artifact_selected_candidate\":null"));
    assert!(
        report_json.contains("\"backend_artifact_selection_reason\":\"all-candidates-blocked\"")
    );
    assert!(report_json.contains("\"backend_artifact_assembly_status\":\"not-applicable\""));
    assert!(report_json.contains("\"backend_artifact_selected_payload_path\":null"));
    assert!(report_json.contains("\"backend_artifact_selected_payload_consumed\":false"));
    assert!(report_json.contains("\"backend_artifact_assembly_first_blocker\":null"));
    assert!(report_json.contains("\"backend_artifact_first_unready\":\"ghost:ghost:none\""));
    assert!(report_json.contains("\"nustar-backend-artifact:ghost:ghost:none:unready\""));
    assert!(report_json.contains("\"final-executable-output:missing\""));
}

#[test]
fn final_executable_output_selects_first_ready_backend_artifact_candidate_by_priority() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-output-backend-selection-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    let shader_payload = dir.join("shader.payload.bin");
    let shader_bridge = dir.join("shader.bridge.stub");
    let kernel_payload = dir.join("kernel.payload.bin");
    let kernel_bridge = dir.join("kernel.bridge.stub");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    fs::write(&shader_payload, b"shader-payload").unwrap();
    fs::write(&shader_bridge, b"shader-bridge").unwrap();
    fs::write(&kernel_payload, b"kernel-payload").unwrap();
    fs::write(&kernel_bridge, b"kernel-bridge").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();
    plan.final_stage.output_path = dir.join("nuis-app.nsb").display().to_string();
    plan.domain_units.push(nuisc::linker::LinkPlanDomainUnit {
        kind: "heterogeneous".to_owned(),
        package_id: "official.shader".to_owned(),
        domain_family: "shader".to_owned(),
        abi: None,
        machine_arch: None,
        machine_os: None,
        backend_family: Some("metal".to_owned()),
        vendor: Some("apple".to_owned()),
        device_class: Some("gpu".to_owned()),
        target_device: Some("apple-silicon-gpu".to_owned()),
        ir_format: Some("yir-shader".to_owned()),
        dispatch_abi: None,
        backend_priority: Some(20),
        verification: None,
        selected_lowering_target: Some("metal.apple-silicon-gpu".to_owned()),
        contract_family: "nustar.shader".to_owned(),
        packaging_role: "heterogeneous-domain".to_owned(),
        artifact_stub_path: None,
        artifact_stub_inline: None,
        artifact_payload_path: None,
        artifact_bridge_stub_path: Some(shader_bridge.display().to_string()),
        artifact_ir_sidecar_path: None,
        artifact_bridge_stub_inline: None,
        artifact_payload_blob_path: Some(shader_payload.display().to_string()),
        artifact_payload_blob_bytes: Some(14),
        artifact_payload_format: Some("nuis-shader-payload-v1".to_owned()),
        artifact_payload_blob_inline: None,
    });
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
    nsld_emit_final_stage_plan_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_executable_report(Path::new("manifest.toml"), &plan).unwrap();
    let report = nsld_final_executable_output_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::nsld_final_executable_output_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert_eq!(report.backend_artifact_candidate_count, 2);
    assert_eq!(report.backend_artifact_ready_count, 2);
    assert_eq!(report.backend_artifact_selection_status, "ready");
    assert_eq!(
        report.backend_artifact_ordered_candidates,
        vec![
            "kernel:aarch64:apple-silicon-cpu".to_owned(),
            "shader:metal:apple-silicon-gpu".to_owned(),
        ]
    );
    assert_eq!(
        report.backend_artifact_selected_candidate.as_deref(),
        Some("kernel:aarch64:apple-silicon-cpu")
    );
    assert_eq!(
        report.backend_artifact_selection_reason,
        "selected-first-ready-candidate"
    );
    assert_eq!(
        report.backend_artifact_assembly_status,
        "consumed-by-final-layout"
    );
    assert_eq!(
        report.backend_artifact_selected_payload_path.as_deref(),
        Some(kernel_payload.to_str().unwrap())
    );
    assert!(report.backend_artifact_selected_payload_consumed);
    assert_eq!(report.backend_artifact_assembly_first_blocker, None);
    assert_eq!(report.backend_artifact_first_unready, None);
    assert!(!report.blockers.iter().any(|blocker| {
        blocker
            == "nustar-backend-artifact:kernel:aarch64:apple-silicon-cpu:not-consumed-by-final-layout"
    }));
    assert!(report_json.contains("\"backend_artifact_selection_status\":\"ready\""));
    assert!(report_json
        .contains("\"backend_artifact_selected_candidate\":\"kernel:aarch64:apple-silicon-cpu\""));
    assert!(report_json
        .contains("\"backend_artifact_selection_reason\":\"selected-first-ready-candidate\""));
    assert!(
        report_json.contains("\"backend_artifact_assembly_status\":\"consumed-by-final-layout\"")
    );
    assert!(report_json.contains("\"backend_artifact_selected_payload_consumed\":true"));
    assert!(report_json.contains("\"backend_artifact_assembly_first_blocker\":null"));
}
