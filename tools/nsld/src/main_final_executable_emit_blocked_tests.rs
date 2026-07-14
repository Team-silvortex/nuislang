use super::{
    artifact_chain::{nsld_artifact_stage_kind_path, NsldArtifactStageKind},
    main_test_support::empty_link_plan,
    nsld_emit_final_executable_report, nsld_emit_final_executable_writer_input_report,
    nsld_prepare_report, nsld_verify_final_executable_emit_report,
};
use std::{env, fs, path::Path};

#[test]
fn emit_final_executable_writes_blocked_boundary_report() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-blocked-{}",
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
    let verify = nsld_verify_final_executable_emit_report(Path::new("manifest.toml"), &plan);
    let emit_json = super::json::nsld_final_executable_emit_report_json(&emit);
    let report_source = fs::read_to_string(&emit.blocked_report_path).unwrap();
    fs::remove_dir_all(dir).unwrap();

    assert!(!emit.emitted);
    assert!(!emit.can_emit_final_executable);
    assert!(!emit.final_stage_ready);
    assert!(emit.final_stage_plan_hash.starts_with("0x"));
    assert_eq!(emit.final_stage_driver, "clang");
    assert_eq!(emit.final_stage_link_mode, "host-toolchain-finalize");
    assert!(emit.host_wrapper_required);
    assert_eq!(emit.writer_kind, "host-assisted-final-executable");
    assert_eq!(emit.writer_status, "blocked");
    assert_eq!(emit.writer_input_path, writer_input.output_path);
    assert_eq!(emit.writer_input_valid, Some(true));
    assert_eq!(
        emit.writer_input_hash.as_deref(),
        Some(writer_input.writer_input_hash.as_str())
    );
    assert!(emit.writer_input_issues.is_empty());
    assert!(emit.host_dry_run_environment_ready.is_some());
    assert!(emit.host_dry_run_driver_available.is_some());
    assert!(emit.host_dry_run_can_invoke.is_some());
    assert_eq!(
        emit.host_invoke_plan_path,
        nsld_artifact_stage_kind_path(
            &plan.output_dir,
            NsldArtifactStageKind::FinalExecutableHostInvokePlan
        )
        .display()
        .to_string()
    );
    assert_eq!(emit.host_invoke_plan_valid, Some(false));
    assert_eq!(emit.host_invoke_plan_would_invoke, Some(false));
    assert!(emit.host_invoke_plan_hash.is_none());
    assert!(emit.host_invoke_plan_invocation_policy.is_none());
    assert_eq!(emit.host_invoke_plan_requires_explicit_allow, Some(false));
    assert_eq!(emit.host_invoke_plan_explicit_allow_present, Some(false));
    assert_eq!(emit.host_invoke_plan_blocker_count, Some(0));
    assert!(emit.host_invoke_plan_issues.iter().any(|issue| {
        issue.starts_with("missing_or_unreadable_final_executable_host_invoke_plan")
    }));
    assert_eq!(
        emit.writer_blockers,
        vec!["final-executable-writer:host-assisted:not-implemented".to_owned()]
    );
    assert_eq!(emit.input_count, 5);
    assert!(emit
        .blockers
        .iter()
        .any(|blocker| blocker == "self-owned-final-native-linker"));
    assert!(emit
        .blockers
        .iter()
        .any(|blocker| blocker == "host-finalizer-invoke-plan:invalid"));
    assert!(emit
        .blockers
        .iter()
        .any(|blocker| blocker == "host-finalizer-invoke-plan:not-allowed"));
    assert!(emit
        .notes
        .iter()
        .any(|note| note == "final-executable-emit-is-contract-only"));
    assert_eq!(
        emit.blocked_report_path,
        nsld_artifact_stage_kind_path(
            &plan.output_dir,
            NsldArtifactStageKind::FinalExecutableBlocked
        )
        .display()
        .to_string()
    );
    assert!(verify.valid, "{:?}", verify.issues);
    assert!(report_source.contains("schema = \"nuis-nsld-final-executable-blocked-v1\""));
    assert!(report_source.contains("producer_phase = \"alpha-0.10.0\""));
    assert!(report_source.contains("writer_kind = \"host-assisted-final-executable\""));
    assert!(report_source.contains("writer_status = \"blocked\""));
    assert!(report_source
        .contains("writer_blockers = [\"final-executable-writer:host-assisted:not-implemented\"]"));
    assert!(report_source.contains("writer_input_valid = true"));
    assert!(report_source.contains("writer_input_hash = \"0x"));
    assert!(report_source.contains("host_dry_run_environment_ready = "));
    assert!(report_source.contains("host_dry_run_driver_available = "));
    assert!(report_source.contains("host_dry_run_can_invoke = "));
    assert!(report_source.contains("host_invoke_plan_valid = false"));
    assert!(report_source.contains("host_invoke_plan_hash = \"\""));
    assert!(report_source.contains("host_invoke_plan_invocation_policy = \"\""));
    assert!(report_source.contains("host_finalizer_gate_status = \"invoke-plan-invalid\""));
    assert!(report_source
        .contains("host_finalizer_gate_action = \"emit-final-executable-host-invoke-plan\""));
    assert!(report_source.contains("host_invoke_plan_requires_explicit_allow = false"));
    assert!(report_source.contains("host_invoke_plan_explicit_allow_present = false"));
    assert!(report_source.contains("host_invoke_plan_would_invoke = false"));
    assert!(report_source.contains("host_invoke_plan_blocker_count = 0"));
    assert!(report_source.contains("image_dry_run_valid = false"));
    assert!(report_source.contains("image_dry_run_hash = \"\""));
    assert!(report_source.contains("final-executable-image-dry-run:invalid"));
    assert!(report_source.contains("emitted = false"));
    assert!(report_source.contains("blocker_count = "));
    assert!(emit_json.contains("\"kind\":\"nsld_final_executable_emit\""));
    assert!(!emit_json.contains("\"kind\":\"nsld_final_executable_readiness\""));
    assert!(emit_json.contains("\"emitted\":false"));
    assert!(emit_json.contains("\"writer_kind\":\"host-assisted-final-executable\""));
    assert!(emit_json.contains("\"writer_input_valid\":true"));
    assert!(emit_json.contains("\"host_dry_run_environment_ready\":"));
    assert!(emit_json.contains("\"host_invoke_plan_valid\":false"));
    assert!(emit_json.contains("\"host_invoke_plan_invocation_policy\":null"));
    assert!(emit_json.contains("\"host_finalizer_gate_status\":\"invoke-plan-invalid\""));
    assert!(emit_json
        .contains("\"host_finalizer_gate_action\":\"emit-final-executable-host-invoke-plan\""));
    assert!(emit_json.contains("\"host_invoke_plan_requires_explicit_allow\":false"));
    assert!(emit_json.contains("\"host_invoke_plan_explicit_allow_present\":false"));
    assert!(emit_json.contains("\"host_invoke_plan_would_invoke\":false"));
    assert!(emit_json.contains("\"host_invoke_plan_blocker_count\":0"));
    assert!(emit_json.contains("\"image_dry_run_valid\":false"));
    assert!(emit_json.contains("\"final_stage_plan_hash\":\"0x"));
}
