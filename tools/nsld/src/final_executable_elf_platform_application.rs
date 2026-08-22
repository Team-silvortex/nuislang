#[path = "final_executable_elf_platform_application_patching.rs"]
mod patching;
#[path = "final_executable_elf_platform_application_report.rs"]
mod report;

pub(crate) use report::{
    bind_audit_hash, ElfAmd64PlatformDynamicBindRecord, ElfAmd64PlatformPatchApplicationReport,
    ElfAmd64PlatformPatchAudit, ElfAmd64PlatformWriteAudit,
};

use super::{
    build_elf_amd64_platform_structure_plan, ElfAmd64PlatformStructurePlanReport,
    ElfAmd64PlatformTargetPlan, ELF_AMD64_PLATFORM_STRUCTURE_PLAN_CONTRACT,
};
use crate::{
    final_executable_elf_layout_report::ElfAmd64PlacementBindingReport,
    final_executable_elf_materialization::application::{
        ElfAmd64AppliedImage, ELF_AMD64_PATCH_APPLICATION_CONTRACT,
    },
    final_executable_elf_relocation_report::ElfAmd64RelocationApplicationReport,
};
use std::collections::{BTreeMap, BTreeSet};

use patching::{
    apply_deferred_bindings, apply_write_once, checked_range, expected_structure_write_count,
    hex_bytes, validate_structure_write_coverage, virtual_address,
};

pub(crate) const ELF_AMD64_PLATFORM_PATCH_APPLICATION_CONTRACT: &str =
    "nuis-nsld-elf-amd64-platform-patch-application-v1";

const PLT_ENTRY_SIZE: usize = 16;
const GOT_ENTRY_SIZE: usize = 8;
const ELF64_SYMBOL_ENTRY_SIZE: usize = 24;
const ELF64_RELA_ENTRY_SIZE: usize = 24;

#[derive(Clone, Copy)]
struct PlatformEncoderRule {
    target_class: &'static str,
    resolver_status: &'static str,
    plt_write_kind: &'static str,
    got_write_kind: &'static str,
    dynamic_symbol_write_kind: &'static str,
    dynamic_relocation_write_kind: &'static str,
    dynamic_symbol_info: u8,
    dynamic_relocation_type: u32,
}

const PLATFORM_ENCODERS: &[PlatformEncoderRule] = &[PlatformEncoderRule {
    target_class: "external-function-nonlazy",
    resolver_status: "external-compatibility",
    plt_write_kind: "x86_64-nonlazy-plt-entry",
    got_write_kind: "x86_64-nonlazy-got-placeholder",
    dynamic_symbol_write_kind: "elf64-dynsym-global-function",
    dynamic_relocation_write_kind: "elf64-rela-jump-slot",
    dynamic_symbol_info: 0x12,
    dynamic_relocation_type: 7,
}];

#[derive(Debug)]
pub(crate) struct ElfAmd64PlatformAppliedImage {
    pub(crate) bytes: Vec<u8>,
    pub(crate) report: ElfAmd64PlatformPatchApplicationReport,
}

