use std::{fmt::Write as _, path::Path};

pub(crate) const REGISTERED_LOADER_PROBE_OUTCOME_CONTRACT: &str =
    "nuis-nsld-registered-loader-probe-outcome-v1";

pub(crate) struct ExecutableFinalizerLoaderProbeContext<'a> {
    pub(crate) plan: &'a nuisc::linker::LinkPlan,
    pub(crate) provider_id: &'a str,
    pub(crate) target_key: &'a str,
    pub(crate) capability_id: &'a str,
    pub(crate) probe_root: &'a Path,
    pub(crate) execute: bool,
}

pub(crate) struct RegisteredLoaderProbeEvidence<'a> {
    pub(crate) provider_id: &'a str,
    pub(crate) target_key: &'a str,
    pub(crate) capability_id: &'a str,
    pub(crate) provider_probe_contract: &'a str,
    pub(crate) provider_probe_status: &'a str,
    pub(crate) probe_mode: &'a str,
    pub(crate) host_supported: bool,
    pub(crate) input_eligible: bool,
    pub(crate) attempted: bool,
    pub(crate) image_span_bytes: usize,
    pub(crate) image_identity_hash: &'a str,
    pub(crate) validation_evidence_hash: &'a str,
    pub(crate) materialized: bool,
    pub(crate) materialized_hash_matches: bool,
    pub(crate) os_loader_accepted: bool,
    pub(crate) process_completed: bool,
    pub(crate) timed_out: bool,
    pub(crate) exit_code: Option<i32>,
    pub(crate) termination_signal: Option<i32>,
    pub(crate) stdout_captured_bytes: usize,
    pub(crate) stdout_truncated: bool,
    pub(crate) stderr_captured_bytes: usize,
    pub(crate) stderr_truncated: bool,
    pub(crate) failure_kind: Option<&'a str>,
    pub(crate) cleanup_attempted: bool,
    pub(crate) cleanup_succeeded: bool,
    pub(crate) execution_admitted: bool,
    pub(crate) blockers: &'a [String],
    pub(crate) provider_evidence_hash: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldRegisteredLoaderProbeOutcome {
    pub(crate) contract: &'static str,
    pub(crate) status: &'static str,
    pub(crate) provider_id: String,
    pub(crate) target_key: String,
    pub(crate) capability_id: String,
    pub(crate) provider_probe_contract: String,
    pub(crate) provider_probe_status: String,
    pub(crate) probe_mode: String,
    pub(crate) host_supported: bool,
    pub(crate) input_eligible: bool,
    pub(crate) attempted: bool,
    pub(crate) image_span_bytes: usize,
    pub(crate) image_identity_hash: String,
    pub(crate) validation_evidence_hash: String,
    pub(crate) materialized: bool,
    pub(crate) materialized_hash_matches: bool,
    pub(crate) os_loader_accepted: bool,
    pub(crate) process_completed: bool,
    pub(crate) timed_out: bool,
    pub(crate) exit_code: Option<i32>,
    pub(crate) termination_signal: Option<i32>,
    pub(crate) stdout_captured_bytes: usize,
    pub(crate) stdout_truncated: bool,
    pub(crate) stderr_captured_bytes: usize,
    pub(crate) stderr_truncated: bool,
    pub(crate) failure_kind: Option<String>,
    pub(crate) cleanup_attempted: bool,
    pub(crate) cleanup_succeeded: bool,
    pub(crate) execution_admitted: bool,
    pub(crate) blockers: Vec<String>,
    pub(crate) provider_evidence_hash: String,
    pub(crate) outcome_ledger_hash: String,
}

