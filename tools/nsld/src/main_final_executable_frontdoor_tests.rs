use super::{
    artifact_chain::{nsld_artifact_stage_kind_path, NsldArtifactStageKind},
    main_test_support::empty_link_plan,
    nsld_emit_final_executable_host_invoke_plan_report, nsld_emit_final_executable_report,
    nsld_emit_final_executable_writer_input_report, nsld_final_executable_host_dry_run_report,
    nsld_final_executable_host_invoke_plan_report, nsld_final_executable_readiness_report,
    nsld_final_executable_writer_plan_report, nsld_prepare_report,
    nsld_verify_final_executable_host_invoke_plan_report,
    nsld_verify_final_executable_writer_input_report, toml,
};
use std::{env, fs, path::Path};

#[test]
fn final_executable_readiness_reports_without_writing_blocked_artifact() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-readiness-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let report = nsld_final_executable_readiness_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::nsld_final_executable_readiness_report_json(&report);
    let blocked_path = nsld_artifact_stage_kind_path(
        &plan.output_dir,
        NsldArtifactStageKind::FinalExecutableBlocked,
    );
    let blocked_present = blocked_path.exists();
    fs::remove_dir_all(dir).unwrap();

    assert!(!report.emitted);
    assert!(!report.can_emit_final_executable);
    assert!(!report.final_stage_ready);
    assert_eq!(
        report.blocked_report_path,
        blocked_path.display().to_string()
    );
    assert!(!blocked_present);
    assert!(report
        .blockers
        .iter()
        .any(|blocker| blocker == "self-owned-final-native-linker"));
    assert_eq!(report.writer_kind, "host-assisted-final-executable");
    assert_eq!(report.writer_status, "blocked");
    assert_eq!(
        report.writer_blockers,
        vec!["final-executable-writer:host-assisted:not-implemented".to_owned()]
    );
    assert!(report_json.contains("\"kind\":\"nsld_final_executable_readiness\""));
    assert!(report_json.contains("\"emitted\":false"));
    assert!(report_json.contains("\"writer_kind\":\"host-assisted-final-executable\""));
    assert!(report_json.contains("\"writer_status\":\"blocked\""));
    assert!(report_json.contains("final-executable-writer:host-assisted:not-implemented"));
    assert!(report_json.contains("\"blocked_report_path\":\""));
}

#[test]
fn final_executable_writer_plan_reports_host_assisted_steps() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-writer-plan-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let report = nsld_final_executable_writer_plan_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::nsld_final_executable_writer_plan_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert_eq!(report.writer_kind, "host-assisted-final-executable");
    assert_eq!(report.writer_status, "blocked");
    assert!(report.final_stage_plan_hash.starts_with("0x"));
    assert_eq!(report.final_stage_driver, "clang");
    assert_eq!(report.final_stage_link_mode, "host-toolchain-finalize");
    assert!(report.host_wrapper_required);
    assert_eq!(report.input_count, 4);
    assert_eq!(report.inputs.len(), 4);
    assert!(report.inputs.iter().any(|input| {
        input.input_id == "fsi0003.native-object" && input.required && input.present
    }));
    assert!(report
        .writer_steps
        .iter()
        .any(|step| step == "prepare-host-assisted-entry-wrapper"));
    assert!(report
        .writer_steps
        .iter()
        .any(|step| step == "invoke-host-finalizer-driver"));
    assert_eq!(
        report.writer_blockers,
        vec!["final-executable-writer:host-assisted:not-implemented".to_owned()]
    );
    assert!(report
        .notes
        .iter()
        .any(|note| note == "final-executable-writer-plan-is-non-mutating"));
    assert!(report_json.contains("\"kind\":\"nsld_final_executable_writer_plan\""));
    assert!(report_json.contains("\"writer_kind\":\"host-assisted-final-executable\""));
    assert!(report_json.contains("\"writer_steps\":["));
    assert!(report_json.contains(
        "\"writer_blockers\":[\"final-executable-writer:host-assisted:not-implemented\"]"
    ));
}

