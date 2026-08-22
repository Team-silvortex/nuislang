use crate::final_executable_registered_loader_probe_admission::NsldRegisteredLoaderProbeAdmissionVerifyReport;

pub(crate) fn print_registered_loader_probe_admission_verify_report(
    report: &NsldRegisteredLoaderProbeAdmissionVerifyReport,
) {
    println!("Nsld registered loader-probe admission verification");
    println!(
        "  receipt: file={} present={} parsed={} canonical={} hash_matches={}",
        report.receipt_file,
        report.receipt_present,
        report.receipt_parsed,
        report.canonical_source,
        report.receipt_hash_matches
    );
    println!(
        "  identity: provider={} target={} capability={} registry_matches={} target_matches={}",
        optional(&report.provider_id),
        optional(&report.target_key),
        optional(&report.capability_id),
        report.finalizer_registry_matches,
        report.target_identity_matches
    );
    println!(
        "  image: admitted={} current={} validation={} current_validation={} matches={}",
        optional(&report.image_identity_hash),
        optional(&report.current_image_identity_hash),
        optional(&report.validation_evidence_hash),
        optional(&report.current_validation_evidence_hash),
        report.current_private_image_matches
    );
    println!(
        "  evidence: outcome_valid={} provider={} outcome={} receipt={} verification={}",
        report.outcome_evidence_valid,
        optional(&report.provider_evidence_hash),
        optional(&report.outcome_ledger_hash),
        optional(&report.receipt_hash_sha256),
        report.verification_ledger_sha256
    );
    println!(
        "  result: contract={} status={} valid={} issues={}",
        report.contract,
        report.status,
        report.valid,
        if report.issues.is_empty() {
            "none".to_owned()
        } else {
            report.issues.join(",")
        }
    );
}

fn optional(value: &Option<String>) -> &str {
    value.as_deref().unwrap_or("none")
}
