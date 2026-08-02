use crate::artifact_nsdb_handoff_dispatch::PersistedProviderCompletion;

use super::link_plan_final_output_summary::{
    ProviderCompletionBoundarySummary, ProviderRequestCompletionBoundarySummary,
    ProviderRequestCompletionCollectionBoundarySummary,
};

pub(super) fn boundary_completion(
    completion: &PersistedProviderCompletion,
) -> ProviderCompletionBoundarySummary {
    let request_completion = &completion.request_completion;
    ProviderCompletionBoundarySummary {
        trace_id: completion.trace_id.clone(),
        provider_family: completion.provider_family.clone(),
        output_contract: completion.output_contract.clone(),
        output_evidence: completion.output_evidence.clone(),
        dispatch_selected_set_hash: completion.dispatch_selected_set_hash.clone(),
        request_completion: ProviderRequestCompletionCollectionBoundarySummary {
            source_trace_id: completion.trace_id.clone(),
            source_provider_family: completion.provider_family.clone(),
            dispatch_selected_set_hash: completion.dispatch_selected_set_hash.clone(),
            contract: request_completion.contract.clone(),
            status: request_completion.status.clone(),
            count: request_completion.count,
            root_hash: request_completion.root_hash.clone(),
            validation_status: request_completion.validation_status.clone(),
            receipts: request_completion
                .receipts
                .iter()
                .map(|receipt| ProviderRequestCompletionBoundarySummary {
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
        },
        record_hash: completion.record_hash.clone(),
    }
}
