use super::{
    object_image_dry_run::{
        nsld_emit_object_image_dry_run_report, nsld_verify_object_image_dry_run_report,
    },
    object_plan::nsld_object_plan_report,
    object_writer_backend::{object_writer_backend, object_writer_backend_readiness},
    reports::{NsldObjectEmitReport, NsldObjectEmitVerifyReport, NsldObjectWriterReadinessReport},
    toml,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(crate) fn nsld_object_writer_readiness_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldObjectWriterReadinessReport {
    let object_plan = nsld_object_plan_report(manifest, plan);
    let backend = object_writer_backend(
        &object_plan.target_arch,
        &object_plan.target_os,
        &object_plan.object_format,
    );
    let readiness =
        object_writer_backend_readiness(&backend, object_plan.ready, &object_plan.blockers);
    NsldObjectWriterReadinessReport {
        manifest: object_plan.manifest,
        writer_target_id: readiness.target_id,
        writer_status: readiness.status,
        object_plan_hash: object_plan.object_plan_hash,
        section_count: object_plan.section_count,
        can_emit_object: readiness.can_emit_object,
        unsupported_features: readiness.unsupported_features,
        blockers: readiness.blockers,
    }
}

pub(crate) fn nsld_emit_object_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<NsldObjectEmitReport, String> {
    let object_plan = nsld_object_plan_report(manifest, plan);
    let readiness = nsld_object_writer_readiness_report(manifest, plan);
    let writer_input_path =
        PathBuf::from(&plan.output_dir).join("nuis.nsld.object-writer-input.toml");
    let blocked_report_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.object.blocked.toml");
    fs::write(
        &writer_input_path,
        toml::render_object_writer_input(&object_plan),
    )
    .map_err(|error| {
        format!(
            "failed to write nsld object writer input `{}`: {error}",
            writer_input_path.display()
        )
    })?;
    let image_dry_run = nsld_emit_object_image_dry_run_report(manifest, plan)?;
    let output_path =
        PathBuf::from(&plan.output_dir).join(format!("nuis.nsld.{}", object_plan.object_format));
    let emitted = readiness.can_emit_object && image_dry_run.image_hash.is_some();
    if emitted {
        fs::copy(&image_dry_run.image_path, &output_path).map_err(|error| {
            format!(
                "failed to write nsld object `{}` from dry-run image `{}`: {error}",
                output_path.display(),
                image_dry_run.image_path
            )
        })?;
    }
    let report = NsldObjectEmitReport {
        manifest: readiness.manifest,
        output_path: output_path.display().to_string(),
        writer_input_path: writer_input_path.display().to_string(),
        blocked_report_path: blocked_report_path.display().to_string(),
        image_dry_run_report_path: image_dry_run.output_path,
        image_dry_run_path: image_dry_run.image_path,
        image_dry_run_hash: image_dry_run.image_hash,
        writer_target_id: readiness.writer_target_id,
        writer_backend_kind: object_plan.writer_backend_kind,
        object_family: object_plan.object_family,
        object_plan_hash: readiness.object_plan_hash,
        emitted,
        can_emit_object: readiness.can_emit_object,
        blockers: readiness.blockers,
    };
    fs::write(
        &blocked_report_path,
        toml::render_object_emit_blocked(&report),
    )
    .map_err(|error| {
        format!(
            "failed to write nsld object emit report `{}`: {error}",
            blocked_report_path.display()
        )
    })?;
    Ok(report)
}

pub(crate) fn nsld_verify_object_emit_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldObjectEmitVerifyReport {
    let expected = nsld_emit_object_report_shape(manifest, plan);
    let input_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.object.blocked.toml");
    let mut issues = Vec::new();
    let actual = fs::read_to_string(&input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_object_emit_blocked `{}`: {error}",
            input_path.display()
        )
    });
    let (
        actual_object_plan_hash,
        actual_writer_backend_kind,
        actual_object_family,
        actual_image_dry_run_hash,
    ) = match actual.as_ref() {
        Ok(source) => (
            toml::string_value(source, "object_plan_hash"),
            toml::string_value(source, "writer_backend_kind"),
            toml::string_value(source, "object_family"),
            optional_string_value(source, "image_dry_run_hash"),
        ),
        Err(error) => {
            issues.push(error.clone());
            (None, None, None, None)
        }
    };
    if actual.is_ok() {
        push_string_mismatch(
            &mut issues,
            "object_plan_hash",
            &expected.object_plan_hash,
            actual_object_plan_hash.as_deref(),
        );
        push_optional_string_mismatch(
            &mut issues,
            "writer_backend_kind",
            Some(&expected.writer_backend_kind),
            actual_writer_backend_kind.as_deref(),
        );
        push_optional_string_mismatch(
            &mut issues,
            "object_family",
            Some(&expected.object_family),
            actual_object_family.as_deref(),
        );
        push_optional_string_mismatch(
            &mut issues,
            "image_dry_run_hash",
            expected.image_dry_run_hash.as_deref(),
            actual_image_dry_run_hash.as_deref(),
        );
    }
    let image_verify = nsld_verify_object_image_dry_run_report(manifest, plan);
    if !image_verify.valid {
        issues.push("object-image-dry-run:invalid".to_owned());
        issues.extend(
            image_verify
                .issues
                .iter()
                .map(|issue| format!("object-image-dry-run:{issue}")),
        );
    }
    if image_verify.actual_image_file_hash != expected.image_dry_run_hash {
        push_optional_string_mismatch(
            &mut issues,
            "image_dry_run_file_hash",
            expected.image_dry_run_hash.as_deref(),
            image_verify.actual_image_file_hash.as_deref(),
        );
    }

    NsldObjectEmitVerifyReport {
        manifest: manifest.display().to_string(),
        input_path: input_path.display().to_string(),
        valid: issues.is_empty(),
        expected_object_plan_hash: expected.object_plan_hash,
        expected_writer_backend_kind: expected.writer_backend_kind,
        expected_object_family: expected.object_family,
        expected_image_dry_run_hash: expected.image_dry_run_hash,
        actual_object_plan_hash,
        actual_writer_backend_kind,
        actual_object_family,
        actual_image_dry_run_hash,
        image_dry_run_report_valid: image_verify.valid,
        issues,
    }
}

