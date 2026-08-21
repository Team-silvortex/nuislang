use super::report::ElfAmd64ShellLayoutPlanReport;

const ELF64_HEADER_SIZE: usize = 64;
const ELF64_PROGRAM_HEADER_SIZE: usize = 56;
const ELF64_SECTION_HEADER_SIZE: usize = 64;
const ELF64_DYNAMIC_ENTRY_SIZE: usize = 16;
const ET_EXEC: u16 = 2;
const EM_X86_64: u16 = 62;
const PT_LOAD: u32 = 1;
const PT_DYNAMIC: u32 = 2;
const PF_X: u32 = 1;
const SHT_STRTAB: u32 = 3;
const SHT_NOBITS: u32 = 8;

pub(super) struct ParsedTableEvidence {
    pub(super) table_kind: &'static str,
    pub(super) file_offset: usize,
    pub(super) width_bytes: usize,
    pub(super) record_count: usize,
    pub(super) bytes_hash: String,
}

pub(super) struct ParsedElfAmd64ShellImage {
    pub(super) tables: Vec<ParsedTableEvidence>,
    pub(super) program_header_count: usize,
    pub(super) load_segment_count: usize,
    pub(super) dynamic_segment_count: usize,
    pub(super) dynamic_entry_count: usize,
    pub(super) section_header_count: usize,
    pub(super) section_name_count: usize,
    pub(super) entry_program_header_index: usize,
}

#[derive(Clone)]
struct ProgramHeader {
    program_type: u32,
    flags: u32,
    file_offset: usize,
    virtual_address: u64,
    physical_address: u64,
    file_size: usize,
    memory_size: usize,
    alignment: usize,
}

struct SectionHeader {
    name_offset: usize,
    section_type: u32,
    flags: u64,
    virtual_address: u64,
    file_offset: usize,
    size: usize,
    link: usize,
    info: usize,
    alignment: usize,
    entry_size: usize,
}

struct ParsedHeader {
    program_offset: usize,
    program_count: usize,
    section_offset: usize,
    section_count: usize,
    section_name_index: usize,
}

struct ParsedProgramHeaders {
    records: Vec<ProgramHeader>,
    load_count: usize,
    dynamic_span: Option<(usize, usize)>,
    entry_index: usize,
}

struct ParsedSectionHeaders {
    records: Vec<SectionHeader>,
    name_table_span: (usize, usize),
    name_count: usize,
}

pub(super) fn parse_and_validate_elf_amd64_shell_image(
    bytes: &[u8],
    shell: &ElfAmd64ShellLayoutPlanReport,
) -> Result<ParsedElfAmd64ShellImage, String> {
    let header = parse_header(bytes, shell)?;
    let programs = parse_program_headers(bytes, shell, &header)?;
    let sections = parse_section_headers(bytes, shell, &header)?;
    let dynamic_entry_count = parse_dynamic_entries(bytes, shell, programs.dynamic_span)?;
    validate_load_nonoverlap(&programs.records)?;

    let program_bytes = checked_mul(
        header.program_count,
        ELF64_PROGRAM_HEADER_SIZE,
        "program-header table",
    )?;
    let section_bytes = checked_mul(
        header.section_count,
        ELF64_SECTION_HEADER_SIZE,
        "section-header table",
    )?;
    let mut tables = vec![table_evidence(
        bytes,
        "elf64-header",
        0,
        ELF64_HEADER_SIZE,
        1,
    )?];
    tables.push(table_evidence(
        bytes,
        "program-header-table",
        header.program_offset,
        program_bytes,
        header.program_count,
    )?);
    if let Some((offset, size)) = programs.dynamic_span {
        tables.push(table_evidence(
            bytes,
            "dynamic-table",
            offset,
            size,
            dynamic_entry_count,
        )?);
    }
    tables.push(table_evidence(
        bytes,
        "section-name-table",
        sections.name_table_span.0,
        sections.name_table_span.1,
        sections.name_count,
    )?);
    tables.push(table_evidence(
        bytes,
        "section-header-table",
        header.section_offset,
        section_bytes,
        sections.records.len(),
    )?);
    Ok(ParsedElfAmd64ShellImage {
        tables,
        program_header_count: programs.records.len(),
        load_segment_count: programs.load_count,
        dynamic_segment_count: usize::from(programs.dynamic_span.is_some()),
        dynamic_entry_count,
        section_header_count: sections.records.len(),
        section_name_count: sections.name_count,
        entry_program_header_index: programs.entry_index,
    })
}

