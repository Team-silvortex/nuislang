use std::fmt::Write as _;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64LoaderProbeReport {
    pub(crate) contract: &'static str,
    pub(crate) status: String,
    pub(crate) probe_mode: &'static str,
    pub(crate) materialization_kind: &'static str,
    pub(crate) target_arch: &'static str,
    pub(crate) target_os: &'static str,
    pub(crate) host_supported: bool,
    pub(crate) input_eligible: bool,
    pub(crate) attempted: bool,
    pub(crate) image_span_bytes: usize,
    pub(crate) shell_image_hash: String,
    pub(crate) validation_contract: &'static str,
    pub(crate) validation_ledger_hash: String,
    pub(crate) serialization_ledger_hash: String,
    pub(crate) dynamic_provenance_contract: Option<String>,
    pub(crate) dynamic_provenance_ledger_hash: Option<String>,
    pub(crate) dynamic_provenance_ready: bool,
    pub(crate) unresolved_external_symbol_count: usize,
    pub(crate) dynamic_segment_count: usize,
    pub(crate) dynamic_entry_count: usize,
    pub(crate) probe_timeout_millis: u64,
    pub(crate) materialized: bool,
    pub(crate) materialized_hash_matches: bool,
    pub(crate) kernel_accepted: bool,
    pub(crate) process_completed: bool,
    pub(crate) timed_out: bool,
    pub(crate) exit_code: Option<i32>,
    pub(crate) termination_signal: Option<i32>,
    pub(crate) stdout_captured_bytes: usize,
    pub(crate) stdout_truncated: bool,
    pub(crate) stdout_hash: String,
    pub(crate) stderr_captured_bytes: usize,
    pub(crate) stderr_truncated: bool,
    pub(crate) stderr_hash: String,
    pub(crate) failure_kind: Option<String>,
    pub(crate) cleanup_attempted: bool,
    pub(crate) cleanup_succeeded: bool,
    pub(crate) publication_eligibility_contract: &'static str,
    pub(crate) publication_eligibility_status: String,
    pub(crate) publication_eligible: bool,
    pub(crate) publication_blockers: Vec<String>,
    pub(crate) probe_ledger_hash: String,
}

impl ElfAmd64LoaderProbeReport {
    pub(crate) fn canonical_ledger(&self) -> String {
        let mut out = String::new();
        for value in [
            self.contract,
            &self.status,
            self.probe_mode,
            self.materialization_kind,
            self.target_arch,
            self.target_os,
            &self.shell_image_hash,
            self.validation_contract,
            &self.validation_ledger_hash,
            &self.serialization_ledger_hash,
            self.dynamic_provenance_contract
                .as_deref()
                .unwrap_or("none"),
            self.dynamic_provenance_ledger_hash
                .as_deref()
                .unwrap_or("none"),
            self.failure_kind.as_deref().unwrap_or("none"),
            self.publication_eligibility_contract,
            &self.publication_eligibility_status,
        ] {
            append_text(&mut out, value);
        }
        writeln!(
            out,
            "input={}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
            self.host_supported,
            self.input_eligible,
            self.attempted,
            self.image_span_bytes,
            self.unresolved_external_symbol_count,
            self.dynamic_segment_count,
            self.dynamic_entry_count,
            self.dynamic_provenance_ready,
            self.probe_timeout_millis,
            self.materialized,
            self.materialized_hash_matches,
        )
        .unwrap();
        writeln!(
            out,
            "result={}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
            self.kernel_accepted,
            self.process_completed,
            self.timed_out,
            optional_i32(self.exit_code),
            optional_i32(self.termination_signal),
            self.stdout_captured_bytes,
            self.stdout_truncated,
            self.stderr_captured_bytes,
            self.stderr_truncated,
            self.cleanup_attempted,
            self.cleanup_succeeded,
            self.publication_eligible,
            self.publication_blockers.len(),
        )
        .unwrap();
        append_text(&mut out, &self.stdout_hash);
        append_text(&mut out, &self.stderr_hash);
        for blocker in &self.publication_blockers {
            append_text(&mut out, blocker);
        }
        out
    }
}

fn append_text(out: &mut String, value: &str) {
    writeln!(out, "text:{}:{value}", value.len()).unwrap();
}

fn optional_i32(value: Option<i32>) -> String {
    value.map_or_else(|| "none".to_owned(), |value| value.to_string())
}
