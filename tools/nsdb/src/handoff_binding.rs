const PROOF_CONTRACT: &str = "nuis-final-image-binding-proof-v1";
const SELECTED_SET_CONTRACT: &str = "nuis-selected-provider-bundle-set-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FinalImageBindingProofInfo {
    pub(crate) contract: String,
    pub(crate) binding_count: usize,
    pub(crate) binding_table_hash: String,
    pub(crate) validation_status: String,
    pub(crate) selected_set_contract: String,
    pub(crate) selected_set_count: usize,
    pub(crate) selected_set_hash: String,
    pub(crate) proof_hash_claim: String,
    pub(crate) proof_hash_actual: String,
    pub(crate) proof_status: String,
}

#[allow(dead_code)]
pub(crate) fn from_claim(
    claim: &crate::model::FinalImageBindingProofClaim,
) -> Result<FinalImageBindingProofInfo, String> {
    let selected_set_contract = claim
        .selected_set_contract
        .clone()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "none".to_owned());
    let selected_set_count = claim.selected_set_count.unwrap_or(0);
    let selected_set_hash = claim
        .selected_set_hash
        .clone()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "none".to_owned());
    let proof_hash_actual = proof_hash(
        claim.binding_count,
        &claim.binding_table_hash,
        &claim.validation_status,
        &selected_set_contract,
        selected_set_count,
        &selected_set_hash,
    );
    let proof_status = classify_proof(
        PROOF_CONTRACT,
        claim.binding_count,
        &claim.binding_table_hash,
        &claim.validation_status,
        &selected_set_contract,
        selected_set_count,
        &selected_set_hash,
        &proof_hash_actual,
        &proof_hash_actual,
    );
    if !matches!(proof_status.as_str(), "verified" | "verified-empty") {
        return Err(format!("final image binding claim invalid: {proof_status}"));
    }
    Ok(FinalImageBindingProofInfo {
        contract: PROOF_CONTRACT.to_owned(),
        binding_count: claim.binding_count,
        binding_table_hash: claim.binding_table_hash.clone(),
        validation_status: claim.validation_status.clone(),
        selected_set_contract,
        selected_set_count,
        selected_set_hash,
        proof_hash_claim: proof_hash_actual.clone(),
        proof_hash_actual,
        proof_status,
    })
}

pub(crate) fn same_proof(
    left: &FinalImageBindingProofInfo,
    right: &FinalImageBindingProofInfo,
) -> bool {
    left.contract == right.contract
        && left.binding_count == right.binding_count
        && left.binding_table_hash == right.binding_table_hash
        && left.validation_status == right.validation_status
        && left.selected_set_contract == right.selected_set_contract
        && left.selected_set_count == right.selected_set_count
        && left.selected_set_hash == right.selected_set_hash
        && left.proof_hash_actual == right.proof_hash_actual
}

pub(crate) fn next_action(status: &str) -> &'static str {
    match status {
        "verified" | "verified-empty" => "none",
        "legacy-unbound" => "rebuild-final-output-binding-proof",
        _ => "repair-final-image-binding-proof",
    }
}

pub(crate) fn replay_identity_hash(proof: &FinalImageBindingProofInfo) -> Result<&str, String> {
    if !matches!(proof.proof_status.as_str(), "verified" | "verified-empty") {
        return Err(format!(
            "final image binding proof is not replayable: {}",
            proof.proof_status
        ));
    }
    if proof.proof_hash_actual == "none" || !valid_fnv1a64(&proof.proof_hash_actual) {
        return Err("final image binding proof has no valid replay identity hash".to_owned());
    }
    Ok(&proof.proof_hash_actual)
}

pub(crate) fn parse_and_verify(source: &str) -> FinalImageBindingProofInfo {
    let contract = string_field(source, "final_image_binding_proof_contract")
        .unwrap_or_else(|| "none".to_owned());
    let binding_count = usize_field(source, "final_image_metadata_binding_count").unwrap_or(0);
    let binding_table_hash = string_field(source, "final_image_metadata_binding_table_hash")
        .unwrap_or_else(|| "none".to_owned());
    let validation_status = string_field(source, "final_image_metadata_binding_validation_status")
        .unwrap_or_else(|| "none".to_owned());
    let selected_set_contract =
        string_field(source, "final_image_selected_provider_bundle_set_contract")
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "none".to_owned());
    let selected_set_count =
        usize_field(source, "final_image_selected_provider_bundle_count").unwrap_or(0);
    let selected_set_hash = string_field(source, "final_image_selected_provider_bundle_set_hash")
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "none".to_owned());
    let proof_hash_claim =
        string_field(source, "final_image_binding_proof_hash").unwrap_or_else(|| "none".to_owned());
    let proof_hash_actual = proof_hash(
        binding_count,
        &binding_table_hash,
        &validation_status,
        &selected_set_contract,
        selected_set_count,
        &selected_set_hash,
    );
    let proof_status = classify_proof(
        &contract,
        binding_count,
        &binding_table_hash,
        &validation_status,
        &selected_set_contract,
        selected_set_count,
        &selected_set_hash,
        &proof_hash_claim,
        &proof_hash_actual,
    );

    FinalImageBindingProofInfo {
        contract,
        binding_count,
        binding_table_hash,
        validation_status,
        selected_set_contract,
        selected_set_count,
        selected_set_hash,
        proof_hash_claim,
        proof_hash_actual,
        proof_status,
    }
}

