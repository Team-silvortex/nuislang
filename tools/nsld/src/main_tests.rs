use super::{
    artifact_chain::{
        nsld_artifact_chain_issues, nsld_artifact_chain_report, nsld_artifact_stage_file_name,
        nsld_artifact_stage_kind_path, NsldArtifactStage, NsldArtifactStageKind,
    },
    fnv1a64_hex,
    main_test_support::empty_link_plan,
    nsld_check_report, nsld_closure_report, nsld_emit_closure_report,
    nsld_emit_final_executable_host_invoke_plan_report,
    nsld_emit_final_executable_image_dry_run_report, nsld_emit_final_executable_layout_plan_report,
    nsld_emit_final_executable_report, nsld_emit_final_executable_writer_input_report,
    nsld_emit_final_stage_plan_report, nsld_final_executable_host_dry_run_report,
    nsld_final_executable_host_invoke_plan_report, nsld_final_executable_image_dry_run_report,
    nsld_final_executable_layout_plan_report, nsld_final_executable_readiness_report,
    nsld_final_executable_writer_plan_report, nsld_final_stage_plan_report,
    nsld_link_input_diagnostics, nsld_link_input_table_hash, nsld_prepare_report,
    nsld_sidecar_capability_diagnostics, nsld_verify_closure_report,
    nsld_verify_final_executable_emit_report, nsld_verify_final_executable_host_invoke_plan_report,
    nsld_verify_final_executable_image_dry_run_report,
    nsld_verify_final_executable_layout_plan_report,
    nsld_verify_final_executable_writer_input_report, nsld_verify_final_stage_plan_report, toml,
};
use crate::container_verify::{self, TomlFieldKind};
use std::{env, fs, path::Path};

