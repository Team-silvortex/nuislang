use super::link_plan_final_output_summary::{
    ProviderCompletionBoundarySummary, ProviderRequestCompletionCollectionBoundarySummary,
};

pub(crate) const VALIDATED_PROVIDER_REQUEST_COMPLETION_CAPABILITY_CONTRACT: &str =
    "nuis-validated-provider-request-completion-capability-v1";
pub(crate) const FINAL_OUTPUT_REQUEST_COMPLETION_PROJECTION_SOURCE: &str =
    "final_output_provider_request_completion_collections";

#[derive(Clone)]
pub(crate) struct ValidatedProviderRequestCompletionCapability {
    pub(crate) contract: &'static str,
    pub(crate) source_contract: String,
    pub(crate) source_status: String,
    pub(crate) projection_source: &'static str,
    pub(crate) ready: bool,
    pub(crate) status: &'static str,
    pub(crate) collection_count: usize,
    pub(crate) receipt_count: usize,
    pub(crate) collections: Vec<ProviderRequestCompletionCollectionBoundarySummary>,
    pub(crate) first_blocker: Option<String>,
}

pub(crate) fn validated_provider_request_completion_capability(
    source_contract: &str,
    source_status: &str,
    completions: &[ProviderCompletionBoundarySummary],
) -> ValidatedProviderRequestCompletionCapability {
    if completions.is_empty() {
        return capability(
            source_contract,
            source_status,
            true,
            "verified-empty",
            Vec::new(),
            None,
        );
    }

    let collections = completions
        .iter()
        .map(|completion| completion.request_completion.clone())
        .collect::<Vec<_>>();
    let first_blocker = completions
        .iter()
        .enumerate()
        .find_map(|(index, completion)| collection_blocker(index, completion));
    capability(
        source_contract,
        source_status,
        first_blocker.is_none(),
        if first_blocker.is_none() {
            "verified"
        } else {
            "blocked"
        },
        collections,
        first_blocker,
    )
}

fn collection_blocker(
    index: usize,
    completion: &ProviderCompletionBoundarySummary,
) -> Option<String> {
    let collection = &completion.request_completion;
    if collection.validation_status != "verified"
        || collection.status != "verified"
        || collection.count == 0
        || collection.count != collection.receipts.len()
        || collection.root_hash == "none"
    {
        return Some(format!(
            "provider-request-completion-collection-{index}-unverified"
        ));
    }
    if collection.source_trace_id != completion.trace_id
        || collection.source_provider_family != completion.provider_family
        || collection.dispatch_selected_set_hash != completion.dispatch_selected_set_hash
    {
        return Some(format!(
            "provider-request-completion-collection-{index}-source-mismatch"
        ));
    }
    if completion.dispatch_selected_set_hash == "none"
        || collection
            .receipts
            .iter()
            .any(|receipt| receipt.selected_set_hash != completion.dispatch_selected_set_hash)
    {
        return Some(format!(
            "provider-request-completion-collection-{index}-selected-set-mismatch"
        ));
    }
    None
}

fn capability(
    source_contract: &str,
    source_status: &str,
    ready: bool,
    status: &'static str,
    collections: Vec<ProviderRequestCompletionCollectionBoundarySummary>,
    first_blocker: Option<String>,
) -> ValidatedProviderRequestCompletionCapability {
    let receipt_count = collections
        .iter()
        .map(|collection| collection.receipts.len())
        .sum();
    ValidatedProviderRequestCompletionCapability {
        contract: VALIDATED_PROVIDER_REQUEST_COMPLETION_CAPABILITY_CONTRACT,
        source_contract: source_contract.to_owned(),
        source_status: source_status.to_owned(),
        projection_source: FINAL_OUTPUT_REQUEST_COMPLETION_PROJECTION_SOURCE,
        ready,
        status,
        collection_count: collections.len(),
        receipt_count,
        collections,
        first_blocker,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::link_plan_final_output_summary::{
        ProviderRequestCompletionBoundarySummary,
        ProviderRequestCompletionCollectionBoundarySummary,
    };

    #[test]
    fn capability_binds_every_receipt_to_its_sealed_dispatch_root() {
        let completion = completion("fnv1a64:1111111111111111");
        let capability = validated_provider_request_completion_capability(
            "nsdb-yir-replay-summary-v1",
            "replay-ready",
            &[completion],
        );

        assert!(capability.ready);
        assert_eq!(capability.status, "verified");
        assert_eq!(capability.collection_count, 1);
        assert_eq!(capability.receipt_count, 1);
    }

    #[test]
    fn capability_rejects_a_receipt_from_another_selected_set() {
        let mut completion = completion("fnv1a64:1111111111111111");
        completion.request_completion.receipts[0].selected_set_hash =
            "fnv1a64:2222222222222222".to_owned();
        let capability = validated_provider_request_completion_capability(
            "nsdb-yir-replay-summary-v1",
            "replay-ready",
            &[completion],
        );

        assert!(!capability.ready);
        assert_eq!(capability.status, "blocked");
        assert_eq!(
            capability.first_blocker.as_deref(),
            Some("provider-request-completion-collection-0-selected-set-mismatch")
        );
    }

    fn completion(selected_set_hash: &str) -> ProviderCompletionBoundarySummary {
        ProviderCompletionBoundarySummary {
            trace_id: "trace-0001".to_owned(),
            provider_family: "cuda".to_owned(),
            output_contract: "nuis-cuda-output-v1".to_owned(),
            output_evidence: "cuda:ok".to_owned(),
            dispatch_selected_set_hash: selected_set_hash.to_owned(),
            request_completion: ProviderRequestCompletionCollectionBoundarySummary {
                source_trace_id: "trace-0001".to_owned(),
                source_provider_family: "cuda".to_owned(),
                dispatch_selected_set_hash: selected_set_hash.to_owned(),
                contract: "nuis-provider-request-completion-receipt-collection-v1".to_owned(),
                status: "verified".to_owned(),
                count: 1,
                root_hash: "fnv1a64:aaaaaaaaaaaaaaaa".to_owned(),
                validation_status: "verified".to_owned(),
                receipts: vec![ProviderRequestCompletionBoundarySummary {
                    contract: "nuis-provider-request-completion-receipt-v1".to_owned(),
                    status: "verified".to_owned(),
                    request_id: "request-0000".to_owned(),
                    provider_family: "cuda".to_owned(),
                    dispatch_id: "dispatch-0000".to_owned(),
                    completion_clock: "1:0".to_owned(),
                    output_hash: "fnv1a64:bbbbbbbbbbbbbbbb".to_owned(),
                    completion_token: "completion:0000".to_owned(),
                    selected_set_hash: selected_set_hash.to_owned(),
                }],
            },
            record_hash: "fnv1a64:cccccccccccccccc".to_owned(),
        }
    }
}
