use super::{
    artifact_chain::{nsld_artifact_stage_kind_path_for_plan, NsldArtifactStageKind},
    object_plan::nsld_object_plan_report,
    object_writer_backend::{object_writer_backend, object_writer_backend_readiness},
    object_writer_input_verify::{
        push_string_mismatch, push_usize_mismatch, writer_input_relocation_seed_table_field_issues,
        writer_input_section_table_field_issues, writer_relocation_seed_entries,
        writer_relocation_seed_table_mismatch_issues, writer_section_entries,
        writer_section_table_mismatch_issues,
    },
    reports::{
        NsldObjectWriterDryRunEmitReport, NsldObjectWriterDryRunReport,
        NsldObjectWriterDryRunVerifyReport, NsldObjectWriterInputVerifyReport,
    },
    toml,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(crate) fn nsld_verify_object_writer_input_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldObjectWriterInputVerifyReport {
    let expected_report = nsld_object_plan_report(manifest, plan);
    let input_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.object-writer-input.toml");
    let mut issues = Vec::new();
    let actual = fs::read_to_string(&input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_object_writer_input `{}`: {error}",
            input_path.display()
        )
    });
    let (
        actual_object_plan_hash,
        actual_writer_backend_kind,
        actual_object_family,
        actual_object_layout_hash,
        actual_relocation_seed_table_hash,
        actual_section_count,
        actual_relocation_seed_count,
    ) = match actual.as_ref() {
        Ok(source) => (
            toml::string_value(source, "object_plan_hash"),
            toml::string_value(source, "writer_backend_kind"),
            toml::string_value(source, "object_family"),
            toml::string_value(source, "object_layout_hash"),
            toml::string_value(source, "relocation_seed_table_hash"),
            toml::usize_value(source, "section_count"),
            toml::usize_value(source, "relocation_seed_count"),
        ),
        Err(error) => {
            issues.push(error.clone());
            (None, None, None, None, None, None, None)
        }
    };
    if let Ok(actual) = actual {
        issues.extend(writer_input_section_table_field_issues(&actual));
        issues.extend(writer_input_relocation_seed_table_field_issues(&actual));
        issues.extend(writer_section_table_mismatch_issues(
            &expected_report.object_sections,
            &writer_section_entries(&actual),
        ));
        issues.extend(writer_relocation_seed_table_mismatch_issues(
            &expected_report.relocation_seeds,
            &writer_relocation_seed_entries(&actual),
        ));
        if actual_object_plan_hash.as_deref() != Some(expected_report.object_plan_hash.as_str()) {
            issues.push(format!(
                "object_plan_hash mismatch: expected {}, found {}",
                expected_report.object_plan_hash,
                actual_object_plan_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_writer_backend_kind.as_deref()
            != Some(expected_report.writer_backend_kind.as_str())
        {
            issues.push(format!(
                "writer_backend_kind mismatch: expected {}, found {}",
                expected_report.writer_backend_kind,
                actual_writer_backend_kind
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_object_family.as_deref() != Some(expected_report.object_family.as_str()) {
            issues.push(format!(
                "object_family mismatch: expected {}, found {}",
                expected_report.object_family,
                actual_object_family
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_object_layout_hash.as_deref() != Some(expected_report.object_layout_hash.as_str())
        {
            issues.push(format!(
                "object_layout_hash mismatch: expected {}, found {}",
                expected_report.object_layout_hash,
                actual_object_layout_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_relocation_seed_table_hash.as_deref()
            != Some(expected_report.relocation_seed_table_hash.as_str())
        {
            issues.push(format!(
                "relocation_seed_table_hash mismatch: expected {}, found {}",
                expected_report.relocation_seed_table_hash,
                actual_relocation_seed_table_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_section_count != Some(expected_report.section_count) {
            issues.push(format!(
                "section_count mismatch: expected {}, found {}",
                expected_report.section_count,
                actual_section_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_relocation_seed_count != Some(expected_report.relocation_seed_count) {
            issues.push(format!(
                "relocation_seed_count mismatch: expected {}, found {}",
                expected_report.relocation_seed_count,
                actual_relocation_seed_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
    }

    NsldObjectWriterInputVerifyReport {
        manifest: manifest.display().to_string(),
        input_path: input_path.display().to_string(),
        valid: issues.is_empty(),
        expected_object_plan_hash: expected_report.object_plan_hash,
        expected_writer_backend_kind: expected_report.writer_backend_kind,
        expected_object_family: expected_report.object_family,
        expected_object_layout_hash: expected_report.object_layout_hash,
        expected_relocation_seed_table_hash: expected_report.relocation_seed_table_hash,
        expected_section_count: expected_report.section_count,
        expected_relocation_seed_count: expected_report.relocation_seed_count,
        actual_object_plan_hash,
        actual_writer_backend_kind,
        actual_object_family,
        actual_object_layout_hash,
        actual_relocation_seed_table_hash,
        actual_section_count,
        actual_relocation_seed_count,
        issues,
    }
}

pub(crate) fn nsld_object_writer_dry_run_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldObjectWriterDryRunReport {
    let object_plan = nsld_object_plan_report(manifest, plan);
    let verify = nsld_verify_object_writer_input_report(manifest, plan);
    let mut blockers = object_plan.blockers.clone();
    if !verify.valid {
        blockers.push("object-writer-input:invalid".to_owned());
        blockers.extend(
            verify
                .issues
                .iter()
                .map(|issue| format!("object-writer-input:{issue}")),
        );
    }
    let backend = object_writer_backend(
        &object_plan.target_arch,
        &object_plan.target_os,
        &object_plan.object_format,
    );
    let readiness =
        object_writer_backend_readiness(&backend, object_plan.ready && verify.valid, &blockers);

    NsldObjectWriterDryRunReport {
        manifest: manifest.display().to_string(),
        writer_input_path: verify.input_path,
        planned_output_path: nsld_artifact_stage_kind_path_for_plan(
            plan,
            NsldArtifactStageKind::ObjectOutput,
        )
        .display()
        .to_string(),
        writer_target_id: readiness.target_id,
        writer_backend_kind: object_plan.writer_backend_kind,
        object_family: object_plan.object_family,
        object_plan_hash: object_plan.object_plan_hash,
        object_layout_hash: object_plan.object_layout_hash,
        relocation_seed_table_hash: object_plan.relocation_seed_table_hash,
        section_count: object_plan.section_count,
        relocation_seed_count: object_plan.relocation_seed_count,
        writer_input_valid: verify.valid,
        can_emit_object: readiness.can_emit_object,
        dry_run_ready: readiness.can_emit_object,
        blockers,
    }
}

pub(crate) fn nsld_emit_object_writer_dry_run_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<NsldObjectWriterDryRunEmitReport, String> {
    let report = nsld_object_writer_dry_run_report(manifest, plan);
    let output_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.object-writer-dry-run.toml");
    fs::write(&output_path, toml::render_object_writer_dry_run(&report)).map_err(|error| {
        format!(
            "failed to write nsld object writer dry run `{}`: {error}",
            output_path.display()
        )
    })?;

    Ok(NsldObjectWriterDryRunEmitReport {
        manifest: report.manifest,
        output_path: output_path.display().to_string(),
        dry_run_ready: report.dry_run_ready,
        object_plan_hash: report.object_plan_hash,
        section_count: report.section_count,
        relocation_seed_count: report.relocation_seed_count,
    })
}

pub(crate) fn nsld_verify_object_writer_dry_run_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldObjectWriterDryRunVerifyReport {
    let expected_report = nsld_object_writer_dry_run_report(manifest, plan);
    let expected = toml::render_object_writer_dry_run(&expected_report);
    let input_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.object-writer-dry-run.toml");
    let mut issues = Vec::new();
    let actual = fs::read_to_string(&input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_object_writer_dry_run `{}`: {error}",
            input_path.display()
        )
    });
    let (
        actual_object_plan_hash,
        actual_writer_backend_kind,
        actual_object_family,
        actual_object_layout_hash,
        actual_relocation_seed_table_hash,
        actual_section_count,
        actual_relocation_seed_count,
        actual_dry_run_ready,
    ) = match actual.as_ref() {
        Ok(source) => (
            toml::string_value(source, "object_plan_hash"),
            toml::string_value(source, "writer_backend_kind"),
            toml::string_value(source, "object_family"),
            toml::string_value(source, "object_layout_hash"),
            toml::string_value(source, "relocation_seed_table_hash"),
            toml::usize_value(source, "section_count"),
            toml::usize_value(source, "relocation_seed_count"),
            toml::bool_value(source, "dry_run_ready"),
        ),
        Err(error) => {
            issues.push(error.clone());
            (None, None, None, None, None, None, None, None)
        }
    };
    if let Ok(actual) = actual {
        if actual != expected {
            issues.push("object-writer-dry-run-content-mismatch".to_owned());
        }
        push_string_mismatch(
            &mut issues,
            "object_plan_hash",
            &expected_report.object_plan_hash,
            actual_object_plan_hash.as_deref(),
        );
        push_string_mismatch(
            &mut issues,
            "writer_backend_kind",
            &expected_report.writer_backend_kind,
            actual_writer_backend_kind.as_deref(),
        );
        push_string_mismatch(
            &mut issues,
            "object_family",
            &expected_report.object_family,
            actual_object_family.as_deref(),
        );
        push_string_mismatch(
            &mut issues,
            "object_layout_hash",
            &expected_report.object_layout_hash,
            actual_object_layout_hash.as_deref(),
        );
        push_string_mismatch(
            &mut issues,
            "relocation_seed_table_hash",
            &expected_report.relocation_seed_table_hash,
            actual_relocation_seed_table_hash.as_deref(),
        );
        push_usize_mismatch(
            &mut issues,
            "section_count",
            expected_report.section_count,
            actual_section_count,
        );
        push_usize_mismatch(
            &mut issues,
            "relocation_seed_count",
            expected_report.relocation_seed_count,
            actual_relocation_seed_count,
        );
        if actual_dry_run_ready != Some(expected_report.dry_run_ready) {
            issues.push(format!(
                "dry_run_ready mismatch: expected {}, found {}",
                expected_report.dry_run_ready,
                actual_dry_run_ready
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
    }

    NsldObjectWriterDryRunVerifyReport {
        manifest: manifest.display().to_string(),
        input_path: input_path.display().to_string(),
        valid: issues.is_empty(),
        expected_object_plan_hash: expected_report.object_plan_hash,
        expected_writer_backend_kind: expected_report.writer_backend_kind,
        expected_object_family: expected_report.object_family,
        expected_object_layout_hash: expected_report.object_layout_hash,
        expected_relocation_seed_table_hash: expected_report.relocation_seed_table_hash,
        expected_section_count: expected_report.section_count,
        expected_relocation_seed_count: expected_report.relocation_seed_count,
        expected_dry_run_ready: expected_report.dry_run_ready,
        actual_object_plan_hash,
        actual_writer_backend_kind,
        actual_object_family,
        actual_object_layout_hash,
        actual_relocation_seed_table_hash,
        actual_section_count,
        actual_relocation_seed_count,
        actual_dry_run_ready,
        issues,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        nsld_emit_object_writer_dry_run_report, nsld_object_writer_dry_run_report,
        nsld_verify_object_writer_dry_run_report, nsld_verify_object_writer_input_report,
    };
    use crate::{
        assembly::{
            nsld_emit_assemble_plan_report, nsld_emit_link_bundle_report,
            nsld_emit_section_manifest_report,
        },
        link_units::{nsld_emit_link_inputs_report, nsld_emit_link_units_report},
        main_test_support::empty_link_plan,
        object_emit::nsld_emit_object_report,
    };
    use std::{fs, path::Path};

    #[test]
    fn dry_run_reports_missing_writer_input_as_blocker() {
        let dir = std::env::temp_dir().join(format!(
            "nsld-object-writer-dry-run-missing-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();

        let report = nsld_object_writer_dry_run_report(Path::new("manifest.toml"), &plan);
        fs::remove_dir_all(dir).unwrap();

        assert!(!report.writer_input_valid);
        assert!(!report.can_emit_object);
        assert!(!report.dry_run_ready);
        assert!(report
            .blockers
            .iter()
            .any(|blocker| blocker == "object-writer-input:invalid"));
    }

    #[test]
    fn dry_run_consumes_emitted_writer_input_and_is_ready_for_mach_o() {
        let dir = std::env::temp_dir().join(format!(
            "nsld-object-writer-dry-run-blocked-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();
        plan.compiled_artifact.path = dir.join("nuis.compiled.artifact").display().to_string();
        fs::write(&plan.compiled_artifact.path, b"compiled-artifact").unwrap();
        emit_object_prerequisites(Path::new("manifest.toml"), &plan);
        nsld_emit_object_report(Path::new("manifest.toml"), &plan).unwrap();

        let report = nsld_object_writer_dry_run_report(Path::new("manifest.toml"), &plan);
        fs::remove_dir_all(dir).unwrap();

        assert!(report.writer_input_valid);
        assert!(report.can_emit_object);
        assert!(report.dry_run_ready);
        assert_eq!(report.section_count, 4);
        assert_eq!(report.relocation_seed_count, 4);
        assert_eq!(report.writer_backend_kind, "mach-o-arm64");
        assert_eq!(report.object_family, "mach-o");
        let rendered = crate::toml::render_object_writer_dry_run(&report);
        let json = crate::json_object::nsld_object_writer_dry_run_report_json(&report);
        assert!(rendered.contains("writer_backend_kind = \"mach-o-arm64\""));
        assert!(rendered.contains("object_family = \"mach-o\""));
        assert!(json.contains("\"writer_backend_kind\":\"mach-o-arm64\""));
        assert!(json.contains("\"object_family\":\"mach-o\""));
        assert!(report.planned_output_path.ends_with("nuis.nsld.mach-o"));
        assert!(report.blockers.is_empty());
    }

    fn emit_object_prerequisites(manifest: &Path, plan: &nuisc::linker::LinkPlan) {
        nsld_emit_link_inputs_report(manifest, plan).unwrap();
        nsld_emit_link_units_report(manifest, plan).unwrap();
        nsld_emit_link_bundle_report(manifest, plan).unwrap();
        nsld_emit_assemble_plan_report(manifest, plan).unwrap();
        nsld_emit_section_manifest_report(manifest, plan).unwrap();
    }

    #[test]
    fn verify_object_writer_input_reports_writer_identity_drift() {
        let dir = std::env::temp_dir().join(format!(
            "nsld-object-writer-input-identity-drift-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();
        nsld_emit_object_report(Path::new("manifest.toml"), &plan).unwrap();
        let input_path = dir.join("nuis.nsld.object-writer-input.toml");
        let damaged = fs::read_to_string(&input_path).unwrap().replace(
            "writer_backend_kind = \"mach-o-arm64\"",
            "writer_backend_kind = \"elf-amd64\"",
        );
        fs::write(&input_path, damaged).unwrap();

        let verify = nsld_verify_object_writer_input_report(Path::new("manifest.toml"), &plan);
        fs::remove_dir_all(dir).unwrap();

        assert!(!verify.valid);
        assert!(verify.issues.iter().any(|issue| {
            issue == "writer_backend_kind mismatch: expected mach-o-arm64, found elf-amd64"
        }));
    }

    #[test]
    fn verify_object_writer_dry_run_reports_writer_identity_drift() {
        let dir = std::env::temp_dir().join(format!(
            "nsld-object-writer-dry-run-identity-drift-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();
        nsld_emit_object_report(Path::new("manifest.toml"), &plan).unwrap();
        nsld_emit_object_writer_dry_run_report(Path::new("manifest.toml"), &plan).unwrap();
        let input_path = dir.join("nuis.nsld.object-writer-dry-run.toml");
        let damaged = fs::read_to_string(&input_path).unwrap().replace(
            "writer_backend_kind = \"mach-o-arm64\"",
            "writer_backend_kind = \"elf-amd64\"",
        );
        fs::write(&input_path, damaged).unwrap();
        let verify = nsld_verify_object_writer_dry_run_report(Path::new("manifest.toml"), &plan);
        fs::remove_dir_all(dir).unwrap();
        assert!(!verify.valid);
        assert!(verify.issues.iter().any(|issue| {
            issue == "writer_backend_kind mismatch: expected mach-o-arm64, found elf-amd64"
        }));
    }

    #[test]
    fn emit_and_verify_object_writer_dry_run_artifact() {
        let dir = std::env::temp_dir().join(format!(
            "nsld-object-writer-dry-run-artifact-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();
        nsld_emit_object_report(Path::new("manifest.toml"), &plan).unwrap();

        let emit =
            nsld_emit_object_writer_dry_run_report(Path::new("manifest.toml"), &plan).unwrap();
        let verify = nsld_verify_object_writer_dry_run_report(Path::new("manifest.toml"), &plan);
        let dry_run_artifact =
            fs::read_to_string(dir.join("nuis.nsld.object-writer-dry-run.toml")).unwrap();
        fs::remove_dir_all(dir).unwrap();
        assert!(emit
            .output_path
            .ends_with("nuis.nsld.object-writer-dry-run.toml"));
        assert!(!emit.dry_run_ready);
        assert!(verify.valid);
        assert!(verify.issues.is_empty());
        assert!(dry_run_artifact.contains("kind = \"object-writer-dry-run\""));
    }

    #[test]
    fn verify_object_writer_dry_run_reports_tampered_ready_flag() {
        let dir = std::env::temp_dir().join(format!(
            "nsld-object-writer-dry-run-tamper-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();
        nsld_emit_object_report(Path::new("manifest.toml"), &plan).unwrap();
        nsld_emit_object_writer_dry_run_report(Path::new("manifest.toml"), &plan).unwrap();
        let input_path = dir.join("nuis.nsld.object-writer-dry-run.toml");
        let damaged = fs::read_to_string(&input_path)
            .unwrap()
            .replace("dry_run_ready = false", "dry_run_ready = true");
        fs::write(&input_path, damaged).unwrap();

        let verify = nsld_verify_object_writer_dry_run_report(Path::new("manifest.toml"), &plan);
        fs::remove_dir_all(dir).unwrap();

        assert!(!verify.valid);
        assert!(verify
            .issues
            .iter()
            .any(|issue| issue == "dry_run_ready mismatch: expected false, found true"));
    }
}
