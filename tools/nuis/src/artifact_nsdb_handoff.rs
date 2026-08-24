use crate::{
    artifact_launch_evidence::RunArtifactLaunchEvidence,
    artifact_nsdb_handoff_binding::{
        independently_verify as verify_final_image_binding_proof, PersistedFinalImageBindingProof,
    },
    artifact_nsdb_handoff_dispatch::{
        dispatch_identity, parse_provider_completions, PersistedProviderCompletion,
        PersistedProviderDispatchIdentity,
    },
    artifact_nsdb_handoff_integrity::{
        legacy_set_hash, set_hash, signature_message,
        CLAIM_AUTHORITY as PROVIDER_COMPLETION_CLAIM_AUTHORITY,
        CLAIM_AUTHORITY_CONTRACT as PROVIDER_COMPLETION_CLAIM_AUTHORITY_CONTRACT,
        DIGEST_FNV1A64_CONTRACT as PROVIDER_COMPLETION_DIGEST_FNV1A64_CONTRACT,
        DIGEST_SHA256_AUTHORITY_CONTRACT as PROVIDER_COMPLETION_DIGEST_SHA256_AUTHORITY_CONTRACT,
        DIGEST_SHA256_CONTRACT as PROVIDER_COMPLETION_DIGEST_SHA256_CONTRACT,
        DIGEST_SHA256_SIGNED_CONTRACT as PROVIDER_COMPLETION_DIGEST_SHA256_SIGNED_CONTRACT,
    },
    artifact_nsdb_handoff_render::render_launch_evidence_nsdb_handoff,
    artifact_nsdb_handoff_signature::{
        parse_and_verify as parse_and_verify_provider_completion_signature,
        validation_error as provider_completion_signature_error,
    },
    artifact_runtime_dispatch_receipt::{
        independently_verify as verify_runtime_dispatch_receipt, upsert_claim,
        PersistedRuntimeDispatchReceipt,
    },
    json_bool_field, json_field, json_optional_string_field, json_usize_field,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

const NSDB_HANDOFF_PROTOCOL: &str = "nuis-nsdb-payload-execution-handoff-v1";
const NSDB_HANDOFF_FILE_NAME: &str = "nuis.nsdb.payload-execution-handoff.toml";

pub(crate) struct LaunchEvidenceNsdbHandoffPersistence {
    persisted: bool,
    path: Option<PathBuf>,
    record_count: usize,
    ready_record_count: usize,
    first_trace_id: Option<String>,
    error: Option<String>,
}

#[cfg(test)]
impl LaunchEvidenceNsdbHandoffPersistence {
    pub(crate) fn persisted(&self) -> bool {
        self.persisted
    }
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
    final_image_binding_proof: PersistedFinalImageBindingProof,
    runtime_dispatch_receipt: PersistedRuntimeDispatchReceipt,
    provider_completion_count: usize,
    first_provider_family: Option<String>,
    first_provider_output_contract: Option<String>,
    first_provider_output_evidence: Option<String>,
    provider_completion_claim_authority_contract: Option<String>,
    provider_completion_claim_authority: Option<String>,
    provider_completion_claim_authority_status: String,
    provider_completion_signature_contract: Option<String>,
    provider_completion_signature_public_key_id: Option<String>,
    provider_completion_signature_status: String,
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

    pub(crate) fn final_image_binding_proof_status(&self) -> &str {
        &self.final_image_binding_proof.verification_status
    }

    pub(crate) fn final_image_binding_proof_hash(&self) -> Option<&str> {
        self.final_image_binding_proof.proof_hash.as_deref()
    }

    pub(crate) fn runtime_dispatch_receipt(&self) -> &PersistedRuntimeDispatchReceipt {
        &self.runtime_dispatch_receipt
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

    pub(crate) fn signature_summary(&self) -> (Option<&str>, Option<&str>, &str) {
        (
            self.provider_completion_signature_contract.as_deref(),
            self.provider_completion_signature_public_key_id.as_deref(),
            &self.provider_completion_signature_status,
        )
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

    pub(crate) fn provider_dispatch_identity(&self) -> PersistedProviderDispatchIdentity {
        dispatch_identity(
            &self.provider_completions,
            self.final_image_binding_proof_status(),
        )
    }

    pub(crate) fn hetero_execution_closure_ready(&self) -> bool {
        matches!(
            (
                self.hetero_execution_closure_status.as_deref(),
                self.hetero_execution_closure_ready.as_deref(),
            ),
            (None, _) | (Some("none"), _) | (Some("closed"), Some("true"))
        )
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
        json::fields(self, prefix)
    }
}

#[path = "artifact_nsdb_handoff_json.rs"]
mod json;

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
            final_image_binding_proof: verify_final_image_binding_proof(""),
            runtime_dispatch_receipt: verify_runtime_dispatch_receipt(""),
            provider_completion_count: 0,
            first_provider_family: None,
            first_provider_output_contract: None,
            first_provider_output_evidence: None,
            provider_completion_claim_authority_contract: None,
            provider_completion_claim_authority: None,
            provider_completion_claim_authority_status: "not-applicable".to_owned(),
            provider_completion_signature_contract: None,
            provider_completion_signature_public_key_id: None,
            provider_completion_signature_status: "not-applicable".to_owned(),
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
            final_image_binding_proof: verify_final_image_binding_proof(""),
            runtime_dispatch_receipt: verify_runtime_dispatch_receipt(""),
            provider_completion_count: 0,
            first_provider_family: None,
            first_provider_output_contract: None,
            first_provider_output_evidence: None,
            provider_completion_claim_authority_contract: None,
            provider_completion_claim_authority: None,
            provider_completion_claim_authority_status: "not-applicable".to_owned(),
            provider_completion_signature_contract: None,
            provider_completion_signature_public_key_id: None,
            provider_completion_signature_status: "not-applicable".to_owned(),
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
    let final_image_binding_proof = verify_final_image_binding_proof(&source);
    let runtime_dispatch_receipt = verify_runtime_dispatch_receipt(&source);
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
    let provider_completions = parse_provider_completions(&source, record_digest_contract);
    let first_provider_completion = provider_completions.first();
    let record_hashes = provider_completions
        .iter()
        .map(|completion| completion.record_hash.as_str())
        .collect::<Vec<_>>();
    let provider_completion_set_hash = match provider_completion_digest_contract.as_deref() {
        None => legacy_set_hash(&record_hashes),
        Some(
            contract @ (PROVIDER_COMPLETION_DIGEST_FNV1A64_CONTRACT
            | PROVIDER_COMPLETION_DIGEST_SHA256_CONTRACT
            | PROVIDER_COMPLETION_DIGEST_SHA256_AUTHORITY_CONTRACT
            | PROVIDER_COMPLETION_DIGEST_SHA256_SIGNED_CONTRACT),
        ) => set_hash(
            &record_hashes,
            protocol.as_deref().unwrap_or("none"),
            record_count,
            contract,
            provider_completion_claim_authority_contract.as_deref(),
            provider_completion_claim_authority.as_deref(),
        ),
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
        && provider_completion_digest_contract.as_deref()
            != Some(PROVIDER_COMPLETION_DIGEST_SHA256_SIGNED_CONTRACT)
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
        && provider_completion_digest_contract.as_deref()
            != Some(PROVIDER_COMPLETION_DIGEST_SHA256_SIGNED_CONTRACT)
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
    let signature_message = signature_message(
        protocol.as_deref().unwrap_or("none"),
        provider_completion_digest_contract
            .as_deref()
            .unwrap_or("none"),
        provider_completion_claim_authority_contract
            .as_deref()
            .unwrap_or("none"),
        provider_completion_claim_authority
            .as_deref()
            .unwrap_or("none"),
        provider_completion_set_hash.as_deref().unwrap_or("none"),
    );
    let signature = parse_and_verify_provider_completion_signature(
        &source,
        !provider_completions.is_empty(),
        provider_completion_digest_contract.as_deref()
            == Some(PROVIDER_COMPLETION_DIGEST_SHA256_SIGNED_CONTRACT),
        &signature_message,
    );
    let provider_completion_signature_status = signature.status;
    let provider_dispatch_identity = dispatch_identity(
        &provider_completions,
        &final_image_binding_proof.verification_status,
    );
    let error = if !matches!(
        runtime_dispatch_receipt.verification_status.as_str(),
        "verified" | "legacy-absent"
    ) {
        Some(format!(
            "runtime-dispatch-receipt-{}",
            runtime_dispatch_receipt.verification_status
        ))
    } else {
        match (
            final_image_binding_proof.verification_status.as_str(),
            provider_completion_set_hash_validation_status.as_str(),
            provider_completion_claim_authority_status.as_str(),
            provider_completion_signature_status.as_str(),
            provider_dispatch_identity.status.as_str(),
        ) {
            (status, _, _, _, _)
                if !matches!(status, "verified" | "verified-empty" | "legacy-unbound") =>
            {
                Some(format!("final-image-binding-proof-{status}"))
            }
            (_, "mismatch", _, _, _) => Some("provider-completion-set-hash-mismatch".to_owned()),
            (_, "unsupported-digest-contract", _, _, _) => {
                Some("provider-completion-digest-contract-unsupported".to_owned())
            }
            (_, _, "authority-missing", _, _) => {
                Some("provider-completion-claim-authority-missing".to_owned())
            }
            (_, _, "unsupported-authority-contract", _, _) => {
                Some("provider-completion-claim-authority-contract-unsupported".to_owned())
            }
            (_, _, "authority-untrusted", _, _) => {
                Some("provider-completion-claim-authority-untrusted".to_owned())
            }
            (_, _, _, _, status @ ("mismatch" | "final-image-authority-missing")) => {
                Some(format!("provider-completion-dispatch-{status}"))
            }
            _ => provider_completion_signature_error(&provider_completion_signature_status),
        }
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
        final_image_binding_proof,
        runtime_dispatch_receipt,
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
        provider_completion_signature_contract: signature.contract,
        provider_completion_signature_public_key_id: signature.public_key_id,
        provider_completion_signature_status,
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
    let existing_handoff = read_persisted_nsdb_handoff(Some(output_dir));
    if existing_handoff.provider_completion_count > 0 && existing_handoff.error.is_none() {
        if let (Some(receipt), Ok(existing)) = (
            evidence.runtime_dispatch_receipt(),
            fs::read_to_string(&path),
        ) {
            let content = upsert_claim(&existing, receipt);
            if let Err(error) = fs::write(&path, content) {
                return LaunchEvidenceNsdbHandoffPersistence {
                    persisted: false,
                    path: Some(path),
                    record_count: existing_handoff.record_count,
                    ready_record_count: existing_handoff.ready_record_count,
                    first_trace_id: existing_handoff.first_trace_id,
                    error: Some(error.to_string()),
                };
            }
        }
        return LaunchEvidenceNsdbHandoffPersistence {
            persisted: true,
            path: Some(path),
            record_count: existing_handoff.record_count,
            ready_record_count: existing_handoff.ready_record_count,
            first_trace_id: existing_handoff.first_trace_id,
            error: None,
        };
    }
    let existing = fs::read_to_string(&path).ok();
    let existing_proof = existing
        .as_deref()
        .map(verify_final_image_binding_proof)
        .filter(|proof| {
            matches!(
                proof.verification_status.as_str(),
                "verified" | "verified-empty"
            )
        });
    let content = render_launch_evidence_nsdb_handoff(evidence, existing_proof.as_ref());
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
