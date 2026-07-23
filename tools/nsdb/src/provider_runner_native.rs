use crate::provider_bundle_registry::{
    ProviderBundleRegistration, PROVIDER_BUNDLE_REGISTRY_CONTRACT,
};
use crate::provider_runner_registry::{
    ProviderRunnerAdapter, ProviderRunnerProfile, PROVIDER_RUNNER_PROFILE_REGISTRY_CONTRACT,
};

pub(crate) const PROVIDER_BUNDLE: ProviderBundleRegistration = ProviderBundleRegistration {
    registry_contract: PROVIDER_BUNDLE_REGISTRY_CONTRACT,
    bundle_id: "data.host.bundle.v1",
    runner_profile: RUNNER_PROFILE,
    #[cfg(unix)]
    execution_adapter: crate::provider_execution_native::REGISTRATION,
};

pub(crate) const RUNNER_PROFILE: ProviderRunnerProfile = ProviderRunnerProfile {
    registry_contract: PROVIDER_RUNNER_PROFILE_REGISTRY_CONTRACT,
    provider_family: "data:host",
    probe_status,
    available_probe_status: "native-provider-worker-available",
    available_adapter: ProviderRunnerAdapter {
        adapter_id: "data.host.provider-worker-native",
        capability_status: "registered-native-worker",
        real_device_capable: true,
        kind: "provider-worker-native-runner",
        execution_mode: "real-device-provider-runner",
    },
    fallback_adapter: ProviderRunnerAdapter {
        adapter_id: "generic.device.host-simulated",
        capability_status: "registered-host-simulated",
        real_device_capable: false,
        kind: "generic-host-simulated-runner",
        execution_mode: "host-simulated-provider-runner",
    },
};

fn probe_status() -> &'static str {
    if cfg!(unix) {
        "native-provider-worker-available"
    } else {
        "native-provider-worker-unavailable"
    }
}
