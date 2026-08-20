#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldMachOMergedSectionPlan {
    pub(crate) section_id: String,
    pub(crate) segment_name: String,
    pub(crate) section_name: String,
    pub(crate) flags: u32,
    pub(crate) alignment: usize,
    pub(crate) output_offset: usize,
    pub(crate) size_bytes: usize,
    pub(crate) contribution_count: usize,
    pub(crate) zero_fill: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldMachOSectionPlacement {
    pub(crate) object_id: String,
    pub(crate) object_role: String,
    pub(crate) input_section_ordinal: usize,
    pub(crate) input_segment_name: String,
    pub(crate) input_section_name: String,
    pub(crate) output_section_id: String,
    pub(crate) output_offset: usize,
    pub(crate) output_section_offset: usize,
    pub(crate) size_bytes: usize,
    pub(crate) alignment: usize,
    pub(crate) zero_fill: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldMachOSymbolBinding {
    pub(crate) symbol: String,
    pub(crate) reference_object_id: String,
    pub(crate) reference_symbol_index: usize,
    pub(crate) status: String,
    pub(crate) target_object_id: Option<String>,
    pub(crate) target_symbol_index: Option<usize>,
    pub(crate) target_kind: Option<String>,
    pub(crate) target_section_id: Option<String>,
    pub(crate) target_output_offset: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldMachOPlacementBindingReport {
    pub(crate) contract: String,
    pub(crate) status: String,
    pub(crate) plan_hash: String,
    pub(crate) image_span_bytes: usize,
    pub(crate) merged_sections: Vec<NsldMachOMergedSectionPlan>,
    pub(crate) section_placements: Vec<NsldMachOSectionPlacement>,
    pub(crate) symbol_bindings: Vec<NsldMachOSymbolBinding>,
    pub(crate) internally_bound_symbol_count: usize,
    pub(crate) external_compatibility_symbol_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldMachOArm64RelocationApplication {
    pub(crate) relocation_id: String,
    pub(crate) object_id: String,
    pub(crate) object_role: String,
    pub(crate) input_section_ordinal: usize,
    pub(crate) source_section_id: String,
    pub(crate) source_offset: usize,
    pub(crate) source_output_offset: usize,
    pub(crate) width_bytes: usize,
    pub(crate) pc_relative: bool,
    pub(crate) external: bool,
    pub(crate) relocation_type: u32,
    pub(crate) relocation_kind: String,
    pub(crate) action_kind: String,
    pub(crate) target_symbol: Option<String>,
    pub(crate) target_symbol_index: Option<usize>,
    pub(crate) target_object_id: Option<String>,
    pub(crate) target_section_id: Option<String>,
    pub(crate) target_output_offset: Option<usize>,
    pub(crate) explicit_addend: Option<i64>,
    pub(crate) pair_relocation_id: Option<String>,
    pub(crate) resolver_status: String,
    pub(crate) application_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldMachOArm64RelocationApplicationReport {
    pub(crate) contract: String,
    pub(crate) status: String,
    pub(crate) plan_hash: String,
    pub(crate) placement_plan_hash: String,
    pub(crate) relocation_count: usize,
    pub(crate) registered_kind_count: usize,
    pub(crate) ready_application_count: usize,
    pub(crate) platform_structure_count: usize,
    pub(crate) external_compatibility_count: usize,
    pub(crate) metadata_record_count: usize,
    pub(crate) applications: Vec<NsldMachOArm64RelocationApplication>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldMachOMergedSectionImageAudit {
    pub(crate) section_id: String,
    pub(crate) output_offset: usize,
    pub(crate) size_bytes: usize,
    pub(crate) copied_bytes: usize,
    pub(crate) zero_fill_bytes: usize,
    pub(crate) content_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldMachOArm64PatchPreview {
    pub(crate) relocation_id: String,
    pub(crate) relocation_kind: String,
    pub(crate) source_output_offset: usize,
    pub(crate) width_bytes: usize,
    pub(crate) target_output_offset: usize,
    pub(crate) effective_addend: i64,
    pub(crate) source_bytes_hex: String,
    pub(crate) encoded_bytes_hex: String,
    pub(crate) source_bytes_hash: String,
    pub(crate) encoded_bytes_hash: String,
    pub(crate) audit_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldMachOArm64MaterializationPreviewReport {
    pub(crate) contract: String,
    pub(crate) status: String,
    pub(crate) placement_plan_hash: String,
    pub(crate) relocation_plan_hash: String,
    pub(crate) image_span_bytes: usize,
    pub(crate) copied_bytes: usize,
    pub(crate) zero_fill_bytes: usize,
    pub(crate) image_hash: String,
    pub(crate) section_audits: Vec<NsldMachOMergedSectionImageAudit>,
    pub(crate) planned_direct_count: usize,
    pub(crate) previewed_patch_count: usize,
    pub(crate) deferred_patch_count: usize,
    pub(crate) metadata_record_count: usize,
    pub(crate) patch_plan_hash: String,
    pub(crate) patches: Vec<NsldMachOArm64PatchPreview>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldExecutableFinalizerInputSummary {
    pub(crate) contract: String,
    pub(crate) status: String,
    pub(crate) object_count: usize,
    pub(crate) section_count: usize,
    pub(crate) symbol_count: usize,
    pub(crate) relocation_count: usize,
    pub(crate) defined_symbol_count: usize,
    pub(crate) undefined_symbol_count: usize,
    pub(crate) internally_resolved_symbol_count: usize,
    pub(crate) unresolved_external_symbol_count: usize,
    pub(crate) unresolved_external_symbols: Vec<String>,
    pub(crate) placement_binding: NsldMachOPlacementBindingReport,
    pub(crate) relocation_application: NsldMachOArm64RelocationApplicationReport,
    pub(crate) materialization_preview: NsldMachOArm64MaterializationPreviewReport,
}
