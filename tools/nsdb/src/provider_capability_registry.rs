use std::collections::BTreeSet;

pub(crate) const PROVIDER_CAPABILITY_REGISTRY_CONTRACT: &str =
    "nuis-provider-capability-registry-v1";
pub(crate) const PROVIDER_CAPABILITY_MANIFEST_CONTRACT: &str =
    "nuis-provider-capability-manifest-v1";
pub(crate) const PROVIDER_CAPABILITY_RECORD_CONTRACT: &str =
    nuisc::registry::NUSTAR_PROVIDER_CAPABILITY_ENTRY_CONTRACT;
pub(crate) const PROVIDER_CAPABILITY_SELECTION_CONTRACT: &str =
    "nuis-provider-capability-selection-v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ProviderCapabilityManifestEntry {
    pub(crate) package_id: &'static str,
    pub(crate) provider_id: &'static str,
    pub(crate) bundle_id: &'static str,
    pub(crate) provider_family: &'static str,
    pub(crate) priority: u16,
    pub(crate) capabilities: &'static [&'static str],
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ProviderCapabilitySelectionEvidence {
    pub(crate) registry_contract: &'static str,
    pub(crate) manifest_contract: &'static str,
    pub(crate) manifest_hash: &'static str,
    pub(crate) manifest_entry_count: usize,
    pub(crate) record_contract: &'static str,
    pub(crate) selection_contract: &'static str,
    pub(crate) package_id: &'static str,
    pub(crate) provider_id: &'static str,
    pub(crate) bundle_id: &'static str,
    pub(crate) provider_family: &'static str,
    pub(crate) priority: u16,
    pub(crate) capabilities: String,
    pub(crate) requirements: String,
    pub(crate) selection_hash: String,
}

include!(concat!(
    env!("OUT_DIR"),
    "/provider_capability_registry_generated.rs"
));

pub(crate) fn select_provider_capability(
    provider_family: &str,
    required_capabilities: &[&str],
) -> Result<ProviderCapabilitySelectionEvidence, String> {
    if !provider_capability_manifest_is_valid() {
        return Err("provider-capability-selection:registry-invalid".to_owned());
    }
    let requirements = canonical_requirements(required_capabilities)?;
    let entry = select_best_provider(
        PROVIDER_CAPABILITY_MANIFEST_ENTRIES,
        provider_family,
        &requirements,
    )
    .ok_or_else(|| {
        format!(
            "provider-capability-selection:unsupported:{provider_family}:{}",
            requirements.join(",")
        )
    })?;
    let capabilities = entry.capabilities.join(",");
    let requirements = requirements.join(",");
    let canonical = format!(
        "{PROVIDER_CAPABILITY_SELECTION_CONTRACT}\n{}\n{}|{}|{}|{}|{}|{}\nrequirements|{}\n",
        PROVIDER_CAPABILITY_MANIFEST_HASH,
        entry.package_id,
        entry.provider_id,
        entry.bundle_id,
        entry.provider_family,
        entry.priority,
        capabilities,
        requirements,
    );
    Ok(ProviderCapabilitySelectionEvidence {
        registry_contract: PROVIDER_CAPABILITY_REGISTRY_CONTRACT,
        manifest_contract: PROVIDER_CAPABILITY_MANIFEST_CONTRACT,
        manifest_hash: PROVIDER_CAPABILITY_MANIFEST_HASH,
        manifest_entry_count: PROVIDER_CAPABILITY_MANIFEST_ENTRY_COUNT,
        record_contract: PROVIDER_CAPABILITY_RECORD_CONTRACT,
        selection_contract: PROVIDER_CAPABILITY_SELECTION_CONTRACT,
        package_id: entry.package_id,
        provider_id: entry.provider_id,
        bundle_id: entry.bundle_id,
        provider_family: entry.provider_family,
        priority: entry.priority,
        capabilities,
        requirements,
        selection_hash: format!("fnv1a64:{:016x}", fnv1a64(canonical.as_bytes())),
    })
}

