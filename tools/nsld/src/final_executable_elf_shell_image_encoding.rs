use super::{
    layout::{
        ELF64_DYNAMIC_ENTRY_SIZE, ELF64_HEADER_SIZE, ELF64_PROGRAM_HEADER_SIZE,
        ELF64_SECTION_HEADER_SIZE,
    },
    report::ElfAmd64ShellLayoutPlanReport,
};

const ELF_CLASS_64: u8 = 2;
const ELF_DATA_LITTLE_ENDIAN: u8 = 1;
const ELF_VERSION_CURRENT: u8 = 1;
const ELF_TYPE_EXECUTABLE: u16 = 2;
const ELF_MACHINE_X86_64: u16 = 62;
const ELF_SECTION_TYPE_NOBITS: u32 = 8;

pub(super) struct EncodedElfAmd64ShellTables {
    pub(super) header: Vec<u8>,
    pub(super) program_headers: Vec<u8>,
    pub(super) dynamic_entries: Vec<u8>,
    pub(super) section_names: Vec<u8>,
    pub(super) section_headers: Vec<u8>,
}

pub(super) fn encode_elf_amd64_shell_tables(
    shell: &ElfAmd64ShellLayoutPlanReport,
) -> Result<EncodedElfAmd64ShellTables, String> {
    validate_table_shape(shell)?;
    let encoded = EncodedElfAmd64ShellTables {
        header: encode_header(shell)?,
        program_headers: encode_program_headers(shell)?,
        dynamic_entries: encode_dynamic_entries(shell),
        section_names: encode_section_names(shell)?,
        section_headers: encode_section_headers(shell)?,
    };
    if encoded.header.len() != shell.elf_header_size_bytes
        || encoded.program_headers.len() != shell.program_header_table_bytes
        || encoded.dynamic_entries.len() != shell.dynamic_table_bytes
        || encoded.section_names.len() != shell.section_name_table_bytes
        || encoded.section_headers.len()
            != checked_mul(
                shell.section_header_count,
                shell.section_header_entry_size_bytes,
                "section-header table",
            )?
    {
        return Err("ELF shell encoded table size drift".to_owned());
    }
    Ok(encoded)
}

fn validate_table_shape(shell: &ElfAmd64ShellLayoutPlanReport) -> Result<(), String> {
    if shell.elf_header_file_offset != 0
        || shell.elf_header_size_bytes != ELF64_HEADER_SIZE
        || shell.program_header_table_file_offset != ELF64_HEADER_SIZE
        || shell.program_header_entry_size_bytes != ELF64_PROGRAM_HEADER_SIZE
        || shell.section_header_entry_size_bytes != ELF64_SECTION_HEADER_SIZE
        || shell.dynamic_table_entry_size_bytes != ELF64_DYNAMIC_ENTRY_SIZE
    {
        return Err("ELF shell encoder rejects table ABI drift".to_owned());
    }
    if shell.program_header_count != shell.program_headers.len()
        || shell.section_header_count != shell.sections.len()
        || shell.dynamic_table_entry_count != shell.dynamic_entries.len()
        || shell.program_header_table_bytes
            != checked_mul(
                shell.program_header_count,
                ELF64_PROGRAM_HEADER_SIZE,
                "program-header table",
            )?
        || shell.dynamic_table_bytes
            != checked_mul(
                shell.dynamic_table_entry_count,
                ELF64_DYNAMIC_ENTRY_SIZE,
                "dynamic table",
            )?
    {
        return Err("ELF shell encoder rejects table count drift".to_owned());
    }
    for (index, header) in shell.program_headers.iter().enumerate() {
        if header.program_header_index != index {
            return Err("ELF shell encoder rejects program-header ordering drift".to_owned());
        }
    }
    for (index, section) in shell.sections.iter().enumerate() {
        if section.section_index != index {
            return Err("ELF shell encoder rejects section ordering drift".to_owned());
        }
    }
    for (index, entry) in shell.dynamic_entries.iter().enumerate() {
        if entry.dynamic_entry_index != index {
            return Err("ELF shell encoder rejects dynamic-entry ordering drift".to_owned());
        }
    }
    let section_names = shell
        .sections
        .get(shell.section_name_table_section_index)
        .ok_or_else(|| "ELF shell section-name table index is out of range".to_owned())?;
    if section_names.section_name != ".shstrtab"
        || section_names.file_offset != shell.section_name_table_file_offset
        || section_names.file_size_bytes != shell.section_name_table_bytes
    {
        return Err("ELF shell section-name table coordinate drift".to_owned());
    }
    let dynamic_shape = (
        shell.dynamic_table_file_offset,
        shell.dynamic_table_virtual_address,
        shell.dynamic_entries.is_empty(),
    );
    if !matches!(
        dynamic_shape,
        (None, None, true) | (Some(_), Some(_), false)
    ) || (!shell.dynamic_entries.is_empty()
        && shell
            .dynamic_entries
            .last()
            .is_none_or(|entry| entry.tag != 0))
    {
        return Err("ELF shell dynamic table boundary drift".to_owned());
    }
    Ok(())
}

