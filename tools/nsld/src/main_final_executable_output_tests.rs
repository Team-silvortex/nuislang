use super::{
    fnv1a64_hex, main_test_support::empty_link_plan, nsld_check_report,
    nsld_emit_final_executable_image_dry_run_report, nsld_emit_final_executable_layout_plan_report,
    nsld_emit_final_executable_report, nsld_emit_final_executable_writer_input_report,
    nsld_emit_final_stage_plan_report, nsld_final_executable_layout_plan_report,
    nsld_final_executable_output_report, nsld_prepare_report,
    nsld_verify_final_executable_emit_report,
};
use std::{env, fs, path::Path};

#[test]
fn final_executable_output_reports_missing_until_real_output_exists() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-output-missing-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();
    plan.final_stage.output_path = dir.join("nuis-app").display().to_string();

    nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_stage_plan_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_executable_report(Path::new("manifest.toml"), &plan).unwrap();
    let report = nsld_final_executable_output_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::nsld_final_executable_output_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(!report.present);
    assert_eq!(report.boundary_status, "missing");
    assert_eq!(report.materialization_status, "blocked");
    assert_eq!(
        report.execution_handoff_contract,
        "nsld-final-output-handoff-v1"
    );
    assert!(!report.execution_handoff_ready);
    assert_eq!(report.execution_handoff_status, "blocked");
    assert_eq!(report.execution_handoff_target, "none");
    assert_eq!(report.execution_handoff_evidence_status, "blocked");
    assert_eq!(
        report.execution_handoff_first_blocker.as_deref(),
        Some("final-executable-output:missing")
    );
    assert_eq!(
        report.execution_handoff_decision_code,
        "emit-final-executable"
    );
    assert_eq!(
        report.recommended_next_action,
        "emit-final-executable-pipeline"
    );
    assert_eq!(
        report.final_output_nsdb_replay_next_action,
        "resolve-final-output-nsdb-replay"
    );
    assert_eq!(
        report.owned_package_summary_contract,
        "nsld-owned-package-summary-v1"
    );
    assert_eq!(report.owned_package_summary_status, "replay-blocked");
    assert!(!report.owned_package_summary_ready);
    assert_eq!(report.owned_package_summary_replay_status, "blocked");
    assert!(!report.owned_package_summary_replay_ready);
    assert_eq!(
        report.owned_package_summary_next_action,
        "resolve-final-output-nsdb-replay"
    );
    assert!(report
        .final_output_nsdb_replay_next_command
        .as_deref()
        .is_some_and(|command| command == "nsld final-executable-output manifest.toml --json"));
    assert!(report
        .owned_package_summary_next_command
        .as_deref()
        .is_some_and(|command| command == "nsld final-executable-output manifest.toml --json"));
    assert_eq!(
        report.object_package_summary_contract,
        "nsld-object-package-summary-v1"
    );
    assert_eq!(report.object_package_summary_status, "replay-blocked");
    assert!(!report.object_package_summary_ready);
    assert_eq!(report.object_package_summary_replay_status, "blocked");
    assert!(!report.object_package_summary_replay_ready);
    assert_eq!(
        report.object_package_summary_next_action,
        "resolve-final-output-nsdb-replay"
    );
    assert!(report
        .object_package_summary_next_command
        .as_deref()
        .is_some_and(|command| command == "nsld final-executable-output manifest.toml --json"));
    assert!(!report.runnable_candidate);
    assert!(!report.matches_expected_image);
    assert!(!report.output_image_header_valid);
    assert_eq!(report.output_image_magic, None);
    assert_eq!(report.output_image_version, None);
    assert!(report
        .blockers
        .iter()
        .any(|blocker| blocker == "final-executable-output:missing"));
    assert!(report
        .blockers
        .iter()
        .any(|blocker| blocker == "final-executable-emit:not-emitted"));
    assert!(report_json.contains("\"kind\":\"nsld_final_executable_output\""));
    assert!(report_json.contains("\"boundary_status\":\"missing\""));
    assert!(report_json.contains("\"materialization_status\":\"blocked\""));
    assert!(report_json.contains("\"execution_handoff_contract\":\"nsld-final-output-handoff-v1\""));
    assert!(report_json.contains("\"execution_handoff_ready\":false"));
    assert!(report_json.contains("\"execution_handoff_status\":\"blocked\""));
    assert!(report_json.contains("\"execution_handoff_target\":\"none\""));
    assert!(report_json.contains("\"execution_handoff_evidence_status\":\"blocked\""));
    assert!(report_json
        .contains("\"execution_handoff_first_blocker\":\"final-executable-output:missing\""));
    assert!(report_json.contains("\"execution_handoff_decision_code\":\"emit-final-executable\""));
    assert!(report_json.contains("\"recommended_next_action\":\"emit-final-executable-pipeline\""));
    assert!(report_json
        .contains("\"final_output_nsdb_replay_next_action\":\"resolve-final-output-nsdb-replay\""));
    assert!(report_json.contains(
        "\"final_output_nsdb_replay_next_command\":\"nsld final-executable-output manifest.toml --json\""
    ));
    assert!(report_json
        .contains("\"owned_package_summary_contract\":\"nsld-owned-package-summary-v1\""));
    assert!(report_json.contains("\"owned_package_summary_status\":\"replay-blocked\""));
    assert!(report_json.contains("\"owned_package_summary_ready\":false"));
    assert!(report_json.contains("\"owned_package_summary_replay_status\":\"blocked\""));
    assert!(report_json.contains("\"owned_package_summary_replay_ready\":false"));
    assert!(report_json
        .contains("\"owned_package_summary_next_action\":\"resolve-final-output-nsdb-replay\""));
    assert!(report_json.contains(
        "\"owned_package_summary_next_command\":\"nsld final-executable-output manifest.toml --json\""
    ));
    assert!(report_json
        .contains("\"object_package_summary_contract\":\"nsld-object-package-summary-v1\""));
    assert!(report_json.contains("\"object_package_summary_status\":\"replay-blocked\""));
    assert!(report_json.contains("\"object_package_summary_ready\":false"));
    assert!(report_json.contains("\"object_package_summary_replay_status\":\"blocked\""));
    assert!(report_json.contains("\"object_package_summary_replay_ready\":false"));
    assert!(report_json
        .contains("\"object_package_summary_next_action\":\"resolve-final-output-nsdb-replay\""));
    assert!(report_json.contains(
        "\"object_package_summary_next_command\":\"nsld final-executable-output manifest.toml --json\""
    ));
    assert!(report_json.contains("\"present\":false"));
    assert!(report_json.contains("\"output_image_header_valid\":false"));
    assert!(report_json.contains("\"output_image_magic\":null"));
    assert!(report_json.contains("\"matches_expected_image\":false"));
    assert!(report_json.contains("\"final_executable_emitted\":false"));
    assert!(report_json.contains("\"final_executable_blocker_count\":"));
    assert!(report_json.contains("\"runnable_candidate\":false"));
    assert!(report_json.contains("\"final-executable-output:missing\""));
    assert!(report_json.contains("\"final-executable-emit:not-emitted\""));
}