#[test]
fn final_executable_writer_input_emit_and_verify_are_deterministic() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-writer-input-{}",
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
    let verify =
        nsld_verify_final_executable_writer_input_report(Path::new("manifest.toml"), &plan);
    let source = fs::read_to_string(&emit.output_path).unwrap();
    let emit_json = super::json::nsld_final_executable_writer_input_emit_report_json(&emit);
    let verify_json = super::json::nsld_final_executable_writer_input_verify_report_json(&verify);
    fs::remove_dir_all(dir).unwrap();

    assert!(Path::new(&emit.output_path)
        .file_name()
        .unwrap()
        .to_string_lossy()
        .contains("final-executable-writer-input"));
    assert!(emit.writer_input_hash.starts_with("0x"));
    assert_eq!(emit.writer_kind, "host-assisted-final-executable");
    assert_eq!(emit.writer_status, "blocked");
    assert_eq!(emit.final_stage_driver, "clang");
    assert_eq!(emit.final_stage_link_mode, "host-toolchain-finalize");
    assert!(emit.host_wrapper_required);
    assert!(emit.command_arg_count >= 4);
    assert_eq!(
        emit.writer_blockers,
        vec!["final-executable-writer:host-assisted:not-implemented".to_owned()]
    );
    assert!(verify.valid, "{:?}", verify.issues);
    assert_eq!(
        verify.actual_writer_input_hash.as_deref(),
        Some(emit.writer_input_hash.as_str())
    );
    assert_eq!(
        verify.actual_final_stage_plan_hash.as_deref(),
        Some(emit.final_stage_plan_hash.as_str())
    );
    assert_eq!(
        verify.actual_command_arg_count,
        Some(emit.command_arg_count)
    );
    assert_eq!(verify.expected_command_args, verify.actual_command_args);
    assert!(verify
        .actual_command_args
        .iter()
        .any(|arg| arg.contains("nuis.nsld.mach-o")));
    assert!(source.contains("schema = \"nuis-nsld-final-executable-writer-input-v1\""));
    assert!(source.contains("command_args = ["));
    assert!(source.contains("nuis.nsld.mach-o"));
    assert!(source
        .contains("writer_blockers = [\"final-executable-writer:host-assisted:not-implemented\"]"));
    assert!(emit_json.contains("\"kind\":\"nsld_final_executable_writer_input_emit\""));
    assert!(verify_json.contains("\"kind\":\"nsld_final_executable_writer_input_verify\""));
    assert!(verify_json.contains("\"valid\":true"));
    assert!(verify_json.contains("\"actual_command_args\":["));
}

#[test]
fn final_executable_host_dry_run_reports_missing_driver_without_invoking() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-host-dry-run-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();
    plan.final_stage.driver = "definitely-missing-nsld-host-driver-for-test".to_owned();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_executable_writer_input_report(Path::new("manifest.toml"), &plan).unwrap();
    let report = nsld_final_executable_host_dry_run_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::nsld_final_executable_host_dry_run_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(report.writer_input_valid);
    assert_eq!(
        report.driver,
        "definitely-missing-nsld-host-driver-for-test"
    );
    assert!(!report.driver_available);
    assert_eq!(report.driver_resolved_path, None);
    assert!(!report.environment_ready);
    assert_eq!(report.invocation_policy, "dry-run-only");
    assert_eq!(
        report.invocation_policy_reason,
        "alpha-host-finalizer-execution-disabled"
    );
    assert!(!report.can_invoke_host_finalizer);
    assert!(report
        .command_args
        .iter()
        .any(|arg| arg == "definitely-missing-nsld-host-driver-for-test"));
    assert!(report.blockers.iter().any(|blocker| blocker
        == "host-finalizer-driver-unavailable:definitely-missing-nsld-host-driver-for-test"));
    assert!(report
        .blockers
        .iter()
        .any(|blocker| blocker == "final-executable-writer:host-assisted:not-implemented"));
    assert!(report
        .blockers
        .iter()
        .any(|blocker| blocker == "host-finalizer-policy:dry-run-only"));
    assert!(report
        .notes
        .iter()
        .any(|note| note == "host-finalizer-is-not-invoked"));
    assert!(report_json.contains("\"kind\":\"nsld_final_executable_host_dry_run\""));
    assert!(report_json.contains("\"driver_available\":false"));
    assert!(report_json.contains("\"invocation_policy\":\"dry-run-only\""));
    assert!(report_json.contains("\"can_invoke_host_finalizer\":false"));
}

#[test]
fn final_executable_host_invoke_plan_requires_explicit_allow() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-host-invoke-plan-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();
    plan.final_stage.driver = "definitely-missing-nsld-host-driver-for-invoke-plan-test".to_owned();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_executable_writer_input_report(Path::new("manifest.toml"), &plan).unwrap();
    let report = nsld_final_executable_host_invoke_plan_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::nsld_final_executable_host_invoke_plan_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert_eq!(report.invocation_kind, "host-finalizer-command");
    assert_eq!(report.invocation_policy, "dry-run-only");
    assert!(report.requires_explicit_allow);
    assert!(!report.explicit_allow_present);
    assert!(!report.environment_ready);
    assert!(!report.driver_available);
    assert!(!report.can_invoke_host_finalizer);
    assert!(!report.would_invoke);
    assert!(report
        .blockers
        .iter()
        .any(|blocker| blocker == "host-finalizer-explicit-allow:missing"));
    assert!(report
        .blockers
        .iter()
        .any(|blocker| blocker == "host-finalizer-policy:dry-run-only"));
    assert!(report
        .notes
        .iter()
        .any(|note| note == "host-finalizer-process-is-not-spawned"));
    assert!(report_json.contains("\"kind\":\"nsld_final_executable_host_invoke_plan\""));
    assert!(report_json.contains("\"requires_explicit_allow\":true"));
    assert!(report_json.contains("\"would_invoke\":false"));
}