fn nsld_emit_object_report_shape(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldObjectEmitReport {
    let object_plan = nsld_object_plan_report(manifest, plan);
    let readiness = nsld_object_writer_readiness_report(manifest, plan);
    let image_dry_run =
        super::object_image_dry_run::nsld_object_image_dry_run_report(manifest, plan);
    let emitted = readiness.can_emit_object && image_dry_run.image_hash.is_some();
    let writer_input_path =
        PathBuf::from(&plan.output_dir).join("nuis.nsld.object-writer-input.toml");
    let blocked_report_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.object.blocked.toml");
    NsldObjectEmitReport {
        manifest: readiness.manifest,
        output_path: PathBuf::from(&plan.output_dir)
            .join(format!("nuis.nsld.{}", object_plan.object_format))
            .display()
            .to_string(),
        writer_input_path: writer_input_path.display().to_string(),
        blocked_report_path: blocked_report_path.display().to_string(),
        image_dry_run_report_path: image_dry_run.output_path,
        image_dry_run_path: image_dry_run.image_path,
        image_dry_run_hash: image_dry_run.image_hash,
        writer_target_id: readiness.writer_target_id,
        writer_backend_kind: object_plan.writer_backend_kind,
        object_family: object_plan.object_family,
        object_plan_hash: readiness.object_plan_hash,
        emitted,
        can_emit_object: readiness.can_emit_object,
        blockers: readiness.blockers,
    }
}

fn optional_string_value(source: &str, key: &str) -> Option<String> {
    toml::string_value(source, key).filter(|value| !value.is_empty())
}

fn push_string_mismatch(
    issues: &mut Vec<String>,
    field: &str,
    expected: &str,
    actual: Option<&str>,
) {
    if actual != Some(expected) {
        issues.push(format!(
            "{field} mismatch: expected {expected}, found {}",
            actual.unwrap_or("missing")
        ));
    }
}

fn push_optional_string_mismatch(
    issues: &mut Vec<String>,
    field: &str,
    expected: Option<&str>,
    actual: Option<&str>,
) {
    if actual != expected {
        issues.push(format!(
            "{field} mismatch: expected {}, found {}",
            expected.unwrap_or("missing"),
            actual.unwrap_or("missing")
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::{
        nsld_emit_object_report, nsld_object_writer_readiness_report,
        nsld_verify_object_emit_report,
    };
    use crate::{
        assembly::{
            nsld_emit_assemble_plan_report, nsld_emit_link_bundle_report,
            nsld_emit_section_manifest_report,
        },
        link_units::{nsld_emit_link_inputs_report, nsld_emit_link_units_report},
        main_test_support::empty_link_plan,
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
        assert!(report.unsupported_features.is_empty());
        assert!(report.blockers.is_empty());
    }

    #[test]
    fn emit_object_writes_minimal_mach_o_bytes() {
        let dir =
            std::env::temp_dir().join(format!("nsld-object-emit-blocked-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();
        plan.compiled_artifact.path = dir.join("nuis.compiled.artifact").display().to_string();
        fs::write(&plan.compiled_artifact.path, b"compiled-artifact").unwrap();
        emit_object_prerequisites(Path::new("manifest.toml"), &plan);
        let report = nsld_emit_object_report(Path::new("manifest.toml"), &plan).unwrap();
        let writer_input =
            fs::read_to_string(dir.join("nuis.nsld.object-writer-input.toml")).unwrap();
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
}
