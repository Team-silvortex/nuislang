use super::{
    main_test_support::empty_link_plan, nsld_emit_final_executable_image_dry_run_report,
    nsld_emit_final_executable_report, nsld_final_executable_image_dry_run_report,
    nsld_prepare_report, nsld_verify_final_executable_emit_report,
    nsld_verify_final_executable_image_dry_run_report,
};
use std::{env, fs, path::Path};

#[test]
fn final_executable_image_dry_run_emit_and_verify_round_trip() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-image-dry-run-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let report = nsld_final_executable_image_dry_run_report(Path::new("manifest.toml"), &plan);
    let emit =
        nsld_emit_final_executable_image_dry_run_report(Path::new("manifest.toml"), &plan).unwrap();
    let verify =
        nsld_verify_final_executable_image_dry_run_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::nsld_final_executable_image_dry_run_report_json(&report);
    let emit_json = super::json::nsld_final_executable_image_dry_run_emit_report_json(&emit);
    let verify_json = super::json::nsld_final_executable_image_dry_run_verify_report_json(&verify);
    let report_source = fs::read_to_string(&emit.output_path).unwrap();
    let image_bytes = fs::read(&emit.image_path).unwrap();
    fs::remove_dir_all(dir).unwrap();

    assert!(report.image_constructed);
    assert!(report.image_ready, "{:?}", report.blockers);
    assert!(report
        .image_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("0x")));
    assert!(emit.image_emitted);
    assert_eq!(emit.image_hash, report.image_hash);
    assert_eq!(emit.image_size_bytes, Some(image_bytes.len()));
    assert!(image_bytes.starts_with(b"NUIFIMG\0"));
    assert_eq!(
        u32::from_le_bytes(image_bytes[8..12].try_into().unwrap()),
        1
    );
    assert_eq!(
        u32::from_le_bytes(image_bytes[12..16].try_into().unwrap()),
        report.image_header_size as u32
    );
    assert_eq!(
        u32::from_le_bytes(image_bytes[16..20].try_into().unwrap()),
        report.payload_count as u32
    );
    assert_eq!(
        u64::from_le_bytes(image_bytes[24..32].try_into().unwrap()),
        report.payload_byte_span as u64
    );
    assert_eq!(
        u64::from_le_bytes(image_bytes[32..40].try_into().unwrap()),
        report.payload_byte_offset as u64
    );
    assert_eq!(
        emit.image_size_bytes,
        Some(report.image_header_size + report.payload_byte_span)
    );
    assert_eq!(
        report.scheduler_metadata_payload_id,
        "payload0004.scheduler-metadata"
    );
    assert!(report.scheduler_metadata_present);
    assert!(report
        .scheduler_metadata_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("0x")));
    assert!(report
        .scheduler_metadata_offset
        .is_some_and(|offset| offset < report.payload_byte_span));
    assert_eq!(report.backend_artifact_payload_count, 0);
    assert_eq!(report.backend_artifact_payload_present_count, 0);
    assert_eq!(report.backend_artifact_payload_role_status, "none");
    assert!(report.backend_artifact_payload_ids.is_empty());
    assert!(report.backend_artifact_payload_kinds.is_empty());
    assert_eq!(report.backend_artifact_payload_first_missing, None);
    assert_eq!(
        report.relocation_application_strategy,
        "nsb-loader-relocation-table"
    );
    assert!(report.relocation_application_count > 0);
    assert!(report.relocation_application_table_hash.starts_with("0x"));
    assert_eq!(report.relocation_application_audit_status, "ready");
    assert_eq!(
        report.relocation_application_audit_count,
        report.relocation_application_count
    );
    assert!(report
        .relocation_application_audit_table_hash
        .starts_with("0x"));
    assert!(report.relocation_application_audit_blockers.is_empty());
    assert_eq!(report.relocation_patch_preview_status, "resolved");
    assert_eq!(
        report.relocation_patch_preview_count,
        report.relocation_application_count
    );
    assert!(report.relocation_patch_preview_table_hash.starts_with("0x"));
    assert_eq!(
        report.relocation_patch_preview_count,
        report.relocation_patch_previews.len()
    );
    assert!(report.relocation_patch_previews.iter().all(|record| {
        record.patch_kind == "u64-le-resolved-image-offset"
            && record.patch_width_bytes == 8
            && record.resolved_patch_value.is_some()
            && record.patch_value_hash.starts_with("0x")
            && record.target_symbol_image_offset.is_some()
            && record.preview_status == "resolved"
            && record.resolver_status == "resolved"
    }));
    assert_eq!(report.relocation_patch_application_status, "applied");
    assert_eq!(
        report.relocation_patch_application_count,
        report.relocation_patch_preview_count
    );
    assert!(report
        .relocation_patch_application_table_hash
        .starts_with("0x"));
    assert!(report.relocation_patch_application_blockers.is_empty());
    let first_patch = report.relocation_patch_previews.first().unwrap();
    let patch_start = first_patch.patch_offset;
    let patch_end = patch_start + first_patch.patch_width_bytes;
    let patched_value = u64::from_le_bytes(
        image_bytes[patch_start..patch_end]
            .try_into()
            .expect("8-byte relocation patch"),
    ) as usize;
    assert_eq!(Some(patched_value), first_patch.resolved_patch_value);
    assert!(verify.valid, "{:?}", verify.issues);
    assert_eq!(verify.actual_image_hash, emit.image_hash);
    assert_eq!(
        verify.actual_image_header_size,
        Some(report.image_header_size)
    );
    assert_eq!(
        verify.actual_payload_byte_offset,
        Some(report.payload_byte_offset)
    );
    assert_eq!(verify.actual_image_magic.as_deref(), Some("NUIFIMG"));
    assert_eq!(verify.actual_image_version, Some(1));
    assert_eq!(
        verify.actual_payload_byte_span,
        Some(report.payload_byte_span)
    );
    assert_eq!(
        verify.actual_header_layout_hash.as_deref(),
        Some(report.layout_hash.as_str())
    );
    assert_eq!(
        verify.actual_header_byte_map_hash.as_deref(),
        Some(report.byte_map_hash.as_str())
    );
    assert_eq!(
        verify.expected_payload_region_count,
        verify.actual_payload_region_count.unwrap()
    );
    assert_eq!(
        verify.expected_payload_region_hash,
        verify.actual_payload_region_hash
    );
    assert_eq!(
        verify.expected_scheduler_metadata_payload_id,
        "payload0004.scheduler-metadata"
    );
    assert_eq!(
        verify.actual_scheduler_metadata_payload_id.as_deref(),
        Some("payload0004.scheduler-metadata")
    );
    assert!(verify.expected_scheduler_metadata_present);
    assert_eq!(verify.actual_scheduler_metadata_present, Some(true));
    assert_eq!(
        verify.expected_scheduler_metadata_offset,
        report.scheduler_metadata_offset
    );
    assert_eq!(
        verify.actual_scheduler_metadata_offset,
        report.scheduler_metadata_offset
    );
    assert_eq!(
        verify.expected_scheduler_metadata_hash,
        report.scheduler_metadata_hash
    );
    assert_eq!(
        verify.actual_scheduler_metadata_hash,
        report.scheduler_metadata_hash
    );
    assert_eq!(
        verify.expected_relocation_application_strategy,
        "nsb-loader-relocation-table"
    );
    assert_eq!(
        verify.actual_relocation_application_strategy.as_deref(),
        Some("nsb-loader-relocation-table")
    );
    assert_eq!(
        verify.expected_relocation_application_count,
        report.relocation_application_count
    );
    assert_eq!(
        verify.actual_relocation_application_count,
        Some(report.relocation_application_count)
    );
    assert_eq!(
        verify.expected_relocation_application_table_hash,
        report.relocation_application_table_hash
    );
    assert_eq!(
        verify.actual_relocation_application_table_hash.as_deref(),
        Some(report.relocation_application_table_hash.as_str())
    );
    assert_eq!(verify.expected_relocation_application_audit_status, "ready");
    assert_eq!(
        verify.actual_relocation_application_audit_status.as_deref(),
        Some("ready")
    );
    assert_eq!(
        verify.expected_relocation_application_audit_count,
        report.relocation_application_audit_count
    );
    assert_eq!(
        verify.actual_relocation_application_audit_count,
        Some(report.relocation_application_audit_count)
    );
    assert_eq!(
        verify.expected_relocation_application_audit_table_hash,
        report.relocation_application_audit_table_hash
    );
    assert_eq!(
        verify
            .actual_relocation_application_audit_table_hash
            .as_deref(),
        Some(report.relocation_application_audit_table_hash.as_str())
    );
    assert!(verify
        .expected_relocation_application_audit_blockers
        .is_empty());
    assert!(verify
        .actual_relocation_application_audit_blockers
        .is_empty());
    assert_eq!(verify.expected_relocation_patch_preview_status, "resolved");
    assert_eq!(
        verify.actual_relocation_patch_preview_status.as_deref(),
        Some("resolved")
    );
    assert_eq!(
        verify.expected_relocation_patch_preview_count,
        report.relocation_patch_preview_count
    );
    assert_eq!(
        verify.actual_relocation_patch_preview_count,
        Some(report.relocation_patch_preview_count)
    );
    assert_eq!(
        verify.expected_relocation_patch_preview_table_hash,
        report.relocation_patch_preview_table_hash
    );
    assert_eq!(
        verify.actual_relocation_patch_preview_table_hash.as_deref(),
        Some(report.relocation_patch_preview_table_hash.as_str())
    );
    assert_eq!(
        verify.expected_relocation_patch_preview_entry_count,
        report.relocation_patch_previews.len()
    );
    assert_eq!(
        verify.actual_relocation_patch_preview_entry_count,
        Some(report.relocation_patch_previews.len())
    );
    assert_eq!(
        verify
            .actual_relocation_patch_preview_record_table_hash
            .as_deref(),
        Some(report.relocation_patch_preview_table_hash.as_str())
    );
    assert!(verify
        .actual_payload_region_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("0x")));
    assert!(report_source.contains("schema = \"nuis-nsld-final-executable-image-dry-run-v1\""));
    assert!(report_source.contains("image_magic = \"NUIFIMG\""));
    assert!(report_source.contains("image_header_size = 64"));
    assert!(report_source
        .contains("scheduler_metadata_payload_id = \"payload0004.scheduler-metadata\""));
    assert!(report_source.contains("scheduler_metadata_present = true"));
    assert!(
        report_source.contains("relocation_application_strategy = \"nsb-loader-relocation-table\"")
    );
    assert!(report_source.contains("relocation_application_table_hash = \"0x"));
    assert!(report_source.contains("relocation_application_audit_status = \"ready\""));
    assert!(report_source.contains("relocation_application_audit_blockers = []"));
    assert!(report_source.contains("relocation_patch_preview_status = \"resolved\""));
    assert!(report_source.contains("relocation_patch_preview_table_hash = \"0x"));
    assert!(report_source.contains("relocation_patch_application_status = \"applied\""));
    assert!(report_source.contains("relocation_patch_application_table_hash = \"0x"));
    assert!(report_source.contains("relocation_patch_application_blockers = []"));
    assert!(report_source.contains("[[relocation_patch_preview]]"));
    assert!(report_source.contains("patch_kind = \"u64-le-resolved-image-offset\""));
    assert!(report_source.contains("resolved_patch_value = "));
    assert!(report_source.contains("target_symbol_image_offset = "));
    assert!(report_source.contains("resolver_status = \"resolved\""));
    assert!(report_json.contains("\"kind\":\"nsld_final_executable_image_dry_run\""));
    assert!(report_json.contains("\"image_magic\":\"NUIFIMG\""));
    assert!(report_json
        .contains("\"scheduler_metadata_payload_id\":\"payload0004.scheduler-metadata\""));
    assert!(report_json.contains("\"scheduler_metadata_present\":true"));
    assert!(
        report_json.contains("\"relocation_application_strategy\":\"nsb-loader-relocation-table\"")
    );
    assert!(report_json.contains("\"relocation_application_table_hash\":\"0x"));
    assert!(report_json.contains("\"relocation_application_audit_status\":\"ready\""));
    assert!(report_json.contains("\"relocation_application_audit_blockers\":[]"));
    assert!(report_json.contains("\"relocation_patch_preview_status\":\"resolved\""));
    assert!(report_json.contains("\"relocation_patch_preview_table_hash\":\"0x"));
    assert!(report_json.contains("\"relocation_patch_application_status\":\"applied\""));
    assert!(report_json.contains("\"relocation_patch_application_table_hash\":\"0x"));
    assert!(report_json.contains("\"relocation_patch_application_blockers\":[]"));
    assert!(report_json.contains("\"relocation_patch_previews\":["));
    assert!(report_json.contains("\"patch_kind\":\"u64-le-resolved-image-offset\""));
    assert!(report_json.contains("\"resolved_patch_value\":"));
    assert!(report_json.contains("\"target_symbol_image_offset\":"));
    assert!(report_json.contains("\"resolver_status\":\"resolved\""));
    assert!(emit_json.contains("\"kind\":\"nsld_final_executable_image_dry_run_emit\""));
    assert!(emit_json.contains("\"image_header_size\":64"));
    assert!(verify_json.contains("\"kind\":\"nsld_final_executable_image_dry_run_verify\""));
    assert!(verify_json.contains("\"actual_image_magic\":\"NUIFIMG\""));
    assert!(verify_json.contains("\"actual_image_version\":1"));
    assert!(verify_json.contains("\"actual_header_layout_hash\":\"0x"));
    assert!(verify_json.contains("\"actual_payload_region_hash\":\"0x"));
    assert!(verify_json
        .contains("\"actual_scheduler_metadata_payload_id\":\"payload0004.scheduler-metadata\""));
    assert!(verify_json.contains("\"actual_scheduler_metadata_present\":true"));
    assert!(verify_json
        .contains("\"actual_relocation_application_strategy\":\"nsb-loader-relocation-table\""));
    assert!(verify_json.contains("\"actual_relocation_application_table_hash\":\"0x"));
    assert!(verify_json.contains("\"actual_relocation_application_audit_status\":\"ready\""));
    assert!(verify_json.contains("\"actual_relocation_application_audit_blockers\":[]"));
    assert!(verify_json.contains("\"actual_relocation_patch_preview_status\":\"resolved\""));
    assert!(verify_json.contains("\"actual_relocation_patch_preview_table_hash\":\"0x"));
    assert!(verify_json.contains("\"actual_relocation_patch_preview_entry_count\":"));
    assert!(verify_json.contains("\"actual_relocation_patch_preview_record_table_hash\":\"0x"));
    assert!(verify_json.contains("\"actual_relocation_patch_application_status\":\"applied\""));
    assert!(verify_json.contains("\"actual_relocation_patch_application_table_hash\":\"0x"));
    assert!(verify_json.contains("\"actual_relocation_patch_application_blockers\":[]"));
    assert!(verify_json.contains("\"actual_relocation_patch_byte_audit_status\":\"verified\""));
    assert!(verify_json.contains("\"actual_relocation_patch_byte_audit_hash\":\"0x"));
    assert!(verify_json.contains("\"actual_relocation_patch_byte_audit_blockers\":[]"));
    assert!(verify_json.contains("\"actual_image_header_size\":64"));
    assert!(verify_json.contains("\"valid\":true"));
}

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

