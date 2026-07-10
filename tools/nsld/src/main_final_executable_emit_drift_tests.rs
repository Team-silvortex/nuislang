use super::{
    main_test_support::empty_link_plan, nsld_emit_final_executable_host_invoke_plan_report,
    nsld_emit_final_executable_report, nsld_emit_final_executable_writer_input_report,
    nsld_prepare_report, nsld_verify_final_executable_emit_report,
};
use std::{env, fs, path::Path};

#[test]
fn verify_final_executable_emit_reports_host_dry_run_drift() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-host-dry-run-drift-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();
    plan.final_stage.driver = "definitely-missing-nsld-host-driver-for-drift-test".to_owned();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_executable_writer_input_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit = nsld_emit_final_executable_report(Path::new("manifest.toml"), &plan).unwrap();
    let blocked_path = Path::new(&emit.blocked_report_path);
    let damaged = fs::read_to_string(blocked_path)
        .unwrap()
        .replace(
            "host_dry_run_environment_ready = false",
            "host_dry_run_environment_ready = true",
        )
        .replace(
            "host_dry_run_can_invoke = false",
            "host_dry_run_can_invoke = true",
        )
        .replace(
            "host_dry_run_invocation_policy = \"dry-run-only\"",
            "host_dry_run_invocation_policy = \"allow-host-invoke\"",
        )
        .replace(
            "host_dry_run_invocation_policy_reason = \"alpha-host-finalizer-execution-disabled\"",
            "host_dry_run_invocation_policy_reason = \"tampered-policy\"",
        )
        .replace(
            &format!(
                "host_dry_run_command_arg_count = {}",
                emit.host_dry_run_command_arg_count
            ),
            "host_dry_run_command_arg_count = 0",
        )
        .replace(
            "definitely-missing-nsld-host-driver-for-drift-test",
            "tampered-host-driver-arg",
        )
        .replace(
            "host-finalizer-driver-unavailable:tampered-host-driver-arg",
            "host-finalizer-driver-unavailable:tampered-driver",
        )
        .replace(
            "host_dry_run_blocker_count = 3",
            "host_dry_run_blocker_count = 0",
        );
    fs::write(blocked_path, damaged).unwrap();
    let verify = nsld_verify_final_executable_emit_report(Path::new("manifest.toml"), &plan);
    let verify_json = super::json::nsld_final_executable_emit_verify_report_json(&verify);
    fs::remove_dir_all(dir).unwrap();

    assert!(!verify.valid);
    assert_eq!(verify.expected_host_dry_run_environment_ready, Some(false));
    assert_eq!(verify.actual_host_dry_run_environment_ready, Some(true));
    assert_eq!(verify.expected_host_dry_run_can_invoke, Some(false));
    assert_eq!(verify.actual_host_dry_run_can_invoke, Some(true));
    assert_eq!(verify.expected_host_dry_run_driver_resolved_path, None);
    assert_eq!(verify.actual_host_dry_run_driver_resolved_path, None);
    assert_eq!(
        verify.expected_host_dry_run_invocation_policy.as_deref(),
        Some("dry-run-only")
    );
    assert_eq!(
        verify.actual_host_dry_run_invocation_policy.as_deref(),
        Some("allow-host-invoke")
    );
    assert_eq!(
        verify
            .expected_host_dry_run_invocation_policy_reason
            .as_deref(),
        Some("alpha-host-finalizer-execution-disabled")
    );
    assert_eq!(
        verify
            .actual_host_dry_run_invocation_policy_reason
            .as_deref(),
        Some("tampered-policy")
    );
    assert!(verify.expected_host_dry_run_blockers.iter().any(|blocker| {
        blocker == "host-finalizer-driver-unavailable:definitely-missing-nsld-host-driver-for-drift-test"
    }));
    assert!(verify
        .actual_host_dry_run_blockers
        .iter()
        .any(|blocker| blocker == "host-finalizer-driver-unavailable:tampered-driver"));
    assert_eq!(verify.expected_host_dry_run_blocker_count, 3);
    assert_eq!(verify.actual_host_dry_run_blocker_count, Some(0));
    assert_eq!(
        verify.expected_host_dry_run_command_arg_count,
        emit.host_dry_run_command_arg_count
    );
    assert_eq!(verify.actual_host_dry_run_command_arg_count, Some(0));
    assert!(verify
        .expected_host_dry_run_command_args
        .iter()
        .any(|arg| arg == "definitely-missing-nsld-host-driver-for-drift-test"));
    assert!(verify
        .actual_host_dry_run_command_args
        .iter()
        .any(|arg| arg == "tampered-host-driver-arg"));
    assert!(verify.issues.iter().any(
        |issue| issue == "host_dry_run_environment_ready mismatch: expected false, found true"
    ));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue == "host_dry_run_can_invoke mismatch: expected false, found true"));
    assert!(verify.issues.iter().any(|issue| {
        issue
            == "host_dry_run_invocation_policy mismatch: expected dry-run-only, found allow-host-invoke"
    }));
    assert!(verify.issues.iter().any(|issue| {
        issue
            == "host_dry_run_invocation_policy_reason mismatch: expected alpha-host-finalizer-execution-disabled, found tampered-policy"
    }));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("host_dry_run_command_arg_count mismatch: expected ")));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("host_dry_run_command_args mismatch")));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("host_dry_run_blockers mismatch")));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue == "host_dry_run_blocker_count mismatch: expected 3, found 0"));
    assert!(verify_json.contains("\"actual_host_dry_run_environment_ready\":true"));
    assert!(verify_json.contains("\"actual_host_dry_run_can_invoke\":true"));
    assert!(verify_json.contains("\"actual_host_dry_run_invocation_policy\":\"allow-host-invoke\""));
    assert!(verify_json
        .contains("\"actual_host_dry_run_invocation_policy_reason\":\"tampered-policy\""));
    assert!(verify_json.contains("\"actual_host_dry_run_command_arg_count\":0"));
    assert!(verify_json.contains("\"tampered-host-driver-arg\""));
    assert!(verify_json.contains("\"actual_host_dry_run_blocker_count\":0"));
    assert!(verify_json.contains("host-finalizer-driver-unavailable:tampered-driver"));
}

