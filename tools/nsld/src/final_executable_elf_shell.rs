#[path = "final_executable_elf_shell_layout.rs"]
mod layout;
#[path = "final_executable_elf_shell_report.rs"]
mod report;

pub(crate) use report::ElfAmd64ShellLayoutPlanReport;

use crate::{
    final_executable_elf_layout::{
        build_elf_amd64_placement_binding, ELF_AMD64_IMAGE_BASE, ELF_AMD64_PAGE_SIZE,
        ELF_AMD64_PLACEMENT_BINDING_CONTRACT,
    },
    final_executable_elf_layout_report::ElfAmd64PlacementBindingReport,
    final_executable_elf_materialization::application::platform::{
        application::{
            ElfAmd64PlatformAppliedImage, ELF_AMD64_PLATFORM_PATCH_APPLICATION_CONTRACT,
        },
        ElfAmd64PlatformStructurePlanReport, ELF_AMD64_PLATFORM_STRUCTURE_PLAN_CONTRACT,
    },
    final_executable_elf_object::ElfAmd64ObjectLinkage,
    final_executable_elf_relocation::{
        build_elf_amd64_relocation_application, ELF_AMD64_RELOCATION_APPLICATION_CONTRACT,
    },
    final_executable_elf_relocation_report::ElfAmd64RelocationApplicationReport,
};
use layout::{
    build_elf_amd64_shell_layout, locate_source_coordinate, ELF64_DYNAMIC_ENTRY_SIZE,
    ELF64_HEADER_SIZE, ELF64_PROGRAM_HEADER_SIZE, ELF64_SECTION_HEADER_SIZE,
};
use std::collections::BTreeSet;
use std::fmt::Write as _;

pub(crate) const ELF_AMD64_SHELL_LAYOUT_PLAN_CONTRACT: &str =
    "nuis-nsld-elf-amd64-shell-layout-plan-v1";

#[derive(Clone, Copy)]
struct EntryRule {
    rule_id: &'static str,
    object_role: &'static str,
    symbol: &'static str,
}

const ENTRY_RULES: &[EntryRule] = &[
    EntryRule {
        rule_id: "amd64.elf.runtime-start.v1",
        object_role: "runtime-shim",
        symbol: "_start",
    },
    EntryRule {
        rule_id: "amd64.elf.program-entry.v1",
        object_role: "program-llvm",
        symbol: "__nuis_entry",
    },
    EntryRule {
        rule_id: "amd64.elf.program-yir-entry.v1",
        object_role: "program-llvm",
        symbol: "nuis_yir_entry",
    },
];

struct EntrySelection {
    rule_id: String,
    symbol: String,
    object_id: String,
    symbol_index: usize,
    source_image_offset: usize,
    section_id: String,
    file_offset: usize,
    virtual_address: u64,
}

