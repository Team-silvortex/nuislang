use crate::{
    container_metadata_binding::MetadataBindingSummary,
    container_toml::{array_table_blocks, string_value, string_value_from_lines, usize_value},
    fnv1a64_hex,
};
use std::collections::BTreeSet;

const DISPATCH_CONTRACT: &str = "nuis-final-image-provider-dispatch-v1";
const SELECTED_SET_CONTRACT: &str = "nuis-selected-provider-bundle-set-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ProviderDispatchEntry {
    pub(super) dispatch_id: String,
    pub(super) package_id: String,
    pub(super) bundle_id: String,
    pub(super) provider_family: String,
    pub(super) runner_contract: String,
    pub(super) runner_adapter_contract: String,
    pub(super) runner_adapter_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ProviderDispatchSummary {
    pub(super) status: String,
    pub(super) declared_count: Option<usize>,
    pub(super) parsed_count: usize,
    pub(super) table_hash: Option<String>,
    pub(super) selected_set_hash: Option<String>,
    pub(super) entries: Vec<ProviderDispatchEntry>,
    pub(super) blockers: Vec<String>,
}

pub(super) fn scan_provider_dispatch(
    source: &str,
    metadata: &MetadataBindingSummary,
) -> ProviderDispatchSummary {
    let contract = string_value(source, "provider_dispatch_contract");
    let declared_status = string_value(source, "provider_dispatch_validation_status");
    let declared_count = usize_value(source, "provider_dispatch_count");
    let table_hash = string_value(source, "provider_dispatch_table_hash");
    let entries = parse_entries(source);
    if entries.is_empty() && metadata.selected_set_count.unwrap_or(0) == 0 {
        return ProviderDispatchSummary::not_applicable(declared_count, table_hash);
    }

    let actual_table_hash = dispatch_table_hash(&entries);
    let actual_selected_set_hash = selected_set_hash(&entries);
    let mut blockers = Vec::new();
    if contract.as_deref() != Some(DISPATCH_CONTRACT) {
        blockers.push("container-loader:provider-dispatch-contract-mismatch".to_owned());
    }
    if declared_status.as_deref() != Some("verified") {
        blockers.push("container-loader:provider-dispatch-status-unverified".to_owned());
    }
    if declared_count != Some(entries.len()) {
        blockers.push("container-loader:provider-dispatch-count-mismatch".to_owned());
    }
    if table_hash.as_deref() != Some(actual_table_hash.as_str()) {
        blockers.push("container-loader:provider-dispatch-table-hash-mismatch".to_owned());
    }
    if metadata.selected_set_count != Some(entries.len())
        || metadata.selected_set_hash.as_deref() != Some(actual_selected_set_hash.as_str())
    {
        blockers.push("container-loader:provider-dispatch-selected-set-mismatch".to_owned());
    }
    if metadata.provider_dispatch_count != Some(entries.len())
        || metadata.provider_dispatch_hash.as_deref() != Some(actual_table_hash.as_str())
    {
        blockers.push("container-loader:provider-dispatch-binding-mismatch".to_owned());
    }
    let mut bundle_ids = BTreeSet::new();
    for entry in &entries {
        if entry_incomplete(entry) || !bundle_ids.insert(entry.bundle_id.as_str()) {
            blockers.push(format!(
                "container-loader:provider-dispatch-entry-invalid:{}",
                entry.dispatch_id
            ));
        }
    }

    ProviderDispatchSummary {
        status: if blockers.is_empty() {
            "verified"
        } else {
            "mismatch"
        }
        .to_owned(),
        declared_count,
        parsed_count: entries.len(),
        table_hash,
        selected_set_hash: Some(actual_selected_set_hash),
        entries,
        blockers,
    }
}

impl ProviderDispatchSummary {
    pub(super) fn not_applicable(
        declared_count: Option<usize>,
        table_hash: Option<String>,
    ) -> Self {
        Self {
            status: "not-applicable".to_owned(),
            declared_count,
            parsed_count: 0,
            table_hash,
            selected_set_hash: None,
            entries: Vec::new(),
            blockers: Vec::new(),
        }
    }
}

fn parse_entries(source: &str) -> Vec<ProviderDispatchEntry> {
    array_table_blocks(source, "provider_dispatch")
        .into_iter()
        .filter_map(|block| {
            Some(ProviderDispatchEntry {
                dispatch_id: string_value_from_lines(&block, "dispatch_id")?,
                package_id: string_value_from_lines(&block, "provider_bundle_package_id")?,
                bundle_id: string_value_from_lines(&block, "provider_bundle_id")?,
                provider_family: string_value_from_lines(&block, "provider_family")?,
                runner_contract: string_value_from_lines(&block, "runner_contract")?,
                runner_adapter_contract: string_value_from_lines(
                    &block,
                    "runner_adapter_contract",
                )?,
                runner_adapter_id: string_value_from_lines(&block, "runner_adapter_id")?,
            })
        })
        .collect()
}

