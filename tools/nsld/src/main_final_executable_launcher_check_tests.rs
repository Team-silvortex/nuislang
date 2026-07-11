use super::{
    main_test_support::empty_link_plan, nsld_check_report,
    nsld_emit_final_executable_image_dry_run_report,
    nsld_emit_final_executable_launcher_dry_run_report,
    nsld_emit_final_executable_launcher_manifest_report,
    nsld_emit_final_executable_layout_plan_report, nsld_emit_final_executable_report,
    nsld_emit_final_executable_writer_input_report, nsld_emit_final_stage_plan_report,
    nsld_prepare_report,
};
use std::{env, fs, path::Path};

#[test]
fn check_reports_final_executable_launcher_stages() {
    let dir = env::temp_dir().join(format!(
        "nsld-check-final-executable-launcher-{}",
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
    nsld_emit_final_executable_launcher_manifest_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_executable_launcher_dry_run_report(Path::new("manifest.toml"), &plan).unwrap();

    let report = nsld_check_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::check_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(report.valid);
    assert!(report.final_executable_launcher_manifest_present);
    assert_eq!(report.final_executable_launcher_manifest_valid, Some(true));
    assert_eq!(report.final_executable_launcher_manifest_ready, Some(true));
    assert_eq!(
        report.final_executable_launcher_manifest_blocker_count,
        Some(0)
    );
    assert!(report.final_executable_launcher_manifest_issues.is_empty());
    assert!(report
        .final_executable_launcher_manifest_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("0x")));
    assert!(report.final_executable_launcher_dry_run_present);
    assert_eq!(report.final_executable_launcher_dry_run_valid, Some(true));
    assert_eq!(report.final_executable_launcher_dry_run_ready, Some(true));
    assert_eq!(
        report.final_executable_launcher_dry_run_would_enter_lifecycle_hook,
        Some(true)
    );
    assert_eq!(
        report.final_executable_launcher_dry_run_blocker_count,
        Some(0)
    );
    assert!(report.final_executable_launcher_dry_run_issues.is_empty());
    assert!(report
        .final_executable_launcher_dry_run_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("0x")));
    assert!(report_json.contains("\"final_executable_launcher_manifest_present\":true"));
    assert!(report_json.contains("\"final_executable_launcher_manifest_valid\":true"));
    assert!(report_json.contains("\"final_executable_launcher_manifest_ready\":true"));
    assert!(report_json.contains("\"final_executable_launcher_dry_run_present\":true"));
    assert!(report_json.contains("\"final_executable_launcher_dry_run_valid\":true"));
    assert!(report_json.contains("\"final_executable_launcher_dry_run_ready\":true"));
    assert!(report_json
        .contains("\"final_executable_launcher_dry_run_would_enter_lifecycle_hook\":true"));
}
