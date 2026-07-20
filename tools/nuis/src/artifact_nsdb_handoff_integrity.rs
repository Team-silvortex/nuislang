pub(crate) const DIGEST_FNV1A64_CONTRACT: &str = "nuis-provider-completion-digest-fnv1a64-v1";
pub(crate) const DIGEST_SHA256_CONTRACT: &str = "nuis-provider-completion-digest-sha256-v1";
pub(crate) const DIGEST_SHA256_AUTHORITY_CONTRACT: &str =
    "nuis-provider-completion-digest-sha256-authority-v1";
pub(crate) const DIGEST_SHA256_SIGNED_CONTRACT: &str =
    "nuis-provider-completion-digest-sha256-signed-v1";
pub(crate) const CLAIM_AUTHORITY_CONTRACT: &str = "nuis-provider-completion-claim-authority-v1";
pub(crate) const CLAIM_AUTHORITY: &str = "nsdb:payload-execution-handoff-writer:v1";

pub(crate) fn record_hash(contract: &str, bytes: &[u8]) -> Option<String> {
    digest_hex(contract, bytes)
}

pub(crate) fn set_hash(
    record_hashes: &[&str],
    protocol: &str,
    record_count: usize,
    digest_contract: &str,
    authority_contract: Option<&str>,
    authority: Option<&str>,
) -> Option<String> {
    if record_hashes.is_empty() {
        return None;
    }
    let material = record_hashes.join("\0");
    let (domain, authority_material) = match digest_contract {
        DIGEST_SHA256_AUTHORITY_CONTRACT | DIGEST_SHA256_SIGNED_CONTRACT => (
            if digest_contract == DIGEST_SHA256_SIGNED_CONTRACT {
                "provider-completion-set-v5"
            } else {
                "provider-completion-set-v4"
            },
            format!(
                "{}\0{}\0",
                authority_contract.unwrap_or("none"),
                authority.unwrap_or("none")
            ),
        ),
        DIGEST_SHA256_CONTRACT => ("provider-completion-set-v3", String::new()),
        DIGEST_FNV1A64_CONTRACT => ("provider-completion-set-v2", String::new()),
        _ => return None,
    };
    digest_hex(
        digest_contract,
        format!(
            "{domain}\0{authority_material}{protocol}\0{record_count}\0{}\0{material}",
            record_hashes.len()
        )
        .as_bytes(),
    )
}

pub(crate) fn legacy_set_hash(record_hashes: &[&str]) -> Option<String> {
    (!record_hashes.is_empty()).then(|| {
        let material = record_hashes.join("\0");
        fnv1a64_hex(format!("provider-completion-set-v1\0{material}").as_bytes())
    })
}

pub(crate) fn signature_message(
    protocol: &str,
    digest_contract: &str,
    authority_contract: &str,
    authority: &str,
    set_hash: &str,
) -> Vec<u8> {
    format!(
        "provider-completion-signature-v1\0{protocol}\0{digest_contract}\0{authority_contract}\0{authority}\0{set_hash}"
    )
    .into_bytes()
}

fn digest_hex(contract: &str, bytes: &[u8]) -> Option<String> {
    match contract {
        DIGEST_FNV1A64_CONTRACT => Some(fnv1a64_hex(bytes)),
        DIGEST_SHA256_CONTRACT
        | DIGEST_SHA256_AUTHORITY_CONTRACT
        | DIGEST_SHA256_SIGNED_CONTRACT => Some(crate::digest_sha256::sha256_hex(bytes)),
        _ => None,
    }
}

fn fnv1a64_hex(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("0x{hash:016x}")
}
