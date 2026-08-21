use super::{
    image::ELF_AMD64_SHELL_IMAGE_SERIALIZATION_CONTRACT,
    report::{
        shell_image_source_validation_audit_hash, shell_image_table_validation_audit_hash,
        shell_image_write_audit_hash, source_preservation_audit_hash,
        ElfAmd64ShellImageSerializationReport, ElfAmd64ShellImageSourceValidation,
        ElfAmd64ShellImageTableValidation, ElfAmd64ShellLayoutPlanReport,
    },
    validation_parser::{ParsedElfAmd64ShellImage, ParsedTableEvidence},
};
use crate::final_executable_elf_materialization::application::platform::application::ElfAmd64PlatformAppliedImage;

const SHT_NOBITS: u32 = 8;

pub(super) struct ValidatedSerializationEvidence {
    pub(super) tables: Vec<ElfAmd64ShellImageTableValidation>,
    pub(super) sources: Vec<ElfAmd64ShellImageSourceValidation>,
    pub(super) preserved_platform_file_bytes: usize,
}

struct SourceCoverage {
    validations: Vec<ElfAmd64ShellImageSourceValidation>,
    file_backed_count: usize,
    zero_fill_count: usize,
    file_backed_bytes: usize,
    zero_fill_bytes: usize,
}

pub(super) fn validate_serialization_evidence(
    bytes: &[u8],
    platform: &ElfAmd64PlatformAppliedImage,
    shell: &ElfAmd64ShellLayoutPlanReport,
    serialization: &ElfAmd64ShellImageSerializationReport,
    parsed: &ParsedElfAmd64ShellImage,
) -> Result<ValidatedSerializationEvidence, String> {
    if serialization.contract != ELF_AMD64_SHELL_IMAGE_SERIALIZATION_CONTRACT {
        return Err("ELF shell validation rejects the serialization contract".to_owned());
    }
    let image_hash = crate::fnv1a64_hex(bytes);
    let tables = validate_shell_writes(
        bytes,
        platform,
        shell,
        serialization,
        &parsed.tables,
        &image_hash,
    )?;
    let preserved_platform_file_bytes = validate_image_baseline(bytes, platform, serialization)?;
    let source_coverage =
        validate_source_spans(bytes, platform, shell, serialization, &image_hash)?;
    validate_serialization_report(
        bytes,
        platform,
        shell,
        serialization,
        parsed,
        preserved_platform_file_bytes,
        &source_coverage,
    )?;
    Ok(ValidatedSerializationEvidence {
        tables,
        sources: source_coverage.validations,
        preserved_platform_file_bytes,
    })
}

