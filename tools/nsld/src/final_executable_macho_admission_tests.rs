use super::*;
use crate::final_executable_macho_admission_receipt::{
    parse_macho_arm64_publication_admission_receipt, receipt_hash_sha256,
    render_macho_arm64_publication_admission_receipt,
};

#[test]
fn publication_admission_receipt_roundtrips_canonically() {
    let receipt = sample_receipt();

    let source = render_macho_arm64_publication_admission_receipt(&receipt).unwrap();
    let parsed = parse_macho_arm64_publication_admission_receipt(&source).unwrap();

    assert_eq!(parsed, receipt);
    assert_eq!(receipt.receipt_hash_sha256.len(), 64);
    assert_eq!(
        receipt_hash_sha256(&receipt).unwrap(),
        receipt.receipt_hash_sha256
    );
}

#[test]
fn publication_admission_receipt_rejects_duplicate_and_unknown_keys() {
    let receipt = sample_receipt();
    let source = render_macho_arm64_publication_admission_receipt(&receipt).unwrap();

    let duplicate = format!("{source}contract = \"duplicate\"\n");
    let unknown = format!("{source}future_field = \"unexpected\"\n");

    assert!(parse_macho_arm64_publication_admission_receipt(&duplicate)
        .unwrap_err()
        .contains("duplicate-key:contract"));
    assert!(parse_macho_arm64_publication_admission_receipt(&unknown)
        .unwrap_err()
        .contains("unknown-keys:future_field"));
}

#[test]
fn publication_admission_hash_binds_probe_success_evidence() {
    let receipt = sample_receipt();
    let mut drifted = receipt.clone();
    drifted.probe_kernel_accepted = false;

    assert_ne!(
        receipt_hash_sha256(&receipt).unwrap(),
        receipt_hash_sha256(&drifted).unwrap()
    );
}

fn sample_receipt() -> NsldMachOArm64PublicationAdmissionReceipt {
    let mut receipt = NsldMachOArm64PublicationAdmissionReceipt {
        contract: MACHO_ARM64_PUBLICATION_ADMISSION_CONTRACT.to_owned(),
        status: MACHO_ARM64_PUBLICATION_ADMISSION_STATUS.to_owned(),
        finalizer_registry_contract: "nuis-nsld-executable-finalizer-registry-v1".to_owned(),
        finalizer_registry_hash: "0x1000".to_owned(),
        finalizer_provider_id: "nsld.finalizer.mach-o.arm64.artifact-image-v1".to_owned(),
        finalizer_target_key: "aarch64-macos-mach-o".to_owned(),
        target_arch: "aarch64".to_owned(),
        target_os: "macos".to_owned(),
        object_format: "mach-o".to_owned(),
        calling_abi: "aapcs64".to_owned(),
        packaging_mode: "native-cpu-llvm".to_owned(),
        object_linkage_hash: "0x1001".to_owned(),
        shell_layout_plan_hash: "0x1002".to_owned(),
        serialization_ledger_hash: "0x1003".to_owned(),
        shell_image_span_bytes: 16_384,
        shell_image_hash: "0x1004".to_owned(),
        shell_image_sha256: "11".repeat(32),
        signature_validation_contract: "nuis-nsld-macho-arm64-signed-image-validation-v1"
            .to_owned(),
        signature_validation_status: "signed-private-image-structurally-valid".to_owned(),
        signature_validation_ledger_hash: "0x1005".to_owned(),
        signature_cdhash: "22".repeat(20),
        probe_contract: "nuis-nsld-macho-arm64-os-loader-probe-v1".to_owned(),
        probe_status: "os-loader-accepted-process-succeeded".to_owned(),
        probe_ledger_hash: "0x1006".to_owned(),
        probe_timeout_millis: 5_000,
        probe_host_supported: true,
        probe_input_eligible: true,
        probe_attempted: true,
        probe_materialized: true,
        probe_materialized_hash_matches: true,
        probe_kernel_accepted: true,
        probe_process_completed: true,
        probe_timed_out: false,
        probe_exit_code: Some(0),
        probe_termination_signal: None,
        probe_stdout_captured_bytes: 0,
        probe_stdout_truncated: false,
        probe_stdout_hash: "0xcbf29ce484222325".to_owned(),
        probe_stderr_captured_bytes: 0,
        probe_stderr_truncated: false,
        probe_stderr_hash: "0xcbf29ce484222325".to_owned(),
        probe_failure_kind: None,
        probe_cleanup_attempted: true,
        probe_cleanup_succeeded: true,
        unresolved_external_symbol_count: 0,
        bind_count: 0,
        publication_eligibility_contract: "nuis-nsld-macho-arm64-publication-eligibility-v1"
            .to_owned(),
        publication_eligibility_status: "eligible-isolated-os-loader-probe-passed".to_owned(),
        publication_eligible: true,
        receipt_hash_sha256: String::new(),
    };
    receipt.receipt_hash_sha256 = receipt_hash_sha256(&receipt).unwrap();
    receipt
}
