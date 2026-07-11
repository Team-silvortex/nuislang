use super::{
    fnv1a64_hex, main_test_support::empty_link_plan, nsld_check_report,
    nsld_emit_final_executable_image_dry_run_report, nsld_emit_final_executable_layout_plan_report,
    nsld_emit_final_executable_report, nsld_emit_final_executable_writer_input_report,
    nsld_emit_final_stage_plan_report, nsld_final_executable_layout_plan_report,
    nsld_final_executable_output_report, nsld_prepare_report,
    nsld_verify_final_executable_emit_report,
};
use std::{env, fs, path::Path};

#[test]
fn final_executable_output_reports_missing_until_real_output_exists() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-output-missing-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();
    plan.final_stage.output_path = dir.join("nuis-app").display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_stage_plan_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_executable_report(Path::new("manifest.toml"), &plan).unwrap();
    let report = nsld_final_executable_output_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::nsld_final_executable_output_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(!report.present);
    assert!(!report.runnable_candidate);
    assert!(!report.matches_expected_image);
    assert!(!report.output_image_header_valid);
    assert_eq!(report.output_image_magic, None);
    assert_eq!(report.output_image_version, None);
    assert!(report
        .blockers
        .iter()
        .any(|blocker| blocker == "final-executable-output:missing"));
    assert!(report
        .blockers
        .iter()
        .any(|blocker| blocker == "final-executable-emit:not-emitted"));
    assert!(report_json.contains("\"kind\":\"nsld_final_executable_output\""));
    assert!(report_json.contains("\"present\":false"));
    assert!(report_json.contains("\"output_image_header_valid\":false"));
    assert!(report_json.contains("\"output_image_magic\":null"));
    assert!(report_json.contains("\"matches_expected_image\":false"));
    assert!(report_json.contains("\"final_executable_emitted\":false"));
    assert!(report_json.contains("\"final_executable_blocker_count\":"));
    assert!(report_json.contains("\"runnable_candidate\":false"));
    assert!(report_json.contains("\"final-executable-output:missing\""));
    assert!(report_json.contains("\"final-executable-emit:not-emitted\""));
}

#[test]
fn self_contained_final_executable_emit_writes_nsld_owned_output() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-output-present-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();
    plan.final_stage.kind = "nuis-self-contained-image".to_owned();
    plan.final_stage.driver = "nsld-internal-image-writer".to_owned();
    plan.final_stage.link_mode = "self-contained".to_owned();
    plan.final_stage.output_path = dir.join("nuis-app.nsb").display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_stage_plan_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_executable_writer_input_report(Path::new("manifest.toml"), &plan).unwrap();
    let layout = nsld_final_executable_layout_plan_report(Path::new("manifest.toml"), &plan);
    nsld_emit_final_executable_layout_plan_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_executable_image_dry_run_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit = nsld_emit_final_executable_report(Path::new("manifest.toml"), &plan).unwrap();
    let verify_emit = nsld_verify_final_executable_emit_report(Path::new("manifest.toml"), &plan);
    let output = nsld_final_executable_output_report(Path::new("manifest.toml"), &plan);
    let output_json = super::json::nsld_final_executable_output_report_json(&output);
    let image_bytes = fs::read(&emit.image_dry_run_bytes_path).unwrap();
    let output_bytes = fs::read(&plan.final_stage.output_path).unwrap();
    fs::remove_dir_all(dir).unwrap();

    assert!(emit.emitted);
    assert!(emit.can_emit_final_executable);
    assert_eq!(emit.writer_status, "ready");
    assert!(verify_emit.valid, "{:?}", verify_emit.issues);
    assert!(output.present);
    assert!(output.runnable_candidate, "{:?}", output.blockers);
    assert!(output.matches_expected_image, "{:?}", output.issues);
    assert_eq!(output.size_bytes, Some(output_bytes.len()));
    assert_eq!(output.output_hash, Some(fnv1a64_hex(&output_bytes)));
    assert!(output.output_image_header_valid);
    assert_eq!(output.output_image_magic.as_deref(), Some("NUIFIMG"));
    assert_eq!(output.output_image_version, Some(1));
    assert_eq!(output.output_image_header_size, Some(64));
    assert_eq!(output.output_payload_byte_offset, Some(64));
    assert_eq!(
        output.output_layout_hash.as_deref(),
        Some(layout.layout_hash.as_str())
    );
    assert_eq!(
        output.output_byte_map_hash.as_deref(),
        Some(layout.byte_map_hash.as_str())
    );
    assert_eq!(output.expected_image_size_bytes, Some(image_bytes.len()));
    assert_eq!(output.expected_image_hash, Some(fnv1a64_hex(&image_bytes)));
    assert_eq!(image_bytes, output_bytes);
    assert!(output_json.contains("\"present\":true"));
    assert!(output_json.contains("\"output_image_header_valid\":true"));
    assert!(output_json.contains("\"output_image_magic\":\"NUIFIMG\""));
    assert!(output_json.contains("\"output_image_version\":1"));
    assert!(output_json.contains("\"output_layout_hash\":\"0x"));
    assert!(output_json.contains("\"output_byte_map_hash\":\"0x"));
    assert!(output_json.contains("\"matches_expected_image\":true"));
    assert!(output_json.contains("\"final_stage_plan_valid\":true"));
    assert!(output_json.contains("\"final_executable_emit_valid\":true"));
    assert!(output_json.contains("\"final_executable_emitted\":true"));
    assert!(output_json.contains("\"runnable_candidate\":true"));
    assert!(output_json.contains("\"blockers\":[]"));
    assert!(output_json.contains("\"issues\":[]"));
}

