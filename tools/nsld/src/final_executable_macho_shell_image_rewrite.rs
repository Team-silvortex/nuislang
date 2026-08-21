use crate::{
    final_executable_macho_shell_image_rewrite_encoding::{
        encode_final_relocation, encode_final_stub,
    },
    final_executable_macho_shell_layout::locate_source_address,
    reports::{
        NsldMachOArm64MaterializationPreviewReport, NsldMachOArm64PlatformPatchApplicationReport,
        NsldMachOArm64PlatformStructurePlanReport, NsldMachOArm64RelocationApplicationReport,
        NsldMachOArm64ShellImageRewriteAudit, NsldMachOArm64ShellLayoutPlanReport,
    },
};
use std::collections::BTreeMap;
use std::fmt::Write as _;

const SHELL_IMAGE_REWRITE_CONTRACT: &str = "nuis-nsld-macho-arm64-shell-image-rewrite-v1";

pub(crate) struct ShellImageRewriteResult {
    pub(crate) relocation_count: usize,
    pub(crate) stub_count: usize,
    pub(crate) got_count: usize,
    pub(crate) audits: Vec<NsldMachOArm64ShellImageRewriteAudit>,
}

pub(crate) fn rewrite_shell_image_addresses(
    relocations: &NsldMachOArm64RelocationApplicationReport,
    preview: &NsldMachOArm64MaterializationPreviewReport,
    platform: &NsldMachOArm64PlatformStructurePlanReport,
    applied: &NsldMachOArm64PlatformPatchApplicationReport,
    shell: &NsldMachOArm64ShellLayoutPlanReport,
    image: &mut [u8],
) -> Result<ShellImageRewriteResult, String> {
    let applications = unique_map(
        relocations
            .applications
            .iter()
            .map(|application| (application.relocation_id.as_str(), application)),
        "relocation application",
    )?;
    let direct_patches = unique_map(
        preview
            .patches
            .iter()
            .map(|patch| (patch.relocation_id.as_str(), patch)),
        "direct patch",
    )?;
    let platform_patches = unique_map(
        applied
            .patches
            .iter()
            .map(|patch| (patch.relocation_id.as_str(), patch)),
        "platform patch",
    )?;
    let bindings = unique_map(
        platform
            .relocation_bindings
            .iter()
            .map(|binding| (binding.relocation_id.as_str(), binding)),
        "platform binding",
    )?;

    let mut spans = Vec::new();
    let mut audits = Vec::with_capacity(shell.required_address_rewrite_count);
    let mut relocation_count = 0usize;
    for application in relocations
        .applications
        .iter()
        .filter(|application| application.application_status != "paired-metadata")
    {
        let (source_hex, source_hash, prewrite_hash, target_source_offset, target_absolute_value) =
            match application.application_status.as_str() {
                "planned-direct" => {
                    let patch = direct_patches
                        .get(application.relocation_id.as_str())
                        .copied()
                        .ok_or_else(|| {
                            format!(
                                "Mach-O shell rewrite has no direct patch for `{}`",
                                application.relocation_id
                            )
                        })?;
                    if patch.source_output_offset != application.source_output_offset
                        || patch.width_bytes != application.width_bytes
                        || patch.relocation_kind != application.relocation_kind
                        || patch.target_output_offset != application.target_output_offset
                        || patch.target_absolute_value != application.target_absolute_value
                    {
                        return Err(format!(
                            "Mach-O direct patch `{}` shape drift",
                            application.relocation_id
                        ));
                    }
                    (
                        patch.source_bytes_hex.as_str(),
                        patch.source_bytes_hash.as_str(),
                        patch.encoded_bytes_hash.as_str(),
                        application.target_output_offset,
                        application.target_absolute_value,
                    )
                }
                "planned-platform-structure" => {
                    let patch = platform_patches
                        .get(application.relocation_id.as_str())
                        .copied()
                        .ok_or_else(|| {
                            format!(
                                "Mach-O shell rewrite has no platform patch for `{}`",
                                application.relocation_id
                            )
                        })?;
                    let binding = bindings
                        .get(application.relocation_id.as_str())
                        .copied()
                        .ok_or_else(|| {
                            format!(
                                "Mach-O shell rewrite has no platform binding for `{}`",
                                application.relocation_id
                            )
                        })?;
                    if patch.source_output_offset != application.source_output_offset
                        || patch.width_bytes != application.width_bytes
                        || patch.relocation_kind != application.relocation_kind
                        || patch.patch_target_output_offset != binding.patch_target_output_offset
                        || binding.source_output_offset != application.source_output_offset
                        || binding.width_bytes != application.width_bytes
                    {
                        return Err(format!(
                            "Mach-O platform patch `{}` shape drift",
                            application.relocation_id
                        ));
                    }
                    (
                        patch.source_bytes_hex.as_str(),
                        patch.source_bytes_hash.as_str(),
                        patch.encoded_bytes_hash.as_str(),
                        Some(binding.patch_target_output_offset),
                        None,
                    )
                }
                other => {
                    return Err(format!(
                        "Mach-O shell relocation `{}` has unregistered status `{other}`",
                        application.relocation_id
                    ));
                }
            };
        let encoding_source = decode_hex(source_hex, &application.relocation_id)?;
        if encoding_source.len() != application.width_bytes
            || crate::fnv1a64_hex(&encoding_source) != source_hash
        {
            return Err(format!(
                "Mach-O shell relocation `{}` source byte drift",
                application.relocation_id
            ));
        }
        let source = locate_source_address(
            application.source_output_offset,
            &shell.sections,
            &shell.segments,
        )?;
        let target_vm_address = match (target_source_offset, target_absolute_value) {
            (Some(offset), None) => {
                locate_source_address(offset, &shell.sections, &shell.segments)?.vm_address
            }
            (None, Some(value)) => value,
            (None, None) => {
                return Err(format!(
                    "Mach-O shell relocation `{}` has no target value",
                    application.relocation_id
                ));
            }
            (Some(_), Some(_)) => {
                return Err(format!(
                    "Mach-O shell relocation `{}` has ambiguous target values",
                    application.relocation_id
                ));
            }
        };
        let file_offset = source.file_offset.ok_or_else(|| {
            format!(
                "Mach-O shell relocation `{}` source is not file-backed",
                application.relocation_id
            )
        })?;
        let (encoded, effective_addend) = encode_final_relocation(
            application,
            &encoding_source,
            source.vm_address,
            target_vm_address,
            &applications,
            shell,
        )?;
        push_rewrite(
            image,
            &mut spans,
            &mut audits,
            "relocation-final-address",
            &application.relocation_id,
            application.source_output_offset,
            file_offset,
            source.vm_address,
            Some(target_vm_address),
            Some(effective_addend),
            prewrite_hash,
            &encoding_source,
            &encoded,
        )?;
        relocation_count += 1;
    }
    if relocation_count + relocations.metadata_record_count != relocations.relocation_count {
        return Err("Mach-O shell relocation rewrite coverage drift".to_owned());
    }

    let stub_count = rewrite_stubs(platform, applied, shell, image, &mut spans, &mut audits)?;
    let got_count = rewrite_internal_got(applied, shell, image, &mut spans, &mut audits)?;
    if audits.len() != shell.required_address_rewrite_count {
        return Err(format!(
            "Mach-O shell rewrite count drift: plan={}, applied={}",
            shell.required_address_rewrite_count,
            audits.len()
        ));
    }
    Ok(ShellImageRewriteResult {
        relocation_count,
        stub_count,
        got_count,
        audits,
    })
}

