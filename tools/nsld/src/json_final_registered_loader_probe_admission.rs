use crate::{
    final_executable_registered_loader_probe_admission::NsldRegisteredLoaderProbeAdmissionVerifyReport,
    json_fields::*,
};

pub(crate) fn registered_loader_probe_admission_verify_report_json(
    report: &NsldRegisteredLoaderProbeAdmissionVerifyReport,
) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field(
            "kind",
            "nsld_registered_loader_probe_admission_verification",
        ),
        json_string_field("contract", report.contract),
        json_string_field("status", report.status),
        json_string_field("receipt_file", report.receipt_file),
        json_bool_field("receipt_present", report.receipt_present),
        json_bool_field("receipt_parsed", report.receipt_parsed),
        json_bool_field("canonical_source", report.canonical_source),
        json_bool_field("receipt_hash_matches", report.receipt_hash_matches),
        json_bool_field(
            "finalizer_registry_matches",
            report.finalizer_registry_matches,
        ),
        json_bool_field("target_identity_matches", report.target_identity_matches),
        json_bool_field("outcome_evidence_valid", report.outcome_evidence_valid),
        json_bool_field(
            "current_private_image_matches",
            report.current_private_image_matches,
        ),
        json_bool_field("valid", report.valid),
        json_optional_string_field("provider_id", report.provider_id.as_deref()),
        json_optional_string_field("target_key", report.target_key.as_deref()),
        json_optional_string_field("capability_id", report.capability_id.as_deref()),
        json_optional_string_field("image_identity_hash", report.image_identity_hash.as_deref()),
        json_optional_string_field(
            "current_image_identity_hash",
            report.current_image_identity_hash.as_deref(),
        ),
        json_optional_string_field(
            "validation_evidence_hash",
            report.validation_evidence_hash.as_deref(),
        ),
        json_optional_string_field(
            "current_validation_evidence_hash",
            report.current_validation_evidence_hash.as_deref(),
        ),
        json_optional_string_field(
            "provider_evidence_hash",
            report.provider_evidence_hash.as_deref(),
        ),
        json_optional_string_field("outcome_ledger_hash", report.outcome_ledger_hash.as_deref()),
        json_optional_string_field("receipt_hash_sha256", report.receipt_hash_sha256.as_deref()),
        json_usize_field("issue_count", report.issue_count),
        json_string_array_field("issues", &report.issues),
        json_string_field(
            "verification_ledger_sha256",
            &report.verification_ledger_sha256,
        ),
    ];
    format!("{{{}}}", fields.join(","))
}
