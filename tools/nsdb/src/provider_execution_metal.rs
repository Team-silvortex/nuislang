#[cfg(target_os = "macos")]
use crate::{
    provider_execution_adapter::PreparedProviderExecutionAdapter,
    provider_process_adapter::{worker_descriptor_argument, ProviderProcessAdapterCache},
};
use crate::{
    provider_execution_adapter::{
        ProviderExecutionAdapterRegistration, ProviderRequestExecution,
        PROVIDER_EXECUTION_ADAPTER_REGISTRY_CONTRACT,
    },
    provider_prepared_input::PreparedProviderInput,
    provider_request::ProviderRequest,
    provider_sample_payload::metal_native_output_summary,
    provider_worker_lease::ProviderWorkerDispatchReceipt,
};
use std::path::Path;

pub(crate) const REGISTRATION: ProviderExecutionAdapterRegistration =
    ProviderExecutionAdapterRegistration {
        registry_contract: PROVIDER_EXECUTION_ADAPTER_REGISTRY_CONTRACT,
        adapter_kind: "metal-real-device-runner",
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
    _output_dir: &Path,
    request: &ProviderRequest,
    inputs: &[PreparedProviderInput],
) -> Result<Option<PreparedProviderExecutionAdapter>, String> {
    if inputs.len() != 1 {
        return Err(format!(
            "Metal provider adapter requires one input for kernel `{}`",
            request.kernel.id
        ));
    }
    let (prepared, arguments) = if is_gray8_invert(request) {
        let max_value = request
            .scalar_u8("max_value")
            .ok_or_else(|| "Metal provider request is missing u8 scalar `max_value`".to_owned())?;
        (
            crate::provider_runner_metal::prepare_gray8_worker_invocation(cache)?,
            vec![
                "literal:invert".to_owned(),
                format!("literal:{max_value}"),
                format!("literal:{max_value}"),
            ],
        )
    } else if is_gray8_threshold(request) {
        let threshold = request
            .scalar_u8("threshold")
            .ok_or_else(|| "Metal provider request is missing u8 scalar `threshold`".to_owned())?;
        let max_value = request
            .scalar_u8("max_value")
            .ok_or_else(|| "Metal provider request is missing u8 scalar `max_value`".to_owned())?;
        (
            crate::provider_runner_metal::prepare_gray8_threshold_worker_invocation(cache)?,
            vec![
                "literal:threshold".to_owned(),
                format!("literal:{threshold}"),
                format!("literal:{max_value}"),
            ],
        )
    } else if is_f32_bias(request) {
        let bias = request
            .scalar_f32("bias")
            .ok_or_else(|| "Metal provider request is missing f32 scalar `bias`".to_owned())?;
        (
            crate::provider_runner_metal::prepare_f32_bias_worker_invocation(cache)?,
            vec![format!("literal:{bias}")],
        )
    } else {
        return Ok(None);
    };
    Ok(Some(PreparedProviderExecutionAdapter {
        executable_path: prepared.executable_path().to_owned(),
        executable_hash: prepared.executable_hash().to_owned(),
        runner_contract: prepared.contract(),
        cache_identity: prepared.cache_identity.to_owned(),
        cache_status: prepared.cache_status,
        arguments: std::iter::once(worker_descriptor_argument(&inputs[0], 0)?)
            .chain(arguments)
            .collect(),
    }))
}

