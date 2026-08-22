use super::*;
use crate::final_executable_registered_loader_probe::{
    build_registered_loader_probe_outcome, RegisteredLoaderProbeEvidence,
};

#[test]
fn canonical_receipt_roundtrips_and_binds_payload_hash() {
    let receipt = fixture_receipt();
    let source = render_registered_loader_probe_admission_receipt(&receipt).unwrap();
    let parsed = parse_registered_loader_probe_admission_receipt(&source).unwrap();

    assert_eq!(parsed, receipt);
    assert_eq!(
        registered_loader_probe_admission_receipt_hash(&parsed).unwrap(),
        parsed.receipt_hash_sha256
    );

    let tampered = source.replacen("image_span_bytes = 64", "image_span_bytes = 65", 1);
    let parsed_tampered = parse_registered_loader_probe_admission_receipt(&tampered).unwrap();
    assert_ne!(
        registered_loader_probe_admission_receipt_hash(&parsed_tampered).unwrap(),
        parsed_tampered.receipt_hash_sha256
    );
}

#[test]
fn parser_rejects_duplicate_and_unknown_fields() {
    let receipt = fixture_receipt();
    let source = render_registered_loader_probe_admission_receipt(&receipt).unwrap();

    let duplicate = format!("contract = \"duplicate\"\n{source}");
    assert!(parse_registered_loader_probe_admission_receipt(&duplicate)
        .unwrap_err()
        .contains("duplicate-key:contract"));

    let unknown = format!("{source}unknown_field = \"value\"\n");
    assert!(parse_registered_loader_probe_admission_receipt(&unknown)
        .unwrap_err()
        .contains("unknown-keys:unknown_field"));
}

fn fixture_receipt() -> NsldRegisteredLoaderProbeAdmissionReceipt {
    let blockers = Vec::new();
    let outcome = build_registered_loader_probe_outcome(RegisteredLoaderProbeEvidence {
        provider_id: "provider.fixture",
        target_key: "x86_64-linux-elf",
        capability_id: "loader-probe.fixture",
        provider_probe_contract: "provider-probe-v1",
        provider_probe_status: "os-loader-accepted-process-succeeded",
        probe_mode: "execute",
        host_supported: true,
        input_eligible: true,
        attempted: true,
        image_span_bytes: 64,
        image_identity_hash: "0x1111111111111111",
        validation_evidence_hash: "0x2222222222222222",
        materialized: true,
        materialized_hash_matches: true,
        os_loader_accepted: true,
        process_completed: true,
        timed_out: false,
        exit_code: Some(0),
        termination_signal: None,
        stdout_captured_bytes: 0,
        stdout_truncated: false,
        stderr_captured_bytes: 0,
        stderr_truncated: false,
        failure_kind: None,
        cleanup_attempted: true,
        cleanup_succeeded: true,
        execution_admitted: true,
        blockers: &blockers,
        provider_evidence_hash: "0x3333333333333333",
    })
    .unwrap();
    let mut receipt = NsldRegisteredLoaderProbeAdmissionReceipt {
        contract: REGISTERED_LOADER_PROBE_ADMISSION_CONTRACT.to_owned(),
        status: REGISTERED_LOADER_PROBE_ADMISSION_STATUS.to_owned(),
        finalizer_registry_contract: "nuis-nsld-executable-finalizer-registry-v1".to_owned(),
        finalizer_registry_hash: "0x4444444444444444".to_owned(),
        finalizer_provider_id: outcome.provider_id.clone(),
        finalizer_target_key: outcome.target_key.clone(),
        loader_probe_capability_id: outcome.capability_id.clone(),
        target_abi: "cpu.x86_64.sysv64".to_owned(),
        machine_arch: "x86_64".to_owned(),
        machine_os: "linux".to_owned(),
        object_format: "elf".to_owned(),
        calling_abi: "sysv64".to_owned(),
        packaging_mode: "native-cpu-llvm".to_owned(),
        outcome,
        receipt_hash_sha256: String::new(),
    };
    receipt.receipt_hash_sha256 = registered_loader_probe_admission_receipt_hash(&receipt).unwrap();
    receipt
}
