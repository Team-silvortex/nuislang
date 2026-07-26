use crate::artifact_host_runner::HostRunnerJsonSurface;

const SELECTED_SET_CONTRACT: &str = "nuis-selected-provider-bundle-set-v1";

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
        .is_some_and(valid_fnv1a64)
    {
        return Some("container-loader-metadata-binding:table-hash-invalid".to_owned());
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

fn valid_fnv1a64(value: &str) -> bool {
    value
        .strip_prefix("fnv1a64:")
        .is_some_and(|hex| hex.len() == 16 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
}
