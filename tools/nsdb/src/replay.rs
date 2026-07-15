use crate::model::{NsdbInspectReport, NsdbPayloadExecutionEvent};

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

#[cfg(test)]
mod tests {
    use super::build_replay_plan;
    use crate::model::{
        NsdbDomainDebugInfo, NsdbHeteroRuntimeTraceInfo, NsdbHeteroRuntimeTraceRecord,
        NsdbInspectReport, NsdbPayloadExecutionEvent, NsdbPayloadExecutionEventFilter,
        NsdbPayloadExecutionHandoffInfo, NsdbSidecarDebugInfo,
    };

    #[test]
    fn builds_replay_checkpoints_from_payload_events() {
        let report = NsdbInspectReport {
            manifest: "manifest.toml".to_owned(),
            debug_model: "yir-metadata".to_owned(),
            native_debugger_visibility: "host-shell-only".to_owned(),
            nsdb_visibility: "domains+clock+segments+lowering-units".to_owned(),
            debug_readiness: "metadata-partial".to_owned(),
            yir_debuggable: false,
            domain_count: 0,
            hetero_domain_count: 0,
            clock_edge_count: 0,
            data_segment_count: 0,
            lowering_unit_count: 0,
            sidecar_count: 0,
            payload_execution_event_filter: NsdbPayloadExecutionEventFilter::default(),
            payload_execution_handoff: NsdbPayloadExecutionHandoffInfo {
                available: true,
                path: "nuis.nsdb.payload-execution-handoff.toml".to_owned(),
                protocol: "nuis-nsdb-payload-execution-handoff-v1".to_owned(),
                debugger_contract: "nsdb-yir-payload-execution-trace-v1".to_owned(),
                status: "ready".to_owned(),
                record_count: 2,
                ready_record_count: 1,
                first_trace_id: "payload-trace:container-loader:nuis.bootstrap.lifecycle.v1"
                    .to_owned(),
                first_status: "ready".to_owned(),
                first_next_action: "handoff-payload-trace-to-nsdb".to_owned(),
                first_entry_symbol: "nuis.bootstrap.lifecycle.v1".to_owned(),
                first_execution_phase: "container-loader-handoff".to_owned(),
                events: vec![
                    NsdbPayloadExecutionEvent {
                        index: 0,
                        trace_id: "payload-trace:container-loader:nuis.bootstrap.lifecycle.v1"
                            .to_owned(),
                        status: "ready".to_owned(),
                        execution_phase: "container-loader-handoff".to_owned(),
                        target: "container-loader".to_owned(),
                        entry_symbol: "nuis.bootstrap.lifecycle.v1".to_owned(),
                        entry_kind: "lifecycle-bootstrap".to_owned(),
                        entry_section_id: "sec0000.compiled-artifact".to_owned(),
                        first_blocker: "none".to_owned(),
                        next_action: "handoff-payload-trace-to-nsdb".to_owned(),
                    },
                    NsdbPayloadExecutionEvent {
                        index: 1,
                        trace_id: "payload-trace:shader:pixelmagic.blur".to_owned(),
                        status: "blocked".to_owned(),
                        execution_phase: "device-dispatch".to_owned(),
                        target: "shader".to_owned(),
                        entry_symbol: "pixelmagic.blur".to_owned(),
                        entry_kind: "shader-kernel".to_owned(),
                        entry_section_id: "sec0002.shader".to_owned(),
                        first_blocker: "device-execution-sample-missing".to_owned(),
                        next_action: "materialize-device-execution-trace".to_owned(),
                    },
                ],
            },
            hetero_runtime_trace: NsdbHeteroRuntimeTraceInfo {
                available: true,
                path: "nuis.nsdb.hetero-runtime-trace.toml".to_owned(),
                protocol: "nuis-nsdb-hetero-runtime-trace-v1".to_owned(),
                debugger_contract: "nsdb-yir-hetero-runtime-trace-v1".to_owned(),
                status: "execution-pending".to_owned(),
                record_count: 1,
                ready_record_count: 0,
                backend_execution_record_count: 1,
                first_trace_id: "hetero-trace:shader:metal:apple-silicon-gpu".to_owned(),
                first_blocker: "none".to_owned(),
                next_action: "materialize-device-execution-trace".to_owned(),
                records: vec![NsdbHeteroRuntimeTraceRecord {
                    index: 0,
                    trace_id: "hetero-trace:shader:metal:apple-silicon-gpu".to_owned(),
                    trace_role: "backend-artifact".to_owned(),
                    status: "execution-pending".to_owned(),
                    domain_family: "shader".to_owned(),
                    backend_family: "metal".to_owned(),
                    target_device: "apple-silicon-gpu".to_owned(),
                    backend_artifact_key: "shader:metal:apple-silicon-gpu".to_owned(),
                    selected_lowering_target: "metal".to_owned(),
                    payload_format: "metallib".to_owned(),
                    payload_path: "pixelmagic.metallib".to_owned(),
                    bridge_stub_path: "pixelmagic.bridge".to_owned(),
                    missing_signals: Vec::new(),
                    next_action: "materialize-device-execution-trace".to_owned(),
                }],
            },
            domains: vec![NsdbDomainDebugInfo {
                domain_family: "shader".to_owned(),
                package_id: "pixelmagic".to_owned(),
                kind: "heterogeneous".to_owned(),
                lowering_target: "metal".to_owned(),
                backend_family: "metal".to_owned(),
                debug_scope: "yir-domain".to_owned(),
            }],
            clock_edges: Vec::new(),
            data_segments: Vec::new(),
            lowering_units: Vec::new(),
            sidecars: vec![NsdbSidecarDebugInfo {
                domain_family: "shader".to_owned(),
                package_id: "pixelmagic".to_owned(),
                path: "pixelmagic.shader.sidecar.json".to_owned(),
                schema: "nuis-yir-sidecar-v1".to_owned(),
                capability_owner: "shader".to_owned(),
                frontend_ir: "nuis-yir.shader".to_owned(),
                native_ir: "msl2.4".to_owned(),
                pipeline_lowering: "metal-compute-pipeline".to_owned(),
                resource_lowering: "metal-buffer".to_owned(),
                dispatch_lowering: "metal-dispatch-threadgroups".to_owned(),
                texture_lowering: "metal-texture".to_owned(),
                transport_lowering: "none".to_owned(),
                tensor_lowering: "none".to_owned(),
                memory_lowering: "metal-shared-buffer".to_owned(),
                result_lowering: "metal-buffer-readback".to_owned(),
                validation_contracts: vec!["shader-yir-contract".to_owned()],
                entry_symbol: "pixelmagic.blur".to_owned(),
                stage_kind: "compute".to_owned(),
            }],
            missing_metadata: Vec::new(),
        };

        let plan = build_replay_plan(&report);

        assert_eq!(plan.protocol, "nsdb-payload-execution-replay-plan-v1");
        assert_eq!(plan.status, "blocked");
        assert_eq!(plan.checkpoint_count, 2);
        assert_eq!(plan.replayable_checkpoint_count, 1);
        assert_eq!(plan.checkpoints[0].checkpoint_kind, "loader-checkpoint");
        assert_eq!(plan.checkpoints[0].replay_status, "replayable");
        assert_eq!(plan.checkpoints[0].frame_id, "frame:payload:0:loader");
        assert_eq!(
            plan.checkpoints[0].slot_scope,
            "container-loader:nuis.bootstrap.lifecycle.v1"
        );
        assert_eq!(plan.checkpoints[0].value_state_status, "metadata-only");
        assert_eq!(
            plan.checkpoints[0].value_sample_contract,
            "nsdb-yir-value-sample-ref-v1"
        );
        assert_eq!(
            plan.checkpoints[0].value_sample_ref,
            "value-sample:payload-trace:container-loader:nuis.bootstrap.lifecycle.v1:container-loader:nuis.bootstrap.lifecycle.v1"
        );
        assert_eq!(
            plan.checkpoints[0].value_sample_source,
            "payload-execution-handoff"
        );
        assert_eq!(
            plan.checkpoints[0].value_sample_resolution_status,
            "metadata-resolved"
        );
        assert_eq!(
            plan.checkpoints[0].value_sample_materialization_status,
            "metadata-sample-materialized"
        );
        assert_eq!(
            plan.checkpoints[0].value_sample_payload_format,
            "payload-execution-metadata"
        );
        assert_eq!(plan.checkpoints[0].value_slot_id, "slot:payload:0:loader");
        assert_eq!(
            plan.checkpoints[0].value_slot_scope,
            "container-loader:nuis.bootstrap.lifecycle.v1"
        );
        assert_eq!(
            plan.checkpoints[0].value_schema_contract,
            "nsdb-yir-value-schema-ref-v1"
        );
        assert_eq!(
            plan.checkpoints[0].value_schema_status,
            "schema-metadata-only"
        );
        assert_eq!(
            plan.checkpoints[0].value_schema_hint,
            "payload-execution-event-metadata"
        );
        assert_eq!(
            plan.checkpoints[1].checkpoint_kind,
            "device-dispatch-checkpoint"
        );
        assert_eq!(plan.checkpoints[1].frame_id, "frame:payload:1:device");
        assert_eq!(plan.checkpoints[1].slot_scope, "shader:pixelmagic.blur");
        assert_eq!(plan.checkpoints[1].value_state_status, "blocked");
        assert_eq!(
            plan.checkpoints[1].value_sample_ref,
            "value-sample:payload-trace:shader:pixelmagic.blur:shader:pixelmagic.blur"
        );
        assert_eq!(
            plan.checkpoints[1].value_sample_source,
            "hetero-runtime-trace"
        );
        assert_eq!(
            plan.checkpoints[1].value_sample_resolution_status,
            "trace-record-observed"
        );
        assert_eq!(
            plan.checkpoints[1].value_sample_resolution_detail,
            "hetero-runtime-trace:hetero-trace:shader:metal:apple-silicon-gpu:execution-pending"
        );
        assert_eq!(
            plan.checkpoints[1].value_sample_materialization_status,
            "sample-descriptor-materialized"
        );
        assert_eq!(plan.checkpoints[1].value_sample_payload_format, "metallib");
        assert_eq!(
            plan.checkpoints[1].value_sample_payload_path,
            "pixelmagic.metallib"
        );
        assert_eq!(
            plan.checkpoints[1].value_sample_bridge_stub_path,
            "pixelmagic.bridge"
        );
        assert_eq!(plan.checkpoints[1].value_slot_id, "slot:payload:1:device");
        assert_eq!(
            plan.checkpoints[1].value_slot_scope,
            "shader:pixelmagic.blur"
        );
        assert_eq!(
            plan.checkpoints[1].value_schema_status,
            "schema-opaque-payload"
        );
        assert_eq!(
            plan.checkpoints[1].value_schema_hint,
            "opaque-runtime-payload:metallib"
        );
        assert_eq!(
            plan.first_blocker.as_deref(),
            Some("device-execution-sample-missing")
        );
    }
}
