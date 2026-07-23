use std::path::PathBuf;
use std::process::Command;

use crate::provider_worker_descriptor_capability::{
    ProviderWorkerDescriptorCapability, ProviderWorkerOutputDescriptorCapability,
    PROVIDER_WORKER_DESCRIPTOR_CAPABILITY_CONTRACT,
    PROVIDER_WORKER_OUTPUT_DESCRIPTOR_CAPABILITY_CONTRACT,
};

pub(crate) const PROVIDER_WORKER_IMAGE_REGISTRY_CONTRACT: &str =
    "nuis-provider-worker-image-registry-v1";
pub(crate) const PROVIDER_WORKER_IMAGE_REGISTRY_SOURCE: &str =
    "builtin-nustar-provider-worker-image-registry";
pub(crate) const PROVIDER_WORKER_OPERATION_REGISTRY_CONTRACT: &str =
    "nuis-provider-worker-operation-registry-v1";
pub(crate) const PROVIDER_RUNNER_PROFILE_REGISTRY_CONTRACT: &str =
    "nuis-provider-runner-profile-registry-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderWorkerImageRegistration {
    pub(crate) registry_contract: &'static str,
    pub(crate) registry_source: &'static str,
    pub(crate) image_id: &'static str,
    pub(crate) source_path: &'static str,
    pub(crate) cache_identity: &'static str,
    pub(crate) provider_key: i64,
    pub(crate) capability_hash: i64,
    pub(crate) descriptor_capability: ProviderWorkerDescriptorCapability,
    pub(crate) output_descriptor_capability: ProviderWorkerOutputDescriptorCapability,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderWorkerOperationRegistration {
    pub(crate) registry_contract: &'static str,
    pub(crate) adapter_id: String,
    pub(crate) operation: String,
    pub(crate) operation_token: String,
}

#[derive(Clone, Copy)]
pub(crate) struct ProviderRunnerAdapter {
    pub(crate) adapter_id: &'static str,
    pub(crate) capability_status: &'static str,
    pub(crate) real_device_capable: bool,
    pub(crate) kind: &'static str,
    pub(crate) execution_mode: &'static str,
}

#[derive(Clone, Copy)]
pub(crate) struct ProviderRunnerProfile {
    pub(crate) registry_contract: &'static str,
    pub(crate) provider_family: &'static str,
    pub(crate) probe_status: fn() -> &'static str,
    pub(crate) available_probe_status: &'static str,
    pub(crate) available_adapter: ProviderRunnerAdapter,
    pub(crate) fallback_adapter: ProviderRunnerAdapter,
}

const GENERIC_HOST_SIMULATED_ADAPTER: ProviderRunnerAdapter = ProviderRunnerAdapter {
    adapter_id: "generic.device.host-simulated",
    capability_status: "registered-host-simulated",
    real_device_capable: false,
    kind: "generic-host-simulated-runner",
    execution_mode: "host-simulated-provider-runner",
};

pub(crate) fn select_provider_runner_adapter(provider_family: &str) -> ProviderRunnerAdapter {
    let Some(profile) = select_provider_runner_profile(provider_family) else {
        return GENERIC_HOST_SIMULATED_ADAPTER;
    };
    if (profile.probe_status)() == profile.available_probe_status {
        profile.available_adapter
    } else {
        profile.fallback_adapter
    }
}

fn select_provider_runner_profile(provider_family: &str) -> Option<&'static ProviderRunnerProfile> {
    crate::provider_bundle_registry::select_provider_bundle_by_family(provider_family)
        .map(|bundle| &bundle.runner_profile)
}