pub(crate) fn apply_elf_amd64_platform_structure_plan(
    placement: &ElfAmd64PlacementBindingReport,
    relocations: &ElfAmd64RelocationApplicationReport,
    applied: &ElfAmd64AppliedImage,
    plan: &ElfAmd64PlatformStructurePlanReport,
) -> Result<ElfAmd64PlatformAppliedImage, String> {
    validate_input_envelope(placement, relocations, applied, plan)?;

    let mut bytes = applied.bytes.clone();
    bytes.resize(plan.planned_memory_span_bytes, 0);
    let mut occupied = vec![false; bytes.len()];
    reserve_direct_patch_spans(applied, &bytes, &mut occupied)?;

    let dynamic_targets = dynamic_symbol_targets(plan)?;
    let expected_structure_write_count = expected_structure_write_count(plan, &dynamic_targets)?;
    let mut structure_writes = Vec::with_capacity(expected_structure_write_count);
    write_plt_entries(&mut bytes, &mut occupied, plan, &mut structure_writes)?;
    write_got_entries(&mut bytes, &mut occupied, plan, &mut structure_writes)?;
    write_dynamic_symbols(
        &mut bytes,
        &mut occupied,
        plan,
        &dynamic_targets,
        &mut structure_writes,
    )?;
    write_dynamic_strings(
        &mut bytes,
        &mut occupied,
        plan,
        &dynamic_targets,
        &mut structure_writes,
    )?;
    let dynamic_bind_records =
        write_dynamic_relocations(&mut bytes, &mut occupied, plan, &mut structure_writes)?;
    let counts = validate_structure_write_coverage(
        plan,
        &dynamic_targets,
        &structure_writes,
        &dynamic_bind_records,
    )?;

    let patches = apply_deferred_bindings(&mut bytes, &mut occupied, relocations, plan)?;
    let applied_file = bytes
        .get(..plan.planned_file_span_bytes)
        .ok_or_else(|| "ELF platform file span exceeds applied memory image".to_owned())?;
    let status = if plan.deferred_relocation_count == 0 {
        "not-required-image-preserved"
    } else {
        "platform-structures-and-deferred-patches-applied-with-unresolved-dynamic-binds"
    };
    let write_once_span_count = structure_writes
        .len()
        .checked_add(patches.len())
        .ok_or_else(|| "ELF platform write-once span count overflows".to_owned())?;
    let mut report = ElfAmd64PlatformPatchApplicationReport {
        contract: ELF_AMD64_PLATFORM_PATCH_APPLICATION_CONTRACT,
        status: status.to_owned(),
        application_ledger_hash: String::new(),
        placement_plan_hash: placement.plan_hash.clone(),
        relocation_plan_hash: relocations.plan_hash.clone(),
        base_patch_application_ledger_hash: applied.report.application_ledger_hash.clone(),
        platform_structure_plan_hash: plan.plan_hash.clone(),
        base_applied_file_image_hash: applied.report.applied_file_image_hash.clone(),
        base_applied_memory_image_hash: applied.report.applied_memory_image_hash.clone(),
        applied_file_image_hash: crate::fnv1a64_hex(applied_file),
        applied_memory_image_hash: crate::fnv1a64_hex(&bytes),
        base_file_span_bytes: plan.base_file_span_bytes,
        base_memory_span_bytes: plan.base_memory_span_bytes,
        applied_file_span_bytes: plan.planned_file_span_bytes,
        applied_memory_span_bytes: plan.planned_memory_span_bytes,
        expected_structure_write_count,
        applied_structure_write_count: structure_writes.len(),
        expected_deferred_patch_count: plan.deferred_relocation_count,
        applied_deferred_patch_count: patches.len(),
        plt_write_count: counts.plt,
        got_write_count: counts.got,
        dynamic_symbol_write_count: counts.dynamic_symbol,
        dynamic_string_write_count: counts.dynamic_string,
        dynamic_relocation_write_count: counts.dynamic_relocation,
        unresolved_dynamic_bind_count: dynamic_bind_records.len(),
        write_once_span_count,
        structure_writes,
        patches,
        dynamic_bind_records,
    };
    report.application_ledger_hash = crate::fnv1a64_hex(report.canonical_ledger().as_bytes());
    Ok(ElfAmd64PlatformAppliedImage { bytes, report })
}

fn validate_input_envelope(
    placement: &ElfAmd64PlacementBindingReport,
    relocations: &ElfAmd64RelocationApplicationReport,
    applied: &ElfAmd64AppliedImage,
    plan: &ElfAmd64PlatformStructurePlanReport,
) -> Result<(), String> {
    if applied.report.contract != ELF_AMD64_PATCH_APPLICATION_CONTRACT
        || plan.contract != ELF_AMD64_PLATFORM_STRUCTURE_PLAN_CONTRACT
    {
        return Err("ELF platform application rejects an upstream contract".to_owned());
    }
    let base_file = applied
        .bytes
        .get(..applied.report.file_span_bytes)
        .ok_or_else(|| "ELF platform base file span exceeds memory image".to_owned())?;
    if applied.bytes.len() != applied.report.memory_span_bytes
        || crate::fnv1a64_hex(base_file) != applied.report.applied_file_image_hash
        || crate::fnv1a64_hex(&applied.bytes) != applied.report.applied_memory_image_hash
        || applied.report.application_ledger_hash
            != crate::fnv1a64_hex(applied.report.canonical_ledger().as_bytes())
    {
        return Err("ELF platform application base image drift".to_owned());
    }
    let expected =
        build_elf_amd64_platform_structure_plan(placement, relocations, &applied.report)?;
    if expected != *plan {
        return Err("ELF platform application structure plan drift".to_owned());
    }
    if plan.base_file_span_bytes != applied.report.file_span_bytes
        || plan.base_memory_span_bytes != applied.report.memory_span_bytes
        || plan.planned_file_span_bytes < plan.base_file_span_bytes
        || plan.planned_memory_span_bytes < plan.base_memory_span_bytes
        || plan.planned_file_span_bytes > plan.planned_memory_span_bytes
    {
        return Err("ELF platform application span envelope drift".to_owned());
    }
    validate_encoder_registry()
}

