use super::{
    main_test_support::empty_link_plan, nsld_check_report,
    nsld_emit_final_executable_host_invoke_plan_report,
    nsld_emit_final_executable_writer_input_report, nsld_prepare_report,
};
use std::{env, fs, path::Path};

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
