use super::{
    main_test_support::empty_link_plan, nsld_closure_report, nsld_emit_closure_report,
    nsld_prepare_report, nsld_verify_closure_report,
};
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
    assert_eq!(report.lowering_plan_index_source, "unavailable");
    assert!(!report.lowering_plan_index_available);
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
    assert!(report_json.contains("\"lowering_plan_index_source\":\"unavailable\""));
    assert!(report_json.contains("\"lowering_plan_index_available\":false"));
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
fn closure_treats_compiled_artifact_section_lowering_index_as_contract() {
    let dir = env::temp_dir().join(format!(
        "nsld-closure-lowering-section-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let artifact_path = dir.join("nuis.compiled.artifact");
    fs::write(&artifact_path, b"compiled-artifact").unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = artifact_path.display().to_string();
    plan.lowering_plan_index_source = "compiled_artifact_section".to_owned();

    let report = nsld_closure_report(Path::new("manifest.toml"), &plan);
    let report_json = super::json::nsld_closure_report_json(&report);
    let emit = nsld_emit_closure_report(Path::new("manifest.toml"), &plan).unwrap();
    let snapshot = fs::read_to_string(&emit.output_path).unwrap();
    fs::remove_dir_all(dir).unwrap();

    assert_eq!(
        report.lowering_plan_index_source,
        "compiled_artifact_section"
    );
    assert!(report.lowering_plan_index_available);
    assert!(report
        .internal_contracts
        .iter()
        .any(|contract| contract == "lowering-plan-index"));
    assert!(report_json.contains("\"lowering_plan_index_source\":\"compiled_artifact_section\""));
    assert!(report_json.contains("\"lowering_plan_index_available\":true"));
    assert!(snapshot.contains("lowering_plan_index_source = \"compiled_artifact_section\""));
    assert!(snapshot.contains("lowering_plan_index_available = true"));
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
