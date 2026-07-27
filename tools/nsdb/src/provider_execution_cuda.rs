#[cfg(target_os = "linux")]
use crate::{
    provider_execution_adapter::PreparedProviderExecutionAdapter,
    provider_process_adapter::{
        validate_provider_code_asset, worker_descriptor_argument, ProviderProcessAdapterCache,
    },
};
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
        adapter_kind: "cuda-ptx-real-device-runner",
        requires_worker_descriptors: true,
        #[cfg(target_os = "linux")]
        prepare_worker_adapter: Some(prepare_worker_adapter),
        #[cfg(all(unix, not(target_os = "linux")))]
        prepare_worker_adapter: None,
        execute,
    };

#[cfg(target_os = "linux")]
fn prepare_worker_adapter(
    cache: &mut ProviderProcessAdapterCache,
    output_dir: &Path,
    request: &ProviderRequest,
    inputs: &[PreparedProviderInput],
) -> Result<Option<PreparedProviderExecutionAdapter>, String> {
    validate_cuda_vector_add_request(request, inputs)?;
    let asset = request
        .code_asset
        .as_ref()
        .expect("validated CUDA request must own a code asset");
    let asset_path = validate_provider_code_asset(output_dir, request)?;
    let asset_path = asset_path
        .to_str()
        .ok_or_else(|| "CUDA PTX asset path is not UTF-8".to_owned())?;
    let element_count = request
        .scalar_u32("element_count")
        .expect("validated CUDA request must own element_count");
    let prepared = crate::provider_runner_cuda::prepare_vector_add_worker_invocation(cache)?;
    Ok(Some(PreparedProviderExecutionAdapter {
        executable_path: prepared.executable_path().to_owned(),
        executable_hash: prepared.executable_hash().to_owned(),
        runner_contract: prepared.contract(),
        cache_identity: prepared.cache_identity.to_owned(),
        cache_status: prepared.cache_status,
        arguments: vec![
            format!("verified-path:{}:{asset_path}", asset.content_hash),
            format!("literal:{}", asset.entry),
            worker_descriptor_argument(&inputs[0], 0)?,
            worker_descriptor_argument(&inputs[1], 1)?,
            format!("literal:{element_count}"),
        ],
    }))
}

#[cfg(target_os = "linux")]
fn execute(
    input_evidence: &str,
    provider_family: &str,
    _output_dir: &Path,
    request: &ProviderRequest,
    inputs: &[PreparedProviderInput],
    worker_receipt: &mut ProviderWorkerDispatchReceipt,
) -> Result<ProviderRequestExecution, String> {
    validate_cuda_vector_add_request(request, inputs)?;
    if worker_receipt.execution_capsule_invocation_mode
        != crate::provider_worker_lease::PROVIDER_WORKER_PROCESS_ADAPTER_CONTRACT
    {
        return Err("CUDA PTX provider requires its registered process adapter".to_owned());
    }
    let (mut summary, output_payload, transferable_output) =
        crate::provider_worker_native_execution::take_provider_worker_native_output(
            input_evidence,
            provider_family,
            request,
            worker_receipt,
        )?;
    summary.execution_contract = "nuis-cuda-ptx-driver-provider-execution-v1".to_owned();
    summary.execution_status = "cuda-driver-kernel-completed".to_owned();
    summary.device = "cuda:nvidia-gpu".to_owned();
    Ok(ProviderRequestExecution {
        summary,
        output_payload,
        transferable_output,
        additional_outputs: Vec::new(),
        transport_receipts: Vec::new(),
    })
}

#[cfg(not(target_os = "linux"))]
fn execute(
    _input_evidence: &str,
    _provider_family: &str,
    _output_dir: &Path,
    _request: &ProviderRequest,
    _inputs: &[PreparedProviderInput],
    _worker_receipt: &mut ProviderWorkerDispatchReceipt,
) -> Result<ProviderRequestExecution, String> {
    Err("CUDA PTX provider execution is unavailable on this host".to_owned())
}

#[cfg(target_os = "linux")]
fn validate_cuda_vector_add_request(
    request: &ProviderRequest,
    inputs: &[PreparedProviderInput],
) -> Result<(), String> {
    let asset = request
        .code_asset
        .as_ref()
        .ok_or_else(|| "CUDA provider request is missing its PTX code asset".to_owned())?;
    let element_count = request
        .scalar_u32("element_count")
        .ok_or_else(|| "CUDA vector-add request is missing u32 `element_count`".to_owned())?;
    let byte_length = usize::try_from(element_count)
        .ok()
        .and_then(|count| count.checked_mul(std::mem::size_of::<f32>()))
        .ok_or_else(|| "CUDA vector-add element count overflows host size".to_owned())?;
    let binding = request
        .adapter_binding
        .as_ref()
        .ok_or_else(|| "CUDA provider request is missing its adapter binding".to_owned())?;
    if request.kernel.operation != "vector-add"
        || inputs.len() != 2
        || request.input_bindings.len() != 2
        || request.output_bindings.len() != 1
        || request
            .input_bindings
            .iter()
            .any(|input| input.element_type != "f32" || input.byte_length != byte_length)
        || request.output_bindings[0].element_type != "f32"
        || request.output_bindings[0].byte_length != byte_length
        || asset.format != "ptx"
        || !asset.target.starts_with("sm_")
        || binding.provider_family != "cuda:nvidia-gpu"
        || binding.execution_requirement != "real-device"
    {
        return Err(
            "CUDA vector-add provider request does not match the registered ABI".to_owned(),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cuda_execution_registration_is_linux_owned_and_fails_closed_elsewhere() {
        assert_eq!(
            REGISTRATION.registry_contract,
            PROVIDER_EXECUTION_ADAPTER_REGISTRY_CONTRACT
        );
        assert_eq!(REGISTRATION.adapter_kind, "cuda-ptx-real-device-runner");
        assert!(REGISTRATION.requires_worker_descriptors);
        #[cfg(target_os = "linux")]
        assert!(REGISTRATION.prepare_worker_adapter.is_some());
        #[cfg(not(target_os = "linux"))]
        assert!(REGISTRATION.prepare_worker_adapter.is_none());
    }
}
