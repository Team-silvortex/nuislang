use crate::{
    final_executable_finalizer_registry::{
        executable_finalizer_registry_validation, invoke_registered_loader_probe,
        select_executable_finalizer,
    },
    final_executable_registered_loader_probe::{
        validate_registered_loader_probe_outcome, NsldRegisteredLoaderProbeOutcome,
        REGISTERED_LOADER_PROBE_OUTCOME_CONTRACT,
    },
    final_executable_registered_loader_probe_admission_receipt::{
        parse_registered_loader_probe_admission_receipt, registered_loader_probe_admission_path,
        registered_loader_probe_admission_receipt_hash,
        render_registered_loader_probe_admission_receipt,
        NsldRegisteredLoaderProbeAdmissionReceipt, REGISTERED_LOADER_PROBE_ADMISSION_CONTRACT,
        REGISTERED_LOADER_PROBE_ADMISSION_FILE, REGISTERED_LOADER_PROBE_ADMISSION_STATUS,
    },
    hash_sha256::sha256_hex,
};
use std::{fmt::Write as _, fs, path::Path};

pub(crate) const REGISTERED_LOADER_PROBE_ADMISSION_VALIDATION_CONTRACT: &str =
    "nuis-nsld-registered-loader-probe-admission-validation-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldRegisteredLoaderProbeAdmissionVerifyReport {
    pub(crate) contract: &'static str,
    pub(crate) status: &'static str,
    pub(crate) receipt_file: &'static str,
    pub(crate) receipt_present: bool,
    pub(crate) receipt_parsed: bool,
    pub(crate) canonical_source: bool,
    pub(crate) receipt_hash_matches: bool,
    pub(crate) finalizer_registry_matches: bool,
    pub(crate) target_identity_matches: bool,
    pub(crate) outcome_evidence_valid: bool,
    pub(crate) current_private_image_matches: bool,
    pub(crate) valid: bool,
    pub(crate) provider_id: Option<String>,
    pub(crate) target_key: Option<String>,
    pub(crate) capability_id: Option<String>,
    pub(crate) image_identity_hash: Option<String>,
    pub(crate) current_image_identity_hash: Option<String>,
    pub(crate) validation_evidence_hash: Option<String>,
    pub(crate) current_validation_evidence_hash: Option<String>,
    pub(crate) provider_evidence_hash: Option<String>,
    pub(crate) outcome_ledger_hash: Option<String>,
    pub(crate) receipt_hash_sha256: Option<String>,
    pub(crate) issue_count: usize,
    pub(crate) issues: Vec<String>,
    pub(crate) verification_ledger_sha256: String,
}

pub(crate) fn build_registered_loader_probe_admission_receipt(
    plan: &nuisc::linker::LinkPlan,
    outcome: &NsldRegisteredLoaderProbeOutcome,
) -> Result<NsldRegisteredLoaderProbeAdmissionReceipt, String> {
    validate_registered_loader_probe_outcome(outcome)?;
    if outcome.contract != REGISTERED_LOADER_PROBE_OUTCOME_CONTRACT
        || outcome.status != "execution-admitted"
        || outcome.probe_mode != "execute"
        || !outcome.execution_admitted
        || !outcome.blockers.is_empty()
    {
        return Err(
            "registered loader-probe admission requires an execution-admitted outcome".to_owned(),
        );
    }
    let registry = executable_finalizer_registry_validation();
    if !registry.valid {
        return Err(
            "registered loader-probe admission rejects an invalid finalizer registry".to_owned(),
        );
    }
    let selection = select_executable_finalizer(plan)?;
    let capability_id = selection
        .loader_probe_ready()
        .then(|| selection.loader_probe_capability())
        .flatten()
        .ok_or_else(|| {
            "registered loader-probe admission requires a ready registered capability".to_owned()
        })?;
    if outcome.provider_id != selection.provider_id()
        || outcome.target_key != selection.target_key
        || outcome.capability_id != capability_id
    {
        return Err(
            "registered loader-probe admission rejects selection identity drift".to_owned(),
        );
    }

    let mut receipt = NsldRegisteredLoaderProbeAdmissionReceipt {
        contract: REGISTERED_LOADER_PROBE_ADMISSION_CONTRACT.to_owned(),
        status: REGISTERED_LOADER_PROBE_ADMISSION_STATUS.to_owned(),
        finalizer_registry_contract: registry.contract.to_owned(),
        finalizer_registry_hash: registry.registry_hash,
        finalizer_provider_id: selection.provider_id().to_owned(),
        finalizer_target_key: selection.target_key,
        loader_probe_capability_id: capability_id.to_owned(),
        target_abi: plan.cpu_target.abi.clone(),
        machine_arch: plan.cpu_target.machine_arch.clone(),
        machine_os: plan.cpu_target.machine_os.clone(),
        object_format: plan.cpu_target.object_format.clone(),
        calling_abi: plan.cpu_target.calling_abi.clone(),
        packaging_mode: plan.packaging_mode.clone(),
        outcome: outcome.clone(),
        receipt_hash_sha256: String::new(),
    };
    receipt.receipt_hash_sha256 = registered_loader_probe_admission_receipt_hash(&receipt)?;
    Ok(receipt)
}