#[test]
fn verify_final_executable_image_dry_run_reports_header_magic_drift() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-image-dry-run-header-drift-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit =
        nsld_emit_final_executable_image_dry_run_report(Path::new("manifest.toml"), &plan).unwrap();
    let mut image_bytes = fs::read(&emit.image_path).unwrap();
    image_bytes[0..8].copy_from_slice(b"BADMAGC\0");
    fs::write(&emit.image_path, image_bytes).unwrap();
    let verify =
        nsld_verify_final_executable_image_dry_run_report(Path::new("manifest.toml"), &plan);
    let verify_json = super::json::nsld_final_executable_image_dry_run_verify_report_json(&verify);
    fs::remove_dir_all(dir).unwrap();

    assert!(!verify.valid);
    assert_eq!(verify.actual_image_magic.as_deref(), Some("BADMAGC"));
    assert_eq!(verify.actual_image_version, Some(1));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue == "image_header_magic mismatch: expected NUIFIMG, found BADMAGC"));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("image_bytes_hash mismatch: expected 0x")));
    assert!(verify_json.contains("\"actual_image_magic\":\"BADMAGC\""));
    assert!(verify_json.contains("image_header_magic mismatch"));
}

#[test]
fn verify_final_executable_image_dry_run_reports_image_byte_drift() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-image-dry-run-drift-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit =
        nsld_emit_final_executable_image_dry_run_report(Path::new("manifest.toml"), &plan).unwrap();
    let report_source = fs::read_to_string(&emit.output_path).unwrap();
    fs::write(
        &emit.output_path,
        report_source.replace("blockers = []", "blockers = [\"tampered-image-blocker\"]"),
    )
    .unwrap();
    fs::write(&emit.image_path, b"drifted-image").unwrap();
    let verify =
        nsld_verify_final_executable_image_dry_run_report(Path::new("manifest.toml"), &plan);
    let verify_json = super::json::nsld_final_executable_image_dry_run_verify_report_json(&verify);
    fs::remove_dir_all(dir).unwrap();

    assert!(!verify.valid);
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("image_bytes_hash mismatch: expected 0x")));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue == "final-executable-image-header:invalid-or-too-short"));
    assert!(verify.expected_blockers.is_empty());
    assert_eq!(
        verify.actual_blockers,
        vec!["tampered-image-blocker".to_owned()]
    );
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue == "blockers mismatch: expected [], found [tampered-image-blocker]"));
    assert!(verify_json.contains("\"valid\":false"));
    assert!(verify_json.contains("\"actual_blockers\":[\"tampered-image-blocker\"]"));
}