#[test]
fn closure_reports_container_metadata_fingerprint() {
    let dir = env::temp_dir().join(format!("nsld-closure-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    let report = nsld_closure_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::nsld_closure_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(report.container_metadata_table_hash.starts_with("0x"));
    assert!(report.container_layout_hash.starts_with("0x"));
    assert!(report.container_hash.starts_with("0x"));
    assert!(report.payload_size_bytes > 0);
    assert!(report.payload_hash.starts_with("0x"));
    assert!(report.linker_contract_hash.starts_with("0x"));
    assert!(matches!(
        report.container_loader_readiness.as_str(),
        "blocked" | "host-assisted" | "self-contained"
    ));
    assert_eq!(report.compatibility_domain_count, 1);
    assert!(report.compatibility_domain_table_hash.starts_with("0x"));
    assert_eq!(
        report.compatibility_domain_id.as_deref(),
        Some("compat0000.cffi-von-neumann")
    );
    assert_eq!(
        report.compatibility_domain_kind.as_deref(),
        Some("cffi-host-compat")
    );
    assert_eq!(
        report.compatibility_domain_paradigm.as_deref(),
        Some("classic-von-neumann-host")
    );
    assert_eq!(
        report.compatibility_domain_lifecycle_hook.as_deref(),
        Some("on_cffi_native_object")
    );
    assert_eq!(
        report.compatibility_domain_abi_family.as_deref(),
        Some("mach-o")
    );
    assert_eq!(
        report.compatibility_domain_wrapper_policy.as_deref(),
        Some("wrapped")
    );
    assert_eq!(report.compatibility_domain_required, Some(true));
    assert!(report_json.contains("\"container_metadata_table_hash\":\"0x"));
    assert!(report_json.contains("\"container_layout_hash\":\"0x"));
    assert!(report_json.contains("\"container_hash\":\"0x"));
    assert!(report_json.contains("\"payload_size_bytes\":"));
    assert!(report_json.contains("\"payload_hash\":\"0x"));
    assert!(report_json.contains("\"linker_contract_hash\":\"0x"));
    assert!(report_json.contains("\"container_loader_readiness\":"));
    assert!(report_json.contains("\"compatibility_domain_count\":1"));
    assert!(report_json.contains("\"compatibility_domain_table_hash\":\"0x"));
    assert!(report_json.contains("\"compatibility_domain_id\":\"compat0000.cffi-von-neumann\""));
    assert!(report_json.contains("\"compatibility_domain_kind\":\"cffi-host-compat\""));
    assert!(report_json.contains("\"compatibility_domain_paradigm\":\"classic-von-neumann-host\""));
    assert!(
        report_json.contains("\"compatibility_domain_lifecycle_hook\":\"on_cffi_native_object\"")
    );
    assert!(report_json.contains("\"compatibility_domain_abi_family\":\"mach-o\""));
    assert!(report_json.contains("\"compatibility_domain_wrapper_policy\":\"wrapped\""));
    assert!(report_json.contains("\"compatibility_domain_required\":true"));
    assert!(
        report_json.contains("\"compatibility_domain_summary\":{\"count\":1,\"table_hash\":\"0x")
    );
}

#[test]
fn closure_linker_contract_hash_is_stable_and_contract_sensitive() {
    let dir = env::temp_dir().join(format!("nsld-closure-contract-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    let first = nsld_closure_report(Path::new("manifest.toml"), &plan);
    let second = nsld_closure_report(Path::new("manifest.toml"), &plan);
    plan.final_stage.link_mode = "bundle-packaging".to_owned();
    let changed = nsld_closure_report(Path::new("manifest.toml"), &plan);
    fs::remove_dir_all(dir).unwrap();

    assert_eq!(first.linker_contract_hash, second.linker_contract_hash);
    assert_ne!(first.linker_contract_hash, changed.linker_contract_hash);
    assert!(changed
        .external_dependencies
        .iter()
        .any(|dependency| dependency == "host-launcher-wrapper"));
}

#[test]
fn verify_closure_reports_linker_contract_hash_drift() {
    let dir = env::temp_dir().join(format!("nsld-closure-verify-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    let emit = nsld_emit_closure_report(Path::new("manifest.toml"), &plan).unwrap();
    let verify = nsld_verify_closure_report(Path::new("manifest.toml"), &plan);
    let snapshot_path = Path::new(&emit.output_path);
    let damaged = fs::read_to_string(snapshot_path).unwrap().replace(
        &format!("linker_contract_hash = \"{}\"", emit.linker_contract_hash),
        "linker_contract_hash = \"0x0000000000000000\"",
    );
    fs::write(snapshot_path, damaged).unwrap();
    let damaged_verify = nsld_verify_closure_report(Path::new("manifest.toml"), &plan);
    let verify_json = super::json::nsld_closure_verify_report_json(&damaged_verify);
    fs::remove_dir_all(dir).unwrap();

    assert!(verify.valid, "{:?}", verify.issues);
    assert_eq!(
        verify.actual_linker_contract_hash.as_deref(),
        Some(emit.linker_contract_hash.as_str())
    );
    assert!(verify
        .actual_container_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("0x")));
    assert!(verify
        .actual_payload_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("0x")));
    assert!(verify
        .actual_payload_size_bytes
        .is_some_and(|size| size > 0));
    assert!(!damaged_verify.valid);
    assert!(damaged_verify.issues.iter().any(|issue| {
        issue.starts_with("linker_contract_hash mismatch: expected 0x")
            && issue.ends_with("found 0x0000000000000000")
    }));
    assert!(verify_json.contains("\"actual_linker_contract_hash\":\"0x0000000000000000\""));
    assert!(verify_json.contains("\"actual_container_hash\":\"0x"));
    assert!(verify_json.contains("\"actual_payload_hash\":\"0x"));
}

#[test]
fn verify_closure_reports_container_hash_drift() {
    let dir = env::temp_dir().join(format!(
        "nsld-closure-container-drift-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    let emit = nsld_emit_closure_report(Path::new("manifest.toml"), &plan).unwrap();
    let snapshot_path = Path::new(&emit.output_path);
    let snapshot = fs::read_to_string(snapshot_path).unwrap();
    fs::write(
        snapshot_path,
        snapshot.replace(
            "container_hash = \"",
            "container_hash = \"0x1111111111111111",
        ),
    )
    .unwrap();
    let verify = nsld_verify_closure_report(Path::new("manifest.toml"), &plan);
    let verify_json = super::json::nsld_closure_verify_report_json(&verify);
    fs::remove_dir_all(dir).unwrap();

    assert!(!verify.valid);
    assert!(verify.issues.iter().any(|issue| {
        issue.starts_with("container_hash mismatch: expected 0x")
            && issue.contains("found 0x1111111111111111")
    }));
    assert!(verify_json.contains("\"actual_container_hash\":\"0x1111111111111111"));
}

#[test]
fn final_stage_plan_reports_deterministic_boundary_after_prepare() {
    let dir = env::temp_dir().join(format!("nsld-final-stage-plan-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let report = nsld_final_stage_plan_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::nsld_final_stage_plan_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(!report.ready);
    assert!(report.plan_hash.starts_with("0x"));
    assert_eq!(report.final_stage_driver, "clang");
    assert_eq!(report.final_stage_link_mode, "host-toolchain-finalize");
    assert!(report.host_wrapper_required);
    assert_eq!(report.compatibility_mode, "host-assisted-wrapper");
    assert_eq!(report.input_count, 4);
    assert!(report.inputs.iter().all(|input| input.present));
    assert!(report.container_hash.starts_with("0x"));
    assert!(report.payload_hash.starts_with("0x"));
    assert!(report.linker_contract_hash.starts_with("0x"));
    assert!(report.native_object_required);
    assert!(report.native_object_present);
    assert!(report
        .blockers
        .iter()
        .any(|blocker| blocker == "self-owned-final-native-linker"));
    assert!(report_json.contains("\"kind\":\"nsld_final_stage_plan\""));
    assert!(report_json.contains("\"plan_hash\":\"0x"));
    assert!(report_json.contains("\"final_stage_driver\":\"clang\""));
    assert!(report_json.contains("\"input_count\":4"));
    assert!(report_json.contains("\"inputs\":[{"));
    assert!(report_json.contains("\"input_id\":\"fsi0002.closure-snapshot\""));
    assert!(report_json.contains("\"container_hash\":\"0x"));
    assert!(report_json.contains("\"payload_hash\":\"0x"));
}

#[test]
fn final_executable_readiness_reports_without_writing_blocked_artifact() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-readiness-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let report = nsld_final_executable_readiness_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::nsld_final_executable_readiness_report_json(&report);
    let blocked_path = nsld_artifact_stage_kind_path(
        &plan.output_dir,
        NsldArtifactStageKind::FinalExecutableBlocked,
    );
    let blocked_present = blocked_path.exists();
    fs::remove_dir_all(dir).unwrap();

    assert!(!report.emitted);
    assert!(!report.can_emit_final_executable);
    assert!(!report.final_stage_ready);
    assert_eq!(
        report.blocked_report_path,
        blocked_path.display().to_string()
    );
    assert!(!blocked_present);
    assert!(report
        .blockers
        .iter()
        .any(|blocker| blocker == "self-owned-final-native-linker"));
    assert_eq!(report.writer_kind, "host-assisted-final-executable");
    assert_eq!(report.writer_status, "blocked");
    assert_eq!(
        report.writer_blockers,
        vec!["final-executable-writer:host-assisted:not-implemented".to_owned()]
    );
    assert!(report_json.contains("\"kind\":\"nsld_final_executable_readiness\""));
    assert!(report_json.contains("\"emitted\":false"));
    assert!(report_json.contains("\"writer_kind\":\"host-assisted-final-executable\""));
    assert!(report_json.contains("\"writer_status\":\"blocked\""));
    assert!(report_json.contains("final-executable-writer:host-assisted:not-implemented"));
    assert!(report_json.contains("\"blocked_report_path\":\""));
}

#[test]
fn final_executable_writer_plan_reports_host_assisted_steps() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-writer-plan-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let report = nsld_final_executable_writer_plan_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::nsld_final_executable_writer_plan_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert_eq!(report.writer_kind, "host-assisted-final-executable");
    assert_eq!(report.writer_status, "blocked");
    assert!(report.final_stage_plan_hash.starts_with("0x"));
    assert_eq!(report.final_stage_driver, "clang");
    assert_eq!(report.final_stage_link_mode, "host-toolchain-finalize");
    assert!(report.host_wrapper_required);
    assert_eq!(report.input_count, 4);
    assert_eq!(report.inputs.len(), 4);
    assert!(report.inputs.iter().any(|input| {
        input.input_id == "fsi0003.native-object" && input.required && input.present
    }));
    assert!(report
        .writer_steps
        .iter()
        .any(|step| step == "prepare-host-assisted-entry-wrapper"));
    assert!(report
        .writer_steps
        .iter()
        .any(|step| step == "invoke-host-finalizer-driver"));
    assert_eq!(
        report.writer_blockers,
        vec!["final-executable-writer:host-assisted:not-implemented".to_owned()]
    );
    assert!(report
        .notes
        .iter()
        .any(|note| note == "final-executable-writer-plan-is-non-mutating"));
    assert!(report_json.contains("\"kind\":\"nsld_final_executable_writer_plan\""));
    assert!(report_json.contains("\"writer_kind\":\"host-assisted-final-executable\""));
    assert!(report_json.contains("\"writer_steps\":["));
    assert!(report_json.contains(
        "\"writer_blockers\":[\"final-executable-writer:host-assisted:not-implemented\"]"
    ));
}

#[test]
fn final_executable_writer_input_emit_and_verify_are_deterministic() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-writer-input-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit =
        nsld_emit_final_executable_writer_input_report(Path::new("manifest.toml"), &plan).unwrap();
    let verify =
        nsld_verify_final_executable_writer_input_report(Path::new("manifest.toml"), &plan);
    let source = fs::read_to_string(&emit.output_path).unwrap();
    let emit_json = super::json::nsld_final_executable_writer_input_emit_report_json(&emit);
    let verify_json = super::json::nsld_final_executable_writer_input_verify_report_json(&verify);
    fs::remove_dir_all(dir).unwrap();

    assert!(Path::new(&emit.output_path)
        .file_name()
        .unwrap()
        .to_string_lossy()
        .contains("final-executable-writer-input"));
    assert!(emit.writer_input_hash.starts_with("0x"));
    assert_eq!(emit.writer_kind, "host-assisted-final-executable");
    assert_eq!(emit.writer_status, "blocked");
    assert_eq!(emit.final_stage_driver, "clang");
    assert_eq!(emit.final_stage_link_mode, "host-toolchain-finalize");
    assert!(emit.host_wrapper_required);
    assert!(emit.command_arg_count >= 4);
    assert_eq!(
        emit.writer_blockers,
        vec!["final-executable-writer:host-assisted:not-implemented".to_owned()]
    );
    assert!(verify.valid, "{:?}", verify.issues);
    assert_eq!(
        verify.actual_writer_input_hash.as_deref(),
        Some(emit.writer_input_hash.as_str())
    );
    assert_eq!(
        verify.actual_final_stage_plan_hash.as_deref(),
        Some(emit.final_stage_plan_hash.as_str())
    );
    assert_eq!(
        verify.actual_command_arg_count,
        Some(emit.command_arg_count)
    );
    assert_eq!(verify.expected_command_args, verify.actual_command_args);
    assert!(verify
        .actual_command_args
        .iter()
        .any(|arg| arg.contains("nuis.nsld.mach-o")));
    assert!(source.contains("schema = \"nuis-nsld-final-executable-writer-input-v1\""));
    assert!(source.contains("command_args = ["));
    assert!(source.contains("nuis.nsld.mach-o"));
    assert!(source
        .contains("writer_blockers = [\"final-executable-writer:host-assisted:not-implemented\"]"));
    assert!(emit_json.contains("\"kind\":\"nsld_final_executable_writer_input_emit\""));
    assert!(verify_json.contains("\"kind\":\"nsld_final_executable_writer_input_verify\""));
    assert!(verify_json.contains("\"valid\":true"));
    assert!(verify_json.contains("\"actual_command_args\":["));
}

#[test]
fn final_executable_host_dry_run_reports_missing_driver_without_invoking() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-host-dry-run-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();
    plan.final_stage.driver = "definitely-missing-nsld-host-driver-for-test".to_owned();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_executable_writer_input_report(Path::new("manifest.toml"), &plan).unwrap();
    let report = nsld_final_executable_host_dry_run_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::nsld_final_executable_host_dry_run_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(report.writer_input_valid);
    assert_eq!(
        report.driver,
        "definitely-missing-nsld-host-driver-for-test"
    );
    assert!(!report.driver_available);
    assert_eq!(report.driver_resolved_path, None);
    assert!(!report.environment_ready);
    assert_eq!(report.invocation_policy, "dry-run-only");
    assert_eq!(
        report.invocation_policy_reason,
        "alpha-host-finalizer-execution-disabled"
    );
    assert!(!report.can_invoke_host_finalizer);
    assert!(report
        .command_args
        .iter()
        .any(|arg| arg == "definitely-missing-nsld-host-driver-for-test"));
    assert!(report.blockers.iter().any(|blocker| blocker
        == "host-finalizer-driver-unavailable:definitely-missing-nsld-host-driver-for-test"));
    assert!(report
        .blockers
        .iter()
        .any(|blocker| blocker == "final-executable-writer:host-assisted:not-implemented"));
    assert!(report
        .blockers
        .iter()
        .any(|blocker| blocker == "host-finalizer-policy:dry-run-only"));
    assert!(report
        .notes
        .iter()
        .any(|note| note == "host-finalizer-is-not-invoked"));
    assert!(report_json.contains("\"kind\":\"nsld_final_executable_host_dry_run\""));
    assert!(report_json.contains("\"driver_available\":false"));
    assert!(report_json.contains("\"invocation_policy\":\"dry-run-only\""));
    assert!(report_json.contains("\"can_invoke_host_finalizer\":false"));
}

#[test]
fn final_executable_host_invoke_plan_requires_explicit_allow() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-host-invoke-plan-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();
    plan.final_stage.driver = "definitely-missing-nsld-host-driver-for-invoke-plan-test".to_owned();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_executable_writer_input_report(Path::new("manifest.toml"), &plan).unwrap();
    let report = nsld_final_executable_host_invoke_plan_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::nsld_final_executable_host_invoke_plan_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert_eq!(report.invocation_kind, "host-finalizer-command");
    assert_eq!(report.invocation_policy, "dry-run-only");
    assert!(report.requires_explicit_allow);
    assert!(!report.explicit_allow_present);
    assert!(!report.environment_ready);
    assert!(!report.driver_available);
    assert!(!report.can_invoke_host_finalizer);
    assert!(!report.would_invoke);
    assert!(report
        .blockers
        .iter()
        .any(|blocker| blocker == "host-finalizer-explicit-allow:missing"));
    assert!(report
        .blockers
        .iter()
        .any(|blocker| blocker == "host-finalizer-policy:dry-run-only"));
    assert!(report
        .notes
        .iter()
        .any(|note| note == "host-finalizer-process-is-not-spawned"));
    assert!(report_json.contains("\"kind\":\"nsld_final_executable_host_invoke_plan\""));
    assert!(report_json.contains("\"requires_explicit_allow\":true"));
    assert!(report_json.contains("\"would_invoke\":false"));
}

#[test]
fn final_executable_host_invoke_plan_emit_and_verify_round_trip() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-host-invoke-plan-emit-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();
    plan.final_stage.driver =
        "definitely-missing-nsld-host-driver-for-invoke-plan-emit-test".to_owned();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_executable_writer_input_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit =
        nsld_emit_final_executable_host_invoke_plan_report(Path::new("manifest.toml"), &plan)
            .unwrap();
    let verify =
        nsld_verify_final_executable_host_invoke_plan_report(Path::new("manifest.toml"), &plan);
    let emit_json = super::json::nsld_final_executable_host_invoke_plan_emit_report_json(&emit);
    let verify_json =
        super::json::nsld_final_executable_host_invoke_plan_verify_report_json(&verify);
    let report_source = fs::read_to_string(&emit.output_path).unwrap();
    fs::remove_dir_all(dir).unwrap();

    assert!(emit.invoke_plan_hash.starts_with("0x"));
    assert_eq!(emit.invocation_policy, "dry-run-only");
    assert!(emit.requires_explicit_allow);
    assert!(!emit.explicit_allow_present);
    assert!(!emit.would_invoke);
    assert!(emit.blocker_count > 0);
    assert!(verify.valid, "{:?}", verify.issues);
    assert_eq!(
        verify.actual_invoke_plan_hash.as_deref(),
        Some(emit.invoke_plan_hash.as_str())
    );
    assert!(report_source.contains("schema = \"nuis-nsld-final-executable-host-invoke-plan-v1\""));
    assert!(report_source.contains("would_invoke = false"));
    assert!(emit_json.contains("\"kind\":\"nsld_final_executable_host_invoke_plan_emit\""));
    assert!(verify_json.contains("\"kind\":\"nsld_final_executable_host_invoke_plan_verify\""));
    assert!(verify_json.contains("\"valid\":true"));
}

#[test]
fn verify_final_executable_host_invoke_plan_reports_gate_drift() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-emit-host-invoke-plan-drift-{}",
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
    let source = fs::read_to_string(&emit.output_path).unwrap();
    let command_arg_count_line = source
        .lines()
        .find(|line| line.starts_with("command_arg_count = "))
        .unwrap()
        .to_owned();
    let damaged = source
        .replace("would_invoke = false", "would_invoke = true")
        .replace(&command_arg_count_line, "command_arg_count = 0")
        .replace(
            "command_args = [\"clang\",",
            "command_args = [\"clang-drift\",",
        );
    fs::write(&emit.output_path, damaged).unwrap();
    let verify =
        nsld_verify_final_executable_host_invoke_plan_report(Path::new("manifest.toml"), &plan);
    let verify_json =
        super::json::nsld_final_executable_host_invoke_plan_verify_report_json(&verify);
    fs::remove_dir_all(dir).unwrap();

    assert!(!verify.valid);
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue == "would_invoke mismatch: expected false, found true"));
    assert!(verify.expected_command_arg_count > 0);
    assert_eq!(verify.actual_command_arg_count, Some(0));
    assert!(verify
        .expected_command_args
        .iter()
        .any(|arg| arg == "clang"));
    assert!(verify
        .actual_command_args
        .iter()
        .any(|arg| arg == "clang-drift"));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("command_arg_count mismatch: expected ")));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("command_args mismatch")));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue == "final-executable-host-invoke-plan-content-mismatch"));
    assert!(verify_json.contains("\"actual_would_invoke\":true"));
    assert!(verify_json.contains("\"actual_command_arg_count\":0"));
    assert!(verify_json.contains("\"clang-drift\""));
}

