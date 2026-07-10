use super::{
    nsld_artifact_chain_issues, nsld_artifact_chain_report, nsld_artifact_stage_file_name,
    nsld_artifact_stage_kind_path, nsld_object_output_file_name, NsldArtifactStage,
    NsldArtifactStageKind,
};
use crate::{
    json, main_test_support::empty_link_plan, nsld_emit_final_executable_host_invoke_plan_report,
    nsld_emit_final_executable_image_dry_run_report, nsld_emit_final_executable_layout_plan_report,
    nsld_emit_final_executable_report, nsld_emit_final_executable_writer_input_report,
    nsld_prepare_report,
};
use std::{env, fs, path::Path};

#[test]
fn artifact_chain_accepts_contiguous_prepared_prefix() {
    let issues = nsld_artifact_chain_issues(&[
        test_artifact_stage("inputs", true),
        test_artifact_stage("units", true),
        test_artifact_stage("bundle", true),
        test_artifact_stage("assemble", false),
        test_artifact_stage("section", false),
        test_artifact_stage("object", false),
    ]);
    assert!(issues.is_empty());
}

#[test]
fn artifact_chain_rejects_later_artifact_without_prerequisite() {
    let issues = nsld_artifact_chain_issues(&[
        test_artifact_stage("inputs", true),
        test_artifact_stage("units", false),
        test_artifact_stage("bundle", true),
        test_artifact_stage("assemble", true),
        test_artifact_stage("section", true),
        test_artifact_stage("object", true),
    ]);
    assert_eq!(
        issues,
        vec![
            "artifact `bundle` is present but prerequisite `units` is missing".to_owned(),
            "artifact `assemble` is present but prerequisite `units` is missing".to_owned(),
            "artifact `section` is present but prerequisite `units` is missing".to_owned(),
            "artifact `object` is present but prerequisite `units` is missing".to_owned(),
        ]
    );
}

#[test]
fn artifact_chain_allows_missing_optional_object_output_before_later_artifacts() {
    let issues = nsld_artifact_chain_issues(&[
        test_artifact_stage("object-emit", true),
        test_optional_artifact_stage("nuis.nsld.mach-o", false),
        test_artifact_stage("object-writer-dry-run", true),
        test_artifact_stage("container-plan", true),
    ]);
    assert!(issues.is_empty());
}

#[test]
fn artifact_chain_treats_closure_snapshot_as_optional_chain_tail() {
    let issues = nsld_artifact_chain_issues(&[
        test_artifact_stage("container", true),
        test_artifact_stage("nuis.nsld.container.payload", true),
        test_optional_artifact_stage("nuis.nsld.closure.toml", false),
    ]);
    assert!(issues.is_empty());
}

#[test]
fn artifact_stage_kind_paths_are_canonical() {
    assert_eq!(
        nsld_artifact_stage_file_name(NsldArtifactStageKind::ObjectWriterInput),
        "nuis.nsld.object-writer-input.toml"
    );
    assert_eq!(
        nsld_artifact_stage_kind_path("out", NsldArtifactStageKind::ContainerPayload)
            .display()
            .to_string(),
        "out/nuis.nsld.container.payload"
    );
    assert_eq!(
        nsld_artifact_stage_file_name(NsldArtifactStageKind::FinalStagePlan),
        "nuis.nsld.final-stage-plan.toml"
    );
    assert_eq!(
        nsld_artifact_stage_file_name(NsldArtifactStageKind::FinalExecutableWriterInput),
        "nuis.nsld.final-executable-writer-input.toml"
    );
    assert_eq!(
        nsld_artifact_stage_file_name(NsldArtifactStageKind::FinalExecutableHostInvokePlan),
        "nuis.nsld.final-executable-host-invoke-plan.toml"
    );
    assert_eq!(
        nsld_artifact_stage_file_name(NsldArtifactStageKind::FinalExecutableLayoutPlan),
        "nuis.nsld.final-executable-layout.toml"
    );
    assert_eq!(
        nsld_artifact_stage_file_name(NsldArtifactStageKind::FinalExecutableImageDryRun),
        "nuis.nsld.final-executable-image-dry-run.toml"
    );
    assert_eq!(
        nsld_artifact_stage_file_name(NsldArtifactStageKind::FinalExecutableImageDryRunBytes),
        "nuis.nsld.final-executable-image-dry-run.bin"
    );
    assert_eq!(
        nsld_artifact_stage_file_name(NsldArtifactStageKind::FinalExecutableBlocked),
        "nuis.nsld.final-executable.blocked.toml"
    );
    assert_eq!(nsld_object_output_file_name("pe/coff"), "nuis.nsld.pe-coff");
}