pub(crate) fn select_provider_worker_image_registration(
    provider_family: &str,
) -> Option<ProviderWorkerImageRegistration> {
    let (domain, backend) = provider_family.split_once(':')?;
    if domain.is_empty() || backend.is_empty() {
        return None;
    }
    Some(ProviderWorkerImageRegistration {
        registry_contract: PROVIDER_WORKER_IMAGE_REGISTRY_CONTRACT,
        registry_source: PROVIDER_WORKER_IMAGE_REGISTRY_SOURCE,
        image_id: "std.provider-worker.unix.v1",
        source_path: "stdlib/std/provider_worker_image.ns",
        cache_identity: "std.provider-worker.unix.aot-v27",
        provider_key: stable_registration_scalar(provider_family.as_bytes()),
        capability_hash: stable_registration_scalar(
            format!("{provider_family}:provider-worker-capability-v1").as_bytes(),
        ),
        descriptor_capability: ProviderWorkerDescriptorCapability {
            contract: PROVIDER_WORKER_DESCRIPTOR_CAPABILITY_CONTRACT,
            max_semantic_descriptors: 31,
            max_control_descriptors: 1,
        },
        output_descriptor_capability: ProviderWorkerOutputDescriptorCapability {
            contract: PROVIDER_WORKER_OUTPUT_DESCRIPTOR_CAPABILITY_CONTRACT,
            max_output_descriptors: 8,
        },
    })
}

pub(crate) fn select_provider_worker_operation_registration(
    provider_family: &str,
    adapter_id: &str,
    operation: &str,
) -> Option<ProviderWorkerOperationRegistration> {
    if provider_family.split_once(':').is_none()
        || !is_registration_token(adapter_id)
        || !is_registration_token(operation)
    {
        return None;
    }
    let identity = format!("{provider_family}:{adapter_id}:{operation}");
    Some(ProviderWorkerOperationRegistration {
        registry_contract: PROVIDER_WORKER_OPERATION_REGISTRY_CONTRACT,
        adapter_id: adapter_id.to_owned(),
        operation: operation.to_owned(),
        operation_token: format!(
            "operation:{}",
            stable_registration_scalar(identity.as_bytes())
        ),
    })
}

fn is_registration_token(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b':' | b'_' | b'-'))
}

fn stable_registration_scalar(bytes: &[u8]) -> i64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    i64::try_from(hash & i64::MAX as u64)
        .unwrap_or(i64::MAX)
        .max(1)
}

pub(crate) fn provider_runner_real_device_probe_status(provider_family: &str) -> &'static str {
    select_provider_runner_profile(provider_family)
        .map(|profile| (profile.probe_status)())
        .unwrap_or("real-device-candidate-unsupported")
}

pub(crate) fn framework_probe_status(framework_name: &str) -> &'static str {
    if has_framework(framework_name) {
        "real-device-candidate-available"
    } else {
        "real-device-candidate-unavailable"
    }
}

#[cfg(not(target_os = "macos"))]
fn has_framework(_framework_name: &str) -> bool {
    false
}

#[cfg(target_os = "macos")]
fn has_framework(framework_name: &str) -> bool {
    framework_roots()
        .into_iter()
        .any(|root| root.join(framework_name).exists())
}

#[cfg(target_os = "macos")]
fn framework_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(sdk_root) = std::env::var("SDKROOT") {
        let path = PathBuf::from(sdk_root);
        if path.exists() {
            roots.push(path.join("System/Library/Frameworks"));
        }
    }
    if let Some(sdk_root) = framework_sdk_root() {
        roots.push(sdk_root.join("System/Library/Frameworks"));
    }
    if let Some(xcode_root) = xcode_select_root() {
        roots.push(
            xcode_root
                .join("Platforms")
                .join("MacOSX.platform")
                .join("Developer")
                .join("SDKs")
                .join("MacOSX.sdk")
                .join("System")
                .join("Library")
                .join("Frameworks"),
        );
    }
    roots.into_iter().filter(|path| path.exists()).collect()
}

#[cfg(target_os = "macos")]
fn framework_sdk_root() -> Option<PathBuf> {
    command_output_trimmed("xcrun", &["--sdk", "macosx", "--show-sdk-path"]).and_then(|output| {
        let path = PathBuf::from(output);
        if path.exists() {
            Some(path)
        } else {
            None
        }
    })
}

#[cfg(target_os = "macos")]
fn xcode_select_root() -> Option<PathBuf> {
    command_output_trimmed("xcode-select", &["-p"]).and_then(|output| {
        let path = PathBuf::from(output);
        if path.exists() {
            Some(path)
        } else {
            None
        }
    })
}

