#[path = "final_executable_elf_platform_report.rs"]
mod report;
#[path = "final_executable_elf_platform_validation.rs"]
mod validation;

pub(crate) use report::{
    ElfAmd64PlatformRelocationBinding, ElfAmd64PlatformStructurePlanReport,
    ElfAmd64PlatformTargetPlan,
};

use crate::{
    final_executable_elf_layout::ELF_AMD64_PAGE_SIZE,
    final_executable_elf_layout_report::ElfAmd64PlacementBindingReport,
    final_executable_elf_materialization::application::ElfAmd64PatchApplicationReport,
    final_executable_elf_relocation_report::{
        ElfAmd64RelocationApplication, ElfAmd64RelocationApplicationReport,
    },
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
};

pub(crate) const ELF_AMD64_PLATFORM_STRUCTURE_PLAN_CONTRACT: &str =
    "nuis-nsld-elf-amd64-platform-structure-plan-v1";

const PLT_ENTRY_SIZE: usize = 16;
const PLT_ALIGNMENT: usize = 16;
const GOT_ENTRY_SIZE: usize = 8;
const GOT_ALIGNMENT: usize = 8;
const ELF64_SYMBOL_ENTRY_SIZE: usize = 24;
const ELF64_RELA_ENTRY_SIZE: usize = 24;
const ELF64_RELA_ALIGNMENT: usize = 8;

#[derive(Clone, Copy)]
struct PlatformRule {
    id: &'static str,
    target_class: &'static str,
    relocation_kind: &'static str,
    action_kind: &'static str,
    resolver_status: &'static str,
    width_bytes: usize,
    pc_relative: bool,
    requires_plt: bool,
    requires_got: bool,
    dynamic_relocation_kind: Option<&'static str>,
    dynamic_relocation_type: Option<u32>,
    patch_target_kind: &'static str,
}

const PLATFORM_RULES: &[PlatformRule] = &[PlatformRule {
    id: "x86_64.external-plt32.nonlazy.v1",
    target_class: "external-function-nonlazy",
    relocation_kind: "x86_64-plt32",
    action_kind: "write-plt-relative-32",
    resolver_status: "external-compatibility",
    width_bytes: 4,
    pc_relative: true,
    requires_plt: true,
    requires_got: true,
    dynamic_relocation_kind: Some("r-x86-64-jump-slot"),
    dynamic_relocation_type: Some(7),
    patch_target_kind: "plt-entry",
}];

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct TargetIdentity {
    target_class: String,
    symbol: String,
    resolver_status: String,
}

impl TargetIdentity {
    fn canonical(&self) -> String {
        let mut out = String::new();
        append_text(&mut out, &self.target_class);
        append_text(&mut out, &self.symbol);
        append_text(&mut out, &self.resolver_status);
        out
    }
}

struct TargetSeed {
    target_key: String,
    target_symbol: String,
    resolver_status: String,
    rule_ids: BTreeSet<String>,
    requires_plt: bool,
    requires_got: bool,
    dynamic_relocation_kind: Option<String>,
    dynamic_relocation_type: Option<u32>,
    relocation_ids: Vec<String>,
}

struct DynamicSymbolSlot {
    index: usize,
    string_offset: usize,
}

struct PlannedTarget {
    canonical_identity: String,
    plan: ElfAmd64PlatformTargetPlan,
}

