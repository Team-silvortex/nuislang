use crate::{
    model::{NsdbInspectReport, NsdbPayloadExecutionEvent},
    payload_decoder::decode_payload_content,
};

pub(crate) struct NsdbReplayPlan {
    pub(crate) protocol: &'static str,
    pub(crate) status: String,
    pub(crate) checkpoint_count: usize,
    pub(crate) replayable_checkpoint_count: usize,
    pub(crate) first_blocker: Option<String>,
    pub(crate) checkpoints: Vec<NsdbReplayCheckpoint>,
}

pub(crate) struct NsdbReplayCheckpoint {
    pub(crate) index: usize,
    pub(crate) trace_id: String,
    pub(crate) checkpoint_kind: String,
    pub(crate) replay_status: String,
    pub(crate) frame_id: String,
    pub(crate) slot_scope: String,
    pub(crate) value_state_status: String,
    pub(crate) value_sample_contract: &'static str,
    pub(crate) value_sample_ref: String,
    pub(crate) value_sample_source: String,
    pub(crate) value_sample_resolution_status: String,
    pub(crate) value_sample_resolution_detail: String,
    pub(crate) value_sample_materialization_status: String,
    pub(crate) value_sample_materialization_detail: String,
    pub(crate) value_sample_payload_format: String,
    pub(crate) value_sample_payload_path: String,
    pub(crate) value_sample_bridge_stub_path: String,
    pub(crate) value_slot_id: String,
    pub(crate) value_slot_scope: String,
    pub(crate) value_schema_contract: &'static str,
    pub(crate) value_schema_status: String,
    pub(crate) value_schema_hint: String,
    pub(crate) value_snapshot_contract: &'static str,
    pub(crate) value_snapshot_status: String,
    pub(crate) value_snapshot_type: String,
    pub(crate) value_snapshot_ref: String,
    pub(crate) value_snapshot_summary: String,
    pub(crate) value_content_status: String,
    pub(crate) value_content_type: String,
    pub(crate) value_content_summary: String,
    pub(crate) value_decoder_id: String,
    pub(crate) value_decoder_status: String,
    pub(crate) value_decoder_detail: String,
    pub(crate) value_decoder_capability: String,
    pub(crate) value_decoder_detail_level: String,
    pub(crate) value_decoder_reads_file_summary: bool,
    pub(crate) value_decoder_manifest_status: String,
    pub(crate) value_decoder_manifest_detail: String,
    pub(crate) value_decoder_format_probe_status: String,
    pub(crate) value_decoder_format_probe_detail: String,
    pub(crate) execution_phase: String,
    pub(crate) entry_symbol: String,
    pub(crate) first_blocker: Option<String>,
    pub(crate) next_action: String,
}

pub(crate) fn build_replay_plan(report: &NsdbInspectReport) -> NsdbReplayPlan {
    let checkpoints = report
        .payload_execution_handoff
        .events
        .iter()
        .map(|event| replay_checkpoint_for_event(report, event))
        .collect::<Vec<_>>();
    let replayable_checkpoint_count = checkpoints
        .iter()
        .filter(|checkpoint| checkpoint.replay_status == "replayable")
        .count();
    let first_blocker = checkpoints
        .iter()
        .find_map(|checkpoint| checkpoint.first_blocker.clone());
    NsdbReplayPlan {
        protocol: "nsdb-payload-execution-replay-plan-v1",
        status: if first_blocker.is_none() {
            "ready".to_owned()
        } else {
            "blocked".to_owned()
        },
        checkpoint_count: checkpoints.len(),
        replayable_checkpoint_count,
        first_blocker,
        checkpoints,
    }
}

