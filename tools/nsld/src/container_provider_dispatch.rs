use super::{
    container::NsldContainerProviderDispatch,
    final_executable_provider_sample::nsld_device_provider_sample_evidence, fnv1a64_hex, toml,
};
use std::{collections::BTreeSet, fs, path::Path};

pub(crate) const PROVIDER_DISPATCH_CONTRACT: &str = "nuis-final-image-provider-dispatch-v1";
pub(crate) const PROVIDER_DISPATCH_BINDING_ID: &str = "runtime.provider-dispatch-table";
const PROVIDER_SAMPLE_FILE_NAME: &str = "nuis.nsdb.device-provider-samples.toml";
const SELECTED_SET_CONTRACT: &str = "nuis-selected-provider-bundle-set-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldProviderDispatchEvidence {
    pub(crate) contract: String,
    pub(crate) status: String,
    pub(crate) table_hash: String,
    pub(crate) selected_set_hash: Option<String>,
    pub(crate) entries: Vec<NsldContainerProviderDispatch>,
    pub(crate) blockers: Vec<String>,
}

impl NsldProviderDispatchEvidence {
    pub(crate) fn not_applicable() -> Self {
        Self {
            contract: PROVIDER_DISPATCH_CONTRACT.to_owned(),
            status: "not-applicable".to_owned(),
            table_hash: provider_dispatch_table_hash(&[]),
            selected_set_hash: None,
            entries: Vec::new(),
            blockers: Vec::new(),
        }
    }
}

pub(crate) fn provider_dispatch_evidence(output_dir: &str) -> NsldProviderDispatchEvidence {
    let provider = nsld_device_provider_sample_evidence(output_dir);
    if !provider.available || provider.record_count == 0 {
        return NsldProviderDispatchEvidence::not_applicable();
    }
    // Selection identity can be bound before materialization completes. Runtime
    // dispatch becomes immutable only once every selected provider is ready.
    if provider.status != "ready" {
        return NsldProviderDispatchEvidence::not_applicable();
    }
    let path = Path::new(output_dir).join(PROVIDER_SAMPLE_FILE_NAME);
    let Ok(source) = fs::read_to_string(&path) else {
        return blocked("provider-dispatch:provider-sample-manifest-unreadable");
    };
    let mut seen_bundle_ids = BTreeSet::new();
    let entries = parse_dispatch_entries(&source)
        .into_iter()
        .filter(|entry| seen_bundle_ids.insert(entry.bundle_id.clone()))
        .collect::<Vec<_>>();
    let selected_set_hash = selected_provider_set_hash(&entries);
    let mut blockers = Vec::new();
    if entries.len() != provider.selected_provider_bundle_count.unwrap_or_default() {
        blockers.push("provider-dispatch:selected-set-count-mismatch".to_owned());
    }
    if provider.selected_provider_bundle_set_contract.as_deref() != Some(SELECTED_SET_CONTRACT) {
        blockers.push("provider-dispatch:selected-set-contract-mismatch".to_owned());
    }
    if provider.selected_provider_bundle_set_hash.as_deref() != Some(selected_set_hash.as_str()) {
        blockers.push("provider-dispatch:selected-set-hash-mismatch".to_owned());
    }
    for entry in &entries {
        if dispatch_entry_incomplete(entry) {
            blockers.push(format!(
                "provider-dispatch:entry-incomplete:{}",
                entry.dispatch_id
            ));
        }
    }
    build_evidence(entries, Some(selected_set_hash), blockers)
}