#[cfg(target_os = "macos")]
fn command_output_trimmed(command: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(command).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let trimmed = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        provider_runner_real_device_probe_status, select_provider_runner_adapter,
        select_provider_worker_image_registration, select_provider_worker_operation_registration,
        PROVIDER_RUNNER_PROFILE_REGISTRY_CONTRACT, PROVIDER_WORKER_IMAGE_REGISTRY_CONTRACT,
    };

    #[test]
    fn runner_profiles_are_unique_contract_bound_static_contributions() {
        let profiles = crate::provider_bundle_registry::provider_bundle_registrations()
            .iter()
            .map(|bundle| &bundle.runner_profile)
            .collect::<Vec<_>>();
        assert!(profiles.len() >= 3);
        assert!(profiles.iter().all(|profile| {
            profile.registry_contract == PROVIDER_RUNNER_PROFILE_REGISTRY_CONTRACT
        }));
        let families = profiles
            .iter()
            .map(|profile| profile.provider_family)
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(families.len(), profiles.len());
    }

    #[test]
    fn runner_selector_contains_no_provider_specific_match() {
        let source = include_str!("provider_runner_registry.rs");
        let selector = source
            .split_once("pub(crate) fn select_provider_runner_adapter")
            .and_then(|(_, tail)| tail.split_once("fn select_provider_runner_profile"))
            .map(|(selector, _)| selector)
            .expect("runner selector source");
        assert!(!selector.contains("match"));
        assert!(!selector.contains("data:host"));
        assert!(!selector.contains("metal:apple-silicon-gpu"));
        assert!(!selector.contains("coreml:apple-ane"));
    }

    #[test]
    fn reports_unknown_provider_family_as_unsupported() {
        assert_eq!(
            provider_runner_real_device_probe_status("spirv:vulkan-gpu"),
            "real-device-candidate-unsupported"
        );
    }

    #[test]
    fn unknown_provider_family_uses_host_simulated_fallback() {
        let adapter = select_provider_runner_adapter("spirv:vulkan-gpu");
        assert_eq!(adapter.adapter_id, "generic.device.host-simulated");
        assert_eq!(adapter.capability_status, "registered-host-simulated");
        assert!(!adapter.real_device_capable);
    }

    #[test]
    fn worker_image_registration_is_open_ended_and_provider_bound() {
        let first = select_provider_worker_image_registration("spirv:vulkan-gpu")
            .expect("generic provider registration");
        let repeated = select_provider_worker_image_registration("spirv:vulkan-gpu")
            .expect("stable provider registration");
        let other = select_provider_worker_image_registration("kernel:cpu-avx2")
            .expect("other provider registration");

        assert_eq!(
            first.registry_contract,
            PROVIDER_WORKER_IMAGE_REGISTRY_CONTRACT
        );
        assert_eq!(first, repeated);
        assert_ne!(first.provider_key, other.provider_key);
        assert_ne!(first.capability_hash, other.capability_hash);
        assert_eq!(first.source_path, other.source_path);
        assert_eq!(first.cache_identity, other.cache_identity);
    }

    #[test]
    fn worker_operation_registration_is_open_ended_and_identity_bound() {
        let first = select_provider_worker_operation_registration(
            "spirv:vulkan-gpu",
            "spirv.vulkan.real-device",
            "convolve",
        )
        .expect("operation");
        let repeated = select_provider_worker_operation_registration(
            "spirv:vulkan-gpu",
            "spirv.vulkan.real-device",
            "convolve",
        )
        .expect("operation");
        let other = select_provider_worker_operation_registration(
            "spirv:vulkan-gpu",
            "spirv.vulkan.real-device",
            "reduce",
        )
        .expect("other operation");
        assert_eq!(first, repeated);
        assert_ne!(first.operation_token, other.operation_token);
        assert!(select_provider_worker_operation_registration(
            "spirv:vulkan-gpu",
            "spirv/vulkan",
            "convolve"
        )
        .is_none());
    }
}
