use crate::{
    final_executable_elf_input::{parse_elf64_amd64_object_linkage, ParsedElfObjectLinkage},
    final_executable_elf_layout::ELF_AMD64_PLACEMENT_BINDING_CONTRACT,
    final_executable_elf_layout_report::{
        ElfAmd64MergedSectionPlan, ElfAmd64PlacementBindingReport,
    },
    final_executable_elf_materialization_report::{
        ElfAmd64MaterializationPreviewReport, ElfAmd64MergedSectionAudit, ElfAmd64ObjectInputAudit,
        ElfAmd64PatchSpanPreview, ElfAmd64SectionMaterializationAudit,
    },
    final_executable_elf_relocation::ELF_AMD64_RELOCATION_APPLICATION_CONTRACT,
    final_executable_elf_relocation_report::{
        ElfAmd64RelocationApplication, ElfAmd64RelocationApplicationReport,
    },
};
use std::{collections::BTreeMap, fmt::Write as _};

pub(crate) const ELF_AMD64_MATERIALIZATION_PREVIEW_CONTRACT: &str =
    "nuis-nsld-elf-amd64-materialization-preview-v1";

pub(crate) struct ElfAmd64ImageObject<'a> {
    pub(crate) object_id: &'a str,
    pub(crate) role: &'a str,
    pub(crate) bytes: &'a [u8],
    pub(crate) planned_size_bytes: usize,
    pub(crate) planned_source_hash: &'a str,
    pub(crate) linkage: &'a ParsedElfObjectLinkage,
}

pub(crate) struct ElfAmd64MergedImage {
    pub(crate) bytes: Vec<u8>,
    pub(crate) copied_bytes: usize,
    pub(crate) zero_fill_bytes: usize,
    pub(crate) object_audits: Vec<ElfAmd64ObjectInputAudit>,
    pub(crate) section_audits: Vec<ElfAmd64SectionMaterializationAudit>,
    pub(crate) merged_section_audits: Vec<ElfAmd64MergedSectionAudit>,
}

