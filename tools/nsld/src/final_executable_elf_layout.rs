use crate::{
    final_executable_elf_input::{ParsedElfSection, ParsedElfSymbol},
    final_executable_elf_layout_report::{
        ElfAmd64CommonAllocation, ElfAmd64MergedSectionPlan, ElfAmd64PlacementBindingReport,
        ElfAmd64SectionPlacement, ElfAmd64SymbolBinding,
    },
    final_executable_elf_object::ElfAmd64ObjectLinkage,
};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const ELF_AMD64_PLACEMENT_BINDING_CONTRACT: &str =
    "nuis-nsld-elf-amd64-placement-binding-v1";
pub(crate) const ELF_AMD64_IMAGE_BASE: u64 = 0x0040_0000;
pub(crate) const ELF_AMD64_PAYLOAD_FILE_OFFSET: usize = 0x1000;
pub(crate) const ELF_AMD64_PAGE_SIZE: usize = 0x1000;

const ELF_SECTION_FLAG_WRITE: u64 = 0x1;
const ELF_SECTION_FLAG_ALLOCATE: u64 = 0x2;
const ELF_SECTION_FLAG_EXECUTE: u64 = 0x4;
const ELF_SECTION_FLAG_TLS: u64 = 0x400;
const ELF_SECTION_FLAG_COMPRESSED: u64 = 0x800;
const ELF_AMD64_MAX_INPUT_ALIGNMENT: usize = 0x20_0000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum SectionClass {
    Text,
    ReadOnlyData,
    Data,
    Bss,
}

impl SectionClass {
    fn label(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::ReadOnlyData => "rodata",
            Self::Data => "data",
            Self::Bss => "bss",
        }
    }

    fn section_name(self) -> &'static str {
        match self {
            Self::Text => ".text",
            Self::ReadOnlyData => ".rodata",
            Self::Data => ".data",
            Self::Bss => ".bss",
        }
    }

    fn zero_fill(self) -> bool {
        self == Self::Bss
    }
}

#[derive(Clone, Copy)]
struct SectionContribution<'a> {
    object: &'a ElfAmd64ObjectLinkage,
    object_order: usize,
    section: &'a ParsedElfSection,
}

#[derive(Clone, Copy)]
struct SymbolRef<'a> {
    object: &'a ElfAmd64ObjectLinkage,
    symbol: &'a ParsedElfSymbol,
}

struct DefinitionCatalog<'a> {
    strong: BTreeMap<String, SymbolRef<'a>>,
    weak: BTreeMap<String, Vec<SymbolRef<'a>>>,
    common: BTreeMap<String, Vec<SymbolRef<'a>>>,
    local_common: Vec<SymbolRef<'a>>,
}

#[derive(Clone)]
struct ResolvedTarget {
    object_id: String,
    symbol_index: usize,
    kind: String,
    section_id: Option<String>,
    image_offset: Option<usize>,
    virtual_address: Option<u64>,
    absolute_value: Option<u64>,
}

pub(crate) fn build_elf_amd64_placement_binding(
    objects: &[ElfAmd64ObjectLinkage],
) -> Result<ElfAmd64PlacementBindingReport, String> {
    let objects = sorted_objects(objects)?;
    let definitions = collect_definitions(&objects)?;
    let (mut merged_sections, section_placements, mut file_span, mut memory_span) =
        build_section_layout(&objects)?;
    let common_allocations =
        append_common_allocations(&definitions, &mut merged_sections, &mut memory_span)?;
    file_span = file_span.max(ELF_AMD64_PAYLOAD_FILE_OFFSET);
    let symbol_bindings = build_symbol_bindings(
        &objects,
        &definitions,
        &section_placements,
        &common_allocations,
    )?;
    let internally_bound_symbol_count = symbol_bindings
        .iter()
        .filter(|binding| binding.status == "internal")
        .count();
    let external_compatibility_symbol_count = symbol_bindings
        .iter()
        .filter(|binding| binding.status == "external-compatibility")
        .count();
    let status = if external_compatibility_symbol_count == 0 {
        "placement-and-internal-binding-ready"
    } else {
        "placement-ready-with-external-compatibility-boundary"
    };
    let mut report = ElfAmd64PlacementBindingReport {
        contract: ELF_AMD64_PLACEMENT_BINDING_CONTRACT,
        status: status.to_owned(),
        plan_hash: String::new(),
        image_base: ELF_AMD64_IMAGE_BASE,
        payload_file_offset: ELF_AMD64_PAYLOAD_FILE_OFFSET,
        file_span_bytes: file_span,
        memory_span_bytes: memory_span,
        merged_sections,
        section_placements,
        common_allocations,
        symbol_bindings,
        internally_bound_symbol_count,
        external_compatibility_symbol_count,
    };
    report.plan_hash = crate::fnv1a64_hex(report.canonical_plan().as_bytes());
    Ok(report)
}

