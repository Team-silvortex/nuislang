use crate::{
    final_executable_macho_layout::MachOLayoutObject,
    final_executable_macho_shell_definitions::{
        collect_shell_definitions, collect_unresolved_shell_symbols, MachOShellDefinition,
        MachOShellDefinitionValue,
    },
    final_executable_macho_shell_layout::{
        locate_source_address, ShellLayoutDraft, SYSTEM_DYLIB_PATH,
    },
    reports::{
        NsldMachOArm64PlatformPatchApplicationReport, NsldMachOArm64PlatformStructurePlanReport,
        NsldMachOArm64ShellBindPlan, NsldMachOArm64ShellIndirectSymbolPlan,
        NsldMachOArm64ShellRebasePlan, NsldMachOArm64ShellSymbolPlan,
        NsldMachOPlacementBindingReport,
    },
};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

const NLIST_64_SIZE: usize = 16;
const INDIRECT_SYMBOL_SIZE: usize = 4;
const SYSTEM_DYLIB_ORDINAL: usize = 1;

#[derive(Clone, Copy)]
struct EntryRule {
    id: &'static str,
    object_role: &'static str,
    symbol: &'static str,
}

const ENTRY_RULES: &[EntryRule] = &[
    EntryRule {
        id: "arm64.macho.runtime-main.v1",
        object_role: "runtime-shim",
        symbol: "_main",
    },
    EntryRule {
        id: "arm64.macho.program-entry.v1",
        object_role: "program-llvm",
        symbol: "_nuis_entry",
    },
    EntryRule {
        id: "arm64.macho.program-yir-entry.v1",
        object_role: "program-llvm",
        symbol: "_nuis_yir_entry",
    },
];

pub(crate) struct ShellEntryPlan {
    pub(crate) rule_id: String,
    pub(crate) symbol: String,
    pub(crate) source_image_offset: usize,
    pub(crate) file_offset: usize,
    pub(crate) vm_address: u64,
}

pub(crate) struct ShellLinkeditPlan {
    pub(crate) entry: ShellEntryPlan,
    pub(crate) symbols: Vec<NsldMachOArm64ShellSymbolPlan>,
    pub(crate) indirect_symbols: Vec<NsldMachOArm64ShellIndirectSymbolPlan>,
    pub(crate) binds: Vec<NsldMachOArm64ShellBindPlan>,
    pub(crate) rebases: Vec<NsldMachOArm64ShellRebasePlan>,
    pub(crate) defined_symbol_count: usize,
    pub(crate) undefined_symbol_count: usize,
    pub(crate) rebase_stream_offset: usize,
    pub(crate) rebase_stream_bytes: usize,
    pub(crate) bind_stream_offset: usize,
    pub(crate) bind_stream_bytes: usize,
    pub(crate) symbol_table_offset: usize,
    pub(crate) symbol_table_bytes: usize,
    pub(crate) indirect_symbol_table_offset: usize,
    pub(crate) indirect_symbol_table_bytes: usize,
    pub(crate) string_table_offset: usize,
    pub(crate) string_table_bytes: usize,
    pub(crate) linkedit_bytes: usize,
}

