use super::super::report::{
    program_header_audit_hash, ElfAmd64ShellProgramHeaderPlan, ElfAmd64ShellSectionPlan,
};
use super::{
    support::section_id, virtual_address, LoadSeed, ELF64_HEADER_SIZE, PF_R, PF_W, PT_DYNAMIC,
    PT_INTERP, PT_LOAD, PT_PHDR,
};
use std::collections::BTreeMap;

#[allow(clippy::too_many_arguments)]
pub(super) fn build_program_headers(
    loads: Vec<LoadSeed>,
    segment_sections: &BTreeMap<String, Vec<String>>,
    sections: &mut [ElfAmd64ShellSectionPlan],
    interpreter_offset: Option<usize>,
    interpreter_virtual: Option<u64>,
    interpreter_bytes: usize,
    dynamic_offset: Option<usize>,
    dynamic_virtual: Option<u64>,
    dynamic_bytes: usize,
    program_header_table_bytes: usize,
    ledger_hash: &str,
) -> Result<Vec<ElfAmd64ShellProgramHeaderPlan>, String> {
    let mut headers = Vec::new();
    push_program_header(
        &mut headers,
        "program-header-table",
        PT_PHDR,
        "read-only-metadata",
        PF_R,
        ELF64_HEADER_SIZE,
        virtual_address(ELF64_HEADER_SIZE)?,
        program_header_table_bytes,
        program_header_table_bytes,
        8,
        Vec::new(),
        ledger_hash,
    );
    if let (Some(offset), Some(virtual_address)) = (interpreter_offset, interpreter_virtual) {
        let interpreter_id = section_id(sections, ".interp")?.to_owned();
        push_program_header(
            &mut headers,
            "interpreter",
            PT_INTERP,
            "read-only-interpreter",
            PF_R,
            offset,
            virtual_address,
            interpreter_bytes,
            interpreter_bytes,
            1,
            vec![interpreter_id],
            ledger_hash,
        );
    }
    for load in loads {
        let section_ids = segment_sections
            .get(&load.segment_key)
            .cloned()
            .unwrap_or_default();
        let index = headers.len();
        let id = format!("elf-amd64-shell-program-header-{index:04}");
        for section in sections
            .iter_mut()
            .filter(|section| section_ids.contains(&section.section_id))
        {
            section.load_segment_id = Some(id.clone());
        }
        push_program_header_with_id(
            &mut headers,
            id,
            "load",
            PT_LOAD,
            load.permission_class,
            load.flags,
            load.file_offset,
            load.virtual_address,
            load.file_size_bytes,
            load.memory_size_bytes,
            load.alignment,
            section_ids,
            ledger_hash,
        );
    }
    if let (Some(offset), Some(virtual_address)) = (dynamic_offset, dynamic_virtual) {
        let dynamic_id = section_id(sections, ".dynamic")?.to_owned();
        push_program_header(
            &mut headers,
            "dynamic-table",
            PT_DYNAMIC,
            "read-write-dynamic",
            PF_R | PF_W,
            offset,
            virtual_address,
            dynamic_bytes,
            dynamic_bytes,
            8,
            vec![dynamic_id],
            ledger_hash,
        );
    }
    Ok(headers)
}

#[allow(clippy::too_many_arguments)]
fn push_program_header(
    headers: &mut Vec<ElfAmd64ShellProgramHeaderPlan>,
    program_kind: &str,
    program_type: u32,
    permission_class: &str,
    flags: u32,
    file_offset: usize,
    virtual_address: u64,
    file_size_bytes: usize,
    memory_size_bytes: usize,
    alignment: usize,
    section_ids: Vec<String>,
    ledger_hash: &str,
) {
    let index = headers.len();
    let id = format!("elf-amd64-shell-program-header-{index:04}");
    push_program_header_with_id(
        headers,
        id,
        program_kind,
        program_type,
        permission_class,
        flags,
        file_offset,
        virtual_address,
        file_size_bytes,
        memory_size_bytes,
        alignment,
        section_ids,
        ledger_hash,
    );
}

#[allow(clippy::too_many_arguments)]
fn push_program_header_with_id(
    headers: &mut Vec<ElfAmd64ShellProgramHeaderPlan>,
    id: String,
    program_kind: &str,
    program_type: u32,
    permission_class: &str,
    flags: u32,
    file_offset: usize,
    virtual_address: u64,
    file_size_bytes: usize,
    memory_size_bytes: usize,
    alignment: usize,
    section_ids: Vec<String>,
    ledger_hash: &str,
) {
    let mut header = ElfAmd64ShellProgramHeaderPlan {
        program_header_id: id,
        program_header_index: headers.len(),
        program_kind: program_kind.to_owned(),
        program_type,
        permission_class: permission_class.to_owned(),
        flags,
        file_offset,
        virtual_address,
        file_size_bytes,
        memory_size_bytes,
        alignment,
        section_ids,
        audit_hash: String::new(),
    };
    header.audit_hash = program_header_audit_hash(ledger_hash, &header);
    headers.push(header);
}
