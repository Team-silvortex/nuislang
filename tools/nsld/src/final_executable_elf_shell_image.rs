use super::{
    build_elf_amd64_shell_layout_plan, build_elf_amd64_shell_layout_plan_with_dynamic_plan,
    image_encoding::{encode_elf_amd64_shell_tables, EncodedElfAmd64ShellTables},
    report::{
        shell_image_write_audit_hash, source_preservation_audit_hash,
        ElfAmd64ShellImageSerializationReport, ElfAmd64ShellImageWriteAudit,
        ElfAmd64ShellLayoutPlanReport, ElfAmd64ShellSourcePreservationAudit,
    },
    version::{append_version_names, encode_version_need_table, encode_version_symbol_table},
    ELF_AMD64_SHELL_LAYOUT_PLAN_CONTRACT,
};
use crate::{
    final_executable_elf_dynamic_plan::ElfAmd64DynamicDependencyPlanReport,
    final_executable_elf_layout_report::ElfAmd64PlacementBindingReport,
    final_executable_elf_materialization::application::platform::{
        application::ElfAmd64PlatformAppliedImage, ElfAmd64PlatformStructurePlanReport,
    },
    final_executable_elf_object::ElfAmd64ObjectLinkage,
    final_executable_elf_relocation_report::ElfAmd64RelocationApplicationReport,
};

pub(crate) const ELF_AMD64_SHELL_IMAGE_SERIALIZATION_CONTRACT: &str =
    "nuis-nsld-elf-amd64-shell-image-serialization-v1";
const SHT_NOBITS: u32 = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64SerializedShellImage {
    pub(crate) bytes: Vec<u8>,
    pub(crate) report: ElfAmd64ShellImageSerializationReport,
}

struct OccupiedSpan {
    start: usize,
    end: usize,
    label: String,
}

struct PreservationSummary {
    audits: Vec<ElfAmd64ShellSourcePreservationAudit>,
    file_backed_count: usize,
    zero_fill_count: usize,
    file_backed_bytes: usize,
    zero_fill_bytes: usize,
}

struct DynamicPayloads {
    interpreter: Vec<u8>,
    dynamic_strings: Vec<u8>,
    version_symbols: Vec<u8>,
    version_needs: Vec<u8>,
}

#[cfg(test)]
pub(crate) fn serialize_elf_amd64_shell_image(
    objects: &[ElfAmd64ObjectLinkage],
    placement: &ElfAmd64PlacementBindingReport,
    relocations: &ElfAmd64RelocationApplicationReport,
    platform_plan: &ElfAmd64PlatformStructurePlanReport,
    platform_applied: &ElfAmd64PlatformAppliedImage,
    shell: &ElfAmd64ShellLayoutPlanReport,
) -> Result<ElfAmd64SerializedShellImage, String> {
    serialize_elf_amd64_shell_image_internal(
        objects,
        placement,
        relocations,
        platform_plan,
        platform_applied,
        None,
        shell,
    )
}

pub(crate) fn serialize_elf_amd64_shell_image_with_dynamic_plan(
    objects: &[ElfAmd64ObjectLinkage],
    placement: &ElfAmd64PlacementBindingReport,
    relocations: &ElfAmd64RelocationApplicationReport,
    platform_plan: &ElfAmd64PlatformStructurePlanReport,
    platform_applied: &ElfAmd64PlatformAppliedImage,
    dynamic_plan: &ElfAmd64DynamicDependencyPlanReport,
    shell: &ElfAmd64ShellLayoutPlanReport,
) -> Result<ElfAmd64SerializedShellImage, String> {
    serialize_elf_amd64_shell_image_internal(
        objects,
        placement,
        relocations,
        platform_plan,
        platform_applied,
        Some(dynamic_plan),
        shell,
    )
}