#[test]
fn verify_final_executable_image_dry_run_reports_payload_region_drift() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-image-dry-run-payload-drift-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit =
        nsld_emit_final_executable_image_dry_run_report(Path::new("manifest.toml"), &plan).unwrap();
    let mut image_bytes = fs::read(&emit.image_path).unwrap();
    image_bytes[64] ^= 0xff;
    fs::write(&emit.image_path, image_bytes).unwrap();
    let verify =
        nsld_verify_final_executable_image_dry_run_report(Path::new("manifest.toml"), &plan);
    let verify_json = super::json::nsld_final_executable_image_dry_run_verify_report_json(&verify);
    fs::remove_dir_all(dir).unwrap();

    assert!(!verify.valid);
    assert_eq!(verify.actual_image_magic.as_deref(), Some("NUIFIMG"));
    assert_ne!(
        verify.expected_payload_region_hash,
        verify.actual_payload_region_hash
    );
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("image_payload_region_hash mismatch")));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("image_payload_region_entry_hash mismatch for ")));
    assert!(verify_json.contains("\"actual_payload_region_hash\":\"0x"));
    assert!(verify_json.contains("image_payload_region_hash mismatch"));
}

#[test]
fn verify_final_executable_image_dry_run_reports_patch_byte_drift() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-image-dry-run-patch-byte-drift-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let report = nsld_final_executable_image_dry_run_report(Path::new("manifest.toml"), &plan);
    let emit =
        nsld_emit_final_executable_image_dry_run_report(Path::new("manifest.toml"), &plan).unwrap();
    let first_patch = report.relocation_patch_previews.first().unwrap();
    let mut image_bytes = fs::read(&emit.image_path).unwrap();
    image_bytes[first_patch.patch_offset] ^= 0x7f;
    fs::write(&emit.image_path, image_bytes).unwrap();
    let verify =
        nsld_verify_final_executable_image_dry_run_report(Path::new("manifest.toml"), &plan);
    let verify_json = super::json::nsld_final_executable_image_dry_run_verify_report_json(&verify);
    fs::remove_dir_all(dir).unwrap();

    assert!(!verify.valid);
    assert_eq!(
        verify.actual_relocation_patch_byte_audit_status.as_deref(),
        Some("blocked")
    );
    assert_eq!(
        verify.actual_relocation_patch_byte_audit_count,
        Some(report.relocation_patch_preview_count - 1)
    );
    assert!(verify
        .actual_relocation_patch_byte_audit_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("0x")));
    assert!(verify
        .actual_relocation_patch_byte_audit_blockers
        .iter()
        .any(|blocker| blocker.ends_with(":patch-value-mismatch")));
    assert!(verify.issues.iter().any(|issue| {
        issue == "relocation_patch_byte_audit_status mismatch: expected verified, found blocked"
    }));
    assert!(verify.issues.iter().any(|issue| {
        issue.starts_with("relocation_patch_byte_audit_blockers mismatch: expected [], found [")
    }));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("image_bytes_hash mismatch: expected 0x")));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("image_payload_region_hash mismatch")));
    assert!(verify_json.contains("\"actual_relocation_patch_byte_audit_status\":\"blocked\""));
    assert!(verify_json.contains("\"actual_relocation_patch_byte_audit_hash\":\"0x"));
    assert!(verify_json.contains("patch-value-mismatch"));
}