#[test]
fn self_contained_final_executable_emit_writes_nsld_owned_output() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-output-present-{}",
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
    let layout = nsld_final_executable_layout_plan_report(Path::new("manifest.toml"), &plan);
    nsld_emit_final_executable_layout_plan_report(Path::new("manifest.toml"), &plan).unwrap();
    nsld_emit_final_executable_image_dry_run_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit = nsld_emit_final_executable_report(Path::new("manifest.toml"), &plan).unwrap();
    let emit_json = super::json::nsld_final_executable_emit_report_json(&emit);
    let verify_emit = nsld_verify_final_executable_emit_report(Path::new("manifest.toml"), &plan);
    let output = nsld_final_executable_output_report(Path::new("manifest.toml"), &plan);
    let output_json = super::json::nsld_final_executable_output_report_json(&output);
    let image_bytes = fs::read(&emit.image_dry_run_bytes_path).unwrap();
    let output_bytes = fs::read(&plan.final_stage.output_path).unwrap();
    fs::remove_dir_all(dir).unwrap();

    assert!(emit.emitted);
    assert!(emit.can_emit_final_executable);
    assert_eq!(emit.writer_status, "ready");
    assert!(emit.final_output_checked);
    assert!(emit.final_output_present);
    assert_eq!(emit.final_output_size_bytes, Some(output_bytes.len()));
    assert_eq!(emit.final_output_hash, Some(fnv1a64_hex(&output_bytes)));
    assert_eq!(emit.final_output_image_header_valid, Some(true));
    assert_eq!(emit.final_output_runnable_candidate, Some(true));
    assert!(verify_emit.valid, "{:?}", verify_emit.issues);
    assert!(verify_emit.expected_final_output_checked);
    assert_eq!(verify_emit.actual_final_output_checked, Some(true));
    assert!(verify_emit.expected_final_output_present);
    assert_eq!(verify_emit.actual_final_output_present, Some(true));
    assert_eq!(
        verify_emit.expected_final_output_size_bytes,
        Some(output_bytes.len())
    );
    assert_eq!(
        verify_emit.actual_final_output_size_bytes,
        Some(output_bytes.len())
    );
    assert_eq!(
        verify_emit.expected_final_output_hash,
        Some(fnv1a64_hex(&output_bytes))
    );
    assert_eq!(
        verify_emit.actual_final_output_hash,
        Some(fnv1a64_hex(&output_bytes))
    );
    assert_eq!(
        verify_emit.expected_final_output_image_header_valid,
        Some(true)
    );
    assert_eq!(
        verify_emit.actual_final_output_image_header_valid,
        Some(true)
    );
    assert_eq!(
        verify_emit.expected_final_output_runnable_candidate,
        Some(true)
    );
    assert_eq!(
        verify_emit.actual_final_output_runnable_candidate,
        Some(true)
    );
    assert!(output.present);
    assert_eq!(output.boundary_status, "ready");
    assert_eq!(output.materialization_status, "self-contained-image-ready");
    assert_eq!(
        output.execution_handoff_contract,
        "nsld-final-output-handoff-v1"
    );
    assert!(output.execution_handoff_ready);
    assert_eq!(
        output.execution_handoff_status,
        "container-loader-handoff-ready"
    );
    assert_eq!(output.execution_handoff_target, "container-loader");
    assert_eq!(
        output.execution_handoff_evidence_status,
        "container-loader-handoff-ready"
    );
    assert_eq!(output.execution_handoff_first_blocker, None);
    assert_eq!(
        output.execution_handoff_decision_code,
        "handoff-container-loader-first-payload"
    );
    assert_eq!(
        output.entrypoint_materialization_evidence_status,
        "launcher-evidence-missing"
    );
    assert!(!output.launcher_manifest_present);
    assert_eq!(output.launcher_manifest_ready, None);
    assert_eq!(output.launcher_manifest_blocker_count, None);
    assert!(!output.launcher_dry_run_present);
    assert_eq!(output.launcher_dry_run_ready, None);
    assert_eq!(output.launcher_dry_run_would_enter_lifecycle_hook, None);
    assert_eq!(output.launcher_dry_run_blocker_count, None);
    assert_eq!(output.container_loader_status, "parsed");
    assert_eq!(
        output.container_loader_payload_scan_kind,
        "nsld-container-toml"
    );
    assert!(output.container_loader_parsed);
    assert_eq!(
        output.container_loader_readiness.as_deref(),
        Some("host-assisted")
    );
    assert_eq!(output.container_loader_ready, Some(true));
    assert_eq!(output.container_loader_handoff_status, "ready");
    assert!(output.container_loader_handoff_ready);
    assert_eq!(output.container_loader_handoff_first_blocker, None);
    assert_eq!(
        output.container_loader_entry_symbol.as_deref(),
        Some("main")
    );
    assert_eq!(
        output.container_loader_entry_kind.as_deref(),
        Some("lifecycle-bootstrap")
    );
    assert_eq!(
        output.container_loader_entry_section_id.as_deref(),
        Some("sec0000.compiled-artifact")
    );
    assert_eq!(output.container_loader_symbol_count, Some(2));
    assert_eq!(output.first_payload_execution_status, "ready");
    assert!(output.first_payload_execution_ready);
    assert_eq!(output.first_payload_execution_target, "container-loader");
    assert_eq!(
        output.first_payload_execution_entry_symbol.as_deref(),
        Some("main")
    );
    assert_eq!(
        output.first_payload_execution_entry_kind.as_deref(),
        Some("lifecycle-bootstrap")
    );
    assert_eq!(
        output.first_payload_execution_entry_section_id.as_deref(),
        Some("sec0000.compiled-artifact")
    );
    assert_eq!(output.first_payload_execution_first_blocker, None);
    assert_eq!(
        output.recommended_next_action,
        "handoff-to-container-loader"
    );
    assert!(output.path_present);
    assert!(output.nsld_owned_output);
    assert!(output.runnable_candidate, "{:?}", output.blockers);
    assert!(output.matches_expected_image, "{:?}", output.issues);
    assert!(output.matches_verified_patched_image, "{:?}", output.issues);
    assert_eq!(output.size_bytes, Some(output_bytes.len()));
    assert_eq!(output.output_hash, Some(fnv1a64_hex(&output_bytes)));
    assert!(output.output_image_header_required);
    assert!(output.output_image_header_valid);
    assert_eq!(output.output_image_magic.as_deref(), Some("NUIFIMG"));
    assert_eq!(output.output_image_version, Some(1));
    assert_eq!(output.output_image_header_size, Some(64));
    assert_eq!(output.output_payload_byte_offset, Some(64));
    assert_eq!(
        output.output_layout_hash.as_deref(),
        Some(layout.layout_hash.as_str())
    );
    assert_eq!(
        output.output_byte_map_hash.as_deref(),
        Some(layout.byte_map_hash.as_str())
    );
    assert_eq!(
        output.scheduler_metadata_payload_id.as_deref(),
        Some("payload0004.scheduler-metadata")
    );
    assert_eq!(output.scheduler_metadata_present, Some(true));
    assert_eq!(
        output.scheduler_metadata_offset,
        layout
            .byte_map_entries
            .iter()
            .find(|entry| entry.payload_id == "payload0004.scheduler-metadata")
            .map(|entry| entry.offset)
    );
    assert_eq!(
        output.scheduler_metadata_hash,
        layout
            .payloads
            .iter()
            .find(|payload| payload.payload_id == "payload0004.scheduler-metadata")
            .map(|payload| payload.content_hash.clone())
    );
    assert_eq!(output.expected_image_size_bytes, Some(image_bytes.len()));
    assert_eq!(output.expected_image_hash, Some(fnv1a64_hex(&image_bytes)));
    assert_eq!(
        output.expected_image_resolver_status.as_deref(),
        Some("resolved")
    );
    assert_eq!(
        output.expected_image_patch_application_status.as_deref(),
        Some("applied")
    );
    assert_eq!(
        output.expected_image_patch_byte_audit_status.as_deref(),
        Some("verified")
    );
    assert!(output
        .expected_image_patch_byte_audit_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("0x")));
    assert_eq!(image_bytes, output_bytes);
    assert!(output_json.contains("\"present\":true"));
    assert!(output_json.contains("\"boundary_status\":\"ready\""));
    assert!(output_json.contains("\"materialization_status\":\"self-contained-image-ready\""));
    assert!(output_json.contains("\"execution_handoff_contract\":\"nsld-final-output-handoff-v1\""));
    assert!(output_json.contains("\"execution_handoff_ready\":true"));
    assert!(output_json.contains("\"execution_handoff_status\":\"container-loader-handoff-ready\""));
    assert!(output_json.contains("\"execution_handoff_target\":\"container-loader\""));
    assert!(output_json
        .contains("\"execution_handoff_evidence_status\":\"container-loader-handoff-ready\""));
    assert!(output_json.contains("\"execution_handoff_first_blocker\":null"));
    assert!(output_json.contains(
        "\"execution_handoff_decision_code\":\"handoff-container-loader-first-payload\""
    ));
    assert!(output_json
        .contains("\"entrypoint_materialization_evidence_status\":\"launcher-evidence-missing\""));
    assert!(output_json.contains("\"launcher_manifest_present\":false"));
    assert!(output_json.contains("\"launcher_manifest_ready\":null"));
    assert!(output_json.contains("\"launcher_dry_run_present\":false"));
    assert!(output_json.contains("\"launcher_dry_run_ready\":null"));
    assert!(output_json.contains("\"container_loader_status\":\"parsed\""));
    assert!(output_json.contains("\"container_loader_payload_scan_kind\":\"nsld-container-toml\""));
    assert!(output_json.contains("\"container_loader_parsed\":true"));
    assert!(output_json.contains("\"container_loader_readiness\":\"host-assisted\""));
    assert!(output_json.contains("\"container_loader_ready\":true"));
    assert!(output_json.contains("\"container_loader_handoff_status\":\"ready\""));
    assert!(output_json.contains("\"container_loader_handoff_ready\":true"));
    assert!(output_json.contains("\"container_loader_handoff_first_blocker\":null"));
    assert!(output_json.contains("\"container_loader_entry_symbol\":\"main\""));
    assert!(output_json.contains("\"container_loader_entry_kind\":\"lifecycle-bootstrap\""));
    assert!(
        output_json.contains("\"container_loader_entry_section_id\":\"sec0000.compiled-artifact\"")
    );
    assert!(output_json.contains("\"container_loader_symbol_count\":2"));
    assert!(output_json.contains("\"first_payload_execution_status\":\"ready\""));
    assert!(output_json.contains("\"first_payload_execution_ready\":true"));
    assert!(output_json.contains("\"first_payload_execution_target\":\"container-loader\""));
    assert!(output_json.contains("\"first_payload_execution_entry_symbol\":\"main\""));
    assert!(output_json.contains("\"first_payload_execution_entry_kind\":\"lifecycle-bootstrap\""));
    assert!(output_json
        .contains("\"first_payload_execution_entry_section_id\":\"sec0000.compiled-artifact\""));
    assert!(output_json.contains("\"first_payload_execution_first_blocker\":null"));
    assert!(output_json
        .contains("\"payload_execution_trace_protocol\":\"nsdb-yir-payload-execution-trace-v1\""));
    assert!(output_json.contains("\"payload_execution_trace_available\":true"));
    assert!(output_json.contains("\"payload_execution_trace_record_count\":1"));
    assert!(output_json.contains("\"payload_execution_trace_ready_record_count\":1"));
    assert!(output_json.contains("\"payload_execution_trace_records\":[{"));
    assert!(output_json.contains("\"trace_id\":\"payload-trace:container-loader:main\""));
    assert!(output_json.contains("\"execution_phase\":\"container-loader-handoff\""));
    assert!(output_json.contains("\"target\":\"container-loader\""));
    assert!(output_json.contains("\"entry_symbol\":\"main\""));
    assert!(output_json.contains("\"entry_kind\":\"lifecycle-bootstrap\""));
    assert!(output_json.contains("\"entry_section_id\":\"sec0000.compiled-artifact\""));
    assert!(output_json.contains("\"next_action\":\"handoff-payload-trace-to-nsdb\""));
    assert!(output_json.contains("\"recommended_next_action\":\"handoff-to-container-loader\""));
    assert!(output_json.contains("\"path_present\":true"));
    assert!(output_json.contains("\"nsld_owned_output\":true"));
    assert!(output_json.contains("\"output_image_header_required\":true"));
    assert!(output_json.contains("\"output_image_header_valid\":true"));
    assert!(output_json.contains("\"output_image_magic\":\"NUIFIMG\""));
    assert!(output_json.contains("\"output_image_version\":1"));
    assert!(output_json.contains("\"output_layout_hash\":\"0x"));
    assert!(output_json.contains("\"output_byte_map_hash\":\"0x"));
    assert!(output_json
        .contains("\"scheduler_metadata_payload_id\":\"payload0004.scheduler-metadata\""));
    assert!(output_json.contains("\"scheduler_metadata_present\":true"));
    assert!(output_json.contains("\"scheduler_metadata_hash\":\"0x"));
    assert!(output_json.contains("\"matches_expected_image\":true"));
    assert!(output_json.contains("\"expected_image_resolver_status\":\"resolved\""));
    assert!(output_json.contains("\"expected_image_patch_application_status\":\"applied\""));
    assert!(output_json.contains("\"expected_image_patch_byte_audit_status\":\"verified\""));
    assert!(output_json.contains("\"expected_image_patch_byte_audit_hash\":\"0x"));
    assert!(output_json.contains("\"matches_verified_patched_image\":true"));
    assert!(output_json.contains("\"final_stage_plan_valid\":true"));
    assert!(output_json.contains("\"final_executable_emit_valid\":true"));
    assert!(output_json.contains("\"final_executable_emitted\":true"));
    assert!(
        output.object_output_valid,
        "{:?}",
        output.object_output_issues
    );
    assert!(output.object_output_path.ends_with("nuis.nsld.mach-o"));
    assert_eq!(
        output.object_output_expected_size_bytes,
        output.object_output_actual_size_bytes
    );
    assert_eq!(
        output.object_output_expected_hash,
        output.object_output_actual_hash
    );
    assert!(output.object_output_issues.is_empty());
    assert!(output_json.contains("\"object_output_valid\":true"));
    assert!(output_json.contains("\"object_output_path\":"));
    assert_eq!(output.object_output_family, "mach-o");
    assert_eq!(output.object_output_magic_status, "valid");
    assert_eq!(output.object_output_magic.as_deref(), Some("0xcffaedfe"));
    assert!(output_json.contains("\"object_output_family\":\"mach-o\""));
    assert!(output_json.contains("\"object_output_magic_status\":\"valid\""));
    assert!(output_json.contains("\"object_output_magic\":\"0xcffaedfe\""));
    assert!(output_json.contains("\"object_output_expected_hash\":\"0x"));
    assert!(output_json.contains("\"object_output_actual_hash\":\"0x"));
    assert!(output_json.contains("\"object_output_issues\":[]"));
    assert!(output_json
        .contains("\"object_package_summary_contract\":\"nsld-object-package-summary-v1\""));
    assert!(output_json.contains("\"object_package_summary_status\":"));
    assert!(output_json.contains("\"object_package_summary_next_action\":"));
    assert!(output_json.contains("\"runnable_candidate\":true"));
    assert!(output_json.contains("\"blockers\":[]"));
    assert!(output_json.contains("\"issues\":[]"));
    assert!(emit_json.contains("\"final_output_checked\":true"));
    assert!(emit_json.contains("\"final_output_present\":true"));
    assert!(emit_json.contains("\"final_output_hash\":\"0x"));
    assert!(emit_json.contains("\"final_output_image_header_valid\":true"));
    assert!(emit_json.contains("\"final_output_runnable_candidate\":true"));
}

