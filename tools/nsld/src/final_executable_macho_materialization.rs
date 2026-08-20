use crate::{
    final_executable_macho_input::ParsedMachOObjectLinkage,
    reports::{
        NsldMachOArm64MaterializationPreviewReport, NsldMachOArm64PatchPreview,
        NsldMachOArm64RelocationApplication, NsldMachOArm64RelocationApplicationReport,
        NsldMachOMergedSectionImageAudit, NsldMachOPlacementBindingReport,
    },
};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

pub(crate) const MACHO_ARM64_MATERIALIZATION_PREVIEW_CONTRACT: &str =
    "nuis-nsld-macho-arm64-materialization-preview-v1";

pub(crate) struct MachOImageObject<'a> {
    pub(crate) object_id: &'a str,
    pub(crate) role: &'a str,
    pub(crate) bytes: &'a [u8],
    pub(crate) linkage: &'a ParsedMachOObjectLinkage,
}

struct MergedSectionImage {
    bytes: Vec<u8>,
    copied_bytes: usize,
    zero_fill_bytes: usize,
    section_audits: Vec<NsldMachOMergedSectionImageAudit>,
}

pub(crate) fn build_macho_arm64_materialization_preview(
    objects: &[MachOImageObject<'_>],
    placement: &NsldMachOPlacementBindingReport,
    relocations: &NsldMachOArm64RelocationApplicationReport,
) -> Result<NsldMachOArm64MaterializationPreviewReport, String> {
    if relocations.placement_plan_hash != placement.plan_hash {
        return Err(format!(
            "Mach-O materialization plan drift: relocation placement hash={}, placement hash={}",
            relocations.placement_plan_hash, placement.plan_hash
        ));
    }
    let image = build_merged_section_image(objects, placement)?;
    let original_image_hash = crate::fnv1a64_hex(&image.bytes);
    let applications = relocations
        .applications
        .iter()
        .map(|item| (item.relocation_id.as_str(), item))
        .collect::<BTreeMap<_, _>>();
    if applications.len() != relocations.applications.len() {
        return Err("Mach-O materialization input contains duplicate relocation ids".to_owned());
    }

    let mut patches = Vec::new();
    let mut occupied_spans = Vec::<(usize, usize, &str)>::new();
    for application in relocations
        .applications
        .iter()
        .filter(|item| item.application_status == "planned-direct")
    {
        let source = image_slice(
            &image.bytes,
            application.source_output_offset,
            application.width_bytes,
            &format!("relocation `{}` source", application.relocation_id),
        )?;
        let source_end = application
            .source_output_offset
            .checked_add(application.width_bytes)
            .ok_or_else(|| "Mach-O patch preview span overflows".to_owned())?;
        if let Some((_, _, previous)) = occupied_spans
            .iter()
            .find(|(start, end, _)| application.source_output_offset < *end && *start < source_end)
        {
            return Err(format!(
                "Mach-O direct patch `{}` overlaps direct patch `{previous}`",
                application.relocation_id
            ));
        }
        occupied_spans.push((
            application.source_output_offset,
            source_end,
            &application.relocation_id,
        ));
        let target_output_offset = application.target_output_offset.ok_or_else(|| {
            format!(
                "Mach-O direct patch `{}` has no resolved target offset",
                application.relocation_id
            )
        })?;
        let (encoded, effective_addend) =
            encode_patch(application, source, target_output_offset, &applications)?;
        let source_bytes_hex = hex_bytes(source);
        let encoded_bytes_hex = hex_bytes(&encoded);
        let source_bytes_hash = crate::fnv1a64_hex(source);
        let encoded_bytes_hash = crate::fnv1a64_hex(&encoded);
        let audit_hash = patch_audit_hash(
            application,
            target_output_offset,
            effective_addend,
            &source_bytes_hash,
            &encoded_bytes_hash,
        );
        patches.push(NsldMachOArm64PatchPreview {
            relocation_id: application.relocation_id.clone(),
            relocation_kind: application.relocation_kind.clone(),
            source_output_offset: application.source_output_offset,
            width_bytes: application.width_bytes,
            target_output_offset,
            effective_addend,
            source_bytes_hex,
            encoded_bytes_hex,
            source_bytes_hash,
            encoded_bytes_hash,
            audit_hash,
        });
    }
    if patches.len() != relocations.ready_application_count {
        return Err(format!(
            "Mach-O patch preview coverage drift: planned={}, previewed={}",
            relocations.ready_application_count,
            patches.len()
        ));
    }
    if crate::fnv1a64_hex(&image.bytes) != original_image_hash {
        return Err("Mach-O patch preview mutated the provider-owned source image".to_owned());
    }

    let patch_plan_hash = canonical_patch_plan(
        &placement.plan_hash,
        &relocations.plan_hash,
        &original_image_hash,
        &patches,
    );
    let status = if relocations.platform_structure_count == 0 {
        "preview-ready"
    } else {
        "preview-ready-with-platform-structure-boundary"
    };
    Ok(NsldMachOArm64MaterializationPreviewReport {
        contract: MACHO_ARM64_MATERIALIZATION_PREVIEW_CONTRACT.to_owned(),
        status: status.to_owned(),
        placement_plan_hash: placement.plan_hash.clone(),
        relocation_plan_hash: relocations.plan_hash.clone(),
        image_span_bytes: image.bytes.len(),
        copied_bytes: image.copied_bytes,
        zero_fill_bytes: image.zero_fill_bytes,
        image_hash: original_image_hash,
        section_audits: image.section_audits,
        planned_direct_count: relocations.ready_application_count,
        previewed_patch_count: patches.len(),
        deferred_patch_count: relocations.platform_structure_count,
        metadata_record_count: relocations.metadata_record_count,
        patch_plan_hash: crate::fnv1a64_hex(patch_plan_hash.as_bytes()),
        patches,
    })
}

fn build_merged_section_image(
    objects: &[MachOImageObject<'_>],
    placement: &NsldMachOPlacementBindingReport,
) -> Result<MergedSectionImage, String> {
    let mut object_map = BTreeMap::new();
    for object in objects {
        if object_map.insert(object.object_id, object).is_some() {
            return Err(format!(
                "Mach-O image input contains duplicate object id `{}`",
                object.object_id
            ));
        }
    }
    let mut bytes = vec![0u8; placement.image_span_bytes];
    let mut occupied = BTreeSet::new();
    let mut copied_bytes = 0usize;

    for contribution in &placement.section_placements {
        let object = object_map
            .get(contribution.object_id.as_str())
            .ok_or_else(|| {
                format!(
                    "Mach-O placement references missing image object `{}`",
                    contribution.object_id
                )
            })?;
        if object.role != contribution.object_role {
            return Err(format!(
                "Mach-O image object `{}` role drift: input={}, placement={}",
                object.object_id, object.role, contribution.object_role
            ));
        }
        let section = object
            .linkage
            .sections
            .iter()
            .find(|section| section.ordinal == contribution.input_section_ordinal)
            .ok_or_else(|| {
                format!(
                    "Mach-O image object `{}` has no section ordinal {}",
                    object.object_id, contribution.input_section_ordinal
                )
            })?;
        let section_size = usize::try_from(section.size).map_err(|_| {
            format!(
                "Mach-O image object `{}` section {} size exceeds host address space",
                object.object_id, section.ordinal
            )
        })?;
        if section_size != contribution.size_bytes
            || section.zero_fill != contribution.zero_fill
            || section.name != contribution.input_section_name
            || section.segment_name != contribution.input_segment_name
        {
            return Err(format!(
                "Mach-O image object `{}` section {} placement identity drift",
                object.object_id, section.ordinal
            ));
        }
        let output = checked_range(
            contribution.output_offset,
            contribution.size_bytes,
            bytes.len(),
            "merged image contribution",
        )?;
        for offset in output.clone() {
            if !occupied.insert(offset) {
                return Err(format!(
                    "Mach-O image placement overlaps at output offset {offset}"
                ));
            }
        }
        if section.zero_fill {
            continue;
        }
        let input = checked_range(
            section.payload_offset,
            section_size,
            object.bytes.len(),
            &format!(
                "object `{}` section {} payload",
                object.object_id, section.ordinal
            ),
        )?;
        bytes[output].copy_from_slice(&object.bytes[input]);
        copied_bytes = copied_bytes
            .checked_add(section_size)
            .ok_or_else(|| "Mach-O copied byte count overflows".to_owned())?;
    }

    let mut section_audits = Vec::with_capacity(placement.merged_sections.len());
    for section in &placement.merged_sections {
        let range = checked_range(
            section.output_offset,
            section.size_bytes,
            bytes.len(),
            &format!("merged section `{}`", section.section_id),
        )?;
        let section_copied = placement
            .section_placements
            .iter()
            .filter(|item| item.output_section_id == section.section_id && !item.zero_fill)
            .try_fold(0usize, |total, item| {
                total
                    .checked_add(item.size_bytes)
                    .ok_or_else(|| "Mach-O section copied byte count overflows".to_owned())
            })?;
        let zero_fill_bytes = section
            .size_bytes
            .checked_sub(section_copied)
            .ok_or_else(|| "Mach-O merged section copied byte count exceeds size".to_owned())?;
        section_audits.push(NsldMachOMergedSectionImageAudit {
            section_id: section.section_id.clone(),
            output_offset: section.output_offset,
            size_bytes: section.size_bytes,
            copied_bytes: section_copied,
            zero_fill_bytes,
            content_hash: crate::fnv1a64_hex(&bytes[range]),
        });
    }
    let zero_fill_bytes = bytes
        .len()
        .checked_sub(copied_bytes)
        .ok_or_else(|| "Mach-O copied byte count exceeds image span".to_owned())?;
    Ok(MergedSectionImage {
        bytes,
        copied_bytes,
        zero_fill_bytes,
        section_audits,
    })
}

fn encode_patch(
    application: &NsldMachOArm64RelocationApplication,
    source: &[u8],
    target_output_offset: usize,
    applications: &BTreeMap<&str, &NsldMachOArm64RelocationApplication>,
) -> Result<(Vec<u8>, i64), String> {
    match application.relocation_kind.as_str() {
        "arm64-unsigned" => {
            encode_unsigned(application, source, target_output_offset, applications)
        }
        "arm64-branch26" => encode_branch26(application, source, target_output_offset),
        "arm64-page21" => encode_page21(application, source, target_output_offset, applications),
        "arm64-pageoff12" => {
            encode_pageoff12(application, source, target_output_offset, applications)
        }
        other => Err(format!(
            "Mach-O direct patch `{}` has unsupported preview kind `{other}`",
            application.relocation_id
        )),
    }
}

fn encode_unsigned(
    application: &NsldMachOArm64RelocationApplication,
    source: &[u8],
    target: usize,
    applications: &BTreeMap<&str, &NsldMachOArm64RelocationApplication>,
) -> Result<(Vec<u8>, i64), String> {
    let embedded = read_signed_le(source)? as i128;
    let subtractor = paired_metadata(application, applications, "arm64-subtractor")?
        .map(|item| required_target(item))
        .transpose()?
        .unwrap_or(0);
    let effective = embedded - subtractor as i128;
    let value = target as i128 + effective;
    let maximum = match source.len() {
        4 => u32::MAX as i128,
        8 => u64::MAX as i128,
        width => {
            return Err(format!(
                "Mach-O unsigned patch `{}` has unsupported width {width}",
                application.relocation_id
            ))
        }
    };
    if !(0..=maximum).contains(&value) {
        return Err(format!(
            "Mach-O unsigned patch `{}` value {value} does not fit {} bytes",
            application.relocation_id,
            source.len()
        ));
    }
    let encoded = if source.len() == 4 {
        (value as u32).to_le_bytes().to_vec()
    } else {
        (value as u64).to_le_bytes().to_vec()
    };
    Ok((
        encoded,
        checked_i64(effective, "unsigned effective addend")?,
    ))
}

fn encode_branch26(
    application: &NsldMachOArm64RelocationApplication,
    source: &[u8],
    target: usize,
) -> Result<(Vec<u8>, i64), String> {
    let word = read_instruction(source, application)?;
    if word & 0x7c00_0000 != 0x1400_0000 {
        return Err(format!(
            "Mach-O branch26 patch `{}` source instruction 0x{word:08x} is not B/BL",
            application.relocation_id
        ));
    }
    let embedded = sign_extend(u64::from(word & 0x03ff_ffff), 26) << 2;
    let displacement = target as i128 + embedded as i128 - application.source_output_offset as i128;
    if displacement % 4 != 0 || !(-0x0800_0000..=0x07ff_fffc).contains(&displacement) {
        return Err(format!(
            "Mach-O branch26 patch `{}` displacement {displacement} is unaligned or out of range",
            application.relocation_id
        ));
    }
    let immediate = ((displacement >> 2) as i64 as u32) & 0x03ff_ffff;
    let encoded = (word & !0x03ff_ffff | immediate).to_le_bytes().to_vec();
    Ok((encoded, embedded))
}

fn encode_page21(
    application: &NsldMachOArm64RelocationApplication,
    source: &[u8],
    target: usize,
    applications: &BTreeMap<&str, &NsldMachOArm64RelocationApplication>,
) -> Result<(Vec<u8>, i64), String> {
    let word = read_instruction(source, application)?;
    if word & 0x9f00_0000 != 0x9000_0000 {
        return Err(format!(
            "Mach-O page21 patch `{}` source instruction 0x{word:08x} is not ADRP",
            application.relocation_id
        ));
    }
    let imm21 = u64::from(((word >> 29) & 0x3) | (((word >> 5) & 0x7ffff) << 2));
    let embedded = sign_extend(imm21, 21) << 12;
    let explicit = explicit_pair_addend(application, applications)?;
    let effective = embedded as i128 + explicit as i128;
    let target_address = target as i128 + effective;
    if target_address < 0 {
        return Err(format!(
            "Mach-O page21 patch `{}` target plus addend is negative",
            application.relocation_id
        ));
    }
    let target_page = target_address & !0xfff;
    let source_page = application.source_output_offset as i128 & !0xfff;
    let page_delta = target_page - source_page;
    if page_delta % 4096 != 0 || !(-0x1_0000_0000..=0x0_ffff_f000).contains(&page_delta) {
        return Err(format!(
            "Mach-O page21 patch `{}` page delta {page_delta} is out of range",
            application.relocation_id
        ));
    }
    let encoded_imm = ((page_delta >> 12) as i64 as u32) & 0x001f_ffff;
    let immlo = (encoded_imm & 0x3) << 29;
    let immhi = ((encoded_imm >> 2) & 0x7ffff) << 5;
    let encoded = (word & !0x60ff_ffe0 | immlo | immhi).to_le_bytes().to_vec();
    Ok((encoded, checked_i64(effective, "page21 effective addend")?))
}

fn encode_pageoff12(
    application: &NsldMachOArm64RelocationApplication,
    source: &[u8],
    target: usize,
    applications: &BTreeMap<&str, &NsldMachOArm64RelocationApplication>,
) -> Result<(Vec<u8>, i64), String> {
    let word = read_instruction(source, application)?;
    let scale = pageoff_scale(word).ok_or_else(|| {
        format!(
            "Mach-O pageoff12 patch `{}` source instruction 0x{word:08x} is not supported ADD/load/store unsigned-immediate",
            application.relocation_id
        )
    })?;
    let embedded = i64::from((word >> 10) & 0x0fff) * scale as i64;
    let explicit = explicit_pair_addend(application, applications)?;
    let effective = embedded as i128 + explicit as i128;
    let address = target as i128 + effective;
    if address < 0 {
        return Err(format!(
            "Mach-O pageoff12 patch `{}` target plus addend is negative",
            application.relocation_id
        ));
    }
    let page_offset = (address as u128 & 0x0fff) as usize;
    if page_offset % scale != 0 {
        return Err(format!(
            "Mach-O pageoff12 patch `{}` offset {page_offset} is not aligned to instruction scale {scale}",
            application.relocation_id
        ));
    }
    let immediate = page_offset / scale;
    if immediate > 0x0fff {
        return Err(format!(
            "Mach-O pageoff12 patch `{}` immediate {immediate} is out of range",
            application.relocation_id
        ));
    }
    let encoded = (word & !(0x0fff << 10) | (immediate as u32) << 10)
        .to_le_bytes()
        .to_vec();
    Ok((
        encoded,
        checked_i64(effective, "pageoff12 effective addend")?,
    ))
}

fn pageoff_scale(word: u32) -> Option<usize> {
    if word & 0x7f00_0000 == 0x1100_0000 && word & (1 << 22) == 0 {
        return Some(1);
    }
    if word & 0x3b00_0000 == 0x3900_0000 {
        let vector_q =
            word & (1 << 26) != 0 && (word >> 30) & 0x3 == 0 && (word >> 22) & 0x3 == 0x3;
        return Some(if vector_q {
            16
        } else {
            1usize << ((word >> 30) & 0x3)
        });
    }
    None
}

fn paired_metadata<'a>(
    application: &NsldMachOArm64RelocationApplication,
    applications: &'a BTreeMap<&str, &NsldMachOArm64RelocationApplication>,
    expected_kind: &str,
) -> Result<Option<&'a NsldMachOArm64RelocationApplication>, String> {
    let Some(pair_id) = application.pair_relocation_id.as_deref() else {
        return Ok(None);
    };
    let pair = applications.get(pair_id).copied().ok_or_else(|| {
        format!(
            "Mach-O direct patch `{}` references missing pair `{pair_id}`",
            application.relocation_id
        )
    })?;
    if pair.relocation_kind != expected_kind {
        return Err(format!(
            "Mach-O direct patch `{}` expected pair kind `{expected_kind}`, found `{}`",
            application.relocation_id, pair.relocation_kind
        ));
    }
    if pair.application_status != "paired-metadata"
        || pair.pair_relocation_id.as_deref() != Some(application.relocation_id.as_str())
    {
        return Err(format!(
            "Mach-O direct patch `{}` has an invalid `{expected_kind}` pair",
            application.relocation_id
        ));
    }
    Ok(Some(pair))
}