fn serialize_elf_amd64_shell_image_internal(
    objects: &[ElfAmd64ObjectLinkage],
    placement: &ElfAmd64PlacementBindingReport,
    relocations: &ElfAmd64RelocationApplicationReport,
    platform_plan: &ElfAmd64PlatformStructurePlanReport,
    platform_applied: &ElfAmd64PlatformAppliedImage,
    dynamic_plan: Option<&ElfAmd64DynamicDependencyPlanReport>,
    shell: &ElfAmd64ShellLayoutPlanReport,
) -> Result<ElfAmd64SerializedShellImage, String> {
    validate_envelope(
        objects,
        placement,
        relocations,
        platform_plan,
        platform_applied,
        dynamic_plan,
        shell,
    )?;
    let tables = encode_elf_amd64_shell_tables(shell)?;
    let dynamic_payloads = encode_dynamic_payloads(platform_applied, shell)?;
    let mut bytes = vec![0; shell.planned_file_span_bytes];
    let source_file = checked_slice(
        &platform_applied.bytes,
        0,
        platform_applied.report.applied_file_span_bytes,
        "platform file image",
    )?;
    bytes
        .get_mut(..source_file.len())
        .ok_or_else(|| "ELF shell image truncates the platform file image".to_owned())?
        .copy_from_slice(source_file);

    let mut occupied = Vec::new();
    let mut writes = Vec::new();
    write_tables(
        &mut bytes,
        &mut occupied,
        &mut writes,
        shell,
        &tables,
        &dynamic_payloads,
    )?;
    let expected_shell_write_count = 4
        + usize::from(!tables.dynamic_entries.is_empty())
        + usize::from(!dynamic_payloads.interpreter.is_empty())
        + usize::from(!dynamic_payloads.dynamic_strings.is_empty())
        + usize::from(!dynamic_payloads.version_symbols.is_empty())
        + usize::from(!dynamic_payloads.version_needs.is_empty());
    if writes.len() != expected_shell_write_count {
        return Err("ELF shell write coverage drift".to_owned());
    }
    let preserved_platform_file_bytes =
        verify_platform_prefix_preserved(source_file, &bytes, &occupied)?;
    let preservation = audit_source_spans(platform_applied, shell, &bytes, &occupied)?;
    let shell_image_hash = crate::fnv1a64_hex(&bytes);
    let status = if shell.dynamic_entries.is_empty() {
        "serialized-static-private-image"
    } else {
        "serialized-private-image-with-external-resolution-boundary"
    };
    let mut report = ElfAmd64ShellImageSerializationReport {
        contract: ELF_AMD64_SHELL_IMAGE_SERIALIZATION_CONTRACT,
        status: status.to_owned(),
        serialization_ledger_hash: String::new(),
        shell_layout_plan_hash: shell.plan_hash.clone(),
        platform_application_ledger_hash: platform_applied.report.application_ledger_hash.clone(),
        source_file_image_hash: platform_applied.report.applied_file_image_hash.clone(),
        source_memory_image_hash: platform_applied.report.applied_memory_image_hash.clone(),
        shell_image_hash,
        shell_image_span_bytes: bytes.len(),
        copied_platform_file_bytes: source_file.len(),
        preserved_platform_file_bytes,
        header_bytes: tables.header.len(),
        program_header_bytes: tables.program_headers.len(),
        dynamic_table_bytes: tables.dynamic_entries.len(),
        section_name_table_bytes: tables.section_names.len(),
        section_header_bytes: tables.section_headers.len(),
        expected_shell_write_count,
        applied_shell_write_count: writes.len(),
        source_preservation_count: preservation.audits.len(),
        file_backed_source_span_count: preservation.file_backed_count,
        zero_fill_source_span_count: preservation.zero_fill_count,
        preserved_file_source_bytes: preservation.file_backed_bytes,
        preserved_zero_fill_bytes: preservation.zero_fill_bytes,
        publication_status: "private-not-published".to_owned(),
        writes,
        source_preservations: preservation.audits,
    };
    report.serialization_ledger_hash = crate::fnv1a64_hex(report.canonical_ledger().as_bytes());
    Ok(ElfAmd64SerializedShellImage { bytes, report })
}

