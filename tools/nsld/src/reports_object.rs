#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldObjectPlanReport {
    pub(crate) manifest: String,
    pub(crate) ready: bool,
    pub(crate) target_arch: String,
    pub(crate) target_os: String,
    pub(crate) object_format: String,
    pub(crate) calling_abi: String,
    pub(crate) clang_target: String,
    pub(crate) output_path: String,
    pub(crate) source_container_path: String,
    pub(crate) source_payload_path: String,
    pub(crate) section_count: usize,
    pub(crate) section_table_hash: String,
    pub(crate) object_plan_hash: String,
    pub(crate) object_layout_hash: String,
    pub(crate) relocation_seed_count: usize,
    pub(crate) relocation_seed_table_hash: String,
    pub(crate) writer_target_id: String,
    pub(crate) writer_status: String,
    pub(crate) unsupported_features: Vec<String>,
    pub(crate) emission_status: String,
    pub(crate) object_sections: Vec<NsldObjectSectionDiagnostic>,
    pub(crate) relocation_seeds: Vec<NsldObjectRelocationSeedDiagnostic>,
    pub(crate) blockers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldObjectSectionDiagnostic {
    pub(crate) order_index: usize,
    pub(crate) source_section_id: String,
    pub(crate) source_section_kind: String,
    pub(crate) object_section_name: String,
    pub(crate) object_section_role: String,
    pub(crate) source_path: String,
    pub(crate) source_hash: String,
    pub(crate) source_size_bytes: usize,
    pub(crate) payload_offset_seed: usize,
    pub(crate) file_offset_seed: usize,
    pub(crate) file_size_seed: usize,
    pub(crate) alignment: usize,
    pub(crate) required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldObjectRelocationSeedDiagnostic {
    pub(crate) order_index: usize,
    pub(crate) relocation_seed_id: String,
    pub(crate) relocation_seed_kind: String,
    pub(crate) source_section_id: String,
    pub(crate) source_offset_seed: usize,
    pub(crate) target_symbol: String,
    pub(crate) addend: isize,
    pub(crate) native_relocation_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldObjectPlanEmitReport {
    pub(crate) manifest: String,
    pub(crate) output_path: String,
    pub(crate) ready: bool,
    pub(crate) object_plan_hash: String,
    pub(crate) section_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldObjectPlanVerifyReport {
    pub(crate) manifest: String,
    pub(crate) input_path: String,
    pub(crate) valid: bool,
    pub(crate) expected_object_plan_hash: String,
    pub(crate) expected_section_count: usize,
    pub(crate) actual_object_plan_hash: Option<String>,
    pub(crate) actual_section_count: Option<usize>,
    pub(crate) issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldObjectWriterReadinessReport {
    pub(crate) manifest: String,
    pub(crate) writer_target_id: String,
    pub(crate) writer_status: String,
    pub(crate) object_plan_hash: String,
    pub(crate) section_count: usize,
    pub(crate) can_emit_object: bool,
    pub(crate) unsupported_features: Vec<String>,
    pub(crate) blockers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldObjectEmitReport {
    pub(crate) manifest: String,
    pub(crate) output_path: String,
    pub(crate) writer_input_path: String,
    pub(crate) blocked_report_path: String,
    pub(crate) writer_target_id: String,
    pub(crate) object_plan_hash: String,
    pub(crate) emitted: bool,
    pub(crate) can_emit_object: bool,
    pub(crate) blockers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldObjectWriterInputVerifyReport {
    pub(crate) manifest: String,
    pub(crate) input_path: String,
    pub(crate) valid: bool,
    pub(crate) expected_object_plan_hash: String,
    pub(crate) expected_object_layout_hash: String,
    pub(crate) expected_relocation_seed_table_hash: String,
    pub(crate) expected_section_count: usize,
    pub(crate) expected_relocation_seed_count: usize,
    pub(crate) actual_object_plan_hash: Option<String>,
    pub(crate) actual_object_layout_hash: Option<String>,
    pub(crate) actual_relocation_seed_table_hash: Option<String>,
    pub(crate) actual_section_count: Option<usize>,
    pub(crate) actual_relocation_seed_count: Option<usize>,
    pub(crate) issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldObjectWriterDryRunReport {
    pub(crate) manifest: String,
    pub(crate) writer_input_path: String,
    pub(crate) planned_output_path: String,
    pub(crate) writer_target_id: String,
    pub(crate) object_plan_hash: String,
    pub(crate) object_layout_hash: String,
    pub(crate) relocation_seed_table_hash: String,
    pub(crate) section_count: usize,
    pub(crate) relocation_seed_count: usize,
    pub(crate) writer_input_valid: bool,
    pub(crate) can_emit_object: bool,
    pub(crate) dry_run_ready: bool,
    pub(crate) blockers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldObjectWriterDryRunEmitReport {
    pub(crate) manifest: String,
    pub(crate) output_path: String,
    pub(crate) dry_run_ready: bool,
    pub(crate) object_plan_hash: String,
    pub(crate) section_count: usize,
    pub(crate) relocation_seed_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldObjectWriterDryRunVerifyReport {
    pub(crate) manifest: String,
    pub(crate) input_path: String,
    pub(crate) valid: bool,
    pub(crate) expected_object_plan_hash: String,
    pub(crate) expected_object_layout_hash: String,
    pub(crate) expected_relocation_seed_table_hash: String,
    pub(crate) expected_section_count: usize,
    pub(crate) expected_relocation_seed_count: usize,
    pub(crate) expected_dry_run_ready: bool,
    pub(crate) actual_object_plan_hash: Option<String>,
    pub(crate) actual_object_layout_hash: Option<String>,
    pub(crate) actual_relocation_seed_table_hash: Option<String>,
    pub(crate) actual_section_count: Option<usize>,
    pub(crate) actual_relocation_seed_count: Option<usize>,
    pub(crate) actual_dry_run_ready: Option<bool>,
    pub(crate) issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldObjectByteLayoutReport {
    pub(crate) manifest: String,
    pub(crate) output_path: String,
    pub(crate) object_plan_hash: String,
    pub(crate) object_layout_hash: String,
    pub(crate) byte_layout_hash: String,
    pub(crate) section_count: usize,
    pub(crate) total_size_bytes: usize,
    pub(crate) layout_ready: bool,
    pub(crate) sections: Vec<NsldObjectByteSectionDiagnostic>,
    pub(crate) blockers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldObjectByteSectionDiagnostic {
    pub(crate) order_index: usize,
    pub(crate) source_section_id: String,
    pub(crate) object_section_name: String,
    pub(crate) file_offset: usize,
    pub(crate) size_bytes: usize,
    pub(crate) alignment: usize,
    pub(crate) source_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldObjectByteLayoutEmitReport {
    pub(crate) manifest: String,
    pub(crate) output_path: String,
    pub(crate) layout_ready: bool,
    pub(crate) byte_layout_hash: String,
    pub(crate) section_count: usize,
    pub(crate) total_size_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldObjectByteLayoutVerifyReport {
    pub(crate) manifest: String,
    pub(crate) input_path: String,
    pub(crate) valid: bool,
    pub(crate) expected_byte_layout_hash: String,
    pub(crate) expected_section_count: usize,
    pub(crate) expected_total_size_bytes: usize,
    pub(crate) actual_byte_layout_hash: Option<String>,
    pub(crate) actual_section_count: Option<usize>,
    pub(crate) actual_total_size_bytes: Option<usize>,
    pub(crate) issues: Vec<String>,
}
