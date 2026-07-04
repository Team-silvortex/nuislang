use super::{
    container_verify::{self, TomlFieldKind},
    fnv1a64_hex,
    object_plan::nsld_object_plan_report,
    object_plan_verify::{toml_block_string_value, toml_block_usize_value, toml_table_blocks},
    object_writer_input::nsld_object_writer_dry_run_report,
    reports::{
        NsldObjectByteLayoutEmitReport, NsldObjectByteLayoutReport,
        NsldObjectByteLayoutVerifyReport, NsldObjectByteSectionDiagnostic,
    },
    toml,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(crate) fn nsld_object_byte_layout_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldObjectByteLayoutReport {
    let object_plan = nsld_object_plan_report(manifest, plan);
    let dry_run = nsld_object_writer_dry_run_report(manifest, plan);
    let sections = object_plan
        .object_sections
        .iter()
        .map(|section| NsldObjectByteSectionDiagnostic {
            order_index: section.order_index,
            source_section_id: section.source_section_id.clone(),
            object_section_name: section.object_section_name.clone(),
            file_offset: section.file_offset_seed,
            size_bytes: section.file_size_seed,
            alignment: section.alignment,
            source_hash: section.source_hash.clone(),
        })
        .collect::<Vec<_>>();
    let total_size_bytes = sections
        .iter()
        .map(|section| section.file_offset.saturating_add(section.size_bytes))
        .max()
        .unwrap_or(0);
    let byte_layout_hash = nsld_object_byte_layout_hash(
        &object_plan.writer_target_id,
        &object_plan.writer_backend_kind,
        &object_plan.object_family,
        &object_plan.object_format,
        &sections,
        total_size_bytes,
    );
    let blockers = dry_run.blockers;
    let layout_ready = dry_run.dry_run_ready && blockers.is_empty();

    NsldObjectByteLayoutReport {
        manifest: manifest.display().to_string(),
        output_path: PathBuf::from(&plan.output_dir)
            .join("nuis.nsld.object-byte-layout.toml")
            .display()
            .to_string(),
        writer_target_id: object_plan.writer_target_id,
        writer_backend_kind: object_plan.writer_backend_kind,
        object_family: object_plan.object_family,
        object_format: object_plan.object_format,
        object_plan_hash: object_plan.object_plan_hash,
        object_layout_hash: object_plan.object_layout_hash,
        byte_layout_hash,
        section_count: sections.len(),
        total_size_bytes,
        layout_ready,
        sections,
        blockers,
    }
}

pub(crate) fn nsld_emit_object_byte_layout_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<NsldObjectByteLayoutEmitReport, String> {
    let report = nsld_object_byte_layout_report(manifest, plan);
    fs::write(
        &report.output_path,
        toml::render_object_byte_layout(&report),
    )
    .map_err(|error| {
        format!(
            "failed to write nsld object byte layout `{}`: {error}",
            report.output_path
        )
    })?;

    Ok(NsldObjectByteLayoutEmitReport {
        manifest: report.manifest,
        output_path: report.output_path,
        layout_ready: report.layout_ready,
        byte_layout_hash: report.byte_layout_hash,
        section_count: report.section_count,
        total_size_bytes: report.total_size_bytes,
    })
}

pub(crate) fn nsld_verify_object_byte_layout_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldObjectByteLayoutVerifyReport {
    let expected_report = nsld_object_byte_layout_report(manifest, plan);
    let expected = toml::render_object_byte_layout(&expected_report);
    let input_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.object-byte-layout.toml");
    let mut issues = Vec::new();
    let actual = fs::read_to_string(&input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_object_byte_layout `{}`: {error}",
            input_path.display()
        )
    });
    let (actual_byte_layout_hash, actual_section_count, actual_total_size_bytes) =
        match actual.as_ref() {
            Ok(source) => (
                toml::string_value(source, "byte_layout_hash"),
                toml::usize_value(source, "section_count"),
                toml::usize_value(source, "total_size_bytes"),
            ),
            Err(error) => {
                issues.push(error.clone());
                (None, None, None)
            }
        };
    if let Ok(actual) = actual {
        if actual != expected {
            issues.push("object-byte-layout-content-mismatch".to_owned());
        }
        issues.extend(byte_section_table_field_issues(&actual));
        issues.extend(byte_section_table_mismatch_issues(
            &expected_report.sections,
            &byte_section_entries(&actual),
        ));
        if actual_byte_layout_hash.as_deref() != Some(expected_report.byte_layout_hash.as_str()) {
            issues.push(format!(
                "byte_layout_hash mismatch: expected {}, found {}",
                expected_report.byte_layout_hash,
                actual_byte_layout_hash
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
        if actual_total_size_bytes != Some(expected_report.total_size_bytes) {
            issues.push(format!(
                "total_size_bytes mismatch: expected {}, found {}",
                expected_report.total_size_bytes,
                actual_total_size_bytes
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
    }

    NsldObjectByteLayoutVerifyReport {
        manifest: manifest.display().to_string(),
        input_path: input_path.display().to_string(),
        valid: issues.is_empty(),
        expected_byte_layout_hash: expected_report.byte_layout_hash,
        expected_section_count: expected_report.section_count,
        expected_total_size_bytes: expected_report.total_size_bytes,
        actual_byte_layout_hash,
        actual_section_count,
        actual_total_size_bytes,
        issues,
    }
}

fn byte_section_table_field_issues(source: &str) -> Vec<String> {
    container_verify::table_field_issues(
        source,
        "byte_section",
        "byte_section",
        &[
            ("order_index", TomlFieldKind::Usize),
            ("source_section_id", TomlFieldKind::String),
            ("object_section_name", TomlFieldKind::String),
            ("file_offset", TomlFieldKind::Usize),
            ("size_bytes", TomlFieldKind::Usize),
            ("alignment", TomlFieldKind::Usize),
            ("source_hash", TomlFieldKind::String),
        ],
    )
}

fn byte_section_entries(source: &str) -> Vec<NsldObjectByteSectionDiagnostic> {
    toml_table_blocks(source, "byte_section")
        .into_iter()
        .filter_map(|block| {
            Some(NsldObjectByteSectionDiagnostic {
                order_index: toml_block_usize_value(&block, "order_index")?,
                source_section_id: toml_block_string_value(&block, "source_section_id")?,
                object_section_name: toml_block_string_value(&block, "object_section_name")?,
                file_offset: toml_block_usize_value(&block, "file_offset")?,
                size_bytes: toml_block_usize_value(&block, "size_bytes")?,
                alignment: toml_block_usize_value(&block, "alignment")?,
                source_hash: toml_block_string_value(&block, "source_hash")?,
            })
        })
        .collect()
}

fn byte_section_table_mismatch_issues(
    expected: &[NsldObjectByteSectionDiagnostic],
    actual: &[NsldObjectByteSectionDiagnostic],
) -> Vec<String> {
    let mut issues = Vec::new();
    if actual.len() != expected.len() {
        issues.push(format!(
            "byte_section_entry_count mismatch: expected {}, found {}",
            expected.len(),
            actual.len()
        ));
    }
    for (index, expected_entry) in expected.iter().enumerate() {
        let Some(actual_entry) = actual.get(index) else {
            issues.push(format!("byte_section[{index}] missing"));
            continue;
        };
        push_usize_field_mismatch(
            &mut issues,
            index,
            "order_index",
            expected_entry.order_index,
            actual_entry.order_index,
        );
        push_string_field_mismatch(
            &mut issues,
            index,
            "source_section_id",
            &expected_entry.source_section_id,
            &actual_entry.source_section_id,
        );
        push_string_field_mismatch(
            &mut issues,
            index,
            "object_section_name",
            &expected_entry.object_section_name,
            &actual_entry.object_section_name,
        );
        push_usize_field_mismatch(
            &mut issues,
            index,
            "file_offset",
            expected_entry.file_offset,
            actual_entry.file_offset,
        );
        push_usize_field_mismatch(
            &mut issues,
            index,
            "size_bytes",
            expected_entry.size_bytes,
            actual_entry.size_bytes,
        );
        push_usize_field_mismatch(
            &mut issues,
            index,
            "alignment",
            expected_entry.alignment,
            actual_entry.alignment,
        );
        push_string_field_mismatch(
            &mut issues,
            index,
            "source_hash",
            &expected_entry.source_hash,
            &actual_entry.source_hash,
        );
    }
    issues
}

fn push_usize_field_mismatch(
    issues: &mut Vec<String>,
    index: usize,
    field: &str,
    expected: usize,
    actual: usize,
) {
    if actual != expected {
        issues.push(format!(
            "byte_section[{index}].{field} mismatch: expected {expected}, found {actual}"
        ));
    }
}

fn push_string_field_mismatch(
    issues: &mut Vec<String>,
    index: usize,
    field: &str,
    expected: &str,
    actual: &str,
) {
    if actual != expected {
        issues.push(format!(
            "byte_section[{index}].{field} mismatch: expected {expected}, found {actual}"
        ));
    }
}

fn nsld_object_byte_layout_hash(
    writer_target_id: &str,
    writer_backend_kind: &str,
    object_family: &str,
    object_format: &str,
    sections: &[NsldObjectByteSectionDiagnostic],
    total_size_bytes: usize,
) -> String {
    let mut material = format!(
        "writer_target_id={writer_target_id}\nwriter_backend_kind={writer_backend_kind}\nobject_family={object_family}\nobject_format={object_format}\ntotal_size_bytes={total_size_bytes}\n"
    );
    for section in sections {
        material.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            section.order_index,
            section.source_section_id,
            section.object_section_name,
            section.file_offset,
            section.size_bytes,
            section.alignment,
            section.source_hash
        ));
    }
    fnv1a64_hex(material.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::{
        nsld_emit_object_byte_layout_report, nsld_object_byte_layout_report,
        nsld_verify_object_byte_layout_report,
    };
    use crate::{
        main_test_support::empty_link_plan, object_emit::nsld_emit_object_report,
        object_writer_input::nsld_emit_object_writer_dry_run_report,
    };
    use std::{fs, path::Path};

    #[test]
    fn emits_and_verifies_object_byte_layout() {
        let dir =
            std::env::temp_dir().join(format!("nsld-object-byte-layout-ok-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();
        nsld_emit_object_report(Path::new("manifest.toml"), &plan).unwrap();
        nsld_emit_object_writer_dry_run_report(Path::new("manifest.toml"), &plan).unwrap();

        let emit = nsld_emit_object_byte_layout_report(Path::new("manifest.toml"), &plan).unwrap();
        let verify = nsld_verify_object_byte_layout_report(Path::new("manifest.toml"), &plan);
        fs::remove_dir_all(dir).unwrap();

        assert!(emit
            .output_path
            .ends_with("nuis.nsld.object-byte-layout.toml"));
        assert!(emit.byte_layout_hash.starts_with("0x"));
        assert_eq!(emit.section_count, 4);
        assert!(verify.valid);
        assert!(verify.issues.is_empty());
    }

    #[test]
    fn object_byte_layout_serializes_writer_identity() {
        let plan = empty_link_plan();
        let report = nsld_object_byte_layout_report(Path::new("manifest.toml"), &plan);
        let rendered = crate::toml::render_object_byte_layout(&report);
        let json = crate::json_object::nsld_object_byte_layout_report_json(&report);

        assert_eq!(report.writer_target_id, "arm64-macos-mach-o");
        assert_eq!(report.writer_backend_kind, "mach-o-arm64");
        assert_eq!(report.object_family, "mach-o");
        assert!(rendered.contains("writer_backend_kind = \"mach-o-arm64\""));
        assert!(rendered.contains("object_family = \"mach-o\""));
        assert!(json.contains("\"writer_backend_kind\":\"mach-o-arm64\""));
        assert!(json.contains("\"object_family\":\"mach-o\""));
    }

    #[test]
    fn verify_object_byte_layout_reports_hash_tamper() {
        let dir = std::env::temp_dir().join(format!(
            "nsld-object-byte-layout-tamper-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();
        nsld_emit_object_report(Path::new("manifest.toml"), &plan).unwrap();
        nsld_emit_object_writer_dry_run_report(Path::new("manifest.toml"), &plan).unwrap();
        nsld_emit_object_byte_layout_report(Path::new("manifest.toml"), &plan).unwrap();
        let path = dir.join("nuis.nsld.object-byte-layout.toml");
        let damaged = fs::read_to_string(&path)
            .unwrap()
            .replace("byte_layout_hash = \"0x", "byte_layout_hash = \"0y");
        fs::write(&path, damaged).unwrap();

        let verify = nsld_verify_object_byte_layout_report(Path::new("manifest.toml"), &plan);
        fs::remove_dir_all(dir).unwrap();

        assert!(!verify.valid);
        assert!(verify
            .issues
            .iter()
            .any(|issue| issue.starts_with("byte_layout_hash mismatch: expected 0x")));
    }

    #[test]
    fn verify_object_byte_layout_reports_missing_byte_section_field() {
        let dir = std::env::temp_dir().join(format!(
            "nsld-object-byte-layout-missing-field-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();
        nsld_emit_object_report(Path::new("manifest.toml"), &plan).unwrap();
        nsld_emit_object_writer_dry_run_report(Path::new("manifest.toml"), &plan).unwrap();
        nsld_emit_object_byte_layout_report(Path::new("manifest.toml"), &plan).unwrap();
        let path = dir.join("nuis.nsld.object-byte-layout.toml");
        let damaged =
            fs::read_to_string(&path)
                .unwrap()
                .replacen("\nsize_bytes = ", "\n# size_bytes = ", 1);
        fs::write(&path, damaged).unwrap();

        let verify = nsld_verify_object_byte_layout_report(Path::new("manifest.toml"), &plan);
        fs::remove_dir_all(dir).unwrap();

        assert!(!verify.valid);
        assert!(verify
            .issues
            .iter()
            .any(|issue| issue == "byte_section[0].size_bytes missing"));
    }

    #[test]
    fn verify_object_byte_layout_reports_byte_section_offset_drift() {
        let dir = std::env::temp_dir().join(format!(
            "nsld-object-byte-layout-offset-drift-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();
        nsld_emit_object_report(Path::new("manifest.toml"), &plan).unwrap();
        nsld_emit_object_writer_dry_run_report(Path::new("manifest.toml"), &plan).unwrap();
        nsld_emit_object_byte_layout_report(Path::new("manifest.toml"), &plan).unwrap();
        let path = dir.join("nuis.nsld.object-byte-layout.toml");
        let damaged = fs::read_to_string(&path)
            .unwrap()
            .replace("file_offset = 0", "file_offset = 8");
        fs::write(&path, damaged).unwrap();

        let verify = nsld_verify_object_byte_layout_report(Path::new("manifest.toml"), &plan);
        fs::remove_dir_all(dir).unwrap();

        assert!(!verify.valid);
        assert!(verify
            .issues
            .iter()
            .any(|issue| issue == "byte_section[0].file_offset mismatch: expected 0, found 8"));
    }

    #[test]
    fn verify_object_byte_layout_reports_byte_section_name_drift() {
        let dir = std::env::temp_dir().join(format!(
            "nsld-object-byte-layout-name-drift-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();
        nsld_emit_object_report(Path::new("manifest.toml"), &plan).unwrap();
        nsld_emit_object_writer_dry_run_report(Path::new("manifest.toml"), &plan).unwrap();
        nsld_emit_object_byte_layout_report(Path::new("manifest.toml"), &plan).unwrap();
        let path = dir.join("nuis.nsld.object-byte-layout.toml");
        let damaged = fs::read_to_string(&path).unwrap().replace(
            "object_section_name = \".nuis.text.compiled\"",
            "object_section_name = \".nuis.text.drift\"",
        );
        fs::write(&path, damaged).unwrap();

        let verify = nsld_verify_object_byte_layout_report(Path::new("manifest.toml"), &plan);
        fs::remove_dir_all(dir).unwrap();

        assert!(!verify.valid);
        assert!(verify.issues.iter().any(|issue| {
            issue
                == "byte_section[0].object_section_name mismatch: expected .nuis.text.compiled, found .nuis.text.drift"
        }));
    }
}
