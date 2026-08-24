#[path = "final_executable_elf_shell_program_headers.rs"]
mod program_headers;
#[path = "final_executable_elf_shell_layout_support.rs"]
mod support;

use super::{
    dynamic::{build_elf_amd64_shell_dynamic_layout, ElfAmd64ShellDynamicLayout},
    report::{
        section_audit_hash, ElfAmd64ShellDynamicEntryPlan, ElfAmd64ShellNeededLibraryPlan,
        ElfAmd64ShellProgramHeaderPlan, ElfAmd64ShellSectionPlan,
    },
    version::{
        ElfAmd64ShellVersionNeedPlan, ElfAmd64ShellVersionSymbolPlan,
        ELF64_VERSION_SYMBOL_ENTRY_SIZE,
    },
};
use crate::{
    final_executable_elf_dynamic_plan::ElfAmd64DynamicDependencyPlanReport,
    final_executable_elf_layout::{ELF_AMD64_IMAGE_BASE, ELF_AMD64_PAGE_SIZE},
    final_executable_elf_layout_report::ElfAmd64PlacementBindingReport,
    final_executable_elf_materialization::application::platform::ElfAmd64PlatformStructurePlanReport,
};
use std::collections::{BTreeMap, BTreeSet};

use program_headers::build_program_headers;
pub(super) use support::locate_source_coordinate;
use support::{
    assign_section_links, build_dynamic_entries, section_index, validate_layout,
    validate_platform_regions,
};

pub(super) const ELF64_HEADER_SIZE: usize = 64;
pub(super) const ELF64_PROGRAM_HEADER_SIZE: usize = 56;
pub(super) const ELF64_SECTION_HEADER_SIZE: usize = 64;
pub(super) const ELF64_DYNAMIC_ENTRY_SIZE: usize = 16;

const PT_LOAD: u32 = 1;
const PT_DYNAMIC: u32 = 2;
const PT_INTERP: u32 = 3;
const PT_PHDR: u32 = 6;
const PF_X: u32 = 1;
const PF_W: u32 = 2;
const PF_R: u32 = 4;
const SHF_WRITE: u64 = 1;
const SHF_ALLOC: u64 = 2;
const SHF_EXECINSTR: u64 = 4;
const SHT_NULL: u32 = 0;
const SHT_PROGBITS: u32 = 1;
const SHT_STRTAB: u32 = 3;
const SHT_RELA: u32 = 4;
const SHT_DYNAMIC: u32 = 6;
const SHT_NOBITS: u32 = 8;
const SHT_DYNSYM: u32 = 11;
const SHT_GNU_VERNEED: u32 = 0x6fff_fffe;
const SHT_GNU_VERSYM: u32 = 0x6fff_ffff;

struct SectionSeed {
    source_kind: &'static str,
    source_id: String,
    name: String,
    section_type: u32,
    flags: u64,
    alignment: usize,
    entry_size: usize,
    source_image_offset: Option<usize>,
    source_size_bytes: usize,
    file_offset: usize,
    file_size_bytes: usize,
    virtual_address: u64,
    memory_size_bytes: usize,
    segment_key: Option<String>,
}

struct LoadSeed {
    segment_key: String,
    permission_class: &'static str,
    flags: u32,
    file_offset: usize,
    virtual_address: u64,
    file_size_bytes: usize,
    memory_size_bytes: usize,
    alignment: usize,
}

struct BuiltSections {
    sections: Vec<ElfAmd64ShellSectionPlan>,
    segment_sections: BTreeMap<String, Vec<String>>,
    string_table_bytes: usize,
}

