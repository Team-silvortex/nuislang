use crate::{
    final_executable_elf_dynamic_provenance::{
        validate_elf_amd64_dynamic_resolution_provenance_report,
        ElfAmd64DynamicResolutionProvenanceReport,
        ELF_AMD64_DYNAMIC_RESOLUTION_PROVENANCE_CONTRACT,
    },
    final_executable_elf_loader_probe_report::ElfAmd64LoaderProbeReport,
    final_executable_elf_shell::{
        validate_elf_amd64_shell_image_validation_report, ElfAmd64ShellImageValidationReport,
        ELF_AMD64_PUBLICATION_ELIGIBILITY_CONTRACT, ELF_AMD64_SHELL_IMAGE_VALIDATION_CONTRACT,
    },
    final_executable_loader_probe_runtime::{
        execute_isolated_loader_probe, LoaderProbeRuntimeObservation, LoaderProbeRuntimeRequest,
        LOADER_PROBE_MATERIALIZATION_KIND, LOADER_PROBE_TIMEOUT_MILLIS,
    },
};
use std::path::Path;

pub(crate) const ELF_AMD64_LOADER_PROBE_CONTRACT: &str = "nuis-nsld-elf-amd64-os-loader-probe-v1";

pub(crate) struct ElfAmd64LoaderProbeInput<'a> {
    pub(crate) bytes: &'a [u8],
    pub(crate) validation: &'a ElfAmd64ShellImageValidationReport,
    pub(crate) unresolved_external_symbol_count: usize,
    pub(crate) dynamic_provenance: Option<&'a ElfAmd64DynamicResolutionProvenanceReport>,
}

pub(crate) fn probe_elf_amd64_private_shell_image(
    input: ElfAmd64LoaderProbeInput<'_>,
    probe_root: &Path,
    execute: bool,
) -> Result<ElfAmd64LoaderProbeReport, String> {
    validate_input(&input)?;
    let host_supported = cfg!(all(target_os = "linux", target_arch = "x86_64"));
    let static_eligible = input.unresolved_external_symbol_count == 0
        && input.validation.dynamic_segment_count == 0
        && input.validation.dynamic_entry_count == 0;
    let dynamic_eligible = input.unresolved_external_symbol_count > 0
        && input.validation.dynamic_segment_count == 1
        && input.validation.dynamic_entry_count > 0
        && input
            .dynamic_provenance
            .is_some_and(|provenance| provenance.provenance_ready);
    let input_eligible = static_eligible || dynamic_eligible;
    let observation = if !input_eligible {
        LoaderProbeRuntimeObservation::blocked(
            "blocked-external-compatibility-input",
            "private-image-has-external-compatibility-bindings",
        )
    } else if !host_supported {
        LoaderProbeRuntimeObservation::blocked(
            "blocked-unsupported-probe-host",
            "unsupported-probe-host",
        )
    } else if !execute {
        LoaderProbeRuntimeObservation::blocked(
            "ready-explicit-apply-required",
            "explicit-loader-probe-apply-required",
        )
    } else {
        execute_isolated_loader_probe(LoaderProbeRuntimeRequest {
            bytes: input.bytes,
            probe_root,
            path_namespace: "elf-amd64",
        })?
    };
    let report = build_report(&input, execute, host_supported, input_eligible, observation);
    validate_elf_amd64_loader_probe_report(&report)?;
    Ok(report)
}

fn validate_input(input: &ElfAmd64LoaderProbeInput<'_>) -> Result<(), String> {
    validate_elf_amd64_shell_image_validation_report(input.validation)?;
    if input.bytes.len() != input.validation.shell_image_span_bytes
        || crate::fnv1a64_hex(input.bytes) != input.validation.shell_image_hash
    {
        return Err("ELF loader probe rejects private image drift".to_owned());
    }
    let dynamic = input.validation.dynamic_segment_count != 0;
    if dynamic != (input.unresolved_external_symbol_count != 0) {
        return Err("ELF loader probe rejects external-boundary lineage drift".to_owned());
    }
    if let Some(provenance) = input.dynamic_provenance {
        validate_elf_amd64_dynamic_resolution_provenance_report(provenance)?;
        if provenance.shell_validation_ledger_hash != input.validation.validation_ledger_hash
            || provenance.shell_image_hash != input.validation.shell_image_hash
            || provenance.unresolved_symbol_count != input.unresolved_external_symbol_count
        {
            return Err("ELF loader probe rejects dynamic provenance lineage drift".to_owned());
        }
    }
    Ok(())
}

