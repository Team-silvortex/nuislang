use crate::{
    model::NsdbInspectReport,
    transcript::{NsdbReplayTranscriptFrame, ReplayControlResult},
};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const REQUEST_REPLAY_FRAME_CONTRACT: &str = "nsdb-provider-request-replay-frame-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsdbRequestReplayFrameEvidence {
    pub(crate) contract: &'static str,
    pub(crate) frame_id: String,
    pub(crate) parent_frame_id: String,
    pub(crate) ordinal: usize,
    pub(crate) source_trace_id: String,
    pub(crate) source_provider_family: String,
    pub(crate) collection_root_hash: String,
    pub(crate) request_id: String,
    pub(crate) provider_family: String,
    pub(crate) dispatch_id: String,
    pub(crate) completion_clock: String,
    pub(crate) output_hash: String,
    pub(crate) completion_token: String,
    pub(crate) selected_set_hash: String,
}

pub(crate) struct NsdbRequestReplayFrameExpansion {
    pub(crate) frames: Vec<NsdbReplayTranscriptFrame>,
    pub(crate) request_frame_count: usize,
    pub(crate) first_blocker: Option<String>,
}

pub(crate) fn verified_request_replay_frames(
    report: &NsdbInspectReport,
) -> Result<Vec<NsdbRequestReplayFrameEvidence>, String> {
    let handoff = &report.payload_execution_handoff;
    let has_collections = handoff.events.iter().any(|event| {
        event
            .provider_completion_evidence
            .request_completions
            .present
    });
    if !has_collections {
        return Ok(Vec::new());
    }
    if !handoff.available || handoff.status != "ready" {
        return Err("request-replay-frame:handoff-unverified".to_owned());
    }
    let identity = &handoff.provider_completion_dispatch_identity;
    if identity.status != "verified" || identity.selected_set_hash == "none" {
        return Err("request-replay-frame:dispatch-authority-unverified".to_owned());
    }

    let mut request_ids = BTreeSet::new();
    let mut frame_ids = BTreeSet::new();
    let mut frames = Vec::new();
    for event in &handoff.events {
        let collection = &event.provider_completion_evidence.request_completions;
        if !collection.present {
            continue;
        }
        if event.execution_phase != "provider-device-completion"
            || collection.status != "verified"
            || !crate::provider_request_completion::verified_shape(collection)
            || event.provider_completion_dispatch.status != "verified"
            || event.provider_completion_dispatch.selected_set_hash != identity.selected_set_hash
        {
            return Err(format!(
                "request-replay-frame:collection-unverified:{}",
                event.trace_id
            ));
        }
        let parent_frame_id = crate::replay::frame_id_for_event(event);
        for (ordinal, receipt) in collection.receipts.iter().enumerate() {
            if receipt.selected_set_hash != identity.selected_set_hash {
                return Err(format!(
                    "request-replay-frame:selected-set-mismatch:{}",
                    receipt.request_id
                ));
            }
            if !request_ids.insert(receipt.request_id.clone()) {
                return Err(format!(
                    "request-replay-frame:request-ambiguous:{}",
                    receipt.request_id
                ));
            }
            let frame_id = request_frame_id(
                event.index,
                ordinal,
                &event.trace_id,
                &collection.root_hash,
                receipt,
            );
            if !frame_ids.insert(frame_id.clone()) {
                return Err(format!("request-replay-frame:id-collision:{frame_id}"));
            }
            frames.push(NsdbRequestReplayFrameEvidence {
                contract: REQUEST_REPLAY_FRAME_CONTRACT,
                frame_id,
                parent_frame_id: parent_frame_id.clone(),
                ordinal,
                source_trace_id: event.trace_id.clone(),
                source_provider_family: event.provider_family.clone(),
                collection_root_hash: collection.root_hash.clone(),
                request_id: receipt.request_id.clone(),
                provider_family: receipt.provider_family.clone(),
                dispatch_id: receipt.dispatch_id.clone(),
                completion_clock: receipt.completion_clock.clone(),
                output_hash: receipt.output_hash.clone(),
                completion_token: receipt.completion_token.clone(),
                selected_set_hash: receipt.selected_set_hash.clone(),
            });
        }
    }
    Ok(frames)
}

