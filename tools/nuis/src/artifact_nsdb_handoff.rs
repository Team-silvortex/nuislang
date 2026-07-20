use crate::{
    artifact_launch_evidence::RunArtifactLaunchEvidence, json_bool_field, json_field,
    json_optional_string_field, json_usize_field,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

const NSDB_HANDOFF_PROTOCOL: &str = "nuis-nsdb-payload-execution-handoff-v1";
const NSDB_HANDOFF_FILE_NAME: &str = "nuis.nsdb.payload-execution-handoff.toml";
const PROVIDER_COMPLETION_DIGEST_FNV1A64_CONTRACT: &str =
    "nuis-provider-completion-digest-fnv1a64-v1";
const PROVIDER_COMPLETION_DIGEST_SHA256_CONTRACT: &str =
    "nuis-provider-completion-digest-sha256-v1";
const PROVIDER_COMPLETION_DIGEST_SHA256_AUTHORITY_CONTRACT: &str =
    "nuis-provider-completion-digest-sha256-authority-v1";
const PROVIDER_COMPLETION_CLAIM_AUTHORITY_CONTRACT: &str =
    "nuis-provider-completion-claim-authority-v1";
const PROVIDER_COMPLETION_CLAIM_AUTHORITY: &str = "nsdb:payload-execution-handoff-writer:v1";

pub(crate) struct LaunchEvidenceNsdbHandoffPersistence {
    persisted: bool,
    path: Option<PathBuf>,
    record_count: usize,
    ready_record_count: usize,
    first_trace_id: Option<String>,
    error: Option<String>,
}

pub(crate) struct PersistedNsdbHandoffSummary {
    available: bool,
    path: PathBuf,
    protocol: Option<String>,
    debugger_contract: Option<String>,
    record_count: usize,
    ready_record_count: usize,
    first_trace_id: Option<String>,
    first_status: Option<String>,
    first_next_action: Option<String>,
    provider_completion_count: usize,
    first_provider_family: Option<String>,
    first_provider_output_contract: Option<String>,
    first_provider_output_evidence: Option<String>,
    provider_completion_claim_authority_contract: Option<String>,
    provider_completion_claim_authority: Option<String>,
    provider_completion_claim_authority_status: String,
    provider_completion_digest_contract: Option<String>,
    provider_completion_set_hash_claim: Option<String>,
    provider_completion_set_hash: Option<String>,
    provider_completion_set_hash_validation_status: String,
    provider_completions: Vec<PersistedProviderCompletion>,
    hetero_execution_closure_status: Option<String>,
    hetero_execution_closure_ready: Option<String>,
    hetero_execution_closure_first_blocker: Option<String>,
    hetero_execution_closure_next_action: Option<String>,
    error: Option<String>,
}

#[derive(Clone)]
pub(crate) struct PersistedProviderCompletion {
    pub(crate) trace_id: String,
    pub(crate) provider_family: String,
    pub(crate) output_contract: String,
    pub(crate) output_evidence: String,
    pub(crate) record_hash: String,
}

impl PersistedNsdbHandoffSummary {
    pub(crate) fn available(&self) -> bool {
        self.available
    }

    pub(crate) fn record_count(&self) -> usize {
        self.record_count
    }

    pub(crate) fn ready_record_count(&self) -> usize {
        self.ready_record_count
    }

    pub(crate) fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub(crate) fn provider_completion_count(&self) -> usize {
        self.provider_completion_count
    }

    pub(crate) fn first_provider_family(&self) -> Option<&str> {
        self.first_provider_family.as_deref()
    }

    pub(crate) fn first_provider_output_contract(&self) -> Option<&str> {
        self.first_provider_output_contract.as_deref()
    }

    pub(crate) fn first_provider_output_evidence(&self) -> Option<&str> {
        self.first_provider_output_evidence.as_deref()
    }

    pub(crate) fn provider_completion_set_hash(&self) -> Option<&str> {
        self.provider_completion_set_hash.as_deref()
    }

    pub(crate) fn provider_completion_digest_contract(&self) -> Option<&str> {
        self.provider_completion_digest_contract.as_deref()
    }

    pub(crate) fn provider_completion_claim_authority_contract(&self) -> Option<&str> {
        self.provider_completion_claim_authority_contract.as_deref()
    }

    pub(crate) fn provider_completion_claim_authority(&self) -> Option<&str> {
        self.provider_completion_claim_authority.as_deref()
    }

    pub(crate) fn provider_completion_claim_authority_status(&self) -> &str {
        &self.provider_completion_claim_authority_status
    }

    pub(crate) fn provider_completion_set_hash_claim(&self) -> Option<&str> {
        self.provider_completion_set_hash_claim.as_deref()
    }

    pub(crate) fn provider_completion_set_hash_validation_status(&self) -> &str {
        &self.provider_completion_set_hash_validation_status
    }

    pub(crate) fn provider_completions(&self) -> &[PersistedProviderCompletion] {
        &self.provider_completions
    }

    pub(crate) fn hetero_execution_closure_ready(&self) -> bool {
        match (
            self.hetero_execution_closure_status.as_deref(),
            self.hetero_execution_closure_ready.as_deref(),
        ) {
            (None, _) => true,
            (Some("closed"), Some("true")) => true,
            _ => false,
        }
    }

    pub(crate) fn hetero_execution_closure_blocker(&self) -> Option<String> {
        if self.hetero_execution_closure_ready() {
            return None;
        }
        self.hetero_execution_closure_first_blocker
            .clone()
            .filter(|value| !value.is_empty())
            .or_else(|| self.hetero_execution_closure_status.clone())
            .map(|value| format!("hetero-execution-closure:{value}"))
    }

    pub(crate) fn json_fields_with_prefix(&self, prefix: &str) -> Vec<String> {
        vec![
            json_bool_field(&format!("{prefix}_available"), self.available),
            json_optional_string_field(&format!("{prefix}_protocol"), self.protocol.as_deref()),
            json_optional_string_field(
                &format!("{prefix}_debugger_contract"),
                self.debugger_contract.as_deref(),
            ),
            json_field(&format!("{prefix}_path"), &self.path.display().to_string()),
            json_usize_field(&format!("{prefix}_record_count"), self.record_count),
            json_usize_field(
                &format!("{prefix}_ready_record_count"),
                self.ready_record_count,
            ),
            json_optional_string_field(
                &format!("{prefix}_first_trace_id"),
                self.first_trace_id.as_deref(),
            ),
            json_optional_string_field(
                &format!("{prefix}_first_status"),
                self.first_status.as_deref(),
            ),
            json_optional_string_field(
                &format!("{prefix}_first_next_action"),
                self.first_next_action.as_deref(),
            ),
            json_usize_field(
                &format!("{prefix}_provider_completion_count"),
                self.provider_completion_count,
            ),
            json_optional_string_field(
                &format!("{prefix}_first_provider_family"),
                self.first_provider_family.as_deref(),
            ),
            json_optional_string_field(
                &format!("{prefix}_first_provider_output_contract"),
                self.first_provider_output_contract.as_deref(),
            ),
            json_optional_string_field(
                &format!("{prefix}_first_provider_output_evidence"),
                self.first_provider_output_evidence.as_deref(),
            ),
            json_optional_string_field(
                &format!("{prefix}_provider_completion_claim_authority_contract"),
                self.provider_completion_claim_authority_contract.as_deref(),
            ),
            json_optional_string_field(
                &format!("{prefix}_provider_completion_claim_authority"),
                self.provider_completion_claim_authority.as_deref(),
            ),
            json_field(
                &format!("{prefix}_provider_completion_claim_authority_status"),
                &self.provider_completion_claim_authority_status,
            ),
            json_optional_string_field(
                &format!("{prefix}_provider_completion_digest_contract"),
                self.provider_completion_digest_contract.as_deref(),
            ),
            json_optional_string_field(
                &format!("{prefix}_provider_completion_set_hash_claim"),
                self.provider_completion_set_hash_claim.as_deref(),
            ),
            json_optional_string_field(
                &format!("{prefix}_provider_completion_set_hash"),
                self.provider_completion_set_hash.as_deref(),
            ),
            json_field(
                &format!("{prefix}_provider_completion_set_hash_validation_status"),
                &self.provider_completion_set_hash_validation_status,
            ),
            json_optional_string_field(
                &format!("{prefix}_hetero_execution_closure_status"),
                self.hetero_execution_closure_status.as_deref(),
            ),
            json_optional_string_field(
                &format!("{prefix}_hetero_execution_closure_ready"),
                self.hetero_execution_closure_ready.as_deref(),
            ),
            json_optional_string_field(
                &format!("{prefix}_hetero_execution_closure_next_action"),
                self.hetero_execution_closure_next_action.as_deref(),
            ),
            json_optional_string_field(&format!("{prefix}_error"), self.error.as_deref()),
        ]
    }
}

pub(crate) fn read_persisted_nsdb_handoff(
    output_dir: Option<&Path>,
) -> PersistedNsdbHandoffSummary {
    let Some(output_dir) = output_dir else {
        return PersistedNsdbHandoffSummary {
            available: false,
            path: PathBuf::from(NSDB_HANDOFF_FILE_NAME),
            protocol: None,
            debugger_contract: None,
            record_count: 0,
            ready_record_count: 0,
            first_trace_id: None,
            first_status: None,
            first_next_action: None,
            provider_completion_count: 0,
            first_provider_family: None,
            first_provider_output_contract: None,
            first_provider_output_evidence: None,
            provider_completion_claim_authority_contract: None,
            provider_completion_claim_authority: None,
            provider_completion_claim_authority_status: "not-applicable".to_owned(),
            provider_completion_digest_contract: None,
            provider_completion_set_hash_claim: None,
            provider_completion_set_hash: None,
            provider_completion_set_hash_validation_status: "not-applicable".to_owned(),
            provider_completions: Vec::new(),
            hetero_execution_closure_status: None,
            hetero_execution_closure_ready: None,
            hetero_execution_closure_first_blocker: None,
            hetero_execution_closure_next_action: None,
            error: Some("output_dir-unavailable".to_owned()),
        };
    };
    let path = output_dir.join(NSDB_HANDOFF_FILE_NAME);
    let Ok(source) = fs::read_to_string(&path) else {
        return PersistedNsdbHandoffSummary {
            available: false,
            path,
            protocol: None,
            debugger_contract: None,
            record_count: 0,
            ready_record_count: 0,
            first_trace_id: None,
            first_status: None,
            first_next_action: None,
            provider_completion_count: 0,
            first_provider_family: None,
            first_provider_output_contract: None,
            first_provider_output_evidence: None,
            provider_completion_claim_authority_contract: None,
            provider_completion_claim_authority: None,
            provider_completion_claim_authority_status: "not-applicable".to_owned(),
            provider_completion_digest_contract: None,
            provider_completion_set_hash_claim: None,
            provider_completion_set_hash: None,
            provider_completion_set_hash_validation_status: "not-applicable".to_owned(),
            provider_completions: Vec::new(),
            hetero_execution_closure_status: None,
            hetero_execution_closure_ready: None,
            hetero_execution_closure_first_blocker: None,
            hetero_execution_closure_next_action: None,
            error: Some("handoff-metadata-missing".to_owned()),
        };
    };
    let protocol = parse_string_toml_field(&source, "protocol");
    let record_count = parse_usize_toml_field(&source, "record_count").unwrap_or(0);
    let provider_completion_claim_authority_contract =
        parse_string_toml_field(&source, "provider_completion_claim_authority_contract")
            .filter(|value| value != "none" && !value.is_empty());
    let provider_completion_claim_authority =
        parse_string_toml_field(&source, "provider_completion_claim_authority")
            .filter(|value| value != "none" && !value.is_empty());
    let provider_completion_digest_contract =
        parse_string_toml_field(&source, "provider_completion_digest_contract")
            .filter(|value| value != "none" && !value.is_empty());
    let record_digest_contract = provider_completion_digest_contract
        .as_deref()
        .unwrap_or(PROVIDER_COMPLETION_DIGEST_FNV1A64_CONTRACT);
    let provider_completions = source
        .split("[[records]]")
        .skip(1)
        .filter(|record| {
            parse_string_toml_field(record, "execution_phase").as_deref()
                == Some("provider-device-completion")
        })
        .map(|record| {
            let trace_id =
                parse_string_toml_field(record, "trace_id").unwrap_or_else(|| "none".to_owned());
            let provider_family = parse_string_toml_field(record, "provider_family")
                .unwrap_or_else(|| "none".to_owned());
            let output_contract = parse_string_toml_field(record, "output_contract")
                .unwrap_or_else(|| "none".to_owned());
            let output_evidence = parse_string_toml_field(record, "output_evidence")
                .unwrap_or_else(|| "none".to_owned());
            let material =
                format!("{trace_id}\0{provider_family}\0{output_contract}\0{output_evidence}");
            PersistedProviderCompletion {
                trace_id,
                provider_family,
                output_contract,
                output_evidence,
                record_hash: digest_hex(record_digest_contract, material.as_bytes())
                    .unwrap_or_else(|| "none".to_owned()),
            }
        })
        .collect::<Vec<_>>();
    let first_provider_completion = provider_completions.first();
    let legacy_provider_completion_set_hash = (!provider_completions.is_empty()).then(|| {
        let material = provider_completions
            .iter()
            .map(|completion| completion.record_hash.as_str())
            .collect::<Vec<_>>()
            .join("\0");
        fnv1a64_hex(format!("provider-completion-set-v1\0{material}").as_bytes())
    });
    let provider_completion_set_hash = match provider_completion_digest_contract.as_deref() {
        None => legacy_provider_completion_set_hash,
        Some(
            contract @ (PROVIDER_COMPLETION_DIGEST_FNV1A64_CONTRACT
            | PROVIDER_COMPLETION_DIGEST_SHA256_CONTRACT
            | PROVIDER_COMPLETION_DIGEST_SHA256_AUTHORITY_CONTRACT),
        ) => (!provider_completions.is_empty()).then(|| {
            let material = provider_completions
                .iter()
                .map(|completion| completion.record_hash.as_str())
                .collect::<Vec<_>>()
                .join("\0");
            let (domain, authority_material) = match contract {
                PROVIDER_COMPLETION_DIGEST_SHA256_AUTHORITY_CONTRACT => (
                    "provider-completion-set-v4",
                    format!(
                        "{}\0{}\0",
                        provider_completion_claim_authority_contract
                            .as_deref()
                            .unwrap_or("none"),
                        provider_completion_claim_authority
                            .as_deref()
                            .unwrap_or("none")
                    ),
                ),
                PROVIDER_COMPLETION_DIGEST_SHA256_CONTRACT => {
                    ("provider-completion-set-v3", String::new())
                }
                _ => ("provider-completion-set-v2", String::new()),
            };
            digest_hex(
                contract,
                format!(
                    "{domain}\0{authority_material}{}\0{record_count}\0{}\0{material}",
                    protocol.as_deref().unwrap_or("none"),
                    provider_completions.len()
                )
                .as_bytes(),
            )
            .expect("validated provider completion digest contract")
        }),
        Some(_) => None,
    };
    let provider_completion_set_hash_claim =
        parse_string_toml_field(&source, "provider_completion_set_hash")
            .filter(|value| value != "none" && !value.is_empty());
    let provider_completion_set_hash_validation_status = if provider_completions.is_empty() {
        "not-applicable"
    } else if provider_completion_digest_contract.is_some()
        && provider_completion_digest_contract.as_deref()
            != Some(PROVIDER_COMPLETION_DIGEST_FNV1A64_CONTRACT)
        && provider_completion_digest_contract.as_deref()
            != Some(PROVIDER_COMPLETION_DIGEST_SHA256_CONTRACT)
        && provider_completion_digest_contract.as_deref()
            != Some(PROVIDER_COMPLETION_DIGEST_SHA256_AUTHORITY_CONTRACT)
    {
        "unsupported-digest-contract"
    } else if provider_completion_set_hash_claim.is_none() {
        "legacy-unclaimed"
    } else if provider_completion_set_hash_claim == provider_completion_set_hash {
        if provider_completion_digest_contract.is_some() {
            "verified"
        } else {
            "legacy-verified"
        }
    } else {
        "mismatch"
    }
    .to_owned();
    let provider_completion_claim_authority_status = if provider_completions.is_empty() {
        "not-applicable"
    } else if provider_completion_digest_contract.as_deref()
        != Some(PROVIDER_COMPLETION_DIGEST_SHA256_AUTHORITY_CONTRACT)
    {
        "legacy-unattributed"
    } else if provider_completion_claim_authority_contract.is_none()
        || provider_completion_claim_authority.is_none()
    {
        "authority-missing"
    } else if provider_completion_claim_authority_contract.as_deref()
        != Some(PROVIDER_COMPLETION_CLAIM_AUTHORITY_CONTRACT)
    {
        "unsupported-authority-contract"
    } else if provider_completion_claim_authority.as_deref()
        != Some(PROVIDER_COMPLETION_CLAIM_AUTHORITY)
    {
        "authority-untrusted"
    } else {
        "authorized"
    }
    .to_owned();
    let error = match (
        provider_completion_set_hash_validation_status.as_str(),
        provider_completion_claim_authority_status.as_str(),
    ) {
        ("mismatch", _) => Some("provider-completion-set-hash-mismatch".to_owned()),
        ("unsupported-digest-contract", _) => {
            Some("provider-completion-digest-contract-unsupported".to_owned())
        }
        (_, "authority-missing") => Some("provider-completion-claim-authority-missing".to_owned()),
        (_, "unsupported-authority-contract") => {
            Some("provider-completion-claim-authority-contract-unsupported".to_owned())
        }
        (_, "authority-untrusted") => {
            Some("provider-completion-claim-authority-untrusted".to_owned())
        }
        _ => None,
    };
    PersistedNsdbHandoffSummary {
        available: true,
        path,
        protocol,
        debugger_contract: parse_string_toml_field(&source, "debugger_contract"),
        record_count,
        ready_record_count: parse_usize_toml_field(&source, "ready_record_count").unwrap_or(0),
        first_trace_id: parse_string_toml_field(&source, "first_trace_id"),
        first_status: parse_string_toml_field(&source, "first_status"),
        first_next_action: parse_string_toml_field(&source, "first_next_action"),
        provider_completion_count: provider_completions.len(),
        first_provider_family: first_provider_completion
            .map(|completion| completion.provider_family.clone())
            .filter(|value| value != "none" && !value.is_empty()),
        first_provider_output_contract: first_provider_completion
            .map(|completion| completion.output_contract.clone())
            .filter(|value| value != "none" && !value.is_empty()),
        first_provider_output_evidence: first_provider_completion
            .map(|completion| completion.output_evidence.clone())
            .filter(|value| value != "none" && !value.is_empty()),
        provider_completion_claim_authority_contract,
        provider_completion_claim_authority,
        provider_completion_claim_authority_status,
        provider_completion_digest_contract,
        provider_completion_set_hash_claim,
        provider_completion_set_hash,
        provider_completion_set_hash_validation_status,
        provider_completions,
        hetero_execution_closure_status: parse_string_toml_field(
            &source,
            "hetero_execution_closure_status",
        ),
        hetero_execution_closure_ready: parse_string_toml_field(
            &source,
            "hetero_execution_closure_ready",
        ),
        hetero_execution_closure_first_blocker: parse_string_toml_field(
            &source,
            "hetero_execution_closure_first_blocker",
        )
        .filter(|value| !value.is_empty()),
        hetero_execution_closure_next_action: parse_string_toml_field(
            &source,
            "hetero_execution_closure_next_action",
        ),
        error,
    }
}

#[cfg(test)]
#[path = "artifact_nsdb_handoff_tests.rs"]
mod provider_completion_tests;

impl LaunchEvidenceNsdbHandoffPersistence {
    pub(crate) fn json_fields(&self) -> Vec<String> {
        vec![
            json_field(
                "launch_evidence_nsdb_handoff_protocol",
                NSDB_HANDOFF_PROTOCOL,
            ),
            json_bool_field("launch_evidence_nsdb_handoff_persisted", self.persisted),
            json_optional_string_field(
                "launch_evidence_nsdb_handoff_path",
                self.path
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .as_deref(),
            ),
            json_usize_field(
                "launch_evidence_nsdb_handoff_record_count",
                self.record_count,
            ),
            json_usize_field(
                "launch_evidence_nsdb_handoff_ready_record_count",
                self.ready_record_count,
            ),
            json_optional_string_field(
                "launch_evidence_nsdb_handoff_first_trace_id",
                self.first_trace_id.as_deref(),
            ),
            json_optional_string_field("launch_evidence_nsdb_handoff_error", self.error.as_deref()),
        ]
    }

    pub(crate) fn print_text(&self) {
        println!("  launch_evidence_nsdb_handoff_protocol: {NSDB_HANDOFF_PROTOCOL}");
        println!(
            "  launch_evidence_nsdb_handoff_persisted: {}",
            self.persisted
        );
        println!(
            "  launch_evidence_nsdb_handoff_path: {}",
            self.path
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "<none>".to_owned())
        );
        println!(
            "  launch_evidence_nsdb_handoff_record_count: {}",
            self.record_count
        );
        println!(
            "  launch_evidence_nsdb_handoff_ready_record_count: {}",
            self.ready_record_count
        );
        println!(
            "  launch_evidence_nsdb_handoff_first_trace_id: {}",
            self.first_trace_id.as_deref().unwrap_or("<none>")
        );
        println!(
            "  launch_evidence_nsdb_handoff_error: {}",
            self.error.as_deref().unwrap_or("<none>")
        );
    }
}

