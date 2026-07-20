use std::path::PathBuf;
use std::process::Command;

pub(crate) struct ProviderRunnerAdapter {
    pub(crate) adapter_id: &'static str,
    pub(crate) capability_status: &'static str,
    pub(crate) real_device_capable: bool,
    pub(crate) kind: &'static str,
    pub(crate) execution_mode: &'static str,
}

pub(crate) fn select_provider_runner_adapter(provider_family: &str) -> ProviderRunnerAdapter {
    let probe_status = provider_runner_real_device_probe_status(provider_family);
    match (provider_family, probe_status) {
        ("metal:apple-silicon-gpu", "real-device-candidate-available") => ProviderRunnerAdapter {
            adapter_id: "metal.apple-silicon-gpu.real-device",
            capability_status: "registered-real-device",
            real_device_capable: true,
            kind: "metal-real-device-runner",
            execution_mode: "real-device-provider-runner",
        },
        ("coreml:apple-ane", "real-device-candidate-available") => ProviderRunnerAdapter {
            adapter_id: "coreml.apple-ane.real-device",
            capability_status: "registered-real-device",
            real_device_capable: true,
            kind: "coreml-real-device-runner",
            execution_mode: "real-device-provider-runner",
        },
        ("metal:apple-silicon-gpu", _) => ProviderRunnerAdapter {
            adapter_id: "metal.apple-silicon-gpu.host-simulated",
            capability_status: "registered-host-simulated",
            real_device_capable: false,
            kind: "metal-host-simulated-runner",
            execution_mode: "host-simulated-provider-runner",
        },
        ("coreml:apple-ane", _) => ProviderRunnerAdapter {
            adapter_id: "coreml.apple-ane.host-simulated",
            capability_status: "registered-host-simulated",
            real_device_capable: false,
            kind: "coreml-host-simulated-runner",
            execution_mode: "host-simulated-provider-runner",
        },
        _ => ProviderRunnerAdapter {
            adapter_id: "generic.device.host-simulated",
            capability_status: "registered-host-simulated",
            real_device_capable: false,
            kind: "generic-host-simulated-runner",
            execution_mode: "host-simulated-provider-runner",
        },
    }
}

pub(crate) fn provider_runner_real_device_probe_status(provider_family: &str) -> &'static str {
    match provider_family {
        "metal:apple-silicon-gpu" => framework_probe_status("Metal.framework"),
        "coreml:apple-ane" => framework_probe_status("CoreML.framework"),
        _ => "real-device-candidate-unsupported",
    }
}

fn framework_probe_status(framework_name: &str) -> &'static str {
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
            xcode_root.join("Platforms").join("MacOSX.platform").join("Developer").join("SDKs").join(
                "MacOSX.sdk",
            )
            .join("System")
            .join("Library")
            .join("Frameworks"),
        );
    }
    roots
        .into_iter()
        .filter(|path| path.exists())
        .collect()
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
    let output = Command::new(command)
        .args(args)
        .output()
        .ok()?;
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
    use super::{provider_runner_real_device_probe_status, select_provider_runner_adapter};

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
}
