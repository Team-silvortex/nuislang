use crate::{
    model::NsdbDeviceProviderSampleRecordInfo,
    provider_sample_execution::{provider_execution_outcome, ProviderExecutionOutcome},
};

pub(crate) struct ProviderSampleRunner {
    pub(crate) contract: &'static str,
    pub(crate) adapter_contract: &'static str,
    pub(crate) adapter_id: &'static str,
    pub(crate) adapter_capability_status: &'static str,
    pub(crate) registry_protocol: &'static str,
    pub(crate) registry_source: &'static str,
    pub(crate) real_device_capable: bool,
    pub(crate) kind: &'static str,
    pub(crate) execution_mode: &'static str,
    pub(crate) backend: &'static str,
    pub(crate) device: &'static str,
}

pub(crate) fn provider_runner_for(
    record: &NsdbDeviceProviderSampleRecordInfo,
) -> ProviderSampleRunner {
    let adapter =
        crate::provider_runner_registry::select_provider_runner_adapter(&record.provider_family);
    match record.provider_family.as_str() {
        "metal:apple-silicon-gpu" => ProviderSampleRunner {
            contract: "nuis-provider-runner-v1",
            adapter_contract: "nuis-provider-runner-adapter-v1",
            adapter_id: adapter.adapter_id,
            adapter_capability_status: adapter.capability_status,
            registry_protocol: "nuis-provider-runner-registry-v1",
            registry_source: "builtin-nustar-provider-runner-registry",
            real_device_capable: adapter.real_device_capable,
            kind: adapter.kind,
            execution_mode: adapter.execution_mode,
            backend: "metal",
            device: "apple-silicon-gpu",
        },
        "coreml:apple-ane" => ProviderSampleRunner {
            contract: "nuis-provider-runner-v1",
            adapter_contract: "nuis-provider-runner-adapter-v1",
            adapter_id: adapter.adapter_id,
            adapter_capability_status: adapter.capability_status,
            registry_protocol: "nuis-provider-runner-registry-v1",
            registry_source: "builtin-nustar-provider-runner-registry",
            real_device_capable: adapter.real_device_capable,
            kind: adapter.kind,
            execution_mode: adapter.execution_mode,
            backend: "coreml",
            device: "apple-ane",
        },
        _ => ProviderSampleRunner {
            contract: "nuis-provider-runner-v1",
            adapter_contract: "nuis-provider-runner-adapter-v1",
            adapter_id: adapter.adapter_id,
            adapter_capability_status: adapter.capability_status,
            registry_protocol: "nuis-provider-runner-registry-v1",
            registry_source: "builtin-nustar-provider-runner-registry",
            real_device_capable: adapter.real_device_capable,
            kind: adapter.kind,
            execution_mode: adapter.execution_mode,
            backend: "generic",
            device: "generic-device",
        },
    }
}

pub(crate) fn provider_execution_for(
    record: &NsdbDeviceProviderSampleRecordInfo,
) -> ProviderExecutionOutcome {
    let adapter =
        crate::provider_runner_registry::select_provider_runner_adapter(&record.provider_family);
    provider_execution_outcome(&adapter)
}

pub(crate) fn provider_execution_outcome_for_runner(
    runner: &ProviderSampleRunner,
) -> ProviderExecutionOutcome {
    provider_execution_outcome(&crate::provider_runner_registry::ProviderRunnerAdapter {
        adapter_id: runner.adapter_id,
        capability_status: runner.adapter_capability_status,
        real_device_capable: runner.real_device_capable,
        kind: runner.kind,
        execution_mode: runner.execution_mode,
    })
}