pub(crate) fn append_provider_capability_evidence(out: &mut String, provider_family: &str) {
    let Ok(selection) = select_provider_capability(provider_family, &[]) else {
        return;
    };
    let push = crate::provider_sample_payload::push_toml_string;
    push(
        out,
        "provider_capability_registry_contract",
        selection.registry_contract,
    );
    push(
        out,
        "provider_capability_manifest_contract",
        selection.manifest_contract,
    );
    push(
        out,
        "provider_capability_manifest_hash",
        selection.manifest_hash,
    );
    out.push_str(&format!(
        "provider_capability_manifest_entry_count = {}\n",
        selection.manifest_entry_count
    ));
    push(
        out,
        "provider_capability_record_contract",
        selection.record_contract,
    );
    push(
        out,
        "provider_capability_selection_contract",
        selection.selection_contract,
    );
    push(out, "provider_capability_package_id", selection.package_id);
    push(
        out,
        "provider_capability_provider_id",
        selection.provider_id,
    );
    push(out, "provider_capability_bundle_id", selection.bundle_id);
    push(
        out,
        "provider_capability_provider_family",
        selection.provider_family,
    );
    out.push_str(&format!(
        "provider_capability_priority = {}\n",
        selection.priority
    ));
    push(out, "provider_capability_values", &selection.capabilities);
    push(
        out,
        "provider_capability_requirements",
        &selection.requirements,
    );
    push(
        out,
        "provider_capability_selection_hash",
        &selection.selection_hash,
    );
}

fn canonical_requirements(required_capabilities: &[&str]) -> Result<Vec<String>, String> {
    let mut requirements = required_capabilities
        .iter()
        .map(|value| (*value).to_owned())
        .collect::<Vec<_>>();
    if requirements.iter().any(|value| !is_identity(value)) {
        return Err("provider-capability-selection:invalid-requirement".to_owned());
    }
    requirements.sort();
    if requirements.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err("provider-capability-selection:duplicate-requirement".to_owned());
    }
    Ok(requirements)
}

fn select_best_provider<'a>(
    entries: &'a [ProviderCapabilityManifestEntry],
    provider_family: &str,
    requirements: &[String],
) -> Option<&'a ProviderCapabilityManifestEntry> {
    entries
        .iter()
        .filter(|entry| {
            entry.provider_family == provider_family
                && requirements
                    .iter()
                    .all(|required| entry.capabilities.binary_search(&required.as_str()).is_ok())
        })
        .min_by(|lhs, rhs| {
            rhs.priority
                .cmp(&lhs.priority)
                .then_with(|| lhs.provider_id.cmp(rhs.provider_id))
        })
}

fn provider_capability_manifest_is_valid() -> bool {
    if PROVIDER_CAPABILITY_MANIFEST_ENTRY_COUNT != PROVIDER_CAPABILITY_MANIFEST_ENTRIES.len()
        || PROVIDER_CAPABILITY_MANIFEST_ENTRIES
            .windows(2)
            .any(|pair| pair[0].provider_id >= pair[1].provider_id)
    {
        return false;
    }
    for entry in PROVIDER_CAPABILITY_MANIFEST_ENTRIES {
        let capabilities = entry.capabilities.iter().copied().collect::<BTreeSet<_>>();
        let bundle =
            crate::provider_bundle_registry::provider_bundle_evidence(entry.provider_family);
        if entry.priority == 0
            || !is_identity(entry.provider_id)
            || entry.capabilities.is_empty()
            || capabilities.len() != entry.capabilities.len()
            || !entry.capabilities.windows(2).all(|pair| pair[0] < pair[1])
            || entry.capabilities.iter().any(|value| !is_identity(value))
            || !bundle.is_some_and(|bundle| {
                bundle.package_id == entry.package_id && bundle.bundle_id == entry.bundle_id
            })
        {
            return false;
        }
    }
    provider_capability_manifest_hash() == PROVIDER_CAPABILITY_MANIFEST_HASH
}

