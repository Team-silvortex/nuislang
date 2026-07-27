#[cfg(unix)]
use crate::provider_execution_adapter::{
    ProviderExecutionAdapterRegistration, PROVIDER_EXECUTION_ADAPTER_REGISTRY_CONTRACT,
};
use crate::provider_runner_registry::{
    ProviderRunnerProfile, PROVIDER_RUNNER_PROFILE_REGISTRY_CONTRACT,
};

pub(crate) const PROVIDER_BUNDLE_REGISTRY_CONTRACT: &str = "nuis-provider-bundle-registry-v1";
pub(crate) const PROVIDER_BUNDLE_MANIFEST_CONTRACT: &str = "nuis-provider-bundle-manifest-v1";
pub(crate) const PROVIDER_BUNDLE_MANIFEST_ENTRY_CONTRACT: &str =
    nuisc::registry::NUSTAR_PROVIDER_BUNDLE_ENTRY_CONTRACT;
pub(crate) const SELECTED_PROVIDER_BUNDLE_SET_CONTRACT: &str =
    "nuis-selected-provider-bundle-set-v1";

#[derive(Clone, Copy)]
pub(crate) struct ProviderBundleRegistration {
    pub(crate) registry_contract: &'static str,
    pub(crate) bundle_id: &'static str,
    pub(crate) runner_profile: ProviderRunnerProfile,
    #[cfg(unix)]
    pub(crate) execution_adapter: ProviderExecutionAdapterRegistration,
}

pub(crate) struct ProviderBundleManifestEntry {
    pub(crate) package_id: &'static str,
    pub(crate) bundle_id: &'static str,
    pub(crate) provider_family: &'static str,
    pub(crate) runner_adapter_id: &'static str,
    pub(crate) adapter_kind: &'static str,
    pub(crate) rust_const: &'static str,
}

#[derive(Clone, Copy)]
pub(crate) struct ProviderBundleEvidence {
    pub(crate) registry_contract: &'static str,
    pub(crate) manifest_contract: &'static str,
    pub(crate) manifest_hash: &'static str,
    pub(crate) manifest_entry_count: usize,
    pub(crate) package_id: &'static str,
    pub(crate) bundle_id: &'static str,
}

pub(crate) struct SelectedProviderBundleSetEvidence {
    pub(crate) contract: &'static str,
    pub(crate) count: usize,
    pub(crate) hash: String,
}

include!(concat!(
    env!("OUT_DIR"),
    "/provider_bundle_registry_generated.rs"
));

pub(crate) fn provider_bundle_registrations() -> &'static [ProviderBundleRegistration] {
    PROVIDER_BUNDLE_REGISTRATIONS
}

pub(crate) fn select_provider_bundle_by_family(
    provider_family: &str,
) -> Option<&'static ProviderBundleRegistration> {
    if !provider_bundle_manifest_is_valid() {
        return None;
    }
    provider_bundle_manifest_entries()
        .find(|(entry, bundle)| {
            provider_bundle_entry_is_valid(entry, bundle)
                && entry.provider_family == provider_family
        })
        .map(|(_, bundle)| bundle)
}

pub(crate) fn provider_bundle_evidence(provider_family: &str) -> Option<ProviderBundleEvidence> {
    if !provider_bundle_manifest_is_valid() {
        return None;
    }
    provider_bundle_manifest_entries()
        .find(|(entry, bundle)| {
            provider_bundle_entry_is_valid(entry, bundle)
                && entry.provider_family == provider_family
        })
        .map(|(entry, _)| ProviderBundleEvidence {
            registry_contract: PROVIDER_BUNDLE_REGISTRY_CONTRACT,
            manifest_contract: PROVIDER_BUNDLE_MANIFEST_CONTRACT,
            manifest_hash: PROVIDER_BUNDLE_MANIFEST_HASH,
            manifest_entry_count: PROVIDER_BUNDLE_MANIFEST_ENTRY_COUNT,
            package_id: entry.package_id,
            bundle_id: entry.bundle_id,
        })
}

