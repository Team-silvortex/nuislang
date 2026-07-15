#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsdbInspectReport {
    pub(crate) manifest: String,
    pub(crate) debug_model: String,
    pub(crate) native_debugger_visibility: String,
    pub(crate) nsdb_visibility: String,
    pub(crate) debug_readiness: String,
    pub(crate) yir_debuggable: bool,
    pub(crate) domain_count: usize,
    pub(crate) hetero_domain_count: usize,
    pub(crate) clock_edge_count: usize,
    pub(crate) data_segment_count: usize,
    pub(crate) lowering_unit_count: usize,
    pub(crate) sidecar_count: usize,
    pub(crate) payload_execution_event_filter: NsdbPayloadExecutionEventFilter,
    pub(crate) payload_execution_handoff: NsdbPayloadExecutionHandoffInfo,
    pub(crate) domains: Vec<NsdbDomainDebugInfo>,
    pub(crate) clock_edges: Vec<NsdbClockEdgeDebugInfo>,
    pub(crate) data_segments: Vec<NsdbDataSegmentDebugInfo>,
    pub(crate) lowering_units: Vec<NsdbLoweringUnitDebugInfo>,
    pub(crate) sidecars: Vec<NsdbSidecarDebugInfo>,
    pub(crate) missing_metadata: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct NsdbPayloadExecutionEventFilter {
    pub(crate) status: Option<String>,
    pub(crate) phase: Option<String>,
    pub(crate) trace_id: Option<String>,
}

impl NsdbPayloadExecutionEventFilter {
    pub(crate) fn active(&self) -> bool {
        self.status.is_some() || self.phase.is_some() || self.trace_id.is_some()
    }

    pub(crate) fn matches(&self, event: &NsdbPayloadExecutionEvent) -> bool {
        self.status
            .as_ref()
            .is_none_or(|status| &event.status == status)
            && self
                .phase
                .as_ref()
                .is_none_or(|phase| &event.execution_phase == phase)
            && self
                .trace_id
                .as_ref()
                .is_none_or(|trace_id| &event.trace_id == trace_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsdbPayloadExecutionHandoffInfo {
    pub(crate) available: bool,
    pub(crate) path: String,
    pub(crate) protocol: String,
    pub(crate) debugger_contract: String,
    pub(crate) status: String,
    pub(crate) record_count: usize,
    pub(crate) ready_record_count: usize,
    pub(crate) first_trace_id: String,
    pub(crate) first_status: String,
    pub(crate) first_next_action: String,
    pub(crate) first_entry_symbol: String,
    pub(crate) first_execution_phase: String,
    pub(crate) events: Vec<NsdbPayloadExecutionEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsdbPayloadExecutionEvent {
    pub(crate) index: usize,
    pub(crate) trace_id: String,
    pub(crate) status: String,
    pub(crate) execution_phase: String,
    pub(crate) target: String,
    pub(crate) entry_symbol: String,
    pub(crate) entry_kind: String,
    pub(crate) entry_section_id: String,
    pub(crate) first_blocker: String,
    pub(crate) next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsdbDomainDebugInfo {
    pub(crate) domain_family: String,
    pub(crate) package_id: String,
    pub(crate) kind: String,
    pub(crate) lowering_target: String,
    pub(crate) backend_family: String,
    pub(crate) debug_scope: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsdbClockEdgeDebugInfo {
    pub(crate) index: usize,
    pub(crate) from: String,
    pub(crate) to: String,
    pub(crate) relation: String,
    pub(crate) source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsdbDataSegmentDebugInfo {
    pub(crate) index: usize,
    pub(crate) segment_id: String,
    pub(crate) domain_family: String,
    pub(crate) owner_package: String,
    pub(crate) order_key: String,
    pub(crate) access_phase: String,
    pub(crate) source_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsdbLoweringUnitDebugInfo {
    pub(crate) index: usize,
    pub(crate) package_id: String,
    pub(crate) domain_family: String,
    pub(crate) backend_family: String,
    pub(crate) selected_lowering_target: String,
    pub(crate) artifact_ir_sidecar_path: String,
    pub(crate) contract_family: String,
    pub(crate) packaging_role: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsdbSidecarDebugInfo {
    pub(crate) domain_family: String,
    pub(crate) package_id: String,
    pub(crate) path: String,
    pub(crate) schema: String,
    pub(crate) capability_owner: String,
    pub(crate) frontend_ir: String,
    pub(crate) native_ir: String,
    pub(crate) pipeline_lowering: String,
    pub(crate) resource_lowering: String,
    pub(crate) dispatch_lowering: String,
    pub(crate) texture_lowering: String,
    pub(crate) transport_lowering: String,
    pub(crate) tensor_lowering: String,
    pub(crate) memory_lowering: String,
    pub(crate) result_lowering: String,
    pub(crate) validation_contracts: Vec<String>,
    pub(crate) entry_symbol: String,
    pub(crate) stage_kind: String,
}

#[cfg(test)]
mod tests {
    use super::{NsdbPayloadExecutionEvent, NsdbPayloadExecutionEventFilter};

    #[test]
    fn payload_execution_event_filter_matches_all_selected_fields() {
        let event = NsdbPayloadExecutionEvent {
            index: 0,
            trace_id: "payload-trace:shader:pixelmagic.blur".to_owned(),
            status: "blocked".to_owned(),
            execution_phase: "device-dispatch".to_owned(),
            target: "shader".to_owned(),
            entry_symbol: "pixelmagic.blur".to_owned(),
            entry_kind: "shader-kernel".to_owned(),
            entry_section_id: "sec0002.shader".to_owned(),
            first_blocker: "device-execution-sample-missing".to_owned(),
            next_action: "materialize-device-execution-trace".to_owned(),
        };
        let filter = NsdbPayloadExecutionEventFilter {
            status: Some("blocked".to_owned()),
            phase: Some("device-dispatch".to_owned()),
            trace_id: Some("payload-trace:shader:pixelmagic.blur".to_owned()),
        };
        assert!(filter.active());
        assert!(filter.matches(&event));

        let wrong_phase = NsdbPayloadExecutionEventFilter {
            phase: Some("container-loader-handoff".to_owned()),
            ..filter
        };
        assert!(!wrong_phase.matches(&event));
    }
}