#[test]
fn verify_final_executable_emit_reports_final_output_drift() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-output-drift-{}",
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
    let emit = nsld_emit_final_executable_report(Path::new("manifest.toml"), &plan).unwrap();
    let blocked_path = Path::new(&emit.blocked_report_path);
    let damaged = fs::read_to_string(blocked_path)
        .unwrap()
        .replace(
            "final_output_present = true",
            "final_output_present = false",
        )
        .replace(
            &format!(
                "final_output_size_bytes = {}",
                emit.final_output_size_bytes.unwrap()
            ),
            "final_output_size_bytes = 1",
        )
        .replace(
            &format!(
                "final_output_hash = \"{}\"",
                emit.final_output_hash.as_deref().unwrap()
            ),
            "final_output_hash = \"0xaaaaaaaaaaaaaaaa\"",
        )
        .replace(
            "final_output_image_header_valid = true",
            "final_output_image_header_valid = false",
        )
        .replace(
            "final_output_runnable_candidate = true",
            "final_output_runnable_candidate = false",
        );
    fs::write(blocked_path, damaged).unwrap();

    let verify = nsld_verify_final_executable_emit_report(Path::new("manifest.toml"), &plan);
    let verify_json = super::json::nsld_final_executable_emit_verify_report_json(&verify);
    fs::remove_dir_all(dir).unwrap();

    assert!(!verify.valid);
    assert_eq!(verify.expected_final_output_present, true);
    assert_eq!(verify.actual_final_output_present, Some(false));
    assert_eq!(verify.actual_final_output_size_bytes, Some(1));
    assert_eq!(
        verify.actual_final_output_hash.as_deref(),
        Some("0xaaaaaaaaaaaaaaaa")
    );
    assert_eq!(verify.actual_final_output_image_header_valid, Some(false));
    assert_eq!(verify.actual_final_output_runnable_candidate, Some(false));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue == "final_output_present mismatch: expected true, found false"));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("final_output_size_bytes mismatch")));
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("final_output_hash mismatch")));
    assert!(verify.issues.iter().any(|issue| {
        issue == "final_output_image_header_valid mismatch: expected true, found false"
    }));
    assert!(verify.issues.iter().any(|issue| {
        issue == "final_output_runnable_candidate mismatch: expected true, found false"
    }));
    assert!(verify_json.contains("\"actual_final_output_present\":false"));
    assert!(verify_json.contains("\"actual_final_output_size_bytes\":1"));
    assert!(verify_json.contains("\"actual_final_output_hash\":\"0xaaaaaaaaaaaaaaaa\""));
    assert!(verify_json.contains("\"actual_final_output_image_header_valid\":false"));
    assert!(verify_json.contains("\"actual_final_output_runnable_candidate\":false"));
}