pub(crate) fn build_registered_loader_probe_outcome(
    evidence: RegisteredLoaderProbeEvidence<'_>,
) -> Result<NsldRegisteredLoaderProbeOutcome, String> {
    let mut outcome = NsldRegisteredLoaderProbeOutcome {
        contract: REGISTERED_LOADER_PROBE_OUTCOME_CONTRACT,
        status: outcome_status(evidence.attempted, evidence.execution_admitted),
        provider_id: evidence.provider_id.to_owned(),
        target_key: evidence.target_key.to_owned(),
        capability_id: evidence.capability_id.to_owned(),
        provider_probe_contract: evidence.provider_probe_contract.to_owned(),
        provider_probe_status: evidence.provider_probe_status.to_owned(),
        probe_mode: evidence.probe_mode.to_owned(),
        host_supported: evidence.host_supported,
        input_eligible: evidence.input_eligible,
        attempted: evidence.attempted,
        image_span_bytes: evidence.image_span_bytes,
        image_identity_hash: evidence.image_identity_hash.to_owned(),
        validation_evidence_hash: evidence.validation_evidence_hash.to_owned(),
        materialized: evidence.materialized,
        materialized_hash_matches: evidence.materialized_hash_matches,
        os_loader_accepted: evidence.os_loader_accepted,
        process_completed: evidence.process_completed,
        timed_out: evidence.timed_out,
        exit_code: evidence.exit_code,
        termination_signal: evidence.termination_signal,
        stdout_captured_bytes: evidence.stdout_captured_bytes,
        stdout_truncated: evidence.stdout_truncated,
        stderr_captured_bytes: evidence.stderr_captured_bytes,
        stderr_truncated: evidence.stderr_truncated,
        failure_kind: evidence.failure_kind.map(str::to_owned),
        cleanup_attempted: evidence.cleanup_attempted,
        cleanup_succeeded: evidence.cleanup_succeeded,
        execution_admitted: evidence.execution_admitted,
        blockers: evidence.blockers.to_vec(),
        provider_evidence_hash: evidence.provider_evidence_hash.to_owned(),
        outcome_ledger_hash: String::new(),
    };
    outcome.outcome_ledger_hash = crate::fnv1a64_hex(outcome.canonical_ledger().as_bytes());
    validate_registered_loader_probe_outcome(&outcome)?;
    Ok(outcome)
}

pub(crate) fn validate_registered_loader_probe_outcome(
    outcome: &NsldRegisteredLoaderProbeOutcome,
) -> Result<(), String> {
    if outcome.contract != REGISTERED_LOADER_PROBE_OUTCOME_CONTRACT
        || outcome.status != outcome_status(outcome.attempted, outcome.execution_admitted)
        || !matches!(outcome.probe_mode.as_str(), "plan-only" | "execute")
    {
        return Err("registered loader-probe outcome contract drift".to_owned());
    }
    if [
        outcome.provider_id.as_str(),
        outcome.target_key.as_str(),
        outcome.capability_id.as_str(),
        outcome.provider_probe_contract.as_str(),
        outcome.provider_probe_status.as_str(),
        outcome.image_identity_hash.as_str(),
        outcome.validation_evidence_hash.as_str(),
        outcome.provider_evidence_hash.as_str(),
    ]
    .iter()
    .any(|value| value.is_empty())
        || outcome.image_span_bytes == 0
    {
        return Err("registered loader-probe outcome identity is incomplete".to_owned());
    }
    if outcome.attempted && outcome.probe_mode != "execute" {
        return Err("registered loader-probe attempted outside execute mode".to_owned());
    }
    if !outcome.attempted
        && (outcome.materialized
            || outcome.materialized_hash_matches
            || outcome.os_loader_accepted
            || outcome.process_completed
            || outcome.cleanup_attempted)
    {
        return Err(
            "registered loader-probe plan-only evidence crossed execution boundary".to_owned(),
        );
    }
    if outcome.execution_admitted
        && (!outcome.host_supported
            || !outcome.input_eligible
            || !outcome.attempted
            || !outcome.materialized
            || !outcome.materialized_hash_matches
            || !outcome.os_loader_accepted
            || !outcome.process_completed
            || outcome.timed_out
            || outcome.exit_code != Some(0)
            || outcome.termination_signal.is_some()
            || outcome.stdout_truncated
            || outcome.stderr_truncated
            || outcome.failure_kind.is_some()
            || !outcome.cleanup_attempted
            || !outcome.cleanup_succeeded
            || !outcome.blockers.is_empty())
    {
        return Err("registered loader-probe admitted incomplete execution evidence".to_owned());
    }
    if !outcome.execution_admitted && outcome.blockers.is_empty() {
        return Err("registered loader-probe blocked outcome has no blocker".to_owned());
    }
    if outcome.outcome_ledger_hash != crate::fnv1a64_hex(outcome.canonical_ledger().as_bytes()) {
        return Err("registered loader-probe outcome ledger drift".to_owned());
    }
    Ok(())
}

