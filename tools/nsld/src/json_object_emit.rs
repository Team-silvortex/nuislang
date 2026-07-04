use super::{
    json_fields::*,
    reports::{NsldObjectEmitReport, NsldObjectEmitVerifyReport},
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
        json_optional_string_field(
            "expected_image_dry_run_hash",
            report.expected_image_dry_run_hash.as_deref(),
        ),
        json_optional_string_field(
            "actual_object_plan_hash",
            report.actual_object_plan_hash.as_deref(),
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