fn sorted_objects(
    objects: &[ElfAmd64ObjectLinkage],
) -> Result<Vec<&ElfAmd64ObjectLinkage>, String> {
    let mut seen = BTreeSet::new();
    for object in objects {
        if !seen.insert(object.object_id.as_str()) {
            return Err(format!(
                "ELF placement input contains duplicate object id `{}`",
                object.object_id
            ));
        }
    }
    let mut sorted = objects.iter().collect::<Vec<_>>();
    sorted.sort_by(|lhs, rhs| {
        object_role_rank(&lhs.role)
            .cmp(&object_role_rank(&rhs.role))
            .then(lhs.role.cmp(&rhs.role))
            .then(lhs.object_id.cmp(&rhs.object_id))
    });
    Ok(sorted)
}

fn build_section_layout(
    objects: &[&ElfAmd64ObjectLinkage],
) -> Result<
    (
        Vec<ElfAmd64MergedSectionPlan>,
        Vec<ElfAmd64SectionPlacement>,
        usize,
        usize,
    ),
    String,
> {
    let mut groups = BTreeMap::<SectionClass, Vec<SectionContribution<'_>>>::new();
    for (object_order, object) in objects.iter().enumerate() {
        for section in object
            .linkage
            .sections
            .iter()
            .filter(|section| section.flags & ELF_SECTION_FLAG_ALLOCATE != 0)
        {
            let class = classify_section(object, section)?;
            groups.entry(class).or_default().push(SectionContribution {
                object,
                object_order,
                section,
            });
        }
    }

    let mut merged = Vec::new();
    let mut placements = Vec::new();
    let mut image_cursor = ELF_AMD64_PAYLOAD_FILE_OFFSET;
    let mut file_span = ELF_AMD64_PAYLOAD_FILE_OFFSET;
    for (class, mut contributions) in groups {
        contributions.sort_by_key(|item| (item.object_order, item.section.index));
        let alignment = contributions
            .iter()
            .map(|item| checked_alignment(item.section.alignment, &item.section.name))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .max()
            .unwrap_or(1)
            .max(ELF_AMD64_PAGE_SIZE);
        image_cursor = align_up(image_cursor, alignment)?;
        let output_offset = image_cursor;
        let section_id = format!("elf-section-{:04}", merged.len());
        let mut section_cursor = 0usize;
        for contribution in &contributions {
            let contribution_alignment =
                checked_alignment(contribution.section.alignment, &contribution.section.name)?;
            section_cursor = align_up(section_cursor, contribution_alignment)?;
            let image_offset = checked_add(
                output_offset,
                section_cursor,
                "ELF section placement offset",
            )?;
            let virtual_address = virtual_address(image_offset)?;
            let file_offset = (!class.zero_fill()).then_some(image_offset);
            placements.push(ElfAmd64SectionPlacement {
                object_id: contribution.object.object_id.clone(),
                object_role: contribution.object.role.clone(),
                input_section_index: contribution.section.index,
                input_section_name: contribution.section.name.clone(),
                output_section_id: section_id.clone(),
                output_section_offset: section_cursor,
                alignment: contribution_alignment,
                size_bytes: contribution.section.size,
                file_offset,
                image_offset,
                virtual_address,
                zero_fill: contribution.section.zero_fill,
            });
            section_cursor = checked_add(
                section_cursor,
                contribution.section.size,
                "ELF merged section size",
            )?;
        }
        image_cursor = checked_add(output_offset, section_cursor, "ELF image span")?;
        if !class.zero_fill() {
            file_span = image_cursor;
        }
        merged.push(ElfAmd64MergedSectionPlan {
            section_id,
            output_section_name: class.section_name().to_owned(),
            class: class.label().to_owned(),
            alignment,
            file_offset: (!class.zero_fill()).then_some(output_offset),
            image_offset: output_offset,
            virtual_address: virtual_address(output_offset)?,
            size_bytes: section_cursor,
            contribution_count: contributions.len(),
            zero_fill: class.zero_fill(),
        });
    }
    Ok((merged, placements, file_span, image_cursor))
}