#[test]
fn final_executable_layout_plan_captures_nsld_owned_binary_boundary() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-layout-plan-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let report = nsld_final_executable_layout_plan_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::nsld_final_executable_layout_plan_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(report.layout_hash.starts_with("0x"));
    assert_eq!(report.internal_binary_format, "nuis-hetero-unified-binary");
    assert_eq!(report.lifecycle_entry_hook, "on_process_start");
    assert_eq!(
        report.scheduler_contract,
        "deterministic-lifecycle-hook-order"
    );
    assert_eq!(
        report.data_segment_ordering,
        "deterministic-data-segment-order"
    );
    assert_eq!(report.compatibility_domain, "cffi-native-object");
    assert_eq!(report.compatibility_lifecycle_hook, "on_cffi_native_object");
    assert_eq!(report.payload_count, report.payloads.len());
    assert!(report
        .payload_names
        .iter()
        .any(|payload| payload == "nsld-container"));
    assert!(report
        .payload_names
        .iter()
        .any(|payload| payload == "native-object-output"));
    assert!(report.payloads.iter().any(|payload| {
        payload.payload_id == "payload0003.native-object"
            && payload.lifecycle_hook == "on_cffi_native_object"
            && payload.content_hash.starts_with("0x")
    }));
    assert_eq!(report.byte_alignment, 16);
    assert!(report.byte_span > 0);
    assert!(report.byte_map_hash.starts_with("0x"));
    assert_eq!(report.byte_map_entries.len(), report.payloads.len());
    assert!(report
        .byte_map_entries
        .iter()
        .all(|entry| entry.offset % entry.alignment == 0));
    assert!(report
        .byte_map_entries
        .windows(2)
        .all(|entries| { entries[0].offset + entries[0].size_bytes <= entries[1].offset }));
    assert!(report
        .notes
        .iter()
        .any(|note| note == "platform-envelope-is-compatibility-shell"));
    assert!(report_json.contains("\"kind\":\"nsld_final_executable_layout_plan\""));
    assert!(report_json.contains("\"internal_binary_format\":\"nuis-hetero-unified-binary\""));
    assert!(report_json.contains("\"lifecycle_entry_hook\":\"on_process_start\""));
    assert!(report_json.contains("\"byte_map_hash\":\"0x"));
    assert!(report_json.contains("\"byte_map_entries\":["));
}

#[test]
fn final_executable_layout_plan_emit_and_verify_round_trip() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-layout-plan-emit-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit =
        nsld_emit_final_executable_layout_plan_report(Path::new("manifest.toml"), &plan).unwrap();
    let verify = nsld_verify_final_executable_layout_plan_report(Path::new("manifest.toml"), &plan);
    let emit_json = super::json::nsld_final_executable_layout_plan_emit_report_json(&emit);
    let verify_json = super::json::nsld_final_executable_layout_plan_verify_report_json(&verify);
    let source = fs::read_to_string(&emit.output_path).unwrap();
    fs::remove_dir_all(dir).unwrap();

    assert!(emit.layout_hash.starts_with("0x"));
    assert_eq!(
        verify.actual_layout_hash.as_deref(),
        Some(emit.layout_hash.as_str())
    );
    assert!(verify.valid, "{:?}", verify.issues);
    assert!(source.contains("schema = \"nuis-nsld-final-executable-layout-plan-v1\""));
    assert!(source.contains("platform_envelope_family = \"mach-o\""));
    assert!(source.contains("payloads = ["));
    assert!(source.contains("byte_alignment = 16"));
    assert!(source.contains("byte_map_hash = \"0x"));
    assert!(source.contains("[[byte_map_entry]]"));
    assert!(emit_json.contains("\"kind\":\"nsld_final_executable_layout_plan_emit\""));
    assert!(verify_json.contains("\"kind\":\"nsld_final_executable_layout_plan_verify\""));
    assert!(verify_json.contains("\"valid\":true"));
}

#[test]
fn final_executable_image_dry_run_emit_and_verify_round_trip() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-image-dry-run-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let report = nsld_final_executable_image_dry_run_report(Path::new("manifest.toml"), &plan);
    let emit =
        nsld_emit_final_executable_image_dry_run_report(Path::new("manifest.toml"), &plan).unwrap();
    let verify =
        nsld_verify_final_executable_image_dry_run_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::nsld_final_executable_image_dry_run_report_json(&report);
    let emit_json = super::json::nsld_final_executable_image_dry_run_emit_report_json(&emit);
    let verify_json = super::json::nsld_final_executable_image_dry_run_verify_report_json(&verify);
    let report_source = fs::read_to_string(&emit.output_path).unwrap();
    let image_bytes = fs::read(&emit.image_path).unwrap();
    fs::remove_dir_all(dir).unwrap();

    assert!(report.image_constructed);
    assert!(report.image_ready, "{:?}", report.blockers);
    assert!(report
        .image_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("0x")));
    assert!(emit.image_emitted);
    assert_eq!(emit.image_hash, report.image_hash);
    assert_eq!(emit.image_size_bytes, Some(image_bytes.len()));
    assert!(image_bytes.starts_with(b"NUIFIMG\0"));
    assert_eq!(
        u32::from_le_bytes(image_bytes[8..12].try_into().unwrap()),
        1
    );
    assert_eq!(
        u32::from_le_bytes(image_bytes[12..16].try_into().unwrap()),
        report.image_header_size as u32
    );
    assert_eq!(
        u32::from_le_bytes(image_bytes[16..20].try_into().unwrap()),
        report.payload_count as u32
    );
    assert_eq!(
        u64::from_le_bytes(image_bytes[24..32].try_into().unwrap()),
        report.payload_byte_span as u64
    );
    assert_eq!(
        u64::from_le_bytes(image_bytes[32..40].try_into().unwrap()),
        report.payload_byte_offset as u64
    );
    assert_eq!(
        emit.image_size_bytes,
        Some(report.image_header_size + report.payload_byte_span)
    );
    assert!(verify.valid, "{:?}", verify.issues);
    assert_eq!(verify.actual_image_hash, emit.image_hash);
    assert_eq!(
        verify.actual_image_header_size,
        Some(report.image_header_size)
    );
    assert_eq!(
        verify.actual_payload_byte_offset,
        Some(report.payload_byte_offset)
    );
    assert_eq!(verify.actual_image_magic.as_deref(), Some("NUIFIMG"));
    assert_eq!(verify.actual_image_version, Some(1));
    assert_eq!(
        verify.actual_payload_byte_span,
        Some(report.payload_byte_span)
    );
    assert_eq!(
        verify.actual_header_layout_hash.as_deref(),
        Some(report.layout_hash.as_str())
    );
    assert_eq!(
        verify.actual_header_byte_map_hash.as_deref(),
        Some(report.byte_map_hash.as_str())
    );
    assert_eq!(
        verify.expected_payload_region_count,
        verify.actual_payload_region_count.unwrap()
    );
    assert_eq!(
        verify.expected_payload_region_hash,
        verify.actual_payload_region_hash
    );
    assert!(verify
        .actual_payload_region_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("0x")));
    assert!(report_source.contains("schema = \"nuis-nsld-final-executable-image-dry-run-v1\""));
    assert!(report_source.contains("image_magic = \"NUIFIMG\""));
    assert!(report_source.contains("image_header_size = 64"));
    assert!(report_json.contains("\"kind\":\"nsld_final_executable_image_dry_run\""));
    assert!(report_json.contains("\"image_magic\":\"NUIFIMG\""));
    assert!(emit_json.contains("\"kind\":\"nsld_final_executable_image_dry_run_emit\""));
    assert!(emit_json.contains("\"image_header_size\":64"));
    assert!(verify_json.contains("\"kind\":\"nsld_final_executable_image_dry_run_verify\""));
    assert!(verify_json.contains("\"actual_image_magic\":\"NUIFIMG\""));
    assert!(verify_json.contains("\"actual_image_version\":1"));
    assert!(verify_json.contains("\"actual_header_layout_hash\":\"0x"));
    assert!(verify_json.contains("\"actual_payload_region_hash\":\"0x"));
    assert!(verify_json.contains("\"actual_image_header_size\":64"));
    assert!(verify_json.contains("\"valid\":true"));
}