pub(crate) fn build_shell_linkedit_plan(
    objects: &[MachOLayoutObject<'_>],
    placement: &NsldMachOPlacementBindingReport,
    platform: &NsldMachOArm64PlatformStructurePlanReport,
    applied: &NsldMachOArm64PlatformPatchApplicationReport,
    layout: &ShellLayoutDraft,
) -> Result<ShellLinkeditPlan, String> {
    let definitions = collect_shell_definitions(objects, placement)?;
    let undefined = collect_unresolved_shell_symbols(objects, &definitions);
    validate_external_library_boundary(&undefined, applied)?;
    let entry = select_entry(&definitions, layout)?;
    let (symbols, string_table_bytes) = build_symbols(&definitions, &undefined, layout)?;
    let symbol_indices = symbols
        .iter()
        .map(|symbol| (symbol.name.as_str(), symbol.symbol_table_index))
        .collect::<BTreeMap<_, _>>();
    let indirect_symbols = build_indirect_symbols(platform, layout, &symbol_indices)?;
    let binds = build_binds(applied, layout, &symbol_indices)?;
    let rebases = build_rebases(platform, layout)?;

    let rebase_stream_offset = layout.linkedit_file_offset;
    let rebase_stream_bytes = stream_size(&rebases, |rebase| rebase.encoded_size_bytes)?;
    let bind_stream_offset = align_up(
        rebase_stream_offset
            .checked_add(rebase_stream_bytes)
            .ok_or_else(|| "Mach-O rebase stream end overflows".to_owned())?,
        8,
    )?;
    let bind_stream_bytes = stream_size(&binds, |bind| bind.encoded_size_bytes)?;
    let symbol_table_offset = align_up(
        bind_stream_offset
            .checked_add(bind_stream_bytes)
            .ok_or_else(|| "Mach-O bind stream end overflows".to_owned())?,
        8,
    )?;
    let symbol_table_bytes = symbols
        .len()
        .checked_mul(NLIST_64_SIZE)
        .ok_or_else(|| "Mach-O symbol table size overflows".to_owned())?;
    let indirect_symbol_table_offset = align_up(
        symbol_table_offset
            .checked_add(symbol_table_bytes)
            .ok_or_else(|| "Mach-O symbol table end overflows".to_owned())?,
        4,
    )?;
    let indirect_symbol_table_bytes = indirect_symbols
        .len()
        .checked_mul(INDIRECT_SYMBOL_SIZE)
        .ok_or_else(|| "Mach-O indirect symbol table size overflows".to_owned())?;
    let string_table_offset = indirect_symbol_table_offset
        .checked_add(indirect_symbol_table_bytes)
        .ok_or_else(|| "Mach-O indirect symbol table end overflows".to_owned())?;
    let linkedit_end = string_table_offset
        .checked_add(string_table_bytes)
        .ok_or_else(|| "Mach-O string table end overflows".to_owned())?;
    let linkedit_bytes = linkedit_end
        .checked_sub(layout.linkedit_file_offset)
        .ok_or_else(|| "Mach-O linkedit span underflows".to_owned())?;
    Ok(ShellLinkeditPlan {
        entry,
        symbols,
        indirect_symbols,
        binds,
        rebases,
        defined_symbol_count: definitions.len(),
        undefined_symbol_count: undefined.len(),
        rebase_stream_offset,
        rebase_stream_bytes,
        bind_stream_offset,
        bind_stream_bytes,
        symbol_table_offset,
        symbol_table_bytes,
        indirect_symbol_table_offset,
        indirect_symbol_table_bytes,
        string_table_offset,
        string_table_bytes,
        linkedit_bytes,
    })
}

fn validate_external_library_boundary(
    undefined: &BTreeSet<String>,
    applied: &NsldMachOArm64PlatformPatchApplicationReport,
) -> Result<(), String> {
    let bound = applied
        .bind_records
        .iter()
        .map(|bind| bind.target_symbol.as_str())
        .collect::<BTreeSet<_>>();
    if !bound.iter().all(|symbol| undefined.contains(*symbol)) {
        return Err("Mach-O shell bind records contain a defined or missing symbol".to_owned());
    }
    if !applied.bind_records.is_empty() && SYSTEM_DYLIB_PATH.is_empty() {
        return Err("Mach-O shell has no registered compatibility dylib".to_owned());
    }
    Ok(())
}

fn select_entry(
    definitions: &BTreeMap<String, MachOShellDefinition<'_>>,
    layout: &ShellLayoutDraft,
) -> Result<ShellEntryPlan, String> {
    for rule in ENTRY_RULES {
        let Some(definition) = definitions.get(rule.symbol) else {
            continue;
        };
        if definition.object.role != rule.object_role {
            continue;
        }
        let source_image_offset = match definition.value {
            MachOShellDefinitionValue::ImageOffset(offset) => offset,
            MachOShellDefinitionValue::Absolute(_) => {
                return Err(format!(
                    "Mach-O entry `{}` resolves to an absolute symbol",
                    rule.symbol
                ));
            }
        };
        let address =
            locate_source_address(source_image_offset, &layout.sections, &layout.segments)?;
        let file_offset = address.file_offset.ok_or_else(|| {
            format!(
                "Mach-O entry `{}` resolves to a zero-fill section",
                rule.symbol
            )
        })?;
        let section = layout
            .sections
            .iter()
            .find(|section| section.section_id == address.section_id)
            .expect("located entry section must remain present");
        if section.segment_name != "__TEXT" {
            return Err(format!("Mach-O entry `{}` is not in __TEXT", rule.symbol));
        }
        return Ok(ShellEntryPlan {
            rule_id: rule.id.to_owned(),
            symbol: rule.symbol.to_owned(),
            source_image_offset,
            file_offset,
            vm_address: address.vm_address,
        });
    }
    Err(format!(
        "Mach-O shell entry registry found no supported definition; expected one of {}",
        ENTRY_RULES
            .iter()
            .map(|rule| format!("{}:{}", rule.object_role, rule.symbol))
            .collect::<Vec<_>>()
            .join(",")
    ))
}

fn build_symbols(
    definitions: &BTreeMap<String, MachOShellDefinition<'_>>,
    undefined: &BTreeSet<String>,
    layout: &ShellLayoutDraft,
) -> Result<(Vec<NsldMachOArm64ShellSymbolPlan>, usize), String> {
    let mut string_offset = 1usize;
    let mut symbols = Vec::with_capacity(definitions.len() + undefined.len());
    for (name, definition) in definitions {
        let (record_kind, section_id, source_offset, vm_address) = match definition.value {
            MachOShellDefinitionValue::ImageOffset(offset) => {
                let address = locate_source_address(offset, &layout.sections, &layout.segments)?;
                (
                    if definition.alias {
                        "external-defined-alias"
                    } else {
                        "external-defined"
                    },
                    Some(address.section_id),
                    Some(offset),
                    Some(address.vm_address),
                )
            }
            MachOShellDefinitionValue::Absolute(value) => (
                if definition.alias {
                    "external-absolute-alias"
                } else {
                    "external-absolute"
                },
                None,
                None,
                Some(value),
            ),
        };
        let symbol_id = format!("macho-arm64-shell-symbol-{:04}", symbols.len());
        let audit_hash = symbol_audit_hash(
            &symbol_id,
            name,
            record_kind,
            Some(definition.object.object_id),
            Some(definition.symbol.index),
            section_id.as_deref(),
            source_offset,
            vm_address,
            symbols.len(),
            string_offset,
            None,
        );
        symbols.push(NsldMachOArm64ShellSymbolPlan {
            symbol_id,
            name: name.clone(),
            record_kind: record_kind.to_owned(),
            object_id: Some(definition.object.object_id.to_owned()),
            source_symbol_index: Some(definition.symbol.index),
            shell_section_id: section_id,
            source_image_offset: source_offset,
            vm_address,
            symbol_table_index: symbols.len(),
            string_table_offset: string_offset,
            dylib_ordinal: None,
            audit_hash,
        });
        string_offset = next_string_offset(string_offset, name)?;
    }
    for name in undefined {
        let symbol_id = format!("macho-arm64-shell-symbol-{:04}", symbols.len());
        let audit_hash = symbol_audit_hash(
            &symbol_id,
            name,
            "external-undefined",
            None,
            None,
            None,
            None,
            None,
            symbols.len(),
            string_offset,
            Some(SYSTEM_DYLIB_ORDINAL),
        );
        symbols.push(NsldMachOArm64ShellSymbolPlan {
            symbol_id,
            name: name.clone(),
            record_kind: "external-undefined".to_owned(),
            object_id: None,
            source_symbol_index: None,
            shell_section_id: None,
            source_image_offset: None,
            vm_address: None,
            symbol_table_index: symbols.len(),
            string_table_offset: string_offset,
            dylib_ordinal: Some(SYSTEM_DYLIB_ORDINAL),
            audit_hash,
        });
        string_offset = next_string_offset(string_offset, name)?;
    }
    Ok((symbols, string_offset))
}

fn build_indirect_symbols(
    platform: &NsldMachOArm64PlatformStructurePlanReport,
    layout: &ShellLayoutDraft,
    symbol_indices: &BTreeMap<&str, usize>,
) -> Result<Vec<NsldMachOArm64ShellIndirectSymbolPlan>, String> {
    let stub_section = (platform.stub_entry_count > 0)
        .then(|| source_section_id(layout, "platform-stubs"))
        .transpose()?;
    let got_section = (platform.got_entry_count > 0)
        .then(|| source_section_id(layout, "platform-got"))
        .transpose()?;
    let mut records = Vec::with_capacity(platform.stub_entry_count + platform.got_entry_count);
    for target in platform
        .targets
        .iter()
        .filter(|target| target.stub_slot_index.is_some())
    {
        records.push(indirect_record(
            records.len(),
            stub_section.ok_or_else(|| "Mach-O shell stub section is absent".to_owned())?,
            target.stub_slot_index.unwrap(),
            target,
            symbol_indices,
        )?);
    }
    for target in platform
        .targets
        .iter()
        .filter(|target| target.got_slot_index.is_some())
    {
        records.push(indirect_record(
            records.len(),
            got_section.ok_or_else(|| "Mach-O shell GOT section is absent".to_owned())?,
            target.got_slot_index.unwrap(),
            target,
            symbol_indices,
        )?);
    }
    Ok(records)
}

fn indirect_record(
    index: usize,
    section_id: &str,
    slot_index: usize,
    target: &crate::reports::NsldMachOArm64PlatformTargetPlan,
    symbol_indices: &BTreeMap<&str, usize>,
) -> Result<NsldMachOArm64ShellIndirectSymbolPlan, String> {
    let (symbol_table_index, marker) = if target.resolver_status == "external-compatibility" {
        (
            Some(
                *symbol_indices
                    .get(target.target_symbol.as_str())
                    .ok_or_else(|| {
                        format!(
                            "Mach-O indirect target `{}` has no undefined symbol record",
                            target.target_symbol
                        )
                    })?,
            ),
            None,
        )
    } else {
        (None, Some("local-absolute".to_owned()))
    };
    let indirect_id = format!("macho-arm64-shell-indirect-{index:04}");
    let audit_hash = indirect_audit_hash(
        &indirect_id,
        section_id,
        slot_index,
        &target.target_symbol,
        symbol_table_index,
        marker.as_deref(),
    );
    Ok(NsldMachOArm64ShellIndirectSymbolPlan {
        indirect_id,
        shell_section_id: section_id.to_owned(),
        slot_index,
        target_symbol: target.target_symbol.clone(),
        symbol_table_index,
        marker,
        audit_hash,
    })
}

fn build_binds(
    applied: &NsldMachOArm64PlatformPatchApplicationReport,
    layout: &ShellLayoutDraft,
    symbol_indices: &BTreeMap<&str, usize>,
) -> Result<Vec<NsldMachOArm64ShellBindPlan>, String> {
    let mut binds = Vec::with_capacity(applied.bind_records.len());
    for source in &applied.bind_records {
        if !symbol_indices.contains_key(source.target_symbol.as_str()) {
            return Err(format!(
                "Mach-O bind target `{}` has no symbol record",
                source.target_symbol
            ));
        }
        let address =
            locate_source_address(source.got_output_offset, &layout.sections, &layout.segments)?;
        let file_offset = address
            .file_offset
            .ok_or_else(|| "Mach-O bind target is not file-backed".to_owned())?;
        if address.segment_index > 15 {
            return Err("Mach-O bind segment index exceeds legacy dyld opcode space".to_owned());
        }
        let encoded_size_bytes = 6usize
            .checked_add(source.target_symbol.len())
            .and_then(|size| size.checked_add(uleb_size(address.segment_offset)))
            .ok_or_else(|| "Mach-O bind opcode size overflows".to_owned())?;
        let bind_id = format!("macho-arm64-shell-bind-{:04}", binds.len());
        let audit_hash = bind_audit_hash(
            &bind_id,
            &source.bind_id,
            &source.target_symbol,
            &address,
            file_offset,
            encoded_size_bytes,
        );
        binds.push(NsldMachOArm64ShellBindPlan {
            bind_id,
            source_bind_id: source.bind_id.clone(),
            target_symbol: source.target_symbol.clone(),
            dylib_ordinal: SYSTEM_DYLIB_ORDINAL,
            got_source_image_offset: source.got_output_offset,
            shell_section_id: address.section_id,
            segment_index: address.segment_index,
            segment_offset: address.segment_offset,
            file_offset,
            vm_address: address.vm_address,
            encoded_size_bytes,
            audit_hash,
        });
    }
    Ok(binds)
}

fn build_rebases(
    platform: &NsldMachOArm64PlatformStructurePlanReport,
    layout: &ShellLayoutDraft,
) -> Result<Vec<NsldMachOArm64ShellRebasePlan>, String> {
    let mut rebases = Vec::new();
    for target in platform.targets.iter().filter(|target| {
        target.got_output_offset.is_some()
            && target.target_output_offset.is_some()
            && target.resolver_status != "external-compatibility"
    }) {
        let got_source_image_offset = target.got_output_offset.unwrap();
        let target_source_image_offset = target.target_output_offset.ok_or_else(|| {
            format!(
                "Mach-O internal GOT target `{}` has no source image offset",
                target.target_symbol
            )
        })?;
        let location =
            locate_source_address(got_source_image_offset, &layout.sections, &layout.segments)?;
        let target_address = locate_source_address(
            target_source_image_offset,
            &layout.sections,
            &layout.segments,
        )?;
        let file_offset = location
            .file_offset
            .ok_or_else(|| "Mach-O rebase location is not file-backed".to_owned())?;
        if location.segment_index > 15 {
            return Err("Mach-O rebase segment index exceeds legacy dyld opcode space".to_owned());
        }
        let encoded_size_bytes = 3usize
            .checked_add(uleb_size(location.segment_offset))
            .ok_or_else(|| "Mach-O rebase opcode size overflows".to_owned())?;
        let rebase_id = format!("macho-arm64-shell-rebase-{:04}", rebases.len());
        let audit_hash = rebase_audit_hash(
            &rebase_id,
            &target.structure_id,
            &target.target_symbol,
            &location,
            target_address.vm_address,
            file_offset,
            encoded_size_bytes,
        );
        rebases.push(NsldMachOArm64ShellRebasePlan {
            rebase_id,
            structure_id: target.structure_id.clone(),
            target_symbol: target.target_symbol.clone(),
            got_source_image_offset,
            target_source_image_offset,
            shell_section_id: location.section_id,
            segment_index: location.segment_index,
            segment_offset: location.segment_offset,
            file_offset,
            vm_address: location.vm_address,
            target_vm_address: target_address.vm_address,
            encoded_size_bytes,
            audit_hash,
        });
    }
    Ok(rebases)
}

fn source_section_id<'a>(layout: &'a ShellLayoutDraft, kind: &str) -> Result<&'a str, String> {
    let matches = layout
        .sections
        .iter()
        .filter(|section| section.source_kind == kind)
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [section] => Ok(&section.section_id),
        [] => Err(format!("Mach-O shell is missing `{kind}` section")),
        _ => Err(format!("Mach-O shell repeats `{kind}` section")),
    }
}

