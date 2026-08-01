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

pub(crate) const VULKAN_HOST_PROBE_CONTRACT: &str = "nuis-vulkan-host-probe-v1";
pub(crate) const VULKAN_AVAILABLE_STATUS: &str = "vulkan-device-probe-available";
const VULKAN_UNAVAILABLE_STATUS: &str = "vulkan-device-probe-unavailable";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct VulkanHostProbeEvidence {
    pub(crate) contract: &'static str,
    pub(crate) status: &'static str,
    pub(crate) instance_api_version: u32,
    pub(crate) physical_device_count: u32,
}

pub(crate) const PROVIDER_BUNDLE: ProviderBundleRegistration = ProviderBundleRegistration {
    registry_contract: PROVIDER_BUNDLE_REGISTRY_CONTRACT,
    bundle_id: "spirv.vulkan-gpu.bundle.v1",
    runner_profile: RUNNER_PROFILE,
    #[cfg(unix)]
    execution_adapter: crate::provider_execution_vulkan::REGISTRATION,
};

pub(crate) const RUNNER_PROFILE: ProviderRunnerProfile = ProviderRunnerProfile {
    registry_contract: PROVIDER_RUNNER_PROFILE_REGISTRY_CONTRACT,
    provider_family: "spirv:vulkan-gpu",
    probe_status: vulkan_probe_status,
    available_probe_status: VULKAN_AVAILABLE_STATUS,
    available_adapter: ProviderRunnerAdapter {
        adapter_id: "spirv.vulkan.real-device",
        capability_status: "registered-real-device",
        real_device_capable: true,
        kind: "vulkan-spirv-real-device-runner",
        execution_mode: "real-device-provider-runner",
    },
    fallback_adapter: ProviderRunnerAdapter {
        adapter_id: "spirv.vulkan.host-unavailable",
        capability_status: "registered-host-unavailable",
        real_device_capable: false,
        kind: "vulkan-device-probe-runner",
        execution_mode: "unavailable-provider-runner",
    },
};

#[cfg(any(target_os = "linux", test))]
const VULKAN_SPIRV_DISPATCH_SOURCE: &str =
    include_str!("../provider-runners/vulkan_spirv_dispatch.c");

#[cfg(target_os = "linux")]
pub(crate) fn prepare_vulkan_worker_invocation(
    cache: &mut ProviderProcessAdapterCache,
) -> Result<ResolvedProviderProcessAdapter<'_>, String> {
    cache.resolve_c_with_libraries(
        "vulkan-spirv-dispatch-adapter",
        VULKAN_SPIRV_DISPATCH_SOURCE,
        "nuis-vulkan-spirv-provider-runner-v1",
        &["dl"],
    )
}

fn vulkan_probe_status() -> &'static str {
    probe_vulkan_host().status
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn probe_vulkan_host() -> VulkanHostProbeEvidence {
    unavailable_evidence()
}

#[cfg(target_os = "linux")]
pub(crate) fn probe_vulkan_host() -> VulkanHostProbeEvidence {
    linux::probe_vulkan_host()
}

