use crate::container_toml::{
    array_table_blocks, bool_value_from_lines, isize_value_from_lines, string_value_from_lines,
    usize_value_from_lines,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ContainerSectionRecord {
    pub(super) section_id: String,
    pub(super) section_kind: String,
    pub(super) offset: usize,
    pub(super) size_bytes: usize,
    pub(super) payload_hash: String,
    pub(super) required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ContainerRelocationRecord {
    pub(super) relocation_id: String,
    pub(super) relocation_kind: String,
    pub(super) source_section_id: String,
    pub(super) source_offset: usize,
    pub(super) target_symbol_id: String,
    pub(super) addend: isize,
}

pub(super) fn section_records(source: &str) -> Vec<ContainerSectionRecord> {
    array_table_blocks(source, "section")
        .into_iter()
        .map(|block| ContainerSectionRecord {
            section_id: string_value_from_lines(&block, "section_id").unwrap_or_default(),
            section_kind: string_value_from_lines(&block, "section_kind").unwrap_or_default(),
            offset: usize_value_from_lines(&block, "offset").unwrap_or(usize::MAX),
            size_bytes: usize_value_from_lines(&block, "size_bytes").unwrap_or(0),
            payload_hash: string_value_from_lines(&block, "payload_hash").unwrap_or_default(),
            required: bool_value_from_lines(&block, "required").unwrap_or(false),
        })
        .collect()
}

pub(super) fn relocation_records(source: &str) -> Vec<ContainerRelocationRecord> {
    array_table_blocks(source, "relocation")
        .into_iter()
        .map(|block| ContainerRelocationRecord {
            relocation_id: string_value_from_lines(&block, "relocation_id").unwrap_or_default(),
            relocation_kind: string_value_from_lines(&block, "relocation_kind").unwrap_or_default(),
            source_section_id: string_value_from_lines(&block, "source_section_id")
                .unwrap_or_default(),
            source_offset: usize_value_from_lines(&block, "source_offset").unwrap_or(usize::MAX),
            target_symbol_id: string_value_from_lines(&block, "target_symbol_id")
                .unwrap_or_default(),
            addend: isize_value_from_lines(&block, "addend").unwrap_or(isize::MAX),
        })
        .collect()
}
