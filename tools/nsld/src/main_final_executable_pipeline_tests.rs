use super::{
    main_test_support::empty_link_plan, nsld_check_report,
    nsld_emit_final_executable_pipeline_report, nsld_prepare_report,
    nsld_verify_final_executable_pipeline_report,
};
use std::{env, fs, path::Path};

#[test]
fn emit_final_executable_pipeline_writes_launcher_closure() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-pipeline path space-{}",
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
    let pipeline =
        nsld_emit_final_executable_pipeline_report(Path::new("manifest.toml"), &plan).unwrap();
    let verify = nsld_verify_final_executable_pipeline_report(Path::new("manifest.toml"), &plan);
    let pipeline_json = super::json::nsld_final_executable_pipeline_emit_report_json(&pipeline);
    let verify_json = super::json::nsld_final_executable_pipeline_verify_report_json(&verify);
    let check = nsld_check_report(Path::new("manifest.toml"), &plan);
    let check_json = super::json::check_report_json(&check);
    let entrypoint_path = pipeline
        .entrypoint_materialization_path
        .clone()
        .expect("entrypoint path");
    let entrypoint_source = fs::read_to_string(&entrypoint_path).expect("entrypoint source");
    fs::remove_dir_all(dir).unwrap();

    assert!(pipeline.valid);
    assert!(verify.valid, "{:?}", verify.issues);
    assert_eq!(verify.actual_valid, Some(true));
    assert_eq!(verify.actual_final_executable_emitted, Some(true));
    assert_eq!(verify.actual_launcher_manifest_ready, Some(true));
    assert_eq!(verify.actual_launcher_dry_run_ready, Some(true));
    assert_eq!(verify.actual_would_enter_lifecycle_hook, Some(true));
    assert_eq!(
        verify.actual_self_owned_image_status.as_deref(),
        Some("ready")
    );
    assert_eq!(
        verify.actual_entrypoint_materialization_status.as_deref(),
        Some("host-launcher-ready")
    );
    assert_eq!(
        verify.actual_entrypoint_materialization_kind.as_deref(),
        Some("host-shell-entrypoint-plan")
    );
    assert!(verify
        .actual_entrypoint_materialization_path
        .as_deref()
        .is_some_and(|path| path.ends_with("nuis.host-entrypoint.sh")));
    assert_eq!(verify.actual_entrypoint_materialization_ready, Some(true));
    assert_eq!(verify.actual_entrypoint_materialization_first_blocker, None);
    assert_eq!(verify.actual_entrypoint_materialization_present, Some(true));
    assert!(verify
        .actual_entrypoint_materialization_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("0x")));
    assert!(verify
        .actual_entrypoint_materialization_runner_command
        .as_deref()
        .is_some_and(|command| command.contains("nuis-host-runner --manifest")));
    assert_eq!(
        verify.actual_execution_handoff_contract.as_deref(),
        Some("nsld-final-output-handoff-v1")
    );
    assert_eq!(verify.actual_execution_handoff_ready, Some(true));
    assert_eq!(
        verify.actual_execution_handoff_status.as_deref(),
        Some("entrypoint-materializer-required")
    );
    assert_eq!(
        verify.actual_execution_handoff_target.as_deref(),
        Some("entrypoint-materializer")
    );
    assert_eq!(
        verify.actual_execution_handoff_evidence_status.as_deref(),
        Some("image-header-and-hash-ready")
    );
    assert_eq!(verify.actual_execution_handoff_first_blocker, None);
    assert_eq!(
        verify.actual_execution_handoff_decision_code.as_deref(),
        Some("handoff-entrypoint-materializer")
    );
    assert_eq!(
        verify.actual_scheduler_metadata_payload_id.as_deref(),
        Some("payload0004.scheduler-metadata")
    );
    assert_eq!(verify.actual_scheduler_metadata_present, Some(true));
    assert!(verify
        .actual_scheduler_metadata_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("0x")));
    assert!(pipeline.final_executable_emitted);
    assert!(pipeline.launcher_manifest_ready);
    assert!(pipeline.launcher_dry_run_ready);
    assert!(pipeline.would_enter_lifecycle_hook);
    assert_eq!(pipeline.self_owned_image_status, "ready");
    assert_eq!(
        pipeline.entrypoint_materialization_status,
        "host-launcher-ready"
    );
    assert_eq!(
        pipeline.entrypoint_materialization_kind,
        "host-shell-entrypoint-plan"
    );
    assert!(pipeline
        .entrypoint_materialization_path
        .as_deref()
        .is_some_and(|path| path.ends_with("nuis.host-entrypoint.sh")));
    assert!(pipeline.entrypoint_materialization_ready);
    assert_eq!(pipeline.entrypoint_materialization_first_blocker, None);
    assert_eq!(pipeline.entrypoint_materialization_present, Some(true));
    assert!(pipeline
        .entrypoint_materialization_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("0x")));
    assert!(pipeline
        .entrypoint_materialization_runner_command
        .as_deref()
        .is_some_and(|command| command.contains("nuis-host-runner --manifest 'manifest.toml'")));
    assert!(pipeline
        .entrypoint_materialization_runner_command
        .as_deref()
        .is_some_and(
            |command| command.contains("--output-dir '") && command.contains("path space")
        ));
    assert!(entrypoint_source.starts_with("#!/bin/sh\n"));
    assert!(entrypoint_source.contains("NUIS_HOST_RUNNER"));
    assert!(entrypoint_source.contains("NUIS_OUTPUT_DIR='"));
    assert!(entrypoint_source.contains("path space"));
    assert!(entrypoint_source.contains("--scheduler-entry"));
    assert!(entrypoint_source.contains("--lifecycle-hook"));
    assert_eq!(
        pipeline.execution_handoff_contract,
        "nsld-final-output-handoff-v1"
    );
    assert!(pipeline.execution_handoff_ready);
    assert_eq!(
        pipeline.execution_handoff_status,
        "entrypoint-materializer-required"
    );
    assert_eq!(pipeline.execution_handoff_target, "entrypoint-materializer");
    assert_eq!(
        pipeline.execution_handoff_evidence_status,
        "image-header-and-hash-ready"
    );
    assert_eq!(pipeline.execution_handoff_first_blocker, None);
    assert_eq!(
        pipeline.execution_handoff_decision_code,
        "handoff-entrypoint-materializer"
    );
    assert_eq!(
        pipeline.scheduler_metadata_payload_id.as_deref(),
        Some("payload0004.scheduler-metadata")
    );
    assert_eq!(pipeline.scheduler_metadata_present, Some(true));
    assert!(pipeline
        .scheduler_metadata_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("0x")));
    assert_eq!(pipeline.required_stage_path_count, 10);
    assert_eq!(pipeline.required_stage_path_present_count, 10);
    assert!(pipeline.missing_required_stage_paths.is_empty());
    assert_eq!(pipeline.blocker_count, 0);
    assert!(pipeline.blockers.is_empty());
    assert!(pipeline.issues.is_empty());
    assert!(pipeline_json.contains("\"kind\":\"nsld_final_executable_pipeline_emit\""));
    assert!(pipeline_json.contains("\"final_stage_plan_path\":"));
    assert!(pipeline_json.contains("\"final_executable_emitted\":true"));
    assert!(pipeline_json.contains("\"launcher_manifest_ready\":true"));
    assert!(pipeline_json.contains("\"launcher_dry_run_ready\":true"));
    assert!(pipeline_json.contains("\"self_owned_image_status\":\"ready\""));
    assert!(pipeline_json.contains("\"entrypoint_materialization_status\":\"host-launcher-ready\""));
    assert!(pipeline_json
        .contains("\"entrypoint_materialization_kind\":\"host-shell-entrypoint-plan\""));
    assert!(pipeline_json.contains("\"entrypoint_materialization_ready\":true"));
    assert!(pipeline_json.contains("\"entrypoint_materialization_first_blocker\":null"));
    assert!(pipeline_json.contains("\"entrypoint_materialization_present\":true"));
    assert!(pipeline_json.contains("\"entrypoint_materialization_hash\":\"0x"));
    assert!(pipeline_json
        .contains("\"entrypoint_materialization_runner_command\":\"nuis-host-runner --manifest 'manifest.toml'"));
    assert!(pipeline_json.contains("\"execution_handoff_ready\":true"));
    assert!(pipeline_json.contains("\"execution_handoff_target\":\"entrypoint-materializer\""));
    assert!(pipeline_json
        .contains("\"execution_handoff_decision_code\":\"handoff-entrypoint-materializer\""));
    assert!(pipeline_json
        .contains("\"scheduler_metadata_payload_id\":\"payload0004.scheduler-metadata\""));
    assert!(pipeline_json.contains("\"scheduler_metadata_present\":true"));
    assert!(pipeline_json.contains("\"required_stage_path_count\":10"));
    assert!(pipeline_json.contains("\"required_stage_path_present_count\":10"));
    assert!(verify_json.contains("\"kind\":\"nsld_final_executable_pipeline_verify\""));
    assert!(verify_json.contains("\"actual_valid\":true"));
    assert!(verify_json.contains("\"actual_self_owned_image_status\":\"ready\""));
    assert!(verify_json
        .contains("\"actual_entrypoint_materialization_status\":\"host-launcher-ready\""));
    assert!(verify_json
        .contains("\"actual_entrypoint_materialization_kind\":\"host-shell-entrypoint-plan\""));
    assert!(verify_json.contains("\"actual_entrypoint_materialization_ready\":true"));
    assert!(verify_json.contains("\"actual_entrypoint_materialization_present\":true"));
    assert!(verify_json.contains("\"actual_entrypoint_materialization_hash\":\"0x"));
    assert!(verify_json.contains(
        "\"actual_entrypoint_materialization_runner_command\":\"nuis-host-runner --manifest 'manifest.toml'"
    ));
    assert!(verify_json.contains("\"actual_execution_handoff_ready\":true"));
    assert!(verify_json.contains("\"actual_execution_handoff_target\":\"entrypoint-materializer\""));
    assert!(verify_json.contains(
        "\"actual_execution_handoff_decision_code\":\"handoff-entrypoint-materializer\""
    ));
    assert!(verify_json
        .contains("\"actual_scheduler_metadata_payload_id\":\"payload0004.scheduler-metadata\""));
    assert!(check_json.contains(
        "\"final_executable_pipeline_execution_handoff_contract\":\"nsld-final-output-handoff-v1\""
    ));
    assert!(check_json.contains(
        "\"final_executable_pipeline_entrypoint_materialization_kind\":\"host-shell-entrypoint-plan\""
    ));
    assert!(
        check_json.contains("\"final_executable_pipeline_entrypoint_materialization_ready\":true")
    );
    assert!(check_json
        .contains("\"final_executable_pipeline_entrypoint_materialization_first_blocker\":null"));
    assert!(check_json
        .contains("\"final_executable_pipeline_entrypoint_materialization_present\":true"));
    assert!(
        check_json.contains("\"final_executable_pipeline_entrypoint_materialization_hash\":\"0x")
    );
    assert!(check_json.contains(
        "\"final_executable_pipeline_entrypoint_materialization_runner_command\":\"nuis-host-runner --manifest 'manifest.toml'"
    ));
    assert!(check_json.contains("\"final_executable_pipeline_execution_handoff_ready\":true"));
    assert!(check_json.contains(
        "\"final_executable_pipeline_execution_handoff_status\":\"entrypoint-materializer-required\""
    ));
    assert!(check_json.contains(
        "\"final_executable_pipeline_execution_handoff_target\":\"entrypoint-materializer\""
    ));
    assert!(check_json.contains(
        "\"final_executable_pipeline_execution_handoff_evidence_status\":\"image-header-and-hash-ready\""
    ));
    assert!(
        check_json.contains("\"final_executable_pipeline_execution_handoff_first_blocker\":null")
    );
    assert!(check_json.contains(
        "\"final_executable_pipeline_execution_handoff_decision_code\":\"handoff-entrypoint-materializer\""
    ));
    assert!(check.valid);
    assert!(check.final_executable_output_present);
    assert!(check.final_executable_launcher_manifest_present);
    assert!(check.final_executable_launcher_dry_run_present);
    assert!(check.final_executable_pipeline_present);
    assert_eq!(check.final_executable_pipeline_valid, Some(true));
    assert_eq!(check.final_executable_pipeline_ready, Some(true));
    assert_eq!(check.final_executable_pipeline_emitted, Some(true));
    assert_eq!(
        check
            .final_executable_pipeline_self_owned_image_status
            .as_deref(),
        Some("ready")
    );
    assert_eq!(
        check
            .final_executable_pipeline_entrypoint_materialization_status
            .as_deref(),
        Some("host-launcher-ready")
    );
    assert_eq!(
        check
            .final_executable_pipeline_entrypoint_materialization_kind
            .as_deref(),
        Some("host-shell-entrypoint-plan")
    );
    assert!(check
        .final_executable_pipeline_entrypoint_materialization_path
        .as_deref()
        .is_some_and(|path| path.ends_with("nuis.host-entrypoint.sh")));
    assert_eq!(
        check.final_executable_pipeline_entrypoint_materialization_ready,
        Some(true)
    );
    assert_eq!(
        check.final_executable_pipeline_entrypoint_materialization_first_blocker,
        None
    );
    assert_eq!(
        check.final_executable_pipeline_entrypoint_materialization_present,
        Some(true)
    );
    assert!(check
        .final_executable_pipeline_entrypoint_materialization_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("0x")));
    assert!(check
        .final_executable_pipeline_entrypoint_materialization_runner_command
        .as_deref()
        .is_some_and(|command| command.contains("nuis-host-runner --manifest")));
    assert_eq!(
        check
            .final_executable_pipeline_execution_handoff_contract
            .as_deref(),
        Some("nsld-final-output-handoff-v1")
    );
    assert_eq!(
        check.final_executable_pipeline_execution_handoff_ready,
        Some(true)
    );
    assert_eq!(
        check
            .final_executable_pipeline_execution_handoff_status
            .as_deref(),
        Some("entrypoint-materializer-required")
    );
    assert_eq!(
        check
            .final_executable_pipeline_execution_handoff_target
            .as_deref(),
        Some("entrypoint-materializer")
    );
    assert_eq!(
        check
            .final_executable_pipeline_execution_handoff_evidence_status
            .as_deref(),
        Some("image-header-and-hash-ready")
    );
    assert_eq!(
        check.final_executable_pipeline_execution_handoff_first_blocker,
        None
    );
    assert_eq!(
        check
            .final_executable_pipeline_execution_handoff_decision_code
            .as_deref(),
        Some("handoff-entrypoint-materializer")
    );
    assert_eq!(
        check
            .final_executable_pipeline_scheduler_metadata_payload_id
            .as_deref(),
        Some("payload0004.scheduler-metadata")
    );
    assert_eq!(
        check.final_executable_pipeline_scheduler_metadata_present,
        Some(true)
    );
    assert!(check
        .final_executable_pipeline_scheduler_metadata_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("0x")));
    assert_eq!(
        check.final_executable_pipeline_required_stage_path_count,
        Some(10)
    );
    assert_eq!(
        check.final_executable_pipeline_required_stage_path_present_count,
        Some(10)
    );
    assert!(check
        .final_executable_pipeline_missing_required_stage_paths
        .is_empty());
    assert_eq!(check.final_executable_pipeline_blocker_count, Some(0));
    assert!(check.final_executable_pipeline_issues.is_empty());
    assert!(check
        .final_executable_pipeline_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("0x")));
}

