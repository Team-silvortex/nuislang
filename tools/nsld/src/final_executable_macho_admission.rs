use crate::{
    final_executable_finalizer_registry::{
        executable_finalizer_registry_validation, select_executable_finalizer,
    },
    final_executable_macho_admission_receipt::{
        macho_arm64_publication_admission_path, parse_macho_arm64_publication_admission_receipt,
        receipt_hash_sha256, render_macho_arm64_publication_admission_receipt,
        MACHO_ARM64_PUBLICATION_ADMISSION_CONTRACT, MACHO_ARM64_PUBLICATION_ADMISSION_FILE,
        MACHO_ARM64_PUBLICATION_ADMISSION_STATUS,
    },
    final_executable_macho_loader_probe::{
        validate_successful_macho_arm64_loader_probe, MACHO_ARM64_LOADER_PROBE_MATERIALIZATION_KIND,
    },
    final_executable_macho_object::MachOArm64PrivateShellProduct,
    final_executable_macho_shell_signature::sha256_hex,
    reports::{
        NsldMachOArm64LoaderProbeReport, NsldMachOArm64PublicationAdmissionReceipt,
        NsldMachOArm64PublicationAdmissionVerifyReport,
    },
};
use std::{fmt::Write as _, fs};

pub(crate) const MACHO_ARM64_PUBLICATION_ADMISSION_VALIDATION_CONTRACT: &str =
    "nuis-nsld-macho-arm64-publication-admission-validation-v1";

pub(crate) fn build_macho_arm64_publication_admission_receipt(
    plan: &nuisc::linker::LinkPlan,
    product: &MachOArm64PrivateShellProduct,
    probe: &NsldMachOArm64LoaderProbeReport,
) -> Result<NsldMachOArm64PublicationAdmissionReceipt, String> {
    validate_successful_macho_arm64_loader_probe(probe)?;
    let summary = &product.summary;
    let serialization = &summary.shell_image_serialization;
    let signature = &serialization.code_signature;
    if summary.unresolved_external_symbol_count != 0
        || !summary.unresolved_external_symbols.is_empty()
        || !summary.shell_layout_plan.binds.is_empty()
        || summary.platform_patch_application.unresolved_bind_count != 0
    {
        return Err("Mach-O admission requires a fully internally closed product".to_owned());
    }
    if probe.image_span_bytes != product.bytes.len()
        || probe.shell_image_hash != serialization.shell_image_hash
        || probe.signature_validation_ledger_hash != signature.validation_ledger_hash
    {
        return Err("Mach-O admission rejects loader-probe product identity drift".to_owned());
    }

    let registry = executable_finalizer_registry_validation();
    if !registry.valid {
        return Err("Mach-O admission rejects an invalid finalizer registry".to_owned());
    }
    let selection = select_executable_finalizer(plan)?;
    if !selection.ready() || selection.target_key != "aarch64-macos-mach-o" {
        return Err("Mach-O admission rejects the selected finalizer target".to_owned());
    }

    let mut receipt = NsldMachOArm64PublicationAdmissionReceipt {
        contract: MACHO_ARM64_PUBLICATION_ADMISSION_CONTRACT.to_owned(),
        status: MACHO_ARM64_PUBLICATION_ADMISSION_STATUS.to_owned(),
        finalizer_registry_contract: registry.contract.to_owned(),
        finalizer_registry_hash: registry.registry_hash,
        finalizer_provider_id: selection.provider_id().to_owned(),
        finalizer_target_key: selection.target_key,
        target_arch: "aarch64".to_owned(),
        target_os: "macos".to_owned(),
        object_format: "mach-o".to_owned(),
        calling_abi: plan.cpu_target.calling_abi.clone(),
        packaging_mode: plan.packaging_mode.clone(),
        object_linkage_hash: summary.shell_layout_plan.object_linkage_hash.clone(),
        shell_layout_plan_hash: summary.shell_layout_plan.plan_hash.clone(),
        serialization_ledger_hash: serialization.serialization_ledger_hash.clone(),
        shell_image_span_bytes: product.bytes.len(),
        shell_image_hash: serialization.shell_image_hash.clone(),
        shell_image_sha256: sha256_hex(&product.bytes),
        signature_validation_contract: signature.validation_contract.clone(),
        signature_validation_status: signature.validation_status.clone(),
        signature_validation_ledger_hash: signature.validation_ledger_hash.clone(),
        signature_cdhash: signature.cdhash.clone(),
        probe_contract: probe.contract.clone(),
        probe_status: probe.status.clone(),
        probe_ledger_hash: probe.probe_ledger_hash.clone(),
        probe_timeout_millis: probe.probe_timeout_millis,
        probe_host_supported: probe.host_supported,
        probe_input_eligible: probe.input_eligible,
        probe_attempted: probe.attempted,
        probe_materialized: probe.materialized,
        probe_materialized_hash_matches: probe.materialized_hash_matches,
        probe_kernel_accepted: probe.kernel_accepted,
        probe_process_completed: probe.process_completed,
        probe_timed_out: probe.timed_out,
        probe_exit_code: probe.exit_code,
        probe_termination_signal: probe.termination_signal,
        probe_stdout_captured_bytes: probe.stdout_captured_bytes,
        probe_stdout_truncated: probe.stdout_truncated,
        probe_stdout_hash: probe.stdout_hash.clone(),
        probe_stderr_captured_bytes: probe.stderr_captured_bytes,
        probe_stderr_truncated: probe.stderr_truncated,
        probe_stderr_hash: probe.stderr_hash.clone(),
        probe_failure_kind: probe.failure_kind.clone(),
        probe_cleanup_attempted: probe.cleanup_attempted,
        probe_cleanup_succeeded: probe.cleanup_succeeded,
        unresolved_external_symbol_count: probe.unresolved_external_symbol_count,
        bind_count: probe.bind_count,
        publication_eligibility_contract: probe.publication_eligibility_contract.clone(),
        publication_eligibility_status: probe.publication_eligibility_status.clone(),
        publication_eligible: probe.publication_eligible,
        receipt_hash_sha256: String::new(),
    };
    receipt.receipt_hash_sha256 = receipt_hash_sha256(&receipt)?;
    Ok(receipt)
}

