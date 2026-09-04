#![allow(dead_code)]

mod digest_sha256;
mod final_image_provider_dispatch;
mod handoff;
mod handoff_binding;
mod handoff_status;
mod model;
mod provider_adapter_binding;
mod provider_bundle_registry;
mod provider_capability_registry;
mod provider_carrier_channel;
mod provider_carrier_channel_registry;
#[cfg(unix)]
mod provider_carrier_channel_unix;
mod provider_carrier_input;
mod provider_code_asset;
mod provider_code_asset_identity;
mod provider_completion_dispatch;
mod provider_completion_evidence;
mod provider_completion_integrity;
mod provider_completion_projection;
mod provider_completion_signature;
mod provider_completion_trust_anchor;
mod provider_completion_trust_registry;
mod provider_conformance_capsule;
#[cfg(test)]
mod provider_conformance_capsule_tests;
mod provider_edge_staging_registry;
mod provider_edge_transport;
#[cfg(unix)]
mod provider_execution_adapter;
mod provider_execution_capsule;
#[cfg(unix)]
mod provider_execution_coreml;
#[cfg(unix)]
mod provider_execution_cuda;
#[cfg(unix)]
mod provider_execution_metal;
#[cfg(unix)]
mod provider_execution_native;
#[cfg(unix)]
mod provider_execution_vulkan;
#[cfg(unix)]
mod provider_execution_vulkan_spirv;
mod provider_graph_output;
mod provider_input_binding;
mod provider_native_output_payload;
mod provider_output_binding;
mod provider_output_carrier_registry;
#[cfg(unix)]
mod provider_output_carrier_unix;
mod provider_output_comparison;
mod provider_output_comparison_descriptor;
mod provider_prepared_input;
mod provider_process_adapter;
mod provider_request;
mod provider_request_completion;
mod provider_request_payload;
mod provider_result_projection;
mod provider_runner_coreml;
mod provider_runner_cuda;
mod provider_runner_metal;
mod provider_runner_metal_u32;
mod provider_runner_native;
mod provider_runner_registry;
mod provider_runner_vulkan;
mod provider_sample;
mod provider_sample_artifact;
mod provider_sample_execute;
#[cfg(test)]
mod provider_sample_execute_tests;
mod provider_sample_execution;
mod provider_sample_materialize;
#[cfg(test)]
mod provider_sample_materialize_tests;
mod provider_sample_output_model;
mod provider_sample_payload;
#[cfg(test)]
mod provider_sample_payload_tests;
mod provider_sample_runner;
mod provider_session_registry;
mod provider_session_summary;
mod provider_transport_receipt_payload;
#[cfg(unix)]
mod provider_worker_control;
mod provider_worker_descriptor_capability;
mod provider_worker_image;
mod provider_worker_ingress;
#[cfg(unix)]
mod provider_worker_lease;
#[cfg(unix)]
mod provider_worker_native_execution;
mod provider_worker_request;
#[cfg(unix)]
mod provider_worker_result;
#[cfg(unix)]
mod provider_worker_summary;
mod provider_worker_transport;
#[cfg(unix)]
mod provider_worker_transport_unix;
mod runtime_dispatch_receipt;

pub use model::{
    CompiledCodeAssetSelectionEvidence, CompiledCodeAssetSelectionItem,
    FinalImageBindingProofClaim, PayloadExecutionHandoffPersistSummary,
    PayloadExecutionHandoffRecord, PayloadExecutionProviderCompletion,
};
pub use provider_conformance_capsule::{
    data_reference_conformance_capsule, replay_provider_conformance_capsule,
    ProviderConformanceCapsule, ProviderConformanceLifecycleEvidence,
    ProviderConformanceObservation, ProviderConformanceReplayEvidence,
    PROVIDER_CONFORMANCE_CAPSULE_CONTRACT, PROVIDER_CONFORMANCE_EXECUTION_AUTHORITY,
    PROVIDER_CONFORMANCE_REPLAY_CONTRACT,
};
pub use provider_sample_execute::{execute_provider_samples, ProviderSampleExecuteReport};
pub use provider_sample_materialize::{
    materialize_provider_samples, ProviderSampleMaterializeReport,
};
pub use runtime_dispatch_receipt::RuntimeDispatchReceiptSummary;

pub fn validate_provider_request_evidence(input_evidence: &str) -> bool {
    provider_request::provider_request_collection_from_evidence(input_evidence).is_some()
}

pub fn persist_payload_execution_handoff_record(
    output_dir: &std::path::Path,
    source: &str,
    record: PayloadExecutionHandoffRecord,
) -> Result<PayloadExecutionHandoffPersistSummary, String> {
    handoff::persist_payload_execution_handoff_record(output_dir, source, record)
}