fn classify_section(
    object: &ElfAmd64ObjectLinkage,
    section: &ParsedElfSection,
) -> Result<SectionClass, String> {
    if section.flags & ELF_SECTION_FLAG_TLS != 0 {
        return Err(format!(
            "ELF object `{}` alloc section `{}` uses unsupported TLS placement",
            object.object_id, section.name
        ));
    }
    if section.flags & ELF_SECTION_FLAG_COMPRESSED != 0 {
        return Err(format!(
            "ELF object `{}` alloc section `{}` is compressed",
            object.object_id, section.name
        ));
    }
    let writable = section.flags & ELF_SECTION_FLAG_WRITE != 0;
    let executable = section.flags & ELF_SECTION_FLAG_EXECUTE != 0;
    if writable && executable {
        return Err(format!(
            "ELF object `{}` alloc section `{}` requests writable-executable placement",
            object.object_id, section.name
        ));
    }
    if section.zero_fill {
        return writable.then_some(SectionClass::Bss).ok_or_else(|| {
            format!(
                "ELF object `{}` zero-fill alloc section `{}` is not writable",
                object.object_id, section.name
            )
        });
    }
    Ok(if executable {
        SectionClass::Text
    } else if writable {
        SectionClass::Data
    } else {
        SectionClass::ReadOnlyData
    })
}

fn collect_definitions<'a>(
    objects: &[&'a ElfAmd64ObjectLinkage],
) -> Result<DefinitionCatalog<'a>, String> {
    let mut catalog = DefinitionCatalog {
        strong: BTreeMap::new(),
        weak: BTreeMap::new(),
        common: BTreeMap::new(),
        local_common: Vec::new(),
    };
    for object in objects {
        for symbol in object
            .linkage
            .symbols
            .iter()
            .skip(1)
            .filter(|symbol| symbol.defined)
        {
            let item = SymbolRef { object, symbol };
            if symbol.common {
                if symbol.external {
                    catalog
                        .common
                        .entry(symbol.name.clone())
                        .or_default()
                        .push(item);
                } else {
                    catalog.local_common.push(item);
                }
            } else if symbol.external && symbol.weak {
                catalog
                    .weak
                    .entry(symbol.name.clone())
                    .or_default()
                    .push(item);
            } else if symbol.external {
                if let Some(previous) = catalog.strong.insert(symbol.name.clone(), item) {
                    return Err(format!(
                        "ELF strong symbol `{}` is defined by both `{}` and `{}`",
                        symbol.name, previous.object.object_id, object.object_id
                    ));
                }
            }
        }
    }
    Ok(catalog)
}