pub(super) struct ElfAmd64ShellLayoutDraft {
    pub(super) sections: Vec<ElfAmd64ShellSectionPlan>,
    pub(super) program_headers: Vec<ElfAmd64ShellProgramHeaderPlan>,
    pub(super) dynamic_entries: Vec<ElfAmd64ShellDynamicEntryPlan>,
    pub(super) program_header_table_bytes: usize,
    pub(super) section_header_table_file_offset: usize,
    pub(super) section_header_count: usize,
    pub(super) section_name_table_section_index: usize,
    pub(super) section_name_table_file_offset: usize,
    pub(super) section_name_table_bytes: usize,
    pub(super) planned_file_span_bytes: usize,
    pub(super) planned_memory_span_bytes: usize,
    pub(super) load_segment_count: usize,
    pub(super) dynamic_table_file_offset: Option<usize>,
    pub(super) dynamic_table_virtual_address: Option<u64>,
    pub(super) dynamic_table_bytes: usize,
    pub(super) dynamic_dependency_plan_hash: Option<String>,
    pub(super) interpreter_identity: Option<String>,
    pub(super) interpreter_path: Option<String>,
    pub(super) interpreter_file_offset: Option<usize>,
    pub(super) interpreter_virtual_address: Option<u64>,
    pub(super) interpreter_bytes: usize,
    pub(super) dynamic_string_source_image_offset: Option<usize>,
    pub(super) dynamic_string_source_bytes: usize,
    pub(super) needed_libraries: Vec<ElfAmd64ShellNeededLibraryPlan>,
    pub(super) version_symbol_table_file_offset: Option<usize>,
    pub(super) version_symbol_table_virtual_address: Option<u64>,
    pub(super) version_symbol_table_bytes: usize,
    pub(super) version_need_table_file_offset: Option<usize>,
    pub(super) version_need_table_virtual_address: Option<u64>,
    pub(super) version_need_table_bytes: usize,
    pub(super) version_symbols: Vec<ElfAmd64ShellVersionSymbolPlan>,
    pub(super) version_needs: Vec<ElfAmd64ShellVersionNeedPlan>,
}

pub(super) struct LocatedSourceCoordinate<'a> {
    pub(super) section: &'a ElfAmd64ShellSectionPlan,
    pub(super) file_offset: usize,
    pub(super) virtual_address: u64,
}

