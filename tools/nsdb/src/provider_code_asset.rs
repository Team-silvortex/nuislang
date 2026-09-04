use std::{
    collections::BTreeMap,
    path::{Component, Path},
};

#[path = "provider_code_asset_contribution.rs"]
pub(crate) mod contribution;
#[path = "provider_code_asset_contribution_set.rs"]
mod contribution_set;
#[path = "provider_code_asset_selection_payload.rs"]
pub(crate) mod selection_payload;

pub(crate) const PROVIDER_CODE_ASSET_DESCRIPTOR_CONTRACT: &str =
    "nuis-provider-code-asset-descriptor-v1";
pub(crate) const PROVIDER_CODE_ASSET_DESCRIPTOR_V2_CONTRACT: &str =
    "nuis-provider-code-asset-descriptor-v2";
pub(crate) const CODE_ASSET_FNV1A64_DIGEST_CONTRACT: &str = "nuis-code-asset-digest-fnv1a64-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderCodeAssetDescriptor {
    pub(crate) descriptor_contract: String,
    pub(crate) id: String,
    pub(crate) format: String,
    pub(crate) target: String,
    pub(crate) entry: String,
    pub(crate) entries: Vec<String>,
    pub(crate) path: String,
    pub(crate) byte_length: usize,
    pub(crate) digest_contract: String,
    pub(crate) content_hash: String,
}

pub(crate) fn parse_code_asset(
    fields: &BTreeMap<String, String>,
    prefix: &str,
) -> Option<Option<ProviderCodeAssetDescriptor>> {
    let contract_key = format!("{prefix}descriptor_contract");
    let Some(contract) = fields.get(&contract_key) else {
        return (!fields.keys().any(|key| key.starts_with(prefix))).then_some(None);
    };
    matches!(
        contract.as_str(),
        PROVIDER_CODE_ASSET_DESCRIPTOR_CONTRACT | PROVIDER_CODE_ASSET_DESCRIPTOR_V2_CONTRACT
    )
    .then_some(())?;
    let entry = field(fields, prefix, "entry")?.clone();
    let entries = if contract == PROVIDER_CODE_ASSET_DESCRIPTOR_CONTRACT {
        (!fields.contains_key(&format!("{prefix}entry_count"))
            && !fields.contains_key(&format!("{prefix}entries")))
        .then_some(vec![entry.clone()])?
    } else {
        let count = field(fields, prefix, "entry_count")?
            .parse::<usize>()
            .ok()?;
        let entries = parse_entries(field(fields, prefix, "entries")?)?;
        (entries.len() == count).then_some(entries)?
    };
    let asset = ProviderCodeAssetDescriptor {
        descriptor_contract: contract.clone(),
        id: field(fields, prefix, "id")?.clone(),
        format: field(fields, prefix, "format")?.clone(),
        target: field(fields, prefix, "target")?.clone(),
        entry,
        entries,
        path: field(fields, prefix, "path")?.clone(),
        byte_length: field(fields, prefix, "byte_length")?.parse().ok()?,
        digest_contract: field(fields, prefix, "digest_contract")?.clone(),
        content_hash: field(fields, prefix, "content_hash")?.clone(),
    };
    validate_code_asset(&asset).then_some(Some(asset))
}

fn validate_code_asset(asset: &ProviderCodeAssetDescriptor) -> bool {
    matches!(
        asset.descriptor_contract.as_str(),
        PROVIDER_CODE_ASSET_DESCRIPTOR_CONTRACT | PROVIDER_CODE_ASSET_DESCRIPTOR_V2_CONTRACT
    ) && token_is_valid(&asset.id)
        && token_is_valid(&asset.format)
        && token_is_valid(&asset.target)
        && symbol_is_valid(&asset.entry)
        && (1..=64).contains(&asset.entries.len())
        && asset.entries.first() == Some(&asset.entry)
        && asset.entries.iter().all(|entry| symbol_is_valid(entry))
        && asset
            .entries
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            == asset.entries.len()
        && relative_asset_path_is_valid(&asset.path)
        && asset.byte_length > 0
        && asset.digest_contract == CODE_ASSET_FNV1A64_DIGEST_CONTRACT
        && fnv1a64_hash_is_valid(&asset.content_hash)
}

fn parse_entries(value: &str) -> Option<Vec<String>> {
    let entries = value.split(',').map(str::to_owned).collect::<Vec<_>>();
    (!entries.is_empty() && entries.iter().all(|entry| !entry.is_empty())).then_some(entries)
}

