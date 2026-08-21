use crate::provider_completion_signature::handoff_error_status;

pub(crate) struct PayloadHandoffStatus<'a> {
    pub(crate) protocol: &'a str,
    pub(crate) debugger_contract: &'a str,
    pub(crate) record_count: usize,
    pub(crate) first_status: &'a str,
    pub(crate) provider_completion_set_hash_validation_status: &'a str,
    pub(crate) provider_completion_claim_authority_status: &'a str,
    pub(crate) provider_completion_signature_status: &'a str,
    pub(crate) final_image_binding_proof_status: &'a str,
    pub(crate) provider_completion_dispatch_status: &'a str,
    pub(crate) runtime_dispatch_receipt_status: &'a str,
}

pub(crate) fn payload_handoff_status(status: PayloadHandoffStatus<'_>) -> String {
    if status.protocol != "nuis-nsdb-payload-execution-handoff-v1" {
        return "unsupported-protocol".to_owned();
    }
    if status.debugger_contract != "nsdb-yir-payload-execution-trace-v1" {
        return "unsupported-debugger-contract".to_owned();
    }
    if status.record_count == 0 {
        return "empty".to_owned();
    }
    if !matches!(
        status.final_image_binding_proof_status,
        "verified" | "verified-empty" | "legacy-unbound"
    ) {
        return format!(
            "final-image-binding-proof-{}",
            status.final_image_binding_proof_status
        );
    }
    if !matches!(
        status.runtime_dispatch_receipt_status,
        "verified" | "legacy-absent"
    ) {
        return format!(
            "runtime-dispatch-receipt-{}",
            status.runtime_dispatch_receipt_status
        );
    }
    if matches!(
        status.provider_completion_dispatch_status,
        "mismatch" | "final-image-authority-missing"
    ) {
        return format!(
            "provider-completion-dispatch-{}",
            status.provider_completion_dispatch_status
        );
    }
    match status.provider_completion_set_hash_validation_status {
        "mismatch" => return "provider-completion-set-hash-mismatch".to_owned(),
        "unsupported-digest-contract" => {
            return "provider-completion-digest-contract-unsupported".to_owned()
        }
        _ => {}
    }
    match status.provider_completion_claim_authority_status {
        "authority-missing" => return "provider-completion-claim-authority-missing".to_owned(),
        "unsupported-authority-contract" => {
            return "provider-completion-claim-authority-contract-unsupported".to_owned()
        }
        "authority-untrusted" => return "provider-completion-claim-authority-untrusted".to_owned(),
        _ => {}
    }
    if let Some(error) = handoff_error_status(status.provider_completion_signature_status) {
        return error.to_owned();
    }
    if status.first_status == "ready" {
        "ready".to_owned()
    } else {
        "blocked".to_owned()
    }
}