pub(super) fn build_elf_amd64_shell_layout(
    placement: &ElfAmd64PlacementBindingReport,
    platform: &ElfAmd64PlatformStructurePlanReport,
    application_ledger_hash: &str,
    dependency_plan: Option<&ElfAmd64DynamicDependencyPlanReport>,
) -> Result<ElfAmd64ShellLayoutDraft, String> {
    let dynamic_enabled = platform.target_count > 0;
    validate_platform_regions(platform, dynamic_enabled, ELF_AMD64_PAGE_SIZE)?;
    let dynamic_layout = build_elf_amd64_shell_dynamic_layout(platform, dependency_plan)?;
    let dynamic_table_bytes = dynamic_layout.dynamic_table_bytes;
    let dynamic_table_file_offset = dynamic_layout.dynamic_table_file_offset;
    let dynamic_table_virtual_address = dynamic_layout.dynamic_table_virtual_address;
    let planned_memory_span_bytes = dynamic_layout.planned_memory_span_bytes;

    let mut seeds = section_seeds(
        placement,
        platform,
        dynamic_layout.emits_registered_dependencies(),
    )?;
    append_shell_dynamic_metadata_sections(&mut seeds, &dynamic_layout)?;
    if let (Some(file_offset), Some(virtual_address)) =
        (dynamic_table_file_offset, dynamic_table_virtual_address)
    {
        seeds.push(SectionSeed {
            source_kind: "shell-dynamic-table",
            source_id: "elf-amd64-shell-dynamic-table".to_owned(),
            name: ".dynamic".to_owned(),
            section_type: SHT_DYNAMIC,
            flags: SHF_ALLOC | SHF_WRITE,
            alignment: 8,
            entry_size: ELF64_DYNAMIC_ENTRY_SIZE,
            source_image_offset: None,
            source_size_bytes: 0,
            file_offset,
            file_size_bytes: dynamic_table_bytes,
            virtual_address,
            memory_size_bytes: dynamic_table_bytes,
            segment_key: Some("shell-dynamic".to_owned()),
        });
    }
    let alloc_file_end = dynamic_table_file_offset
        .map(|offset| checked_add(offset, dynamic_table_bytes, "dynamic table"))
        .transpose()?
        .unwrap_or(
            platform
                .planned_file_span_bytes
                .max(platform.planned_memory_span_bytes),
        );
    let shstrtab_offset = align_up(alloc_file_end, 8)?;
    seeds.push(SectionSeed {
        source_kind: "shell-section-names",
        source_id: "elf-amd64-shell-shstrtab".to_owned(),
        name: ".shstrtab".to_owned(),
        section_type: SHT_STRTAB,
        flags: 0,
        alignment: 1,
        entry_size: 0,
        source_image_offset: None,
        source_size_bytes: 0,
        file_offset: shstrtab_offset,
        file_size_bytes: 0,
        virtual_address: 0,
        memory_size_bytes: 0,
        segment_key: None,
    });

    let built_sections = build_sections(seeds, application_ledger_hash)?;
    let mut sections = built_sections.sections;
    let segment_sections = built_sections.segment_sections;
    let shstrtab_bytes = built_sections.string_table_bytes;
    let shstrtab_index = section_index(&sections, ".shstrtab")?;
    let shstrtab = sections
        .get_mut(shstrtab_index)
        .ok_or_else(|| "ELF shell section-name table disappeared".to_owned())?;
    shstrtab.file_size_bytes = shstrtab_bytes;
    shstrtab.source_size_bytes = shstrtab_bytes;
    shstrtab.audit_hash = section_audit_hash(application_ledger_hash, shstrtab);
    assign_section_links(&mut sections, dynamic_layout.version_needs.len());

    let section_header_table_file_offset = align_up(
        checked_add(shstrtab_offset, shstrtab_bytes, "section-name table")?,
        8,
    )?;
    let section_header_bytes = sections
        .len()
        .checked_mul(ELF64_SECTION_HEADER_SIZE)
        .ok_or_else(|| "ELF shell section-header table size overflows".to_owned())?;
    let planned_file_span_bytes = checked_add(
        section_header_table_file_offset,
        section_header_bytes,
        "section-header table",
    )?;

    let load_seeds = load_seeds(placement, platform, &dynamic_layout)?;
    let program_header_count = 1usize
        .checked_add(load_seeds.len())
        .and_then(|count| {
            count.checked_add(usize::from(
                dynamic_layout.interpreter_file_offset.is_some(),
            ))
        })
        .and_then(|count| count.checked_add(usize::from(dynamic_enabled)))
        .ok_or_else(|| "ELF shell program-header count overflows".to_owned())?;
    let program_header_table_bytes = program_header_count
        .checked_mul(ELF64_PROGRAM_HEADER_SIZE)
        .ok_or_else(|| "ELF shell program-header table size overflows".to_owned())?;
    if checked_add(
        ELF64_HEADER_SIZE,
        program_header_table_bytes,
        "program-header table",
    )? > placement.payload_file_offset
    {
        return Err("ELF shell program-header table exceeds reserved header page".to_owned());
    }
    let program_headers = build_program_headers(
        load_seeds,
        &segment_sections,
        &mut sections,
        dynamic_layout.interpreter_file_offset,
        dynamic_layout.interpreter_virtual_address,
        dynamic_layout.interpreter_bytes,
        dynamic_table_file_offset,
        dynamic_table_virtual_address,
        dynamic_table_bytes,
        program_header_table_bytes,
        application_ledger_hash,
    )?;
    for section in &mut sections {
        section.audit_hash = section_audit_hash(application_ledger_hash, section);
    }
    let dynamic_entries = build_dynamic_entries(
        &sections,
        dynamic_enabled,
        &dynamic_layout.needed_libraries,
        dynamic_layout.version_needs.len(),
        application_ledger_hash,
    )?;
    validate_layout(
        &sections,
        &program_headers,
        platform,
        planned_file_span_bytes,
        planned_memory_span_bytes,
    )?;
    Ok(ElfAmd64ShellLayoutDraft {
        section_header_count: sections.len(),
        sections,
        program_headers,
        dynamic_entries,
        program_header_table_bytes,
        section_header_table_file_offset,
        section_name_table_section_index: shstrtab_index,
        section_name_table_file_offset: shstrtab_offset,
        section_name_table_bytes: shstrtab_bytes,
        planned_file_span_bytes,
        planned_memory_span_bytes,
        load_segment_count: program_header_count
            - 1
            - usize::from(dynamic_enabled)
            - usize::from(dynamic_layout.interpreter_file_offset.is_some()),
        dynamic_table_file_offset,
        dynamic_table_virtual_address,
        dynamic_table_bytes,
        dynamic_dependency_plan_hash: dynamic_layout.dependency_plan_hash,
        interpreter_identity: dynamic_layout.interpreter_identity,
        interpreter_path: dynamic_layout.interpreter_path,
        interpreter_file_offset: dynamic_layout.interpreter_file_offset,
        interpreter_virtual_address: dynamic_layout.interpreter_virtual_address,
        interpreter_bytes: dynamic_layout.interpreter_bytes,
        dynamic_string_source_image_offset: dynamic_layout.dynamic_string_source_image_offset,
        dynamic_string_source_bytes: dynamic_layout.dynamic_string_source_bytes,
        needed_libraries: dynamic_layout.needed_libraries,
        version_symbol_table_file_offset: dynamic_layout.version_symbol_file_offset,
        version_symbol_table_virtual_address: dynamic_layout.version_symbol_virtual_address,
        version_symbol_table_bytes: dynamic_layout.version_symbol_bytes,
        version_need_table_file_offset: dynamic_layout.version_need_file_offset,
        version_need_table_virtual_address: dynamic_layout.version_need_virtual_address,
        version_need_table_bytes: dynamic_layout.version_need_bytes,
        version_symbols: dynamic_layout.version_symbols,
        version_needs: dynamic_layout.version_needs,
    })
}