fn append_common_allocations(
    definitions: &DefinitionCatalog<'_>,
    merged: &mut Vec<ElfAmd64MergedSectionPlan>,
    memory_span: &mut usize,
) -> Result<Vec<ElfAmd64CommonAllocation>, String> {
    let mut declarations = definitions
        .common
        .iter()
        .filter(|(name, _)| !definitions.strong.contains_key(*name))
        .map(|(name, items)| {
            (
                format!("external:{name}"),
                name.clone(),
                true,
                items.clone(),
            )
        })
        .collect::<Vec<_>>();
    declarations.extend(definitions.local_common.iter().map(|item| {
        (
            format!("local:{}:{}", item.object.object_id, item.symbol.index),
            item.symbol.name.clone(),
            false,
            vec![*item],
        )
    }));
    declarations.sort_by(|lhs, rhs| lhs.0.cmp(&rhs.0));
    if declarations.is_empty() {
        return Ok(Vec::new());
    }
    let section_alignment = declarations
        .iter()
        .try_fold(1usize, |maximum, (_, _, _, items)| {
            items.iter().try_fold(maximum, |maximum, item| {
                Ok::<usize, String>(maximum.max(common_alignment(item)?))
            })
        })?
        .max(ELF_AMD64_PAGE_SIZE);
    *memory_span = align_up(*memory_span, section_alignment)?;
    let section_offset = *memory_span;
    let section_id = format!("elf-section-{:04}", merged.len());
    let mut section_cursor = 0usize;
    let mut allocations = Vec::with_capacity(declarations.len());
    for (_, name, external, items) in declarations {
        let alignment = items.iter().try_fold(1usize, |maximum, item| {
            Ok::<usize, String>(maximum.max(common_alignment(item)?))
        })?;
        let size_bytes = items.iter().try_fold(0usize, |maximum, item| {
            Ok::<usize, String>(maximum.max(common_size(item)?))
        })?;
        section_cursor = align_up(section_cursor, alignment)?;
        let image_offset = checked_add(section_offset, section_cursor, "ELF common offset")?;
        let owner = items[0];
        allocations.push(ElfAmd64CommonAllocation {
            allocation_id: format!("elf-common-{:04}", allocations.len()),
            symbol: name,
            external,
            owner_object_id: owner.object.object_id.clone(),
            owner_object_role: owner.object.role.clone(),
            owner_symbol_index: owner.symbol.index,
            declaration_count: items.len(),
            size_bytes,
            alignment,
            output_section_id: section_id.clone(),
            output_section_offset: section_cursor,
            image_offset,
            virtual_address: virtual_address(image_offset)?,
        });
        section_cursor = checked_add(section_cursor, size_bytes, "ELF common section size")?;
    }
    *memory_span = checked_add(section_offset, section_cursor, "ELF common image span")?;
    merged.push(ElfAmd64MergedSectionPlan {
        section_id,
        output_section_name: ".bss.nuis_common".to_owned(),
        class: "common".to_owned(),
        alignment: section_alignment,
        file_offset: None,
        image_offset: section_offset,
        virtual_address: virtual_address(section_offset)?,
        size_bytes: section_cursor,
        contribution_count: allocations.len(),
        zero_fill: true,
    });
    Ok(allocations)
}