fn parse_header(
    bytes: &[u8],
    shell: &ElfAmd64ShellLayoutPlanReport,
) -> Result<ParsedHeader, String> {
    let header = checked_slice(bytes, 0, ELF64_HEADER_SIZE, "ELF64 header")?;
    if header.get(..4) != Some(b"\x7fELF")
        || header[4] != 2
        || header[5] != 1
        || header[6] != 1
        || header[7..16].iter().any(|byte| *byte != 0)
    {
        return Err("ELF shell validation rejects the ELF64 ident".to_owned());
    }
    if read_u16(header, 16, "file type")? != ET_EXEC
        || read_u16(header, 18, "machine")? != EM_X86_64
        || read_u32(header, 20, "version")? != 1
        || read_u32(header, 48, "flags")? != 0
    {
        return Err("ELF shell validation rejects the executable header identity".to_owned());
    }
    let entry = read_u64(header, 24, "entry")?;
    let program_offset = u64_usize(read_u64(header, 32, "program offset")?, "program offset")?;
    let section_offset = u64_usize(read_u64(header, 40, "section offset")?, "section offset")?;
    let header_size = usize::from(read_u16(header, 52, "header size")?);
    let program_entry_size = usize::from(read_u16(header, 54, "program entry size")?);
    let program_count = usize::from(read_u16(header, 56, "program count")?);
    let section_entry_size = usize::from(read_u16(header, 58, "section entry size")?);
    let section_count = usize::from(read_u16(header, 60, "section count")?);
    let section_name_index = usize::from(read_u16(header, 62, "section-name index")?);
    if entry != shell.entry_virtual_address
        || program_offset != shell.program_header_table_file_offset
        || section_offset != shell.section_header_table_file_offset
        || header_size != shell.elf_header_size_bytes
        || program_entry_size != shell.program_header_entry_size_bytes
        || program_count != shell.program_header_count
        || section_entry_size != shell.section_header_entry_size_bytes
        || section_count != shell.section_header_count
        || section_name_index != shell.section_name_table_section_index
        || header_size != ELF64_HEADER_SIZE
        || program_entry_size != ELF64_PROGRAM_HEADER_SIZE
        || section_entry_size != ELF64_SECTION_HEADER_SIZE
        || program_count == 0
        || section_count == 0
        || section_name_index >= section_count
    {
        return Err("ELF shell validation header/layout drift".to_owned());
    }
    checked_table_span(
        bytes,
        program_offset,
        program_entry_size,
        program_count,
        "program-header table",
    )?;
    checked_table_span(
        bytes,
        section_offset,
        section_entry_size,
        section_count,
        "section-header table",
    )?;
    Ok(ParsedHeader {
        program_offset,
        program_count,
        section_offset,
        section_count,
        section_name_index,
    })
}

fn parse_program_headers(
    bytes: &[u8],
    shell: &ElfAmd64ShellLayoutPlanReport,
    header: &ParsedHeader,
) -> Result<ParsedProgramHeaders, String> {
    let mut programs = Vec::with_capacity(header.program_count);
    let mut dynamic_span = None;
    let mut entry_index = None;
    let mut load_count = 0usize;
    for index in 0..header.program_count {
        let offset = table_entry_offset(
            header.program_offset,
            index,
            ELF64_PROGRAM_HEADER_SIZE,
            "program header",
        )?;
        let program = ProgramHeader {
            program_type: read_u32(bytes, offset, "program type")?,
            flags: read_u32(bytes, offset + 4, "program flags")?,
            file_offset: read_usize_u64(bytes, offset + 8, "program file offset")?,
            virtual_address: read_u64(bytes, offset + 16, "program virtual address")?,
            physical_address: read_u64(bytes, offset + 24, "program physical address")?,
            file_size: read_usize_u64(bytes, offset + 32, "program file size")?,
            memory_size: read_usize_u64(bytes, offset + 40, "program memory size")?,
            alignment: read_usize_u64(bytes, offset + 48, "program alignment")?,
        };
        validate_program_against_plan(&program, &shell.program_headers[index], index)?;
        checked_slice(
            bytes,
            program.file_offset,
            program.file_size,
            "program file range",
        )?;
        if program.memory_size < program.file_size
            || program.alignment == 0
            || !program.alignment.is_power_of_two()
            || program.file_offset as u64 % program.alignment as u64
                != program.virtual_address % program.alignment as u64
        {
            return Err(format!(
                "ELF shell program header {index} has an invalid envelope"
            ));
        }
        if program.program_type == PT_LOAD {
            load_count += 1;
            let file_backed_end = program
                .virtual_address
                .checked_add(usize_u64(program.file_size, "load file size")?)
                .ok_or_else(|| "ELF shell executable load range overflows".to_owned())?;
            if program.flags & PF_X != 0
                && (program.virtual_address..file_backed_end).contains(&shell.entry_virtual_address)
                && entry_index.replace(index).is_some()
            {
                return Err("ELF shell entry maps to multiple executable loads".to_owned());
            }
        }
        if program.program_type == PT_DYNAMIC
            && dynamic_span
                .replace((program.file_offset, program.file_size))
                .is_some()
        {
            return Err("ELF shell image has multiple PT_DYNAMIC records".to_owned());
        }
        programs.push(program);
    }
    let entry_index = entry_index
        .ok_or_else(|| "ELF shell entry is outside file-backed executable PT_LOAD".to_owned())?;
    Ok(ParsedProgramHeaders {
        records: programs,
        load_count,
        dynamic_span,
        entry_index,
    })
}

