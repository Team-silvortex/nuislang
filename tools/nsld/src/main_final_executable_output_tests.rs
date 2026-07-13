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
    assert_eq!(report.boundary_status, "missing");
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
    assert!(report_json.contains("\"boundary_status\":\"missing\""));
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
fn final_executable_launcher_manifest_describes_runnable_nsb_entry() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-launcher-manifest-{}",
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
    let manifest =
        super::nsld_final_executable_launcher_manifest_report(Path::new("manifest.toml"), &plan);
    let emit = super::nsld_emit_final_executable_launcher_manifest_report(
        Path::new("manifest.toml"),
        &plan,
    )
    .unwrap();
    let verify = super::nsld_verify_final_executable_launcher_manifest_report(
        Path::new("manifest.toml"),
        &plan,
    );
    let dry_run =
        super::nsld_final_executable_launcher_dry_run_report(Path::new("manifest.toml"), &plan);
    let dry_run_emit = super::nsld_emit_final_executable_launcher_dry_run_report(
        Path::new("manifest.toml"),
        &plan,
    )
    .unwrap();
    let dry_run_verify = super::nsld_verify_final_executable_launcher_dry_run_report(
        Path::new("manifest.toml"),
        &plan,
    );
    let manifest_json = super::json::nsld_final_executable_launcher_manifest_report_json(&manifest);
    let verify_json =
        super::json::nsld_final_executable_launcher_manifest_verify_report_json(&verify);
    let dry_run_json = super::json::nsld_final_executable_launcher_dry_run_report_json(&dry_run);
    let dry_run_emit_json =
        super::json::nsld_final_executable_launcher_dry_run_emit_report_json(&dry_run_emit);
    let dry_run_verify_json =
        super::json::nsld_final_executable_launcher_dry_run_verify_report_json(&dry_run_verify);
    let manifest_source = fs::read_to_string(&emit.output_path).unwrap();
    let dry_run_source = fs::read_to_string(&dry_run_emit.output_path).unwrap();
    let output_bytes = fs::read(&plan.final_stage.output_path).unwrap();
    fs::remove_dir_all(dir).unwrap();

    assert!(manifest.ready, "{:?}", manifest.blockers);
    assert_eq!(manifest.launcher_format, "nuis-host-launcher-manifest-v1");
    assert_eq!(manifest.nsb_path, plan.final_stage.output_path);
    assert_eq!(manifest.nsb_size_bytes, Some(output_bytes.len()));
    assert_eq!(manifest.nsb_hash, Some(fnv1a64_hex(&output_bytes)));
    assert!(manifest.image_header_valid);
    assert_eq!(manifest.entry_lifecycle_hook, "on_process_start");
    assert_eq!(manifest.scheduler_entry, "nuis.scheduler.loop.v1");
    assert_eq!(
        manifest.scheduler_metadata_payload_id.as_deref(),
        Some("payload0004.scheduler-metadata")
    );
    assert_eq!(manifest.scheduler_metadata_present, Some(true));
    assert!(manifest
        .scheduler_metadata_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("0x")));
    assert!(manifest
        .verification_steps
        .iter()
        .any(|step| step == "enter-lifecycle-hook:on_process_start"));
    assert!(emit.ready);
    assert_eq!(emit.blocker_count, 0);
    assert!(verify.valid, "{:?}", verify.issues);
    assert_eq!(verify.actual_ready, Some(true));
    assert_eq!(verify.actual_nsb_hash, Some(fnv1a64_hex(&output_bytes)));
    assert_eq!(
        verify.actual_entry_lifecycle_hook.as_deref(),
        Some("on_process_start")
    );
    assert_eq!(
        verify.actual_scheduler_entry.as_deref(),
        Some("nuis.scheduler.loop.v1")
    );
    assert_eq!(
        verify.actual_scheduler_metadata_payload_id.as_deref(),
        Some("payload0004.scheduler-metadata")
    );
    assert_eq!(verify.actual_scheduler_metadata_present, Some(true));
    assert_eq!(
        verify.expected_scheduler_metadata_hash,
        manifest.scheduler_metadata_hash
    );
    assert_eq!(
        verify.actual_scheduler_metadata_hash,
        manifest.scheduler_metadata_hash
    );
    assert!(dry_run.dry_run_ready, "{:?}", dry_run.blockers);
    assert!(dry_run.would_enter_lifecycle_hook);
    assert_eq!(dry_run.nsb_hash_actual, Some(fnv1a64_hex(&output_bytes)));
    assert_eq!(
        dry_run.scheduler_metadata_payload_id.as_deref(),
        Some("payload0004.scheduler-metadata")
    );
    assert_eq!(dry_run.scheduler_metadata_present, Some(true));
    assert_eq!(
        dry_run.scheduler_metadata_hash,
        manifest.scheduler_metadata_hash
    );
    assert!(dry_run
        .launch_steps
        .iter()
        .any(|step| step == "enter-lifecycle-hook:on_process_start"));
    assert!(dry_run_emit.dry_run_ready);
    assert_eq!(dry_run_emit.blocker_count, 0);
    assert!(dry_run_verify.valid, "{:?}", dry_run_verify.issues);
    assert_eq!(dry_run_verify.actual_dry_run_ready, Some(true));
    assert_eq!(dry_run_verify.actual_would_enter_lifecycle_hook, Some(true));
    assert_eq!(
        dry_run_verify.actual_nsb_hash_actual,
        Some(fnv1a64_hex(&output_bytes))
    );
    assert!(manifest_source.contains("schema = \"nuis-host-launcher-manifest-v1\""));
    assert!(dry_run_source.contains("schema = \"nuis-host-launcher-dry-run-v1\""));
    assert!(manifest_source.contains("entry_lifecycle_hook = \"on_process_start\""));
    assert!(manifest_source.contains("scheduler_entry = \"nuis.scheduler.loop.v1\""));
    assert!(manifest_source
        .contains("scheduler_metadata_payload_id = \"payload0004.scheduler-metadata\""));
    assert!(manifest_source.contains("scheduler_metadata_present = true"));
    assert!(dry_run_source
        .contains("scheduler_metadata_payload_id = \"payload0004.scheduler-metadata\""));
    assert!(dry_run_source.contains("scheduler_metadata_present = true"));
    assert!(manifest_json.contains("\"kind\":\"nsld_final_executable_launcher_manifest\""));
    assert!(manifest_json.contains("\"ready\":true"));
    assert!(manifest_json.contains("\"nsb_hash\":\"0x"));
    assert!(manifest_json
        .contains("\"scheduler_metadata_payload_id\":\"payload0004.scheduler-metadata\""));
    assert!(manifest_json.contains("\"scheduler_metadata_present\":true"));
    assert!(verify_json.contains("\"kind\":\"nsld_final_executable_launcher_manifest_verify\""));
    assert!(verify_json.contains("\"valid\":true"));
    assert!(verify_json
        .contains("\"actual_scheduler_metadata_payload_id\":\"payload0004.scheduler-metadata\""));
    assert!(dry_run_json.contains("\"kind\":\"nsld_final_executable_launcher_dry_run\""));
    assert!(dry_run_json.contains("\"dry_run_ready\":true"));
    assert!(dry_run_json.contains("\"would_enter_lifecycle_hook\":true"));
    assert!(dry_run_json
        .contains("\"scheduler_metadata_payload_id\":\"payload0004.scheduler-metadata\""));
    assert!(dry_run_emit_json.contains("\"kind\":\"nsld_final_executable_launcher_dry_run_emit\""));
    assert!(dry_run_emit_json.contains("\"dry_run_ready\":true"));
    assert!(
        dry_run_verify_json.contains("\"kind\":\"nsld_final_executable_launcher_dry_run_verify\"")
    );
    assert!(dry_run_verify_json.contains("\"valid\":true"));
}

