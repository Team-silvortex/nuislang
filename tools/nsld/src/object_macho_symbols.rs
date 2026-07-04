use super::reports::{NsldObjectFileLayoutRecordDiagnostic, NsldObjectFileLayoutReport};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldMachOSymbolTablePlan {
    pub(crate) symbol_count: usize,
    pub(crate) string_table_size: usize,
    pub(crate) symbols: Vec<NsldMachOSymbolPlan>,
    pub(crate) strings: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldMachOSymbolPlan {
    pub(crate) name: String,
    pub(crate) string_offset: u32,
    pub(crate) symbol_type: u8,
    pub(crate) section_index: u8,
    pub(crate) description: u16,
    pub(crate) value: u64,
}

pub(crate) fn mach_o_arm64_symbol_table_plan(
    file_layout: &NsldObjectFileLayoutReport,
) -> Option<NsldMachOSymbolTablePlan> {
    if file_layout.writer_backend_kind != "mach-o-arm64" {
        return None;
    }
    let section_records = file_layout
        .records
        .iter()
        .filter(|record| record.record_kind == "section-payload")
        .collect::<Vec<_>>();
    let expected_symbol_count = section_records.len().saturating_add(1);
    let expected_string_size = record_by_kind(file_layout, "macho-string-table")?.size_bytes;
    let mut strings = vec![0u8];
    let mut symbols = Vec::new();
    symbols.push(symbol_plan("__nuis_entry", &mut strings, 0, 0, 1));
    for (index, record) in section_records.iter().enumerate() {
        symbols.push(symbol_plan(
            &mach_o_symbol_name(record),
            &mut strings,
            (index + 1) as u8,
            record.file_offset as u64,
            0xe,
        ));
    }
    strings.resize(expected_string_size, 0);

    Some(NsldMachOSymbolTablePlan {
        symbol_count: expected_symbol_count,
        string_table_size: expected_string_size,
        symbols,
        strings,
    })
}

pub(crate) fn encode_mach_o_symbols(plan: &NsldMachOSymbolTablePlan) -> Vec<u8> {
    let mut bytes = vec![0u8; plan.symbol_count.saturating_mul(16)];
    for (index, symbol) in plan.symbols.iter().enumerate() {
        let offset = index * 16;
        write_u32_le(&mut bytes, offset, symbol.string_offset);
        bytes[offset + 4] = symbol.symbol_type;
        bytes[offset + 5] = symbol.section_index;
        write_u16_le(&mut bytes, offset + 6, symbol.description);
        write_u64_le(&mut bytes, offset + 8, symbol.value);
    }
    bytes
}

pub(crate) fn encode_mach_o_string_table(plan: &NsldMachOSymbolTablePlan) -> Vec<u8> {
    plan.strings.clone()
}

fn symbol_plan(
    name: &str,
    strings: &mut Vec<u8>,
    section_index: u8,
    value: u64,
    symbol_type: u8,
) -> NsldMachOSymbolPlan {
    let string_offset = strings.len() as u32;
    strings.extend_from_slice(name.as_bytes());
    strings.push(0);
    NsldMachOSymbolPlan {
        name: name.to_owned(),
        string_offset,
        symbol_type,
        section_index,
        description: 0,
        value,
    }
}

fn mach_o_symbol_name(record: &NsldObjectFileLayoutRecordDiagnostic) -> String {
    let raw = record
        .record_id
        .strip_prefix("section.")
        .unwrap_or(&record.record_id);
    format!("__nuis_{}", raw.replace('.', "_"))
}

fn record_by_kind<'a>(
    file_layout: &'a NsldObjectFileLayoutReport,
    kind: &str,
) -> Option<&'a NsldObjectFileLayoutRecordDiagnostic> {
    file_layout
        .records
        .iter()
        .find(|record| record.record_kind == kind)
}

fn write_u16_le(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn write_u32_le(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_u64_le(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::{
        encode_mach_o_string_table, encode_mach_o_symbols, mach_o_arm64_symbol_table_plan,
    };
    use crate::{
        main_test_support::empty_link_plan, object_file_layout::nsld_object_file_layout_report,
    };
    use std::path::Path;

    #[test]
    fn plans_and_encodes_mach_o_symbol_and_string_tables() {
        let plan = empty_link_plan();
        let file_layout = nsld_object_file_layout_report(Path::new("manifest.toml"), &plan);
        let symbols = mach_o_arm64_symbol_table_plan(&file_layout).unwrap();
        let symbol_bytes = encode_mach_o_symbols(&symbols);
        let string_bytes = encode_mach_o_string_table(&symbols);

        assert_eq!(symbols.symbol_count, 5);
        assert_eq!(symbols.symbols[0].name, "__nuis_entry");
        assert_eq!(symbols.symbols[0].string_offset, 1);
        assert_eq!(symbol_bytes.len(), 80);
        assert_eq!(string_bytes.len(), symbols.string_table_size);
        assert_eq!(string_bytes[0], 0);
        assert!(string_bytes
            .windows("__nuis_entry".len())
            .any(|window| window == b"__nuis_entry"));
        assert_eq!(&symbol_bytes[0..4], &[1, 0, 0, 0]);
    }
}