fn validate_program_against_plan(
    actual: &ProgramHeader,
    expected: &super::report::ElfAmd64ShellProgramHeaderPlan,
    index: usize,
) -> Result<(), String> {
    if actual.program_type != expected.program_type
        || actual.flags != expected.flags
        || actual.file_offset != expected.file_offset
        || actual.virtual_address != expected.virtual_address
        || actual.physical_address != expected.virtual_address
        || actual.file_size != expected.file_size_bytes
        || actual.memory_size != expected.memory_size_bytes
        || actual.alignment != expected.alignment
    {
        return Err(format!(
            "ELF shell program header {index} differs from its plan"
        ));
    }
    Ok(())
}

fn parse_section_headers(
    bytes: &[u8],
    shell: &ElfAmd64ShellLayoutPlanReport,
    header: &ParsedHeader,
) -> Result<ParsedSectionHeaders, String> {
    let mut sections = Vec::with_capacity(header.section_count);
    for index in 0..header.section_count {
        let offset = table_entry_offset(
            header.section_offset,
            index,
            ELF64_SECTION_HEADER_SIZE,
            "section header",
        )?;
        let section = SectionHeader {
            name_offset: usize::try_from(read_u32(bytes, offset, "section name offset")?)
                .map_err(|_| "ELF section name offset exceeds usize".to_owned())?,
            section_type: read_u32(bytes, offset + 4, "section type")?,
            flags: read_u64(bytes, offset + 8, "section flags")?,
            virtual_address: read_u64(bytes, offset + 16, "section address")?,
            file_offset: read_usize_u64(bytes, offset + 24, "section file offset")?,
            size: read_usize_u64(bytes, offset + 32, "section size")?,
            link: usize::try_from(read_u32(bytes, offset + 40, "section link")?)
                .map_err(|_| "ELF section link exceeds usize".to_owned())?,
            info: usize::try_from(read_u32(bytes, offset + 44, "section info")?)
                .map_err(|_| "ELF section info exceeds usize".to_owned())?,
            alignment: read_usize_u64(bytes, offset + 48, "section alignment")?,
            entry_size: read_usize_u64(bytes, offset + 56, "section entry size")?,
        };
        validate_section_against_plan(&section, &shell.sections[index], index)?;
        if section.section_type != SHT_NOBITS {
            checked_slice(
                bytes,
                section.file_offset,
                section.size,
                "section file range",
            )?;
        } else if section.file_offset > bytes.len() {
            return Err(format!(
                "ELF shell NOBITS section {index} has an invalid offset"
            ));
        }
        sections.push(section);
    }
    let names = sections
        .get(header.section_name_index)
        .ok_or_else(|| "ELF shell section-name header is absent".to_owned())?;
    if names.section_type != SHT_STRTAB {
        return Err("ELF shell section-name table is not SHT_STRTAB".to_owned());
    }
    let string_table = checked_slice(
        bytes,
        names.file_offset,
        names.size,
        "section-name string table",
    )?;
    let section_name_span = (names.file_offset, names.size);
    for (index, section) in sections.iter().enumerate() {
        let name = read_string(string_table, section.name_offset, "section name")?;
        if name != shell.sections[index].section_name {
            return Err(format!(
                "ELF shell section name {index} differs from its plan"
            ));
        }
    }
    Ok(ParsedSectionHeaders {
        records: sections,
        name_table_span: section_name_span,
        name_count: header.section_count,
    })
}

