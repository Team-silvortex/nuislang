use crate::{json_fields::*, reports::NsldPrivateImagePublicationReport};

pub(crate) fn private_image_publication_report_json(
    report: &NsldPrivateImagePublicationReport,
) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_registered_private_image_publication"),
        json_string_field("contract", &report.contract),
        json_string_field("status", &report.status),
        json_string_field("provider_id", &report.provider_id),
        json_string_field("target_key", &report.target_key),
        json_string_field("capability_id", &report.capability_id),
        json_bool_field("apply_requested", report.apply_requested),
        json_bool_field("publication_ready", report.publication_ready),
        json_string_field("admission_contract", &report.admission_contract),
        json_string_field("admission_status", &report.admission_status),
        json_string_field("admission_receipt_file", &report.admission_receipt_file),
        json_bool_field(
            "admission_receipt_present",
            report.admission_receipt_present,
        ),
        json_bool_field("admission_receipt_valid", report.admission_receipt_valid),
        json_optional_string_field(
            "admission_receipt_hash_sha256",
            report.admission_receipt_hash_sha256.as_deref(),
        ),
        json_string_field(
            "admission_verification_ledger_sha256",
            &report.admission_verification_ledger_sha256,
        ),
        json_usize_field("source_image_span_bytes", report.source_image_span_bytes),
        json_string_field("source_image_hash", &report.source_image_hash),
        json_string_field("source_image_sha256", &report.source_image_sha256),
        json_string_field("output_path", &report.output_path),
        json_bool_field("output_present_before", report.output_present_before),
        json_optional_string_field(
            "output_sha256_before",
            report.output_sha256_before.as_deref(),
        ),
        json_bool_field("installation_attempted", report.installation_attempted),
        json_bool_field("installed", report.installed),
        json_bool_field("output_present_after", report.output_present_after),
        json_optional_usize_field("output_span_bytes_after", report.output_span_bytes_after),
        json_optional_string_field("output_sha256_after", report.output_sha256_after.as_deref()),
        json_bool_field(
            "output_matches_private_image",
            report.output_matches_private_image,
        ),
        json_bool_field("output_executable", report.output_executable),
        json_bool_field("output_changed", report.output_changed),
        json_usize_field("issue_count", report.issue_count),
        json_string_array_field("issues", &report.issues),
        json_string_field(
            "publication_ledger_sha256",
            &report.publication_ledger_sha256,
        ),
    ];
    format!("{{{}}}", fields.join(","))
}
