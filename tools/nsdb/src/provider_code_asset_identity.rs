use crate::provider_request::ProviderRequest;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const PROJECT_CODE_ASSET_IDENTITY_CONTRACT: &str =
    "nuis-kernel-project-code-asset-identity-v1";
pub(crate) const DESCRIPTOR_CODE_ASSET_IDENTITY_CONTRACT: &str =
    "nuis-provider-code-asset-descriptor-identity-v1";
pub(crate) const CODE_ASSET_IDENTITY_SET_CONTRACT: &str =
    "nuis-provider-code-asset-identity-set-v1";
const EVIDENCE_PREFIX: &str = "provider_code_asset_identity_";
const SET_EVIDENCE_PREFIX: &str = "provider_code_asset_identity_set_";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderCodeAssetIdentitySet {
    pub(crate) asset_ids: Vec<String>,
    pub(crate) contracts: Vec<String>,
    pub(crate) identity_hashes: Vec<String>,
    pub(crate) root_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderCodeAssetIdentity {
    pub(crate) contract: String,
    pub(crate) status: &'static str,
    pub(crate) asset_id: String,
    pub(crate) identity_hash: String,
    pub(crate) identity_set: ProviderCodeAssetIdentitySet,
}

struct DeclaredIdentityItem {
    asset_id: String,
    contract: String,
    identity_hash: String,
}

struct IdentityValidatorRegistration {
    contract: &'static str,
    validate:
        fn(&BTreeMap<String, String>, usize, &DeclaredIdentityItem, &[&ProviderRequest]) -> bool,
}

const IDENTITY_VALIDATORS: &[IdentityValidatorRegistration] = &[
    IdentityValidatorRegistration {
        contract: PROJECT_CODE_ASSET_IDENTITY_CONTRACT,
        validate: validate_project_identity_item,
    },
    IdentityValidatorRegistration {
        contract: DESCRIPTOR_CODE_ASSET_IDENTITY_CONTRACT,
        validate: validate_descriptor_identity_item,
    },
];

pub(crate) fn append_provider_output_identity(
    out: &mut String,
    identity: Option<&ProviderCodeAssetIdentity>,
) {
    let (status, asset_id, identity_hash) = identity
        .map(|identity| {
            (
                identity.status,
                identity.asset_id.as_str(),
                identity.identity_hash.as_str(),
            )
        })
        .unwrap_or(("not-applicable", "none", "none"));
    let contract = identity
        .map(|identity| identity.contract.as_str())
        .unwrap_or(PROJECT_CODE_ASSET_IDENTITY_CONTRACT);
    for (key, value) in [
        ("provider_code_asset_identity_contract", contract),
        ("provider_code_asset_identity_status", status),
        ("provider_code_asset_identity_asset_id", asset_id),
        ("provider_code_asset_identity_hash", identity_hash),
    ] {
        crate::provider_sample_payload::push_toml_string(out, key, value);
    }
    let (set_status, set_count, set_root_hash) = identity
        .map(|identity| {
            (
                identity.status,
                identity.identity_set.asset_ids.len().to_string(),
                identity.identity_set.root_hash.as_str(),
            )
        })
        .unwrap_or(("not-applicable", "0".to_owned(), "none"));
    for (key, value) in [
        (
            "provider_code_asset_identity_set_contract",
            CODE_ASSET_IDENTITY_SET_CONTRACT,
        ),
        ("provider_code_asset_identity_set_status", set_status),
        ("provider_code_asset_identity_set_count", &set_count),
        ("provider_code_asset_identity_set_root_hash", set_root_hash),
    ] {
        crate::provider_sample_payload::push_toml_string(out, key, value);
    }
}

pub(crate) fn verified_code_asset_identity_collection(
    fields: &BTreeMap<String, String>,
    requests: &[ProviderRequest],
) -> Option<Option<ProviderCodeAssetIdentity>> {
    let has_identity_fields = fields.keys().any(|key| key.starts_with(EVIDENCE_PREFIX));
    let has_project_asset = requests.iter().any(|request| {
        request
            .code_asset
            .as_ref()
            .is_some_and(|asset| asset.id.contains(".project."))
    });
    let Some(set_contract) = set_field(fields, "contract") else {
        return (!has_identity_fields && !has_project_asset).then_some(None);
    };
    (set_contract == CODE_ASSET_IDENTITY_SET_CONTRACT).then_some(())?;
    let count = set_field(fields, "count")?.parse::<usize>().ok()?;
    (1..=64).contains(&count).then_some(())?;
    let items = (0..count)
        .map(|index| parse_identity_item(fields, index))
        .collect::<Option<Vec<_>>>()?;
    let item_refs = items
        .iter()
        .map(|item| {
            (
                item.asset_id.as_str(),
                item.contract.as_str(),
                item.identity_hash.as_str(),
            )
        })
        .collect::<Vec<_>>();
    let root_hash = set_field(fields, "root_hash")?;
    let ordered_asset_ids = ordered_request_asset_ids(requests);
    if items
        .iter()
        .map(|item| item.asset_id.as_str())
        .collect::<Vec<_>>()
        != ordered_asset_ids
        || items
            .iter()
            .map(|item| item.asset_id.as_str())
            .collect::<BTreeSet<_>>()
            .len()
            != count
        || root_hash != &code_asset_identity_set_root_hash(&item_refs)
    {
        return None;
    }
    for (index, item) in items.iter().enumerate() {
        let partition = requests
            .iter()
            .filter(|request| {
                request
                    .code_asset
                    .as_ref()
                    .is_some_and(|asset| asset.id == item.asset_id)
            })
            .collect::<Vec<_>>();
        let validator = IDENTITY_VALIDATORS
            .iter()
            .find(|validator| validator.contract == item.contract)?;
        ((validator.validate)(fields, index, item, &partition)).then_some(())?;
    }
    validate_primary_compatibility(fields, &items[0])?;
    let first = &items[0];
    Some(Some(ProviderCodeAssetIdentity {
        contract: first.contract.clone(),
        status: "verified",
        asset_id: first.asset_id.clone(),
        identity_hash: first.identity_hash.clone(),
        identity_set: ProviderCodeAssetIdentitySet {
            asset_ids: items.iter().map(|item| item.asset_id.clone()).collect(),
            contracts: items.iter().map(|item| item.contract.clone()).collect(),
            identity_hashes: items
                .iter()
                .map(|item| item.identity_hash.clone())
                .collect(),
            root_hash: root_hash.clone(),
        },
    }))
}

fn parse_identity_item(
    fields: &BTreeMap<String, String>,
    index: usize,
) -> Option<DeclaredIdentityItem> {
    let item = DeclaredIdentityItem {
        asset_id: item_field(fields, index, "asset_id")?.clone(),
        contract: item_field(fields, index, "contract")?.clone(),
        identity_hash: item_field(fields, index, "hash")?.clone(),
    };
    (token_is_valid(&item.asset_id)
        && token_is_valid(&item.contract)
        && fnv1a64_hash_is_valid(&item.identity_hash))
    .then_some(item)
}

fn ordered_request_asset_ids(requests: &[ProviderRequest]) -> Vec<&str> {
    let mut seen = BTreeSet::new();
    requests
        .iter()
        .filter_map(|request| request.code_asset.as_ref().map(|asset| asset.id.as_str()))
        .filter(|asset_id| seen.insert(*asset_id))
        .collect()
}

fn validate_project_identity_item(
    fields: &BTreeMap<String, String>,
    index: usize,
    item: &DeclaredIdentityItem,
    partition: &[&ProviderRequest],
) -> bool {
    let Some(source_fnv1a64) = item_detail(fields, index, "source_fnv1a64") else {
        return false;
    };
    let Some(lowering_target) = item_detail(fields, index, "lowering_target") else {
        return false;
    };
    let Some(entry_count) =
        item_detail(fields, index, "entry_count").and_then(|value| value.parse::<usize>().ok())
    else {
        return false;
    };
    let Some(entries) = item_detail(fields, index, "entries").and_then(parse_entries) else {
        return false;
    };
    let entry_refs = entries.iter().map(String::as_str).collect::<Vec<_>>();
    let expected_hash =
        project_code_asset_identity_hash(source_fnv1a64, lowering_target, &entry_refs);
    if !fnv1a64_hash_is_valid(source_fnv1a64)
        || !token_is_valid(lowering_target)
        || partition.is_empty()
        || partition.len() != entry_count
        || entries.len() != entry_count
        || entries.iter().collect::<BTreeSet<_>>().len() != entries.len()
        || item.identity_hash != expected_hash
        || !project_asset_id_matches_hash(&item.asset_id, &expected_hash)
    {
        return false;
    }
    shared_descriptors_match(partition, &entries)
}

fn validate_descriptor_identity_item(
    _: &BTreeMap<String, String>,
    _: usize,
    item: &DeclaredIdentityItem,
    partition: &[&ProviderRequest],
) -> bool {
    let Some(first) = partition
        .first()
        .and_then(|request| request.code_asset.as_ref())
    else {
        return false;
    };
    let entries = partition
        .iter()
        .filter_map(|request| {
            request
                .code_asset
                .as_ref()
                .map(|asset| asset.entry.as_str())
        })
        .collect::<Vec<_>>();
    shared_descriptors_match(partition, &entries)
        && item.identity_hash
            == descriptor_code_asset_identity_hash(
                &item.asset_id,
                &first.format,
                &first.target,
                &first.path,
                first.byte_length,
                &first.digest_contract,
                &first.content_hash,
                &entries,
            )
}

fn shared_descriptors_match(partition: &[&ProviderRequest], entries: &[impl AsRef<str>]) -> bool {
    let Some(first) = partition
        .first()
        .and_then(|request| request.code_asset.as_ref())
    else {
        return false;
    };
    partition
        .iter()
        .zip(entries)
        .all(|(request, expected_entry)| {
            request.code_asset.as_ref().is_some_and(|asset| {
                asset.entry == expected_entry.as_ref()
                    && asset.format == first.format
                    && asset.target == first.target
                    && asset.path == first.path
                    && asset.byte_length == first.byte_length
                    && asset.digest_contract == first.digest_contract
                    && asset.content_hash == first.content_hash
            })
        })
}

fn validate_primary_compatibility(
    fields: &BTreeMap<String, String>,
    first: &DeclaredIdentityItem,
) -> Option<()> {
    if first.contract != PROJECT_CODE_ASSET_IDENTITY_CONTRACT {
        return Some(());
    }
    (field(fields, "contract")? == &first.contract
        && field(fields, "asset_id")? == &first.asset_id
        && field(fields, "hash")? == &first.identity_hash
        && [
            "source_fnv1a64",
            "lowering_target",
            "entry_count",
            "entries",
        ]
        .iter()
        .all(|name| field(fields, name) == item_field(fields, 0, name)))
    .then_some(())
}

fn project_asset_id_matches_hash(asset_id: &str, identity_hash: &str) -> bool {
    asset_id.starts_with("kernel.")
        && asset_id.contains(".project.")
        && asset_id.ends_with(&identity_hash[2..])
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

#[allow(clippy::too_many_arguments)]
pub(crate) fn descriptor_code_asset_identity_hash(
    asset_id: &str,
    format: &str,
    target: &str,
    path: &str,
    byte_length: usize,
    digest_contract: &str,
    content_hash: &str,
    entries: &[&str],
) -> String {
    fnv1a64_hex(
        format!(
            "{DESCRIPTOR_CODE_ASSET_IDENTITY_CONTRACT}\n{asset_id}\n{format}\n{target}\n{path}\n{byte_length}\n{digest_contract}\n{content_hash}\n{}\n{}",
            entries.len(),
            entries.join("\n")
        )
        .as_bytes(),
    )
}

pub(crate) fn code_asset_identity_set_root_hash(items: &[(&str, &str, &str)]) -> String {
    let ordered_items = items
        .iter()
        .map(|(asset_id, contract, identity_hash)| {
            format!("{asset_id}\n{contract}\n{identity_hash}")
        })
        .collect::<Vec<_>>()
        .join("\n");
    fnv1a64_hex(
        format!(
            "{CODE_ASSET_IDENTITY_SET_CONTRACT}\n{}\n{ordered_items}",
            items.len()
        )
        .as_bytes(),
    )
}

fn field<'a>(fields: &'a BTreeMap<String, String>, name: &str) -> Option<&'a String> {
    fields.get(&format!("{EVIDENCE_PREFIX}{name}"))
}

fn set_field<'a>(fields: &'a BTreeMap<String, String>, name: &str) -> Option<&'a String> {
    fields.get(&format!("{SET_EVIDENCE_PREFIX}{name}"))
}

fn item_field<'a>(
    fields: &'a BTreeMap<String, String>,
    index: usize,
    name: &str,
) -> Option<&'a String> {
    fields.get(&format!("{EVIDENCE_PREFIX}item_{index}_{name}"))
}

fn item_detail<'a>(
    fields: &'a BTreeMap<String, String>,
    index: usize,
    name: &str,
) -> Option<&'a String> {
    item_field(fields, index, name).or_else(|| (index == 0).then(|| field(fields, name)).flatten())
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