pub(crate) fn provider_dispatch_from_container(
    source: &str,
    selected_set_count: Option<usize>,
    selected_set_hash: Option<&str>,
    dispatch_binding_count: Option<usize>,
    dispatch_binding_hash: Option<&str>,
) -> NsldProviderDispatchEvidence {
    let declared_contract = toml::string_value(source, "provider_dispatch_contract");
    let declared_count = toml::usize_value(source, "provider_dispatch_count");
    let declared_table_hash = toml::string_value(source, "provider_dispatch_table_hash");
    let declared_status = toml::string_value(source, "provider_dispatch_validation_status");
    let entries = parse_dispatch_entries(source);
    if declared_count == Some(0) && entries.is_empty() {
        return NsldProviderDispatchEvidence::not_applicable();
    }
    let actual_table_hash = provider_dispatch_table_hash(&entries);
    let actual_selected_hash = selected_provider_set_hash(&entries);
    let mut blockers = Vec::new();
    if declared_contract.as_deref() != Some(PROVIDER_DISPATCH_CONTRACT) {
        blockers.push("container-loader:provider-dispatch-contract-mismatch".to_owned());
    }
    if declared_count != Some(entries.len()) {
        blockers.push("container-loader:provider-dispatch-count-mismatch".to_owned());
    }
    if declared_table_hash.as_deref() != Some(actual_table_hash.as_str()) {
        blockers.push("container-loader:provider-dispatch-table-hash-mismatch".to_owned());
    }
    if declared_status.as_deref() != Some("verified") {
        blockers.push("container-loader:provider-dispatch-status-unverified".to_owned());
    }
    if selected_set_count != Some(entries.len())
        || selected_set_hash != Some(actual_selected_hash.as_str())
    {
        blockers.push("container-loader:provider-dispatch-selected-set-mismatch".to_owned());
    }
    if dispatch_binding_count != Some(entries.len())
        || dispatch_binding_hash != Some(actual_table_hash.as_str())
    {
        blockers.push("container-loader:provider-dispatch-binding-mismatch".to_owned());
    }
    let mut bundle_ids = BTreeSet::new();
    for entry in &entries {
        if dispatch_entry_incomplete(entry) || !bundle_ids.insert(entry.bundle_id.as_str()) {
            blockers.push(format!(
                "container-loader:provider-dispatch-entry-invalid:{}",
                entry.dispatch_id
            ));
        }
    }
    build_evidence(entries, Some(actual_selected_hash), blockers)
}

