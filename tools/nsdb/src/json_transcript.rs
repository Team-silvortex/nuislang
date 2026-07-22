use crate::{
    json::{json_bool_field, json_optional_string_field, json_string_field, json_usize_field},
    model::NsdbInspectReport,
    transcript::{
        build_replay_transcript, build_replay_transcript_with_control, NsdbReplayControl,
        NsdbReplayTranscriptFrame,
    },
};

pub(crate) fn nsdb_replay_transcript_json(report: &NsdbInspectReport) -> String {
    let transcript = build_replay_transcript(report);
    nsdb_replay_transcript_json_from_transcript(report, transcript)
}

pub(crate) fn nsdb_replay_transcript_json_with_control(
    report: &NsdbInspectReport,
    control: &NsdbReplayControl,
) -> String {
    let transcript = build_replay_transcript_with_control(report, control);
    nsdb_replay_transcript_json_from_transcript(report, transcript)
}

fn nsdb_replay_transcript_json_from_transcript(
    report: &NsdbInspectReport,
    transcript: crate::transcript::NsdbReplayTranscript,
) -> String {
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
        json_string_field(
            "debugger_transcript_control_contract",
            transcript.control_protocol,
        ),
        json_string_field("debugger_transcript_control_mode", transcript.control_mode),
        json_optional_string_field(
            "debugger_transcript_control_selector",
            transcript.control_selector.as_deref(),
        ),
        json_string_field(
            "debugger_transcript_control_status",
            transcript.control_status,
        ),
        json_string_field(
            "debugger_transcript_breakpoint_predicate_contract",
            transcript.breakpoint_predicate_protocol,
        ),
        json_optional_string_field(
            "debugger_transcript_breakpoint_phase",
            transcript.breakpoint_phase.as_deref(),
        ),
        json_optional_string_field(
            "debugger_transcript_breakpoint_entry",
            transcript.breakpoint_entry.as_deref(),
        ),
        json_string_field(
            "debugger_transcript_resume_input_contract",
            transcript.resume_input_protocol,
        ),
        json_string_field(
            "debugger_transcript_resume_input_status",
            transcript.resume_input_status,
        ),
        json_optional_string_field(
            "debugger_transcript_resume_input_after_frame_id",
            transcript.resume_input_after_frame_id.as_deref(),
        ),
        json_optional_string_field(
            "debugger_transcript_resume_input_next_frame_id",
            transcript.resume_input_next_frame_id.as_deref(),
        ),
        transcript.selected_frame_index.map_or_else(
            || "\"debugger_transcript_selected_frame_index\":null".to_owned(),
            |index| json_usize_field("debugger_transcript_selected_frame_index", index),
        ),
        json_optional_string_field(
            "debugger_transcript_selected_frame_id",
            transcript.selected_frame_id.as_deref(),
        ),
        json_string_field("debugger_transcript_stop_reason", transcript.stop_reason),
        json_string_field(
            "debugger_transcript_resume_cursor_contract",
            transcript.resume_cursor_protocol,
        ),
        json_string_field(
            "debugger_transcript_resume_cursor_status",
            transcript.resume_cursor_status,
        ),
        json_bool_field(
            "debugger_transcript_resume_cursor_ready",
            transcript.resume_cursor_ready,
        ),
        json_optional_string_field(
            "debugger_transcript_resume_after_frame_id",
            transcript.resume_after_frame_id.as_deref(),
        ),
        transcript.resume_next_frame_index.map_or_else(
            || "\"debugger_transcript_resume_next_frame_index\":null".to_owned(),
            |index| json_usize_field("debugger_transcript_resume_next_frame_index", index),
        ),
        json_optional_string_field(
            "debugger_transcript_resume_next_frame_id",
            transcript.resume_next_frame_id.as_deref(),
        ),
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