fn encode_dynamic_payloads(
    platform_applied: &ElfAmd64PlatformAppliedImage,
    shell: &ElfAmd64ShellLayoutPlanReport,
) -> Result<DynamicPayloads, String> {
    let Some(interpreter_path) = shell.interpreter_path.as_deref() else {
        if shell.interpreter_identity.is_some()
            || shell.interpreter_file_offset.is_some()
            || shell.interpreter_virtual_address.is_some()
            || shell.interpreter_bytes != 0
            || shell.dynamic_string_source_image_offset.is_some()
            || shell.dynamic_string_source_bytes != 0
            || !shell.needed_libraries.is_empty()
            || !shell.version_symbols.is_empty()
            || !shell.version_needs.is_empty()
        {
            return Err("ELF shell dynamic payload plan is incomplete".to_owned());
        }
        return Ok(DynamicPayloads {
            interpreter: Vec::new(),
            dynamic_strings: Vec::new(),
            version_symbols: Vec::new(),
            version_needs: Vec::new(),
        });
    };
    if interpreter_path.as_bytes().contains(&0)
        || shell
            .interpreter_identity
            .as_deref()
            .is_none_or(str::is_empty)
    {
        return Err("ELF shell interpreter identity is invalid".to_owned());
    }
    let mut interpreter = interpreter_path.as_bytes().to_vec();
    interpreter.push(0);
    if interpreter.len() != shell.interpreter_bytes {
        return Err("ELF shell interpreter payload width drift".to_owned());
    }
    let source_offset = shell
        .dynamic_string_source_image_offset
        .ok_or_else(|| "ELF shell dynamic string source is absent".to_owned())?;
    let mut dynamic_strings = checked_slice(
        &platform_applied.bytes,
        source_offset,
        shell.dynamic_string_source_bytes,
        "platform dynamic string source",
    )?
    .to_vec();
    for needed in &shell.needed_libraries {
        if needed.dynamic_string_offset != dynamic_strings.len()
            || needed.needed_name.is_empty()
            || needed.needed_name.as_bytes().contains(&0)
        {
            return Err(format!(
                "ELF shell needed library `{}` has an invalid string slot",
                needed.needed_id
            ));
        }
        dynamic_strings.extend_from_slice(needed.needed_name.as_bytes());
        dynamic_strings.push(0);
    }
    append_version_names(&mut dynamic_strings, &shell.version_needs)?;
    let dynstr = shell
        .sections
        .iter()
        .find(|section| section.section_name == ".dynstr")
        .ok_or_else(|| "ELF shell final dynamic string section is absent".to_owned())?;
    if dynstr.source_kind != "shell-final-dynamic-string-table"
        || dynstr.file_size_bytes != dynamic_strings.len()
        || dynstr.memory_size_bytes != dynamic_strings.len()
    {
        return Err("ELF shell final dynamic string layout drift".to_owned());
    }
    let version_symbols =
        encode_version_symbol_table(shell.version_symbol_table_bytes, &shell.version_symbols)?;
    let version_needs =
        encode_version_need_table(shell.version_need_table_bytes, &shell.version_needs)?;
    Ok(DynamicPayloads {
        interpreter,
        dynamic_strings,
        version_symbols,
        version_needs,
    })
}

