use crate::{
    final_executable_macho_input::{ParsedMachORelocation, ParsedMachOSymbol},
    final_executable_macho_layout::MachOLayoutObject,
    reports::{
        NsldMachOArm64RelocationApplication, NsldMachOArm64RelocationApplicationReport,
        NsldMachOPlacementBindingReport, NsldMachOSectionPlacement,
    },
};
use std::collections::BTreeSet;
use std::fmt::Write as _;

pub(crate) const MACHO_ARM64_RELOCATION_APPLICATION_CONTRACT: &str =
    "nuis-nsld-macho-arm64-relocation-application-v1";

const ARM64_RELOC_UNSIGNED: u32 = 0;
const ARM64_RELOC_SUBTRACTOR: u32 = 1;
const ARM64_RELOC_BRANCH26: u32 = 2;
const ARM64_RELOC_PAGE21: u32 = 3;
const ARM64_RELOC_PAGEOFF12: u32 = 4;
const ARM64_RELOC_GOT_LOAD_PAGE21: u32 = 5;
const ARM64_RELOC_GOT_LOAD_PAGEOFF12: u32 = 6;
const ARM64_RELOC_ADDEND: u32 = 10;

#[derive(Clone, Copy)]
struct RelocationShape {
    kind: &'static str,
    action: &'static str,
    metadata: bool,
    requires_linker_structure: bool,
}

#[derive(Default)]
struct TargetResolution {
    symbol: Option<String>,
    symbol_index: Option<usize>,
    object_id: Option<String>,
    section_id: Option<String>,
    output_offset: Option<usize>,
    absolute_value: Option<u64>,
    alias_chain: Vec<String>,
    status: String,
}

