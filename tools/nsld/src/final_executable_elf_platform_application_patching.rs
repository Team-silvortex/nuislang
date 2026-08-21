use super::{
    report, ElfAmd64PlatformDynamicBindRecord, ElfAmd64PlatformPatchAudit,
    ElfAmd64PlatformWriteAudit,
};
use crate::{
    final_executable_elf_materialization::application::platform::{
        ElfAmd64PlatformRelocationBinding, ElfAmd64PlatformStructurePlanReport,
        ElfAmd64PlatformTargetPlan,
    },
    final_executable_elf_relocation_report::{
        ElfAmd64RelocationApplication, ElfAmd64RelocationApplicationReport,
    },
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn apply_deferred_bindings(
    bytes: &mut [u8],
    occupied: &mut [bool],
    relocations: &ElfAmd64RelocationApplicationReport,
    plan: &ElfAmd64PlatformStructurePlanReport,
) -> Result<Vec<ElfAmd64PlatformPatchAudit>, String> {
    let applications = relocation_application_map(relocations)?;
    let targets = target_map(plan)?;
    let mut seen = BTreeSet::new();
    let mut patches = Vec::with_capacity(plan.deferred_relocation_count);
    for binding in &plan.relocation_bindings {
        if !seen.insert(binding.relocation_id.as_str()) {
            return Err(format!(
                "ELF platform application repeats deferred relocation `{}`",
                binding.relocation_id
            ));
        }
        let application = applications
            .get(binding.relocation_id.as_str())
            .copied()
            .ok_or_else(|| {
                format!(
                    "ELF platform binding references missing relocation `{}`",
                    binding.relocation_id
                )
            })?;
        let target = targets
            .get(binding.structure_id.as_str())
            .copied()
            .ok_or_else(|| {
                format!(
                    "ELF platform binding references missing target `{}`",
                    binding.structure_id
                )
            })?;
        validate_binding_identity(application, binding, target)?;
        let range = checked_range(
            binding.source_image_offset,
            binding.width_bytes,
            plan.base_memory_span_bytes,
            &binding.relocation_id,
        )?;
        let source = bytes[range.clone()].to_vec();
        if binding.encoded_bytes.len() != binding.width_bytes
            || crate::fnv1a64_hex(&binding.encoded_bytes) != binding.encoded_bytes_hash
        {
            return Err(format!(
                "ELF platform binding `{}` encoded bytes drift",
                binding.relocation_id
            ));
        }
        apply_write_once(
            bytes,
            occupied,
            binding.source_image_offset,
            &source,
            &binding.encoded_bytes,
            &binding.relocation_id,
        )?;
        let post_write_bytes_hash = crate::fnv1a64_hex(&bytes[range]);
        if post_write_bytes_hash != binding.encoded_bytes_hash {
            return Err(format!(
                "ELF platform binding `{}` post-write hash drift",
                binding.relocation_id
            ));
        }
        let mut audit = ElfAmd64PlatformPatchAudit {
            relocation_id: binding.relocation_id.clone(),
            structure_id: binding.structure_id.clone(),
            rule_id: binding.rule_id.clone(),
            relocation_kind: binding.relocation_kind.clone(),
            source_file_offset: binding.source_file_offset,
            source_image_offset: binding.source_image_offset,
            source_virtual_address: binding.source_virtual_address,
            width_bytes: binding.width_bytes,
            patch_target_image_offset: binding.patch_target_image_offset,
            patch_target_virtual_address: binding.patch_target_virtual_address,
            source_bytes_hash: crate::fnv1a64_hex(&source),
            encoded_bytes_hash: binding.encoded_bytes_hash.clone(),
            post_write_bytes_hash,
            binding_audit_hash: binding.audit_hash.clone(),
            write_audit_hash: String::new(),
            status: "applied-write-once".to_owned(),
        };
        audit.write_audit_hash = report::patch_audit_hash(&audit);
        patches.push(audit);
    }
    if seen.len() != plan.deferred_relocation_count
        || patches.len() != relocations.platform_structure_count
    {
        return Err("ELF platform deferred patch coverage drift".to_owned());
    }
    Ok(patches)
}

fn validate_binding_identity(
    application: &ElfAmd64RelocationApplication,
    binding: &ElfAmd64PlatformRelocationBinding,
    target: &ElfAmd64PlatformTargetPlan,
) -> Result<(), String> {
    if application.application_status != "planned-platform-structure"
        || application.relocation_kind != binding.relocation_kind
        || application.action_kind != binding.action_kind
        || application.source_file_offset != binding.source_file_offset
        || application.source_image_offset != binding.source_image_offset
        || application.source_virtual_address != binding.source_virtual_address
        || application.width_bytes != binding.width_bytes
        || application.target_symbol.as_deref() != Some(target.target_symbol.as_str())
        || !target.relocation_ids.contains(&binding.relocation_id)
    {
        return Err(format!(
            "ELF platform binding `{}` identity drift",
            binding.relocation_id
        ));
    }
    Ok(())
}

fn relocation_application_map(
    relocations: &ElfAmd64RelocationApplicationReport,
) -> Result<BTreeMap<&str, &ElfAmd64RelocationApplication>, String> {
    let mut applications = BTreeMap::new();
    for application in &relocations.applications {
        if applications
            .insert(application.relocation_id.as_str(), application)
            .is_some()
        {
            return Err(format!(
                "ELF relocation report repeats id `{}`",
                application.relocation_id
            ));
        }
    }
    Ok(applications)
}

fn target_map(
    plan: &ElfAmd64PlatformStructurePlanReport,
) -> Result<BTreeMap<&str, &ElfAmd64PlatformTargetPlan>, String> {
    let mut targets = BTreeMap::new();
    for target in &plan.targets {
        if targets
            .insert(target.structure_id.as_str(), target)
            .is_some()
        {
            return Err(format!(
                "ELF platform plan repeats target `{}`",
                target.structure_id
            ));
        }
    }
    Ok(targets)
}

pub(super) struct StructureWriteCounts {
    pub(super) plt: usize,
    pub(super) got: usize,
    pub(super) dynamic_symbol: usize,
    pub(super) dynamic_string: usize,
    pub(super) dynamic_relocation: usize,
}

pub(super) fn expected_structure_write_count(
    plan: &ElfAmd64PlatformStructurePlanReport,
    dynamic_targets: &[&ElfAmd64PlatformTargetPlan],
) -> Result<usize, String> {
    let dynamic_string = if dynamic_targets.is_empty() {
        0
    } else {
        dynamic_targets
            .len()
            .checked_add(1)
            .ok_or_else(|| "ELF dynamic string write count overflows".to_owned())?
    };
    plan.plt_entry_count
        .checked_add(plan.got_entry_count)
        .and_then(|count| count.checked_add(plan.dynamic_symbol_entry_count))
        .and_then(|count| count.checked_add(dynamic_string))
        .and_then(|count| count.checked_add(plan.dynamic_relocation_entry_count))
        .ok_or_else(|| "ELF platform structure write count overflows".to_owned())
}

pub(super) fn validate_structure_write_coverage(
    plan: &ElfAmd64PlatformStructurePlanReport,
    dynamic_targets: &[&ElfAmd64PlatformTargetPlan],
    writes: &[ElfAmd64PlatformWriteAudit],
    binds: &[ElfAmd64PlatformDynamicBindRecord],
) -> Result<StructureWriteCounts, String> {
    let counts = StructureWriteCounts {
        plt: count_write_kind(writes, "x86_64-nonlazy-plt-entry"),
        got: count_write_kind(writes, "x86_64-nonlazy-got-placeholder"),
        dynamic_symbol: writes
            .iter()
            .filter(|write| write.write_kind.starts_with("elf64-dynsym-"))
            .count(),
        dynamic_string: writes
            .iter()
            .filter(|write| write.write_kind.starts_with("elf64-dynstr-"))
            .count(),
        dynamic_relocation: count_write_kind(writes, "elf64-rela-jump-slot"),
    };
    let expected_dynamic_strings = usize::from(!dynamic_targets.is_empty()) + dynamic_targets.len();
    let expected_total = expected_structure_write_count(plan, dynamic_targets)?;
    if counts.plt != plan.plt_entry_count
        || counts.got != plan.got_entry_count
        || counts.dynamic_symbol != plan.dynamic_symbol_entry_count
        || counts.dynamic_string != expected_dynamic_strings
        || counts.dynamic_relocation != plan.dynamic_relocation_entry_count
        || binds.len() != plan.dynamic_relocation_entry_count
        || writes.len() != expected_total
    {
        return Err("ELF platform structure write coverage drift".to_owned());
    }
    Ok(counts)
}

fn count_write_kind(writes: &[ElfAmd64PlatformWriteAudit], kind: &str) -> usize {
    writes
        .iter()
        .filter(|write| write.write_kind == kind)
        .count()
}

pub(super) fn apply_write_once(
    image: &mut [u8],
    occupied: &mut [bool],
    offset: usize,
    source: &[u8],
    encoded: &[u8],
    write_id: &str,
) -> Result<(), String> {
    if image.len() != occupied.len() || source.len() != encoded.len() || encoded.is_empty() {
        return Err(format!(
            "ELF platform write `{write_id}` buffer shape drift"
        ));
    }
    let range = checked_range(offset, encoded.len(), image.len(), write_id)?;
    if occupied[range.clone()].iter().any(|value| *value) {
        return Err(format!(
            "ELF platform write `{write_id}` overlaps a previously committed span"
        ));
    }
    if image[range.clone()] != *source {
        return Err(format!("ELF platform write `{write_id}` source drift"));
    }
    image[range.clone()].copy_from_slice(encoded);
    occupied[range].fill(true);
    Ok(())
}

pub(super) fn checked_range(
    offset: usize,
    width: usize,
    limit: usize,
    write_id: &str,
) -> Result<std::ops::Range<usize>, String> {
    let end = offset
        .checked_add(width)
        .ok_or_else(|| format!("ELF platform write `{write_id}` range overflows"))?;
    if end > limit {
        return Err(format!(
            "ELF platform write `{write_id}` range {offset}..{end} exceeds image {limit}"
        ));
    }
    Ok(offset..end)
}

pub(super) fn virtual_address(image_base: u64, image_offset: usize) -> Result<u64, String> {
    let offset = u64::try_from(image_offset)
        .map_err(|_| "ELF platform image offset exceeds u64".to_owned())?;
    image_base
        .checked_add(offset)
        .ok_or_else(|| "ELF platform virtual address overflows".to_owned())
}

pub(super) fn hex_bytes(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}