fn validate_envelope(
    objects: &[ElfAmd64ObjectLinkage],
    placement: &ElfAmd64PlacementBindingReport,
    relocations: &ElfAmd64RelocationApplicationReport,
    platform_plan: &ElfAmd64PlatformStructurePlanReport,
    platform_applied: &ElfAmd64PlatformAppliedImage,
    dynamic_plan: Option<&ElfAmd64DynamicDependencyPlanReport>,
    shell: &ElfAmd64ShellLayoutPlanReport,
) -> Result<(), String> {
    if shell.contract != ELF_AMD64_SHELL_LAYOUT_PLAN_CONTRACT {
        return Err("ELF shell serializer rejects the layout contract".to_owned());
    }
    let expected = match dynamic_plan {
        Some(plan) => build_elf_amd64_shell_layout_plan_with_dynamic_plan(
            objects,
            placement,
            relocations,
            platform_plan,
            platform_applied,
            plan,
        )?,
        None => build_elf_amd64_shell_layout_plan(
            objects,
            placement,
            relocations,
            platform_plan,
            platform_applied,
        )?,
    };
    if expected != *shell {
        return Err("ELF shell serializer rejects layout plan drift".to_owned());
    }
    if shell.plan_hash != crate::fnv1a64_hex(shell.canonical_plan().as_bytes())
        || shell.platform_application_ledger_hash != platform_applied.report.application_ledger_hash
        || shell.platform_image_hash != platform_applied.report.applied_memory_image_hash
        || shell.applied_file_span_bytes != platform_applied.report.applied_file_span_bytes
        || shell.applied_memory_span_bytes != platform_applied.report.applied_memory_span_bytes
        || shell.planned_file_span_bytes < shell.applied_file_span_bytes
    {
        return Err("ELF shell serializer rejects plan lineage drift".to_owned());
    }
    let source_file = checked_slice(
        &platform_applied.bytes,
        0,
        platform_applied.report.applied_file_span_bytes,
        "platform file image",
    )?;
    if platform_applied.bytes.len() != platform_applied.report.applied_memory_span_bytes
        || crate::fnv1a64_hex(source_file) != platform_applied.report.applied_file_image_hash
        || crate::fnv1a64_hex(&platform_applied.bytes)
            != platform_applied.report.applied_memory_image_hash
        || platform_applied.report.application_ledger_hash
            != crate::fnv1a64_hex(platform_applied.report.canonical_ledger().as_bytes())
    {
        return Err("ELF shell serializer rejects platform image drift".to_owned());
    }
    Ok(())
}

fn write_tables(
    image: &mut [u8],
    occupied: &mut Vec<OccupiedSpan>,
    writes: &mut Vec<ElfAmd64ShellImageWriteAudit>,
    shell: &ElfAmd64ShellLayoutPlanReport,
    tables: &EncodedElfAmd64ShellTables,
    dynamic_payloads: &DynamicPayloads,
) -> Result<(), String> {
    write_shell_region(
        image,
        occupied,
        writes,
        shell,
        shell.elf_header_file_offset,
        &tables.header,
        "elf64-header",
    )?;
    write_shell_region(
        image,
        occupied,
        writes,
        shell,
        shell.program_header_table_file_offset,
        &tables.program_headers,
        "program-header-table",
    )?;
    if let Some(offset) = shell.interpreter_file_offset {
        write_shell_region(
            image,
            occupied,
            writes,
            shell,
            offset,
            &dynamic_payloads.interpreter,
            "interpreter-path",
        )?;
    } else if !dynamic_payloads.interpreter.is_empty() {
        return Err("ELF shell interpreter bytes have no planned coordinate".to_owned());
    }
    if !dynamic_payloads.dynamic_strings.is_empty() {
        let dynstr = shell
            .sections
            .iter()
            .find(|section| section.section_name == ".dynstr")
            .ok_or_else(|| "ELF shell final dynamic string section is absent".to_owned())?;
        write_shell_region(
            image,
            occupied,
            writes,
            shell,
            dynstr.file_offset,
            &dynamic_payloads.dynamic_strings,
            "final-dynamic-string-table",
        )?;
    }
    if let Some(offset) = shell.version_symbol_table_file_offset {
        write_shell_region(
            image,
            occupied,
            writes,
            shell,
            offset,
            &dynamic_payloads.version_symbols,
            "gnu-version-symbol-table",
        )?;
    } else if !dynamic_payloads.version_symbols.is_empty() {
        return Err("ELF shell version-symbol bytes have no planned coordinate".to_owned());
    }
    if let Some(offset) = shell.version_need_table_file_offset {
        write_shell_region(
            image,
            occupied,
            writes,
            shell,
            offset,
            &dynamic_payloads.version_needs,
            "gnu-version-need-table",
        )?;
    } else if !dynamic_payloads.version_needs.is_empty() {
        return Err("ELF shell version-need bytes have no planned coordinate".to_owned());
    }
    if let Some(offset) = shell.dynamic_table_file_offset {
        write_shell_region(
            image,
            occupied,
            writes,
            shell,
            offset,
            &tables.dynamic_entries,
            "dynamic-table",
        )?;
    } else if !tables.dynamic_entries.is_empty() {
        return Err("ELF shell dynamic bytes have no planned coordinate".to_owned());
    }
    write_shell_region(
        image,
        occupied,
        writes,
        shell,
        shell.section_name_table_file_offset,
        &tables.section_names,
        "section-name-table",
    )?;
    write_shell_region(
        image,
        occupied,
        writes,
        shell,
        shell.section_header_table_file_offset,
        &tables.section_headers,
        "section-header-table",
    )
}

