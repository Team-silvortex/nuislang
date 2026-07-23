#![allow(dead_code)]

mod digest_sha256;
mod handoff;
mod model;
mod provider_adapter_binding;
mod provider_carrier_channel;
mod provider_carrier_channel_registry;
#[cfg(unix)]
mod provider_carrier_channel_unix;
mod provider_carrier_input;
mod provider_completion_integrity;
mod provider_completion_signature;
mod provider_completion_trust_anchor;
mod provider_completion_trust_registry;
mod provider_edge_staging_registry;
mod provider_edge_transport;
mod provider_execution_capsule;
mod provider_input_binding;
mod provider_native_output_payload;
mod provider_output_carrier_registry;
#[cfg(unix)]
mod provider_output_carrier_unix;
mod provider_output_comparison;
mod provider_prepared_input;
mod provider_process_adapter;
mod provider_request;
mod provider_request_payload;
mod provider_runner_coreml;
mod provider_runner_metal;
mod provider_runner_registry;
mod provider_sample;
mod provider_sample_artifact;
mod provider_sample_execute;
mod provider_sample_execution;
mod provider_sample_materialize;
#[cfg(test)]
mod provider_sample_materialize_tests;
mod provider_sample_payload;
#[cfg(test)]
mod provider_sample_payload_tests;
mod provider_sample_runner;
mod provider_session_registry;
mod provider_transport_receipt_payload;
mod provider_worker_image;
mod provider_worker_ingress;
#[cfg(unix)]
mod provider_worker_lease;
mod provider_worker_request;
#[cfg(unix)]
mod provider_worker_result;
#[cfg(unix)]
mod provider_worker_summary;
mod provider_worker_transport;
#[cfg(unix)]
mod provider_worker_transport_unix;

pub use model::{
    PayloadExecutionHandoffPersistSummary, PayloadExecutionHandoffRecord,
    PayloadExecutionProviderCompletion,
};
pub use provider_sample_execute::{execute_provider_samples, ProviderSampleExecuteReport};
pub use provider_sample_materialize::{
    materialize_provider_samples, ProviderSampleMaterializeReport,
};

