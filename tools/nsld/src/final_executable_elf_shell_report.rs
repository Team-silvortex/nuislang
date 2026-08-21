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