pub(crate) fn persist_launch_evidence_nsdb_handoff(
    output_dir: Option<&Path>,
    evidence: &RunArtifactLaunchEvidence,
) -> LaunchEvidenceNsdbHandoffPersistence {
    let records = evidence.payload_execution_trace_records();
    let ready_record_count = records
        .iter()
        .filter(|record| record.status == "ready")
        .count();
    let first_trace_id = records.first().map(|record| record.trace_id.clone());

    let Some(output_dir) = output_dir else {
        return LaunchEvidenceNsdbHandoffPersistence {
            persisted: false,
            path: None,
            record_count: records.len(),
            ready_record_count,
            first_trace_id,
            error: Some("output_dir-unavailable".to_owned()),
        };
    };
    if records.is_empty() {
        return LaunchEvidenceNsdbHandoffPersistence {
            persisted: false,
            path: Some(output_dir.join(NSDB_HANDOFF_FILE_NAME)),
            record_count: 0,
            ready_record_count: 0,
            first_trace_id: None,
            error: Some("payload-execution-trace-unavailable".to_owned()),
        };
    }

    let path = output_dir.join(NSDB_HANDOFF_FILE_NAME);
    let content = render_launch_evidence_nsdb_handoff(evidence);
    match fs::write(&path, content) {
        Ok(()) => LaunchEvidenceNsdbHandoffPersistence {
            persisted: true,
            path: Some(path),
            record_count: records.len(),
            ready_record_count,
            first_trace_id,
            error: None,
        },
        Err(error) => LaunchEvidenceNsdbHandoffPersistence {
            persisted: false,
            path: Some(path),
            record_count: records.len(),
            ready_record_count,
            first_trace_id,
            error: Some(error.to_string()),
        },
    }
}

