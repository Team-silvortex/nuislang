use super::{container, container_verify, fnv1a64_hex, toml};
use std::collections::BTreeSet;

const SELECTED_SET_BINDING_ID: &str = "identity.selected-provider-bundle-set";
const SELECTED_SET_CONTRACT: &str = "nuis-selected-provider-bundle-set-v1";
const BINDING_TABLE_MARKER: &[u8] = b"\n[[metadata_binding]]\n";
const NEXT_TABLE_MARKER: &[u8] = b"\n[[loader_symbol]]\n";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FinalExecutableContainerBindingEvidence {
    pub(crate) count: Option<usize>,
    pub(crate) table_hash: Option<String>,
    pub(crate) validation_status: String,
    pub(crate) selected_set_contract: Option<String>,
    pub(crate) selected_set_count: Option<usize>,
    pub(crate) selected_set_hash: Option<String>,
    pub(crate) blockers: Vec<String>,
}

pub(crate) fn container_binding_evidence(
    payload: &[u8],
    top_level: &str,
) -> FinalExecutableContainerBindingEvidence {
    let count = toml::usize_value(top_level, "metadata_binding_count");
    let table_hash = toml::string_value(top_level, "metadata_binding_table_hash");
    let table_source = metadata_binding_table_source(payload, count);
    let bindings = table_source
        .as_deref()
        .map(container_verify::metadata_binding_entries)
        .unwrap_or_default();
    let actual_hash = container::metadata_binding_table_hash(&bindings, fnv1a64_hex);
    let mut blockers = Vec::new();
    if count != Some(bindings.len()) {
        blockers.push(format!(
            "container-loader:metadata-binding-count-mismatch:expected={}:actual={}",
            count
                .map(|value| value.to_string())
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
    let mut binding_ids = BTreeSet::new();
    for binding in &bindings {
        if !binding_ids.insert(binding.binding_id.as_str()) {
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
    if let Some(binding) = selected {
        if binding.contract != SELECTED_SET_CONTRACT
            || binding.value_count == 0
            || !valid_fnv1a64(&binding.value_hash)
            || binding.validation_status != "verified"
            || !binding.required
        {
            blockers
                .push("container-loader:selected-provider-bundle-set-binding-invalid".to_owned());
        }
    }
    let validation_status = if blockers.is_empty() {
        if bindings.is_empty() {
            "not-applicable"
        } else {
            "verified"
        }
    } else {
        "mismatch"
    }
    .to_owned();

    FinalExecutableContainerBindingEvidence {
        count,
        table_hash,
        validation_status,
        selected_set_contract: selected.map(|binding| binding.contract.clone()),
        selected_set_count: selected.map(|binding| binding.value_count),
        selected_set_hash: selected.map(|binding| binding.value_hash.clone()),
        blockers,
    }
}

fn metadata_binding_table_source(payload: &[u8], count: Option<usize>) -> Option<String> {
    if count == Some(0) {
        return Some(String::new());
    }
    let start = find_bytes(payload, BINDING_TABLE_MARKER)?;
    let tail = &payload[start..];
    let end = find_bytes(tail, NEXT_TABLE_MARKER)
        .or_else(|| tail.iter().position(|byte| *byte == 0))
        .unwrap_or(tail.len());
    std::str::from_utf8(&tail[..end]).ok().map(str::to_owned)
}

fn valid_fnv1a64(value: &str) -> bool {
    value
        .strip_prefix("fnv1a64:")
        .is_some_and(|hex| hex.len() == 16 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}