#[test]
fn verify_final_executable_launcher_manifest_reports_drift() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-launcher-manifest-drift-{}",
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
    let emit = super::nsld_emit_final_executable_launcher_manifest_report(
        Path::new("manifest.toml"),
        &plan,
    )
    .unwrap();
    let damaged = fs::read_to_string(&emit.output_path)
        .unwrap()
        .replace("ready = true", "ready = false")
        .replace(
            "entry_lifecycle_hook = \"on_process_start\"",
            "entry_lifecycle_hook = \"tampered_hook\"",
        )
        .replace(
            "scheduler_entry = \"nuis.scheduler.loop.v1\"",
            "scheduler_entry = \"tampered.scheduler\"",
        )
        .replace(
            "verification_steps = [\"read-nsb-header\", \"verify-nsb-magic-and-version\", \"verify-nsb-size-and-hash\", \"map-payload-region\", \"enter-lifecycle-hook:on_process_start\"]",
            "verification_steps = [\"tampered-step\"]",
        );
    fs::write(&emit.output_path, damaged).unwrap();
    let verify = super::nsld_verify_final_executable_launcher_manifest_report(
        Path::new("manifest.toml"),
        &plan,
    );
    let verify_json =
        super::json::nsld_final_executable_launcher_manifest_verify_report_json(&verify);
    fs::remove_dir_all(dir).unwrap();

    assert!(!verify.valid);
    assert_eq!(verify.actual_ready, Some(false));
    assert_eq!(
        verify.actual_entry_lifecycle_hook.as_deref(),
        Some("tampered_hook")
    );
    assert_eq!(
        verify.actual_scheduler_entry.as_deref(),
        Some("tampered.scheduler")
    );
    assert_eq!(verify.actual_verification_steps, vec!["tampered-step"]);
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue == "ready mismatch: expected true, found false"));
    assert!(verify.issues.iter().any(|issue| issue
        == "entry_lifecycle_hook mismatch: expected on_process_start, found tampered_hook"));
    assert!(verify.issues.iter().any(|issue| issue
        == "scheduler_entry mismatch: expected nuis.scheduler.loop.v1, found tampered.scheduler"));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("verification_steps mismatch")));
    assert!(verify_json.contains("\"actual_ready\":false"));
    assert!(verify_json.contains("\"actual_entry_lifecycle_hook\":\"tampered_hook\""));
    assert!(verify_json.contains("\"actual_scheduler_entry\":\"tampered.scheduler\""));
    assert!(verify_json.contains("\"actual_verification_steps\":[\"tampered-step\"]"));
}

