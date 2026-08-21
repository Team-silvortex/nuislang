use crate::reports::NsldPrivateImagePublicationReport;

pub(crate) fn print_private_image_publication_report(report: &NsldPrivateImagePublicationReport) {
    println!("Nsld registered private-image publication");
    println!(
        "  publication: contract={} status={} provider={} target={} capability={} apply={} ready={}",
        report.contract,
        report.status,
        report.provider_id,
        report.target_key,
        report.capability_id,
        report.apply_requested,
        report.publication_ready
    );
    println!(
        "  admission: contract={} status={} file={} present={} valid={} receipt_sha256={} verification_sha256={}",
        report.admission_contract,
        report.admission_status,
        report.admission_receipt_file,
        report.admission_receipt_present,
        report.admission_receipt_valid,
        report
            .admission_receipt_hash_sha256
            .as_deref()
            .unwrap_or("none"),
        report.admission_verification_ledger_sha256
    );
    println!(
        "  image: bytes={} fnv={} sha256={}",
        report.source_image_span_bytes, report.source_image_hash, report.source_image_sha256
    );
    println!(
        "  output: path={} before={} before_sha256={} attempted={} installed={} after={} after_bytes={} after_sha256={} matches={} executable={} changed={}",
        report.output_path,
        report.output_present_before,
        report.output_sha256_before.as_deref().unwrap_or("none"),
        report.installation_attempted,
        report.installed,
        report.output_present_after,
        report
            .output_span_bytes_after
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_owned()),
        report.output_sha256_after.as_deref().unwrap_or("none"),
        report.output_matches_private_image,
        report.output_executable,
        report.output_changed
    );
    println!(
        "  issues: count={} values={} publication_sha256={}",
        report.issue_count,
        report.issues.join(","),
        report.publication_ledger_sha256
    );
}
