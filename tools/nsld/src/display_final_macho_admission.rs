use crate::reports::NsldMachOArm64PublicationAdmissionVerifyReport;

pub(crate) fn print_macho_arm64_publication_admission_verify_report(
    report: &NsldMachOArm64PublicationAdmissionVerifyReport,
) {
    println!("Nsld Mach-O arm64 publication-admission verification");
    println!(
        "  receipt: contract={} status={} file={} present={} parsed={} canonical={} hash_matches={}",
        report.contract,
        report.status,
        report.receipt_file,
        report.receipt_present,
        report.receipt_parsed,
        report.canonical_source,
        report.receipt_hash_matches
    );
    println!(
        "  identity: registry={} target={} image={} signature={} probe={} valid={}",
        report.finalizer_registry_matches,
        report.target_identity_matches,
        report.private_image_matches,
        report.signature_identity_matches,
        report.probe_evidence_valid,
        report.valid
    );
    println!(
        "  hashes: current_fnv={} current_sha256={} receipt_sha256={} probe_ledger={} verification_sha256={}",
        report.current_shell_image_hash,
        report.current_shell_image_sha256,
        report.receipt_hash_sha256.as_deref().unwrap_or("none"),
        report.probe_ledger_hash.as_deref().unwrap_or("none"),
        report.verification_ledger_sha256
    );
    println!(
        "  issues: count={} values={}",
        report.issue_count,
        report.issues.join(",")
    );
}