#[test]
fn final_executable_launcher_dry_run_rejects_tampered_final_output_bytes() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-launcher-dry-run-tamper-{}",
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
    super::nsld_emit_final_executable_launcher_manifest_report(Path::new("manifest.toml"), &plan)
        .unwrap();
    fs::write(&plan.final_stage.output_path, b"tampered-nsb").unwrap();

    let dry_run =
        super::nsld_final_executable_launcher_dry_run_report(Path::new("manifest.toml"), &plan);
    let dry_run_json = super::json::nsld_final_executable_launcher_dry_run_report_json(&dry_run);
    fs::remove_dir_all(dir).unwrap();

    assert!(!dry_run.dry_run_ready);
    assert!(!dry_run.would_enter_lifecycle_hook);
    assert!(dry_run.nsb_readable);
    assert!(!dry_run.nsb_hash_matches);
    assert!(dry_run.launch_steps.is_empty());
    assert!(dry_run
        .blockers
        .iter()
        .any(|blocker| blocker == "host-launcher:final-output-hash-mismatch"));
    assert!(dry_run_json.contains("\"dry_run_ready\":false"));
    assert!(dry_run_json.contains("\"would_enter_lifecycle_hook\":false"));
    assert!(dry_run_json.contains("\"host-launcher:final-output-hash-mismatch\""));
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
    let emit_json = super::json::nsld_final_executable_emit_report_json(&emit);
    let verify_emit = nsld_verify_final_executable_emit_report(Path::new("manifest.toml"), &plan);
    let output = nsld_final_executable_output_report(Path::new("manifest.toml"), &plan);
    let output_json = super::json::nsld_final_executable_output_report_json(&output);
    let image_bytes = fs::read(&emit.image_dry_run_bytes_path).unwrap();
    let output_bytes = fs::read(&plan.final_stage.output_path).unwrap();
    fs::remove_dir_all(dir).unwrap();

    assert!(emit.emitted);
    assert!(emit.can_emit_final_executable);
    assert_eq!(emit.writer_status, "ready");
    assert!(emit.final_output_checked);
    assert!(emit.final_output_present);
    assert_eq!(emit.final_output_size_bytes, Some(output_bytes.len()));
    assert_eq!(emit.final_output_hash, Some(fnv1a64_hex(&output_bytes)));
    assert_eq!(emit.final_output_image_header_valid, Some(true));
    assert_eq!(emit.final_output_runnable_candidate, Some(true));
    assert!(verify_emit.valid, "{:?}", verify_emit.issues);
    assert!(verify_emit.expected_final_output_checked);
    assert_eq!(verify_emit.actual_final_output_checked, Some(true));
    assert!(verify_emit.expected_final_output_present);
    assert_eq!(verify_emit.actual_final_output_present, Some(true));
    assert_eq!(
        verify_emit.expected_final_output_size_bytes,
        Some(output_bytes.len())
    );
    assert_eq!(
        verify_emit.actual_final_output_size_bytes,
        Some(output_bytes.len())
    );
    assert_eq!(
        verify_emit.expected_final_output_hash,
        Some(fnv1a64_hex(&output_bytes))
    );
    assert_eq!(
        verify_emit.actual_final_output_hash,
        Some(fnv1a64_hex(&output_bytes))
    );
    assert_eq!(
        verify_emit.expected_final_output_image_header_valid,
        Some(true)
    );
    assert_eq!(
        verify_emit.actual_final_output_image_header_valid,
        Some(true)
    );
    assert_eq!(
        verify_emit.expected_final_output_runnable_candidate,
        Some(true)
    );
    assert_eq!(
        verify_emit.actual_final_output_runnable_candidate,
        Some(true)
    );
    assert!(output.present);
    assert_eq!(output.boundary_status, "ready");
    assert!(output.path_present);
    assert!(output.nsld_owned_output);
    assert!(output.runnable_candidate, "{:?}", output.blockers);
    assert!(output.matches_expected_image, "{:?}", output.issues);
    assert_eq!(output.size_bytes, Some(output_bytes.len()));
    assert_eq!(output.output_hash, Some(fnv1a64_hex(&output_bytes)));
    assert!(output.output_image_header_required);
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
    assert_eq!(
        output.scheduler_metadata_payload_id.as_deref(),
        Some("payload0004.scheduler-metadata")
    );
    assert_eq!(output.scheduler_metadata_present, Some(true));
    assert_eq!(
        output.scheduler_metadata_offset,
        layout
            .byte_map_entries
            .iter()
            .find(|entry| entry.payload_id == "payload0004.scheduler-metadata")
            .map(|entry| entry.offset)
    );
    assert_eq!(
        output.scheduler_metadata_hash,
        layout
            .payloads
            .iter()
            .find(|payload| payload.payload_id == "payload0004.scheduler-metadata")
            .map(|payload| payload.content_hash.clone())
    );
    assert_eq!(output.expected_image_size_bytes, Some(image_bytes.len()));
    assert_eq!(output.expected_image_hash, Some(fnv1a64_hex(&image_bytes)));
    assert_eq!(image_bytes, output_bytes);
    assert!(output_json.contains("\"present\":true"));
    assert!(output_json.contains("\"boundary_status\":\"ready\""));
    assert!(output_json.contains("\"path_present\":true"));
    assert!(output_json.contains("\"nsld_owned_output\":true"));
    assert!(output_json.contains("\"output_image_header_required\":true"));
    assert!(output_json.contains("\"output_image_header_valid\":true"));
    assert!(output_json.contains("\"output_image_magic\":\"NUIFIMG\""));
    assert!(output_json.contains("\"output_image_version\":1"));
    assert!(output_json.contains("\"output_layout_hash\":\"0x"));
    assert!(output_json.contains("\"output_byte_map_hash\":\"0x"));
    assert!(output_json
        .contains("\"scheduler_metadata_payload_id\":\"payload0004.scheduler-metadata\""));
    assert!(output_json.contains("\"scheduler_metadata_present\":true"));
    assert!(output_json.contains("\"scheduler_metadata_hash\":\"0x"));
    assert!(output_json.contains("\"matches_expected_image\":true"));
    assert!(output_json.contains("\"final_stage_plan_valid\":true"));
    assert!(output_json.contains("\"final_executable_emit_valid\":true"));
    assert!(output_json.contains("\"final_executable_emitted\":true"));
    assert!(output_json.contains("\"runnable_candidate\":true"));
    assert!(output_json.contains("\"blockers\":[]"));
    assert!(output_json.contains("\"issues\":[]"));
    assert!(emit_json.contains("\"final_output_checked\":true"));
    assert!(emit_json.contains("\"final_output_present\":true"));
    assert!(emit_json.contains("\"final_output_hash\":\"0x"));
    assert!(emit_json.contains("\"final_output_image_header_valid\":true"));
    assert!(emit_json.contains("\"final_output_runnable_candidate\":true"));
}