pub(crate) fn build_elf_amd64_materialization_preview(
    objects: &[ElfAmd64ImageObject<'_>],
    placement: &ElfAmd64PlacementBindingReport,
    relocations: &ElfAmd64RelocationApplicationReport,
) -> Result<ElfAmd64MaterializationPreviewReport, String> {
    validate_report_envelope(placement, relocations)?;
    let image = build_elf_amd64_merged_image(objects, placement)?;
    let source_object_set_hash = object_set_hash(&image.object_audits);
    let file_image = image
        .bytes
        .get(..placement.file_span_bytes)
        .ok_or_else(|| "ELF materialization file span exceeds memory image".to_owned())?;
    let file_image_hash = crate::fnv1a64_hex(file_image);
    let memory_image_hash = crate::fnv1a64_hex(&image.bytes);
    let patches = build_patch_spans(&image.bytes, placement, relocations)?;
    if crate::fnv1a64_hex(file_image) != file_image_hash
        || crate::fnv1a64_hex(&image.bytes) != memory_image_hash
    {
        return Err("ELF patch-span preview mutated the merged source image".to_owned());
    }
    let status = if relocations.platform_structure_count == 0 {
        "preview-ready"
    } else {
        "preview-ready-with-platform-structure-boundary"
    };
    let zero_fill_section_count = image
        .merged_section_audits
        .iter()
        .filter(|audit| audit.zero_fill)
        .count();
    let mut report = ElfAmd64MaterializationPreviewReport {
        contract: ELF_AMD64_MATERIALIZATION_PREVIEW_CONTRACT,
        status: status.to_owned(),
        plan_hash: String::new(),
        placement_plan_hash: placement.plan_hash.clone(),
        relocation_plan_hash: relocations.plan_hash.clone(),
        source_object_set_hash,
        file_image_hash,
        memory_image_hash,
        file_span_bytes: placement.file_span_bytes,
        memory_span_bytes: placement.memory_span_bytes,
        copied_bytes: image.copied_bytes,
        zero_fill_bytes: image.zero_fill_bytes,
        input_object_count: image.object_audits.len(),
        section_audit_count: image.section_audits.len(),
        merged_section_audit_count: image.merged_section_audits.len(),
        zero_fill_section_count,
        planned_direct_count: relocations.direct_preview_count,
        previewed_patch_count: patches.len(),
        deferred_patch_count: relocations.platform_structure_count,
        no_op_count: relocations.no_op_count,
        object_audits: image.object_audits,
        section_audits: image.section_audits,
        merged_section_audits: image.merged_section_audits,
        patches,
    };
    report.plan_hash = crate::fnv1a64_hex(report.canonical_plan().as_bytes());
    Ok(report)
}

pub(crate) fn build_elf_amd64_merged_image(
    objects: &[ElfAmd64ImageObject<'_>],
    placement: &ElfAmd64PlacementBindingReport,
) -> Result<ElfAmd64MergedImage, String> {
    validate_placement_report(placement)?;
    if placement.file_span_bytes > placement.memory_span_bytes {
        return Err("ELF placement file span exceeds memory span".to_owned());
    }
    let (object_map, object_audits) = validate_image_objects(objects)?;
    let mut bytes = vec![0u8; placement.memory_span_bytes];
    let mut occupied = vec![false; placement.memory_span_bytes];
    let mut section_audits = Vec::with_capacity(placement.section_placements.len());
    let mut copied_bytes = 0usize;

    for contribution in &placement.section_placements {
        let object = object_map
            .get(contribution.object_id.as_str())
            .copied()
            .ok_or_else(|| {
                format!(
                    "ELF placement references missing image object `{}`",
                    contribution.object_id
                )
            })?;
        if object.role != contribution.object_role {
            return Err(format!(
                "ELF image object `{}` role drift: input={}, placement={}",
                object.object_id, object.role, contribution.object_role
            ));
        }
        let section = object
            .linkage
            .sections
            .iter()
            .find(|section| section.index == contribution.input_section_index)
            .ok_or_else(|| {
                format!(
                    "ELF image object `{}` has no section index {}",
                    object.object_id, contribution.input_section_index
                )
            })?;
        if section.name != contribution.input_section_name
            || section.size != contribution.size_bytes
            || section.zero_fill != contribution.zero_fill
        {
            return Err(format!(
                "ELF image object `{}` section {} placement identity drift",
                object.object_id, section.index
            ));
        }
        let merged = merged_section(placement, &contribution.output_section_id)?;
        validate_contribution_coordinates(contribution, merged, placement)?;
        let output_range = checked_range(
            contribution.image_offset,
            contribution.size_bytes,
            bytes.len(),
            "section output",
        )?;
        claim_span(
            &mut occupied,
            output_range.clone(),
            &format!("section {}:{}", object.object_id, section.index),
        )?;

        let (source_bytes_hash, status) = if contribution.zero_fill {
            if section.payload_offset.is_some() || contribution.file_offset.is_some() {
                return Err(format!(
                    "ELF zero-fill section `{}` unexpectedly owns file bytes",
                    section.name
                ));
            }
            ensure_zero(&bytes[output_range.clone()], &section.name)?;
            (None, "verified-zero-fill")
        } else {
            let source_offset = section.payload_offset.ok_or_else(|| {
                format!(
                    "ELF file-backed section `{}` has no payload offset",
                    section.name
                )
            })?;
            let source_range = checked_range(
                source_offset,
                section.size,
                object.bytes.len(),
                "section source",
            )?;
            let source = &object.bytes[source_range];
            bytes[output_range.clone()].copy_from_slice(source);
            copied_bytes = copied_bytes
                .checked_add(source.len())
                .ok_or_else(|| "ELF copied-byte count overflows".to_owned())?;
            (Some(crate::fnv1a64_hex(source)), "copied-file-backed")
        };
        let materialized_bytes_hash = crate::fnv1a64_hex(&bytes[output_range]);
        section_audits.push(ElfAmd64SectionMaterializationAudit {
            object_id: object.object_id.to_owned(),
            object_role: object.role.to_owned(),
            input_section_index: section.index,
            input_section_name: section.name.clone(),
            output_section_id: contribution.output_section_id.clone(),
            source_payload_offset: section.payload_offset,
            output_file_offset: contribution.file_offset,
            output_image_offset: contribution.image_offset,
            size_bytes: contribution.size_bytes,
            zero_fill: contribution.zero_fill,
            source_bytes_hash,
            materialized_bytes_hash,
            status: status.to_owned(),
        });
    }
    validate_common_allocations(&bytes, placement, &mut occupied)?;
    let (merged_section_audits, zero_fill_bytes) = audit_merged_sections(&bytes, placement)?;
    Ok(ElfAmd64MergedImage {
        bytes,
        copied_bytes,
        zero_fill_bytes,
        object_audits,
        section_audits,
        merged_section_audits,
    })
}

fn validate_report_envelope(
    placement: &ElfAmd64PlacementBindingReport,
    relocations: &ElfAmd64RelocationApplicationReport,
) -> Result<(), String> {
    validate_placement_report(placement)?;
    if relocations.contract != ELF_AMD64_RELOCATION_APPLICATION_CONTRACT {
        return Err(format!(
            "ELF materialization received relocation contract `{}`",
            relocations.contract
        ));
    }
    let actual_relocation_hash = crate::fnv1a64_hex(relocations.canonical_plan().as_bytes());
    if relocations.plan_hash != actual_relocation_hash {
        return Err(format!(
            "ELF materialization relocation hash mismatch: declared={}, actual={actual_relocation_hash}",
            relocations.plan_hash
        ));
    }
    if relocations.placement_plan_hash != placement.plan_hash {
        return Err("ELF materialization placement/relocation plan drift".to_owned());
    }
    if relocations.relocation_count != relocations.applications.len()
        || relocations.direct_preview_count != count_status(relocations, "planned-direct")
        || relocations.platform_structure_count
            != count_status(relocations, "planned-platform-structure")
        || relocations.no_op_count != count_status(relocations, "no-op")
    {
        return Err("ELF materialization relocation count drift".to_owned());
    }
    let classified_count = relocations
        .direct_preview_count
        .checked_add(relocations.platform_structure_count)
        .and_then(|count| count.checked_add(relocations.no_op_count))
        .ok_or_else(|| "ELF materialization relocation class count overflows".to_owned())?;
    if classified_count != relocations.relocation_count {
        return Err("ELF materialization contains an unclassified relocation status".to_owned());
    }
    let expected_status = if relocations.platform_structure_count == 0 {
        "ready-for-byte-preview"
    } else {
        "preview-ready-with-platform-structure-boundary"
    };
    if relocations.status != expected_status {
        return Err(format!(
            "ELF materialization relocation status drift: expected={expected_status}, actual={}",
            relocations.status
        ));
    }
    Ok(())
}

fn validate_placement_report(placement: &ElfAmd64PlacementBindingReport) -> Result<(), String> {
    if placement.contract != ELF_AMD64_PLACEMENT_BINDING_CONTRACT {
        return Err(format!(
            "ELF materialization received placement contract `{}`",
            placement.contract
        ));
    }
    let actual = crate::fnv1a64_hex(placement.canonical_plan().as_bytes());
    if placement.plan_hash != actual {
        return Err(format!(
            "ELF materialization placement hash mismatch: declared={}, actual={actual}",
            placement.plan_hash
        ));
    }
    Ok(())
}

fn validate_image_objects<'a>(
    objects: &'a [ElfAmd64ImageObject<'a>],
) -> Result<
    (
        BTreeMap<&'a str, &'a ElfAmd64ImageObject<'a>>,
        Vec<ElfAmd64ObjectInputAudit>,
    ),
    String,