fn build_report(
    input: &ElfAmd64LoaderProbeInput<'_>,
    execute: bool,
    host_supported: bool,
    input_eligible: bool,
    observation: LoaderProbeRuntimeObservation,
) -> ElfAmd64LoaderProbeReport {
    let publication_eligible = observation.attempted
        && observation.materialized_hash_matches
        && observation.kernel_accepted
        && observation.process_completed
        && !observation.timed_out
        && observation.exit_code == Some(0)
        && observation.termination_signal.is_none()
        && !observation.stdout.truncated
        && !observation.stderr.truncated
        && observation.failure_kind.is_none()
        && observation.cleanup_succeeded
        && observation.blockers.is_empty();
    let publication_status = if publication_eligible {
        "eligible-isolated-os-loader-probe-passed"
    } else {
        "blocked-isolated-os-loader-probe-incomplete"
    };
    let mut report = ElfAmd64LoaderProbeReport {
        contract: ELF_AMD64_LOADER_PROBE_CONTRACT,
        status: observation.status,
        probe_mode: if execute { "execute" } else { "plan-only" },
        materialization_kind: LOADER_PROBE_MATERIALIZATION_KIND,
        target_arch: "x86_64",
        target_os: "linux",
        host_supported,
        input_eligible,
        attempted: observation.attempted,
        image_span_bytes: input.bytes.len(),
        shell_image_hash: input.validation.shell_image_hash.clone(),
        validation_contract: ELF_AMD64_SHELL_IMAGE_VALIDATION_CONTRACT,
        validation_ledger_hash: input.validation.validation_ledger_hash.clone(),
        serialization_ledger_hash: input.validation.serialization_ledger_hash.clone(),
        dynamic_provenance_contract: input
            .dynamic_provenance
            .map(|provenance| provenance.contract.to_owned()),
        dynamic_provenance_ledger_hash: input
            .dynamic_provenance
            .map(|provenance| provenance.provenance_ledger_hash.clone()),
        dynamic_provenance_ready: input
            .dynamic_provenance
            .is_some_and(|provenance| provenance.provenance_ready),
        unresolved_external_symbol_count: input.unresolved_external_symbol_count,
        dynamic_segment_count: input.validation.dynamic_segment_count,
        dynamic_entry_count: input.validation.dynamic_entry_count,
        probe_timeout_millis: LOADER_PROBE_TIMEOUT_MILLIS,
        materialized: observation.materialized,
        materialized_hash_matches: observation.materialized_hash_matches,
        kernel_accepted: observation.kernel_accepted,
        process_completed: observation.process_completed,
        timed_out: observation.timed_out,
        exit_code: observation.exit_code,
        termination_signal: observation.termination_signal,
        stdout_captured_bytes: observation.stdout.bytes,
        stdout_truncated: observation.stdout.truncated,
        stdout_hash: observation.stdout.hash,
        stderr_captured_bytes: observation.stderr.bytes,
        stderr_truncated: observation.stderr.truncated,
        stderr_hash: observation.stderr.hash,
        failure_kind: observation.failure_kind,
        cleanup_attempted: observation.cleanup_attempted,
        cleanup_succeeded: observation.cleanup_succeeded,
        publication_eligibility_contract: ELF_AMD64_PUBLICATION_ELIGIBILITY_CONTRACT,
        publication_eligibility_status: publication_status.to_owned(),
        publication_eligible,
        publication_blockers: observation.blockers,
        probe_ledger_hash: String::new(),
    };
    report.probe_ledger_hash = crate::fnv1a64_hex(report.canonical_ledger().as_bytes());
    report
}