pub(crate) fn verify_macho_arm64_publication_admission_receipt(
    plan: &nuisc::linker::LinkPlan,
    product: &MachOArm64PrivateShellProduct,
) -> NsldMachOArm64PublicationAdmissionVerifyReport {
    let path = macho_arm64_publication_admission_path(plan);
    let receipt_present = path.is_file();
    let mut issues = Vec::new();
    let source = fs::read_to_string(&path).map_err(|error| {
        format!("publication-admission-receipt-unreadable:{}", error.kind()).to_ascii_lowercase()
    });
    let parsed = source
        .as_ref()
        .map_err(Clone::clone)
        .and_then(|source| parse_macho_arm64_publication_admission_receipt(source));
    if let Err(error) = &parsed {
        issues.push(error.clone());
    }

    let mut canonical_source = false;
    let mut receipt_hash_matches = false;
    let mut finalizer_registry_matches = false;
    let mut target_identity_matches = false;
    let mut private_image_matches = false;
    let mut signature_identity_matches = false;
    let mut probe_evidence_valid = false;
    let mut receipt_hash = None;
    let mut probe_ledger_hash = None;
    if let Ok(receipt) = parsed.as_ref() {
        canonical_source = source.as_ref().is_ok_and(|source| {
            render_macho_arm64_publication_admission_receipt(receipt)
                .is_ok_and(|canonical| canonical == *source)
        });
        receipt_hash_matches =
            receipt_hash_sha256(receipt).is_ok_and(|actual| actual == receipt.receipt_hash_sha256);
        finalizer_registry_matches = receipt_registry_matches(plan, receipt);
        target_identity_matches = receipt_target_matches(plan, receipt);
        private_image_matches = receipt_private_image_matches(product, receipt);
        signature_identity_matches = receipt_signature_matches(product, receipt);
        probe_evidence_valid = receipt_probe_evidence_valid(receipt);
        receipt_hash = Some(receipt.receipt_hash_sha256.clone());
        probe_ledger_hash = Some(receipt.probe_ledger_hash.clone());

        if receipt.contract != MACHO_ARM64_PUBLICATION_ADMISSION_CONTRACT
            || receipt.status != MACHO_ARM64_PUBLICATION_ADMISSION_STATUS
        {
            issues.push("publication-admission-contract-mismatch".to_owned());
        }
        push_failed_check(
            &mut issues,
            canonical_source,
            "receipt-source-not-canonical",
        );
        push_failed_check(&mut issues, receipt_hash_matches, "receipt-hash-mismatch");
        push_failed_check(
            &mut issues,
            finalizer_registry_matches,
            "finalizer-registry-identity-mismatch",
        );
        push_failed_check(
            &mut issues,
            target_identity_matches,
            "target-identity-mismatch",
        );
        push_failed_check(
            &mut issues,
            private_image_matches,
            "private-image-identity-mismatch",
        );
        push_failed_check(
            &mut issues,
            signature_identity_matches,
            "signature-identity-mismatch",
        );
        push_failed_check(
            &mut issues,
            probe_evidence_valid,
            "loader-probe-evidence-invalid",
        );
    }
    let valid = issues.is_empty();
    let current_shell_image_hash = product
        .summary
        .shell_image_serialization
        .shell_image_hash
        .clone();
    let current_shell_image_sha256 = sha256_hex(&product.bytes);
    let mut report = NsldMachOArm64PublicationAdmissionVerifyReport {
        contract: MACHO_ARM64_PUBLICATION_ADMISSION_VALIDATION_CONTRACT.to_owned(),
        status: if valid {
            "publication-admission-replay-verified"
        } else {
            "publication-admission-replay-invalid"
        }
        .to_owned(),
        receipt_file: MACHO_ARM64_PUBLICATION_ADMISSION_FILE.to_owned(),
        receipt_present,
        receipt_parsed: parsed.is_ok(),
        canonical_source,
        receipt_hash_matches,
        finalizer_registry_matches,
        target_identity_matches,
        private_image_matches,
        signature_identity_matches,
        probe_evidence_valid,
        valid,
        current_shell_image_hash,
        current_shell_image_sha256,
        receipt_hash_sha256: receipt_hash,
        probe_ledger_hash,
        issue_count: issues.len(),
        issues,
        verification_ledger_sha256: String::new(),
    };
    report.verification_ledger_sha256 = verification_ledger_sha256(&report);
    report
}

