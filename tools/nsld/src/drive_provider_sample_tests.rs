use super::{
    final_output_boundary_stop_reason, nsld_drive_apply_next_action, nsld_drive_apply_report_json,
    nsld_drive_apply_until_clean, nsld_drive_until_clean_report_json, NsldDriveUntilCleanReport,
};
use crate::{commands::NsldCheckNextAction, main_test_support::empty_link_plan, nsld_check_report};
use std::path::Path;
use std::{env, fs};

#[test]
fn drive_apply_materializes_provider_sample_boundary_via_nsdb() {
    let dir = env::temp_dir().join(format!("nsld-drive-provider-sample-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    write_device_provider_sample_manifest(&dir);
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    let next_action = NsldCheckNextAction {
        available: true,
        source: Some("final-output-boundary".to_owned()),
        command_id: Some("materialize-provider-samples".to_owned()),
        command: Some("nsdb materialize-provider-samples <artifact-output-dir> --json".to_owned()),
        command_resolved: Some(format!(
            "nsdb materialize-provider-samples {} --json",
            dir.display()
        )),
        reason: Some(
            "final executable output boundary is blocked by `device-provider-sample:metal:apple-silicon-gpu:pending:1`; materialize provider samples before relinking"
                .to_owned(),
        ),
        gate_action: None,
        gate_env_assignments: Vec::new(),
        crossing_env_assignments: Vec::new(),
        crossing_command_resolved: None,
    };

    let report =
        nsld_drive_apply_next_action(Path::new("manifest.toml"), &plan, &next_action).unwrap();
    let manifest = fs::read_to_string(dir.join("nuis.nsdb.device-provider-samples.toml")).unwrap();
    fs::remove_dir_all(dir).unwrap();

    assert!(report.applied);
    assert_eq!(
        report.command_id.as_deref(),
        Some("materialize-provider-samples")
    );
    assert_eq!(
        report.message,
        "applied materialize-provider-samples:ready:1"
    );
    assert!(manifest.contains("materialization_status = \"provider-sample-materialized\""));
    let json = nsld_drive_apply_report_json(&report);
    assert!(json.contains("\"applied\":true"));
    assert!(json.contains("\"mutates_artifacts\":true"));
    assert!(json.contains("\"mutation_policy\":\"whitelisted-boundary-materialization\""));
    assert!(json.contains("\"command_id\":\"materialize-provider-samples\""));
    assert!(json.contains("nsdb materialize-provider-samples"));
    assert!(json.contains("\"safe_next_contract\":\"nsld-drive-safe-next-v1\""));
    assert!(json.contains("\"safe_next_action\":\"rerun-drive-to-refresh-next-action\""));
    assert!(json.contains("\"safe_next_command\":null"));
    assert!(json.contains("\"safe_next_gate_required\":false"));
    assert!(json.contains(
        "\"safe_next_reason\":\"drive applied one mutation; rerun drive to observe the next deterministic action\""
    ));
}

#[test]
fn drive_apply_keeps_blocked_provider_sample_repair_boundary_read_only() {
    let dir = env::temp_dir().join(format!(
        "nsld-drive-provider-sample-repair-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    let next_action = NsldCheckNextAction {
        available: true,
        source: Some("final-output-boundary".to_owned()),
        command_id: Some("repair-provider-output-payload".to_owned()),
        command: Some("nsdb materialize-provider-samples <artifact-output-dir> --json".to_owned()),
        command_resolved: Some(format!(
            "nsdb materialize-provider-samples {} --json",
            dir.display()
        )),
        reason: Some(
            "final executable output boundary is blocked by `device-provider-sample:metal:apple-silicon-gpu:blocked:1:provider-sample-blocked`; repair provider output payload diagnostics before relinking"
                .to_owned(),
        ),
        gate_action: None,
        gate_env_assignments: Vec::new(),
        crossing_env_assignments: Vec::new(),
        crossing_command_resolved: None,
    };

    let report =
        nsld_drive_apply_next_action(Path::new("manifest.toml"), &plan, &next_action).unwrap();
    fs::remove_dir_all(dir).unwrap();

    assert!(!report.applied);
    assert_eq!(
        report.command_id.as_deref(),
        Some("repair-provider-output-payload")
    );
    assert_eq!(
        report.message,
        "read-only-boundary:repair-provider-output-payload"
    );
    let json = nsld_drive_apply_report_json(&report);
    assert!(json.contains("\"applied\":false"));
    assert!(json.contains("\"mutates_artifacts\":false"));
    assert!(json.contains("\"mutation_policy\":\"blocked-read-only-boundary\""));
    assert!(json.contains("\"command_id\":\"repair-provider-output-payload\""));
}

#[test]
fn drive_until_clean_crosses_provider_sample_boundary_then_reports_next_blocker() {
    let dir = env::temp_dir().join(format!(
        "nsld-drive-provider-sample-until-clean-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    write_device_provider_sample_manifest(&dir);
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();
    plan.final_stage.output_path = dir.join("demo").display().to_string();

    let report = nsld_drive_apply_until_clean(Path::new("manifest.toml"), &plan).unwrap();
    let check = nsld_check_report(Path::new("manifest.toml"), &plan);
    let manifest = fs::read_to_string(dir.join("nuis.nsdb.device-provider-samples.toml")).unwrap();
    fs::remove_dir_all(dir).unwrap();

    assert!(!report.completed, "{:?}", report.messages);
    assert!(!report.capped);
    assert_eq!(report.stop_reason, "final-output-missing");
    assert_eq!(
        report.last_command_id.as_deref(),
        Some("materialize-provider-samples")
    );
    assert!(report
        .messages
        .iter()
        .any(|message| message == "applied materialize-provider-samples:ready:1"));
    let json = nsld_drive_until_clean_report_json(&report);
    assert!(json.contains("\"safe_next_contract\":\"nsld-drive-safe-next-v1\""));
    assert!(json.contains("\"safe_next_action\":\"explicit-boundary-crossing-command-available\""));
    assert!(json.contains("\"safe_next_command\":\""));
    assert!(json.contains("\"safe_next_gate_required\":true"));
    assert!(json.contains(
        "\"safe_next_gate_action\":\"set-env:NUIS_NSLD_HOST_FINALIZER_POLICY=allow-host-invoke\""
    ));
    assert!(json.contains(
        "\"safe_next_reason\":\"drive stopped at an explicit boundary; run the safe_next_command only if you accept the listed gate\""
    ));
    assert_eq!(
        check.final_executable_output_device_provider_sample_manifest_status,
        "ready"
    );
    assert_eq!(
        check.final_executable_output_device_provider_sample_manifest_pending_record_count,
        0
    );
    assert_eq!(
        check.final_executable_output_device_provider_sample_manifest_first_blocker,
        None
    );
    assert!(manifest.contains("pending_record_count = 0"));
    assert!(manifest.contains("materialization_status = \"provider-sample-materialized\""));
}

#[test]
fn drive_until_clean_stops_at_blocked_provider_sample_repair_boundary() {
    let dir = env::temp_dir().join(format!(
        "nsld-drive-provider-sample-blocked-until-clean-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    write_blocked_device_provider_sample_manifest(&dir);
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();
    plan.final_stage.output_path = dir.join("demo").display().to_string();

    let report = nsld_drive_apply_until_clean(Path::new("manifest.toml"), &plan).unwrap();
    let check = nsld_check_report(Path::new("manifest.toml"), &plan);
    let manifest = fs::read_to_string(dir.join("nuis.nsdb.device-provider-samples.toml")).unwrap();
    fs::remove_dir_all(dir).unwrap();

    assert!(!report.completed, "{:?}", report.messages);
    assert!(!report.capped);
    assert_eq!(
        report.stop_reason,
        "provider-output-payload-repair-required"
    );
    assert_eq!(
        report.stop_command_id.as_deref(),
        Some("repair-provider-output-payload")
    );
    assert!(report
        .messages
        .iter()
        .any(|message| message == "read-only-boundary:repair-provider-output-payload"));
    assert_eq!(
        check.final_executable_output_device_provider_sample_manifest_status,
        "blocked-provider-sample"
    );
    assert_eq!(
        check
            .artifact_chain_final_output_boundary_command_id
            .as_deref(),
        Some("repair-provider-output-payload")
    );
    assert!(manifest.contains("materialization_status = \"provider-sample-blocked\""));
    let json = nsld_drive_until_clean_report_json(&report);
    assert!(json
        .contains("\"mutation_policy\":\"whitelisted-artifact-mutation-loop-diagnostic-repair\""));
    assert!(json.contains("\"safe_next_action\":\"inspect-stop-command\""));
}

#[test]
fn drive_until_clean_names_provider_sample_boundary_stop_reason() {
    let reason =
        "final executable output boundary is blocked by `device-provider-sample:metal:apple-silicon-gpu:pending:1`; materialize provider samples before relinking";

    assert_eq!(
        final_output_boundary_stop_reason(Some(reason)),
        "provider-sample-materialization-required"
    );
}

#[test]
fn drive_until_clean_names_blocked_provider_sample_repair_stop_reason() {
    let reason =
        "final executable output boundary is blocked by `device-provider-sample:metal:apple-silicon-gpu:blocked:1:provider-sample-blocked`; repair provider output payload diagnostics before relinking";

    assert_eq!(
        final_output_boundary_stop_reason(Some(reason)),
        "provider-output-payload-repair-required"
    );

    let report = NsldDriveUntilCleanReport {
        completed: false,
        applied_steps: 0,
        capped: false,
        stop_reason: "provider-output-payload-repair-required".to_owned(),
        stop_command_id: Some("repair-provider-output-payload".to_owned()),
        stop_source: Some("final-output-boundary".to_owned()),
        stop_command_resolved: Some(
            "nsdb materialize-provider-samples /tmp/nuis --json".to_owned(),
        ),
        stop_action_reason: Some(reason.to_owned()),
        stop_gate_action: None,
        stop_gate_env_assignments: Vec::new(),
        stop_crossing_env_assignments: Vec::new(),
        stop_crossing_command_resolved: None,
        last_command_id: None,
        messages: vec!["read-only-boundary:repair-provider-output-payload".to_owned()],
    };
    let json = nsld_drive_until_clean_report_json(&report);

    assert!(json.contains("\"stop_reason\":\"provider-output-payload-repair-required\""));
    assert!(json.contains("\"mutation_policy\":\"blocked-boundary-diagnostic-repair\""));
    assert!(json.contains("\"safe_next_action\":\"inspect-stop-command\""));
    assert!(json.contains("\"safe_next_gate_required\":false"));
}

fn write_device_provider_sample_manifest(output_dir: &Path) {
    fs::write(
        output_dir.join("nuis.nsdb.device-provider-samples.toml"),
        r#"
protocol = "nuis-device-provider-samples-v1"
schema = "nsdb-yir-device-provider-sample-v1"
source = "run-artifact-provider-sample-manifest"
status = "awaiting-provider-materialization"
record_count = 1
ready_record_count = 0
pending_record_count = 1

[[device_provider_samples]]
trace_id = "hetero-trace:shader:metal:apple-silicon-gpu"
provider = "nustar-deferred-device-sample-v1"
provider_family = "metal:apple-silicon-gpu"
handoff_target = "metal:apple-silicon-gpu"
sample_status = "pending-provider-execution"
validation_status = "pending-provider-execution"
input_evidence = "metallib:pixelmagic.metallib"
output_evidence = "not-materialized"
materialization_status = "provider-sample-pending"
materialization_detail = "awaiting-provider-runtime"
next_action = "execute-provider-sample"
"#,
    )
    .unwrap();
}

fn write_blocked_device_provider_sample_manifest(output_dir: &Path) {
    fs::write(
        output_dir.join("nuis.nsdb.device-provider-samples.toml"),
        r#"
protocol = "nuis-device-provider-samples-v1"
schema = "nsdb-yir-device-provider-sample-v1"
source = "run-artifact-provider-sample-manifest"
status = "blocked-provider-sample"
record_count = 1
ready_record_count = 0
pending_record_count = 0
blocked_record_count = 1

[[device_provider_samples]]
trace_id = "hetero-trace:shader:metal:apple-silicon-gpu"
provider = "nustar-deferred-device-sample-v1"
provider_family = "metal:apple-silicon-gpu"
handoff_target = "metal:apple-silicon-gpu"
sample_status = "provider-execution-blocked"
validation_status = "provider-output-payload-invalid"
input_evidence = "metallib:pixelmagic.metallib"
output_evidence = "provider-output-payload-invalid"
materialization_status = "provider-sample-blocked"
materialization_detail = "provider output payload rejected by nsdb"
next_action = "repair-provider-output-payload"
"#,
    )
    .unwrap();
}
