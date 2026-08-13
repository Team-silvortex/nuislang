use crate::{
    final_executable_macho_input::{
        ParsedMachOObjectLinkage, ParsedMachOSection, ParsedMachOSymbol,
    },
    reports::{
        NsldMachOMergedSectionPlan, NsldMachOPlacementBindingReport, NsldMachOSectionPlacement,
        NsldMachOSymbolBinding,
    },
};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

pub(crate) const MACHO_PLACEMENT_BINDING_CONTRACT: &str = "nuis-nsld-macho-placement-binding-v1";

pub(crate) struct MachOLayoutObject<'a> {
    pub(crate) object_id: &'a str,
    pub(crate) role: &'a str,
    pub(crate) linkage: &'a ParsedMachOObjectLinkage,
}

#[derive(Clone, Copy)]
struct SectionContribution<'a> {
    object_id: &'a str,
    object_role: &'a str,
    object_order: usize,
    section: &'a ParsedMachOSection,
}

#[derive(Clone, Copy)]
struct SymbolDefinition<'a> {
    object_id: &'a str,
    symbol: &'a ParsedMachOSymbol,
}

pub(crate) fn build_macho_placement_binding_report(
    objects: &[MachOLayoutObject<'_>],
) -> Result<NsldMachOPlacementBindingReport, String> {
    let objects = sorted_objects(objects)?;
    let (merged_sections, section_placements) = build_section_layout(&objects)?;
    let symbol_bindings = build_symbol_bindings(&objects, &section_placements)?;
    let internally_bound_symbol_count = symbol_bindings
        .iter()
        .filter(|binding| binding.status == "internal")
        .count();
    let external_compatibility_symbol_count = symbol_bindings
        .iter()
        .filter(|binding| binding.status == "external-compatibility")
        .count();
    let image_span_bytes = merged_sections
        .last()
        .map(|section| section.output_offset + section.size_bytes)
        .unwrap_or(0);
    let status = if external_compatibility_symbol_count == 0 {
        "placement-and-internal-binding-ready"
    } else {
        "placement-ready-with-external-compatibility-boundary"
    };
    let canonical = canonical_plan(
        status,
        image_span_bytes,
        &merged_sections,
        &section_placements,
        &symbol_bindings,
    );

    Ok(NsldMachOPlacementBindingReport {
        contract: MACHO_PLACEMENT_BINDING_CONTRACT.to_owned(),
        status: status.to_owned(),
        plan_hash: crate::fnv1a64_hex(canonical.as_bytes()),
        image_span_bytes,
        merged_sections,
        section_placements,
        symbol_bindings,
        internally_bound_symbol_count,
        external_compatibility_symbol_count,
    })
}

fn sorted_objects<'a>(
    objects: &'a [MachOLayoutObject<'a>],
) -> Result<Vec<&'a MachOLayoutObject<'a>>, String> {
    let mut seen = BTreeSet::new();
    for object in objects {
        if !seen.insert(object.object_id) {
            return Err(format!(
                "Mach-O placement input contains duplicate object id `{}`",
                object.object_id
            ));
        }
    }
    let mut sorted = objects.iter().collect::<Vec<_>>();
    sorted.sort_by(|lhs, rhs| {
        object_role_rank(lhs.role)
            .cmp(&object_role_rank(rhs.role))
            .then(lhs.role.cmp(rhs.role))
            .then(lhs.object_id.cmp(rhs.object_id))
    });
    Ok(sorted)
}

