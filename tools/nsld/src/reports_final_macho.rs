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
pub(crate) struct NsldMachOArm64AppliedPatchAudit {
    pub(crate) relocation_id: String,
    pub(crate) source_output_offset: usize,
    pub(crate) width_bytes: usize,
    pub(crate) source_bytes_hash: String,
    pub(crate) encoded_bytes_hash: String,
    pub(crate) post_write_bytes_hash: String,
    pub(crate) preview_audit_hash: String,
    pub(crate) write_audit_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldMachOArm64PatchApplicationReport {
    pub(crate) contract: String,
    pub(crate) status: String,
    pub(crate) placement_plan_hash: String,
    pub(crate) relocation_plan_hash: String,
    pub(crate) patch_plan_hash: String,
    pub(crate) original_image_hash: String,
    pub(crate) applied_image_hash: String,
    pub(crate) image_span_bytes: usize,
    pub(crate) expected_patch_count: usize,
    pub(crate) applied_patch_count: usize,
    pub(crate) deferred_patch_count: usize,
    pub(crate) write_once_span_count: usize,
    pub(crate) application_ledger_hash: String,
    pub(crate) patches: Vec<NsldMachOArm64AppliedPatchAudit>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldMachOArm64PlatformTargetPlan {
    pub(crate) structure_id: String,
    pub(crate) target_key: String,
    pub(crate) target_symbol: String,
    pub(crate) resolver_status: String,
    pub(crate) target_object_id: Option<String>,
    pub(crate) target_section_id: Option<String>,
    pub(crate) target_output_offset: Option<usize>,
    pub(crate) got_slot_index: Option<usize>,
    pub(crate) got_output_offset: Option<usize>,
    pub(crate) stub_slot_index: Option<usize>,
    pub(crate) stub_output_offset: Option<usize>,
    pub(crate) relocation_ids: Vec<String>,
    pub(crate) audit_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldMachOArm64PlatformRelocationBinding {
    pub(crate) relocation_id: String,
    pub(crate) relocation_kind: String,
    pub(crate) action_kind: String,
    pub(crate) source_output_offset: usize,
    pub(crate) width_bytes: usize,
    pub(crate) structure_id: String,
    pub(crate) patch_target_kind: String,
    pub(crate) patch_target_output_offset: usize,
    pub(crate) audit_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldMachOArm64PlatformStructurePlanReport {
    pub(crate) contract: String,
    pub(crate) status: String,
    pub(crate) placement_plan_hash: String,
    pub(crate) relocation_plan_hash: String,
    pub(crate) patch_application_ledger_hash: String,
    pub(crate) applied_image_hash: String,
    pub(crate) base_image_span_bytes: usize,
    pub(crate) planned_image_span_bytes: usize,
    pub(crate) registered_rule_count: usize,
    pub(crate) deferred_relocation_count: usize,
    pub(crate) target_count: usize,
    pub(crate) stub_region_offset: usize,
    pub(crate) stub_region_bytes: usize,
    pub(crate) stub_entry_size: usize,
    pub(crate) stub_alignment: usize,
    pub(crate) stub_entry_count: usize,
    pub(crate) got_region_offset: usize,
    pub(crate) got_region_bytes: usize,
    pub(crate) got_entry_size: usize,
    pub(crate) got_alignment: usize,
    pub(crate) got_entry_count: usize,
    pub(crate) plan_hash: String,
    pub(crate) targets: Vec<NsldMachOArm64PlatformTargetPlan>,
    pub(crate) relocation_bindings: Vec<NsldMachOArm64PlatformRelocationBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldMachOArm64PlatformWriteAudit {
    pub(crate) write_id: String,
    pub(crate) structure_id: String,
    pub(crate) write_kind: String,
    pub(crate) target_symbol: String,
    pub(crate) output_offset: usize,
    pub(crate) width_bytes: usize,
    pub(crate) encoded_bytes_hex: String,
    pub(crate) encoded_bytes_hash: String,
    pub(crate) write_audit_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldMachOArm64PlatformPatchAudit {
    pub(crate) relocation_id: String,
    pub(crate) relocation_kind: String,
    pub(crate) source_output_offset: usize,
    pub(crate) width_bytes: usize,
    pub(crate) patch_target_output_offset: usize,
    pub(crate) effective_addend: i64,
    pub(crate) source_bytes_hex: String,
    pub(crate) encoded_bytes_hex: String,
    pub(crate) source_bytes_hash: String,
    pub(crate) encoded_bytes_hash: String,
    pub(crate) binding_audit_hash: String,
    pub(crate) write_audit_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldMachOArm64PlatformBindRecord {
    pub(crate) bind_id: String,
    pub(crate) structure_id: String,
    pub(crate) target_key: String,
    pub(crate) target_symbol: String,
    pub(crate) got_output_offset: usize,
    pub(crate) width_bytes: usize,
    pub(crate) placeholder_bytes_hash: String,
    pub(crate) status: String,
    pub(crate) audit_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldMachOArm64PlatformPatchApplicationReport {
    pub(crate) contract: String,
    pub(crate) status: String,
    pub(crate) placement_plan_hash: String,
    pub(crate) relocation_plan_hash: String,
    pub(crate) direct_patch_application_ledger_hash: String,
    pub(crate) platform_structure_plan_hash: String,
    pub(crate) base_applied_image_hash: String,
    pub(crate) platform_image_hash: String,
    pub(crate) base_image_span_bytes: usize,
    pub(crate) platform_image_span_bytes: usize,
    pub(crate) expected_deferred_patch_count: usize,
    pub(crate) applied_deferred_patch_count: usize,
    pub(crate) stub_write_count: usize,
    pub(crate) got_write_count: usize,
    pub(crate) unresolved_bind_count: usize,
    pub(crate) write_once_span_count: usize,
    pub(crate) application_ledger_hash: String,
    pub(crate) structure_writes: Vec<NsldMachOArm64PlatformWriteAudit>,
    pub(crate) patches: Vec<NsldMachOArm64PlatformPatchAudit>,
    pub(crate) bind_records: Vec<NsldMachOArm64PlatformBindRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldMachOArm64ShellSectionPlan {
    pub(crate) section_id: String,
    pub(crate) source_kind: String,
    pub(crate) source_id: String,
    pub(crate) segment_name: String,
    pub(crate) section_name: String,
    pub(crate) section_ordinal: usize,
    pub(crate) source_image_offset: Option<usize>,
    pub(crate) source_size_bytes: usize,
    pub(crate) alignment: usize,
    pub(crate) file_offset: Option<usize>,
    pub(crate) file_size_bytes: usize,
    pub(crate) vm_address: u64,
    pub(crate) vm_size_bytes: usize,
    pub(crate) flags: u32,
    pub(crate) reserved1: u32,
    pub(crate) reserved2: u32,
    pub(crate) audit_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldMachOArm64ShellSegmentPlan {
    pub(crate) segment_id: String,
    pub(crate) segment_name: String,
    pub(crate) segment_index: usize,
    pub(crate) file_offset: usize,
    pub(crate) file_size_bytes: usize,
    pub(crate) vm_address: u64,
    pub(crate) vm_size_bytes: usize,
    pub(crate) max_protection: u32,
    pub(crate) initial_protection: u32,
    pub(crate) section_ids: Vec<String>,
    pub(crate) audit_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldMachOArm64ShellSymbolPlan {
    pub(crate) symbol_id: String,
    pub(crate) name: String,
    pub(crate) record_kind: String,
    pub(crate) object_id: Option<String>,
    pub(crate) source_symbol_index: Option<usize>,
    pub(crate) shell_section_id: Option<String>,
    pub(crate) source_image_offset: Option<usize>,
    pub(crate) vm_address: Option<u64>,
    pub(crate) symbol_table_index: usize,
    pub(crate) string_table_offset: usize,
    pub(crate) dylib_ordinal: Option<usize>,
    pub(crate) audit_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldMachOArm64ShellIndirectSymbolPlan {
    pub(crate) indirect_id: String,
    pub(crate) shell_section_id: String,
    pub(crate) slot_index: usize,
    pub(crate) target_symbol: String,
    pub(crate) symbol_table_index: Option<usize>,
    pub(crate) marker: Option<String>,
    pub(crate) audit_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldMachOArm64ShellBindPlan {
    pub(crate) bind_id: String,
    pub(crate) source_bind_id: String,
    pub(crate) target_symbol: String,
    pub(crate) dylib_ordinal: usize,
    pub(crate) got_source_image_offset: usize,
    pub(crate) shell_section_id: String,
    pub(crate) segment_index: usize,
    pub(crate) segment_offset: usize,
    pub(crate) file_offset: usize,
    pub(crate) vm_address: u64,
    pub(crate) encoded_size_bytes: usize,
    pub(crate) audit_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldMachOArm64ShellRebasePlan {
    pub(crate) rebase_id: String,
    pub(crate) structure_id: String,
    pub(crate) target_symbol: String,
    pub(crate) got_source_image_offset: usize,
    pub(crate) target_source_image_offset: usize,
    pub(crate) shell_section_id: String,
    pub(crate) segment_index: usize,
    pub(crate) segment_offset: usize,
    pub(crate) file_offset: usize,
    pub(crate) vm_address: u64,
    pub(crate) target_vm_address: u64,
    pub(crate) encoded_size_bytes: usize,
    pub(crate) audit_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldMachOArm64ShellLoadCommandPlan {
    pub(crate) command_id: String,
    pub(crate) command_kind: String,
    pub(crate) command_value: u32,
    pub(crate) command_offset: usize,
    pub(crate) command_size_bytes: usize,
    pub(crate) segment_id: Option<String>,
    pub(crate) status: String,
    pub(crate) audit_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldMachOArm64ShellLayoutPlanReport {
    pub(crate) contract: String,
    pub(crate) status: String,
    pub(crate) object_linkage_hash: String,
    pub(crate) placement_plan_hash: String,
    pub(crate) platform_structure_plan_hash: String,
    pub(crate) platform_application_ledger_hash: String,
    pub(crate) platform_image_hash: String,
    pub(crate) page_size: usize,
    pub(crate) image_base_vm_address: u64,
    pub(crate) header_size_bytes: usize,
    pub(crate) load_command_count: usize,
    pub(crate) load_command_size_bytes: usize,
    pub(crate) first_content_file_offset: usize,
    pub(crate) entry_rule_id: String,
    pub(crate) entry_symbol: String,
    pub(crate) entry_source_image_offset: usize,
    pub(crate) entry_file_offset: usize,
    pub(crate) entry_vm_address: u64,
    pub(crate) segment_count: usize,
    pub(crate) section_count: usize,
    pub(crate) defined_symbol_count: usize,
    pub(crate) undefined_symbol_count: usize,
    pub(crate) symbol_table_offset: usize,
    pub(crate) symbol_table_bytes: usize,
    pub(crate) indirect_symbol_table_offset: usize,
    pub(crate) indirect_symbol_count: usize,
    pub(crate) indirect_symbol_table_bytes: usize,
    pub(crate) string_table_offset: usize,
    pub(crate) string_table_bytes: usize,
    pub(crate) rebase_stream_offset: usize,
    pub(crate) rebase_stream_bytes: usize,
    pub(crate) bind_stream_offset: usize,
    pub(crate) bind_stream_bytes: usize,
    pub(crate) linkedit_file_offset: usize,
    pub(crate) linkedit_bytes: usize,
    pub(crate) code_signature_file_offset: usize,
    pub(crate) code_signature_status: String,
    pub(crate) required_address_rewrite_count: usize,
    pub(crate) planned_file_span_bytes: usize,
    pub(crate) plan_hash: String,
    pub(crate) segments: Vec<NsldMachOArm64ShellSegmentPlan>,
    pub(crate) sections: Vec<NsldMachOArm64ShellSectionPlan>,
    pub(crate) symbols: Vec<NsldMachOArm64ShellSymbolPlan>,
    pub(crate) indirect_symbols: Vec<NsldMachOArm64ShellIndirectSymbolPlan>,
    pub(crate) binds: Vec<NsldMachOArm64ShellBindPlan>,
    pub(crate) rebases: Vec<NsldMachOArm64ShellRebasePlan>,
    pub(crate) load_commands: Vec<NsldMachOArm64ShellLoadCommandPlan>,
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
    pub(crate) patch_application: NsldMachOArm64PatchApplicationReport,
    pub(crate) platform_structure_plan: NsldMachOArm64PlatformStructurePlanReport,
    pub(crate) platform_patch_application: NsldMachOArm64PlatformPatchApplicationReport,
    pub(crate) shell_layout_plan: NsldMachOArm64ShellLayoutPlanReport,
}