fn rewrite_stubs(
    platform: &NsldMachOArm64PlatformStructurePlanReport,
    applied: &NsldMachOArm64PlatformPatchApplicationReport,
    shell: &NsldMachOArm64ShellLayoutPlanReport,
    image: &mut [u8],
    spans: &mut Vec<(usize, usize, String)>,
    audits: &mut Vec<NsldMachOArm64ShellImageRewriteAudit>,
) -> Result<usize, String> {
    let mut count = 0usize;
    for target in platform
        .targets
        .iter()
        .filter(|target| target.stub_output_offset.is_some())
    {
        let stub_source = target.stub_output_offset.unwrap();
        let got_source = target
            .got_output_offset
            .ok_or_else(|| format!("Mach-O shell stub `{}` has no GOT", target.structure_id))?;
        let stub = locate_source_address(stub_source, &shell.sections, &shell.segments)?;
        let got = locate_source_address(got_source, &shell.sections, &shell.segments)?;
        let file_offset = stub
            .file_offset
            .ok_or_else(|| "Mach-O shell stub is not file-backed".to_owned())?;
        let write = find_structure_write(applied, &target.structure_id, stub_source)?;
        if write.write_kind != "arm64-branch-stub" || write.width_bytes != 12 {
            return Err(format!(
                "Mach-O shell stub `{}` write shape drift",
                target.structure_id
            ));
        }
        let encoding_source = decode_structure_write(write)?;
        let encoded = encode_final_stub(stub.vm_address, got.vm_address, &target.structure_id)?;
        push_rewrite(
            image,
            spans,
            audits,
            "stub-final-address",
            &target.structure_id,
            stub_source,
            file_offset,
            stub.vm_address,
            Some(got.vm_address),
            None,
            &write.encoded_bytes_hash,
            &encoding_source,
            &encoded,
        )?;
        count += 1;
    }
    if count != platform.stub_entry_count {
        return Err("Mach-O shell stub rewrite coverage drift".to_owned());
    }
    Ok(count)
}

