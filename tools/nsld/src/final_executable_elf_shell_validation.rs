use super::{
    image::ELF_AMD64_SHELL_IMAGE_SERIALIZATION_CONTRACT,
    report::{
        dynamic_entry_audit_hash, program_header_audit_hash, section_audit_hash,
        shell_image_source_validation_audit_hash, shell_image_table_validation_audit_hash,
        ElfAmd64ShellImageSerializationReport, ElfAmd64ShellImageValidationReport,
        ElfAmd64ShellLayoutPlanReport,
    },
    validation_parser::parse_and_validate_elf_amd64_shell_image,
    validation_support::validate_serialization_evidence,
    ELF_AMD64_SHELL_LAYOUT_PLAN_CONTRACT,
};
use crate::final_executable_elf_materialization::application::platform::application::ElfAmd64PlatformAppliedImage;
use std::collections::BTreeSet;

pub(crate) const ELF_AMD64_SHELL_IMAGE_VALIDATION_CONTRACT: &str =
    "nuis-nsld-elf-amd64-shell-image-validation-v1";
pub(crate) const ELF_AMD64_PUBLICATION_ELIGIBILITY_CONTRACT: &str =
    "nuis-nsld-elf-amd64-publication-eligibility-v1";

pub(crate) fn validate_elf_amd64_shell_image(
    bytes: &[u8],
    platform: &ElfAmd64PlatformAppliedImage,
    shell: &ElfAmd64ShellLayoutPlanReport,
    serialization: &ElfAmd64ShellImageSerializationReport,
) -> Result<ElfAmd64ShellImageValidationReport, String> {
    validate_input_envelope(bytes, platform, shell, serialization)?;
    let parsed = parse_and_validate_elf_amd64_shell_image(bytes, shell)?;
    let evidence = validate_serialization_evidence(bytes, platform, shell, serialization, &parsed)?;
    let dynamic_boundary = !shell.dynamic_entries.is_empty();
    let (publication_status, publication_blockers) = if dynamic_boundary {
        (
            "blocked-os-loader-and-external-resolution-pending",
            vec![
                "os-loader-probe-pending".to_owned(),
                "registered-external-resolution-provenance-pending".to_owned(),
            ],
        )
    } else {
        (
            "blocked-os-loader-probe-pending",
            vec!["os-loader-probe-pending".to_owned()],
        )
    };
    let expected_source_count = shell
        .sections
        .iter()
        .filter(|section| section.source_image_offset.is_some())
        .count();
    let expected_table_count = 4 + usize::from(dynamic_boundary);
    if parsed.tables.len() != expected_table_count
        || evidence.tables.len() != expected_table_count
        || evidence.sources.len() != expected_source_count
    {
        return Err("ELF shell validation coverage assembly drift".to_owned());
    }
    let mut report = ElfAmd64ShellImageValidationReport {
        contract: ELF_AMD64_SHELL_IMAGE_VALIDATION_CONTRACT,
        status: "independently-validated-private-image".to_owned(),
        validation_ledger_hash: String::new(),
        shell_layout_plan_hash: shell.plan_hash.clone(),
        serialization_ledger_hash: serialization.serialization_ledger_hash.clone(),
        platform_application_ledger_hash: platform.report.application_ledger_hash.clone(),
        shell_image_hash: crate::fnv1a64_hex(bytes),
        shell_image_span_bytes: bytes.len(),
        header_valid: true,
        expected_table_count,
        verified_table_count: evidence.tables.len(),
        program_header_count: parsed.program_header_count,
        load_segment_count: parsed.load_segment_count,
        dynamic_segment_count: parsed.dynamic_segment_count,
        dynamic_entry_count: parsed.dynamic_entry_count,
        section_header_count: parsed.section_header_count,
        section_name_count: parsed.section_name_count,
        entry_program_header_index: parsed.entry_program_header_index,
        expected_shell_write_count: serialization.expected_shell_write_count,
        verified_shell_write_count: evidence.tables.len(),
        expected_source_validation_count: expected_source_count,
        verified_source_validation_count: evidence.sources.len(),
        preserved_platform_file_bytes: evidence.preserved_platform_file_bytes,
        unexplained_platform_change_count: 0,
        publication_eligibility_contract: ELF_AMD64_PUBLICATION_ELIGIBILITY_CONTRACT,
        publication_eligibility_status: publication_status.to_owned(),
        publication_eligible: false,
        publication_blockers,
        tables: evidence.tables,
        sources: evidence.sources,
    };
    report.validation_ledger_hash = crate::fnv1a64_hex(report.canonical_ledger().as_bytes());
    validate_elf_amd64_shell_image_validation_report(&report)?;
    Ok(report)
}