#[test]
fn verify_final_executable_image_dry_run_reports_header_magic_drift() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-image-dry-run-header-drift-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit =
        nsld_emit_final_executable_image_dry_run_report(Path::new("manifest.toml"), &plan).unwrap();
    let mut image_bytes = fs::read(&emit.image_path).unwrap();
    image_bytes[0..8].copy_from_slice(b"BADMAGC\0");
    fs::write(&emit.image_path, image_bytes).unwrap();
    let verify =
        nsld_verify_final_executable_image_dry_run_report(Path::new("manifest.toml"), &plan);
    let verify_json = super::json::nsld_final_executable_image_dry_run_verify_report_json(&verify);
    fs::remove_dir_all(dir).unwrap();

    assert!(!verify.valid);
    assert_eq!(verify.actual_image_magic.as_deref(), Some("BADMAGC"));
    assert_eq!(verify.actual_image_version, Some(1));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue == "image_header_magic mismatch: expected NUIFIMG, found BADMAGC"));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("image_bytes_hash mismatch: expected 0x")));
    assert!(verify_json.contains("\"actual_image_magic\":\"BADMAGC\""));
    assert!(verify_json.contains("image_header_magic mismatch"));
}

#[test]
fn verify_final_executable_image_dry_run_reports_image_byte_drift() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-image-dry-run-drift-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit =
        nsld_emit_final_executable_image_dry_run_report(Path::new("manifest.toml"), &plan).unwrap();
    fs::write(&emit.image_path, b"drifted-image").unwrap();
    let verify =
        nsld_verify_final_executable_image_dry_run_report(Path::new("manifest.toml"), &plan);
    let verify_json = super::json::nsld_final_executable_image_dry_run_verify_report_json(&verify);
    fs::remove_dir_all(dir).unwrap();

    assert!(!verify.valid);
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("image_bytes_hash mismatch: expected 0x")));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue == "final-executable-image-header:invalid-or-too-short"));
    assert!(verify_json.contains("\"valid\":false"));
}

#[test]
fn verify_final_executable_image_dry_run_reports_payload_region_drift() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-image-dry-run-payload-drift-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit =
        nsld_emit_final_executable_image_dry_run_report(Path::new("manifest.toml"), &plan).unwrap();
    let mut image_bytes = fs::read(&emit.image_path).unwrap();
    image_bytes[64] ^= 0xff;
    fs::write(&emit.image_path, image_bytes).unwrap();
    let verify =
        nsld_verify_final_executable_image_dry_run_report(Path::new("manifest.toml"), &plan);
    let verify_json = super::json::nsld_final_executable_image_dry_run_verify_report_json(&verify);
    fs::remove_dir_all(dir).unwrap();

    assert!(!verify.valid);
    assert_eq!(verify.actual_image_magic.as_deref(), Some("NUIFIMG"));
    assert_ne!(
        verify.expected_payload_region_hash,
        verify.actual_payload_region_hash
    );
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("image_payload_region_hash mismatch")));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("image_payload_region_entry_hash mismatch for ")));
    assert!(verify_json.contains("\"actual_payload_region_hash\":\"0x"));
    assert!(verify_json.contains("image_payload_region_hash mismatch"));
}

#[test]
fn verify_final_executable_layout_plan_reports_protocol_drift() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-layout-plan-drift-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit =
        nsld_emit_final_executable_layout_plan_report(Path::new("manifest.toml"), &plan).unwrap();
    let source = fs::read_to_string(&emit.output_path).unwrap();
    let byte_span_line = source
        .lines()
        .find(|line| line.starts_with("byte_span = "))
        .unwrap()
        .to_owned();
    let damaged = source
        .replace(
            "lifecycle_entry_hook = \"on_process_start\"",
            "lifecycle_entry_hook = \"drift\"",
        )
        .replace(
            "platform_envelope_family = \"mach-o\"",
            "platform_envelope_family = \"elf\"",
        )
        .replace(&byte_span_line, "byte_span = 0")
        .replace("payload_count = 4", "payload_count = 0");
    fs::write(&emit.output_path, damaged).unwrap();
    let verify = nsld_verify_final_executable_layout_plan_report(Path::new("manifest.toml"), &plan);
    let verify_json = super::json::nsld_final_executable_layout_plan_verify_report_json(&verify);
    fs::remove_dir_all(dir).unwrap();

    assert!(!verify.valid);
    assert_eq!(verify.actual_lifecycle_entry_hook.as_deref(), Some("drift"));
    assert_eq!(
        verify.actual_platform_envelope_family.as_deref(),
        Some("elf")
    );
    assert_eq!(verify.actual_payload_count, Some(0));
    assert_eq!(verify.actual_byte_span, Some(0));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue == "final-executable-layout-plan-content-mismatch"));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("byte_span mismatch: expected ")));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue
            == "lifecycle_entry_hook mismatch: expected on_process_start, found drift"));
    assert!(verify_json.contains("\"actual_lifecycle_entry_hook\":\"drift\""));
    assert!(verify_json.contains("\"actual_platform_envelope_family\":\"elf\""));
}

#[test]
fn verify_final_executable_writer_input_reports_command_args_drift() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-writer-input-args-drift-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit =
        nsld_emit_final_executable_writer_input_report(Path::new("manifest.toml"), &plan).unwrap();
    let source = fs::read_to_string(&emit.output_path).unwrap();
    let damaged = source.replace(
        "command_args = [\"clang\",",
        "command_args = [\"clang-drift\",",
    );
    fs::write(&emit.output_path, damaged).unwrap();
    let verify =
        nsld_verify_final_executable_writer_input_report(Path::new("manifest.toml"), &plan);
    fs::remove_dir_all(dir).unwrap();

    assert!(!verify.valid);
    assert_ne!(verify.expected_command_args, verify.actual_command_args);
    assert!(verify
        .actual_command_args
        .iter()
        .any(|arg| arg == "clang-drift"));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("command_args mismatch")));
}

#[test]
fn verify_final_executable_writer_input_reports_plan_hash_drift() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-emit-writer-input-drift-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit =
        nsld_emit_final_executable_writer_input_report(Path::new("manifest.toml"), &plan).unwrap();
    let damaged = fs::read_to_string(&emit.output_path).unwrap().replace(
        &format!("final_stage_plan_hash = \"{}\"", emit.final_stage_plan_hash),
        "final_stage_plan_hash = \"0x3333333333333333\"",
    );
    fs::write(&emit.output_path, damaged).unwrap();
    let verify =
        nsld_verify_final_executable_writer_input_report(Path::new("manifest.toml"), &plan);
    fs::remove_dir_all(dir).unwrap();

    assert!(!verify.valid);
    assert_eq!(
        verify.actual_final_stage_plan_hash.as_deref(),
        Some("0x3333333333333333")
    );
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.contains("final_stage_plan_hash mismatch")));
}

#[test]
fn verify_final_stage_plan_reports_plan_hash_drift() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-stage-plan-drift-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit = nsld_emit_final_stage_plan_report(Path::new("manifest.toml"), &plan).unwrap();
    let verify = nsld_verify_final_stage_plan_report(Path::new("manifest.toml"), &plan);
    let plan_path = Path::new(&emit.output_path);
    let damaged = fs::read_to_string(plan_path).unwrap().replace(
        &format!("plan_hash = \"{}\"", emit.plan_hash),
        "plan_hash = \"0x2222222222222222\"",
    );
    fs::write(plan_path, damaged).unwrap();
    let damaged_verify = nsld_verify_final_stage_plan_report(Path::new("manifest.toml"), &plan);
    let verify_json = super::json::nsld_final_stage_plan_verify_report_json(&damaged_verify);
    fs::remove_dir_all(dir).unwrap();

    assert!(verify.valid, "{:?}", verify.issues);
    assert_eq!(
        verify.actual_plan_hash.as_deref(),
        Some(emit.plan_hash.as_str())
    );
    assert!(!damaged_verify.valid);
    assert!(damaged_verify.issues.iter().any(|issue| {
        issue.starts_with("plan_hash mismatch: expected 0x")
            && issue.ends_with("found 0x2222222222222222")
    }));
    assert!(verify_json.contains("\"actual_plan_hash\":\"0x2222222222222222\""));
}

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
    assert!(emit
        .host_invoke_plan_issues
        .iter()
        .any(|issue| issue.starts_with("missing_or_unreadable_final_executable_host_invoke_plan")));
    assert_eq!(
        emit.writer_blockers,
        vec!["final-executable-writer:host-assisted:not-implemented".to_owned()]
    );
    assert_eq!(emit.input_count, 4);
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
    assert!(report_source.contains("producer_phase = \"alpha-0.8.0\""));
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
    assert!(report_source.contains("host_invoke_plan_would_invoke = false"));
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
    assert!(emit_json.contains("\"host_invoke_plan_would_invoke\":false"));
    assert!(emit_json.contains("\"image_dry_run_valid\":false"));
    assert!(emit_json.contains("\"final_stage_plan_hash\":\"0x"));
}

#[test]
fn emit_final_executable_consumes_valid_image_dry_run_snapshot() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-valid-image-dry-run-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let image =
        nsld_emit_final_executable_image_dry_run_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit = nsld_emit_final_executable_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit_json = super::json::nsld_final_executable_emit_report_json(&emit);
    let verify = nsld_verify_final_executable_emit_report(Path::new("manifest.toml"), &plan);
    fs::remove_dir_all(dir).unwrap();

    assert_eq!(emit.image_dry_run_valid, Some(true));
    assert_eq!(emit.image_dry_run_hash, image.image_hash);
    assert_eq!(emit.image_dry_run_size_bytes, image.image_size_bytes);
    assert!(emit.image_dry_run_issues.is_empty());
    assert!(!emit
        .blockers
        .iter()
        .any(|blocker| blocker == "final-executable-image-dry-run:invalid"));
    assert!(emit_json.contains("\"image_dry_run_valid\":true"));
    assert!(emit_json.contains("\"image_dry_run_hash\":\"0x"));
    assert!(emit_json.contains("\"image_dry_run_size_bytes\":"));
    assert!(verify.valid, "{:?}", verify.issues);
    assert_eq!(verify.expected_image_dry_run_valid, Some(true));
    assert_eq!(verify.actual_image_dry_run_valid, Some(true));
    assert_eq!(verify.expected_image_dry_run_hash, image.image_hash);
    assert_eq!(verify.actual_image_dry_run_hash, image.image_hash);
}

