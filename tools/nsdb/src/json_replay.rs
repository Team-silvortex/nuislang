use crate::json::{
    json_bool_field, json_optional_string_field, json_string_field, json_usize_field,
};
use crate::model::NsdbInspectReport;
use crate::replay::{build_replay_plan, NsdbReplayCheckpoint};

pub(crate) fn nsdb_replay_plan_json(report: &NsdbInspectReport) -> String {
    let plan = build_replay_plan(report);
    let fields = vec![
        json_string_field("tool", "nsdb"),
        json_string_field("kind", "nsdb_payload_execution_replay_plan"),
        json_string_field("manifest", &report.manifest),
        json_string_field("replay_protocol", plan.protocol),
        json_string_field(
            "replay_event_query_contract",
            "nsdb-payload-execution-event-query-v1",
        ),
        json_string_field(
            "replay_checkpoint_source",
            "payload-execution-handoff-events",
        ),
        json_string_field(
            "replay_event_source_protocol",
            &report.payload_execution_handoff.protocol,
        ),
        json_string_field(
            "replay_event_source_debugger_contract",
            &report.payload_execution_handoff.debugger_contract,
        ),
        json_string_field(
            "replay_hetero_execution_closure_protocol",
            &report
                .payload_execution_handoff
                .hetero_execution_closure_protocol,
        ),
        json_string_field(
            "replay_hetero_execution_closure_status",
            &report
                .payload_execution_handoff
                .hetero_execution_closure_status,
        ),
        json_string_field(
            "replay_hetero_execution_closure_ready",
            &report
                .payload_execution_handoff
                .hetero_execution_closure_ready,
        ),
        json_string_field(
            "replay_hetero_execution_closure_next_action",
            &report
                .payload_execution_handoff
                .hetero_execution_closure_next_action,
        ),
        json_string_field("replay_status", &plan.status),
        json_usize_field("replay_checkpoint_count", plan.checkpoint_count),
        json_usize_field("replay_event_query_result_count", plan.checkpoint_count),
        json_usize_field(
            "replayable_checkpoint_count",
            plan.replayable_checkpoint_count,
        ),
        json_optional_string_field("replay_first_blocker", plan.first_blocker.as_deref()),
        format!(
            "\"replay_checkpoints\":[{}]",
            replay_checkpoints_json(&plan.checkpoints)
        ),
    ];
    format!("{{{}}}", fields.join(","))
}

fn replay_checkpoints_json(checkpoints: &[NsdbReplayCheckpoint]) -> String {
    checkpoints
        .iter()
        .map(|checkpoint| {
            let fields = vec![
                json_usize_field("index", checkpoint.index),
                json_string_field("trace_id", &checkpoint.trace_id),
                json_string_field("checkpoint_kind", &checkpoint.checkpoint_kind),
                json_string_field("replay_status", &checkpoint.replay_status),
                json_string_field("frame_id", &checkpoint.frame_id),
                json_string_field("slot_scope", &checkpoint.slot_scope),
                json_string_field("value_state_status", &checkpoint.value_state_status),
                json_string_field("value_sample_contract", checkpoint.value_sample_contract),
                json_string_field("value_sample_ref", &checkpoint.value_sample_ref),
                json_string_field("value_sample_source", &checkpoint.value_sample_source),
                json_string_field(
                    "value_sample_resolution_status",
                    &checkpoint.value_sample_resolution_status,
                ),
                json_string_field(
                    "value_sample_resolution_detail",
                    &checkpoint.value_sample_resolution_detail,
                ),
                json_string_field(
                    "value_sample_materialization_status",
                    &checkpoint.value_sample_materialization_status,
                ),
                json_string_field(
                    "value_sample_materialization_detail",
                    &checkpoint.value_sample_materialization_detail,
                ),
                json_string_field(
                    "value_sample_payload_format",
                    &checkpoint.value_sample_payload_format,
                ),
                json_string_field(
                    "value_sample_payload_path",
                    &checkpoint.value_sample_payload_path,
                ),
                json_string_field(
                    "value_sample_bridge_stub_path",
                    &checkpoint.value_sample_bridge_stub_path,
                ),
                json_string_field("value_slot_id", &checkpoint.value_slot_id),
                json_string_field("value_slot_scope", &checkpoint.value_slot_scope),
                json_string_field("value_schema_contract", checkpoint.value_schema_contract),
                json_string_field("value_schema_status", &checkpoint.value_schema_status),
                json_string_field("value_schema_hint", &checkpoint.value_schema_hint),
                json_string_field(
                    "value_snapshot_contract",
                    checkpoint.value_snapshot_contract,
                ),
                json_string_field("value_snapshot_status", &checkpoint.value_snapshot_status),
                json_string_field("value_snapshot_type", &checkpoint.value_snapshot_type),
                json_string_field("value_snapshot_ref", &checkpoint.value_snapshot_ref),
                json_string_field("value_snapshot_summary", &checkpoint.value_snapshot_summary),
                json_string_field("value_content_status", &checkpoint.value_content_status),
                json_string_field("value_content_type", &checkpoint.value_content_type),
                json_string_field("value_content_summary", &checkpoint.value_content_summary),
                json_string_field("value_decoder_id", &checkpoint.value_decoder_id),
                json_string_field("value_decoder_status", &checkpoint.value_decoder_status),
                json_string_field("value_decoder_detail", &checkpoint.value_decoder_detail),
                json_string_field(
                    "value_decoder_capability",
                    &checkpoint.value_decoder_capability,
                ),
                json_string_field(
                    "value_decoder_detail_level",
                    &checkpoint.value_decoder_detail_level,
                ),
                json_bool_field(
                    "value_decoder_reads_file_summary",
                    checkpoint.value_decoder_reads_file_summary,
                ),
                json_string_field(
                    "value_decoder_manifest_status",
                    &checkpoint.value_decoder_manifest_status,
                ),
                json_string_field(
                    "value_decoder_manifest_detail",
                    &checkpoint.value_decoder_manifest_detail,
                ),
                json_string_field(
                    "value_decoder_format_probe_status",
                    &checkpoint.value_decoder_format_probe_status,
                ),
                json_string_field(
                    "value_decoder_format_probe_detail",
                    &checkpoint.value_decoder_format_probe_detail,
                ),
                json_string_field("execution_phase", &checkpoint.execution_phase),
                json_string_field("entry_symbol", &checkpoint.entry_symbol),
                json_optional_string_field("first_blocker", checkpoint.first_blocker.as_deref()),
                json_string_field("next_action", &checkpoint.next_action),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}
