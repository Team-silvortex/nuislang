use crate::{
    main_test_support::empty_link_plan,
    object_image_dry_run::{
        nsld_emit_object_image_dry_run_report, nsld_object_image_dry_run_report,
        nsld_verify_object_image_dry_run_report,
    },
};
use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn reports_mach_o_image_hash_when_encoder_can_construct_image() {
    let plan = empty_link_plan();
    let report = nsld_object_image_dry_run_report(Path::new("manifest.toml"), &plan);

    assert_eq!(report.writer_backend_kind, "mach-o-arm64");
    assert_eq!(report.object_family, "mach-o");
    assert_eq!(report.backend_kind, "mach-o-arm64");
    assert_eq!(report.backend_family, "mach-o");
    assert_eq!(report.backend_status, "ready");
    assert!(report
        .backend_capabilities
        .iter()
        .all(|capability| capability.required && capability.status == "ready"));
    assert!(report.image_constructed);
    assert_eq!(report.image_size_bytes, Some(report.total_file_size_bytes));
    assert!(report.image_hash.as_deref().unwrap().starts_with("0x"));
    assert!(report.relocation_lowering_valid);
    assert_eq!(report.relocation_lowering_rule_count, 4);
    assert_eq!(report.relocation_lowering_rules.len(), 4);
    assert_eq!(
        report.relocation_lowering_rules[0].source_seed_kind,
        "bootstrap-entry-seed"
    );
    assert!(report.relocation_lowering_issues.is_empty());
    assert_eq!(report.relocation_record_count, 4);
    assert!(report.relocation_record_table_hash.starts_with("0x"));
    assert_eq!(report.relocation_records.len(), 4);
    assert_eq!(
        report.relocation_records[0].relocation_seed_id,
        "orel0000.compiled_artifact"
    );
    assert_eq!(report.relocation_records[0].symbol_index, 1);
    assert!(report
        .blockers
        .iter()
        .any(|blocker| blocker.starts_with("object-file-layout:")));
}

#[test]
fn object_image_dry_run_serializes_writer_identity() {
    let plan = empty_link_plan();
    let report = nsld_object_image_dry_run_report(Path::new("manifest.toml"), &plan);
    let rendered = crate::toml::render_object_image_dry_run(&report);
    let json = crate::json_object_image::nsld_object_image_dry_run_report_json(&report);

    assert!(rendered.contains("writer_backend_kind = \"mach-o-arm64\""));
    assert!(rendered.contains("object_family = \"mach-o\""));
    assert!(rendered.contains("[[backend_capability]]"));
    assert!(rendered.contains("capability_id = \"object-image-encoder\""));
    assert!(rendered.contains("status = \"ready\""));
    assert!(rendered.contains("relocation_lowering_valid = true"));
    assert!(rendered.contains("relocation_lowering_rule_count = 4"));
    assert!(rendered.contains("[[relocation_lowering_rule]]"));
    assert!(rendered.contains("source_seed_kind = \"bootstrap-entry-seed\""));
    assert!(rendered.contains("relocation_record_count = 4"));
    assert!(rendered.contains("relocation_record_table_hash = \"0x"));
    assert!(rendered.contains("[[relocation_record]]"));
    assert!(rendered.contains("relocation_seed_id = \"orel0000.compiled_artifact\""));
    assert!(rendered.contains("symbol_index = 1"));
    assert!(json.contains("\"writer_backend_kind\":\"mach-o-arm64\""));
    assert!(json.contains("\"object_family\":\"mach-o\""));
    assert!(json.contains("\"backend_capabilities\":[{"));
    assert!(json.contains("\"capability_id\":\"object-image-encoder\""));
    assert!(json.contains("\"relocation_lowering_valid\":true"));
    assert!(json.contains("\"relocation_lowering_rule_count\":4"));
    assert!(json.contains("\"relocation_lowering_issues\":[]"));
    assert!(json.contains("\"relocation_lowering_rules\":[{"));
    assert!(json.contains("\"source_seed_kind\":\"bootstrap-entry-seed\""));
    assert!(json.contains("\"target_relocation_kind\":\"arm64-unsigned-pointer\""));
    assert!(json.contains("\"relocation_record_count\":4"));
    assert!(json.contains("\"relocation_record_table_hash\":\"0x"));
    assert!(json.contains("\"relocation_records\":[{"));
    assert!(json.contains("\"relocation_seed_id\":\"orel0000.compiled_artifact\""));
    assert!(json.contains("\"symbol_index\":1"));
}

