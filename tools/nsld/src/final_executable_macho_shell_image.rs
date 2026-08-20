use crate::{
    final_executable_macho_materialization::MACHO_ARM64_MATERIALIZATION_PREVIEW_CONTRACT,
    final_executable_macho_platform::MACHO_ARM64_PLATFORM_STRUCTURE_PLAN_CONTRACT,
    final_executable_macho_platform_application::{
        MachOArm64PlatformAppliedImage, MACHO_ARM64_PLATFORM_PATCH_APPLICATION_CONTRACT,
    },
    final_executable_macho_relocation::MACHO_ARM64_RELOCATION_APPLICATION_CONTRACT,
    final_executable_macho_shell::MACHO_ARM64_SHELL_LAYOUT_PLAN_CONTRACT,
    final_executable_macho_shell_image_commands::encode_shell_header_and_commands,
    final_executable_macho_shell_image_linkedit::{encode_shell_linkedit, EncodedShellLinkedit},
    final_executable_macho_shell_image_rewrite::rewrite_shell_image_addresses,
    final_executable_macho_shell_signature::{
        encode_macho_arm64_ad_hoc_signature, plan_macho_arm64_ad_hoc_signature,
    },
    final_executable_macho_shell_signature_validation::validate_macho_arm64_signed_shell_image,
    reports::{
        NsldMachOArm64MaterializationPreviewReport, NsldMachOArm64PlatformStructurePlanReport,
        NsldMachOArm64RelocationApplicationReport, NsldMachOArm64ShellImageSerializationReport,
        NsldMachOArm64ShellLayoutPlanReport,
    },
};
use std::fmt::Write as _;

pub(crate) const MACHO_ARM64_SHELL_IMAGE_SERIALIZATION_CONTRACT: &str =
    "nuis-nsld-macho-arm64-shell-image-serialization-v2";

#[derive(Debug)]
pub(crate) struct MachOArm64SerializedShellImage {
    pub(crate) bytes: Vec<u8>,
    pub(crate) report: NsldMachOArm64ShellImageSerializationReport,
}

pub(crate) fn serialize_macho_arm64_shell_image(
    relocations: &NsldMachOArm64RelocationApplicationReport,
    preview: &NsldMachOArm64MaterializationPreviewReport,
    platform: &NsldMachOArm64PlatformStructurePlanReport,
    applied: &MachOArm64PlatformAppliedImage,
    shell: &NsldMachOArm64ShellLayoutPlanReport,
) -> Result<MachOArm64SerializedShellImage, String> {
    validate_envelope(relocations, preview, platform, applied, shell)?;
    let signature_plan = plan_macho_arm64_ad_hoc_signature(shell)?;
    let commands = encode_shell_header_and_commands(shell, signature_plan.signature_payload_bytes)?;
    let linkedit = encode_shell_linkedit(shell)?;
    let mut image = vec![0; shell.code_signature_file_offset];
    let mut occupied = Vec::new();

    write_region(&mut image, &mut occupied, 0, &commands.header, "header")?;
    write_region(
        &mut image,
        &mut occupied,
        shell.header_size_bytes,
        &commands.load_commands,
        "load-commands",
    )?;
    let (copied_section_count, copied_section_bytes) =
        copy_sections(applied, shell, &mut image, &mut occupied)?;
    write_linkedit(shell, &linkedit, &mut image, &mut occupied)?;

    let rewrites = rewrite_shell_image_addresses(
        relocations,
        preview,
        platform,
        &applied.report,
        shell,
        &mut image,
    )?;
    let signature_payload = encode_macho_arm64_ad_hoc_signature(&image, &signature_plan)?;
    image.extend_from_slice(&signature_payload);
    let code_signature = validate_macho_arm64_signed_shell_image(&image, shell, &signature_plan)?;
    let header_hash = crate::fnv1a64_hex(&commands.header);
    let load_commands_hash = crate::fnv1a64_hex(&commands.load_commands);
    let rebase_stream_hash = crate::fnv1a64_hex(&linkedit.rebase_stream);
    let bind_stream_hash = crate::fnv1a64_hex(&linkedit.bind_stream);
    let symbol_table_hash = crate::fnv1a64_hex(&linkedit.symbol_table);
    let indirect_symbol_table_hash = crate::fnv1a64_hex(&linkedit.indirect_symbol_table);
    let string_table_hash = crate::fnv1a64_hex(&linkedit.string_table);
    let linkedit_hash = hash_range(
        &image,
        shell.linkedit_file_offset,
        shell.linkedit_bytes,
        "linkedit",
    )?;
    let shell_image_hash = crate::fnv1a64_hex(&image);
    let status = "signed-private-image-validated";
    let publication_status = "private-not-published";
    let code_signature_status = code_signature.status.as_str();
    let serialization_ledger_hash = serialization_ledger_hash(
        status,
        publication_status,
        code_signature_status,
        shell,
        applied,
        &header_hash,
        &load_commands_hash,
        &rebase_stream_hash,
        &bind_stream_hash,
        &symbol_table_hash,
        &indirect_symbol_table_hash,
        &string_table_hash,
        &linkedit_hash,
        &shell_image_hash,
        &code_signature.validation_ledger_hash,
        &code_signature.signature_payload_sha256,
        copied_section_count,
        copied_section_bytes,
        &rewrites.audits,
    );
    let report = NsldMachOArm64ShellImageSerializationReport {
        contract: MACHO_ARM64_SHELL_IMAGE_SERIALIZATION_CONTRACT.to_owned(),
        status: status.to_owned(),
        shell_layout_plan_hash: shell.plan_hash.clone(),
        platform_application_ledger_hash: applied.report.application_ledger_hash.clone(),
        platform_image_hash: applied.report.platform_image_hash.clone(),
        shell_image_span_bytes: image.len(),
        header_bytes: commands.header.len(),
        load_command_bytes: commands.load_commands.len(),
        copied_section_count,
        copied_section_bytes,
        relocation_rewrite_count: rewrites.relocation_count,
        stub_rewrite_count: rewrites.stub_count,
        got_rewrite_count: rewrites.got_count,
        rewrite_count: rewrites.audits.len(),
        header_hash,
        load_commands_hash,
        rebase_stream_hash,
        bind_stream_hash,
        symbol_table_hash,
        indirect_symbol_table_hash,
        string_table_hash,
        linkedit_hash,
        shell_image_hash,
        serialization_ledger_hash,
        code_signature_file_offset: shell.code_signature_file_offset,
        code_signature_status: code_signature_status.to_owned(),
        publication_status: publication_status.to_owned(),
        code_signature,
        rewrites: rewrites.audits,
    };
    Ok(MachOArm64SerializedShellImage {
        bytes: image,
        report,
    })
}