fn next_string_offset(offset: usize, name: &str) -> Result<usize, String> {
    let entry_bytes = name
        .len()
        .checked_add(1)
        .ok_or_else(|| "Mach-O string entry size overflows".to_owned())?;
    offset
        .checked_add(entry_bytes)
        .ok_or_else(|| "Mach-O string table size overflows".to_owned())
}

fn stream_size<T>(records: &[T], size: impl Fn(&T) -> usize) -> Result<usize, String> {
    if records.is_empty() {
        Ok(0)
    } else {
        records
            .iter()
            .try_fold(0usize, |total, record| {
                total
                    .checked_add(size(record))
                    .ok_or_else(|| "Mach-O dyld stream size overflows".to_owned())
            })?
            .checked_add(1)
            .ok_or_else(|| "Mach-O dyld stream terminator overflows".to_owned())
    }
}

fn uleb_size(mut value: usize) -> usize {
    let mut size = 1;
    while value >= 0x80 {
        value >>= 7;
        size += 1;
    }
    size
}

fn align_up(value: usize, alignment: usize) -> Result<usize, String> {
    if alignment == 0 || !alignment.is_power_of_two() {
        return Err("Mach-O linkedit alignment must be a nonzero power of two".to_owned());
    }
    let mask = alignment - 1;
    value
        .checked_add(mask)
        .map(|value| value & !mask)
        .ok_or_else(|| "Mach-O linkedit alignment overflows".to_owned())
}

