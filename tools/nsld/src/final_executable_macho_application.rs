use crate::{
    final_executable_macho_materialization::{
        build_merged_section_image, materialization_patch_plan_hash, patch_audit_hash,
        MachOImageObject, MACHO_ARM64_MATERIALIZATION_PREVIEW_CONTRACT,
    },
    reports::{
        NsldMachOArm64AppliedPatchAudit, NsldMachOArm64MaterializationPreviewReport,
        NsldMachOArm64PatchApplicationReport, NsldMachOArm64PatchPreview,
        NsldMachOArm64RelocationApplication, NsldMachOArm64RelocationApplicationReport,
        NsldMachOPlacementBindingReport,
    },
};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

pub(crate) const MACHO_ARM64_PATCH_APPLICATION_CONTRACT: &str =
    "nuis-nsld-macho-arm64-patch-application-v1";

#[derive(Debug)]
pub(crate) struct MachOArm64AppliedImage {
    pub(crate) bytes: Vec<u8>,
    pub(crate) report: NsldMachOArm64PatchApplicationReport,
}

pub(crate) fn apply_macho_arm64_patch_previews(
    objects: &[MachOImageObject<'_>],
    placement: &NsldMachOPlacementBindingReport,
    relocations: &NsldMachOArm64RelocationApplicationReport,
    preview: &NsldMachOArm64MaterializationPreviewReport,
) -> Result<MachOArm64AppliedImage, String> {
    validate_report_envelope(placement, relocations, preview)?;
    let image = build_merged_section_image(objects, placement)?;
    validate_source_image(&image, preview)?;

    let direct = relocations
        .applications
        .iter()
        .filter(|item| item.application_status == "planned-direct")
        .map(|item| (item.relocation_id.as_str(), item))
        .collect::<BTreeMap<_, _>>();
    if direct.len() != relocations.ready_application_count {
        return Err("Mach-O patch application contains duplicate direct relocation ids".to_owned());
    }

    let original = image.bytes;
    let mut applied = original.clone();
    let mut written = vec![false; applied.len()];
    let mut seen = BTreeSet::new();
    let mut patch_audits = Vec::with_capacity(preview.patches.len());
    for patch in &preview.patches {
        if !seen.insert(patch.relocation_id.as_str()) {
            return Err(format!(
                "Mach-O patch application repeats preview `{}`",
                patch.relocation_id
            ));
        }
        let application = direct
            .get(patch.relocation_id.as_str())
            .copied()
            .ok_or_else(|| {
                format!(
                    "Mach-O patch application has no planned direct relocation `{}`",
                    patch.relocation_id
                )
            })?;
        validate_patch_identity(application, patch)?;
        let source = decode_canonical_hex(&patch.source_bytes_hex, "source", patch)?;
        let encoded = decode_canonical_hex(&patch.encoded_bytes_hex, "encoded", patch)?;
        if source.len() != patch.width_bytes || encoded.len() != patch.width_bytes {
            return Err(format!(
                "Mach-O patch `{}` byte width drift: declared={}, source={}, encoded={}",
                patch.relocation_id,
                patch.width_bytes,
                source.len(),
                encoded.len()
            ));
        }
        validate_patch_hashes(application, patch, &source, &encoded)?;
        let range = checked_range(
            patch.source_output_offset,
            patch.width_bytes,
            original.len(),
            &patch.relocation_id,
        )?;
        if original[range.clone()] != source {
            return Err(format!(
                "Mach-O patch `{}` source image drift",
                patch.relocation_id
            ));
        }
        apply_write_once(
            &mut applied,
            &mut written,
            patch.source_output_offset,
            &source,
            &encoded,
            &patch.relocation_id,
        )?;
        let post_write_bytes_hash = crate::fnv1a64_hex(&applied[range]);
        if post_write_bytes_hash != patch.encoded_bytes_hash {
            return Err(format!(
                "Mach-O patch `{}` post-write hash drift",
                patch.relocation_id
            ));
        }
        let write_audit_hash = write_audit_hash(patch, &post_write_bytes_hash);
        patch_audits.push(NsldMachOArm64AppliedPatchAudit {
            relocation_id: patch.relocation_id.clone(),
            source_output_offset: patch.source_output_offset,
            width_bytes: patch.width_bytes,
            source_bytes_hash: patch.source_bytes_hash.clone(),
            encoded_bytes_hash: patch.encoded_bytes_hash.clone(),
            post_write_bytes_hash,
            preview_audit_hash: patch.audit_hash.clone(),
            write_audit_hash,
        });
    }
    if seen.len() != direct.len() {
        return Err(format!(
            "Mach-O patch application coverage drift: planned={}, applied={}",
            direct.len(),
            seen.len()
        ));
    }

    let applied_image_hash = crate::fnv1a64_hex(&applied);
    let status = if preview.deferred_patch_count == 0 {
        "direct-patches-applied"
    } else {
        "direct-patches-applied-with-platform-structure-boundary"
    };
    let application_ledger_hash = application_ledger_hash(
        placement,
        relocations,
        preview,
        &applied_image_hash,
        status,
        &patch_audits,
    );
    Ok(MachOArm64AppliedImage {
        bytes: applied,
        report: NsldMachOArm64PatchApplicationReport {
            contract: MACHO_ARM64_PATCH_APPLICATION_CONTRACT.to_owned(),
            status: status.to_owned(),
            placement_plan_hash: placement.plan_hash.clone(),
            relocation_plan_hash: relocations.plan_hash.clone(),
            patch_plan_hash: preview.patch_plan_hash.clone(),
            original_image_hash: preview.image_hash.clone(),
            applied_image_hash,
            image_span_bytes: preview.image_span_bytes,
            expected_patch_count: preview.planned_direct_count,
            applied_patch_count: patch_audits.len(),
            deferred_patch_count: preview.deferred_patch_count,
            write_once_span_count: patch_audits.len(),
            application_ledger_hash,
            patches: patch_audits,
        },
    })
}

fn validate_report_envelope(
    placement: &NsldMachOPlacementBindingReport,
    relocations: &NsldMachOArm64RelocationApplicationReport,
    preview: &NsldMachOArm64MaterializationPreviewReport,
) -> Result<(), String> {
    if preview.contract != MACHO_ARM64_MATERIALIZATION_PREVIEW_CONTRACT {
        return Err(format!(
            "Mach-O patch application rejects preview contract `{}`",
            preview.contract
        ));
    }
    if relocations.placement_plan_hash != placement.plan_hash
        || preview.placement_plan_hash != placement.plan_hash
        || preview.relocation_plan_hash != relocations.plan_hash
    {
        return Err("Mach-O patch application plan hash drift".to_owned());
    }
    if preview.planned_direct_count != relocations.ready_application_count
        || preview.previewed_patch_count != preview.patches.len()
        || preview.previewed_patch_count != preview.planned_direct_count
        || preview.deferred_patch_count != relocations.platform_structure_count
        || preview.metadata_record_count != relocations.metadata_record_count
    {
        return Err("Mach-O patch application report count drift".to_owned());
    }
    let expected_status = if preview.deferred_patch_count == 0 {
        "preview-ready"
    } else {
        "preview-ready-with-platform-structure-boundary"
    };
    if preview.status != expected_status {
        return Err("Mach-O patch application preview status drift".to_owned());
    }
    let expected_patch_ids = relocations
        .applications
        .iter()
        .filter(|item| item.application_status == "planned-direct")
        .map(|item| item.relocation_id.as_str())
        .collect::<Vec<_>>();
    let preview_patch_ids = preview
        .patches
        .iter()
        .map(|item| item.relocation_id.as_str())
        .collect::<Vec<_>>();
    if preview_patch_ids != expected_patch_ids {
        return Err("Mach-O patch application preview order drift".to_owned());
    }
    let expected_plan_hash = materialization_patch_plan_hash(
        &placement.plan_hash,
        &relocations.plan_hash,
        &preview.image_hash,
        &preview.patches,
    );
    if preview.patch_plan_hash != expected_plan_hash {
        return Err("Mach-O patch application preview plan hash drift".to_owned());
    }
    Ok(())
}

fn validate_source_image(
    image: &crate::final_executable_macho_materialization::MergedSectionImage,
    preview: &NsldMachOArm64MaterializationPreviewReport,
) -> Result<(), String> {
    if image.bytes.len() != preview.image_span_bytes
        || image.copied_bytes != preview.copied_bytes
        || image.zero_fill_bytes != preview.zero_fill_bytes
        || image.section_audits != preview.section_audits
        || crate::fnv1a64_hex(&image.bytes) != preview.image_hash
    {
        return Err("Mach-O patch application source image drift".to_owned());
    }
    Ok(())
}

fn validate_patch_identity(
    application: &NsldMachOArm64RelocationApplication,
    patch: &NsldMachOArm64PatchPreview,
) -> Result<(), String> {
    if application.relocation_kind != patch.relocation_kind
        || application.source_output_offset != patch.source_output_offset
        || application.width_bytes != patch.width_bytes
        || application.target_output_offset != Some(patch.target_output_offset)
    {
        return Err(format!(
            "Mach-O patch `{}` relocation identity drift",
            patch.relocation_id
        ));
    }
    Ok(())
}

fn validate_patch_hashes(
    application: &NsldMachOArm64RelocationApplication,
    patch: &NsldMachOArm64PatchPreview,
    source: &[u8],
    encoded: &[u8],
) -> Result<(), String> {
    if crate::fnv1a64_hex(source) != patch.source_bytes_hash {
        return Err(format!(
            "Mach-O patch `{}` source byte hash drift",
            patch.relocation_id
        ));
    }
    if crate::fnv1a64_hex(encoded) != patch.encoded_bytes_hash {
        return Err(format!(
            "Mach-O patch `{}` encoded byte hash drift",
            patch.relocation_id
        ));
    }
    let expected_audit = patch_audit_hash(
        application,
        patch.target_output_offset,
        patch.effective_addend,
        &patch.source_bytes_hash,
        &patch.encoded_bytes_hash,
    );
    if patch.audit_hash != expected_audit {
        return Err(format!(
            "Mach-O patch `{}` preview audit hash drift",
            patch.relocation_id
        ));
    }
    Ok(())
}

fn decode_canonical_hex(
    value: &str,
    label: &str,
    patch: &NsldMachOArm64PatchPreview,
) -> Result<Vec<u8>, String> {
    if !value.len().is_multiple_of(2) {
        return Err(format!(
            "Mach-O patch `{}` {label} bytes have odd hex length",
            patch.relocation_id
        ));
    }
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len() / 2);
    for pair in bytes.chunks_exact(2) {
        let high = hex_nibble(pair[0]).ok_or_else(|| {
            format!(
                "Mach-O patch `{}` {label} bytes are not canonical lowercase hex",
                patch.relocation_id
            )
        })?;
        let low = hex_nibble(pair[1]).ok_or_else(|| {
            format!(
                "Mach-O patch `{}` {label} bytes are not canonical lowercase hex",
                patch.relocation_id
            )
        })?;
        decoded.push(high << 4 | low);
    }
    Ok(decoded)
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
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
            "Mach-O patch `{relocation_id}` write buffer shape drift"
        ));
    }
    let range = checked_range(offset, encoded.len(), image.len(), relocation_id)?;
    if written[range.clone()].iter().any(|occupied| *occupied) {
        return Err(format!(
            "Mach-O patch `{relocation_id}` overlaps a previously applied patch"
        ));
    }
    if image[range.clone()] != *source {
        return Err(format!("Mach-O patch `{relocation_id}` write source drift"));
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
        .ok_or_else(|| format!("Mach-O patch `{relocation_id}` application range overflows"))?;
    if end > limit {
        return Err(format!(
            "Mach-O patch `{relocation_id}` application range {offset}..{end} exceeds image {limit}"
        ));
    }
    Ok(offset..end)
}

