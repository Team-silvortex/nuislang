use super::{
    checked_add, ElfAmd64ShellDynamicEntryPlan, ElfAmd64ShellNeededLibraryPlan,
    ElfAmd64ShellProgramHeaderPlan, ElfAmd64ShellSectionPlan, LocatedSourceCoordinate, PF_R, PF_W,
    PF_X, PT_LOAD, SHF_ALLOC, SHF_EXECINSTR, SHF_WRITE,
};
use crate::final_executable_elf_materialization::application::platform::ElfAmd64PlatformStructurePlanReport;
use std::collections::BTreeMap;

use super::super::report::dynamic_entry_audit_hash;

pub(in crate::final_executable_elf_shell) fn locate_source_coordinate(
    source_image_offset: usize,
    sections: &[ElfAmd64ShellSectionPlan],
) -> Result<LocatedSourceCoordinate<'_>, String> {
    let matches = sections
        .iter()
        .filter(|section| {
            section.source_image_offset.is_some_and(|start| {
                start
                    .checked_add(section.source_size_bytes)
                    .is_some_and(|end| (start..end).contains(&source_image_offset))
            })
        })
        .collect::<Vec<_>>();
    let [section] = matches.as_slice() else {
        return Err(format!(
            "ELF shell source offset {source_image_offset} maps to {} sections",
            matches.len()
        ));
    };
    if section.file_size_bytes == 0 || section.flags & SHF_EXECINSTR == 0 {
        return Err(format!(
            "ELF shell source offset {source_image_offset} is not file-backed executable content"
        ));
    }
    let relative = source_image_offset - section.source_image_offset.unwrap();
    Ok(LocatedSourceCoordinate {
        section,
        file_offset: checked_add(section.file_offset, relative, "entry file coordinate")?,
        virtual_address: section
            .virtual_address
            .checked_add(
                u64::try_from(relative).map_err(|_| "ELF entry offset exceeds u64".to_owned())?,
            )
            .ok_or_else(|| "ELF entry virtual address overflows".to_owned())?,
    })
}

pub(super) fn assign_section_links(
    sections: &mut [ElfAmd64ShellSectionPlan],
    version_need_count: usize,
) {
    let indices = sections
        .iter()
        .map(|section| (section.section_name.clone(), section.section_index))
        .collect::<BTreeMap<_, _>>();
    let dynstr = indices.get(".dynstr").copied().unwrap_or(0);
    let dynsym = indices.get(".dynsym").copied().unwrap_or(0);
    let got = indices.get(".got.plt").copied().unwrap_or(0);
    for section in sections {
        match section.section_name.as_str() {
            ".dynsym" | ".dynamic" => section.link_section_index = dynstr,
            ".rela.plt" => {
                section.link_section_index = dynsym;
                section.info_section_index = got;
            }
            ".gnu.version" => section.link_section_index = dynsym,
            ".gnu.version_r" => {
                section.link_section_index = dynstr;
                section.info_section_index = version_need_count;
            }
            _ => {}
        }
    }
}

