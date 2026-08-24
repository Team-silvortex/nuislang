use super::{
    checked_slice, read_string, read_u16, read_u32, section_by_name, ParsedSectionHeaders,
};
use crate::final_executable_elf_shell::{
    version::{
        ELF64_VERSION_NEED_AUX_SIZE, ELF64_VERSION_NEED_HEADER_SIZE,
        ELF64_VERSION_SYMBOL_ENTRY_SIZE, ELF_VERSION_NEED_CURRENT,
    },
    ElfAmd64ShellLayoutPlanReport,
};

pub(super) struct ParsedElfAmd64VersionMetadata {
    pub(super) version_symbol_span: Option<(usize, usize)>,
    pub(super) version_need_span: Option<(usize, usize)>,
    pub(super) version_symbol_indexes: Vec<u16>,
    pub(super) version_requirements: Vec<String>,
}

pub(super) fn parse_and_validate_version_metadata(
    bytes: &[u8],
    shell: &ElfAmd64ShellLayoutPlanReport,
    sections: &ParsedSectionHeaders,
) -> Result<ParsedElfAmd64VersionMetadata, String> {
    if shell.version_symbols.is_empty() && shell.version_needs.is_empty() {
        if shell.version_symbol_table_file_offset.is_some()
            || shell.version_symbol_table_virtual_address.is_some()
            || shell.version_symbol_table_bytes != 0
            || shell.version_need_table_file_offset.is_some()
            || shell.version_need_table_virtual_address.is_some()
            || shell.version_need_table_bytes != 0
        {
            return Err("ELF shell empty version metadata has planned coordinates".to_owned());
        }
        return Ok(ParsedElfAmd64VersionMetadata {
            version_symbol_span: None,
            version_need_span: None,
            version_symbol_indexes: Vec::new(),
            version_requirements: Vec::new(),
        });
    }
    if shell.version_symbols.is_empty() || shell.version_needs.is_empty() {
        return Err("ELF shell version metadata plan is incomplete".to_owned());
    }
    let dynstr = section_by_name(shell, sections, ".dynstr")?;
    let strings = checked_slice(
        bytes,
        dynstr.file_offset,
        dynstr.size,
        "version dynamic string table",
    )?;
    let versym = section_by_name(shell, sections, ".gnu.version")?;
    let verneed = section_by_name(shell, sections, ".gnu.version_r")?;
    if shell.version_symbol_table_file_offset != Some(versym.file_offset)
        || shell.version_symbol_table_virtual_address != Some(versym.virtual_address)
        || shell.version_symbol_table_bytes != versym.size
        || shell.version_need_table_file_offset != Some(verneed.file_offset)
        || shell.version_need_table_virtual_address != Some(verneed.virtual_address)
        || shell.version_need_table_bytes != verneed.size
    {
        return Err("ELF shell version section coordinates differ from their plan".to_owned());
    }
    let version_symbol_indexes =
        parse_version_symbols(bytes, shell, versym.file_offset, versym.size)?;
    let version_requirements =
        parse_version_needs(bytes, strings, shell, verneed.file_offset, verneed.size)?;
    Ok(ParsedElfAmd64VersionMetadata {
        version_symbol_span: Some((versym.file_offset, versym.size)),
        version_need_span: Some((verneed.file_offset, verneed.size)),
        version_symbol_indexes,
        version_requirements,
    })
}

fn parse_version_symbols(
    bytes: &[u8],
    shell: &ElfAmd64ShellLayoutPlanReport,
    offset: usize,
    size: usize,
) -> Result<Vec<u16>, String> {
    if !size.is_multiple_of(ELF64_VERSION_SYMBOL_ENTRY_SIZE)
        || size / ELF64_VERSION_SYMBOL_ENTRY_SIZE != shell.version_symbols.len() + 1
    {
        return Err("ELF shell GNU version-symbol table width differs from its plan".to_owned());
    }
    let mut indexes = Vec::with_capacity(size / ELF64_VERSION_SYMBOL_ENTRY_SIZE);
    for index in 0..size / ELF64_VERSION_SYMBOL_ENTRY_SIZE {
        let value = read_u16(
            bytes,
            offset + index * ELF64_VERSION_SYMBOL_ENTRY_SIZE,
            "GNU version-symbol index",
        )?;
        if value & 0x8000 != 0 {
            return Err("ELF shell GNU version-symbol hidden bit is not registered".to_owned());
        }
        let expected = if index == 0 {
            0
        } else {
            let plan = shell
                .version_symbols
                .iter()
                .find(|symbol| symbol.dynamic_symbol_index == index)
                .ok_or_else(|| "ELF shell GNU version-symbol coverage differs".to_owned())?;
            plan.version_index
        };
        if value != expected {
            return Err("ELF shell GNU version-symbol value differs from its plan".to_owned());
        }
        indexes.push(value);
    }
    Ok(indexes)
}