#[allow(clippy::too_many_arguments)]
fn classify_proof(
    contract: &str,
    binding_count: usize,
    binding_table_hash: &str,
    validation_status: &str,
    selected_set_contract: &str,
    selected_set_count: usize,
    selected_set_hash: &str,
    proof_hash_claim: &str,
    proof_hash_actual: &str,
) -> String {
    if contract == "none" {
        "legacy-unbound"
    } else if contract != PROOF_CONTRACT {
        "unsupported-contract"
    } else if proof_hash_claim != proof_hash_actual {
        "mismatch"
    } else if !valid_nsld_table_hash(binding_table_hash) {
        "binding-table-hash-invalid"
    } else if binding_count == 0 {
        if validation_status == "not-applicable"
            && selected_set_contract == "none"
            && selected_set_count == 0
            && selected_set_hash == "none"
        {
            "verified-empty"
        } else {
            "empty-proof-invalid"
        }
    } else if validation_status == "verified"
        && selected_set_contract == SELECTED_SET_CONTRACT
        && selected_set_count > 0
        && valid_fnv1a64(&selected_set_hash)
    {
        "verified"
    } else {
        "selected-set-proof-invalid"
    }
    .to_owned()
}

pub(crate) fn render_fields(proof: &FinalImageBindingProofInfo) -> String {
    if proof.contract == "none" {
        return String::new();
    }
    format!(
        "final_image_binding_proof_contract = \"{}\"\n\
         final_image_metadata_binding_count = {}\n\
         final_image_metadata_binding_table_hash = \"{}\"\n\
         final_image_metadata_binding_validation_status = \"{}\"\n\
         final_image_selected_provider_bundle_set_contract = \"{}\"\n\
         final_image_selected_provider_bundle_count = {}\n\
         final_image_selected_provider_bundle_set_hash = \"{}\"\n\
         final_image_binding_proof_hash = \"{}\"\n",
        proof.contract,
        proof.binding_count,
        proof.binding_table_hash,
        proof.validation_status,
        if proof.selected_set_contract == "none" {
            ""
        } else {
            &proof.selected_set_contract
        },
        proof.selected_set_count,
        if proof.selected_set_hash == "none" {
            ""
        } else {
            &proof.selected_set_hash
        },
        proof.proof_hash_claim,
    )
}

pub(crate) fn proof_hash(
    count: usize,
    table_hash: &str,
    status: &str,
    selected_contract: &str,
    selected_count: usize,
    selected_hash: &str,
) -> String {
    let material = format!(
        "{PROOF_CONTRACT}\0{count}\0{table_hash}\0{status}\0{selected_contract}\0{}\0{selected_hash}",
        if selected_count == 0 {
            "none".to_owned()
        } else {
            selected_count.to_string()
        }
    );
    fnv1a64_hex(material.as_bytes())
}

fn string_field(source: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} = ");
    source.lines().find_map(|line| {
        let value = line.trim().strip_prefix(&prefix)?.trim();
        value
            .strip_prefix('"')
            .and_then(|value| value.strip_suffix('"'))
            .map(str::to_owned)
    })
}

fn usize_field(source: &str, key: &str) -> Option<usize> {
    let prefix = format!("{key} = ");
    source
        .lines()
        .find_map(|line| line.trim().strip_prefix(&prefix)?.trim().parse().ok())
}

fn valid_fnv1a64(value: &str) -> bool {
    value
        .strip_prefix("fnv1a64:")
        .is_some_and(|hex| hex.len() == 16 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

fn valid_nsld_table_hash(value: &str) -> bool {
    value
        .strip_prefix("0x")
        .is_some_and(|hex| hex.len() == 16 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

fn fnv1a64_hex(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn independently_verifies_and_rejects_tampered_binding_proof() {
        let table_hash = "0x1111111111111111";
        let selected_hash = "fnv1a64:2222222222222222";
        let hash = proof_hash(
            1,
            table_hash,
            "verified",
            SELECTED_SET_CONTRACT,
            2,
            selected_hash,
        );
        let source = format!(
            "final_image_binding_proof_contract = \"{PROOF_CONTRACT}\"\n\
             final_image_metadata_binding_count = 1\n\
             final_image_metadata_binding_table_hash = \"{table_hash}\"\n\
             final_image_metadata_binding_validation_status = \"verified\"\n\
             final_image_selected_provider_bundle_set_contract = \"{SELECTED_SET_CONTRACT}\"\n\
             final_image_selected_provider_bundle_count = 2\n\
             final_image_selected_provider_bundle_set_hash = \"{selected_hash}\"\n\
             final_image_binding_proof_hash = \"{hash}\"\n"
        );
        assert_eq!(parse_and_verify(&source).proof_status, "verified");
        assert_eq!(
            parse_and_verify(&source.replace(selected_hash, "fnv1a64:3333333333333333"))
                .proof_status,
            "mismatch"
        );
    }

    #[test]
    fn claim_builder_accepts_nsld_empty_table_and_rejects_wrong_hash_contract() {
        let claim = crate::model::FinalImageBindingProofClaim {
            binding_count: 0,
            binding_table_hash: "0xcbf29ce484222325".to_owned(),
            validation_status: "not-applicable".to_owned(),
            selected_set_contract: None,
            selected_set_count: None,
            selected_set_hash: None,
        };
        let proof = from_claim(&claim).unwrap();
        assert_eq!(proof.proof_status, "verified-empty");
        assert!(proof.proof_hash_actual.starts_with("fnv1a64:"));

        let mut invalid = claim;
        invalid.binding_table_hash = "fnv1a64:cbf29ce484222325".to_owned();
        assert_eq!(
            from_claim(&invalid).unwrap_err(),
            "final image binding claim invalid: binding-table-hash-invalid"
        );
    }
}