#[test]
fn emit_final_executable_records_host_dry_run_driver_blocker() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-host-driver-blocked-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();
    plan.final_stage.driver = "definitely-missing-nsld-host-driver-for-emit-test".to_owned();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_executable_writer_input_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit = nsld_emit_final_executable_report(Path::new("manifest.toml"), &plan).unwrap();
    let report_source = fs::read_to_string(&emit.blocked_report_path).unwrap();
    fs::remove_dir_all(dir).unwrap();

    assert_eq!(emit.writer_input_valid, Some(true));
    assert_eq!(emit.host_dry_run_environment_ready, Some(false));
    assert_eq!(emit.host_dry_run_driver_available, Some(false));
    assert_eq!(emit.host_dry_run_driver_resolved_path, None);
    assert_eq!(emit.host_dry_run_can_invoke, Some(false));
    assert_eq!(
        emit.host_dry_run_invocation_policy.as_deref(),
        Some("dry-run-only")
    );
    assert_eq!(
        emit.host_dry_run_invocation_policy_reason.as_deref(),
        Some("alpha-host-finalizer-execution-disabled")
    );
    assert_eq!(
        emit.host_dry_run_blocker_count,
        emit.host_dry_run_blockers.len()
    );
    assert!(emit.host_dry_run_blockers.iter().any(|blocker| blocker
        == "host-finalizer-driver-unavailable:definitely-missing-nsld-host-driver-for-emit-test"));
    assert!(emit
        .blockers
        .iter()
        .any(|blocker| blocker == "host-finalizer-environment:not-ready"));
    assert!(report_source.contains("host_dry_run_environment_ready = false"));
    assert!(report_source.contains("host_dry_run_driver_available = false"));
    assert!(report_source.contains("host_dry_run_driver_resolved_path = \"\""));
    assert!(report_source.contains("host_dry_run_can_invoke = false"));
    assert!(report_source.contains("host_dry_run_invocation_policy = \"dry-run-only\""));
    assert!(report_source.contains(
        "host_dry_run_invocation_policy_reason = \"alpha-host-finalizer-execution-disabled\""
    ));
    assert!(report_source.contains("host_dry_run_blocker_count = "));
    assert!(report_source.contains(
        "host-finalizer-driver-unavailable:definitely-missing-nsld-host-driver-for-emit-test"
    ));
}

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
    assert_eq!(emit.host_invoke_plan_would_invoke, Some(false));
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
    assert!(report_source.contains("host_invoke_plan_would_invoke = false"));
    assert!(emit_json.contains("\"host_invoke_plan_valid\":true"));
    assert!(emit_json.contains("\"host_invoke_plan_hash\":\"0x"));
    assert!(emit_json.contains("\"host_invoke_plan_would_invoke\":false"));
}

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
    assert!(verify
        .expected_host_dry_run_blockers
        .iter()
        .any(|blocker| blocker
            == "host-finalizer-driver-unavailable:definitely-missing-nsld-host-driver-for-drift-test"));
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
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue
            == "host_dry_run_invocation_policy mismatch: expected dry-run-only, found allow-host-invoke"));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue
            == "host_dry_run_invocation_policy_reason mismatch: expected alpha-host-finalizer-execution-disabled, found tampered-policy"));
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
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue
            == "writer_input_issues mismatch: expected [], found [tampered-writer-input]"));
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
            "host_invoke_plan_would_invoke = false",
            "host_invoke_plan_would_invoke = true",
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
        == "host_invoke_plan_issues mismatch: expected [], found [tampered-invoke-plan]"));
    assert!(verify_json.contains("\"actual_host_invoke_plan_valid\":false"));
    assert!(verify_json.contains("\"actual_host_invoke_plan_would_invoke\":true"));
    assert!(verify_json.contains("\"actual_host_invoke_plan_hash\":\"0x9999999999999999\""));
    assert!(verify_json.contains("\"actual_host_invoke_plan_issues\":[\"tampered-invoke-plan\"]"));
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
    let damaged = fs::read_to_string(report_path).unwrap().replace(
        &format!("final_stage_plan_hash = \"{}\"", emit.final_stage_plan_hash),
        "final_stage_plan_hash = \"0x3333333333333333\"",
    );
    fs::write(report_path, damaged).unwrap();
    let verify = nsld_verify_final_executable_emit_report(Path::new("manifest.toml"), &plan);
    let verify_json = super::json::nsld_final_executable_emit_verify_report_json(&verify);
    fs::remove_dir_all(dir).unwrap();

    assert!(!verify.valid);
    assert!(verify.issues.iter().any(|issue| {
        issue.starts_with("final_stage_plan_hash mismatch: expected 0x")
            && issue.ends_with("found 0x3333333333333333")
    }));
    assert!(verify_json.contains("\"actual_final_stage_plan_hash\":\"0x3333333333333333\""));
}

#[test]
fn closure_reports_verified_prepared_artifact_chain_after_prepare() {
    let dir = env::temp_dir().join(format!("nsld-closure-prepared-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let report = nsld_closure_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::nsld_closure_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(report.prepared_artifact_chain_valid);
    assert!(report.prepared_artifact_chain_issues.is_empty());
    assert!(report
        .internal_contracts
        .iter()
        .any(|contract| contract == "verified-prepared-artifact-chain"));
    assert!(report
        .internal_contracts
        .iter()
        .any(|contract| contract == "verified-object-writer-input"));
    assert!(report
        .internal_contracts
        .iter()
        .any(|contract| contract == "verified-object-output"));
    assert!(report
        .internal_contracts
        .iter()
        .any(|contract| contract == "verified-object-writer-dry-run"));
    assert!(report
        .internal_contracts
        .iter()
        .any(|contract| contract == "verified-object-image-relocation-record-table"));
    assert!(report.linker_contract_hash.starts_with("0x"));
    assert_eq!(report.object_image_relocation_lowering_valid, Some(true));
    assert_eq!(report.object_image_relocation_lowering_rule_count, Some(4));
    assert_eq!(report.object_image_relocation_lowering_rules.len(), 4);
    assert_eq!(
        report.object_image_relocation_lowering_rules[0].source_seed_kind,
        "bootstrap-entry-seed"
    );
    assert!(report.object_image_relocation_lowering_issues.is_empty());
    assert_eq!(report.object_image_relocation_record_count, Some(4));
    assert!(report
        .object_image_relocation_record_table_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("0x")));
    assert_eq!(report.object_image_relocation_records.len(), 4);
    assert_eq!(
        report.object_image_relocation_records[0].relocation_seed_id,
        "orel0000.compiled_artifact"
    );
    assert!(report_json.contains("\"prepared_artifact_chain_valid\":true"));
    assert!(report_json.contains("\"linker_contract_hash\":\"0x"));
    assert!(report_json.contains("\"prepared_artifact_chain_issues\":[]"));
    assert!(report_json.contains("\"object_image_relocation_lowering_valid\":true"));
    assert!(report_json.contains("\"object_image_relocation_lowering_rule_count\":4"));
    assert!(report_json.contains("\"object_image_relocation_lowering_rules\":[{"));
    assert!(report_json.contains("\"source_seed_kind\":\"bootstrap-entry-seed\""));
    assert!(report_json.contains("\"object_image_relocation_lowering_issues\":[]"));
    assert!(report_json.contains("\"object_image_relocation_record_count\":4"));
    assert!(report_json.contains("\"object_image_relocation_record_table_hash\":\"0x"));
    assert!(report_json.contains("\"object_image_relocation_records\":[{"));
    assert!(report_json.contains("\"relocation_seed_id\":\"orel0000.compiled_artifact\""));
}

#[test]
fn scoped_toml_helpers_read_the_first_matching_table_only() {
    let source = r#"
[[loader_symbol]]
symbol_id = "sym0000.loader-entry"
section_id = "sec0000.compiled-artifact"

[[external_import]]
import_id = "imp0000.final-stage-driver"
required = true

[[section]]
section_id = "sec9999.section-table"

[[external_import]]
import_id = "imp0001.clang-target"
required = false
"#;

    assert_eq!(
        toml::first_table_string_value(source, "loader_symbol", "section_id").as_deref(),
        Some("sec0000.compiled-artifact")
    );
    assert_eq!(
        toml::first_table_string_value(source, "external_import", "import_id").as_deref(),
        Some("imp0000.final-stage-driver")
    );
    assert_eq!(
        toml::first_table_bool_value(source, "external_import", "required"),
        Some(true)
    );
    assert_eq!(
        toml::first_table_string_value(source, "missing", "section_id"),
        None
    );
}

#[test]
fn table_field_issues_report_missing_and_invalid_fields() {
    let source = r#"
[[relocation]]
relocation_id = "rel0000.lifecycle-entry"
source_offset = "not-a-number"

[[relocation]]
relocation_id = "rel0001.hetero-node"
source_offset = 12
"#;

    let issues = container_verify::table_field_issues(
        source,
        "relocation",
        "relocation",
        &[
            ("relocation_id", TomlFieldKind::String),
            ("relocation_kind", TomlFieldKind::String),
            ("source_offset", TomlFieldKind::Usize),
        ],
    );

    assert!(issues
        .iter()
        .any(|issue| issue == "relocation[0].relocation_kind missing"));
    assert!(issues
        .iter()
        .any(|issue| issue == "relocation[0].source_offset invalid"));
    assert!(issues
        .iter()
        .any(|issue| issue == "relocation[1].relocation_kind missing"));
}

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
    let report_json = super::json::nsld_artifact_chain_report_json(&report);
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
    let report_json = super::json::nsld_artifact_chain_report_json(&report);
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
        file_name,
        present,
        required: true,
    }
}