fn validate_envelope(
    relocations: &NsldMachOArm64RelocationApplicationReport,
    preview: &NsldMachOArm64MaterializationPreviewReport,
    platform: &NsldMachOArm64PlatformStructurePlanReport,
    applied: &MachOArm64PlatformAppliedImage,
    shell: &NsldMachOArm64ShellLayoutPlanReport,
) -> Result<(), String> {
    if relocations.contract != MACHO_ARM64_RELOCATION_APPLICATION_CONTRACT
        || preview.contract != MACHO_ARM64_MATERIALIZATION_PREVIEW_CONTRACT
        || platform.contract != MACHO_ARM64_PLATFORM_STRUCTURE_PLAN_CONTRACT
        || applied.report.contract != MACHO_ARM64_PLATFORM_PATCH_APPLICATION_CONTRACT
        || shell.contract != MACHO_ARM64_SHELL_LAYOUT_PLAN_CONTRACT
    {
        return Err("Mach-O shell image serializer rejects an upstream contract".to_owned());
    }
    if preview.relocation_plan_hash != relocations.plan_hash
        || platform.relocation_plan_hash != relocations.plan_hash
        || applied.report.relocation_plan_hash != relocations.plan_hash
        || applied.report.platform_structure_plan_hash != platform.plan_hash
        || shell.platform_structure_plan_hash != platform.plan_hash
        || shell.platform_application_ledger_hash != applied.report.application_ledger_hash
        || shell.platform_image_hash != applied.report.platform_image_hash
    {
        return Err("Mach-O shell image serializer input hash drift".to_owned());
    }
    if applied.bytes.len() != applied.report.platform_image_span_bytes
        || crate::fnv1a64_hex(&applied.bytes) != applied.report.platform_image_hash
    {
        return Err("Mach-O shell image serializer platform image drift".to_owned());
    }
    let linkedit_end = shell
        .linkedit_file_offset
        .checked_add(shell.linkedit_bytes)
        .ok_or_else(|| "Mach-O shell linkedit span overflows".to_owned())?;
    let command_end = shell
        .header_size_bytes
        .checked_add(shell.load_command_size_bytes)
        .ok_or_else(|| "Mach-O shell command span overflows".to_owned())?;
    if shell.status != "layout-planned-with-code-signature-boundary"
        || shell.code_signature_status != "required-payload-pending"
        || shell.planned_file_span_bytes != linkedit_end
        || shell.code_signature_file_offset < shell.planned_file_span_bytes
        || command_end > shell.first_content_file_offset
        || shell.segments.len() != shell.segment_count
        || shell.sections.len() != shell.section_count
        || shell.load_commands.len() != shell.load_command_count
    {
        return Err("Mach-O shell image serializer layout envelope drift".to_owned());
    }
    Ok(())
}

fn copy_sections(
    applied: &MachOArm64PlatformAppliedImage,
    shell: &NsldMachOArm64ShellLayoutPlanReport,
    image: &mut [u8],
    occupied: &mut Vec<(usize, usize, String)>,
) -> Result<(usize, usize), String> {
    let mut count = 0usize;
    let mut copied = 0usize;
    for section in &shell.sections {
        match (section.source_image_offset, section.file_offset) {
            (Some(source_offset), Some(file_offset)) => {
                if section.file_size_bytes != section.source_size_bytes {
                    return Err(format!(
                        "Mach-O shell section `{}` file/source size drift",
                        section.section_id
                    ));
                }
                let source = checked_slice(
                    &applied.bytes,
                    source_offset,
                    section.source_size_bytes,
                    &section.section_id,
                )?;
                write_region(image, occupied, file_offset, source, &section.section_id)?;
                count += 1;
                copied = copied
                    .checked_add(source.len())
                    .ok_or_else(|| "Mach-O shell copied byte count overflows".to_owned())?;
            }
            (Some(_), None) if section.file_size_bytes == 0 => {}
            _ => {
                return Err(format!(
                    "Mach-O shell section `{}` has an invalid source/file pair",
                    section.section_id
                ));
            }
        }
    }
    Ok((count, copied))
}