pub(crate) fn build_elf_amd64_platform_structure_plan(
    placement: &ElfAmd64PlacementBindingReport,
    relocations: &ElfAmd64RelocationApplicationReport,
    applied: &ElfAmd64PatchApplicationReport,
) -> Result<ElfAmd64PlatformStructurePlanReport, String> {
    validation::validate_input_envelope(placement, relocations, applied)?;
    let registry_hash = validate_registry()?;
    let deferred = relocations
        .applications
        .iter()
        .filter(|application| application.application_status == "planned-platform-structure")
        .collect::<Vec<_>>();
    if deferred.len() != relocations.platform_structure_count {
        return Err("ELF platform plan deferred relocation coverage drift".to_owned());
    }

    let mut seeds = BTreeMap::<TargetIdentity, TargetSeed>::new();
    let mut rules = BTreeMap::<&str, PlatformRule>::new();
    let mut source_spans = Vec::new();
    for application in &deferred {
        validation::validate_deferred_source(application, placement)?;
        validate_nonoverlapping_source(application, &mut source_spans)?;
        let rule = registered_rule(application)?;
        if rules
            .insert(application.relocation_id.as_str(), rule)
            .is_some()
        {
            return Err(format!(
                "ELF platform plan repeats relocation `{}`",
                application.relocation_id
            ));
        }
        let (identity, seed) = target_seed(application, rule)?;
        match seeds.get_mut(&identity) {
            Some(existing) => merge_seed(existing, seed, application)?,
            None => {
                seeds.insert(identity, seed);
            }
        }
    }

    let (dynamic_symbols, dynamic_string_bytes) = dynamic_symbol_slots(&seeds)?;
    let plt_count = seeds.values().filter(|seed| seed.requires_plt).count();
    let got_count = seeds.values().filter(|seed| seed.requires_got).count();
    let dynamic_relocation_count = seeds
        .values()
        .filter(|seed| seed.dynamic_relocation_type.is_some())
        .count();
    let dynamic_symbol_count = if dynamic_symbols.is_empty() {
        0
    } else {
        dynamic_symbols.len() + 1
    };
    let layout = platform_layout(
        applied.file_span_bytes,
        applied.memory_span_bytes,
        plt_count,
        got_count,
        dynamic_symbol_count,
        dynamic_string_bytes,
        dynamic_relocation_count,
    )?;
    let planned_targets =
        assign_target_slots(seeds, &dynamic_symbols, &layout, placement.image_base)?;
    let target_lookup = planned_targets
        .iter()
        .map(|target| (target.canonical_identity.as_str(), &target.plan))
        .collect::<BTreeMap<_, _>>();
    let relocation_bindings = deferred
        .iter()
        .map(|application| {
            let rule = rules
                .get(application.relocation_id.as_str())
                .copied()
                .expect("registered deferred ELF rule must remain available");
            build_relocation_binding(application, rule, &target_lookup)
        })
        .collect::<Result<Vec<_>, String>>()?;
    let targets = planned_targets
        .into_iter()
        .map(|target| target.plan)
        .collect::<Vec<_>>();
    let status = if deferred.is_empty() {
        "not-required"
    } else {
        "allocated-ready-for-platform-patching"
    };
    let mut report = ElfAmd64PlatformStructurePlanReport {
        contract: ELF_AMD64_PLATFORM_STRUCTURE_PLAN_CONTRACT,
        status: status.to_owned(),
        plan_hash: String::new(),
        registry_hash,
        placement_plan_hash: placement.plan_hash.clone(),
        relocation_plan_hash: relocations.plan_hash.clone(),
        patch_application_ledger_hash: applied.application_ledger_hash.clone(),
        applied_memory_image_hash: applied.applied_memory_image_hash.clone(),
        image_base: placement.image_base,
        base_file_span_bytes: applied.file_span_bytes,
        base_memory_span_bytes: applied.memory_span_bytes,
        planned_file_span_bytes: layout.planned_file_span,
        planned_memory_span_bytes: layout.planned_memory_span,
        registered_rule_count: PLATFORM_RULES.len(),
        deferred_relocation_count: deferred.len(),
        target_count: targets.len(),
        plt_region_image_offset: layout.plt_offset,
        plt_region_bytes: layout.plt_bytes,
        plt_entry_size: PLT_ENTRY_SIZE,
        plt_alignment: PLT_ALIGNMENT,
        plt_entry_count: plt_count,
        got_region_image_offset: layout.got_offset,
        got_region_bytes: layout.got_bytes,
        got_entry_size: GOT_ENTRY_SIZE,
        got_alignment: GOT_ALIGNMENT,
        got_entry_count: got_count,
        metadata_region_image_offset: layout.metadata_offset,
        metadata_region_bytes: layout.metadata_bytes,
        dynamic_symbol_region_image_offset: layout.dynsym_offset,
        dynamic_symbol_region_bytes: layout.dynsym_bytes,
        dynamic_symbol_entry_size: ELF64_SYMBOL_ENTRY_SIZE,
        dynamic_symbol_entry_count: dynamic_symbol_count,
        dynamic_string_region_image_offset: layout.dynstr_offset,
        dynamic_string_region_bytes: dynamic_string_bytes,
        dynamic_relocation_region_image_offset: layout.rela_offset,
        dynamic_relocation_region_bytes: layout.rela_bytes,
        dynamic_relocation_entry_size: ELF64_RELA_ENTRY_SIZE,
        dynamic_relocation_alignment: ELF64_RELA_ALIGNMENT,
        dynamic_relocation_entry_count: dynamic_relocation_count,
        targets,
        relocation_bindings,
    };
    report.plan_hash = crate::fnv1a64_hex(report.canonical_plan().as_bytes());
    Ok(report)
}

