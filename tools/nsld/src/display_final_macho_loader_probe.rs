use crate::reports::NsldMachOArm64LoaderProbeReport;

pub(crate) fn print_macho_arm64_loader_probe_report(report: &NsldMachOArm64LoaderProbeReport) {
    println!("Nsld Mach-O arm64 private-image loader probe");
    println!(
        "  probe: contract={} status={} mode={} materialization={} target={}:{} host_supported={} input_eligible={} attempted={} timeout_ms={} image={}:{} signature_ledger={}",
        report.contract,
        report.status,
        report.probe_mode,
        report.materialization_kind,
        report.target_os,
        report.target_arch,
        report.host_supported,
        report.input_eligible,
        report.attempted,
        report.probe_timeout_millis,
        report.image_span_bytes,
        report.shell_image_hash,
        report.signature_validation_ledger_hash
    );
    println!(
        "  admission: unresolved_external_symbols={} binds={} materialized={} hash_matches={} kernel_accepted={} process_completed={} timed_out={} exit_code={} signal={} failure={}",
        report.unresolved_external_symbol_count,
        report.bind_count,
        report.materialized,
        report.materialized_hash_matches,
        report.kernel_accepted,
        report.process_completed,
        report.timed_out,
        option_i32(report.exit_code),
        option_i32(report.termination_signal),
        report.failure_kind.as_deref().unwrap_or("none")
    );
    println!(
        "  capture: stdout={}:{}:{} stderr={}:{}:{} cleanup={}:{}",
        report.stdout_captured_bytes,
        report.stdout_truncated,
        report.stdout_hash,
        report.stderr_captured_bytes,
        report.stderr_truncated,
        report.stderr_hash,
        report.cleanup_attempted,
        report.cleanup_succeeded
    );
    println!(
        "  publication_eligibility: contract={} status={} eligible={} blockers={} ledger_hash={}",
        report.publication_eligibility_contract,
        report.publication_eligibility_status,
        report.publication_eligible,
        report.publication_blockers.join(","),
        report.probe_ledger_hash
    );
}

fn option_i32(value: Option<i32>) -> String {
    value.map_or_else(|| "none".to_owned(), |value| value.to_string())
}