fn explicit_pair_addend(
    application: &NsldMachOArm64RelocationApplication,
    applications: &BTreeMap<&str, &NsldMachOArm64RelocationApplication>,
) -> Result<i64, String> {
    let Some(pair) = paired_metadata(application, applications, "arm64-addend")? else {
        return Ok(0);
    };
    pair.explicit_addend.ok_or_else(|| {
        format!(
            "Mach-O addend pair `{}` has no decoded addend",
            pair.relocation_id
        )
    })
}

fn required_target(application: &NsldMachOArm64RelocationApplication) -> Result<usize, String> {
    application.target_output_offset.ok_or_else(|| {
        format!(
            "Mach-O paired relocation `{}` has no resolved target offset",
            application.relocation_id
        )
    })
}

fn read_instruction(
    source: &[u8],
    application: &NsldMachOArm64RelocationApplication,
) -> Result<u32, String> {
    if application.source_output_offset % 4 != 0 {
        return Err(format!(
            "Mach-O instruction patch `{}` source offset {} is not 4-byte aligned",
            application.relocation_id, application.source_output_offset
        ));
    }
    let bytes: [u8; 4] = source.try_into().map_err(|_| {
        format!(
            "Mach-O instruction patch `{}` requires exactly 4 bytes",
            application.relocation_id
        )
    })?;
    Ok(u32::from_le_bytes(bytes))
}

