use crate::{json_fields::*, reports::NsldMachOArm64LoaderProbeReport};

pub(crate) fn macho_arm64_loader_probe_report_json(
    report: &NsldMachOArm64LoaderProbeReport,
) -> String {
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_macho_arm64_private_image_loader_probe"),
        json_string_field("contract", &report.contract),
        json_string_field("status", &report.status),
        json_string_field("probe_mode", &report.probe_mode),
        json_string_field("materialization_kind", &report.materialization_kind),
        json_string_field("target_arch", &report.target_arch),
        json_string_field("target_os", &report.target_os),
        json_bool_field("host_supported", report.host_supported),
        json_bool_field("input_eligible", report.input_eligible),
        json_bool_field("attempted", report.attempted),
        json_usize_field("image_span_bytes", report.image_span_bytes),
        json_string_field("shell_image_hash", &report.shell_image_hash),
        json_string_field(
            "signature_validation_ledger_hash",
            &report.signature_validation_ledger_hash,
        ),
        json_usize_field(
            "unresolved_external_symbol_count",
            report.unresolved_external_symbol_count,
        ),
        json_usize_field("bind_count", report.bind_count),
        format!("\"probe_timeout_millis\":{}", report.probe_timeout_millis),
        json_bool_field("materialized", report.materialized),
        json_bool_field(
            "materialized_hash_matches",
            report.materialized_hash_matches,
        ),
        json_bool_field("kernel_accepted", report.kernel_accepted),
        json_bool_field("process_completed", report.process_completed),
        json_bool_field("timed_out", report.timed_out),
        json_optional_i64_field("exit_code", report.exit_code.map(i64::from)),
        json_optional_i64_field(
            "termination_signal",
            report.termination_signal.map(i64::from),
        ),
        json_usize_field("stdout_captured_bytes", report.stdout_captured_bytes),
        json_bool_field("stdout_truncated", report.stdout_truncated),
        json_string_field("stdout_hash", &report.stdout_hash),
        json_usize_field("stderr_captured_bytes", report.stderr_captured_bytes),
        json_bool_field("stderr_truncated", report.stderr_truncated),
        json_string_field("stderr_hash", &report.stderr_hash),
        json_optional_string_field("failure_kind", report.failure_kind.as_deref()),
        json_bool_field("cleanup_attempted", report.cleanup_attempted),
        json_bool_field("cleanup_succeeded", report.cleanup_succeeded),
        json_string_field(
            "publication_eligibility_contract",
            &report.publication_eligibility_contract,
        ),
        json_string_field(
            "publication_eligibility_status",
            &report.publication_eligibility_status,
        ),
        json_bool_field("publication_eligible", report.publication_eligible),
        json_string_array_field("publication_blockers", &report.publication_blockers),
        json_string_field("probe_ledger_hash", &report.probe_ledger_hash),
    ];
    format!("{{{}}}", fields.join(","))
}
