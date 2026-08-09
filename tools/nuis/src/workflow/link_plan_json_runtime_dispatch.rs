use super::*;

pub(crate) fn runtime_dispatch_json_fields(
    summary: Option<&NsldFinalExecutableOutputBoundarySummary>,
) -> Vec<String> {
    let receipt = summary.map(|summary| &summary.runtime_dispatch_receipt);
    vec![
        json_optional_string_field(
            "nsld_final_executable_output_nsdb_replay_contract",
            summary.map(|summary| summary.nsdb_replay_contract.as_str()),
        ),
        json_bool_field(
            "nsld_final_executable_output_nsdb_replay_ready",
            summary.is_some_and(|summary| summary.nsdb_replay_ready),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_nsdb_replay_status",
            summary.map(|summary| summary.nsdb_replay_status.as_str()),
        ),
        json_usize_field(
            "nsld_final_executable_output_nsdb_replay_checkpoint_count",
            summary
                .map(|summary| summary.nsdb_replay_checkpoint_count)
                .unwrap_or(0),
        ),
        json_usize_field(
            "nsld_final_executable_output_nsdb_replayable_checkpoint_count",
            summary
                .map(|summary| summary.nsdb_replayable_checkpoint_count)
                .unwrap_or(0),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_nsdb_runtime_dispatch_receipt_contract",
            receipt.and_then(|receipt| receipt.contract.as_deref()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_nsdb_runtime_dispatch_receipt_status",
            receipt.map(|receipt| receipt.status.as_str()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_nsdb_runtime_dispatch_receipt_hash",
            receipt.and_then(|receipt| receipt.receipt_hash.as_deref()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_nsdb_runtime_dispatch_execution_identity_hash",
            receipt.and_then(|receipt| receipt.execution_identity_hash.as_deref()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_nsdb_runtime_dispatch_import_identity_hash",
            receipt.and_then(|receipt| receipt.import_identity_hash.as_deref()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_nsdb_runtime_dispatch_table_identity",
            receipt.and_then(|receipt| receipt.table_identity.as_deref()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_nsdb_runtime_dispatch_capability_mask",
            receipt.and_then(|receipt| receipt.capability_mask.as_deref()),
        ),
        optional_u32_field(
            "nsld_final_executable_output_nsdb_runtime_dispatch_slot",
            receipt.and_then(|receipt| receipt.slot),
        ),
        optional_i32_field(
            "nsld_final_executable_output_nsdb_runtime_dispatch_status_code",
            receipt.and_then(|receipt| receipt.status_code),
        ),
        json_optional_bool_field(
            "nsld_final_executable_output_nsdb_runtime_dispatch_acknowledged",
            receipt.and_then(|receipt| receipt.acknowledged),
        ),
        json_usize_field(
            "nsld_final_executable_output_nsdb_provider_completion_count",
            summary
                .map(|summary| summary.nsdb_provider_completion_count)
                .unwrap_or(0),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_nsdb_first_provider_family",
            summary.and_then(|summary| summary.nsdb_first_provider_family.as_deref()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_nsdb_first_provider_output_contract",
            summary.and_then(|summary| summary.nsdb_first_provider_output_contract.as_deref()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_nsdb_first_provider_output_evidence",
            summary.and_then(|summary| summary.nsdb_first_provider_output_evidence.as_deref()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_nsdb_replay_command",
            summary.and_then(|summary| summary.nsdb_replay_command.as_deref()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_nsdb_replay_next_action",
            summary.map(|summary| summary.nsdb_replay_next_action.as_str()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_nsdb_replay_next_command",
            summary.and_then(|summary| summary.nsdb_replay_next_command.as_deref()),
        ),
        json_optional_string_field(
            "nsld_final_executable_output_nsdb_replay_first_blocker",
            summary.and_then(|summary| summary.nsdb_replay_first_blocker.as_deref()),
        ),
    ]
}

fn optional_u32_field(name: &str, value: Option<u32>) -> String {
    value.map_or_else(
        || format!("\"{name}\":null"),
        |value| format!("\"{name}\":{value}"),
    )
}

fn optional_i32_field(name: &str, value: Option<i32>) -> String {
    value.map_or_else(
        || format!("\"{name}\":null"),
        |value| format!("\"{name}\":{value}"),
    )
}