fn token_is_valid(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

fn symbol_is_valid(value: &str) -> bool {
    value
        .bytes()
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'$'))
}

fn relative_asset_path_is_valid(value: &str) -> bool {
    let path = Path::new(value);
    !value.is_empty()
        && !value.contains(['\\', ':'])
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn fnv1a64_hash_is_valid(value: &str) -> bool {
    value
        .strip_prefix("0x")
        .is_some_and(|hex| hex.len() == 16 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

fn field<'a>(fields: &'a BTreeMap<String, String>, prefix: &str, name: &str) -> Option<&'a String> {
    fields.get(&format!("{prefix}{name}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn descriptor(path: &str) -> BTreeMap<String, String> {
        [
            (
                "asset_descriptor_contract",
                PROVIDER_CODE_ASSET_DESCRIPTOR_CONTRACT,
            ),
            ("asset_id", "kernel.vector-add"),
            ("asset_format", "ptx"),
            ("asset_target", "sm_80"),
            ("asset_entry", "nuis_kernel_vector_add_f32"),
            ("asset_path", path),
            ("asset_byte_length", "512"),
            ("asset_digest_contract", CODE_ASSET_FNV1A64_DIGEST_CONTRACT),
            ("asset_content_hash", "0x0123456789abcdef"),
        ]
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect()
    }

    #[test]
    fn parses_provider_neutral_ptx_asset() {
        let asset = parse_code_asset(&descriptor("payload/kernel.ptx"), "asset_")
            .expect("valid descriptor")
            .expect("present descriptor");
        assert_eq!(asset.format, "ptx");
        assert_eq!(asset.target, "sm_80");
        assert_eq!(asset.entry, "nuis_kernel_vector_add_f32");
        assert_eq!(asset.entries, ["nuis_kernel_vector_add_f32"]);
    }

    #[test]
    fn parses_multi_entry_descriptor_v2() {
        let mut fields = descriptor("render.metal");
        fields.insert(
            "asset_descriptor_contract".to_owned(),
            PROVIDER_CODE_ASSET_DESCRIPTOR_V2_CONTRACT.to_owned(),
        );
        fields.insert("asset_entry".to_owned(), "vs_main".to_owned());
        fields.insert("asset_entry_count".to_owned(), "2".to_owned());
        fields.insert("asset_entries".to_owned(), "vs_main,fs_main".to_owned());

        let asset = parse_code_asset(&fields, "asset_")
            .expect("valid v2 descriptor")
            .expect("present descriptor");
        assert_eq!(asset.entry, "vs_main");
        assert_eq!(asset.entries, ["vs_main", "fs_main"]);
    }

    #[test]
    fn rejects_drifted_multi_entry_descriptor_v2() {
        let mut fields = descriptor("render.metal");
        fields.insert(
            "asset_descriptor_contract".to_owned(),
            PROVIDER_CODE_ASSET_DESCRIPTOR_V2_CONTRACT.to_owned(),
        );
        fields.insert("asset_entry".to_owned(), "vs_main".to_owned());
        fields.insert("asset_entry_count".to_owned(), "2".to_owned());
        fields.insert("asset_entries".to_owned(), "fs_main,vs_main".to_owned());
        assert!(parse_code_asset(&fields, "asset_").is_none());

        fields.insert("asset_entries".to_owned(), "vs_main,vs_main".to_owned());
        assert!(parse_code_asset(&fields, "asset_").is_none());
    }

    #[test]
    fn rejects_absolute_or_traversing_asset_paths() {
        assert!(parse_code_asset(&descriptor("/tmp/kernel.ptx"), "asset_").is_none());
        assert!(parse_code_asset(&descriptor("../kernel.ptx"), "asset_").is_none());
        assert!(parse_code_asset(&descriptor(r"C:\temp\kernel.ptx"), "asset_").is_none());
    }

    #[test]
    fn rejects_partial_or_malformed_hash_binding() {
        let mut partial = descriptor("kernel.ptx");
        partial.remove("asset_descriptor_contract");
        assert!(parse_code_asset(&partial, "asset_").is_none());

        let mut malformed = descriptor("kernel.ptx");
        malformed.insert("asset_content_hash".to_owned(), "0xabcd".to_owned());
        assert!(parse_code_asset(&malformed, "asset_").is_none());
    }
}