pub(crate) fn verify_registered_loader_probe_admission_receipt(
    plan: &nuisc::linker::LinkPlan,
) -> NsldRegisteredLoaderProbeAdmissionVerifyReport {
    let path = registered_loader_probe_admission_path(plan);
    let receipt_present = path.is_file();
    let source = fs::read_to_string(&path).map_err(|error| {
        format!(
            "registered-loader-probe-admission-receipt-unreadable:{}",
            error.kind()
        )
        .to_ascii_lowercase()
    });
    let parsed = source
        .as_ref()
        .map_err(Clone::clone)
        .and_then(|source| parse_registered_loader_probe_admission_receipt(source));
    let current = invoke_registered_loader_probe(plan, Path::new(&plan.output_dir), false);
    let mut issues = Vec::new();
    if let Err(error) = &parsed {
        issues.push(error.clone());
    }
    if let Err(error) = &current {
        issues.push(format!(
            "registered-loader-probe-current-image-unavailable:{error}"
        ));
    }

    let mut canonical_source = false;
    let mut receipt_hash_matches = false;
    let mut finalizer_registry_matches = false;
    let mut target_identity_matches = false;
    let mut outcome_evidence_valid = false;
    let mut current_private_image_matches = false;
    if let Ok(receipt) = parsed.as_ref() {
        canonical_source = source.as_ref().is_ok_and(|source| {
            render_registered_loader_probe_admission_receipt(receipt)
                .is_ok_and(|canonical| canonical == *source)
        });
        receipt_hash_matches = registered_loader_probe_admission_receipt_hash(receipt)
            .is_ok_and(|actual| actual == receipt.receipt_hash_sha256);
        finalizer_registry_matches = receipt_registry_matches(plan, receipt);
        target_identity_matches = receipt_target_matches(plan, receipt);
        outcome_evidence_valid = receipt_outcome_valid(receipt);
        current_private_image_matches = current
            .as_ref()
            .is_ok_and(|current| current_image_matches(receipt, current));

        if receipt.contract != REGISTERED_LOADER_PROBE_ADMISSION_CONTRACT
            || receipt.status != REGISTERED_LOADER_PROBE_ADMISSION_STATUS
        {
            issues.push("registered-loader-probe-admission-contract-mismatch".to_owned());
        }
        push_failed_check(
            &mut issues,
            canonical_source,
            "registered-loader-probe-admission-source-not-canonical",
        );
        push_failed_check(
            &mut issues,
            receipt_hash_matches,
            "registered-loader-probe-admission-receipt-hash-mismatch",
        );
        push_failed_check(
            &mut issues,
            finalizer_registry_matches,
            "registered-loader-probe-admission-registry-identity-mismatch",
        );
        push_failed_check(
            &mut issues,
            target_identity_matches,
            "registered-loader-probe-admission-target-identity-mismatch",
        );
        push_failed_check(
            &mut issues,
            outcome_evidence_valid,
            "registered-loader-probe-admission-outcome-invalid",
        );
        push_failed_check(
            &mut issues,
            current_private_image_matches,
            "registered-loader-probe-admission-current-image-mismatch",
        );
    }

    let valid = issues.is_empty();
    let mut report = NsldRegisteredLoaderProbeAdmissionVerifyReport {
        contract: REGISTERED_LOADER_PROBE_ADMISSION_VALIDATION_CONTRACT,
        status: if valid {
            "registered-loader-probe-admission-replay-verified"
        } else {
            "registered-loader-probe-admission-replay-invalid"
        },
        receipt_file: REGISTERED_LOADER_PROBE_ADMISSION_FILE,
        receipt_present,
        receipt_parsed: parsed.is_ok(),
        canonical_source,
        receipt_hash_matches,
        finalizer_registry_matches,
        target_identity_matches,
        outcome_evidence_valid,
        current_private_image_matches,
        valid,
        provider_id: parsed
            .as_ref()
            .ok()
            .map(|receipt| receipt.finalizer_provider_id.clone()),
        target_key: parsed
            .as_ref()
            .ok()
            .map(|receipt| receipt.finalizer_target_key.clone()),
        capability_id: parsed
            .as_ref()
            .ok()
            .map(|receipt| receipt.loader_probe_capability_id.clone()),
        image_identity_hash: parsed
            .as_ref()
            .ok()
            .map(|receipt| receipt.outcome.image_identity_hash.clone()),
        current_image_identity_hash: current
            .as_ref()
            .ok()
            .map(|outcome| outcome.image_identity_hash.clone()),
        validation_evidence_hash: parsed
            .as_ref()
            .ok()
            .map(|receipt| receipt.outcome.validation_evidence_hash.clone()),
        current_validation_evidence_hash: current
            .as_ref()
            .ok()
            .map(|outcome| outcome.validation_evidence_hash.clone()),
        provider_evidence_hash: parsed
            .as_ref()
            .ok()
            .map(|receipt| receipt.outcome.provider_evidence_hash.clone()),
        outcome_ledger_hash: parsed
            .as_ref()
            .ok()
            .map(|receipt| receipt.outcome.outcome_ledger_hash.clone()),
        receipt_hash_sha256: parsed
            .as_ref()
            .ok()
            .map(|receipt| receipt.receipt_hash_sha256.clone()),
        issue_count: issues.len(),
        issues,
        verification_ledger_sha256: String::new(),
    };
    report.verification_ledger_sha256 = verification_ledger_sha256(&report);
    report
}

