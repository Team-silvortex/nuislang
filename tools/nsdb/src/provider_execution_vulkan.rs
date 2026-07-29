use crate::{
    provider_execution_adapter::{
        ProviderExecutionAdapterRegistration, ProviderRequestExecution,
        PROVIDER_EXECUTION_ADAPTER_REGISTRY_CONTRACT,
    },
    provider_prepared_input::PreparedProviderInput,
    provider_request::ProviderRequest,
    provider_worker_lease::ProviderWorkerDispatchReceipt,
};
use std::path::Path;

pub(crate) const REGISTRATION: ProviderExecutionAdapterRegistration =
    ProviderExecutionAdapterRegistration {
        registry_contract: PROVIDER_EXECUTION_ADAPTER_REGISTRY_CONTRACT,
        adapter_kind: "vulkan-device-probe-runner",
        requires_worker_descriptors: false,
        prepare_worker_adapter: None,
        execute,
    };

fn execute(
    _input_evidence: &str,
    _provider_family: &str,
    _output_dir: &Path,
    _request: &ProviderRequest,
    _inputs: &[PreparedProviderInput],
    _worker_receipt: &mut ProviderWorkerDispatchReceipt,
) -> Result<ProviderRequestExecution, String> {
    Err(
        "Vulkan provider is probe-only until a registered SPIR-V asset and execution adapter exist"
            .to_owned(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vulkan_execution_registration_fails_closed_at_probe_boundary() {
        assert_eq!(
            REGISTRATION.registry_contract,
            PROVIDER_EXECUTION_ADAPTER_REGISTRY_CONTRACT
        );
        assert_eq!(REGISTRATION.adapter_kind, "vulkan-device-probe-runner");
        assert!(!REGISTRATION.requires_worker_descriptors);
        assert!(REGISTRATION.prepare_worker_adapter.is_none());
    }
}
