use crate::model::{
    NsdbPayloadExecutionEvent, PayloadExecutionProviderCompletion,
    PayloadExecutionProviderRequestCompletion,
};

pub(crate) fn public_completion(
    event: &NsdbPayloadExecutionEvent,
    digest_contract: &str,
) -> PayloadExecutionProviderCompletion {
    let evidence = &event.provider_completion_evidence;
    let requests = &evidence.request_completions;
    let dispatch = &event.provider_completion_dispatch;
    PayloadExecutionProviderCompletion {
        trace_id: event.trace_id.clone(),
        provider_family: event.provider_family.clone(),
        output_contract: event.output_contract.clone(),
        output_evidence: event.output_evidence.clone(),
        completion_evidence_contract: evidence.contract.clone(),
        completion_evidence_status: evidence.status.clone(),
        completion_evidence_count: evidence.count,
        completion_clock_evidence: evidence.clock_evidence.clone(),
        completion_tokens: evidence.completion_tokens.clone(),
        glm_release_contract: evidence.glm_release_contract.clone(),
        glm_release_tokens: evidence.glm_release_tokens.clone(),
        glm_release_status: evidence.glm_release_status.clone(),
        code_asset_identity_contract: evidence.code_asset_identity_contract.clone(),
        code_asset_identity_status: evidence.code_asset_identity_status.clone(),
        code_asset_identity_asset_id: evidence.code_asset_identity_asset_id.clone(),
        code_asset_identity_hash: evidence.code_asset_identity_hash.clone(),
        code_asset_identity_set_contract: evidence.code_asset_identity_set_contract.clone(),
        code_asset_identity_set_status: evidence.code_asset_identity_set_status.clone(),
        code_asset_identity_set_count: evidence.code_asset_identity_set_count,
        code_asset_identity_set_root_hash: evidence.code_asset_identity_set_root_hash.clone(),
        compiled_code_asset_selection: evidence.compiled_code_asset_selection.clone(),
        request_completion_contract: requests.contract.clone(),
        request_completion_status: requests.status.clone(),
        request_completion_count: requests.count,
        request_completion_root_hash: requests.root_hash.clone(),
        request_completions: requests
            .receipts
            .iter()
            .map(|receipt| PayloadExecutionProviderRequestCompletion {
                contract: receipt.contract.clone(),
                status: receipt.status.clone(),
                request_id: receipt.request_id.clone(),
                provider_family: receipt.provider_family.clone(),
                dispatch_id: receipt.dispatch_id.clone(),
                completion_clock: receipt.completion_clock.clone(),
                output_hash: receipt.output_hash.clone(),
                completion_token: receipt.completion_token.clone(),
                selected_set_hash: receipt.selected_set_hash.clone(),
            })
            .collect(),
        dispatch_authority_contract: dispatch.contract.clone(),
        dispatch_authority_status: dispatch.status.clone(),
        dispatch_table_hash: dispatch.table_hash.clone(),
        dispatch_selected_set_hash: dispatch.selected_set_hash.clone(),
        dispatch_id: dispatch.dispatch_id.clone(),
        dispatch_package_id: dispatch.package_id.clone(),
        dispatch_bundle_id: dispatch.bundle_id.clone(),
        dispatch_runner_adapter_id: dispatch.runner_adapter_id.clone(),
        record_hash: crate::provider_completion_integrity::record_hash(event, digest_contract)
            .unwrap_or_else(|| "none".to_owned()),
    }
}
