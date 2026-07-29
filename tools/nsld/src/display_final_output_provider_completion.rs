use super::reports::NsldFinalExecutableOutputReport;

pub(crate) fn display_provider_completions(report: &NsldFinalExecutableOutputReport) {
    for completion in &report.final_output_nsdb_provider_completions {
        let compiled = &completion.compiled_code_asset_selection;
        println!(
            "  final_output_nsdb_provider_completion: {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
            completion.trace_id,
            completion.provider_family,
            completion.output_contract,
            completion.output_evidence,
            completion.completion_evidence_contract,
            completion.completion_evidence_status,
            completion.completion_evidence_count,
            completion.completion_clock_evidence,
            completion.completion_tokens,
            completion.glm_release_contract,
            completion.glm_release_tokens,
            completion.glm_release_status,
            completion.code_asset_identity_contract,
            completion.code_asset_identity_status,
            completion.code_asset_identity_asset_id,
            completion.code_asset_identity_hash,
            completion.code_asset_identity_set_contract,
            completion.code_asset_identity_set_status,
            completion.code_asset_identity_set_count,
            completion.code_asset_identity_set_root_hash,
            compiled.contract,
            compiled.status,
            compiled.table_contract,
            compiled.table_hash,
            compiled.contribution_count,
            compiled.identity_set_root_hash,
            compiled.contribution_index,
            compiled.asset_id,
            compiled.identity_hash,
            compiled.selections.len(),
            compiled
                .selections
                .iter()
                .map(|item| item.asset_id.as_str())
                .collect::<Vec<_>>()
                .join(","),
            completion.record_hash
        );
    }
}
