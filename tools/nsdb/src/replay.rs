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
        .map(replay_checkpoint_for_event)
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

fn replay_checkpoint_for_event(event: &NsdbPayloadExecutionEvent) -> NsdbReplayCheckpoint {
    let first_blocker = if event.first_blocker == "none" && event.status == "ready" {
        None
    } else if event.first_blocker == "none" {
        Some(format!("payload-event-status:{}", event.status))
    } else {
        Some(event.first_blocker.clone())
    };
    NsdbReplayCheckpoint {
        index: event.index,
        trace_id: event.trace_id.clone(),
        checkpoint_kind: checkpoint_kind_for_phase(&event.execution_phase).to_owned(),
        replay_status: if first_blocker.is_none() {
            "replayable".to_owned()
        } else {
            "blocked".to_owned()
        },
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

#[cfg(test)]
mod tests {
    use super::build_replay_plan;
    use crate::model::{
        NsdbInspectReport, NsdbPayloadExecutionEvent, NsdbPayloadExecutionEventFilter,
        NsdbPayloadExecutionHandoffInfo,
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
            domains: Vec::new(),
            clock_edges: Vec::new(),
            data_segments: Vec::new(),
            lowering_units: Vec::new(),
            sidecars: Vec::new(),
            missing_metadata: Vec::new(),
        };

        let plan = build_replay_plan(&report);

        assert_eq!(plan.protocol, "nsdb-payload-execution-replay-plan-v1");
        assert_eq!(plan.status, "blocked");
        assert_eq!(plan.checkpoint_count, 2);
        assert_eq!(plan.replayable_checkpoint_count, 1);
        assert_eq!(plan.checkpoints[0].checkpoint_kind, "loader-checkpoint");
        assert_eq!(plan.checkpoints[0].replay_status, "replayable");
        assert_eq!(
            plan.checkpoints[1].checkpoint_kind,
            "device-dispatch-checkpoint"
        );
        assert_eq!(
            plan.first_blocker.as_deref(),
            Some("device-execution-sample-missing")
        );
    }
}
