use super::{
    fnv1a64_hex, main_test_support::empty_link_plan, nsld_check_report,
    nsld_emit_final_executable_host_invoke_plan_report,
    nsld_emit_final_executable_image_dry_run_report, nsld_emit_final_executable_layout_plan_report,
    nsld_emit_final_executable_report, nsld_emit_final_executable_writer_input_report,
    nsld_emit_final_stage_plan_report, nsld_prepare_report,
};
use std::{env, fs, path::Path};

#[test]
fn check_reports_valid_final_executable_writer_input_when_present() {
    let dir = env::temp_dir().join(format!(
        "nsld-check-final-executable-writer-input-{}",
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
        nsld_emit_final_executable_writer_input_report(Path::new("manifest.toml"), &plan).unwrap();
    let report = nsld_check_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::check_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(report.valid, "{:?}", report.issues);
    assert!(report.final_executable_writer_input_present);
    assert_eq!(report.final_executable_writer_input_valid, Some(true));
    assert_eq!(
        report.final_executable_writer_input_hash.as_deref(),
        Some(emit.writer_input_hash.as_str())
    );
    assert_eq!(
        report.final_executable_writer_input_command_arg_count,
        Some(emit.command_arg_count)
    );
    assert!(report.final_executable_writer_input_issues.is_empty());
    assert!(report_json.contains("\"final_executable_writer_input_present\":true"));
    assert!(report_json.contains("\"final_executable_writer_input_valid\":true"));
    assert!(report_json.contains("\"final_executable_writer_input_hash\":\"0x"));
    assert!(report_json.contains("\"final_executable_writer_input_command_arg_count\":"));
}

#[test]
fn check_reports_tampered_final_executable_writer_input() {
    let dir = env::temp_dir().join(format!(
        "nsld-check-final-executable-writer-input-drift-{}",
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
        nsld_emit_final_executable_writer_input_report(Path::new("manifest.toml"), &plan).unwrap();
    let writer_input_source = fs::read_to_string(&emit.output_path).unwrap();
    fs::write(
        &emit.output_path,
        writer_input_source.replace(
            &format!("command_arg_count = {}", emit.command_arg_count),
            "command_arg_count = 999",
        ),
    )
    .unwrap();
    let report = nsld_check_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::check_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(!report.valid);
    assert!(report.final_executable_writer_input_present);
    assert_eq!(report.final_executable_writer_input_valid, Some(false));
    assert_eq!(
        report.final_executable_writer_input_command_arg_count,
        Some(999)
    );
    assert!(report
        .final_executable_writer_input_issues
        .iter()
        .any(|issue| issue.starts_with("command_arg_count mismatch: expected ")));
    assert!(report
        .issues
        .iter()
        .any(|issue| issue == "final executable writer input verification failed"));
    assert!(report_json.contains("\"final_executable_writer_input_present\":true"));
    assert!(report_json.contains("\"final_executable_writer_input_valid\":false"));
    assert!(report_json.contains("\"final_executable_writer_input_command_arg_count\":999"));
}

#[test]
fn check_reports_valid_final_executable_host_invoke_plan_when_present() {
    let dir = env::temp_dir().join(format!(
        "nsld-check-final-executable-host-invoke-plan-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_executable_writer_input_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit =
        nsld_emit_final_executable_host_invoke_plan_report(Path::new("manifest.toml"), &plan)
            .unwrap();
    let report = nsld_check_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::check_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(report.valid, "{:?}", report.issues);
    assert!(report.final_executable_host_invoke_plan_present);
    assert_eq!(report.final_executable_host_invoke_plan_valid, Some(true));
    assert_eq!(
        report.final_executable_host_invoke_plan_hash.as_deref(),
        Some(emit.invoke_plan_hash.as_str())
    );
    assert_eq!(
        report
            .final_executable_host_invoke_plan_invocation_policy
            .as_deref(),
        Some(emit.invocation_policy.as_str())
    );
    assert_eq!(
        report.final_executable_host_invoke_plan_requires_explicit_allow,
        Some(true)
    );
    assert_eq!(
        report.final_executable_host_invoke_plan_explicit_allow_present,
        Some(false)
    );
    assert_eq!(
        report.final_executable_host_invoke_plan_would_invoke,
        Some(false)
    );
    assert_eq!(
        report.final_executable_host_invoke_plan_blocker_count,
        Some(emit.blocker_count)
    );
    assert!(report.final_executable_host_invoke_plan_issues.is_empty());
    assert!(report_json.contains("\"final_executable_host_invoke_plan_present\":true"));
    assert!(report_json.contains("\"final_executable_host_invoke_plan_valid\":true"));
    assert!(report_json.contains("\"final_executable_host_invoke_plan_hash\":\"0x"));
    assert!(report_json
        .contains("\"final_executable_host_invoke_plan_invocation_policy\":\"dry-run-only\""));
    assert!(
        report_json.contains("\"final_executable_host_invoke_plan_requires_explicit_allow\":true")
    );
    assert!(
        report_json.contains("\"final_executable_host_invoke_plan_explicit_allow_present\":false")
    );
    assert!(report_json.contains("\"final_executable_host_invoke_plan_would_invoke\":false"));
}

#[test]
fn check_reports_tampered_final_executable_host_invoke_plan() {
    let dir = env::temp_dir().join(format!(
        "nsld-check-final-executable-host-invoke-plan-drift-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_executable_writer_input_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit =
        nsld_emit_final_executable_host_invoke_plan_report(Path::new("manifest.toml"), &plan)
            .unwrap();
    let invoke_plan_source = fs::read_to_string(&emit.output_path).unwrap();
    fs::write(
        &emit.output_path,
        invoke_plan_source
            .replace(
                "invocation_policy = \"dry-run-only\"",
                "invocation_policy = \"allow-host-invoke\"",
            )
            .replace(
                "requires_explicit_allow = true",
                "requires_explicit_allow = false",
            )
            .replace(
                "explicit_allow_present = false",
                "explicit_allow_present = true",
            )
            .replace("would_invoke = false", "would_invoke = true"),
    )
    .unwrap();
    let report = nsld_check_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::check_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(!report.valid);
    assert!(report.final_executable_host_invoke_plan_present);
    assert_eq!(report.final_executable_host_invoke_plan_valid, Some(false));
    assert_eq!(
        report
            .final_executable_host_invoke_plan_invocation_policy
            .as_deref(),
        Some("allow-host-invoke")
    );
    assert_eq!(
        report.final_executable_host_invoke_plan_requires_explicit_allow,
        Some(false)
    );
    assert_eq!(
        report.final_executable_host_invoke_plan_explicit_allow_present,
        Some(true)
    );
    assert_eq!(
        report.final_executable_host_invoke_plan_would_invoke,
        Some(true)
    );
    assert!(report
        .final_executable_host_invoke_plan_issues
        .iter()
        .any(|issue| issue
            == "invocation_policy mismatch: expected dry-run-only, found allow-host-invoke"));
    assert!(report
        .final_executable_host_invoke_plan_issues
        .iter()
        .any(|issue| issue == "requires_explicit_allow mismatch: expected true, found false"));
    assert!(report
        .final_executable_host_invoke_plan_issues
        .iter()
        .any(|issue| issue == "explicit_allow_present mismatch: expected false, found true"));
    assert!(report
        .final_executable_host_invoke_plan_issues
        .iter()
        .any(|issue| issue == "would_invoke mismatch: expected false, found true"));
    assert!(report
        .issues
        .iter()
        .any(|issue| issue == "final executable host invoke plan verification failed"));
    assert!(report_json.contains("\"final_executable_host_invoke_plan_present\":true"));
    assert!(report_json.contains("\"final_executable_host_invoke_plan_valid\":false"));
    assert!(report_json
        .contains("\"final_executable_host_invoke_plan_invocation_policy\":\"allow-host-invoke\""));
    assert!(
        report_json.contains("\"final_executable_host_invoke_plan_requires_explicit_allow\":false")
    );
    assert!(
        report_json.contains("\"final_executable_host_invoke_plan_explicit_allow_present\":true")
    );
    assert!(report_json.contains("\"final_executable_host_invoke_plan_would_invoke\":true"));
}

#[test]
fn check_reports_valid_final_executable_layout_plan_when_present() {
    let dir = env::temp_dir().join(format!(
        "nsld-check-final-executable-layout-plan-{}",
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
        nsld_emit_final_executable_layout_plan_report(Path::new("manifest.toml"), &plan).unwrap();
    let report = nsld_check_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::check_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(report.valid, "{:?}", report.issues);
    assert!(report.final_executable_layout_plan_present);
    assert_eq!(report.final_executable_layout_plan_valid, Some(true));
    assert_eq!(
        report.final_executable_layout_plan_hash.as_deref(),
        Some(emit.layout_hash.as_str())
    );
    assert_eq!(
        report.final_executable_layout_plan_payload_count,
        Some(emit.payload_count)
    );
    assert!(report.final_executable_layout_plan_issues.is_empty());
    assert!(report_json.contains("\"final_executable_layout_plan_present\":true"));
    assert!(report_json.contains("\"final_executable_layout_plan_valid\":true"));
    assert!(report_json.contains("\"final_executable_layout_plan_hash\":\"0x"));
}

#[test]
fn check_reports_tampered_final_executable_layout_plan() {
    let dir = env::temp_dir().join(format!(
        "nsld-check-final-executable-layout-plan-drift-{}",
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
        nsld_emit_final_executable_layout_plan_report(Path::new("manifest.toml"), &plan).unwrap();
    let source = fs::read_to_string(&emit.output_path).unwrap();
    fs::write(
        &emit.output_path,
        source.replace(
            "lifecycle_entry_hook = \"on_process_start\"",
            "lifecycle_entry_hook = \"drift\"",
        ),
    )
    .unwrap();
    let report = nsld_check_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::check_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(!report.valid);
    assert!(report.final_executable_layout_plan_present);
    assert_eq!(report.final_executable_layout_plan_valid, Some(false));
    assert!(report
        .final_executable_layout_plan_issues
        .iter()
        .any(|issue| issue
            == "lifecycle_entry_hook mismatch: expected on_process_start, found drift"));
    assert!(report
        .issues
        .iter()
        .any(|issue| issue == "final executable layout plan verification failed"));
    assert!(report_json.contains("\"final_executable_layout_plan_present\":true"));
    assert!(report_json.contains("\"final_executable_layout_plan_valid\":false"));
}

#[test]
fn check_reports_valid_final_executable_image_dry_run_when_present() {
    let dir = env::temp_dir().join(format!(
        "nsld-check-final-executable-image-dry-run-{}",
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
    let report = nsld_check_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::check_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(report.valid, "{:?}", report.issues);
    assert!(report.final_executable_image_dry_run_present);
    assert_eq!(report.final_executable_image_dry_run_valid, Some(true));
    assert_eq!(
        report.final_executable_image_dry_run_hash.as_deref(),
        emit.image_hash.as_deref()
    );
    assert_eq!(
        report.final_executable_image_dry_run_size_bytes,
        emit.image_size_bytes
    );
    assert!(report.final_executable_image_dry_run_issues.is_empty());
    assert!(report_json.contains("\"final_executable_image_dry_run_present\":true"));
    assert!(report_json.contains("\"final_executable_image_dry_run_valid\":true"));
    assert!(report_json.contains("\"final_executable_image_dry_run_hash\":\"0x"));
    assert!(report_json.contains("\"final_executable_image_dry_run_size_bytes\":"));
}

#[test]
fn check_reports_tampered_final_executable_image_dry_run_bytes() {
    let dir = env::temp_dir().join(format!(
        "nsld-check-final-executable-image-dry-run-drift-{}",
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
    fs::write(&emit.image_path, b"drifted-final-image").unwrap();
    let report = nsld_check_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::check_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(!report.valid);
    assert!(report.final_executable_image_dry_run_present);
    assert_eq!(report.final_executable_image_dry_run_valid, Some(false));
    assert_eq!(
        report.final_executable_image_dry_run_hash.as_deref(),
        emit.image_hash.as_deref()
    );
    assert!(report
        .final_executable_image_dry_run_issues
        .iter()
        .any(|issue| issue.starts_with("image_bytes_hash mismatch: expected 0x")));
    assert!(report
        .issues
        .iter()
        .any(|issue| issue == "final executable image dry-run verification failed"));
    assert!(report_json.contains("\"final_executable_image_dry_run_present\":true"));
    assert!(report_json.contains("\"final_executable_image_dry_run_valid\":false"));
    assert!(report_json.contains("image_bytes_hash mismatch: expected 0x"));
}

#[test]
fn check_reports_valid_final_executable_blocked_artifact_when_present() {
    let dir = env::temp_dir().join(format!(
        "nsld-check-final-executable-blocked-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit = nsld_emit_final_executable_report(Path::new("manifest.toml"), &plan).unwrap();
    let report = nsld_check_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::check_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(report.valid, "{:?}", report.issues);
    assert!(report.final_executable_blocked_present);
    assert_eq!(report.final_executable_blocked_valid, Some(true));
    assert_eq!(report.final_executable_blocked_emitted, Some(false));
    assert_eq!(
        report.final_executable_blocked_plan_hash.as_deref(),
        Some(emit.final_stage_plan_hash.as_str())
    );
    assert_eq!(
        report.final_executable_blocked_blocker_count,
        Some(emit.blockers.len())
    );
    assert!(report.final_executable_blocked_issues.is_empty());
    assert!(report_json.contains("\"final_executable_blocked_present\":true"));
    assert!(report_json.contains("\"final_executable_blocked_valid\":true"));
    assert!(report_json.contains("\"final_executable_blocked_emitted\":false"));
    assert!(report_json.contains("\"final_executable_blocked_plan_hash\":\"0x"));
}

#[test]
fn check_reports_valid_final_executable_output_when_present() {
    let dir = env::temp_dir().join(format!(
        "nsld-check-final-executable-output-{}",
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
    let output_bytes = fs::read(&plan.final_stage.output_path).unwrap();
    let report = nsld_check_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::check_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(report.valid, "{:?}", report.issues);
    assert!(report.final_executable_output_present);
    assert_eq!(
        report.final_executable_output_size_bytes,
        Some(output_bytes.len())
    );
    assert_eq!(
        report.final_executable_output_hash,
        Some(fnv1a64_hex(&output_bytes))
    );
    assert_eq!(
        report.final_executable_output_runnable_candidate,
        Some(true)
    );
    assert_eq!(report.final_executable_output_blocker_count, Some(0));
    assert!(report.final_executable_output_issues.is_empty());
    assert!(report_json.contains("\"final_executable_output_present\":true"));
    assert!(report_json.contains("\"final_executable_output_runnable_candidate\":true"));
    assert!(report_json.contains("\"final_executable_output_hash\":\"0x"));
}

#[test]
fn check_reports_tampered_final_executable_blocked_artifact() {
    let dir = env::temp_dir().join(format!(
        "nsld-check-final-executable-blocked-drift-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit = nsld_emit_final_executable_report(Path::new("manifest.toml"), &plan).unwrap();
    let blocked_path = Path::new(&emit.blocked_report_path);
    let blocked_source = fs::read_to_string(blocked_path).unwrap();
    fs::write(
        blocked_path,
        blocked_source.replace(
            &format!("final_stage_plan_hash = \"{}\"", emit.final_stage_plan_hash),
            "final_stage_plan_hash = \"0x4444444444444444\"",
        ),
    )
    .unwrap();
    let report = nsld_check_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::check_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(!report.valid);
    assert!(report.final_executable_blocked_present);
    assert_eq!(report.final_executable_blocked_valid, Some(false));
    assert_eq!(
        report.final_executable_blocked_plan_hash.as_deref(),
        Some("0x4444444444444444")
    );
    assert!(report
        .final_executable_blocked_issues
        .iter()
        .any(|issue| issue.starts_with("final_stage_plan_hash mismatch: expected 0x")));
    assert!(report
        .issues
        .iter()
        .any(|issue| issue == "final executable blocked report verification failed"));
    assert!(report_json.contains("\"final_executable_blocked_present\":true"));
    assert!(report_json.contains("\"final_executable_blocked_valid\":false"));
    assert!(report_json.contains("\"final_executable_blocked_plan_hash\":\"0x4444444444444444\""));
}
