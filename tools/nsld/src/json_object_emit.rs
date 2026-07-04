use super::{
    json_fields::*,
    reports::{NsldObjectEmitReport, NsldObjectEmitVerifyReport, NsldObjectOutputVerifyReport},
};

pub(crate) fn nsld_object_emit_report_json(report: &NsldObjectEmitReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_object_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_string_field("writer_input_path", &report.writer_input_path),
        json_string_field("blocked_report_path", &report.blocked_report_path),
        json_string_field(
            "image_dry_run_report_path",
            &report.image_dry_run_report_path,
        ),
        json_string_field("image_dry_run_path", &report.image_dry_run_path),
        json_optional_string_field("image_dry_run_hash", report.image_dry_run_hash.as_deref()),
        json_string_field("writer_target_id", &report.writer_target_id),
        json_string_field("writer_backend_kind", &report.writer_backend_kind),
        json_string_field("object_family", &report.object_family),
        json_string_field("object_plan_hash", &report.object_plan_hash),
        json_bool_field("emitted", report.emitted),
        json_bool_field("can_emit_object", report.can_emit_object),
        json_string_array_field("blockers", &report.blockers),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_object_emit_verify_report_json(report: &NsldObjectEmitVerifyReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_object_emit_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_string_field(
            "expected_object_plan_hash",
            &report.expected_object_plan_hash,
        ),
        json_string_field(
            "expected_writer_backend_kind",
            &report.expected_writer_backend_kind,
        ),
        json_string_field("expected_object_family", &report.expected_object_family),
        json_optional_string_field(
            "expected_image_dry_run_hash",
            report.expected_image_dry_run_hash.as_deref(),
        ),
        json_optional_string_field(
            "actual_object_plan_hash",
            report.actual_object_plan_hash.as_deref(),
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
            "actual_image_dry_run_hash",
            report.actual_image_dry_run_hash.as_deref(),
        ),
        json_bool_field(
            "image_dry_run_report_valid",
            report.image_dry_run_report_valid,
        ),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_object_output_verify_report_json(
    report: &NsldObjectOutputVerifyReport,
) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_object_output_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("object_output_path", &report.object_output_path),
        json_string_field("image_dry_run_path", &report.image_dry_run_path),
        json_bool_field("valid", report.valid),
        json_optional_usize_field("expected_size_bytes", report.expected_size_bytes),
        json_optional_usize_field("actual_size_bytes", report.actual_size_bytes),
        json_optional_string_field("expected_hash", report.expected_hash.as_deref()),
        json_optional_string_field("actual_hash", report.actual_hash.as_deref()),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}