#[allow(clippy::too_many_arguments)]
fn symbol_audit_hash(
    symbol_id: &str,
    name: &str,
    kind: &str,
    object_id: Option<&str>,
    source_symbol_index: Option<usize>,
    section_id: Option<&str>,
    source_offset: Option<usize>,
    vm_address: Option<u64>,
    table_index: usize,
    string_offset: usize,
    dylib_ordinal: Option<usize>,
) -> String {
    let mut out = String::new();
    for value in [
        symbol_id,
        name,
        kind,
        object_id.unwrap_or("none"),
        section_id.unwrap_or("none"),
    ] {
        append_text(&mut out, value);
    }
    writeln!(
        out,
        "facts={}|{}|{}|{}|{}|{}",
        optional_usize(source_symbol_index),
        optional_usize(source_offset),
        vm_address.map_or("none".to_owned(), |value| value.to_string()),
        table_index,
        string_offset,
        optional_usize(dylib_ordinal)
    )
    .unwrap();
    crate::fnv1a64_hex(out.as_bytes())
}

fn indirect_audit_hash(
    id: &str,
    section_id: &str,
    slot: usize,
    symbol: &str,
    symbol_index: Option<usize>,
    marker: Option<&str>,
) -> String {
    let mut out = String::new();
    append_text(&mut out, id);
    append_text(&mut out, section_id);
    append_text(&mut out, symbol);
    append_text(&mut out, marker.unwrap_or("none"));
    writeln!(out, "facts={slot}|{}", optional_usize(symbol_index)).unwrap();
    crate::fnv1a64_hex(out.as_bytes())
}