pub fn persist_payload_execution_handoff_record_with_final_image_binding(
    output_dir: &std::path::Path,
    source: &str,
    record: PayloadExecutionHandoffRecord,
    binding: FinalImageBindingProofClaim,
) -> Result<PayloadExecutionHandoffPersistSummary, String> {
    handoff::persist_payload_execution_handoff_record_with_final_image_binding(
        output_dir, source, record, binding,
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayloadExecutionReplaySummary {
    pub contract: &'static str,
    pub status: String,
    pub next_action: String,
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
    pub provider_completion_dispatch_authority_contract: String,
    pub provider_completion_dispatch_authority_status: String,
    pub provider_completion_dispatch_table_hash: Option<String>,
    pub provider_completion_dispatch_selected_set_hash: Option<String>,
    pub provider_completion_dispatch_identity_hash: Option<String>,
    pub provider_completions: Vec<PayloadExecutionProviderCompletion>,
    pub runtime_bootstrap_identity_contract: Option<String>,
    pub runtime_bootstrap_identity_status: String,
    pub runtime_bootstrap_identity_hash: Option<String>,
    pub runtime_dispatch_receipt: RuntimeDispatchReceiptSummary,
    pub final_image_binding_proof_contract: Option<String>,
    pub final_image_binding_proof_status: String,
    pub final_image_binding_proof_hash: Option<String>,
    pub final_image_binding_proof_next_action: String,
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
        .map(|event| {
            provider_completion_projection::public_completion(
                event,
                if handoff.provider_completion_digest_contract == "none" {
                    "nuis-provider-completion-digest-fnv1a64-v1"
                } else {
                    &handoff.provider_completion_digest_contract
                },
            )
        })
        .collect::<Vec<_>>();
    let first_provider_completion = provider_completions.first();
    let bootstrap_event = handoff
        .events
        .iter()
        .find(|event| event.execution_phase == "container-loader-handoff");
    let bootstrap_contract = bootstrap_event.map(|event| event.output_contract.as_str());
    let bootstrap_hash = bootstrap_event.map(|event| event.output_evidence.as_str());
    let runtime_bootstrap_identity_status = match (bootstrap_contract, bootstrap_hash) {
        (Some(nuis_runtime_contract), Some(hash))
            if nuis_runtime_contract == "nuis-runtime-lifecycle-bootstrap-plan-identity-v1"
                && valid_runtime_bootstrap_hash(hash) =>
        {
            "verified"
        }
        (None, None) | (Some("" | "none"), Some("" | "none")) => "legacy-unbound",
        _ => "invalid",
    }
    .to_owned();
    let provider_completion_set_hash = (handoff.provider_completion_set_hash_actual != "none")
        .then(|| handoff.provider_completion_set_hash_actual.clone());
    let first_blocker = if !handoff.available {
        Some("payload-execution-handoff-missing".to_owned())
    } else if handoff.status != "ready" {
        Some(format!("payload-execution-handoff:{}", handoff.status))
    } else if handoff.final_image_binding_proof.proof_status == "legacy-unbound" {
        Some("final-image-binding-proof:legacy-unbound".to_owned())
    } else if runtime_bootstrap_identity_status == "invalid" {
        Some("runtime-bootstrap-identity:invalid".to_owned())
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
    let replay_ready = first_blocker.is_none();
    let final_image_binding_proof_next_action =
        handoff_binding::next_action(&handoff.final_image_binding_proof.proof_status).to_owned();
    PayloadExecutionReplaySummary {
        contract: "nsdb-payload-execution-replay-plan-v1",
        status: if replay_ready {
            "replay-evidence-ready".to_owned()
        } else {
            "blocked".to_owned()
        },
        next_action: if replay_ready {
            "replay-nsdb-payload-execution"
        } else if handoff.available
            && handoff.final_image_binding_proof.proof_status == "legacy-unbound"
        {
            "rebuild-final-output-binding-proof"
        } else {
            "resolve-payload-execution-replay"
        }
        .to_owned(),
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
        provider_completion_dispatch_authority_contract: handoff
            .provider_completion_dispatch_identity
            .contract
            .clone(),
        provider_completion_dispatch_authority_status: handoff
            .provider_completion_dispatch_identity
            .status
            .clone(),
        provider_completion_dispatch_table_hash: optional_identity_field(
            &handoff.provider_completion_dispatch_identity.table_hash,
        ),
        provider_completion_dispatch_selected_set_hash: optional_identity_field(
            &handoff
                .provider_completion_dispatch_identity
                .selected_set_hash,
        ),
        provider_completion_dispatch_identity_hash: optional_identity_field(
            &handoff.provider_completion_dispatch_identity.identity_hash,
        ),
        provider_completions,
        runtime_bootstrap_identity_contract: bootstrap_contract
            .filter(|value| !value.is_empty() && *value != "none")
            .map(str::to_owned),
        runtime_bootstrap_identity_status,
        runtime_bootstrap_identity_hash: bootstrap_hash
            .filter(|value| valid_runtime_bootstrap_hash(value))
            .map(str::to_owned),
        runtime_dispatch_receipt: runtime_dispatch_receipt::public_summary(
            &handoff.runtime_dispatch_receipt,
        ),
        final_image_binding_proof_contract: (handoff.final_image_binding_proof.contract != "none")
            .then(|| handoff.final_image_binding_proof.contract.clone()),
        final_image_binding_proof_status: handoff.final_image_binding_proof.proof_status,
        final_image_binding_proof_hash: (handoff.final_image_binding_proof.proof_hash_actual
            != "none")
            .then(|| handoff.final_image_binding_proof.proof_hash_actual.clone()),
        final_image_binding_proof_next_action,
        first_blocker,
    }
}

fn optional_identity_field(value: &str) -> Option<String> {
    (value != "none").then(|| value.to_owned())
}

fn valid_runtime_bootstrap_hash(value: &str) -> bool {
    value
        .strip_prefix("0x")
        .is_some_and(|hex| hex.len() == 16 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

#[cfg(test)]
mod tests {
    use super::{
        payload_execution_replay_summary, persist_payload_execution_handoff_record,
        persist_payload_execution_handoff_record_with_final_image_binding,
        FinalImageBindingProofClaim, PayloadExecutionHandoffRecord,
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
final_image_binding_proof_contract = "nuis-final-image-binding-proof-v1"
final_image_metadata_binding_count = 0
final_image_metadata_binding_table_hash = "0xcbf29ce484222325"
final_image_metadata_binding_validation_status = "not-applicable"
final_image_selected_provider_bundle_set_contract = ""
final_image_selected_provider_bundle_count = 0
final_image_selected_provider_bundle_set_hash = ""
final_image_binding_proof_hash = "fnv1a64:981b10a68f4e3dd7"

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
        assert_eq!(summary.next_action, "replay-nsdb-payload-execution");
        assert_eq!(summary.final_image_binding_proof_next_action, "none");
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
final_image_binding_proof_contract = "nuis-final-image-binding-proof-v1"
final_image_metadata_binding_count = 0
final_image_metadata_binding_table_hash = "0xcbf29ce484222325"
final_image_metadata_binding_validation_status = "not-applicable"
final_image_selected_provider_bundle_set_contract = ""
final_image_selected_provider_bundle_count = 0
final_image_selected_provider_bundle_set_hash = ""
final_image_binding_proof_hash = "fnv1a64:981b10a68f4e3dd7"

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
    fn replay_rejects_tampered_final_image_binding_proof() {
        let dir = std::env::temp_dir().join(format!(
            "nsdb-final-image-binding-proof-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let table_hash = "0x1111111111111111";
        let selected_hash = "fnv1a64:2222222222222222";
        let proof_hash = crate::handoff_binding::proof_hash(
            1,
            table_hash,
            "verified",
            "nuis-selected-provider-bundle-set-v1",
            2,
            selected_hash,
        );
        let source = format!(
            r#"protocol = "nuis-nsdb-payload-execution-handoff-v1"
debugger_contract = "nsdb-yir-payload-execution-trace-v1"
record_count = 1
ready_record_count = 1
first_status = "ready"
final_image_binding_proof_contract = "nuis-final-image-binding-proof-v1"
final_image_metadata_binding_count = 1
final_image_metadata_binding_table_hash = "{table_hash}"
final_image_metadata_binding_validation_status = "verified"
final_image_selected_provider_bundle_set_contract = "nuis-selected-provider-bundle-set-v1"
final_image_selected_provider_bundle_count = 2
final_image_selected_provider_bundle_set_hash = "{selected_hash}"
final_image_binding_proof_hash = "{proof_hash}"

[[records]]
trace_id = "payload-trace:container-loader:main"
status = "ready"
execution_phase = "container-loader-handoff"
entry_symbol = "main"
output_contract = "nuis-runtime-lifecycle-bootstrap-plan-identity-v1"
output_evidence = "0x1234567890abcdef"
next_action = "handoff-payload-trace-to-nsdb"
"#
        );
        let path = dir.join("nuis.nsdb.payload-execution-handoff.toml");
        fs::write(&path, &source).unwrap();
        let verified = payload_execution_replay_summary(&dir);
        assert_eq!(verified.status, "replay-evidence-ready");
        assert_eq!(
            verified.final_image_binding_proof_contract.as_deref(),
            Some("nuis-final-image-binding-proof-v1")
        );
        assert_eq!(verified.final_image_binding_proof_status, "verified");
        assert_eq!(
            verified.runtime_bootstrap_identity_contract.as_deref(),
            Some("nuis-runtime-lifecycle-bootstrap-plan-identity-v1")
        );
        assert_eq!(verified.runtime_bootstrap_identity_status, "verified");
        assert_eq!(
            verified.runtime_bootstrap_identity_hash.as_deref(),
            Some("0x1234567890abcdef")
        );
        assert!(verified
            .final_image_binding_proof_hash
            .as_deref()
            .is_some_and(|hash| hash.starts_with("fnv1a64:")));

        fs::write(
            &path,
            source.replace("0x1234567890abcdef", "invalid-bootstrap-hash"),
        )
        .unwrap();
        let invalid_bootstrap = payload_execution_replay_summary(&dir);
        assert_eq!(invalid_bootstrap.status, "blocked");
        assert_eq!(
            invalid_bootstrap.first_blocker.as_deref(),
            Some("runtime-bootstrap-identity:invalid")
        );
        fs::write(&path, &source).unwrap();

        persist_payload_execution_handoff_record(
            &dir,
            "proof-preservation-test",
            PayloadExecutionHandoffRecord {
                trace_id: "payload-trace:container-loader:main".to_owned(),
                status: "ready".to_owned(),
                execution_phase: "container-loader-handoff".to_owned(),
                target: "container-loader".to_owned(),
                entry_symbol: "main".to_owned(),
                entry_kind: "lifecycle-bootstrap".to_owned(),
                entry_section_id: "sec0000.compiled-artifact".to_owned(),
                provider_family: String::new(),
                output_contract: String::new(),
                output_evidence: String::new(),
                first_blocker: String::new(),
                next_action: "handoff-payload-trace-to-nsdb".to_owned(),
            },
        )
        .unwrap();
        let preserved_source = fs::read_to_string(&path).unwrap();
        assert!(preserved_source.contains(
            "final_image_binding_proof_contract = \"nuis-final-image-binding-proof-v1\""
        ));
        assert_eq!(
            payload_execution_replay_summary(&dir).final_image_binding_proof_status,
            "verified"
        );

        fs::write(
            &path,
            preserved_source.replace(selected_hash, "fnv1a64:3333333333333333"),
        )
        .unwrap();
        let rejected = payload_execution_replay_summary(&dir);
        fs::remove_dir_all(dir).unwrap();
        assert_eq!(rejected.status, "blocked");
        assert_eq!(
            rejected.first_blocker.as_deref(),
            Some("payload-execution-handoff:final-image-binding-proof-mismatch")
        );
    }

    #[test]
    fn binding_claim_write_is_idempotent_and_rejects_final_image_conflict() {
        let dir = std::env::temp_dir().join(format!(
            "nsdb-final-image-binding-claim-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let record = PayloadExecutionHandoffRecord {
            trace_id: "payload-trace:container-loader:main".to_owned(),
            status: "ready".to_owned(),
            execution_phase: "container-loader-handoff".to_owned(),
            target: "container-loader".to_owned(),
            entry_symbol: "main".to_owned(),
            entry_kind: "lifecycle-bootstrap".to_owned(),
            entry_section_id: "sec0000.compiled-artifact".to_owned(),
            provider_family: String::new(),
            output_contract: String::new(),
            output_evidence: String::new(),
            first_blocker: String::new(),
            next_action: "handoff-payload-trace-to-nsdb".to_owned(),
        };
        let claim = FinalImageBindingProofClaim {
            binding_count: 1,
            binding_table_hash: "0x1111111111111111".to_owned(),
            validation_status: "verified".to_owned(),
            selected_set_contract: Some("nuis-selected-provider-bundle-set-v1".to_owned()),
            selected_set_count: Some(2),
            selected_set_hash: Some("fnv1a64:2222222222222222".to_owned()),
        };
        persist_payload_execution_handoff_record_with_final_image_binding(
            &dir,
            "claim-test",
            record.clone(),
            claim.clone(),
        )
        .unwrap();
        persist_payload_execution_handoff_record_with_final_image_binding(
            &dir,
            "claim-test",
            record.clone(),
            claim.clone(),
        )
        .unwrap();
        assert_eq!(
            payload_execution_replay_summary(&dir).final_image_binding_proof_status,
            "verified"
        );

        let mut conflicting = claim;
        conflicting.binding_table_hash = "0x3333333333333333".to_owned();
        let error = persist_payload_execution_handoff_record_with_final_image_binding(
            &dir,
            "claim-test",
            record,
            conflicting,
        )
        .unwrap_err();
        assert_eq!(
            error,
            "final image binding proof conflicts with existing handoff"
        );
        assert_eq!(
            payload_execution_replay_summary(&dir).final_image_binding_proof_status,
            "verified"
        );
        fs::remove_dir_all(dir).unwrap();
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
