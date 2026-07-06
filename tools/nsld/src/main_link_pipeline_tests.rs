use super::{
    main_test_support::empty_link_plan, nsld_assemble_plan_report, nsld_prepare_report,
    nsld_verify_assemble_plan_report, nsld_verify_section_manifest_report,
    object_file_layout::nsld_object_file_layout_report, toml,
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
    let report_json = super::json::nsld_prepare_report_json(&report);
    let payload_bytes = fs::read(&report.container_payload_path).unwrap();
    let object_bytes = fs::read(&report.object_output_path).unwrap();
    let object_file_layout = nsld_object_file_layout_report(Path::new("manifest.toml"), &plan);
    let compiled_artifact_record = object_file_layout
        .records
        .iter()
        .find(|record| record.record_id == "section.sec0000.compiled-artifact")
        .unwrap();

    assert!(report.valid);
    assert!(report.issues.is_empty());
    assert!(Path::new(&report.link_input_table_path).exists());
    assert!(Path::new(&report.link_unit_table_path).exists());
    assert!(Path::new(&report.link_bundle_path).exists());
    assert!(Path::new(&report.assemble_plan_path).exists());
    assert!(Path::new(&report.section_manifest_path).exists());
    assert!(Path::new(&report.object_plan_path).exists());
    assert!(Path::new(&report.object_writer_input_path).exists());
    assert!(Path::new(&report.object_byte_layout_path).exists());
    assert!(Path::new(&report.object_file_layout_path).exists());
    assert!(Path::new(&report.object_image_dry_run_path).exists());
    assert!(Path::new(&report.object_image_dry_run_bytes_path).exists());
    assert!(Path::new(&report.object_emit_blocked_path).exists());
    assert!(Path::new(&report.object_output_path).exists());
    assert!(Path::new(&report.object_writer_dry_run_path).exists());
    assert!(Path::new(&report.container_plan_path).exists());
    assert!(Path::new(&report.container_path).exists());
    assert!(Path::new(&report.container_payload_path).exists());
    assert!(Path::new(&report.closure_snapshot_path).exists());
    assert!(Path::new(&report.final_stage_plan_path).exists());
    assert_eq!(
        report.final_executable_writer_input_path,
        dir.join("nuis.nsld.final-executable-writer-input.toml")
            .display()
            .to_string()
    );
    assert_eq!(
        report.final_executable_host_invoke_plan_path,
        dir.join("nuis.nsld.final-executable-host-invoke-plan.toml")
            .display()
            .to_string()
    );
    assert_eq!(
        report.final_executable_blocked_path,
        dir.join("nuis.nsld.final-executable.blocked.toml")
            .display()
            .to_string()
    );
    assert!(!Path::new(&report.final_executable_writer_input_path).exists());
    assert!(!Path::new(&report.final_executable_host_invoke_plan_path).exists());
    assert!(!Path::new(&report.final_executable_blocked_path).exists());
    assert!(dir.join("nuis.nsld.object-plan.toml").exists());
    assert!(dir.join("nuis.nsld.object-writer-input.toml").exists());
    assert!(dir.join("nuis.nsld.object.blocked.toml").exists());
    assert!(dir.join("nuis.nsld.mach-o").exists());
    assert!(dir.join("nuis.nsld.object-writer-dry-run.toml").exists());
    assert!(payload_bytes
        .windows(4)
        .any(|window| window == [0xcf, 0xfa, 0xed, 0xfe]));
    assert_eq!(
        &object_bytes[compiled_artifact_record.file_offset
            ..compiled_artifact_record.file_offset + "compiled-artifact".len()],
        b"compiled-artifact"
    );
    assert_eq!(report.link_input_count, 1);
    assert_eq!(report.unit_count, 1);
    assert!(report.bundle_ready);
    assert_ne!(report.assemble_plan_hash, "missing");
    assert_ne!(report.section_table_hash, "missing");
    assert_ne!(report.object_plan_hash, "missing");
    assert!(report.object_emitted);
    assert_ne!(report.byte_layout_hash, "missing");
    assert_ne!(report.file_layout_hash, "missing");
    assert!(report
        .object_image_hash
        .as_deref()
        .unwrap()
        .starts_with("0x"));
    assert!(report.object_image_relocation_lowering_valid);
    assert_eq!(report.object_image_relocation_lowering_rule_count, 4);
    assert_eq!(report.object_image_relocation_lowering_rules.len(), 4);
    assert_eq!(
        report.object_image_relocation_lowering_rules[0].source_seed_kind,
        "bootstrap-entry-seed"
    );
    assert!(report.object_image_relocation_lowering_issues.is_empty());
    assert_eq!(
        report.object_image_relocation_record_count,
        report.object_image_relocation_records.len()
    );
    assert!(report.object_image_relocation_record_count >= 4);
    assert!(report
        .object_image_relocation_record_table_hash
        .starts_with("0x"));
    assert_eq!(
        report.object_image_relocation_records[0].relocation_seed_id,
        "orel0000.compiled_artifact"
    );
    assert_ne!(report.metadata_table_hash, "missing");
    assert_eq!(report.compatibility_domain_count, Some(1));
    assert!(report
        .compatibility_domain_table_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("0x")));
    assert_eq!(
        report.compatibility_domain_id.as_deref(),
        Some("compat0000.cffi-von-neumann")
    );
    assert_eq!(
        report.compatibility_domain_kind.as_deref(),
        Some("cffi-host-compat")
    );
    assert_eq!(
        report.compatibility_domain_paradigm.as_deref(),
        Some("classic-von-neumann-host")
    );
    assert_eq!(
        report.compatibility_domain_lifecycle_hook.as_deref(),
        Some("on_cffi_native_object")
    );
    assert_eq!(
        report.compatibility_domain_abi_family.as_deref(),
        Some("mach-o")
    );
    assert_eq!(
        report.compatibility_domain_wrapper_policy.as_deref(),
        Some("wrapped")
    );
    assert_eq!(report.compatibility_domain_required, Some(true));
    assert!(report_json.contains("\"compatibility_domain_count\":1"));
    assert!(report_json.contains("\"compatibility_domain_table_hash\":\"0x"));
    assert!(report_json.contains("\"compatibility_domain_id\":\"compat0000.cffi-von-neumann\""));
    assert!(report_json.contains("\"compatibility_domain_kind\":\"cffi-host-compat\""));
    assert!(report_json.contains("\"compatibility_domain_paradigm\":\"classic-von-neumann-host\""));
    assert!(
        report_json.contains("\"compatibility_domain_lifecycle_hook\":\"on_cffi_native_object\"")
    );
    assert!(report_json.contains("\"compatibility_domain_abi_family\":\"mach-o\""));
    assert!(report_json.contains("\"compatibility_domain_wrapper_policy\":\"wrapped\""));
    assert!(report_json.contains("\"compatibility_domain_required\":true"));
    assert!(
        report_json.contains("\"compatibility_domain_summary\":{\"count\":1,\"table_hash\":\"0x")
    );
    assert!(report_json.contains("\"object_image_relocation_lowering_valid\":true"));
    assert!(report_json.contains("\"object_image_relocation_lowering_rule_count\":4"));
    assert!(report_json.contains("\"object_image_relocation_lowering_rules\":[{"));
    assert!(report_json.contains("\"source_seed_kind\":\"bootstrap-entry-seed\""));
    assert!(report_json.contains("\"object_image_relocation_lowering_issues\":[]"));
    assert!(report_json.contains("\"object_image_relocation_record_count\":"));
    assert!(report_json.contains("\"object_image_relocation_record_table_hash\":\"0x"));
    assert!(report_json.contains("\"object_image_relocation_records\":[{"));
    assert!(report_json.contains("\"relocation_seed_id\":\"orel0000.compiled_artifact\""));
    assert!(report_json.contains("\"closure_snapshot_path\":"));
    assert!(report_json.contains("\"final_executable_writer_input_path\":"));
    assert!(report_json.contains("\"final_executable_host_invoke_plan_path\":"));
    assert!(report_json.contains("\"final_executable_blocked_path\":"));
    assert_ne!(report.container_layout_hash, "missing");
    assert_ne!(report.container_hash, "missing");
    assert!(report.payload_size_bytes > 0);
    assert_ne!(report.payload_hash, "missing");
    assert!(!report.final_stage_plan_ready);
    assert!(report.final_stage_plan_hash.starts_with("0x"));
    assert!(report.final_stage_plan_blocker_count >= 1);
    assert!(report_json.contains("\"final_stage_plan_path\":"));
    assert!(report_json.contains("\"final_stage_plan_ready\":false"));
    assert!(report_json.contains("\"final_stage_plan_hash\":\"0x"));

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