#[test]
fn verify_final_executable_image_dry_run_reports_patch_preview_record_drift() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-image-dry-run-patch-preview-drift-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit =
        nsld_emit_final_executable_image_dry_run_report(Path::new("manifest.toml"), &plan).unwrap();
    let report_source = fs::read_to_string(&emit.output_path).unwrap();
    fs::write(
        &emit.output_path,
        report_source.replacen(
            "patch_kind = \"u64-le-resolved-image-offset\"",
            "patch_kind = \"tampered-placeholder\"",
            1,
        ),
    )
    .unwrap();
    let verify =
        nsld_verify_final_executable_image_dry_run_report(Path::new("manifest.toml"), &plan);
    let verify_json = super::json::nsld_final_executable_image_dry_run_verify_report_json(&verify);
    fs::remove_dir_all(dir).unwrap();

    assert!(!verify.valid);
    assert_eq!(
        verify.actual_relocation_patch_preview_entry_count,
        Some(verify.expected_relocation_patch_preview_entry_count)
    );
    assert_ne!(
        verify.expected_relocation_patch_preview_table_hash,
        verify
            .actual_relocation_patch_preview_record_table_hash
            .clone()
            .unwrap()
    );
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue == "final-executable-image-dry-run-content-mismatch"));
    assert!(verify.issues.iter().any(|issue| issue
        .starts_with("relocation_patch_preview_record_table_hash mismatch: expected 0x")));
    assert!(verify_json.contains("\"actual_relocation_patch_preview_entry_count\":"));
    assert!(verify_json.contains("\"actual_relocation_patch_preview_record_table_hash\":\"0x"));
    assert!(verify_json.contains("relocation_patch_preview_record_table_hash mismatch"));
}