#[test]
fn emit_final_executable_pipeline_reports_host_driver_blockers() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-pipeline-blocked-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();
    plan.final_stage.driver = "definitely-missing-nsld-host-driver-for-pipeline-test".to_owned();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let pipeline =
        nsld_emit_final_executable_pipeline_report(Path::new("manifest.toml"), &plan).unwrap();
    let pipeline_json = super::json::nsld_final_executable_pipeline_emit_report_json(&pipeline);
    let check = nsld_check_report(Path::new("manifest.toml"), &plan);
    fs::remove_dir_all(dir).unwrap();

    assert!(!pipeline.valid);
    assert!(!pipeline.final_executable_emitted);
    assert!(!pipeline.launcher_manifest_ready);
    assert!(!pipeline.launcher_dry_run_ready);
    assert!(!pipeline.would_enter_lifecycle_hook);
    assert_eq!(pipeline.required_stage_path_count, 8);
    assert_eq!(pipeline.required_stage_path_present_count, 8);
    assert!(pipeline.missing_required_stage_paths.is_empty());
    assert!(pipeline.blocker_count >= 1);
    assert!(pipeline
        .blockers
        .iter()
        .any(|blocker| blocker == "host-finalizer-environment:not-ready"));
    assert!(pipeline.blockers.iter().any(|blocker| blocker
        == "host-finalizer-dry-run:host-finalizer-driver-unavailable:definitely-missing-nsld-host-driver-for-pipeline-test"));
    assert!(pipeline
        .blockers
        .iter()
        .any(|blocker| blocker == "final-executable-launcher-manifest:not-ready"));
    assert!(pipeline
        .blockers
        .iter()
        .any(|blocker| blocker == "final-executable-launcher-dry-run:not-ready"));
    assert!(pipeline_json.contains("\"valid\":false"));
    assert!(pipeline_json.contains("pipeline:host-finalizer-environment:not-ready"));
    assert!(pipeline_json.contains(
        "host-finalizer-driver-unavailable:definitely-missing-nsld-host-driver-for-pipeline-test"
    ));
    assert!(pipeline_json.contains("final-executable-launcher-dry-run:not-ready"));
    assert!(check.valid);
    assert!(check.final_executable_blocked_present);
    assert!(check.final_executable_launcher_manifest_present);
    assert!(check.final_executable_launcher_dry_run_present);
    assert!(check.final_executable_pipeline_present);
    assert_eq!(check.final_executable_pipeline_valid, Some(true));
    assert_eq!(check.final_executable_pipeline_ready, Some(false));
    assert_eq!(check.final_executable_pipeline_emitted, Some(false));
    assert!(check
        .final_executable_pipeline_blocker_count
        .is_some_and(|count| count >= 1));
    assert_eq!(check.final_executable_launcher_manifest_ready, Some(false));
    assert_eq!(check.final_executable_launcher_dry_run_ready, Some(false));
}

