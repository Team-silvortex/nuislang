#[cfg(unix)]
use crate::provider_process_adapter::ProviderProcessAdapterCache;
use crate::{
    provider_edge_transport::ProviderEdgeTransportReceipt,
    provider_graph_output::CompletedProviderOutput,
    provider_output_carrier_registry::ProviderOutputPayload,
    provider_prepared_input::PreparedProviderInput, provider_request::ProviderRequest,
    provider_sample_payload::ProviderNativeOutputSummary,
    provider_worker_lease::ProviderWorkerDispatchReceipt,
};
use std::path::{Path, PathBuf};

pub(crate) const PROVIDER_EXECUTION_ADAPTER_REGISTRY_CONTRACT: &str =
    "nuis-provider-execution-adapter-registry-v1";

pub(crate) struct ProviderRequestExecution {
    pub(crate) summary: ProviderNativeOutputSummary,
    pub(crate) output_payload: ProviderOutputPayload,
    pub(crate) transferable_output:
        Option<crate::provider_carrier_channel_registry::PreparedProviderCarrierChannel>,
    pub(crate) additional_outputs: Vec<CompletedProviderOutput>,
    pub(crate) transport_receipts: Vec<ProviderEdgeTransportReceipt>,
}

#[cfg(unix)]
pub(crate) struct PreparedProviderExecutionAdapter {
    pub(crate) executable_path: PathBuf,
    pub(crate) executable_hash: String,
    pub(crate) runner_contract: &'static str,
    pub(crate) cache_identity: String,
    pub(crate) cache_status: &'static str,
    pub(crate) arguments: Vec<String>,
}

pub(crate) type ExecuteProviderRequest = fn(
    input_evidence: &str,
    provider_family: &str,
    output_dir: &Path,
    request: &ProviderRequest,
    inputs: &[PreparedProviderInput],
    worker_receipt: &mut ProviderWorkerDispatchReceipt,
) -> Result<ProviderRequestExecution, String>;

#[cfg(unix)]
pub(crate) type PrepareProviderWorkerAdapter =
    fn(
        cache: &mut ProviderProcessAdapterCache,
        output_dir: &Path,
        request: &ProviderRequest,
        inputs: &[PreparedProviderInput],
    ) -> Result<Option<PreparedProviderExecutionAdapter>, String>;

#[derive(Clone, Copy)]
pub(crate) struct ProviderExecutionAdapterRegistration {
    pub(crate) registry_contract: &'static str,
    pub(crate) adapter_kind: &'static str,
    pub(crate) requires_worker_descriptors: bool,
    #[cfg(unix)]
    pub(crate) prepare_worker_adapter: Option<PrepareProviderWorkerAdapter>,
    pub(crate) execute: ExecuteProviderRequest,
}

pub(crate) fn select_provider_execution_adapter(
    adapter_kind: &str,
) -> Option<&'static ProviderExecutionAdapterRegistration> {
    crate::provider_bundle_registry::select_provider_bundle_by_adapter_kind(adapter_kind)
        .map(|bundle| &bundle.execution_adapter)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execution_adapters_are_unique_and_selected_without_backend_branching() {
        let registrations = crate::provider_bundle_registry::provider_bundle_registrations()
            .iter()
            .map(|bundle| &bundle.execution_adapter)
            .collect::<Vec<_>>();
        assert!(registrations.len() >= 3);
        assert!(registrations.iter().all(|registration| {
            registration.registry_contract == PROVIDER_EXECUTION_ADAPTER_REGISTRY_CONTRACT
        }));
        let kinds = registrations
            .iter()
            .map(|registration| registration.adapter_kind)
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(kinds.len(), registrations.len());
        assert!(select_provider_execution_adapter("metal-real-device-runner").is_some());
        assert!(select_provider_execution_adapter("missing-runner").is_none());
    }

    #[test]
    fn provider_sample_frontdoor_contains_no_concrete_execution_branches() {
        let frontdoor = include_str!("provider_sample_execute.rs");
        for forbidden in [
            "metal-real-device-runner",
            "coreml-real-device-runner",
            "provider-worker-native-runner",
            "prepare_gray8_worker_invocation",
            "parse_coreml_worker_output",
        ] {
            assert!(
                !frontdoor.contains(forbidden),
                "provider frontdoor regained concrete branch `{forbidden}`"
            );
        }
    }
}
