use std::collections::BTreeMap;

const ELF_HEADER_SIZE: usize = 64;
const ELF_SECTION_HEADER_SIZE: usize = 64;
const ELF_SYMBOL_SIZE: usize = 24;
const ELF_RELA_SIZE: usize = 24;

pub(crate) const R_X86_64_PLT32: u32 = 4;

pub(crate) fn elf_program_object(relocation_type: u32) -> Vec<u8> {
    elf_program_object_with_external_symbol(relocation_type, "nuis_runtime_entry")
}

pub(crate) fn elf_program_object_with_external_symbol(
    relocation_type: u32,
    external_symbol: &str,
) -> Vec<u8> {
    let relocations = [(1, relocation_type, -4)];
    build_object(ObjectFixture {
        text: &[0xe8, 0, 0, 0, 0, 0xc3],
        defined_symbol: "__nuis_entry",
        undefined_symbol: Some(external_symbol),
        relocations: &relocations,
        bss_size: 0,
    })
}

pub(crate) fn elf_program_object_two_plt32_calls() -> Vec<u8> {
    let relocations = [(1, R_X86_64_PLT32, -4), (6, R_X86_64_PLT32, -4)];
    build_object(ObjectFixture {
        text: &[0xe8, 0, 0, 0, 0, 0xe8, 0, 0, 0, 0, 0xc3],
        defined_symbol: "__nuis_entry",
        undefined_symbol: Some("nuis_runtime_entry"),
        relocations: &relocations,
        bss_size: 0,
    })
}

pub(crate) fn elf_runtime_object() -> Vec<u8> {
    build_object(ObjectFixture {
        text: &[0xc3],
        defined_symbol: "nuis_runtime_entry",
        undefined_symbol: None,
        relocations: &[],
        bss_size: 0,
    })
}

pub(crate) fn elf_runtime_object_with_bss() -> Vec<u8> {
    build_object(ObjectFixture {
        text: &[0xc3],
        defined_symbol: "nuis_runtime_entry",
        undefined_symbol: None,
        relocations: &[],
        bss_size: 32,
    })
}

pub(crate) fn elf_unrelated_runtime_object() -> Vec<u8> {
    build_object(ObjectFixture {
        text: &[0xc3],
        defined_symbol: "nuis_unrelated_runtime_entry",
        undefined_symbol: None,
        relocations: &[],
        bss_size: 0,
    })
}

pub(crate) fn elf_exit_program_object() -> Vec<u8> {
    build_object(ObjectFixture {
        text: &[0x31, 0xc0, 0xc3],
        defined_symbol: "__nuis_entry",
        undefined_symbol: None,
        relocations: &[],
        bss_size: 0,
    })
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub(crate) fn elf_alternate_exit_program_object() -> Vec<u8> {
    build_object(ObjectFixture {
        text: &[0xb8, 0, 0, 0, 0, 0xc3],
        defined_symbol: "__nuis_entry",
        undefined_symbol: None,
        relocations: &[],
        bss_size: 0,
    })
}

pub(crate) fn elf_linux_exit_runtime_object() -> Vec<u8> {
    let relocations = [(1, R_X86_64_PLT32, -4)];
    build_object(ObjectFixture {
        text: &[
            0xe8, 0, 0, 0, 0, // call __nuis_entry
            0x89, 0xc7, // mov edi, eax
            0xb8, 0x3c, 0, 0, 0, // mov eax, SYS_exit
            0x0f, 0x05, // syscall
        ],
        defined_symbol: "_start",
        undefined_symbol: Some("__nuis_entry"),
        relocations: &relocations,
        bss_size: 0,
    })
}

struct ObjectFixture<'a> {
    text: &'a [u8],
    defined_symbol: &'a str,
    undefined_symbol: Option<&'a str>,
    relocations: &'a [(u64, u32, i64)],
    bss_size: usize,
}