impl NsldRegisteredLoaderProbeOutcome {
    fn canonical_ledger(&self) -> String {
        let mut ledger = String::new();
        ledger_field(&mut ledger, "contract", self.contract);
        ledger_field(&mut ledger, "status", self.status);
        ledger_field(&mut ledger, "provider_id", &self.provider_id);
        ledger_field(&mut ledger, "target_key", &self.target_key);
        ledger_field(&mut ledger, "capability_id", &self.capability_id);
        ledger_field(
            &mut ledger,
            "provider_probe_contract",
            &self.provider_probe_contract,
        );
        ledger_field(
            &mut ledger,
            "provider_probe_status",
            &self.provider_probe_status,
        );
        ledger_field(&mut ledger, "probe_mode", &self.probe_mode);
        ledger_field(
            &mut ledger,
            "host_supported",
            &self.host_supported.to_string(),
        );
        ledger_field(
            &mut ledger,
            "input_eligible",
            &self.input_eligible.to_string(),
        );
        ledger_field(&mut ledger, "attempted", &self.attempted.to_string());
        ledger_field(
            &mut ledger,
            "image_span_bytes",
            &self.image_span_bytes.to_string(),
        );
        ledger_field(
            &mut ledger,
            "image_identity_hash",
            &self.image_identity_hash,
        );
        ledger_field(
            &mut ledger,
            "validation_evidence_hash",
            &self.validation_evidence_hash,
        );
        ledger_field(&mut ledger, "materialized", &self.materialized.to_string());
        ledger_field(
            &mut ledger,
            "materialized_hash_matches",
            &self.materialized_hash_matches.to_string(),
        );
        ledger_field(
            &mut ledger,
            "os_loader_accepted",
            &self.os_loader_accepted.to_string(),
        );
        ledger_field(
            &mut ledger,
            "process_completed",
            &self.process_completed.to_string(),
        );
        ledger_field(&mut ledger, "timed_out", &self.timed_out.to_string());
        ledger_field(&mut ledger, "exit_code", &optional_i32(self.exit_code));
        ledger_field(
            &mut ledger,
            "termination_signal",
            &optional_i32(self.termination_signal),
        );
        ledger_field(
            &mut ledger,
            "stdout_captured_bytes",
            &self.stdout_captured_bytes.to_string(),
        );
        ledger_field(
            &mut ledger,
            "stdout_truncated",
            &self.stdout_truncated.to_string(),
        );
        ledger_field(
            &mut ledger,
            "stderr_captured_bytes",
            &self.stderr_captured_bytes.to_string(),
        );
        ledger_field(
            &mut ledger,
            "stderr_truncated",
            &self.stderr_truncated.to_string(),
        );
        ledger_field(
            &mut ledger,
            "failure_kind",
            self.failure_kind.as_deref().unwrap_or("none"),
        );
        ledger_field(
            &mut ledger,
            "cleanup_attempted",
            &self.cleanup_attempted.to_string(),
        );
        ledger_field(
            &mut ledger,
            "cleanup_succeeded",
            &self.cleanup_succeeded.to_string(),
        );
        ledger_field(
            &mut ledger,
            "execution_admitted",
            &self.execution_admitted.to_string(),
        );
        ledger_field(
            &mut ledger,
            "blocker_count",
            &self.blockers.len().to_string(),
        );
        for (index, blocker) in self.blockers.iter().enumerate() {
            ledger_field(&mut ledger, &format!("blocker.{index}"), blocker);
        }
        ledger_field(
            &mut ledger,
            "provider_evidence_hash",
            &self.provider_evidence_hash,
        );
        ledger
    }
}

fn outcome_status(attempted: bool, admitted: bool) -> &'static str {
    if admitted {
        "execution-admitted"
    } else if attempted {
        "execution-not-admitted"
    } else {
        "execution-not-attempted"
    }
}

fn optional_i32(value: Option<i32>) -> String {
    value.map_or_else(|| "none".to_owned(), |value| value.to_string())
}

fn ledger_field(ledger: &mut String, key: &str, value: &str) {
    writeln!(ledger, "{key}={}:{}", value.len(), value).unwrap();
}