pub(crate) fn build_elf_amd64_shell_layout_plan(
    objects: &[ElfAmd64ObjectLinkage],
    placement: &ElfAmd64PlacementBindingReport,
    relocations: &ElfAmd64RelocationApplicationReport,
    platform_plan: &ElfAmd64PlatformStructurePlanReport,
    platform_applied: &ElfAmd64PlatformAppliedImage,
) -> Result<ElfAmd64ShellLayoutPlanReport, String> {
    validate_input_envelope(
        objects,
        placement,
        relocations,
        platform_plan,
        platform_applied,
    )?;
    let object_linkage_hash = elf_object_linkage_hash(objects);
    let layout = build_elf_amd64_shell_layout(
        placement,
        platform_plan,
        &platform_applied.report.application_ledger_hash,
    )?;
    let entry = select_entry(objects, placement, &layout.sections)?;
    validate_entry_segment(&entry, &layout)?;
    let status = if platform_applied.report.unresolved_dynamic_bind_count == 0 {
        "static-closure-layout-planned"
    } else {
        "layout-planned-with-external-resolution-boundary"
    };
    let mut report = ElfAmd64ShellLayoutPlanReport {
        contract: ELF_AMD64_SHELL_LAYOUT_PLAN_CONTRACT,
        status: status.to_owned(),
        plan_hash: String::new(),
        object_linkage_hash,
        placement_plan_hash: placement.plan_hash.clone(),
        relocation_plan_hash: relocations.plan_hash.clone(),
        platform_structure_plan_hash: platform_plan.plan_hash.clone(),
        platform_application_ledger_hash: platform_applied.report.application_ledger_hash.clone(),
        platform_image_hash: platform_applied.report.applied_memory_image_hash.clone(),
        image_base: ELF_AMD64_IMAGE_BASE,
        page_size: ELF_AMD64_PAGE_SIZE,
        elf_header_file_offset: 0,
        elf_header_size_bytes: ELF64_HEADER_SIZE,
        program_header_table_file_offset: ELF64_HEADER_SIZE,
        program_header_entry_size_bytes: ELF64_PROGRAM_HEADER_SIZE,
        program_header_count: layout.program_headers.len(),
        program_header_table_bytes: layout.program_header_table_bytes,
        section_header_table_file_offset: layout.section_header_table_file_offset,
        section_header_entry_size_bytes: ELF64_SECTION_HEADER_SIZE,
        section_header_count: layout.section_header_count,
        section_name_table_section_index: layout.section_name_table_section_index,
        section_name_table_file_offset: layout.section_name_table_file_offset,
        section_name_table_bytes: layout.section_name_table_bytes,
        entry_rule_id: entry.rule_id,
        entry_symbol: entry.symbol,
        entry_source_object_id: entry.object_id,
        entry_source_symbol_index: entry.symbol_index,
        entry_source_image_offset: entry.source_image_offset,
        entry_section_id: entry.section_id,
        entry_file_offset: entry.file_offset,
        entry_virtual_address: entry.virtual_address,
        applied_file_span_bytes: platform_applied.report.applied_file_span_bytes,
        applied_memory_span_bytes: platform_applied.report.applied_memory_span_bytes,
        planned_file_span_bytes: layout.planned_file_span_bytes,
        planned_memory_span_bytes: layout.planned_memory_span_bytes,
        load_segment_count: layout.load_segment_count,
        dynamic_table_file_offset: layout.dynamic_table_file_offset,
        dynamic_table_virtual_address: layout.dynamic_table_virtual_address,
        dynamic_table_entry_size_bytes: ELF64_DYNAMIC_ENTRY_SIZE,
        dynamic_table_entry_count: layout.dynamic_entries.len(),
        dynamic_table_bytes: layout.dynamic_table_bytes,
        program_headers: layout.program_headers,
        sections: layout.sections,
        dynamic_entries: layout.dynamic_entries,
    };
    report.plan_hash = crate::fnv1a64_hex(report.canonical_plan().as_bytes());
    Ok(report)
}

fn validate_input_envelope(
    objects: &[ElfAmd64ObjectLinkage],
    placement: &ElfAmd64PlacementBindingReport,
    relocations: &ElfAmd64RelocationApplicationReport,
    platform_plan: &ElfAmd64PlatformStructurePlanReport,
    platform_applied: &ElfAmd64PlatformAppliedImage,
) -> Result<(), String> {
    if placement.contract != ELF_AMD64_PLACEMENT_BINDING_CONTRACT
        || relocations.contract != ELF_AMD64_RELOCATION_APPLICATION_CONTRACT
        || platform_plan.contract != ELF_AMD64_PLATFORM_STRUCTURE_PLAN_CONTRACT
        || platform_applied.report.contract != ELF_AMD64_PLATFORM_PATCH_APPLICATION_CONTRACT
    {
        return Err("ELF shell layout rejects an upstream contract".to_owned());
    }
    if objects.is_empty() {
        return Err("ELF shell layout has no input objects".to_owned());
    }
    let unique_ids = objects
        .iter()
        .map(|object| object.object_id.as_str())
        .collect::<BTreeSet<_>>();
    if unique_ids.len() != objects.len() {
        return Err("ELF shell layout repeats an input object id".to_owned());
    }
    let rebuilt_placement = build_elf_amd64_placement_binding(objects)?;
    if rebuilt_placement != *placement {
        return Err("ELF shell layout rejects object/placement drift".to_owned());
    }
    let rebuilt_relocations = build_elf_amd64_relocation_application(objects, placement)?;
    if rebuilt_relocations != *relocations {
        return Err("ELF shell layout rejects object/relocation drift".to_owned());
    }
    if placement.plan_hash != crate::fnv1a64_hex(placement.canonical_plan().as_bytes())
        || relocations.plan_hash != crate::fnv1a64_hex(relocations.canonical_plan().as_bytes())
        || platform_plan.plan_hash != crate::fnv1a64_hex(platform_plan.canonical_plan().as_bytes())
        || platform_applied.report.application_ledger_hash
            != crate::fnv1a64_hex(platform_applied.report.canonical_ledger().as_bytes())
    {
        return Err("ELF shell layout rejects an upstream hash or ledger drift".to_owned());
    }
    if relocations.placement_plan_hash != placement.plan_hash
        || platform_plan.placement_plan_hash != placement.plan_hash
        || platform_plan.relocation_plan_hash != relocations.plan_hash
        || platform_applied.report.placement_plan_hash != placement.plan_hash
        || platform_applied.report.relocation_plan_hash != relocations.plan_hash
        || platform_applied.report.platform_structure_plan_hash != platform_plan.plan_hash
        || platform_applied.report.base_applied_memory_image_hash
            != platform_plan.applied_memory_image_hash
    {
        return Err("ELF shell layout rejects upstream lineage drift".to_owned());
    }
    let file_image = platform_applied
        .bytes
        .get(..platform_applied.report.applied_file_span_bytes)
        .ok_or_else(|| "ELF shell platform file span exceeds its image".to_owned())?;
    if platform_applied.bytes.len() != platform_applied.report.applied_memory_span_bytes
        || platform_applied.report.applied_file_span_bytes != platform_plan.planned_file_span_bytes
        || platform_applied.report.applied_memory_span_bytes
            != platform_plan.planned_memory_span_bytes
        || crate::fnv1a64_hex(file_image) != platform_applied.report.applied_file_image_hash
        || crate::fnv1a64_hex(&platform_applied.bytes)
            != platform_applied.report.applied_memory_image_hash
    {
        return Err("ELF shell layout rejects platform image drift".to_owned());
    }
    validate_platform_application_counts(relocations, platform_plan, platform_applied)
}