pub fn persist_payload_execution_handoff_record(
    output_dir: &std::path::Path,
    source: &str,
    record: PayloadExecutionHandoffRecord,
) -> Result<PayloadExecutionHandoffPersistSummary, String> {
    handoff::persist_payload_execution_handoff_record(output_dir, source, record)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayloadExecutionReplaySummary {
    pub contract: &'static str,
    pub status: String,
    pub checkpoint_count: usize,
    pub replayable_checkpoint_count: usize,
    pub provider_completion_count: usize,
    pub first_provider_family: Option<String>,
    pub first_provider_output_contract: Option<String>,
    pub first_provider_output_evidence: Option<String>,
    pub provider_completion_claim_authority_contract: Option<String>,
    pub provider_completion_claim_authority: Option<String>,
    pub provider_completion_claim_authority_status: String,
    pub provider_completion_signature_contract: Option<String>,
    pub provider_completion_signature_public_key_id: Option<String>,
    pub provider_completion_signature_status: String,
    pub provider_completion_digest_contract: Option<String>,
    pub provider_completion_set_hash_claim: Option<String>,
    pub provider_completion_set_hash: Option<String>,
    pub provider_completion_set_hash_validation_status: String,
    pub provider_completions: Vec<PayloadExecutionProviderCompletion>,
    pub first_blocker: Option<String>,
}

pub fn payload_execution_replay_summary(
    output_dir: &std::path::Path,
) -> PayloadExecutionReplaySummary {
    let handoff = handoff::read_payload_execution_handoff(output_dir);
    let checkpoint_count = handoff.events.len();
    let replayable_checkpoint_count = handoff
        .events
        .iter()
        .filter(|event| event.status == "ready")
        .count();
    let provider_completions = handoff
        .events
        .iter()
        .filter(|event| event.execution_phase == "provider-device-completion")
        .map(|event| PayloadExecutionProviderCompletion {
            trace_id: event.trace_id.clone(),
            provider_family: event.provider_family.clone(),
            output_contract: event.output_contract.clone(),
            output_evidence: event.output_evidence.clone(),
            record_hash: provider_completion_integrity::record_hash(
                event,
                if handoff.provider_completion_digest_contract == "none" {
                    "nuis-provider-completion-digest-fnv1a64-v1"
                } else {
                    &handoff.provider_completion_digest_contract
                },
            )
            .unwrap_or_else(|| "none".to_owned()),
        })
        .collect::<Vec<_>>();
    let first_provider_completion = provider_completions.first();
    let provider_completion_set_hash = (handoff.provider_completion_set_hash_actual != "none")
        .then(|| handoff.provider_completion_set_hash_actual.clone());
    let first_blocker = if !handoff.available {
        Some("payload-execution-handoff-missing".to_owned())
    } else if handoff.status != "ready" {
        Some(format!("payload-execution-handoff:{}", handoff.status))
    } else if handoff.hetero_execution_closure_status != "none"
        && (handoff.hetero_execution_closure_status != "closed"
            || handoff.hetero_execution_closure_ready != "true")
    {
        Some(
            if handoff.hetero_execution_closure_first_blocker != "none" {
                format!(
                    "hetero-execution-closure:{}",
                    handoff.hetero_execution_closure_first_blocker
                )
            } else {
                format!(
                    "hetero-execution-closure:{}",
                    handoff.hetero_execution_closure_status
                )
            },
        )
    } else if checkpoint_count == 0 {
        Some("payload-execution-replay:no-checkpoints".to_owned())
    } else if replayable_checkpoint_count != checkpoint_count {
        Some("payload-execution-replay:blocked-checkpoint".to_owned())
    } else {
        None
    };
    PayloadExecutionReplaySummary {
        contract: "nsdb-payload-execution-replay-plan-v1",
        status: if first_blocker.is_none() {
            "replay-evidence-ready".to_owned()
        } else {
            "blocked".to_owned()
        },
        checkpoint_count,
        replayable_checkpoint_count,
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
        provider_completion_claim_authority_contract: (handoff
            .provider_completion_claim_authority_contract
            != "none")
            .then(|| handoff.provider_completion_claim_authority_contract.clone()),
        provider_completion_claim_authority: (handoff.provider_completion_claim_authority
            != "none")
            .then(|| handoff.provider_completion_claim_authority.clone()),
        provider_completion_claim_authority_status: handoff
            .provider_completion_claim_authority_status,
        provider_completion_signature_contract: (handoff.provider_completion_signature_contract
            != "none")
            .then(|| handoff.provider_completion_signature_contract.clone()),
        provider_completion_signature_public_key_id: (handoff
            .provider_completion_signature_public_key_id
            != "none")
            .then(|| handoff.provider_completion_signature_public_key_id.clone()),
        provider_completion_signature_status: handoff.provider_completion_signature_status,
        provider_completion_digest_contract: (handoff.provider_completion_digest_contract
            != "none")
            .then(|| handoff.provider_completion_digest_contract.clone()),
        provider_completion_set_hash_claim: (handoff.provider_completion_set_hash_claim != "none")
            .then(|| handoff.provider_completion_set_hash_claim.clone()),
        provider_completion_set_hash,
        provider_completion_set_hash_validation_status: handoff
            .provider_completion_set_hash_validation_status,
        provider_completions,
        first_blocker,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        payload_execution_replay_summary, persist_payload_execution_handoff_record,
        PayloadExecutionHandoffRecord,
    };
    use std::{fs, path::Path};

    #[test]
    fn payload_execution_replay_summary_consumes_ready_handoff() {
        let dir =
            std::env::temp_dir().join(format!("nsdb-lib-replay-summary-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("nuis.nsdb.payload-execution-handoff.toml"),
            r#"
protocol = "nuis-nsdb-payload-execution-handoff-v1"
debugger_contract = "nsdb-yir-payload-execution-trace-v1"
source = "nsld-final-executable-output"
record_count = 1
ready_record_count = 1
first_trace_id = "payload-trace:container-loader:main"
first_status = "ready"
first_next_action = "handoff-payload-trace-to-nsdb"

[[records]]
trace_id = "payload-trace:container-loader:main"
status = "ready"
execution_phase = "container-loader-handoff"
target = "container-loader"
entry_symbol = "main"
entry_kind = "lifecycle-bootstrap"
entry_section_id = "sec0000.compiled-artifact"
first_blocker = ""
next_action = "handoff-payload-trace-to-nsdb"
"#,
        )
        .unwrap();

        let summary = payload_execution_replay_summary(Path::new(&dir));
        fs::remove_dir_all(dir).unwrap();

        assert_eq!(summary.contract, "nsdb-payload-execution-replay-plan-v1");
        assert_eq!(summary.status, "replay-evidence-ready");
        assert_eq!(summary.checkpoint_count, 1);
        assert_eq!(summary.replayable_checkpoint_count, 1);
        assert_eq!(summary.first_blocker, None);
    }

    #[test]
    fn payload_execution_replay_summary_blocks_pending_hetero_closure() {
        let dir = std::env::temp_dir().join(format!(
            "nsdb-lib-replay-summary-closure-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("nuis.nsdb.payload-execution-handoff.toml"),
            r#"
protocol = "nuis-nsdb-payload-execution-handoff-v1"
debugger_contract = "nsdb-yir-payload-execution-trace-v1"
record_count = 1
ready_record_count = 1
hetero_execution_closure_protocol = "nuis-hetero-execution-closure-v1"
hetero_execution_closure_status = "host-runner-pending"
hetero_execution_closure_ready = "false"
hetero_execution_closure_first_blocker = "host-runner-backend-artifact-payload:not-observed"
hetero_execution_closure_next_action = "run-host-runner-payload-probe"

[[records]]
trace_id = "payload-trace:container-loader:main"
status = "ready"
execution_phase = "container-loader-handoff"
entry_symbol = "main"
next_action = "handoff-payload-trace-to-nsdb"
"#,
        )
        .unwrap();

        let summary = payload_execution_replay_summary(Path::new(&dir));
        fs::remove_dir_all(dir).unwrap();

        assert_eq!(summary.status, "blocked");
        assert_eq!(summary.checkpoint_count, 1);
        assert_eq!(summary.replayable_checkpoint_count, 1);
        assert_eq!(
            summary.first_blocker.as_deref(),
            Some("hetero-execution-closure:host-runner-backend-artifact-payload:not-observed")
        );
    }

    #[test]
    fn provider_completion_collection_preserves_order_and_hashes_records() {
        let dir = std::env::temp_dir().join(format!(
            "nsdb-provider-completion-set-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        for (trace_id, family, evidence) in [
            (
                "hetero-trace:shader:metal:apple-silicon-gpu",
                "metal:apple-silicon-gpu",
                "metal-output:hash=0x1234",
            ),
            (
                "hetero-trace:kernel:coreml:apple-ane",
                "coreml:apple-ane",
                "coreml-output:hash=0x5678",
            ),
        ] {
            persist_payload_execution_handoff_record(
                &dir,
                "provider-set-test",
                PayloadExecutionHandoffRecord {
                    trace_id: trace_id.to_owned(),
                    status: "ready".to_owned(),
                    execution_phase: "provider-device-completion".to_owned(),
                    target: family.to_owned(),
                    entry_symbol: "registered-provider".to_owned(),
                    entry_kind: "nuis-provider-output-payload-handoff-v1".to_owned(),
                    entry_section_id: evidence.to_owned(),
                    provider_family: family.to_owned(),
                    output_contract: "nuis-provider-output-payload-handoff-v1".to_owned(),
                    output_evidence: evidence.to_owned(),
                    first_blocker: String::new(),
                    next_action: "replay-provider-completion".to_owned(),
                },
            )
            .unwrap();
        }

        let summary = payload_execution_replay_summary(&dir);

        assert_eq!(summary.provider_completion_count, 2);
        assert_eq!(summary.provider_completions.len(), 2);
        assert_eq!(
            summary.provider_completions[0].provider_family,
            "metal:apple-silicon-gpu"
        );
        assert_eq!(
            summary.provider_completions[1].provider_family,
            "coreml:apple-ane"
        );
        assert_ne!(
            summary.provider_completions[0].record_hash,
            summary.provider_completions[1].record_hash
        );
        assert!(summary
            .provider_completion_set_hash
            .as_deref()
            .is_some_and(
                |hash| hash.len() == 64 && hash.bytes().all(|byte| byte.is_ascii_hexdigit())
            ));
        assert_eq!(
            summary.provider_completion_digest_contract.as_deref(),
            Some("nuis-provider-completion-digest-sha256-authority-v1")
        );
        assert_eq!(
            summary.provider_completion_claim_authority.as_deref(),
            Some("nsdb:payload-execution-handoff-writer:v1")
        );
        assert_eq!(
            summary.provider_completion_claim_authority_status,
            "authorized"
        );
        let path = dir.join("nuis.nsdb.payload-execution-handoff.toml");
        let source = fs::read_to_string(&path).unwrap();
        assert!(source.contains(
            "provider_completion_digest_contract = \"nuis-provider-completion-digest-sha256-authority-v1\""
        ));
        let claim = summary.provider_completion_set_hash.as_deref().unwrap();
        assert!(source.contains(&format!("provider_completion_set_hash = \"{claim}\"")));
        fs::write(
            &path,
            source.replacen("record_count = 2", "record_count = 3", 1),
        )
        .unwrap();
        let count_rejected = payload_execution_replay_summary(&dir);
        assert_eq!(
            count_rejected.provider_completion_set_hash_validation_status,
            "mismatch"
        );
        assert_eq!(
            count_rejected.first_blocker.as_deref(),
            Some("payload-execution-handoff:provider-completion-set-hash-mismatch")
        );
        fs::write(
            &path,
            source.replace("coreml-output:hash=0x5678", "coreml-output:hash=0xtampered"),
        )
        .unwrap();
        let rejected = payload_execution_replay_summary(&dir);
        let rewrite = persist_payload_execution_handoff_record(
            &dir,
            "provider-set-test",
            PayloadExecutionHandoffRecord {
                trace_id: "hetero-trace:kernel:coreml:apple-ane".to_owned(),
                status: "ready".to_owned(),
                execution_phase: "provider-device-completion".to_owned(),
                target: "coreml:apple-ane".to_owned(),
                entry_symbol: "registered-provider".to_owned(),
                entry_kind: "nuis-provider-output-payload-handoff-v1".to_owned(),
                entry_section_id: "coreml-output:hash=0x5678".to_owned(),
                provider_family: "coreml:apple-ane".to_owned(),
                output_contract: "nuis-provider-output-payload-handoff-v1".to_owned(),
                output_evidence: "coreml-output:hash=0x5678".to_owned(),
                first_blocker: String::new(),
                next_action: "replay-provider-completion".to_owned(),
            },
        );
        fs::remove_dir_all(dir).unwrap();

        assert_eq!(rejected.status, "blocked");
        assert_eq!(
            rejected.first_blocker.as_deref(),
            Some("payload-execution-handoff:provider-completion-set-hash-mismatch")
        );
        assert_eq!(
            rewrite.unwrap_err(),
            "provider completion digest validation failed in existing handoff: mismatch"
        );
    }
}
