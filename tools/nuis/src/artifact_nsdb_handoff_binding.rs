const PROOF_CONTRACT: &str = "nuis-final-image-binding-proof-v1";
const SELECTED_SET_CONTRACT: &str = "nuis-selected-provider-bundle-set-v1";

pub(crate) struct PersistedFinalImageBindingProof {
    pub(crate) contract: Option<String>,
    pub(crate) binding_count: usize,
    pub(crate) binding_table_hash: Option<String>,
    pub(crate) validation_status: String,
    pub(crate) selected_set_contract: Option<String>,
    pub(crate) selected_set_count: Option<usize>,
    pub(crate) selected_set_hash: Option<String>,
    pub(crate) proof_hash: Option<String>,
    pub(crate) verification_status: String,
}

pub(crate) fn independently_verify(source: &str) -> PersistedFinalImageBindingProof {
    let contract = string_field(source, "final_image_binding_proof_contract");
    let binding_count = usize_field(source, "final_image_metadata_binding_count").unwrap_or(0);
    let binding_table_hash = string_field(source, "final_image_metadata_binding_table_hash")
        .filter(|value| !value.is_empty());
    let validation_status = string_field(source, "final_image_metadata_binding_validation_status")
        .unwrap_or_else(|| "none".to_owned());
    let selected_set_contract =
        string_field(source, "final_image_selected_provider_bundle_set_contract")
            .filter(|value| !value.is_empty());
    let selected_set_count = usize_field(source, "final_image_selected_provider_bundle_count")
        .filter(|count| *count > 0);
    let selected_set_hash = string_field(source, "final_image_selected_provider_bundle_set_hash")
        .filter(|value| !value.is_empty());
    let proof_hash =
        string_field(source, "final_image_binding_proof_hash").filter(|value| !value.is_empty());
    let actual_hash = canonical_hash(
        binding_count,
        binding_table_hash.as_deref().unwrap_or("none"),
        &validation_status,
        selected_set_contract.as_deref().unwrap_or("none"),
        selected_set_count,
        selected_set_hash.as_deref().unwrap_or("none"),
    );
    let verification_status = if contract.is_none() {
        "legacy-unbound"
    } else if contract.as_deref() != Some(PROOF_CONTRACT) {
        "unsupported-contract"
    } else if proof_hash.as_deref() != Some(actual_hash.as_str()) {
        "mismatch"
    } else if !binding_table_hash
        .as_deref()
        .is_some_and(valid_nsld_table_hash)
    {
        "binding-table-hash-invalid"
    } else if binding_count == 0 {
        if validation_status == "not-applicable"
            && selected_set_contract.is_none()
            && selected_set_count.is_none()
            && selected_set_hash.is_none()
        {
            "verified-empty"
        } else {
            "empty-proof-invalid"
        }
    } else if validation_status == "verified" {
        let provider_selection_absent = selected_set_contract.is_none()
            && selected_set_count.is_none()
            && selected_set_hash.is_none();
        let provider_selection_verified = selected_set_contract.as_deref()
            == Some(SELECTED_SET_CONTRACT)
            && selected_set_count.is_some()
            && selected_set_hash.as_deref().is_some_and(valid_fnv1a64);
        if provider_selection_absent || provider_selection_verified {
            "verified"
        } else {
            "selected-set-proof-invalid"
        }
    } else {
        "selected-set-proof-invalid"
    }
    .to_owned();

    PersistedFinalImageBindingProof {
        contract,
        binding_count,
        binding_table_hash,
        validation_status,
        selected_set_contract,
        selected_set_count,
        selected_set_hash,
        proof_hash,
        verification_status,
    }
}

pub(crate) fn next_action(status: &str) -> &'static str {
    match status {
        "verified" | "verified-empty" => "none",
        "legacy-unbound" => "rebuild-final-output-binding-proof",
        _ => "repair-final-image-binding-proof",
    }
}

fn canonical_hash(
    count: usize,
    table_hash: &str,
    status: &str,
    selected_contract: &str,
    selected_count: Option<usize>,
    selected_hash: &str,
) -> String {
    let material = format!(
        "{PROOF_CONTRACT}\0{count}\0{table_hash}\0{status}\0{selected_contract}\0{}\0{selected_hash}",
        selected_count
            .map(|count| count.to_string())
            .unwrap_or_else(|| "none".to_owned())
    );
    fnv1a64_hex(material.as_bytes())
}

fn string_field(source: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} = ");
    source.lines().find_map(|line| {
        line.trim()
            .strip_prefix(&prefix)?
            .trim()
            .strip_prefix('"')?
            .strip_suffix('"')
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
    fn independently_verifies_runtime_only_binding_proof() {
        let hash = canonical_hash(2, "0x1111111111111111", "verified", "none", None, "none");
        let source = format!(
            "final_image_binding_proof_contract = \"{PROOF_CONTRACT}\"\n\
             final_image_metadata_binding_count = 2\n\
             final_image_metadata_binding_table_hash = \"0x1111111111111111\"\n\
             final_image_metadata_binding_validation_status = \"verified\"\n\
             final_image_selected_provider_bundle_set_contract = \"\"\n\
             final_image_selected_provider_bundle_count = 0\n\
             final_image_selected_provider_bundle_set_hash = \"\"\n\
             final_image_binding_proof_hash = \"{hash}\"\n"
        );

        assert_eq!(
            independently_verify(&source).verification_status,
            "verified"
        );
    }
}
