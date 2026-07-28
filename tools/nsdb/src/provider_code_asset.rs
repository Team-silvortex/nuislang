use std::{
    collections::BTreeMap,
    path::{Component, Path},
};

#[path = "provider_code_asset_contribution.rs"]
pub(crate) mod contribution;

pub(crate) const PROVIDER_CODE_ASSET_DESCRIPTOR_CONTRACT: &str =
    "nuis-provider-code-asset-descriptor-v1";
pub(crate) const CODE_ASSET_FNV1A64_DIGEST_CONTRACT: &str = "nuis-code-asset-digest-fnv1a64-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderCodeAssetDescriptor {
    pub(crate) id: String,
    pub(crate) format: String,
    pub(crate) target: String,
    pub(crate) entry: String,
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
    (contract == PROVIDER_CODE_ASSET_DESCRIPTOR_CONTRACT).then_some(())?;
    let asset = ProviderCodeAssetDescriptor {
        id: field(fields, prefix, "id")?.clone(),
        format: field(fields, prefix, "format")?.clone(),
        target: field(fields, prefix, "target")?.clone(),
        entry: field(fields, prefix, "entry")?.clone(),
        path: field(fields, prefix, "path")?.clone(),
        byte_length: field(fields, prefix, "byte_length")?.parse().ok()?,
        digest_contract: field(fields, prefix, "digest_contract")?.clone(),
        content_hash: field(fields, prefix, "content_hash")?.clone(),
    };
    validate_code_asset(&asset).then_some(Some(asset))
}

fn validate_code_asset(asset: &ProviderCodeAssetDescriptor) -> bool {
    token_is_valid(&asset.id)
        && token_is_valid(&asset.format)
        && token_is_valid(&asset.target)
        && symbol_is_valid(&asset.entry)
        && relative_asset_path_is_valid(&asset.path)
        && asset.byte_length > 0
        && asset.digest_contract == CODE_ASSET_FNV1A64_DIGEST_CONTRACT
        && fnv1a64_hash_is_valid(&asset.content_hash)
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
