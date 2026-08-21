use std::fmt::Write as _;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64RelocationApplicationReport {
    pub(crate) contract: &'static str,
    pub(crate) status: String,
    pub(crate) plan_hash: String,
    pub(crate) placement_plan_hash: String,
    pub(crate) relocation_count: usize,
    pub(crate) registered_kind_count: usize,
    pub(crate) direct_preview_count: usize,
    pub(crate) platform_structure_count: usize,
    pub(crate) no_op_count: usize,
    pub(crate) applications: Vec<ElfAmd64RelocationApplication>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64RelocationApplication {
    pub(crate) relocation_id: String,
    pub(crate) object_id: String,
    pub(crate) object_role: String,
    pub(crate) relocation_section_index: usize,
    pub(crate) input_section_index: usize,
    pub(crate) source_section_id: String,
    pub(crate) source_offset: usize,
    pub(crate) source_file_offset: usize,
    pub(crate) source_image_offset: usize,
    pub(crate) source_virtual_address: u64,
    pub(crate) width_bytes: usize,
    pub(crate) pc_relative: bool,
    pub(crate) relocation_type: u32,
    pub(crate) relocation_kind: String,
    pub(crate) action_kind: String,
    pub(crate) target_symbol: Option<String>,
    pub(crate) target_symbol_index: Option<usize>,
    pub(crate) target_symbol_external: bool,
    pub(crate) target_object_id: Option<String>,
    pub(crate) target_kind: Option<String>,
    pub(crate) target_section_id: Option<String>,
    pub(crate) target_image_offset: Option<usize>,
    pub(crate) target_virtual_address: Option<u64>,
    pub(crate) target_absolute_value: Option<u64>,
    pub(crate) addend: i64,
    pub(crate) computed_value: Option<i128>,
    pub(crate) encoded_value: Option<u64>,
    pub(crate) encoded_bytes: Vec<u8>,
    pub(crate) resolver_status: String,
    pub(crate) application_status: String,
}

impl ElfAmd64RelocationApplicationReport {
    pub(crate) fn canonical_plan(&self) -> String {
        let mut out = String::new();
        append_text(&mut out, self.contract);
        append_text(&mut out, &self.status);
        append_text(&mut out, &self.placement_plan_hash);
        writeln!(
            out,
            "counts={}|{}|{}|{}|{}",
            self.relocation_count,
            self.registered_kind_count,
            self.direct_preview_count,
            self.platform_structure_count,
            self.no_op_count
        )
        .unwrap();
        for item in &self.applications {
            append_application(&mut out, item);
        }
        out
    }
}

fn append_application(out: &mut String, item: &ElfAmd64RelocationApplication) {
    append_text(out, &item.relocation_id);
    append_text(out, &item.object_id);
    append_text(out, &item.object_role);
    append_text(out, &item.source_section_id);
    append_text(out, &item.relocation_kind);
    append_text(out, &item.action_kind);
    append_text(out, item.target_symbol.as_deref().unwrap_or("none"));
    append_text(out, item.target_object_id.as_deref().unwrap_or("none"));
    append_text(out, item.target_kind.as_deref().unwrap_or("none"));
    append_text(out, item.target_section_id.as_deref().unwrap_or("none"));
    append_text(out, &item.resolver_status);
    append_text(out, &item.application_status);
    writeln!(
        out,
        "source={}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
        item.relocation_section_index,
        item.input_section_index,
        item.source_offset,
        item.source_file_offset,
        item.source_image_offset,
        item.source_virtual_address,
        item.width_bytes,
        item.pc_relative,
        item.relocation_type,
        item.target_symbol_external
    )
    .unwrap();
    writeln!(
        out,
        "target={}|{}|{}|{}|{}|{}|{}|{}|{}",
        optional_usize(item.target_symbol_index),
        optional_usize(item.target_image_offset),
        optional_u64(item.target_virtual_address),
        optional_u64(item.target_absolute_value),
        item.addend,
        optional_i128(item.computed_value),
        optional_u64(item.encoded_value),
        hex_bytes(&item.encoded_bytes),
        item.encoded_bytes.len()
    )
    .unwrap();
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

fn optional_i128(value: Option<i128>) -> String {
    value.map_or_else(|| "none".to_owned(), |value| value.to_string())
}

fn hex_bytes(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}