fn validate_encoder_registry() -> Result<(), String> {
    let mut identities = BTreeSet::new();
    for rule in PLATFORM_ENCODERS {
        if rule.target_class.is_empty()
            || rule.resolver_status.is_empty()
            || !identities.insert((rule.target_class, rule.resolver_status))
            || rule.dynamic_symbol_info == 0
            || rule.dynamic_relocation_type == 0
        {
            return Err("invalid ELF platform encoder registry".to_owned());
        }
    }
    Ok(())
}

fn encoder_rule(target: &ElfAmd64PlatformTargetPlan) -> Result<PlatformEncoderRule, String> {
    let matches = PLATFORM_ENCODERS
        .iter()
        .filter(|rule| {
            rule.target_class == target.target_class
                && rule.resolver_status == target.resolver_status
        })
        .copied()
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [rule] => Ok(*rule),
        [] => Err(format!(
            "ELF platform target `{}` has no registered byte encoder",
            target.structure_id
        )),
        _ => Err(format!(
            "ELF platform target `{}` matches multiple byte encoders",
            target.structure_id
        )),
    }
}

fn reserve_direct_patch_spans(
    applied: &ElfAmd64AppliedImage,
    bytes: &[u8],
    occupied: &mut [bool],
) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for patch in &applied.report.patches {
        if !seen.insert(patch.relocation_id.as_str()) {
            return Err(format!(
                "ELF platform application repeats inherited patch `{}`",
                patch.relocation_id
            ));
        }
        checked_range(
            patch.source_file_offset,
            patch.width_bytes,
            applied.report.file_span_bytes,
            &patch.relocation_id,
        )?;
        let range = checked_range(
            patch.source_image_offset,
            patch.width_bytes,
            applied.report.memory_span_bytes,
            &patch.relocation_id,
        )?;
        if occupied[range.clone()].iter().any(|value| *value) {
            return Err(format!(
                "ELF inherited patch `{}` overlaps an earlier direct patch",
                patch.relocation_id
            ));
        }
        let observed_hash = crate::fnv1a64_hex(&bytes[range.clone()]);
        if observed_hash != patch.post_write_bytes_hash
            || patch.post_write_bytes_hash != patch.encoded_bytes_hash
        {
            return Err(format!(
                "ELF inherited patch `{}` byte hash drift",
                patch.relocation_id
            ));
        }
        occupied[range].fill(true);
    }
    if seen.len() != applied.report.applied_patch_count
        || applied.report.write_once_span_count != applied.report.applied_patch_count
    {
        return Err("ELF inherited direct patch coverage drift".to_owned());
    }
    Ok(())
}

fn dynamic_symbol_targets(
    plan: &ElfAmd64PlatformStructurePlanReport,
) -> Result<Vec<&ElfAmd64PlatformTargetPlan>, String> {
    let mut targets = BTreeMap::new();
    for target in &plan.targets {
        if target.dynamic_symbol_index == 0 {
            return Err(format!(
                "ELF platform target `{}` uses reserved dynamic symbol zero",
                target.structure_id
            ));
        }
        if let Some(previous) = targets.insert(target.dynamic_symbol_index, target) {
            if previous.target_symbol != target.target_symbol
                || previous.dynamic_string_offset != target.dynamic_string_offset
                || previous.dynamic_symbol_image_offset != target.dynamic_symbol_image_offset
                || previous.dynamic_string_image_offset != target.dynamic_string_image_offset
            {
                return Err(format!(
                    "ELF dynamic symbol index {} has conflicting targets",
                    target.dynamic_symbol_index
                ));
            }
        }
    }
    let targets = targets.into_values().collect::<Vec<_>>();
    for (ordinal, target) in targets.iter().enumerate() {
        if target.dynamic_symbol_index != ordinal + 1 {
            return Err("ELF dynamic symbol indexes are not contiguous".to_owned());
        }
    }
    let expected = if targets.is_empty() {
        0
    } else {
        targets
            .len()
            .checked_add(1)
            .ok_or_else(|| "ELF dynamic symbol count overflows".to_owned())?
    };
    if plan.dynamic_symbol_entry_count != expected {
        return Err("ELF dynamic symbol plan coverage drift".to_owned());
    }
    Ok(targets)
}