pub(super) fn build_dynamic_entries(
    sections: &[ElfAmd64ShellSectionPlan],
    enabled: bool,
    needed_libraries: &[ElfAmd64ShellNeededLibraryPlan],
    version_need_count: usize,
    ledger_hash: &str,
) -> Result<Vec<ElfAmd64ShellDynamicEntryPlan>, String> {
    if !enabled {
        return Ok(Vec::new());
    }
    let got = section_by_name(sections, ".got.plt")?;
    let dynsym = section_by_name(sections, ".dynsym")?;
    let dynstr = section_by_name(sections, ".dynstr")?;
    let rela = section_by_name(sections, ".rela.plt")?;
    let mut seeds = needed_libraries
        .iter()
        .map(|needed| {
            (
                format!("DT_NEEDED:{}", needed.needed_name),
                1,
                "dynamic-string-offset".to_owned(),
                needed.dynamic_string_offset as u64,
                Some(dynstr.section_id.clone()),
            )
        })
        .collect::<Vec<_>>();
    seeds.extend([
        (
            "DT_PLTGOT".to_owned(),
            3,
            "section-address".to_owned(),
            got.virtual_address,
            Some(got.section_id.clone()),
        ),
        (
            "DT_STRTAB".to_owned(),
            5,
            "section-address".to_owned(),
            dynstr.virtual_address,
            Some(dynstr.section_id.clone()),
        ),
        (
            "DT_SYMTAB".to_owned(),
            6,
            "section-address".to_owned(),
            dynsym.virtual_address,
            Some(dynsym.section_id.clone()),
        ),
        (
            "DT_STRSZ".to_owned(),
            10,
            "byte-size".to_owned(),
            dynstr.file_size_bytes as u64,
            Some(dynstr.section_id.clone()),
        ),
        (
            "DT_SYMENT".to_owned(),
            11,
            "entry-size".to_owned(),
            dynsym.entry_size as u64,
            Some(dynsym.section_id.clone()),
        ),
        (
            "DT_RELA".to_owned(),
            7,
            "section-address".to_owned(),
            rela.virtual_address,
            Some(rela.section_id.clone()),
        ),
        (
            "DT_RELASZ".to_owned(),
            8,
            "byte-size".to_owned(),
            rela.file_size_bytes as u64,
            Some(rela.section_id.clone()),
        ),
        (
            "DT_RELAENT".to_owned(),
            9,
            "entry-size".to_owned(),
            rela.entry_size as u64,
            Some(rela.section_id.clone()),
        ),
        ("DT_PLTREL".to_owned(), 20, "tag-value".to_owned(), 7, None),
        (
            "DT_JMPREL".to_owned(),
            23,
            "section-address".to_owned(),
            rela.virtual_address,
            Some(rela.section_id.clone()),
        ),
        (
            "DT_PLTRELSZ".to_owned(),
            2,
            "byte-size".to_owned(),
            rela.file_size_bytes as u64,
            Some(rela.section_id.clone()),
        ),
        ("DT_NULL".to_owned(), 0, "terminator".to_owned(), 0, None),
    ]);
    if !needed_libraries.is_empty() {
        let terminator = seeds.len() - 1;
        seeds.insert(
            terminator,
            (
                "DT_BIND_NOW".to_owned(),
                24,
                "bind-policy".to_owned(),
                0,
                None,
            ),
        );
    }
    if version_need_count > 0 {
        let versym = section_by_name(sections, ".gnu.version")?;
        let verneed = section_by_name(sections, ".gnu.version_r")?;
        let terminator = seeds.len() - 1;
        seeds.splice(
            terminator..terminator,
            [
                (
                    "DT_VERSYM".to_owned(),
                    0x6fff_fff0,
                    "section-address".to_owned(),
                    versym.virtual_address,
                    Some(versym.section_id.clone()),
                ),
                (
                    "DT_VERNEED".to_owned(),
                    0x6fff_fffe,
                    "section-address".to_owned(),
                    verneed.virtual_address,
                    Some(verneed.section_id.clone()),
                ),
                (
                    "DT_VERNEEDNUM".to_owned(),
                    0x6fff_ffff,
                    "record-count".to_owned(),
                    version_need_count as u64,
                    Some(verneed.section_id.clone()),
                ),
            ],
        );
    }
    let mut entries = Vec::with_capacity(seeds.len());
    for (index, (name, tag, value_kind, value, section_id)) in seeds.into_iter().enumerate() {
        let mut entry = ElfAmd64ShellDynamicEntryPlan {
            dynamic_entry_id: format!("elf-amd64-shell-dynamic-entry-{index:04}"),
            dynamic_entry_index: index,
            tag_name: name,
            tag,
            value_kind,
            value,
            referenced_section_id: section_id,
            audit_hash: String::new(),
        };
        entry.audit_hash = dynamic_entry_audit_hash(ledger_hash, &entry);
        entries.push(entry);
    }
    Ok(entries)
}