fn replay_checkpoint_for_event(
    report: &NsdbInspectReport,
    event: &NsdbPayloadExecutionEvent,
) -> NsdbReplayCheckpoint {
    let first_blocker = if event.first_blocker == "none" && event.status == "ready" {
        None
    } else if event.first_blocker == "none" {
        Some(format!("payload-event-status:{}", event.status))
    } else {
        Some(event.first_blocker.clone())
    };
    let checkpoint_kind = checkpoint_kind_for_phase(&event.execution_phase);
    let sample_resolution = value_sample_resolution_for_event(report, event);
    let slot_scope = slot_scope_for_event(event);
    let value_schema = value_schema_for_sample(&sample_resolution.payload_format);
    let value_snapshot = value_snapshot_for_schema(event, &sample_resolution, &value_schema);
    let value_content = value_content_for_snapshot(report, event, &value_snapshot);
    NsdbReplayCheckpoint {
        index: event.index,
        trace_id: event.trace_id.clone(),
        checkpoint_kind: checkpoint_kind.to_owned(),
        replay_status: if first_blocker.is_none() {
            "replayable".to_owned()
        } else {
            "blocked".to_owned()
        },
        frame_id: frame_id_for_event(event),
        slot_scope: slot_scope.clone(),
        value_state_status: value_state_status_for_event(&event.execution_phase, &first_blocker)
            .to_owned(),
        value_sample_contract: "nsdb-yir-value-sample-ref-v1",
        value_sample_ref: value_sample_ref_for_event(event),
        value_sample_source: value_sample_source_for_phase(&event.execution_phase).to_owned(),
        value_sample_resolution_status: sample_resolution.status,
        value_sample_resolution_detail: sample_resolution.detail,
        value_sample_materialization_status: sample_resolution.materialization_status,
        value_sample_materialization_detail: sample_resolution.materialization_detail,
        value_sample_payload_format: sample_resolution.payload_format,
        value_sample_payload_path: sample_resolution.payload_path,
        value_sample_bridge_stub_path: sample_resolution.bridge_stub_path,
        value_slot_id: value_slot_id_for_event(event),
        value_slot_scope: slot_scope,
        value_schema_contract: "nsdb-yir-value-schema-ref-v1",
        value_schema_status: value_schema.status,
        value_schema_hint: value_schema.hint,
        value_snapshot_contract: "nsdb-yir-value-snapshot-v1",
        value_snapshot_status: value_snapshot.status,
        value_snapshot_type: value_snapshot.value_type,
        value_snapshot_ref: value_snapshot.value_ref,
        value_snapshot_summary: value_snapshot.summary,
        value_content_status: value_content.status,
        value_content_type: value_content.value_type,
        value_content_summary: value_content.summary,
        value_decoder_id: value_content.decoder_id,
        value_decoder_status: value_content.decoder_status,
        value_decoder_detail: value_content.decoder_detail,
        value_decoder_capability: value_content.decoder_capability,
        value_decoder_detail_level: value_content.decoder_detail_level,
        value_decoder_reads_file_summary: value_content.decoder_reads_file_summary,
        value_decoder_manifest_status: value_content.decoder_manifest_status,
        value_decoder_manifest_detail: value_content.decoder_manifest_detail,
        value_decoder_format_probe_status: value_content.decoder_format_probe_status,
        value_decoder_format_probe_detail: value_content.decoder_format_probe_detail,
        execution_phase: event.execution_phase.clone(),
        entry_symbol: event.entry_symbol.clone(),
        first_blocker,
        next_action: event.next_action.clone(),
    }
}

fn checkpoint_kind_for_phase(phase: &str) -> &'static str {
    match phase {
        "container-loader-handoff" => "loader-checkpoint",
        "device-dispatch" => "device-dispatch-checkpoint",
        _ => "payload-execution-checkpoint",
    }
}

struct ValueSampleResolution {
    status: String,
    detail: String,
    materialization_status: String,
    materialization_detail: String,
    payload_format: String,
    payload_path: String,
    bridge_stub_path: String,
}

struct ValueSchemaRef {
    status: String,
    hint: String,
}

struct ValueSnapshotRef {
    status: String,
    value_type: String,
    value_ref: String,
    summary: String,
}

