use super::{json_fields::*, reports::*};

pub(crate) fn nsld_final_executable_image_dry_run_report_json(
    report: &NsldFinalExecutableImageDryRunReport,
) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_executable_image_dry_run"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_string_field("image_path", &report.image_path),
        json_string_field("image_format", &report.image_format),
        json_string_field("image_magic", &report.image_magic),
        json_usize_field("image_header_size", report.image_header_size),
        json_usize_field("payload_byte_offset", report.payload_byte_offset),
        json_usize_field("payload_byte_span", report.payload_byte_span),
        json_string_field("layout_hash", &report.layout_hash),
        json_string_field("byte_map_hash", &report.byte_map_hash),
        json_usize_field("payload_count", report.payload_count),
        json_usize_field("byte_span", report.byte_span),
        json_string_field(
            "scheduler_metadata_payload_id",
            &report.scheduler_metadata_payload_id,
        ),
        json_bool_field(
            "scheduler_metadata_present",
            report.scheduler_metadata_present,
        ),
        json_optional_usize_field(
            "scheduler_metadata_offset",
            report.scheduler_metadata_offset,
        ),
        json_optional_string_field(
            "scheduler_metadata_hash",
            report.scheduler_metadata_hash.as_deref(),
        ),
        json_string_field(
            "relocation_application_strategy",
            &report.relocation_application_strategy,
        ),
        json_usize_field(
            "relocation_application_count",
            report.relocation_application_count,
        ),
        json_string_field(
            "relocation_application_table_hash",
            &report.relocation_application_table_hash,
        ),
        json_string_field(
            "relocation_application_audit_status",
            &report.relocation_application_audit_status,
        ),
        json_usize_field(
            "relocation_application_audit_count",
            report.relocation_application_audit_count,
        ),
        json_string_field(
            "relocation_application_audit_table_hash",
            &report.relocation_application_audit_table_hash,
        ),
        json_string_array_field(
            "relocation_application_audit_blockers",
            &report.relocation_application_audit_blockers,
        ),
        json_string_field(
            "relocation_patch_preview_status",
            &report.relocation_patch_preview_status,
        ),
        json_usize_field(
            "relocation_patch_preview_count",
            report.relocation_patch_preview_count,
        ),
        json_string_field(
            "relocation_patch_preview_table_hash",
            &report.relocation_patch_preview_table_hash,
        ),
        format!(
            "\"relocation_patch_previews\":[{}]",
            relocation_patch_previews_json(&report.relocation_patch_previews)
        ),
        json_bool_field("image_constructed", report.image_constructed),
        json_bool_field("image_ready", report.image_ready),
        json_optional_usize_field("image_size_bytes", report.image_size_bytes),
        json_optional_string_field("image_hash", report.image_hash.as_deref()),
        json_string_array_field("blockers", &report.blockers),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_final_executable_image_dry_run_emit_report_json(
    report: &NsldFinalExecutableImageDryRunEmitReport,
) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_executable_image_dry_run_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_string_field("image_path", &report.image_path),
        json_bool_field("image_emitted", report.image_emitted),
        json_bool_field("image_constructed", report.image_constructed),
        json_bool_field("image_ready", report.image_ready),
        json_string_field("image_format", &report.image_format),
        json_usize_field("image_header_size", report.image_header_size),
        json_usize_field("payload_byte_offset", report.payload_byte_offset),
        json_optional_usize_field("image_size_bytes", report.image_size_bytes),
        json_optional_string_field("image_hash", report.image_hash.as_deref()),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_final_executable_image_dry_run_verify_report_json(
    report: &NsldFinalExecutableImageDryRunVerifyReport,
) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_executable_image_dry_run_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_string_field("image_path", &report.image_path),
        json_bool_field("valid", report.valid),
        json_string_field("expected_layout_hash", &report.expected_layout_hash),
        json_optional_string_field("actual_layout_hash", report.actual_layout_hash.as_deref()),
        json_string_field("expected_byte_map_hash", &report.expected_byte_map_hash),
        json_optional_string_field(
            "actual_byte_map_hash",
            report.actual_byte_map_hash.as_deref(),
        ),
        json_string_field("expected_image_magic", &report.expected_image_magic),
        json_optional_string_field("actual_image_magic", report.actual_image_magic.as_deref()),
        json_usize_field(
            "expected_image_version",
            report.expected_image_version as usize,
        ),
        json_optional_usize_field(
            "actual_image_version",
            report.actual_image_version.map(|value| value as usize),
        ),
        json_usize_field(
            "expected_image_header_size",
            report.expected_image_header_size,
        ),
        json_optional_usize_field("actual_image_header_size", report.actual_image_header_size),
        json_usize_field(
            "expected_payload_byte_offset",
            report.expected_payload_byte_offset,
        ),
        json_optional_usize_field(
            "actual_payload_byte_offset",
            report.actual_payload_byte_offset,
        ),
        json_usize_field(
            "expected_payload_byte_span",
            report.expected_payload_byte_span,
        ),
        json_optional_usize_field("actual_payload_byte_span", report.actual_payload_byte_span),
        json_optional_string_field(
            "actual_header_layout_hash",
            report.actual_header_layout_hash.as_deref(),
        ),
        json_optional_string_field(
            "actual_header_byte_map_hash",
            report.actual_header_byte_map_hash.as_deref(),
        ),
        json_usize_field(
            "expected_payload_region_count",
            report.expected_payload_region_count,
        ),
        json_optional_usize_field(
            "actual_payload_region_count",
            report.actual_payload_region_count,
        ),
        json_optional_string_field(
            "expected_payload_region_hash",
            report.expected_payload_region_hash.as_deref(),
        ),
        json_optional_string_field(
            "actual_payload_region_hash",
            report.actual_payload_region_hash.as_deref(),
        ),
        json_string_field(
            "expected_scheduler_metadata_payload_id",
            &report.expected_scheduler_metadata_payload_id,
        ),
        json_optional_string_field(
            "actual_scheduler_metadata_payload_id",
            report.actual_scheduler_metadata_payload_id.as_deref(),
        ),
        json_bool_field(
            "expected_scheduler_metadata_present",
            report.expected_scheduler_metadata_present,
        ),
        json_optional_bool_field(
            "actual_scheduler_metadata_present",
            report.actual_scheduler_metadata_present,
        ),
        json_optional_usize_field(
            "expected_scheduler_metadata_offset",
            report.expected_scheduler_metadata_offset,
        ),
        json_optional_usize_field(
            "actual_scheduler_metadata_offset",
            report.actual_scheduler_metadata_offset,
        ),
        json_optional_string_field(
            "expected_scheduler_metadata_hash",
            report.expected_scheduler_metadata_hash.as_deref(),
        ),
        json_optional_string_field(
            "actual_scheduler_metadata_hash",
            report.actual_scheduler_metadata_hash.as_deref(),
        ),
        json_string_field(
            "expected_relocation_application_strategy",
            &report.expected_relocation_application_strategy,
        ),
        json_optional_string_field(
            "actual_relocation_application_strategy",
            report.actual_relocation_application_strategy.as_deref(),
        ),
        json_usize_field(
            "expected_relocation_application_count",
            report.expected_relocation_application_count,
        ),
        json_optional_usize_field(
            "actual_relocation_application_count",
            report.actual_relocation_application_count,
        ),
        json_string_field(
            "expected_relocation_application_table_hash",
            &report.expected_relocation_application_table_hash,
        ),
        json_optional_string_field(
            "actual_relocation_application_table_hash",
            report.actual_relocation_application_table_hash.as_deref(),
        ),
        json_string_field(
            "expected_relocation_application_audit_status",
            &report.expected_relocation_application_audit_status,
        ),
        json_optional_string_field(
            "actual_relocation_application_audit_status",
            report.actual_relocation_application_audit_status.as_deref(),
        ),
        json_usize_field(
            "expected_relocation_application_audit_count",
            report.expected_relocation_application_audit_count,
        ),
        json_optional_usize_field(
            "actual_relocation_application_audit_count",
            report.actual_relocation_application_audit_count,
        ),
        json_string_field(
            "expected_relocation_application_audit_table_hash",
            &report.expected_relocation_application_audit_table_hash,
        ),
        json_optional_string_field(
            "actual_relocation_application_audit_table_hash",
            report
                .actual_relocation_application_audit_table_hash
                .as_deref(),
        ),
        json_string_field(
            "expected_relocation_patch_preview_status",
            &report.expected_relocation_patch_preview_status,
        ),
        json_optional_string_field(
            "actual_relocation_patch_preview_status",
            report.actual_relocation_patch_preview_status.as_deref(),
        ),
        json_usize_field(
            "expected_relocation_patch_preview_count",
            report.expected_relocation_patch_preview_count,
        ),
        json_optional_usize_field(
            "actual_relocation_patch_preview_count",
            report.actual_relocation_patch_preview_count,
        ),
        json_string_field(
            "expected_relocation_patch_preview_table_hash",
            &report.expected_relocation_patch_preview_table_hash,
        ),
        json_optional_string_field(
            "actual_relocation_patch_preview_table_hash",
            report.actual_relocation_patch_preview_table_hash.as_deref(),
        ),
        json_usize_field(
            "expected_relocation_patch_preview_entry_count",
            report.expected_relocation_patch_preview_entry_count,
        ),
        json_optional_usize_field(
            "actual_relocation_patch_preview_entry_count",
            report.actual_relocation_patch_preview_entry_count,
        ),
        json_optional_string_field(
            "actual_relocation_patch_preview_record_table_hash",
            report
                .actual_relocation_patch_preview_record_table_hash
                .as_deref(),
        ),
        json_string_array_field(
            "expected_relocation_application_audit_blockers",
            &report.expected_relocation_application_audit_blockers,
        ),
        json_string_array_field(
            "actual_relocation_application_audit_blockers",
            &report.actual_relocation_application_audit_blockers,
        ),
        json_bool_field(
            "expected_image_constructed",
            report.expected_image_constructed,
        ),
        json_optional_bool_field("actual_image_constructed", report.actual_image_constructed),
        json_bool_field("expected_image_ready", report.expected_image_ready),
        json_optional_bool_field("actual_image_ready", report.actual_image_ready),
        json_optional_usize_field(
            "expected_image_size_bytes",
            report.expected_image_size_bytes,
        ),
        json_optional_usize_field("actual_image_size_bytes", report.actual_image_size_bytes),
        json_optional_string_field("expected_image_hash", report.expected_image_hash.as_deref()),
        json_optional_string_field("actual_image_hash", report.actual_image_hash.as_deref()),
        json_string_array_field("expected_blockers", &report.expected_blockers),
        json_string_array_field("actual_blockers", &report.actual_blockers),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

fn relocation_patch_previews_json(
    records: &[NsldFinalExecutableRelocationPatchPreviewRecord],
) -> String {
    records
        .iter()
        .map(|record| {
            let fields = [
                json_usize_field("order_index", record.order_index),
                json_string_field("relocation_id", &record.relocation_id),
                json_string_field("patch_kind", &record.patch_kind),
                json_usize_field("patch_offset", record.patch_offset),
                json_usize_field("patch_width_bytes", record.patch_width_bytes),
                json_string_field("patch_value_hash", &record.patch_value_hash),
                json_string_field("target_symbol_id", &record.target_symbol_id),
                json_string_field("preview_status", &record.preview_status),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}