pub(super) fn validate_platform_regions(
    platform: &ElfAmd64PlatformStructurePlanReport,
    dynamic_enabled: bool,
    page_size: usize,
) -> Result<(), String> {
    let regions = [
        (platform.plt_region_image_offset, platform.plt_region_bytes),
        (platform.got_region_image_offset, platform.got_region_bytes),
        (
            platform.metadata_region_image_offset,
            platform.metadata_region_bytes,
        ),
    ];
    let mut previous_end = platform.base_memory_span_bytes;
    for (offset, bytes) in regions {
        if bytes == 0 {
            continue;
        }
        if offset < previous_end || offset % page_size != 0 {
            return Err("ELF shell platform permission-region ordering drift".to_owned());
        }
        previous_end = checked_add(offset, bytes, "platform permission region")?;
    }
    let has_dynamic_regions = platform.dynamic_symbol_region_bytes > 0
        || platform.dynamic_string_region_bytes > 0
        || platform.dynamic_relocation_region_bytes > 0;
    if dynamic_enabled != has_dynamic_regions
        || dynamic_enabled != (platform.got_region_bytes > 0)
        || dynamic_enabled != (platform.plt_region_bytes > 0)
    {
        return Err("ELF shell dynamic region coverage drift".to_owned());
    }
    Ok(())
}

pub(super) fn validate_layout(
    sections: &[ElfAmd64ShellSectionPlan],
    headers: &[ElfAmd64ShellProgramHeaderPlan],
    platform: &ElfAmd64PlatformStructurePlanReport,
    planned_file_span: usize,
    planned_memory_span: usize,
) -> Result<(), String> {
    let loads = headers
        .iter()
        .filter(|header| header.program_type == PT_LOAD)
        .collect::<Vec<_>>();
    validate_file_coordinates(sections, headers, planned_file_span)?;
    validate_load_envelopes(&loads)?;
    for section in sections
        .iter()
        .filter(|section| section.flags & SHF_ALLOC != 0)
    {
        validate_alloc_section(section, &loads)?;
    }
    if platform.planned_file_span_bytes > planned_file_span
        || platform.planned_memory_span_bytes > planned_memory_span
    {
        return Err("ELF shell planned span truncates the platform image".to_owned());
    }
    Ok(())
}

fn validate_file_coordinates(
    sections: &[ElfAmd64ShellSectionPlan],
    headers: &[ElfAmd64ShellProgramHeaderPlan],
    planned_file_span: usize,
) -> Result<(), String> {
    for header in headers {
        let end = checked_add(
            header.file_offset,
            header.file_size_bytes,
            "ELF program file range",
        )?;
        if header.file_offset > planned_file_span || end > planned_file_span {
            return Err(format!(
                "ELF shell program header `{}` exceeds the planned file",
                header.program_header_id
            ));
        }
    }
    for section in sections {
        let end = checked_add(
            section.file_offset,
            section.file_size_bytes,
            "ELF section file range",
        )?;
        if section.file_offset > planned_file_span || end > planned_file_span {
            return Err(format!(
                "ELF shell section `{}` exceeds the planned file",
                section.section_name
            ));
        }
    }
    Ok(())
}

pub(super) fn section_index(
    sections: &[ElfAmd64ShellSectionPlan],
    name: &str,
) -> Result<usize, String> {
    Ok(section_by_name(sections, name)?.section_index)
}

pub(super) fn section_id<'a>(
    sections: &'a [ElfAmd64ShellSectionPlan],
    name: &str,
) -> Result<&'a str, String> {
    Ok(&section_by_name(sections, name)?.section_id)
}

fn validate_load_envelopes(loads: &[&ElfAmd64ShellProgramHeaderPlan]) -> Result<(), String> {
    for (index, load) in loads.iter().enumerate() {
        if load.memory_size_bytes < load.file_size_bytes
            || load.alignment == 0
            || !load.alignment.is_power_of_two()
            || load.file_offset as u64 % load.alignment as u64
                != load.virtual_address % load.alignment as u64
        {
            return Err(format!(
                "ELF shell load segment `{}` has an invalid envelope",
                load.program_header_id
            ));
        }
        for other in loads.iter().skip(index + 1) {
            if ranges_overlap(
                load.virtual_address,
                load.memory_size_bytes,
                other.virtual_address,
                other.memory_size_bytes,
            )? || ranges_overlap_usize(
                load.file_offset,
                load.file_size_bytes,
                other.file_offset,
                other.file_size_bytes,
            )? {
                return Err("ELF shell PT_LOAD permission segments overlap".to_owned());
            }
        }
    }
    Ok(())
}

