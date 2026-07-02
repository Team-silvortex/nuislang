use super::{
    main_test_support::empty_link_plan, nsld_assemble_plan_report, nsld_prepare_report,
    nsld_verify_assemble_plan_report, nsld_verify_section_manifest_report, toml,
};
use nuisc::linker::LinkPlanHeteroNode;
use std::{env, fs, path::Path};

#[test]
fn prepare_emits_and_verifies_all_linker_artifacts() {
    let dir = env::temp_dir().join(format!("nsld-prepare-{}", std::process::id()));
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

    let report = nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();

    assert!(report.valid);
    assert!(report.issues.is_empty());
    assert!(Path::new(&report.link_input_table_path).exists());
    assert!(Path::new(&report.link_unit_table_path).exists());
    assert!(Path::new(&report.link_bundle_path).exists());
    assert!(Path::new(&report.assemble_plan_path).exists());
    assert!(Path::new(&report.section_manifest_path).exists());
    assert!(Path::new(&report.container_plan_path).exists());
    assert!(Path::new(&report.container_path).exists());
    assert_eq!(report.link_input_count, 1);
    assert_eq!(report.unit_count, 1);
    assert!(report.bundle_ready);
    assert_ne!(report.assemble_plan_hash, "missing");
    assert_ne!(report.section_table_hash, "missing");
    assert_ne!(report.metadata_table_hash, "missing");
    assert_ne!(report.container_layout_hash, "missing");
    assert_ne!(report.container_hash, "missing");
    assert!(report.payload_size_bytes > 0);
    assert_ne!(report.payload_hash, "missing");

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn assemble_plan_lists_prepared_linker_sections() {
    let dir = env::temp_dir().join(format!("nsld-assemble-plan-{}", std::process::id()));
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
    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();

    let report = nsld_assemble_plan_report(Path::new("manifest.toml"), &plan);
    fs::remove_dir_all(dir).unwrap();

    assert!(report.ready);
    assert!(report.blockers.is_empty());
    assert_eq!(report.section_count, 5);
    assert_eq!(report.sections[0].section_kind, "compiled-artifact");
    assert_eq!(report.sections[1].section_kind, "nsld-link-input-table");
    assert_eq!(report.sections[2].section_kind, "nsld-link-unit-table");
    assert_eq!(report.sections[3].section_kind, "nsld-link-bundle");
    assert_eq!(report.sections[4].section_kind, "lowering-sidecar-input");
    assert!(report
        .sections
        .iter()
        .all(|section| section.source_hash != "missing"));
}

#[test]
fn verify_assemble_plan_accepts_matching_emitted_plan() {
    let dir = env::temp_dir().join(format!("nsld-assemble-plan-verify-{}", std::process::id()));
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
    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let assemble_plan = nsld_assemble_plan_report(Path::new("manifest.toml"), &plan);
    fs::write(
        dir.join("nuis.nsld.assemble-plan.toml"),
        toml::render_assemble_plan(&assemble_plan),
    )
    .unwrap();

    let report = nsld_verify_assemble_plan_report(Path::new("manifest.toml"), &plan);
    fs::remove_dir_all(dir).unwrap();

    assert!(report.valid);
    assert!(report.issues.is_empty());
    assert_eq!(
        report.actual_assemble_plan_hash,
        Some(assemble_plan.assemble_plan_hash)
    );
    assert_eq!(
        report.actual_section_count,
        Some(assemble_plan.section_count)
    );
}

#[test]
fn verify_section_manifest_accepts_matching_emitted_manifest() {
    let dir = env::temp_dir().join(format!(
        "nsld-section-manifest-verify-{}",
        std::process::id()
    ));
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
    let source = fs::read_to_string(&prepare.section_manifest_path).unwrap();
    fs::write(dir.join("nuis.nsld.section-manifest.toml"), source).unwrap();

    let report = nsld_verify_section_manifest_report(Path::new("manifest.toml"), &plan);
    fs::remove_dir_all(dir).unwrap();

    assert!(report.valid);
    assert!(report.issues.is_empty());
    assert_eq!(report.actual_section_count, Some(5));
    assert_eq!(
        report.actual_section_table_hash,
        Some(prepare.section_table_hash)
    );
}