pub(crate) fn selected_provider_bundle_set_evidence<'a>(
    provider_families: impl IntoIterator<Item = &'a str>,
) -> Option<SelectedProviderBundleSetEvidence> {
    let mut selected = Vec::new();
    let mut seen_bundle_ids = std::collections::BTreeSet::new();
    for provider_family in provider_families {
        let bundle = provider_bundle_evidence(provider_family)?;
        if seen_bundle_ids.insert(bundle.bundle_id) {
            selected.push((bundle.package_id, bundle.bundle_id, provider_family));
        }
    }
    if selected.is_empty() {
        return None;
    }
    let mut canonical = format!("{SELECTED_PROVIDER_BUNDLE_SET_CONTRACT}\n");
    for (index, (package_id, bundle_id, provider_family)) in selected.iter().enumerate() {
        canonical.push_str(&format!(
            "{index}|{package_id}|{bundle_id}|{provider_family}\n"
        ));
    }
    Some(SelectedProviderBundleSetEvidence {
        contract: SELECTED_PROVIDER_BUNDLE_SET_CONTRACT,
        count: selected.len(),
        hash: format!("fnv1a64:{:016x}", fnv1a64(canonical.as_bytes())),
    })
}

pub(crate) fn append_provider_bundle_evidence(out: &mut String, provider_family: &str) {
    let Some(bundle) = provider_bundle_evidence(provider_family) else {
        return;
    };
    let push = crate::provider_sample_payload::push_toml_string;
    push(
        out,
        "provider_bundle_registry_contract",
        bundle.registry_contract,
    );
    push(
        out,
        "provider_bundle_manifest_contract",
        bundle.manifest_contract,
    );
    push(out, "provider_bundle_manifest_hash", bundle.manifest_hash);
    out.push_str(&format!(
        "provider_bundle_manifest_entry_count = {}\n",
        bundle.manifest_entry_count
    ));
    push(out, "provider_bundle_package_id", bundle.package_id);
    push(out, "provider_bundle_id", bundle.bundle_id);
}

#[cfg(unix)]
pub(crate) fn select_provider_bundle_by_adapter_kind(
    adapter_kind: &str,
) -> Option<&'static ProviderBundleRegistration> {
    if !provider_bundle_manifest_is_valid() {
        return None;
    }
    provider_bundle_manifest_entries()
        .find(|(entry, bundle)| {
            provider_bundle_entry_is_valid(entry, bundle) && entry.adapter_kind == adapter_kind
        })
        .map(|(_, bundle)| bundle)
}

fn provider_bundle_manifest_entries() -> impl Iterator<
    Item = (
        &'static ProviderBundleManifestEntry,
        &'static ProviderBundleRegistration,
    ),
> {
    PROVIDER_BUNDLE_MANIFEST_ENTRIES
        .iter()
        .zip(provider_bundle_registrations())
}

fn provider_bundle_entry_is_valid(
    entry: &ProviderBundleManifestEntry,
    bundle: &ProviderBundleRegistration,
) -> bool {
    !entry.package_id.is_empty()
        && !entry.rust_const.is_empty()
        && bundle.registry_contract == PROVIDER_BUNDLE_REGISTRY_CONTRACT
        && bundle.bundle_id == entry.bundle_id
        && bundle.runner_profile.registry_contract == PROVIDER_RUNNER_PROFILE_REGISTRY_CONTRACT
        && bundle.runner_profile.provider_family == entry.provider_family
        && bundle.runner_profile.available_adapter.adapter_id == entry.runner_adapter_id
        && bundle.runner_profile.available_adapter.kind == entry.adapter_kind
        && {
            #[cfg(unix)]
            {
                bundle.execution_adapter.registry_contract
                    == PROVIDER_EXECUTION_ADAPTER_REGISTRY_CONTRACT
                    && bundle.execution_adapter.adapter_kind == entry.adapter_kind
            }
            #[cfg(not(unix))]
            {
                true
            }
        }
}

fn provider_bundle_manifest_is_valid() -> bool {
    if PROVIDER_BUNDLE_MANIFEST_ENTRY_COUNT != PROVIDER_BUNDLE_REGISTRATIONS.len()
        || PROVIDER_BUNDLE_MANIFEST_ENTRY_COUNT != PROVIDER_BUNDLE_MANIFEST_ENTRIES.len()
        || PROVIDER_BUNDLE_MANIFEST_ENTRIES
            .windows(2)
            .any(|window| window[0].bundle_id >= window[1].bundle_id)
        || provider_bundle_manifest_entries()
            .any(|(entry, bundle)| !provider_bundle_entry_is_valid(entry, bundle))
    {
        return false;
    }
    provider_bundle_manifest_hash() == PROVIDER_BUNDLE_MANIFEST_HASH
}

