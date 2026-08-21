use crate::{
    final_executable_loader_probe_runtime::{
        execute_isolated_loader_probe, LoaderProbeRuntimeObservation as ProbeObservation,
        LoaderProbeRuntimeRequest, LOADER_PROBE_MATERIALIZATION_KIND, LOADER_PROBE_TIMEOUT_MILLIS,
    },
    final_executable_macho_shell_image::MACHO_ARM64_SHELL_IMAGE_SERIALIZATION_CONTRACT,
    final_executable_macho_shell_signature_validation::{
        MACHO_ARM64_PUBLICATION_ELIGIBILITY_CONTRACT, MACHO_ARM64_SIGNED_IMAGE_VALIDATION_CONTRACT,
    },
    reports::{NsldMachOArm64LoaderProbeReport, NsldMachOArm64ShellImageSerializationReport},
};
use std::path::Path;

pub(crate) const MACHO_ARM64_LOADER_PROBE_CONTRACT: &str =
    "nuis-nsld-macho-arm64-os-loader-probe-v1";
pub(crate) const MACHO_ARM64_LOADER_PROBE_MATERIALIZATION_KIND: &str =
    LOADER_PROBE_MATERIALIZATION_KIND;
pub(crate) const MACHO_ARM64_LOADER_PROBE_TIMEOUT_MILLIS: u64 = LOADER_PROBE_TIMEOUT_MILLIS;

pub(crate) struct MachOArm64LoaderProbeInput<'a> {
    pub(crate) bytes: &'a [u8],
    pub(crate) serialization: &'a NsldMachOArm64ShellImageSerializationReport,
    pub(crate) unresolved_external_symbol_count: usize,
    pub(crate) bind_count: usize,
}

pub(crate) fn probe_macho_arm64_signed_shell_image(
    input: MachOArm64LoaderProbeInput<'_>,
    probe_root: &Path,
    execute: bool,
) -> Result<NsldMachOArm64LoaderProbeReport, String> {
    validate_input(&input)?;
    let host_supported = cfg!(all(target_os = "macos", target_arch = "aarch64"));
    let input_eligible = input.unresolved_external_symbol_count == 0 && input.bind_count == 0;
    let observation = if !host_supported {
        ProbeObservation::blocked("blocked-unsupported-probe-host", "unsupported-probe-host")
    } else if !input_eligible {
        ProbeObservation::blocked(
            "blocked-external-compatibility-input",
            "private-image-has-external-compatibility-bindings",
        )
    } else if !execute {
        ProbeObservation::blocked(
            "ready-explicit-apply-required",
            "explicit-loader-probe-apply-required",
        )
    } else {
        execute_probe(&input, probe_root)?
    };
    Ok(build_report(
        &input,
        execute,
        host_supported,
        input_eligible,
        observation,
    ))
}

fn validate_input(input: &MachOArm64LoaderProbeInput<'_>) -> Result<(), String> {
    let report = input.serialization;
    if report.contract != MACHO_ARM64_SHELL_IMAGE_SERIALIZATION_CONTRACT
        || report.status != "signed-private-image-validated"
        || report.code_signature.validation_contract != MACHO_ARM64_SIGNED_IMAGE_VALIDATION_CONTRACT
        || report.code_signature.validation_status != "signed-private-image-structurally-valid"
        || report.code_signature.publication_eligibility_contract
            != MACHO_ARM64_PUBLICATION_ELIGIBILITY_CONTRACT
    {
        return Err("Mach-O loader probe rejects the serialization contract".to_owned());
    }
    if input.bytes.len() != report.shell_image_span_bytes
        || crate::fnv1a64_hex(input.bytes) != report.shell_image_hash
        || report.code_signature.signature_file_offset
            + report.code_signature.signature_payload_bytes
            != input.bytes.len()
        || report.code_signature.verified_code_slot_count != report.code_signature.code_slot_count
    {
        return Err("Mach-O loader probe rejects private image drift".to_owned());
    }
    Ok(())
}

