use super::{
    main_test_support::empty_link_plan, nsld_closure_report,
    nsld_emit_final_executable_host_invoke_plan_report, nsld_emit_final_executable_report,
    nsld_emit_final_executable_writer_input_report, nsld_prepare_report,
    nsld_verify_final_executable_emit_report, toml,
};
use std::{env, fs, path::Path};

#[test]
fn emit_final_executable_consumes_valid_host_invoke_plan_snapshot() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-valid-host-invoke-plan-{}",
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
    let verify = nsld_verify_final_executable_emit_report(Path::new("manifest.toml"), &plan);
    let emit_json = super::json::nsld_final_executable_emit_report_json(&emit);
    let report_source = fs::read_to_string(&emit.blocked_report_path).unwrap();
    fs::remove_dir_all(dir).unwrap();

    assert!(verify.valid, "{:?}", verify.issues);
    assert_eq!(emit.host_invoke_plan_valid, Some(true));
    assert_eq!(
        emit.host_invoke_plan_hash.as_deref(),
        Some(invoke_plan.invoke_plan_hash.as_str())
    );
    assert_eq!(
        emit.host_invoke_plan_invocation_policy.as_deref(),
        Some(invoke_plan.invocation_policy.as_str())
    );
    assert_eq!(emit.host_invoke_plan_requires_explicit_allow, Some(true));
    assert_eq!(emit.host_invoke_plan_explicit_allow_present, Some(false));
    assert_eq!(emit.host_invoke_plan_would_invoke, Some(false));
    assert_eq!(
        emit.host_invoke_plan_blocker_count,
        Some(invoke_plan.blocker_count)
    );
    assert!(emit.host_invoke_plan_issues.is_empty());
    assert!(!emit
        .blockers
        .iter()
        .any(|blocker| blocker == "host-finalizer-invoke-plan:invalid"));
    assert!(emit
        .blockers
        .iter()
        .any(|blocker| blocker == "host-finalizer-invoke-plan:not-allowed"));
    assert!(report_source.contains("host_invoke_plan_valid = true"));
    assert!(report_source.contains(&format!(
        "host_invoke_plan_hash = \"{}\"",
        invoke_plan.invoke_plan_hash
    )));
    assert!(report_source.contains(&format!(
        "host_invoke_plan_invocation_policy = \"{}\"",
        invoke_plan.invocation_policy
    )));
    assert!(report_source.contains("host_invoke_plan_requires_explicit_allow = true"));
    assert!(report_source.contains("host_invoke_plan_explicit_allow_present = false"));
    assert!(report_source.contains("host_invoke_plan_would_invoke = false"));
    assert!(report_source.contains("host_finalizer_gate_status = \"policy-blocked\""));
    assert!(report_source.contains(
        "host_finalizer_gate_action = \"set-env:NUIS_NSLD_HOST_FINALIZER_POLICY=allow-host-invoke\""
    ));
    assert!(report_source.contains(&format!(
        "host_invoke_plan_blocker_count = {}",
        invoke_plan.blocker_count
    )));
    assert!(emit_json.contains("\"host_invoke_plan_valid\":true"));
    assert!(emit_json.contains("\"host_invoke_plan_hash\":\"0x"));
    assert!(emit_json.contains("\"host_invoke_plan_invocation_policy\":\"dry-run-only\""));
    assert!(emit_json.contains("\"host_invoke_plan_requires_explicit_allow\":true"));
    assert!(emit_json.contains("\"host_invoke_plan_explicit_allow_present\":false"));
    assert!(emit_json.contains("\"host_invoke_plan_would_invoke\":false"));
    assert!(emit_json.contains("\"host_finalizer_gate_status\":\"policy-blocked\""));
    assert!(emit_json.contains(
        "\"host_finalizer_gate_action\":\"set-env:NUIS_NSLD_HOST_FINALIZER_POLICY=allow-host-invoke\""
    ));
    assert!(emit_json.contains("\"host_invoke_plan_blocker_count\":"));
}