#[test]
fn verify_final_executable_pipeline_reports_missing_required_stage_path() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-pipeline-missing-stage-{}",
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
    let pipeline =
        nsld_emit_final_executable_pipeline_report(Path::new("manifest.toml"), &plan).unwrap();
    fs::remove_file(&pipeline.launcher_dry_run_path).unwrap();
    let verify = nsld_verify_final_executable_pipeline_report(Path::new("manifest.toml"), &plan);
    let check = nsld_check_report(Path::new("manifest.toml"), &plan);
    let verify_json = super::json::nsld_final_executable_pipeline_verify_report_json(&verify);
    fs::remove_dir_all(dir).unwrap();

    assert!(!verify.valid);
    assert_eq!(verify.expected_required_stage_path_count, 9);
    assert_eq!(verify.actual_required_stage_path_count, Some(10));
    assert_eq!(verify.expected_required_stage_path_present_count, 8);
    assert_eq!(verify.actual_required_stage_path_present_count, Some(10));
    assert_eq!(
        verify.expected_missing_required_stage_paths,
        vec![pipeline.launcher_dry_run_path.clone()]
    );
    assert!(verify.actual_missing_required_stage_paths.is_empty());
    assert!(verify
        .expected_blockers
        .iter()
        .any(|blocker| blocker.starts_with("required-stage-path-missing:")));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.contains("missing_required_stage_paths mismatch")));
    assert!(!check.valid);
    assert_eq!(
        check.final_executable_pipeline_required_stage_path_count,
        Some(9)
    );
    assert_eq!(
        check.final_executable_pipeline_required_stage_path_present_count,
        Some(8)
    );
    assert_eq!(
        check.final_executable_pipeline_missing_required_stage_paths,
        vec![pipeline.launcher_dry_run_path]
    );
    assert!(verify_json.contains("\"expected_required_stage_path_present_count\":8"));
    assert!(verify_json.contains("\"actual_required_stage_path_present_count\":10"));
}

