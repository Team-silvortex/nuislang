use crate::{
    provider_execution_adapter::{
        PreparedProviderExecutionAdapter, ProviderExecutionAdapterRegistration,
        ProviderRequestExecution, PROVIDER_EXECUTION_ADAPTER_REGISTRY_CONTRACT,
    },
    provider_prepared_input::PreparedProviderInput,
    provider_process_adapter::ProviderProcessAdapterCache,
    provider_request::ProviderRequest,
    provider_worker_lease::ProviderWorkerDispatchReceipt,
};
use std::path::Path;

pub(crate) const REGISTRATION: ProviderExecutionAdapterRegistration =
    ProviderExecutionAdapterRegistration {
        registry_contract: PROVIDER_EXECUTION_ADAPTER_REGISTRY_CONTRACT,
        adapter_kind: "provider-worker-native-runner",
        requires_worker_descriptors: false,
        prepare_worker_adapter: Some(prepare_worker_adapter),
        execute,
    };

const DATA_FAN_OUT_SOURCE: &str = include_str!("../provider-runners/data_fan_out.c");

fn prepare_worker_adapter(
    cache: &mut ProviderProcessAdapterCache,
    _output_dir: &Path,
    request: &ProviderRequest,
    _inputs: &[PreparedProviderInput],
) -> Result<Option<PreparedProviderExecutionAdapter>, String> {
    if request.kernel.operation != "fan-out"
        || request.output_bindings.len() != 2
        || request
            .output_bindings
            .iter()
            .any(|binding| binding.byte_length != 24)
    {
        return Ok(None);
    }
    let prepared = cache.resolve_c(
        "data-fan-out-adapter",
        DATA_FAN_OUT_SOURCE,
        "nuis-data-fan-out-provider-runner-v1",
    )?;
    Ok(Some(PreparedProviderExecutionAdapter {
        executable_path: prepared.executable_path().to_owned(),
        executable_hash: prepared.executable_hash().to_owned(),
        runner_contract: prepared.contract(),
        cache_identity: prepared.cache_identity.to_owned(),
        cache_status: prepared.cache_status,
        arguments: vec!["literal:fan-out".to_owned()],
    }))
}

fn execute(
    input_evidence: &str,
    provider_family: &str,
    _output_dir: &Path,
    request: &ProviderRequest,
    _inputs: &[PreparedProviderInput],
    worker_receipt: &mut ProviderWorkerDispatchReceipt,
) -> Result<ProviderRequestExecution, String> {
    let (summary, output_payload, transferable_output) =
        crate::provider_worker_native_execution::take_provider_worker_native_output(
            input_evidence,
            provider_family,
            request,
            worker_receipt,
        )?;
    Ok(ProviderRequestExecution {
        summary,
        output_payload,
        transferable_output,
        additional_outputs: Vec::new(),
        transport_receipts: Vec::new(),
    })
}