fn validate_shell_writes(
    bytes: &[u8],
    platform: &ElfAmd64PlatformAppliedImage,
    shell: &ElfAmd64ShellLayoutPlanReport,
    serialization: &ElfAmd64ShellImageSerializationReport,
    parsed_tables: &[ParsedTableEvidence],
    image_hash: &str,
) -> Result<Vec<ElfAmd64ShellImageTableValidation>, String> {
    if parsed_tables.len() != serialization.writes.len() {
        return Err("ELF shell validation table/write coverage drift".to_owned());
    }
    let mut spans = Vec::new();
    let mut validations = Vec::with_capacity(parsed_tables.len());
    for (index, (table, write)) in parsed_tables.iter().zip(&serialization.writes).enumerate() {
        let expected_write_id = format!("elf-amd64-shell-image-write-{index:04}");
        if write.write_id != expected_write_id
            || write.write_kind != table.table_kind
            || write.file_offset != table.file_offset
            || write.width_bytes != table.width_bytes
            || write.encoded_bytes_hash != table.bytes_hash
            || write.post_write_bytes_hash != table.bytes_hash
            || write.status != "applied-write-once"
            || write.audit_hash
                != shell_image_write_audit_hash(
                    &shell.plan_hash,
                    &shell.platform_application_ledger_hash,
                    write,
                )
        {
            return Err(format!(
                "ELF shell validation rejects write audit `{}`",
                write.write_id
            ));
        }
        let end = checked_end(write.file_offset, write.width_bytes, &write.write_kind)?;
        if spans
            .iter()
            .any(|(start, stop)| write.file_offset < *stop && *start < end)
        {
            return Err("ELF shell validation finds overlapping write audits".to_owned());
        }
        let baseline = platform_baseline(
            platform,
            write.file_offset,
            write.width_bytes,
            &write.write_kind,
        )?;
        if baseline.iter().any(|byte| *byte != 0)
            || crate::fnv1a64_hex(&baseline) != write.source_bytes_hash
        {
            return Err(format!(
                "ELF shell validation rejects write-before bytes for `{}`",
                write.write_kind
            ));
        }
        let actual = checked_slice(
            bytes,
            write.file_offset,
            write.width_bytes,
            &write.write_kind,
        )?;
        if crate::fnv1a64_hex(actual) != table.bytes_hash {
            return Err(format!(
                "ELF shell validation rejects actual bytes for `{}`",
                write.write_kind
            ));
        }
        let mut validation = ElfAmd64ShellImageTableValidation {
            table_id: format!("elf-amd64-shell-image-table-validation-{index:04}"),
            table_kind: table.table_kind.to_owned(),
            file_offset: table.file_offset,
            width_bytes: table.width_bytes,
            expected_record_count: table.record_count,
            verified_record_count: table.record_count,
            bytes_hash: table.bytes_hash.clone(),
            status: "parsed-and-write-audit-verified".to_owned(),
            audit_hash: String::new(),
        };
        validation.audit_hash = shell_image_table_validation_audit_hash(
            &shell.plan_hash,
            &serialization.serialization_ledger_hash,
            image_hash,
            &validation,
        );
        spans.push((write.file_offset, end));
        validations.push(validation);
    }
    Ok(validations)
}

fn validate_image_baseline(
    bytes: &[u8],
    platform: &ElfAmd64PlatformAppliedImage,
    serialization: &ElfAmd64ShellImageSerializationReport,
) -> Result<usize, String> {
    let source = checked_slice(
        &platform.bytes,
        0,
        platform.report.applied_file_span_bytes,
        "platform file image",
    )?;
    let mut preserved = 0usize;
    let mut unexplained = 0usize;
    for (offset, after) in bytes.iter().enumerate() {
        if serialization.writes.iter().any(|write| {
            write
                .file_offset
                .checked_add(write.width_bytes)
                .is_some_and(|end| (write.file_offset..end).contains(&offset))
        }) {
            continue;
        }
        let before = source.get(offset).copied().unwrap_or(0);
        if before == *after {
            if offset < source.len() {
                preserved += 1;
            }
        } else {
            unexplained += 1;
        }
    }
    if unexplained != 0 {
        return Err(format!(
            "ELF shell validation found {unexplained} unexplained platform-prefix changes or zero-tail bytes"
        ));
    }
    Ok(preserved)
}