fn write_plt_entries(
    bytes: &mut [u8],
    occupied: &mut [bool],
    plan: &ElfAmd64PlatformStructurePlanReport,
    writes: &mut Vec<ElfAmd64PlatformWriteAudit>,
) -> Result<(), String> {
    for target in &plan.targets {
        if let Some(offset) = target.plt_image_offset {
            let rule = encoder_rule(target)?;
            let encoded = encode_plt_entry(target)?;
            write_structure(
                bytes,
                occupied,
                plan,
                target,
                rule.plt_write_kind,
                offset,
                &encoded,
                writes,
            )?;
        }
    }
    Ok(())
}

fn write_got_entries(
    bytes: &mut [u8],
    occupied: &mut [bool],
    plan: &ElfAmd64PlatformStructurePlanReport,
    writes: &mut Vec<ElfAmd64PlatformWriteAudit>,
) -> Result<(), String> {
    for target in &plan.targets {
        if let Some(offset) = target.got_image_offset {
            let rule = encoder_rule(target)?;
            write_structure(
                bytes,
                occupied,
                plan,
                target,
                rule.got_write_kind,
                offset,
                &[0; GOT_ENTRY_SIZE],
                writes,
            )?;
        }
    }
    Ok(())
}

fn write_dynamic_symbols(
    bytes: &mut [u8],
    occupied: &mut [bool],
    plan: &ElfAmd64PlatformStructurePlanReport,
    targets: &[&ElfAmd64PlatformTargetPlan],
    writes: &mut Vec<ElfAmd64PlatformWriteAudit>,
) -> Result<(), String> {
    if targets.is_empty() {
        return Ok(());
    }
    write_anonymous_structure(
        bytes,
        occupied,
        plan,
        "elf-amd64-platform-dynsym-null",
        "elf64-dynsym-null",
        plan.dynamic_symbol_region_image_offset,
        &[0; ELF64_SYMBOL_ENTRY_SIZE],
        writes,
    )?;
    for target in targets {
        let rule = encoder_rule(target)?;
        let encoded = encode_dynamic_symbol(target, rule)?;
        write_structure(
            bytes,
            occupied,
            plan,
            target,
            rule.dynamic_symbol_write_kind,
            target.dynamic_symbol_image_offset,
            &encoded,
            writes,
        )?;
    }
    Ok(())
}

fn write_dynamic_strings(
    bytes: &mut [u8],
    occupied: &mut [bool],
    plan: &ElfAmd64PlatformStructurePlanReport,
    targets: &[&ElfAmd64PlatformTargetPlan],
    writes: &mut Vec<ElfAmd64PlatformWriteAudit>,
) -> Result<(), String> {
    if targets.is_empty() {
        return Ok(());
    }
    write_anonymous_structure(
        bytes,
        occupied,
        plan,
        "elf-amd64-platform-dynstr-null",
        "elf64-dynstr-null",
        plan.dynamic_string_region_image_offset,
        &[0],
        writes,
    )?;
    for target in targets {
        let mut encoded = target.target_symbol.as_bytes().to_vec();
        encoded.push(0);
        write_structure(
            bytes,
            occupied,
            plan,
            target,
            "elf64-dynstr-symbol",
            target.dynamic_string_image_offset,
            &encoded,
            writes,
        )?;
    }
    Ok(())
}

fn write_dynamic_relocations(
    bytes: &mut [u8],
    occupied: &mut [bool],
    plan: &ElfAmd64PlatformStructurePlanReport,
    writes: &mut Vec<ElfAmd64PlatformWriteAudit>,
) -> Result<Vec<ElfAmd64PlatformDynamicBindRecord>, String> {
    let mut binds = Vec::with_capacity(plan.dynamic_relocation_entry_count);
    for target in &plan.targets {
        if let Some(offset) = target.dynamic_relocation_image_offset {
            let rule = encoder_rule(target)?;
            let encoded = encode_dynamic_relocation(target, rule)?;
            write_structure(
                bytes,
                occupied,
                plan,
                target,
                rule.dynamic_relocation_write_kind,
                offset,
                &encoded,
                writes,
            )?;
            binds.push(build_dynamic_bind_record(binds.len(), target)?);
        }
    }
    Ok(binds)
}