fn write_linkedit(
    shell: &NsldMachOArm64ShellLayoutPlanReport,
    linkedit: &EncodedShellLinkedit,
    image: &mut [u8],
    occupied: &mut Vec<(usize, usize, String)>,
) -> Result<(), String> {
    for (offset, bytes, label) in [
        (
            shell.rebase_stream_offset,
            linkedit.rebase_stream.as_slice(),
            "rebase-stream",
        ),
        (
            shell.bind_stream_offset,
            linkedit.bind_stream.as_slice(),
            "bind-stream",
        ),
        (
            shell.symbol_table_offset,
            linkedit.symbol_table.as_slice(),
            "symbol-table",
        ),
        (
            shell.indirect_symbol_table_offset,
            linkedit.indirect_symbol_table.as_slice(),
            "indirect-symbol-table",
        ),
        (
            shell.string_table_offset,
            linkedit.string_table.as_slice(),
            "string-table",
        ),
    ] {
        write_region(image, occupied, offset, bytes, label)?;
    }
    Ok(())
}

fn write_region(
    image: &mut [u8],
    occupied: &mut Vec<(usize, usize, String)>,
    offset: usize,
    bytes: &[u8],
    label: &str,
) -> Result<(), String> {
    if bytes.is_empty() {
        return Ok(());
    }
    let end = offset
        .checked_add(bytes.len())
        .ok_or_else(|| format!("Mach-O shell `{label}` span overflows"))?;
    if end > image.len() {
        return Err(format!("Mach-O shell `{label}` exceeds private image"));
    }
    if let Some((_, _, previous)) = occupied
        .iter()
        .find(|(start, stop, _)| offset < *stop && *start < end)
    {
        return Err(format!(
            "Mach-O shell `{label}` overlaps serialized region `{previous}`"
        ));
    }
    image[offset..end].copy_from_slice(bytes);
    occupied.push((offset, end, label.to_owned()));
    Ok(())
}

fn checked_slice<'a>(
    bytes: &'a [u8],
    offset: usize,
    size: usize,
    label: &str,
) -> Result<&'a [u8], String> {
    let end = offset
        .checked_add(size)
        .ok_or_else(|| format!("Mach-O shell source `{label}` span overflows"))?;
    bytes
        .get(offset..end)
        .ok_or_else(|| format!("Mach-O shell source `{label}` exceeds platform image"))
}

fn hash_range(bytes: &[u8], offset: usize, size: usize, label: &str) -> Result<String, String> {
    checked_slice(bytes, offset, size, label).map(crate::fnv1a64_hex)
}

#[allow(clippy::too_many_arguments)]
fn serialization_ledger_hash(
    status: &str,
    publication_status: &str,
    code_signature_status: &str,
    shell: &NsldMachOArm64ShellLayoutPlanReport,
    applied: &MachOArm64PlatformAppliedImage,
    header_hash: &str,
    load_commands_hash: &str,
    rebase_hash: &str,
    bind_hash: &str,
    symbol_hash: &str,
    indirect_hash: &str,
    string_hash: &str,
    linkedit_hash: &str,
    image_hash: &str,
    signature_validation_hash: &str,
    signature_payload_hash: &str,
    copied_section_count: usize,
    copied_section_bytes: usize,
    rewrites: &[crate::reports::NsldMachOArm64ShellImageRewriteAudit],
) -> String {
    let mut out = String::new();
    for value in [
        MACHO_ARM64_SHELL_IMAGE_SERIALIZATION_CONTRACT,
        status,
        publication_status,
        code_signature_status,
        &shell.plan_hash,
        &applied.report.application_ledger_hash,
        &applied.report.platform_image_hash,
        header_hash,
        load_commands_hash,
        rebase_hash,
        bind_hash,
        symbol_hash,
        indirect_hash,
        string_hash,
        linkedit_hash,
        image_hash,
        signature_validation_hash,
        signature_payload_hash,
    ] {
        writeln!(out, "text:{}:{value}", value.len()).unwrap();
    }
    writeln!(
        out,
        "counts={copied_section_count}|{copied_section_bytes}|{}|{}",
        rewrites.len(),
        shell.code_signature_file_offset
    )
    .unwrap();
    for rewrite in rewrites {
        writeln!(out, "rewrite={}|{}", rewrite.rewrite_id, rewrite.audit_hash).unwrap();
    }
    crate::fnv1a64_hex(out.as_bytes())
}

#[cfg(test)]
#[path = "final_executable_macho_shell_image_tests.rs"]
mod tests;