fn build_symbol_bindings(
    objects: &[&ElfAmd64ObjectLinkage],
    definitions: &DefinitionCatalog<'_>,
    placements: &[ElfAmd64SectionPlacement],
    common: &[ElfAmd64CommonAllocation],
) -> Result<Vec<ElfAmd64SymbolBinding>, String> {
    let placement_by_input = placements
        .iter()
        .map(|placement| {
            (
                (placement.object_id.as_str(), placement.input_section_index),
                placement,
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut bindings = Vec::new();
    for object in objects {
        for symbol in object.linkage.symbols.iter().skip(1) {
            let (status, target) = if symbol.defined {
                definition_binding(object, symbol, definitions, &placement_by_input, common)?
            } else if symbol.external {
                match resolve_external_target(
                    &symbol.name,
                    definitions,
                    &placement_by_input,
                    common,
                )? {
                    Some(target) => ("internal", Some(target)),
                    None if symbol.weak => (
                        "weak-zero",
                        Some(ResolvedTarget {
                            object_id: object.object_id.clone(),
                            symbol_index: symbol.index,
                            kind: "weak-zero".to_owned(),
                            section_id: None,
                            image_offset: None,
                            virtual_address: Some(0),
                            absolute_value: Some(0),
                        }),
                    ),
                    None => ("external-compatibility", None),
                }
            } else {
                return Err(format!(
                    "ELF object `{}` has non-null undefined local symbol {}",
                    object.object_id, symbol.index
                ));
            };
            bindings.push(binding_report(object, symbol, status, target));
        }
    }
    Ok(bindings)
}

fn definition_binding(
    object: &ElfAmd64ObjectLinkage,
    symbol: &ParsedElfSymbol,
    definitions: &DefinitionCatalog<'_>,
    placements: &BTreeMap<(&str, usize), &ElfAmd64SectionPlacement>,
    common: &[ElfAmd64CommonAllocation],
) -> Result<(&'static str, Option<ResolvedTarget>), String> {
    if symbol.common && symbol.external && definitions.strong.contains_key(&symbol.name) {
        return Ok((
            "coalesced-to-definition",
            resolve_external_target(&symbol.name, definitions, placements, common)?,
        ));
    }
    if symbol.common {
        return Ok((
            "common-allocation",
            Some(resolve_common_target(object, symbol, common)?),
        ));
    }
    match direct_target(object, symbol, placements)? {
        Some(target) => Ok(("definition", Some(target))),
        None => Ok(("metadata-only", None)),
    }
}

fn resolve_external_target(
    name: &str,
    definitions: &DefinitionCatalog<'_>,
    placements: &BTreeMap<(&str, usize), &ElfAmd64SectionPlacement>,
    common: &[ElfAmd64CommonAllocation],
) -> Result<Option<ResolvedTarget>, String> {
    if let Some(target) = definitions.strong.get(name) {
        return direct_target(target.object, target.symbol, placements).and_then(|value| {
            value.map(Some).ok_or_else(|| {
                format!("ELF symbol `{name}` resolves to a non-alloc metadata section")
            })
        });
    }
    if let Some(items) = definitions.common.get(name) {
        return resolve_common_target(items[0].object, items[0].symbol, common).map(Some);
    }
    if let Some(target) = definitions.weak.get(name).and_then(|items| items.first()) {
        return direct_target(target.object, target.symbol, placements).and_then(|value| {
            value.map(Some).ok_or_else(|| {
                format!("ELF weak symbol `{name}` resolves to a non-alloc metadata section")
            })
        });
    }
    Ok(None)
}

fn direct_target(
    object: &ElfAmd64ObjectLinkage,
    symbol: &ParsedElfSymbol,
    placements: &BTreeMap<(&str, usize), &ElfAmd64SectionPlacement>,
) -> Result<Option<ResolvedTarget>, String> {
    if symbol.absolute {
        return Ok(Some(ResolvedTarget {
            object_id: object.object_id.clone(),
            symbol_index: symbol.index,
            kind: "absolute".to_owned(),
            section_id: None,
            image_offset: None,
            virtual_address: Some(symbol.value),
            absolute_value: Some(symbol.value),
        }));
    }
    let Some(section_index) = symbol.section_index else {
        return Ok(None);
    };
    let Some(placement) = placements.get(&(object.object_id.as_str(), section_index)) else {
        return Ok(None);
    };
    let relative = checked_usize(symbol.value, "ELF section-relative symbol value")?;
    let image_offset = checked_add(placement.image_offset, relative, "ELF symbol image offset")?;
    Ok(Some(ResolvedTarget {
        object_id: object.object_id.clone(),
        symbol_index: symbol.index,
        kind: "section".to_owned(),
        section_id: Some(placement.output_section_id.clone()),
        image_offset: Some(image_offset),
        virtual_address: Some(virtual_address(image_offset)?),
        absolute_value: None,
    }))
}

fn resolve_common_target(
    object: &ElfAmd64ObjectLinkage,
    symbol: &ParsedElfSymbol,
    common: &[ElfAmd64CommonAllocation],
) -> Result<ResolvedTarget, String> {
    let allocation = common
        .iter()
        .find(|allocation| {
            if symbol.external {
                allocation.external && allocation.symbol == symbol.name
            } else {
                !allocation.external
                    && allocation.owner_object_id == object.object_id
                    && allocation.owner_symbol_index == symbol.index
            }
        })
        .ok_or_else(|| format!("ELF common symbol `{}` has no allocation", symbol.name))?;
    Ok(ResolvedTarget {
        object_id: allocation.owner_object_id.clone(),
        symbol_index: allocation.owner_symbol_index,
        kind: "common".to_owned(),
        section_id: Some(allocation.output_section_id.clone()),
        image_offset: Some(allocation.image_offset),
        virtual_address: Some(allocation.virtual_address),
        absolute_value: None,
    })
}

fn binding_report(
    object: &ElfAmd64ObjectLinkage,
    symbol: &ParsedElfSymbol,
    status: &str,
    target: Option<ResolvedTarget>,
) -> ElfAmd64SymbolBinding {
    ElfAmd64SymbolBinding {
        symbol: symbol.name.clone(),
        reference_object_id: object.object_id.clone(),
        reference_symbol_index: symbol.index,
        status: status.to_owned(),
        target_object_id: target.as_ref().map(|target| target.object_id.clone()),
        target_symbol_index: target.as_ref().map(|target| target.symbol_index),
        target_kind: target.as_ref().map(|target| target.kind.clone()),
        target_section_id: target.as_ref().and_then(|target| target.section_id.clone()),
        target_image_offset: target.as_ref().and_then(|target| target.image_offset),
        target_virtual_address: target.as_ref().and_then(|target| target.virtual_address),
        target_absolute_value: target.and_then(|target| target.absolute_value),
    }
}

fn common_alignment(item: &SymbolRef<'_>) -> Result<usize, String> {
    checked_alignment(item.symbol.value, &item.symbol.name)
}

fn common_size(item: &SymbolRef<'_>) -> Result<usize, String> {
    let size = checked_usize(item.symbol.size, "ELF common size")?;
    if size == 0 {
        return Err(format!(
            "ELF common symbol `{}` has zero size",
            item.symbol.name
        ));
    }
    Ok(size)
}

fn object_role_rank(role: &str) -> usize {
    match role {
        "program-llvm" => 0,
        "runtime-shim" => 1,
        _ => 2,
    }
}

fn checked_alignment(value: u64, label: &str) -> Result<usize, String> {
    let alignment = if value == 0 {
        1
    } else {
        checked_usize(value, "ELF alignment")?
    };
    if !alignment.is_power_of_two() || alignment > ELF_AMD64_MAX_INPUT_ALIGNMENT {
        return Err(format!(
            "ELF section or common `{label}` alignment {alignment} is outside the supported power-of-two range 1..={ELF_AMD64_MAX_INPUT_ALIGNMENT}"
        ));
    }
    Ok(alignment)
}

fn align_up(value: usize, alignment: usize) -> Result<usize, String> {
    value
        .checked_add(alignment - 1)
        .map(|value| value & !(alignment - 1))
        .ok_or_else(|| "ELF placement alignment overflows".to_owned())
}

fn checked_add(lhs: usize, rhs: usize, label: &str) -> Result<usize, String> {
    lhs.checked_add(rhs)
        .ok_or_else(|| format!("{label} overflows"))
}

fn checked_usize(value: u64, label: &str) -> Result<usize, String> {
    usize::try_from(value).map_err(|_| format!("{label} {value} exceeds host address space"))
}

fn virtual_address(image_offset: usize) -> Result<u64, String> {
    ELF_AMD64_IMAGE_BASE
        .checked_add(
            u64::try_from(image_offset)
                .map_err(|_| "ELF image offset exceeds 64-bit address space".to_owned())?,
        )
        .ok_or_else(|| "ELF virtual address overflows".to_owned())
}

#[cfg(test)]
#[path = "final_executable_elf_layout_tests.rs"]
mod tests;