fn test_optional_artifact_stage(file_name: &'static str, present: bool) -> NsldArtifactStage {
    NsldArtifactStage {
        kind: NsldArtifactStageKind::ObjectOutput,
        file_name,
        present,
        required: false,
    }
}

#[test]
fn sidecar_capability_check_skips_hetero_domains_without_ir_sidecars() {
    let path = env::temp_dir().join(format!("nsld-sidecar-cap-{}.toml", std::process::id()));
    let sidecar_source = r#"
schema = "nuis-shader-ir-sidecar-v1"
[lowering_capabilities]
capability_owner = "shader-nustar"
frontend_ir = "nuis-yir.shader"
native_ir = "msl2.4"
dispatch_lowering = "command-encoder-draw-dispatch"
validation_contracts = ["glm.resource-lifetime"]
"#;
    fs::write(&path, sidecar_source).unwrap();
    let mut plan = empty_link_plan();
    plan.domain_units.push(nuisc::linker::LinkPlanDomainUnit {
        kind: "heterogeneous".to_owned(),
        package_id: "official.data".to_owned(),
        domain_family: "data".to_owned(),
        abi: None,
        machine_arch: None,
        machine_os: None,
        backend_family: None,
        vendor: None,
        device_class: None,
        selected_lowering_target: None,
        contract_family: "nustar.data".to_owned(),
        packaging_role: "domain-sidecar".to_owned(),
        artifact_stub_path: None,
        artifact_stub_inline: None,
        artifact_payload_path: None,
        artifact_bridge_stub_path: None,
        artifact_ir_sidecar_path: None,
        artifact_bridge_stub_inline: None,
        artifact_payload_blob_path: None,
        artifact_payload_blob_bytes: None,
        artifact_payload_format: None,
        artifact_payload_blob_inline: None,
    });
    plan.domain_units.push(nuisc::linker::LinkPlanDomainUnit {
        kind: "heterogeneous".to_owned(),
        package_id: "official.shader".to_owned(),
        domain_family: "shader".to_owned(),
        abi: None,
        machine_arch: None,
        machine_os: None,
        backend_family: Some("metal".to_owned()),
        vendor: None,
        device_class: None,
        selected_lowering_target: Some("metal.apple-silicon-gpu".to_owned()),
        contract_family: "nustar.shader".to_owned(),
        packaging_role: "hetero-contract".to_owned(),
        artifact_stub_path: None,
        artifact_stub_inline: None,
        artifact_payload_path: None,
        artifact_bridge_stub_path: None,
        artifact_ir_sidecar_path: Some(path.display().to_string()),
        artifact_bridge_stub_inline: None,
        artifact_payload_blob_path: None,
        artifact_payload_blob_bytes: None,
        artifact_payload_format: None,
        artifact_payload_blob_inline: None,
    });

    let diagnostics = nsld_sidecar_capability_diagnostics(&plan);
    fs::remove_file(path).unwrap();

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].domain_family, "shader");
    assert_eq!(diagnostics[0].content_bytes, sidecar_source.len());
    assert_eq!(
        diagnostics[0].content_hash,
        fnv1a64_hex(sidecar_source.as_bytes())
    );
    assert!(diagnostics[0].valid);
    let link_inputs = nsld_link_input_diagnostics(&diagnostics);
    assert_eq!(link_inputs.len(), 1);
    assert_eq!(link_inputs[0].order_index, 0);
    assert_eq!(link_inputs[0].input_id, "li0000.shader.official.shader");
    assert_eq!(link_inputs[0].input_kind, "lowering-ir-sidecar");
    assert_eq!(link_inputs[0].native_ir, "msl2.4");
    assert_eq!(
        link_inputs[0].dispatch_lowering,
        "command-encoder-draw-dispatch"
    );
    assert_eq!(link_inputs[0].content_bytes, sidecar_source.len());
    assert_eq!(
        link_inputs[0].content_hash,
        fnv1a64_hex(sidecar_source.as_bytes())
    );
    let expected_material = format!(
        "0\tli0000.shader.official.shader\tlowering-ir-sidecar\tshader\tofficial.shader\tmsl2.4\tcommand-encoder-draw-dispatch\t1\t{}\t{}\n",
        sidecar_source.len(),
        fnv1a64_hex(sidecar_source.as_bytes())
    );
    assert_eq!(
        nsld_link_input_table_hash(&link_inputs),
        fnv1a64_hex(expected_material.as_bytes())
    );
    let table = toml::render_link_input_table(
        &link_inputs,
        link_inputs
            .iter()
            .map(|input| input.content_bytes)
            .sum::<usize>(),
        &nsld_link_input_table_hash(&link_inputs),
    );
    assert!(table.contains("schema = \"nuis-nsld-link-input-table-v1\""));
    assert!(table.contains("schema_version = 1"));
    assert!(table.contains("table_kind = \"lowering-sidecar-link-inputs\""));
    assert!(table.contains("producer = \"nsld\""));
    assert!(table.contains("producer_phase = \"alpha-0.6.0\""));
    assert!(table.contains("link_input_count = 1"));
    assert!(table.contains("input_id = \"li0000.shader.official.shader\""));
    assert!(table.contains("native_ir = \"msl2.4\""));
    assert!(table.contains("content_hash = \""));
}