pub(crate) fn provider_dispatch_table_hash(entries: &[NsldContainerProviderDispatch]) -> String {
    let mut canonical = format!("{PROVIDER_DISPATCH_CONTRACT}\n");
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

fn selected_provider_set_hash(entries: &[NsldContainerProviderDispatch]) -> String {
    let mut canonical = format!("{SELECTED_SET_CONTRACT}\n");
    for (index, entry) in entries.iter().enumerate() {
        canonical.push_str(&format!(
            "{index}|{}|{}|{}\n",
            entry.package_id, entry.bundle_id, entry.provider_family
        ));
    }
    format!(
        "fnv1a64:{}",
        provider_dispatch_table_hash_bytes(canonical.as_bytes())
    )
}

fn provider_dispatch_table_hash_bytes(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn parse_dispatch_entries(source: &str) -> Vec<NsldContainerProviderDispatch> {
    table_blocks(source, "device_provider_samples")
        .into_iter()
        .chain(table_blocks(source, "provider_dispatch"))
        .enumerate()
        .filter_map(|(index, block)| {
            let package_id = toml::string_value(&block, "provider_bundle_package_id")?;
            let bundle_id = toml::string_value(&block, "provider_bundle_id")?;
            let provider_family = toml::string_value(&block, "provider_family")?;
            Some(NsldContainerProviderDispatch {
                dispatch_id: toml::string_value(&block, "dispatch_id")
                    .unwrap_or_else(|| format!("dispatch{index:04}")),
                package_id,
                bundle_id,
                provider_family,
                runner_contract: toml::string_value(&block, "provider_runner_contract")
                    .or_else(|| toml::string_value(&block, "requested_runner_contract"))
                    .or_else(|| toml::string_value(&block, "runner_contract"))
                    .unwrap_or_else(|| "none".to_owned()),
                runner_adapter_contract: toml::string_value(
                    &block,
                    "provider_runner_adapter_contract",
                )
                .or_else(|| toml::string_value(&block, "requested_runner_adapter_contract"))
                .or_else(|| toml::string_value(&block, "runner_adapter_contract"))
                .unwrap_or_else(|| "none".to_owned()),
                runner_adapter_id: toml::string_value(&block, "provider_runner_adapter_id")
                    .or_else(|| toml::string_value(&block, "requested_runner_adapter_id"))
                    .or_else(|| toml::string_value(&block, "runner_adapter_id"))
                    .unwrap_or_else(|| "none".to_owned()),
            })
        })
        .collect()
}

fn table_blocks(source: &str, table: &str) -> Vec<String> {
    let header = format!("[[{table}]]");
    let mut blocks = Vec::new();
    let mut current = String::new();
    let mut active = false;
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("[[") && trimmed.ends_with("]]") {
            if active && !current.is_empty() {
                blocks.push(std::mem::take(&mut current));
            }
            active = trimmed == header;
            continue;
        }
        if active {
            current.push_str(line);
            current.push('\n');
        }
    }
    if active && !current.is_empty() {
        blocks.push(current);
    }
    blocks
}

fn dispatch_entry_incomplete(entry: &NsldContainerProviderDispatch) -> bool {
    [
        entry.package_id.as_str(),
        entry.bundle_id.as_str(),
        entry.provider_family.as_str(),
        entry.runner_contract.as_str(),
        entry.runner_adapter_contract.as_str(),
        entry.runner_adapter_id.as_str(),
    ]
    .iter()
    .any(|value| value.is_empty() || *value == "none")
}

fn build_evidence(
    entries: Vec<NsldContainerProviderDispatch>,
    selected_set_hash: Option<String>,
    blockers: Vec<String>,
) -> NsldProviderDispatchEvidence {
    let table_hash = provider_dispatch_table_hash(&entries);
    NsldProviderDispatchEvidence {
        contract: PROVIDER_DISPATCH_CONTRACT.to_owned(),
        status: if blockers.is_empty() {
            "verified"
        } else {
            "mismatch"
        }
        .to_owned(),
        table_hash,
        selected_set_hash,
        entries,
        blockers,
    }
}

fn blocked(blocker: &str) -> NsldProviderDispatchEvidence {
    build_evidence(Vec::new(), None, vec![blocker.to_owned()])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn dispatch_entries() -> Vec<NsldContainerProviderDispatch> {
        vec![
            NsldContainerProviderDispatch {
                dispatch_id: "dispatch0000".to_owned(),
                package_id: "official.shader".to_owned(),
                bundle_id: "bundle.shader.metal".to_owned(),
                provider_family: "metal:registered-gpu".to_owned(),
                runner_contract: "nuis-provider-runner-v1".to_owned(),
                runner_adapter_contract: "nuis-provider-runner-adapter-v1".to_owned(),
                runner_adapter_id: "adapter.shader.metal".to_owned(),
            },
            NsldContainerProviderDispatch {
                dispatch_id: "dispatch0001".to_owned(),
                package_id: "official.kernel".to_owned(),
                bundle_id: "bundle.kernel.compute".to_owned(),
                provider_family: "kernel:registered-accelerator".to_owned(),
                runner_contract: "nuis-provider-runner-v1".to_owned(),
                runner_adapter_contract: "nuis-provider-runner-adapter-v1".to_owned(),
                runner_adapter_id: "adapter.kernel.compute".to_owned(),
            },
        ]
    }

    fn render_dispatch_records(entries: &[NsldContainerProviderDispatch], table: &str) -> String {
        let mut source = format!(
            "provider_dispatch_contract = \"{PROVIDER_DISPATCH_CONTRACT}\"\n\
             provider_dispatch_validation_status = \"verified\"\n\
             provider_dispatch_count = {}\n\
             provider_dispatch_table_hash = \"{table}\"\n",
            entries.len()
        );
        for entry in entries {
            source.push_str(&format!(
                "\n[[provider_dispatch]]\n\
                 dispatch_id = \"{}\"\n\
                 provider_bundle_package_id = \"{}\"\n\
                 provider_bundle_id = \"{}\"\n\
                 provider_family = \"{}\"\n\
                 runner_contract = \"{}\"\n\
                 runner_adapter_contract = \"{}\"\n\
                 runner_adapter_id = \"{}\"\n",
                entry.dispatch_id,
                entry.package_id,
                entry.bundle_id,
                entry.provider_family,
                entry.runner_contract,
                entry.runner_adapter_contract,
                entry.runner_adapter_id
            ));
        }
        source
    }

    #[test]
    fn final_image_dispatch_table_cross_checks_selected_set_and_binding() {
        let entries = dispatch_entries();
        let table_hash = provider_dispatch_table_hash(&entries);
        let selected_hash = selected_provider_set_hash(&entries);
        let source = render_dispatch_records(&entries, &table_hash);

        let evidence = provider_dispatch_from_container(
            &source,
            Some(entries.len()),
            Some(&selected_hash),
            Some(entries.len()),
            Some(&table_hash),
        );

        assert_eq!(evidence.status, "verified");
        assert_eq!(evidence.entries, entries);
        assert_eq!(evidence.table_hash, table_hash);
        assert_eq!(
            evidence.selected_set_hash.as_deref(),
            Some(selected_hash.as_str())
        );
        assert!(evidence.blockers.is_empty());
    }

    #[test]
    fn final_image_dispatch_table_rejects_adapter_identity_tamper() {
        let entries = dispatch_entries();
        let table_hash = provider_dispatch_table_hash(&entries);
        let selected_hash = selected_provider_set_hash(&entries);
        let source = render_dispatch_records(&entries, &table_hash)
            .replace("adapter.shader.metal", "adapter.shader.drift");

        let evidence = provider_dispatch_from_container(
            &source,
            Some(entries.len()),
            Some(&selected_hash),
            Some(entries.len()),
            Some(&table_hash),
        );

        assert_eq!(evidence.status, "mismatch");
        assert!(evidence
            .blockers
            .contains(&"container-loader:provider-dispatch-table-hash-mismatch".to_owned()));
        assert!(evidence
            .blockers
            .contains(&"container-loader:provider-dispatch-binding-mismatch".to_owned()));
    }

    #[test]
    fn ready_provider_manifest_seals_open_dispatch_entries() {
        let dir = env::temp_dir().join(format!(
            "nsld-provider-dispatch-source-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let entries = dispatch_entries();
        let selected_hash = selected_provider_set_hash(&entries);
        let mut source = format!(
            "protocol = \"nuis-device-provider-samples-v1\"\n\
             schema = \"nsdb-yir-device-provider-sample-v1\"\n\
             status = \"ready\"\n\
             record_count = 2\n\
             ready_record_count = 2\n\
             pending_record_count = 0\n\
             provider_bundle_registry_contract = \"nuis-provider-bundle-registry-v1\"\n\
             provider_bundle_manifest_contract = \"nuis-provider-bundle-manifest-v1\"\n\
             provider_bundle_manifest_hash = \"fnv1a64:1111111111111111\"\n\
             provider_bundle_manifest_entry_count = 2\n\
             selected_provider_bundle_set_contract = \"{SELECTED_SET_CONTRACT}\"\n\
             selected_provider_bundle_count = 2\n\
             selected_provider_bundle_set_hash = \"{selected_hash}\"\n"
        );
        for (index, entry) in entries.iter().enumerate() {
            source.push_str(&format!(
                "\n[[device_provider_samples]]\n\
                 trace_id = \"trace-{index}\"\n\
                 provider = \"registered-provider\"\n\
                 provider_family = \"{}\"\n\
                 provider_bundle_package_id = \"{}\"\n\
                 provider_bundle_id = \"{}\"\n\
                 requested_runner_contract = \"{}\"\n\
                 requested_runner_adapter_contract = \"{}\"\n\
                 requested_runner_adapter_id = \"{}\"\n\
                 materialization_status = \"provider-sample-materialized\"\n",
                entry.provider_family,
                entry.package_id,
                entry.bundle_id,
                entry.runner_contract,
                entry.runner_adapter_contract,
                entry.runner_adapter_id
            ));
        }
        fs::write(dir.join(PROVIDER_SAMPLE_FILE_NAME), source).unwrap();

        let evidence = provider_dispatch_evidence(&dir.display().to_string());

        assert_eq!(evidence.status, "verified");
        assert_eq!(evidence.entries, entries);
        assert_eq!(
            evidence.selected_set_hash.as_deref(),
            Some(selected_hash.as_str())
        );
        assert!(evidence.blockers.is_empty());
        fs::remove_dir_all(dir).unwrap();
    }
}