fn receipt_registry_matches(
    plan: &nuisc::linker::LinkPlan,
    receipt: &NsldMachOArm64PublicationAdmissionReceipt,
) -> bool {
    let registry = executable_finalizer_registry_validation();
    let Ok(selection) = select_executable_finalizer(plan) else {
        return false;
    };
    registry.valid
        && receipt.finalizer_registry_contract == registry.contract
        && receipt.finalizer_registry_hash == registry.registry_hash
        && receipt.finalizer_provider_id == selection.provider_id()
        && receipt.finalizer_target_key == selection.target_key
        && selection.ready()
}

fn receipt_target_matches(
    plan: &nuisc::linker::LinkPlan,
    receipt: &NsldMachOArm64PublicationAdmissionReceipt,
) -> bool {
    receipt.finalizer_target_key == "aarch64-macos-mach-o"
        && receipt.target_arch == "aarch64"
        && receipt.target_os == "macos"
        && receipt.object_format == "mach-o"
        && receipt.calling_abi == plan.cpu_target.calling_abi
        && receipt.packaging_mode == plan.packaging_mode
}

fn receipt_private_image_matches(
    product: &MachOArm64PrivateShellProduct,
    receipt: &NsldMachOArm64PublicationAdmissionReceipt,
) -> bool {
    let summary = &product.summary;
    let serialization = &summary.shell_image_serialization;
    summary.unresolved_external_symbol_count == 0
        && summary.unresolved_external_symbols.is_empty()
        && summary.shell_layout_plan.binds.is_empty()
        && summary.platform_patch_application.unresolved_bind_count == 0
        && receipt.unresolved_external_symbol_count == 0
        && receipt.bind_count == 0
        && receipt.object_linkage_hash == summary.shell_layout_plan.object_linkage_hash
        && receipt.shell_layout_plan_hash == summary.shell_layout_plan.plan_hash
        && receipt.serialization_ledger_hash == serialization.serialization_ledger_hash
        && receipt.shell_image_span_bytes == product.bytes.len()
        && receipt.shell_image_hash == serialization.shell_image_hash
        && receipt.shell_image_sha256 == sha256_hex(&product.bytes)
}

