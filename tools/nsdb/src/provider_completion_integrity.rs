use crate::model::NsdbPayloadExecutionEvent;

pub(crate) const DIGEST_FNV1A64_CONTRACT: &str = "nuis-provider-completion-digest-fnv1a64-v1";
pub(crate) const DIGEST_SHA256_CONTRACT: &str = "nuis-provider-completion-digest-sha256-v1";
pub(crate) const DIGEST_SHA256_AUTHORITY_CONTRACT: &str =
    "nuis-provider-completion-digest-sha256-authority-v1";
pub(crate) const CLAIM_AUTHORITY_CONTRACT: &str = "nuis-provider-completion-claim-authority-v1";
pub(crate) const CLAIM_AUTHORITY: &str = "nsdb:payload-execution-handoff-writer:v1";

pub(crate) fn record_hash(
    event: &NsdbPayloadExecutionEvent,
    digest_contract: &str,
) -> Option<String> {
    let material = format!(
        "{}\0{}\0{}\0{}",
        event.trace_id, event.provider_family, event.output_contract, event.output_evidence
    );
    digest_hex(digest_contract, material.as_bytes())
}

pub(crate) fn set_hash(
    events: &[NsdbPayloadExecutionEvent],
    protocol: &str,
    record_count: usize,
    digest_contract: &str,
    authority_contract: &str,
    authority: &str,
) -> Option<String> {
    let record_hashes = events
        .iter()
        .filter(|event| event.execution_phase == "provider-device-completion")
        .map(|event| record_hash(event, digest_contract))
        .collect::<Option<Vec<_>>>()?;
    (!record_hashes.is_empty()).then(|| {
        let material = record_hashes.join("\0");
        let (domain, authority_material) = match digest_contract {
            DIGEST_SHA256_AUTHORITY_CONTRACT => (
                "provider-completion-set-v4",
                format!("{authority_contract}\0{authority}\0"),
            ),
            DIGEST_SHA256_CONTRACT => ("provider-completion-set-v3", String::new()),
            _ => ("provider-completion-set-v2", String::new()),
        };
        digest_hex(
            digest_contract,
            format!(
                "{domain}\0{authority_material}{protocol}\0{record_count}\0{}\0{material}",
                record_hashes.len()
            )
            .as_bytes(),
        )
        .expect("validated provider completion digest contract")
    })
}

pub(crate) fn legacy_set_hash(events: &[NsdbPayloadExecutionEvent]) -> Option<String> {
    let record_hashes = events
        .iter()
        .filter(|event| event.execution_phase == "provider-device-completion")
        .map(|event| record_hash(event, DIGEST_FNV1A64_CONTRACT).expect("legacy FNV-1a contract"))
        .collect::<Vec<_>>();
    (!record_hashes.is_empty()).then(|| {
        let material = record_hashes.join("\0");
        fnv1a64_hex(format!("provider-completion-set-v1\0{material}").as_bytes())
    })
}

fn digest_hex(contract: &str, bytes: &[u8]) -> Option<String> {
    match contract {
        DIGEST_FNV1A64_CONTRACT => Some(fnv1a64_hex(bytes)),
        DIGEST_SHA256_CONTRACT | DIGEST_SHA256_AUTHORITY_CONTRACT => {
            Some(crate::digest_sha256::sha256_hex(bytes))
        }
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
