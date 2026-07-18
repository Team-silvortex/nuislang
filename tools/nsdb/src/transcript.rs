use crate::{
    model::NsdbInspectReport,
    replay::{build_replay_plan, NsdbReplayCheckpoint},
};

pub(crate) struct NsdbReplayTranscript {
    pub(crate) protocol: &'static str,
    pub(crate) source_contract: &'static str,
    pub(crate) status: &'static str,
    pub(crate) ready: bool,
    pub(crate) checkpoint_count: usize,
    pub(crate) replayed_checkpoint_count: usize,
    pub(crate) first_blocker: Option<String>,
    pub(crate) frames: Vec<NsdbReplayTranscriptFrame>,
}

pub(crate) struct NsdbReplayTranscriptFrame {
    pub(crate) index: usize,
    pub(crate) trace_id: String,
    pub(crate) frame_id: String,
    pub(crate) checkpoint_kind: String,
    pub(crate) execution_phase: String,
    pub(crate) entry_symbol: String,
    pub(crate) replay_status: String,
    pub(crate) consumed: bool,
    pub(crate) value_slot_id: String,
    pub(crate) value_snapshot_status: String,
    pub(crate) value_snapshot_type: String,
    pub(crate) value_snapshot_summary: String,
    pub(crate) value_content_status: String,
    pub(crate) value_content_summary: String,
    pub(crate) next_action: String,
}

pub(crate) fn build_replay_transcript(report: &NsdbInspectReport) -> NsdbReplayTranscript {
    let plan = build_replay_plan(report);
    let ready = plan.status == "ready"
        && plan.checkpoint_count > 0
        && plan.replayable_checkpoint_count == plan.checkpoint_count;
    let frames = plan
        .checkpoints
        .iter()
        .map(|checkpoint| transcript_frame(checkpoint, ready))
        .collect::<Vec<_>>();
    NsdbReplayTranscript {
        protocol: "nsdb-yir-replay-transcript-v1",
        source_contract: plan.protocol,
        status: if ready {
            "transcript-consumed"
        } else {
            "transcript-blocked"
        },
        ready,
        checkpoint_count: plan.checkpoint_count,
        replayed_checkpoint_count: if ready {
            plan.replayable_checkpoint_count
        } else {
            0
        },
        first_blocker: if ready {
            None
        } else {
            plan.first_blocker
                .or_else(|| Some("payload-execution-replay:no-checkpoints".to_owned()))
        },
        frames,
    }
}

fn transcript_frame(
    checkpoint: &NsdbReplayCheckpoint,
    transcript_ready: bool,
) -> NsdbReplayTranscriptFrame {
    NsdbReplayTranscriptFrame {
        index: checkpoint.index,
        trace_id: checkpoint.trace_id.clone(),
        frame_id: checkpoint.frame_id.clone(),
        checkpoint_kind: checkpoint.checkpoint_kind.clone(),
        execution_phase: checkpoint.execution_phase.clone(),
        entry_symbol: checkpoint.entry_symbol.clone(),
        replay_status: checkpoint.replay_status.clone(),
        consumed: transcript_ready && checkpoint.replay_status == "replayable",
        value_slot_id: checkpoint.value_slot_id.clone(),
        value_snapshot_status: checkpoint.value_snapshot_status.clone(),
        value_snapshot_type: checkpoint.value_snapshot_type.clone(),
        value_snapshot_summary: checkpoint.value_snapshot_summary.clone(),
        value_content_status: checkpoint.value_content_status.clone(),
        value_content_summary: checkpoint.value_content_summary.clone(),
        next_action: checkpoint.next_action.clone(),
    }
}