fn parse_version_needs(
    bytes: &[u8],
    strings: &[u8],
    shell: &ElfAmd64ShellLayoutPlanReport,
    table_offset: usize,
    table_size: usize,
) -> Result<Vec<String>, String> {
    let table = checked_slice(bytes, table_offset, table_size, "GNU version-need table")?;
    let mut requirements = Vec::new();
    let mut expected_cursor = 0usize;
    for (need_index, expected) in shell.version_needs.iter().enumerate() {
        if expected.record_offset != expected_cursor {
            return Err("ELF shell GNU version-need plan is not contiguous".to_owned());
        }
        let version = read_u16(table, expected_cursor, "GNU version-need version")?;
        let auxiliary_count = usize::from(read_u16(
            table,
            expected_cursor + 2,
            "GNU version-need count",
        )?);
        let file_name_offset = usize::try_from(read_u32(
            table,
            expected_cursor + 4,
            "GNU version-need file",
        )?)
        .map_err(|_| "ELF GNU version-need file offset exceeds usize".to_owned())?;
        let auxiliary_offset = usize::try_from(read_u32(
            table,
            expected_cursor + 8,
            "GNU version-need auxiliary offset",
        )?)
        .map_err(|_| "ELF GNU version-need auxiliary offset exceeds usize".to_owned())?;
        let next_offset = usize::try_from(read_u32(
            table,
            expected_cursor + 12,
            "GNU version-need next offset",
        )?)
        .map_err(|_| "ELF GNU version-need next offset exceeds usize".to_owned())?;
        let needed_name = read_string(strings, file_name_offset, "GNU version-need file name")?;
        if version != ELF_VERSION_NEED_CURRENT
            || auxiliary_count != expected.auxiliaries.len()
            || file_name_offset != expected.needed_name_dynamic_string_offset
            || needed_name != expected.needed_name
            || auxiliary_offset != expected.auxiliary_offset
            || next_offset != expected.next_offset
        {
            return Err("ELF shell GNU version-need header differs from its plan".to_owned());
        }
        let mut auxiliary_cursor = expected_cursor + auxiliary_offset;
        for (aux_index, auxiliary) in expected.auxiliaries.iter().enumerate() {
            if auxiliary.record_offset != auxiliary_cursor {
                return Err("ELF shell GNU version auxiliary plan is not contiguous".to_owned());
            }
            let version_hash = read_u32(table, auxiliary_cursor, "GNU version hash")?;
            let flags = read_u16(table, auxiliary_cursor + 4, "GNU version flags")?;
            let version_index = read_u16(table, auxiliary_cursor + 6, "GNU version index")?;
            let version_name_offset = usize::try_from(read_u32(
                table,
                auxiliary_cursor + 8,
                "GNU version name offset",
            )?)
            .map_err(|_| "ELF GNU version name offset exceeds usize".to_owned())?;
            let auxiliary_next = usize::try_from(read_u32(
                table,
                auxiliary_cursor + 12,
                "GNU version auxiliary next offset",
            )?)
            .map_err(|_| "ELF GNU version auxiliary next offset exceeds usize".to_owned())?;
            let version_name = read_string(strings, version_name_offset, "GNU version name")?;
            if version_hash != auxiliary.version_hash
                || flags != 0
                || version_index != auxiliary.version_index
                || version_name_offset != auxiliary.dynamic_string_offset
                || version_name != auxiliary.symbol_version_name
                || auxiliary_next != auxiliary.next_offset
                || (auxiliary_next == 0) != (aux_index + 1 == auxiliary_count)
            {
                return Err("ELF shell GNU version auxiliary differs from its plan".to_owned());
            }
            requirements.push(format!(
                "{}@{}#{}",
                needed_name, version_name, version_index
            ));
            auxiliary_cursor = auxiliary_cursor
                .checked_add(if auxiliary_next == 0 {
                    ELF64_VERSION_NEED_AUX_SIZE
                } else {
                    auxiliary_next
                })
                .ok_or_else(|| "ELF GNU version auxiliary traversal overflows".to_owned())?;
        }
        let record_end = auxiliary_cursor;
        let planned_end = expected_cursor
            .checked_add(if next_offset == 0 {
                ELF64_VERSION_NEED_HEADER_SIZE
                    + expected.auxiliaries.len() * ELF64_VERSION_NEED_AUX_SIZE
            } else {
                next_offset
            })
            .ok_or_else(|| "ELF GNU version-need traversal overflows".to_owned())?;
        if record_end != planned_end {
            return Err("ELF shell GNU version-need record width differs from its plan".to_owned());
        }
        expected_cursor = planned_end;
        if (next_offset == 0) != (need_index + 1 == shell.version_needs.len()) {
            return Err("ELF shell GNU version-need chain termination differs".to_owned());
        }
    }
    if expected_cursor != table.len() {
        return Err("ELF shell GNU version-need table has unexplained bytes".to_owned());
    }
    Ok(requirements)
}