#[test]
fn verify_final_executable_emit_reports_final_output_drift() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-output-drift-{}",
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
    let emit = nsld_emit_final_executable_report(Path::new("manifest.toml"), &plan).unwrap();
    let blocked_path = Path::new(&emit.blocked_report_path);
    let damaged = fs::read_to_string(blocked_path)
        .unwrap()
        .replace(
            "final_output_present = true",
            "final_output_present = false",
        )
        .replace(
            &format!(
                "final_output_size_bytes = {}",
                emit.final_output_size_bytes.unwrap()
            ),
            "final_output_size_bytes = 1",
        )
        .replace(
            &format!(
                "final_output_hash = \"{}\"",
                emit.final_output_hash.as_deref().unwrap()
            ),
            "final_output_hash = \"0xaaaaaaaaaaaaaaaa\"",
        )
        .replace(
            "final_output_image_header_valid = true",
            "final_output_image_header_valid = false",
        )
        .replace(
            "final_output_runnable_candidate = true",
            "final_output_runnable_candidate = false",
        );
    fs::write(blocked_path, damaged).unwrap();

    let verify = nsld_verify_final_executable_emit_report(Path::new("manifest.toml"), &plan);
    let verify_json = super::json::nsld_final_executable_emit_verify_report_json(&verify);
    fs::remove_dir_all(dir).unwrap();

    assert!(!verify.valid);
    assert_eq!(verify.expected_final_output_present, true);
    assert_eq!(verify.actual_final_output_present, Some(false));
    assert_eq!(verify.actual_final_output_size_bytes, Some(1));
    assert_eq!(
        verify.actual_final_output_hash.as_deref(),
        Some("0xaaaaaaaaaaaaaaaa")
    );
    assert_eq!(verify.actual_final_output_image_header_valid, Some(false));
    assert_eq!(verify.actual_final_output_runnable_candidate, Some(false));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue == "final_output_present mismatch: expected true, found false"));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("final_output_size_bytes mismatch")));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("final_output_hash mismatch")));
    assert!(verify.issues.iter().any(|issue| {
        issue == "final_output_image_header_valid mismatch: expected true, found false"
    }));
    assert!(verify.issues.iter().any(|issue| {
        issue == "final_output_runnable_candidate mismatch: expected true, found false"
    }));
    assert!(verify_json.contains("\"actual_final_output_present\":false"));
    assert!(verify_json.contains("\"actual_final_output_size_bytes\":1"));
    assert!(verify_json.contains("\"actual_final_output_hash\":\"0xaaaaaaaaaaaaaaaa\""));
    assert!(verify_json.contains("\"actual_final_output_image_header_valid\":false"));
    assert!(verify_json.contains("\"actual_final_output_runnable_candidate\":false"));
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
    assert_eq!(output.boundary_status, "invalid");
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
    assert_eq!(check.final_executable_output_boundary_status, "invalid");
    assert!(check
        .final_executable_output_issues
        .iter()
        .any(|issue| issue.contains("image-header-invalid")));
    assert!(check
        .final_executable_output_issues
        .iter()
        .any(|issue| issue.contains("hash mismatch")));
    assert!(output_json.contains("\"present\":true"));
    assert!(output_json.contains("\"boundary_status\":\"invalid\""));
    assert!(output_json.contains("\"output_image_header_valid\":false"));
    assert!(output_json.contains("\"matches_expected_image\":false"));
    assert!(output_json.contains("\"runnable_candidate\":false"));
    assert!(output_json.contains("\"final-executable-output:image-header-invalid\""));
    assert!(output_json.contains("\"final-executable-output:size-mismatch\""));
    assert!(output_json.contains("\"final-executable-output:hash-mismatch\""));
}