fn validate_registry() -> Result<String, String> {
    let mut identities = BTreeSet::new();
    let mut canonical = String::new();
    for rule in PLATFORM_RULES {
        let identity = (rule.relocation_kind, rule.action_kind, rule.resolver_status);
        let dynamic_pair =
            rule.dynamic_relocation_kind.is_some() == rule.dynamic_relocation_type.is_some();
        let dynamic_requires_got = rule.dynamic_relocation_type.is_none() || rule.requires_got;
        let plt_requires_got = !rule.requires_plt || rule.requires_got;
        let patch_target_valid = match rule.patch_target_kind {
            "plt-entry" => rule.requires_plt,
            "got-entry" => rule.requires_got,
            _ => false,
        };
        if !identities.insert(identity)
            || rule.id.is_empty()
            || rule.target_class.is_empty()
            || rule.width_bytes == 0
            || !dynamic_pair
            || !dynamic_requires_got
            || !plt_requires_got
            || !patch_target_valid
        {
            return Err(format!("invalid ELF platform registry rule `{}`", rule.id));
        }
        append_rule(&mut canonical, *rule);
    }
    Ok(crate::fnv1a64_hex(canonical.as_bytes()))
}

fn registered_rule(application: &ElfAmd64RelocationApplication) -> Result<PlatformRule, String> {
    let matches = PLATFORM_RULES
        .iter()
        .filter(|rule| {
            rule.relocation_kind == application.relocation_kind
                && rule.action_kind == application.action_kind
                && rule.resolver_status == application.resolver_status
        })
        .copied()
        .collect::<Vec<_>>();
    let rule = match matches.as_slice() {
        [rule] => *rule,
        [] => {
            return Err(format!(
                "ELF deferred relocation `{}` has no registered platform structure rule for kind `{}`, action `{}`, resolver `{}`",
                application.relocation_id,
                application.relocation_kind,
                application.action_kind,
                application.resolver_status
            ))
        }
        _ => {
            return Err(format!(
                "ELF deferred relocation `{}` matches multiple platform structure rules",
                application.relocation_id
            ))
        }
    };
    if application.width_bytes != rule.width_bytes || application.pc_relative != rule.pc_relative {
        return Err(format!(
            "ELF deferred relocation `{}` shape disagrees with rule `{}`",
            application.relocation_id, rule.id
        ));
    }
    Ok(rule)
}

fn target_seed(
    application: &ElfAmd64RelocationApplication,
    rule: PlatformRule,
) -> Result<(TargetIdentity, TargetSeed), String> {
    let symbol = application
        .target_symbol
        .as_deref()
        .filter(|symbol| !symbol.is_empty())
        .ok_or_else(|| {
            format!(
                "ELF deferred relocation `{}` has no target symbol",
                application.relocation_id
            )
        })?;
    let identity = TargetIdentity {
        target_class: rule.target_class.to_owned(),
        symbol: symbol.to_owned(),
        resolver_status: application.resolver_status.clone(),
    };
    let target_key = crate::fnv1a64_hex(identity.canonical().as_bytes());
    Ok((
        identity,
        TargetSeed {
            target_key,
            target_symbol: symbol.to_owned(),
            resolver_status: application.resolver_status.clone(),
            rule_ids: BTreeSet::from([rule.id.to_owned()]),
            requires_plt: rule.requires_plt,
            requires_got: rule.requires_got,
            dynamic_relocation_kind: rule.dynamic_relocation_kind.map(str::to_owned),
            dynamic_relocation_type: rule.dynamic_relocation_type,
            relocation_ids: vec![application.relocation_id.clone()],
        },
    ))
}

