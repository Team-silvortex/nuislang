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

#[cfg(any(target_os = "linux", test))]
pub(crate) const CUDA_DEVICE_SELECTION_REGISTRY_CONTRACT: &str =
    "nuis-cuda-device-selection-registry-v1";
#[cfg(any(target_os = "linux", test))]
pub(crate) const CUDA_DEVICE_SELECTION_CONTRACT: &str = "nuis-cuda-device-selection-v1";
#[cfg(any(target_os = "linux", test))]
pub(crate) const CUDA_DEVICE_INVENTORY_CONTRACT: &str = "nuis-cuda-device-inventory-v1";
#[cfg(any(target_os = "linux", test))]
pub(crate) const CUDA_CAPABILITY_RANKED_POLICY_CODE: u32 = 1;

#[cfg(any(target_os = "linux", test))]
pub(crate) struct CudaDeviceSelectionProfile {
    pub(crate) registry_contract: &'static str,
    pub(crate) selection_contract: &'static str,
    pub(crate) inventory_contract: &'static str,
    pub(crate) provider_family: &'static str,
    pub(crate) policy: &'static str,
    pub(crate) policy_code: u32,
    pub(crate) capability_query: &'static str,
}

#[cfg(any(target_os = "linux", test))]
pub(crate) const DEVICE_SELECTION_PROFILE: CudaDeviceSelectionProfile =
    CudaDeviceSelectionProfile {
        registry_contract: CUDA_DEVICE_SELECTION_REGISTRY_CONTRACT,
        selection_contract: CUDA_DEVICE_SELECTION_CONTRACT,
        inventory_contract: CUDA_DEVICE_INVENTORY_CONTRACT,
        provider_family: "cuda:nvidia-gpu",
        policy: "capability-ranked-lowest-ordinal",
        policy_code: CUDA_CAPABILITY_RANKED_POLICY_CODE,
        capability_query: "cuda-driver-device-compute-capability",
    };

#[cfg(any(target_os = "linux", test))]
pub(crate) fn validates_device_selection(
    policy_code: u32,
    minimum_compute_capability: u32,
) -> bool {
    DEVICE_SELECTION_PROFILE.registry_contract == CUDA_DEVICE_SELECTION_REGISTRY_CONTRACT
        && DEVICE_SELECTION_PROFILE.selection_contract == CUDA_DEVICE_SELECTION_CONTRACT
        && DEVICE_SELECTION_PROFILE.inventory_contract == CUDA_DEVICE_INVENTORY_CONTRACT
        && DEVICE_SELECTION_PROFILE.provider_family == RUNNER_PROFILE.provider_family
        && DEVICE_SELECTION_PROFILE.policy == "capability-ranked-lowest-ordinal"
        && DEVICE_SELECTION_PROFILE.policy_code == policy_code
        && DEVICE_SELECTION_PROFILE.capability_query == "cuda-driver-device-compute-capability"
        && minimum_compute_capability > 0
}

#[cfg(any(target_os = "linux", test))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CudaDeviceInventoryEntry {
    pub(crate) ordinal: u32,
    pub(crate) compute_capability: u32,
}

#[cfg(any(target_os = "linux", test))]
pub(crate) fn select_cuda_device(
    inventory: &[CudaDeviceInventoryEntry],
    policy_code: u32,
    minimum_compute_capability: u32,
) -> Option<CudaDeviceInventoryEntry> {
    if !validates_device_selection(policy_code, minimum_compute_capability) {
        return None;
    }
    let mut selected: Option<CudaDeviceInventoryEntry> = None;
    for (index, candidate) in inventory.iter().copied().enumerate() {
        if candidate.ordinal > i32::MAX as u32
            || candidate.compute_capability == 0
            || inventory[..index]
                .iter()
                .any(|entry| entry.ordinal == candidate.ordinal)
        {
            return None;
        }
        if candidate.compute_capability < minimum_compute_capability {
            continue;
        }
        if selected.is_none_or(|current| {
            candidate.compute_capability > current.compute_capability
                || (candidate.compute_capability == current.compute_capability
                    && candidate.ordinal < current.ordinal)
        }) {
            selected = Some(candidate);
        }
    }
    selected
}

#[cfg(target_os = "linux")]
const CUDA_PTX_DISPATCH_SOURCE: &str = include_str!("../provider-runners/cuda_ptx_dispatch.c");

#[cfg(target_os = "linux")]
pub(crate) fn prepare_cuda_worker_invocation(
    cache: &mut ProviderProcessAdapterCache,
) -> Result<ResolvedProviderProcessAdapter<'_>, String> {
    cache.resolve_c_with_libraries(
        "cuda-ptx-dispatch-adapter",
        CUDA_PTX_DISPATCH_SOURCE,
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
        assert!(validates_device_selection(
            CUDA_CAPABILITY_RANKED_POLICY_CODE,
            80
        ));
        assert!(!validates_device_selection(
            CUDA_CAPABILITY_RANKED_POLICY_CODE,
            0
        ));
        #[cfg(not(target_os = "linux"))]
        assert_eq!(
            (RUNNER_PROFILE.probe_status)(),
            "cuda-launch-candidate-unavailable"
        );
    }

    #[test]
    fn cuda_inventory_selection_is_capability_ranked_and_deterministic() {
        let inventory = [
            CudaDeviceInventoryEntry {
                ordinal: 2,
                compute_capability: 80,
            },
            CudaDeviceInventoryEntry {
                ordinal: 1,
                compute_capability: 89,
            },
            CudaDeviceInventoryEntry {
                ordinal: 0,
                compute_capability: 89,
            },
        ];
        assert_eq!(
            select_cuda_device(&inventory, CUDA_CAPABILITY_RANKED_POLICY_CODE, 80),
            Some(CudaDeviceInventoryEntry {
                ordinal: 0,
                compute_capability: 89,
            })
        );
        assert_eq!(
            select_cuda_device(&inventory, CUDA_CAPABILITY_RANKED_POLICY_CODE, 90),
            None
        );
        assert_eq!(select_cuda_device(&inventory, 99, 80), None);
    }

    #[test]
    fn cuda_inventory_rejects_duplicate_ordinals() {
        let inventory = [
            CudaDeviceInventoryEntry {
                ordinal: 0,
                compute_capability: 80,
            },
            CudaDeviceInventoryEntry {
                ordinal: 0,
                compute_capability: 89,
            },
        ];
        assert_eq!(
            select_cuda_device(&inventory, CUDA_CAPABILITY_RANKED_POLICY_CODE, 80),
            None
        );
    }
}