fn validate_section_against_plan(
    actual: &SectionHeader,
    expected: &super::report::ElfAmd64ShellSectionPlan,
    index: usize,
) -> Result<(), String> {
    let expected_size = if expected.section_type == SHT_NOBITS {
        expected.memory_size_bytes
    } else {
        expected.file_size_bytes
    };
    if actual.name_offset != expected.section_name_offset
        || actual.section_type != expected.section_type
        || actual.flags != expected.flags
        || actual.virtual_address != expected.virtual_address
        || actual.file_offset != expected.file_offset
        || actual.size != expected_size
        || actual.link != expected.link_section_index
        || actual.info != expected.info_section_index
        || actual.alignment != expected.alignment
        || actual.entry_size != expected.entry_size
    {
        return Err(format!(
            "ELF shell section header {index} differs from its plan"
        ));
    }
    Ok(())
}

fn parse_dynamic_entries(
    bytes: &[u8],
    shell: &ElfAmd64ShellLayoutPlanReport,
    dynamic_span: Option<(usize, usize)>,
) -> Result<usize, String> {
    let Some((offset, size)) = dynamic_span else {
        if shell.dynamic_entries.is_empty() {
            return Ok(0);
        }
        return Err("ELF shell dynamic plan has no PT_DYNAMIC".to_owned());
    };
    if size == 0 || size % ELF64_DYNAMIC_ENTRY_SIZE != 0 {
        return Err("ELF shell PT_DYNAMIC has an invalid byte size".to_owned());
    }
    let count = size / ELF64_DYNAMIC_ENTRY_SIZE;
    if count != shell.dynamic_entries.len() {
        return Err("ELF shell dynamic entry count differs from its plan".to_owned());
    }
    for (index, expected) in shell.dynamic_entries.iter().enumerate() {
        let entry_offset = table_entry_offset(offset, index, ELF64_DYNAMIC_ENTRY_SIZE, "dynamic")?;
        let tag = read_i64(bytes, entry_offset, "dynamic tag")?;
        let value = read_u64(bytes, entry_offset + 8, "dynamic value")?;
        if tag != expected.tag || value != expected.value || (tag == 0) != (index + 1 == count) {
            return Err(format!(
                "ELF shell dynamic entry {index} differs from its plan"
            ));
        }
    }
    Ok(count)
}

fn validate_load_nonoverlap(programs: &[ProgramHeader]) -> Result<(), String> {
    let loads = programs
        .iter()
        .filter(|program| program.program_type == PT_LOAD)
        .collect::<Vec<_>>();
    for (index, load) in loads.iter().enumerate() {
        for other in loads.iter().skip(index + 1) {
            if ranges_overlap(
                load.file_offset,
                load.file_size,
                other.file_offset,
                other.file_size,
            )? || ranges_overlap_u64(
                load.virtual_address,
                load.memory_size,
                other.virtual_address,
                other.memory_size,
            )? {
                return Err("ELF shell parsed PT_LOAD records overlap".to_owned());
            }
        }
    }
    Ok(())
}

fn table_evidence(
    bytes: &[u8],
    table_kind: &'static str,
    offset: usize,
    size: usize,
    record_count: usize,
) -> Result<ParsedTableEvidence, String> {
    let table = checked_slice(bytes, offset, size, table_kind)?;
    Ok(ParsedTableEvidence {
        table_kind,
        file_offset: offset,
        width_bytes: size,
        record_count,
        bytes_hash: crate::fnv1a64_hex(table),
    })
}

fn read_string<'a>(bytes: &'a [u8], offset: usize, label: &str) -> Result<&'a str, String> {
    let tail = bytes
        .get(offset..)
        .ok_or_else(|| format!("ELF shell {label} offset is out of range"))?;
    let end = tail
        .iter()
        .position(|byte| *byte == 0)
        .ok_or_else(|| format!("ELF shell {label} is not terminated"))?;
    std::str::from_utf8(&tail[..end]).map_err(|_| format!("ELF shell {label} is not UTF-8"))
}