#[allow(clippy::too_many_arguments)]
fn write_structure(
    bytes: &mut [u8],
    occupied: &mut [bool],
    plan: &ElfAmd64PlatformStructurePlanReport,
    target: &ElfAmd64PlatformTargetPlan,
    write_kind: &str,
    image_offset: usize,
    encoded: &[u8],
    writes: &mut Vec<ElfAmd64PlatformWriteAudit>,
) -> Result<(), String> {
    write_structure_record(
        bytes,
        occupied,
        plan,
        &target.structure_id,
        write_kind,
        &target.target_symbol,
        image_offset,
        encoded,
        writes,
    )
}

#[allow(clippy::too_many_arguments)]
fn write_anonymous_structure(
    bytes: &mut [u8],
    occupied: &mut [bool],
    plan: &ElfAmd64PlatformStructurePlanReport,
    structure_id: &str,
    write_kind: &str,
    image_offset: usize,
    encoded: &[u8],
    writes: &mut Vec<ElfAmd64PlatformWriteAudit>,
) -> Result<(), String> {
    write_structure_record(
        bytes,
        occupied,
        plan,
        structure_id,
        write_kind,
        "",
        image_offset,
        encoded,
        writes,
    )
}

#[allow(clippy::too_many_arguments)]
fn write_structure_record(
    bytes: &mut [u8],
    occupied: &mut [bool],
    plan: &ElfAmd64PlatformStructurePlanReport,
    structure_id: &str,
    write_kind: &str,
    target_symbol: &str,
    image_offset: usize,
    encoded: &[u8],
    writes: &mut Vec<ElfAmd64PlatformWriteAudit>,
) -> Result<(), String> {
    let write_id = format!("elf-amd64-platform-write-{:06}", writes.len());
    let source = vec![0; encoded.len()];
    apply_write_once(bytes, occupied, image_offset, &source, encoded, &write_id)?;
    let source_bytes_hash = crate::fnv1a64_hex(&source);
    let encoded_bytes_hash = crate::fnv1a64_hex(encoded);
    let mut audit = ElfAmd64PlatformWriteAudit {
        write_id,
        structure_id: structure_id.to_owned(),
        write_kind: write_kind.to_owned(),
        target_symbol: target_symbol.to_owned(),
        image_offset,
        virtual_address: virtual_address(plan.image_base, image_offset)?,
        width_bytes: encoded.len(),
        source_bytes_hash,
        encoded_bytes_hash: encoded_bytes_hash.clone(),
        post_write_bytes_hash: encoded_bytes_hash,
        encoded_bytes_hex: hex_bytes(encoded),
        status: "applied-write-once".to_owned(),
        audit_hash: String::new(),
    };
    audit.audit_hash = report::write_audit_hash(&audit);
    writes.push(audit);
    Ok(())
}

fn encode_plt_entry(target: &ElfAmd64PlatformTargetPlan) -> Result<[u8; PLT_ENTRY_SIZE], String> {
    let displacement = target.plt_got_displacement.ok_or_else(|| {
        format!(
            "ELF PLT target `{}` has no GOT displacement",
            target.structure_id
        )
    })?;
    let displacement = i32::try_from(displacement).map_err(|_| {
        format!(
            "ELF PLT target `{}` displacement overflows i32",
            target.structure_id
        )
    })?;
    let plt = target.plt_virtual_address.ok_or_else(|| {
        format!(
            "ELF PLT target `{}` has no virtual address",
            target.structure_id
        )
    })?;
    let got = target.got_virtual_address.ok_or_else(|| {
        format!(
            "ELF PLT target `{}` has no GOT address",
            target.structure_id
        )
    })?;
    let instruction_end = plt
        .checked_add(6)
        .ok_or_else(|| "ELF PLT instruction address overflows".to_owned())?;
    let expected = i128::from(got) - i128::from(instruction_end);
    if expected != i128::from(displacement) {
        return Err(format!(
            "ELF PLT target `{}` displacement drift",
            target.structure_id
        ));
    }
    let mut encoded = [0x90; PLT_ENTRY_SIZE];
    encoded[0] = 0xff;
    encoded[1] = 0x25;
    encoded[2..6].copy_from_slice(&displacement.to_le_bytes());
    Ok(encoded)
}

