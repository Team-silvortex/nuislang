use crate::{json_fields::*, reports::NsldFinalOutputSelectionReport};

pub(crate) fn final_output_selection_json_field(report: &NsldFinalOutputSelectionReport) -> String {
    let fields = [
        json_string_field("contract", &report.contract),
        json_string_field("registry_contract", &report.registry_contract),
        json_string_field("registry_hash", &report.registry_hash),
        json_string_field("policy_id", &report.policy_id),
        json_string_field("policy_status", &report.policy_status),
        json_string_field("selection_kind", &report.selection_kind),
        json_bool_field("default_policy", report.default_policy),
        json_bool_field("explicit_request", report.explicit_request),
        json_bool_field("apply_requested", report.apply_requested),
        json_string_field("status", &report.status),
        json_bool_field("selection_ready", report.selection_ready),
        json_bool_field("installation_attempted", report.installation_attempted),
        json_bool_field("selected", report.selected),
        json_optional_string_field("provider_id", report.provider_id.as_deref()),
        json_optional_string_field("target_key", report.target_key.as_deref()),
        json_optional_string_field("capability_id", report.capability_id.as_deref()),
        json_optional_string_field("admission_contract", report.admission_contract.as_deref()),
        json_string_field("admission_status", &report.admission_status),
        json_optional_string_field(
            "admission_receipt_file",
            report.admission_receipt_file.as_deref(),
        ),
        json_optional_bool_field("admission_receipt_valid", report.admission_receipt_valid),
        json_optional_string_field(
            "admission_receipt_hash_sha256",
            report.admission_receipt_hash_sha256.as_deref(),
        ),
        json_optional_string_field(
            "admission_verification_ledger_sha256",
            report.admission_verification_ledger_sha256.as_deref(),
        ),
        json_optional_usize_field(
            "candidate_image_span_bytes",
            report.candidate_image_span_bytes,
        ),
        json_optional_string_field(
            "candidate_image_sha256",
            report.candidate_image_sha256.as_deref(),
        ),
        json_string_field("selected_output_path", &report.selected_output_path),
        json_string_field("selected_output_name", &report.selected_output_name),
        json_bool_field("selected_output_present", report.selected_output_present),
        json_optional_usize_field(
            "selected_output_span_bytes",
            report.selected_output_span_bytes,
        ),
        json_optional_string_field(
            "selected_output_sha256",
            report.selected_output_sha256.as_deref(),
        ),
        json_bool_field(
            "selected_output_executable",
            report.selected_output_executable,
        ),
        json_bool_field(
            "selected_output_identity_matches",
            report.selected_output_identity_matches,
        ),
        json_optional_string_field(
            "publication_contract",
            report.publication_contract.as_deref(),
        ),
        json_string_field("publication_status", &report.publication_status),
        json_optional_string_field(
            "publication_ledger_sha256",
            report.publication_ledger_sha256.as_deref(),
        ),
        json_usize_field("issue_count", report.issue_count),
        json_string_array_field("issues", &report.issues),
        json_string_field("selection_ledger_sha256", &report.selection_ledger_sha256),
    ];
    format!("\"selection\":{{{}}}", fields.join(","))
}