#[test]
fn reports_elf_image_backend_capabilities_as_blocked() {
    let mut plan = empty_link_plan();
    plan.cpu_target.machine_arch = "x86_64".to_owned();
    plan.cpu_target.machine_os = "linux".to_owned();
    plan.cpu_target.object_format = "elf".to_owned();
    let report = nsld_object_image_dry_run_report(Path::new("manifest.toml"), &plan);
    let rendered = crate::toml::render_object_image_dry_run(&report);
    let json = crate::json_object_image::nsld_object_image_dry_run_report_json(&report);

    assert_eq!(report.writer_backend_kind, "elf-amd64");
    assert_eq!(report.backend_family, "elf");
    assert_eq!(report.backend_status, "not-implemented");
    assert!(!report.image_constructed);
    assert!(!report.image_ready);
    assert!(report.backend_capabilities.iter().any(|capability| {
        capability.capability_id == "object-image-encoder" && capability.status == "not-implemented"
    }));
    assert!(report
        .blockers
        .contains(&"object-image-backend:elf-amd64:not-implemented".to_owned()));
    assert!(rendered.contains("backend_status = \"not-implemented\""));
    assert!(rendered.contains("capability_id = \"object-image-encoder\""));
    assert!(rendered.contains("status = \"not-implemented\""));
    assert!(json.contains("\"backend_status\":\"not-implemented\""));
    assert!(json.contains("\"capability_id\":\"object-image-encoder\""));
}

#[test]
fn verify_object_image_dry_run_reports_writer_identity_drift() {
    let mut plan = empty_link_plan();
    plan.output_dir = temp_output_dir("nsld-object-image-dry-run-identity-drift");
    fs::create_dir_all(&plan.output_dir).unwrap();
    let manifest = Path::new("manifest.toml");
    nsld_emit_object_image_dry_run_report(manifest, &plan).unwrap();

    let path = Path::new(&plan.output_dir).join("nuis.nsld.object-image-dry-run.toml");
    let damaged = fs::read_to_string(&path).unwrap().replace(
        "writer_backend_kind = \"mach-o-arm64\"",
        "writer_backend_kind = \"elf-amd64\"",
    );
    fs::write(&path, damaged).unwrap();

    let verify = nsld_verify_object_image_dry_run_report(manifest, &plan);
    fs::remove_dir_all(&plan.output_dir).unwrap();

    assert!(!verify.valid);
    assert!(verify.issues.iter().any(|issue| {
        issue == "writer_backend_kind mismatch: expected mach-o-arm64, found elf-amd64"
    }));
}

#[test]
fn emits_and_verifies_object_image_dry_run_report() {
    let mut plan = empty_link_plan();
    plan.output_dir = temp_output_dir("nsld-object-image-dry-run");
    fs::create_dir_all(&plan.output_dir).unwrap();
    let manifest = Path::new("manifest.toml");

    let emit = nsld_emit_object_image_dry_run_report(manifest, &plan).unwrap();
    assert!(emit
        .output_path
        .ends_with("nuis.nsld.object-image-dry-run.toml"));
    assert!(emit
        .image_path
        .ends_with("nuis.nsld.object-image-dry-run.bin"));
    assert!(emit.image_emitted);
    assert!(emit.image_constructed);
    assert!(Path::new(&emit.image_path).exists());

    let verify = nsld_verify_object_image_dry_run_report(manifest, &plan);
    assert!(verify.valid, "{:?}", verify.issues);
    assert!(verify.expected_relocation_lowering_valid);
    assert_eq!(verify.actual_relocation_lowering_valid, Some(true));
    assert_eq!(verify.expected_relocation_lowering_rule_count, 4);
    assert_eq!(verify.actual_relocation_lowering_rule_count, Some(4));
    assert_eq!(verify.expected_relocation_lowering_rules.len(), 4);
    assert_eq!(
        verify
            .actual_relocation_lowering_rules
            .as_deref()
            .map(|rules| rules.len()),
        Some(4)
    );
    assert!(verify.expected_relocation_lowering_issues.is_empty());
    assert_eq!(
        verify.actual_relocation_lowering_issues.as_deref(),
        Some([].as_slice())
    );
    assert_eq!(verify.expected_relocation_record_count, 4);
    assert_eq!(verify.actual_relocation_record_count, Some(4));
    assert!(verify
        .expected_relocation_record_table_hash
        .starts_with("0x"));
    assert_eq!(
        verify.actual_relocation_record_table_hash.as_deref(),
        Some(verify.expected_relocation_record_table_hash.as_str())
    );
    assert_eq!(verify.expected_relocation_records.len(), 4);
    assert_eq!(
        verify
            .actual_relocation_records
            .as_deref()
            .map(|records| records.len()),
        Some(4)
    );
    assert_eq!(
        verify.expected_relocation_records[0].relocation_seed_id,
        "orel0000.compiled_artifact"
    );
    assert_eq!(
        verify.actual_image_hash.as_deref(),
        verify.expected_image_hash.as_deref()
    );
    assert_eq!(verify.actual_backend_family.as_deref(), Some("mach-o"));
    assert_eq!(verify.actual_backend_status.as_deref(), Some("ready"));
    assert_eq!(
        verify.actual_image_file_hash.as_deref(),
        verify.expected_image_hash.as_deref()
    );
    assert_eq!(
        verify.actual_image_file_size_bytes,
        verify.expected_image_size_bytes
    );
}