fn execute_probe(
    input: &MachOArm64LoaderProbeInput<'_>,
    probe_root: &Path,
) -> Result<ProbeObservation, String> {
    execute_isolated_loader_probe(LoaderProbeRuntimeRequest {
        bytes: input.bytes,
        probe_root,
        path_namespace: "macho",
    })
}
fn build_report(
    input: &MachOArm64LoaderProbeInput<'_>,
    execute: bool,
    host_supported: bool,
    input_eligible: bool,
    observation: ProbeObservation,
) -> NsldMachOArm64LoaderProbeReport {
    let publication_eligible = observation.attempted
        && observation.materialized_hash_matches
        && observation.kernel_accepted
        && observation.process_completed
        && observation.exit_code == Some(0)
        && observation.cleanup_succeeded
        && observation.blockers.is_empty();
    let publication_eligibility_status = if publication_eligible {
        "eligible-isolated-os-loader-probe-passed"
    } else {
        "blocked-isolated-os-loader-probe-incomplete"
    };
    let mut report = NsldMachOArm64LoaderProbeReport {
        contract: MACHO_ARM64_LOADER_PROBE_CONTRACT.to_owned(),
        status: observation.status,
        probe_mode: if execute { "execute" } else { "plan-only" }.to_owned(),
        materialization_kind: MACHO_ARM64_LOADER_PROBE_MATERIALIZATION_KIND.to_owned(),
        target_arch: "aarch64".to_owned(),
        target_os: "macos".to_owned(),
        host_supported,
        input_eligible,
        attempted: observation.attempted,
        image_span_bytes: input.bytes.len(),
        shell_image_hash: input.serialization.shell_image_hash.clone(),
        signature_validation_ledger_hash: input
            .serialization
            .code_signature
            .validation_ledger_hash
            .clone(),
        unresolved_external_symbol_count: input.unresolved_external_symbol_count,
        bind_count: input.bind_count,
        probe_timeout_millis: MACHO_ARM64_LOADER_PROBE_TIMEOUT_MILLIS,
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
        publication_eligibility_contract: MACHO_ARM64_PUBLICATION_ELIGIBILITY_CONTRACT.to_owned(),
        publication_eligibility_status: publication_eligibility_status.to_owned(),
        publication_eligible,
        publication_blockers: observation.blockers,
        probe_ledger_hash: String::new(),
        admission_receipt_file: None,
        admission_receipt_persisted: false,
        admission_receipt_hash_sha256: None,
        admission_receipt_validation_status: "not-requested".to_owned(),
    };
    report.probe_ledger_hash = probe_ledger_hash(&report);
    report
}

pub(crate) fn validate_successful_macho_arm64_loader_probe(
    report: &NsldMachOArm64LoaderProbeReport,
) -> Result<(), String> {
    if report.contract != MACHO_ARM64_LOADER_PROBE_CONTRACT
        || report.status != "os-loader-accepted-process-succeeded"
        || report.probe_mode != "execute"
        || report.materialization_kind != MACHO_ARM64_LOADER_PROBE_MATERIALIZATION_KIND
        || report.target_arch != "aarch64"
        || report.target_os != "macos"
    {
        return Err("Mach-O admission rejects the loader-probe contract identity".to_owned());
    }
    if !report.host_supported
        || !report.input_eligible
        || !report.attempted
        || report.unresolved_external_symbol_count != 0
        || report.bind_count != 0
    {
        return Err("Mach-O admission rejects loader-probe input eligibility".to_owned());
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
    {
        return Err("Mach-O admission rejects unsuccessful loader-probe execution".to_owned());
    }
    if !report.cleanup_attempted || !report.cleanup_succeeded {
        return Err("Mach-O admission rejects incomplete loader-probe cleanup".to_owned());
    }
    if report.publication_eligibility_contract != MACHO_ARM64_PUBLICATION_ELIGIBILITY_CONTRACT
        || report.publication_eligibility_status != "eligible-isolated-os-loader-probe-passed"
        || !report.publication_eligible
        || !report.publication_blockers.is_empty()
    {
        return Err("Mach-O admission rejects loader-probe publication eligibility".to_owned());
    }
    if report.probe_ledger_hash != probe_ledger_hash(report) {
        return Err("Mach-O admission rejects loader-probe ledger drift".to_owned());
    }
    Ok(())
}

fn probe_ledger_hash(report: &NsldMachOArm64LoaderProbeReport) -> String {
    let mut material = format!(
        "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}\n",
        report.contract,
        report.status,
        report.probe_mode,
        report.materialization_kind,
        report.target_arch,
        report.target_os,
        report.host_supported,
        report.input_eligible,
        report.attempted,
        report.image_span_bytes,
        report.shell_image_hash,
        report.signature_validation_ledger_hash,
        report.unresolved_external_symbol_count,
        report.bind_count,
        report.probe_timeout_millis,
        report.materialized,
        report.materialized_hash_matches,
        report.kernel_accepted,
        report.process_completed,
        report.timed_out,
        option_i32(report.exit_code),
        option_i32(report.termination_signal),
        report.stdout_captured_bytes,
        report.stdout_truncated,
        report.stdout_hash,
        report.stderr_captured_bytes,
        report.stderr_truncated,
        report.stderr_hash,
        report.failure_kind.as_deref().unwrap_or("none"),
        report.cleanup_attempted,
        report.cleanup_succeeded,
    );
    material.push_str(&format!(
        "{}|{}|{}\n",
        report.publication_eligibility_contract,
        report.publication_eligibility_status,
        report.publication_eligible
    ));
    for blocker in &report.publication_blockers {
        material.push_str("blocker=");
        material.push_str(blocker);
        material.push('\n');
    }
    crate::fnv1a64_hex(material.as_bytes())
}

fn option_i32(value: Option<i32>) -> String {
    value.map_or_else(|| "none".to_owned(), |value| value.to_string())
}

#[cfg(test)]
#[path = "final_executable_macho_loader_probe_tests.rs"]
mod tests;
