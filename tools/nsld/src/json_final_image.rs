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
