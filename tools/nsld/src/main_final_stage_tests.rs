use super::{
    main_test_support::empty_link_plan, nsld_emit_final_stage_plan_report,
    nsld_final_stage_plan_report, nsld_prepare_report, nsld_verify_final_stage_plan_report, toml,
};
use std::{env, fs, path::Path};

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
    assert_eq!(report.input_count, 5);
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
    assert!(report_json.contains("\"input_count\":5"));
    assert!(report_json.contains("\"inputs\":[{"));
    assert!(report_json.contains("\"input_id\":\"fsi0002.closure-snapshot\""));
    assert!(report_json.contains("\"input_id\":\"fsi0004.scheduler-metadata\""));
    assert!(report_json.contains("\"container_hash\":\"0x"));
    assert!(report_json.contains("\"payload_hash\":\"0x"));
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
    let expected_report = nsld_final_stage_plan_report(Path::new("manifest.toml"), &plan);
    let blockers_line = format!(
        "blockers = [{}]",
        toml::toml_string_array_literal(&expected_report.blockers)
    );
    let notes_line = format!(
        "notes = [{}]",
        toml::toml_string_array_literal(&expected_report.notes)
    );
    let damaged = fs::read_to_string(plan_path)
        .unwrap()
        .replace(
            &format!("plan_hash = \"{}\"", emit.plan_hash),
            "plan_hash = \"0x2222222222222222\"",
        )
        .replace(
            &blockers_line,
            "blockers = [\"tampered-final-stage-blocker\"]",
        )
        .replace(&notes_line, "notes = [\"tampered-final-stage-note\"]")
        .replacen(
            "input_id = \"fsi0001.container-payload\"",
            "input_id = \"tampered-final-stage-input\"",
            1,
        )
        .replacen("[[final_stage_input]]", "[[final_stage_input_tampered]]", 1);
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
    assert!(damaged_verify
        .actual_input_ids
        .iter()
        .any(|input_id| input_id == "tampered-final-stage-input"));
    assert_eq!(
        damaged_verify.actual_input_entry_count + 1,
        damaged_verify.expected_input_entry_count
    );
    assert!(damaged_verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("input_ids mismatch")));
    assert!(damaged_verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("input_entry_count mismatch")));
    assert_eq!(
        damaged_verify.actual_blockers,
        vec!["tampered-final-stage-blocker".to_owned()]
    );
    assert_eq!(
        damaged_verify.actual_notes,
        vec!["tampered-final-stage-note".to_owned()]
    );
    assert!(damaged_verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("blockers mismatch")));
    assert!(damaged_verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("notes mismatch")));
    assert!(verify_json.contains("\"actual_plan_hash\":\"0x2222222222222222\""));
    assert!(verify_json.contains("tampered-final-stage-input"));
    assert!(verify_json.contains("tampered-final-stage-blocker"));
    assert!(verify_json.contains("tampered-final-stage-note"));
    assert!(verify_json.contains("\"actual_input_entry_count\":"));
}