> {
    let mut object_map = BTreeMap::new();
    let mut audits = Vec::with_capacity(objects.len());
    for object in objects {
        if object_map.insert(object.object_id, object).is_some() {
            return Err(format!(
                "ELF materialization contains duplicate object id `{}`",
                object.object_id
            ));
        }
        let source_hash = crate::fnv1a64_hex(object.bytes);
        if object.planned_size_bytes != object.bytes.len() {
            return Err(format!(
                "ELF materialization object `{}` size drift: planned={}, actual={}",
                object.object_id,
                object.planned_size_bytes,
                object.bytes.len()
            ));
        }
        if object.planned_source_hash != source_hash {
            return Err(format!(
                "ELF materialization object `{}` source hash drift: planned={}, actual={source_hash}",
                object.object_id, object.planned_source_hash
            ));
        }
        let reparsed = parse_elf64_amd64_object_linkage(object.bytes).map_err(|error| {
            format!(
                "ELF materialization object `{}` failed source reparse: {error}",
                object.object_id
            )
        })?;
        if reparsed != *object.linkage {
            return Err(format!(
                "ELF materialization object `{}` source/linkage drift",
                object.object_id
            ));
        }
        audits.push(ElfAmd64ObjectInputAudit {
            object_id: object.object_id.to_owned(),
            object_role: object.role.to_owned(),
            planned_size_bytes: object.planned_size_bytes,
            size_bytes: object.bytes.len(),
            planned_source_hash: object.planned_source_hash.to_owned(),
            source_hash,
            status: "verified-plan-bound".to_owned(),
        });
    }
    audits.sort_by(|lhs, rhs| {
        object_role_rank(&lhs.object_role)
            .cmp(&object_role_rank(&rhs.object_role))
            .then(lhs.object_role.cmp(&rhs.object_role))
            .then(lhs.object_id.cmp(&rhs.object_id))
    });
    Ok((object_map, audits))
}

