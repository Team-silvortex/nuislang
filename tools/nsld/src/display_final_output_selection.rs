use crate::reports::NsldFinalOutputSelectionReport;

pub(crate) fn print_final_output_selection(report: &NsldFinalOutputSelectionReport) {
    println!(
        "  selection: contract={} registry={} registry_hash={} policy={} policy_status={} kind={} default={} explicit={} apply={} status={} ready={} attempted={} selected={}",
        report.contract,
        report.registry_contract,
        report.registry_hash,
        report.policy_id,
        report.policy_status,
        report.selection_kind,
        report.default_policy,
        report.explicit_request,
        report.apply_requested,
        report.status,
        report.selection_ready,
        report.installation_attempted,
        report.selected
    );
    println!(
        "  selection_provider: provider={} target={} capability={}",
        report.provider_id.as_deref().unwrap_or("none"),
        report.target_key.as_deref().unwrap_or("none"),
        report.capability_id.as_deref().unwrap_or("none")
    );
    println!(
        "  selection_admission: contract={} status={} file={} valid={} receipt_sha256={} verification_sha256={}",
        report.admission_contract.as_deref().unwrap_or("none"),
        report.admission_status,
        report.admission_receipt_file.as_deref().unwrap_or("none"),
        report
            .admission_receipt_valid
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_owned()),
        report
            .admission_receipt_hash_sha256
            .as_deref()
            .unwrap_or("none"),
        report
            .admission_verification_ledger_sha256
            .as_deref()
            .unwrap_or("none")
    );
    println!(
        "  selection_output: path={} name={} present={} bytes={} sha256={} executable={} identity_matches={} candidate_bytes={} candidate_sha256={}",
        report.selected_output_path,
        report.selected_output_name,
        report.selected_output_present,
        report
            .selected_output_span_bytes
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_owned()),
        report.selected_output_sha256.as_deref().unwrap_or("none"),
        report.selected_output_executable,
        report.selected_output_identity_matches,
        report
            .candidate_image_span_bytes
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_owned()),
        report.candidate_image_sha256.as_deref().unwrap_or("none")
    );
    println!(
        "  selection_publication: contract={} status={} ledger_sha256={} selection_sha256={} issues={}",
        report.publication_contract.as_deref().unwrap_or("none"),
        report.publication_status,
        report
            .publication_ledger_sha256
            .as_deref()
            .unwrap_or("none"),
        report.selection_ledger_sha256,
        report.issues.join(",")
    );
}
