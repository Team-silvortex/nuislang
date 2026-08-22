use crate::final_executable_registered_loader_probe::NsldRegisteredLoaderProbeOutcome;

pub(crate) fn print_registered_loader_probe_outcome(outcome: &NsldRegisteredLoaderProbeOutcome) {
    println!("Nsld registered loader probe");
    println!(
        "  outcome: contract={} status={} provider={} target={} capability={} mode={}",
        outcome.contract,
        outcome.status,
        outcome.provider_id,
        outcome.target_key,
        outcome.capability_id,
        outcome.probe_mode
    );
    println!(
        "  provider: contract={} status={} evidence={}",
        outcome.provider_probe_contract,
        outcome.provider_probe_status,
        outcome.provider_evidence_hash
    );
    println!(
        "  execution: host_supported={} input_eligible={} attempted={} materialized={} hash_matches={} loader_accepted={} completed={} timed_out={} exit={} signal={} admitted={}",
        outcome.host_supported,
        outcome.input_eligible,
        outcome.attempted,
        outcome.materialized,
        outcome.materialized_hash_matches,
        outcome.os_loader_accepted,
        outcome.process_completed,
        outcome.timed_out,
        optional_i32(outcome.exit_code),
        optional_i32(outcome.termination_signal),
        outcome.execution_admitted
    );
    println!(
        "  capture: stdout_bytes={} stdout_truncated={} stderr_bytes={} stderr_truncated={} failure={} cleanup_attempted={} cleanup_succeeded={}",
        outcome.stdout_captured_bytes,
        outcome.stdout_truncated,
        outcome.stderr_captured_bytes,
        outcome.stderr_truncated,
        outcome.failure_kind.as_deref().unwrap_or("none"),
        outcome.cleanup_attempted,
        outcome.cleanup_succeeded
    );
    println!(
        "  image: bytes={} identity={} validation={} blockers={} ledger={}",
        outcome.image_span_bytes,
        outcome.image_identity_hash,
        outcome.validation_evidence_hash,
        if outcome.blockers.is_empty() {
            "none".to_owned()
        } else {
            outcome.blockers.join(",")
        },
        outcome.outcome_ledger_hash
    );
}

fn optional_i32(value: Option<i32>) -> String {
    value.map_or_else(|| "none".to_owned(), |value| value.to_string())
}