fn merge_seed(
    existing: &mut TargetSeed,
    incoming: TargetSeed,
    application: &ElfAmd64RelocationApplication,
) -> Result<(), String> {
    if existing.dynamic_relocation_kind != incoming.dynamic_relocation_kind
        || existing.dynamic_relocation_type != incoming.dynamic_relocation_type
    {
        return Err(format!(
            "ELF platform target for `{}` has conflicting dynamic relocation rules",
            application.relocation_id
        ));
    }
    existing.rule_ids.extend(incoming.rule_ids);
    existing.requires_plt |= incoming.requires_plt;
    existing.requires_got |= incoming.requires_got;
    existing.relocation_ids.extend(incoming.relocation_ids);
    Ok(())
}

fn validate_nonoverlapping_source(
    application: &ElfAmd64RelocationApplication,
    spans: &mut Vec<(usize, usize, String)>,
) -> Result<(), String> {
    let end = application
        .source_image_offset
        .checked_add(application.width_bytes)
        .ok_or_else(|| "ELF deferred source image span overflows".to_owned())?;
    if let Some((_, _, previous)) = spans.iter().find(|(start, previous_end, _)| {
        application.source_image_offset < *previous_end && *start < end
    }) {
        return Err(format!(
            "ELF deferred relocation `{}` overlaps `{previous}`",
            application.relocation_id
        ));
    }
    spans.push((
        application.source_image_offset,
        end,
        application.relocation_id.clone(),
    ));
    Ok(())
}

fn dynamic_symbol_slots(
    seeds: &BTreeMap<TargetIdentity, TargetSeed>,
) -> Result<(BTreeMap<String, DynamicSymbolSlot>, usize), String> {
    let symbols = seeds
        .values()
        .map(|seed| seed.target_symbol.as_str())
        .collect::<BTreeSet<_>>();
    let mut slots = BTreeMap::new();
    let mut string_offset = usize::from(!symbols.is_empty());
    for (ordinal, symbol) in symbols.into_iter().enumerate() {
        u32::try_from(string_offset)
            .map_err(|_| "ELF dynamic string table exceeds 32-bit offsets".to_owned())?;
        slots.insert(
            symbol.to_owned(),
            DynamicSymbolSlot {
                index: ordinal + 1,
                string_offset,
            },
        );
        let stored_bytes = symbol
            .len()
            .checked_add(1)
            .ok_or_else(|| "ELF dynamic symbol name size overflows".to_owned())?;
        string_offset = string_offset
            .checked_add(stored_bytes)
            .ok_or_else(|| "ELF dynamic string table size overflows".to_owned())?;
    }
    Ok((slots, string_offset))
}

struct PlatformLayout {
    planned_file_span: usize,
    planned_memory_span: usize,
    plt_offset: usize,
    plt_bytes: usize,
    got_offset: usize,
    got_bytes: usize,
    metadata_offset: usize,
    metadata_bytes: usize,
    dynsym_offset: usize,
    dynsym_bytes: usize,
    dynstr_offset: usize,
    rela_offset: usize,
    rela_bytes: usize,
}