fn provider_capability_manifest_hash() -> String {
    let mut canonical = format!("{PROVIDER_CAPABILITY_MANIFEST_CONTRACT}\n");
    for entry in PROVIDER_CAPABILITY_MANIFEST_ENTRIES {
        canonical.push_str(&format!(
            "{PROVIDER_CAPABILITY_RECORD_CONTRACT}|{}|{}|{}|{}|{}|{}\n",
            entry.package_id,
            entry.provider_id,
            entry.bundle_id,
            entry.provider_family,
            entry.priority,
            entry.capabilities.join(","),
        ));
    }
    format!("fnv1a64:{:016x}", fnv1a64(canonical.as_bytes()))
}

fn is_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 255
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b':' | b'_' | b'-'))
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

    const LOW: ProviderCapabilityManifestEntry = ProviderCapabilityManifestEntry {
        package_id: "test.data",
        provider_id: "data.cpu-memory.low.v1",
        bundle_id: "data.host.bundle.v1",
        provider_family: "data:host",
        priority: 10,
        capabilities: &["memory.cpu", "movement.copy"],
    };
    const HIGH: ProviderCapabilityManifestEntry = ProviderCapabilityManifestEntry {
        package_id: "test.data",
        provider_id: "data.cpu-memory.high.v1",
        bundle_id: "data.host.bundle.v1",
        provider_family: "data:host",
        priority: 20,
        capabilities: &["memory.cpu", "movement.copy"],
    };
    const HIGH_LATE: ProviderCapabilityManifestEntry = ProviderCapabilityManifestEntry {
        package_id: "test.data",
        provider_id: "data.cpu-memory.zeta.v1",
        bundle_id: "data.host.bundle.v1",
        provider_family: "data:host",
        priority: 20,
        capabilities: &["memory.cpu", "movement.copy"],
    };

    #[test]
    fn generated_inventory_selects_cpu_memory_reference_by_open_capabilities() {
        assert!(provider_capability_manifest_is_valid());
        let first = select_provider_capability(
            "data:host",
            &["movement.copy", "memory.cpu", "completion.verified"],
        )
        .unwrap();
        let reordered = select_provider_capability(
            "data:host",
            &["completion.verified", "memory.cpu", "movement.copy"],
        )
        .unwrap();

        assert_eq!(first.provider_id, "data.cpu-memory.reference.v1");
        assert_eq!(first.bundle_id, "data.host.bundle.v1");
        assert_eq!(first.manifest_hash, "fnv1a64:d9930501cc045ea0");
        assert_eq!(first.selection_hash, "fnv1a64:80daf599c694cf56");
        assert_eq!(first.selection_hash, reordered.selection_hash);
        assert_eq!(
            first.requirements,
            "completion.verified,memory.cpu,movement.copy"
        );
    }

    #[test]
    fn selector_is_registration_order_independent_and_priority_ranked() {
        let requirements = vec!["memory.cpu".to_owned()];
        let forward_entries = [LOW, HIGH_LATE, HIGH];
        let reverse_entries = [HIGH, HIGH_LATE, LOW];
        let forward = select_best_provider(&forward_entries, "data:host", &requirements).unwrap();
        let reverse = select_best_provider(&reverse_entries, "data:host", &requirements).unwrap();
        assert_eq!(forward.provider_id, HIGH.provider_id);
        assert_eq!(reverse.provider_id, HIGH.provider_id);
    }

    #[test]
    fn selector_rejects_duplicate_and_unsupported_requirements() {
        assert!(
            select_provider_capability("data:host", &["memory.cpu", "memory.cpu"])
                .unwrap_err()
                .contains("duplicate-requirement")
        );
        assert!(select_provider_capability("data:host", &["memory.quantum"])
            .unwrap_err()
            .contains("unsupported"));
    }
}