fn write_shell_region(
    image: &mut [u8],
    occupied: &mut Vec<OccupiedSpan>,
    writes: &mut Vec<ElfAmd64ShellImageWriteAudit>,
    shell: &ElfAmd64ShellLayoutPlanReport,
    offset: usize,
    encoded: &[u8],
    write_kind: &str,
) -> Result<(), String> {
    if encoded.is_empty() {
        return Err(format!("ELF shell `{write_kind}` write is empty"));
    }
    let end = checked_end(offset, encoded.len(), write_kind)?;
    let source = image
        .get(offset..end)
        .ok_or_else(|| format!("ELF shell `{write_kind}` exceeds the private image"))?;
    if let Some(previous) = occupied
        .iter()
        .find(|span| offset < span.end && span.start < end)
    {
        return Err(format!(
            "ELF shell `{write_kind}` overlaps shell write `{}`",
            previous.label
        ));
    }
    if source.iter().any(|byte| *byte != 0) {
        return Err(format!(
            "ELF shell `{write_kind}` overwrites a nonzero platform byte"
        ));
    }
    let source_bytes_hash = crate::fnv1a64_hex(source);
    let encoded_bytes_hash = crate::fnv1a64_hex(encoded);
    image[offset..end].copy_from_slice(encoded);
    let post_write_bytes_hash = crate::fnv1a64_hex(&image[offset..end]);
    if encoded_bytes_hash != post_write_bytes_hash {
        return Err(format!("ELF shell `{write_kind}` post-write hash drift"));
    }
    let mut audit = ElfAmd64ShellImageWriteAudit {
        write_id: format!("elf-amd64-shell-image-write-{:04}", writes.len()),
        write_kind: write_kind.to_owned(),
        file_offset: offset,
        width_bytes: encoded.len(),
        source_bytes_hash,
        encoded_bytes_hash,
        post_write_bytes_hash,
        status: "applied-write-once".to_owned(),
        audit_hash: String::new(),
    };
    audit.audit_hash = shell_image_write_audit_hash(
        &shell.plan_hash,
        &shell.platform_application_ledger_hash,
        &audit,
    );
    occupied.push(OccupiedSpan {
        start: offset,
        end,
        label: write_kind.to_owned(),
    });
    writes.push(audit);
    Ok(())
}

fn verify_platform_prefix_preserved(
    source: &[u8],
    result: &[u8],
    occupied: &[OccupiedSpan],
) -> Result<usize, String> {
    let result_prefix = result
        .get(..source.len())
        .ok_or_else(|| "ELF shell result truncates the platform prefix".to_owned())?;
    let mut preserved = 0usize;
    for (offset, (before, after)) in source.iter().zip(result_prefix).enumerate() {
        if occupied
            .iter()
            .any(|span| (span.start..span.end).contains(&offset))
        {
            continue;
        }
        if before != after {
            return Err(format!(
                "ELF shell changed platform byte {offset} outside a shell write"
            ));
        }
        preserved += 1;
    }
    Ok(preserved)
}