fn rewrite_internal_got(
    applied: &NsldMachOArm64PlatformPatchApplicationReport,
    shell: &NsldMachOArm64ShellLayoutPlanReport,
    image: &mut [u8],
    spans: &mut Vec<(usize, usize, String)>,
    audits: &mut Vec<NsldMachOArm64ShellImageRewriteAudit>,
) -> Result<usize, String> {
    for rebase in &shell.rebases {
        let location = locate_source_address(
            rebase.got_source_image_offset,
            &shell.sections,
            &shell.segments,
        )?;
        let target = locate_source_address(
            rebase.target_source_image_offset,
            &shell.sections,
            &shell.segments,
        )?;
        if location.file_offset != Some(rebase.file_offset)
            || location.vm_address != rebase.vm_address
            || target.vm_address != rebase.target_vm_address
        {
            return Err(format!(
                "Mach-O shell rebase `{}` address drift",
                rebase.rebase_id
            ));
        }
        let write = find_structure_write(
            applied,
            &rebase.structure_id,
            rebase.got_source_image_offset,
        )?;
        if write.write_kind != "internal-image-relative-got" || write.width_bytes != 8 {
            return Err(format!(
                "Mach-O shell rebase `{}` GOT write shape drift",
                rebase.rebase_id
            ));
        }
        let encoding_source = decode_structure_write(write)?;
        let encoded = rebase.target_vm_address.to_le_bytes();
        push_rewrite(
            image,
            spans,
            audits,
            "internal-got-final-address",
            &rebase.rebase_id,
            rebase.got_source_image_offset,
            rebase.file_offset,
            rebase.vm_address,
            Some(rebase.target_vm_address),
            None,
            &write.encoded_bytes_hash,
            &encoding_source,
            &encoded,
        )?;
    }
    Ok(shell.rebases.len())
}

#[allow(clippy::too_many_arguments)]
fn push_rewrite(
    image: &mut [u8],
    spans: &mut Vec<(usize, usize, String)>,
    audits: &mut Vec<NsldMachOArm64ShellImageRewriteAudit>,
    kind: &str,
    source_id: &str,
    source_image_offset: usize,
    file_offset: usize,
    vm_address: u64,
    target_vm_address: Option<u64>,
    effective_addend: Option<i64>,
    expected_prewrite_hash: &str,
    encoding_source: &[u8],
    encoded: &[u8],
) -> Result<(), String> {
    if encoding_source.len() != encoded.len() {
        return Err(format!("Mach-O shell rewrite `{source_id}` width drift"));
    }
    let end = file_offset
        .checked_add(encoded.len())
        .ok_or_else(|| "Mach-O shell rewrite span overflows".to_owned())?;
    if end > image.len() {
        return Err(format!(
            "Mach-O shell rewrite `{source_id}` exceeds private image"
        ));
    }
    if let Some((_, _, previous)) = spans
        .iter()
        .find(|(start, stop, _)| file_offset < *stop && *start < end)
    {
        return Err(format!(
            "Mach-O shell rewrite `{source_id}` overlaps `{previous}`"
        ));
    }
    let prewrite_hash = crate::fnv1a64_hex(&image[file_offset..end]);
    if prewrite_hash != expected_prewrite_hash {
        return Err(format!(
            "Mach-O shell rewrite `{source_id}` prewrite byte drift"
        ));
    }
    let encoding_source_bytes_hash = crate::fnv1a64_hex(encoding_source);
    let encoded_bytes_hash = crate::fnv1a64_hex(encoded);
    let rewrite_id = format!("macho-arm64-shell-image-rewrite-{:06}", audits.len());
    let audit_hash = rewrite_audit_hash(
        &rewrite_id,
        kind,
        source_id,
        source_image_offset,
        file_offset,
        vm_address,
        target_vm_address,
        effective_addend,
        encoded.len(),
        &prewrite_hash,
        &encoding_source_bytes_hash,
        &encoded_bytes_hash,
    );
    image[file_offset..end].copy_from_slice(encoded);
    spans.push((file_offset, end, rewrite_id.clone()));
    audits.push(NsldMachOArm64ShellImageRewriteAudit {
        rewrite_id,
        rewrite_kind: kind.to_owned(),
        source_id: source_id.to_owned(),
        source_image_offset,
        file_offset,
        vm_address,
        target_vm_address,
        effective_addend,
        width_bytes: encoded.len(),
        prewrite_bytes_hash: prewrite_hash,
        encoding_source_bytes_hash,
        encoded_bytes_hash,
        audit_hash,
    });
    Ok(())
}

