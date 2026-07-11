use super::nsld_artifact_chain_report;
use crate::{
    json, main_test_support::empty_link_plan, nsld_check_report,
    nsld_emit_final_executable_pipeline_report, nsld_prepare_report,
};
use std::{env, fs, path::Path};

#[test]
fn artifact_chain_reports_advisory_for_broken_final_executable_tail() {
    let dir = env::temp_dir().join(format!(
        "nsld-artifact-chain-broken-final-tail-{}",
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
    let report = nsld_artifact_chain_report(Path::new("manifest.toml"), &plan);
    let check = nsld_check_report(Path::new("manifest.toml"), &plan);
    let report_json = json::nsld_artifact_chain_report_json(&report);
    let check_json = json::check_report_json(&check);
    fs::remove_dir_all(dir).unwrap();

    assert!(report.valid, "{:?}", report.issues);
    assert!(report.issues.is_empty());
    assert!(report.advisories.iter().any(|advisory| advisory.contains(
        "optional final executable tail artifact `final-executable-pipeline` is present"
    )));
    assert!(report
        .advisories
        .iter()
        .any(|advisory| advisory.contains("`final-executable-launcher-dry-run` is missing")));
    assert!(report
        .advisories
        .iter()
        .any(|advisory| advisory.contains("emit-final-executable-pipeline")));
    assert_eq!(
        report.advisory_command_id.as_deref(),
        Some("emit-final-executable-pipeline")
    );
    assert_eq!(
        report.advisory_command.as_deref(),
        Some("nsld emit-final-executable-pipeline <input>")
    );
    assert_eq!(
        report.advisory_command_resolved.as_deref(),
        Some("nsld emit-final-executable-pipeline manifest.toml")
    );
    assert!(report
        .advisory_command_reason
        .as_deref()
        .is_some_and(|reason| reason.contains("first artifact-chain advisory")));
    assert_eq!(
        report.next_action_command_id.as_deref(),
        Some("emit-final-executable-pipeline")
    );
    assert_eq!(
        report.next_action_command_resolved.as_deref(),
        Some("nsld emit-final-executable-pipeline manifest.toml")
    );
    assert!(report
        .next_action_command_reason
        .as_deref()
        .is_some_and(|reason| reason.contains("first artifact-chain advisory")));
    assert_eq!(report.next_action_source.as_deref(), Some("advisory"));
    assert!(report.next_action_available);
    assert!(check
        .artifact_chain_advisories
        .iter()
        .any(|advisory| advisory.contains("final-executable-launcher-dry-run")));
    assert_eq!(check.advisory_count, 1);
    assert_eq!(
        check.artifact_chain_advisory_command_id.as_deref(),
        Some("emit-final-executable-pipeline")
    );
    assert_eq!(
        check.artifact_chain_advisory_command_resolved.as_deref(),
        Some("nsld emit-final-executable-pipeline manifest.toml")
    );
    assert!(check
        .artifact_chain_advisory_command_reason
        .as_deref()
        .is_some_and(|reason| reason.contains("first artifact-chain advisory")));
    assert_eq!(
        check.artifact_chain_next_action_command_id.as_deref(),
        Some("emit-final-executable-pipeline")
    );
    assert_eq!(
        check.artifact_chain_next_action_command_resolved.as_deref(),
        Some("nsld emit-final-executable-pipeline manifest.toml")
    );
    assert!(check
        .artifact_chain_next_action_command_reason
        .as_deref()
        .is_some_and(|reason| reason.contains("first artifact-chain advisory")));
    assert_eq!(
        check.next_action_command_id.as_deref(),
        Some("emit-final-executable-pipeline")
    );
    assert_eq!(
        check.next_action_command_resolved.as_deref(),
        Some("nsld emit-final-executable-pipeline manifest.toml")
    );
    assert!(check
        .next_action_command_reason
        .as_deref()
        .is_some_and(|reason| reason.contains("first artifact-chain advisory")));
    assert_eq!(check.next_action_source.as_deref(), Some("advisory"));
    assert!(check.next_action_available);
    assert_eq!(
        check.artifact_chain_next_action_source.as_deref(),
        Some("advisory")
    );
    assert!(check.artifact_chain_next_action_available);
    assert!(check.artifact_chain_issues.is_empty());
    assert!(check_json.contains("\"advisory_count\":1"));
    assert!(check_json.contains("\"next_action_source\":\"advisory\""));
    assert!(check_json.contains("\"next_action_available\":true"));
    assert!(check_json.contains("\"next_action_command_id\":\"emit-final-executable-pipeline\""));
    assert!(check_json
        .contains("\"artifact_chain_next_action_command_id\":\"emit-final-executable-pipeline\""));
    assert!(report_json.contains("\"advisories\":["));
    assert!(report_json.contains("\"advisory_command_id\":\"emit-final-executable-pipeline\""));
    assert!(report_json.contains("\"next_action_source\":\"advisory\""));
    assert!(report_json.contains("\"next_action_available\":true"));
    assert!(report_json.contains("\"next_action_command_id\":\"emit-final-executable-pipeline\""));
    assert!(report_json.contains(
        "\"advisory_command_resolved\":\"nsld emit-final-executable-pipeline manifest.toml\""
    ));
    assert!(report_json.contains("final-executable-launcher-dry-run"));
}

#[test]
fn artifact_chain_report_points_to_first_missing_required_stage() {
    let dir = env::temp_dir().join(format!(
        "nsld-artifact-chain-report-missing-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();

    let report = nsld_artifact_chain_report(Path::new("manifest.toml"), &plan);
    let check = nsld_check_report(Path::new("manifest.toml"), &plan);
    let report_json = json::nsld_artifact_chain_report_json(&report);
    let check_json = json::check_report_json(&check);
    fs::remove_dir_all(dir).unwrap();

    assert!(report.valid);
    assert!(report.missing_required_count > 0);
    assert_eq!(
        report.first_missing_required_stage.as_deref(),
        Some("link-inputs")
    );
    assert_eq!(report.next_required_stage.as_deref(), Some("link-inputs"));
    assert_eq!(report.suggested_command_id.as_deref(), Some("emit-inputs"));
    assert_eq!(
        report.suggested_command.as_deref(),
        Some("nsld emit-inputs <input>")
    );
    assert_eq!(
        report.suggested_command_resolved.as_deref(),
        Some("nsld emit-inputs manifest.toml")
    );
    assert_eq!(
        report.suggested_command_reason.as_deref(),
        Some("first missing required artifact stage `link-inputs`")
    );
    assert_eq!(
        report.next_action_command_id.as_deref(),
        Some("emit-inputs")
    );
    assert_eq!(
        report.next_action_command_resolved.as_deref(),
        Some("nsld emit-inputs manifest.toml")
    );
    assert_eq!(
        report.next_action_command.as_deref(),
        Some("nsld emit-inputs <input>")
    );
    assert_eq!(
        report.next_action_command_reason.as_deref(),
        Some("first missing required artifact stage `link-inputs`")
    );
    assert_eq!(report.next_action_source.as_deref(), Some("required"));
    assert!(report.next_action_available);
    assert_eq!(check.next_action_command_id.as_deref(), Some("emit-inputs"));
    assert_eq!(
        check.next_action_command.as_deref(),
        Some("nsld emit-inputs <input>")
    );
    assert_eq!(
        check.next_action_command_resolved.as_deref(),
        Some("nsld emit-inputs manifest.toml")
    );
    assert_eq!(
        check.next_action_command_reason.as_deref(),
        Some("first missing required artifact stage `link-inputs`")
    );
    assert_eq!(
        check.artifact_chain_next_action_command_id.as_deref(),
        Some("emit-inputs")
    );
    assert_eq!(check.next_action_source.as_deref(), Some("required"));
    assert_eq!(
        check.artifact_chain_next_action_source.as_deref(),
        Some("required")
    );
    assert!(check.next_action_available);
    assert!(check.artifact_chain_next_action_available);
    assert_eq!(check.advisory_count, 0);
    assert!(report_json.contains("\"first_missing_required_stage\":\"link-inputs\""));
    assert!(report_json.contains("\"next_required_stage\":\"link-inputs\""));
    assert!(report_json.contains("\"suggested_command_id\":\"emit-inputs\""));
    assert!(report_json.contains("\"next_action_command_id\":\"emit-inputs\""));
    assert!(report_json.contains("\"next_action_source\":\"required\""));
    assert!(report_json.contains("\"next_action_available\":true"));
    assert!(report_json.contains("\"suggested_command\":\"nsld emit-inputs <input>\""));
    assert!(check_json.contains("\"advisory_count\":0"));
    assert!(check_json.contains("\"next_action_source\":\"required\""));
    assert!(check_json.contains("\"next_action_available\":true"));
    assert!(check_json.contains("\"next_action_command_id\":\"emit-inputs\""));
    assert!(check_json.contains("\"next_action_command\":\"nsld emit-inputs <input>\""));
    assert!(
        report_json.contains("\"suggested_command_resolved\":\"nsld emit-inputs manifest.toml\"")
    );
    assert!(report_json.contains(
        "\"suggested_command_reason\":\"first missing required artifact stage `link-inputs`\""
    ));
}