#[allow(clippy::too_many_arguments)]
fn platform_layout(
    base_file_span: usize,
    base_memory_span: usize,
    plt_count: usize,
    got_count: usize,
    dynsym_count: usize,
    dynstr_bytes: usize,
    rela_count: usize,
) -> Result<PlatformLayout, String> {
    let plt_offset = region_start(base_memory_span, plt_count, ELF_AMD64_PAGE_SIZE)?;
    let plt_bytes = checked_product(plt_count, PLT_ENTRY_SIZE, "PLT")?;
    let plt_end = checked_sum(plt_offset, plt_bytes, "PLT")?;
    let got_offset = region_start(plt_end, got_count, ELF_AMD64_PAGE_SIZE)?;
    let got_bytes = checked_product(got_count, GOT_ENTRY_SIZE, "GOT")?;
    let got_end = checked_sum(got_offset, got_bytes, "GOT")?;
    let has_metadata = dynsym_count != 0 || dynstr_bytes != 0 || rela_count != 0;
    let metadata_offset = region_start(got_end, usize::from(has_metadata), ELF_AMD64_PAGE_SIZE)?;
    let dynsym_offset = metadata_offset;
    let dynsym_bytes = checked_product(dynsym_count, ELF64_SYMBOL_ENTRY_SIZE, "dynamic symbol")?;
    let dynstr_offset = checked_sum(dynsym_offset, dynsym_bytes, "dynamic symbol")?;
    let dynstr_end = checked_sum(dynstr_offset, dynstr_bytes, "dynamic string")?;
    let rela_offset = region_start(dynstr_end, rela_count, ELF64_RELA_ALIGNMENT)?;
    let rela_bytes = checked_product(rela_count, ELF64_RELA_ENTRY_SIZE, "dynamic relocation")?;
    let planned_memory_span = checked_sum(rela_offset, rela_bytes, "dynamic relocation")?;
    let metadata_bytes = planned_memory_span
        .checked_sub(metadata_offset)
        .ok_or_else(|| "ELF metadata region underflows".to_owned())?;
    let has_structures = plt_count != 0 || got_count != 0 || has_metadata;
    let planned_file_span = if has_structures {
        planned_memory_span.max(base_file_span)
    } else {
        base_file_span
    };
    Ok(PlatformLayout {
        planned_file_span,
        planned_memory_span,
        plt_offset,
        plt_bytes,
        got_offset,
        got_bytes,
        metadata_offset,
        metadata_bytes,
        dynsym_offset,
        dynsym_bytes,
        dynstr_offset,
        rela_offset,
        rela_bytes,
    })
}

fn assign_target_slots(
    seeds: BTreeMap<TargetIdentity, TargetSeed>,
    symbols: &BTreeMap<String, DynamicSymbolSlot>,
    layout: &PlatformLayout,
    image_base: u64,
) -> Result<Vec<PlannedTarget>, String> {
    let mut next_plt = 0usize;
    let mut next_got = 0usize;
    let mut next_rela = 0usize;
    let mut targets = Vec::with_capacity(seeds.len());
    for (ordinal, (identity, mut seed)) in seeds.into_iter().enumerate() {
        seed.relocation_ids.sort();
        let symbol = symbols.get(&seed.target_symbol).ok_or_else(|| {
            format!(
                "ELF platform target `{}` has no dynamic symbol slot",
                seed.target_symbol
            )
        })?;
        let plt_slot = if seed.requires_plt {
            Some(take_slot(&mut next_plt)?)
        } else {
            None
        };
        let got_slot = if seed.requires_got {
            Some(take_slot(&mut next_got)?)
        } else {
            None
        };
        let rela_slot = if seed.dynamic_relocation_type.is_some() {
            Some(take_slot(&mut next_rela)?)
        } else {
            None
        };
        let plt_offset = optional_slot_offset(layout.plt_offset, plt_slot, PLT_ENTRY_SIZE)?;
        let got_offset = optional_slot_offset(layout.got_offset, got_slot, GOT_ENTRY_SIZE)?;
        let rela_offset =
            optional_slot_offset(layout.rela_offset, rela_slot, ELF64_RELA_ENTRY_SIZE)?;
        let plt_virtual = optional_virtual_address(image_base, plt_offset)?;
        let got_virtual = optional_virtual_address(image_base, got_offset)?;
        let plt_got_displacement = match (plt_virtual, got_virtual) {
            (Some(plt), Some(got)) => Some(plt_to_got_displacement(plt, got)?),
            _ => None,
        };
        let dynsym_offset =
            slot_offset(layout.dynsym_offset, symbol.index, ELF64_SYMBOL_ENTRY_SIZE)?;
        let dynstr_offset = checked_sum(
            layout.dynstr_offset,
            symbol.string_offset,
            "dynamic string slot",
        )?;
        let dynamic_info = match (seed.dynamic_relocation_type, got_virtual) {
            (Some(kind), Some(_)) => {
                let symbol_index = u32::try_from(symbol.index)
                    .map_err(|_| "ELF dynamic symbol index exceeds r_info field".to_owned())?;
                Some((u64::from(symbol_index) << 32) | u64::from(kind))
            }
            (Some(_), None) => {
                return Err(format!(
                    "ELF platform target `{}` has a dynamic relocation without a GOT slot",
                    seed.target_symbol
                ))
            }
            (None, _) => None,
        };
        let structure_id = format!("elf-amd64-platform-target-{ordinal:06}");
        let mut plan = ElfAmd64PlatformTargetPlan {
            structure_id,
            rule_ids: seed.rule_ids.into_iter().collect(),
            target_class: identity.target_class.clone(),
            target_key: seed.target_key,
            target_symbol: seed.target_symbol,
            resolver_status: seed.resolver_status,
            dynamic_symbol_index: symbol.index,
            dynamic_string_offset: symbol.string_offset,
            dynamic_symbol_image_offset: dynsym_offset,
            dynamic_string_image_offset: dynstr_offset,
            plt_slot_index: plt_slot,
            plt_image_offset: plt_offset,
            plt_virtual_address: plt_virtual,
            plt_got_displacement,
            got_slot_index: got_slot,
            got_image_offset: got_offset,
            got_virtual_address: got_virtual,
            dynamic_relocation_index: rela_slot,
            dynamic_relocation_image_offset: rela_offset,
            dynamic_relocation_kind: seed.dynamic_relocation_kind,
            dynamic_relocation_type: seed.dynamic_relocation_type,
            dynamic_relocation_offset: got_virtual,
            dynamic_relocation_info: dynamic_info,
            dynamic_relocation_addend: rela_slot.map(|_| 0),
            relocation_ids: seed.relocation_ids,
            audit_hash: String::new(),
        };
        plan.audit_hash = report::target_audit_hash(&plan);
        targets.push(PlannedTarget {
            canonical_identity: identity.canonical(),
            plan,
        });
    }
    Ok(targets)
}