fn render_launch_evidence_nsdb_handoff(evidence: &RunArtifactLaunchEvidence) -> String {
    let records = evidence.payload_execution_trace_records();
    let ready_record_count = records
        .iter()
        .filter(|record| record.status == "ready")
        .count();
    let mut out = String::new();
    push_toml_string(&mut out, "protocol", NSDB_HANDOFF_PROTOCOL);
    push_toml_string(
        &mut out,
        "debugger_contract",
        evidence.payload_execution_trace_protocol(),
    );
    push_toml_string(&mut out, "source", "run-artifact-launch-evidence");
    out.push_str(&format!("record_count = {}\n", records.len()));
    out.push_str(&format!("ready_record_count = {ready_record_count}\n"));
    push_toml_string(
        &mut out,
        "hetero_execution_closure_protocol",
        evidence.hetero_execution_closure_protocol(),
    );
    push_toml_string(
        &mut out,
        "hetero_execution_closure_status",
        evidence.hetero_execution_closure_status(),
    );
    push_toml_string(
        &mut out,
        "hetero_execution_closure_ready",
        if evidence.hetero_execution_closure_ready() {
            "true"
        } else {
            "false"
        },
    );
    push_toml_optional_string(
        &mut out,
        "hetero_execution_closure_first_blocker",
        evidence.hetero_execution_closure_first_blocker(),
    );
    push_toml_string(
        &mut out,
        "hetero_execution_closure_next_action",
        evidence.hetero_execution_closure_next_action(),
    );
    if let Some(first) = records.first() {
        push_toml_string(&mut out, "first_trace_id", &first.trace_id);
        push_toml_string(&mut out, "first_status", &first.status);
        push_toml_string(&mut out, "first_next_action", &first.next_action);
    }
    for record in records {
        out.push_str("\n[[records]]\n");
        push_toml_string(&mut out, "trace_id", &record.trace_id);
        push_toml_string(&mut out, "status", &record.status);
        push_toml_string(&mut out, "execution_phase", &record.execution_phase);
        push_toml_optional_string(&mut out, "target", record.target.as_deref());
        push_toml_optional_string(&mut out, "entry_symbol", record.entry_symbol.as_deref());
        push_toml_optional_string(&mut out, "entry_kind", record.entry_kind.as_deref());
        push_toml_optional_string(
            &mut out,
            "entry_section_id",
            record.entry_section_id.as_deref(),
        );
        push_toml_optional_string(&mut out, "first_blocker", record.first_blocker.as_deref());
        push_toml_string(&mut out, "next_action", &record.next_action);
    }
    out
}