fn read_signed_le(bytes: &[u8]) -> Result<i64, String> {
    match bytes {
        [a, b, c, d] => Ok(i32::from_le_bytes([*a, *b, *c, *d]) as i64),
        [a, b, c, d, e, f, g, h] => Ok(i64::from_le_bytes([*a, *b, *c, *d, *e, *f, *g, *h])),
        _ => Err(format!(
            "Mach-O absolute relocation width {} is unsupported",
            bytes.len()
        )),
    }
}

fn sign_extend(value: u64, bits: u32) -> i64 {
    let shift = 64 - bits;
    ((value << shift) as i64) >> shift
}

fn checked_i64(value: i128, label: &str) -> Result<i64, String> {
    i64::try_from(value).map_err(|_| format!("Mach-O {label} exceeds signed 64-bit range"))
}

fn image_slice<'a>(
    bytes: &'a [u8],
    offset: usize,
    size: usize,
    label: &str,
) -> Result<&'a [u8], String> {
    let range = checked_range(offset, size, bytes.len(), label)?;
    Ok(&bytes[range])
}

fn checked_range(
    offset: usize,
    size: usize,
    limit: usize,
    label: &str,
) -> Result<std::ops::Range<usize>, String> {
    let end = offset
        .checked_add(size)
        .ok_or_else(|| format!("Mach-O {label} range overflows"))?;
    if end > limit {
        return Err(format!(
            "Mach-O {label} range {offset}..{end} exceeds limit {limit}"
        ));
    }
    Ok(offset..end)
}