fn audit_source_spans(
    applied: &ElfAmd64PlatformAppliedImage,
    shell: &ElfAmd64ShellLayoutPlanReport,
    result: &[u8],
    occupied: &[OccupiedSpan],
) -> Result<PreservationSummary, String> {
    let mut summary = PreservationSummary {
        audits: Vec::new(),
        file_backed_count: 0,
        zero_fill_count: 0,
        file_backed_bytes: 0,
        zero_fill_bytes: 0,
    };
    for section in shell
        .sections
        .iter()
        .filter(|section| section.source_image_offset.is_some())
    {
        let source_offset = section.source_image_offset.unwrap();
        let source = checked_slice(
            &applied.bytes,
            source_offset,
            section.source_size_bytes,
            &section.section_id,
        )?;
        let (kind, result_offset, result_bytes, status) = if section.file_size_bytes > 0 {
            if section.file_size_bytes != section.source_size_bytes
                || overlaps_shell_write(section.file_offset, section.file_size_bytes, occupied)?
            {
                return Err(format!(
                    "ELF shell source section `{}` has an invalid file preservation span",
                    section.section_id
                ));
            }
            let bytes = checked_slice(
                result,
                section.file_offset,
                section.file_size_bytes,
                &section.section_id,
            )?;
            summary.file_backed_count += 1;
            summary.file_backed_bytes = summary
                .file_backed_bytes
                .checked_add(source.len())
                .ok_or_else(|| "ELF shell preserved file-byte count overflows".to_owned())?;
            (
                "file-backed-byte-span",
                Some(section.file_offset),
                bytes,
                "preserved-byte-for-byte",
            )
        } else {
            if section.section_type != SHT_NOBITS
                || section.memory_size_bytes != section.source_size_bytes
                || source.iter().any(|byte| *byte != 0)
            {
                return Err(format!(
                    "ELF shell source section `{}` has an invalid zero-fill span",
                    section.section_id
                ));
            }
            summary.zero_fill_count += 1;
            summary.zero_fill_bytes = summary
                .zero_fill_bytes
                .checked_add(source.len())
                .ok_or_else(|| "ELF shell preserved zero-fill count overflows".to_owned())?;
            (
                "nobits-zero-fill-span",
                None,
                source,
                "preserved-as-nobits-zero-fill",
            )
        };
        let source_bytes_hash = crate::fnv1a64_hex(source);
        let result_bytes_hash = crate::fnv1a64_hex(result_bytes);
        if source_bytes_hash != result_bytes_hash {
            return Err(format!(
                "ELF shell source section `{}` changed during serialization",
                section.section_id
            ));
        }
        let mut audit = ElfAmd64ShellSourcePreservationAudit {
            preservation_id: format!(
                "elf-amd64-shell-source-preservation-{:04}",
                summary.audits.len()
            ),
            section_id: section.section_id.clone(),
            section_name: section.section_name.clone(),
            preservation_kind: kind.to_owned(),
            source_image_offset: source_offset,
            source_size_bytes: source.len(),
            result_file_offset: result_offset,
            result_size_bytes: result_bytes.len(),
            source_bytes_hash,
            result_bytes_hash,
            status: status.to_owned(),
            audit_hash: String::new(),
        };
        audit.audit_hash = source_preservation_audit_hash(
            &shell.plan_hash,
            &shell.platform_application_ledger_hash,
            &audit,
        );
        summary.audits.push(audit);
    }
    Ok(summary)
}

fn overlaps_shell_write(
    offset: usize,
    size: usize,
    occupied: &[OccupiedSpan],
) -> Result<bool, String> {
    let end = checked_end(offset, size, "source preservation")?;
    Ok(occupied
        .iter()
        .any(|span| offset < span.end && span.start < end))
}

fn checked_slice<'a>(
    bytes: &'a [u8],
    offset: usize,
    size: usize,
    label: &str,
) -> Result<&'a [u8], String> {
    let end = checked_end(offset, size, label)?;
    bytes
        .get(offset..end)
        .ok_or_else(|| format!("ELF shell `{label}` span exceeds its image"))
}

fn checked_end(offset: usize, size: usize, label: &str) -> Result<usize, String> {
    offset
        .checked_add(size)
        .ok_or_else(|| format!("ELF shell `{label}` span overflows"))
}