#[test]
fn emit_final_executable_blocks_when_writer_input_is_missing() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-missing-writer-input-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit = nsld_emit_final_executable_report(Path::new("manifest.toml"), &plan).unwrap();
    let report_source = fs::read_to_string(&emit.blocked_report_path).unwrap();
    fs::remove_dir_all(dir).unwrap();

    assert_eq!(emit.writer_input_valid, Some(false));
    assert_eq!(emit.writer_input_hash, None);
    assert!(emit
        .blockers
        .iter()
        .any(|blocker| blocker == "final-executable-writer-input:invalid"));
    assert!(emit
        .writer_input_issues
        .iter()
        .any(|issue| { issue.starts_with("missing_or_unreadable_final_executable_writer_input") }));
    assert!(report_source.contains("writer_input_valid = false"));
    assert!(report_source.contains("writer_input_hash = \"\""));
    assert!(report_source.contains("final-executable-writer-input:invalid"));
}

#[test]
fn final_executable_blocked_artifact_does_not_change_closure_contract_hash() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-closure-stable-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let before = nsld_closure_report(Path::new("manifest.toml"), &plan);
    nsld_emit_final_executable_report(Path::new("manifest.toml"), &plan).unwrap();
    let after = nsld_closure_report(Path::new("manifest.toml"), &plan);
    fs::remove_dir_all(dir).unwrap();

    assert_eq!(after.linker_contract_hash, before.linker_contract_hash);
    assert_eq!(
        after.prepared_artifact_chain_valid,
        before.prepared_artifact_chain_valid
    );
    assert_eq!(
        after.prepared_artifact_chain_issues,
        before.prepared_artifact_chain_issues
    );
    assert!(!after
        .internal_contracts
        .iter()
        .any(|contract| contract.contains("final-executable")));
    assert!(!after
        .unresolved
        .iter()
        .any(|issue| issue.contains("final-executable")));
}

#[test]
fn verify_final_executable_emit_reports_plan_hash_drift() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-drift-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit = nsld_emit_final_executable_report(Path::new("manifest.toml"), &plan).unwrap();
    let report_path = Path::new(&emit.blocked_report_path);
    let original_blockers_line = format!(
        "blockers = [{}]",
        toml::toml_string_array_literal(&emit.blockers)
    );
    let mut tampered_blockers = emit.blockers.clone();
    tampered_blockers[0] = "tampered-final-executable-blocker".to_owned();
    let tampered_blockers_line = format!(
        "blockers = [{}]",
        toml::toml_string_array_literal(&tampered_blockers)
    );
    let damaged = fs::read_to_string(report_path)
        .unwrap()
        .replace(
            &format!("final_stage_plan_hash = \"{}\"", emit.final_stage_plan_hash),
            "final_stage_plan_hash = \"0x3333333333333333\"",
        )
        .replace(&original_blockers_line, &tampered_blockers_line);
    fs::write(report_path, damaged).unwrap();
    let verify = nsld_verify_final_executable_emit_report(Path::new("manifest.toml"), &plan);
    let verify_json = super::json::nsld_final_executable_emit_verify_report_json(&verify);
    fs::remove_dir_all(dir).unwrap();

    assert!(!verify.valid);
    assert!(verify.issues.iter().any(|issue| {
        issue.starts_with("final_stage_plan_hash mismatch: expected 0x")
            && issue.ends_with("found 0x3333333333333333")
    }));
    assert!(verify
        .expected_blockers
        .iter()
        .any(|blocker| blocker == &emit.blockers[0]));
    assert!(verify
        .actual_blockers
        .iter()
        .any(|blocker| blocker == "tampered-final-executable-blocker"));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("blockers mismatch")));
    assert!(verify_json.contains("\"actual_final_stage_plan_hash\":\"0x3333333333333333\""));
    assert!(verify_json.contains("\"tampered-final-executable-blocker\""));
}
