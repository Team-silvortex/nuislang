use crate::{
    final_executable_registered_loader_probe::NsldRegisteredLoaderProbeOutcome, json_fields::*,
};

pub(crate) fn registered_loader_probe_outcome_json(
    outcome: &NsldRegisteredLoaderProbeOutcome,
) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_registered_loader_probe_outcome"),
        json_string_field("contract", outcome.contract),
        json_string_field("status", outcome.status),
        json_string_field("provider_id", &outcome.provider_id),
        json_string_field("target_key", &outcome.target_key),
        json_string_field("capability_id", &outcome.capability_id),
        json_string_field("provider_probe_contract", &outcome.provider_probe_contract),
        json_string_field("provider_probe_status", &outcome.provider_probe_status),
        json_string_field("probe_mode", &outcome.probe_mode),
        json_bool_field("host_supported", outcome.host_supported),
        json_bool_field("input_eligible", outcome.input_eligible),
        json_bool_field("attempted", outcome.attempted),
        json_usize_field("image_span_bytes", outcome.image_span_bytes),
        json_string_field("image_identity_hash", &outcome.image_identity_hash),
        json_string_field(
            "validation_evidence_hash",
            &outcome.validation_evidence_hash,
        ),
        json_bool_field("materialized", outcome.materialized),
        json_bool_field(
            "materialized_hash_matches",
            outcome.materialized_hash_matches,
        ),
        json_bool_field("os_loader_accepted", outcome.os_loader_accepted),
        json_bool_field("process_completed", outcome.process_completed),
        json_bool_field("timed_out", outcome.timed_out),
        json_optional_i64_field("exit_code", outcome.exit_code.map(i64::from)),
        json_optional_i64_field(
            "termination_signal",
            outcome.termination_signal.map(i64::from),
        ),
        json_usize_field("stdout_captured_bytes", outcome.stdout_captured_bytes),
        json_bool_field("stdout_truncated", outcome.stdout_truncated),
        json_usize_field("stderr_captured_bytes", outcome.stderr_captured_bytes),
        json_bool_field("stderr_truncated", outcome.stderr_truncated),
        json_optional_string_field("failure_kind", outcome.failure_kind.as_deref()),
        json_bool_field("cleanup_attempted", outcome.cleanup_attempted),
        json_bool_field("cleanup_succeeded", outcome.cleanup_succeeded),
        json_bool_field("execution_admitted", outcome.execution_admitted),
        json_string_array_field("blockers", &outcome.blockers),
        json_string_field("provider_evidence_hash", &outcome.provider_evidence_hash),
        json_string_field("outcome_ledger_hash", &outcome.outcome_ledger_hash),
    ];
    format!("{{{}}}", fields.join(","))
}