struct ValueContentRef {
    decoder_id: String,
    decoder_status: String,
    decoder_detail: String,
    decoder_capability: String,
    decoder_detail_level: String,
    decoder_reads_file_summary: bool,
    decoder_manifest_status: String,
    decoder_manifest_detail: String,
    decoder_format_probe_status: String,
    decoder_format_probe_detail: String,
    status: String,
    value_type: String,
    summary: String,
}

fn value_sample_resolution_for_event(
    report: &NsdbInspectReport,
    event: &NsdbPayloadExecutionEvent,
) -> ValueSampleResolution {
    match event.execution_phase.as_str() {
        "device-dispatch" => device_sample_resolution(report, event),
        "container-loader-handoff" => payload_handoff_sample_resolution(report, event),
        _ => generic_sample_resolution(event),
    }
}

fn device_sample_resolution(
    report: &NsdbInspectReport,
    event: &NsdbPayloadExecutionEvent,
) -> ValueSampleResolution {
    if let Some(record) = report.hetero_runtime_trace.records.iter().find(|record| {
        record.domain_family == event.target
            || record.backend_artifact_key.contains(&event.target)
            || record.trace_id.contains(&event.target)
    }) {
        return ValueSampleResolution {
            status: if record.status == "trace-ready" {
                "trace-record-resolved".to_owned()
            } else {
                "trace-record-observed".to_owned()
            },
            detail: format!("hetero-runtime-trace:{}:{}", record.trace_id, record.status),
            materialization_status: materialization_status_for_record(record),
            materialization_detail: format!(
                "payload:{}:{}",
                record.payload_format, record.payload_path
            ),
            payload_format: record.payload_format.clone(),
            payload_path: record.payload_path.clone(),
            bridge_stub_path: record.bridge_stub_path.clone(),
        };
    }
    if let Some(sidecar) = report.sidecars.iter().find(|sidecar| {
        sidecar.domain_family == event.target
            || sidecar.capability_owner == event.target
            || sidecar.entry_symbol == event.entry_symbol
    }) {
        return ValueSampleResolution {
            status: "trace-record-resolvable".to_owned(),
            detail: format!("sidecar:{}:{}", sidecar.domain_family, sidecar.entry_symbol),
            materialization_status: "sample-awaiting-trace-record".to_owned(),
            materialization_detail: "hetero-runtime-trace-record-missing".to_owned(),
            payload_format: "none".to_owned(),
            payload_path: "none".to_owned(),
            bridge_stub_path: "none".to_owned(),
        };
    }
    if report
        .domains
        .iter()
        .any(|domain| domain.domain_family == event.target)
    {
        return ValueSampleResolution {
            status: "trace-record-pending".to_owned(),
            detail: format!("domain:{}", event.target),
            materialization_status: "sample-awaiting-trace-record".to_owned(),
            materialization_detail: "domain-visible-without-runtime-trace-record".to_owned(),
            payload_format: "none".to_owned(),
            payload_path: "none".to_owned(),
            bridge_stub_path: "none".to_owned(),
        };
    }
    ValueSampleResolution {
        status: "trace-record-missing".to_owned(),
        detail: format!("target:{}", event.target),
        materialization_status: "sample-missing".to_owned(),
        materialization_detail: "no-runtime-trace-source".to_owned(),
        payload_format: "none".to_owned(),
        payload_path: "none".to_owned(),
        bridge_stub_path: "none".to_owned(),
    }
}

fn materialization_status_for_record(
    record: &crate::model::NsdbHeteroRuntimeTraceRecord,
) -> String {
    if record.payload_format != "none" && record.payload_path != "none" {
        "sample-descriptor-materialized".to_owned()
    } else {
        "sample-descriptor-incomplete".to_owned()
    }
}