fn receipt_registry_matches(
    plan: &nuisc::linker::LinkPlan,
    receipt: &NsldRegisteredLoaderProbeAdmissionReceipt,
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
        && selection.loader_probe_ready()
        && selection.loader_probe_capability() == Some(receipt.loader_probe_capability_id.as_str())
}

fn receipt_target_matches(
    plan: &nuisc::linker::LinkPlan,
    receipt: &NsldRegisteredLoaderProbeAdmissionReceipt,
) -> bool {
    receipt.target_abi == plan.cpu_target.abi
        && receipt.machine_arch == plan.cpu_target.machine_arch
        && receipt.machine_os == plan.cpu_target.machine_os
        && receipt.object_format == plan.cpu_target.object_format
        && receipt.calling_abi == plan.cpu_target.calling_abi
        && receipt.packaging_mode == plan.packaging_mode
}

fn receipt_outcome_valid(receipt: &NsldRegisteredLoaderProbeAdmissionReceipt) -> bool {
    receipt.outcome.provider_id == receipt.finalizer_provider_id
        && receipt.outcome.target_key == receipt.finalizer_target_key
        && receipt.outcome.capability_id == receipt.loader_probe_capability_id
        && receipt.outcome.status == "execution-admitted"
        && receipt.outcome.probe_mode == "execute"
        && receipt.outcome.execution_admitted
        && receipt.outcome.blockers.is_empty()
        && validate_registered_loader_probe_outcome(&receipt.outcome).is_ok()
}

fn current_image_matches(
    receipt: &NsldRegisteredLoaderProbeAdmissionReceipt,
    current: &NsldRegisteredLoaderProbeOutcome,
) -> bool {
    validate_registered_loader_probe_outcome(current).is_ok()
        && current.provider_id == receipt.finalizer_provider_id
        && current.target_key == receipt.finalizer_target_key
        && current.capability_id == receipt.loader_probe_capability_id
        && current.provider_probe_contract == receipt.outcome.provider_probe_contract
        && current.input_eligible
        && current.image_span_bytes == receipt.outcome.image_span_bytes
        && current.image_identity_hash == receipt.outcome.image_identity_hash
        && current.validation_evidence_hash == receipt.outcome.validation_evidence_hash
}

fn push_failed_check(issues: &mut Vec<String>, passed: bool, issue: &str) {
    if !passed {
        issues.push(issue.to_owned());
    }
}

fn verification_ledger_sha256(report: &NsldRegisteredLoaderProbeAdmissionVerifyReport) -> String {
    let mut material = format!(
        "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}\n",
        report.contract,
        report.status,
        report.receipt_file,
        report.receipt_present,
        report.receipt_parsed,
        report.canonical_source,
        report.receipt_hash_matches,
        report.finalizer_registry_matches,
        report.target_identity_matches,
        report.outcome_evidence_valid,
        report.current_private_image_matches,
        report.valid,
        option(&report.provider_id),
        option(&report.target_key),
        option(&report.capability_id),
        option(&report.image_identity_hash),
        option(&report.current_image_identity_hash),
        option(&report.validation_evidence_hash),
        option(&report.current_validation_evidence_hash),
        option(&report.provider_evidence_hash),
        option(&report.outcome_ledger_hash),
        option(&report.receipt_hash_sha256),
        report.issue_count,
        report.issues.len(),
        report.valid,
    );
    for issue in &report.issues {
        writeln!(material, "issue={issue}").unwrap();
    }
    sha256_hex(material.as_bytes())
}

fn option(value: &Option<String>) -> &str {
    value.as_deref().unwrap_or("none")
}
