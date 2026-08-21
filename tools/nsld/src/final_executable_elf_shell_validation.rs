use super::{
    image::ELF_AMD64_SHELL_IMAGE_SERIALIZATION_CONTRACT,
    report::{
        dynamic_entry_audit_hash, program_header_audit_hash, section_audit_hash,
        ElfAmd64ShellImageSerializationReport, ElfAmd64ShellImageValidationReport,
        ElfAmd64ShellLayoutPlanReport,
    },
    validation_parser::parse_and_validate_elf_amd64_shell_image,
    validation_support::validate_serialization_evidence,
    ELF_AMD64_SHELL_LAYOUT_PLAN_CONTRACT,
};
use crate::final_executable_elf_materialization::application::platform::application::ElfAmd64PlatformAppliedImage;

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
    Ok(report)
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