fn validate_platform_application_counts(
    relocations: &ElfAmd64RelocationApplicationReport,
    platform_plan: &ElfAmd64PlatformStructurePlanReport,
    platform_applied: &ElfAmd64PlatformAppliedImage,
) -> Result<(), String> {
    let report = &platform_applied.report;
    let expected_plan_status = if relocations.platform_structure_count == 0 {
        "not-required"
    } else {
        "allocated-ready-for-platform-patching"
    };
    let expected_application_status = if relocations.platform_structure_count == 0 {
        "not-required-image-preserved"
    } else {
        "platform-structures-and-deferred-patches-applied-with-unresolved-dynamic-binds"
    };
    if platform_plan.status != expected_plan_status
        || report.status != expected_application_status
        || platform_plan.deferred_relocation_count != relocations.platform_structure_count
        || report.expected_deferred_patch_count != platform_plan.deferred_relocation_count
        || report.applied_deferred_patch_count != report.patches.len()
        || report.applied_deferred_patch_count != platform_plan.deferred_relocation_count
        || report.applied_structure_write_count != report.structure_writes.len()
        || report.expected_structure_write_count != report.structure_writes.len()
        || report.unresolved_dynamic_bind_count != report.dynamic_bind_records.len()
        || report.unresolved_dynamic_bind_count != platform_plan.dynamic_relocation_entry_count
        || report.write_once_span_count
            != report
                .structure_writes
                .len()
                .checked_add(report.patches.len())
                .ok_or_else(|| "ELF shell application count overflows".to_owned())?
    {
        return Err("ELF shell layout rejects platform application coverage drift".to_owned());
    }
    Ok(())
}

fn select_entry(
    objects: &[ElfAmd64ObjectLinkage],
    placement: &ElfAmd64PlacementBindingReport,
    sections: &[report::ElfAmd64ShellSectionPlan],
) -> Result<EntrySelection, String> {
    for rule in ENTRY_RULES {
        let matches = objects
            .iter()
            .filter(|object| object.role == rule.object_role)
            .flat_map(|object| {
                object
                    .linkage
                    .symbols
                    .iter()
                    .filter(move |symbol| symbol.name == rule.symbol)
                    .map(move |symbol| (object, symbol))
            })
            .collect::<Vec<_>>();
        if matches.is_empty() {
            continue;
        }
        let [(object, symbol)] = matches.as_slice() else {
            return Err(format!(
                "ELF shell entry `{}` has {} definitions",
                rule.symbol,
                matches.len()
            ));
        };
        if !symbol.external
            || !symbol.defined
            || symbol.symbol_type != 2
            || symbol.absolute
            || symbol.common
        {
            return Err(format!(
                "ELF shell entry `{}` is not a section-backed global function",
                rule.symbol
            ));
        }
        let input_section_index = symbol
            .section_index
            .ok_or_else(|| format!("ELF shell entry `{}` has no input section", rule.symbol))?;
        let source = placement
            .section_placements
            .iter()
            .find(|item| {
                item.object_id == object.object_id
                    && item.input_section_index == input_section_index
            })
            .ok_or_else(|| format!("ELF shell entry `{}` has no section placement", rule.symbol))?;
        let relative = usize::try_from(symbol.value)
            .map_err(|_| "ELF shell entry offset exceeds host space".to_owned())?;
        if relative >= source.size_bytes {
            return Err(format!(
                "ELF shell entry `{}` exceeds its input section",
                rule.symbol
            ));
        }
        let source_image_offset = source
            .image_offset
            .checked_add(relative)
            .ok_or_else(|| "ELF shell entry source offset overflows".to_owned())?;
        let coordinate = locate_source_coordinate(source_image_offset, sections)?;
        return Ok(EntrySelection {
            rule_id: rule.rule_id.to_owned(),
            symbol: rule.symbol.to_owned(),
            object_id: object.object_id.clone(),
            symbol_index: symbol.index,
            source_image_offset,
            section_id: coordinate.section.section_id.clone(),
            file_offset: coordinate.file_offset,
            virtual_address: coordinate.virtual_address,
        });
    }
    Err(format!(
        "ELF shell entry registry found no supported definition; expected one of {}",
        ENTRY_RULES
            .iter()
            .map(|rule| format!("{}:{}", rule.object_role, rule.symbol))
            .collect::<Vec<_>>()
            .join(",")
    ))
}