pub(crate) fn validate_elf_amd64_shell_image_validation_report(
    report: &ElfAmd64ShellImageValidationReport,
) -> Result<(), String> {
    let dynamic = match (report.dynamic_segment_count, report.dynamic_entry_count) {
        (0, 0) => false,
        (1, count) if count > 0 => true,
        _ => return Err("ELF shell validation report has an invalid dynamic shape".to_owned()),
    };
    let expected_table_kinds = if dynamic {
        vec![
            ("elf64-header", 1),
            ("program-header-table", report.program_header_count),
            ("dynamic-table", report.dynamic_entry_count),
            ("section-name-table", report.section_name_count),
            ("section-header-table", report.section_header_count),
        ]
    } else {
        vec![
            ("elf64-header", 1),
            ("program-header-table", report.program_header_count),
            ("section-name-table", report.section_name_count),
            ("section-header-table", report.section_header_count),
        ]
    };
    let (publication_status, publication_blockers): (&str, &[&str]) = if dynamic {
        (
            "blocked-os-loader-and-external-resolution-pending",
            &[
                "os-loader-probe-pending",
                "registered-external-resolution-provenance-pending",
            ],
        )
    } else {
        (
            "blocked-os-loader-probe-pending",
            &["os-loader-probe-pending"],
        )
    };
    if report.contract != ELF_AMD64_SHELL_IMAGE_VALIDATION_CONTRACT
        || report.status != "independently-validated-private-image"
        || !report.header_valid
        || report.shell_image_span_bytes == 0
        || report.program_header_count == 0
        || report.load_segment_count == 0
        || report.load_segment_count > report.program_header_count
        || report.section_header_count == 0
        || report.section_name_count != report.section_header_count
        || report.entry_program_header_index >= report.program_header_count
        || report.expected_table_count != expected_table_kinds.len()
        || report.verified_table_count != report.tables.len()
        || report.expected_table_count != report.verified_table_count
        || report.expected_shell_write_count != report.tables.len()
        || report.verified_shell_write_count != report.tables.len()
        || report.expected_source_validation_count != report.sources.len()
        || report.verified_source_validation_count != report.sources.len()
        || report.unexplained_platform_change_count != 0
        || report.publication_eligibility_contract != ELF_AMD64_PUBLICATION_ELIGIBILITY_CONTRACT
        || report.publication_eligibility_status != publication_status
        || report.publication_eligible
        || report
            .publication_blockers
            .iter()
            .map(String::as_str)
            .ne(publication_blockers.iter().copied())
    {
        return Err("ELF shell validation report envelope drift".to_owned());
    }
    for (index, (table, (kind, records))) in
        report.tables.iter().zip(expected_table_kinds).enumerate()
    {
        if table.table_id != format!("elf-amd64-shell-image-table-validation-{index:04}")
            || table.table_kind != kind
            || table.width_bytes == 0
            || table.expected_record_count != records
            || table.verified_record_count != records
            || table.status != "parsed-and-write-audit-verified"
            || table.audit_hash
                != shell_image_table_validation_audit_hash(
                    &report.shell_layout_plan_hash,
                    &report.serialization_ledger_hash,
                    &report.shell_image_hash,
                    table,
                )
        {
            return Err(format!("ELF shell validation report table {index} drift"));
        }
    }
    let mut section_ids = BTreeSet::new();
    for (index, source) in report.sources.iter().enumerate() {
        let preservation_shape_valid = match source.preservation_kind.as_str() {
            "file-backed-byte-span" => {
                source.result_file_offset.is_some()
                    && source.source_size_bytes == source.result_size_bytes
            }
            "nobits-zero-fill-span" => {
                source.result_file_offset.is_none()
                    && source.source_size_bytes == source.result_size_bytes
            }
            _ => false,
        };
        if source.validation_id != format!("elf-amd64-shell-image-source-validation-{index:04}")
            || !section_ids.insert(source.section_id.as_str())
            || !preservation_shape_valid
            || source.source_bytes_hash != source.result_bytes_hash
            || source.status != "independently-preserved"
            || source.audit_hash
                != shell_image_source_validation_audit_hash(
                    &report.shell_layout_plan_hash,
                    &report.serialization_ledger_hash,
                    &report.shell_image_hash,
                    source,
                )
        {
            return Err(format!("ELF shell validation report source {index} drift"));
        }
    }
    if report.validation_ledger_hash != crate::fnv1a64_hex(report.canonical_ledger().as_bytes()) {
        return Err("ELF shell validation report ledger drift".to_owned());
    }
    Ok(())
}