#[test]
fn check_reports_container_loader_readiness_without_failing_host_assisted_state() {
    let dir = env::temp_dir().join(format!("nsld-check-loader-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let report = nsld_check_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::check_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(report.valid);
    assert!(report.object_plan_present);
    assert_eq!(report.object_plan_valid, Some(true));
    assert!(report.object_plan_issues.is_empty());
    assert!(report.object_writer_input_present);
    assert_eq!(report.object_writer_input_valid, Some(true));
    assert!(report.object_writer_input_issues.is_empty());
    assert!(report.object_byte_layout_present);
    assert_eq!(report.object_byte_layout_valid, Some(true));
    assert!(report.object_byte_layout_issues.is_empty());
    assert!(report.object_file_layout_present);
    assert_eq!(report.object_file_layout_valid, Some(true));
    assert!(report.object_file_layout_issues.is_empty());
    assert!(report.object_image_dry_run_present);
    assert_eq!(report.object_image_dry_run_valid, Some(true));
    assert!(report.object_image_dry_run_issues.is_empty());
    assert_eq!(report.object_image_relocation_lowering_valid, Some(true));
    assert_eq!(report.object_image_relocation_lowering_rule_count, Some(4));
    assert_eq!(report.object_image_relocation_lowering_rules.len(), 4);
    assert_eq!(
        report.object_image_relocation_lowering_rules[0].source_seed_kind,
        "bootstrap-entry-seed"
    );
    assert!(report.object_image_relocation_lowering_issues.is_empty());
    assert_eq!(report.object_image_relocation_record_count, Some(4));
    assert!(report
        .object_image_relocation_record_table_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("0x")));
    assert_eq!(report.object_image_relocation_records.len(), 4);
    assert_eq!(
        report.object_image_relocation_records[0].relocation_seed_id,
        "orel0000.compiled_artifact"
    );
    assert!(report.object_image_dry_run_bytes_present);
    assert!(report.object_emit_blocked_present);
    assert_eq!(report.object_emit_blocked_valid, Some(true));
    assert!(report.object_emit_blocked_issues.is_empty());
    assert!(report.object_output_present);
    assert_eq!(report.object_output_valid, Some(true));
    assert!(report.object_output_expected_size_bytes.is_some());
    assert_eq!(
        report.object_output_expected_size_bytes,
        report.object_output_actual_size_bytes
    );
    assert!(report
        .object_output_expected_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("0x")));
    assert_eq!(
        report.object_output_expected_hash,
        report.object_output_actual_hash
    );
    assert!(report.object_output_issues.is_empty());
    assert!(report.object_writer_dry_run_present);
    assert_eq!(report.object_writer_dry_run_valid, Some(true));
    assert!(report.object_writer_dry_run_issues.is_empty());
    assert!(report.container_section_issues.is_empty());
    assert!(report.container_loader_symbol_issues.is_empty());
    assert!(report.container_relocation_issues.is_empty());
    assert!(report.container_compatibility_domain_issues.is_empty());
    assert!(report.container_external_import_issues.is_empty());
    assert!(report.closure_snapshot_present);
    assert_eq!(report.closure_snapshot_valid, Some(true));
    assert!(report.closure_snapshot_issues.is_empty());
    assert!(report
        .closure_snapshot_linker_contract_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("0x")));
    assert!(report
        .closure_snapshot_container_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("0x")));
    assert!(report
        .closure_snapshot_payload_size_bytes
        .is_some_and(|size| size > 0));
    assert!(report
        .closure_snapshot_payload_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("0x")));
    assert!(report.final_stage_plan_present);
    assert_eq!(report.final_stage_plan_valid, Some(true));
    assert_eq!(report.final_stage_plan_ready, Some(false));
    assert!(report
        .final_stage_plan_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("0x")));
    assert!(report
        .final_stage_plan_blocker_count
        .is_some_and(|count| count >= 1));
    assert!(report.final_stage_plan_issues.is_empty());
    assert!(!report.final_executable_blocked_present);
    assert_eq!(report.final_executable_blocked_valid, None);
    assert_eq!(report.final_executable_blocked_emitted, None);
    assert_eq!(report.final_executable_blocked_plan_hash, None);
    assert_eq!(report.final_executable_blocked_blocker_count, None);
    assert!(report.final_executable_blocked_issues.is_empty());
    assert!(report_json.contains("\"container_section_issues\":[]"));
    assert!(report_json.contains("\"container_loader_symbol_issues\":[]"));
    assert!(report_json.contains("\"container_relocation_issues\":[]"));
    assert!(report_json.contains("\"container_compatibility_domain_issues\":[]"));
    assert!(report_json.contains("\"container_external_import_issues\":[]"));
    assert!(report_json.contains("\"closure_snapshot_present\":true"));
    assert!(report_json.contains("\"closure_snapshot_valid\":true"));
    assert!(report_json.contains("\"closure_snapshot_issues\":[]"));
    assert!(report_json.contains("\"closure_snapshot_linker_contract_hash\":\"0x"));
    assert!(report_json.contains("\"closure_snapshot_container_hash\":\"0x"));
    assert!(report_json.contains("\"closure_snapshot_payload_size_bytes\":"));
    assert!(report_json.contains("\"closure_snapshot_payload_hash\":\"0x"));
    assert!(report_json.contains("\"final_stage_plan_present\":true"));
    assert!(report_json.contains("\"final_stage_plan_valid\":true"));
    assert!(report_json.contains("\"final_stage_plan_ready\":false"));
    assert!(report_json.contains("\"final_stage_plan_hash\":\"0x"));
    assert!(report_json.contains("\"final_executable_blocked_present\":false"));
    assert!(report_json.contains("\"final_executable_blocked_valid\":null"));
    assert!(report_json.contains("\"object_plan_present\":true"));
    assert!(report_json.contains("\"object_plan_valid\":true"));
    assert!(report_json.contains("\"object_plan_issues\":[]"));
    assert!(report_json.contains("\"object_writer_input_present\":true"));
    assert!(report_json.contains("\"object_writer_input_valid\":true"));
    assert!(report_json.contains("\"object_byte_layout_present\":true"));
    assert!(report_json.contains("\"object_byte_layout_valid\":true"));
    assert!(report_json.contains("\"object_file_layout_present\":true"));
    assert!(report_json.contains("\"object_file_layout_valid\":true"));
    assert!(report_json.contains("\"object_image_dry_run_present\":true"));
    assert!(report_json.contains("\"object_image_dry_run_valid\":true"));
    assert!(report_json.contains("\"object_image_relocation_lowering_valid\":true"));
    assert!(report_json.contains("\"object_image_relocation_lowering_rule_count\":4"));
    assert!(report_json.contains("\"object_image_relocation_lowering_rules\":[{"));
    assert!(report_json.contains("\"source_seed_kind\":\"bootstrap-entry-seed\""));
    assert!(report_json.contains("\"object_image_relocation_lowering_issues\":[]"));
    assert!(report_json.contains("\"object_image_relocation_record_count\":4"));
    assert!(report_json.contains("\"object_image_relocation_record_table_hash\":\"0x"));
    assert!(report_json.contains("\"object_image_relocation_records\":[{"));
    assert!(report_json.contains("\"relocation_seed_id\":\"orel0000.compiled_artifact\""));
    assert!(report_json.contains("\"object_image_dry_run_bytes_present\":true"));
    assert!(report_json.contains("\"object_emit_blocked_present\":true"));
    assert!(report_json.contains("\"object_emit_blocked_valid\":true"));
    assert!(report_json.contains("\"object_output_present\":true"));
    assert!(report_json.contains("\"object_output_valid\":true"));
    assert!(report_json.contains("\"object_output_expected_size_bytes\":"));
    assert!(report_json.contains("\"object_output_actual_size_bytes\":"));
    assert!(report_json.contains("\"object_output_expected_hash\":\"0x"));
    assert!(report_json.contains("\"object_output_actual_hash\":\"0x"));
    assert!(report_json.contains("\"object_writer_dry_run_present\":true"));
    assert!(report_json.contains("\"object_writer_dry_run_valid\":true"));
    assert_eq!(
        report.container_loader_readiness.as_deref(),
        Some("host-assisted")
    );
    assert!(report
        .container_metadata_table_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("0x")));
    assert_eq!(report.container_compatibility_domain_count, Some(1));
    assert!(report
        .container_compatibility_domain_table_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("0x")));
    assert_eq!(
        report.container_compatibility_domain_id.as_deref(),
        Some("compat0000.cffi-von-neumann")
    );
    assert_eq!(
        report.container_compatibility_domain_kind.as_deref(),
        Some("cffi-host-compat")
    );
    assert_eq!(
        report.container_compatibility_domain_paradigm.as_deref(),
        Some("classic-von-neumann-host")
    );
    assert_eq!(
        report
            .container_compatibility_domain_lifecycle_hook
            .as_deref(),
        Some("on_cffi_native_object")
    );
    assert_eq!(
        report.container_compatibility_domain_abi_family.as_deref(),
        Some("mach-o")
    );
    assert_eq!(
        report
            .container_compatibility_domain_wrapper_policy
            .as_deref(),
        Some("wrapped")
    );
    assert_eq!(report.container_compatibility_domain_required, Some(true));
    assert_eq!(report.container_external_import_count, Some(3));
    assert!(report.container_native_object_section_present);
    assert_eq!(
        report.container_native_object_section_id.as_deref(),
        Some("sec0004.native-object-output")
    );
    assert!(report.container_native_object_loader_symbol_present);
    assert_eq!(
        report.container_native_object_loader_symbol_id.as_deref(),
        Some("sym0001.native-object-output")
    );
    assert!(report.container_native_object_relocation_present);
    assert_eq!(
        report.container_native_object_relocation_id.as_deref(),
        Some("rel0001.native-object")
    );
    assert!(report_json.contains("\"container_native_object_section_present\":true"));
    assert!(report_json.contains("\"container_compatibility_domain_count\":1"));
    assert!(report_json.contains("\"container_compatibility_domain_table_hash\":\"0x"));
    assert!(report_json
        .contains("\"container_compatibility_domain_id\":\"compat0000.cffi-von-neumann\""));
    assert!(report_json.contains("\"container_compatibility_domain_kind\":\"cffi-host-compat\""));
    assert!(report_json
        .contains("\"container_compatibility_domain_paradigm\":\"classic-von-neumann-host\""));
    assert!(report_json
        .contains("\"container_compatibility_domain_lifecycle_hook\":\"on_cffi_native_object\""));
    assert!(report_json.contains("\"container_compatibility_domain_abi_family\":\"mach-o\""));
    assert!(report_json.contains("\"container_compatibility_domain_wrapper_policy\":\"wrapped\""));
    assert!(report_json.contains("\"container_compatibility_domain_required\":true"));
    assert!(report_json
        .contains("\"container_compatibility_domain_summary\":{\"count\":1,\"table_hash\":\"0x"));
    assert!(report_json
        .contains("\"container_native_object_section_id\":\"sec0004.native-object-output\""));
    assert!(report_json
        .contains("\"container_native_object_loader_symbol_id\":\"sym0001.native-object-output\""));
    assert!(
        report_json.contains("\"container_native_object_relocation_id\":\"rel0001.native-object\"")
    );
    assert!(report
        .container_loader_blockers
        .iter()
        .any(|blocker| blocker == "external-import:final-stage-driver:clang"));
    assert!(report
        .issues
        .iter()
        .all(|issue| !issue.contains("container loader readiness is blocked")));
}

