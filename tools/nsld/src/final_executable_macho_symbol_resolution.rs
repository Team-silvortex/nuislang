use crate::{
    final_executable_macho_input::ParsedMachOSymbol,
    final_executable_macho_layout::MachOLayoutObject,
    reports::{NsldMachOCommonAllocation, NsldMachOSectionPlacement},
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone)]
pub(crate) struct MachOSymbolDefinition {
    pub(crate) object_id: String,
    pub(crate) object_role: String,
    pub(crate) symbol: ParsedMachOSymbol,
}

pub(crate) struct MachODefinitionCatalog {
    pub(crate) strong: BTreeMap<String, MachOSymbolDefinition>,
    pub(crate) common: BTreeMap<String, Vec<MachOSymbolDefinition>>,
}

pub(crate) struct ResolvedMachOSymbolTarget {
    pub(crate) object_id: String,
    pub(crate) symbol_index: usize,
    pub(crate) kind: String,
    pub(crate) section_id: Option<String>,
    pub(crate) output_offset: Option<usize>,
    pub(crate) absolute_value: Option<u64>,
    pub(crate) alias_chain: Vec<String>,
}

pub(crate) fn collect_definition_catalog(
    objects: &[&MachOLayoutObject<'_>],
) -> Result<MachODefinitionCatalog, String> {
    let mut strong = BTreeMap::new();
    let mut common = BTreeMap::<String, Vec<MachOSymbolDefinition>>::new();
    for object in objects {
        for symbol in &object.linkage.symbols {
            if symbol.kind == "common" {
                if !symbol.defined || !symbol.external || symbol.name.is_empty() {
                    return Err(format!(
                        "Mach-O common symbol {} in object `{}` must be a named external definition",
                        symbol.index, object.object_id
                    ));
                }
                common
                    .entry(symbol.name.clone())
                    .or_default()
                    .push(definition(object, symbol));
                continue;
            }
            if !symbol.external || !symbol.defined || symbol.name.is_empty() {
                continue;
            }
            let item = definition(object, symbol);
            if let Some(previous) = strong.insert(symbol.name.clone(), item) {
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
    for declarations in common.values_mut() {
        declarations.sort_by(|lhs, rhs| {
            object_role_rank(&lhs.object_role)
                .cmp(&object_role_rank(&rhs.object_role))
                .then(lhs.object_role.cmp(&rhs.object_role))
                .then(lhs.object_id.cmp(&rhs.object_id))
                .then(lhs.symbol.index.cmp(&rhs.symbol.index))
        });
    }
    Ok(MachODefinitionCatalog { strong, common })
}

pub(crate) fn resolve_definition_target(
    name: &str,
    definitions: &MachODefinitionCatalog,
    objects: &[&MachOLayoutObject<'_>],
    placements: &[NsldMachOSectionPlacement],
    common_allocations: &[NsldMachOCommonAllocation],
) -> Result<ResolvedMachOSymbolTarget, String> {
    let mut current = name.to_owned();
    let mut visited = BTreeSet::new();
    let mut alias_chain = Vec::new();
    let mut last_alias = None::<String>;
    loop {
        if !visited.insert(current.clone()) {
            alias_chain.push(current.clone());
            return Err(format!(
                "Mach-O indirect alias cycle detected: {}",
                alias_chain.join(" -> ")
            ));
        }
        let target = find_definition(definitions, &current).ok_or_else(|| {
            if let Some(alias) = last_alias.as_deref() {
                format!(
                    "Mach-O indirect alias `{alias}` target `{current}` has no external definition; chain: {}",
                    alias_path(&alias_chain, &current)
                )
            } else {
                format!("Mach-O symbol `{current}` has no external definition")
            }
        })?;
        if target.symbol.kind == "indirect" {
            alias_chain.push(current.clone());
            let next = target
                .symbol
                .indirect_target
                .as_deref()
                .filter(|value| !value.is_empty())
                .ok_or_else(|| {
                    format!(
                        "Mach-O indirect alias `{current}` in object `{}` has an empty target",
                        target.object_id
                    )
                })?;
            last_alias = Some(current);
            current = next.to_owned();
            continue;
        }
        if !alias_chain.is_empty() {
            alias_chain.push(current);
        }
        return terminal_target(target, alias_chain, objects, placements, common_allocations);
    }
}

fn terminal_target(
    target: &MachOSymbolDefinition,
    alias_chain: Vec<String>,
    objects: &[&MachOLayoutObject<'_>],
    placements: &[NsldMachOSectionPlacement],
    common_allocations: &[NsldMachOCommonAllocation],
) -> Result<ResolvedMachOSymbolTarget, String> {
    let (section_id, output_offset, absolute_value) = match target.symbol.kind.as_str() {
        "common" => {
            let allocation = common_allocations
                .iter()
                .find(|allocation| allocation.symbol == target.symbol.name)
                .ok_or_else(|| {
                    format!(
                        "Mach-O common definition `{}` has no provider allocation",
                        target.symbol.name
                    )
                })?;
            (
                Some(allocation.output_section_id.clone()),
                Some(allocation.output_offset),
                None,
            )
        }
        "absolute" => (None, None, Some(target.symbol.value)),
        "section" => {
            let (section, offset) = section_target(target, objects, placements)?;
            (Some(section), Some(offset), None)
        }
        other => {
            return Err(format!(
                "Mach-O definition `{}` has unsupported terminal kind `{other}`",
                target.symbol.name
            ));
        }
    };
    Ok(ResolvedMachOSymbolTarget {
        object_id: target.object_id.clone(),
        symbol_index: target.symbol.index,
        kind: target.symbol.kind.clone(),
        section_id,
        output_offset,
        absolute_value,
        alias_chain,
    })
}

fn section_target(
    target: &MachOSymbolDefinition,
    objects: &[&MachOLayoutObject<'_>],
    placements: &[NsldMachOSectionPlacement],
) -> Result<(String, usize), String> {
    let ordinal = target.symbol.section_ordinal.ok_or_else(|| {
        format!(
            "Mach-O section definition `{}` has no section ordinal",
            target.symbol.name
        )
    })?;
    let object = objects
        .iter()
        .find(|object| object.object_id == target.object_id)
        .expect("symbol definition object must remain registered");
    let section = object
        .linkage
        .sections
        .iter()
        .find(|section| section.ordinal == ordinal)
        .ok_or_else(|| {
            format!(
                "Mach-O definition `{}` references missing section ordinal {ordinal}",
                target.symbol.name
            )
        })?;
    let relative = target
        .symbol
        .value
        .checked_sub(section.address)
        .ok_or_else(|| {
            format!(
                "Mach-O definition `{}` value {} precedes section `{}` address {}",
                target.symbol.name, target.symbol.value, section.name, section.address
            )
        })?;
    if relative > section.size {
        return Err(format!(
            "Mach-O definition `{}` offset {relative} exceeds section `{}` size {}",
            target.symbol.name, section.name, section.size
        ));
    }
    let relative = usize::try_from(relative)
        .map_err(|_| "Mach-O symbol section offset exceeds host space".to_owned())?;
    let placement = placements
        .iter()
        .find(|placement| {
            placement.object_id == target.object_id && placement.input_section_ordinal == ordinal
        })
        .ok_or_else(|| {
            format!(
                "Mach-O definition `{}` has no deterministic section placement",
                target.symbol.name
            )
        })?;
    let output_offset = placement
        .output_offset
        .checked_add(relative)
        .ok_or_else(|| "Mach-O symbol output offset overflows".to_owned())?;
    Ok((placement.output_section_id.clone(), output_offset))
}

fn find_definition<'a>(
    definitions: &'a MachODefinitionCatalog,
    name: &str,
) -> Option<&'a MachOSymbolDefinition> {
    definitions
        .strong
        .get(name)
        .or_else(|| definitions.common.get(name).and_then(|items| items.first()))
}

fn definition(object: &MachOLayoutObject<'_>, symbol: &ParsedMachOSymbol) -> MachOSymbolDefinition {
    MachOSymbolDefinition {
        object_id: object.object_id.to_owned(),
        object_role: object.role.to_owned(),
        symbol: symbol.clone(),
    }
}

fn alias_path(chain: &[String], missing: &str) -> String {
    chain
        .iter()
        .map(String::as_str)
        .chain(std::iter::once(missing))
        .collect::<Vec<_>>()
        .join(" -> ")
}

fn object_role_rank(role: &str) -> usize {
    match role {
        "program-llvm" => 0,
        "runtime-shim" => 1,
        _ => 2,
    }
}