pub(crate) fn validate_successful_elf_amd64_loader_probe(
    report: &ElfAmd64LoaderProbeReport,
) -> Result<(), String> {
    if report.contract != ELF_AMD64_LOADER_PROBE_CONTRACT
        || report.status != "os-loader-accepted-process-succeeded"
        || report.probe_mode != "execute"
        || report.materialization_kind != LOADER_PROBE_MATERIALIZATION_KIND
        || report.target_arch != "x86_64"
        || report.target_os != "linux"
        || report.validation_contract != ELF_AMD64_SHELL_IMAGE_VALIDATION_CONTRACT
    {
        return Err("ELF admission rejects loader-probe contract identity".to_owned());
    }
    let static_input = report.unresolved_external_symbol_count == 0
        && report.dynamic_segment_count == 0
        && report.dynamic_entry_count == 0;
    let dynamic_input = report.unresolved_external_symbol_count > 0
        && report.dynamic_segment_count == 1
        && report.dynamic_entry_count > 0
        && report.dynamic_provenance_ready
        && report.dynamic_provenance_contract.as_deref()
            == Some(ELF_AMD64_DYNAMIC_RESOLUTION_PROVENANCE_CONTRACT)
        && report
            .dynamic_provenance_ledger_hash
            .as_deref()
            .is_some_and(|hash| !hash.is_empty());
    if !report.host_supported
        || !report.input_eligible
        || !report.attempted
        || !(static_input || dynamic_input)
    {
        return Err("ELF admission rejects loader-probe input eligibility".to_owned());
    }
    if !report.materialized
        || !report.materialized_hash_matches
        || !report.kernel_accepted
        || !report.process_completed
        || report.timed_out
        || report.exit_code != Some(0)
        || report.termination_signal.is_some()
        || report.stdout_truncated
        || report.stderr_truncated
        || report.failure_kind.is_some()
        || !report.cleanup_attempted
        || !report.cleanup_succeeded
    {
        return Err("ELF admission rejects unsuccessful loader-probe execution".to_owned());
    }
    if report.publication_eligibility_contract != ELF_AMD64_PUBLICATION_ELIGIBILITY_CONTRACT
        || report.publication_eligibility_status != "eligible-isolated-os-loader-probe-passed"
        || !report.publication_eligible
        || !report.publication_blockers.is_empty()
    {
        return Err("ELF admission rejects loader-probe publication eligibility".to_owned());
    }
    if report.probe_ledger_hash != crate::fnv1a64_hex(report.canonical_ledger().as_bytes()) {
        return Err("ELF admission rejects loader-probe ledger drift".to_owned());
    }
    Ok(())
}

fn validate_elf_amd64_loader_probe_report(
    report: &ElfAmd64LoaderProbeReport,
) -> Result<(), String> {
    if report.contract != ELF_AMD64_LOADER_PROBE_CONTRACT
        || report.materialization_kind != LOADER_PROBE_MATERIALIZATION_KIND
        || report.target_arch != "x86_64"
        || report.target_os != "linux"
        || report.validation_contract != ELF_AMD64_SHELL_IMAGE_VALIDATION_CONTRACT
        || report.probe_timeout_millis != LOADER_PROBE_TIMEOUT_MILLIS
        || report.publication_eligibility_contract != ELF_AMD64_PUBLICATION_ELIGIBILITY_CONTRACT
    {
        return Err("ELF loader-probe report contract drift".to_owned());
    }
    if report.dynamic_provenance_ledger_hash.is_some()
        != report.dynamic_provenance_contract.is_some()
        || (report.dynamic_provenance_ready && report.dynamic_provenance_ledger_hash.is_none())
    {
        return Err("ELF loader-probe report provenance shape drift".to_owned());
    }
    if report.probe_ledger_hash != crate::fnv1a64_hex(report.canonical_ledger().as_bytes()) {
        return Err("ELF loader-probe report ledger drift".to_owned());
    }
    if report.publication_eligible {
        validate_successful_elf_amd64_loader_probe(report)
    } else if report.publication_eligibility_status != "blocked-isolated-os-loader-probe-incomplete"
        || report.publication_blockers.is_empty()
    {
        Err("ELF loader-probe report blocker drift".to_owned())
    } else {
        Ok(())
    }
}

#[cfg(test)]
#[path = "final_executable_elf_loader_probe_tests.rs"]
mod tests;