fn write_audit_hash(patch: &NsldMachOArm64PatchPreview, post_write_hash: &str) -> String {
    let mut canonical = String::new();
    append_text(&mut canonical, MACHO_ARM64_PATCH_APPLICATION_CONTRACT);
    append_text(&mut canonical, &patch.relocation_id);
    append_text(&mut canonical, &patch.audit_hash);
    append_text(&mut canonical, &patch.source_bytes_hash);
    append_text(&mut canonical, &patch.encoded_bytes_hash);
    append_text(&mut canonical, post_write_hash);
    writeln!(
        canonical,
        "span={}|{}",
        patch.source_output_offset, patch.width_bytes
    )
    .unwrap();
    crate::fnv1a64_hex(canonical.as_bytes())
}

fn application_ledger_hash(
    placement: &NsldMachOPlacementBindingReport,
    relocations: &NsldMachOArm64RelocationApplicationReport,
    preview: &NsldMachOArm64MaterializationPreviewReport,
    applied_image_hash: &str,
    status: &str,
    patches: &[NsldMachOArm64AppliedPatchAudit],
) -> String {
    let mut canonical = String::new();
    append_text(&mut canonical, MACHO_ARM64_PATCH_APPLICATION_CONTRACT);
    append_text(&mut canonical, status);
    append_text(&mut canonical, &placement.plan_hash);
    append_text(&mut canonical, &relocations.plan_hash);
    append_text(&mut canonical, &preview.patch_plan_hash);
    append_text(&mut canonical, &preview.image_hash);
    append_text(&mut canonical, applied_image_hash);
    writeln!(
        canonical,
        "counts={}|{}|{}",
        preview.planned_direct_count,
        patches.len(),
        preview.deferred_patch_count
    )
    .unwrap();
    for patch in patches {
        append_text(&mut canonical, &patch.relocation_id);
        append_text(&mut canonical, &patch.write_audit_hash);
    }
    crate::fnv1a64_hex(canonical.as_bytes())
}

fn append_text(out: &mut String, value: &str) {
    writeln!(out, "text:{}:{value}", value.len()).unwrap();
}

#[cfg(test)]
mod tests {
    use super::apply_write_once;

    #[test]
    fn write_once_rejects_overlap_and_source_drift() {
        let mut image = vec![1, 2, 3, 4, 5];
        let mut written = vec![false; image.len()];
        apply_write_once(&mut image, &mut written, 1, &[2, 3], &[8, 9], "first").unwrap();
        assert_eq!(image, [1, 8, 9, 4, 5]);

        let overlap =
            apply_write_once(&mut image, &mut written, 2, &[9, 4], &[7, 6], "overlap").unwrap_err();
        assert!(overlap.contains("overlaps a previously applied patch"));

        let drift = apply_write_once(&mut image, &mut written, 3, &[7], &[6], "drift").unwrap_err();
        assert!(drift.contains("write source drift"));
    }
}
