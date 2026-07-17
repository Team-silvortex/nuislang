use crate::model::{
    NsdbClockEdgeDebugInfo, NsdbDataSegmentDebugInfo, NsdbDeviceProviderSampleRecordInfo,
    NsdbDeviceSampleHandoffRecord, NsdbDomainDebugInfo, NsdbHeteroRuntimeTraceRecord,
    NsdbInspectReport, NsdbLoweringUnitDebugInfo, NsdbPayloadDecoderManifestRecordInfo,
    NsdbPayloadExecutionEvent, NsdbSidecarDebugInfo,
};
use crate::provider_sample_payload::{
    provider_output_payload_from_record, provider_output_payload_summary,
};
// Replay JSON contract anchors live in json_replay.rs:
// replay_hetero_execution_closure_status, replay_hetero_execution_closure_ready.
pub(crate) use crate::json_replay::nsdb_replay_plan_json;

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
        json_string_field(
            "payload_execution_handoff_hetero_execution_closure_protocol",
            &report
                .payload_execution_handoff
                .hetero_execution_closure_protocol,
        ),
        json_string_field(
            "payload_execution_handoff_hetero_execution_closure_status",
            &report
                .payload_execution_handoff
                .hetero_execution_closure_status,
        ),
        json_string_field(
            "payload_execution_handoff_hetero_execution_closure_ready",
            &report
                .payload_execution_handoff
                .hetero_execution_closure_ready,
        ),
        json_string_field(
            "payload_execution_handoff_hetero_execution_closure_next_action",
            &report
                .payload_execution_handoff
                .hetero_execution_closure_next_action,
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
        json_usize_field(
            "hetero_runtime_trace_device_sample_handoff_record_count",
            report
                .hetero_runtime_trace
                .device_sample_handoff_record_count,
        ),
        json_string_field(
            "hetero_runtime_trace_device_sample_handoff_protocol",
            &report.hetero_runtime_trace.device_sample_handoff_protocol,
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
        format!(
            "\"device_sample_handoffs\":[{}]",
            device_sample_handoffs_json(&report.hetero_runtime_trace.device_sample_handoffs)
        ),
        json_bool_field(
            "payload_decoder_manifest_available",
            report.payload_decoder_manifest.available,
        ),
        json_string_field(
            "payload_decoder_manifest_path",
            &report.payload_decoder_manifest.path,
        ),
        json_string_field(
            "payload_decoder_manifest_protocol",
            &report.payload_decoder_manifest.protocol,
        ),
        json_string_field(
            "payload_decoder_manifest_schema",
            &report.payload_decoder_manifest.schema,
        ),
        json_string_field(
            "payload_decoder_manifest_status",
            &report.payload_decoder_manifest.status,
        ),
        json_usize_field(
            "payload_decoder_manifest_record_count",
            report.payload_decoder_manifest.record_count,
        ),
        json_usize_field(
            "payload_decoder_manifest_valid_record_count",
            report.payload_decoder_manifest.valid_record_count,
        ),
        json_usize_field(
            "payload_decoder_manifest_invalid_record_count",
            report.payload_decoder_manifest.invalid_record_count,
        ),
        json_string_field(
            "payload_decoder_manifest_first_payload_format",
            &report.payload_decoder_manifest.first_payload_format,
        ),
        json_string_field(
            "payload_decoder_manifest_first_decoder_id",
            &report.payload_decoder_manifest.first_decoder_id,
        ),
        json_string_field(
            "payload_decoder_manifest_first_diagnostic",
            &report.payload_decoder_manifest.first_diagnostic,
        ),
        format!(
            "\"payload_decoder_manifest_records\":[{}]",
            payload_decoder_manifest_records_json(&report.payload_decoder_manifest.records)
        ),
        json_bool_field(
            "device_provider_sample_manifest_available",
            report.device_provider_sample_manifest.available,
        ),
        json_string_field(
            "device_provider_sample_manifest_path",
            &report.device_provider_sample_manifest.path,
        ),
        json_string_field(
            "device_provider_sample_manifest_protocol",
            &report.device_provider_sample_manifest.protocol,
        ),
        json_string_field(
            "device_provider_sample_manifest_schema",
            &report.device_provider_sample_manifest.schema,
        ),
        json_string_field(
            "device_provider_sample_manifest_status",
            &report.device_provider_sample_manifest.status,
        ),
        json_usize_field(
            "device_provider_sample_manifest_record_count",
            report.device_provider_sample_manifest.record_count,
        ),
        json_usize_field(
            "device_provider_sample_manifest_pending_record_count",
            report.device_provider_sample_manifest.pending_record_count,
        ),
        json_usize_field(
            "device_provider_sample_manifest_invalid_record_count",
            report.device_provider_sample_manifest.invalid_record_count,
        ),
        json_string_field(
            "device_provider_sample_manifest_first_trace_id",
            &report.device_provider_sample_manifest.first_trace_id,
        ),
        json_string_field(
            "device_provider_sample_manifest_first_provider_family",
            &report.device_provider_sample_manifest.first_provider_family,
        ),
        json_string_field(
            "device_provider_sample_manifest_first_materialization_status",
            &report
                .device_provider_sample_manifest
                .first_materialization_status,
        ),
        json_string_field(
            "device_provider_sample_manifest_first_diagnostic",
            &report.device_provider_sample_manifest.first_diagnostic,
        ),
        format!(
            "\"device_provider_sample_manifest_records\":[{}]",
            device_provider_sample_records_json(&report.device_provider_sample_manifest.records)
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

fn payload_decoder_manifest_records_json(
    records: &[NsdbPayloadDecoderManifestRecordInfo],
) -> String {
    records
        .iter()
        .map(|record| {
            let fields = vec![
                json_usize_field("index", record.index),
                json_bool_field("valid", record.valid),
                json_string_field("payload_format", &record.payload_format),
                json_string_field("decoder_id", &record.decoder_id),
                json_string_field("diagnostic", &record.diagnostic),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn device_provider_sample_records_json(records: &[NsdbDeviceProviderSampleRecordInfo]) -> String {
    records
        .iter()
        .map(|record| {
            let payload_summary = provider_output_payload_summary(
                provider_output_payload_from_record(record).as_ref(),
            );
            let fields = vec![
                json_usize_field("index", record.index),
                json_bool_field("valid", record.valid),
                json_string_field("trace_id", &record.trace_id),
                json_string_field("provider", &record.provider),
                json_string_field("provider_family", &record.provider_family),
                json_string_field("handoff_target", &record.handoff_target),
                json_string_field("sample_status", &record.sample_status),
                json_string_field("validation_status", &record.validation_status),
                json_string_field("input_evidence", &record.input_evidence),
                json_string_field("output_evidence", &record.output_evidence),
                json_string_field(
                    "provider_output_payload_contract",
                    &record.provider_output_payload_contract,
                ),
                json_string_field(
                    "provider_output_payload_status",
                    &record.provider_output_payload_status,
                ),
                json_string_field(
                    "provider_output_payload_evidence_status",
                    &record.provider_output_payload_evidence_status,
                ),
                json_string_field(
                    "provider_output_payload_evidence",
                    &record.provider_output_payload_evidence,
                ),
                json_string_field(
                    "provider_output_payload_detail",
                    &record.provider_output_payload_detail,
                ),
                json_string_field("provider_output_payload_path", &payload_summary.path),
                json_string_field("provider_output_payload_hash", &payload_summary.hash),
                json_string_field(
                    "provider_output_payload_attach_status",
                    &payload_summary.attach_status,
                ),
                json_string_field(
                    "provider_output_payload_next_action",
                    &record.provider_output_payload_next_action,
                ),
                json_string_field("materialization_status", &record.materialization_status),
                json_string_field("materialization_detail", &record.materialization_detail),
                json_string_field("next_action", &record.next_action),
                json_string_field("diagnostic", &record.diagnostic),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

pub(crate) fn nsdb_events_report_json(report: &NsdbInspectReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsdb"),
        json_string_field("kind", "nsdb_payload_execution_events"),
        json_string_field("manifest", &report.manifest),
        json_string_field(
            "payload_execution_event_query_contract",
            "nsdb-payload-execution-event-query-v1",
        ),
        json_string_field(
            "payload_execution_event_source",
            "payload-execution-handoff-events",
        ),
        json_string_field(
            "payload_execution_event_source_protocol",
            &report.payload_execution_handoff.protocol,
        ),
        json_string_field(
            "payload_execution_event_source_debugger_contract",
            &report.payload_execution_handoff.debugger_contract,
        ),
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
        json_usize_field(
            "payload_execution_event_query_result_count",
            report.payload_execution_handoff.events.len(),
        ),
        format!(
            "\"payload_execution_events\":[{}]",
            payload_execution_events_json(&report.payload_execution_handoff.events)
        ),
    ];
    format!("{{{}}}", fields.join(","))
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

fn device_sample_handoffs_json(handoffs: &[NsdbDeviceSampleHandoffRecord]) -> String {
    handoffs
        .iter()
        .map(|handoff| {
            let fields = vec![
                json_usize_field("index", handoff.index),
                json_string_field("trace_id", &handoff.trace_id),
                json_string_field("protocol", &handoff.protocol),
                json_string_field("provider", &handoff.provider),
                json_string_field("provider_family", &handoff.provider_family),
                json_string_field("handoff_target", &handoff.handoff_target),
                json_string_field("handoff_status", &handoff.handoff_status),
                json_string_field("validation_status", &handoff.validation_status),
                json_string_field("input_evidence", &handoff.input_evidence),
                json_string_field("output_evidence", &handoff.output_evidence),
                json_string_field("next_action", &handoff.next_action),
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

pub(crate) fn json_bool_field(name: &str, value: bool) -> String {
    format!("\"{name}\":{value}")
}

pub(crate) fn json_string_field(name: &str, value: &str) -> String {
    format!("\"{name}\":\"{}\"", json_escape(value))
}

pub(crate) fn json_usize_field(name: &str, value: usize) -> String {
    format!("\"{name}\":{value}")
}

pub(crate) fn json_optional_string_field(name: &str, value: Option<&str>) -> String {
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
