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
    output_dir: &Path,
    request: &ProviderRequest,
    inputs: &[PreparedProviderInput],
) -> Result<Option<PreparedProviderExecutionAdapter>, String> {
    validate_metal_input_count(output_dir, request, inputs.len())?;
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
        let mut arguments = metal_code_asset_arguments(output_dir, request)?;
        arguments.push(format!("literal:{bias}"));
        (
            crate::provider_runner_metal::prepare_f32_bias_worker_invocation(cache)?,
            arguments,
        )
    } else if is_f32_argmax(request) {
        (
            crate::provider_runner_metal::prepare_f32_argmax_worker_invocation(cache)?,
            metal_code_asset_arguments(output_dir, request)?,
        )
    } else if is_u32_compute(request) {
        let mut arguments = metal_code_asset_arguments(output_dir, request)?;
        arguments.push(format!("literal:{}", request.kernel.operation));
        (
            if is_u32_copy(request) {
                crate::provider_runner_metal::prepare_u32_copy_worker_invocation(cache)?
            } else {
                crate::provider_runner_metal::prepare_u32_canonical_worker_invocation(cache)?
            },
            arguments,
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
            .chain(
                inputs
                    .iter()
                    .enumerate()
                    .skip(1)
                    .map(|(index, input)| worker_descriptor_argument(input, index))
                    .collect::<Result<Vec<_>, _>>()?,
            )
            .chain(arguments)
            .collect(),
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
    validate_metal_input_count(output_dir, request, inputs.len())?;
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
        let (code_asset_path, entry) = validated_metal_code_asset(output_dir, request)?;
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
                &code_asset_path,
                &entry,
            )?
        } else {
            crate::provider_runner_metal::execute_f32_bias_input(
                inputs[0].input(),
                bias,
                &code_asset_path,
                &entry,
            )?
        }
    } else if is_f32_argmax(request) {
        let (code_asset_path, entry) = validated_metal_code_asset(output_dir, request)?;
        if uses_process_adapter(worker_receipt) {
            crate::provider_runner_metal::parse_metal_worker_output(
                &worker_receipt.worker_output_payload,
                "nuis-metal-f32-argmax-provider-runner-v1",
                worker_receipt.worker_output_result.take(),
            )?
        } else if let Some(channel) = inputs[0].direct_channel() {
            crate::provider_runner_metal::execute_f32_argmax_prepared_channel(
                channel,
                request.input_bindings[0].byte_length,
                &code_asset_path,
                &entry,
            )?
        } else {
            crate::provider_runner_metal::execute_f32_argmax_input(
                inputs[0].input(),
                &code_asset_path,
                &entry,
            )?
        }
    } else if is_u32_compute(request) {
        let (code_asset_path, entry) = validated_metal_code_asset(output_dir, request)?;
        let runner_contract =
            crate::provider_runner_metal::u32_compute_runner_contract(&request.kernel.operation);
        if uses_process_adapter(worker_receipt) {
            crate::provider_runner_metal::parse_metal_worker_output(
                &worker_receipt.worker_output_payload,
                runner_contract,
                worker_receipt.worker_output_result.take(),
            )?
        } else if inputs.len() != 1 || request.output_bindings.len() != 1 {
            return Err(format!(
                "Metal multi-input or multi-output kernel `{}` requires the process adapter",
                request.kernel.id
            ));
        } else if let Some(channel) = inputs[0].direct_channel() {
            crate::provider_runner_metal::execute_u32_canonical_prepared_channel(
                channel,
                request.input_bindings[0].byte_length,
                &code_asset_path,
                &entry,
                &request.kernel.operation,
            )?
        } else {
            crate::provider_runner_metal::execute_u32_canonical_input(
                inputs[0].input(),
                &code_asset_path,
                &entry,
                &request.kernel.operation,
            )?
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
        } else if is_f32_argmax(request) {
            metal_native_output_summary(
                request.kernel.id.clone(),
                "provider-scalar-u32",
                &execution,
                None,
            )
        } else if is_u32_compute(request) {
            metal_native_output_summary(
                request.kernel.id.clone(),
                "provider-tensor-u32",
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

#[cfg(target_os = "macos")]
fn metal_code_asset_arguments(
    output_dir: &Path,
    request: &ProviderRequest,
) -> Result<Vec<String>, String> {
    let (path, entry) = validated_metal_code_asset(output_dir, request)?;
    let hash = &request
        .code_asset
        .as_ref()
        .expect("validated Metal code asset")
        .content_hash;
    Ok(vec![
        format!("verified-path:{hash}:{}", path.display()),
        format!("literal:{entry}"),
    ])
}

fn validated_metal_code_asset(
    output_dir: &Path,
    request: &ProviderRequest,
) -> Result<(std::path::PathBuf, String), String> {
    let asset = request
        .code_asset
        .as_ref()
        .ok_or_else(|| "Metal f32 request is missing its code asset".to_owned())?;
    if asset.format != "metal-source"
        || !matches!(
            asset.target.as_str(),
            "metal.apple-silicon-gpu" | "metal.mac-discrete-or-integrated-gpu" | "msl2.4"
        )
        || asset.entry.is_empty()
    {
        return Err("Metal request code asset descriptor is incompatible".to_owned());
    }
    let path = crate::provider_process_adapter::validate_provider_code_asset(output_dir, request)?;
    Ok((path, asset.entry.clone()))
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

fn is_f32_argmax(request: &ProviderRequest) -> bool {
    request.buffer.element_type == "f32"
        && request.buffer.layout == "tensor-contiguous"
        && request.kernel.operation == "argmax"
        && request.output_bindings.len() == 1
        && request.output_bindings[0].element_type == "u32"
        && request.output_bindings[0].shape == [1]
}

fn is_u32_copy(request: &ProviderRequest) -> bool {
    is_u32_compute(request) && request.kernel.operation == "copy-u32"
}

fn is_u32_compute(request: &ProviderRequest) -> bool {
    request.buffer.element_type == "u32"
        && metal_dense_u32_layout(request)
        && request.kernel.operation.ends_with("-u32")
        && !request.output_bindings.is_empty()
        && request.output_bindings.iter().all(|binding| {
            crate::provider_output_binding::output_binding_matches_buffer(binding, &request.buffer)
        })
}

fn metal_dense_u32_layout(request: &ProviderRequest) -> bool {
    match request.buffer.layout.as_str() {
        "tensor-contiguous" => request.buffer.row_stride_bytes == request.buffer.byte_length,
        "tensor-row-major" => {
            let [width, height] = request.buffer.shape.as_slice() else {
                return false;
            };
            width.checked_mul(std::mem::size_of::<u32>()) == Some(request.buffer.row_stride_bytes)
                && request.buffer.row_stride_bytes.checked_mul(*height)
                    == Some(request.buffer.byte_length)
        }
        _ => false,
    }
}

fn validate_metal_input_count(
    output_dir: &Path,
    request: &ProviderRequest,
    input_count: usize,
) -> Result<(), String> {
    if is_u32_compute(request) {
        let (path, entry) = validated_metal_code_asset(output_dir, request)?;
        let (expected_inputs, expected_outputs) = metal_u32_buffer_counts(&path, &entry)?;
        if input_count != expected_inputs || request.input_bindings.len() != expected_inputs {
            return Err(format!(
                "Metal kernel `{}` declares {expected_inputs} input buffer(s), but the request carries {} binding(s) and {input_count} prepared input(s)",
                request.kernel.id,
                request.input_bindings.len()
            ));
        }
        if request.output_bindings.len() != expected_outputs {
            return Err(format!(
                "Metal kernel `{}` declares {expected_outputs} output buffer(s), but the request carries {} binding(s)",
                request.kernel.id,
                request.output_bindings.len()
            ));
        }
        if request.input_bindings.iter().any(|binding| {
            !crate::provider_input_binding::input_binding_matches_buffer(binding, &request.buffer)
        }) || request.output_bindings.iter().any(|binding| {
            !crate::provider_output_binding::output_binding_matches_buffer(binding, &request.buffer)
        }) {
            return Err(format!(
                "Metal kernel `{}` input/output layouts do not match its primary typed buffer",
                request.kernel.id
            ));
        }
        return Ok(());
    }
    if input_count == 1 && request.input_bindings.len() == 1 {
        Ok(())
    } else {
        Err(format!(
            "Metal provider adapter requires one input for kernel `{}`",
            request.kernel.id
        ))
    }
}

fn metal_u32_buffer_counts(path: &Path, entry: &str) -> Result<(usize, usize), String> {
    let source = std::fs::read_to_string(path)
        .map_err(|error| format!("failed to read verified Metal code asset: {error}"))?;
    let marker = format!("kernel void {entry}(");
    let signature_start = source
        .find(&marker)
        .ok_or_else(|| format!("Metal code asset is missing entry `{entry}`"))?;
    let signature = source[signature_start..]
        .split_once('{')
        .map(|(signature, _)| signature)
        .ok_or_else(|| format!("Metal entry `{entry}` has no function body"))?;
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();
    for line in signature.lines() {
        let Some(slot) = metal_buffer_slot(line) else {
            continue;
        };
        if line.contains("device const uint*") {
            inputs.push(slot);
        } else if line.contains("device uint*") {
            outputs.push(slot);
        }
    }
    let expected_inputs: Vec<usize> = (0..inputs.len()).collect();
    let expected_outputs = (inputs.len()..inputs.len() + outputs.len()).collect::<Vec<_>>();
    if inputs.is_empty()
        || outputs.is_empty()
        || inputs != expected_inputs
        || outputs != expected_outputs
    {
        return Err(format!(
            "Metal entry `{entry}` must declare contiguous read-only u32 inputs followed by contiguous writable u32 outputs"
        ));
    }
    Ok((inputs.len(), outputs.len()))
}

fn metal_buffer_slot(line: &str) -> Option<usize> {
    let start = line.find("[[buffer(")? + "[[buffer(".len();
    let end = line[start..].find(")]]")? + start;
    line[start..end].parse().ok()
}

fn uses_process_adapter(receipt: &ProviderWorkerDispatchReceipt) -> bool {
    receipt.execution_capsule_invocation_mode == "nuis-provider-worker-process-adapter-v5"
}

#[cfg(test)]
mod tests {
    use super::metal_u32_buffer_counts;
    use std::{env, fs};

    #[test]
    fn metal_layout_uses_contiguous_read_inputs_followed_by_output() {
        let path = env::temp_dir().join(format!(
            "nuis-metal-layout-{}-{}.metal",
            std::process::id(),
            "pair"
        ));
        fs::write(
            &path,
            "kernel void pair(device const uint* left [[buffer(0)]],\n\
             device const uint* right [[buffer(1)]],\n\
             device uint* output [[buffer(2)]]) { }\n",
        )
        .unwrap();

        assert_eq!(metal_u32_buffer_counts(&path, "pair").unwrap(), (2, 1));
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn metal_layout_accepts_contiguous_multiple_outputs() {
        let path = env::temp_dir().join(format!(
            "nuis-metal-layout-{}-{}.metal",
            std::process::id(),
            "fan-out"
        ));
        fs::write(
            &path,
            "kernel void fan_out(device const uint* left [[buffer(0)]],\n\
             device const uint* right [[buffer(1)]],\n\
             device uint* sum [[buffer(2)]],\n\
             device uint* audit [[buffer(3)]]) { }\n",
        )
        .unwrap();

        assert_eq!(metal_u32_buffer_counts(&path, "fan_out").unwrap(), (2, 2));
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn metal_layout_rejects_a_gap_before_output() {
        let path = env::temp_dir().join(format!(
            "nuis-metal-layout-{}-{}.metal",
            std::process::id(),
            "gap"
        ));
        fs::write(
            &path,
            "kernel void gap(device const uint* input [[buffer(0)]],\n\
             device uint* output [[buffer(2)]]) { }\n",
        )
        .unwrap();

        assert!(metal_u32_buffer_counts(&path, "gap")
            .unwrap_err()
            .contains("contiguous read-only u32 inputs"));
        fs::remove_file(path).unwrap();
    }
}