#[test]
fn verify_final_executable_pipeline_reports_missing_entrypoint_materialization() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-pipeline-missing-entrypoint-{}",
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
    let pipeline =
        nsld_emit_final_executable_pipeline_report(Path::new("manifest.toml"), &plan).unwrap();
    let entrypoint_path = pipeline
        .entrypoint_materialization_path
        .clone()
        .expect("entrypoint path");
    fs::remove_file(&entrypoint_path).unwrap();

    let verify = nsld_verify_final_executable_pipeline_report(Path::new("manifest.toml"), &plan);
    let check = nsld_check_report(Path::new("manifest.toml"), &plan);
    let verify_json = super::json::nsld_final_executable_pipeline_verify_report_json(&verify);
    fs::remove_dir_all(dir).unwrap();

    assert!(!verify.valid);
    assert_eq!(
        verify.expected_entrypoint_materialization_present,
        Some(false)
    );
    assert_eq!(verify.actual_entrypoint_materialization_present, Some(true));
    assert_eq!(verify.expected_entrypoint_materialization_hash, None);
    assert!(verify
        .actual_entrypoint_materialization_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("0x")));
    assert_eq!(verify.expected_required_stage_path_count, 10);
    assert_eq!(verify.actual_required_stage_path_count, Some(10));
    assert_eq!(verify.expected_required_stage_path_present_count, 9);
    assert_eq!(verify.actual_required_stage_path_present_count, Some(10));
    assert_eq!(
        verify.expected_missing_required_stage_paths,
        vec![entrypoint_path.clone()]
    );
    assert!(verify.actual_missing_required_stage_paths.is_empty());
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.contains("entrypoint_materialization_present mismatch")));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.contains("entrypoint_materialization_hash mismatch")));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.contains("missing_required_stage_paths mismatch")));

    assert!(!check.valid);
    assert_eq!(
        check.final_executable_pipeline_entrypoint_materialization_present,
        Some(true)
    );
    assert!(check
        .final_executable_pipeline_entrypoint_materialization_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("0x")));
    assert_eq!(
        check.final_executable_pipeline_required_stage_path_present_count,
        Some(9)
    );
    assert_eq!(
        check.final_executable_pipeline_missing_required_stage_paths,
        vec![entrypoint_path]
    );
    assert!(verify_json.contains("\"expected_entrypoint_materialization_present\":false"));
    assert!(verify_json.contains("\"actual_entrypoint_materialization_present\":true"));
    assert!(verify_json.contains("\"expected_entrypoint_materialization_hash\":null"));
    assert!(verify_json.contains("\"actual_entrypoint_materialization_hash\":\"0x"));
}