#[test]
fn verify_final_executable_emit_reports_writer_input_drift() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-writer-input-drift-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let writer_input =
        nsld_emit_final_executable_writer_input_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit = nsld_emit_final_executable_report(Path::new("manifest.toml"), &plan).unwrap();
    let blocked_path = Path::new(&emit.blocked_report_path);
    let damaged = fs::read_to_string(blocked_path)
        .unwrap()
        .replace("writer_input_valid = true", "writer_input_valid = false")
        .replace(
            &format!("writer_input_hash = \"{}\"", writer_input.writer_input_hash),
            "writer_input_hash = \"0x8888888888888888\"",
        )
        .replace(
            "writer_input_issues = []",
            "writer_input_issues = [\"tampered-writer-input\"]",
        );
    fs::write(blocked_path, damaged).unwrap();
    let verify = nsld_verify_final_executable_emit_report(Path::new("manifest.toml"), &plan);
    let verify_json = super::json::nsld_final_executable_emit_verify_report_json(&verify);
    fs::remove_dir_all(dir).unwrap();

    assert!(!verify.valid);
    assert_eq!(verify.expected_writer_input_valid, Some(true));
    assert_eq!(verify.actual_writer_input_valid, Some(false));
    assert_eq!(
        verify.expected_writer_input_hash.as_deref(),
        Some(writer_input.writer_input_hash.as_str())
    );
    assert_eq!(
        verify.actual_writer_input_hash.as_deref(),
        Some("0x8888888888888888")
    );
    assert!(verify.expected_writer_input_issues.is_empty());
    assert_eq!(
        verify.actual_writer_input_issues,
        vec!["tampered-writer-input".to_owned()]
    );
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue == "writer_input_valid mismatch: expected true, found false"));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("writer_input_hash mismatch: expected 0x")));
    assert!(verify.issues.iter().any(|issue| {
        issue == "writer_input_issues mismatch: expected [], found [tampered-writer-input]"
    }));
    assert!(verify_json.contains("\"actual_writer_input_valid\":false"));
    assert!(verify_json.contains("\"actual_writer_input_hash\":\"0x8888888888888888\""));
    assert!(verify_json.contains("\"actual_writer_input_issues\":[\"tampered-writer-input\"]"));
}

