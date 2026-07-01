#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldCheckReport {
    pub(crate) manifest: String,
    pub(crate) valid: bool,
    pub(crate) checks: usize,
    pub(crate) failures: usize,
    pub(crate) artifact_lowering_alignment_consistent: bool,
    pub(crate) artifact_lowering_alignment_mismatches: usize,
    pub(crate) clock_protocol_valid: bool,
    pub(crate) clock_protocol_issues: Vec<String>,
    pub(crate) hetero_calculate_valid: bool,
    pub(crate) hetero_calculate_issues: Vec<String>,
    pub(crate) static_link: bool,
    pub(crate) lifecycle_driven: bool,
    pub(crate) sidecar_capability_valid: bool,
    pub(crate) sidecar_capability_issues: Vec<String>,
    pub(crate) link_input_table_present: bool,
    pub(crate) link_input_table_valid: Option<bool>,
    pub(crate) link_input_table_issues: Vec<String>,
    pub(crate) link_unit_table_present: bool,
    pub(crate) link_unit_table_valid: Option<bool>,
    pub(crate) link_unit_table_issues: Vec<String>,
    pub(crate) link_bundle_present: bool,
    pub(crate) link_bundle_valid: Option<bool>,
    pub(crate) link_bundle_issues: Vec<String>,
    pub(crate) assemble_plan_present: bool,
    pub(crate) assemble_plan_valid: Option<bool>,
    pub(crate) assemble_plan_issues: Vec<String>,
    pub(crate) section_manifest_present: bool,
    pub(crate) section_manifest_valid: Option<bool>,
    pub(crate) section_manifest_issues: Vec<String>,
    pub(crate) container_plan_present: bool,
    pub(crate) container_plan_valid: Option<bool>,
    pub(crate) container_plan_issues: Vec<String>,
    pub(crate) container_present: bool,
    pub(crate) container_valid: Option<bool>,
    pub(crate) container_issues: Vec<String>,
    pub(crate) final_stage_link_mode: String,
    pub(crate) domains: Vec<NsldDomainDiagnostic>,
    pub(crate) sidecar_capabilities: Vec<NsldSidecarCapabilityDiagnostic>,
    pub(crate) clock_edges: Vec<NsldClockEdgeDiagnostic>,
    pub(crate) data_segments: Vec<NsldDataSegmentDiagnostic>,
    pub(crate) issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldDomainDiagnostic {
    pub(crate) domain_family: String,
    pub(crate) package_id: String,
    pub(crate) kind: String,
    pub(crate) packaging_role: String,
    pub(crate) lowering_target: String,
    pub(crate) backend_family: String,
    pub(crate) alignment_consistent: bool,
    pub(crate) alignment_issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldSidecarCapabilityDiagnostic {
    pub(crate) domain_family: String,
    pub(crate) package_id: String,
    pub(crate) path: String,
    pub(crate) content_bytes: usize,
    pub(crate) content_hash: String,
    pub(crate) valid: bool,
    pub(crate) capability_owner: String,
    pub(crate) frontend_ir: String,
    pub(crate) native_ir: String,
    pub(crate) dispatch_lowering: String,
    pub(crate) validation_contracts: Vec<String>,
    pub(crate) issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldClockEdgeDiagnostic {
    pub(crate) index: usize,
    pub(crate) from: String,
    pub(crate) to: String,
    pub(crate) relation: String,
    pub(crate) source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldDataSegmentDiagnostic {
    pub(crate) index: usize,
    pub(crate) segment_id: String,
    pub(crate) domain_family: String,
    pub(crate) owner_package: String,
    pub(crate) order_key: String,
    pub(crate) access_phase: String,
    pub(crate) source_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldClosureReport {
    pub(crate) manifest: String,
    pub(crate) closed: bool,
    pub(crate) internal_contracts: Vec<String>,
    pub(crate) link_inputs: Vec<NsldLinkInputDiagnostic>,
    pub(crate) link_input_count: usize,
    pub(crate) link_input_total_bytes: usize,
    pub(crate) link_input_table_hash: String,
    pub(crate) link_input_table_present: bool,
    pub(crate) link_input_table_valid: Option<bool>,
    pub(crate) external_dependencies: Vec<String>,
    pub(crate) unresolved: Vec<String>,
    pub(crate) host_wrapper_required: bool,
    pub(crate) domain_count: usize,
    pub(crate) hetero_domain_count: usize,
    pub(crate) sidecar_capability_count: usize,
    pub(crate) clock_edge_count: usize,
    pub(crate) data_segment_count: usize,
    pub(crate) final_stage_link_mode: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldLinkUnitReport {
    pub(crate) manifest: String,
    pub(crate) unit_count: usize,
    pub(crate) hetero_unit_count: usize,
    pub(crate) link_input_count: usize,
    pub(crate) clock_edge_count: usize,
    pub(crate) data_segment_count: usize,
    pub(crate) unit_table_hash: String,
    pub(crate) units: Vec<NsldLinkUnitDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldLinkUnitDiagnostic {
    pub(crate) order_index: usize,
    pub(crate) unit_id: String,
    pub(crate) unit_kind: String,
    pub(crate) domain_family: String,
    pub(crate) package_id: String,
    pub(crate) backend_family: String,
    pub(crate) lowering_target: String,
    pub(crate) packaging_role: String,
    pub(crate) link_input_ids: Vec<String>,
    pub(crate) clock_edge_count: usize,
    pub(crate) data_segment_count: usize,
    pub(crate) requires_host_wrapper: bool,
    pub(crate) deterministic_order_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldLinkUnitsEmitReport {
    pub(crate) manifest: String,
    pub(crate) output_path: String,
    pub(crate) unit_count: usize,
    pub(crate) hetero_unit_count: usize,
    pub(crate) link_input_count: usize,
    pub(crate) unit_table_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldLinkUnitsVerifyReport {
    pub(crate) manifest: String,
    pub(crate) input_path: String,
    pub(crate) valid: bool,
    pub(crate) expected_unit_count: usize,
    pub(crate) expected_hetero_unit_count: usize,
    pub(crate) expected_link_input_count: usize,
    pub(crate) expected_unit_table_hash: String,
    pub(crate) actual_unit_count: Option<usize>,
    pub(crate) actual_hetero_unit_count: Option<usize>,
    pub(crate) actual_link_input_count: Option<usize>,
    pub(crate) actual_unit_table_hash: Option<String>,
    pub(crate) issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldLinkBundleReport {
    pub(crate) manifest: String,
    pub(crate) bundle_id: String,
    pub(crate) bundle_hash: String,
    pub(crate) bundle_ready: bool,
    pub(crate) unit_count: usize,
    pub(crate) hetero_unit_count: usize,
    pub(crate) link_input_count: usize,
    pub(crate) link_input_total_bytes: usize,
    pub(crate) link_input_table_hash: String,
    pub(crate) unit_table_hash: String,
    pub(crate) clock_edge_count: usize,
    pub(crate) data_segment_count: usize,
    pub(crate) final_stage_link_mode: String,
    pub(crate) host_wrapper_required: bool,
    pub(crate) compiled_artifact_path: String,
    pub(crate) native_output_path: String,
    pub(crate) issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldLinkBundleEmitReport {
    pub(crate) manifest: String,
    pub(crate) output_path: String,
    pub(crate) bundle_id: String,
    pub(crate) bundle_hash: String,
    pub(crate) bundle_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldLinkBundleVerifyReport {
    pub(crate) manifest: String,
    pub(crate) input_path: String,
    pub(crate) valid: bool,
    pub(crate) expected_bundle_id: String,
    pub(crate) expected_bundle_hash: String,
    pub(crate) actual_bundle_id: Option<String>,
    pub(crate) actual_bundle_hash: Option<String>,
    pub(crate) issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldPrepareReport {
    pub(crate) manifest: String,
    pub(crate) valid: bool,
    pub(crate) output_dir: String,
    pub(crate) link_input_table_path: String,
    pub(crate) link_unit_table_path: String,
    pub(crate) link_bundle_path: String,
    pub(crate) assemble_plan_path: String,
    pub(crate) section_manifest_path: String,
    pub(crate) container_plan_path: String,
    pub(crate) container_path: String,
    pub(crate) container_payload_path: String,
    pub(crate) link_input_count: usize,
    pub(crate) link_input_table_hash: String,
    pub(crate) unit_count: usize,
    pub(crate) unit_table_hash: String,
    pub(crate) bundle_id: String,
    pub(crate) bundle_hash: String,
    pub(crate) bundle_ready: bool,
    pub(crate) assemble_plan_hash: String,
    pub(crate) section_table_hash: String,
    pub(crate) container_layout_hash: String,
    pub(crate) container_hash: String,
    pub(crate) payload_size_bytes: usize,
    pub(crate) payload_hash: String,
    pub(crate) issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldAssemblePlanReport {
    pub(crate) manifest: String,
    pub(crate) ready: bool,
    pub(crate) bundle_id: String,
    pub(crate) bundle_hash: String,
    pub(crate) assemble_plan_hash: String,
    pub(crate) section_count: usize,
    pub(crate) sections: Vec<NsldAssembleSectionDiagnostic>,
    pub(crate) blockers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldAssembleSectionDiagnostic {
    pub(crate) order_index: usize,
    pub(crate) section_id: String,
    pub(crate) section_kind: String,
    pub(crate) source_path: String,
    pub(crate) source_hash: String,
    pub(crate) required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldAssemblePlanEmitReport {
    pub(crate) manifest: String,
    pub(crate) output_path: String,
    pub(crate) ready: bool,
    pub(crate) assemble_plan_hash: String,
    pub(crate) section_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldAssemblePlanVerifyReport {
    pub(crate) manifest: String,
    pub(crate) input_path: String,
    pub(crate) valid: bool,
    pub(crate) expected_assemble_plan_hash: String,
    pub(crate) expected_section_count: usize,
    pub(crate) actual_assemble_plan_hash: Option<String>,
    pub(crate) actual_section_count: Option<usize>,
    pub(crate) issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldSectionManifestReport {
    pub(crate) manifest: String,
    pub(crate) ready: bool,
    pub(crate) assemble_plan_hash: String,
    pub(crate) section_count: usize,
    pub(crate) section_table_hash: String,
    pub(crate) sections: Vec<NsldAssembleSectionDiagnostic>,
    pub(crate) blockers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldSectionManifestEmitReport {
    pub(crate) manifest: String,
    pub(crate) output_path: String,
    pub(crate) ready: bool,
    pub(crate) section_count: usize,
    pub(crate) section_table_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldSectionManifestVerifyReport {
    pub(crate) manifest: String,
    pub(crate) input_path: String,
    pub(crate) valid: bool,
    pub(crate) expected_section_count: usize,
    pub(crate) expected_section_table_hash: String,
    pub(crate) actual_section_count: Option<usize>,
    pub(crate) actual_section_table_hash: Option<String>,
    pub(crate) issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldLinkInputDiagnostic {
    pub(crate) order_index: usize,
    pub(crate) input_id: String,
    pub(crate) input_kind: String,
    pub(crate) domain_family: String,
    pub(crate) package_id: String,
    pub(crate) path: String,
    pub(crate) native_ir: String,
    pub(crate) dispatch_lowering: String,
    pub(crate) contract_count: usize,
    pub(crate) content_bytes: usize,
    pub(crate) content_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldLinkInputSummary {
    pub(crate) inputs: Vec<NsldLinkInputDiagnostic>,
    pub(crate) count: usize,
    pub(crate) total_bytes: usize,
    pub(crate) table_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldLinkInputsEmitReport {
    pub(crate) manifest: String,
    pub(crate) output_path: String,
    pub(crate) link_input_count: usize,
    pub(crate) link_input_total_bytes: usize,
    pub(crate) link_input_table_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldLinkInputsVerifyReport {
    pub(crate) manifest: String,
    pub(crate) input_path: String,
    pub(crate) valid: bool,
    pub(crate) expected_link_input_count: usize,
    pub(crate) expected_link_input_total_bytes: usize,
    pub(crate) expected_link_input_table_hash: String,
    pub(crate) actual_link_input_count: Option<usize>,
    pub(crate) actual_link_input_total_bytes: Option<usize>,
    pub(crate) actual_link_input_table_hash: Option<String>,
    pub(crate) issues: Vec<String>,
}
