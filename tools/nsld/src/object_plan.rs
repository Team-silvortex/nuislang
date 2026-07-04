use super::{
    assembly::nsld_section_manifest_report,
    object_layout::{
        nsld_object_layout_hash, nsld_object_plan_hash, nsld_relocation_seed_table_hash,
        object_relocation_seeds, object_section_layout,
    },
    object_plan_verify::{
        object_section_entries, object_section_table_field_issues,
        object_section_table_mismatch_issues, relocation_seed_entries,
        relocation_seed_table_field_issues, relocation_seed_table_mismatch_issues,
    },
    object_writer_backend::{object_format_family, object_writer_backend, object_writer_blockers},
    reports::{NsldObjectPlanEmitReport, NsldObjectPlanReport, NsldObjectPlanVerifyReport},
    toml,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(crate) fn nsld_object_plan_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldObjectPlanReport {
    let section_manifest = nsld_section_manifest_report(manifest, plan);
    let output_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.object-plan.toml");
    let source_container_path = PathBuf::from(&plan.output_dir)
        .join("nuis.nsld.container")
        .display()
        .to_string();
    let source_payload_path = PathBuf::from(&plan.output_dir)
        .join("nuis.nsld.container.payload")
        .display()
        .to_string();
    let backend = object_writer_backend(
        &plan.cpu_target.machine_arch,
        &plan.cpu_target.machine_os,
        &plan.cpu_target.object_format,
    );
    let object_family = object_format_family(&plan.cpu_target.object_format).to_owned();
    let blockers = object_writer_blockers(&backend, &section_manifest.blockers);
    let object_sections = object_section_layout(&section_manifest.sections);
    let relocation_seeds = object_relocation_seeds(&object_sections);
    let object_layout_hash = nsld_object_layout_hash(&object_sections);
    let relocation_seed_table_hash = nsld_relocation_seed_table_hash(&relocation_seeds);
    let object_plan_hash = nsld_object_plan_hash(
        &plan.cpu_target.machine_arch,
        &plan.cpu_target.machine_os,
        &plan.cpu_target.object_format,
        &object_family,
        &backend.target_id,
        &backend.backend_kind,
        &section_manifest.section_table_hash,
        &object_layout_hash,
        &relocation_seed_table_hash,
        &source_container_path,
        &source_payload_path,
        &object_sections,
        &relocation_seeds,
        &blockers,
    );

    NsldObjectPlanReport {
        manifest: manifest.display().to_string(),
        ready: section_manifest.ready && blockers.is_empty(),
        target_arch: plan.cpu_target.machine_arch.clone(),
        target_os: plan.cpu_target.machine_os.clone(),
        object_format: plan.cpu_target.object_format.clone(),
        calling_abi: plan.cpu_target.calling_abi.clone(),
        clang_target: plan.cpu_target.clang_target.clone(),
        output_path: output_path.display().to_string(),
        source_container_path,
        source_payload_path,
        section_count: section_manifest.section_count,
        section_table_hash: section_manifest.section_table_hash,
        object_plan_hash,
        object_layout_hash,
        relocation_seed_count: relocation_seeds.len(),
        relocation_seed_table_hash,
        writer_target_id: backend.target_id,
        writer_backend_kind: backend.backend_kind,
        writer_status: backend.status,
        object_family,
        unsupported_features: backend.unsupported_features,
        emission_status: "plan-only".to_owned(),
        object_sections,
        relocation_seeds,
        blockers,
    }
}

pub(crate) fn nsld_emit_object_plan_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<NsldObjectPlanEmitReport, String> {
    let report = nsld_object_plan_report(manifest, plan);
    fs::write(&report.output_path, toml::render_object_plan(&report)).map_err(|error| {
        format!(
            "failed to write nsld object plan `{}`: {error}",
            report.output_path
        )
    })?;

    Ok(NsldObjectPlanEmitReport {
        manifest: report.manifest,
        output_path: report.output_path,
        ready: report.ready,
        object_plan_hash: report.object_plan_hash,
        section_count: report.section_count,
    })
}

pub(crate) fn nsld_verify_object_plan_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldObjectPlanVerifyReport {
    let expected_report = nsld_object_plan_report(manifest, plan);
    let expected = toml::render_object_plan(&expected_report);
    let input_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.object-plan.toml");
    let mut issues = Vec::new();
    let actual = fs::read_to_string(&input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_object_plan `{}`: {error}",
            input_path.display()
        )
    });
    let (actual_object_plan_hash, actual_section_count) = match actual.as_ref() {
        Ok(source) => (
            toml::string_value(source, "object_plan_hash"),
            toml::usize_value(source, "section_count"),
        ),
        Err(error) => {
            issues.push(error.clone());
            (None, None)
        }
    };
    if let Ok(actual) = actual {
        if actual != expected {
            issues.push("object-plan-content-mismatch".to_owned());
        }
        issues.extend(object_section_table_field_issues(&actual));
        issues.extend(object_section_table_mismatch_issues(
            &expected_report.object_sections,
            &object_section_entries(&actual),
        ));
        issues.extend(relocation_seed_table_field_issues(&actual));
        issues.extend(relocation_seed_table_mismatch_issues(
            &expected_report.relocation_seeds,
            &relocation_seed_entries(&actual),
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
        if actual_section_count != Some(expected_report.section_count) {
            issues.push(format!(
                "section_count mismatch: expected {}, found {}",
                expected_report.section_count,
                actual_section_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
    }

    NsldObjectPlanVerifyReport {
        manifest: manifest.display().to_string(),
        input_path: input_path.display().to_string(),
        valid: issues.is_empty(),
        expected_object_plan_hash: expected_report.object_plan_hash,
        expected_section_count: expected_report.section_count,
        actual_object_plan_hash,
        actual_section_count,
        issues,
    }
}

#[cfg(test)]
mod tests {
    use super::{nsld_object_plan_report, nsld_verify_object_plan_report};
    use crate::main_test_support::empty_link_plan;
    use std::{fs, path::Path};

    #[test]
    fn object_plan_reports_ready_mach_o_writer_identity() {
        let plan = empty_link_plan();
        let report = nsld_object_plan_report(Path::new("nuis.build.manifest.toml"), &plan);

        assert_eq!(report.object_format, "mach-o");
        assert_eq!(report.emission_status, "plan-only");
        assert_eq!(report.writer_target_id, "arm64-macos-mach-o");
        assert_eq!(report.writer_backend_kind, "mach-o-arm64");
        assert_eq!(report.writer_status, "ready");
        assert_eq!(report.object_family, "mach-o");
        assert!(report.unsupported_features.is_empty());
        assert_eq!(
            report.object_sections[0].object_section_name,
            ".nuis.text.compiled"
        );
        assert_eq!(
            report.object_sections[0].object_section_role,
            "native-bootstrap-input"
        );
        assert_eq!(report.object_sections[0].alignment, 16);
        assert_eq!(report.object_sections[0].file_offset_seed, 0);
        assert_eq!(report.relocation_seed_count, report.relocation_seeds.len());
        assert!(report.object_layout_hash.starts_with("0x"));
        assert!(report.relocation_seed_table_hash.starts_with("0x"));
        assert_eq!(
            report.relocation_seeds[0].relocation_seed_kind,
            "bootstrap-entry-seed"
        );
        assert!(!report.relocation_seeds[0].native_relocation_ready);
        assert!(report
            .blockers
            .iter()
            .any(|blocker| blocker.contains("section:nsld-link-input-table:")));
    }

    #[test]
    fn verify_object_plan_reports_missing_object_section_fields() {
        let dir = std::env::temp_dir().join(format!(
            "nsld-object-plan-field-tamper-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let artifact_path = dir.join("nuis.compiled.artifact");
        fs::write(&artifact_path, b"compiled-artifact").unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();
        plan.compiled_artifact.path = artifact_path.display().to_string();
        let report = nsld_object_plan_report(Path::new("manifest.toml"), &plan);
        let damaged = crate::toml::render_object_plan(&report)
            .replace("object_section_role = \"", "# object_section_role = \"");
        fs::write(dir.join("nuis.nsld.object-plan.toml"), damaged).unwrap();

        let verify = nsld_verify_object_plan_report(Path::new("manifest.toml"), &plan);
        fs::remove_dir_all(dir).unwrap();

        assert!(!verify.valid);
        assert!(verify
            .issues
            .iter()
            .any(|issue| issue == "object_section[0].object_section_role missing"));
    }

    #[test]
    fn object_plan_serializes_writer_backend_identity() {
        let plan = empty_link_plan();
        let report = nsld_object_plan_report(Path::new("nuis.build.manifest.toml"), &plan);
        let rendered = crate::toml::render_object_plan(&report);
        let json = crate::json_object::nsld_object_plan_report_json(&report);

        assert!(rendered.contains("writer_backend_kind = \"mach-o-arm64\""));
        assert!(rendered.contains("object_family = \"mach-o\""));
        assert!(json.contains("\"writer_backend_kind\":\"mach-o-arm64\""));
        assert!(json.contains("\"object_family\":\"mach-o\""));
    }

    #[test]
    fn verify_object_plan_reports_object_section_name_drift() {
        let dir = std::env::temp_dir().join(format!(
            "nsld-object-plan-section-drift-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let artifact_path = dir.join("nuis.compiled.artifact");
        fs::write(&artifact_path, b"compiled-artifact").unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();
        plan.compiled_artifact.path = artifact_path.display().to_string();
        let report = nsld_object_plan_report(Path::new("manifest.toml"), &plan);
        let damaged = crate::toml::render_object_plan(&report)
            .replace(".nuis.text.compiled", ".nuis.text.wrong");
        fs::write(dir.join("nuis.nsld.object-plan.toml"), damaged).unwrap();

        let verify = nsld_verify_object_plan_report(Path::new("manifest.toml"), &plan);
        fs::remove_dir_all(dir).unwrap();

        assert!(!verify.valid);
        assert!(verify.issues.iter().any(|issue| {
            issue
                == "object_section[0].object_section_name mismatch: expected .nuis.text.compiled, found .nuis.text.wrong"
        }));
    }

    #[test]
    fn verify_object_plan_reports_relocation_seed_drift() {
        let dir = std::env::temp_dir().join(format!(
            "nsld-object-plan-relocation-seed-drift-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let artifact_path = dir.join("nuis.compiled.artifact");
        fs::write(&artifact_path, b"compiled-artifact").unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();
        plan.compiled_artifact.path = artifact_path.display().to_string();
        let report = nsld_object_plan_report(Path::new("manifest.toml"), &plan);
        let damaged = crate::toml::render_object_plan(&report).replace(
            "relocation_seed_kind = \"bootstrap-entry-seed\"",
            "relocation_seed_kind = \"wrong-seed\"",
        );
        fs::write(dir.join("nuis.nsld.object-plan.toml"), damaged).unwrap();

        let verify = nsld_verify_object_plan_report(Path::new("manifest.toml"), &plan);
        fs::remove_dir_all(dir).unwrap();

        assert!(!verify.valid);
        assert!(verify.issues.iter().any(|issue| {
            issue
                == "object_relocation_seed[0].relocation_seed_kind mismatch: expected bootstrap-entry-seed, found wrong-seed"
        }));
    }

    #[test]
    fn verify_object_plan_reports_missing_writer_header_fields() {
        let dir = std::env::temp_dir().join(format!(
            "nsld-object-plan-writer-header-tamper-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let artifact_path = dir.join("nuis.compiled.artifact");
        fs::write(&artifact_path, b"compiled-artifact").unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();
        plan.compiled_artifact.path = artifact_path.display().to_string();
        let report = nsld_object_plan_report(Path::new("manifest.toml"), &plan);
        let damaged = crate::toml::render_object_plan(&report)
            .replace("writer_status = \"", "# writer_status = \"");
        fs::write(dir.join("nuis.nsld.object-plan.toml"), damaged).unwrap();

        let verify = nsld_verify_object_plan_report(Path::new("manifest.toml"), &plan);
        fs::remove_dir_all(dir).unwrap();

        assert!(!verify.valid);
        assert!(verify
            .issues
            .iter()
            .any(|issue| issue == "object_plan_header[0].writer_status missing"));
    }
}