fn payload_handoff_sample_resolution(
    report: &NsdbInspectReport,
    event: &NsdbPayloadExecutionEvent,
) -> ValueSampleResolution {
    if report.payload_execution_handoff.available {
        ValueSampleResolution {
            status: "metadata-resolved".to_owned(),
            detail: format!("payload-execution-handoff:{}", event.trace_id),
            materialization_status: "metadata-sample-materialized".to_owned(),
            materialization_detail: format!("payload-handoff:{}", event.entry_symbol),
            payload_format: "payload-execution-metadata".to_owned(),
            payload_path: report.payload_execution_handoff.path.clone(),
            bridge_stub_path: "none".to_owned(),
        }
    } else {
        ValueSampleResolution {
            status: "metadata-missing".to_owned(),
            detail: "payload-execution-handoff".to_owned(),
            materialization_status: "sample-missing".to_owned(),
            materialization_detail: "payload-execution-handoff-missing".to_owned(),
            payload_format: "none".to_owned(),
            payload_path: "none".to_owned(),
            bridge_stub_path: "none".to_owned(),
        }
    }
}

fn generic_sample_resolution(event: &NsdbPayloadExecutionEvent) -> ValueSampleResolution {
    ValueSampleResolution {
        status: "metadata-pending".to_owned(),
        detail: format!("payload-execution-event:{}", event.execution_phase),
        materialization_status: "metadata-sample-pending".to_owned(),
        materialization_detail: format!("payload-execution-event:{}", event.entry_symbol),
        payload_format: "payload-execution-metadata".to_owned(),
        payload_path: "none".to_owned(),
        bridge_stub_path: "none".to_owned(),
    }
}

fn frame_id_for_event(event: &NsdbPayloadExecutionEvent) -> String {
    format!(
        "frame:payload:{}:{}",
        event.index,
        frame_kind_for_phase(&event.execution_phase)
    )
}

fn value_slot_id_for_event(event: &NsdbPayloadExecutionEvent) -> String {
    format!(
        "slot:payload:{}:{}",
        event.index,
        frame_kind_for_phase(&event.execution_phase)
    )
}

fn value_schema_for_sample(payload_format: &str) -> ValueSchemaRef {
    match payload_format {
        "none" => ValueSchemaRef {
            status: "schema-missing".to_owned(),
            hint: "no-sample-payload".to_owned(),
        },
        "payload-execution-metadata" => ValueSchemaRef {
            status: "schema-metadata-only".to_owned(),
            hint: "payload-execution-event-metadata".to_owned(),
        },
        format => ValueSchemaRef {
            status: "schema-opaque-payload".to_owned(),
            hint: format!("opaque-runtime-payload:{format}"),
        },
    }
}

fn value_snapshot_for_schema(
    event: &NsdbPayloadExecutionEvent,
    sample: &ValueSampleResolution,
    schema: &ValueSchemaRef,
) -> ValueSnapshotRef {
    match schema.status.as_str() {
        "schema-metadata-only" => ValueSnapshotRef {
            status: "snapshot-metadata-only".to_owned(),
            value_type: "payload-execution-metadata".to_owned(),
            value_ref: format!("snapshot:{}:metadata", event.trace_id),
            summary: sample.materialization_detail.clone(),
        },
        "schema-opaque-payload" => ValueSnapshotRef {
            status: "snapshot-opaque-payload".to_owned(),
            value_type: sample.payload_format.clone(),
            value_ref: sample.payload_path.clone(),
            summary: format!(
                "opaque-payload:{}:{}",
                sample.payload_format, sample.payload_path
            ),
        },
        _ => ValueSnapshotRef {
            status: "snapshot-missing".to_owned(),
            value_type: "none".to_owned(),
            value_ref: "none".to_owned(),
            summary: "no-decodable-sample".to_owned(),
        },
    }
}

