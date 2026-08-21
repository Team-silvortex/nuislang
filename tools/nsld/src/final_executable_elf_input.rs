use std::collections::BTreeSet;

const ELF64_HEADER_SIZE: usize = 64;
const ELF64_SECTION_HEADER_SIZE: usize = 64;
const ELF64_SYMBOL_SIZE: usize = 24;
const ELF64_RELA_SIZE: usize = 24;
const ELF_TYPE_RELOCATABLE: u16 = 1;
const ELF_MACHINE_X86_64: u16 = 62;
const ELF_SECTION_TYPE_NULL: u32 = 0;
const ELF_SECTION_TYPE_SYMBOL_TABLE: u32 = 2;
const ELF_SECTION_TYPE_STRING_TABLE: u32 = 3;
const ELF_SECTION_TYPE_RELA: u32 = 4;
const ELF_SECTION_TYPE_NOBITS: u32 = 8;
const ELF_SECTION_TYPE_REL: u32 = 9;
const ELF_SYMBOL_BIND_LOCAL: u8 = 0;
const ELF_SYMBOL_BIND_GLOBAL: u8 = 1;
const ELF_SYMBOL_BIND_WEAK: u8 = 2;
const ELF_SECTION_UNDEFINED: usize = 0;
const ELF_SECTION_ABSOLUTE: usize = 0xfff1;
const ELF_SECTION_COMMON: usize = 0xfff2;
const R_X86_64_NONE: u32 = 0;
const R_X86_64_64: u32 = 1;
const R_X86_64_PC32: u32 = 2;
const R_X86_64_PLT32: u32 = 4;
const R_X86_64_32: u32 = 10;
const R_X86_64_32S: u32 = 11;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedElfObjectLinkage {
    pub(crate) section_count: usize,
    pub(crate) symbol_count: usize,
    pub(crate) relocation_count: usize,
    pub(crate) defined_symbol_count: usize,
    pub(crate) undefined_symbol_count: usize,
    pub(crate) external_definitions: BTreeSet<String>,
    pub(crate) external_undefined: BTreeSet<String>,
    pub(crate) sections: Vec<ParsedElfSection>,
    pub(crate) symbols: Vec<ParsedElfSymbol>,
    pub(crate) relocations: Vec<ParsedElfRelocation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedElfSection {
    pub(crate) index: usize,
    pub(crate) name: String,
    pub(crate) section_type: u32,
    pub(crate) flags: u64,
    pub(crate) size: usize,
    pub(crate) alignment: u64,
    pub(crate) payload_offset: Option<usize>,
    pub(crate) zero_fill: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedElfSymbol {
    pub(crate) index: usize,
    pub(crate) name: String,
    pub(crate) binding: u8,
    pub(crate) symbol_type: u8,
    pub(crate) external: bool,
    pub(crate) weak: bool,
    pub(crate) defined: bool,
    pub(crate) section_index: Option<usize>,
    pub(crate) absolute: bool,
    pub(crate) common: bool,
    pub(crate) value: u64,
    pub(crate) size: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedElfRelocation {
    pub(crate) relocation_section_index: usize,
    pub(crate) target_section_index: usize,
    pub(crate) symbol_index: usize,
    pub(crate) offset: u64,
    pub(crate) addend: i64,
    pub(crate) relocation_type: u32,
    pub(crate) width_bytes: u64,
    pub(crate) pc_relative: bool,
}

#[derive(Debug, Clone)]
struct RawElfSection {
    index: usize,
    name_offset: usize,
    section_type: u32,
    flags: u64,
    payload_offset: usize,
    size: usize,
    link: usize,
    info: usize,
    alignment: u64,
    entry_size: usize,
}

pub(crate) fn parse_elf64_amd64_object_linkage(
    bytes: &[u8],
) -> Result<ParsedElfObjectLinkage, String> {
    validate_header(bytes)?;
    let raw_sections = parse_section_headers(bytes)?;
    let sections = parse_section_names(bytes, &raw_sections)?;
    let symbol_table_index = unique_symbol_table_index(&raw_sections)?;
    let symbols = parse_symbols(bytes, &raw_sections, symbol_table_index)?;
    let relocations = parse_relocations(bytes, &raw_sections, symbol_table_index, &symbols)?;
    let defined_symbol_count = symbols
        .iter()
        .skip(1)
        .filter(|symbol| symbol.defined)
        .count();
    let undefined_symbol_count = symbols
        .iter()
        .skip(1)
        .filter(|symbol| !symbol.defined)
        .count();
    let external_definitions = symbols
        .iter()
        .filter(|symbol| symbol.external && symbol.defined)
        .map(|symbol| symbol.name.clone())
        .collect();
    let external_undefined = symbols
        .iter()
        .filter(|symbol| symbol.external && !symbol.defined)
        .map(|symbol| symbol.name.clone())
        .collect();

    Ok(ParsedElfObjectLinkage {
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
    if bytes.len() < ELF64_HEADER_SIZE {
        return Err(format!(
            "ELF object is truncated: expected at least {ELF64_HEADER_SIZE} bytes, found {}",
            bytes.len()
        ));
    }
    if bytes.get(..7) != Some(&[0x7f, b'E', b'L', b'F', 2, 1, 1]) {
        return Err("ELF object ident is not little-endian ELF64 EV_CURRENT".to_owned());
    }
    if read_u16(bytes, 16, "ELF object type")? != ELF_TYPE_RELOCATABLE {
        return Err("ELF host object is not ET_REL".to_owned());
    }
    if read_u16(bytes, 18, "ELF object machine")? != ELF_MACHINE_X86_64 {
        return Err("ELF host object machine is not x86_64".to_owned());
    }
    if read_u32(bytes, 20, "ELF object version")? != 1 {
        return Err("ELF host object version is not EV_CURRENT".to_owned());
    }
    if read_u16(bytes, 52, "ELF header size")? as usize != ELF64_HEADER_SIZE {
        return Err("ELF host object header size is not 64 bytes".to_owned());
    }
    if read_u16(bytes, 54, "ELF program-header size")? != 0
        || read_u16(bytes, 56, "ELF program-header count")? != 0
    {
        return Err("ELF relocatable object unexpectedly declares program headers".to_owned());
    }
    Ok(())
}

fn parse_section_headers(bytes: &[u8]) -> Result<Vec<RawElfSection>, String> {
    let table_offset = checked_usize(
        read_u64(bytes, 40, "ELF section-header offset")?,
        "ELF section-header offset",
    )?;
    let entry_size = read_u16(bytes, 58, "ELF section-header size")? as usize;
    let section_count = read_u16(bytes, 60, "ELF section count")? as usize;
    if entry_size != ELF64_SECTION_HEADER_SIZE || section_count == 0 {
        return Err(format!(
            "ELF section table shape is unsupported: entry_size={entry_size} count={section_count}"
        ));
    }
    if table_offset < ELF64_HEADER_SIZE {
        return Err("ELF section table overlaps the file header".to_owned());
    }
    checked_table_end(
        table_offset,
        entry_size,
        section_count,
        bytes.len(),
        "ELF section table",
    )?;

    let mut sections = Vec::with_capacity(section_count);
    for index in 0..section_count {
        let offset = table_offset + index * entry_size;
        let section_type = read_u32(bytes, offset + 4, "ELF section type")?;
        let payload_offset = checked_usize(
            read_u64(bytes, offset + 24, "ELF section payload offset")?,
            "ELF section payload offset",
        )?;
        let size = checked_usize(
            read_u64(bytes, offset + 32, "ELF section size")?,
            "ELF section size",
        )?;
        let alignment = read_u64(bytes, offset + 48, "ELF section alignment")?;
        let entry_size = checked_usize(
            read_u64(bytes, offset + 56, "ELF section entry size")?,
            "ELF section entry size",
        )?;
        if alignment > 1 && !alignment.is_power_of_two() {
            return Err(format!(
                "ELF section {index} alignment {alignment} is not a power of two"
            ));
        }
        if entry_size != 0 && !size.is_multiple_of(entry_size) {
            return Err(format!(
                "ELF section {index} size {size} is not divisible by entry size {entry_size}"
            ));
        }
        if section_type != ELF_SECTION_TYPE_NOBITS {
            checked_end(
                payload_offset,
                size,
                bytes.len(),
                &format!("ELF section {index} payload"),
            )?;
        }
        sections.push(RawElfSection {
            index,
            name_offset: read_u32(bytes, offset, "ELF section name offset")? as usize,
            section_type,
            flags: read_u64(bytes, offset + 8, "ELF section flags")?,
            payload_offset,
            size,
            link: read_u32(bytes, offset + 40, "ELF section link")? as usize,
            info: read_u32(bytes, offset + 44, "ELF section info")? as usize,
            alignment,
            entry_size,
        });
    }
    if sections[0].section_type != ELF_SECTION_TYPE_NULL {
        return Err("ELF section zero is not SHT_NULL".to_owned());
    }
    Ok(sections)
}

fn parse_section_names(
    bytes: &[u8],
    sections: &[RawElfSection],
) -> Result<Vec<ParsedElfSection>, String> {
    let name_table_index = read_u16(bytes, 62, "ELF section-name table index")? as usize;
    let name_table = sections.get(name_table_index).ok_or_else(|| {
        format!("ELF section-name table index {name_table_index} is out of bounds")
    })?;
    if name_table.section_type != ELF_SECTION_TYPE_STRING_TABLE {
        return Err("ELF section-name table is not SHT_STRTAB".to_owned());
    }
    let names = section_payload(bytes, name_table)?;
    sections
        .iter()
        .skip(1)
        .map(|section| {
            Ok(ParsedElfSection {
                index: section.index,
                name: read_string(
                    names,
                    section.name_offset,
                    &format!("ELF section {} name", section.index),
                )?,
                section_type: section.section_type,
                flags: section.flags,
                size: section.size,
                alignment: section.alignment,
                payload_offset: (section.section_type != ELF_SECTION_TYPE_NOBITS)
                    .then_some(section.payload_offset),
                zero_fill: section.section_type == ELF_SECTION_TYPE_NOBITS,
            })
        })
        .collect()
}

fn unique_symbol_table_index(sections: &[RawElfSection]) -> Result<usize, String> {
    let indices = sections
        .iter()
        .filter(|section| section.section_type == ELF_SECTION_TYPE_SYMBOL_TABLE)
        .map(|section| section.index)
        .collect::<Vec<_>>();
    match indices.as_slice() {
        [index] => Ok(*index),
        [] => Err("ELF host object has no SHT_SYMTAB section".to_owned()),
        _ => Err(format!(
            "ELF host object contains {} SHT_SYMTAB sections; expected one",
            indices.len()
        )),
    }
}

fn parse_symbols(
    bytes: &[u8],
    sections: &[RawElfSection],
    symbol_table_index: usize,
) -> Result<Vec<ParsedElfSymbol>, String> {
    let table = &sections[symbol_table_index];
    if table.entry_size != ELF64_SYMBOL_SIZE || !table.size.is_multiple_of(ELF64_SYMBOL_SIZE) {
        return Err(format!(
            "ELF symbol table shape is invalid: size={} entry_size={}",
            table.size, table.entry_size
        ));
    }
    let strings = sections.get(table.link).ok_or_else(|| {
        format!(
            "ELF symbol string-table index {} is out of bounds",
            table.link
        )
    })?;
    if strings.section_type != ELF_SECTION_TYPE_STRING_TABLE {
        return Err("ELF symbol table does not link to SHT_STRTAB".to_owned());
    }
    let table_bytes = section_payload(bytes, table)?;
    let string_bytes = section_payload(bytes, strings)?;
    let symbol_count = table_bytes.len() / ELF64_SYMBOL_SIZE;
    if symbol_count == 0 || table.info == 0 || table.info > symbol_count {
        return Err(format!(
            "ELF symbol table local/global boundary {} is invalid for {symbol_count} symbols",
            table.info
        ));
    }
    let mut symbols = Vec::with_capacity(symbol_count);
    for index in 0..symbol_count {
        let offset = index * ELF64_SYMBOL_SIZE;
        let info = table_bytes[offset + 4];
        let binding = info >> 4;
        let symbol_type = info & 0x0f;
        if !matches!(
            binding,
            ELF_SYMBOL_BIND_LOCAL | ELF_SYMBOL_BIND_GLOBAL | ELF_SYMBOL_BIND_WEAK
        ) {
            return Err(format!(
                "ELF symbol {index} uses unsupported binding {binding}"
            ));
        }
        if symbol_type > 6 {
            return Err(format!(
                "ELF symbol {index} uses unsupported type {symbol_type}"
            ));
        }
        if table_bytes[offset + 5] & !0x03 != 0 {
            return Err(format!(
                "ELF symbol {index} uses unsupported visibility bits"
            ));
        }
        if index < table.info && binding != ELF_SYMBOL_BIND_LOCAL {
            return Err(format!(
                "ELF symbol {index} precedes sh_info but is not local"
            ));
        }
        if index >= table.info && binding == ELF_SYMBOL_BIND_LOCAL {
            return Err(format!(
                "ELF symbol {index} follows sh_info but remains local"
            ));
        }
        let section = read_u16(table_bytes, offset + 6, "ELF symbol section index")? as usize;
        let value = read_u64(table_bytes, offset + 8, "ELF symbol value")?;
        let size = read_u64(table_bytes, offset + 16, "ELF symbol size")?;
        let name = read_string(
            string_bytes,
            read_u32(table_bytes, offset, "ELF symbol name offset")? as usize,
            &format!("ELF symbol {index} name"),
        )?;
        let (defined, section_index, absolute, common) = match section {
            ELF_SECTION_UNDEFINED => (false, None, false, false),
            ELF_SECTION_ABSOLUTE => (true, None, true, false),
            ELF_SECTION_COMMON => {
                if value > 1 && !value.is_power_of_two() {
                    return Err(format!(
                        "ELF common symbol `{name}` alignment {value} is not a power of two"
                    ));
                }
                (true, None, false, true)
            }
            index if index < sections.len() => {
                let target = &sections[index];
                let end = value.checked_add(size).ok_or_else(|| {
                    format!("ELF symbol `{name}` section-relative range overflows")
                })?;
                if end > target.size as u64 {
                    return Err(format!(
                        "ELF symbol `{name}` range {value}..{end} exceeds section {index} size {}",
                        target.size
                    ));
                }
                (true, Some(index), false, false)
            }
            _ => {
                return Err(format!(
                    "ELF symbol `{name}` has unsupported section index 0x{section:04x}"
                ));
            }
        };
        let external = binding != ELF_SYMBOL_BIND_LOCAL && !name.is_empty();
        if binding != ELF_SYMBOL_BIND_LOCAL && index != 0 && name.is_empty() {
            return Err(format!("ELF external symbol {index} has an empty name"));
        }
        symbols.push(ParsedElfSymbol {
            index,
            name,
            binding,
            symbol_type,
            external,
            weak: binding == ELF_SYMBOL_BIND_WEAK,
            defined,
            section_index,
            absolute,
            common,
            value,
            size,
        });
    }
    let null = &symbols[0];
    if !null.name.is_empty()
        || null.binding != ELF_SYMBOL_BIND_LOCAL
        || null.defined
        || null.value != 0
        || null.size != 0
    {
        return Err("ELF symbol zero is not the required null symbol".to_owned());
    }
    Ok(symbols)
}

fn parse_relocations(
    bytes: &[u8],
    sections: &[RawElfSection],
    symbol_table_index: usize,
    symbols: &[ParsedElfSymbol],
) -> Result<Vec<ParsedElfRelocation>, String> {
    if let Some(section) = sections
        .iter()
        .find(|section| section.section_type == ELF_SECTION_TYPE_REL)
    {
        return Err(format!(
            "ELF relocation section {} uses SHT_REL; explicit-addend SHT_RELA is required",
            section.index
        ));
    }
    let mut relocations = Vec::new();
    for section in sections
        .iter()
        .filter(|section| section.section_type == ELF_SECTION_TYPE_RELA)
    {
        if section.link != symbol_table_index {
            return Err(format!(
                "ELF RELA section {} links symbol table {}, expected {symbol_table_index}",
                section.index, section.link
            ));
        }
        let target = sections.get(section.info).ok_or_else(|| {
            format!(
                "ELF RELA section {} target index {} is out of bounds",
                section.index, section.info
            )
        })?;
        if target.index == 0 || target.section_type == ELF_SECTION_TYPE_NOBITS {
            return Err(format!(
                "ELF RELA section {} targets a non-patchable section {}",
                section.index, target.index
            ));
        }
        if section.entry_size != ELF64_RELA_SIZE || !section.size.is_multiple_of(ELF64_RELA_SIZE) {
            return Err(format!(
                "ELF RELA section {} shape is invalid: size={} entry_size={}",
                section.index, section.size, section.entry_size
            ));
        }
        let entries = section_payload(bytes, section)?;
        for index in 0..entries.len() / ELF64_RELA_SIZE {
            let offset = index * ELF64_RELA_SIZE;
            let source_offset = read_u64(entries, offset, "ELF relocation offset")?;
            let info = read_u64(entries, offset + 8, "ELF relocation info")?;
            let symbol_index = (info >> 32) as usize;
            let relocation_type = info as u32;
            if symbol_index >= symbols.len() {
                return Err(format!(
                    "ELF relocation {}:{} symbol index {symbol_index} is out of bounds",
                    section.index, index
                ));
            }
            let (width_bytes, pc_relative) =
                relocation_shape(relocation_type).ok_or_else(|| {
                    format!(
                        "ELF relocation {}:{} uses unsupported R_X86_64 type {relocation_type}",
                        section.index, index
                    )
                })?;
            let end = source_offset.checked_add(width_bytes).ok_or_else(|| {
                format!(
                    "ELF relocation {}:{} patch range overflows",
                    section.index, index
                )
            })?;
            if end > target.size as u64 {
                return Err(format!(
                    "ELF relocation {}:{} patch range {source_offset}..{end} exceeds target section {} size {}",
                    section.index, index, target.index, target.size
                ));
            }
            relocations.push(ParsedElfRelocation {
                relocation_section_index: section.index,
                target_section_index: target.index,
                symbol_index,
                offset: source_offset,
                addend: read_i64(entries, offset + 16, "ELF relocation addend")?,
                relocation_type,
                width_bytes,
                pc_relative,
            });
        }
    }
    Ok(relocations)
}

fn relocation_shape(relocation_type: u32) -> Option<(u64, bool)> {
    match relocation_type {
        R_X86_64_NONE => Some((0, false)),
        R_X86_64_64 => Some((8, false)),
        R_X86_64_PC32 | R_X86_64_PLT32 => Some((4, true)),
        R_X86_64_32 | R_X86_64_32S => Some((4, false)),
        _ => None,
    }
}

fn section_payload<'a>(bytes: &'a [u8], section: &RawElfSection) -> Result<&'a [u8], String> {
    if section.section_type == ELF_SECTION_TYPE_NOBITS {
        return Err(format!("ELF section {} has no file payload", section.index));
    }
    bytes
        .get(section.payload_offset..section.payload_offset + section.size)
        .ok_or_else(|| format!("ELF section {} payload is truncated", section.index))
}

fn read_string(bytes: &[u8], offset: usize, label: &str) -> Result<String, String> {
    let tail = bytes
        .get(offset..)
        .ok_or_else(|| format!("{label} offset {offset} is out of bounds"))?;
    let end = tail
        .iter()
        .position(|byte| *byte == 0)
        .ok_or_else(|| format!("{label} is not NUL terminated"))?;
    std::str::from_utf8(&tail[..end])
        .map(str::to_owned)
        .map_err(|error| format!("{label} is not UTF-8: {error}"))
}

fn checked_table_end(
    offset: usize,
    entry_size: usize,
    count: usize,
    image_size: usize,
    label: &str,
) -> Result<usize, String> {
    let size = entry_size
        .checked_mul(count)
        .ok_or_else(|| format!("{label} size overflows"))?;
    checked_end(offset, size, image_size, label)
}

fn checked_end(
    offset: usize,
    size: usize,
    image_size: usize,
    label: &str,
) -> Result<usize, String> {
    offset
        .checked_add(size)
        .filter(|end| *end <= image_size)
        .ok_or_else(|| format!("{label} exceeds image bounds"))
}

fn checked_usize(value: u64, label: &str) -> Result<usize, String> {
    usize::try_from(value).map_err(|_| format!("{label} exceeds host address space"))
}

fn read_u16(bytes: &[u8], offset: usize, label: &str) -> Result<u16, String> {
    let raw: [u8; 2] = bytes
        .get(offset..offset.saturating_add(2))
        .ok_or_else(|| format!("{label} at offset {offset} is truncated"))?
        .try_into()
        .map_err(|_| format!("{label} at offset {offset} is malformed"))?;
    Ok(u16::from_le_bytes(raw))
}

fn read_u32(bytes: &[u8], offset: usize, label: &str) -> Result<u32, String> {
    let raw: [u8; 4] = bytes
        .get(offset..offset.saturating_add(4))
        .ok_or_else(|| format!("{label} at offset {offset} is truncated"))?
        .try_into()
        .map_err(|_| format!("{label} at offset {offset} is malformed"))?;
    Ok(u32::from_le_bytes(raw))
}

fn read_u64(bytes: &[u8], offset: usize, label: &str) -> Result<u64, String> {
    let raw: [u8; 8] = bytes
        .get(offset..offset.saturating_add(8))
        .ok_or_else(|| format!("{label} at offset {offset} is truncated"))?
        .try_into()
        .map_err(|_| format!("{label} at offset {offset} is malformed"))?;
    Ok(u64::from_le_bytes(raw))
}

fn read_i64(bytes: &[u8], offset: usize, label: &str) -> Result<i64, String> {
    read_u64(bytes, offset, label).map(|value| i64::from_le_bytes(value.to_le_bytes()))
}

#[cfg(test)]
#[path = "final_executable_elf_input_tests.rs"]
mod tests;
