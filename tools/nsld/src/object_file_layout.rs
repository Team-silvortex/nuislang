use super::{
    container_verify::{self, TomlFieldKind},
    fnv1a64_hex,
    object_byte_layout::nsld_object_byte_layout_report,
    object_plan::nsld_object_plan_report,
    object_plan_verify::{toml_block_string_value, toml_block_usize_value, toml_table_blocks},
    reports::{
        NsldObjectFileLayoutEmitReport, NsldObjectFileLayoutRecordDiagnostic,
        NsldObjectFileLayoutReport, NsldObjectFileLayoutVerifyReport,
    },
    toml,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(crate) fn nsld_object_file_layout_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldObjectFileLayoutReport {
    let object_plan = nsld_object_plan_report(manifest, plan);
    let byte_layout = nsld_object_byte_layout_report(manifest, plan);
    let records = match object_plan.writer_backend_kind.as_str() {
        "mach-o-arm64" => mach_o_arm64_file_layout_records(
            &byte_layout.sections,
            object_plan.relocation_seed_count,
        ),
        _ => generic_unknown_file_layout_records(),
    };
    let total_file_size_bytes = records
        .iter()
        .map(|record| record.file_offset.saturating_add(record.size_bytes))
        .max()
        .unwrap_or(0);
    let file_layout_hash = nsld_object_file_layout_hash(
        &object_plan.writer_target_id,
        &object_plan.writer_backend_kind,
        &object_plan.object_family,
        &object_plan.object_format,
        &records,
        total_file_size_bytes,
    );
    let layout_ready = byte_layout.layout_ready && byte_layout.blockers.is_empty();

    NsldObjectFileLayoutReport {
        manifest: manifest.display().to_string(),
        output_path: PathBuf::from(&plan.output_dir)
            .join("nuis.nsld.object-file-layout.toml")
            .display()
            .to_string(),
        writer_target_id: object_plan.writer_target_id,
        writer_backend_kind: object_plan.writer_backend_kind,
        object_family: object_plan.object_family,
        object_format: object_plan.object_format,
        object_plan_hash: object_plan.object_plan_hash,
        byte_layout_hash: byte_layout.byte_layout_hash,
        file_layout_hash,
        record_count: records.len(),
        total_file_size_bytes,
        layout_ready,
        records,
        blockers: byte_layout.blockers,
    }
}

pub(crate) fn nsld_emit_object_file_layout_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<NsldObjectFileLayoutEmitReport, String> {
    let report = nsld_object_file_layout_report(manifest, plan);
    fs::write(
        &report.output_path,
        toml::render_object_file_layout(&report),
    )
    .map_err(|error| {
        format!(
            "failed to write nsld object file layout `{}`: {error}",
            report.output_path
        )
    })?;

    Ok(NsldObjectFileLayoutEmitReport {
        manifest: report.manifest,
        output_path: report.output_path,
        layout_ready: report.layout_ready,
        file_layout_hash: report.file_layout_hash,
        record_count: report.record_count,
        total_file_size_bytes: report.total_file_size_bytes,
    })
}

pub(crate) fn nsld_verify_object_file_layout_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldObjectFileLayoutVerifyReport {
    let expected_report = nsld_object_file_layout_report(manifest, plan);
    let expected = toml::render_object_file_layout(&expected_report);
    let input_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.object-file-layout.toml");
    let mut issues = Vec::new();
    let actual = fs::read_to_string(&input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_object_file_layout `{}`: {error}",
            input_path.display()
        )
    });
    let (actual_file_layout_hash, actual_record_count, actual_total_file_size_bytes) =
        match actual.as_ref() {
            Ok(source) => (
                toml::string_value(source, "file_layout_hash"),
                toml::usize_value(source, "record_count"),
                toml::usize_value(source, "total_file_size_bytes"),
            ),
            Err(error) => {
                issues.push(error.clone());
                (None, None, None)
            }
        };
    if let Ok(actual) = actual {
        if actual != expected {
            issues.push("object-file-layout-content-mismatch".to_owned());
        }
        issues.extend(file_layout_record_table_field_issues(&actual));
        issues.extend(file_layout_record_table_mismatch_issues(
            &expected_report.records,
            &file_layout_record_entries(&actual),
        ));
        if actual_file_layout_hash.as_deref() != Some(expected_report.file_layout_hash.as_str()) {
            issues.push(format!(
                "file_layout_hash mismatch: expected {}, found {}",
                expected_report.file_layout_hash,
                actual_file_layout_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_record_count != Some(expected_report.record_count) {
            issues.push(format!(
                "record_count mismatch: expected {}, found {}",
                expected_report.record_count,
                actual_record_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_total_file_size_bytes != Some(expected_report.total_file_size_bytes) {
            issues.push(format!(
                "total_file_size_bytes mismatch: expected {}, found {}",
                expected_report.total_file_size_bytes,
                actual_total_file_size_bytes
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
    }

    NsldObjectFileLayoutVerifyReport {
        manifest: manifest.display().to_string(),
        input_path: input_path.display().to_string(),
        valid: issues.is_empty(),
        expected_file_layout_hash: expected_report.file_layout_hash,
        expected_record_count: expected_report.record_count,
        expected_total_file_size_bytes: expected_report.total_file_size_bytes,
        actual_file_layout_hash,
        actual_record_count,
        actual_total_file_size_bytes,
        issues,
    }
}

fn mach_o_arm64_file_layout_records(
    sections: &[super::reports::NsldObjectByteSectionDiagnostic],
    relocation_seed_count: usize,
) -> Vec<NsldObjectFileLayoutRecordDiagnostic> {
    let header_size = 32usize;
    let segment_command_size = 72usize;
    let section_command_size = 80usize.saturating_mul(sections.len());
    let symtab_command_size = 24usize;
    let load_commands_size = segment_command_size
        .saturating_add(section_command_size)
        .saturating_add(symtab_command_size);
    let metadata_size = header_size.saturating_add(load_commands_size);
    let mut records = vec![
        layout_record(0, "macho.header", "macho-header", 0, header_size, 8),
        layout_record(
            1,
            "macho.load_commands",
            "macho-load-commands",
            header_size,
            load_commands_size,
            8,
        ),
    ];
    let mut cursor = align_to(metadata_size, 16);
    for section in sections {
        cursor = align_to(cursor, section.alignment.max(1));
        records.push(layout_record(
            records.len(),
            &format!("section.{}", section.source_section_id),
            "section-payload",
            cursor,
            section.size_bytes,
            section.alignment.max(1),
        ));
        cursor = cursor.saturating_add(section.size_bytes);
    }

    cursor = align_to(cursor, 8);
    let relocation_size = relocation_seed_count.saturating_mul(8);
    records.push(layout_record(
        records.len(),
        "macho.relocations",
        "macho-relocation-table",
        cursor,
        relocation_size,
        8,
    ));
    cursor = cursor.saturating_add(relocation_size);

    cursor = align_to(cursor, 8);
    let symbol_count = sections.len().saturating_add(1);
    let symbol_table_size = symbol_count.saturating_mul(16);
    records.push(layout_record(
        records.len(),
        "macho.symbols",
        "macho-symbol-table",
        cursor,
        symbol_table_size,
        8,
    ));
    cursor = cursor.saturating_add(symbol_table_size);

    cursor = align_to(cursor, 1);
    let string_table_size = mach_o_string_table_size(sections);
    records.push(layout_record(
        records.len(),
        "macho.strings",
        "macho-string-table",
        cursor,
        string_table_size,
        1,
    ));

    records
}

fn generic_unknown_file_layout_records() -> Vec<NsldObjectFileLayoutRecordDiagnostic> {
    vec![layout_record(
        0,
        "object-file-layout.target-selection",
        "target-selection",
        0,
        0,
        1,
    )]
}

fn file_layout_record_table_field_issues(source: &str) -> Vec<String> {
    container_verify::table_field_issues(
        source,
        "file_layout_record",
        "file_layout_record",
        &[
            ("order_index", TomlFieldKind::Usize),
            ("record_id", TomlFieldKind::String),
            ("record_kind", TomlFieldKind::String),
            ("file_offset", TomlFieldKind::Usize),
            ("size_bytes", TomlFieldKind::Usize),
            ("alignment", TomlFieldKind::Usize),
        ],
    )
}

fn file_layout_record_entries(source: &str) -> Vec<NsldObjectFileLayoutRecordDiagnostic> {
    toml_table_blocks(source, "file_layout_record")
        .into_iter()
        .filter_map(|block| {
            Some(NsldObjectFileLayoutRecordDiagnostic {
                order_index: toml_block_usize_value(&block, "order_index")?,
                record_id: toml_block_string_value(&block, "record_id")?,
                record_kind: toml_block_string_value(&block, "record_kind")?,
                file_offset: toml_block_usize_value(&block, "file_offset")?,
                size_bytes: toml_block_usize_value(&block, "size_bytes")?,
                alignment: toml_block_usize_value(&block, "alignment")?,
            })
        })
        .collect()
}

fn file_layout_record_table_mismatch_issues(
    expected: &[NsldObjectFileLayoutRecordDiagnostic],
    actual: &[NsldObjectFileLayoutRecordDiagnostic],
) -> Vec<String> {
    let mut issues = Vec::new();
    if actual.len() != expected.len() {
        issues.push(format!(
            "file_layout_record_entry_count mismatch: expected {}, found {}",
            expected.len(),
            actual.len()
        ));
    }
    for (index, expected_entry) in expected.iter().enumerate() {
        let Some(actual_entry) = actual.get(index) else {
            issues.push(format!("file_layout_record[{index}] missing"));
            continue;
        };
        push_usize_mismatch(
            &mut issues,
            index,
            "order_index",
            expected_entry.order_index,
            actual_entry.order_index,
        );
        push_string_mismatch(
            &mut issues,
            index,
            "record_id",
            &expected_entry.record_id,
            &actual_entry.record_id,
        );
        push_string_mismatch(
            &mut issues,
            index,
            "record_kind",
            &expected_entry.record_kind,
            &actual_entry.record_kind,
        );
        push_usize_mismatch(
            &mut issues,
            index,
            "file_offset",
            expected_entry.file_offset,
            actual_entry.file_offset,
        );
        push_usize_mismatch(
            &mut issues,
            index,
            "size_bytes",
            expected_entry.size_bytes,
            actual_entry.size_bytes,
        );
        push_usize_mismatch(
            &mut issues,
            index,
            "alignment",
            expected_entry.alignment,
            actual_entry.alignment,
        );
    }
    issues
}

fn push_usize_mismatch(
    issues: &mut Vec<String>,
    index: usize,
    field: &str,
    expected: usize,
    actual: usize,
) {
    if actual != expected {
        issues.push(format!(
            "file_layout_record[{index}].{field} mismatch: expected {expected}, found {actual}"
        ));
    }
}

fn push_string_mismatch(
    issues: &mut Vec<String>,
    index: usize,
    field: &str,
    expected: &str,
    actual: &str,
) {
    if actual != expected {
        issues.push(format!(
            "file_layout_record[{index}].{field} mismatch: expected {expected}, found {actual}"
        ));
    }
}

fn layout_record(
    order_index: usize,
    record_id: &str,
    record_kind: &str,
    file_offset: usize,
    size_bytes: usize,
    alignment: usize,
) -> NsldObjectFileLayoutRecordDiagnostic {
    NsldObjectFileLayoutRecordDiagnostic {
        order_index,
        record_id: record_id.to_owned(),
        record_kind: record_kind.to_owned(),
        file_offset,
        size_bytes,
        alignment,
    }
}

fn align_to(value: usize, alignment: usize) -> usize {
    if alignment <= 1 {
        return value;
    }
    value
        .saturating_add(alignment - 1)
        .saturating_div(alignment)
        .saturating_mul(alignment)
}

fn mach_o_string_table_size(sections: &[super::reports::NsldObjectByteSectionDiagnostic]) -> usize {
    let mut size = 1usize;
    size = size.saturating_add("__nuis_entry".len() + 1);
    for section in sections {
        size =
            size.saturating_add(mach_o_section_symbol_name(&section.source_section_id).len() + 1);
    }
    size
}

fn mach_o_section_symbol_name(source_section_id: &str) -> String {
    format!("__nuis_{}", source_section_id.replace('.', "_"))
}

fn nsld_object_file_layout_hash(
    writer_target_id: &str,
    writer_backend_kind: &str,
    object_family: &str,
    object_format: &str,
    records: &[NsldObjectFileLayoutRecordDiagnostic],
    total_file_size_bytes: usize,
) -> String {
    let mut material = format!(
        "writer_target_id={writer_target_id}\nwriter_backend_kind={writer_backend_kind}\nobject_family={object_family}\nobject_format={object_format}\ntotal_file_size_bytes={total_file_size_bytes}\n"
    );
    for record in records {
        material.push_str(&format!(
            "{}|{}|{}|{}|{}|{}\n",
            record.order_index,
            record.record_id,
            record.record_kind,
            record.file_offset,
            record.size_bytes,
            record.alignment
        ));
    }
    fnv1a64_hex(material.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::{
        nsld_emit_object_file_layout_report, nsld_object_file_layout_report,
        nsld_verify_object_file_layout_report,
    };
    use crate::main_test_support::empty_link_plan;
    use std::{fs, path::Path};

    #[test]
    fn mach_o_arm64_file_layout_places_metadata_before_payload() {
        let plan = empty_link_plan();
        let report = nsld_object_file_layout_report(Path::new("manifest.toml"), &plan);

        assert_eq!(report.writer_backend_kind, "mach-o-arm64");
        assert_eq!(report.object_family, "mach-o");
        assert_eq!(report.object_format, "mach-o");
        assert!(!report.layout_ready);
        assert!(report.file_layout_hash.starts_with("0x"));
        assert!(report.record_count >= 6);
        assert_eq!(report.records[0].record_kind, "macho-header");
        assert_eq!(report.records[0].file_offset, 0);
        assert_eq!(report.records[0].size_bytes, 32);
        assert_eq!(report.records[1].record_kind, "macho-load-commands");
        assert!(report
            .records
            .iter()
            .any(|record| record.record_kind == "macho-symbol-table"));
        assert!(report
            .records
            .iter()
            .any(|record| record.record_kind == "macho-string-table"));
        assert!(report.total_file_size_bytes > 0);
    }

    #[test]
    fn object_file_layout_serializes_writer_identity() {
        let plan = empty_link_plan();
        let report = nsld_object_file_layout_report(Path::new("manifest.toml"), &plan);
        let rendered = crate::toml::render_object_file_layout(&report);
        let json = crate::json_object::nsld_object_file_layout_report_json(&report);

        assert_eq!(report.writer_target_id, "arm64-macos-mach-o");
        assert_eq!(report.writer_backend_kind, "mach-o-arm64");
        assert_eq!(report.object_family, "mach-o");
        assert!(rendered.contains("writer_backend_kind = \"mach-o-arm64\""));
        assert!(rendered.contains("object_family = \"mach-o\""));
        assert!(json.contains("\"writer_backend_kind\":\"mach-o-arm64\""));
        assert!(json.contains("\"object_family\":\"mach-o\""));
    }

    #[test]
    fn emit_and_verify_object_file_layout_snapshot() {
        let dir = std::env::temp_dir().join(format!(
            "nsld-object-file-layout-emit-ok-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();

        let emit = nsld_emit_object_file_layout_report(Path::new("manifest.toml"), &plan).unwrap();
        let verify = nsld_verify_object_file_layout_report(Path::new("manifest.toml"), &plan);
        fs::remove_dir_all(dir).unwrap();

        assert!(emit
            .output_path
            .ends_with("nuis.nsld.object-file-layout.toml"));
        assert!(emit.file_layout_hash.starts_with("0x"));
        assert!(verify.valid);
        assert!(verify.issues.is_empty());
    }

    #[test]
    fn verify_object_file_layout_reports_record_drift() {
        let dir = std::env::temp_dir().join(format!(
            "nsld-object-file-layout-record-drift-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();
        nsld_emit_object_file_layout_report(Path::new("manifest.toml"), &plan).unwrap();
        let path = dir.join("nuis.nsld.object-file-layout.toml");
        let damaged = fs::read_to_string(&path).unwrap().replace(
            "record_kind = \"macho-header\"",
            "record_kind = \"wrong-header\"",
        );
        fs::write(&path, damaged).unwrap();

        let verify = nsld_verify_object_file_layout_report(Path::new("manifest.toml"), &plan);
        fs::remove_dir_all(dir).unwrap();

        assert!(!verify.valid);
        assert!(verify.issues.iter().any(|issue| {
            issue
                == "file_layout_record[0].record_kind mismatch: expected macho-header, found wrong-header"
        }));
    }
}
