use super::{json_fields::*, reports::*};

pub(crate) fn nsld_object_image_dry_run_report_json(
    report: &NsldObjectImageDryRunReport,
) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_object_image_dry_run"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_string_field("image_path", &report.image_path),
        json_string_field("writer_target_id", &report.writer_target_id),
        json_string_field("writer_backend_kind", &report.writer_backend_kind),
        json_string_field("object_family", &report.object_family),
        json_string_field("backend_kind", &report.backend_kind),
        json_string_field("backend_family", &report.backend_family),
        json_string_field("backend_status", &report.backend_status),
        json_string_field("object_format", &report.object_format),
        json_string_field("file_layout_hash", &report.file_layout_hash),
        json_usize_field("record_count", report.record_count),
        json_usize_field("total_file_size_bytes", report.total_file_size_bytes),
        json_bool_field("image_constructed", report.image_constructed),
        json_bool_field("image_ready", report.image_ready),
        json_optional_usize_field("image_size_bytes", report.image_size_bytes),
        json_optional_string_field("image_hash", report.image_hash.as_deref()),
        json_bool_field(
            "relocation_lowering_valid",
            report.relocation_lowering_valid,
        ),
        json_usize_field(
            "relocation_lowering_rule_count",
            report.relocation_lowering_rule_count,
        ),
        json_string_array_field(
            "relocation_lowering_issues",
            &report.relocation_lowering_issues,
        ),
        format!(
            "\"relocation_lowering_rules\":[{}]",
            relocation_lowering_rules_json(&report.relocation_lowering_rules)
        ),
        json_usize_field("relocation_record_count", report.relocation_record_count),
        json_string_field(
            "relocation_record_table_hash",
            &report.relocation_record_table_hash,
        ),
        format!(
            "\"relocation_records\":[{}]",
            relocation_records_json(&report.relocation_records)
        ),
        json_string_array_field("blockers", &report.blockers),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn relocation_lowering_rules_json(
    rules: &[NsldRelocationLoweringRuleDiagnostic],
) -> String {
    rules
        .iter()
        .map(|rule| {
            let fields = vec![
                json_string_field("rule_id", &rule.rule_id),
                json_string_field("source_seed_kind", &rule.source_seed_kind),
                json_string_field("target_relocation_kind", &rule.target_relocation_kind),
                json_bool_field("pc_relative", rule.pc_relative),
                json_usize_field("length_power", rule.length_power as usize),
                json_bool_field("external", rule.external),
                json_usize_field("relocation_type", rule.relocation_type as usize),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

pub(crate) fn relocation_records_json(
    records: &[NsldObjectImageRelocationRecordDiagnostic],
) -> String {
    records
        .iter()
        .map(|record| {
            let fields = vec![
                json_string_field("record_id", &record.record_id),
                json_string_field("relocation_seed_id", &record.relocation_seed_id),
                json_string_field("source_section_id", &record.source_section_id),
                json_usize_field("source_offset", record.source_offset),
                json_string_field("source_seed_kind", &record.source_seed_kind),
                json_string_field("target_relocation_kind", &record.target_relocation_kind),
                json_usize_field("symbol_index", record.symbol_index as usize),
                json_bool_field("pc_relative", record.pc_relative),
                json_usize_field("length_power", record.length_power as usize),
                json_bool_field("external", record.external),
                json_usize_field("relocation_type", record.relocation_type as usize),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

pub(crate) fn nsld_object_image_dry_run_emit_report_json(
    report: &NsldObjectImageDryRunEmitReport,
) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_object_image_dry_run_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_string_field("image_path", &report.image_path),
        json_bool_field("image_emitted", report.image_emitted),
        json_bool_field("image_constructed", report.image_constructed),
        json_bool_field("image_ready", report.image_ready),
        json_optional_usize_field("image_size_bytes", report.image_size_bytes),
        json_optional_string_field("image_hash", report.image_hash.as_deref()),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_object_image_dry_run_verify_report_json(
    report: &NsldObjectImageDryRunVerifyReport,
) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_object_image_dry_run_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_string_field("image_path", &report.image_path),
        json_bool_field("valid", report.valid),
        json_string_field(
            "expected_writer_backend_kind",
            &report.expected_writer_backend_kind,
        ),
        json_string_field("expected_object_family", &report.expected_object_family),
        json_string_field("expected_backend_family", &report.expected_backend_family),
        json_string_field("expected_backend_status", &report.expected_backend_status),
        json_string_field(
            "expected_file_layout_hash",
            &report.expected_file_layout_hash,
        ),
        json_bool_field(
            "expected_image_constructed",
            report.expected_image_constructed,
        ),
        json_bool_field("expected_image_ready", report.expected_image_ready),
        json_optional_usize_field(
            "expected_image_size_bytes",
            report.expected_image_size_bytes,
        ),
        json_optional_string_field("expected_image_hash", report.expected_image_hash.as_deref()),
        json_bool_field(
            "expected_relocation_lowering_valid",
            report.expected_relocation_lowering_valid,
        ),
        json_usize_field(
            "expected_relocation_lowering_rule_count",
            report.expected_relocation_lowering_rule_count,
        ),
        json_string_array_field(
            "expected_relocation_lowering_issues",
            &report.expected_relocation_lowering_issues,
        ),
        format!(
            "\"expected_relocation_lowering_rules\":[{}]",
            relocation_lowering_rules_json(&report.expected_relocation_lowering_rules)
        ),
        json_usize_field(
            "expected_relocation_record_count",
            report.expected_relocation_record_count,
        ),
        json_string_field(
            "expected_relocation_record_table_hash",
            &report.expected_relocation_record_table_hash,
        ),
        format!(
            "\"expected_relocation_records\":[{}]",
            relocation_records_json(&report.expected_relocation_records)
        ),
        json_optional_string_field(
            "actual_file_layout_hash",
            report.actual_file_layout_hash.as_deref(),
        ),
        json_optional_string_field(
            "actual_writer_backend_kind",
            report.actual_writer_backend_kind.as_deref(),
        ),
        json_optional_string_field(
            "actual_object_family",
            report.actual_object_family.as_deref(),
        ),
        json_optional_string_field(
            "actual_backend_family",
            report.actual_backend_family.as_deref(),
        ),
        json_optional_string_field(
            "actual_backend_status",
            report.actual_backend_status.as_deref(),
        ),
        json_optional_bool_field("actual_image_constructed", report.actual_image_constructed),
        json_optional_bool_field("actual_image_ready", report.actual_image_ready),
        json_optional_usize_field("actual_image_size_bytes", report.actual_image_size_bytes),
        json_optional_string_field("actual_image_hash", report.actual_image_hash.as_deref()),
        json_optional_bool_field(
            "actual_relocation_lowering_valid",
            report.actual_relocation_lowering_valid,
        ),
        json_optional_usize_field(
            "actual_relocation_lowering_rule_count",
            report.actual_relocation_lowering_rule_count,
        ),
        json_string_array_field(
            "actual_relocation_lowering_issues",
            report
                .actual_relocation_lowering_issues
                .as_deref()
                .unwrap_or(&[]),
        ),
        format!(
            "\"actual_relocation_lowering_rules\":[{}]",
            relocation_lowering_rules_json(
                report
                    .actual_relocation_lowering_rules
                    .as_deref()
                    .unwrap_or(&[])
            )
        ),
        json_optional_usize_field(
            "actual_relocation_record_count",
            report.actual_relocation_record_count,
        ),
        json_optional_string_field(
            "actual_relocation_record_table_hash",
            report.actual_relocation_record_table_hash.as_deref(),
        ),
        format!(
            "\"actual_relocation_records\":[{}]",
            relocation_records_json(report.actual_relocation_records.as_deref().unwrap_or(&[]))
        ),
        json_optional_usize_field(
            "actual_image_file_size_bytes",
            report.actual_image_file_size_bytes,
        ),
        json_optional_string_field(
            "actual_image_file_hash",
            report.actual_image_file_hash.as_deref(),
        ),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}