fn receipt_signature_matches(
    product: &MachOArm64PrivateShellProduct,
    receipt: &NsldMachOArm64PublicationAdmissionReceipt,
) -> bool {
    let signature = &product.summary.shell_image_serialization.code_signature;
    receipt.signature_validation_contract == signature.validation_contract
        && receipt.signature_validation_status == signature.validation_status
        && receipt.signature_validation_ledger_hash == signature.validation_ledger_hash
        && receipt.signature_cdhash == signature.cdhash
}

fn receipt_probe_evidence_valid(receipt: &NsldMachOArm64PublicationAdmissionReceipt) -> bool {
    validate_successful_macho_arm64_loader_probe(&NsldMachOArm64LoaderProbeReport {
        contract: receipt.probe_contract.clone(),
        status: receipt.probe_status.clone(),
        probe_mode: "execute".to_owned(),
        materialization_kind: MACHO_ARM64_LOADER_PROBE_MATERIALIZATION_KIND.to_owned(),
        target_arch: receipt.target_arch.clone(),
        target_os: receipt.target_os.clone(),
        host_supported: receipt.probe_host_supported,
        input_eligible: receipt.probe_input_eligible,
        attempted: receipt.probe_attempted,
        image_span_bytes: receipt.shell_image_span_bytes,
        shell_image_hash: receipt.shell_image_hash.clone(),
        signature_validation_ledger_hash: receipt.signature_validation_ledger_hash.clone(),
        unresolved_external_symbol_count: receipt.unresolved_external_symbol_count,
        bind_count: receipt.bind_count,
        probe_timeout_millis: receipt.probe_timeout_millis,
        materialized: receipt.probe_materialized,
        materialized_hash_matches: receipt.probe_materialized_hash_matches,
        kernel_accepted: receipt.probe_kernel_accepted,
        process_completed: receipt.probe_process_completed,
        timed_out: receipt.probe_timed_out,
        exit_code: receipt.probe_exit_code,
        termination_signal: receipt.probe_termination_signal,
        stdout_captured_bytes: receipt.probe_stdout_captured_bytes,
        stdout_truncated: receipt.probe_stdout_truncated,
        stdout_hash: receipt.probe_stdout_hash.clone(),
        stderr_captured_bytes: receipt.probe_stderr_captured_bytes,
        stderr_truncated: receipt.probe_stderr_truncated,
        stderr_hash: receipt.probe_stderr_hash.clone(),
        failure_kind: receipt.probe_failure_kind.clone(),
        cleanup_attempted: receipt.probe_cleanup_attempted,
        cleanup_succeeded: receipt.probe_cleanup_succeeded,
        publication_eligibility_contract: receipt.publication_eligibility_contract.clone(),
        publication_eligibility_status: receipt.publication_eligibility_status.clone(),
        publication_eligible: receipt.publication_eligible,
        publication_blockers: Vec::new(),
        probe_ledger_hash: receipt.probe_ledger_hash.clone(),
        admission_receipt_file: None,
        admission_receipt_persisted: false,
        admission_receipt_hash_sha256: None,
        admission_receipt_validation_status: "not-replayed".to_owned(),
    })
    .is_ok()
}

fn verification_ledger_sha256(report: &NsldMachOArm64PublicationAdmissionVerifyReport) -> String {
    let mut material = format!(
        "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}\n",
        report.contract,
        report.status,
        report.receipt_file,
        report.receipt_present,
        report.receipt_parsed,
        report.canonical_source,
        report.receipt_hash_matches,
        report.finalizer_registry_matches,
        report.target_identity_matches,
        report.private_image_matches,
        report.signature_identity_matches,
        report.probe_evidence_valid,
        report.valid,
        report.current_shell_image_hash,
        report.current_shell_image_sha256,
        report.receipt_hash_sha256.as_deref().unwrap_or("none"),
        report.probe_ledger_hash.as_deref().unwrap_or("none"),
    );
    for issue in &report.issues {
        writeln!(material, "issue={issue}").unwrap();
    }
    sha256_hex(material.as_bytes())
}

fn push_failed_check(issues: &mut Vec<String>, passed: bool, issue: &str) {
    if !passed {
        issues.push(issue.to_owned());
    }
}

#[cfg(test)]
#[path = "final_executable_macho_admission_tests.rs"]
mod tests;