fn encode_header(shell: &ElfAmd64ShellLayoutPlanReport) -> Result<Vec<u8>, String> {
    let mut out = Vec::with_capacity(ELF64_HEADER_SIZE);
    out.extend_from_slice(&[
        0x7f,
        b'E',
        b'L',
        b'F',
        ELF_CLASS_64,
        ELF_DATA_LITTLE_ENDIAN,
        ELF_VERSION_CURRENT,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    ]);
    push_u16(&mut out, ELF_TYPE_EXECUTABLE);
    push_u16(&mut out, ELF_MACHINE_X86_64);
    push_u32(&mut out, u32::from(ELF_VERSION_CURRENT));
    push_u64(&mut out, shell.entry_virtual_address);
    push_u64(
        &mut out,
        usize_u64(
            shell.program_header_table_file_offset,
            "program-header offset",
        )?,
    );
    push_u64(
        &mut out,
        usize_u64(
            shell.section_header_table_file_offset,
            "section-header offset",
        )?,
    );
    push_u32(&mut out, 0);
    push_u16(
        &mut out,
        usize_u16(shell.elf_header_size_bytes, "ELF header size")?,
    );
    push_u16(
        &mut out,
        usize_u16(
            shell.program_header_entry_size_bytes,
            "program-header entry size",
        )?,
    );
    push_u16(
        &mut out,
        usize_u16(shell.program_header_count, "program-header count")?,
    );
    push_u16(
        &mut out,
        usize_u16(
            shell.section_header_entry_size_bytes,
            "section-header entry size",
        )?,
    );
    push_u16(
        &mut out,
        usize_u16(shell.section_header_count, "section-header count")?,
    );
    push_u16(
        &mut out,
        usize_u16(
            shell.section_name_table_section_index,
            "section-name table index",
        )?,
    );
    if out.len() != ELF64_HEADER_SIZE {
        return Err("ELF shell header encoder produced an invalid size".to_owned());
    }
    Ok(out)
}

fn encode_program_headers(shell: &ElfAmd64ShellLayoutPlanReport) -> Result<Vec<u8>, String> {
    let mut out = Vec::with_capacity(shell.program_header_table_bytes);
    for header in &shell.program_headers {
        push_u32(&mut out, header.program_type);
        push_u32(&mut out, header.flags);
        push_u64(
            &mut out,
            usize_u64(header.file_offset, "segment file offset")?,
        );
        push_u64(&mut out, header.virtual_address);
        push_u64(&mut out, header.virtual_address);
        push_u64(
            &mut out,
            usize_u64(header.file_size_bytes, "segment file size")?,
        );
        push_u64(
            &mut out,
            usize_u64(header.memory_size_bytes, "segment memory size")?,
        );
        push_u64(&mut out, usize_u64(header.alignment, "segment alignment")?);
    }
    Ok(out)
}

fn encode_dynamic_entries(shell: &ElfAmd64ShellLayoutPlanReport) -> Vec<u8> {
    let mut out = Vec::with_capacity(shell.dynamic_table_bytes);
    for entry in &shell.dynamic_entries {
        out.extend_from_slice(&entry.tag.to_le_bytes());
        push_u64(&mut out, entry.value);
    }
    out
}

fn encode_section_names(shell: &ElfAmd64ShellLayoutPlanReport) -> Result<Vec<u8>, String> {
    let mut out = vec![0];
    for section in shell.sections.iter().skip(1) {
        if section.section_name.is_empty() || section.section_name.as_bytes().contains(&0) {
            return Err(format!(
                "ELF shell section `{}` has an invalid name",
                section.section_id
            ));
        }
        if section.section_name_offset != out.len() {
            return Err(format!(
                "ELF shell section `{}` name offset drift",
                section.section_id
            ));
        }
        out.extend_from_slice(section.section_name.as_bytes());
        out.push(0);
    }
    Ok(out)
}

fn encode_section_headers(shell: &ElfAmd64ShellLayoutPlanReport) -> Result<Vec<u8>, String> {
    let capacity = checked_mul(
        shell.section_header_count,
        ELF64_SECTION_HEADER_SIZE,
        "section-header table",
    )?;
    let mut out = Vec::with_capacity(capacity);
    for section in &shell.sections {
        push_u32(
            &mut out,
            usize_u32(section.section_name_offset, "section-name offset")?,
        );
        push_u32(&mut out, section.section_type);
        push_u64(&mut out, section.flags);
        push_u64(&mut out, section.virtual_address);
        push_u64(
            &mut out,
            usize_u64(section.file_offset, "section file offset")?,
        );
        let section_size = if section.section_type == ELF_SECTION_TYPE_NOBITS {
            section.memory_size_bytes
        } else {
            section.file_size_bytes
        };
        push_u64(&mut out, usize_u64(section_size, "section size")?);
        push_u32(
            &mut out,
            usize_u32(section.link_section_index, "section link index")?,
        );
        push_u32(
            &mut out,
            usize_u32(section.info_section_index, "section info index")?,
        );
        push_u64(&mut out, usize_u64(section.alignment, "section alignment")?);
        push_u64(
            &mut out,
            usize_u64(section.entry_size, "section entry size")?,
        );
    }
    Ok(out)
}

fn checked_mul(lhs: usize, rhs: usize, label: &str) -> Result<usize, String> {
    lhs.checked_mul(rhs)
        .ok_or_else(|| format!("ELF shell {label} size overflows"))
}

fn usize_u16(value: usize, label: &str) -> Result<u16, String> {
    u16::try_from(value).map_err(|_| format!("ELF shell {label} exceeds u16"))
}

fn usize_u32(value: usize, label: &str) -> Result<u32, String> {
    u32::try_from(value).map_err(|_| format!("ELF shell {label} exceeds u32"))
}

fn usize_u64(value: usize, label: &str) -> Result<u64, String> {
    u64::try_from(value).map_err(|_| format!("ELF shell {label} exceeds u64"))
}

fn push_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn push_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn push_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}