fn bind_audit_hash(
    id: &str,
    source_id: &str,
    symbol: &str,
    address: &crate::final_executable_macho_shell_layout::LocatedShellAddress,
    file_offset: usize,
    encoded_size: usize,
) -> String {
    let mut out = String::new();
    append_text(&mut out, id);
    append_text(&mut out, source_id);
    append_text(&mut out, symbol);
    append_text(&mut out, &address.section_id);
    writeln!(
        out,
        "facts={SYSTEM_DYLIB_ORDINAL}|{}|{}|{}|{}|{encoded_size}",
        address.segment_index, address.segment_offset, file_offset, address.vm_address
    )
    .unwrap();
    crate::fnv1a64_hex(out.as_bytes())
}

#[allow(clippy::too_many_arguments)]
fn rebase_audit_hash(
    id: &str,
    structure_id: &str,
    symbol: &str,
    location: &crate::final_executable_macho_shell_layout::LocatedShellAddress,
    target_vm_address: u64,
    file_offset: usize,
    encoded_size: usize,
) -> String {
    let mut out = String::new();
    append_text(&mut out, id);
    append_text(&mut out, structure_id);
    append_text(&mut out, symbol);
    append_text(&mut out, &location.section_id);
    writeln!(
        out,
        "facts={}|{}|{}|{}|{}|{encoded_size}",
        location.segment_index,
        location.segment_offset,
        file_offset,
        location.vm_address,
        target_vm_address
    )
    .unwrap();
    crate::fnv1a64_hex(out.as_bytes())
}

fn optional_usize(value: Option<usize>) -> String {
    value.map_or("none".to_owned(), |value| value.to_string())
}

fn append_text(out: &mut String, value: &str) {
    writeln!(out, "text:{}:{value}", value.len()).unwrap();
}