fn build_object(fixture: ObjectFixture<'_>) -> Vec<u8> {
    let has_relocation = !fixture.relocations.is_empty();
    let mut section_names = vec![".text"];
    if has_relocation {
        section_names.push(".rela.text");
    }
    if fixture.bss_size > 0 {
        section_names.push(".bss");
    }
    section_names.extend([".symtab", ".strtab", ".shstrtab"]);
    let section_strings = string_table(section_names.iter().copied());
    let symbol_strings =
        string_table(std::iter::once(fixture.defined_symbol).chain(fixture.undefined_symbol));
    let symbol_count = 2 + usize::from(fixture.undefined_symbol.is_some());
    let section_count = section_names.len() + 1;
    let text_offset = ELF_HEADER_SIZE;
    let relocation_offset = align_up(text_offset + fixture.text.len(), 8);
    let relocation_size = fixture.relocations.len() * ELF_RELA_SIZE;
    let symbol_offset = align_up(relocation_offset + relocation_size, 8);
    let symbol_size = symbol_count * ELF_SYMBOL_SIZE;
    let string_offset = symbol_offset + symbol_size;
    let section_string_offset = string_offset + symbol_strings.bytes.len();
    let section_table_offset = align_up(section_string_offset + section_strings.bytes.len(), 8);
    let mut bytes = vec![0u8; section_table_offset + section_count * ELF_SECTION_HEADER_SIZE];

    write_header(
        &mut bytes,
        section_table_offset,
        section_count,
        section_count - 1,
    );
    bytes[text_offset..text_offset + fixture.text.len()].copy_from_slice(fixture.text);
    for (index, (source_offset, relocation_type, addend)) in
        fixture.relocations.iter().copied().enumerate()
    {
        let entry_offset = relocation_offset + index * ELF_RELA_SIZE;
        write_u64(&mut bytes, entry_offset, source_offset);
        write_u64(
            &mut bytes,
            entry_offset + 8,
            (2u64 << 32) | u64::from(relocation_type),
        );
        write_i64(&mut bytes, entry_offset + 16, addend);
    }
    write_symbol(
        &mut bytes,
        symbol_offset + ELF_SYMBOL_SIZE,
        symbol_strings.offsets[fixture.defined_symbol],
        0x12,
        1,
        0,
        fixture.text.len() as u64,
    );
    if let Some(undefined) = fixture.undefined_symbol {
        write_symbol(
            &mut bytes,
            symbol_offset + 2 * ELF_SYMBOL_SIZE,
            symbol_strings.offsets[undefined],
            0x12,
            0,
            0,
            0,
        );
    }
    bytes[string_offset..string_offset + symbol_strings.bytes.len()]
        .copy_from_slice(&symbol_strings.bytes);
    bytes[section_string_offset..section_string_offset + section_strings.bytes.len()]
        .copy_from_slice(&section_strings.bytes);

    let text_index = 1;
    let relocation_index = has_relocation.then_some(2);
    let bss_index = (fixture.bss_size > 0).then_some(2 + usize::from(has_relocation));
    let symbol_index = 2 + usize::from(has_relocation) + usize::from(fixture.bss_size > 0);
    let string_index = symbol_index + 1;
    let section_string_index = string_index + 1;
    write_section_header(
        &mut bytes,
        section_table_offset,
        text_index,
        SectionHeader {
            name: section_strings.offsets[".text"],
            kind: 1,
            flags: 0x6,
            offset: text_offset,
            size: fixture.text.len(),
            link: 0,
            info: 0,
            alignment: 16,
            entry_size: 0,
        },
    );
    if let Some(index) = relocation_index {
        write_section_header(
            &mut bytes,
            section_table_offset,
            index,
            SectionHeader {
                name: section_strings.offsets[".rela.text"],
                kind: 4,
                flags: 0,
                offset: relocation_offset,
                size: relocation_size,
                link: symbol_index,
                info: text_index,
                alignment: 8,
                entry_size: ELF_RELA_SIZE,
            },
        );
    }
    if let Some(index) = bss_index {
        write_section_header(
            &mut bytes,
            section_table_offset,
            index,
            SectionHeader {
                name: section_strings.offsets[".bss"],
                kind: 8,
                flags: 0x3,
                offset: 0,
                size: fixture.bss_size,
                link: 0,
                info: 0,
                alignment: 16,
                entry_size: 0,
            },
        );
    }
    write_section_header(
        &mut bytes,
        section_table_offset,
        symbol_index,
        SectionHeader {
            name: section_strings.offsets[".symtab"],
            kind: 2,
            flags: 0,
            offset: symbol_offset,
            size: symbol_size,
            link: string_index,
            info: 1,
            alignment: 8,
            entry_size: ELF_SYMBOL_SIZE,
        },
    );
    write_section_header(
        &mut bytes,
        section_table_offset,
        string_index,
        SectionHeader {
            name: section_strings.offsets[".strtab"],
            kind: 3,
            flags: 0,
            offset: string_offset,
            size: symbol_strings.bytes.len(),
            link: 0,
            info: 0,
            alignment: 1,
            entry_size: 0,
        },
    );
    write_section_header(
        &mut bytes,
        section_table_offset,
        section_string_index,
        SectionHeader {
            name: section_strings.offsets[".shstrtab"],
            kind: 3,
            flags: 0,
            offset: section_string_offset,
            size: section_strings.bytes.len(),
            link: 0,
            info: 0,
            alignment: 1,
            entry_size: 0,
        },
    );
    bytes
}

