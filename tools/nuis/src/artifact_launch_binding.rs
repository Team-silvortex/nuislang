use crate::artifact_host_runner::HostRunnerJsonSurface;

const SELECTED_SET_CONTRACT: &str = "nuis-selected-provider-bundle-set-v1";
pub(crate) const FINAL_IMAGE_BINDING_PROOF_CONTRACT: &str = "nuis-final-image-binding-proof-v1";

#[derive(Clone)]
pub(crate) struct FinalImageBindingProof {
    pub(crate) binding_count: usize,
    pub(crate) binding_table_hash: String,
    pub(crate) validation_status: String,
    pub(crate) selected_set_contract: Option<String>,
    pub(crate) selected_set_count: Option<usize>,
    pub(crate) selected_set_hash: Option<String>,
}

impl FinalImageBindingProof {
    pub(crate) fn from_host_runner(host_runner: &HostRunnerJsonSurface) -> Option<Self> {
        Some(Self {
            binding_count: host_runner.container_loader_metadata_binding_count?,
            binding_table_hash: host_runner
                .container_loader_metadata_binding_table_hash
                .clone()?,
            validation_status: host_runner
                .container_loader_metadata_binding_validation_status
                .clone()?,
            selected_set_contract: host_runner
                .container_loader_selected_provider_bundle_set_contract
                .clone(),
            selected_set_count: host_runner.container_loader_selected_provider_bundle_count,
            selected_set_hash: host_runner
                .container_loader_selected_provider_bundle_set_hash
                .clone(),
        })
    }

    pub(crate) fn proof_hash(&self) -> String {
        let material = format!(
            "{FINAL_IMAGE_BINDING_PROOF_CONTRACT}\0{}\0{}\0{}\0{}\0{}\0{}",
            self.binding_count,
            self.binding_table_hash,
            self.validation_status,
            self.selected_set_contract.as_deref().unwrap_or("none"),
            self.selected_set_count
                .map(|count| count.to_string())
                .unwrap_or_else(|| "none".to_owned()),
            self.selected_set_hash.as_deref().unwrap_or("none"),
        );
        fnv1a64_hex(material.as_bytes())
    }
}

pub(crate) fn host_runner_binding_first_blocker(
    host_runner: &HostRunnerJsonSurface,
) -> Option<String> {
    let Some(count) = host_runner.container_loader_metadata_binding_count else {
        return Some("container-loader-metadata-binding:count-missing".to_owned());
    };
    if host_runner.container_loader_metadata_binding_parsed_count != Some(count) {
        return Some("container-loader-metadata-binding:count-mismatch".to_owned());
    }
    if count == 0 {
        return (host_runner
            .container_loader_metadata_binding_validation_status
            .as_deref()
            != Some("not-applicable"))
        .then(|| "container-loader-metadata-binding:empty-status-mismatch".to_owned());
    }
    if host_runner
        .container_loader_metadata_binding_validation_status
        .as_deref()
        != Some("verified")
    {
        return Some("container-loader-metadata-binding:not-verified".to_owned());
    }
    if !host_runner
        .container_loader_metadata_binding_table_hash
        .as_deref()
        .is_some_and(valid_nsld_table_hash)
    {
        return Some("container-loader-metadata-binding:table-hash-invalid".to_owned());
    }
    let provider_selection_present = host_runner
        .container_loader_selected_provider_bundle_set_contract
        .is_some()
        || host_runner
            .container_loader_selected_provider_bundle_count
            .is_some()
        || host_runner
            .container_loader_selected_provider_bundle_set_hash
            .is_some();
    if !provider_selection_present {
        return None;
    }
    if host_runner
        .container_loader_selected_provider_bundle_set_contract
        .as_deref()
        != Some(SELECTED_SET_CONTRACT)
    {
        return Some("selected-provider-bundle-set:contract-mismatch".to_owned());
    }
    if host_runner
        .container_loader_selected_provider_bundle_count
        .is_none_or(|value| value == 0)
    {
        return Some("selected-provider-bundle-set:count-invalid".to_owned());
    }
    if !host_runner
        .container_loader_selected_provider_bundle_set_hash
        .as_deref()
        .is_some_and(valid_fnv1a64)
    {
        return Some("selected-provider-bundle-set:hash-invalid".to_owned());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_only_binding_table_does_not_require_provider_selection() {
        let mut host_runner = HostRunnerJsonSurface::not_invoked("test");
        host_runner.container_loader_metadata_binding_count = Some(2);
        host_runner.container_loader_metadata_binding_parsed_count = Some(2);
        host_runner.container_loader_metadata_binding_table_hash =
            Some("0x1111111111111111".to_owned());
        host_runner.container_loader_metadata_binding_validation_status =
            Some("verified".to_owned());

        assert_eq!(host_runner_binding_first_blocker(&host_runner), None);
    }
}

fn valid_fnv1a64(value: &str) -> bool {
    value
        .strip_prefix("fnv1a64:")
        .is_some_and(|hex| hex.len() == 16 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

fn valid_nsld_table_hash(value: &str) -> bool {
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
    format!("fnv1a64:{hash:016x}")
}