fn validate_source_spans(
    bytes: &[u8],
    platform: &ElfAmd64PlatformAppliedImage,
    shell: &ElfAmd64ShellLayoutPlanReport,
    serialization: &ElfAmd64ShellImageSerializationReport,
    image_hash: &str,
) -> Result<SourceCoverage, String> {
    let source_sections = shell
        .sections
        .iter()
        .filter(|section| section.source_image_offset.is_some())
        .collect::<Vec<_>>();
    if source_sections.len() != serialization.source_preservations.len() {
        return Err("ELF shell validation source-preservation coverage drift".to_owned());
    }
    let mut coverage = SourceCoverage {
        validations: Vec::with_capacity(source_sections.len()),
        file_backed_count: 0,
        zero_fill_count: 0,
        file_backed_bytes: 0,
        zero_fill_bytes: 0,
    };
    for (index, (section, serialized)) in source_sections
        .into_iter()
        .zip(&serialization.source_preservations)
        .enumerate()
    {
        let source_offset = section.source_image_offset.unwrap();
        let source = checked_slice(
            &platform.bytes,
            source_offset,
            section.source_size_bytes,
            &section.section_id,
        )?;
        let (kind, result_offset, result, status) = if section.file_size_bytes > 0 {
            if section.file_size_bytes != section.source_size_bytes
                || overlaps_writes(
                    section.file_offset,
                    section.file_size_bytes,
                    &serialization.writes,
                )?
            {
                return Err(format!(
                    "ELF shell validation rejects source section `{}` file span",
                    section.section_id
                ));
            }
            coverage.file_backed_count += 1;
            coverage.file_backed_bytes = coverage
                .file_backed_bytes
                .checked_add(source.len())
                .ok_or_else(|| "ELF source validation byte count overflows".to_owned())?;
            (
                "file-backed-byte-span",
                Some(section.file_offset),
                checked_slice(
                    bytes,
                    section.file_offset,
                    section.file_size_bytes,
                    &section.section_id,
                )?,
                "preserved-byte-for-byte",
            )
        } else {
            if section.section_type != SHT_NOBITS
                || section.memory_size_bytes != section.source_size_bytes
                || source.iter().any(|byte| *byte != 0)
            {
                return Err(format!(
                    "ELF shell validation rejects source section `{}` zero-fill span",
                    section.section_id
                ));
            }
            coverage.zero_fill_count += 1;
            coverage.zero_fill_bytes = coverage
                .zero_fill_bytes
                .checked_add(source.len())
                .ok_or_else(|| "ELF zero-fill validation byte count overflows".to_owned())?;
            (
                "nobits-zero-fill-span",
                None,
                source,
                "preserved-as-nobits-zero-fill",
            )
        };
        let source_hash = crate::fnv1a64_hex(source);
        let result_hash = crate::fnv1a64_hex(result);
        let expected_serialization_id = format!("elf-amd64-shell-source-preservation-{index:04}");
        if source_hash != result_hash
            || serialized.preservation_id != expected_serialization_id
            || serialized.section_id != section.section_id
            || serialized.section_name != section.section_name
            || serialized.preservation_kind != kind
            || serialized.source_image_offset != source_offset
            || serialized.source_size_bytes != source.len()
            || serialized.result_file_offset != result_offset
            || serialized.result_size_bytes != result.len()
            || serialized.source_bytes_hash != source_hash
            || serialized.result_bytes_hash != result_hash
            || serialized.status != status
            || serialized.audit_hash
                != source_preservation_audit_hash(
                    &shell.plan_hash,
                    &shell.platform_application_ledger_hash,
                    serialized,
                )
        {
            return Err(format!(
                "ELF shell validation rejects source preservation `{}`",
                serialized.preservation_id
            ));
        }
        let mut validation = ElfAmd64ShellImageSourceValidation {
            validation_id: format!("elf-amd64-shell-image-source-validation-{index:04}"),
            section_id: section.section_id.clone(),
            section_name: section.section_name.clone(),
            preservation_kind: kind.to_owned(),
            source_image_offset: source_offset,
            source_size_bytes: source.len(),
            result_file_offset: result_offset,
            result_size_bytes: result.len(),
            source_bytes_hash: source_hash,
            result_bytes_hash: result_hash,
            serialization_audit_hash: serialized.audit_hash.clone(),
            status: "independently-preserved".to_owned(),
            audit_hash: String::new(),
        };
        validation.audit_hash = shell_image_source_validation_audit_hash(
            &shell.plan_hash,
            &serialization.serialization_ledger_hash,
            image_hash,
            &validation,
        );
        coverage.validations.push(validation);
    }
    Ok(coverage)
}

