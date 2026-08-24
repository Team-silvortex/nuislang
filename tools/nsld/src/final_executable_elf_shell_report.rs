use super::version::{
    append_version_plan_canonical, ElfAmd64ShellVersionNeedPlan, ElfAmd64ShellVersionSymbolPlan,
};
use std::fmt::Write as _;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64ShellSectionPlan {
    pub(crate) section_id: String,
    pub(crate) section_index: usize,
    pub(crate) section_name: String,
    pub(crate) section_name_offset: usize,
    pub(crate) source_kind: String,
    pub(crate) source_id: String,
    pub(crate) section_type: u32,
    pub(crate) flags: u64,
    pub(crate) alignment: usize,
    pub(crate) entry_size: usize,
    pub(crate) link_section_index: usize,
    pub(crate) info_section_index: usize,
    pub(crate) source_image_offset: Option<usize>,
    pub(crate) source_size_bytes: usize,
    pub(crate) file_offset: usize,
    pub(crate) file_size_bytes: usize,
    pub(crate) virtual_address: u64,
    pub(crate) memory_size_bytes: usize,
    pub(crate) load_segment_id: Option<String>,
    pub(crate) audit_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64ShellProgramHeaderPlan {
    pub(crate) program_header_id: String,
    pub(crate) program_header_index: usize,
    pub(crate) program_kind: String,
    pub(crate) program_type: u32,
    pub(crate) permission_class: String,
    pub(crate) flags: u32,
    pub(crate) file_offset: usize,
    pub(crate) virtual_address: u64,
    pub(crate) file_size_bytes: usize,
    pub(crate) memory_size_bytes: usize,
    pub(crate) alignment: usize,
    pub(crate) section_ids: Vec<String>,
    pub(crate) audit_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64ShellDynamicEntryPlan {
    pub(crate) dynamic_entry_id: String,
    pub(crate) dynamic_entry_index: usize,
    pub(crate) tag_name: String,
    pub(crate) tag: i64,
    pub(crate) value_kind: String,
    pub(crate) value: u64,
    pub(crate) referenced_section_id: Option<String>,
    pub(crate) audit_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64ShellNeededLibraryPlan {
    pub(crate) needed_id: String,
    pub(crate) dependency_audit_hash: String,
    pub(crate) dependency_identity: String,
    pub(crate) needed_name: String,
    pub(crate) dynamic_string_offset: usize,
    pub(crate) audit_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64ShellLayoutPlanReport {
    pub(crate) contract: &'static str,
    pub(crate) status: String,
    pub(crate) plan_hash: String,
    pub(crate) object_linkage_hash: String,
    pub(crate) placement_plan_hash: String,
    pub(crate) relocation_plan_hash: String,
    pub(crate) platform_structure_plan_hash: String,
    pub(crate) platform_application_ledger_hash: String,
    pub(crate) platform_image_hash: String,
    pub(crate) image_base: u64,
    pub(crate) page_size: usize,
    pub(crate) elf_header_file_offset: usize,
    pub(crate) elf_header_size_bytes: usize,
    pub(crate) program_header_table_file_offset: usize,
    pub(crate) program_header_entry_size_bytes: usize,
    pub(crate) program_header_count: usize,
    pub(crate) program_header_table_bytes: usize,
    pub(crate) section_header_table_file_offset: usize,
    pub(crate) section_header_entry_size_bytes: usize,
    pub(crate) section_header_count: usize,
    pub(crate) section_name_table_section_index: usize,
    pub(crate) section_name_table_file_offset: usize,
    pub(crate) section_name_table_bytes: usize,
    pub(crate) entry_rule_id: String,
    pub(crate) entry_symbol: String,
    pub(crate) entry_source_object_id: String,
    pub(crate) entry_source_symbol_index: usize,
    pub(crate) entry_source_image_offset: usize,
    pub(crate) entry_section_id: String,
    pub(crate) entry_file_offset: usize,
    pub(crate) entry_virtual_address: u64,
    pub(crate) applied_file_span_bytes: usize,
    pub(crate) applied_memory_span_bytes: usize,
    pub(crate) planned_file_span_bytes: usize,
    pub(crate) planned_memory_span_bytes: usize,
    pub(crate) load_segment_count: usize,
    pub(crate) dynamic_table_file_offset: Option<usize>,
    pub(crate) dynamic_table_virtual_address: Option<u64>,
    pub(crate) dynamic_table_entry_size_bytes: usize,
    pub(crate) dynamic_table_entry_count: usize,
    pub(crate) dynamic_table_bytes: usize,
    pub(crate) dynamic_dependency_plan_hash: Option<String>,
    pub(crate) interpreter_identity: Option<String>,
    pub(crate) interpreter_path: Option<String>,
    pub(crate) interpreter_file_offset: Option<usize>,
    pub(crate) interpreter_virtual_address: Option<u64>,
    pub(crate) interpreter_bytes: usize,
    pub(crate) dynamic_string_source_image_offset: Option<usize>,
    pub(crate) dynamic_string_source_bytes: usize,
    pub(crate) needed_libraries: Vec<ElfAmd64ShellNeededLibraryPlan>,
    pub(crate) version_symbol_table_file_offset: Option<usize>,
    pub(crate) version_symbol_table_virtual_address: Option<u64>,
    pub(crate) version_symbol_table_bytes: usize,
    pub(crate) version_need_table_file_offset: Option<usize>,
    pub(crate) version_need_table_virtual_address: Option<u64>,
    pub(crate) version_need_table_bytes: usize,
    pub(crate) version_symbols: Vec<ElfAmd64ShellVersionSymbolPlan>,
    pub(crate) version_needs: Vec<ElfAmd64ShellVersionNeedPlan>,
    pub(crate) program_headers: Vec<ElfAmd64ShellProgramHeaderPlan>,
    pub(crate) sections: Vec<ElfAmd64ShellSectionPlan>,
    pub(crate) dynamic_entries: Vec<ElfAmd64ShellDynamicEntryPlan>,
}

impl ElfAmd64ShellLayoutPlanReport {
    pub(crate) fn canonical_plan(&self) -> String {
        let mut out = String::new();
        for value in [
            self.contract,
            &self.status,
            &self.object_linkage_hash,
            &self.placement_plan_hash,
            &self.relocation_plan_hash,
            &self.platform_structure_plan_hash,
            &self.platform_application_ledger_hash,
            &self.platform_image_hash,
            &self.entry_rule_id,
            &self.entry_symbol,
            &self.entry_source_object_id,
            &self.entry_section_id,
        ] {
            append_text(&mut out, value);
        }
        writeln!(
            out,
            "header={}|{}|{}|{}|{}|{}|{}|{}",
            self.image_base,
            self.page_size,
            self.elf_header_file_offset,
            self.elf_header_size_bytes,
            self.program_header_table_file_offset,
            self.program_header_entry_size_bytes,
            self.program_header_count,
            self.program_header_table_bytes
        )
        .unwrap();
        writeln!(
            out,
            "sections={}|{}|{}|{}|{}",
            self.section_header_table_file_offset,
            self.section_header_entry_size_bytes,
            self.section_header_count,
            self.section_name_table_section_index,
            self.section_name_table_file_offset
        )
        .unwrap();
        writeln!(
            out,
            "entry={}|{}|{}|{}",
            self.entry_source_symbol_index,
            self.entry_source_image_offset,
            self.entry_file_offset,
            self.entry_virtual_address
        )
        .unwrap();
        writeln!(
            out,
            "spans={}|{}|{}|{}|{}|{}",
            self.applied_file_span_bytes,
            self.applied_memory_span_bytes,
            self.planned_file_span_bytes,
            self.planned_memory_span_bytes,
            self.load_segment_count,
            self.section_name_table_bytes
        )
        .unwrap();
        writeln!(
            out,
            "dynamic={}|{}|{}|{}|{}",
            optional_usize(self.dynamic_table_file_offset),
            optional_u64(self.dynamic_table_virtual_address),
            self.dynamic_table_entry_size_bytes,
            self.dynamic_table_entry_count,
            self.dynamic_table_bytes
        )
        .unwrap();
        for value in [
            self.dynamic_dependency_plan_hash
                .as_deref()
                .unwrap_or("none"),
            self.interpreter_identity.as_deref().unwrap_or("none"),
            self.interpreter_path.as_deref().unwrap_or("none"),
        ] {
            append_text(&mut out, value);
        }
        writeln!(
            out,
            "dynamic-loader={}|{}|{}|{}|{}|{}|{}",
            optional_usize(self.interpreter_file_offset),
            optional_u64(self.interpreter_virtual_address),
            self.interpreter_bytes,
            optional_usize(self.dynamic_string_source_image_offset),
            self.dynamic_string_source_bytes,
            self.needed_libraries.len(),
            self.dynamic_dependency_plan_hash.is_some()
        )
        .unwrap();
        for needed in &self.needed_libraries {
            append_text(&mut out, &needed.needed_id);
            append_text(&mut out, &needed.audit_hash);
        }
        writeln!(
            out,
            "versions={}|{}|{}|{}|{}|{}|{}|{}",
            optional_usize(self.version_symbol_table_file_offset),
            optional_u64(self.version_symbol_table_virtual_address),
            self.version_symbol_table_bytes,
            optional_usize(self.version_need_table_file_offset),
            optional_u64(self.version_need_table_virtual_address),
            self.version_need_table_bytes,
            self.version_symbols.len(),
            self.version_needs.len()
        )
        .unwrap();
        append_version_plan_canonical(&mut out, &self.version_symbols, &self.version_needs);
        for header in &self.program_headers {
            append_text(&mut out, &header.program_header_id);
            append_text(&mut out, &header.audit_hash);
        }
        for section in &self.sections {
            append_text(&mut out, &section.section_id);
            append_text(&mut out, &section.audit_hash);
        }
        for entry in &self.dynamic_entries {
            append_text(&mut out, &entry.dynamic_entry_id);
            append_text(&mut out, &entry.audit_hash);
        }
        out
    }
}

pub(super) fn needed_library_audit_hash(
    dependency_plan_hash: &str,
    needed: &ElfAmd64ShellNeededLibraryPlan,
) -> String {
    let mut out = String::new();
    append_text(&mut out, dependency_plan_hash);
    append_text(&mut out, &needed.needed_id);
    append_text(&mut out, &needed.dependency_audit_hash);
    append_text(&mut out, &needed.dependency_identity);
    append_text(&mut out, &needed.needed_name);
    writeln!(out, "offset={}", needed.dynamic_string_offset).unwrap();
    crate::fnv1a64_hex(out.as_bytes())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64ShellImageWriteAudit {
    pub(crate) write_id: String,
    pub(crate) write_kind: String,
    pub(crate) file_offset: usize,
    pub(crate) width_bytes: usize,
    pub(crate) source_bytes_hash: String,
    pub(crate) encoded_bytes_hash: String,
    pub(crate) post_write_bytes_hash: String,
    pub(crate) status: String,
    pub(crate) audit_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64ShellSourcePreservationAudit {
    pub(crate) preservation_id: String,
    pub(crate) section_id: String,
    pub(crate) section_name: String,
    pub(crate) preservation_kind: String,
    pub(crate) source_image_offset: usize,
    pub(crate) source_size_bytes: usize,
    pub(crate) result_file_offset: Option<usize>,
    pub(crate) result_size_bytes: usize,
    pub(crate) source_bytes_hash: String,
    pub(crate) result_bytes_hash: String,
    pub(crate) status: String,
    pub(crate) audit_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64ShellImageSerializationReport {
    pub(crate) contract: &'static str,
    pub(crate) status: String,
    pub(crate) serialization_ledger_hash: String,
    pub(crate) shell_layout_plan_hash: String,
    pub(crate) platform_application_ledger_hash: String,
    pub(crate) source_file_image_hash: String,
    pub(crate) source_memory_image_hash: String,
    pub(crate) shell_image_hash: String,
    pub(crate) shell_image_span_bytes: usize,
    pub(crate) copied_platform_file_bytes: usize,
    pub(crate) preserved_platform_file_bytes: usize,
    pub(crate) header_bytes: usize,
    pub(crate) program_header_bytes: usize,
    pub(crate) dynamic_table_bytes: usize,
    pub(crate) section_name_table_bytes: usize,
    pub(crate) section_header_bytes: usize,
    pub(crate) expected_shell_write_count: usize,
    pub(crate) applied_shell_write_count: usize,
    pub(crate) source_preservation_count: usize,
    pub(crate) file_backed_source_span_count: usize,
    pub(crate) zero_fill_source_span_count: usize,
    pub(crate) preserved_file_source_bytes: usize,
    pub(crate) preserved_zero_fill_bytes: usize,
    pub(crate) publication_status: String,
    pub(crate) writes: Vec<ElfAmd64ShellImageWriteAudit>,
    pub(crate) source_preservations: Vec<ElfAmd64ShellSourcePreservationAudit>,
}

impl ElfAmd64ShellImageSerializationReport {
    pub(crate) fn canonical_ledger(&self) -> String {
        let mut out = String::new();
        for value in [
            self.contract,
            &self.status,
            &self.shell_layout_plan_hash,
            &self.platform_application_ledger_hash,
            &self.source_file_image_hash,
            &self.source_memory_image_hash,
            &self.shell_image_hash,
            &self.publication_status,
        ] {
            append_text(&mut out, value);
        }
        writeln!(
            out,
            "spans={}|{}|{}|{}|{}|{}|{}|{}|{}",
            self.shell_image_span_bytes,
            self.copied_platform_file_bytes,
            self.preserved_platform_file_bytes,
            self.header_bytes,
            self.program_header_bytes,
            self.dynamic_table_bytes,
            self.section_name_table_bytes,
            self.section_header_bytes,
            self.preserved_file_source_bytes
        )
        .unwrap();
        writeln!(
            out,
            "counts={}|{}|{}|{}|{}|{}",
            self.expected_shell_write_count,
            self.applied_shell_write_count,
            self.source_preservation_count,
            self.file_backed_source_span_count,
            self.zero_fill_source_span_count,
            self.preserved_zero_fill_bytes
        )
        .unwrap();
        for write in &self.writes {
            append_text(&mut out, &write.write_id);
            append_text(&mut out, &write.audit_hash);
        }
        for preservation in &self.source_preservations {
            append_text(&mut out, &preservation.preservation_id);
            append_text(&mut out, &preservation.audit_hash);
        }
        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64ShellImageTableValidation {
    pub(crate) table_id: String,
    pub(crate) table_kind: String,
    pub(crate) file_offset: usize,
    pub(crate) width_bytes: usize,
    pub(crate) expected_record_count: usize,
    pub(crate) verified_record_count: usize,
    pub(crate) bytes_hash: String,
    pub(crate) status: String,
    pub(crate) audit_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64ShellImageSourceValidation {
    pub(crate) validation_id: String,
    pub(crate) section_id: String,
    pub(crate) section_name: String,
    pub(crate) preservation_kind: String,
    pub(crate) source_image_offset: usize,
    pub(crate) source_size_bytes: usize,
    pub(crate) result_file_offset: Option<usize>,
    pub(crate) result_size_bytes: usize,
    pub(crate) source_bytes_hash: String,
    pub(crate) result_bytes_hash: String,
    pub(crate) serialization_audit_hash: String,
    pub(crate) status: String,
    pub(crate) audit_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64ShellImageValidationReport {
    pub(crate) contract: &'static str,
    pub(crate) status: String,
    pub(crate) validation_ledger_hash: String,
    pub(crate) shell_layout_plan_hash: String,
    pub(crate) serialization_ledger_hash: String,
    pub(crate) platform_application_ledger_hash: String,
    pub(crate) shell_image_hash: String,
    pub(crate) shell_image_span_bytes: usize,
    pub(crate) header_valid: bool,
    pub(crate) expected_table_count: usize,
    pub(crate) verified_table_count: usize,
    pub(crate) program_header_count: usize,
    pub(crate) load_segment_count: usize,
    pub(crate) dynamic_segment_count: usize,
    pub(crate) dynamic_entry_count: usize,
    pub(crate) interpreter_segment_count: usize,
    pub(crate) interpreter_path: Option<String>,
    pub(crate) needed_libraries: Vec<String>,
    pub(crate) version_symbol_indexes: Vec<u16>,
    pub(crate) version_requirements: Vec<String>,
    pub(crate) section_header_count: usize,
    pub(crate) section_name_count: usize,
    pub(crate) entry_program_header_index: usize,
    pub(crate) expected_shell_write_count: usize,
    pub(crate) verified_shell_write_count: usize,
    pub(crate) expected_source_validation_count: usize,
    pub(crate) verified_source_validation_count: usize,
    pub(crate) preserved_platform_file_bytes: usize,
    pub(crate) unexplained_platform_change_count: usize,
    pub(crate) publication_eligibility_contract: &'static str,
    pub(crate) publication_eligibility_status: String,
    pub(crate) publication_eligible: bool,
    pub(crate) publication_blockers: Vec<String>,
    pub(crate) tables: Vec<ElfAmd64ShellImageTableValidation>,
    pub(crate) sources: Vec<ElfAmd64ShellImageSourceValidation>,
}

impl ElfAmd64ShellImageValidationReport {
    pub(crate) fn canonical_ledger(&self) -> String {
        let mut out = String::new();
        for value in [
            self.contract,
            &self.status,
            &self.shell_layout_plan_hash,
            &self.serialization_ledger_hash,
            &self.platform_application_ledger_hash,
            &self.shell_image_hash,
            self.interpreter_path.as_deref().unwrap_or("none"),
            self.publication_eligibility_contract,
            &self.publication_eligibility_status,
        ] {
            append_text(&mut out, value);
        }
        writeln!(
            out,
            "shape={}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
            self.shell_image_span_bytes,
            self.header_valid,
            self.program_header_count,
            self.load_segment_count,
            self.dynamic_segment_count,
            self.dynamic_entry_count,
            self.interpreter_segment_count,
            self.needed_libraries.len(),
            self.section_header_count,
            self.section_name_count,
            self.entry_program_header_index,
            self.preserved_platform_file_bytes
        )
        .unwrap();
        for needed in &self.needed_libraries {
            append_text(&mut out, needed);
        }
        writeln!(
            out,
            "versions={}|{}",
            self.version_symbol_indexes.len(),
            self.version_requirements.len()
        )
        .unwrap();
        for index in &self.version_symbol_indexes {
            writeln!(out, "version-index={index}").unwrap();
        }
        for requirement in &self.version_requirements {
            append_text(&mut out, requirement);
        }
        writeln!(
            out,
            "coverage={}|{}|{}|{}|{}|{}|{}|{}",
            self.expected_table_count,
            self.verified_table_count,
            self.expected_shell_write_count,
            self.verified_shell_write_count,
            self.expected_source_validation_count,
            self.verified_source_validation_count,
            self.unexplained_platform_change_count,
            self.publication_eligible
        )
        .unwrap();
        for blocker in &self.publication_blockers {
            append_text(&mut out, blocker);
        }
        for table in &self.tables {
            append_text(&mut out, &table.table_id);
            append_text(&mut out, &table.audit_hash);
        }
        for source in &self.sources {
            append_text(&mut out, &source.validation_id);
            append_text(&mut out, &source.audit_hash);
        }
        out
    }
}

pub(super) fn shell_image_table_validation_audit_hash(
    plan_hash: &str,
    serialization_ledger_hash: &str,
    image_hash: &str,
    table: &ElfAmd64ShellImageTableValidation,
) -> String {
    let mut out = String::new();
    append_text(&mut out, plan_hash);
    append_text(&mut out, serialization_ledger_hash);
    append_text(&mut out, image_hash);
    append_text(&mut out, &table.table_id);
    append_text(&mut out, &table.table_kind);
    append_text(&mut out, &table.bytes_hash);
    append_text(&mut out, &table.status);
    writeln!(
        out,
        "table={}|{}|{}|{}",
        table.file_offset,
        table.width_bytes,
        table.expected_record_count,
        table.verified_record_count
    )
    .unwrap();
    crate::fnv1a64_hex(out.as_bytes())
}

pub(super) fn shell_image_source_validation_audit_hash(
    plan_hash: &str,
    serialization_ledger_hash: &str,
    image_hash: &str,
    source: &ElfAmd64ShellImageSourceValidation,
) -> String {
    let mut out = String::new();
    append_text(&mut out, plan_hash);
    append_text(&mut out, serialization_ledger_hash);
    append_text(&mut out, image_hash);
    append_text(&mut out, &source.validation_id);
    append_text(&mut out, &source.section_id);
    append_text(&mut out, &source.section_name);
    append_text(&mut out, &source.preservation_kind);
    append_text(&mut out, &source.source_bytes_hash);
    append_text(&mut out, &source.result_bytes_hash);
    append_text(&mut out, &source.serialization_audit_hash);
    append_text(&mut out, &source.status);
    writeln!(
        out,
        "source={}|{}|{}|{}",
        source.source_image_offset,
        source.source_size_bytes,
        optional_usize(source.result_file_offset),
        source.result_size_bytes
    )
    .unwrap();
    crate::fnv1a64_hex(out.as_bytes())
}

pub(super) fn shell_image_write_audit_hash(
    plan_hash: &str,
    application_ledger_hash: &str,
    write: &ElfAmd64ShellImageWriteAudit,
) -> String {
    let mut out = String::new();
    append_text(&mut out, plan_hash);
    append_text(&mut out, application_ledger_hash);
    append_text(&mut out, &write.write_id);
    append_text(&mut out, &write.write_kind);
    append_text(&mut out, &write.source_bytes_hash);
    append_text(&mut out, &write.encoded_bytes_hash);
    append_text(&mut out, &write.post_write_bytes_hash);
    append_text(&mut out, &write.status);
    writeln!(out, "write={}|{}", write.file_offset, write.width_bytes).unwrap();
    crate::fnv1a64_hex(out.as_bytes())
}

pub(super) fn source_preservation_audit_hash(
    plan_hash: &str,
    application_ledger_hash: &str,
    audit: &ElfAmd64ShellSourcePreservationAudit,
) -> String {
    let mut out = String::new();
    append_text(&mut out, plan_hash);
    append_text(&mut out, application_ledger_hash);
    append_text(&mut out, &audit.preservation_id);
    append_text(&mut out, &audit.section_id);
    append_text(&mut out, &audit.section_name);
    append_text(&mut out, &audit.preservation_kind);
    append_text(&mut out, &audit.source_bytes_hash);
    append_text(&mut out, &audit.result_bytes_hash);
    append_text(&mut out, &audit.status);
    writeln!(
        out,
        "span={}|{}|{}|{}",
        audit.source_image_offset,
        audit.source_size_bytes,
        optional_usize(audit.result_file_offset),
        audit.result_size_bytes
    )
    .unwrap();
    crate::fnv1a64_hex(out.as_bytes())
}

pub(super) fn section_audit_hash(ledger_hash: &str, section: &ElfAmd64ShellSectionPlan) -> String {
    let mut out = String::new();
    append_text(&mut out, ledger_hash);
    append_text(&mut out, &section.section_id);
    append_text(&mut out, &section.section_name);
    append_text(&mut out, &section.source_kind);
    append_text(&mut out, &section.source_id);
    append_text(
        &mut out,
        section.load_segment_id.as_deref().unwrap_or("none"),
    );
    writeln!(
        out,
        "shape={}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
        section.section_index,
        section.section_name_offset,
        section.section_type,
        section.flags,
        section.alignment,
        section.entry_size,
        section.link_section_index,
        section.info_section_index,
        optional_usize(section.source_image_offset),
        section.source_size_bytes,
        section.file_offset,
        section.file_size_bytes,
        section.virtual_address,
        section.memory_size_bytes,
        section.load_segment_id.is_some(),
        section.section_name.len()
    )
    .unwrap();
    crate::fnv1a64_hex(out.as_bytes())
}

pub(super) fn program_header_audit_hash(
    ledger_hash: &str,
    header: &ElfAmd64ShellProgramHeaderPlan,
) -> String {
    let mut out = String::new();
    append_text(&mut out, ledger_hash);
    append_text(&mut out, &header.program_header_id);
    append_text(&mut out, &header.program_kind);
    append_text(&mut out, &header.permission_class);
    for section_id in &header.section_ids {
        append_text(&mut out, section_id);
    }
    writeln!(
        out,
        "shape={}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
        header.program_header_index,
        header.program_type,
        header.flags,
        header.file_offset,
        header.virtual_address,
        header.file_size_bytes,
        header.memory_size_bytes,
        header.alignment,
        header.section_ids.len(),
        header.program_kind.len()
    )
    .unwrap();
    crate::fnv1a64_hex(out.as_bytes())
}

pub(super) fn dynamic_entry_audit_hash(
    ledger_hash: &str,
    entry: &ElfAmd64ShellDynamicEntryPlan,
) -> String {
    let mut out = String::new();
    append_text(&mut out, ledger_hash);
    append_text(&mut out, &entry.dynamic_entry_id);
    append_text(&mut out, &entry.tag_name);
    append_text(&mut out, &entry.value_kind);
    append_text(
        &mut out,
        entry.referenced_section_id.as_deref().unwrap_or("none"),
    );
    writeln!(
        out,
        "entry={}|{}|{}|{}",
        entry.dynamic_entry_index,
        entry.tag,
        entry.value,
        entry.tag_name.len()
    )
    .unwrap();
    crate::fnv1a64_hex(out.as_bytes())
}

fn append_text(out: &mut String, value: &str) {
    writeln!(out, "text:{}:{value}", value.len()).unwrap();
}

fn optional_usize(value: Option<usize>) -> String {
    value.map_or_else(|| "none".to_owned(), |value| value.to_string())
}

fn optional_u64(value: Option<u64>) -> String {
    value.map_or_else(|| "none".to_owned(), |value| value.to_string())
}
