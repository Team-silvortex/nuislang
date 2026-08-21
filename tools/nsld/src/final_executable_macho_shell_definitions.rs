use crate::{
    final_executable_macho_input::ParsedMachOSymbol,
    final_executable_macho_layout::MachOLayoutObject, reports::NsldMachOPlacementBindingReport,
};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) struct MachOShellDefinition<'a> {
    pub(crate) object: &'a MachOLayoutObject<'a>,
    pub(crate) symbol: &'a ParsedMachOSymbol,
    pub(crate) value: MachOShellDefinitionValue,
    pub(crate) alias: bool,
}

#[derive(Clone, Copy)]
pub(crate) enum MachOShellDefinitionValue {
    ImageOffset(usize),
    Absolute(u64),
}

pub(crate) fn collect_shell_definitions<'a>(
    objects: &'a [MachOLayoutObject<'a>],
    placement: &NsldMachOPlacementBindingReport,
) -> Result<BTreeMap<String, MachOShellDefinition<'a>>, String> {
    let mut definitions = BTreeMap::new();
    for object in objects {
        for symbol in object.linkage.symbols.iter().filter(|symbol| {
            symbol.external && symbol.defined && symbol.kind != "common" && !symbol.name.is_empty()
        }) {
            let value = definition_value(object, symbol, placement)?;
            if definitions
                .insert(
                    symbol.name.clone(),
                    MachOShellDefinition {
                        object,
                        symbol,
                        value,
                        alias: symbol.kind == "indirect",
                    },
                )
                .is_some()
            {
                return Err(format!(
                    "Mach-O shell repeats external definition `{}`",
                    symbol.name
                ));
            }
        }
    }
    append_common_definitions(objects, placement, &mut definitions)?;
    Ok(definitions)
}

pub(crate) fn collect_unresolved_shell_symbols(
    objects: &[MachOLayoutObject<'_>],
    definitions: &BTreeMap<String, MachOShellDefinition<'_>>,
) -> BTreeSet<String> {
    objects
        .iter()
        .flat_map(|object| object.linkage.symbols.iter())
        .filter(|symbol| {
            symbol.external
                && matches!(symbol.kind.as_str(), "undefined" | "prebound-undefined")
                && !symbol.name.is_empty()
        })
        .filter(|symbol| !definitions.contains_key(&symbol.name))
        .map(|symbol| symbol.name.clone())
        .collect()
}

fn append_common_definitions<'a>(
    objects: &'a [MachOLayoutObject<'a>],
    placement: &NsldMachOPlacementBindingReport,
    definitions: &mut BTreeMap<String, MachOShellDefinition<'a>>,
) -> Result<(), String> {
    for allocation in &placement.common_allocations {
        let object = objects
            .iter()
            .find(|object| object.object_id == allocation.owner_object_id)
            .ok_or_else(|| {
                format!(
                    "Mach-O common allocation `{}` references missing owner `{}`",
                    allocation.allocation_id, allocation.owner_object_id
                )
            })?;
        if object.role != allocation.owner_object_role {
            return Err(format!(
                "Mach-O common allocation `{}` owner role drift",
                allocation.allocation_id
            ));
        }
        let symbol = object
            .linkage
            .symbols
            .iter()
            .find(|symbol| symbol.index == allocation.owner_symbol_index)
            .ok_or_else(|| {
                format!(
                    "Mach-O common allocation `{}` references missing owner symbol {}",
                    allocation.allocation_id, allocation.owner_symbol_index
                )
            })?;
        if symbol.kind != "common" || symbol.name != allocation.symbol {
            return Err(format!(
                "Mach-O common allocation `{}` owner symbol identity drift",
                allocation.allocation_id
            ));
        }
        if definitions
            .insert(
                allocation.symbol.clone(),
                MachOShellDefinition {
                    object,
                    symbol,
                    value: MachOShellDefinitionValue::ImageOffset(allocation.output_offset),
                    alias: false,
                },
            )
            .is_some()
        {
            return Err(format!(
                "Mach-O common allocation `{}` conflicts with a strong definition",
                allocation.allocation_id
            ));
        }
    }
    Ok(())
}

fn definition_value(
    object: &MachOLayoutObject<'_>,
    symbol: &ParsedMachOSymbol,
    placement: &NsldMachOPlacementBindingReport,
) -> Result<MachOShellDefinitionValue, String> {
    if symbol.kind == "section" {
        return section_definition_source(object, symbol, placement)
            .map(MachOShellDefinitionValue::ImageOffset);
    }
    if matches!(symbol.kind.as_str(), "absolute" | "indirect") {
        let binding = placement
            .symbol_bindings
            .iter()
            .find(|binding| {
                binding.reference_object_id == object.object_id
                    && binding.reference_symbol_index == symbol.index
            })
            .ok_or_else(|| {
                format!(
                    "Mach-O shell definition `{}` has no provider resolution binding",
                    symbol.name
                )
            })?;
        return match (binding.target_output_offset, binding.target_absolute_value) {
            (Some(offset), None) => Ok(MachOShellDefinitionValue::ImageOffset(offset)),
            (None, Some(value)) => Ok(MachOShellDefinitionValue::Absolute(value)),
            (None, None) => Err(format!(
                "Mach-O shell definition `{}` has no resolved target value",
                symbol.name
            )),
            (Some(_), Some(_)) => Err(format!(
                "Mach-O shell definition `{}` has ambiguous target values",
                symbol.name
            )),
        };
    }
    Err(format!(
        "Mach-O shell cannot lower definition `{}` kind `{}`",
        symbol.name, symbol.kind
    ))
}

fn section_definition_source(
    object: &MachOLayoutObject<'_>,
    symbol: &ParsedMachOSymbol,
    placement: &NsldMachOPlacementBindingReport,
) -> Result<usize, String> {
    let ordinal = symbol.section_ordinal.ok_or_else(|| {
        format!(
            "Mach-O shell cannot place unsupported non-section definition `{}` kind `{}`",
            symbol.name, symbol.kind
        )
    })?;
    let section = object
        .linkage
        .sections
        .iter()
        .find(|section| section.ordinal == ordinal)
        .ok_or_else(|| {
            format!(
                "Mach-O shell definition `{}` references missing section {ordinal}",
                symbol.name
            )
        })?;
    let relative = symbol.value.checked_sub(section.address).ok_or_else(|| {
        format!(
            "Mach-O shell definition `{}` precedes its section address",
            symbol.name
        )
    })?;
    if relative >= section.size {
        return Err(format!(
            "Mach-O shell definition `{}` offset {relative} exceeds section size {}",
            symbol.name, section.size
        ));
    }
    let placement = placement
        .section_placements
        .iter()
        .find(|item| item.object_id == object.object_id && item.input_section_ordinal == ordinal)
        .ok_or_else(|| {
            format!(
                "Mach-O shell definition `{}` has no section placement",
                symbol.name
            )
        })?;
    let relative = usize::try_from(relative)
        .map_err(|_| "Mach-O definition offset exceeds host space".to_owned())?;
    placement
        .output_offset
        .checked_add(relative)
        .ok_or_else(|| "Mach-O definition source offset overflows".to_owned())
}