#[test]
fn final_executable_output_rejects_tampered_output_bytes() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-output-tamper-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();
    plan.final_stage.kind = "nuis-self-contained-image".to_owned();
    plan.final_stage.driver = "nsld-internal-image-writer".to_owned();
    plan.final_stage.link_mode = "self-contained".to_owned();
    plan.final_stage.output_path = dir.join("nuis-app.nsb").display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_stage_plan_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_executable_writer_input_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_executable_layout_plan_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_executable_image_dry_run_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_executable_report(Path::new("manifest.toml"), &plan).unwrap();
    fs::write(&plan.final_stage.output_path, b"tampered-final-output").unwrap();

    let output = nsld_final_executable_output_report(Path::new("manifest.toml"), &plan);
    let output_json = super::json::nsld_final_executable_output_report_json(&output);
    let check = nsld_check_report(Path::new("manifest.toml"), &plan);
    fs::remove_dir_all(dir).unwrap();

    assert!(output.present);
    assert!(!output.runnable_candidate);
    assert!(!output.matches_expected_image);
    assert!(!output.output_image_header_valid);
    assert!(output
        .blockers
        .iter()
        .any(|blocker| blocker == "final-executable-output:image-header-invalid"));
    assert!(output
        .blockers
        .iter()
        .any(|blocker| blocker == "final-executable-output:size-mismatch"));
    assert!(output
        .blockers
        .iter()
        .any(|blocker| blocker == "final-executable-output:hash-mismatch"));
    assert!(!check.valid);
    assert_eq!(
        check.final_executable_output_image_header_valid,
        Some(false)
    );
    assert_eq!(
        check.final_executable_output_runnable_candidate,
        Some(false)
    );
    assert!(check
        .final_executable_output_issues
        .iter()
        .any(|issue| issue.contains("image-header-invalid")));
    assert!(check
        .final_executable_output_issues
        .iter()
        .any(|issue| issue.contains("hash mismatch")));
    assert!(output_json.contains("\"present\":true"));
    assert!(output_json.contains("\"output_image_header_valid\":false"));
    assert!(output_json.contains("\"matches_expected_image\":false"));
    assert!(output_json.contains("\"runnable_candidate\":false"));
    assert!(output_json.contains("\"final-executable-output:image-header-invalid\""));
    assert!(output_json.contains("\"final-executable-output:size-mismatch\""));
    assert!(output_json.contains("\"final-executable-output:hash-mismatch\""));
}