fn validate_entry_segment(
    entry: &EntrySelection,
    layout: &layout::ElfAmd64ShellLayoutDraft,
) -> Result<(), String> {
    let section = layout
        .sections
        .iter()
        .find(|section| section.section_id == entry.section_id)
        .ok_or_else(|| "ELF shell entry section disappeared".to_owned())?;
    let segment_id = section
        .load_segment_id
        .as_deref()
        .ok_or_else(|| "ELF shell entry section has no PT_LOAD".to_owned())?;
    let segment = layout
        .program_headers
        .iter()
        .find(|header| header.program_header_id == segment_id)
        .ok_or_else(|| "ELF shell entry PT_LOAD disappeared".to_owned())?;
    let file_end = segment
        .file_offset
        .checked_add(segment.file_size_bytes)
        .ok_or_else(|| "ELF shell entry segment file range overflows".to_owned())?;
    let virtual_end = segment
        .virtual_address
        .checked_add(
            u64::try_from(segment.file_size_bytes)
                .map_err(|_| "ELF shell entry segment size exceeds u64".to_owned())?,
        )
        .ok_or_else(|| "ELF shell entry segment VM range overflows".to_owned())?;
    if segment.program_kind != "load"
        || segment.flags & 1 == 0
        || !(segment.file_offset..file_end).contains(&entry.file_offset)
        || !(segment.virtual_address..virtual_end).contains(&entry.virtual_address)
    {
        return Err("ELF shell entry is not inside a file-backed executable PT_LOAD".to_owned());
    }
    Ok(())
}

fn elf_object_linkage_hash(objects: &[ElfAmd64ObjectLinkage]) -> String {
    let mut objects = objects.iter().collect::<Vec<_>>();
    objects.sort_by(|lhs, rhs| {
        lhs.role
            .cmp(&rhs.role)
            .then(lhs.object_id.cmp(&rhs.object_id))
    });
    let mut out = String::new();
    for object in objects {
        append_text(&mut out, &object.object_id);
        append_text(&mut out, &object.role);
        writeln!(
            out,
            "counts={}|{}|{}|{}|{}",
            object.linkage.section_count,
            object.linkage.symbol_count,
            object.linkage.relocation_count,
            object.linkage.defined_symbol_count,
            object.linkage.undefined_symbol_count
        )
        .unwrap();
        for section in &object.linkage.sections {
            append_text(&mut out, &section.name);
            writeln!(
                out,
                "section={}|{}|{}|{}|{}|{}|{}",
                section.index,
                section.section_type,
                section.flags,
                section.size,
                section.alignment,
                optional_usize(section.payload_offset),
                section.zero_fill
            )
            .unwrap();
        }
        for symbol in &object.linkage.symbols {
            append_text(&mut out, &symbol.name);
            writeln!(
                out,
                "symbol={}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
                symbol.index,
                symbol.binding,
                symbol.symbol_type,
                symbol.external,
                symbol.weak,
                symbol.defined,
                optional_usize(symbol.section_index),
                symbol.absolute,
                symbol.common,
                symbol.value,
                symbol.size
            )
            .unwrap();
        }
        for relocation in &object.linkage.relocations {
            writeln!(
                out,
                "relocation={}|{}|{}|{}|{}|{}|{}|{}",
                relocation.relocation_section_index,
                relocation.target_section_index,
                relocation.symbol_index,
                relocation.offset,
                relocation.addend,
                relocation.relocation_type,
                relocation.width_bytes,
                relocation.pc_relative
            )
            .unwrap();
        }
    }
    crate::fnv1a64_hex(out.as_bytes())
}

fn append_text(out: &mut String, value: &str) {
    writeln!(out, "text:{}:{value}", value.len()).unwrap();
}

fn optional_usize(value: Option<usize>) -> String {
    value.map_or_else(|| "none".to_owned(), |value| value.to_string())
}

#[cfg(test)]
#[path = "final_executable_elf_shell_tests.rs"]
mod tests;
