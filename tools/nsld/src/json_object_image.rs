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
        json_string_array_field("blockers", &report.blockers),
    ];
    format!("{{{}}}", fields.join(","))
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
        json_optional_string_field(
            "actual_file_layout_hash",
            report.actual_file_layout_hash.as_deref(),
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
