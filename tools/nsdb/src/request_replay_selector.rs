use crate::{
    model::NsdbInspectReport,
    request_replay_frames::{verified_request_replay_frames, NsdbRequestReplayFrameEvidence},
};

pub(crate) const REQUEST_REPLAY_SELECTOR_CONTRACT: &str =
    "nsdb-provider-request-replay-selector-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsdbRequestReplaySelection {
    pub(crate) contract: &'static str,
    pub(crate) status: &'static str,
    pub(crate) ready: bool,
    pub(crate) request_id: Option<String>,
    pub(crate) request_frame_id: Option<String>,
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

    let frames = match verified_request_replay_frames(report) {
        Ok(frames) => frames,
        Err(blocker) => {
            return selection(
                "blocked",
                false,
                Some(request_id),
                Some(blocker.replacen("request-replay-frame", "request-replay-selector", 1)),
            );
        }
    };
    let matches = frames
        .iter()
        .filter(|frame| frame.request_id == request_id)
        .collect::<Vec<_>>();
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
    resolved_selection(matches[0])
}

fn resolved_selection(frame: &NsdbRequestReplayFrameEvidence) -> NsdbRequestReplaySelection {
    NsdbRequestReplaySelection {
        contract: REQUEST_REPLAY_SELECTOR_CONTRACT,
        status: "request-resolved",
        ready: true,
        request_id: Some(frame.request_id.clone()),
        request_frame_id: Some(frame.frame_id.clone()),
        source_trace_id: Some(frame.source_trace_id.clone()),
        source_provider_family: Some(frame.source_provider_family.clone()),
        collection_root_hash: Some(frame.collection_root_hash.clone()),
        provider_family: Some(frame.provider_family.clone()),
        dispatch_id: Some(frame.dispatch_id.clone()),
        completion_clock: Some(frame.completion_clock.clone()),
        output_hash: Some(frame.output_hash.clone()),
        completion_token: Some(frame.completion_token.clone()),
        selected_set_hash: Some(frame.selected_set_hash.clone()),
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
        request_frame_id: None,
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