fn value_content_for_snapshot(
    report: &NsdbInspectReport,
    event: &NsdbPayloadExecutionEvent,
    snapshot: &ValueSnapshotRef,
) -> ValueContentRef {
    match snapshot.status.as_str() {
        "snapshot-metadata-only" => ValueContentRef {
            decoder_id: "nsdb-metadata-summary-decoder-v1".to_owned(),
            decoder_status: "decoder-ready".to_owned(),
            decoder_detail: "metadata-summary".to_owned(),
            decoder_capability: "metadata-summary".to_owned(),
            decoder_detail_level: "semantic-metadata".to_owned(),
            decoder_reads_file_summary: false,
            decoder_manifest_status: "manifest-not-needed".to_owned(),
            decoder_manifest_detail: "metadata-summary".to_owned(),
            decoder_format_probe_status: "format-probe-not-needed".to_owned(),
            decoder_format_probe_detail: "metadata-summary".to_owned(),
            status: "content-metadata-summary".to_owned(),
            value_type: snapshot.value_type.clone(),
            summary: format!(
                "entry={} phase={} trace={}",
                event.entry_symbol, event.execution_phase, event.trace_id
            ),
        },
        "snapshot-opaque-payload" => opaque_payload_content(report, snapshot),
        _ => ValueContentRef {
            decoder_id: "nsdb-noop-decoder-v1".to_owned(),
            decoder_status: "decoder-missing".to_owned(),
            decoder_detail: "no snapshot content".to_owned(),
            decoder_capability: "none".to_owned(),
            decoder_detail_level: "none".to_owned(),
            decoder_reads_file_summary: false,
            decoder_manifest_status: "manifest-not-needed".to_owned(),
            decoder_manifest_detail: "no snapshot content".to_owned(),
            decoder_format_probe_status: "format-probe-not-needed".to_owned(),
            decoder_format_probe_detail: "no snapshot content".to_owned(),
            status: "content-missing".to_owned(),
            value_type: "none".to_owned(),
            summary: "no snapshot content".to_owned(),
        },
    }
}

fn opaque_payload_content(
    report: &NsdbInspectReport,
    snapshot: &ValueSnapshotRef,
) -> ValueContentRef {
    let decoded = decode_payload_content(
        &report.output_dir,
        &snapshot.value_type,
        &snapshot.value_ref,
    );
    ValueContentRef {
        decoder_id: decoded.decoder_id,
        decoder_status: decoded.decoder_status,
        decoder_detail: decoded.decoder_detail,
        decoder_capability: decoded.decoder_capability,
        decoder_detail_level: decoded.decoder_detail_level,
        decoder_reads_file_summary: decoded.decoder_reads_file_summary,
        decoder_manifest_status: decoded.decoder_manifest_status,
        decoder_manifest_detail: decoded.decoder_manifest_detail,
        decoder_format_probe_status: decoded.decoder_format_probe_status,
        decoder_format_probe_detail: decoded.decoder_format_probe_detail,
        status: decoded.content_status,
        value_type: decoded.content_type,
        summary: decoded.content_summary,
    }
}

fn frame_kind_for_phase(phase: &str) -> &'static str {
    match phase {
        "container-loader-handoff" => "loader",
        "device-dispatch" => "device",
        _ => "payload",
    }
}

fn slot_scope_for_event(event: &NsdbPayloadExecutionEvent) -> String {
    if event.target == "none" {
        format!("payload:{}", event.entry_symbol)
    } else {
        format!("{}:{}", event.target, event.entry_symbol)
    }
}

fn value_state_status_for_event(phase: &str, first_blocker: &Option<String>) -> &'static str {
    if first_blocker.is_some() {
        "blocked"
    } else if phase == "device-dispatch" {
        "awaiting-device-sample"
    } else {
        "metadata-only"
    }
}

fn value_sample_ref_for_event(event: &NsdbPayloadExecutionEvent) -> String {
    format!(
        "value-sample:{}:{}",
        event.trace_id,
        slot_scope_for_event(event)
    )
}

fn value_sample_source_for_phase(phase: &str) -> &'static str {
    match phase {
        "device-dispatch" => "hetero-runtime-trace",
        "container-loader-handoff" => "payload-execution-handoff",
        _ => "payload-execution-event",
    }
}
