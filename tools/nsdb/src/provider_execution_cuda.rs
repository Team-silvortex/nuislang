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
#[cfg(any(target_os = "linux", test))]
use std::{collections::BTreeMap, str};

#[cfg(any(target_os = "linux", test))]
struct CudaDeviceSelectionEvidence {
    inventory_count: u32,
    ordinal: u32,
    minimum_compute_capability: u32,
    selected_compute_capability: u32,
}

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
    validate_cuda_request(request, inputs)?;
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
    let device_selection_policy = request
        .scalar_u32("device_selection_policy")
        .expect("validated CUDA request must own device_selection_policy");
    let minimum_compute_capability = request
        .scalar_u32("minimum_compute_capability")
        .expect("validated CUDA request must own minimum_compute_capability");
    let prepared = crate::provider_runner_cuda::prepare_cuda_worker_invocation(cache)?;
    let operation_argument = match request.kernel.operation.as_str() {
        "vector-add" => worker_descriptor_argument(&inputs[1], 1)?,
        "scale" => format!(
            "literal:{}",
            request
                .scalar_f32("scale")
                .expect("validated CUDA scale request must own scale")
        ),
        "copy-u32" => "literal:0".to_owned(),
        "add-scalar-i64" => format!(
            "literal:{}",
            request
                .scalar_i64("scalar")
                .expect("validated CUDA i64 scalar request must own scalar")
        ),
        "reduce-sum-i64" => "literal:0".to_owned(),
        _ => unreachable!("validated CUDA operation"),
    };
    Ok(Some(PreparedProviderExecutionAdapter {
        executable_path: prepared.executable_path().to_owned(),
        executable_hash: prepared.executable_hash().to_owned(),
        runner_contract: prepared.contract(),
        cache_identity: prepared.cache_identity.to_owned(),
        cache_status: prepared.cache_status,
        arguments: vec![
            format!("verified-path:{}:{asset_path}", asset.content_hash),
            format!("literal:{}", asset.entry),
            format!("literal:{}", request.kernel.operation),
            worker_descriptor_argument(&inputs[0], 0)?,
            operation_argument,
            format!("literal:{element_count}"),
            format!("literal:{device_selection_policy}"),
            format!("literal:{minimum_compute_capability}"),
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
    validate_cuda_request(request, inputs)?;
    if worker_receipt.execution_capsule_invocation_mode
        != crate::provider_worker_lease::PROVIDER_WORKER_PROCESS_ADAPTER_CONTRACT
    {
        return Err("CUDA PTX provider requires its registered process adapter".to_owned());
    }
    let selection = parse_cuda_device_selection(
        &worker_receipt.worker_output_payload,
        request
            .scalar_u32("device_selection_policy")
            .expect("validated CUDA request must own device_selection_policy"),
        request
            .scalar_u32("minimum_compute_capability")
            .expect("validated CUDA request must own minimum_compute_capability"),
    )?;
    let (mut summary, output_payload, transferable_output) =
        crate::provider_worker_native_execution::take_provider_worker_native_output(
            input_evidence,
            provider_family,
            request,
            worker_receipt,
        )?;
    summary.execution_contract = "nuis-cuda-ptx-driver-provider-execution-v1".to_owned();
    summary.execution_status = "cuda-driver-kernel-completed".to_owned();
    summary.device = format!(
        "cuda:nvidia-gpu:ordinal-{}:sm_{}",
        selection.ordinal, selection.selected_compute_capability
    );
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
fn validate_cuda_request(
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
    let binding = request
        .adapter_binding
        .as_ref()
        .ok_or_else(|| "CUDA provider request is missing its adapter binding".to_owned())?;
    let device_selection_policy = request.scalar_u32("device_selection_policy");
    let minimum_compute_capability = request.scalar_u32("minimum_compute_capability");
    let asset_compute_capability = asset
        .target
        .strip_prefix("sm_")
        .and_then(|value| value.parse::<u32>().ok());
    let (element_type, element_width, output_element_count, operation_valid) =
        match request.kernel.operation.as_str() {
            "vector-add" => (
                "f32",
                std::mem::size_of::<f32>(),
                element_count,
                asset.entry == "nuis_kernel_vector_add_f32"
                    && inputs.len() == 2
                    && request.input_bindings.len() == 2,
            ),
            "scale" => (
                "f32",
                std::mem::size_of::<f32>(),
                element_count,
                asset.entry == "nuis_kernel_scale_f32"
                    && inputs.len() == 1
                    && request.input_bindings.len() == 1
                    && request.scalar_f32("scale").is_some_and(f32::is_finite),
            ),
            "copy-u32" => (
                "u32",
                std::mem::size_of::<u32>(),
                element_count,
                asset.entry == "nuis_kernel_copy_u32"
                    && inputs.len() == 1
                    && request.input_bindings.len() == 1,
            ),
            "add-scalar-i64" => (
                "i64",
                std::mem::size_of::<i64>(),
                element_count,
                inputs.len() == 1
                    && request.input_bindings.len() == 1
                    && request.scalar_i64("scalar").is_some(),
            ),
            "reduce-sum-i64" => (
                "i64",
                std::mem::size_of::<i64>(),
                1,
                inputs.len() == 1
                    && request.input_bindings.len() == 1
                    && request.kernel.dispatch == [1, 1, 1],
            ),
            _ => ("none", 0, 0, false),
        };
    let input_byte_length = usize::try_from(element_count)
        .ok()
        .and_then(|count| count.checked_mul(element_width))
        .ok_or_else(|| "CUDA provider element count overflows host size".to_owned())?;
    let output_byte_length = usize::try_from(output_element_count)
        .ok()
        .and_then(|count| count.checked_mul(element_width))
        .ok_or_else(|| "CUDA provider output count overflows host size".to_owned())?;
    if !operation_valid
        || request.output_bindings.len() != 1
        || request.input_bindings.iter().any(|input| {
            input.element_type != element_type || input.byte_length != input_byte_length
        })
        || request.output_bindings[0].element_type != element_type
        || request.output_bindings[0].byte_length != output_byte_length
        || asset.format != "ptx"
        || !asset.target.starts_with("sm_")
        || !device_selection_policy
            .zip(minimum_compute_capability)
            .is_some_and(|(policy, minimum)| {
                crate::provider_runner_cuda::validates_device_selection(policy, minimum)
            })
        || minimum_compute_capability != asset_compute_capability
        || binding.provider_family != "cuda:nvidia-gpu"
        || binding.execution_requirement != "real-device"
    {
        return Err("CUDA provider request does not match the registered ABI".to_owned());
    }
    Ok(())
}

#[cfg(any(target_os = "linux", test))]
fn parse_cuda_device_selection(
    protocol: &[u8],
    expected_policy_code: u32,
    expected_minimum_compute_capability: u32,
) -> Result<CudaDeviceSelectionEvidence, String> {
    let protocol =
        str::from_utf8(protocol).map_err(|_| "CUDA adapter protocol is not UTF-8".to_owned())?;
    let mut fields = BTreeMap::new();
    for line in protocol.lines() {
        let Some((key, value)) = line.split_once('=') else {
            return Err("CUDA adapter protocol field is malformed".to_owned());
        };
        if fields.insert(key, value).is_some() {
            return Err(format!("CUDA adapter protocol duplicates `{key}`"));
        }
    }
    let required = |key: &str| {
        fields
            .get(key)
            .copied()
            .ok_or_else(|| format!("CUDA adapter protocol is missing `{key}`"))
    };
    if required("protocol")? != "nuis-cuda-ptx-driver-provider-runner-v1"
        || required("status")? != "ready"
        || required("device_selection_contract")?
            != crate::provider_runner_cuda::CUDA_DEVICE_SELECTION_CONTRACT
        || required("device_inventory_contract")?
            != crate::provider_runner_cuda::CUDA_DEVICE_INVENTORY_CONTRACT
        || required("device_selection_policy")?
            != crate::provider_runner_cuda::DEVICE_SELECTION_PROFILE.policy
        || required("device_selection_status")? != "verified"
    {
        return Err("CUDA adapter protocol device selection is unverified".to_owned());
    }
    let parse_u32 = |key: &str| {
        required(key)?
            .parse::<u32>()
            .map_err(|error| format!("CUDA adapter protocol `{key}` is invalid: {error}"))
    };
    let evidence = CudaDeviceSelectionEvidence {
        inventory_count: parse_u32("device_inventory_count")?,
        ordinal: parse_u32("selected_device_ordinal")?,
        minimum_compute_capability: parse_u32("minimum_compute_capability")?,
        selected_compute_capability: parse_u32("selected_compute_capability")?,
    };
    if parse_u32("device_selection_policy_code")? != expected_policy_code
        || evidence.inventory_count == 0
        || evidence.ordinal >= evidence.inventory_count
        || evidence.minimum_compute_capability != expected_minimum_compute_capability
        || evidence.selected_compute_capability < evidence.minimum_compute_capability
    {
        return Err("CUDA adapter protocol device selection does not match the request".to_owned());
    }
    Ok(evidence)
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
        const { assert!(REGISTRATION.requires_worker_descriptors) };
        #[cfg(target_os = "linux")]
        assert!(REGISTRATION.prepare_worker_adapter.is_some());
        #[cfg(not(target_os = "linux"))]
        assert!(REGISTRATION.prepare_worker_adapter.is_none());
    }

    #[test]
    fn cuda_device_selection_protocol_is_request_bound() {
        let protocol = b"protocol=nuis-cuda-ptx-driver-provider-runner-v1\nstatus=ready\ndevice_inventory_contract=nuis-cuda-device-inventory-v1\ndevice_inventory_count=3\ndevice_selection_contract=nuis-cuda-device-selection-v1\ndevice_selection_policy=capability-ranked-lowest-ordinal\ndevice_selection_policy_code=1\ndevice_selection_status=verified\nselected_device_ordinal=0\nminimum_compute_capability=80\nselected_compute_capability=89\noutput_hash=1\n";
        let evidence = parse_cuda_device_selection(protocol, 1, 80).expect("selection evidence");
        assert_eq!(evidence.inventory_count, 3);
        assert_eq!(evidence.ordinal, 0);
        assert_eq!(evidence.selected_compute_capability, 89);
        assert!(parse_cuda_device_selection(protocol, 2, 80).is_err());
        assert!(parse_cuda_device_selection(protocol, 1, 90).is_err());
    }
}