fn section_seeds(
    placement: &ElfAmd64PlacementBindingReport,
    platform: &ElfAmd64PlatformStructurePlanReport,
    replace_dynamic_string_table: bool,
) -> Result<Vec<SectionSeed>, String> {
    let mut seeds = placement
        .merged_sections
        .iter()
        .map(|section| {
            let (section_type, flags, file_size) = match section.class.as_str() {
                "text" => (SHT_PROGBITS, SHF_ALLOC | SHF_EXECINSTR, section.size_bytes),
                "rodata" => (SHT_PROGBITS, SHF_ALLOC, section.size_bytes),
                "data" => (SHT_PROGBITS, SHF_ALLOC | SHF_WRITE, section.size_bytes),
                "bss" | "common" => (SHT_NOBITS, SHF_ALLOC | SHF_WRITE, 0),
                other => return Err(format!("ELF shell rejects section class `{other}`")),
            };
            Ok(SectionSeed {
                source_kind: "merged-section",
                source_id: section.section_id.clone(),
                name: section.output_section_name.clone(),
                section_type,
                flags,
                alignment: section.alignment,
                entry_size: 0,
                source_image_offset: Some(section.image_offset),
                source_size_bytes: section.size_bytes,
                file_offset: section.file_offset.unwrap_or(section.image_offset),
                file_size_bytes: file_size,
                virtual_address: section.virtual_address,
                memory_size_bytes: section.size_bytes,
                segment_key: Some(format!("base:{}", section.section_id)),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    append_platform_sections(&mut seeds, platform, replace_dynamic_string_table)?;
    seeds.sort_by(|lhs, rhs| {
        lhs.source_image_offset
            .cmp(&rhs.source_image_offset)
            .then(lhs.name.cmp(&rhs.name))
    });
    Ok(seeds)
}

fn append_platform_sections(
    seeds: &mut Vec<SectionSeed>,
    platform: &ElfAmd64PlatformStructurePlanReport,
    replace_dynamic_string_table: bool,
) -> Result<(), String> {
    append_platform_section(
        seeds,
        platform.plt_region_bytes,
        platform.plt_region_image_offset,
        ".plt",
        "platform-plt",
        SHT_PROGBITS,
        SHF_ALLOC | SHF_EXECINSTR,
        platform.plt_alignment,
        platform.plt_entry_size,
    )?;
    append_platform_section(
        seeds,
        platform.got_region_bytes,
        platform.got_region_image_offset,
        ".got.plt",
        "platform-got",
        SHT_PROGBITS,
        SHF_ALLOC | SHF_WRITE,
        platform.got_alignment,
        platform.got_entry_size,
    )?;
    append_platform_section(
        seeds,
        platform.dynamic_symbol_region_bytes,
        platform.dynamic_symbol_region_image_offset,
        ".dynsym",
        "platform-metadata",
        SHT_DYNSYM,
        SHF_ALLOC,
        8,
        platform.dynamic_symbol_entry_size,
    )?;
    if !replace_dynamic_string_table {
        append_platform_section(
            seeds,
            platform.dynamic_string_region_bytes,
            platform.dynamic_string_region_image_offset,
            ".dynstr",
            "platform-metadata",
            SHT_STRTAB,
            SHF_ALLOC,
            1,
            0,
        )?;
    }
    append_platform_section(
        seeds,
        platform.dynamic_relocation_region_bytes,
        platform.dynamic_relocation_region_image_offset,
        ".rela.plt",
        "platform-metadata",
        SHT_RELA,
        SHF_ALLOC,
        platform.dynamic_relocation_alignment,
        platform.dynamic_relocation_entry_size,
    )
}

fn append_shell_dynamic_metadata_sections(
    seeds: &mut Vec<SectionSeed>,
    dynamic: &ElfAmd64ShellDynamicLayout,
) -> Result<(), String> {
    let Some(interpreter_offset) = dynamic.interpreter_file_offset else {
        return Ok(());
    };
    let interpreter_virtual = dynamic
        .interpreter_virtual_address
        .ok_or_else(|| "ELF shell interpreter virtual coordinate is absent".to_owned())?;
    seeds.push(SectionSeed {
        source_kind: "shell-interpreter",
        source_id: "elf-amd64-shell-interpreter".to_owned(),
        name: ".interp".to_owned(),
        section_type: SHT_PROGBITS,
        flags: SHF_ALLOC,
        alignment: 1,
        entry_size: 0,
        source_image_offset: None,
        source_size_bytes: 0,
        file_offset: interpreter_offset,
        file_size_bytes: dynamic.interpreter_bytes,
        virtual_address: interpreter_virtual,
        memory_size_bytes: dynamic.interpreter_bytes,
        segment_key: Some("shell-dynamic-metadata".to_owned()),
    });
    seeds.push(SectionSeed {
        source_kind: "shell-final-dynamic-string-table",
        source_id: "elf-amd64-shell-final-dynstr".to_owned(),
        name: ".dynstr".to_owned(),
        section_type: SHT_STRTAB,
        flags: SHF_ALLOC,
        alignment: 1,
        entry_size: 0,
        source_image_offset: None,
        source_size_bytes: 0,
        file_offset: dynamic
            .dynamic_string_file_offset
            .ok_or_else(|| "ELF shell final dynamic string coordinate is absent".to_owned())?,
        file_size_bytes: dynamic.dynamic_string_bytes,
        virtual_address: dynamic
            .dynamic_string_virtual_address
            .ok_or_else(|| "ELF shell final dynamic string address is absent".to_owned())?,
        memory_size_bytes: dynamic.dynamic_string_bytes,
        segment_key: Some("shell-dynamic-metadata".to_owned()),
    });
    seeds.push(SectionSeed {
        source_kind: "shell-version-symbol-table",
        source_id: "elf-amd64-shell-gnu-version".to_owned(),
        name: ".gnu.version".to_owned(),
        section_type: SHT_GNU_VERSYM,
        flags: SHF_ALLOC,
        alignment: 2,
        entry_size: ELF64_VERSION_SYMBOL_ENTRY_SIZE,
        source_image_offset: None,
        source_size_bytes: 0,
        file_offset: dynamic
            .version_symbol_file_offset
            .ok_or_else(|| "ELF shell version-symbol coordinate is absent".to_owned())?,
        file_size_bytes: dynamic.version_symbol_bytes,
        virtual_address: dynamic
            .version_symbol_virtual_address
            .ok_or_else(|| "ELF shell version-symbol address is absent".to_owned())?,
        memory_size_bytes: dynamic.version_symbol_bytes,
        segment_key: Some("shell-dynamic-metadata".to_owned()),
    });
    seeds.push(SectionSeed {
        source_kind: "shell-version-need-table",
        source_id: "elf-amd64-shell-gnu-version-r".to_owned(),
        name: ".gnu.version_r".to_owned(),
        section_type: SHT_GNU_VERNEED,
        flags: SHF_ALLOC,
        alignment: 8,
        entry_size: 0,
        source_image_offset: None,
        source_size_bytes: 0,
        file_offset: dynamic
            .version_need_file_offset
            .ok_or_else(|| "ELF shell version-need coordinate is absent".to_owned())?,
        file_size_bytes: dynamic.version_need_bytes,
        virtual_address: dynamic
            .version_need_virtual_address
            .ok_or_else(|| "ELF shell version-need address is absent".to_owned())?,
        memory_size_bytes: dynamic.version_need_bytes,
        segment_key: Some("shell-dynamic-metadata".to_owned()),
    });
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn append_platform_section(
    seeds: &mut Vec<SectionSeed>,
    size: usize,
    offset: usize,
    name: &str,
    segment_key: &'static str,
    section_type: u32,
    flags: u64,
    alignment: usize,
    entry_size: usize,
) -> Result<(), String> {
    if size == 0 {
        return Ok(());
    }
    seeds.push(SectionSeed {
        source_kind: segment_key,
        source_id: format!("elf-amd64-{segment_key}-{name}"),
        name: name.to_owned(),
        section_type,
        flags,
        alignment,
        entry_size,
        source_image_offset: Some(offset),
        source_size_bytes: size,
        file_offset: offset,
        file_size_bytes: size,
        virtual_address: virtual_address(offset)?,
        memory_size_bytes: size,
        segment_key: Some(segment_key.to_owned()),
    });
    Ok(())
}

fn build_sections(seeds: Vec<SectionSeed>, ledger_hash: &str) -> Result<BuiltSections, String> {
    let mut names = BTreeSet::new();
    let mut name_cursor = 1usize;
    let mut segment_sections = BTreeMap::<String, Vec<String>>::new();
    let mut sections = vec![ElfAmd64ShellSectionPlan {
        section_id: "elf-amd64-shell-section-0000".to_owned(),
        section_index: 0,
        section_name: String::new(),
        section_name_offset: 0,
        source_kind: "shell-null".to_owned(),
        source_id: "elf-amd64-shell-null".to_owned(),
        section_type: SHT_NULL,
        flags: 0,
        alignment: 0,
        entry_size: 0,
        link_section_index: 0,
        info_section_index: 0,
        source_image_offset: None,
        source_size_bytes: 0,
        file_offset: 0,
        file_size_bytes: 0,
        virtual_address: 0,
        memory_size_bytes: 0,
        load_segment_id: None,
        audit_hash: String::new(),
    }];
    for seed in seeds {
        if seed.name.is_empty() || !names.insert(seed.name.clone()) {
            return Err(format!(
                "ELF shell section name `{}` is empty or repeated",
                seed.name
            ));
        }
        let index = sections.len();
        let section_id = format!("elf-amd64-shell-section-{index:04}");
        let name_offset = name_cursor;
        name_cursor = checked_add(
            name_cursor,
            checked_add(seed.name.len(), 1, "section name")?,
            "section-name table",
        )?;
        if let Some(key) = &seed.segment_key {
            segment_sections
                .entry(key.clone())
                .or_default()
                .push(section_id.clone());
        }
        sections.push(ElfAmd64ShellSectionPlan {
            section_id,
            section_index: index,
            section_name: seed.name,
            section_name_offset: name_offset,
            source_kind: seed.source_kind.to_owned(),
            source_id: seed.source_id,
            section_type: seed.section_type,
            flags: seed.flags,
            alignment: seed.alignment,
            entry_size: seed.entry_size,
            link_section_index: 0,
            info_section_index: 0,
            source_image_offset: seed.source_image_offset,
            source_size_bytes: seed.source_size_bytes,
            file_offset: seed.file_offset,
            file_size_bytes: seed.file_size_bytes,
            virtual_address: seed.virtual_address,
            memory_size_bytes: seed.memory_size_bytes,
            load_segment_id: None,
            audit_hash: String::new(),
        });
    }
    for section in &mut sections {
        section.audit_hash = section_audit_hash(ledger_hash, section);
    }
    Ok(BuiltSections {
        sections,
        segment_sections,
        string_table_bytes: name_cursor,
    })
}

fn load_seeds(
    placement: &ElfAmd64PlacementBindingReport,
    platform: &ElfAmd64PlatformStructurePlanReport,
    dynamic: &ElfAmd64ShellDynamicLayout,
) -> Result<Vec<LoadSeed>, String> {
    let mut seeds = vec![LoadSeed {
        segment_key: "shell-header".to_owned(),
        permission_class: "read-only-header",
        flags: PF_R,
        file_offset: 0,
        virtual_address: ELF_AMD64_IMAGE_BASE,
        file_size_bytes: placement.payload_file_offset,
        memory_size_bytes: placement.payload_file_offset,
        alignment: ELF_AMD64_PAGE_SIZE,
    }];
    for section in &placement.merged_sections {
        let (permission_class, flags) = match section.class.as_str() {
            "text" => ("read-execute", PF_R | PF_X),
            "rodata" => ("read-only", PF_R),
            "data" | "bss" | "common" => ("read-write", PF_R | PF_W),
            other => return Err(format!("ELF shell rejects load class `{other}`")),
        };
        seeds.push(LoadSeed {
            segment_key: format!("base:{}", section.section_id),
            permission_class,
            flags,
            file_offset: section.file_offset.unwrap_or(section.image_offset),
            virtual_address: section.virtual_address,
            file_size_bytes: if section.zero_fill {
                0
            } else {
                section.size_bytes
            },
            memory_size_bytes: section.size_bytes,
            alignment: section.alignment.max(ELF_AMD64_PAGE_SIZE),
        });
    }
    append_load_seed(
        &mut seeds,
        "platform-plt",
        "read-execute",
        PF_R | PF_X,
        platform.plt_region_image_offset,
        platform.plt_region_bytes,
    )?;
    append_load_seed(
        &mut seeds,
        "platform-got",
        "read-write",
        PF_R | PF_W,
        platform.got_region_image_offset,
        platform.got_region_bytes,
    )?;
    append_load_seed(
        &mut seeds,
        "platform-metadata",
        "read-only",
        PF_R,
        platform.metadata_region_image_offset,
        platform.metadata_region_bytes,
    )?;
    if let (Some(offset), Some(virtual_address)) = (
        dynamic.metadata_file_offset,
        dynamic.metadata_virtual_address,
    ) {
        seeds.push(LoadSeed {
            segment_key: "shell-dynamic-metadata".to_owned(),
            permission_class: "read-only-dynamic-metadata",
            flags: PF_R,
            file_offset: offset,
            virtual_address,
            file_size_bytes: dynamic.metadata_bytes,
            memory_size_bytes: dynamic.metadata_bytes,
            alignment: ELF_AMD64_PAGE_SIZE,
        });
    }
    if let (Some(offset), Some(virtual_address)) = (
        dynamic.dynamic_table_file_offset,
        dynamic.dynamic_table_virtual_address,
    ) {
        seeds.push(LoadSeed {
            segment_key: "shell-dynamic".to_owned(),
            permission_class: "read-write",
            flags: PF_R | PF_W,
            file_offset: offset,
            virtual_address,
            file_size_bytes: dynamic.dynamic_table_bytes,
            memory_size_bytes: dynamic.dynamic_table_bytes,
            alignment: ELF_AMD64_PAGE_SIZE,
        });
    }
    seeds.sort_by_key(|seed| seed.file_offset);
    Ok(seeds)
}

fn append_load_seed(
    seeds: &mut Vec<LoadSeed>,
    key: &str,
    permission_class: &'static str,
    flags: u32,
    offset: usize,
    bytes: usize,
) -> Result<(), String> {
    if bytes > 0 {
        seeds.push(LoadSeed {
            segment_key: key.to_owned(),
            permission_class,
            flags,
            file_offset: offset,
            virtual_address: virtual_address(offset)?,
            file_size_bytes: bytes,
            memory_size_bytes: bytes,
            alignment: ELF_AMD64_PAGE_SIZE,
        });
    }
    Ok(())
}

fn align_up(value: usize, alignment: usize) -> Result<usize, String> {
    value
        .checked_add(alignment - 1)
        .map(|value| value & !(alignment - 1))
        .ok_or_else(|| "ELF shell alignment overflows".to_owned())
}

fn checked_add(lhs: usize, rhs: usize, label: &str) -> Result<usize, String> {
    lhs.checked_add(rhs)
        .ok_or_else(|| format!("ELF shell {label} span overflows"))
}

fn virtual_address(offset: usize) -> Result<u64, String> {
    ELF_AMD64_IMAGE_BASE
        .checked_add(
            u64::try_from(offset)
                .map_err(|_| "ELF shell offset exceeds u64 address space".to_owned())?,
        )
        .ok_or_else(|| "ELF shell virtual address overflows".to_owned())
}
