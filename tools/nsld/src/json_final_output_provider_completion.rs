use super::{json_fields::*, reports::NsldFinalExecutableOutputReport};

pub(crate) fn provider_completion_records_json(report: &NsldFinalExecutableOutputReport) -> String {
    let records = report
        .final_output_nsdb_provider_completions
        .iter()
        .map(|completion| {
            let fields = [
                json_string_field("trace_id", &completion.trace_id),
                json_string_field("provider_family", &completion.provider_family),
                json_string_field("output_contract", &completion.output_contract),
                json_string_field("output_evidence", &completion.output_evidence),
                json_string_field(
                    "completion_evidence_contract",
                    &completion.completion_evidence_contract,
                ),
                json_string_field(
                    "completion_evidence_status",
                    &completion.completion_evidence_status,
                ),
                json_usize_field(
                    "completion_evidence_count",
                    completion.completion_evidence_count,
                ),
                json_string_field(
                    "completion_clock_evidence",
                    &completion.completion_clock_evidence,
                ),
                json_string_field("completion_tokens", &completion.completion_tokens),
                json_string_field("glm_release_contract", &completion.glm_release_contract),
                json_string_field("glm_release_tokens", &completion.glm_release_tokens),
                json_string_field("glm_release_status", &completion.glm_release_status),
                json_string_field(
                    "code_asset_identity_contract",
                    &completion.code_asset_identity_contract,
                ),
                json_string_field(
                    "code_asset_identity_status",
                    &completion.code_asset_identity_status,
                ),
                json_string_field(
                    "code_asset_identity_asset_id",
                    &completion.code_asset_identity_asset_id,
                ),
                json_string_field(
                    "code_asset_identity_hash",
                    &completion.code_asset_identity_hash,
                ),
                json_string_field(
                    "code_asset_identity_set_contract",
                    &completion.code_asset_identity_set_contract,
                ),
                json_string_field(
                    "code_asset_identity_set_status",
                    &completion.code_asset_identity_set_status,
                ),
                json_usize_field(
                    "code_asset_identity_set_count",
                    completion.code_asset_identity_set_count,
                ),
                json_string_field(
                    "code_asset_identity_set_root_hash",
                    &completion.code_asset_identity_set_root_hash,
                ),
                json_string_field(
                    "compiled_code_asset_selection_contract",
                    &completion.compiled_code_asset_selection.contract,
                ),
                json_string_field(
                    "compiled_code_asset_selection_status",
                    &completion.compiled_code_asset_selection.status,
                ),
                json_string_field(
                    "compiled_code_asset_table_contract",
                    &completion.compiled_code_asset_selection.table_contract,
                ),
                json_string_field(
                    "compiled_code_asset_table_hash",
                    &completion.compiled_code_asset_selection.table_hash,
                ),
                json_usize_field(
                    "compiled_code_asset_contribution_count",
                    completion.compiled_code_asset_selection.contribution_count,
                ),
                json_string_field(
                    "compiled_code_asset_identity_set_root_hash",
                    &completion
                        .compiled_code_asset_selection
                        .identity_set_root_hash,
                ),
                json_usize_field(
                    "compiled_code_asset_contribution_index",
                    completion.compiled_code_asset_selection.contribution_index,
                ),
                json_string_field(
                    "compiled_code_asset_asset_id",
                    &completion.compiled_code_asset_selection.asset_id,
                ),
                json_string_field(
                    "compiled_code_asset_identity_hash",
                    &completion.compiled_code_asset_selection.identity_hash,
                ),
                json_string_field("record_hash", &completion.record_hash),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("\"final_output_nsdb_provider_completions\":[{records}]")
}