#[test]
fn verify_final_executable_pipeline_reports_tampered_entrypoint_materialization() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-pipeline-tampered-entrypoint-{}",
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
    let pipeline =
        nsld_emit_final_executable_pipeline_report(Path::new("manifest.toml"), &plan).unwrap();
    let entrypoint_path = pipeline
        .entrypoint_materialization_path
        .clone()
        .expect("entrypoint path");
    fs::write(
        &entrypoint_path,
        "#!/bin/sh\nset -eu\necho tampered-entrypoint\n",
    )
    .unwrap();

    let verify = nsld_verify_final_executable_pipeline_report(Path::new("manifest.toml"), &plan);
    let check = nsld_check_report(Path::new("manifest.toml"), &plan);
    let verify_json = super::json::nsld_final_executable_pipeline_verify_report_json(&verify);
    fs::remove_dir_all(dir).unwrap();

    assert!(!verify.valid);
    assert_eq!(
        verify.expected_entrypoint_materialization_present,
        Some(true)
    );
    assert_eq!(verify.actual_entrypoint_materialization_present, Some(true));
    assert!(verify
        .expected_entrypoint_materialization_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("0x")));
    assert!(verify
        .actual_entrypoint_materialization_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("0x")));
    assert_ne!(
        verify.expected_entrypoint_materialization_hash,
        verify.actual_entrypoint_materialization_hash
    );
    assert_eq!(verify.expected_required_stage_path_count, 10);
    assert_eq!(verify.actual_required_stage_path_count, Some(10));
    assert_eq!(verify.expected_required_stage_path_present_count, 10);
    assert_eq!(verify.actual_required_stage_path_present_count, Some(10));
    assert!(verify.expected_missing_required_stage_paths.is_empty());
    assert!(verify.actual_missing_required_stage_paths.is_empty());
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.contains("entrypoint_materialization_hash mismatch")));

    assert!(!check.valid);
    assert_eq!(
        check.final_executable_pipeline_required_stage_path_present_count,
        Some(10)
    );
    assert!(check
        .final_executable_pipeline_missing_required_stage_paths
        .is_empty());
    assert!(check
        .final_executable_pipeline_issues
        .iter()
        .any(|issue| issue.contains("entrypoint_materialization_hash mismatch")));
    assert!(verify_json.contains("\"expected_entrypoint_materialization_present\":true"));
    assert!(verify_json.contains("\"actual_entrypoint_materialization_present\":true"));
    assert!(verify_json.contains("\"expected_entrypoint_materialization_hash\":\"0x"));
    assert!(verify_json.contains("\"actual_entrypoint_materialization_hash\":\"0x"));
}
