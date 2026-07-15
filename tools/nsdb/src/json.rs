use crate::model::{
    NsdbClockEdgeDebugInfo, NsdbDataSegmentDebugInfo, NsdbDomainDebugInfo,
    NsdbHeteroRuntimeTraceRecord, NsdbInspectReport, NsdbLoweringUnitDebugInfo,
    NsdbPayloadExecutionEvent, NsdbSidecarDebugInfo,
};
use crate::replay::{build_replay_plan, NsdbReplayCheckpoint};

pub(crate) fn nsdb_inspect_report_json(report: &NsdbInspectReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsdb"),
        json_string_field("kind", "nsdb_yir_debug_inspect"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_dir", &report.output_dir),
        json_string_field("debug_model", &report.debug_model),
        json_string_field(
            "native_debugger_visibility",
            &report.native_debugger_visibility,
        ),
        json_string_field("nsdb_visibility", &report.nsdb_visibility),
        json_string_field("debug_readiness", &report.debug_readiness),
        json_bool_field("yir_debuggable", report.yir_debuggable),
        json_usize_field("domain_count", report.domain_count),
        json_usize_field("hetero_domain_count", report.hetero_domain_count),
        json_usize_field("clock_edge_count", report.clock_edge_count),
        json_usize_field("data_segment_count", report.data_segment_count),
        json_usize_field("lowering_unit_count", report.lowering_unit_count),
        json_usize_field("sidecar_count", report.sidecar_count),
        json_bool_field(
            "payload_execution_event_filter_active",
            report.payload_execution_event_filter.active(),
        ),
        json_optional_string_field(
            "payload_execution_event_filter_status",
            report.payload_execution_event_filter.status.as_deref(),
        ),
        json_optional_string_field(
            "payload_execution_event_filter_phase",
            report.payload_execution_event_filter.phase.as_deref(),
        ),
        json_optional_string_field(
            "payload_execution_event_filter_trace_id",
            report.payload_execution_event_filter.trace_id.as_deref(),
        ),
        json_bool_field(
            "payload_execution_handoff_available",
            report.payload_execution_handoff.available,
        ),
        json_string_field(
            "payload_execution_handoff_path",
            &report.payload_execution_handoff.path,
        ),
        json_string_field(
            "payload_execution_handoff_protocol",
            &report.payload_execution_handoff.protocol,
        ),
        json_string_field(
            "payload_execution_handoff_debugger_contract",
            &report.payload_execution_handoff.debugger_contract,
        ),
        json_string_field(
            "payload_execution_handoff_status",
            &report.payload_execution_handoff.status,
        ),
        json_usize_field(
            "payload_execution_handoff_record_count",
            report.payload_execution_handoff.record_count,
        ),
        json_usize_field(
            "payload_execution_handoff_ready_record_count",
            report.payload_execution_handoff.ready_record_count,
        ),
        json_string_field(
            "payload_execution_handoff_first_trace_id",
            &report.payload_execution_handoff.first_trace_id,
        ),
        json_string_field(
            "payload_execution_handoff_first_status",
            &report.payload_execution_handoff.first_status,
        ),
        json_string_field(
            "payload_execution_handoff_first_next_action",
            &report.payload_execution_handoff.first_next_action,
        ),
        json_string_field(
            "payload_execution_handoff_first_entry_symbol",
            &report.payload_execution_handoff.first_entry_symbol,
        ),
        json_string_field(
            "payload_execution_handoff_first_execution_phase",
            &report.payload_execution_handoff.first_execution_phase,
        ),
        json_usize_field(
            "payload_execution_event_count",
            report.payload_execution_handoff.events.len(),
        ),
        format!(
            "\"payload_execution_events\":[{}]",
            payload_execution_events_json(&report.payload_execution_handoff.events)
        ),
        json_bool_field(
            "hetero_runtime_trace_available",
            report.hetero_runtime_trace.available,
        ),
        json_string_field(
            "hetero_runtime_trace_path",
            &report.hetero_runtime_trace.path,
        ),
        json_string_field(
            "hetero_runtime_trace_protocol",
            &report.hetero_runtime_trace.protocol,
        ),
        json_string_field(
            "hetero_runtime_trace_debugger_contract",
            &report.hetero_runtime_trace.debugger_contract,
        ),
        json_string_field(
            "hetero_runtime_trace_status",
            &report.hetero_runtime_trace.status,
        ),
        json_usize_field(
            "hetero_runtime_trace_record_count",
            report.hetero_runtime_trace.record_count,
        ),
        json_usize_field(
            "hetero_runtime_trace_ready_record_count",
            report.hetero_runtime_trace.ready_record_count,
        ),
        json_usize_field(
            "hetero_runtime_trace_backend_execution_record_count",
            report.hetero_runtime_trace.backend_execution_record_count,
        ),
        json_string_field(
            "hetero_runtime_trace_first_trace_id",
            &report.hetero_runtime_trace.first_trace_id,
        ),
        json_string_field(
            "hetero_runtime_trace_first_blocker",
            &report.hetero_runtime_trace.first_blocker,
        ),
        json_string_field(
            "hetero_runtime_trace_next_action",
            &report.hetero_runtime_trace.next_action,
        ),
        format!(
            "\"hetero_runtime_trace_records\":[{}]",
            hetero_runtime_trace_records_json(&report.hetero_runtime_trace.records)
        ),
        format!("\"domains\":[{}]", domains_json(&report.domains)),
        format!(
            "\"clock_edges\":[{}]",
            clock_edges_json(&report.clock_edges)
        ),
        format!(
            "\"data_segments\":[{}]",
            data_segments_json(&report.data_segments)
        ),
        format!(
            "\"lowering_units\":[{}]",
            lowering_units_json(&report.lowering_units)
        ),
        format!("\"sidecars\":[{}]", sidecars_json(&report.sidecars)),
        json_string_array_field("missing_metadata", &report.missing_metadata),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsdb_events_report_json(report: &NsdbInspectReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsdb"),
        json_string_field("kind", "nsdb_payload_execution_events"),
        json_string_field("manifest", &report.manifest),
        json_bool_field(
            "payload_execution_event_filter_active",
            report.payload_execution_event_filter.active(),
        ),
        json_optional_string_field(
            "payload_execution_event_filter_status",
            report.payload_execution_event_filter.status.as_deref(),
        ),
        json_optional_string_field(
            "payload_execution_event_filter_phase",
            report.payload_execution_event_filter.phase.as_deref(),
        ),
        json_optional_string_field(
            "payload_execution_event_filter_trace_id",
            report.payload_execution_event_filter.trace_id.as_deref(),
        ),
        json_bool_field(
            "payload_execution_handoff_available",
            report.payload_execution_handoff.available,
        ),
        json_string_field(
            "payload_execution_handoff_status",
            &report.payload_execution_handoff.status,
        ),
        json_usize_field(
            "payload_execution_event_count",
            report.payload_execution_handoff.events.len(),
        ),
        format!(
            "\"payload_execution_events\":[{}]",
            payload_execution_events_json(&report.payload_execution_handoff.events)
        ),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsdb_replay_plan_json(report: &NsdbInspectReport) -> String {
    let plan = build_replay_plan(report);
    let fields = vec![
        json_string_field("tool", "nsdb"),
        json_string_field("kind", "nsdb_payload_execution_replay_plan"),
        json_string_field("manifest", &report.manifest),
        json_string_field("replay_protocol", plan.protocol),
        json_string_field("replay_status", &plan.status),
        json_usize_field("replay_checkpoint_count", plan.checkpoint_count),
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
                    checkpoint.value_decoder_capability,
                ),
                json_string_field(
                    "value_decoder_detail_level",
                    checkpoint.value_decoder_detail_level,
                ),
                json_bool_field(
                    "value_decoder_reads_file_summary",
                    checkpoint.value_decoder_reads_file_summary,
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

fn payload_execution_events_json(events: &[NsdbPayloadExecutionEvent]) -> String {
    events
        .iter()
        .map(|event| {
            let fields = vec![
                json_usize_field("index", event.index),
                json_string_field("trace_id", &event.trace_id),
                json_string_field("status", &event.status),
                json_string_field("execution_phase", &event.execution_phase),
                json_string_field("target", &event.target),
                json_string_field("entry_symbol", &event.entry_symbol),
                json_string_field("entry_kind", &event.entry_kind),
                json_string_field("entry_section_id", &event.entry_section_id),
                json_string_field("first_blocker", &event.first_blocker),
                json_string_field("next_action", &event.next_action),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn hetero_runtime_trace_records_json(records: &[NsdbHeteroRuntimeTraceRecord]) -> String {
    records
        .iter()
        .map(|record| {
            let fields = vec![
                json_usize_field("index", record.index),
                json_string_field("trace_id", &record.trace_id),
                json_string_field("trace_role", &record.trace_role),
                json_string_field("status", &record.status),
                json_string_field("domain_family", &record.domain_family),
                json_string_field("backend_family", &record.backend_family),
                json_string_field("target_device", &record.target_device),
                json_string_field("backend_artifact_key", &record.backend_artifact_key),
                json_string_field("selected_lowering_target", &record.selected_lowering_target),
                json_string_field("payload_format", &record.payload_format),
                json_string_field("payload_path", &record.payload_path),
                json_string_field("bridge_stub_path", &record.bridge_stub_path),
                json_string_array_field("missing_signals", &record.missing_signals),
                json_string_field("next_action", &record.next_action),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn domains_json(domains: &[NsdbDomainDebugInfo]) -> String {
    domains
        .iter()
        .map(|domain| {
            let fields = vec![
                json_string_field("domain_family", &domain.domain_family),
                json_string_field("package_id", &domain.package_id),
                json_string_field("kind", &domain.kind),
                json_string_field("lowering_target", &domain.lowering_target),
                json_string_field("backend_family", &domain.backend_family),
                json_string_field("debug_scope", &domain.debug_scope),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn clock_edges_json(edges: &[NsdbClockEdgeDebugInfo]) -> String {
    edges
        .iter()
        .map(|edge| {
            let fields = vec![
                json_usize_field("index", edge.index),
                json_string_field("from", &edge.from),
                json_string_field("to", &edge.to),
                json_string_field("relation", &edge.relation),
                json_string_field("source", &edge.source),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn data_segments_json(segments: &[NsdbDataSegmentDebugInfo]) -> String {
    segments
        .iter()
        .map(|segment| {
            let fields = vec![
                json_usize_field("index", segment.index),
                json_string_field("segment_id", &segment.segment_id),
                json_string_field("domain_family", &segment.domain_family),
                json_string_field("owner_package", &segment.owner_package),
                json_string_field("order_key", &segment.order_key),
                json_string_field("access_phase", &segment.access_phase),
                json_string_field("source_path", &segment.source_path),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn lowering_units_json(units: &[NsdbLoweringUnitDebugInfo]) -> String {
    units
        .iter()
        .map(|unit| {
            let fields = vec![
                json_usize_field("index", unit.index),
                json_string_field("package_id", &unit.package_id),
                json_string_field("domain_family", &unit.domain_family),
                json_string_field("backend_family", &unit.backend_family),
                json_string_field("selected_lowering_target", &unit.selected_lowering_target),
                json_string_field("artifact_ir_sidecar_path", &unit.artifact_ir_sidecar_path),
                json_string_field("contract_family", &unit.contract_family),
                json_string_field("packaging_role", &unit.packaging_role),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn sidecars_json(sidecars: &[NsdbSidecarDebugInfo]) -> String {
    sidecars
        .iter()
        .map(|sidecar| {
            let fields = vec![
                json_string_field("domain_family", &sidecar.domain_family),
                json_string_field("package_id", &sidecar.package_id),
                json_string_field("path", &sidecar.path),
                json_string_field("schema", &sidecar.schema),
                json_string_field("capability_owner", &sidecar.capability_owner),
                json_string_field("frontend_ir", &sidecar.frontend_ir),
                json_string_field("native_ir", &sidecar.native_ir),
                json_string_field("pipeline_lowering", &sidecar.pipeline_lowering),
                json_string_field("resource_lowering", &sidecar.resource_lowering),
                json_string_field("dispatch_lowering", &sidecar.dispatch_lowering),
                json_string_field("texture_lowering", &sidecar.texture_lowering),
                json_string_field("transport_lowering", &sidecar.transport_lowering),
                json_string_field("tensor_lowering", &sidecar.tensor_lowering),
                json_string_field("memory_lowering", &sidecar.memory_lowering),
                json_string_field("result_lowering", &sidecar.result_lowering),
                json_string_array_field("validation_contracts", &sidecar.validation_contracts),
                json_string_field("entry_symbol", &sidecar.entry_symbol),
                json_string_field("stage_kind", &sidecar.stage_kind),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn json_bool_field(name: &str, value: bool) -> String {
    format!("\"{name}\":{value}")
}

fn json_string_field(name: &str, value: &str) -> String {
    format!("\"{name}\":\"{}\"", json_escape(value))
}

fn json_usize_field(name: &str, value: usize) -> String {
    format!("\"{name}\":{value}")
}

fn json_optional_string_field(name: &str, value: Option<&str>) -> String {
    value
        .map(|value| json_string_field(name, value))
        .unwrap_or_else(|| format!("\"{name}\":null"))
}

fn json_string_array_field(name: &str, values: &[String]) -> String {
    let body = values
        .iter()
        .map(|value| format!("\"{}\"", json_escape(value)))
        .collect::<Vec<_>>()
        .join(",");
    format!("\"{name}\":[{body}]")
}

fn json_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}