fn validate_alloc_section(
    section: &ElfAmd64ShellSectionPlan,
    loads: &[&ElfAmd64ShellProgramHeaderPlan],
) -> Result<(), String> {
    let segment_id = section.load_segment_id.as_deref().ok_or_else(|| {
        format!(
            "ELF shell section `{}` has no PT_LOAD",
            section.section_name
        )
    })?;
    let load = loads
        .iter()
        .find(|load| load.program_header_id == segment_id)
        .ok_or_else(|| {
            format!(
                "ELF shell section `{}` has a missing PT_LOAD",
                section.section_name
            )
        })?;
    if !contains_u64(
        load.virtual_address,
        load.memory_size_bytes,
        section.virtual_address,
        section.memory_size_bytes,
    )? || (section.file_size_bytes > 0
        && !contains_usize(
            load.file_offset,
            load.file_size_bytes,
            section.file_offset,
            section.file_size_bytes,
        )?)
    {
        return Err(format!(
            "ELF shell section `{}` exceeds its PT_LOAD",
            section.section_name
        ));
    }
    let required_flags =
        PF_R | if section.flags & SHF_WRITE != 0 {
            PF_W
        } else {
            0
        } | if section.flags & SHF_EXECINSTR != 0 {
            PF_X
        } else {
            0
        };
    if load.flags != required_flags {
        return Err(format!(
            "ELF shell section `{}` permission drift",
            section.section_name
        ));
    }
    Ok(())
}

fn section_by_name<'a>(
    sections: &'a [ElfAmd64ShellSectionPlan],
    name: &str,
) -> Result<&'a ElfAmd64ShellSectionPlan, String> {
    let matches = sections
        .iter()
        .filter(|section| section.section_name == name)
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [section] => Ok(section),
        _ => Err(format!(
            "ELF shell expected one `{name}` section, found {}",
            matches.len()
        )),
    }
}

fn ranges_overlap(
    lhs_start: u64,
    lhs_size: usize,
    rhs_start: u64,
    rhs_size: usize,
) -> Result<bool, String> {
    if lhs_size == 0 || rhs_size == 0 {
        return Ok(false);
    }
    let lhs_end = lhs_start
        .checked_add(u64::try_from(lhs_size).map_err(|_| "ELF VM size exceeds u64".to_owned())?)
        .ok_or_else(|| "ELF VM range overflows".to_owned())?;
    let rhs_end = rhs_start
        .checked_add(u64::try_from(rhs_size).map_err(|_| "ELF VM size exceeds u64".to_owned())?)
        .ok_or_else(|| "ELF VM range overflows".to_owned())?;
    Ok(lhs_start < rhs_end && rhs_start < lhs_end)
}

fn ranges_overlap_usize(
    lhs_start: usize,
    lhs_size: usize,
    rhs_start: usize,
    rhs_size: usize,
) -> Result<bool, String> {
    if lhs_size == 0 || rhs_size == 0 {
        return Ok(false);
    }
    let lhs_end = checked_add(lhs_start, lhs_size, "ELF file range")?;
    let rhs_end = checked_add(rhs_start, rhs_size, "ELF file range")?;
    Ok(lhs_start < rhs_end && rhs_start < lhs_end)
}

fn contains_u64(
    outer_start: u64,
    outer_size: usize,
    inner_start: u64,
    inner_size: usize,
) -> Result<bool, String> {
    let outer_end = outer_start
        .checked_add(u64::try_from(outer_size).map_err(|_| "ELF VM size exceeds u64".to_owned())?)
        .ok_or_else(|| "ELF VM range overflows".to_owned())?;
    let inner_end = inner_start
        .checked_add(u64::try_from(inner_size).map_err(|_| "ELF VM size exceeds u64".to_owned())?)
        .ok_or_else(|| "ELF VM range overflows".to_owned())?;
    Ok(inner_start >= outer_start && inner_end <= outer_end)
}

fn contains_usize(
    outer_start: usize,
    outer_size: usize,
    inner_start: usize,
    inner_size: usize,
) -> Result<bool, String> {
    let outer_end = checked_add(outer_start, outer_size, "ELF file range")?;
    let inner_end = checked_add(inner_start, inner_size, "ELF file range")?;
    Ok(inner_start >= outer_start && inner_end <= outer_end)
}
