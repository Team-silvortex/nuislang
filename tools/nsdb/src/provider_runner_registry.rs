use std::path::Path;

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
        "metal:apple-silicon-gpu" => {
            framework_probe_status("/System/Library/Frameworks/Metal.framework")
        }
        "coreml:apple-ane" => framework_probe_status("/System/Library/Frameworks/CoreML.framework"),
        _ => "real-device-candidate-unsupported",
    }
}

fn framework_probe_status(path: &str) -> &'static str {
    if Path::new(path).exists() {
        "real-device-candidate-available"
    } else {
        "real-device-candidate-unavailable"
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