#[test]
fn verify_final_executable_emit_reports_host_invoke_plan_drift() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-host-invoke-plan-drift-{}",
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
    let invoke_plan =
        nsld_emit_final_executable_host_invoke_plan_report(Path::new("manifest.toml"), &plan)
            .unwrap();
    let emit = nsld_emit_final_executable_report(Path::new("manifest.toml"), &plan).unwrap();
    let blocked_path = Path::new(&emit.blocked_report_path);
    let damaged = fs::read_to_string(blocked_path)
        .unwrap()
        .replace(
            "host_invoke_plan_valid = true",
            "host_invoke_plan_valid = false",
        )
        .replace(
            &format!(
                "host_invoke_plan_hash = \"{}\"",
                invoke_plan.invoke_plan_hash
            ),
            "host_invoke_plan_hash = \"0x9999999999999999\"",
        )
        .replace(
            "host_invoke_plan_invocation_policy = \"dry-run-only\"",
            "host_invoke_plan_invocation_policy = \"allow-host-invoke\"",
        )
        .replace(
            "host_invoke_plan_requires_explicit_allow = true",
            "host_invoke_plan_requires_explicit_allow = false",
        )
        .replace(
            "host_invoke_plan_explicit_allow_present = false",
            "host_invoke_plan_explicit_allow_present = true",
        )
        .replace(
            "host_invoke_plan_would_invoke = false",
            "host_invoke_plan_would_invoke = true",
        )
        .replace(
            &format!(
                "host_invoke_plan_blocker_count = {}",
                invoke_plan.blocker_count
            ),
            "host_invoke_plan_blocker_count = 99",
        )
        .replace(
            "host_invoke_plan_issues = []",
            "host_invoke_plan_issues = [\"tampered-invoke-plan\"]",
        );
    fs::write(blocked_path, damaged).unwrap();
    let verify = nsld_verify_final_executable_emit_report(Path::new("manifest.toml"), &plan);
    let verify_json = super::json::nsld_final_executable_emit_verify_report_json(&verify);
    fs::remove_dir_all(dir).unwrap();

    assert!(!verify.valid);
    assert_eq!(verify.expected_host_invoke_plan_valid, Some(true));
    assert_eq!(verify.actual_host_invoke_plan_valid, Some(false));
    assert_eq!(verify.expected_host_invoke_plan_would_invoke, Some(false));
    assert_eq!(verify.actual_host_invoke_plan_would_invoke, Some(true));
    assert_eq!(
        verify.expected_host_invoke_plan_hash.as_deref(),
        Some(invoke_plan.invoke_plan_hash.as_str())
    );
    assert_eq!(
        verify.actual_host_invoke_plan_hash.as_deref(),
        Some("0x9999999999999999")
    );
    assert_eq!(
        verify
            .expected_host_invoke_plan_invocation_policy
            .as_deref(),
        Some("dry-run-only")
    );
    assert_eq!(
        verify.actual_host_invoke_plan_invocation_policy.as_deref(),
        Some("allow-host-invoke")
    );
    assert_eq!(
        verify.expected_host_invoke_plan_requires_explicit_allow,
        Some(true)
    );
    assert_eq!(
        verify.actual_host_invoke_plan_requires_explicit_allow,
        Some(false)
    );
    assert_eq!(
        verify.expected_host_invoke_plan_explicit_allow_present,
        Some(false)
    );
    assert_eq!(
        verify.actual_host_invoke_plan_explicit_allow_present,
        Some(true)
    );
    assert_eq!(
        verify.expected_host_invoke_plan_blocker_count,
        Some(invoke_plan.blocker_count)
    );
    assert_eq!(verify.actual_host_invoke_plan_blocker_count, Some(99));
    assert!(verify.expected_host_invoke_plan_issues.is_empty());
    assert_eq!(
        verify.actual_host_invoke_plan_issues,
        vec!["tampered-invoke-plan".to_owned()]
    );
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue == "host_invoke_plan_valid mismatch: expected true, found false"));
    assert!(
        verify
            .issues
            .iter()
            .any(|issue| issue
                == "host_invoke_plan_would_invoke mismatch: expected false, found true")
    );
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("host_invoke_plan_hash mismatch: expected 0x")));
    assert!(verify.issues.iter().any(|issue| issue
        == "host_invoke_plan_invocation_policy mismatch: expected dry-run-only, found allow-host-invoke"));
    assert!(verify.issues.iter().any(|issue| issue
        == "host_invoke_plan_requires_explicit_allow mismatch: expected true, found false"));
    assert!(verify.issues.iter().any(|issue| issue
        == "host_invoke_plan_explicit_allow_present mismatch: expected false, found true"));
    assert!(verify.issues.iter().any(|issue| issue
        == &format!(
            "host_invoke_plan_blocker_count mismatch: expected {}, found 99",
            invoke_plan.blocker_count
        )));
    assert!(verify.issues.iter().any(|issue| issue
        == "host_invoke_plan_issues mismatch: expected [], found [tampered-invoke-plan]"));
    assert!(verify_json.contains("\"actual_host_invoke_plan_valid\":false"));
    assert!(verify_json.contains("\"actual_host_invoke_plan_would_invoke\":true"));
    assert!(verify_json.contains("\"actual_host_invoke_plan_hash\":\"0x9999999999999999\""));
    assert!(
        verify_json.contains("\"actual_host_invoke_plan_invocation_policy\":\"allow-host-invoke\"")
    );
    assert!(verify_json.contains("\"actual_host_invoke_plan_requires_explicit_allow\":false"));
    assert!(verify_json.contains("\"actual_host_invoke_plan_explicit_allow_present\":true"));
    assert!(verify_json.contains("\"actual_host_invoke_plan_blocker_count\":99"));
    assert!(verify_json.contains("\"actual_host_invoke_plan_issues\":[\"tampered-invoke-plan\"]"));
}
