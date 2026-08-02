use crate::{
    model::NsdbInspectReport,
    provider_request_completion::{
        REQUEST_COMPLETION_COLLECTION_CONTRACT, REQUEST_COMPLETION_RECEIPT_CONTRACT,
    },
};

pub(crate) const REQUEST_REPLAY_SELECTOR_CONTRACT: &str =
    "nsdb-provider-request-replay-selector-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsdbRequestReplaySelection {
    pub(crate) contract: &'static str,
    pub(crate) status: &'static str,
    pub(crate) ready: bool,
    pub(crate) request_id: Option<String>,
    pub(crate) source_trace_id: Option<String>,
    pub(crate) source_provider_family: Option<String>,
    pub(crate) collection_root_hash: Option<String>,
    pub(crate) provider_family: Option<String>,
    pub(crate) dispatch_id: Option<String>,
    pub(crate) completion_clock: Option<String>,
    pub(crate) output_hash: Option<String>,
    pub(crate) completion_token: Option<String>,
    pub(crate) selected_set_hash: Option<String>,
    pub(crate) first_blocker: Option<String>,
}

pub(crate) fn resolve_request_replay_selection(
    report: &NsdbInspectReport,
    request_id: Option<&str>,
) -> NsdbRequestReplaySelection {
    let Some(request_id) = request_id else {
        return selection("not-requested", false, None, None);
    };
    if request_id.is_empty() || request_id == "none" {
        return selection(
            "blocked",
            false,
            Some(request_id),
            Some("request-replay-selector:request-id-invalid".to_owned()),
        );
    }
    let handoff = &report.payload_execution_handoff;
    if !handoff.available || handoff.status != "ready" {
        return selection(
            "blocked",
            false,
            Some(request_id),
            Some("request-replay-selector:handoff-unverified".to_owned()),
        );
    }
    let dispatch = &handoff.provider_completion_dispatch_identity;
    if dispatch.status != "verified" || dispatch.selected_set_hash == "none" {
        return selection(
            "blocked",
            false,
            Some(request_id),
            Some("request-replay-selector:dispatch-authority-unverified".to_owned()),
        );
    }

    let mut matches = Vec::new();
    let mut integrity_blocker = None;
    for event in &handoff.events {
        let collection = &event.provider_completion_evidence.request_completions;
        for receipt in &collection.receipts {
            if receipt.request_id != request_id {
                continue;
            }
            let valid = event.execution_phase == "provider-device-completion"
                && collection.contract == REQUEST_COMPLETION_COLLECTION_CONTRACT
                && collection.status == "verified"
                && collection.count == collection.receipts.len()
                && collection.root_hash != "none"
                && receipt.contract == REQUEST_COMPLETION_RECEIPT_CONTRACT
                && receipt.status == "verified"
                && receipt.selected_set_hash == dispatch.selected_set_hash
                && event.provider_completion_dispatch.status == "verified"
                && event.provider_completion_dispatch.selected_set_hash
                    == dispatch.selected_set_hash
                && [
                    receipt.provider_family.as_str(),
                    receipt.dispatch_id.as_str(),
                    receipt.completion_clock.as_str(),
                    receipt.output_hash.as_str(),
                    receipt.completion_token.as_str(),
                ]
                .iter()
                .all(|value| !matches!(*value, "" | "none"));
            if !valid {
                integrity_blocker = Some(format!(
                    "request-replay-selector:receipt-unverified:{request_id}"
                ));
                continue;
            }
            matches.push((event, collection, receipt));
        }
    }
    if let Some(blocker) = integrity_blocker {
        return selection("blocked", false, Some(request_id), Some(blocker));
    }
    if matches.len() != 1 {
        let reason = if matches.is_empty() {
            "request-not-found"
        } else {
            "request-ambiguous"
        };
        return selection(
            reason,
            false,
            Some(request_id),
            Some(format!("request-replay-selector:{reason}:{request_id}")),
        );
    }
    let (event, collection, receipt) = matches[0];
    NsdbRequestReplaySelection {
        contract: REQUEST_REPLAY_SELECTOR_CONTRACT,
        status: "request-resolved",
        ready: true,
        request_id: Some(request_id.to_owned()),
        source_trace_id: Some(event.trace_id.clone()),
        source_provider_family: Some(event.provider_family.clone()),
        collection_root_hash: Some(collection.root_hash.clone()),
        provider_family: Some(receipt.provider_family.clone()),
        dispatch_id: Some(receipt.dispatch_id.clone()),
        completion_clock: Some(receipt.completion_clock.clone()),
        output_hash: Some(receipt.output_hash.clone()),
        completion_token: Some(receipt.completion_token.clone()),
        selected_set_hash: Some(receipt.selected_set_hash.clone()),
        first_blocker: None,
    }
}

fn selection(
    status: &'static str,
    ready: bool,
    request_id: Option<&str>,
    first_blocker: Option<String>,
) -> NsdbRequestReplaySelection {
    NsdbRequestReplaySelection {
        contract: REQUEST_REPLAY_SELECTOR_CONTRACT,
        status,
        ready,
        request_id: request_id.map(str::to_owned),
        source_trace_id: None,
        source_provider_family: None,
        collection_root_hash: None,
        provider_family: None,
        dispatch_id: None,
        completion_clock: None,
        output_hash: None,
        completion_token: None,
        selected_set_hash: None,
        first_blocker,
    }
}
