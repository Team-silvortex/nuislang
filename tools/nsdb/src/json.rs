use crate::model::{
    NsdbClockEdgeDebugInfo, NsdbDataSegmentDebugInfo, NsdbDomainDebugInfo, NsdbInspectReport,
    NsdbLoweringUnitDebugInfo, NsdbPayloadExecutionEvent, NsdbSidecarDebugInfo,
};
use crate::replay::{build_replay_plan, NsdbReplayCheckpoint};

pub(crate) fn nsdb_inspect_report_json(report: &NsdbInspectReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsdb"),
        json_string_field("kind", "nsdb_yir_debug_inspect"),
        json_string_field("manifest", &report.manifest),
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