fn encode_dynamic_symbol(
    target: &ElfAmd64PlatformTargetPlan,
    rule: PlatformEncoderRule,
) -> Result<[u8; ELF64_SYMBOL_ENTRY_SIZE], String> {
    let name = u32::try_from(target.dynamic_string_offset)
        .map_err(|_| "ELF dynamic string offset exceeds u32".to_owned())?;
    let mut encoded = [0; ELF64_SYMBOL_ENTRY_SIZE];
    encoded[0..4].copy_from_slice(&name.to_le_bytes());
    encoded[4] = rule.dynamic_symbol_info;
    Ok(encoded)
}

fn encode_dynamic_relocation(
    target: &ElfAmd64PlatformTargetPlan,
    rule: PlatformEncoderRule,
) -> Result<[u8; ELF64_RELA_ENTRY_SIZE], String> {
    let relocation_type = target.dynamic_relocation_type.ok_or_else(|| {
        format!(
            "ELF platform target `{}` has no relocation type",
            target.structure_id
        )
    })?;
    let offset = target.dynamic_relocation_offset.ok_or_else(|| {
        format!(
            "ELF platform target `{}` has no relocation offset",
            target.structure_id
        )
    })?;
    let info = target.dynamic_relocation_info.ok_or_else(|| {
        format!(
            "ELF platform target `{}` has no relocation info",
            target.structure_id
        )
    })?;
    let addend = target.dynamic_relocation_addend.ok_or_else(|| {
        format!(
            "ELF platform target `{}` has no relocation addend",
            target.structure_id
        )
    })?;
    let expected_info = (u64::try_from(target.dynamic_symbol_index)
        .map_err(|_| "ELF dynamic symbol index exceeds u64".to_owned())?
        << 32)
        | u64::from(relocation_type);
    if relocation_type != rule.dynamic_relocation_type
        || info != expected_info
        || target.got_virtual_address != Some(offset)
        || addend != 0
    {
        return Err(format!(
            "ELF platform target `{}` dynamic relocation drift",
            target.structure_id
        ));
    }
    let mut encoded = [0; ELF64_RELA_ENTRY_SIZE];
    encoded[0..8].copy_from_slice(&offset.to_le_bytes());
    encoded[8..16].copy_from_slice(&info.to_le_bytes());
    encoded[16..24].copy_from_slice(&addend.to_le_bytes());
    Ok(encoded)
}

fn build_dynamic_bind_record(
    index: usize,
    target: &ElfAmd64PlatformTargetPlan,
) -> Result<ElfAmd64PlatformDynamicBindRecord, String> {
    let mut bind = ElfAmd64PlatformDynamicBindRecord {
        bind_id: format!("elf-amd64-platform-bind-{index:06}"),
        structure_id: target.structure_id.clone(),
        target_key: target.target_key.clone(),
        target_symbol: target.target_symbol.clone(),
        dynamic_symbol_index: target.dynamic_symbol_index,
        got_image_offset: required(target.got_image_offset, target, "GOT image offset")?,
        got_virtual_address: required(target.got_virtual_address, target, "GOT address")?,
        relocation_image_offset: required(
            target.dynamic_relocation_image_offset,
            target,
            "relocation image offset",
        )?,
        relocation_kind: target.dynamic_relocation_kind.clone().ok_or_else(|| {
            format!(
                "ELF platform target `{}` has no relocation kind",
                target.structure_id
            )
        })?,
        relocation_type: required(target.dynamic_relocation_type, target, "relocation type")?,
        relocation_offset: required(
            target.dynamic_relocation_offset,
            target,
            "relocation offset",
        )?,
        relocation_info: required(target.dynamic_relocation_info, target, "relocation info")?,
        relocation_addend: required(
            target.dynamic_relocation_addend,
            target,
            "relocation addend",
        )?,
        status: "unresolved-external-dynamic-bind".to_owned(),
        audit_hash: String::new(),
    };
    bind.audit_hash = report::bind_audit_hash(&bind);
    Ok(bind)
}

fn required<T: Copy>(
    value: Option<T>,
    target: &ElfAmd64PlatformTargetPlan,
    label: &str,
) -> Result<T, String> {
    value.ok_or_else(|| {
        format!(
            "ELF platform target `{}` has no {label}",
            target.structure_id
        )
    })
}

#[cfg(test)]
#[path = "final_executable_elf_platform_application_tests.rs"]
mod tests;