fn validate_input_envelope(
    bytes: &[u8],
    platform: &ElfAmd64PlatformAppliedImage,
    shell: &ElfAmd64ShellLayoutPlanReport,
    serialization: &ElfAmd64ShellImageSerializationReport,
) -> Result<(), String> {
    if shell.contract != ELF_AMD64_SHELL_LAYOUT_PLAN_CONTRACT
        || serialization.contract != ELF_AMD64_SHELL_IMAGE_SERIALIZATION_CONTRACT
    {
        return Err("ELF shell validation rejects an upstream contract".to_owned());
    }
    if shell.plan_hash != crate::fnv1a64_hex(shell.canonical_plan().as_bytes())
        || serialization.shell_layout_plan_hash != shell.plan_hash
        || serialization.platform_application_ledger_hash != platform.report.application_ledger_hash
        || bytes.len() != shell.planned_file_span_bytes
    {
        return Err("ELF shell validation rejects upstream lineage drift".to_owned());
    }
    let platform_file = platform
        .bytes
        .get(..platform.report.applied_file_span_bytes)
        .ok_or_else(|| "ELF shell validation platform file span is invalid".to_owned())?;
    if platform.bytes.len() != platform.report.applied_memory_span_bytes
        || crate::fnv1a64_hex(platform_file) != platform.report.applied_file_image_hash
        || crate::fnv1a64_hex(&platform.bytes) != platform.report.applied_memory_image_hash
        || platform.report.application_ledger_hash
            != crate::fnv1a64_hex(platform.report.canonical_ledger().as_bytes())
    {
        return Err("ELF shell validation rejects platform image drift".to_owned());
    }
    validate_layout_audits(shell)
}

fn validate_layout_audits(shell: &ElfAmd64ShellLayoutPlanReport) -> Result<(), String> {
    if shell.program_header_count != shell.program_headers.len()
        || shell.section_header_count != shell.sections.len()
        || shell.dynamic_table_entry_count != shell.dynamic_entries.len()
    {
        return Err("ELF shell validation rejects layout count drift".to_owned());
    }
    for header in &shell.program_headers {
        if header.audit_hash
            != program_header_audit_hash(&shell.platform_application_ledger_hash, header)
        {
            return Err(format!(
                "ELF shell validation rejects program audit `{}`",
                header.program_header_id
            ));
        }
    }
    for section in &shell.sections {
        if section.audit_hash
            != section_audit_hash(&shell.platform_application_ledger_hash, section)
        {
            return Err(format!(
                "ELF shell validation rejects section audit `{}`",
                section.section_id
            ));
        }
    }
    for entry in &shell.dynamic_entries {
        if entry.audit_hash
            != dynamic_entry_audit_hash(&shell.platform_application_ledger_hash, entry)
        {
            return Err(format!(
                "ELF shell validation rejects dynamic audit `{}`",
                entry.dynamic_entry_id
            ));
        }
    }
    Ok(())
}
