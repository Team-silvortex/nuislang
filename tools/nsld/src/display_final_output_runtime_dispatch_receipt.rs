use super::{display_text::optional_string_text, reports::NsldFinalExecutableOutputReport};

pub(super) fn display_runtime_dispatch_receipt(report: &NsldFinalExecutableOutputReport) {
    println!(
        "  final_output_nsdb_runtime_bootstrap_identity: contract={} status={} hash={}",
        optional_string_text(
            report
                .final_output_nsdb_runtime_bootstrap_identity_contract
                .as_deref()
        ),
        report.final_output_nsdb_runtime_bootstrap_identity_status,
        optional_string_text(
            report
                .final_output_nsdb_runtime_bootstrap_identity_hash
                .as_deref()
        )
    );
    println!(
        "  final_output_nsdb_runtime_dispatch_receipt: contract={} status={} hash={} execution={} import={} table={} capabilities={} slot={} code={} acknowledged={}",
        optional_string_text(
            report
                .final_output_nsdb_runtime_dispatch_receipt_contract
                .as_deref()
        ),
        report.final_output_nsdb_runtime_dispatch_receipt_status,
        optional_string_text(
            report
                .final_output_nsdb_runtime_dispatch_receipt_hash
                .as_deref()
        ),
        optional_string_text(
            report
                .final_output_nsdb_runtime_dispatch_execution_identity_hash
                .as_deref()
        ),
        optional_string_text(
            report
                .final_output_nsdb_runtime_dispatch_import_identity_hash
                .as_deref()
        ),
        optional_string_text(
            report
                .final_output_nsdb_runtime_dispatch_table_identity
                .as_deref()
        ),
        optional_string_text(
            report
                .final_output_nsdb_runtime_dispatch_capability_mask
                .as_deref()
        ),
        optional_number(report.final_output_nsdb_runtime_dispatch_slot),
        optional_number(report.final_output_nsdb_runtime_dispatch_status_code),
        report
            .final_output_nsdb_runtime_dispatch_acknowledged
            .map(|value| value.to_string())
            .unwrap_or_else(|| "<none>".to_owned())
    );
}

fn optional_number(value: Option<impl ToString>) -> String {
    value.map_or_else(|| "<none>".to_owned(), |value| value.to_string())
}
