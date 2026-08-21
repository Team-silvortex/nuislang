use crate::{
    final_executable_elf_layout::{ELF_AMD64_PAGE_SIZE, ELF_AMD64_PLACEMENT_BINDING_CONTRACT},
    final_executable_elf_layout_report::{
        ElfAmd64PlacementBindingReport, ElfAmd64SectionPlacement,
    },
    final_executable_elf_materialization::application::{
        ElfAmd64PatchApplicationReport, ELF_AMD64_PATCH_APPLICATION_CONTRACT,
    },
    final_executable_elf_relocation::ELF_AMD64_RELOCATION_APPLICATION_CONTRACT,
    final_executable_elf_relocation_report::{
        ElfAmd64RelocationApplication, ElfAmd64RelocationApplicationReport,
    },
};
use std::collections::BTreeSet;

pub(super) fn validate_input_envelope(
    placement: &ElfAmd64PlacementBindingReport,
    relocations: &ElfAmd64RelocationApplicationReport,
    applied: &ElfAmd64PatchApplicationReport,
) -> Result<(), String> {
    validate_placement(placement)?;
    validate_relocations(placement, relocations)?;
    validate_application(placement, relocations, applied)
}

pub(super) fn validate_deferred_source(
    application: &ElfAmd64RelocationApplication,
    placement: &ElfAmd64PlacementBindingReport,
) -> Result<(), String> {
    if application.application_status != "planned-platform-structure"
        || application.resolver_status != "external-compatibility"
        || !application.target_symbol_external
        || application
            .target_symbol
            .as_deref()
            .is_none_or(str::is_empty)
        || application.target_object_id.is_some()
        || application.target_kind.is_some()
        || application.target_section_id.is_some()
        || application.target_image_offset.is_some()
        || application.target_virtual_address.is_some()
        || application.target_absolute_value.is_some()
        || application.computed_value.is_some()
        || application.encoded_value.is_some()
        || !application.encoded_bytes.is_empty()
    {
        return Err(format!(
            "ELF deferred relocation `{}` has an invalid external target envelope",
            application.relocation_id
        ));
    }
    let source = source_placement(application, placement)?;
    let source_end = application
        .source_offset
        .checked_add(application.width_bytes)
        .ok_or_else(|| {
            format!(
                "ELF deferred relocation `{}` source span overflows",
                application.relocation_id
            )
        })?;
    if source_end > source.size_bytes {
        return Err(format!(
            "ELF deferred relocation `{}` exceeds source section `{}`",
            application.relocation_id, source.output_section_id
        ));
    }
    let expected_file = source
        .file_offset
        .ok_or_else(|| {
            format!(
                "ELF deferred relocation `{}` source is not file-backed",
                application.relocation_id
            )
        })?
        .checked_add(application.source_offset)
        .ok_or_else(|| "ELF deferred source file offset overflows".to_owned())?;
    let expected_image = source
        .image_offset
        .checked_add(application.source_offset)
        .ok_or_else(|| "ELF deferred source image offset overflows".to_owned())?;
    let source_offset_u64 = u64::try_from(application.source_offset)
        .map_err(|_| "ELF deferred source offset exceeds u64".to_owned())?;
    let expected_virtual = source
        .virtual_address
        .checked_add(source_offset_u64)
        .ok_or_else(|| "ELF deferred source virtual address overflows".to_owned())?;
    let file_end = expected_file
        .checked_add(application.width_bytes)
        .ok_or_else(|| "ELF deferred source file span overflows".to_owned())?;
    let image_end = expected_image
        .checked_add(application.width_bytes)
        .ok_or_else(|| "ELF deferred source image span overflows".to_owned())?;
    if application.source_file_offset != expected_file
        || application.source_image_offset != expected_image
        || application.source_virtual_address != expected_virtual
        || file_end > placement.file_span_bytes
        || image_end > placement.memory_span_bytes
    {
        return Err(format!(
            "ELF deferred relocation `{}` source placement drift",
            application.relocation_id
        ));
    }
    Ok(())
}

fn validate_placement(placement: &ElfAmd64PlacementBindingReport) -> Result<(), String> {
    if placement.contract != ELF_AMD64_PLACEMENT_BINDING_CONTRACT {
        return Err(format!(
            "ELF platform plan rejects placement contract `{}`",
            placement.contract
        ));
    }
    if placement.plan_hash != crate::fnv1a64_hex(placement.canonical_plan().as_bytes()) {
        return Err("ELF platform plan placement hash drift".to_owned());
    }
    let internal_count = placement
        .symbol_bindings
        .iter()
        .filter(|binding| binding.status == "internal")
        .count();
    let external_count = placement
        .symbol_bindings
        .iter()
        .filter(|binding| binding.status == "external-compatibility")
        .count();
    let expected_status = if external_count == 0 {
        "placement-and-internal-binding-ready"
    } else {
        "placement-ready-with-external-compatibility-boundary"
    };
    if placement.status != expected_status
        || placement.internally_bound_symbol_count != internal_count
        || placement.external_compatibility_symbol_count != external_count
        || placement.payload_file_offset != ELF_AMD64_PAGE_SIZE
        || placement.file_span_bytes > placement.memory_span_bytes
    {
        return Err("ELF platform plan placement envelope drift".to_owned());
    }
    Ok(())
}