#[test]
fn emit_final_executable_consumes_valid_image_dry_run_snapshot() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-valid-image-dry-run-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let image =
        nsld_emit_final_executable_image_dry_run_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit = nsld_emit_final_executable_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit_json = super::json::nsld_final_executable_emit_report_json(&emit);
    let verify = nsld_verify_final_executable_emit_report(Path::new("manifest.toml"), &plan);
    fs::remove_dir_all(dir).unwrap();

    assert_eq!(emit.image_dry_run_valid, Some(true));
    assert_eq!(emit.image_dry_run_hash, image.image_hash);
    assert_eq!(emit.image_dry_run_size_bytes, image.image_size_bytes);
    assert_eq!(
        emit.image_dry_run_resolver_status.as_deref(),
        Some("resolved")
    );
    assert_eq!(
        emit.image_dry_run_patch_application_status.as_deref(),
        Some("applied")
    );
    assert_eq!(
        emit.image_dry_run_patch_byte_audit_status.as_deref(),
        Some("verified")
    );
    assert!(emit
        .image_dry_run_patch_byte_audit_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("0x")));
    assert!(emit.image_dry_run_issues.is_empty());
    assert!(!emit
        .blockers
        .iter()
        .any(|blocker| blocker == "final-executable-image-dry-run:invalid"));
    assert!(emit_json.contains("\"image_dry_run_valid\":true"));
    assert!(emit_json.contains("\"image_dry_run_hash\":\"0x"));
    assert!(emit_json.contains("\"image_dry_run_size_bytes\":"));
    assert!(emit_json.contains("\"image_dry_run_resolver_status\":\"resolved\""));
    assert!(emit_json.contains("\"image_dry_run_patch_application_status\":\"applied\""));
    assert!(emit_json.contains("\"image_dry_run_patch_byte_audit_status\":\"verified\""));
    assert!(emit_json.contains("\"image_dry_run_patch_byte_audit_hash\":\"0x"));
    assert!(verify.valid, "{:?}", verify.issues);
    assert_eq!(verify.expected_image_dry_run_valid, Some(true));
    assert_eq!(verify.actual_image_dry_run_valid, Some(true));
    assert_eq!(verify.expected_image_dry_run_hash, image.image_hash);
    assert_eq!(verify.actual_image_dry_run_hash, image.image_hash);
    assert_eq!(
        verify.expected_image_dry_run_resolver_status.as_deref(),
        Some("resolved")
    );
    assert_eq!(
        verify.actual_image_dry_run_resolver_status.as_deref(),
        Some("resolved")
    );
    assert_eq!(
        verify
            .expected_image_dry_run_patch_application_status
            .as_deref(),
        Some("applied")
    );
    assert_eq!(
        verify
            .actual_image_dry_run_patch_application_status
            .as_deref(),
        Some("applied")
    );
    assert_eq!(
        verify
            .expected_image_dry_run_patch_byte_audit_status
            .as_deref(),
        Some("verified")
    );
    assert_eq!(
        verify
            .actual_image_dry_run_patch_byte_audit_status
            .as_deref(),
        Some("verified")
    );
    assert_eq!(
        verify.expected_image_dry_run_patch_byte_audit_hash,
        emit.image_dry_run_patch_byte_audit_hash
    );
    assert_eq!(
        verify.actual_image_dry_run_patch_byte_audit_hash,
        emit.image_dry_run_patch_byte_audit_hash
    );
}