fn build_relocation_binding(
    application: &ElfAmd64RelocationApplication,
    rule: PlatformRule,
    targets: &BTreeMap<&str, &ElfAmd64PlatformTargetPlan>,
) -> Result<ElfAmd64PlatformRelocationBinding, String> {
    let symbol = application.target_symbol.as_deref().ok_or_else(|| {
        format!(
            "ELF deferred relocation `{}` lost its target symbol",
            application.relocation_id
        )
    })?;
    let identity = TargetIdentity {
        target_class: rule.target_class.to_owned(),
        symbol: symbol.to_owned(),
        resolver_status: application.resolver_status.clone(),
    }
    .canonical();
    let target = targets.get(identity.as_str()).copied().ok_or_else(|| {
        format!(
            "ELF deferred relocation `{}` has no allocated platform target",
            application.relocation_id
        )
    })?;
    let (patch_offset, patch_virtual) = match rule.patch_target_kind {
        "plt-entry" => (target.plt_image_offset, target.plt_virtual_address),
        "got-entry" => (target.got_image_offset, target.got_virtual_address),
        other => {
            return Err(format!(
                "ELF platform rule `{}` has unknown patch target `{other}`",
                rule.id
            ))
        }
    };
    let patch_offset = patch_offset.ok_or_else(|| {
        format!(
            "ELF platform target `{}` has no `{}` offset",
            target.structure_id, rule.patch_target_kind
        )
    })?;
    let patch_virtual = patch_virtual.ok_or_else(|| {
        format!(
            "ELF platform target `{}` has no `{}` address",
            target.structure_id, rule.patch_target_kind
        )
    })?;
    let computed = i128::from(patch_virtual) + i128::from(application.addend)
        - i128::from(application.source_virtual_address);
    let signed = i32::try_from(computed).map_err(|_| {
        format!(
            "ELF platform relocation `{}` displacement {computed} overflows i32",
            application.relocation_id
        )
    })?;
    let encoded_value = u64::from(signed as u32);
    let encoded_bytes = signed.to_le_bytes().to_vec();
    let encoded_bytes_hash = crate::fnv1a64_hex(&encoded_bytes);
    let mut binding = ElfAmd64PlatformRelocationBinding {
        relocation_id: application.relocation_id.clone(),
        relocation_kind: application.relocation_kind.clone(),
        action_kind: application.action_kind.clone(),
        rule_id: rule.id.to_owned(),
        structure_id: target.structure_id.clone(),
        source_file_offset: application.source_file_offset,
        source_image_offset: application.source_image_offset,
        source_virtual_address: application.source_virtual_address,
        width_bytes: application.width_bytes,
        patch_target_kind: rule.patch_target_kind.to_owned(),
        patch_target_image_offset: patch_offset,
        patch_target_virtual_address: patch_virtual,
        computed_value: i64::from(signed),
        encoded_value,
        encoded_bytes,
        encoded_bytes_hash,
        audit_hash: String::new(),
    };
    binding.audit_hash = report::binding_audit_hash(&binding);
    Ok(binding)
}