fn execute(
    _input_evidence: &str,
    _provider_family: &str,
    _output_dir: &Path,
    request: &ProviderRequest,
    inputs: &[PreparedProviderInput],
    worker_receipt: &mut ProviderWorkerDispatchReceipt,
) -> Result<ProviderRequestExecution, String> {
    if inputs.len() != 1 {
        return Err(format!(
            "Metal provider adapter requires one input for kernel `{}`",
            request.kernel.id
        ));
    }
    let execution = if is_gray8_invert(request) {
        let max_value = request
            .scalar_u8("max_value")
            .ok_or_else(|| "Metal provider request is missing u8 scalar `max_value`".to_owned())?;
        if uses_process_adapter(worker_receipt) {
            crate::provider_runner_metal::parse_metal_worker_output(
                &worker_receipt.worker_output_payload,
                "nuis-metal-gray8-provider-runner-v1",
                worker_receipt.worker_output_result.take(),
            )?
        } else {
            let path = inputs[0]
                .input()
                .path()
                .ok_or_else(|| "Metal gray8 provider requires a path input".to_owned())?;
            crate::provider_runner_metal::execute_gray8_invert(path, max_value)?
        }
    } else if is_gray8_threshold(request) {
        let threshold = request
            .scalar_u8("threshold")
            .ok_or_else(|| "Metal provider request is missing u8 scalar `threshold`".to_owned())?;
        let max_value = request
            .scalar_u8("max_value")
            .ok_or_else(|| "Metal provider request is missing u8 scalar `max_value`".to_owned())?;
        if uses_process_adapter(worker_receipt) {
            crate::provider_runner_metal::parse_metal_worker_output(
                &worker_receipt.worker_output_payload,
                "nuis-metal-gray8-threshold-provider-runner-v1",
                worker_receipt.worker_output_result.take(),
            )?
        } else {
            let path = inputs[0]
                .input()
                .path()
                .ok_or_else(|| "Metal gray8 provider requires a path input".to_owned())?;
            crate::provider_runner_metal::execute_gray8_threshold(path, threshold, max_value)?
        }
    } else if is_f32_bias(request) {
        let bias = request
            .scalar_f32("bias")
            .ok_or_else(|| "Metal provider request is missing f32 scalar `bias`".to_owned())?;
        if uses_process_adapter(worker_receipt) {
            crate::provider_runner_metal::parse_metal_worker_output(
                &worker_receipt.worker_output_payload,
                "nuis-metal-f32-bias-provider-runner-v1",
                worker_receipt.worker_output_result.take(),
            )?
        } else if let Some(channel) = inputs[0].direct_channel() {
            crate::provider_runner_metal::execute_f32_bias_prepared_channel(
                channel,
                request.input_bindings[0].byte_length,
                bias,
            )?
        } else {
            crate::provider_runner_metal::execute_f32_bias_input(inputs[0].input(), bias)?
        }
    } else {
        return Err(format!(
            "Metal provider adapter does not support buffer `{}` operation `{}`",
            request.buffer.layout, request.kernel.operation
        ));
    };
    Ok(ProviderRequestExecution {
        summary: if is_gray8_invert(request) || is_gray8_threshold(request) {
            metal_native_output_summary(
                request.kernel.id.clone(),
                "pixelmagic-image-bytes",
                &execution,
                None,
            )
        } else {
            metal_native_output_summary(
                request.kernel.id.clone(),
                "provider-tensor-f32",
                &execution,
                None,
            )
        },
        output_payload: execution.output_payload,
        transferable_output: execution.transferable_output,
        additional_outputs: Vec::new(),
        transport_receipts: Vec::new(),
    })
}

fn is_gray8_invert(request: &ProviderRequest) -> bool {
    request.buffer.element_type == "u8"
        && request.buffer.layout.contains("pixel-format=gray8")
        && request.kernel.operation == "invert"
}

fn is_gray8_threshold(request: &ProviderRequest) -> bool {
    request.buffer.element_type == "u8"
        && request.buffer.layout.contains("pixel-format=gray8")
        && request.kernel.operation == "threshold"
}

fn is_f32_bias(request: &ProviderRequest) -> bool {
    request.buffer.element_type == "f32"
        && request.buffer.layout == "tensor-contiguous"
        && request.kernel.operation == "bias"
}

fn uses_process_adapter(receipt: &ProviderWorkerDispatchReceipt) -> bool {
    receipt.execution_capsule_invocation_mode == "nuis-provider-worker-process-adapter-v5"
}