fn validate_contribution_coordinates(
    contribution: &crate::final_executable_elf_layout_report::ElfAmd64SectionPlacement,
    merged: &ElfAmd64MergedSectionPlan,
    placement: &ElfAmd64PlacementBindingReport,
) -> Result<(), String> {
    let expected_image = merged
        .image_offset
        .checked_add(contribution.output_section_offset)
        .ok_or_else(|| "ELF contribution image offset overflows".to_owned())?;
    let image_offset = u64::try_from(contribution.image_offset)
        .map_err(|_| "ELF contribution image offset exceeds u64".to_owned())?;
    if expected_image != contribution.image_offset
        || contribution.virtual_address
            != placement
                .image_base
                .checked_add(image_offset)
                .ok_or_else(|| "ELF contribution virtual address overflows".to_owned())?
    {
        return Err(format!(
            "ELF section placement `{}` coordinate drift",
            contribution.input_section_name
        ));
    }
    let contribution_end = contribution
        .output_section_offset
        .checked_add(contribution.size_bytes)
        .ok_or_else(|| "ELF contribution merged-section span overflows".to_owned())?;
    if contribution_end > merged.size_bytes || contribution.zero_fill != merged.zero_fill {
        return Err(format!(
            "ELF section placement `{}` exceeds merged section `{}`",
            contribution.input_section_name, merged.section_id
        ));
    }
    let expected_file = (!contribution.zero_fill).then_some(contribution.image_offset);
    if contribution.file_offset != expected_file {
        return Err(format!(
            "ELF section placement `{}` file/image coordinate drift",
            contribution.input_section_name
        ));
    }
    if let Some(file_offset) = contribution.file_offset {
        checked_range(
            file_offset,
            contribution.size_bytes,
            placement.file_span_bytes,
            "section file output",
        )?;
    }
    Ok(())
}

