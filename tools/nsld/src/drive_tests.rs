use super::{
    final_output_boundary_crossing_enabled_for, nsld_drive_apply_next_action,
    nsld_drive_apply_report_json, nsld_drive_apply_until_clean, nsld_drive_until_clean_report_json,
    run_drive_command, NsldDriveUntilCleanReport,
};
use crate::{
    commands::NsldCheckNextAction, main_test_support::empty_link_plan, nsld_check_report,
    nsld_emit_final_executable_image_dry_run_report, nsld_emit_final_executable_layout_plan_report,
    nsld_emit_final_executable_report, nsld_emit_final_executable_writer_input_report,
    nsld_final_executable_output_report,
};
use nuisc::aot::{BuildManifestContext, CompileArtifacts};
use std::path::Path;
use std::{env, fs};

#[test]
fn drive_apply_dispatches_whitelisted_emit_inputs() {
    let dir = env::temp_dir().join(format!(
        "nsld-drive-apply-emit-inputs-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    let next_action = NsldCheckNextAction {
        available: true,
        source: Some("required".to_owned()),
        command_id: Some("emit-inputs".to_owned()),
        command: Some("nsld emit-inputs <input>".to_owned()),
        command_resolved: Some("nsld emit-inputs manifest.toml".to_owned()),
        reason: Some("first missing required artifact stage `link-inputs`".to_owned()),
        gate_action: None,
        gate_env_assignments: Vec::new(),
        crossing_env_assignments: Vec::new(),
        crossing_command_resolved: None,
    };

    let report =
        nsld_drive_apply_next_action(Path::new("manifest.toml"), &plan, &next_action).unwrap();
    let output_present = dir.join("nuis.nsld.link-inputs.toml").exists();
    fs::remove_dir_all(dir).unwrap();

    assert!(report.applied);
    assert_eq!(report.command_id.as_deref(), Some("emit-inputs"));
    assert!(output_present);
    assert_eq!(report.message, "applied emit-inputs");
    assert!(nsld_drive_apply_report_json(&report).contains("\"mutates_artifacts\":true"));
    assert!(nsld_drive_apply_report_json(&report)
        .contains("\"mutation_policy\":\"whitelisted-artifact-mutation\""));
}

#[test]
fn drive_apply_dispatches_whitelisted_emit_object() {
    let dir = env::temp_dir().join(format!(
        "nsld-drive-apply-emit-object-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    let next_action = NsldCheckNextAction {
        available: true,
        source: Some("required".to_owned()),
        command_id: Some("emit-object".to_owned()),
        command: Some("nsld emit-object <input>".to_owned()),
        command_resolved: Some("nsld emit-object manifest.toml".to_owned()),
        reason: Some("first missing required artifact stage `object-emit-blocked`".to_owned()),
        gate_action: None,
        gate_env_assignments: Vec::new(),
        crossing_env_assignments: Vec::new(),
        crossing_command_resolved: None,
    };

    let report =
        nsld_drive_apply_next_action(Path::new("manifest.toml"), &plan, &next_action).unwrap();
    let blocked_report_present = dir.join("nuis.nsld.object.blocked.toml").exists();
    fs::remove_dir_all(dir).unwrap();

    assert!(report.applied);
    assert_eq!(report.command_id.as_deref(), Some("emit-object"));
    assert!(blocked_report_present);
    assert_eq!(report.message, "applied emit-object");
}

#[test]
fn drive_apply_dispatches_whitelisted_launcher_manifest_and_dry_run() {
    let dir = env::temp_dir().join(format!(
        "nsld-drive-apply-launcher-evidence-{}",
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
    super::nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    super::nsld_emit_final_stage_plan_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_executable_writer_input_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_executable_layout_plan_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_executable_image_dry_run_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_executable_report(Path::new("manifest.toml"), &plan).unwrap();
    let manifest_action = NsldCheckNextAction {
        available: true,
        source: Some("final-output-materialization".to_owned()),
        command_id: Some("emit-final-executable-launcher-manifest".to_owned()),
        command: Some("nsld emit-final-executable-launcher-manifest <input>".to_owned()),
        command_resolved: Some(
            "nsld emit-final-executable-launcher-manifest manifest.toml".to_owned(),
        ),
        reason: Some("final executable output is ready".to_owned()),
        gate_action: None,
        gate_env_assignments: Vec::new(),
        crossing_env_assignments: Vec::new(),
        crossing_command_resolved: None,
    };
    let dry_run_action = NsldCheckNextAction {
        available: true,
        source: Some("final-output-materialization".to_owned()),
        command_id: Some("emit-final-executable-launcher-dry-run".to_owned()),
        command: Some("nsld emit-final-executable-launcher-dry-run <input>".to_owned()),
        command_resolved: Some(
            "nsld emit-final-executable-launcher-dry-run manifest.toml".to_owned(),
        ),
        reason: Some("launcher manifest is ready".to_owned()),
        gate_action: None,
        gate_env_assignments: Vec::new(),
        crossing_env_assignments: Vec::new(),
        crossing_command_resolved: None,
    };

    let manifest_report =
        nsld_drive_apply_next_action(Path::new("manifest.toml"), &plan, &manifest_action).unwrap();
    let dry_run_report =
        nsld_drive_apply_next_action(Path::new("manifest.toml"), &plan, &dry_run_action).unwrap();
    let output = nsld_final_executable_output_report(Path::new("manifest.toml"), &plan);
    let launcher_manifest_present = dir
        .join("nuis.nsld.final-executable-launcher.toml")
        .exists();
    let launcher_dry_run_present = dir
        .join("nuis.nsld.final-executable-launcher-dry-run.toml")
        .exists();
    fs::remove_dir_all(dir).unwrap();

    assert!(manifest_report.applied);
    assert_eq!(
        manifest_report.message,
        "applied emit-final-executable-launcher-manifest"
    );
    assert!(dry_run_report.applied);
    assert_eq!(
        dry_run_report.message,
        "applied emit-final-executable-launcher-dry-run"
    );
    assert!(launcher_manifest_present);
    assert!(launcher_dry_run_present);
    assert_eq!(
        output.entrypoint_materialization_evidence_status,
        "launcher-dry-run-ready"
    );
    assert_eq!(
        output.recommended_next_action,
        "handoff-to-container-loader"
    );
}

#[test]
fn drive_apply_dispatches_native_object_alias() {
    let dir = env::temp_dir().join(format!(
        "nsld-drive-apply-emit-native-object-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    let next_action = NsldCheckNextAction {
        available: true,
        source: Some("required".to_owned()),
        command_id: Some("emit-native-object".to_owned()),
        command: Some("nsld emit-native-object <input>".to_owned()),
        command_resolved: Some("nsld emit-native-object manifest.toml".to_owned()),
        reason: Some("first missing required native object output".to_owned()),
        gate_action: None,
        gate_env_assignments: Vec::new(),
        crossing_env_assignments: Vec::new(),
        crossing_command_resolved: None,
    };

    let report =
        nsld_drive_apply_next_action(Path::new("manifest.toml"), &plan, &next_action).unwrap();
    let blocked_report_present = dir.join("nuis.nsld.object.blocked.toml").exists();
    fs::remove_dir_all(dir).unwrap();

    assert!(report.applied);
    assert_eq!(report.command_id.as_deref(), Some("emit-native-object"));
    assert!(blocked_report_present);
    assert_eq!(report.message, "applied emit-native-object");
    let json = nsld_drive_apply_report_json(&report);
    assert!(json.contains("\"applied\":true"));
    assert!(json.contains("\"mutates_artifacts\":true"));
    assert!(json.contains("\"mutation_policy\":\"whitelisted-artifact-mutation\""));
}

#[test]
fn final_output_boundary_crossing_requires_policy_and_explicit_allow() {
    assert!(final_output_boundary_crossing_enabled_for(
        Some("allow-host-invoke"),
        Some("1")
    ));
    assert!(final_output_boundary_crossing_enabled_for(
        Some("ALLOW"),
        Some("YES")
    ));
    assert!(!final_output_boundary_crossing_enabled_for(
        Some("allow-host-invoke"),
        None
    ));
    assert!(!final_output_boundary_crossing_enabled_for(None, Some("1")));
    assert!(!final_output_boundary_crossing_enabled_for(
        Some("dry-run-only"),
        Some("1")
    ));
}

#[test]
fn drive_until_clean_json_can_report_native_object_alias_step() {
    let next_action = NsldCheckNextAction {
        available: true,
        source: Some("required".to_owned()),
        command_id: Some("emit-native-object".to_owned()),
        command: Some("nsld emit-native-object <input>".to_owned()),
        command_resolved: Some("nsld emit-native-object manifest.toml".to_owned()),
        reason: Some("first missing required native object output".to_owned()),
        gate_action: None,
        gate_env_assignments: Vec::new(),
        crossing_env_assignments: Vec::new(),
        crossing_command_resolved: None,
    };
    let report = NsldDriveUntilCleanReport {
        completed: true,
        applied_steps: 1,
        capped: false,
        stop_reason: "clean".to_owned(),
        stop_command_id: None,
        stop_source: None,
        stop_command_resolved: None,
        stop_action_reason: None,
        stop_gate_action: None,
        stop_gate_env_assignments: Vec::new(),
        stop_crossing_env_assignments: Vec::new(),
        stop_crossing_command_resolved: None,
        last_command_id: next_action.command_id.clone(),
        messages: vec![
            "applied emit-native-object".to_owned(),
            "no-next-action".to_owned(),
        ],
    };
    let json = nsld_drive_until_clean_report_json(&report);

    assert!(json.contains("\"stop_reason\":\"clean\""));
    assert!(json.contains("\"stop_command_id\":null"));
    assert!(json.contains("\"last_command_id\":\"emit-native-object\""));
    assert!(json.contains("\"applied emit-native-object\""));
}

#[test]
fn drive_apply_until_clean_materializes_self_contained_pipeline() {
    let dir = env::temp_dir().join(format!(
        "nsld-drive-until-clean-self-contained-{}",
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

    let report = nsld_drive_apply_until_clean(Path::new("manifest.toml"), &plan).unwrap();
    let check = nsld_check_report(Path::new("manifest.toml"), &plan);
    let final_output_present = Path::new(&plan.final_stage.output_path).exists();
    fs::remove_dir_all(dir).unwrap();

    assert!(report.completed, "{:?}", report.messages);
    assert!(!report.capped);
    assert_eq!(report.stop_reason, "clean");
    assert_eq!(report.stop_command_id, None);
    assert_eq!(report.stop_source, None);
    assert_eq!(report.stop_command_resolved, None);
    assert_eq!(report.stop_action_reason, None);
    assert_eq!(
        report.last_command_id.as_deref(),
        Some("emit-final-executable-pipeline")
    );
    assert!(report.applied_steps >= 5, "{:?}", report.messages);
    assert!(report
        .messages
        .iter()
        .any(|message| message == "applied emit-inputs"));
    assert!(report
        .messages
        .iter()
        .any(|message| message == "applied emit-final-executable-pipeline"));
    assert_eq!(
        report.messages.last().map(String::as_str),
        Some("no-next-action")
    );
    assert!(check.valid, "{:?}", check.issues);
    assert!(check.final_executable_pipeline_present);
    assert_eq!(check.final_executable_pipeline_ready, Some(true));
    assert!(final_output_present);
}

#[test]
fn drive_apply_until_clean_materializes_manifest_selected_self_contained_route() {
    let dir = env::temp_dir().join(format!(
        "nsld-drive-until-clean-manifest-self-contained-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let manifest = write_test_build_manifest_with_packaging_mode(&dir, "nuis-self-contained-image");
    let plan = nuisc::linker::build_link_plan_from_manifest(Path::new(&manifest)).unwrap();

    let report = nsld_drive_apply_until_clean(Path::new(&manifest), &plan).unwrap();
    let check = nsld_check_report(Path::new(&manifest), &plan);
    let final_output_present = Path::new(&plan.final_stage.output_path).exists();
    fs::remove_dir_all(dir).unwrap();

    assert_eq!(plan.final_stage.kind, "nuis-self-contained-image");
    assert_eq!(plan.final_stage.driver, "nsld-internal-image-writer");
    assert_eq!(plan.final_stage.link_mode, "self-contained");
    assert!(plan.final_stage.output_path.ends_with(".nsb"));
    assert!(report.completed, "{:?}", report.messages);
    assert!(!report.capped);
    assert_eq!(report.stop_reason, "clean");
    assert_eq!(
        report.last_command_id.as_deref(),
        Some("emit-final-executable-pipeline")
    );
    assert!(report
        .messages
        .iter()
        .any(|message| message == "applied emit-final-executable-pipeline"));
    assert!(check.valid, "{:?}", check.issues);
    assert!(check.final_executable_pipeline_present);
    assert_eq!(check.final_executable_pipeline_ready, Some(true));
    assert!(final_output_present);
}

#[test]
fn drive_apply_until_clean_reaches_host_assisted_pipeline_blockers() {
    let dir = env::temp_dir().join(format!(
        "nsld-drive-until-clean-host-assisted-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();
    plan.final_stage.output_path = dir.join("demo").display().to_string();

    let report = nsld_drive_apply_until_clean(Path::new("manifest.toml"), &plan).unwrap();
    let check = nsld_check_report(Path::new("manifest.toml"), &plan);
    let final_output_present = Path::new(&plan.final_stage.output_path).exists();
    fs::remove_dir_all(dir).unwrap();

    assert!(!report.completed, "{:?}", report.messages);
    assert!(!report.capped);
    assert_eq!(report.stop_reason, "final-output-missing");
    assert_eq!(
        report.stop_command_id.as_deref(),
        Some("final-executable-output")
    );
    assert_eq!(report.stop_source.as_deref(), Some("final-output-boundary"));
    assert!(report
        .stop_command_resolved
        .as_deref()
        .is_some_and(|command| command.contains("nsld final-executable-output")));
    assert!(report
        .stop_action_reason
        .as_deref()
        .is_some_and(|reason| reason.contains("final-executable-output")));
    assert_eq!(
        report.last_command_id.as_deref(),
        Some("emit-final-executable-pipeline")
    );
    assert!(report.applied_steps >= 5, "{:?}", report.messages);
    assert!(report
        .messages
        .iter()
        .any(|message| message == "applied emit-final-executable-pipeline"));
    assert_eq!(
        report.messages.last().map(String::as_str),
        Some("read-only-boundary:final-executable-output")
    );
    assert!(
        check.valid,
        "failures={} issues={:?} artifact_chain={:?}",
        check.failures, check.issues, check.artifact_chain_issues
    );
    assert!(check.final_executable_pipeline_present);
    assert_eq!(check.final_executable_pipeline_ready, Some(false));
    assert!(!final_output_present);
    assert!(check.object_output_present);
    assert_eq!(check.object_output_valid, Some(true));
    assert!(check.container_present);
    assert_eq!(check.container_valid, Some(true));
    assert!(check.closure_snapshot_present);
    assert_eq!(check.closure_snapshot_valid, Some(true));
    assert!(check
        .final_executable_blocked_blocker_count
        .is_some_and(|count| count > 0));
    assert!(check
        .final_executable_host_invoke_plan_blocker_count
        .is_some_and(|count| count > 0));
    assert!(check
        .final_executable_pipeline_blocker_count
        .is_some_and(|count| count > 0));
}

#[test]
fn drive_apply_command_loads_manifest_directory_and_emits_next_artifact() {
    let dir = env::temp_dir().join(format!(
        "nsld-drive-command-manifest-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let manifest = write_test_build_manifest(&dir);
    let link_inputs = dir.join("nuis.nsld.link-inputs.toml");

    assert!(!link_inputs.exists());
    run_drive_command(&dir, false, true, false).unwrap();
    let emitted = link_inputs.exists();
    fs::remove_dir_all(dir).unwrap();

    assert!(manifest.ends_with("nuis.build.manifest.toml"));
    assert!(emitted);
}

#[test]
fn drive_until_clean_command_reaches_host_assisted_pipeline_block() {
    let dir = env::temp_dir().join(format!(
        "nsld-drive-command-until-clean-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let manifest = write_test_build_manifest(&dir);

    run_drive_command(&dir, true, true, true).unwrap();
    let plan = nuisc::linker::build_link_plan_from_manifest(Path::new(&manifest)).unwrap();
    let check = nsld_check_report(Path::new(&manifest), &plan);
    let output = nsld_final_executable_output_report(Path::new(&manifest), &plan);
    let drive_report = nsld_drive_apply_until_clean(Path::new(&manifest), &plan).unwrap();
    let drive_json = nsld_drive_until_clean_report_json(&drive_report);
    let expected_boundary_command = format!("nsld final-executable-output {manifest}");
    let expected_crossing_command = format!(
        "env NUIS_NSLD_HOST_FINALIZER_POLICY=allow-host-invoke NUIS_NSLD_ALLOW_HOST_FINALIZER=1 {expected_boundary_command}"
    );
    let final_output_present = Path::new(&plan.final_stage.output_path).exists();
    fs::remove_dir_all(dir).unwrap();

    assert!(
        check.valid,
        "failures={} issues={:?} artifact_chain={:?}",
        check.failures, check.issues, check.artifact_chain_issues
    );
    assert!(!check.artifact_chain_next_action_available);
    assert_eq!(check.artifact_chain_next_action_command_id, None);
    assert_eq!(check.artifact_chain_next_action_source, None);
    assert!(check.next_action_available);
    assert_eq!(drive_report.stop_reason, "host-finalizer-policy-required");
    assert!(drive_json.contains("\"mutation_policy\":\"blocked-read-only-boundary\""));
    assert_eq!(
        drive_report.stop_command_id.as_deref(),
        Some("final-executable-output")
    );
    assert!(drive_report
        .stop_action_reason
        .as_deref()
        .is_some_and(|reason| reason.contains(
            "next_gate_action:set-env:NUIS_NSLD_HOST_FINALIZER_POLICY=allow-host-invoke"
        )));
    assert_eq!(
        drive_report.stop_gate_action.as_deref(),
        Some("set-env:NUIS_NSLD_HOST_FINALIZER_POLICY=allow-host-invoke")
    );
    assert_eq!(
        drive_report.stop_gate_env_assignments,
        vec!["NUIS_NSLD_HOST_FINALIZER_POLICY=allow-host-invoke".to_owned()]
    );
    assert_eq!(
        drive_report.stop_crossing_env_assignments,
        vec![
            "NUIS_NSLD_HOST_FINALIZER_POLICY=allow-host-invoke".to_owned(),
            "NUIS_NSLD_ALLOW_HOST_FINALIZER=1".to_owned()
        ]
    );
    assert_eq!(
        drive_report.stop_crossing_command_resolved.as_deref(),
        Some(expected_crossing_command.as_str())
    );
    assert!(drive_json
        .contains("\"safe_next_action\":\"explicit-boundary-crossing-command-available\""));
    assert!(drive_json.contains(&format!(
        "\"safe_next_command\":\"{}\"",
        expected_crossing_command
    )));
    assert!(drive_json.contains(
        "\"safe_next_reason\":\"drive stopped at an explicit boundary; run the safe_next_command only if you accept the listed gate\""
    ));
    assert_eq!(
        check.next_action_source.as_deref(),
        Some("final-output-boundary")
    );
    assert_eq!(
        check.next_action_command_id.as_deref(),
        Some("final-executable-output")
    );
    assert_eq!(
        check.next_action_command_resolved.as_deref(),
        Some(expected_boundary_command.as_str())
    );
    assert!(!check.artifact_chain_final_output_boundary_ready);
    assert_eq!(
        check
            .artifact_chain_final_output_boundary_command_id
            .as_deref(),
        Some("final-executable-output")
    );
    assert_eq!(
        check
            .artifact_chain_final_output_boundary_command_resolved
            .as_deref(),
        Some(expected_boundary_command.as_str())
    );
    assert!(check
        .artifact_chain_final_output_boundary_reason
        .as_deref()
        .is_some_and(|reason| reason.contains("final-executable-output:not-nsld-owned")));
    assert!(final_output_present);
    assert!(output.path_present);
    assert!(!output.nsld_owned_output);
    assert!(!output.present);
    assert!(output
        .blockers
        .iter()
        .any(|blocker| blocker == "final-executable-output:not-nsld-owned"));
    assert!(!output
        .blockers
        .iter()
        .any(|blocker| blocker == "final-executable-output:missing"));
    assert!(check.final_executable_output_path_present);
    assert!(!check.final_executable_output_nsld_owned);
    assert!(!check.final_executable_output_present);
    assert_eq!(
        check.final_executable_output_runnable_candidate,
        Some(false)
    );
    assert!(check
        .final_executable_output_blockers
        .iter()
        .any(|blocker| blocker == "final-executable-output:not-nsld-owned"));
    assert_eq!(
        check.final_executable_host_finalizer_gate_status.as_deref(),
        Some("policy-blocked")
    );
    assert_eq!(
        check.final_executable_host_finalizer_gate_action.as_deref(),
        Some("set-env:NUIS_NSLD_HOST_FINALIZER_POLICY=allow-host-invoke")
    );
}

#[test]
fn drive_until_clean_command_materializes_self_contained_manifest_route() {
    let dir = env::temp_dir().join(format!(
        "nsld-drive-command-self-contained-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let manifest = write_test_build_manifest_with_packaging_mode(&dir, "nuis-self-contained-image");

    run_drive_command(&dir, true, true, true).unwrap();
    let plan = nuisc::linker::build_link_plan_from_manifest(Path::new(&manifest)).unwrap();
    let check = nsld_check_report(Path::new(&manifest), &plan);
    let final_output_present = Path::new(&plan.final_stage.output_path).exists();
    fs::remove_dir_all(dir).unwrap();

    assert_eq!(plan.final_stage.kind, "nuis-self-contained-image");
    assert_eq!(plan.final_stage.driver, "nsld-internal-image-writer");
    assert_eq!(plan.final_stage.link_mode, "self-contained");
    assert!(check.valid, "{:?}", check.issues);
    assert!(!check.next_action_available);
    assert!(check.final_executable_pipeline_present);
    assert_eq!(check.final_executable_pipeline_ready, Some(true));
    assert!(final_output_present);
}

#[test]
fn drive_until_clean_keeps_generic_stop_for_unknown_final_output_boundary() {
    let report = NsldDriveUntilCleanReport {
        completed: false,
        applied_steps: 0,
        capped: false,
        stop_reason: "blocked-boundary".to_owned(),
        stop_command_id: Some("final-executable-output".to_owned()),
        stop_source: Some("final-output-boundary".to_owned()),
        stop_command_resolved: Some("nsld final-executable-output manifest.toml".to_owned()),
        stop_action_reason: Some("final executable output boundary is blocked".to_owned()),
        stop_gate_action: None,
        stop_gate_env_assignments: Vec::new(),
        stop_crossing_env_assignments: Vec::new(),
        stop_crossing_command_resolved: None,
        last_command_id: None,
        messages: vec!["read-only-boundary:final-executable-output".to_owned()],
    };
    let json = nsld_drive_until_clean_report_json(&report);

    assert!(json.contains("\"stop_reason\":\"blocked-boundary\""));
    assert!(json.contains("\"mutation_policy\":\"blocked-boundary\""));
    assert!(json.contains("\"stop_source\":\"final-output-boundary\""));
}

fn write_test_build_manifest(dir: &Path) -> String {
    write_test_build_manifest_with_packaging_mode(dir, "native-cpu-llvm")
}

fn write_test_build_manifest_with_packaging_mode(dir: &Path, packaging_mode: &str) -> String {
    let ast = dir.join("demo.ast.txt");
    let nir = dir.join("demo.nir.txt");
    let yir = dir.join("demo.yir");
    let ll = dir.join("demo.ll");
    let bin = dir.join("demo.bin");
    fs::write(&ast, "ast").unwrap();
    fs::write(&nir, "nir").unwrap();
    fs::write(&yir, "yir").unwrap();
    fs::write(&ll, "llvm").unwrap();
    fs::write(&bin, "bin").unwrap();

    let written = CompileArtifacts {
        ast_path: ast.display().to_string(),
        nir_path: nir.display().to_string(),
        yir_path: yir.display().to_string(),
        llvm_ir_path: ll.display().to_string(),
        binary_path: bin.display().to_string(),
        packaging_mode: packaging_mode.to_owned(),
    };
    nuisc::aot::write_build_manifest(
        dir,
        &written,
        &BuildManifestContext {
            input_path: dir.join("demo.ns").display().to_string(),
            output_dir: dir.display().to_string(),
            loaded_nustar: vec!["official.cpu".to_owned()],
            compile_cache: None,
            project: None,
            doc_index: None,
            cpu_target: nuisc::aot::host_cpu_build_target(),
        },
    )
    .unwrap()
}