fn patch_audit_hash(
    application: &NsldMachOArm64RelocationApplication,
    target: usize,
    effective_addend: i64,
    source_hash: &str,
    encoded_hash: &str,
) -> String {
    let mut canonical = String::new();
    append_text(&mut canonical, &application.relocation_id);
    append_text(&mut canonical, &application.relocation_kind);
    append_text(&mut canonical, source_hash);
    append_text(&mut canonical, encoded_hash);
    writeln!(
        canonical,
        "facts={}|{}|{}|{}|{}",
        application.source_output_offset,
        application.width_bytes,
        target,
        effective_addend,
        application.action_kind
    )
    .unwrap();
    crate::fnv1a64_hex(canonical.as_bytes())
}

fn canonical_patch_plan(
    placement_hash: &str,
    relocation_hash: &str,
    image_hash: &str,
    patches: &[NsldMachOArm64PatchPreview],
) -> String {
    let mut canonical = String::new();
    append_text(&mut canonical, MACHO_ARM64_MATERIALIZATION_PREVIEW_CONTRACT);
    append_text(&mut canonical, placement_hash);
    append_text(&mut canonical, relocation_hash);
    append_text(&mut canonical, image_hash);
    for patch in patches {
        append_text(&mut canonical, &patch.relocation_id);
        append_text(&mut canonical, &patch.audit_hash);
    }
    canonical
}

fn hex_bytes(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(out, "{byte:02x}").unwrap();
    }
    out
}

fn append_text(out: &mut String, value: &str) {
    writeln!(out, "text:{}:{value}", value.len()).unwrap();
}

#[cfg(test)]
#[path = "final_executable_macho_materialization_tests.rs"]
mod tests;
