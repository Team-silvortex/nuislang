use super::{
    build_elf_amd64_materialization_preview, build_elf_amd64_merged_image, ElfAmd64ImageObject,
    ELF_AMD64_MATERIALIZATION_PREVIEW_CONTRACT,
};
use crate::{
    final_executable_elf_layout_report::ElfAmd64PlacementBindingReport,
    final_executable_elf_materialization_report::{
        ElfAmd64MaterializationPreviewReport, ElfAmd64PatchSpanPreview,
    },
    final_executable_elf_relocation_report::{
        ElfAmd64RelocationApplication, ElfAmd64RelocationApplicationReport,
    },
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
};

#[path = "final_executable_elf_platform.rs"]
pub(crate) mod platform;

pub(crate) const ELF_AMD64_PATCH_APPLICATION_CONTRACT: &str =
    "nuis-nsld-elf-amd64-patch-application-v1";

#[derive(Debug)]
pub(crate) struct ElfAmd64AppliedImage {
    pub(crate) bytes: Vec<u8>,
    pub(crate) report: ElfAmd64PatchApplicationReport,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64AppliedPatchAudit {
    pub(crate) relocation_id: String,
    pub(crate) relocation_kind: String,
    pub(crate) source_file_offset: usize,
    pub(crate) source_image_offset: usize,
    pub(crate) width_bytes: usize,
    pub(crate) source_bytes_hash: String,
    pub(crate) encoded_bytes_hash: String,
    pub(crate) post_write_bytes_hash: String,
    pub(crate) preview_audit_hash: String,
    pub(crate) write_audit_hash: String,
    pub(crate) status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64PatchApplicationReport {
    pub(crate) contract: &'static str,
    pub(crate) status: String,
    pub(crate) application_ledger_hash: String,
    pub(crate) placement_plan_hash: String,
    pub(crate) relocation_plan_hash: String,
    pub(crate) materialization_plan_hash: String,
    pub(crate) source_file_image_hash: String,
    pub(crate) source_memory_image_hash: String,
    pub(crate) applied_file_image_hash: String,
    pub(crate) applied_memory_image_hash: String,
    pub(crate) file_span_bytes: usize,
    pub(crate) memory_span_bytes: usize,
    pub(crate) expected_patch_count: usize,
    pub(crate) applied_patch_count: usize,
    pub(crate) deferred_patch_count: usize,
    pub(crate) no_op_count: usize,
    pub(crate) write_once_span_count: usize,
    pub(crate) patches: Vec<ElfAmd64AppliedPatchAudit>,
}

impl ElfAmd64PatchApplicationReport {
    pub(crate) fn canonical_ledger(&self) -> String {
        let mut out = String::new();
        append_text(&mut out, self.contract);
        append_text(&mut out, &self.status);
        append_text(&mut out, &self.placement_plan_hash);
        append_text(&mut out, &self.relocation_plan_hash);
        append_text(&mut out, &self.materialization_plan_hash);
        append_text(&mut out, &self.source_file_image_hash);
        append_text(&mut out, &self.source_memory_image_hash);
        append_text(&mut out, &self.applied_file_image_hash);
        append_text(&mut out, &self.applied_memory_image_hash);
        writeln!(
            out,
            "spans={}|{}|{}|{}|{}|{}|{}",
            self.file_span_bytes,
            self.memory_span_bytes,
            self.expected_patch_count,
            self.applied_patch_count,
            self.deferred_patch_count,
            self.no_op_count,
            self.write_once_span_count
        )
        .unwrap();
        for patch in &self.patches {
            append_text(&mut out, &patch.relocation_id);
            append_text(&mut out, &patch.relocation_kind);
            append_text(&mut out, &patch.source_bytes_hash);
            append_text(&mut out, &patch.encoded_bytes_hash);
            append_text(&mut out, &patch.post_write_bytes_hash);
            append_text(&mut out, &patch.preview_audit_hash);
            append_text(&mut out, &patch.write_audit_hash);
            append_text(&mut out, &patch.status);
            writeln!(
                out,
                "patch={}|{}|{}",
                patch.source_file_offset, patch.source_image_offset, patch.width_bytes
            )
            .unwrap();
        }
        out
    }
}

pub(crate) fn apply_elf_amd64_patch_previews(
    objects: &[ElfAmd64ImageObject<'_>],
    placement: &ElfAmd64PlacementBindingReport,
    relocations: &ElfAmd64RelocationApplicationReport,
    preview: &ElfAmd64MaterializationPreviewReport,
) -> Result<ElfAmd64AppliedImage, String> {
    validate_preview_envelope(placement, relocations, preview)?;
    let rebuilt_preview = build_elf_amd64_materialization_preview(objects, placement, relocations)?;
    if rebuilt_preview != *preview {
        return Err("ELF patch application materialization preview drift".to_owned());
    }
    let image = build_elf_amd64_merged_image(objects, placement)?;
    validate_source_image(&image, placement, preview)?;

    let direct = relocations
        .applications
        .iter()
        .filter(|application| application.application_status == "planned-direct")
        .map(|application| (application.relocation_id.as_str(), application))
        .collect::<BTreeMap<_, _>>();
    if direct.len() != relocations.direct_preview_count {
        return Err("ELF patch application contains duplicate direct relocation ids".to_owned());
    }

    let original = image.bytes;
    let mut applied = original.clone();
    let mut written = vec![false; applied.len()];
    let mut seen = BTreeSet::new();
    let mut patch_audits = Vec::with_capacity(preview.patches.len());
    for patch in &preview.patches {
        if !seen.insert(patch.relocation_id.as_str()) {
            return Err(format!(
                "ELF patch application repeats preview `{}`",
                patch.relocation_id
            ));
        }
        let application = direct
            .get(patch.relocation_id.as_str())
            .copied()
            .ok_or_else(|| {
                format!(
                    "ELF patch application has no planned direct relocation `{}`",
                    patch.relocation_id
                )
            })?;
        validate_patch_identity(application, patch)?;
        validate_patch_hashes(patch)?;
        checked_range(
            patch.source_file_offset,
            patch.width_bytes,
            preview.file_span_bytes,
            &patch.relocation_id,
        )?;
        let range = checked_range(
            patch.source_image_offset,
            patch.width_bytes,
            original.len(),
            &patch.relocation_id,
        )?;
        if original[range.clone()] != patch.source_bytes {
            return Err(format!(
                "ELF patch `{}` source image drift",
                patch.relocation_id
            ));
        }
        apply_write_once(
            &mut applied,
            &mut written,
            patch.source_image_offset,
            &patch.source_bytes,
            &patch.encoded_bytes,
            &patch.relocation_id,
        )?;
        let post_write_bytes_hash = crate::fnv1a64_hex(&applied[range]);
        if post_write_bytes_hash != patch.encoded_bytes_hash {
            return Err(format!(
                "ELF patch `{}` post-write hash drift",
                patch.relocation_id
            ));
        }
        let write_audit_hash = write_audit_hash(preview, patch, &post_write_bytes_hash);
        patch_audits.push(ElfAmd64AppliedPatchAudit {
            relocation_id: patch.relocation_id.clone(),
            relocation_kind: patch.relocation_kind.clone(),
            source_file_offset: patch.source_file_offset,
            source_image_offset: patch.source_image_offset,
            width_bytes: patch.width_bytes,
            source_bytes_hash: patch.source_bytes_hash.clone(),
            encoded_bytes_hash: patch.encoded_bytes_hash.clone(),
            post_write_bytes_hash,
            preview_audit_hash: patch.audit_hash.clone(),
            write_audit_hash,
            status: "applied-write-once".to_owned(),
        });
    }
    if seen.len() != direct.len() {
        return Err(format!(
            "ELF patch application coverage drift: planned={}, applied={}",
            direct.len(),
            seen.len()
        ));
    }

    let applied_file_image = applied
        .get(..preview.file_span_bytes)
        .ok_or_else(|| "ELF applied file span exceeds memory image".to_owned())?;
    let status = if preview.deferred_patch_count == 0 {
        "direct-patches-applied"
    } else {
        "direct-patches-applied-with-platform-structure-boundary"
    };
    let mut report = ElfAmd64PatchApplicationReport {
        contract: ELF_AMD64_PATCH_APPLICATION_CONTRACT,
        status: status.to_owned(),
        application_ledger_hash: String::new(),
        placement_plan_hash: placement.plan_hash.clone(),
        relocation_plan_hash: relocations.plan_hash.clone(),
        materialization_plan_hash: preview.plan_hash.clone(),
        source_file_image_hash: preview.file_image_hash.clone(),
        source_memory_image_hash: preview.memory_image_hash.clone(),
        applied_file_image_hash: crate::fnv1a64_hex(applied_file_image),
        applied_memory_image_hash: crate::fnv1a64_hex(&applied),
        file_span_bytes: preview.file_span_bytes,
        memory_span_bytes: preview.memory_span_bytes,
        expected_patch_count: preview.planned_direct_count,
        applied_patch_count: patch_audits.len(),
        deferred_patch_count: preview.deferred_patch_count,
        no_op_count: preview.no_op_count,
        write_once_span_count: patch_audits.len(),
        patches: patch_audits,
    };
    report.application_ledger_hash = crate::fnv1a64_hex(report.canonical_ledger().as_bytes());
    Ok(ElfAmd64AppliedImage {
        bytes: applied,
        report,
    })
}

fn validate_preview_envelope(
    placement: &ElfAmd64PlacementBindingReport,
    relocations: &ElfAmd64RelocationApplicationReport,
    preview: &ElfAmd64MaterializationPreviewReport,
) -> Result<(), String> {
    if preview.contract != ELF_AMD64_MATERIALIZATION_PREVIEW_CONTRACT {
        return Err(format!(
            "ELF patch application rejects preview contract `{}`",
            preview.contract
        ));
    }
    if preview.plan_hash != crate::fnv1a64_hex(preview.canonical_plan().as_bytes()) {
        return Err("ELF patch application preview plan hash drift".to_owned());
    }
    if preview.placement_plan_hash != placement.plan_hash
        || preview.relocation_plan_hash != relocations.plan_hash
        || relocations.placement_plan_hash != placement.plan_hash
    {
        return Err("ELF patch application upstream plan hash drift".to_owned());
    }
    if preview.file_span_bytes != placement.file_span_bytes
        || preview.memory_span_bytes != placement.memory_span_bytes
        || preview.planned_direct_count != relocations.direct_preview_count
        || preview.previewed_patch_count != preview.patches.len()
        || preview.previewed_patch_count != preview.planned_direct_count
        || preview.deferred_patch_count != relocations.platform_structure_count
        || preview.no_op_count != relocations.no_op_count
    {
        return Err("ELF patch application preview count/span drift".to_owned());
    }
    let expected_status = if preview.deferred_patch_count == 0 {
        "preview-ready"
    } else {
        "preview-ready-with-platform-structure-boundary"
    };
    if preview.status != expected_status {
        return Err("ELF patch application preview status drift".to_owned());
    }
    let expected_ids = relocations
        .applications
        .iter()
        .filter(|application| application.application_status == "planned-direct")
        .map(|application| application.relocation_id.as_str())
        .collect::<Vec<_>>();
    let preview_ids = preview
        .patches
        .iter()
        .map(|patch| patch.relocation_id.as_str())
        .collect::<Vec<_>>();
    if preview_ids != expected_ids {
        return Err("ELF patch application preview order drift".to_owned());
    }
    Ok(())
}

fn validate_source_image(
    image: &super::ElfAmd64MergedImage,
    placement: &ElfAmd64PlacementBindingReport,
    preview: &ElfAmd64MaterializationPreviewReport,
) -> Result<(), String> {
    let file_image = image
        .bytes
        .get(..placement.file_span_bytes)
        .ok_or_else(|| "ELF source file span exceeds memory image".to_owned())?;
    if image.bytes.len() != preview.memory_span_bytes
        || image.copied_bytes != preview.copied_bytes
        || image.zero_fill_bytes != preview.zero_fill_bytes
        || image.object_audits != preview.object_audits
        || image.section_audits != preview.section_audits
        || image.merged_section_audits != preview.merged_section_audits
        || crate::fnv1a64_hex(file_image) != preview.file_image_hash
        || crate::fnv1a64_hex(&image.bytes) != preview.memory_image_hash
    {
        return Err("ELF patch application source image drift".to_owned());
    }
    Ok(())
}

fn validate_patch_identity(
    application: &ElfAmd64RelocationApplication,
    patch: &ElfAmd64PatchSpanPreview,
) -> Result<(), String> {
    if application.relocation_kind != patch.relocation_kind
        || application.source_file_offset != patch.source_file_offset
        || application.source_image_offset != patch.source_image_offset
        || application.width_bytes != patch.width_bytes
        || application.encoded_bytes != patch.encoded_bytes
        || patch.status != "write-once-preview"
    {
        return Err(format!(
            "ELF patch `{}` relocation identity drift",
            patch.relocation_id
        ));
    }
    Ok(())
}

fn validate_patch_hashes(patch: &ElfAmd64PatchSpanPreview) -> Result<(), String> {
    if patch.source_bytes.len() != patch.width_bytes
        || patch.encoded_bytes.len() != patch.width_bytes
    {
        return Err(format!(
            "ELF patch `{}` byte width drift",
            patch.relocation_id
        ));
    }
    if crate::fnv1a64_hex(&patch.source_bytes) != patch.source_bytes_hash {
        return Err(format!(
            "ELF patch `{}` source byte hash drift",
            patch.relocation_id
        ));
    }
    if crate::fnv1a64_hex(&patch.encoded_bytes) != patch.encoded_bytes_hash {
        return Err(format!(
            "ELF patch `{}` encoded byte hash drift",
            patch.relocation_id
        ));
    }
    Ok(())
}

fn apply_write_once(
    image: &mut [u8],
    written: &mut [bool],
    offset: usize,
    source: &[u8],
    encoded: &[u8],
    relocation_id: &str,
) -> Result<(), String> {
    if image.len() != written.len() || source.len() != encoded.len() {
        return Err(format!(
            "ELF patch `{relocation_id}` write buffer shape drift"
        ));
    }
    let range = checked_range(offset, encoded.len(), image.len(), relocation_id)?;
    if written[range.clone()].iter().any(|occupied| *occupied) {
        return Err(format!(
            "ELF patch `{relocation_id}` overlaps a previously applied patch"
        ));
    }
    if image[range.clone()] != *source {
        return Err(format!("ELF patch `{relocation_id}` write source drift"));
    }
    image[range.clone()].copy_from_slice(encoded);
    written[range].fill(true);
    Ok(())
}

fn checked_range(
    offset: usize,
    size: usize,
    limit: usize,
    relocation_id: &str,
) -> Result<std::ops::Range<usize>, String> {
    let end = offset
        .checked_add(size)
        .ok_or_else(|| format!("ELF patch `{relocation_id}` application range overflows"))?;
    if end > limit {
        return Err(format!(
            "ELF patch `{relocation_id}` application range {offset}..{end} exceeds image {limit}"
        ));
    }
    Ok(offset..end)
}

fn write_audit_hash(
    preview: &ElfAmd64MaterializationPreviewReport,
    patch: &ElfAmd64PatchSpanPreview,
    post_write_hash: &str,
) -> String {
    let mut canonical = String::new();
    append_text(&mut canonical, ELF_AMD64_PATCH_APPLICATION_CONTRACT);
    append_text(&mut canonical, &preview.plan_hash);
    append_text(&mut canonical, &patch.relocation_id);
    append_text(&mut canonical, &patch.audit_hash);
    append_text(&mut canonical, &patch.source_bytes_hash);
    append_text(&mut canonical, &patch.encoded_bytes_hash);
    append_text(&mut canonical, post_write_hash);
    writeln!(
        canonical,
        "span={}|{}|{}",
        patch.source_file_offset, patch.source_image_offset, patch.width_bytes
    )
    .unwrap();
    crate::fnv1a64_hex(canonical.as_bytes())
}

fn append_text(out: &mut String, value: &str) {
    writeln!(out, "text:{}:{value}", value.len()).unwrap();
}

#[cfg(test)]
#[path = "final_executable_elf_application_tests.rs"]
mod tests;
