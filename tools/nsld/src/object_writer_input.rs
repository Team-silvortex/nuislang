use super::{
    container_verify::{self, TomlFieldKind},
    object_plan::nsld_object_plan_report,
    object_plan_verify::{
        object_section_table_mismatch_issues, relocation_seed_table_mismatch_issues,
        toml_block_bool_value, toml_block_isize_value, toml_block_string_value,
        toml_block_usize_value, toml_table_blocks,
    },
    reports::{
        NsldObjectRelocationSeedDiagnostic, NsldObjectSectionDiagnostic,
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
        actual_object_layout_hash,
        actual_relocation_seed_table_hash,
        actual_section_count,
        actual_relocation_seed_count,
    ) = match actual.as_ref() {
        Ok(source) => (
            toml::string_value(source, "object_plan_hash"),
            toml::string_value(source, "object_layout_hash"),
            toml::string_value(source, "relocation_seed_table_hash"),
            toml::usize_value(source, "section_count"),
            toml::usize_value(source, "relocation_seed_count"),
        ),
        Err(error) => {
            issues.push(error.clone());
            (None, None, None, None, None)
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
        expected_object_layout_hash: expected_report.object_layout_hash,
        expected_relocation_seed_table_hash: expected_report.relocation_seed_table_hash,
        expected_section_count: expected_report.section_count,
        expected_relocation_seed_count: expected_report.relocation_seed_count,
        actual_object_plan_hash,
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
    let can_emit_object = object_plan.ready
        && verify.valid
        && object_plan.unsupported_features.is_empty()
        && blockers.is_empty();

    NsldObjectWriterDryRunReport {
        manifest: manifest.display().to_string(),
        writer_input_path: verify.input_path,
        planned_output_path: PathBuf::from(&plan.output_dir)
            .join(format!("nuis.nsld.{}", object_plan.object_format))
            .display()
            .to_string(),
        writer_target_id: object_plan.writer_target_id,
        object_plan_hash: object_plan.object_plan_hash,
        object_layout_hash: object_plan.object_layout_hash,
        relocation_seed_table_hash: object_plan.relocation_seed_table_hash,
        section_count: object_plan.section_count,
        relocation_seed_count: object_plan.relocation_seed_count,
        writer_input_valid: verify.valid,
        can_emit_object,
        dry_run_ready: can_emit_object,
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
        actual_object_layout_hash,
        actual_relocation_seed_table_hash,
        actual_section_count,
        actual_relocation_seed_count,
        actual_dry_run_ready,
    ) = match actual.as_ref() {
        Ok(source) => (
            toml::string_value(source, "object_plan_hash"),
            toml::string_value(source, "object_layout_hash"),
            toml::string_value(source, "relocation_seed_table_hash"),
            toml::usize_value(source, "section_count"),
            toml::usize_value(source, "relocation_seed_count"),
            toml::bool_value(source, "dry_run_ready"),
        ),
        Err(error) => {
            issues.push(error.clone());
            (None, None, None, None, None, None)
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
        expected_object_layout_hash: expected_report.object_layout_hash,
        expected_relocation_seed_table_hash: expected_report.relocation_seed_table_hash,
        expected_section_count: expected_report.section_count,
        expected_relocation_seed_count: expected_report.relocation_seed_count,
        expected_dry_run_ready: expected_report.dry_run_ready,
        actual_object_plan_hash,
        actual_object_layout_hash,
        actual_relocation_seed_table_hash,
        actual_section_count,
        actual_relocation_seed_count,
        actual_dry_run_ready,
        issues,
    }
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

fn push_usize_mismatch(
    issues: &mut Vec<String>,
    field: &str,
    expected: usize,
    actual: Option<usize>,
) {
    if actual != Some(expected) {
        issues.push(format!(
            "{field} mismatch: expected {expected}, found {}",
            actual
                .map(|value| value.to_string())
                .unwrap_or_else(|| "missing".to_owned())
        ));
    }
}

fn writer_input_section_table_field_issues(source: &str) -> Vec<String> {
    container_verify::table_field_issues(
        source,
        "writer_section",
        "writer_section",
        &[
            ("order_index", TomlFieldKind::Usize),
            ("source_section_id", TomlFieldKind::String),
            ("object_section_name", TomlFieldKind::String),
            ("object_section_role", TomlFieldKind::String),
            ("source_path", TomlFieldKind::String),
            ("source_hash", TomlFieldKind::String),
            ("source_size_bytes", TomlFieldKind::Usize),
            ("file_offset_seed", TomlFieldKind::Usize),
            ("file_size_seed", TomlFieldKind::Usize),
            ("alignment", TomlFieldKind::Usize),
            ("required", TomlFieldKind::Bool),
        ],
    )
}

fn writer_input_relocation_seed_table_field_issues(source: &str) -> Vec<String> {
    container_verify::table_field_issues(
        source,
        "writer_relocation_seed",
        "writer_relocation_seed",
        &[
            ("order_index", TomlFieldKind::Usize),
            ("relocation_seed_id", TomlFieldKind::String),
            ("relocation_seed_kind", TomlFieldKind::String),
            ("source_section_id", TomlFieldKind::String),
            ("source_offset_seed", TomlFieldKind::Usize),
            ("target_symbol", TomlFieldKind::String),
            ("addend", TomlFieldKind::Isize),
            ("native_relocation_ready", TomlFieldKind::Bool),
        ],
    )
}

fn writer_section_entries(source: &str) -> Vec<NsldObjectSectionDiagnostic> {
    toml_table_blocks(source, "writer_section")
        .into_iter()
        .filter_map(|block| {
            Some(NsldObjectSectionDiagnostic {
                order_index: toml_block_usize_value(&block, "order_index")?,
                source_section_id: toml_block_string_value(&block, "source_section_id")?,
                source_section_kind: String::new(),
                object_section_name: toml_block_string_value(&block, "object_section_name")?,
                object_section_role: toml_block_string_value(&block, "object_section_role")?,
                source_path: toml_block_string_value(&block, "source_path")?,
                source_hash: toml_block_string_value(&block, "source_hash")?,
                source_size_bytes: toml_block_usize_value(&block, "source_size_bytes")?,
                payload_offset_seed: 0,
                file_offset_seed: toml_block_usize_value(&block, "file_offset_seed")?,
                file_size_seed: toml_block_usize_value(&block, "file_size_seed")?,
                alignment: toml_block_usize_value(&block, "alignment")?,
                required: toml_block_bool_value(&block, "required")?,
            })
        })
        .collect()
}

fn writer_relocation_seed_entries(source: &str) -> Vec<NsldObjectRelocationSeedDiagnostic> {
    toml_table_blocks(source, "writer_relocation_seed")
        .into_iter()
        .filter_map(|block| {
            Some(NsldObjectRelocationSeedDiagnostic {
                order_index: toml_block_usize_value(&block, "order_index")?,
                relocation_seed_id: toml_block_string_value(&block, "relocation_seed_id")?,
                relocation_seed_kind: toml_block_string_value(&block, "relocation_seed_kind")?,
                source_section_id: toml_block_string_value(&block, "source_section_id")?,
                source_offset_seed: toml_block_usize_value(&block, "source_offset_seed")?,
                target_symbol: toml_block_string_value(&block, "target_symbol")?,
                addend: toml_block_isize_value(&block, "addend")?,
                native_relocation_ready: toml_block_bool_value(&block, "native_relocation_ready")?,
            })
        })
        .collect()
}

fn writer_section_table_mismatch_issues(
    expected: &[NsldObjectSectionDiagnostic],
    actual: &[NsldObjectSectionDiagnostic],
) -> Vec<String> {
    object_section_table_mismatch_issues(expected, actual)
        .into_iter()
        .filter(|issue| {
            !issue.contains(".source_section_kind mismatch")
                && !issue.contains(".payload_offset_seed mismatch")
        })
        .map(|issue| {
            issue
                .replace("object_section_entry_count", "writer_section_entry_count")
                .replace("object_section[", "writer_section[")
        })
        .collect()
}

fn writer_relocation_seed_table_mismatch_issues(
    expected: &[NsldObjectRelocationSeedDiagnostic],
    actual: &[NsldObjectRelocationSeedDiagnostic],
) -> Vec<String> {
    relocation_seed_table_mismatch_issues(expected, actual)
        .into_iter()
        .map(|issue| {
            issue
                .replace(
                    "object_relocation_seed_entry_count",
                    "writer_relocation_seed_entry_count",
                )
                .replace("object_relocation_seed[", "writer_relocation_seed[")
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::nsld_object_writer_dry_run_report;
    use super::{nsld_emit_object_writer_dry_run_report, nsld_verify_object_writer_dry_run_report};
    use crate::{main_test_support::empty_link_plan, object_plan::nsld_emit_object_report};
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
    fn dry_run_consumes_emitted_writer_input_but_stays_blocked_without_writer() {
        let dir = std::env::temp_dir().join(format!(
            "nsld-object-writer-dry-run-blocked-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();
        nsld_emit_object_report(Path::new("manifest.toml"), &plan).unwrap();

        let report = nsld_object_writer_dry_run_report(Path::new("manifest.toml"), &plan);
        fs::remove_dir_all(dir).unwrap();

        assert!(report.writer_input_valid);
        assert!(!report.can_emit_object);
        assert!(!report.dry_run_ready);
        assert_eq!(report.section_count, 4);
        assert_eq!(report.relocation_seed_count, 4);
        assert!(report.planned_output_path.ends_with("nuis.nsld.mach-o"));
        assert!(report
            .blockers
            .contains(&"object-byte-emitter:not-implemented".to_owned()));
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