fn build_section_layout(
    objects: &[&MachOLayoutObject<'_>],
) -> Result<
    (
        Vec<NsldMachOMergedSectionPlan>,
        Vec<NsldMachOSectionPlacement>,
    ),
    String,
> {
    let mut groups = BTreeMap::<(String, String), Vec<SectionContribution<'_>>>::new();
    for (object_order, object) in objects.iter().enumerate() {
        for section in &object.linkage.sections {
            let key = (output_segment(section).to_owned(), section.name.clone());
            groups.entry(key).or_default().push(SectionContribution {
                object_id: object.object_id,
                object_role: object.role,
                object_order,
                section,
            });
        }
    }

    let mut merged_sections = Vec::new();
    let mut placements = Vec::new();
    let mut image_cursor = 0usize;
    for ((segment_name, section_name), mut contributions) in groups {
        contributions.sort_by_key(|item| (item.object_order, item.section.ordinal));
        let expected_flags = contributions[0].section.flags;
        if let Some(conflict) = contributions
            .iter()
            .find(|item| item.section.flags != expected_flags)
        {
            return Err(format!(
                "Mach-O output section `{segment_name},{section_name}` has incompatible flags: expected 0x{expected_flags:08x}, object `{}` section {} has 0x{:08x}",
                conflict.object_id,
                conflict.section.ordinal,
                conflict.section.flags
            ));
        }
        let alignment = contributions
            .iter()
            .map(|item| checked_usize(item.section.alignment, "section alignment"))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .max()
            .unwrap_or(1);
        image_cursor = align_up(image_cursor, alignment)?;
        let output_offset = image_cursor;
        let section_id = format!("macho-section-{:04}", merged_sections.len());
        let mut section_cursor = 0usize;
        for contribution in &contributions {
            let contribution_alignment =
                checked_usize(contribution.section.alignment, "section alignment")?;
            let size_bytes = checked_usize(contribution.section.size, "section size")?;
            section_cursor = align_up(section_cursor, contribution_alignment)?;
            let placement_offset = output_offset
                .checked_add(section_cursor)
                .ok_or_else(|| "Mach-O section placement offset overflows".to_owned())?;
            placements.push(NsldMachOSectionPlacement {
                object_id: contribution.object_id.to_owned(),
                object_role: contribution.object_role.to_owned(),
                input_section_ordinal: contribution.section.ordinal,
                input_segment_name: contribution.section.segment_name.clone(),
                input_section_name: contribution.section.name.clone(),
                output_section_id: section_id.clone(),
                output_offset: placement_offset,
                output_section_offset: section_cursor,
                size_bytes,
                alignment: contribution_alignment,
                zero_fill: contribution.section.zero_fill,
            });
            section_cursor = section_cursor
                .checked_add(size_bytes)
                .ok_or_else(|| "Mach-O merged section size overflows".to_owned())?;
        }
        image_cursor = output_offset
            .checked_add(section_cursor)
            .ok_or_else(|| "Mach-O merged image span overflows".to_owned())?;
        merged_sections.push(NsldMachOMergedSectionPlan {
            section_id,
            segment_name,
            section_name,
            flags: expected_flags,
            alignment,
            output_offset,
            size_bytes: section_cursor,
            contribution_count: contributions.len(),
            zero_fill: contributions[0].section.zero_fill,
        });
    }
    Ok((merged_sections, placements))
}

fn build_symbol_bindings(
    objects: &[&MachOLayoutObject<'_>],
    placements: &[NsldMachOSectionPlacement],
) -> Result<Vec<NsldMachOSymbolBinding>, String> {
    let mut definitions = BTreeMap::<String, SymbolDefinition<'_>>::new();
    for object in objects {
        for symbol in object
            .linkage
            .symbols
            .iter()
            .filter(|symbol| symbol.external && symbol.defined && !symbol.name.is_empty())
        {
            if let Some(previous) = definitions.insert(
                symbol.name.clone(),
                SymbolDefinition {
                    object_id: object.object_id,
                    symbol,
                },
            ) {
                return Err(format!(
                    "duplicate external Mach-O definition `{}` in object `{}` symbol {} and object `{}` symbol {}",
                    symbol.name,
                    previous.object_id,
                    previous.symbol.index,
                    object.object_id,
                    symbol.index
                ));
            }
        }
    }

    let mut bindings = Vec::new();
    for object in objects {
        for symbol in object
            .linkage
            .symbols
            .iter()
            .filter(|symbol| symbol.external && !symbol.defined && !symbol.name.is_empty())
        {
            let Some(target) = definitions.get(&symbol.name) else {
                bindings.push(NsldMachOSymbolBinding {
                    symbol: symbol.name.clone(),
                    reference_object_id: object.object_id.to_owned(),
                    reference_symbol_index: symbol.index,
                    status: "external-compatibility".to_owned(),
                    target_object_id: None,
                    target_symbol_index: None,
                    target_kind: None,
                    target_section_id: None,
                    target_output_offset: None,
                });
                continue;
            };
            let (target_section_id, target_output_offset) =
                section_symbol_target(target.object_id, target.symbol, objects, placements)?;
            bindings.push(NsldMachOSymbolBinding {
                symbol: symbol.name.clone(),
                reference_object_id: object.object_id.to_owned(),
                reference_symbol_index: symbol.index,
                status: "internal".to_owned(),
                target_object_id: Some(target.object_id.to_owned()),
                target_symbol_index: Some(target.symbol.index),
                target_kind: Some(target.symbol.kind.clone()),
                target_section_id,
                target_output_offset,
            });
        }
    }
    Ok(bindings)
}

fn section_symbol_target(
    object_id: &str,
    symbol: &ParsedMachOSymbol,
    objects: &[&MachOLayoutObject<'_>],
    placements: &[NsldMachOSectionPlacement],
) -> Result<(Option<String>, Option<usize>), String> {
    let Some(ordinal) = symbol.section_ordinal else {
        return Err(format!(
            "Mach-O definition `{}` kind `{}` is not section-backed and cannot enter placement binding yet",
            symbol.name, symbol.kind
        ));
    };
    let object = objects
        .iter()
        .find(|object| object.object_id == object_id)
        .expect("symbol definition object must remain registered");
    let section = object
        .linkage
        .sections
        .iter()
        .find(|section| section.ordinal == ordinal)
        .ok_or_else(|| {
            format!(
                "Mach-O definition `{}` references missing section ordinal {ordinal}",
                symbol.name
            )
        })?;
    let relative = symbol.value.checked_sub(section.address).ok_or_else(|| {
        format!(
            "Mach-O definition `{}` value {} precedes section `{}` address {}",
            symbol.name, symbol.value, section.name, section.address
        )
    })?;
    if relative > section.size {
        return Err(format!(
            "Mach-O definition `{}` offset {relative} exceeds section `{}` size {}",
            symbol.name, section.name, section.size
        ));
    }
    let relative = checked_usize(relative, "symbol section offset")?;
    let placement = placements
        .iter()
        .find(|placement| {
            placement.object_id == object_id && placement.input_section_ordinal == ordinal
        })
        .ok_or_else(|| {
            format!(
                "Mach-O definition `{}` has no deterministic section placement",
                symbol.name
            )
        })?;
    let target_offset = placement
        .output_offset
        .checked_add(relative)
        .ok_or_else(|| "Mach-O symbol output offset overflows".to_owned())?;
    Ok((
        Some(placement.output_section_id.clone()),
        Some(target_offset),
    ))
}

fn canonical_plan(
    status: &str,
    image_span_bytes: usize,
    merged: &[NsldMachOMergedSectionPlan],
    placements: &[NsldMachOSectionPlacement],
    bindings: &[NsldMachOSymbolBinding],
) -> String {
    let mut out = String::new();
    append_text(&mut out, MACHO_PLACEMENT_BINDING_CONTRACT);
    append_text(&mut out, status);
    writeln!(out, "image_span_bytes={image_span_bytes}").unwrap();
    for section in merged {
        out.push_str("merged\n");
        append_text(&mut out, &section.section_id);
        append_text(&mut out, &section.segment_name);
        append_text(&mut out, &section.section_name);
        writeln!(
            out,
            "facts={:08x}|{}|{}|{}|{}|{}",
            section.flags,
            section.alignment,
            section.output_offset,
            section.size_bytes,
            section.contribution_count,
            section.zero_fill
        )
        .unwrap();
    }
    for placement in placements {
        out.push_str("placement\n");
        append_text(&mut out, &placement.object_id);
        append_text(&mut out, &placement.object_role);
        append_text(&mut out, &placement.input_segment_name);
        append_text(&mut out, &placement.input_section_name);
        append_text(&mut out, &placement.output_section_id);
        writeln!(
            out,
            "facts={}|{}|{}|{}|{}|{}",
            placement.input_section_ordinal,
            placement.output_offset,
            placement.output_section_offset,
            placement.size_bytes,
            placement.alignment,
            placement.zero_fill
        )
        .unwrap();
    }
    for binding in bindings {
        out.push_str("binding\n");
        append_text(&mut out, &binding.symbol);
        append_text(&mut out, &binding.reference_object_id);
        append_text(&mut out, &binding.status);
        append_text(
            &mut out,
            binding.target_object_id.as_deref().unwrap_or("none"),
        );
        append_text(&mut out, binding.target_kind.as_deref().unwrap_or("none"));
        append_text(
            &mut out,
            binding.target_section_id.as_deref().unwrap_or("none"),
        );
        writeln!(
            out,
            "facts={}|{}|{}",
            binding.reference_symbol_index,
            binding
                .target_symbol_index
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_owned()),
            binding
                .target_output_offset
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_owned())
        )
        .unwrap();
    }
    out
}

fn append_text(out: &mut String, value: &str) {
    writeln!(out, "text:{}:{value}", value.len()).unwrap();
}

fn output_segment(section: &ParsedMachOSection) -> &str {
    if !section.segment_name.is_empty() {
        return &section.segment_name;
    }
    if section.flags & 0x8000_0400 != 0
        || matches!(
            section.name.as_str(),
            "__text" | "__const" | "__cstring" | "__eh_frame" | "__compact_unwind"
        )
    {
        "__TEXT"
    } else {
        "__DATA"
    }
}

fn object_role_rank(role: &str) -> usize {
    match role {
        "program-llvm" => 0,
        "runtime-shim" => 1,
        _ => 2,
    }
}

fn checked_usize(value: u64, label: &str) -> Result<usize, String> {
    usize::try_from(value).map_err(|_| format!("Mach-O {label} {value} exceeds host address space"))
}

fn align_up(value: usize, alignment: usize) -> Result<usize, String> {
    if alignment == 0 || !alignment.is_power_of_two() {
        return Err(format!(
            "Mach-O section alignment {alignment} is not a nonzero power of two"
        ));
    }
    value
        .checked_add(alignment - 1)
        .map(|value| value & !(alignment - 1))
        .ok_or_else(|| "Mach-O section alignment overflows".to_owned())
}

#[cfg(test)]
#[path = "final_executable_macho_layout_tests.rs"]
mod tests;
