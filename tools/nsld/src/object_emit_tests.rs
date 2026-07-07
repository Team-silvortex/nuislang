use crate::{
    assembly::{
        nsld_emit_assemble_plan_report, nsld_emit_link_bundle_report,
        nsld_emit_section_manifest_report,
    },
    link_units::{nsld_emit_link_inputs_report, nsld_emit_link_units_report},
    main_test_support::empty_link_plan,
    object_emit::{
        nsld_emit_object_report, nsld_object_writer_readiness_report,
        nsld_verify_object_emit_report,
    },
    object_writer_input::nsld_verify_object_writer_input_report,
};
use std::{fs, path::Path};

#[test]
fn object_writer_readiness_allows_minimal_mach_o_emit() {
    let dir = std::env::temp_dir().join(format!(
        "nsld-object-writer-readiness-ready-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = dir.join("nuis.compiled.artifact").display().to_string();
    fs::write(&plan.compiled_artifact.path, b"compiled-artifact").unwrap();
    emit_object_prerequisites(Path::new("manifest.toml"), &plan);
    let report = nsld_object_writer_readiness_report(Path::new("manifest.toml"), &plan);
    fs::remove_dir_all(dir).unwrap();

    assert!(report.can_emit_object);
    assert_eq!(report.writer_status, "ready");
    assert!(report
        .writer_stages
        .iter()
        .any(|stage| stage.stage_id == "macho-header" && stage.status == "ready"));
    assert!(report
        .writer_stages
        .iter()
        .all(|stage| stage.required && stage.status == "ready"));
    assert!(report.unsupported_features.is_empty());
    assert!(report.blockers.is_empty());
}

#[test]
fn object_writer_readiness_reports_blocked_elf_stages() {
    let dir = std::env::temp_dir().join(format!(
        "nsld-object-writer-readiness-elf-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = dir.join("nuis.compiled.artifact").display().to_string();
    plan.cpu_target.machine_arch = "x86_64".to_owned();
    plan.cpu_target.machine_os = "linux".to_owned();
    plan.cpu_target.object_format = "elf".to_owned();
    fs::write(&plan.compiled_artifact.path, b"compiled-artifact").unwrap();
    emit_object_prerequisites(Path::new("manifest.toml"), &plan);
    let report = nsld_object_writer_readiness_report(Path::new("manifest.toml"), &plan);
    let report_json = crate::json::nsld_object_writer_readiness_report_json(&report);
    fs::remove_dir_all(dir).unwrap();

    assert!(!report.can_emit_object);
    assert_eq!(report.writer_status, "recognized-blocked");
    assert!(report
        .writer_stages
        .iter()
        .any(|stage| stage.stage_id == "elf-header" && stage.status == "not-implemented"));
    assert!(report
        .blockers
        .iter()
        .any(|blocker| { blocker == "object-writer-stage:elf-byte-emission:not-implemented" }));
    assert!(report_json.contains("\"writer_stages\":[{"));
    assert!(report_json.contains("\"stage_id\":\"elf-header\""));
    assert!(report_json.contains("\"status\":\"not-implemented\""));
}

#[test]
fn emit_object_uses_plan_specific_output_path_for_pe_coff_alias() {
    let dir = std::env::temp_dir().join(format!(
        "nsld-object-emit-pe-coff-output-path-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = dir.join("nuis.compiled.artifact").display().to_string();
    plan.cpu_target.machine_arch = "amd64".to_owned();
    plan.cpu_target.machine_os = "windows".to_owned();
    plan.cpu_target.object_format = "pe/coff".to_owned();
    fs::write(&plan.compiled_artifact.path, b"compiled-artifact").unwrap();
    emit_object_prerequisites(Path::new("manifest.toml"), &plan);

    let report = nsld_emit_object_report(Path::new("manifest.toml"), &plan).unwrap();
    fs::remove_dir_all(dir).unwrap();

    assert!(!report.emitted);
    assert_eq!(report.writer_backend_kind, "coff-amd64");
    assert_eq!(report.object_family, "coff");
    assert!(report.output_path.ends_with("nuis.nsld.pe-coff"));
    assert!(!report.output_path.ends_with("nuis.nsld.mach-o"));
    assert!(report
        .blockers
        .iter()
        .any(|blocker| blocker == "object-writer-stage:coff-byte-emission:not-implemented"));
}

#[test]
fn emit_object_writes_minimal_mach_o_bytes() {
    let dir = std::env::temp_dir().join(format!("nsld-object-emit-blocked-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    plan.compiled_artifact.path = dir.join("nuis.compiled.artifact").display().to_string();
    fs::write(&plan.compiled_artifact.path, b"compiled-artifact").unwrap();
    emit_object_prerequisites(Path::new("manifest.toml"), &plan);
    let report = nsld_emit_object_report(Path::new("manifest.toml"), &plan).unwrap();
    let writer_input = fs::read_to_string(dir.join("nuis.nsld.object-writer-input.toml")).unwrap();
    let blocked_report = fs::read_to_string(dir.join("nuis.nsld.object.blocked.toml")).unwrap();
    let image_dry_run_report =
        fs::read_to_string(dir.join("nuis.nsld.object-image-dry-run.toml")).unwrap();
    let image_dry_run_bytes = fs::read(dir.join("nuis.nsld.object-image-dry-run.bin")).unwrap();
    let object_bytes = fs::read(dir.join("nuis.nsld.mach-o")).unwrap();
    fs::remove_dir_all(dir).unwrap();

    assert!(report.emitted);
    assert!(report.can_emit_object);
    assert!(report.output_path.ends_with("nuis.nsld.mach-o"));
    assert!(report
        .writer_input_path
        .ends_with("nuis.nsld.object-writer-input.toml"));
    assert!(report
        .blocked_report_path
        .ends_with("nuis.nsld.object.blocked.toml"));
    assert!(report
        .image_dry_run_report_path
        .ends_with("nuis.nsld.object-image-dry-run.toml"));
    assert!(report
        .image_dry_run_path
        .ends_with("nuis.nsld.object-image-dry-run.bin"));
    assert!(report
        .image_dry_run_hash
        .as_deref()
        .unwrap()
        .starts_with("0x"));
    assert!(writer_input.contains("kind = \"object-writer-input\""));
    assert!(writer_input.contains("writer_backend_kind = \"mach-o-arm64\""));
    assert!(writer_input.contains("object_family = \"mach-o\""));
    assert!(writer_input.contains("[[writer_section]]"));
    assert!(writer_input.contains("[[writer_relocation_seed]]"));
    assert!(image_dry_run_report.contains("kind = \"object-image-dry-run\""));
    assert!(image_dry_run_report.contains("backend_status = \"ready\""));
    assert!(!image_dry_run_bytes.is_empty());
    assert_eq!(object_bytes, image_dry_run_bytes);
    assert_eq!(&object_bytes[0..4], &[0xcf, 0xfa, 0xed, 0xfe]);
    assert!(report.image_dry_run_hash.is_some());
    assert!(blocked_report.contains("kind = \"object-emit-blocked\""));
    assert!(blocked_report.contains("writer_input_path = \""));
    assert!(blocked_report.contains("image_dry_run_report_path = \""));
    assert!(blocked_report.contains("image_dry_run_path = \""));
    assert!(blocked_report.contains("image_dry_run_hash = \"0x"));
    assert!(blocked_report.contains("writer_backend_kind = \"mach-o-arm64\""));
    assert!(blocked_report.contains("object_family = \"mach-o\""));
    assert_eq!(report.writer_backend_kind, "mach-o-arm64");
    assert_eq!(report.object_family, "mach-o");
    assert!(blocked_report.contains("emitted = true"));
    assert!(report.blockers.is_empty());
}

#[test]
fn emit_object_without_prepared_sources_stays_blocked() {
    let dir = std::env::temp_dir().join(format!(
        "nsld-object-emit-unprepared-blocked-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();

    let report = nsld_emit_object_report(Path::new("manifest.toml"), &plan).unwrap();
    fs::remove_dir_all(dir).unwrap();

    assert!(!report.emitted);
    assert!(!report.can_emit_object);
    assert!(report
        .blockers
        .iter()
        .any(|blocker| blocker.contains("section:compiled-artifact:")));
}

#[test]
fn verify_object_writer_input_accepts_emit_object_snapshot() {
    let dir = std::env::temp_dir().join(format!(
        "nsld-object-writer-input-verify-ok-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    nsld_emit_object_report(Path::new("manifest.toml"), &plan).unwrap();

    let verify = nsld_verify_object_writer_input_report(Path::new("manifest.toml"), &plan);
    fs::remove_dir_all(dir).unwrap();

    assert!(verify.valid);
    assert!(verify.issues.is_empty());
    assert_eq!(verify.actual_section_count, Some(4));
}

fn emit_object_prerequisites(manifest: &Path, plan: &nuisc::linker::LinkPlan) {
    nsld_emit_link_inputs_report(manifest, plan).unwrap();
    nsld_emit_link_units_report(manifest, plan).unwrap();
    nsld_emit_link_bundle_report(manifest, plan).unwrap();
    nsld_emit_assemble_plan_report(manifest, plan).unwrap();
    nsld_emit_section_manifest_report(manifest, plan).unwrap();
}

#[test]
fn verify_object_emit_accepts_blocked_emit_snapshot() {
    let dir =
        std::env::temp_dir().join(format!("nsld-object-emit-verify-ok-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    nsld_emit_object_report(Path::new("manifest.toml"), &plan).unwrap();

    let verify = nsld_verify_object_emit_report(Path::new("manifest.toml"), &plan);
    fs::remove_dir_all(dir).unwrap();

    assert!(verify.valid, "{:?}", verify.issues);
    assert!(verify.image_dry_run_report_valid);
    assert_eq!(
        verify.actual_image_dry_run_hash.as_deref(),
        verify.expected_image_dry_run_hash.as_deref()
    );
}

#[test]
fn verify_object_emit_reports_dry_run_hash_drift() {
    let dir = std::env::temp_dir().join(format!(
        "nsld-object-emit-verify-hash-drift-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    nsld_emit_object_report(Path::new("manifest.toml"), &plan).unwrap();
    let blocked_path = dir.join("nuis.nsld.object.blocked.toml");
    let damaged = fs::read_to_string(&blocked_path)
        .unwrap()
        .replace("image_dry_run_hash = \"0x", "image_dry_run_hash = \"0y");
    fs::write(&blocked_path, damaged).unwrap();

    let verify = nsld_verify_object_emit_report(Path::new("manifest.toml"), &plan);
    fs::remove_dir_all(dir).unwrap();

    assert!(!verify.valid);
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("image_dry_run_hash mismatch")));
}

#[test]
fn verify_object_emit_reports_writer_identity_drift() {
    let dir = std::env::temp_dir().join(format!(
        "nsld-object-emit-verify-writer-identity-drift-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    nsld_emit_object_report(Path::new("manifest.toml"), &plan).unwrap();
    let blocked_path = dir.join("nuis.nsld.object.blocked.toml");
    let damaged = fs::read_to_string(&blocked_path).unwrap().replace(
        "writer_backend_kind = \"mach-o-arm64\"",
        "writer_backend_kind = \"elf-amd64\"",
    );
    fs::write(&blocked_path, damaged).unwrap();

    let verify = nsld_verify_object_emit_report(Path::new("manifest.toml"), &plan);
    fs::remove_dir_all(dir).unwrap();

    assert!(!verify.valid);
    assert!(verify.issues.iter().any(|issue| {
        issue == "writer_backend_kind mismatch: expected mach-o-arm64, found elf-amd64"
    }));
}

#[test]
fn verify_object_writer_input_reports_tampered_layout_hash() {
    let dir = std::env::temp_dir().join(format!(
        "nsld-object-writer-input-layout-tamper-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    nsld_emit_object_report(Path::new("manifest.toml"), &plan).unwrap();
    let input_path = dir.join("nuis.nsld.object-writer-input.toml");
    let damaged = fs::read_to_string(&input_path)
        .unwrap()
        .replace("object_layout_hash = \"0x", "object_layout_hash = \"0y");
    fs::write(&input_path, damaged).unwrap();

    let verify = nsld_verify_object_writer_input_report(Path::new("manifest.toml"), &plan);
    fs::remove_dir_all(dir).unwrap();

    assert!(!verify.valid);
    assert!(verify
        .issues
        .iter()
        .any(|issue| issue.starts_with("object_layout_hash mismatch: expected 0x")));
}

#[test]
fn verify_object_writer_input_reports_writer_section_drift() {
    let dir = std::env::temp_dir().join(format!(
        "nsld-object-writer-input-section-drift-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    nsld_emit_object_report(Path::new("manifest.toml"), &plan).unwrap();
    let input_path = dir.join("nuis.nsld.object-writer-input.toml");
    let damaged = fs::read_to_string(&input_path).unwrap().replace(
        "object_section_name = \".nuis.text.compiled\"",
        "object_section_name = \".nuis.text.drift\"",
    );
    fs::write(&input_path, damaged).unwrap();

    let verify = nsld_verify_object_writer_input_report(Path::new("manifest.toml"), &plan);
    fs::remove_dir_all(dir).unwrap();

    assert!(!verify.valid);
    assert!(verify.issues.iter().any(|issue| {
            issue
                == "writer_section[0].object_section_name mismatch: expected .nuis.text.compiled, found .nuis.text.drift"
        }));
}

#[test]
fn verify_object_writer_input_reports_writer_relocation_seed_drift() {
    let dir = std::env::temp_dir().join(format!(
        "nsld-object-writer-input-relocation-drift-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let mut plan = empty_link_plan();
    plan.output_dir = dir.display().to_string();
    nsld_emit_object_report(Path::new("manifest.toml"), &plan).unwrap();
    let input_path = dir.join("nuis.nsld.object-writer-input.toml");
    let damaged = fs::read_to_string(&input_path).unwrap().replace(
        "target_symbol = \"__nuis_section_sec0000_compiled_artifact\"",
        "target_symbol = \"__nuis_section_wrong\"",
    );
    fs::write(&input_path, damaged).unwrap();

    let verify = nsld_verify_object_writer_input_report(Path::new("manifest.toml"), &plan);
    fs::remove_dir_all(dir).unwrap();

    assert!(!verify.valid);
    assert!(verify.issues.iter().any(|issue| {
            issue
                == "writer_relocation_seed[0].target_symbol mismatch: expected __nuis_section_sec0000_compiled_artifact, found __nuis_section_wrong"
        }));
}