fn push_toml_optional_string(out: &mut String, key: &str, value: Option<&str>) {
    match value {
        Some(value) => push_toml_string(out, key, value),
        None => out.push_str(&format!("{key} = \"\"\n")),
    }
}

fn push_toml_string(out: &mut String, key: &str, value: &str) {
    out.push_str(key);
    out.push_str(" = \"");
    out.push_str(&toml_escape(value));
    out.push_str("\"\n");
}

fn toml_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn parse_usize_toml_field(source: &str, key: &str) -> Option<usize> {
    parse_toml_field_value(source, key)?.parse().ok()
}

fn parse_string_toml_field(source: &str, key: &str) -> Option<String> {
    let value = parse_toml_field_value(source, key)?;
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .map(unescape_basic_toml_string)
}

fn parse_toml_field_value<'a>(source: &'a str, key: &str) -> Option<&'a str> {
    let prefix = format!("{key} = ");
    source
        .lines()
        .find_map(|line| line.trim().strip_prefix(&prefix).map(str::trim))
}

fn unescape_basic_toml_string(value: &str) -> String {
    let mut out = String::new();
    let mut escaped = false;
    for ch in value.chars() {
        if escaped {
            out.push(match ch {
                '"' => '"',
                '\\' => '\\',
                'n' => '\n',
                't' => '\t',
                other => other,
            });
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else {
            out.push(ch);
        }
    }
    out
}

fn fnv1a64_hex(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("0x{hash:016x}")
}

fn digest_hex(contract: &str, bytes: &[u8]) -> Option<String> {
    match contract {
        PROVIDER_COMPLETION_DIGEST_FNV1A64_CONTRACT => Some(fnv1a64_hex(bytes)),
        PROVIDER_COMPLETION_DIGEST_SHA256_CONTRACT
        | PROVIDER_COMPLETION_DIGEST_SHA256_AUTHORITY_CONTRACT => {
            Some(crate::digest_sha256::sha256_hex(bytes))
        }
        _ => None,
    }
}