fn unavailable_evidence() -> VulkanHostProbeEvidence {
    VulkanHostProbeEvidence {
        contract: VULKAN_HOST_PROBE_CONTRACT,
        status: VULKAN_UNAVAILABLE_STATUS,
        instance_api_version: 0,
        physical_device_count: 0,
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use super::{
        unavailable_evidence, VulkanHostProbeEvidence, VULKAN_AVAILABLE_STATUS,
        VULKAN_HOST_PROBE_CONTRACT,
    };
    use std::ffi::{c_char, c_void};

    const VK_SUCCESS: i32 = 0;
    const VK_STRUCTURE_TYPE_APPLICATION_INFO: u32 = 0;
    const VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO: u32 = 1;
    const VK_API_VERSION_1_0: u32 = 1 << 22;

    type VkInstance = *mut c_void;
    type EnumerateInstanceVersion = unsafe extern "system" fn(*mut u32) -> i32;
    type CreateInstance = unsafe extern "system" fn(
        *const VkInstanceCreateInfo,
        *const c_void,
        *mut VkInstance,
    ) -> i32;
    type EnumeratePhysicalDevices =
        unsafe extern "system" fn(VkInstance, *mut u32, *mut *mut c_void) -> i32;
    type DestroyInstance = unsafe extern "system" fn(VkInstance, *const c_void);

    #[repr(C)]
    struct VkApplicationInfo {
        s_type: u32,
        p_next: *const c_void,
        p_application_name: *const c_char,
        application_version: u32,
        p_engine_name: *const c_char,
        engine_version: u32,
        api_version: u32,
    }

    #[repr(C)]
    struct VkInstanceCreateInfo {
        s_type: u32,
        p_next: *const c_void,
        flags: u32,
        p_application_info: *const VkApplicationInfo,
        enabled_layer_count: u32,
        pp_enabled_layer_names: *const *const c_char,
        enabled_extension_count: u32,
        pp_enabled_extension_names: *const *const c_char,
    }

    struct VulkanLibrary(*mut c_void);

    impl VulkanLibrary {
        fn open() -> Option<Self> {
            let handle = unsafe {
                libc::dlopen(
                    c"libvulkan.so.1".as_ptr(),
                    libc::RTLD_NOW | libc::RTLD_LOCAL,
                )
            };
            (!handle.is_null()).then_some(Self(handle))
        }

        unsafe fn symbol<T: Copy>(&self, name: &'static std::ffi::CStr) -> Option<T> {
            let pointer = unsafe { libc::dlsym(self.0, name.as_ptr()) };
            (!pointer.is_null()).then(|| unsafe { std::mem::transmute_copy(&pointer) })
        }
    }

    impl Drop for VulkanLibrary {
        fn drop(&mut self) {
            unsafe {
                libc::dlclose(self.0);
            }
        }
    }

    struct Instance {
        raw: VkInstance,
        destroy: DestroyInstance,
    }

    impl Drop for Instance {
        fn drop(&mut self) {
            unsafe {
                (self.destroy)(self.raw, std::ptr::null());
            }
        }
    }

    pub(super) fn probe_vulkan_host() -> VulkanHostProbeEvidence {
        let Some(library) = VulkanLibrary::open() else {
            return unavailable_evidence();
        };
        let Some(create_instance) =
            (unsafe { library.symbol::<CreateInstance>(c"vkCreateInstance") })
        else {
            return unavailable_evidence();
        };
        let Some(enumerate_devices) =
            (unsafe { library.symbol::<EnumeratePhysicalDevices>(c"vkEnumeratePhysicalDevices") })
        else {
            return unavailable_evidence();
        };
        let Some(destroy_instance) =
            (unsafe { library.symbol::<DestroyInstance>(c"vkDestroyInstance") })
        else {
            return unavailable_evidence();
        };
        let api_version = unsafe {
            library
                .symbol::<EnumerateInstanceVersion>(c"vkEnumerateInstanceVersion")
                .and_then(|enumerate| {
                    let mut version = VK_API_VERSION_1_0;
                    (enumerate)(&mut version).eq(&VK_SUCCESS).then_some(version)
                })
                .unwrap_or(VK_API_VERSION_1_0)
        };
        let application = VkApplicationInfo {
            s_type: VK_STRUCTURE_TYPE_APPLICATION_INFO,
            p_next: std::ptr::null(),
            p_application_name: c"nuis-nsdb-vulkan-probe".as_ptr(),
            application_version: 1,
            p_engine_name: c"nuis".as_ptr(),
            engine_version: 1,
            api_version,
        };
        let create_info = VkInstanceCreateInfo {
            s_type: VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: 0,
            p_application_info: &application,
            enabled_layer_count: 0,
            pp_enabled_layer_names: std::ptr::null(),
            enabled_extension_count: 0,
            pp_enabled_extension_names: std::ptr::null(),
        };
        let mut raw_instance = std::ptr::null_mut();
        if unsafe { create_instance(&create_info, std::ptr::null(), &mut raw_instance) }
            != VK_SUCCESS
            || raw_instance.is_null()
        {
            return unavailable_evidence();
        }
        let instance = Instance {
            raw: raw_instance,
            destroy: destroy_instance,
        };
        let mut device_count = 0;
        if unsafe { enumerate_devices(instance.raw, &mut device_count, std::ptr::null_mut()) }
            != VK_SUCCESS
            || device_count == 0
        {
            return unavailable_evidence();
        }
        VulkanHostProbeEvidence {
            contract: VULKAN_HOST_PROBE_CONTRACT,
            status: VULKAN_AVAILABLE_STATUS,
            instance_api_version: api_version,
            physical_device_count: device_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vulkan_bundle_is_static_and_real_device_when_probe_available() {
        assert_eq!(PROVIDER_BUNDLE.bundle_id, "spirv.vulkan-gpu.bundle.v1");
        assert_eq!(RUNNER_PROFILE.provider_family, "spirv:vulkan-gpu");
        assert_eq!(
            RUNNER_PROFILE.available_adapter.kind,
            "vulkan-spirv-real-device-runner"
        );
        assert!(RUNNER_PROFILE.available_adapter.real_device_capable);
        assert_eq!(
            RUNNER_PROFILE.available_adapter.execution_mode,
            "real-device-provider-runner"
        );
    }

    #[test]
    fn vulkan_probe_reports_a_consistent_bounded_shape() {
        let evidence = probe_vulkan_host();
        assert_eq!(evidence.contract, VULKAN_HOST_PROBE_CONTRACT);
        assert_eq!(evidence.status, (RUNNER_PROFILE.probe_status)());
        if evidence.status == VULKAN_AVAILABLE_STATUS {
            assert!(evidence.instance_api_version >= 1 << 22);
            assert!(evidence.physical_device_count > 0);
        } else {
            assert_eq!(evidence.instance_api_version, 0);
            assert_eq!(evidence.physical_device_count, 0);
        }
        if std::env::var("NUIS_REQUIRE_VULKAN_DEVICE_PROBE").as_deref() == Ok("1") {
            assert_eq!(evidence.status, VULKAN_AVAILABLE_STATUS);
            assert!(evidence.physical_device_count > 0);
        }
    }

    #[test]
    fn vulkan_dispatch_source_keeps_dynamic_one_or_two_input_descriptor_abi() {
        for required in [
            "argc != 5 && argc != 6",
            "int input_count = argc == 6 ? 2 : 1",
            "uint32_t descriptor_count = (uint32_t)(input_count + 1)",
            "VkDescriptorSetLayoutBinding bindings[3]",
            "VkBuffer buffers[3]",
            "VkDeviceMemory memories[3]",
            "valid_spirv_entry",
        ] {
            assert!(VULKAN_SPIRV_DISPATCH_SOURCE.contains(required));
        }
    }
}