fn find_structure_write<'a>(
    applied: &'a NsldMachOArm64PlatformPatchApplicationReport,
    structure_id: &str,
    output_offset: usize,
) -> Result<&'a crate::reports::NsldMachOArm64PlatformWriteAudit, String> {
    let matches = applied
        .structure_writes
        .iter()
        .filter(|write| write.structure_id == structure_id && write.output_offset == output_offset)
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [write] => Ok(*write),
        _ => Err(format!(
            "Mach-O shell structure `{structure_id}` offset {output_offset} maps to {} writes",
            matches.len()
        )),
    }
}

fn decode_structure_write(
    write: &crate::reports::NsldMachOArm64PlatformWriteAudit,
) -> Result<Vec<u8>, String> {
    let bytes = decode_hex(&write.encoded_bytes_hex, &write.write_id)?;
    if bytes.len() != write.width_bytes || crate::fnv1a64_hex(&bytes) != write.encoded_bytes_hash {
        return Err(format!(
            "Mach-O shell structure write `{}` encoded byte drift",
            write.write_id
        ));
    }
    Ok(bytes)
}

fn decode_hex(value: &str, source_id: &str) -> Result<Vec<u8>, String> {
    if !value.len().is_multiple_of(2) {
        return Err(format!(
            "Mach-O shell source `{source_id}` has odd hex length"
        ));
    }
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = hex_nibble(pair[0])
                .ok_or_else(|| format!("Mach-O shell source `{source_id}` is not canonical hex"))?;
            let low = hex_nibble(pair[1])
                .ok_or_else(|| format!("Mach-O shell source `{source_id}` is not canonical hex"))?;
            Ok(high << 4 | low)
        })
        .collect()
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

fn unique_map<'a, T>(
    values: impl Iterator<Item = (&'a str, &'a T)>,
    label: &str,
) -> Result<BTreeMap<&'a str, &'a T>, String> {
    let mut map = BTreeMap::new();
    for (id, value) in values {
        if map.insert(id, value).is_some() {
            return Err(format!("Mach-O shell repeats {label} `{id}`"));
        }
    }
    Ok(map)
}

#[allow(clippy::too_many_arguments)]
fn rewrite_audit_hash(
    rewrite_id: &str,
    kind: &str,
    source_id: &str,
    source_image_offset: usize,
    file_offset: usize,
    vm_address: u64,
    target_vm_address: Option<u64>,
    effective_addend: Option<i64>,
    width: usize,
    prewrite_hash: &str,
    encoding_source_hash: &str,
    encoded_hash: &str,
) -> String {
    let mut out = String::new();
    for value in [
        SHELL_IMAGE_REWRITE_CONTRACT,
        rewrite_id,
        kind,
        source_id,
        prewrite_hash,
        encoding_source_hash,
        encoded_hash,
    ] {
        writeln!(out, "text:{}:{value}", value.len()).unwrap();
    }
    writeln!(
        out,
        "address={source_image_offset}|{file_offset}|{vm_address}|{}|{}|{width}",
        target_vm_address.map_or("none".to_owned(), |value| value.to_string()),
        effective_addend.map_or("none".to_owned(), |value| value.to_string()),
    )
    .unwrap();
    crate::fnv1a64_hex(out.as_bytes())
}
