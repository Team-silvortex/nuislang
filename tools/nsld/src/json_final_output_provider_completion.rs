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
                conformance_json(&completion.conformance),
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
                json_usize_field(
                    "compiled_code_asset_selection_count",
                    completion.compiled_code_asset_selection.selections.len(),
                ),
                format!(
                    "\"compiled_code_asset_selections\":[{}]",
                    completion
                        .compiled_code_asset_selection
                        .selections
                        .iter()
                        .map(|item| format!(
                            "{{{},{},{}}}",
                            json_usize_field("contribution_index", item.contribution_index),
                            json_string_field("asset_id", &item.asset_id),
                            json_string_field("identity_hash", &item.identity_hash),
                        ))
                        .collect::<Vec<_>>()
                        .join(",")
                ),
                json_string_field(
                    "request_completion_contract",
                    &completion.request_completion_contract,
                ),
                json_string_field(
                    "request_completion_status",
                    &completion.request_completion_status,
                ),
                json_usize_field(
                    "request_completion_count",
                    completion.request_completion_count,
                ),
                json_string_field(
                    "request_completion_root_hash",
                    &completion.request_completion_root_hash,
                ),
                format!(
                    "\"request_completions\":[{}]",
                    completion
                        .request_completions
                        .iter()
                        .map(|request| format!(
                            "{{{},{},{},{},{},{},{},{},{}}}",
                            json_string_field("contract", &request.contract),
                            json_string_field("status", &request.status),
                            json_string_field("request_id", &request.request_id),
                            json_string_field("provider_family", &request.provider_family),
                            json_string_field("dispatch_id", &request.dispatch_id),
                            json_string_field("completion_clock", &request.completion_clock),
                            json_string_field("output_hash", &request.output_hash),
                            json_string_field("completion_token", &request.completion_token),
                            json_string_field("selected_set_hash", &request.selected_set_hash),
                        ))
                        .collect::<Vec<_>>()
                        .join(",")
                ),
                json_string_field("record_hash", &completion.record_hash),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("\"final_output_nsdb_provider_completions\":[{records}]")
}

fn conformance_json(evidence: &nsdb::ProviderConformanceLifecycleEvidence) -> String {
    let fields = [
        json_string_field("capsule_contract", &evidence.capsule_contract),
        json_string_field("status", &evidence.status),
        json_string_field("scenario_contract", &evidence.scenario_contract),
        json_string_field("scenario_id", &evidence.scenario_id),
        json_string_field("package_id", &evidence.package_id),
        json_string_field("provider_id", &evidence.provider_id),
        json_string_field("bundle_id", &evidence.bundle_id),
        json_string_field("provider_family", &evidence.provider_family),
        json_string_field(
            "capability_selection_hash",
            &evidence.capability_selection_hash,
        ),
        json_string_field("capsule_hash", &evidence.capsule_hash),
        json_string_field("replay_contract", &evidence.replay_contract),
        json_string_field("replay_status", &evidence.replay_status),
        json_string_field("replay_hash", &evidence.replay_hash),
        json_string_field("execution_authority", &evidence.execution_authority),
        json_bool_field(
            "physical_execution_claimed",
            evidence.physical_execution_claimed,
        ),
    ];
    format!("\"conformance\":{{{}}}", fields.join(","))
}
