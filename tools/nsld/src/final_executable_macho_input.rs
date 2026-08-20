use std::{collections::BTreeSet, ops::Range};

const MACH_O_64_HEADER_SIZE: usize = 32;
const MACH_O_64_LE_MAGIC: [u8; 4] = [0xcf, 0xfa, 0xed, 0xfe];
const MACH_O_CPU_TYPE_ARM64: u32 = 0x0100_000c;
const MACH_O_FILE_TYPE_OBJECT: u32 = 1;
const LC_SEGMENT_64: u32 = 0x19;
const LC_SYMTAB: u32 = 0x2;
const SEGMENT_COMMAND_64_SIZE: usize = 72;
const SECTION_64_SIZE: usize = 80;
const SYMTAB_COMMAND_SIZE: usize = 24;
const NLIST_64_SIZE: usize = 16;
const RELOCATION_SIZE: usize = 8;
const N_STAB: u8 = 0xe0;
const N_TYPE: u8 = 0x0e;
const N_EXT: u8 = 0x01;
const N_UNDF: u8 = 0x00;
const N_ABS: u8 = 0x02;
const N_SECT: u8 = 0x0e;
const N_PBUD: u8 = 0x0c;
const N_INDR: u8 = 0x0a;
const S_ZEROFILL: u32 = 0x01;
const S_GB_ZEROFILL: u32 = 0x0c;
const S_THREAD_LOCAL_ZEROFILL: u32 = 0x12;
const ARM64_RELOCATION_TYPE_MAX: u32 = 11;
const ARM64_RELOCATION_ADDEND: u32 = 10;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedMachOObjectLinkage {
    pub(crate) section_count: usize,
    pub(crate) symbol_count: usize,
    pub(crate) relocation_count: usize,
    pub(crate) defined_symbol_count: usize,
    pub(crate) undefined_symbol_count: usize,
    pub(crate) external_definitions: BTreeSet<String>,
    pub(crate) external_undefined: BTreeSet<String>,
    pub(crate) sections: Vec<ParsedMachOSection>,
    pub(crate) symbols: Vec<ParsedMachOSymbol>,
    pub(crate) relocations: Vec<ParsedMachORelocation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedMachOSection {
    pub(crate) ordinal: usize,
    pub(crate) segment_name: String,
    pub(crate) name: String,
    pub(crate) address: u64,
    pub(crate) size: u64,
    pub(crate) alignment: u64,
    pub(crate) flags: u32,
    pub(crate) zero_fill: bool,
    pub(crate) payload_offset: usize,
    pub(crate) relocation_offset: usize,
    pub(crate) relocation_count: usize,
}

#[derive(Debug, Clone, Copy)]
struct SymbolTableCommand {
    symbol_offset: usize,
    symbol_count: usize,
    string_offset: usize,
    string_size: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedMachOSymbol {
    pub(crate) index: usize,
    pub(crate) name: String,
    pub(crate) kind: String,
    pub(crate) external: bool,
    pub(crate) defined: bool,
    pub(crate) section_ordinal: Option<usize>,
    pub(crate) value: u64,
    pub(crate) common_alignment: Option<u64>,
    pub(crate) indirect_target: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedMachORelocation {
    pub(crate) section_ordinal: usize,
    pub(crate) offset: u32,
    pub(crate) symbol_number: usize,
    pub(crate) width_bytes: u64,
    pub(crate) pc_relative: bool,
    pub(crate) external: bool,
    pub(crate) relocation_type: u32,
}

pub(crate) fn parse_macho_arm64_object_linkage(
    bytes: &[u8],
) -> Result<ParsedMachOObjectLinkage, String> {
    validate_header(bytes)?;
    let command_count = read_u32_le(bytes, 16)? as usize;
    let command_span = read_u32_le(bytes, 20)? as usize;
    let command_end = checked_end(
        MACH_O_64_HEADER_SIZE,
        command_span,
        bytes.len(),
        "load-command span",
    )?;
    let (sections, symbol_table) = parse_load_commands(bytes, command_count, command_end)?;
    let symbol_table =
        symbol_table.ok_or_else(|| "Mach-O object has no LC_SYMTAB command".to_owned())?;
    let (
        symbols,
        defined_symbol_count,
        undefined_symbol_count,
        external_definitions,
        external_undefined,
    ) = parse_symbols(bytes, symbol_table, sections.len())?;
    let relocations = parse_relocations(bytes, &sections, &symbols)?;

    Ok(ParsedMachOObjectLinkage {
        section_count: sections.len(),
        symbol_count: symbols.len(),
        relocation_count: relocations.len(),
        defined_symbol_count,
        undefined_symbol_count,
        external_definitions,
        external_undefined,
        sections,
        symbols,
        relocations,
    })
}

fn validate_header(bytes: &[u8]) -> Result<(), String> {
    if bytes.len() < MACH_O_64_HEADER_SIZE {
        return Err(format!(
            "Mach-O object is truncated: expected at least {MACH_O_64_HEADER_SIZE} bytes, found {}",
            bytes.len()
        ));
    }
    if bytes[..4] != MACH_O_64_LE_MAGIC {
        return Err("Mach-O object magic is not little-endian MH_MAGIC_64".to_owned());
    }
    let cpu_type = read_u32_le(bytes, 4)?;
    if cpu_type != MACH_O_CPU_TYPE_ARM64 {
        return Err(format!(
            "Mach-O object CPU type is 0x{cpu_type:08x}; expected ARM64"
        ));
    }
    let file_type = read_u32_le(bytes, 12)?;
    if file_type != MACH_O_FILE_TYPE_OBJECT {
        return Err(format!(
            "Mach-O file type is {file_type}; expected MH_OBJECT"
        ));
    }
    Ok(())
}

fn parse_load_commands(
    bytes: &[u8],
    command_count: usize,
    command_end: usize,
) -> Result<(Vec<ParsedMachOSection>, Option<SymbolTableCommand>), String> {
    let mut cursor = MACH_O_64_HEADER_SIZE;
    let mut sections = Vec::new();
    let mut symbol_table = None;
    let mut segment_present = false;

    for index in 0..command_count {
        checked_end(
            cursor,
            8,
            command_end,
            &format!("load command {index} header"),
        )?;
        let command = read_u32_le(bytes, cursor)?;
        let command_size = read_u32_le(bytes, cursor + 4)? as usize;
        if command_size < 8 || command_size % 4 != 0 {
            return Err(format!(
                "Mach-O object load command {index} has invalid size {command_size}"
            ));
        }
        let next = checked_end(
            cursor,
            command_size,
            command_end,
            &format!("load command {index}"),
        )?;
        match command {
            LC_SEGMENT_64 => {
                segment_present = true;
                parse_segment_command(bytes, cursor, command_size, &mut sections)?;
            }
            LC_SYMTAB => {
                if symbol_table.is_some() {
                    return Err("Mach-O object contains duplicate LC_SYMTAB commands".to_owned());
                }
                symbol_table = Some(parse_symbol_table_command(bytes, cursor, command_size)?);
            }
            _ => {}
        }
        cursor = next;
    }
    if cursor != command_end {
        return Err(format!(
            "Mach-O object load-command count consumes {} bytes, declared span is {}",
            cursor.saturating_sub(MACH_O_64_HEADER_SIZE),
            command_end.saturating_sub(MACH_O_64_HEADER_SIZE)
        ));
    }
    if !segment_present {
        return Err("Mach-O object has no LC_SEGMENT_64 command".to_owned());
    }
    Ok((sections, symbol_table))
}

fn parse_segment_command(
    bytes: &[u8],
    command_offset: usize,
    command_size: usize,
    sections: &mut Vec<ParsedMachOSection>,
) -> Result<(), String> {
    if command_size < SEGMENT_COMMAND_64_SIZE {
        return Err("Mach-O object LC_SEGMENT_64 is shorter than 72 bytes".to_owned());
    }
    let section_count = read_u32_le(bytes, command_offset + 64)? as usize;
    let section_bytes = section_count
        .checked_mul(SECTION_64_SIZE)
        .ok_or_else(|| "Mach-O section table size overflows address space".to_owned())?;
    let required_size = SEGMENT_COMMAND_64_SIZE
        .checked_add(section_bytes)
        .ok_or_else(|| "Mach-O segment command size overflows address space".to_owned())?;
    if required_size > command_size {
        return Err(format!(
            "Mach-O LC_SEGMENT_64 declares {section_count} sections but command size is {command_size}"
        ));
    }

    for local_index in 0..section_count {
        let offset = command_offset + SEGMENT_COMMAND_64_SIZE + local_index * SECTION_64_SIZE;
        let name = fixed_name(bytes, offset, 16)?;
        let segment_name = fixed_name(bytes, offset + 16, 16)?;
        let address = read_u64_le(bytes, offset + 32)?;
        let size = read_u64_le(bytes, offset + 40)?;
        let payload_offset = read_u32_le(bytes, offset + 48)? as usize;
        let alignment_exponent = read_u32_le(bytes, offset + 52)?;
        let alignment = 1u64.checked_shl(alignment_exponent).ok_or_else(|| {
            format!(
                "Mach-O section `{name}` alignment exponent {alignment_exponent} is unsupported"
            )
        })?;
        let relocation_offset = read_u32_le(bytes, offset + 56)? as usize;
        let relocation_count = read_u32_le(bytes, offset + 60)? as usize;
        let flags = read_u32_le(bytes, offset + 64)?;
        let zero_fill = is_zero_fill(flags);
        if !zero_fill {
            let payload_size = usize::try_from(size).map_err(|_| {
                format!("Mach-O section `{name}` size does not fit host address space")
            })?;
            checked_end(
                payload_offset,
                payload_size,
                bytes.len(),
                &format!("section `{name}` payload"),
            )?;
        }
        sections.push(ParsedMachOSection {
            ordinal: sections.len() + 1,
            segment_name,
            name,
            address,
            size,
            alignment,
            flags,
            zero_fill,
            payload_offset,
            relocation_offset,
            relocation_count,
        });
    }
    Ok(())
}

fn parse_symbol_table_command(
    bytes: &[u8],
    command_offset: usize,
    command_size: usize,
) -> Result<SymbolTableCommand, String> {
    if command_size < SYMTAB_COMMAND_SIZE {
        return Err("Mach-O object LC_SYMTAB is shorter than 24 bytes".to_owned());
    }
    Ok(SymbolTableCommand {
        symbol_offset: read_u32_le(bytes, command_offset + 8)? as usize,
        symbol_count: read_u32_le(bytes, command_offset + 12)? as usize,
        string_offset: read_u32_le(bytes, command_offset + 16)? as usize,
        string_size: read_u32_le(bytes, command_offset + 20)? as usize,
    })
}

type ParsedSymbols = (
    Vec<ParsedMachOSymbol>,
    usize,
    usize,
    BTreeSet<String>,
    BTreeSet<String>,
);

fn parse_symbols(
    bytes: &[u8],
    table: SymbolTableCommand,
    section_count: usize,
) -> Result<ParsedSymbols, String> {
    let symbol_bytes = table
        .symbol_count
        .checked_mul(NLIST_64_SIZE)
        .ok_or_else(|| "Mach-O symbol table size overflows address space".to_owned())?;
    checked_end(
        table.symbol_offset,
        symbol_bytes,
        bytes.len(),
        "symbol table",
    )?;
    let string_range = checked_range(
        table.string_offset,
        table.string_size,
        bytes.len(),
        "string table",
    )?;
    let strings = &bytes[string_range];
    let mut symbols = Vec::with_capacity(table.symbol_count);
    let mut defined_count = 0usize;
    let mut undefined_count = 0usize;
    let mut external_definitions = BTreeSet::new();
    let mut external_undefined = BTreeSet::new();

    for index in 0..table.symbol_count {
        let offset = table.symbol_offset + index * NLIST_64_SIZE;
        let string_index = read_u32_le(bytes, offset)? as usize;
        let symbol_type = *bytes
            .get(offset + 4)
            .ok_or_else(|| format!("Mach-O symbol {index} type is truncated"))?;
        let section_ordinal = *bytes
            .get(offset + 5)
            .ok_or_else(|| format!("Mach-O symbol {index} section ordinal is truncated"))?
            as usize;
        let description = read_u16_le(bytes, offset + 6)?;
        let value = read_u64_le(bytes, offset + 8)?;
        let name = string_name(strings, string_index, index)?;
        let mut kind_name = "debug";
        let mut defined = false;
        let mut resolved_section_ordinal = None;
        let mut common_alignment = None;
        let mut indirect_target = None;
        let external = symbol_type & N_EXT != 0;
        if symbol_type & N_STAB == 0 {
            let kind = symbol_type & N_TYPE;
            match kind {
                N_SECT => {
                    if section_ordinal == 0 || section_ordinal > section_count {
                        return Err(format!(
                            "Mach-O symbol {index} `{name}` references invalid section ordinal {section_ordinal}"
                        ));
                    }
                    kind_name = "section";
                    defined = true;
                    resolved_section_ordinal = Some(section_ordinal);
                    defined_count += 1;
                    if external && !name.is_empty() {
                        external_definitions.insert(name.clone());
                    }
                }
                N_ABS | N_INDR => {
                    kind_name = if kind == N_ABS {
                        "absolute"
                    } else {
                        "indirect"
                    };
                    defined = true;
                    defined_count += 1;
                    if kind == N_INDR {
                        let target_index = usize::try_from(value).map_err(|_| {
                            format!("Mach-O indirect symbol {index} target index overflows")
                        })?;
                        indirect_target = Some(string_name(strings, target_index, index)?);
                    }
                    if external && !name.is_empty() {
                        external_definitions.insert(name.clone());
                    }
                }
                N_UNDF if value != 0 => {
                    kind_name = "common";
                    defined = true;
                    let exponent = u32::from((description >> 8) & 0x0f);
                    common_alignment = Some(1u64 << exponent);
                    defined_count += 1;
                    if external && !name.is_empty() {
                        external_definitions.insert(name.clone());
                    }
                }
                N_UNDF | N_PBUD => {
                    kind_name = if kind == N_UNDF {
                        "undefined"
                    } else {
                        "prebound-undefined"
                    };
                    undefined_count += 1;
                    if external {
                        if name.is_empty() {
                            return Err(format!(
                                "Mach-O external undefined symbol {index} has an empty name"
                            ));
                        }
                        external_undefined.insert(name.clone());
                    }
                }
                other => {
                    return Err(format!(
                        "Mach-O symbol {index} `{name}` has unsupported n_type 0x{other:02x}"
                    ));
                }
            }
        }
        symbols.push(ParsedMachOSymbol {
            index,
            name,
            kind: kind_name.to_owned(),
            external,
            defined,
            section_ordinal: resolved_section_ordinal,
            value,
            common_alignment,
            indirect_target,
        });
    }
    Ok((
        symbols,
        defined_count,
        undefined_count,
        external_definitions,
        external_undefined,
    ))
}

fn parse_relocations(
    bytes: &[u8],
    sections: &[ParsedMachOSection],
    symbols: &[ParsedMachOSymbol],
) -> Result<Vec<ParsedMachORelocation>, String> {
    let mut relocations = Vec::new();
    for section in sections {
        let table_size = section
            .relocation_count
            .checked_mul(RELOCATION_SIZE)
            .ok_or_else(|| {
                format!(
                    "Mach-O section `{}` relocation table overflows",
                    section.name
                )
            })?;
        checked_end(
            section.relocation_offset,
            table_size,
            bytes.len(),
            &format!("section `{}` relocation table", section.name),
        )?;
        for index in 0..section.relocation_count {
            let offset = section.relocation_offset + index * RELOCATION_SIZE;
            let address = read_u32_le(bytes, offset)?;
            if address & 0x8000_0000 != 0 {
                return Err(format!(
                    "Mach-O section `{}` relocation {index} uses unsupported scattered/negative address",
                    section.name
                ));
            }
            let word = read_u32_le(bytes, offset + 4)?;
            let symbol_number = (word & 0x00ff_ffff) as usize;
            let pc_relative = (word >> 24) & 0x1 != 0;
            let length = (word >> 25) & 0x3;
            let external = (word >> 27) & 0x1 != 0;
            let relocation_type = word >> 28;
            if relocation_type > ARM64_RELOCATION_TYPE_MAX {
                return Err(format!(
                    "Mach-O section `{}` relocation {index} has unsupported ARM64 type {relocation_type}",
                    section.name
                ));
            }
            let width = 1u64 << length;
            let relocation_end = u64::from(address).checked_add(width).ok_or_else(|| {
                format!(
                    "Mach-O section `{}` relocation {index} overflows",
                    section.name
                )
            })?;
            if relocation_end > section.size {
                return Err(format!(
                    "Mach-O section `{}` relocation {index} span ends at {relocation_end}, beyond section size {}",
                    section.name, section.size
                ));
            }
            if relocation_type == ARM64_RELOCATION_ADDEND {
                if external {
                    return Err(format!(
                        "Mach-O section `{}` ADDEND relocation {index} cannot use an external symbol reference",
                        section.name
                    ));
                }
            } else if external {
                let symbol = symbols.get(symbol_number).ok_or_else(|| {
                    format!(
                        "Mach-O section `{}` relocation {index} references missing symbol index {symbol_number}",
                        section.name
                    )
                })?;
                let _ = &symbol.name;
            } else if symbol_number > sections.len() {
                return Err(format!(
                    "Mach-O section `{}` relocation {index} references invalid local section ordinal {symbol_number}",
                    section.name
                ));
            }
            relocations.push(ParsedMachORelocation {
                section_ordinal: section.ordinal,
                offset: address,
                symbol_number,
                width_bytes: width,
                pc_relative,
                external,
                relocation_type,
            });
        }
    }
    Ok(relocations)
}

fn string_name(strings: &[u8], index: usize, symbol_index: usize) -> Result<String, String> {
    if index == 0 {
        return Ok(String::new());
    }
    let tail = strings.get(index..).ok_or_else(|| {
        format!("Mach-O symbol {symbol_index} string index {index} exceeds string table")
    })?;
    let end = tail
        .iter()
        .position(|byte| *byte == 0)
        .ok_or_else(|| format!("Mach-O symbol {symbol_index} name is not NUL-terminated"))?;
    std::str::from_utf8(&tail[..end])
        .map(str::to_owned)
        .map_err(|_| format!("Mach-O symbol {symbol_index} name is not valid UTF-8"))
}

fn fixed_name(bytes: &[u8], offset: usize, width: usize) -> Result<String, String> {
    let range = checked_range(offset, width, bytes.len(), "fixed-width name")?;
    let raw = &bytes[range];
    let end = raw.iter().position(|byte| *byte == 0).unwrap_or(raw.len());
    Ok(String::from_utf8_lossy(&raw[..end]).into_owned())
}

fn is_zero_fill(flags: u32) -> bool {
    matches!(
        flags & 0xff,
        S_ZEROFILL | S_GB_ZEROFILL | S_THREAD_LOCAL_ZEROFILL
    )
}

fn checked_range(
    offset: usize,
    size: usize,
    limit: usize,
    label: &str,
) -> Result<Range<usize>, String> {
    let end = checked_end(offset, size, limit, label)?;
    Ok(offset..end)
}

fn checked_end(offset: usize, size: usize, limit: usize, label: &str) -> Result<usize, String> {
    offset
        .checked_add(size)
        .filter(|end| *end <= limit)
        .ok_or_else(|| format!("Mach-O object {label} exceeds object boundary {limit}"))
}

fn read_u32_le(bytes: &[u8], offset: usize) -> Result<u32, String> {
    let range = checked_range(offset, 4, bytes.len(), "u32")?;
    let raw: [u8; 4] = bytes[range]
        .try_into()
        .map_err(|_| format!("Mach-O object u32 at offset {offset} is malformed"))?;
    Ok(u32::from_le_bytes(raw))
}

fn read_u16_le(bytes: &[u8], offset: usize) -> Result<u16, String> {
    let range = checked_range(offset, 2, bytes.len(), "u16")?;
    let raw: [u8; 2] = bytes[range]
        .try_into()
        .map_err(|_| format!("Mach-O object u16 at offset {offset} is malformed"))?;
    Ok(u16::from_le_bytes(raw))
}

fn read_u64_le(bytes: &[u8], offset: usize) -> Result<u64, String> {
    let range = checked_range(offset, 8, bytes.len(), "u64")?;
    let raw: [u8; 8] = bytes[range]
        .try_into()
        .map_err(|_| format!("Mach-O object u64 at offset {offset} is malformed"))?;
    Ok(u64::from_le_bytes(raw))
}

#[cfg(test)]
#[path = "final_executable_macho_input_tests.rs"]
mod tests;
