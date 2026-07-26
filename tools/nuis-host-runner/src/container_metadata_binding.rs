use crate::{
    container_toml::{
        array_table_blocks, bool_value_from_lines, string_value_from_lines, usize_value,
        usize_value_from_lines,
    },
    fnv1a64_hex,
};
use std::collections::BTreeSet;

const SELECTED_SET_BINDING_ID: &str = "identity.selected-provider-bundle-set";
const SELECTED_SET_CONTRACT: &str = "nuis-selected-provider-bundle-set-v1";
const PROVIDER_DISPATCH_BINDING_ID: &str = "runtime.provider-dispatch-table";
const PROVIDER_DISPATCH_CONTRACT: &str = "nuis-final-image-provider-dispatch-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
struct MetadataBinding {
    binding_id: String,
    contract: String,
    value_count: usize,
    value_hash: String,
    validation_status: String,
    required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct MetadataBindingSummary {
    pub(super) declared_count: Option<usize>,
    pub(super) parsed_count: usize,
    pub(super) table_hash: Option<String>,
    pub(super) validation_status: String,
    pub(super) selected_set_contract: Option<String>,
    pub(super) selected_set_count: Option<usize>,
    pub(super) selected_set_hash: Option<String>,
    pub(super) provider_dispatch_count: Option<usize>,
    pub(super) provider_dispatch_hash: Option<String>,
    pub(super) blockers: Vec<String>,
}

pub(super) fn scan_metadata_bindings(source: &str) -> MetadataBindingSummary {
    let declared_count = usize_value(source, "metadata_binding_count");
    let table_hash = crate::container_toml::string_value(source, "metadata_binding_table_hash");
    let bindings = array_table_blocks(source, "metadata_binding")
        .into_iter()
        .filter_map(|block| {
            Some(MetadataBinding {
                binding_id: string_value_from_lines(&block, "binding_id")?,
                contract: string_value_from_lines(&block, "contract")?,
                value_count: usize_value_from_lines(&block, "value_count")?,
                value_hash: string_value_from_lines(&block, "value_hash")?,
                validation_status: string_value_from_lines(&block, "validation_status")?,
                required: bool_value_from_lines(&block, "required")?,
            })
        })
        .collect::<Vec<_>>();

    if declared_count.is_none() && table_hash.is_none() && bindings.is_empty() {
        return MetadataBindingSummary::not_applicable();
    }

    let actual_hash = metadata_binding_table_hash(&bindings);
    let mut blockers = Vec::new();
    if declared_count != Some(bindings.len()) {
        blockers.push(format!(
            "container-loader:metadata-binding-count-mismatch:expected={}:actual={}",
            declared_count
                .map(|count| count.to_string())
                .unwrap_or_else(|| "missing".to_owned()),
            bindings.len()
        ));
    }
    if table_hash.as_deref() != Some(actual_hash.as_str()) {
        blockers.push(format!(
            "container-loader:metadata-binding-table-hash-mismatch:expected={}:actual={actual_hash}",
            table_hash.as_deref().unwrap_or("missing")
        ));
    }

    let mut ids = BTreeSet::new();
    for binding in &bindings {
        if !ids.insert(binding.binding_id.as_str()) {
            blockers.push(format!(
                "container-loader:metadata-binding-duplicate:{}",
                binding.binding_id
            ));
        }
        if binding.required && binding.validation_status != "verified" {
            blockers.push(format!(
                "container-loader:metadata-binding-unverified:{}",
                binding.binding_id
            ));
        }
    }

    let selected = bindings
        .iter()
        .find(|binding| binding.binding_id == SELECTED_SET_BINDING_ID);
    let provider_dispatch = bindings
        .iter()
        .find(|binding| binding.binding_id == PROVIDER_DISPATCH_BINDING_ID);
    if selected.is_some_and(|binding| {
        binding.contract != SELECTED_SET_CONTRACT
            || binding.value_count == 0
            || !valid_fnv1a64(&binding.value_hash)
            || binding.validation_status != "verified"
            || !binding.required
    }) {
        blockers.push("container-loader:selected-provider-bundle-set-binding-invalid".to_owned());
    }
    if provider_dispatch.is_some_and(|binding| {
        binding.contract != PROVIDER_DISPATCH_CONTRACT
            || binding.value_count == 0
            || !valid_table_hash(&binding.value_hash)
            || binding.validation_status != "verified"
            || !binding.required
    }) {
        blockers.push("container-loader:provider-dispatch-binding-invalid".to_owned());
    }

    MetadataBindingSummary {
        declared_count,
        parsed_count: bindings.len(),
        table_hash,
        validation_status: if blockers.is_empty() {
            if bindings.is_empty() {
                "not-applicable"
            } else {
                "verified"
            }
        } else {
            "mismatch"
        }
        .to_owned(),
        selected_set_contract: selected.map(|binding| binding.contract.clone()),
        selected_set_count: selected.map(|binding| binding.value_count),
        selected_set_hash: selected.map(|binding| binding.value_hash.clone()),
        provider_dispatch_count: provider_dispatch.map(|binding| binding.value_count),
        provider_dispatch_hash: provider_dispatch.map(|binding| binding.value_hash.clone()),
        blockers,
    }
}

impl MetadataBindingSummary {
    pub(super) fn not_applicable() -> Self {
        Self {
            declared_count: None,
            parsed_count: 0,
            table_hash: None,
            validation_status: "not-applicable".to_owned(),
            selected_set_contract: None,
            selected_set_count: None,
            selected_set_hash: None,
            provider_dispatch_count: None,
            provider_dispatch_hash: None,
            blockers: Vec::new(),
        }
    }
}

fn metadata_binding_table_hash(bindings: &[MetadataBinding]) -> String {
    let mut material = String::new();
    for binding in bindings {
        material.push_str(&binding.binding_id);
        material.push('\t');
        material.push_str(&binding.contract);
        material.push('\t');
        material.push_str(&binding.value_count.to_string());
        material.push('\t');
        material.push_str(&binding.value_hash);
        material.push('\t');
        material.push_str(&binding.validation_status);
        material.push('\t');
        material.push_str(if binding.required { "true" } else { "false" });
        material.push('\n');
    }
    fnv1a64_hex(material.as_bytes())
}

fn valid_fnv1a64(value: &str) -> bool {
    value
        .strip_prefix("fnv1a64:")
        .is_some_and(|hex| hex.len() == 16 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

fn valid_table_hash(value: &str) -> bool {
    value
        .strip_prefix("0x")
        .is_some_and(|hex| hex.len() == 16 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn binding_source(value_hash: &str, table_hash: &str) -> String {
        format!(
            "metadata_binding_count = 1\nmetadata_binding_table_hash = \
             \"{table_hash}\"\n\n[[metadata_binding]]\nbinding_id = \
             \"identity.selected-provider-bundle-set\"\ncontract = \
             \"nuis-selected-provider-bundle-set-v1\"\nvalue_count = 2\nvalue_hash = \
             \"{value_hash}\"\nvalidation_status = \"verified\"\nrequired = true\n"
        )
    }

    #[test]
    fn verifies_selected_provider_binding_from_container_bytes() {
        let value_hash = "fnv1a64:1234567890abcdef";
        let binding = MetadataBinding {
            binding_id: SELECTED_SET_BINDING_ID.to_owned(),
            contract: SELECTED_SET_CONTRACT.to_owned(),
            value_count: 2,
            value_hash: value_hash.to_owned(),
            validation_status: "verified".to_owned(),
            required: true,
        };
        let table_hash = metadata_binding_table_hash(&[binding]);
        let summary = scan_metadata_bindings(&binding_source(value_hash, &table_hash));

        assert_eq!(summary.validation_status, "verified");
        assert!(summary.blockers.is_empty());
        assert_eq!(summary.declared_count, Some(1));
        assert_eq!(summary.parsed_count, 1);
        assert_eq!(summary.selected_set_count, Some(2));
        assert_eq!(summary.selected_set_hash.as_deref(), Some(value_hash));
    }

    #[test]
    fn rejects_length_preserving_selected_provider_binding_tamper() {
        let original_hash = "fnv1a64:1234567890abcdef";
        let binding = MetadataBinding {
            binding_id: SELECTED_SET_BINDING_ID.to_owned(),
            contract: SELECTED_SET_CONTRACT.to_owned(),
            value_count: 2,
            value_hash: original_hash.to_owned(),
            validation_status: "verified".to_owned(),
            required: true,
        };
        let table_hash = metadata_binding_table_hash(&[binding]);
        let source = binding_source("fnv1a64:fedcba0987654321", &table_hash);
        let summary = scan_metadata_bindings(&source);

        assert_eq!(summary.validation_status, "mismatch");
        assert!(summary
            .blockers
            .iter()
            .any(|blocker| blocker.contains("metadata-binding-table-hash-mismatch")));
    }
}