fn dispatch_table_hash(entries: &[ProviderDispatchEntry]) -> String {
    let mut canonical = format!("{DISPATCH_CONTRACT}\n");
    for entry in entries {
        canonical.push_str(&format!(
            "{}|{}|{}|{}|{}|{}|{}\n",
            entry.dispatch_id,
            entry.package_id,
            entry.bundle_id,
            entry.provider_family,
            entry.runner_contract,
            entry.runner_adapter_contract,
            entry.runner_adapter_id
        ));
    }
    fnv1a64_hex(canonical.as_bytes())
}

fn selected_set_hash(entries: &[ProviderDispatchEntry]) -> String {
    let mut canonical = format!("{SELECTED_SET_CONTRACT}\n");
    for (index, entry) in entries.iter().enumerate() {
        canonical.push_str(&format!(
            "{index}|{}|{}|{}\n",
            entry.package_id, entry.bundle_id, entry.provider_family
        ));
    }
    format!(
        "fnv1a64:{}",
        fnv1a64_hex(canonical.as_bytes()).trim_start_matches("0x")
    )
}

fn entry_incomplete(entry: &ProviderDispatchEntry) -> bool {
    [
        entry.dispatch_id.as_str(),
        entry.package_id.as_str(),
        entry.bundle_id.as_str(),
        entry.provider_family.as_str(),
        entry.runner_contract.as_str(),
        entry.runner_adapter_contract.as_str(),
        entry.runner_adapter_id.as_str(),
    ]
    .into_iter()
    .any(|value| value.is_empty() || value == "none")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(adapter_id: &str, table_hash: &str) -> String {
        format!(
            "provider_dispatch_contract = \"{DISPATCH_CONTRACT}\"\n\
             provider_dispatch_validation_status = \"verified\"\n\
             provider_dispatch_count = 1\n\
             provider_dispatch_table_hash = \"{table_hash}\"\n\
             [[provider_dispatch]]\n\
             dispatch_id = \"dispatch0000\"\n\
             provider_bundle_package_id = \"official.shader\"\n\
             provider_bundle_id = \"shader.bundle.v1\"\n\
             provider_family = \"shader:registered-gpu\"\n\
             runner_contract = \"nuis-provider-runner-v1\"\n\
             runner_adapter_contract = \"nuis-provider-runner-adapter-v1\"\n\
             runner_adapter_id = \"{adapter_id}\"\n"
        )
    }

    fn metadata(table_hash: &str) -> MetadataBindingSummary {
        MetadataBindingSummary {
            declared_count: Some(2),
            parsed_count: 2,
            table_hash: Some("0x1111111111111111".to_owned()),
            validation_status: "verified".to_owned(),
            selected_set_contract: Some(SELECTED_SET_CONTRACT.to_owned()),
            selected_set_count: Some(1),
            selected_set_hash: Some("fnv1a64:4dafc34057ecda12".to_owned()),
            provider_dispatch_count: Some(1),
            provider_dispatch_hash: Some(table_hash.to_owned()),
            blockers: Vec::new(),
        }
    }

    #[test]
    fn independently_verifies_final_image_dispatch() {
        let parsed = parse_entries(&source("adapter.shader", "pending"));
        let table_hash = dispatch_table_hash(&parsed);
        let selected_hash = selected_set_hash(&parsed);
        let mut metadata = metadata(&table_hash);
        metadata.selected_set_hash = Some(selected_hash);

        let summary = scan_provider_dispatch(&source("adapter.shader", &table_hash), &metadata);

        assert_eq!(summary.status, "verified");
        assert_eq!(summary.parsed_count, 1);
        assert_eq!(summary.entries[0].runner_adapter_id, "adapter.shader");
        assert!(summary.blockers.is_empty());
    }

    #[test]
    fn rejects_final_image_adapter_drift() {
        let original = parse_entries(&source("adapter.shader", "pending"));
        let table_hash = dispatch_table_hash(&original);
        let selected_hash = selected_set_hash(&original);
        let mut metadata = metadata(&table_hash);
        metadata.selected_set_hash = Some(selected_hash);

        let summary = scan_provider_dispatch(&source("adapter.driftt", &table_hash), &metadata);

        assert_eq!(summary.status, "mismatch");
        assert!(summary
            .blockers
            .contains(&"container-loader:provider-dispatch-table-hash-mismatch".to_owned()));
    }
}