#[allow(clippy::too_many_arguments)]
fn validate_serialization_report(
    bytes: &[u8],
    platform: &ElfAmd64PlatformAppliedImage,
    shell: &ElfAmd64ShellLayoutPlanReport,
    report: &ElfAmd64ShellImageSerializationReport,
    parsed: &ParsedElfAmd64ShellImage,
    preserved_platform_file_bytes: usize,
    sources: &SourceCoverage,
) -> Result<(), String> {
    let expected_status = if shell.dynamic_entries.is_empty() {
        "serialized-static-private-image"
    } else {
        "serialized-private-image-with-external-resolution-boundary"
    };
    if report.status != expected_status
        || report.shell_layout_plan_hash != shell.plan_hash
        || report.platform_application_ledger_hash != platform.report.application_ledger_hash
        || report.source_file_image_hash != platform.report.applied_file_image_hash
        || report.source_memory_image_hash != platform.report.applied_memory_image_hash
        || report.shell_image_hash != crate::fnv1a64_hex(bytes)
        || report.shell_image_span_bytes != bytes.len()
        || report.copied_platform_file_bytes != platform.report.applied_file_span_bytes
        || report.preserved_platform_file_bytes != preserved_platform_file_bytes
        || report.header_bytes != table_width(&parsed.tables, "elf64-header")?
        || report.program_header_bytes != table_width(&parsed.tables, "program-header-table")?
        || report.dynamic_table_bytes
            != optional_table_width(&parsed.tables, "dynamic-table").unwrap_or(0)
        || report.section_name_table_bytes != table_width(&parsed.tables, "section-name-table")?
        || report.section_header_bytes != table_width(&parsed.tables, "section-header-table")?
        || report.expected_shell_write_count != parsed.tables.len()
        || report.applied_shell_write_count != report.writes.len()
        || report.source_preservation_count != sources.validations.len()
        || report.file_backed_source_span_count != sources.file_backed_count
        || report.zero_fill_source_span_count != sources.zero_fill_count
        || report.preserved_file_source_bytes != sources.file_backed_bytes
        || report.preserved_zero_fill_bytes != sources.zero_fill_bytes
        || report.publication_status != "private-not-published"
        || report.serialization_ledger_hash
            != crate::fnv1a64_hex(report.canonical_ledger().as_bytes())
    {
        return Err("ELF shell validation rejects the serialization report envelope".to_owned());
    }
    Ok(())
}

fn platform_baseline(
    platform: &ElfAmd64PlatformAppliedImage,
    offset: usize,
    size: usize,
    label: &str,
) -> Result<Vec<u8>, String> {
    let end = checked_end(offset, size, label)?;
    let mut baseline = vec![0; size];
    let file_end = platform.report.applied_file_span_bytes;
    if offset < file_end {
        let copied_end = end.min(file_end);
        let copied = checked_slice(
            &platform.bytes,
            offset,
            copied_end - offset,
            "platform write baseline",
        )?;
        baseline[..copied.len()].copy_from_slice(copied);
    }
    Ok(baseline)
}

fn overlaps_writes(
    offset: usize,
    size: usize,
    writes: &[super::report::ElfAmd64ShellImageWriteAudit],
) -> Result<bool, String> {
    let end = checked_end(offset, size, "source span")?;
    Ok(writes.iter().any(|write| {
        write
            .file_offset
            .checked_add(write.width_bytes)
            .is_some_and(|write_end| offset < write_end && write.file_offset < end)
    }))
}

fn table_width(tables: &[ParsedTableEvidence], kind: &str) -> Result<usize, String> {
    optional_table_width(tables, kind)
        .ok_or_else(|| format!("ELF shell validation cannot find `{kind}`"))
}

fn optional_table_width(tables: &[ParsedTableEvidence], kind: &str) -> Option<usize> {
    tables
        .iter()
        .find(|table| table.table_kind == kind)
        .map(|table| table.width_bytes)
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
        .ok_or_else(|| format!("ELF shell validation `{label}` exceeds its image"))
}

fn checked_end(offset: usize, size: usize, label: &str) -> Result<usize, String> {
    offset
        .checked_add(size)
        .ok_or_else(|| format!("ELF shell validation `{label}` span overflows"))
}
