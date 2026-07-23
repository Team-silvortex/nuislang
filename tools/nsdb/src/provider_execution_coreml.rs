#[cfg(target_os = "macos")]
use crate::{
    provider_execution_adapter::PreparedProviderExecutionAdapter,
    provider_process_adapter::{coreml_worker_arguments, ProviderProcessAdapterCache},
};
use crate::{
    provider_execution_adapter::{
        ProviderExecutionAdapterRegistration, ProviderRequestExecution,
        PROVIDER_EXECUTION_ADAPTER_REGISTRY_CONTRACT,
    },
    provider_prepared_input::PreparedProviderInput,
    provider_process_adapter::validate_provider_model_asset,
    provider_request::ProviderRequest,
    provider_sample_payload::coreml_native_output_summary,
    provider_worker_lease::ProviderWorkerDispatchReceipt,
};
use std::path::Path;

pub(crate) const REGISTRATION: ProviderExecutionAdapterRegistration =
    ProviderExecutionAdapterRegistration {
        registry_contract: PROVIDER_EXECUTION_ADAPTER_REGISTRY_CONTRACT,
        adapter_kind: "coreml-real-device-runner",
        requires_worker_descriptors: true,
        #[cfg(target_os = "macos")]
        prepare_worker_adapter: Some(prepare_worker_adapter),
        #[cfg(all(unix, not(target_os = "macos")))]
        prepare_worker_adapter: None,
        execute,
    };

#[cfg(target_os = "macos")]
fn prepare_worker_adapter(
    cache: &mut ProviderProcessAdapterCache,
    output_dir: &Path,
    request: &ProviderRequest,
    inputs: &[PreparedProviderInput],
) -> Result<Option<PreparedProviderExecutionAdapter>, String> {
    if inputs.is_empty()
        || inputs
            .iter()
            .any(|input| input.worker_adapter_argument().is_none())
        || !request
            .model_asset
            .as_ref()
            .is_some_and(|model| model.input_features.len() == inputs.len())
    {
        return Ok(None);
    }
    validate_request_shape(request)?;
    let model_path = validate_provider_model_asset(output_dir, request)?;
    let prepared = crate::provider_runner_coreml::prepare_coreml_worker_invocation(cache)?;
    Ok(Some(PreparedProviderExecutionAdapter {
        executable_path: prepared.executable_path().to_owned(),
        executable_hash: prepared.executable_hash().to_owned(),
        runner_contract: prepared.contract(),
        cache_identity: prepared.cache_identity.to_owned(),
        cache_status: prepared.cache_status,
        arguments: coreml_worker_arguments(request, inputs, &model_path)?,
    }))
}

fn execute(
    _input_evidence: &str,
    _provider_family: &str,
    output_dir: &Path,
    request: &ProviderRequest,
    inputs: &[PreparedProviderInput],
    worker_receipt: &mut ProviderWorkerDispatchReceipt,
) -> Result<ProviderRequestExecution, String> {
    validate_request_shape(request)?;
    let model = request
        .model_asset
        .as_ref()
        .ok_or_else(|| "CoreML provider request is missing a model descriptor".to_owned())?;
    if model.input_features.len() != inputs.len()
        || request.input_bindings.len() != inputs.len()
        || inputs.is_empty()
    {
        return Err("CoreML provider input feature/binding count mismatch".to_owned());
    }
    let model_path = validate_provider_model_asset(output_dir, request)?;
    let coreml_inputs = inputs
        .iter()
        .zip(&model.input_features)
        .zip(&request.input_bindings)
        .map(|((input, feature), binding)| {
            let source = input.direct_channel().map_or_else(
                || crate::provider_runner_coreml::CoreMlProviderInputSource::Carrier(input.input()),
                crate::provider_runner_coreml::CoreMlProviderInputSource::PreparedChannel,
            );
            crate::provider_runner_coreml::CoreMlProviderInput {
                source,
                feature,
                shape: &binding.shape,
            }
        })
        .collect::<Vec<_>>();
    let output_shape = request
        .output_comparison
        .as_ref()
        .map(|comparison| comparison.shape.as_slice())
        .unwrap_or(request.buffer.shape.as_slice());
    let execution = if worker_receipt.execution_capsule_invocation_mode
        == "nuis-provider-worker-process-adapter-v5"
    {
        crate::provider_runner_coreml::parse_coreml_worker_output(
            &worker_receipt.worker_output_payload,
            worker_receipt.worker_output_result.take(),
        )?
    } else {
        crate::provider_runner_coreml::execute_model_prediction_inputs(
            &model_path,
            &coreml_inputs,
            &model.output_feature,
            output_shape,
        )?
    };
    Ok(ProviderRequestExecution {
        summary: coreml_native_output_summary(&request.kernel.id, &execution, None),
        output_payload: execution.output_payload,
        transferable_output: execution.transferable_output,
        additional_outputs: Vec::new(),
        transport_receipts: Vec::new(),
    })
}

fn validate_request_shape(request: &ProviderRequest) -> Result<(), String> {
    if request.buffer.element_type != "f32" || request.buffer.layout != "tensor-contiguous" {
        return Err(format!(
            "CoreML provider adapter requires a contiguous f32 tensor, got `{}` with `{}` elements",
            request.buffer.layout, request.buffer.element_type
        ));
    }
    Ok(())
}