fn plt_to_got_displacement(plt: u64, got: u64) -> Result<i64, String> {
    let instruction_end = plt
        .checked_add(6)
        .ok_or_else(|| "ELF PLT instruction address overflows".to_owned())?;
    let displacement = i128::from(got) - i128::from(instruction_end);
    let displacement = i32::try_from(displacement)
        .map_err(|_| format!("ELF PLT-to-GOT displacement {displacement} overflows i32"))?;
    Ok(i64::from(displacement))
}

fn take_slot(next: &mut usize) -> Result<usize, String> {
    let slot = *next;
    *next = next
        .checked_add(1)
        .ok_or_else(|| "ELF platform slot count overflows".to_owned())?;
    Ok(slot)
}

fn optional_slot_offset(
    region: usize,
    slot: Option<usize>,
    size: usize,
) -> Result<Option<usize>, String> {
    slot.map(|slot| slot_offset(region, slot, size)).transpose()
}

fn slot_offset(region: usize, slot: usize, size: usize) -> Result<usize, String> {
    let relative = slot
        .checked_mul(size)
        .ok_or_else(|| "ELF platform slot offset overflows".to_owned())?;
    checked_sum(region, relative, "platform slot")
}

fn optional_virtual_address(
    image_base: u64,
    image_offset: Option<usize>,
) -> Result<Option<u64>, String> {
    image_offset
        .map(|offset| virtual_address(image_base, offset))
        .transpose()
}

fn virtual_address(image_base: u64, image_offset: usize) -> Result<u64, String> {
    let offset = u64::try_from(image_offset)
        .map_err(|_| "ELF platform image offset exceeds u64".to_owned())?;
    image_base
        .checked_add(offset)
        .ok_or_else(|| "ELF platform virtual address overflows".to_owned())
}

fn region_start(value: usize, count: usize, alignment: usize) -> Result<usize, String> {
    if count == 0 {
        Ok(value)
    } else {
        align_up(value, alignment)
    }
}

fn align_up(value: usize, alignment: usize) -> Result<usize, String> {
    let remainder = value
        .checked_rem(alignment)
        .ok_or_else(|| "ELF platform alignment must be nonzero".to_owned())?;
    if remainder == 0 {
        return Ok(value);
    }
    value
        .checked_add(alignment - remainder)
        .ok_or_else(|| "ELF platform alignment overflows".to_owned())
}

fn checked_product(count: usize, size: usize, label: &str) -> Result<usize, String> {
    count
        .checked_mul(size)
        .ok_or_else(|| format!("ELF {label} region size overflows"))
}

fn checked_sum(offset: usize, size: usize, label: &str) -> Result<usize, String> {
    offset
        .checked_add(size)
        .ok_or_else(|| format!("ELF {label} region end overflows"))
}

fn append_rule(out: &mut String, rule: PlatformRule) {
    append_text(out, rule.id);
    append_text(out, rule.target_class);
    append_text(out, rule.relocation_kind);
    append_text(out, rule.action_kind);
    append_text(out, rule.resolver_status);
    append_text(out, rule.dynamic_relocation_kind.unwrap_or("none"));
    append_text(out, rule.patch_target_kind);
    writeln!(
        out,
        "rule={}|{}|{}|{}|{}",
        rule.width_bytes,
        rule.pc_relative,
        rule.requires_plt,
        rule.requires_got,
        rule.dynamic_relocation_type
            .map_or_else(|| "none".to_owned(), |value| value.to_string())
    )
    .unwrap();
}

fn append_text(out: &mut String, value: &str) {
    writeln!(out, "text:{}:{value}", value.len()).unwrap();
}

#[cfg(test)]
#[path = "final_executable_elf_platform_tests.rs"]
mod tests;
