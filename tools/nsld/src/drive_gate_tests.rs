use super::{
    nsld_drive_apply_next_action, nsld_drive_apply_report_json, nsld_drive_dry_run_json,
    nsld_drive_until_clean_report_json, NsldDriveUntilCleanReport,
};
use crate::{commands::NsldCheckNextAction, main_test_support::empty_link_plan};
use std::path::Path;

#[test]
fn drive_dry_run_json_reports_next_action_without_execution() {
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
    let json = nsld_drive_dry_run_json(&next_action);

    assert!(json.contains("\"kind\":\"nsld_drive_dry_run\""));
    assert!(json.contains("\"would_execute\":true"));
    assert!(json.contains("\"mutates_artifacts\":false"));
    assert!(json.contains("\"mutation_policy\":\"read-only-artifact-observe\""));
    assert!(json.contains("\"command_resolved\":\"nsld emit-inputs manifest.toml\""));
    assert!(json.contains("\"gate_action\":null"));
    assert!(json.contains("\"gate_env_assignments\":[]"));
    assert!(json.contains("\"crossing_env_assignments\":[]"));
    assert!(json.contains("\"crossing_command_resolved\":null"));
}

#[test]
fn drive_dry_run_json_reports_gate_action_for_final_output_boundary() {
    let next_action = final_output_boundary_next_action();
    let json = nsld_drive_dry_run_json(&next_action);

    assert!(json.contains("\"source\":\"final-output-boundary\""));
    assert!(json.contains("\"mutation_policy\":\"read-only-boundary-observe\""));
    assert!(json
        .contains("\"gate_action\":\"set-env:NUIS_NSLD_HOST_FINALIZER_POLICY=allow-host-invoke\""));
    assert!(json.contains(
        "\"gate_env_assignments\":[\"NUIS_NSLD_HOST_FINALIZER_POLICY=allow-host-invoke\"]"
    ));
    assert!(json.contains(
        "\"crossing_env_assignments\":[\"NUIS_NSLD_HOST_FINALIZER_POLICY=allow-host-invoke\",\"NUIS_NSLD_ALLOW_HOST_FINALIZER=1\"]"
    ));
    assert!(json.contains(
        "\"crossing_command_resolved\":\"env NUIS_NSLD_HOST_FINALIZER_POLICY=allow-host-invoke NUIS_NSLD_ALLOW_HOST_FINALIZER=1 nsld final-executable-output manifest.toml\""
    ));
}

#[test]
fn drive_until_clean_json_reports_loop_shape() {
    let report = NsldDriveUntilCleanReport {
        completed: true,
        applied_steps: 2,
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
        last_command_id: Some("emit-inputs".to_owned()),
        messages: vec![
            "applied emit-inputs".to_owned(),
            "no-next-action".to_owned(),
        ],
    };
    let json = nsld_drive_until_clean_report_json(&report);

    assert!(json.contains("\"kind\":\"nsld_drive_until_clean\""));
    assert!(json.contains("\"completed\":true"));
    assert!(json.contains("\"applied_steps\":2"));
    assert!(json.contains("\"mutates_artifacts\":true"));
    assert!(json.contains("\"mutation_policy\":\"whitelisted-artifact-mutation-loop-clean\""));
    assert!(json.contains("\"stop_reason\":\"clean\""));
    assert!(json.contains("\"stop_command_id\":null"));
    assert!(json.contains("\"stop_source\":null"));
    assert!(json.contains("\"stop_command_resolved\":null"));
    assert!(json.contains("\"stop_action_reason\":null"));
    assert!(json.contains("\"stop_gate_action\":null"));
    assert!(json.contains("\"stop_gate_env_assignments\":[]"));
    assert!(json.contains("\"stop_crossing_env_assignments\":[]"));
    assert!(json.contains("\"stop_crossing_command_resolved\":null"));
    assert!(json.contains("\"safe_next_action\":\"clean\""));
    assert!(json.contains("\"safe_next_command\":null"));
    assert!(json.contains("\"safe_next_reason\":\"drive reached a clean artifact chain\""));
    assert!(json.contains("\"last_command_id\":\"emit-inputs\""));
    assert!(json.contains("\"messages\":[\"applied emit-inputs\",\"no-next-action\"]"));
}

