#[cfg(unix)]
use crate::provider_execution_adapter::{
    ProviderExecutionAdapterRegistration, PROVIDER_EXECUTION_ADAPTER_REGISTRY_CONTRACT,
};
use crate::provider_runner_registry::{
    ProviderRunnerProfile, PROVIDER_RUNNER_PROFILE_REGISTRY_CONTRACT,
};

pub(crate) const PROVIDER_BUNDLE_REGISTRY_CONTRACT: &str = "nuis-provider-bundle-registry-v1";

#[derive(Clone, Copy)]
pub(crate) struct ProviderBundleRegistration {
    pub(crate) registry_contract: &'static str,
    pub(crate) bundle_id: &'static str,
    pub(crate) runner_profile: ProviderRunnerProfile,
    #[cfg(unix)]
    pub(crate) execution_adapter: ProviderExecutionAdapterRegistration,
}

pub(crate) fn provider_bundle_registrations() -> &'static [ProviderBundleRegistration] {
    &[
        crate::provider_runner_native::PROVIDER_BUNDLE,
        crate::provider_runner_metal::PROVIDER_BUNDLE,
        crate::provider_runner_coreml::PROVIDER_BUNDLE,
    ]
}

pub(crate) fn select_provider_bundle_by_family(
    provider_family: &str,
) -> Option<&'static ProviderBundleRegistration> {
    provider_bundle_registrations().iter().find(|bundle| {
        bundle.registry_contract == PROVIDER_BUNDLE_REGISTRY_CONTRACT
            && !bundle.bundle_id.is_empty()
            && bundle.runner_profile.registry_contract == PROVIDER_RUNNER_PROFILE_REGISTRY_CONTRACT
            && bundle.runner_profile.provider_family == provider_family
    })
}

#[cfg(unix)]
pub(crate) fn select_provider_bundle_by_adapter_kind(
    adapter_kind: &str,
) -> Option<&'static ProviderBundleRegistration> {
    provider_bundle_registrations().iter().find(|bundle| {
        bundle.registry_contract == PROVIDER_BUNDLE_REGISTRY_CONTRACT
            && !bundle.bundle_id.is_empty()
            && bundle.runner_profile.registry_contract == PROVIDER_RUNNER_PROFILE_REGISTRY_CONTRACT
            && bundle.execution_adapter.registry_contract
                == PROVIDER_EXECUTION_ADAPTER_REGISTRY_CONTRACT
            && bundle.execution_adapter.adapter_kind == adapter_kind
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_bundles_cross_bind_unique_runner_and_execution_registrations() {
        let bundles = provider_bundle_registrations();
        assert!(bundles.len() >= 3);
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
}