#[test]
fn final_executable_output_rejects_tampered_output_bytes() {
    let dir = env::temp_dir().join(format!(
        "nsld-final-executable-output-tamper-{}",
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
    fs::write(&plan.final_stage.output_path, b"tampered-final-output").unwrap();

    let output = nsld_final_executable_output_report(Path::new("manifest.toml"), &plan);
    let output_json = super::json::nsld_final_executable_output_report_json(&output);
    let check = nsld_check_report(Path::new("manifest.toml"), &plan);
    fs::remove_dir_all(dir).unwrap();

    assert!(output.present);
    assert_eq!(output.boundary_status, "invalid");
    assert_eq!(output.materialization_status, "blocked");
    assert_eq!(
        output.execution_handoff_contract,
        "nsld-final-output-handoff-v1"
    );
    assert!(!output.execution_handoff_ready);
    assert_eq!(output.execution_handoff_status, "blocked");
    assert_eq!(output.execution_handoff_target, "none");
    assert_eq!(output.execution_handoff_evidence_status, "blocked");
    assert_eq!(
        output.execution_handoff_first_blocker.as_deref(),
        Some("final-executable-output:image-header-invalid")
    );
    assert_eq!(
        output.execution_handoff_decision_code,
        "inspect-output-diagnostics"
    );
    assert_eq!(
        output.recommended_next_action,
        "inspect-final-output-diagnostics"
    );
    assert!(!output.runnable_candidate);
    assert!(!output.matches_expected_image);
    assert!(!output.matches_verified_patched_image);
    assert!(!output.output_image_header_valid);
    assert!(output
        .blockers
        .iter()
        .any(|blocker| blocker == "final-executable-output:image-header-invalid"));
    assert!(output
        .blockers
        .iter()
        .any(|blocker| blocker == "final-executable-output:size-mismatch"));
    assert!(output
        .blockers
        .iter()
        .any(|blocker| blocker == "final-executable-output:hash-mismatch"));
    assert!(!check.valid);
    assert_eq!(
        check.final_executable_output_image_header_valid,
        Some(false)
    );
    assert_eq!(
        check.final_executable_output_runnable_candidate,
        Some(false)
    );
    assert!(!check.final_executable_output_matches_verified_patched_image);
    assert_eq!(check.final_executable_output_boundary_status, "invalid");
    assert_eq!(
        check.final_executable_output_materialization_status,
        "blocked"
    );
    assert_eq!(
        check.final_executable_output_execution_handoff_contract,
        "nsld-final-output-handoff-v1"
    );
    assert!(!check.final_executable_output_execution_handoff_ready);
    assert_eq!(
        check.final_executable_output_execution_handoff_status,
        "blocked"
    );
    assert_eq!(
        check.final_executable_output_execution_handoff_target,
        "none"
    );
    assert_eq!(
        check.final_executable_output_execution_handoff_evidence_status,
        "blocked"
    );
    assert_eq!(
        check
            .final_executable_output_execution_handoff_first_blocker
            .as_deref(),
        Some("final-executable-output:image-header-invalid")
    );
    assert_eq!(
        check.final_executable_output_execution_handoff_decision_code,
        "inspect-output-diagnostics"
    );
    assert_eq!(
        check.final_executable_output_recommended_next_action,
        "inspect-final-output-diagnostics"
    );
    assert!(check
        .final_executable_output_issues
        .iter()
        .any(|issue| issue.contains("image-header-invalid")));
    assert!(check
        .final_executable_output_issues
        .iter()
        .any(|issue| issue.contains("hash mismatch")));
    assert!(output_json.contains("\"present\":true"));
    assert!(output_json.contains("\"boundary_status\":\"invalid\""));
    assert!(output_json.contains("\"materialization_status\":\"blocked\""));
    assert!(output_json.contains("\"execution_handoff_contract\":\"nsld-final-output-handoff-v1\""));
    assert!(output_json.contains("\"execution_handoff_ready\":false"));
    assert!(output_json.contains("\"execution_handoff_status\":\"blocked\""));
    assert!(output_json.contains("\"execution_handoff_target\":\"none\""));
    assert!(output_json.contains("\"execution_handoff_evidence_status\":\"blocked\""));
    assert!(output_json.contains(
        "\"execution_handoff_first_blocker\":\"final-executable-output:image-header-invalid\""
    ));
    assert!(
        output_json.contains("\"execution_handoff_decision_code\":\"inspect-output-diagnostics\"")
    );
    assert!(
        output_json.contains("\"recommended_next_action\":\"inspect-final-output-diagnostics\"")
    );
    assert!(output_json.contains("\"output_image_header_valid\":false"));
    assert!(output_json.contains("\"matches_expected_image\":false"));
    assert!(output_json.contains("\"runnable_candidate\":false"));
    assert!(output_json.contains("\"final-executable-output:image-header-invalid\""));
    assert!(output_json.contains("\"final-executable-output:size-mismatch\""));
    assert!(output_json.contains("\"final-executable-output:hash-mismatch\""));
}