#[test]
fn verify_object_image_dry_run_reports_relocation_lowering_drift() {
    let mut plan = empty_link_plan();
    plan.output_dir = temp_output_dir("nsld-object-image-dry-run-relocation-drift");
    fs::create_dir_all(&plan.output_dir).unwrap();
    let manifest = Path::new("manifest.toml");
    nsld_emit_object_image_dry_run_report(manifest, &plan).unwrap();

    let path = Path::new(&plan.output_dir).join("nuis.nsld.object-image-dry-run.toml");
    let damaged = fs::read_to_string(&path).unwrap().replace(
        "relocation_lowering_rule_count = 4",
        "relocation_lowering_rule_count = 0",
    );
    fs::write(&path, damaged).unwrap();

    let verify = nsld_verify_object_image_dry_run_report(manifest, &plan);
    fs::remove_dir_all(&plan.output_dir).unwrap();

    assert!(!verify.valid);
    assert_eq!(verify.actual_relocation_lowering_rule_count, Some(0));
    assert!(verify
        .issues
        .iter()
        .any(|issue| { issue == "relocation_lowering_rule_count mismatch: expected 4, found 0" }));
}

#[test]
fn verify_object_image_dry_run_reports_relocation_rule_drift() {
    let mut plan = empty_link_plan();
    plan.output_dir = temp_output_dir("nsld-object-image-dry-run-rule-drift");
    fs::create_dir_all(&plan.output_dir).unwrap();
    let manifest = Path::new("manifest.toml");
    nsld_emit_object_image_dry_run_report(manifest, &plan).unwrap();

    let path = Path::new(&plan.output_dir).join("nuis.nsld.object-image-dry-run.toml");
    let damaged = fs::read_to_string(&path).unwrap().replace(
        "target_relocation_kind = \"arm64-unsigned-pointer\"",
        "target_relocation_kind = \"wrong-relocation\"",
    );
    fs::write(&path, damaged).unwrap();

    let verify = nsld_verify_object_image_dry_run_report(manifest, &plan);
    fs::remove_dir_all(&plan.output_dir).unwrap();

    assert!(!verify.valid);
    assert!(verify.issues.iter().any(|issue| {
            issue == "relocation_lowering_rule[0].target_relocation_kind mismatch: expected arm64-unsigned-pointer, found wrong-relocation"
        }));
}

#[test]
fn verify_object_image_dry_run_reports_relocation_record_drift() {
    let mut plan = empty_link_plan();
    plan.output_dir = temp_output_dir("nsld-object-image-dry-run-record-drift");
    fs::create_dir_all(&plan.output_dir).unwrap();
    let manifest = Path::new("manifest.toml");
    nsld_emit_object_image_dry_run_report(manifest, &plan).unwrap();

    let path = Path::new(&plan.output_dir).join("nuis.nsld.object-image-dry-run.toml");
    let damaged = fs::read_to_string(&path)
        .unwrap()
        .replace("symbol_index = 1", "symbol_index = 99");
    fs::write(&path, damaged).unwrap();

    let verify = nsld_verify_object_image_dry_run_report(manifest, &plan);
    fs::remove_dir_all(&plan.output_dir).unwrap();

    assert!(!verify.valid);
    assert_eq!(verify.actual_relocation_record_count, Some(4));
    assert!(verify.issues.iter().any(|issue| {
        issue == "relocation_record[0].symbol_index mismatch: expected 1, found 99"
    }));
}

fn temp_output_dir(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir()
        .join(format!("{prefix}-{nanos}"))
        .display()
        .to_string()
}
