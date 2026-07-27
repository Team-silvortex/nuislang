use crate::provider_bundle_registry::{
    ProviderBundleRegistration, PROVIDER_BUNDLE_REGISTRY_CONTRACT,
};
#[cfg(target_os = "linux")]
use crate::provider_process_adapter::{
    ProviderProcessAdapterCache, ResolvedProviderProcessAdapter,
};
use crate::provider_runner_registry::{
    ProviderRunnerAdapter, ProviderRunnerProfile, PROVIDER_RUNNER_PROFILE_REGISTRY_CONTRACT,
};
#[cfg(target_os = "linux")]
use std::path::Path;

#[cfg(target_os = "linux")]
const CUDA_VECTOR_ADD_SOURCE: &str = include_str!("../provider-runners/cuda_vector_add.c");

#[cfg(target_os = "linux")]
pub(crate) fn prepare_vector_add_worker_invocation(
    cache: &mut ProviderProcessAdapterCache,
) -> Result<ResolvedProviderProcessAdapter<'_>, String> {
    cache.resolve_c_with_libraries(
        "cuda-vector-add-adapter",
        CUDA_VECTOR_ADD_SOURCE,
        "nuis-cuda-ptx-driver-provider-runner-v1",
        &["dl"],
    )
}

pub(crate) const PROVIDER_BUNDLE: ProviderBundleRegistration = ProviderBundleRegistration {
    registry_contract: PROVIDER_BUNDLE_REGISTRY_CONTRACT,
    bundle_id: "cuda.nvidia-gpu.bundle.v1",
    runner_profile: RUNNER_PROFILE,
    #[cfg(unix)]
    execution_adapter: crate::provider_execution_cuda::REGISTRATION,
};

pub(crate) const RUNNER_PROFILE: ProviderRunnerProfile = ProviderRunnerProfile {
    registry_contract: PROVIDER_RUNNER_PROFILE_REGISTRY_CONTRACT,
    provider_family: "cuda:nvidia-gpu",
    probe_status: cuda_probe_status,
    available_probe_status: "cuda-launch-candidate-available",
    available_adapter: ProviderRunnerAdapter {
        adapter_id: "cuda.nvidia-gpu.real-device",
        capability_status: "registered-real-device",
        real_device_capable: true,
        kind: "cuda-ptx-real-device-runner",
        execution_mode: "real-device-provider-runner",
    },
    fallback_adapter: ProviderRunnerAdapter {
        adapter_id: "cuda.nvidia-gpu.host-simulated",
        capability_status: "registered-host-simulated",
        real_device_capable: false,
        kind: "cuda-host-simulated-runner",
        execution_mode: "host-simulated-provider-runner",
    },
};

#[cfg(not(target_os = "linux"))]
fn cuda_probe_status() -> &'static str {
    "cuda-launch-candidate-unavailable"
}

#[cfg(target_os = "linux")]
fn cuda_probe_status() -> &'static str {
    if !Path::new("/dev/nvidiactl").exists() || !cuda_driver_library_available() {
        return "cuda-launch-candidate-unavailable";
    }
    "cuda-launch-candidate-available"
}

#[cfg(target_os = "linux")]
fn cuda_driver_library_available() -> bool {
    let handle =
        unsafe { libc::dlopen(c"libcuda.so.1".as_ptr(), libc::RTLD_NOW | libc::RTLD_LOCAL) };
    if handle.is_null() {
        return false;
    }
    unsafe {
        libc::dlclose(handle);
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cuda_profile_is_registered_without_claiming_cross_platform_availability() {
        assert_eq!(PROVIDER_BUNDLE.bundle_id, "cuda.nvidia-gpu.bundle.v1");
        assert_eq!(RUNNER_PROFILE.provider_family, "cuda:nvidia-gpu");
        assert_eq!(
            RUNNER_PROFILE.available_adapter.kind,
            "cuda-ptx-real-device-runner"
        );
        #[cfg(not(target_os = "linux"))]
        assert_eq!(
            (RUNNER_PROFILE.probe_status)(),
            "cuda-launch-candidate-unavailable"
        );
    }
}