struct StringTable {
    bytes: Vec<u8>,
    offsets: BTreeMap<String, u32>,
}

fn string_table<'a>(values: impl IntoIterator<Item = &'a str>) -> StringTable {
    let mut bytes = vec![0u8];
    let mut offsets = BTreeMap::new();
    for value in values {
        offsets.entry(value.to_owned()).or_insert_with(|| {
            let offset = bytes.len() as u32;
            bytes.extend_from_slice(value.as_bytes());
            bytes.push(0);
            offset
        });
    }
    StringTable { bytes, offsets }
}

struct SectionHeader {
    name: u32,
    kind: u32,
    flags: u64,
    offset: usize,
    size: usize,
    link: usize,
    info: usize,
    alignment: u64,
    entry_size: usize,
}

fn write_header(
    bytes: &mut [u8],
    section_table_offset: usize,
    section_count: usize,
    section_string_index: usize,
) {
    bytes[..16].copy_from_slice(&[0x7f, b'E', b'L', b'F', 2, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    write_u16(bytes, 16, 1);
    write_u16(bytes, 18, 62);
    write_u32(bytes, 20, 1);
    write_u64(bytes, 40, section_table_offset as u64);
    write_u16(bytes, 52, ELF_HEADER_SIZE as u16);
    write_u16(bytes, 58, ELF_SECTION_HEADER_SIZE as u16);
    write_u16(bytes, 60, section_count as u16);
    write_u16(bytes, 62, section_string_index as u16);
}

fn write_symbol(
    bytes: &mut [u8],
    offset: usize,
    name: u32,
    info: u8,
    section_index: u16,
    value: u64,
    size: u64,
) {
    write_u32(bytes, offset, name);
    bytes[offset + 4] = info;
    write_u16(bytes, offset + 6, section_index);
    write_u64(bytes, offset + 8, value);
    write_u64(bytes, offset + 16, size);
}

fn write_section_header(
    bytes: &mut [u8],
    table_offset: usize,
    index: usize,
    header: SectionHeader,
) {
    let offset = table_offset + index * ELF_SECTION_HEADER_SIZE;
    write_u32(bytes, offset, header.name);
    write_u32(bytes, offset + 4, header.kind);
    write_u64(bytes, offset + 8, header.flags);
    write_u64(bytes, offset + 24, header.offset as u64);
    write_u64(bytes, offset + 32, header.size as u64);
    write_u32(bytes, offset + 40, header.link as u32);
    write_u32(bytes, offset + 44, header.info as u32);
    write_u64(bytes, offset + 48, header.alignment);
    write_u64(bytes, offset + 56, header.entry_size as u64);
}

fn align_up(value: usize, alignment: usize) -> usize {
    (value + alignment - 1) & !(alignment - 1)
}

fn write_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_u64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

fn write_i64(bytes: &mut [u8], offset: usize, value: i64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}