pub(crate) fn expand_request_replay_frames(
    report: &NsdbInspectReport,
    frames: Vec<NsdbReplayTranscriptFrame>,
) -> NsdbRequestReplayFrameExpansion {
    let evidence = match verified_request_replay_frames(report) {
        Ok(evidence) => evidence,
        Err(blocker) => {
            return NsdbRequestReplayFrameExpansion {
                frames,
                request_frame_count: 0,
                first_blocker: Some(blocker),
            };
        }
    };
    match expand_verified_request_replay_frames(frames, evidence) {
        Ok(expansion) => expansion,
        Err((frames, blocker)) => NsdbRequestReplayFrameExpansion {
            frames,
            request_frame_count: 0,
            first_blocker: Some(blocker),
        },
    }
}

fn expand_verified_request_replay_frames(
    frames: Vec<NsdbReplayTranscriptFrame>,
    evidence: Vec<NsdbRequestReplayFrameEvidence>,
) -> Result<NsdbRequestReplayFrameExpansion, (Vec<NsdbReplayTranscriptFrame>, String)> {
    let request_frame_count = evidence.len();
    let parent_ids = frames
        .iter()
        .map(|frame| frame.frame_id.as_str())
        .collect::<BTreeSet<_>>();
    if let Some(missing) = evidence
        .iter()
        .find(|request| !parent_ids.contains(request.parent_frame_id.as_str()))
    {
        return Err((
            frames,
            format!(
                "request-replay-frame:parent-unresolved:{}",
                missing.parent_frame_id
            ),
        ));
    }

    let mut by_parent = BTreeMap::<String, Vec<NsdbRequestReplayFrameEvidence>>::new();
    for request in evidence {
        by_parent
            .entry(request.parent_frame_id.clone())
            .or_default()
            .push(request);
    }
    let mut expanded = Vec::with_capacity(frames.len() + request_frame_count);
    for parent in frames {
        let children = by_parent.remove(&parent.frame_id).unwrap_or_default();
        let request_frames = children
            .into_iter()
            .map(|request| request_transcript_frame(&parent, request))
            .collect::<Vec<_>>();
        expanded.push(parent);
        expanded.extend(request_frames);
    }
    for (index, frame) in expanded.iter_mut().enumerate() {
        frame.index = index;
    }
    Ok(NsdbRequestReplayFrameExpansion {
        frames: expanded,
        request_frame_count,
        first_blocker: None,
    })
}

pub(crate) fn apply_request_replay_control(
    frames: &mut [NsdbReplayTranscriptFrame],
    replay_ready: bool,
    selection: &crate::request_replay_selector::NsdbRequestReplaySelection,
) -> ReplayControlResult {
    if !replay_ready || !selection.ready {
        return ReplayControlResult {
            mode: "request",
            selector: selection.request_id.clone(),
            status: if replay_ready {
                selection.status
            } else {
                "not-evaluated"
            },
            selected_frame_index: None,
            selected_frame_id: None,
            stop_reason: "request-selector-blocked",
            resume_input_status: "not-requested",
            blocker: selection.first_blocker.clone(),
        };
    }
    let request_frame_id = selection.request_frame_id.as_deref().unwrap_or("none");
    let matches = frames
        .iter()
        .enumerate()
        .filter(|(_, frame)| frame.frame_id == request_frame_id)
        .map(|(position, _)| position)
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return ReplayControlResult {
            mode: "request",
            selector: selection.request_id.clone(),
            status: "request-frame-unresolved",
            selected_frame_index: None,
            selected_frame_id: None,
            stop_reason: "request-selector-blocked",
            resume_input_status: "not-requested",
            blocker: Some(format!(
                "request-replay-selector:request-frame-unresolved:{request_frame_id}"
            )),
        };
    }
    let selected_position = matches[0];
    for frame in frames.iter_mut().take(selected_position + 1) {
        frame.consumed = true;
    }
    ReplayControlResult {
        mode: "request",
        selector: selection.request_id.clone(),
        status: "request-selected",
        selected_frame_index: Some(frames[selected_position].index),
        selected_frame_id: Some(frames[selected_position].frame_id.clone()),
        stop_reason: "request-selected",
        resume_input_status: "not-requested",
        blocker: None,
    }
}