fn validate_common_allocations(
    bytes: &[u8],
    placement: &ElfAmd64PlacementBindingReport,
    occupied: &mut [bool],
) -> Result<(), String> {
    for allocation in &placement.common_allocations {
        let merged = merged_section(placement, &allocation.output_section_id)?;
        if !merged.zero_fill {
            return Err(format!(
                "ELF common allocation `{}` targets file-backed section",
                allocation.symbol
            ));
        }
        let expected = merged
            .image_offset
            .checked_add(allocation.output_section_offset)
            .ok_or_else(|| "ELF common allocation offset overflows".to_owned())?;
        if expected != allocation.image_offset {
            return Err(format!(
                "ELF common allocation `{}` coordinate drift",
                allocation.symbol
            ));
        }
        let range = checked_range(
            allocation.image_offset,
            allocation.size_bytes,
            bytes.len(),
            "common allocation",
        )?;
        claim_span(occupied, range.clone(), &allocation.symbol)?;
        ensure_zero(&bytes[range], &allocation.symbol)?;
    }
    Ok(())
}

fn audit_merged_sections(
    bytes: &[u8],
    placement: &ElfAmd64PlacementBindingReport,
) -> Result<(Vec<ElfAmd64MergedSectionAudit>, usize), String> {
    let mut occupied = vec![false; bytes.len()];
    let mut audits = Vec::with_capacity(placement.merged_sections.len());
    let mut zero_fill_bytes = 0usize;
    for merged in &placement.merged_sections {
        let range = checked_range(
            merged.image_offset,
            merged.size_bytes,
            bytes.len(),
            "merged section",
        )?;
        claim_span(&mut occupied, range.clone(), &merged.section_id)?;
        let image_offset = u64::try_from(merged.image_offset)
            .map_err(|_| "ELF merged-section image offset exceeds u64".to_owned())?;
        let expected_virtual_address = placement
            .image_base
            .checked_add(image_offset)
            .ok_or_else(|| "ELF merged-section virtual address overflows".to_owned())?;
        if merged.virtual_address != expected_virtual_address {
            return Err(format!(
                "ELF merged section `{}` virtual-address drift",
                merged.section_id
            ));
        }
        let expected_file = (!merged.zero_fill).then_some(merged.image_offset);
        if merged.file_offset != expected_file {
            return Err(format!(
                "ELF merged section `{}` file/image coordinate drift",
                merged.section_id
            ));
        }
        if let Some(file_offset) = merged.file_offset {
            checked_range(
                file_offset,
                merged.size_bytes,
                placement.file_span_bytes,
                "merged section file span",
            )?;
        }
        let placement_count = placement
            .section_placements
            .iter()
            .filter(|item| item.output_section_id == merged.section_id)
            .count();
        let common_count = placement
            .common_allocations
            .iter()
            .filter(|item| item.output_section_id == merged.section_id)
            .count();
        if merged.contribution_count != placement_count + common_count {
            return Err(format!(
                "ELF merged section `{}` contribution count drift",
                merged.section_id
            ));
        }
        let status = if merged.zero_fill {
            ensure_zero(&bytes[range.clone()], &merged.section_id)?;
            zero_fill_bytes = zero_fill_bytes
                .checked_add(merged.size_bytes)
                .ok_or_else(|| "ELF zero-fill byte count overflows".to_owned())?;
            "verified-zero-fill"
        } else {
            "materialized-file-backed"
        };
        audits.push(ElfAmd64MergedSectionAudit {
            section_id: merged.section_id.clone(),
            class: merged.class.clone(),
            file_offset: merged.file_offset,
            image_offset: merged.image_offset,
            size_bytes: merged.size_bytes,
            contribution_count: merged.contribution_count,
            zero_fill: merged.zero_fill,
            materialized_bytes_hash: crate::fnv1a64_hex(&bytes[range]),
            status: status.to_owned(),
        });
    }
    Ok((audits, zero_fill_bytes))
}