pub(crate) fn build_macho_arm64_relocation_application_report(
    objects: &[MachOLayoutObject<'_>],
    placement: &NsldMachOPlacementBindingReport,
) -> Result<NsldMachOArm64RelocationApplicationReport, String> {
    let objects = sorted_objects(objects)?;
    let mut applications = Vec::new();
    let mut registered_kinds = BTreeSet::new();
    let mut next_id = 0usize;

    for object in objects {
        let ids = object
            .linkage
            .relocations
            .iter()
            .map(|_| {
                let id = format!("macho-arm64-reloc-{next_id:06}");
                next_id += 1;
                id
            })
            .collect::<Vec<_>>();
        for (index, relocation) in object.linkage.relocations.iter().enumerate() {
            let shape = registered_shape(relocation)?;
            registered_kinds.insert(shape.kind);
            let pair_relocation_id =
                paired_relocation_id(&object.object_id, index, &object.linkage.relocations, &ids)?;
            let source =
                source_placement(&object.object_id, relocation, &placement.section_placements)?;
            let source_offset = usize::try_from(relocation.offset).map_err(|_| {
                format!(
                    "Mach-O object `{}` relocation offset {} exceeds host address space",
                    object.object_id, relocation.offset
                )
            })?;
            let width_bytes = usize::try_from(relocation.width_bytes).map_err(|_| {
                format!(
                    "Mach-O object `{}` relocation width {} exceeds host address space",
                    object.object_id, relocation.width_bytes
                )
            })?;
            let source_end = source_offset
                .checked_add(width_bytes)
                .ok_or_else(|| "Mach-O relocation source span overflows".to_owned())?;
            if source_end > source.size_bytes {
                return Err(format!(
                    "Mach-O object `{}` relocation source span {source_offset}..{source_end} exceeds placed section size {}",
                    object.object_id, source.size_bytes
                ));
            }
            let source_output_offset = source
                .output_offset
                .checked_add(source_offset)
                .ok_or_else(|| "Mach-O relocation output offset overflows".to_owned())?;
            let target = resolve_target(object, relocation, placement)?;
            let explicit_addend = (relocation.relocation_type == ARM64_RELOC_ADDEND)
                .then(|| decode_addend(relocation.symbol_number));
            let application_status = if shape.metadata {
                "paired-metadata"
            } else if shape.requires_linker_structure || target.status == "external-compatibility" {
                "planned-platform-structure"
            } else {
                "planned-direct"
            };

            applications.push(NsldMachOArm64RelocationApplication {
                relocation_id: ids[index].clone(),
                object_id: object.object_id.to_owned(),
                object_role: object.role.to_owned(),
                input_section_ordinal: relocation.section_ordinal,
                source_section_id: source.output_section_id.clone(),
                source_offset,
                source_output_offset,
                width_bytes,
                pc_relative: relocation.pc_relative,
                external: relocation.external,
                relocation_type: relocation.relocation_type,
                relocation_kind: shape.kind.to_owned(),
                action_kind: shape.action.to_owned(),
                target_symbol: target.symbol,
                target_symbol_index: target.symbol_index,
                target_object_id: target.object_id,
                target_section_id: target.section_id,
                target_output_offset: target.output_offset,
                target_absolute_value: target.absolute_value,
                target_alias_chain: target.alias_chain,
                explicit_addend,
                pair_relocation_id,
                resolver_status: target.status,
                application_status: application_status.to_owned(),
            });
        }
    }

    let ready_application_count = applications
        .iter()
        .filter(|item| item.application_status == "planned-direct")
        .count();
    let platform_structure_count = applications
        .iter()
        .filter(|item| item.application_status == "planned-platform-structure")
        .count();
    let metadata_record_count = applications
        .iter()
        .filter(|item| item.application_status == "paired-metadata")
        .count();
    let external_compatibility_count = applications
        .iter()
        .filter(|item| item.resolver_status == "external-compatibility")
        .count();
    let status = if platform_structure_count == 0 {
        "ready-for-byte-encoding"
    } else {
        "planned-with-platform-structure-boundary"
    };
    let canonical = canonical_plan(status, &placement.plan_hash, &applications);

    Ok(NsldMachOArm64RelocationApplicationReport {
        contract: MACHO_ARM64_RELOCATION_APPLICATION_CONTRACT.to_owned(),
        status: status.to_owned(),
        plan_hash: crate::fnv1a64_hex(canonical.as_bytes()),
        placement_plan_hash: placement.plan_hash.clone(),
        relocation_count: applications.len(),
        registered_kind_count: registered_kinds.len(),
        ready_application_count,
        platform_structure_count,
        external_compatibility_count,
        metadata_record_count,
        applications,
    })
}

fn sorted_objects<'a>(
    objects: &'a [MachOLayoutObject<'a>],
) -> Result<Vec<&'a MachOLayoutObject<'a>>, String> {
    let mut seen = BTreeSet::new();
    for object in objects {
        if !seen.insert(object.object_id) {
            return Err(format!(
                "Mach-O relocation application input contains duplicate object id `{}`",
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

fn registered_shape(relocation: &ParsedMachORelocation) -> Result<RelocationShape, String> {
    let (shape, expected_pc_relative, expected_external, widths) =
        match relocation.relocation_type {
            ARM64_RELOC_UNSIGNED => (
                RelocationShape {
                    kind: "arm64-unsigned",
                    action: "write-absolute",
                    metadata: false,
                    requires_linker_structure: false,
                },
                false,
                None,
                &[4, 8][..],
            ),
            ARM64_RELOC_SUBTRACTOR => (
                RelocationShape {
                    kind: "arm64-subtractor",
                    action: "paired-subtractor",
                    metadata: true,
                    requires_linker_structure: false,
                },
                false,
                Some(true),
                &[4, 8][..],
            ),
            ARM64_RELOC_BRANCH26 => (instruction_shape("arm64-branch26", "rewrite-branch26"), true, Some(true), &[4][..]),
            ARM64_RELOC_PAGE21 => (instruction_shape("arm64-page21", "rewrite-page21"), true, Some(true), &[4][..]),
            ARM64_RELOC_PAGEOFF12 => (instruction_shape("arm64-pageoff12", "rewrite-pageoff12"), false, Some(true), &[4][..]),
            ARM64_RELOC_GOT_LOAD_PAGE21 => (linker_shape("arm64-got-load-page21", "rewrite-got-load-page21"), true, Some(true), &[4][..]),
            ARM64_RELOC_GOT_LOAD_PAGEOFF12 => (linker_shape("arm64-got-load-pageoff12", "rewrite-got-load-pageoff12"), false, Some(true), &[4][..]),
            ARM64_RELOC_ADDEND => (
                RelocationShape {
                    kind: "arm64-addend",
                    action: "paired-addend",
                    metadata: true,
                    requires_linker_structure: false,
                },
                false,
                Some(false),
                &[4][..],
            ),
            other => {
                return Err(format!(
                    "unregistered ARM64 Mach-O relocation type {other}; the provider fails closed outside its static application registry"
                ))
            }
        };
    if relocation.pc_relative != expected_pc_relative {
        return Err(format!(
            "{} relocation has pc_relative={}, expected {}",
            shape.kind, relocation.pc_relative, expected_pc_relative
        ));
    }
    if expected_external.is_some_and(|expected| relocation.external != expected) {
        return Err(format!(
            "{} relocation has external={}, expected {}",
            shape.kind,
            relocation.external,
            expected_external.unwrap()
        ));
    }
    if !widths.contains(&relocation.width_bytes) {
        return Err(format!(
            "{} relocation has width {}, expected one of {:?}",
            shape.kind, relocation.width_bytes, widths
        ));
    }
    Ok(shape)
}

fn instruction_shape(kind: &'static str, action: &'static str) -> RelocationShape {
    RelocationShape {
        kind,
        action,
        metadata: false,
        requires_linker_structure: false,
    }
}

fn linker_shape(kind: &'static str, action: &'static str) -> RelocationShape {
    RelocationShape {
        kind,
        action,
        metadata: false,
        requires_linker_structure: true,
    }
}

fn paired_relocation_id(
    object_id: &str,
    index: usize,
    relocations: &[ParsedMachORelocation],
    ids: &[String],
) -> Result<Option<String>, String> {
    let relocation = &relocations[index];
    if relocation.relocation_type == ARM64_RELOC_SUBTRACTOR {
        let pair = require_pair(object_id, index, relocations, ARM64_RELOC_UNSIGNED)?;
        return Ok(Some(ids[pair].clone()));
    }
    if relocation.relocation_type == ARM64_RELOC_ADDEND {
        let pair = relocations.get(index + 1).ok_or_else(|| {
            format!("Mach-O object `{object_id}` ADDEND relocation {index} has no paired record")
        })?;
        if !matches!(
            pair.relocation_type,
            ARM64_RELOC_PAGE21 | ARM64_RELOC_PAGEOFF12
        ) || !same_source(relocation, pair)
        {
            return Err(format!(
                "Mach-O object `{object_id}` ADDEND relocation {index} must precede a same-source PAGE21 or PAGEOFF12 record"
            ));
        }
        return Ok(Some(ids[index + 1].clone()));
    }
    if index > 0 {
        let previous = &relocations[index - 1];
        if relocation.relocation_type == ARM64_RELOC_UNSIGNED
            && previous.relocation_type == ARM64_RELOC_SUBTRACTOR
            && same_source(previous, relocation)
        {
            return Ok(Some(ids[index - 1].clone()));
        }
        if matches!(
            relocation.relocation_type,
            ARM64_RELOC_PAGE21 | ARM64_RELOC_PAGEOFF12
        ) && previous.relocation_type == ARM64_RELOC_ADDEND
            && same_source(previous, relocation)
        {
            return Ok(Some(ids[index - 1].clone()));
        }
    }
    Ok(None)
}

fn require_pair(
    object_id: &str,
    index: usize,
    relocations: &[ParsedMachORelocation],
    expected_type: u32,
) -> Result<usize, String> {
    let relocation = &relocations[index];
    let pair = relocations.get(index + 1).ok_or_else(|| {
        format!(
            "Mach-O object `{object_id}` relocation type {} at index {index} has no paired record",
            relocation.relocation_type
        )
    })?;
    if pair.relocation_type != expected_type || !same_source(relocation, pair) || !pair.external {
        return Err(format!(
            "Mach-O object `{object_id}` SUBTRACTOR relocation {index} must precede a same-source external UNSIGNED record"
        ));
    }
    Ok(index + 1)
}

fn same_source(lhs: &ParsedMachORelocation, rhs: &ParsedMachORelocation) -> bool {
    lhs.section_ordinal == rhs.section_ordinal
        && lhs.offset == rhs.offset
        && lhs.width_bytes == rhs.width_bytes
}

fn source_placement<'a>(
    object_id: &str,
    relocation: &ParsedMachORelocation,
    placements: &'a [NsldMachOSectionPlacement],
) -> Result<&'a NsldMachOSectionPlacement, String> {
    placements
        .iter()
        .find(|item| {
            item.object_id == object_id && item.input_section_ordinal == relocation.section_ordinal
        })
        .ok_or_else(|| {
            format!(
                "Mach-O object `{object_id}` relocation has no placement for section ordinal {}",
                relocation.section_ordinal
            )
        })
}

fn resolve_target(
    object: &MachOLayoutObject<'_>,
    relocation: &ParsedMachORelocation,
    placement: &NsldMachOPlacementBindingReport,
) -> Result<TargetResolution, String> {
    if relocation.relocation_type == ARM64_RELOC_ADDEND {
        return Ok(TargetResolution {
            status: "paired-addend".to_owned(),
            ..TargetResolution::default()
        });
    }
    if !relocation.external {
        if relocation.symbol_number == 0 {
            return Err(format!(
                "Mach-O object `{}` local relocation references reserved section ordinal 0",
                object.object_id
            ));
        }
        let target = placement
            .section_placements
            .iter()
            .find(|item| {
                item.object_id == object.object_id
                    && item.input_section_ordinal == relocation.symbol_number
            })
            .ok_or_else(|| {
                format!(
                    "Mach-O object `{}` local relocation references unplaced section ordinal {}",
                    object.object_id, relocation.symbol_number
                )
            })?;
        return Ok(TargetResolution {
            object_id: Some(object.object_id.to_owned()),
            section_id: Some(target.output_section_id.clone()),
            output_offset: Some(target.output_offset),
            status: "local-section".to_owned(),
            ..TargetResolution::default()
        });
    }

    let symbol = object
        .linkage
        .symbols
        .get(relocation.symbol_number)
        .ok_or_else(|| {
            format!(
                "Mach-O object `{}` relocation references missing symbol index {}",
                object.object_id, relocation.symbol_number
            )
        })?;
    if matches!(symbol.kind.as_str(), "common" | "absolute" | "indirect") {
        let binding = placement
            .symbol_bindings
            .iter()
            .find(|item| {
                item.reference_object_id == object.object_id
                    && item.reference_symbol_index == symbol.index
            })
            .ok_or_else(|| {
                format!(
                    "Mach-O object `{}` non-section relocation symbol `{}` has no placement binding",
                    object.object_id, symbol.name
                )
            })?;
        return Ok(TargetResolution {
            symbol: Some(symbol.name.clone()),
            symbol_index: Some(symbol.index),
            object_id: binding.target_object_id.clone(),
            section_id: binding.target_section_id.clone(),
            output_offset: binding.target_output_offset,
            absolute_value: binding.target_absolute_value,
            alias_chain: binding.alias_chain.clone(),
            status: binding.status.clone(),
        });
    }
    if symbol.defined {
        let (section_id, output_offset) = resolve_defined_symbol(object, symbol, placement)?;
        return Ok(TargetResolution {
            symbol: Some(symbol.name.clone()),
            symbol_index: Some(symbol.index),
            object_id: Some(object.object_id.to_owned()),
            section_id: Some(section_id),
            output_offset: Some(output_offset),
            status: "internal-symbol".to_owned(),
            ..TargetResolution::default()
        });
    }
    let binding = placement
        .symbol_bindings
        .iter()
        .find(|item| {
            item.reference_object_id == object.object_id
                && item.reference_symbol_index == symbol.index
        })
        .ok_or_else(|| {
            format!(
                "Mach-O object `{}` undefined relocation symbol `{}` has no placement binding",
                object.object_id, symbol.name
            )
        })?;
    Ok(TargetResolution {
        symbol: Some(symbol.name.clone()),
        symbol_index: Some(symbol.index),
        object_id: binding.target_object_id.clone(),
        section_id: binding.target_section_id.clone(),
        output_offset: binding.target_output_offset,
        absolute_value: binding.target_absolute_value,
        alias_chain: binding.alias_chain.clone(),
        status: binding.status.clone(),
    })
}

fn resolve_defined_symbol(
    object: &MachOLayoutObject<'_>,
    symbol: &ParsedMachOSymbol,
    placement: &NsldMachOPlacementBindingReport,
) -> Result<(String, usize), String> {
    let ordinal = symbol.section_ordinal.ok_or_else(|| {
        format!(
            "Mach-O relocation target `{}` kind `{}` is not section-backed",
            symbol.name, symbol.kind
        )
    })?;
    let section = object
        .linkage
        .sections
        .iter()
        .find(|item| item.ordinal == ordinal)
        .ok_or_else(|| {
            format!(
                "Mach-O relocation target `{}` references missing section ordinal {ordinal}",
                symbol.name
            )
        })?;
    let relative = symbol.value.checked_sub(section.address).ok_or_else(|| {
        format!(
            "Mach-O relocation target `{}` precedes section `{}`",
            symbol.name, section.name
        )
    })?;
    if relative > section.size {
        return Err(format!(
            "Mach-O relocation target `{}` offset {relative} exceeds section `{}` size {}",
            symbol.name, section.name, section.size
        ));
    }
    let relative = usize::try_from(relative).map_err(|_| {
        format!(
            "Mach-O relocation target `{}` offset overflows",
            symbol.name
        )
    })?;
    let target = placement
        .section_placements
        .iter()
        .find(|item| item.object_id == object.object_id && item.input_section_ordinal == ordinal)
        .ok_or_else(|| {
            format!(
                "Mach-O relocation target `{}` has no section placement",
                symbol.name
            )
        })?;
    let output_offset = target
        .output_offset
        .checked_add(relative)
        .ok_or_else(|| "Mach-O relocation target output offset overflows".to_owned())?;
    Ok((target.output_section_id.clone(), output_offset))
}

fn decode_addend(value: usize) -> i64 {
    let raw = (value & 0x00ff_ffff) as i64;
    if raw & 0x0080_0000 != 0 {
        raw - 0x0100_0000
    } else {
        raw
    }
}

fn canonical_plan(
    status: &str,
    placement_plan_hash: &str,
    applications: &[NsldMachOArm64RelocationApplication],
) -> String {
    let mut out = String::new();
    append_text(&mut out, MACHO_ARM64_RELOCATION_APPLICATION_CONTRACT);
    append_text(&mut out, status);
    append_text(&mut out, placement_plan_hash);
    for item in applications {
        append_text(&mut out, &item.relocation_id);
        append_text(&mut out, &item.object_id);
        append_text(&mut out, &item.object_role);
        append_text(&mut out, &item.source_section_id);
        append_text(&mut out, &item.relocation_kind);
        append_text(&mut out, &item.action_kind);
        append_text(&mut out, item.target_symbol.as_deref().unwrap_or("none"));
        append_text(&mut out, item.target_object_id.as_deref().unwrap_or("none"));
        append_text(
            &mut out,
            item.target_section_id.as_deref().unwrap_or("none"),
        );
        append_text(
            &mut out,
            item.pair_relocation_id.as_deref().unwrap_or("none"),
        );
        for alias in &item.target_alias_chain {
            append_text(&mut out, alias);
        }
        append_text(&mut out, &item.resolver_status);
        append_text(&mut out, &item.application_status);
        writeln!(
            out,
            "facts={}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
            item.input_section_ordinal,
            item.source_offset,
            item.source_output_offset,
            item.width_bytes,
            item.pc_relative,
            item.external,
            item.relocation_type,
            optional_usize(item.target_symbol_index),
            optional_usize(item.target_output_offset),
            item.target_absolute_value
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_owned()),
            item.target_alias_chain.len(),
            item.explicit_addend
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_owned())
        )
        .unwrap();
    }
    out
}

fn optional_usize(value: Option<usize>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_owned())
}

fn append_text(out: &mut String, value: &str) {
    writeln!(out, "text:{}:{value}", value.len()).unwrap();
}

fn object_role_rank(role: &str) -> usize {
    match role {
        "program-llvm" => 0,
        "runtime-shim" => 1,
        _ => 2,
    }
}

#[cfg(test)]
#[path = "final_executable_macho_relocation_tests.rs"]
mod tests;