fn validate_relocations(
    placement: &ElfAmd64PlacementBindingReport,
    relocations: &ElfAmd64RelocationApplicationReport,
) -> Result<(), String> {
    if relocations.contract != ELF_AMD64_RELOCATION_APPLICATION_CONTRACT {
        return Err(format!(
            "ELF platform plan rejects relocation contract `{}`",
            relocations.contract
        ));
    }
    if relocations.plan_hash != crate::fnv1a64_hex(relocations.canonical_plan().as_bytes())
        || relocations.placement_plan_hash != placement.plan_hash
    {
        return Err("ELF platform plan relocation hash drift".to_owned());
    }
    let direct = count_status(relocations, "planned-direct");
    let deferred = count_status(relocations, "planned-platform-structure");
    let no_op = count_status(relocations, "no-op");
    let expected_status = if deferred == 0 {
        "ready-for-byte-preview"
    } else {
        "preview-ready-with-platform-structure-boundary"
    };
    let unique_ids = relocations
        .applications
        .iter()
        .map(|application| application.relocation_id.as_str())
        .collect::<BTreeSet<_>>();
    if relocations.status != expected_status
        || relocations.relocation_count != relocations.applications.len()
        || unique_ids.len() != relocations.applications.len()
        || direct + deferred + no_op != relocations.applications.len()
        || relocations.direct_preview_count != direct
        || relocations.platform_structure_count != deferred
        || relocations.no_op_count != no_op
    {
        return Err("ELF platform plan relocation envelope drift".to_owned());
    }
    Ok(())
}

fn validate_application(
    placement: &ElfAmd64PlacementBindingReport,
    relocations: &ElfAmd64RelocationApplicationReport,
    applied: &ElfAmd64PatchApplicationReport,
) -> Result<(), String> {
    if applied.contract != ELF_AMD64_PATCH_APPLICATION_CONTRACT {
        return Err(format!(
            "ELF platform plan rejects patch application contract `{}`",
            applied.contract
        ));
    }
    if applied.application_ledger_hash != crate::fnv1a64_hex(applied.canonical_ledger().as_bytes())
    {
        return Err("ELF platform plan patch application ledger drift".to_owned());
    }
    let expected_status = if relocations.platform_structure_count == 0 {
        "direct-patches-applied"
    } else {
        "direct-patches-applied-with-platform-structure-boundary"
    };
    let direct_ids = relocations
        .applications
        .iter()
        .filter(|application| application.application_status == "planned-direct")
        .map(|application| application.relocation_id.as_str())
        .collect::<Vec<_>>();
    let applied_ids = applied
        .patches
        .iter()
        .map(|patch| patch.relocation_id.as_str())
        .collect::<Vec<_>>();
    validate_direct_patch_audits(relocations, applied)?;
    if applied.status != expected_status
        || applied.placement_plan_hash != placement.plan_hash
        || applied.relocation_plan_hash != relocations.plan_hash
        || applied.file_span_bytes != placement.file_span_bytes
        || applied.memory_span_bytes != placement.memory_span_bytes
        || applied.expected_patch_count != relocations.direct_preview_count
        || applied.applied_patch_count != applied.expected_patch_count
        || applied.write_once_span_count != applied.applied_patch_count
        || applied.patches.len() != applied.applied_patch_count
        || applied.deferred_patch_count != relocations.platform_structure_count
        || applied.no_op_count != relocations.no_op_count
        || direct_ids != applied_ids
        || applied.applied_file_image_hash.is_empty()
        || applied.applied_memory_image_hash.is_empty()
    {
        return Err("ELF platform plan patch application envelope drift".to_owned());
    }
    Ok(())
}

fn validate_direct_patch_audits(
    relocations: &ElfAmd64RelocationApplicationReport,
    applied: &ElfAmd64PatchApplicationReport,
) -> Result<(), String> {
    let direct = relocations
        .applications
        .iter()
        .filter(|application| application.application_status == "planned-direct");
    for (application, patch) in direct.zip(&applied.patches) {
        let encoded_hash = crate::fnv1a64_hex(&application.encoded_bytes);
        if patch.relocation_id != application.relocation_id
            || patch.relocation_kind != application.relocation_kind
            || patch.source_file_offset != application.source_file_offset
            || patch.source_image_offset != application.source_image_offset
            || patch.width_bytes != application.width_bytes
            || patch.encoded_bytes_hash != encoded_hash
            || patch.post_write_bytes_hash != patch.encoded_bytes_hash
            || patch.source_bytes_hash.is_empty()
            || patch.preview_audit_hash.is_empty()
            || patch.write_audit_hash.is_empty()
            || patch.status != "applied-write-once"
        {
            return Err(format!(
                "ELF platform plan patch audit `{}` drift",
                application.relocation_id
            ));
        }
    }
    Ok(())
}

fn source_placement<'a>(
    application: &ElfAmd64RelocationApplication,
    placement: &'a ElfAmd64PlacementBindingReport,
) -> Result<&'a ElfAmd64SectionPlacement, String> {
    placement
        .section_placements
        .iter()
        .find(|source| {
            source.object_id == application.object_id
                && source.input_section_index == application.input_section_index
                && source.output_section_id == application.source_section_id
        })
        .ok_or_else(|| {
            format!(
                "ELF deferred relocation `{}` has no source placement",
                application.relocation_id
            )
        })
}

fn count_status(relocations: &ElfAmd64RelocationApplicationReport, status: &str) -> usize {
    relocations
        .applications
        .iter()
        .filter(|application| application.application_status == status)
        .count()
}