fn build_patch_spans(
    image: &[u8],
    placement: &ElfAmd64PlacementBindingReport,
    relocations: &ElfAmd64RelocationApplicationReport,
) -> Result<Vec<ElfAmd64PatchSpanPreview>, String> {
    let mut seen_ids = BTreeMap::new();
    let mut occupied = vec![false; image.len()];
    let mut patches = Vec::with_capacity(relocations.direct_preview_count);
    for application in &relocations.applications {
        if seen_ids
            .insert(
                application.relocation_id.as_str(),
                application.application_status.as_str(),
            )
            .is_some()
        {
            return Err(format!(
                "ELF materialization repeats relocation id `{}`",
                application.relocation_id
            ));
        }
        if application.application_status != "planned-direct" {
            continue;
        }
        if application.encoded_bytes.len() != application.width_bytes {
            return Err(format!(
                "ELF direct relocation `{}` encoded width drift",
                application.relocation_id
            ));
        }
        if application.computed_value.is_none() || application.encoded_value.is_none() {
            return Err(format!(
                "ELF direct relocation `{}` has no computed encoding",
                application.relocation_id
            ));
        }
        let encoded_value = application.encoded_value.unwrap_or(0).to_le_bytes();
        let expected_bytes = encoded_value
            .get(..application.width_bytes)
            .ok_or_else(|| {
                format!(
                    "ELF direct relocation `{}` width exceeds its encoded value",
                    application.relocation_id
                )
            })?;
        if application.encoded_bytes != expected_bytes {
            return Err(format!(
                "ELF direct relocation `{}` encoded byte drift",
                application.relocation_id
            ));
        }
        validate_patch_source(application, placement)?;
        checked_range(
            application.source_file_offset,
            application.width_bytes,
            placement.file_span_bytes,
            "patch file span",
        )?;
        let range = checked_range(
            application.source_image_offset,
            application.width_bytes,
            image.len(),
            "patch image span",
        )?;
        claim_span(&mut occupied, range.clone(), &application.relocation_id)?;
        let source_bytes = image[range].to_vec();
        let source_bytes_hash = crate::fnv1a64_hex(&source_bytes);
        let encoded_bytes_hash = crate::fnv1a64_hex(&application.encoded_bytes);
        let audit_hash = patch_audit_hash(application, &source_bytes_hash, &encoded_bytes_hash);
        patches.push(ElfAmd64PatchSpanPreview {
            relocation_id: application.relocation_id.clone(),
            relocation_kind: application.relocation_kind.clone(),
            source_file_offset: application.source_file_offset,
            source_image_offset: application.source_image_offset,
            width_bytes: application.width_bytes,
            source_bytes,
            encoded_bytes: application.encoded_bytes.clone(),
            source_bytes_hash,
            encoded_bytes_hash,
            audit_hash,
            status: "write-once-preview".to_owned(),
        });
    }
    if patches.len() != relocations.direct_preview_count {
        return Err(format!(
            "ELF patch preview coverage drift: planned={}, previewed={}",
            relocations.direct_preview_count,
            patches.len()
        ));
    }
    Ok(patches)
}

fn validate_patch_source(
    application: &ElfAmd64RelocationApplication,
    placement: &ElfAmd64PlacementBindingReport,
) -> Result<(), String> {
    let source = placement
        .section_placements
        .iter()
        .find(|item| {
            item.object_id == application.object_id
                && item.input_section_index == application.input_section_index
        })
        .ok_or_else(|| {
            format!(
                "ELF patch `{}` has no source placement",
                application.relocation_id
            )
        })?;
    let expected_image = source
        .image_offset
        .checked_add(application.source_offset)
        .ok_or_else(|| "ELF patch source image offset overflows".to_owned())?;
    let expected_file = source
        .file_offset
        .ok_or_else(|| {
            format!(
                "ELF patch `{}` source is not file-backed",
                application.relocation_id
            )
        })?
        .checked_add(application.source_offset)
        .ok_or_else(|| "ELF patch source file offset overflows".to_owned())?;
    let source_end = application
        .source_offset
        .checked_add(application.width_bytes)
        .ok_or_else(|| "ELF patch source span overflows".to_owned())?;
    if source_end > source.size_bytes
        || source.output_section_id != application.source_section_id
        || expected_image != application.source_image_offset
        || expected_file != application.source_file_offset
    {
        return Err(format!(
            "ELF patch `{}` source placement drift",
            application.relocation_id
        ));
    }
    Ok(())
}

