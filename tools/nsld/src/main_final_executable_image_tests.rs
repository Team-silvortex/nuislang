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
    assert!(report_json.contains("\"kind\":\"nsld_final_executable_image_dry_run\""));
    assert!(report_json.contains("\"image_magic\":\"NUIFIMG\""));
    assert!(report_json
        .contains("\"scheduler_metadata_payload_id\":\"payload0004.scheduler-metadata\""));
    assert!(report_json.contains("\"scheduler_metadata_present\":true"));
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
    assert!(verify_json.contains("\"actual_image_header_size\":64"));
    assert!(verify_json.contains("\"valid\":true"));
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
    assert!(emit.image_dry_run_issues.is_empty());
    assert!(!emit
        .blockers
        .iter()
        .any(|blocker| blocker == "final-executable-image-dry-run:invalid"));
    assert!(emit_json.contains("\"image_dry_run_valid\":true"));
    assert!(emit_json.contains("\"image_dry_run_hash\":\"0x"));
    assert!(emit_json.contains("\"image_dry_run_size_bytes\":"));
    assert!(verify.valid, "{:?}", verify.issues);
    assert_eq!(verify.expected_image_dry_run_valid, Some(true));
    assert_eq!(verify.actual_image_dry_run_valid, Some(true));
    assert_eq!(verify.expected_image_dry_run_hash, image.image_hash);
    assert_eq!(verify.actual_image_dry_run_hash, image.image_hash);
}