fn checked_table_span(
    bytes: &[u8],
    offset: usize,
    entry_size: usize,
    count: usize,
    label: &str,
) -> Result<(), String> {
    let size = checked_mul(count, entry_size, label)?;
    checked_slice(bytes, offset, size, label).map(|_| ())
}

fn table_entry_offset(
    table_offset: usize,
    index: usize,
    entry_size: usize,
    label: &str,
) -> Result<usize, String> {
    table_offset
        .checked_add(checked_mul(index, entry_size, label)?)
        .ok_or_else(|| format!("ELF shell {label} offset overflows"))
}

fn checked_slice<'a>(
    bytes: &'a [u8],
    offset: usize,
    size: usize,
    label: &str,
) -> Result<&'a [u8], String> {
    let end = offset
        .checked_add(size)
        .ok_or_else(|| format!("ELF shell {label} span overflows"))?;
    bytes
        .get(offset..end)
        .ok_or_else(|| format!("ELF shell {label} exceeds image bounds"))
}

fn checked_mul(lhs: usize, rhs: usize, label: &str) -> Result<usize, String> {
    lhs.checked_mul(rhs)
        .ok_or_else(|| format!("ELF shell {label} size overflows"))
}

fn ranges_overlap(
    lhs: usize,
    lhs_size: usize,
    rhs: usize,
    rhs_size: usize,
) -> Result<bool, String> {
    if lhs_size == 0 || rhs_size == 0 {
        return Ok(false);
    }
    let lhs_end = lhs
        .checked_add(lhs_size)
        .ok_or_else(|| "ELF shell file range overflows".to_owned())?;
    let rhs_end = rhs
        .checked_add(rhs_size)
        .ok_or_else(|| "ELF shell file range overflows".to_owned())?;
    Ok(lhs < rhs_end && rhs < lhs_end)
}

fn ranges_overlap_u64(
    lhs: u64,
    lhs_size: usize,
    rhs: u64,
    rhs_size: usize,
) -> Result<bool, String> {
    if lhs_size == 0 || rhs_size == 0 {
        return Ok(false);
    }
    let lhs_end = lhs
        .checked_add(usize_u64(lhs_size, "VM range")?)
        .ok_or_else(|| "ELF shell VM range overflows".to_owned())?;
    let rhs_end = rhs
        .checked_add(usize_u64(rhs_size, "VM range")?)
        .ok_or_else(|| "ELF shell VM range overflows".to_owned())?;
    Ok(lhs < rhs_end && rhs < lhs_end)
}

fn read_usize_u64(bytes: &[u8], offset: usize, label: &str) -> Result<usize, String> {
    u64_usize(read_u64(bytes, offset, label)?, label)
}

fn u64_usize(value: u64, label: &str) -> Result<usize, String> {
    usize::try_from(value).map_err(|_| format!("ELF shell {label} exceeds host space"))
}

fn usize_u64(value: usize, label: &str) -> Result<u64, String> {
    u64::try_from(value).map_err(|_| format!("ELF shell {label} exceeds u64"))
}

fn read_u16(bytes: &[u8], offset: usize, label: &str) -> Result<u16, String> {
    let raw: [u8; 2] = checked_slice(bytes, offset, 2, label)?
        .try_into()
        .map_err(|_| format!("ELF shell {label} is malformed"))?;
    Ok(u16::from_le_bytes(raw))
}

fn read_u32(bytes: &[u8], offset: usize, label: &str) -> Result<u32, String> {
    let raw: [u8; 4] = checked_slice(bytes, offset, 4, label)?
        .try_into()
        .map_err(|_| format!("ELF shell {label} is malformed"))?;
    Ok(u32::from_le_bytes(raw))
}

fn read_u64(bytes: &[u8], offset: usize, label: &str) -> Result<u64, String> {
    let raw: [u8; 8] = checked_slice(bytes, offset, 8, label)?
        .try_into()
        .map_err(|_| format!("ELF shell {label} is malformed"))?;
    Ok(u64::from_le_bytes(raw))
}

fn read_i64(bytes: &[u8], offset: usize, label: &str) -> Result<i64, String> {
    let raw: [u8; 8] = checked_slice(bytes, offset, 8, label)?
        .try_into()
        .map_err(|_| format!("ELF shell {label} is malformed"))?;
    Ok(i64::from_le_bytes(raw))
}
