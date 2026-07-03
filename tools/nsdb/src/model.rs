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
    pub(crate) domains: Vec<NsdbDomainDebugInfo>,
    pub(crate) clock_edges: Vec<NsdbClockEdgeDebugInfo>,
    pub(crate) data_segments: Vec<NsdbDataSegmentDebugInfo>,
    pub(crate) lowering_units: Vec<NsdbLoweringUnitDebugInfo>,
    pub(crate) sidecars: Vec<NsdbSidecarDebugInfo>,
    pub(crate) missing_metadata: Vec<String>,
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
