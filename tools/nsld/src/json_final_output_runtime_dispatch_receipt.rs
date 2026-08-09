use super::{json_fields::*, reports::NsldFinalExecutableOutputReport};

pub(super) fn runtime_dispatch_receipt_json_fields(
    report: &NsldFinalExecutableOutputReport,
) -> Vec<String> {
    vec![
        json_optional_string_field(
            "final_output_nsdb_runtime_dispatch_receipt_contract",
            report
                .final_output_nsdb_runtime_dispatch_receipt_contract
                .as_deref(),
        ),
        json_string_field(
            "final_output_nsdb_runtime_dispatch_receipt_status",
            &report.final_output_nsdb_runtime_dispatch_receipt_status,
        ),
        json_optional_string_field(
            "final_output_nsdb_runtime_dispatch_receipt_hash",
            report
                .final_output_nsdb_runtime_dispatch_receipt_hash
                .as_deref(),
        ),
        json_optional_string_field(
            "final_output_nsdb_runtime_dispatch_execution_identity_hash",
            report
                .final_output_nsdb_runtime_dispatch_execution_identity_hash
                .as_deref(),
        ),
        json_optional_string_field(
            "final_output_nsdb_runtime_dispatch_import_identity_hash",
            report
                .final_output_nsdb_runtime_dispatch_import_identity_hash
                .as_deref(),
        ),
        json_optional_string_field(
            "final_output_nsdb_runtime_dispatch_table_identity",
            report
                .final_output_nsdb_runtime_dispatch_table_identity
                .as_deref(),
        ),
        json_optional_string_field(
            "final_output_nsdb_runtime_dispatch_capability_mask",
            report
                .final_output_nsdb_runtime_dispatch_capability_mask
                .as_deref(),
        ),
        json_optional_usize_field(
            "final_output_nsdb_runtime_dispatch_slot",
            report
                .final_output_nsdb_runtime_dispatch_slot
                .map(|value| value as usize),
        ),
        json_optional_isize_field(
            "final_output_nsdb_runtime_dispatch_status_code",
            report
                .final_output_nsdb_runtime_dispatch_status_code
                .map(|value| value as isize),
        ),
        json_optional_bool_field(
            "final_output_nsdb_runtime_dispatch_acknowledged",
            report.final_output_nsdb_runtime_dispatch_acknowledged,
        ),
    ]
}
