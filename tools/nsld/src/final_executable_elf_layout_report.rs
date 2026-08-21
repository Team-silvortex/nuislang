use std::fmt::Write as _;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64PlacementBindingReport {
    pub(crate) contract: &'static str,
    pub(crate) status: String,
    pub(crate) plan_hash: String,
    pub(crate) image_base: u64,
    pub(crate) payload_file_offset: usize,
    pub(crate) file_span_bytes: usize,
    pub(crate) memory_span_bytes: usize,
    pub(crate) merged_sections: Vec<ElfAmd64MergedSectionPlan>,
    pub(crate) section_placements: Vec<ElfAmd64SectionPlacement>,
    pub(crate) common_allocations: Vec<ElfAmd64CommonAllocation>,
    pub(crate) symbol_bindings: Vec<ElfAmd64SymbolBinding>,
    pub(crate) internally_bound_symbol_count: usize,
    pub(crate) external_compatibility_symbol_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64MergedSectionPlan {
    pub(crate) section_id: String,
    pub(crate) output_section_name: String,
    pub(crate) class: String,
    pub(crate) alignment: usize,
    pub(crate) file_offset: Option<usize>,
    pub(crate) image_offset: usize,
    pub(crate) virtual_address: u64,
    pub(crate) size_bytes: usize,
    pub(crate) contribution_count: usize,
    pub(crate) zero_fill: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64SectionPlacement {
    pub(crate) object_id: String,
    pub(crate) object_role: String,
    pub(crate) input_section_index: usize,
    pub(crate) input_section_name: String,
    pub(crate) output_section_id: String,
    pub(crate) output_section_offset: usize,
    pub(crate) alignment: usize,
    pub(crate) size_bytes: usize,
    pub(crate) file_offset: Option<usize>,
    pub(crate) image_offset: usize,
    pub(crate) virtual_address: u64,
    pub(crate) zero_fill: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64CommonAllocation {
    pub(crate) allocation_id: String,
    pub(crate) symbol: String,
    pub(crate) external: bool,
    pub(crate) owner_object_id: String,
    pub(crate) owner_object_role: String,
    pub(crate) owner_symbol_index: usize,
    pub(crate) declaration_count: usize,
    pub(crate) size_bytes: usize,
    pub(crate) alignment: usize,
    pub(crate) output_section_id: String,
    pub(crate) output_section_offset: usize,
    pub(crate) image_offset: usize,
    pub(crate) virtual_address: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64SymbolBinding {
    pub(crate) symbol: String,
    pub(crate) reference_object_id: String,
    pub(crate) reference_symbol_index: usize,
    pub(crate) status: String,
    pub(crate) target_object_id: Option<String>,
    pub(crate) target_symbol_index: Option<usize>,
    pub(crate) target_kind: Option<String>,
    pub(crate) target_section_id: Option<String>,
    pub(crate) target_image_offset: Option<usize>,
    pub(crate) target_virtual_address: Option<u64>,
    pub(crate) target_absolute_value: Option<u64>,
}

impl ElfAmd64PlacementBindingReport {
    pub(crate) fn canonical_plan(&self) -> String {
        let mut out = String::new();
        append_text(&mut out, self.contract);
        append_text(&mut out, &self.status);
        writeln!(
            out,
            "base={}|payload={}|file={}|memory={}",
            self.image_base, self.payload_file_offset, self.file_span_bytes, self.memory_span_bytes
        )
        .unwrap();
        for section in &self.merged_sections {
            append_text(&mut out, "merged");
            append_text(&mut out, &section.section_id);
            append_text(&mut out, &section.output_section_name);
            append_text(&mut out, &section.class);
            writeln!(
                out,
                "facts={}|{}|{}|{}|{}|{}|{}",
                section.alignment,
                optional_usize(section.file_offset),
                section.image_offset,
                section.virtual_address,
                section.size_bytes,
                section.contribution_count,
                section.zero_fill
            )
            .unwrap();
        }
        append_placements(&mut out, &self.section_placements);
        append_common_allocations(&mut out, &self.common_allocations);
        append_symbol_bindings(&mut out, &self.symbol_bindings);
        out
    }
}

fn append_placements(out: &mut String, placements: &[ElfAmd64SectionPlacement]) {
    for placement in placements {
        append_text(out, "placement");
        append_text(out, &placement.object_id);
        append_text(out, &placement.object_role);
        append_text(out, &placement.input_section_name);
        append_text(out, &placement.output_section_id);
        writeln!(
            out,
            "facts={}|{}|{}|{}|{}|{}|{}|{}",
            placement.input_section_index,
            placement.output_section_offset,
            placement.alignment,
            placement.size_bytes,
            optional_usize(placement.file_offset),
            placement.image_offset,
            placement.virtual_address,
            placement.zero_fill
        )
        .unwrap();
    }
}

fn append_common_allocations(out: &mut String, allocations: &[ElfAmd64CommonAllocation]) {
    for allocation in allocations {
        append_text(out, "common");
        append_text(out, &allocation.allocation_id);
        append_text(out, &allocation.symbol);
        append_text(out, &allocation.owner_object_id);
        append_text(out, &allocation.output_section_id);
        writeln!(
            out,
            "facts={}|{}|{}|{}|{}|{}|{}|{}",
            allocation.external,
            allocation.owner_symbol_index,
            allocation.declaration_count,
            allocation.size_bytes,
            allocation.alignment,
            allocation.output_section_offset,
            allocation.image_offset,
            allocation.virtual_address
        )
        .unwrap();
    }
}

fn append_symbol_bindings(out: &mut String, bindings: &[ElfAmd64SymbolBinding]) {
    for binding in bindings {
        append_text(out, "binding");
        append_text(out, &binding.symbol);
        append_text(out, &binding.reference_object_id);
        append_text(out, &binding.status);
        append_text(out, binding.target_object_id.as_deref().unwrap_or("none"));
        append_text(out, binding.target_kind.as_deref().unwrap_or("none"));
        append_text(out, binding.target_section_id.as_deref().unwrap_or("none"));
        writeln!(
            out,
            "facts={}|{}|{}|{}|{}",
            binding.reference_symbol_index,
            optional_usize(binding.target_symbol_index),
            optional_usize(binding.target_image_offset),
            optional_u64(binding.target_virtual_address),
            optional_u64(binding.target_absolute_value)
        )
        .unwrap();
    }
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