#[test]
fn artifact_chain_report_lists_registered_stages_and_optional_tail() {
    let dir = env::temp_dir().join(format!("nsld-artifact-chain-report-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_executable_report(Path::new("manifest.toml"), &plan).unwrap();
    let report = nsld_artifact_chain_report(Path::new("manifest.toml"), &plan);
    let report_json = json::nsld_artifact_chain_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(report.valid, "{:?}", report.issues);
    assert_eq!(report.stage_count, 25);
    assert!(report.present_count >= report.required_count);
    assert_eq!(report.missing_required_count, 0);
    assert!(report.optional_present_count >= 3);
    assert_eq!(report.first_missing_required_stage, None);
    assert_eq!(report.next_required_stage, None);
    assert_eq!(report.suggested_command_id, None);
    assert_eq!(report.suggested_command, None);
    assert_eq!(report.suggested_command_resolved, None);
    assert_eq!(report.suggested_command_reason, None);
    assert_eq!(
        report.next_optional_stage.as_deref(),
        Some("final-executable-writer-input")
    );
    assert_eq!(
        report.next_optional_command_id.as_deref(),
        Some("emit-final-executable-writer-input")
    );
    assert_eq!(
        report.next_optional_command.as_deref(),
        Some("nsld emit-final-executable-writer-input <input>")
    );
    assert_eq!(
        report.next_optional_command_resolved.as_deref(),
        Some("nsld emit-final-executable-writer-input manifest.toml")
    );
    assert_eq!(
        report.next_optional_command_reason.as_deref(),
        Some("first missing optional artifact stage `final-executable-writer-input`")
    );
    assert!(report
        .stages
        .iter()
        .any(|stage| stage.stage_id == "final-stage-plan" && stage.present && !stage.required));
    assert!(report.stages.iter().any(|stage| {
        stage.stage_id == "final-executable-writer-input" && !stage.present && !stage.required
    }));
    assert!(report.stages.iter().any(|stage| {
        stage.stage_id == "final-executable-host-invoke-plan" && !stage.present && !stage.required
    }));
    assert!(report.stages.iter().any(|stage| {
        stage.stage_id == "final-executable-layout" && !stage.present && !stage.required
    }));
    assert!(report.stages.iter().any(|stage| {
        stage.stage_id == "final-executable-image-dry-run" && !stage.present && !stage.required
    }));
    assert!(report.stages.iter().any(|stage| {
        stage.stage_id == "final-executable-image-dry-run-bytes"
            && !stage.present
            && !stage.required
    }));
    assert!(report.stages.iter().any(|stage| {
        stage.stage_id == "final-executable-blocked" && stage.present && !stage.required
    }));
    assert!(report_json.contains("\"kind\":\"nsld_artifact_chain\""));
    assert!(report_json.contains("\"stage_id\":\"final-executable-writer-input\""));
    assert!(report_json.contains("\"stage_id\":\"final-executable-host-invoke-plan\""));
    assert!(report_json.contains("\"stage_id\":\"final-executable-layout\""));
    assert!(report_json.contains("\"stage_id\":\"final-executable-image-dry-run\""));
    assert!(report_json.contains("\"stage_id\":\"final-executable-image-dry-run-bytes\""));
    assert!(report_json.contains("\"stage_id\":\"final-executable-blocked\""));
    assert!(report_json.contains("\"missing_required_count\":0"));
    assert!(report_json.contains("\"first_missing_required_stage\":null"));
    assert!(report_json.contains("\"next_required_stage\":null"));
    assert!(report_json.contains("\"suggested_command_id\":null"));
    assert!(report_json.contains("\"suggested_command\":null"));
    assert!(report_json.contains("\"suggested_command_resolved\":null"));
    assert!(report_json.contains("\"suggested_command_reason\":null"));
    assert!(report_json.contains("\"next_optional_stage\":\"final-executable-writer-input\""));
    assert!(
        report_json.contains("\"next_optional_command_id\":\"emit-final-executable-writer-input\"")
    );
    assert!(report_json
        .contains("\"next_optional_command\":\"nsld emit-final-executable-writer-input <input>\""));
    assert!(report_json.contains(
        "\"next_optional_command_resolved\":\"nsld emit-final-executable-writer-input manifest.toml\""
    ));
}

#[test]
fn artifact_chain_next_optional_stage_advances_through_final_executable_tail() {
    let dir = env::temp_dir().join(format!(
        "nsld-artifact-chain-final-executable-tail-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let after_prepare = nsld_artifact_chain_report(Path::new("manifest.toml"), &plan);
    nsld_emit_final_executable_writer_input_report(Path::new("manifest.toml"), &plan).unwrap();
    let after_writer_input = nsld_artifact_chain_report(Path::new("manifest.toml"), &plan);
    nsld_emit_final_executable_host_invoke_plan_report(Path::new("manifest.toml"), &plan).unwrap();
    let after_invoke_plan = nsld_artifact_chain_report(Path::new("manifest.toml"), &plan);
    nsld_emit_final_executable_layout_plan_report(Path::new("manifest.toml"), &plan).unwrap();
    let after_layout_plan = nsld_artifact_chain_report(Path::new("manifest.toml"), &plan);
    nsld_emit_final_executable_image_dry_run_report(Path::new("manifest.toml"), &plan).unwrap();
    let after_image_dry_run = nsld_artifact_chain_report(Path::new("manifest.toml"), &plan);
    nsld_emit_final_executable_report(Path::new("manifest.toml"), &plan).unwrap();
    let after_blocked = nsld_artifact_chain_report(Path::new("manifest.toml"), &plan);
    fs::remove_dir_all(dir).unwrap();

    assert_eq!(
        after_prepare.next_optional_stage.as_deref(),
        Some("final-executable-writer-input")
    );
    assert_eq!(
        after_prepare.next_optional_command_id.as_deref(),
        Some("emit-final-executable-writer-input")
    );
    assert_eq!(
        after_writer_input.next_optional_stage.as_deref(),
        Some("final-executable-host-invoke-plan")
    );
    assert_eq!(
        after_writer_input.next_optional_command_id.as_deref(),
        Some("emit-final-executable-host-invoke-plan")
    );
    assert_eq!(
        after_invoke_plan.next_optional_stage.as_deref(),
        Some("final-executable-layout")
    );
    assert_eq!(
        after_invoke_plan.next_optional_command_id.as_deref(),
        Some("emit-final-executable-layout")
    );
    assert_eq!(
        after_layout_plan.next_optional_stage.as_deref(),
        Some("final-executable-image-dry-run")
    );
    assert_eq!(
        after_layout_plan.next_optional_command_id.as_deref(),
        Some("emit-final-executable-image-dry-run")
    );
    assert_eq!(
        after_image_dry_run.next_optional_stage.as_deref(),
        Some("final-executable-blocked")
    );
    assert_eq!(
        after_image_dry_run.next_optional_command_id.as_deref(),
        Some("emit-final-executable")
    );
    assert_eq!(after_blocked.next_optional_stage, None);
    assert_eq!(after_blocked.next_optional_command_id, None);
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
    let report_json = json::nsld_artifact_chain_report_json(&report);
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
    assert!(report_json.contains("\"first_missing_required_stage\":\"link-inputs\""));
    assert!(report_json.contains("\"next_required_stage\":\"link-inputs\""));
    assert!(report_json.contains("\"suggested_command_id\":\"emit-inputs\""));
    assert!(report_json.contains("\"suggested_command\":\"nsld emit-inputs <input>\""));
    assert!(
        report_json.contains("\"suggested_command_resolved\":\"nsld emit-inputs manifest.toml\"")
    );
    assert!(report_json.contains(
        "\"suggested_command_reason\":\"first missing required artifact stage `link-inputs`\""
    ));
}

fn test_artifact_stage(file_name: &'static str, present: bool) -> NsldArtifactStage {
    NsldArtifactStage {
        kind: NsldArtifactStageKind::LinkInputs,
        file_name: file_name.to_owned(),
        present,
        required: true,
    }
}

fn test_optional_artifact_stage(file_name: &'static str, present: bool) -> NsldArtifactStage {
    NsldArtifactStage {
        kind: NsldArtifactStageKind::ObjectOutput,
        file_name: file_name.to_owned(),
        present,
        required: false,
    }
}
