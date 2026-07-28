use crate::provider_request::ProviderRequest;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const PROJECT_CODE_ASSET_IDENTITY_CONTRACT: &str =
    "nuis-kernel-project-code-asset-identity-v1";
const EVIDENCE_PREFIX: &str = "provider_code_asset_identity_";

pub(crate) fn validate_project_code_asset_identity(
    fields: &BTreeMap<String, String>,
    requests: &[ProviderRequest],
) -> bool {
    let has_identity_fields = fields.keys().any(|key| key.starts_with(EVIDENCE_PREFIX));
    let has_project_asset = requests.iter().any(|request| {
        request
            .code_asset
            .as_ref()
            .is_some_and(|asset| asset.id.starts_with("kernel.cuda.project."))
    });
    let Some(contract) = field(fields, "contract") else {
        return !has_identity_fields && !has_project_asset;
    };
    if contract != PROJECT_CODE_ASSET_IDENTITY_CONTRACT || requests.is_empty() {
        return false;
    }

    let Some(asset_id) = field(fields, "asset_id") else {
        return false;
    };
    let Some(source_fnv1a64) = field(fields, "source_fnv1a64") else {
        return false;
    };
    let Some(lowering_target) = field(fields, "lowering_target") else {
        return false;
    };
    let Some(entry_count) = field(fields, "entry_count").and_then(|value| value.parse().ok())
    else {
        return false;
    };
    let Some(entries) = field(fields, "entries").and_then(parse_entries) else {
        return false;
    };
    let Some(identity_hash) = field(fields, "hash") else {
        return false;
    };
    let entry_refs = entries.iter().map(String::as_str).collect::<Vec<_>>();
    let expected_hash =
        project_code_asset_identity_hash(source_fnv1a64, lowering_target, &entry_refs);
    let expected_id = format!("kernel.cuda.project.{}", &expected_hash[2..]);
    if !fnv1a64_hash_is_valid(source_fnv1a64)
        || !token_is_valid(lowering_target)
        || entry_count != requests.len()
        || entries.len() != entry_count
        || entries.iter().collect::<BTreeSet<_>>().len() != entries.len()
        || identity_hash != &expected_hash
        || asset_id != &expected_id
    {
        return false;
    }

    let Some(first_asset) = requests
        .first()
        .and_then(|request| request.code_asset.as_ref())
    else {
        return false;
    };
    requests
        .iter()
        .zip(entries)
        .all(|(request, expected_entry)| {
            request.code_asset.as_ref().is_some_and(|asset| {
                asset.id == *asset_id
                    && asset.entry == expected_entry
                    && asset.format == first_asset.format
                    && asset.target == first_asset.target
                    && asset.path == first_asset.path
                    && asset.byte_length == first_asset.byte_length
                    && asset.digest_contract == first_asset.digest_contract
                    && asset.content_hash == first_asset.content_hash
            })
        })
}

pub(crate) fn project_code_asset_identity_hash(
    source_fnv1a64: &str,
    lowering_target: &str,
    entries: &[&str],
) -> String {
    fnv1a64_hex(
        format!(
            "{PROJECT_CODE_ASSET_IDENTITY_CONTRACT}\n{source_fnv1a64}\n{lowering_target}\n{}\n{}",
            entries.len(),
            entries.join("\n")
        )
        .as_bytes(),
    )
}

fn field<'a>(fields: &'a BTreeMap<String, String>, name: &str) -> Option<&'a String> {
    fields.get(&format!("{EVIDENCE_PREFIX}{name}"))
}

fn parse_entries(value: &String) -> Option<Vec<String>> {
    let entries = value.split(',').map(str::to_owned).collect::<Vec<_>>();
    (!entries.is_empty()
        && entries.iter().all(|entry| {
            entry
                .bytes()
                .next()
                .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
                && entry
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        }))
    .then_some(entries)
}

fn token_is_valid(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

fn fnv1a64_hash_is_valid(value: &str) -> bool {
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
    format!("0x{hash:016x}")
}