#[test]
fn check_reports_tampered_object_output() {
    let dir = env::temp_dir().join(format!(
        "nsld-check-object-output-drift-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    fs::write(dir.join("nuis.nsld.mach-o"), b"damaged-object").unwrap();
    let report = nsld_check_report(Path::new("manifest.toml"), &plan);
    fs::remove_dir_all(dir).unwrap();

    assert!(!report.valid);
    assert!(report.object_output_present);
    assert_eq!(report.object_output_valid, Some(false));
    assert_ne!(
        report.object_output_expected_size_bytes,
        report.object_output_actual_size_bytes
    );
    assert_ne!(
        report.object_output_expected_hash,
        report.object_output_actual_hash
    );
    assert!(report
        .object_output_issues
        .iter()
        .any(|issue| issue.contains("object_output_hash mismatch")));
    assert!(report
        .issues
        .iter()
        .any(|issue| issue == "object output verification failed"));
}

#[test]
fn check_reports_tampered_closure_snapshot() {
    let dir = env::temp_dir().join(format!(
        "nsld-check-closure-snapshot-drift-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    let prepare = nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let snapshot_path = Path::new(&prepare.closure_snapshot_path);
    let snapshot = fs::read_to_string(snapshot_path).unwrap();
    fs::write(
        snapshot_path,
        snapshot.replace(
            "linker_contract_hash = \"",
            "linker_contract_hash = \"0x0000000000000000",
        ),
    )
    .unwrap();
    let report = nsld_check_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::check_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(!report.valid);
    assert!(report.closure_snapshot_present);
    assert_eq!(report.closure_snapshot_valid, Some(false));
    assert!(report.closure_snapshot_issues.iter().any(|issue| {
        issue.starts_with("linker_contract_hash mismatch: expected 0x")
            && issue.contains("found 0x0000000000000000")
    }));
    assert!(report
        .issues
        .iter()
        .any(|issue| issue == "closure snapshot verification failed"));
    assert!(report_json.contains("\"closure_snapshot_present\":true"));
    assert!(report_json.contains("\"closure_snapshot_valid\":false"));
    assert!(report_json.contains("\"closure_snapshot_container_hash\":\"0x"));
    assert!(report_json.contains("\"closure_snapshot_payload_hash\":\"0x"));
    assert!(report_json.contains("linker_contract_hash mismatch: expected 0x"));
}

#[test]
fn check_reports_tampered_final_stage_plan() {
    let dir = env::temp_dir().join(format!(
        "nsld-check-final-stage-plan-drift-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    let prepare = nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let final_stage_plan_path = Path::new(&prepare.final_stage_plan_path);
    let final_stage_plan = fs::read_to_string(final_stage_plan_path).unwrap();
    fs::write(
        final_stage_plan_path,
        final_stage_plan.replace("plan_hash = \"", "plan_hash = \"0x3333333333333333"),
    )
    .unwrap();
    let report = nsld_check_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::check_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(!report.valid);
    assert!(report.final_stage_plan_present);
    assert_eq!(report.final_stage_plan_valid, Some(false));
    assert!(report.final_stage_plan_issues.iter().any(|issue| {
        issue.starts_with("plan_hash mismatch: expected 0x")
            && issue.contains("found 0x3333333333333333")
    }));
    assert!(report
        .issues
        .iter()
        .any(|issue| issue == "final-stage plan verification failed"));
    assert!(report_json.contains("\"final_stage_plan_present\":true"));
    assert!(report_json.contains("\"final_stage_plan_valid\":false"));
    assert!(report_json.contains("plan_hash mismatch: expected 0x"));
}

#[test]
fn check_reports_valid_final_executable_writer_input_when_present() {
    let dir = env::temp_dir().join(format!(
        "nsld-check-final-executable-writer-input-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit =
        nsld_emit_final_executable_writer_input_report(Path::new("manifest.toml"), &plan).unwrap();
    let report = nsld_check_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::check_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(report.valid, "{:?}", report.issues);
    assert!(report.final_executable_writer_input_present);
    assert_eq!(report.final_executable_writer_input_valid, Some(true));
    assert_eq!(
        report.final_executable_writer_input_hash.as_deref(),
        Some(emit.writer_input_hash.as_str())
    );
    assert_eq!(
        report.final_executable_writer_input_command_arg_count,
        Some(emit.command_arg_count)
    );
    assert!(report.final_executable_writer_input_issues.is_empty());
    assert!(report_json.contains("\"final_executable_writer_input_present\":true"));
    assert!(report_json.contains("\"final_executable_writer_input_valid\":true"));
    assert!(report_json.contains("\"final_executable_writer_input_hash\":\"0x"));
    assert!(report_json.contains("\"final_executable_writer_input_command_arg_count\":"));
}

#[test]
fn check_reports_tampered_final_executable_writer_input() {
    let dir = env::temp_dir().join(format!(
        "nsld-check-final-executable-writer-input-drift-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit =
        nsld_emit_final_executable_writer_input_report(Path::new("manifest.toml"), &plan).unwrap();
    let writer_input_source = fs::read_to_string(&emit.output_path).unwrap();
    fs::write(
        &emit.output_path,
        writer_input_source.replace(
            &format!("command_arg_count = {}", emit.command_arg_count),
            "command_arg_count = 999",
        ),
    )
    .unwrap();
    let report = nsld_check_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::check_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(!report.valid);
    assert!(report.final_executable_writer_input_present);
    assert_eq!(report.final_executable_writer_input_valid, Some(false));
    assert_eq!(
        report.final_executable_writer_input_command_arg_count,
        Some(999)
    );
    assert!(report
        .final_executable_writer_input_issues
        .iter()
        .any(|issue| issue.starts_with("command_arg_count mismatch: expected ")));
    assert!(report
        .issues
        .iter()
        .any(|issue| issue == "final executable writer input verification failed"));
    assert!(report_json.contains("\"final_executable_writer_input_present\":true"));
    assert!(report_json.contains("\"final_executable_writer_input_valid\":false"));
    assert!(report_json.contains("\"final_executable_writer_input_command_arg_count\":999"));
}

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
        invoke_plan_source.replace("would_invoke = false", "would_invoke = true"),
    )
    .unwrap();
    let report = nsld_check_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::check_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(!report.valid);
    assert!(report.final_executable_host_invoke_plan_present);
    assert_eq!(report.final_executable_host_invoke_plan_valid, Some(false));
    assert_eq!(
        report.final_executable_host_invoke_plan_would_invoke,
        Some(true)
    );
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
    assert!(report_json.contains("\"final_executable_host_invoke_plan_would_invoke\":true"));
}

#[test]
fn check_reports_valid_final_executable_layout_plan_when_present() {
    let dir = env::temp_dir().join(format!(
        "nsld-check-final-executable-layout-plan-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit =
        nsld_emit_final_executable_layout_plan_report(Path::new("manifest.toml"), &plan).unwrap();
    let report = nsld_check_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::check_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(report.valid, "{:?}", report.issues);
    assert!(report.final_executable_layout_plan_present);
    assert_eq!(report.final_executable_layout_plan_valid, Some(true));
    assert_eq!(
        report.final_executable_layout_plan_hash.as_deref(),
        Some(emit.layout_hash.as_str())
    );
    assert_eq!(
        report.final_executable_layout_plan_payload_count,
        Some(emit.payload_count)
    );
    assert!(report.final_executable_layout_plan_issues.is_empty());
    assert!(report_json.contains("\"final_executable_layout_plan_present\":true"));
    assert!(report_json.contains("\"final_executable_layout_plan_valid\":true"));
    assert!(report_json.contains("\"final_executable_layout_plan_hash\":\"0x"));
}

#[test]
fn check_reports_tampered_final_executable_layout_plan() {
    let dir = env::temp_dir().join(format!(
        "nsld-check-final-executable-layout-plan-drift-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit =
        nsld_emit_final_executable_layout_plan_report(Path::new("manifest.toml"), &plan).unwrap();
    let source = fs::read_to_string(&emit.output_path).unwrap();
    fs::write(
        &emit.output_path,
        source.replace(
            "lifecycle_entry_hook = \"on_process_start\"",
            "lifecycle_entry_hook = \"drift\"",
        ),
    )
    .unwrap();
    let report = nsld_check_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::check_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(!report.valid);
    assert!(report.final_executable_layout_plan_present);
    assert_eq!(report.final_executable_layout_plan_valid, Some(false));
    assert!(report
        .final_executable_layout_plan_issues
        .iter()
        .any(|issue| issue
            == "lifecycle_entry_hook mismatch: expected on_process_start, found drift"));
    assert!(report
        .issues
        .iter()
        .any(|issue| issue == "final executable layout plan verification failed"));
    assert!(report_json.contains("\"final_executable_layout_plan_present\":true"));
    assert!(report_json.contains("\"final_executable_layout_plan_valid\":false"));
}

#[test]
fn check_reports_valid_final_executable_image_dry_run_when_present() {
    let dir = env::temp_dir().join(format!(
        "nsld-check-final-executable-image-dry-run-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit =
        nsld_emit_final_executable_image_dry_run_report(Path::new("manifest.toml"), &plan).unwrap();
    let report = nsld_check_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::check_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(report.valid, "{:?}", report.issues);
    assert!(report.final_executable_image_dry_run_present);
    assert_eq!(report.final_executable_image_dry_run_valid, Some(true));
    assert_eq!(
        report.final_executable_image_dry_run_hash.as_deref(),
        emit.image_hash.as_deref()
    );
    assert_eq!(
        report.final_executable_image_dry_run_size_bytes,
        emit.image_size_bytes
    );
    assert!(report.final_executable_image_dry_run_issues.is_empty());
    assert!(report_json.contains("\"final_executable_image_dry_run_present\":true"));
    assert!(report_json.contains("\"final_executable_image_dry_run_valid\":true"));
    assert!(report_json.contains("\"final_executable_image_dry_run_hash\":\"0x"));
    assert!(report_json.contains("\"final_executable_image_dry_run_size_bytes\":"));
}

#[test]
fn check_reports_tampered_final_executable_image_dry_run_bytes() {
    let dir = env::temp_dir().join(format!(
        "nsld-check-final-executable-image-dry-run-drift-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit =
        nsld_emit_final_executable_image_dry_run_report(Path::new("manifest.toml"), &plan).unwrap();
    fs::write(&emit.image_path, b"drifted-final-image").unwrap();
    let report = nsld_check_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::check_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(!report.valid);
    assert!(report.final_executable_image_dry_run_present);
    assert_eq!(report.final_executable_image_dry_run_valid, Some(false));
    assert_eq!(
        report.final_executable_image_dry_run_hash.as_deref(),
        emit.image_hash.as_deref()
    );
    assert!(report
        .final_executable_image_dry_run_issues
        .iter()
        .any(|issue| issue.starts_with("image_bytes_hash mismatch: expected 0x")));
    assert!(report
        .issues
        .iter()
        .any(|issue| issue == "final executable image dry-run verification failed"));
    assert!(report_json.contains("\"final_executable_image_dry_run_present\":true"));
    assert!(report_json.contains("\"final_executable_image_dry_run_valid\":false"));
    assert!(report_json.contains("image_bytes_hash mismatch: expected 0x"));
}

#[test]
fn check_reports_valid_final_executable_blocked_artifact_when_present() {
    let dir = env::temp_dir().join(format!(
        "nsld-check-final-executable-blocked-{}",
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
    let report = nsld_check_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::check_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(report.valid, "{:?}", report.issues);
    assert!(report.final_executable_blocked_present);
    assert_eq!(report.final_executable_blocked_valid, Some(true));
    assert_eq!(report.final_executable_blocked_emitted, Some(false));
    assert_eq!(
        report.final_executable_blocked_plan_hash.as_deref(),
        Some(emit.final_stage_plan_hash.as_str())
    );
    assert_eq!(
        report.final_executable_blocked_blocker_count,
        Some(emit.blockers.len())
    );
    assert!(report.final_executable_blocked_issues.is_empty());
    assert!(report_json.contains("\"final_executable_blocked_present\":true"));
    assert!(report_json.contains("\"final_executable_blocked_valid\":true"));
    assert!(report_json.contains("\"final_executable_blocked_emitted\":false"));
    assert!(report_json.contains("\"final_executable_blocked_plan_hash\":\"0x"));
}

#[test]
fn check_reports_tampered_final_executable_blocked_artifact() {
    let dir = env::temp_dir().join(format!(
        "nsld-check-final-executable-blocked-drift-{}",
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
    let blocked_path = Path::new(&emit.blocked_report_path);
    let blocked_source = fs::read_to_string(blocked_path).unwrap();
    fs::write(
        blocked_path,
        blocked_source.replace(
            &format!("final_stage_plan_hash = \"{}\"", emit.final_stage_plan_hash),
            "final_stage_plan_hash = \"0x4444444444444444\"",
        ),
    )
    .unwrap();
    let report = nsld_check_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::check_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(!report.valid);
    assert!(report.final_executable_blocked_present);
    assert_eq!(report.final_executable_blocked_valid, Some(false));
    assert_eq!(
        report.final_executable_blocked_plan_hash.as_deref(),
        Some("0x4444444444444444")
    );
    assert!(report
        .final_executable_blocked_issues
        .iter()
        .any(|issue| issue.starts_with("final_stage_plan_hash mismatch: expected 0x")));
    assert!(report
        .issues
        .iter()
        .any(|issue| issue == "final executable blocked report verification failed"));
    assert!(report_json.contains("\"final_executable_blocked_present\":true"));
    assert!(report_json.contains("\"final_executable_blocked_valid\":false"));
    assert!(report_json.contains("\"final_executable_blocked_plan_hash\":\"0x4444444444444444\""));
}