#[test]
fn final_executable_host_invoke_plan_emit_and_verify_round_trip() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-host-invoke-plan-emit-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();
    plan.final_stage.driver =
        "definitely-missing-nsld-host-driver-for-invoke-plan-emit-test".to_owned();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_executable_writer_input_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit =
        nsld_emit_final_executable_host_invoke_plan_report(Path::new("manifest.toml"), &plan)
            .unwrap();
    let verify =
        nsld_verify_final_executable_host_invoke_plan_report(Path::new("manifest.toml"), &plan);
    let emit_json = super::json::nsld_final_executable_host_invoke_plan_emit_report_json(&emit);
    let verify_json =
        super::json::nsld_final_executable_host_invoke_plan_verify_report_json(&verify);
    let report_source = fs::read_to_string(&emit.output_path).unwrap();
    fs::remove_dir_all(dir).unwrap();

    assert!(emit.invoke_plan_hash.starts_with("0x"));
    assert_eq!(emit.invocation_policy, "dry-run-only");
    assert!(emit.requires_explicit_allow);
    assert!(!emit.explicit_allow_present);
    assert!(!emit.would_invoke);
    assert!(emit.blocker_count > 0);
    assert!(verify.valid, "{:?}", verify.issues);
    assert_eq!(
        verify.actual_invoke_plan_hash.as_deref(),
        Some(emit.invoke_plan_hash.as_str())
    );
    assert!(report_source.contains("schema = \"nuis-nsld-final-executable-host-invoke-plan-v1\""));
    assert!(report_source.contains("would_invoke = false"));
    assert!(emit_json.contains("\"kind\":\"nsld_final_executable_host_invoke_plan_emit\""));
    assert!(verify_json.contains("\"kind\":\"nsld_final_executable_host_invoke_plan_verify\""));
    assert!(verify_json.contains("\"valid\":true"));
}

#[test]
fn verify_final_executable_host_invoke_plan_reports_gate_drift() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-emit-host-invoke-plan-drift-{}",
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
    let source = fs::read_to_string(&emit.output_path).unwrap();
    let command_arg_count_line = source
        .lines()
        .find(|line| line.starts_with("command_arg_count = "))
        .unwrap()
        .to_owned();
    let damaged = source
        .replace("would_invoke = false", "would_invoke = true")
        .replace(&command_arg_count_line, "command_arg_count = 0")
        .replace(
            "command_args = [\"clang\",",
            "command_args = [\"clang-drift\",",
        )
        .replace(
            "blockers = [\"final-executable-writer:host-assisted:not-implemented\", \"host-finalizer-policy:dry-run-only\", \"host-finalizer-explicit-allow:missing\"]",
            "blockers = [\"tampered-host-finalizer-blocker\", \"host-finalizer-policy:dry-run-only\", \"host-finalizer-explicit-allow:missing\"]",
        );
    fs::write(&emit.output_path, damaged).unwrap();
    let verify =
        nsld_verify_final_executable_host_invoke_plan_report(Path::new("manifest.toml"), &plan);
    let verify_json =
        super::json::nsld_final_executable_host_invoke_plan_verify_report_json(&verify);
    fs::remove_dir_all(dir).unwrap();

    assert!(!verify.valid);
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue == "would_invoke mismatch: expected false, found true"));
    assert!(verify.expected_command_arg_count > 0);
    assert_eq!(verify.actual_command_arg_count, Some(0));
    assert!(verify
        .expected_command_args
        .iter()
        .any(|arg| arg == "clang"));
    assert!(verify
        .actual_command_args
        .iter()
        .any(|arg| arg == "clang-drift"));
    assert!(verify
        .expected_blockers
        .iter()
        .any(|blocker| blocker == "final-executable-writer:host-assisted:not-implemented"));
    assert!(verify
        .actual_blockers
        .iter()
        .any(|blocker| blocker == "tampered-host-finalizer-blocker"));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("command_arg_count mismatch: expected ")));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("command_args mismatch")));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("blockers mismatch")));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue == "final-executable-host-invoke-plan-content-mismatch"));
    assert!(verify_json.contains("\"actual_would_invoke\":true"));
    assert!(verify_json.contains("\"actual_command_arg_count\":0"));
    assert!(verify_json.contains("\"clang-drift\""));
    assert!(verify_json.contains("\"tampered-host-finalizer-blocker\""));
}