fn provider_bundle_manifest_hash() -> String {
    let mut canonical = format!("{PROVIDER_BUNDLE_MANIFEST_CONTRACT}\n");
    for entry in PROVIDER_BUNDLE_MANIFEST_ENTRIES {
        canonical.push_str(&format!(
            "{PROVIDER_BUNDLE_MANIFEST_ENTRY_CONTRACT}|{}|{}|{}|{}|{}|{}\n",
            entry.package_id,
            entry.bundle_id,
            entry.provider_family,
            entry.runner_adapter_id,
            entry.adapter_kind,
            entry.rust_const,
        ));
    }
    format!("fnv1a64:{:016x}", fnv1a64(canonical.as_bytes()))
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_bundles_cross_bind_unique_runner_and_execution_registrations() {
        assert!(provider_bundle_manifest_is_valid());
        assert_eq!(
            provider_bundle_manifest_hash(),
            PROVIDER_BUNDLE_MANIFEST_HASH
        );
        let bundles = provider_bundle_registrations();
        assert!(bundles.len() >= 4);
        assert!(bundles.iter().all(|bundle| {
            bundle.registry_contract == PROVIDER_BUNDLE_REGISTRY_CONTRACT
                && bundle.runner_profile.registry_contract
                    == PROVIDER_RUNNER_PROFILE_REGISTRY_CONTRACT
                && !bundle.bundle_id.is_empty()
        }));
        #[cfg(unix)]
        assert!(bundles.iter().all(|bundle| {
            bundle.execution_adapter.registry_contract
                == PROVIDER_EXECUTION_ADAPTER_REGISTRY_CONTRACT
                && bundle.runner_profile.available_adapter.kind
                    == bundle.execution_adapter.adapter_kind
        }));

        let bundle_ids = bundles
            .iter()
            .map(|bundle| bundle.bundle_id)
            .collect::<std::collections::BTreeSet<_>>();
        let families = bundles
            .iter()
            .map(|bundle| bundle.runner_profile.provider_family)
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(bundle_ids.len(), bundles.len());
        assert_eq!(families.len(), bundles.len());
        #[cfg(unix)]
        {
            let adapter_kinds = bundles
                .iter()
                .map(|bundle| bundle.execution_adapter.adapter_kind)
                .collect::<std::collections::BTreeSet<_>>();
            assert_eq!(adapter_kinds.len(), bundles.len());
        }
    }

    #[test]
    fn selected_bundle_set_is_unique_and_first_occurrence_ordered() {
        let evidence = selected_provider_bundle_set_evidence([
            "coreml:apple-ane",
            "coreml:apple-ane",
            "metal:apple-silicon-gpu",
        ])
        .unwrap();
        let reversed =
            selected_provider_bundle_set_evidence(["metal:apple-silicon-gpu", "coreml:apple-ane"])
                .unwrap();

        assert_eq!(evidence.contract, SELECTED_PROVIDER_BUNDLE_SET_CONTRACT);
        assert_eq!(evidence.count, 2);
        assert_eq!(evidence.hash, "fnv1a64:0126ed9d38f1895f");
        assert_ne!(evidence.hash, reversed.hash);
    }

    #[test]
    fn cuda_bundle_is_manifest_owned_and_cross_bound() {
        let bundle = select_provider_bundle_by_family("cuda:nvidia-gpu").unwrap();
        let evidence = provider_bundle_evidence("cuda:nvidia-gpu").unwrap();

        assert_eq!(bundle.bundle_id, "cuda.nvidia-gpu.bundle.v1");
        assert_eq!(
            bundle.runner_profile.available_adapter.adapter_id,
            "cuda.nvidia-gpu.real-device"
        );
        assert_eq!(evidence.package_id, "official.kernel");
        assert_eq!(evidence.bundle_id, bundle.bundle_id);
        #[cfg(unix)]
        assert_eq!(
            bundle.execution_adapter.adapter_kind,
            "cuda-ptx-real-device-runner"
        );
    }
}
