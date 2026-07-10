use super::{
    main_container_verify_assertions::{
        assert_corrupted_payload_report, assert_matching_container_artifacts,
        assert_matching_container_verify_json, assert_matching_container_verify_report,
        assert_tampered_container_report,
    },
    main_test_support::empty_link_plan,
    nsld_container_report, nsld_prepare_report, nsld_verify_container_report,
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
    let container_source = fs::read_to_string(&prepare.container_path).unwrap();
    let payload_bytes = fs::read(&prepare.container_payload_path).unwrap();
    let preview = nsld_container_report(Path::new("manifest.toml"), &plan);

    let report = nsld_verify_container_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::nsld_container_verify_report_json(&report);

    assert_matching_container_artifacts(&preview, &prepare, &payload_bytes, &container_source);
    assert_matching_container_verify_report(&report);
    assert_eq!(
        report.actual_container_layout_hash,
        Some(prepare.container_layout_hash)
    );
    assert_eq!(report.actual_container_hash, Some(prepare.container_hash));
    assert_matching_container_verify_json(&report_json);

    fs::write(
        &prepare.container_path,
        tampered_container_source(&container_source, &report),
    )
    .unwrap();
    let tampered_report = nsld_verify_container_report(Path::new("manifest.toml"), &plan);
    let tampered_json = super::json::nsld_container_verify_report_json(&tampered_report);
    assert_tampered_container_report(&tampered_report, &tampered_json);
    fs::write(&prepare.container_path, container_source).unwrap();

    let mut corrupted_payload = payload_bytes;
    corrupted_payload[0] ^= 0xff;
    fs::write(&prepare.container_payload_path, corrupted_payload).unwrap();
    let corrupted_report = nsld_verify_container_report(Path::new("manifest.toml"), &plan);
    assert_corrupted_payload_report(&corrupted_report);
    fs::remove_dir_all(dir).unwrap();
}