#[test]
fn verify_final_executable_writer_input_reports_command_args_drift() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-writer-input-args-drift-{}",
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
    let source = fs::read_to_string(&emit.output_path).unwrap();
    let damaged = source.replace(
        "command_args = [\"clang\",",
        "command_args = [\"clang-drift\",",
    );
    fs::write(&emit.output_path, damaged).unwrap();
    let verify =
        nsld_verify_final_executable_writer_input_report(Path::new("manifest.toml"), &plan);
    fs::remove_dir_all(dir).unwrap();

    assert!(!verify.valid);
    assert_ne!(verify.expected_command_args, verify.actual_command_args);
    assert!(verify
        .actual_command_args
        .iter()
        .any(|arg| arg == "clang-drift"));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("command_args mismatch")));
}

#[test]
fn verify_final_executable_writer_input_reports_plan_hash_drift() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-emit-writer-input-drift-{}",
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
    let writer_blockers_line = format!(
        "writer_blockers = [{}]",
        toml::toml_string_array_literal(&emit.writer_blockers)
    );
    let damaged = fs::read_to_string(&emit.output_path)
        .unwrap()
        .replace(
            &format!("final_stage_plan_hash = \"{}\"", emit.final_stage_plan_hash),
            "final_stage_plan_hash = \"0x3333333333333333\"",
        )
        .replace(
            &writer_blockers_line,
            "writer_blockers = [\"tampered-writer-blocker\"]",
        );
    fs::write(&emit.output_path, damaged).unwrap();
    let verify =
        nsld_verify_final_executable_writer_input_report(Path::new("manifest.toml"), &plan);
    fs::remove_dir_all(dir).unwrap();

    assert!(!verify.valid);
    assert_eq!(
        verify.actual_final_stage_plan_hash.as_deref(),
        Some("0x3333333333333333")
    );
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.contains("final_stage_plan_hash mismatch")));
    assert_eq!(
        verify.actual_writer_blockers,
        vec!["tampered-writer-blocker".to_owned()]
    );
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("writer_blockers mismatch")));
}

#[test]
fn emit_final_executable_records_host_dry_run_driver_blocker() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-host-driver-blocked-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();
    plan.final_stage.driver = "definitely-missing-nsld-host-driver-for-emit-test".to_owned();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_executable_writer_input_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit = nsld_emit_final_executable_report(Path::new("manifest.toml"), &plan).unwrap();
    let report_source = fs::read_to_string(&emit.blocked_report_path).unwrap();
    fs::remove_dir_all(dir).unwrap();

    assert_eq!(emit.writer_input_valid, Some(true));
    assert_eq!(emit.host_dry_run_environment_ready, Some(false));
    assert_eq!(emit.host_dry_run_driver_available, Some(false));
    assert_eq!(emit.host_dry_run_driver_resolved_path, None);
    assert_eq!(emit.host_dry_run_can_invoke, Some(false));
    assert_eq!(
        emit.host_dry_run_invocation_policy.as_deref(),
        Some("dry-run-only")
    );
    assert_eq!(
        emit.host_dry_run_invocation_policy_reason.as_deref(),
        Some("alpha-host-finalizer-execution-disabled")
    );
    assert_eq!(
        emit.host_dry_run_blocker_count,
        emit.host_dry_run_blockers.len()
    );
    assert!(emit.host_dry_run_blockers.iter().any(|blocker| blocker
        == "host-finalizer-driver-unavailable:definitely-missing-nsld-host-driver-for-emit-test"));
    assert!(emit
        .blockers
        .iter()
        .any(|blocker| blocker == "host-finalizer-environment:not-ready"));
    assert!(report_source.contains("host_dry_run_environment_ready = false"));
    assert!(report_source.contains("host_dry_run_driver_available = false"));
    assert!(report_source.contains("host_dry_run_driver_resolved_path = \"\""));
    assert!(report_source.contains("host_dry_run_can_invoke = false"));
    assert!(report_source.contains("host_dry_run_invocation_policy = \"dry-run-only\""));
    assert!(report_source.contains(
        "host_dry_run_invocation_policy_reason = \"alpha-host-finalizer-execution-disabled\""
    ));
    assert!(report_source.contains("host_dry_run_blocker_count = "));
    assert!(report_source.contains(
        "host-finalizer-driver-unavailable:definitely-missing-nsld-host-driver-for-emit-test"
    ));
}
