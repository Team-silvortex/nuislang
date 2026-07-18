use crate::{
    json::{json_bool_field, json_optional_string_field, json_string_field, json_usize_field},
    model::NsdbInspectReport,
    transcript::{build_replay_transcript, NsdbReplayTranscriptFrame},
};

pub(crate) fn nsdb_replay_transcript_json(report: &NsdbInspectReport) -> String {
    let transcript = build_replay_transcript(report);
    let fields = vec![
        json_string_field("tool", "nsdb"),
        json_string_field("kind", "nsdb_yir_replay_transcript"),
        json_string_field("manifest", &report.manifest),
        json_string_field("debugger_transcript_contract", transcript.protocol),
        json_string_field(
            "debugger_transcript_source_contract",
            transcript.source_contract,
        ),
        json_string_field("debugger_transcript_status", transcript.status),
        json_bool_field("debugger_transcript_ready", transcript.ready),
        json_usize_field(
            "debugger_transcript_checkpoint_count",
            transcript.checkpoint_count,
        ),
        json_usize_field(
            "debugger_transcript_replayed_checkpoint_count",
            transcript.replayed_checkpoint_count,
        ),
        json_optional_string_field(
            "debugger_transcript_first_blocker",
            transcript.first_blocker.as_deref(),
        ),
        format!(
            "\"debugger_transcript_frames\":[{}]",
            transcript_frames_json(&transcript.frames)
        ),
    ];
    format!("{{{}}}", fields.join(","))
}

fn transcript_frames_json(frames: &[NsdbReplayTranscriptFrame]) -> String {
    frames
        .iter()
        .map(|frame| {
            let fields = vec![
                json_usize_field("index", frame.index),
                json_string_field("trace_id", &frame.trace_id),
                json_string_field("frame_id", &frame.frame_id),
                json_string_field("checkpoint_kind", &frame.checkpoint_kind),
                json_string_field("execution_phase", &frame.execution_phase),
                json_string_field("entry_symbol", &frame.entry_symbol),
                json_string_field("replay_status", &frame.replay_status),
                json_bool_field("consumed", frame.consumed),
                json_string_field("value_slot_id", &frame.value_slot_id),
                json_string_field("value_snapshot_status", &frame.value_snapshot_status),
                json_string_field("value_snapshot_type", &frame.value_snapshot_type),
                json_string_field("value_snapshot_summary", &frame.value_snapshot_summary),
                json_string_field("value_content_status", &frame.value_content_status),
                json_string_field("value_content_summary", &frame.value_content_summary),
                json_string_field("next_action", &frame.next_action),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}
