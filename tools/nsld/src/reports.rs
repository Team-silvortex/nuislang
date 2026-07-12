pub(crate) use super::reports_check::*;
pub(crate) use super::reports_final::*;
pub(crate) use super::reports_final_launcher::*;
pub(crate) use super::reports_link_inputs::*;
pub(crate) use super::reports_object::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldArtifactChainReport {
    pub(crate) manifest: String,
    pub(crate) output_dir: String,
    pub(crate) valid: bool,
    pub(crate) stage_count: usize,
    pub(crate) present_count: usize,
    pub(crate) required_count: usize,
    pub(crate) missing_required_count: usize,
    pub(crate) optional_present_count: usize,
    pub(crate) first_missing_required_stage: Option<String>,
    pub(crate) next_required_stage: Option<String>,
    pub(crate) suggested_command_id: Option<String>,
    pub(crate) suggested_command: Option<String>,
    pub(crate) suggested_command_resolved: Option<String>,
    pub(crate) suggested_command_reason: Option<String>,
    pub(crate) next_optional_stage: Option<String>,
    pub(crate) next_optional_command_id: Option<String>,
    pub(crate) next_optional_command: Option<String>,
    pub(crate) next_optional_command_resolved: Option<String>,
    pub(crate) next_optional_command_reason: Option<String>,
    pub(crate) advisory_command_id: Option<String>,
    pub(crate) advisory_command: Option<String>,
    pub(crate) advisory_command_resolved: Option<String>,
    pub(crate) advisory_command_reason: Option<String>,
    pub(crate) next_action_command_id: Option<String>,
    pub(crate) next_action_command: Option<String>,
    pub(crate) next_action_command_resolved: Option<String>,
    pub(crate) next_action_command_reason: Option<String>,
    pub(crate) next_action_source: Option<String>,
    pub(crate) next_action_available: bool,
    pub(crate) stages: Vec<NsldArtifactStageDiagnostic>,
    pub(crate) advisories: Vec<String>,
    pub(crate) issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldArtifactStageDiagnostic {
    pub(crate) order_index: usize,
    pub(crate) stage_id: String,
    pub(crate) file_name: String,
    pub(crate) path: String,
    pub(crate) required: bool,
    pub(crate) present: bool,
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
    pub(crate) lowering_plan_index_source: String,
    pub(crate) lowering_plan_index_available: bool,
    pub(crate) internal_contracts: Vec<String>,
    pub(crate) linker_contract_hash: String,
    pub(crate) link_inputs: Vec<NsldLinkInputDiagnostic>,
    pub(crate) link_input_count: usize,
    pub(crate) link_input_total_bytes: usize,
    pub(crate) link_input_table_hash: String,
    pub(crate) link_input_table_present: bool,
    pub(crate) link_input_table_valid: Option<bool>,
    pub(crate) prepared_artifact_chain_valid: bool,
    pub(crate) prepared_artifact_chain_issues: Vec<String>,
    pub(crate) container_metadata_table_hash: String,
    pub(crate) container_layout_hash: String,
    pub(crate) container_hash: String,
    pub(crate) payload_size_bytes: usize,
    pub(crate) payload_hash: String,
    pub(crate) container_loader_readiness: String,
    pub(crate) compatibility_domain_count: usize,
    pub(crate) compatibility_domain_table_hash: String,
    pub(crate) compatibility_domain_id: Option<String>,
    pub(crate) compatibility_domain_kind: Option<String>,
    pub(crate) compatibility_domain_paradigm: Option<String>,
    pub(crate) compatibility_domain_lifecycle_hook: Option<String>,
    pub(crate) compatibility_domain_abi_family: Option<String>,
    pub(crate) compatibility_domain_wrapper_policy: Option<String>,
    pub(crate) compatibility_domain_required: Option<bool>,
    pub(crate) object_image_relocation_lowering_valid: Option<bool>,
    pub(crate) object_image_relocation_lowering_rule_count: Option<usize>,
    pub(crate) object_image_relocation_lowering_rules: Vec<NsldRelocationLoweringRuleDiagnostic>,
    pub(crate) object_image_relocation_lowering_issues: Vec<String>,
    pub(crate) object_image_relocation_record_count: Option<usize>,
    pub(crate) object_image_relocation_record_table_hash: Option<String>,
    pub(crate) object_image_relocation_records: Vec<NsldObjectImageRelocationRecordDiagnostic>,
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
pub(crate) struct NsldClosureEmitReport {
    pub(crate) manifest: String,
    pub(crate) output_path: String,
    pub(crate) linker_contract_hash: String,
    pub(crate) closed: bool,
    pub(crate) internal_contract_count: usize,
    pub(crate) unresolved_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldClosureVerifyReport {
    pub(crate) manifest: String,
    pub(crate) input_path: String,
    pub(crate) valid: bool,
    pub(crate) expected_linker_contract_hash: String,
    pub(crate) expected_container_hash: String,
    pub(crate) expected_payload_size_bytes: usize,
    pub(crate) expected_payload_hash: String,
    pub(crate) expected_closed: bool,
    pub(crate) expected_internal_contract_count: usize,
    pub(crate) expected_unresolved_count: usize,
    pub(crate) actual_linker_contract_hash: Option<String>,
    pub(crate) actual_container_hash: Option<String>,
    pub(crate) actual_payload_size_bytes: Option<usize>,
    pub(crate) actual_payload_hash: Option<String>,
    pub(crate) actual_closed: Option<bool>,
    pub(crate) actual_internal_contract_count: Option<usize>,
    pub(crate) actual_unresolved_count: Option<usize>,
    pub(crate) issues: Vec<String>,
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
    pub(crate) object_plan_path: String,
    pub(crate) object_writer_input_path: String,
    pub(crate) object_byte_layout_path: String,
    pub(crate) object_file_layout_path: String,
    pub(crate) object_image_dry_run_path: String,
    pub(crate) object_image_dry_run_bytes_path: String,
    pub(crate) object_emit_blocked_path: String,
    pub(crate) object_output_path: String,
    pub(crate) object_writer_dry_run_path: String,
    pub(crate) container_plan_path: String,
    pub(crate) container_path: String,
    pub(crate) container_payload_path: String,
    pub(crate) closure_snapshot_path: String,
    pub(crate) final_stage_plan_path: String,
    pub(crate) final_executable_writer_input_path: String,
    pub(crate) final_executable_host_invoke_plan_path: String,
    pub(crate) final_executable_layout_plan_path: String,
    pub(crate) final_executable_image_dry_run_path: String,
    pub(crate) final_executable_image_dry_run_bytes_path: String,
    pub(crate) final_executable_blocked_path: String,
    pub(crate) final_executable_output_path: String,
    pub(crate) final_executable_launcher_manifest_path: String,
    pub(crate) final_executable_launcher_dry_run_path: String,
    pub(crate) link_input_count: usize,
    pub(crate) link_input_table_hash: String,
    pub(crate) unit_count: usize,
    pub(crate) unit_table_hash: String,
    pub(crate) bundle_id: String,
    pub(crate) bundle_hash: String,
    pub(crate) bundle_ready: bool,
    pub(crate) assemble_plan_hash: String,
    pub(crate) section_table_hash: String,
    pub(crate) object_plan_hash: String,
    pub(crate) object_emitted: bool,
    pub(crate) byte_layout_hash: String,
    pub(crate) file_layout_hash: String,
    pub(crate) object_image_hash: Option<String>,
    pub(crate) object_image_relocation_lowering_valid: bool,
    pub(crate) object_image_relocation_lowering_rule_count: usize,
    pub(crate) object_image_relocation_lowering_rules: Vec<NsldRelocationLoweringRuleDiagnostic>,
    pub(crate) object_image_relocation_lowering_issues: Vec<String>,
    pub(crate) object_image_relocation_record_count: usize,
    pub(crate) object_image_relocation_record_table_hash: String,
    pub(crate) object_image_relocation_records: Vec<NsldObjectImageRelocationRecordDiagnostic>,
    pub(crate) metadata_table_hash: String,
    pub(crate) compatibility_domain_count: Option<usize>,
    pub(crate) compatibility_domain_table_hash: Option<String>,
    pub(crate) compatibility_domain_id: Option<String>,
    pub(crate) compatibility_domain_kind: Option<String>,
    pub(crate) compatibility_domain_paradigm: Option<String>,
    pub(crate) compatibility_domain_lifecycle_hook: Option<String>,
    pub(crate) compatibility_domain_abi_family: Option<String>,
    pub(crate) compatibility_domain_wrapper_policy: Option<String>,
    pub(crate) compatibility_domain_required: Option<bool>,
    pub(crate) container_layout_hash: String,
    pub(crate) container_hash: String,
    pub(crate) payload_size_bytes: usize,
    pub(crate) payload_hash: String,
    pub(crate) final_stage_plan_ready: bool,
    pub(crate) final_stage_plan_hash: String,
    pub(crate) final_stage_plan_blocker_count: usize,
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