#[test]
fn drive_apply_report_json_keeps_empty_gate_fields_for_plain_actions() {
    let mut plan = empty_link_plan();
    plan.output_dir = std::env::temp_dir()
        .join(format!("nsld-drive-gate-plain-{}", std::process::id()))
        .display()
        .to_string();
    std::fs::create_dir_all(&plan.output_dir).unwrap();
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
    let json = nsld_drive_apply_report_json(&report);
    let _ = std::fs::remove_dir_all(&plan.output_dir);

    assert!(report.applied);
    assert!(json.contains("\"mutates_artifacts\":true"));
    assert!(json.contains("\"mutation_policy\":\"whitelisted-artifact-mutation\""));
    assert!(json.contains("\"gate_action\":null"));
    assert!(json.contains("\"gate_env_assignments\":[]"));
    assert!(json.contains("\"crossing_env_assignments\":[]"));
    assert!(json.contains("\"crossing_command_resolved\":null"));
    assert!(json.contains("\"safe_next_action\":\"rerun-drive-to-refresh-next-action\""));
    assert!(json.contains("\"safe_next_command\":null"));
    assert!(json.contains(
        "\"safe_next_reason\":\"drive applied one mutation; rerun drive to observe the next deterministic action\""
    ));
}

#[test]
fn drive_apply_report_json_exposes_gate_fields_when_boundary_is_read_only() {
    let plan = empty_link_plan();
    let next_action = final_output_boundary_next_action();

    let report =
        nsld_drive_apply_next_action(Path::new("manifest.toml"), &plan, &next_action).unwrap();
    let json = nsld_drive_apply_report_json(&report);

    assert!(!report.applied);
    assert_eq!(
        report.command_id.as_deref(),
        Some("final-executable-output")
    );
    assert_eq!(report.message, "read-only-boundary:final-executable-output");
    assert!(json.contains("\"applied\":false"));
    assert!(json.contains("\"mutates_artifacts\":false"));
    assert!(json.contains("\"mutation_policy\":\"blocked-read-only-boundary\""));
    assert!(json.contains("\"command_id\":\"final-executable-output\""));
    assert!(json
        .contains("\"gate_action\":\"set-env:NUIS_NSLD_HOST_FINALIZER_POLICY=allow-host-invoke\""));
    assert!(json.contains(
        "\"gate_env_assignments\":[\"NUIS_NSLD_HOST_FINALIZER_POLICY=allow-host-invoke\"]"
    ));
    assert!(json.contains(
        "\"crossing_env_assignments\":[\"NUIS_NSLD_HOST_FINALIZER_POLICY=allow-host-invoke\",\"NUIS_NSLD_ALLOW_HOST_FINALIZER=1\"]"
    ));
    assert!(json.contains(
        "\"crossing_command_resolved\":\"env NUIS_NSLD_HOST_FINALIZER_POLICY=allow-host-invoke NUIS_NSLD_ALLOW_HOST_FINALIZER=1 nsld final-executable-output manifest.toml\""
    ));
    assert!(json.contains("\"safe_next_action\":\"explicit-boundary-crossing-command-available\""));
    assert!(json.contains(
        "\"safe_next_command\":\"env NUIS_NSLD_HOST_FINALIZER_POLICY=allow-host-invoke NUIS_NSLD_ALLOW_HOST_FINALIZER=1 nsld final-executable-output manifest.toml\""
    ));
    assert!(json.contains(
        "\"safe_next_reason\":\"drive stopped at an explicit boundary; run the safe_next_command only if you accept the listed gate\""
    ));
}

fn final_output_boundary_next_action() -> NsldCheckNextAction {
    NsldCheckNextAction {
        available: true,
        source: Some("final-output-boundary".to_owned()),
        command_id: Some("final-executable-output".to_owned()),
        command: Some("nsld final-executable-output <input>".to_owned()),
        command_resolved: Some("nsld final-executable-output manifest.toml".to_owned()),
        reason: Some(
            "final executable output boundary is blocked by `final-executable-output:not-nsld-owned`"
                .to_owned(),
        ),
        gate_action: Some("set-env:NUIS_NSLD_HOST_FINALIZER_POLICY=allow-host-invoke".to_owned()),
        gate_env_assignments: vec!["NUIS_NSLD_HOST_FINALIZER_POLICY=allow-host-invoke".to_owned()],
        crossing_env_assignments: vec![
            "NUIS_NSLD_HOST_FINALIZER_POLICY=allow-host-invoke".to_owned(),
            "NUIS_NSLD_ALLOW_HOST_FINALIZER=1".to_owned(),
        ],
        crossing_command_resolved: Some(
            "env NUIS_NSLD_HOST_FINALIZER_POLICY=allow-host-invoke NUIS_NSLD_ALLOW_HOST_FINALIZER=1 nsld final-executable-output manifest.toml"
                .to_owned(),
        ),
    }
}