fn request_frame_id(
    source_index: usize,
    ordinal: usize,
    trace_id: &str,
    collection_root_hash: &str,
    receipt: &crate::provider_request_completion::ProviderRequestCompletionReceipt,
) -> String {
    let material = format!(
        "{REQUEST_REPLAY_FRAME_CONTRACT}\0{source_index}\0{ordinal}\0{trace_id}\0{collection_root_hash}\0{}\0{}\0{}\0{}",
        receipt.request_id,
        receipt.completion_clock,
        receipt.output_hash,
        receipt.selected_set_hash
    );
    format!(
        "frame:request:{source_index}:{ordinal}:{}",
        crate::digest_sha256::sha256_hex(material.as_bytes())
    )
}

fn request_transcript_frame(
    parent: &NsdbReplayTranscriptFrame,
    request: NsdbRequestReplayFrameEvidence,
) -> NsdbReplayTranscriptFrame {
    let summary = format!(
        "request={};provider={};dispatch={};clock={};output={}",
        request.request_id,
        request.provider_family,
        request.dispatch_id,
        request.completion_clock,
        request.output_hash
    );
    NsdbReplayTranscriptFrame {
        index: 0,
        source_checkpoint_index: parent.source_checkpoint_index,
        frame_scope: "provider-request",
        trace_id: request.source_trace_id.clone(),
        frame_id: request.frame_id.clone(),
        checkpoint_kind: "provider-request-completion".to_owned(),
        execution_phase: "provider-request-completion".to_owned(),
        entry_symbol: request.request_id.clone(),
        replay_status: "replayable".to_owned(),
        consumed: false,
        value_slot_id: format!("slot:request:{}", request.frame_id),
        value_snapshot_status: "request-evidence-ready".to_owned(),
        value_snapshot_type: "provider-request-completion-receipt".to_owned(),
        value_snapshot_summary: summary.clone(),
        value_content_status: "request-evidence-verified".to_owned(),
        value_content_summary: summary,
        next_action: "resume-next-request-frame".to_owned(),
        request: Some(request),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        apply_request_replay_control, expand_verified_request_replay_frames, request_frame_id,
        NsdbRequestReplayFrameEvidence, REQUEST_REPLAY_FRAME_CONTRACT,
    };
    use crate::{
        provider_request_completion::ProviderRequestCompletionReceipt,
        request_replay_selector::NsdbRequestReplaySelection, transcript::NsdbReplayTranscriptFrame,
    };

    fn frame(index: usize, frame_id: &str, scope: &'static str) -> NsdbReplayTranscriptFrame {
        NsdbReplayTranscriptFrame {
            index,
            source_checkpoint_index: 0,
            frame_scope: scope,
            trace_id: "trace-provider".to_owned(),
            frame_id: frame_id.to_owned(),
            checkpoint_kind: "provider-request-completion".to_owned(),
            execution_phase: "provider-request-completion".to_owned(),
            entry_symbol: frame_id.to_owned(),
            replay_status: "replayable".to_owned(),
            consumed: false,
            value_slot_id: format!("slot:{frame_id}"),
            value_snapshot_status: "request-evidence-ready".to_owned(),
            value_snapshot_type: "provider-request-completion-receipt".to_owned(),
            value_snapshot_summary: frame_id.to_owned(),
            value_content_status: "request-evidence-verified".to_owned(),
            value_content_summary: frame_id.to_owned(),
            next_action: "continue".to_owned(),
            request: None,
        }
    }

    fn receipt(clock: &str) -> ProviderRequestCompletionReceipt {
        ProviderRequestCompletionReceipt {
            contract: "nuis-provider-request-completion-receipt-v1".to_owned(),
            status: "verified".to_owned(),
            request_id: "kernel.cuda.copy".to_owned(),
            provider_family: "cuda:nvidia-gpu".to_owned(),
            dispatch_id: "dispatch0001".to_owned(),
            completion_clock: clock.to_owned(),
            output_hash: "0x1111111111111111".to_owned(),
            completion_token: "provider-completion:cuda".to_owned(),
            selected_set_hash: "fnv1a64:2222222222222222".to_owned(),
        }
    }

    fn evidence(
        ordinal: usize,
        request_id: &str,
        provider: &str,
    ) -> NsdbRequestReplayFrameEvidence {
        NsdbRequestReplayFrameEvidence {
            contract: REQUEST_REPLAY_FRAME_CONTRACT,
            frame_id: format!("frame:request:0:{ordinal}:{request_id}"),
            parent_frame_id: "frame:payload:0:payload".to_owned(),
            ordinal,
            source_trace_id: "trace-provider".to_owned(),
            source_provider_family: "spirv:vulkan-gpu".to_owned(),
            collection_root_hash: "0xcollection".to_owned(),
            request_id: request_id.to_owned(),
            provider_family: provider.to_owned(),
            dispatch_id: if provider == "cuda:nvidia-gpu" {
                "dispatch0001"
            } else {
                "dispatch0000"
            }
            .to_owned(),
            completion_clock: format!("clock:{ordinal}"),
            output_hash: format!("0xoutput{ordinal}"),
            completion_token: format!("provider-completion:{ordinal}"),
            selected_set_hash: "fnv1a64:selected".to_owned(),
        }
    }

    #[test]
    fn request_frame_identity_binds_ordinal_and_clock() {
        let first = request_frame_id(2, 1, "trace", "root", &receipt("clock:1"));
        assert_eq!(
            first,
            request_frame_id(2, 1, "trace", "root", &receipt("clock:1"))
        );
        assert_ne!(
            first,
            request_frame_id(2, 2, "trace", "root", &receipt("clock:1"))
        );
        assert_ne!(
            first,
            request_frame_id(2, 1, "trace", "root", &receipt("clock:2"))
        );
    }

    #[test]
    fn expansion_places_ordered_requests_after_parent() {
        let expansion = expand_verified_request_replay_frames(
            vec![frame(0, "frame:payload:0:payload", "checkpoint")],
            vec![
                evidence(0, "request.vulkan", "spirv:vulkan-gpu"),
                evidence(1, "request.cuda", "cuda:nvidia-gpu"),
                evidence(2, "request.vulkan.final", "spirv:vulkan-gpu"),
            ],
        )
        .expect("expand verified request replay frames");

        assert_eq!(expansion.request_frame_count, 3);
        assert_eq!(
            expansion
                .frames
                .iter()
                .map(|frame| frame.index)
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3]
        );
        assert_eq!(
            expansion
                .frames
                .iter()
                .filter_map(|frame| frame.request.as_ref())
                .map(|request| request.request_id.as_str())
                .collect::<Vec<_>>(),
            vec!["request.vulkan", "request.cuda", "request.vulkan.final"]
        );
    }

    #[test]
    fn request_control_stops_on_child_frame() {
        let mut frames = vec![
            frame(0, "frame:payload:0:payload", "checkpoint"),
            frame(1, "frame:request:0:0:first", "provider-request"),
            frame(2, "frame:request:0:1:cuda", "provider-request"),
            frame(3, "frame:request:0:2:last", "provider-request"),
        ];
        let result = apply_request_replay_control(
            &mut frames,
            true,
            &NsdbRequestReplaySelection {
                contract: "nsdb-provider-request-replay-selector-v1",
                status: "request-resolved",
                ready: true,
                request_id: Some("kernel.cuda.copy".to_owned()),
                request_frame_id: Some("frame:request:0:1:cuda".to_owned()),
                source_trace_id: Some("trace-provider".to_owned()),
                source_provider_family: Some("spirv:vulkan-gpu".to_owned()),
                collection_root_hash: Some("0xcollection".to_owned()),
                provider_family: Some("cuda:nvidia-gpu".to_owned()),
                dispatch_id: Some("dispatch0001".to_owned()),
                completion_clock: Some("clock:1".to_owned()),
                output_hash: Some("0xoutput".to_owned()),
                completion_token: Some("provider-completion:cuda".to_owned()),
                selected_set_hash: Some("fnv1a64:selected".to_owned()),
                first_blocker: None,
            },
        );

        assert_eq!(result.selected_frame_index, Some(2));
        assert_eq!(
            result.selected_frame_id.as_deref(),
            Some("frame:request:0:1:cuda")
        );
        assert_eq!(
            frames
                .iter()
                .map(|frame| frame.consumed)
                .collect::<Vec<_>>(),
            vec![true, true, true, false]
        );
    }
}
