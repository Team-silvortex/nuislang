use super::{main_test_support::empty_link_plan, nsld_check_report, nsld_prepare_report};
use std::{env, fs, path::Path};

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