fn patch_audit_hash(
    application: &ElfAmd64RelocationApplication,
    source_hash: &str,
    encoded_hash: &str,
) -> String {
    let mut canonical = String::new();
    append_text(&mut canonical, ELF_AMD64_MATERIALIZATION_PREVIEW_CONTRACT);
    append_text(&mut canonical, &application.relocation_id);
    append_text(&mut canonical, &application.relocation_kind);
    append_text(&mut canonical, &application.source_section_id);
    append_text(&mut canonical, source_hash);
    append_text(&mut canonical, encoded_hash);
    writeln!(
        canonical,
        "facts={}|{}|{}|{}|{}|{}",
        application.source_file_offset,
        application.source_image_offset,
        application.width_bytes,
        application.relocation_type,
        application.addend,
        application.computed_value.unwrap_or(0)
    )
    .unwrap();
    crate::fnv1a64_hex(canonical.as_bytes())
}

fn object_set_hash(audits: &[ElfAmd64ObjectInputAudit]) -> String {
    let mut canonical = String::new();
    for audit in audits {
        append_text(&mut canonical, &audit.object_id);
        append_text(&mut canonical, &audit.object_role);
        append_text(&mut canonical, &audit.planned_source_hash);
        append_text(&mut canonical, &audit.source_hash);
        append_text(&mut canonical, &audit.status);
        writeln!(
            canonical,
            "sizes={}|{}",
            audit.planned_size_bytes, audit.size_bytes
        )
        .unwrap();
    }
    crate::fnv1a64_hex(canonical.as_bytes())
}

fn merged_section<'a>(
    placement: &'a ElfAmd64PlacementBindingReport,
    section_id: &str,
) -> Result<&'a ElfAmd64MergedSectionPlan, String> {
    placement
        .merged_sections
        .iter()
        .find(|section| section.section_id == section_id)
        .ok_or_else(|| format!("ELF placement references missing merged section `{section_id}`"))
}

fn count_status(relocations: &ElfAmd64RelocationApplicationReport, status: &str) -> usize {
    relocations
        .applications
        .iter()
        .filter(|application| application.application_status == status)
        .count()
}

fn claim_span(
    occupied: &mut [bool],
    range: std::ops::Range<usize>,
    label: &str,
) -> Result<(), String> {
    if occupied[range.clone()].iter().any(|claimed| *claimed) {
        return Err(format!(
            "ELF materialization span `{label}` overlaps an earlier span"
        ));
    }
    occupied[range].fill(true);
    Ok(())
}

fn ensure_zero(bytes: &[u8], label: &str) -> Result<(), String> {
    if bytes.iter().any(|byte| *byte != 0) {
        return Err(format!(
            "ELF zero-fill span `{label}` contains materialized nonzero bytes"
        ));
    }
    Ok(())
}

fn checked_range(
    offset: usize,
    size: usize,
    limit: usize,
    label: &str,
) -> Result<std::ops::Range<usize>, String> {
    let end = offset
        .checked_add(size)
        .ok_or_else(|| format!("ELF {label} range overflows"))?;
    if end > limit {
        return Err(format!(
            "ELF {label} range {offset}..{end} exceeds limit {limit}"
        ));
    }
    Ok(offset..end)
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
#[path = "final_executable_elf_materialization_tests.rs"]
mod tests;
