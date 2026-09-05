use crate::provider_execution_vulkan_spirv::validate_spirv_u32_module;
#[cfg(test)]
use crate::provider_sample_payload::fnv1a64_hex;
use crate::{
    provider_code_asset::ProviderCodeAssetDescriptor,
    provider_execution_adapter::{
        ProviderExecutionAdapterRegistration, ProviderRequestExecution,
        PROVIDER_EXECUTION_ADAPTER_REGISTRY_CONTRACT,
    },
    provider_prepared_input::PreparedProviderInput,
    provider_request::ProviderRequest,
    provider_worker_lease::ProviderWorkerDispatchReceipt,
};
#[cfg(target_os = "linux")]
use crate::{
    provider_execution_adapter::PreparedProviderExecutionAdapter,
    provider_process_adapter::{worker_descriptor_argument, ProviderProcessAdapterCache},
};
use std::{fs, path::Path};

pub(crate) const VULKAN_EXECUTION_SESSION_PLAN_CONTRACT: &str =
    "nuis-vulkan-spirv-execution-session-plan-v1";
const VULKAN_EXECUTION_SESSION_PLAN_STATUS: &str = "dispatch-readback-pending";
const VULKAN_PROVIDER_FAMILY: &str = "spirv:vulkan-gpu";
const VULKAN_SPIRV_FORMAT: &str = "spirv-binary";
const VULKAN_SPIRV_TARGET: &str = "vulkan1.3-spirv1.6";

pub(crate) const REGISTRATION: ProviderExecutionAdapterRegistration =
    ProviderExecutionAdapterRegistration {
        prepare_runtime_arguments: None,
        registry_contract: PROVIDER_EXECUTION_ADAPTER_REGISTRY_CONTRACT,
        adapter_kind: "vulkan-spirv-real-device-runner",
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
    let plan = validate_vulkan_execution_session_plan(
        output_dir,
        VULKAN_PROVIDER_FAMILY,
        request,
        inputs.len(),
    )?;
    let asset = request
        .code_asset
        .as_ref()
        .expect("validated Vulkan request must own a code asset");
    let asset_path =
        crate::provider_process_adapter::validate_provider_code_asset(output_dir, request)?;
    let asset_path = asset_path
        .to_str()
        .ok_or_else(|| "Vulkan SPIR-V asset path is not UTF-8".to_owned())?;
    let prepared = crate::provider_runner_vulkan::prepare_vulkan_worker_invocation(cache)?;
    let mut arguments = vec![
        format!("verified-path:{}:{asset_path}", asset.content_hash),
        format!("literal:{}", asset.entry),
    ];
    for (index, input) in inputs.iter().enumerate() {
        arguments.push(worker_descriptor_argument(input, index)?);
    }
    arguments.push(format!("literal:{}", plan.element_count));
    arguments.push(format!(
        "literal:{}",
        render_vulkan_output_layout_manifest(&plan.output_layouts)
    ));
    Ok(Some(PreparedProviderExecutionAdapter {
        executable_path: prepared.executable_path().to_owned(),
        executable_hash: prepared.executable_hash().to_owned(),
        runner_contract: prepared.contract(),
        cache_identity: prepared.cache_identity.to_owned(),
        cache_status: prepared.cache_status,
        arguments,
    }))
}

fn execute(
    input_evidence: &str,
    provider_family: &str,
    output_dir: &Path,
    request: &ProviderRequest,
    inputs: &[PreparedProviderInput],
    worker_receipt: &mut ProviderWorkerDispatchReceipt,
) -> Result<ProviderRequestExecution, String> {
    let plan =
        validate_vulkan_execution_session_plan(output_dir, provider_family, request, inputs.len())?;
    if worker_receipt.execution_capsule_invocation_mode
        != crate::provider_worker_lease::PROVIDER_WORKER_PROCESS_ADAPTER_CONTRACT
    {
        return Err("Vulkan SPIR-V provider requires its registered process adapter".to_owned());
    }
    let selection = parse_vulkan_worker_protocol(&worker_receipt.worker_output_payload)?;
    let planned_output_lengths = plan
        .output_layouts
        .iter()
        .map(|output| output.carrier_byte_length)
        .collect::<Vec<_>>();
    if selection.output_byte_lengths != planned_output_lengths {
        return Err("Vulkan adapter protocol outputs do not match the execution plan".to_owned());
    }
    let (mut summary, output_payload, transferable_output) =
        crate::provider_worker_native_execution::take_provider_worker_native_output(
            input_evidence,
            provider_family,
            request,
            worker_receipt,
        )?;
    summary.execution_contract = "nuis-vulkan-spirv-provider-execution-v1".to_owned();
    summary.execution_status = "vulkan-compute-dispatch-completed".to_owned();
    summary.device = format!(
        "vulkan:spirv-gpu:device-{}:queue-family-{}:api-{}",
        selection.selected_device_index,
        selection.selected_queue_family_index,
        selection.instance_api_version
    );
    Ok(ProviderRequestExecution {
        summary,
        output_payload,
        transferable_output,
        additional_outputs: Vec::new(),
        transport_receipts: Vec::new(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VulkanExecutionSessionPlan {
    contract: &'static str,
    status: &'static str,
    asset_id: String,
    asset_path: String,
    entry: String,
    element_count: u32,
    input_byte_length: usize,
    descriptor_set: u32,
    input_bindings: Vec<u32>,
    output_layouts: Vec<VulkanOutputLayout>,
    dispatch: [usize; 3],
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VulkanOutputLayout {
    binding: u32,
    logical_byte_length: usize,
    carrier_byte_length: usize,
    row_byte_length: usize,
    row_stride_bytes: usize,
    row_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VulkanWorkerProtocol {
    device_inventory_count: u32,
    selected_device_index: u32,
    selected_queue_family_index: u32,
    instance_api_version: u32,
    output_byte_lengths: Vec<usize>,
}

fn validate_vulkan_execution_session_plan(
    output_dir: &Path,
    provider_family: &str,
    request: &ProviderRequest,
    input_count: usize,
) -> Result<VulkanExecutionSessionPlan, String> {
    if provider_family != VULKAN_PROVIDER_FAMILY {
        return Err(format!(
            "Vulkan execution session only accepts `{VULKAN_PROVIDER_FAMILY}`"
        ));
    }
    let asset = request
        .code_asset
        .as_ref()
        .ok_or_else(|| "Vulkan provider request is missing its SPIR-V code asset".to_owned())?;
    let binding = request
        .adapter_binding
        .as_ref()
        .ok_or_else(|| "Vulkan provider request is missing its adapter binding".to_owned())?;
    let element_count = u32_scalar(request, "element_count")
        .ok_or_else(|| "Vulkan u32 request is missing u32 `element_count`".to_owned())?;
    let expected_bytes = usize::try_from(element_count)
        .ok()
        .and_then(|count| count.checked_mul(std::mem::size_of::<u32>()))
        .ok_or_else(|| "Vulkan u32 byte length overflows host size".to_owned())?;
    let dispatch = request
        .kernel
        .dispatch
        .as_slice()
        .try_into()
        .map_err(|_| "Vulkan u32 dispatch must have three dimensions".to_owned())?;
    let request_input_names = request
        .input_bindings
        .iter()
        .map(|binding| binding.name.as_str())
        .collect::<Vec<_>>();
    let output_layouts = request
        .output_bindings
        .iter()
        .enumerate()
        .map(|(index, output)| {
            vulkan_u32_output_layout(
                output,
                element_count,
                u32::try_from(input_count + index).ok()?,
            )
        })
        .collect::<Option<Vec<_>>>();
    if request.buffer.element_type != "u32"
        || !vulkan_dense_u32_layout(request)
        || request.buffer.byte_length != expected_bytes
        || request.kernel.input_buffer != request.buffer.id
        || request.kernel.input_buffers.len() != request_input_names.len()
        || request
            .kernel
            .input_buffers
            .iter()
            .map(String::as_str)
            .ne(request_input_names.iter().copied())
        || request.output_bindings.is_empty()
        || request.kernel.output_buffer != request.output_bindings[0].buffer
        || dispatch != [usize::try_from(element_count).unwrap_or(0), 1, 1]
        || !(1..=2).contains(&input_count)
        || request.input_bindings.len() != input_count
        || request.input_bindings.iter().any(|input| {
            !vulkan_input_source_is_supported(&input.source)
                || !crate::provider_input_binding::input_binding_matches_buffer(
                    input,
                    &request.buffer,
                )
        })
        || request.output_bindings.len() > 8
        || output_layouts.is_none()
        || binding.provider_family != VULKAN_PROVIDER_FAMILY
        || binding.execution_requirement != "real-device"
        || !asset_descriptor_matches_vulkan_u32(asset, &request.kernel.operation)
    {
        return Err("Vulkan provider request does not match the registered SPIR-V ABI".to_owned());
    }
    let asset_path =
        crate::provider_process_adapter::validate_provider_code_asset(output_dir, request)?;
    let bytes = fs::read(&asset_path).map_err(|error| {
        format!(
            "failed to read provider SPIR-V code asset `{}`: {error}",
            asset_path.display()
        )
    })?;
    let layout = validate_spirv_u32_module(asset, &bytes)?;
    let output_layouts = output_layouts.expect("validated Vulkan output layouts");
    let expected_input_bindings = (0..u32::try_from(input_count).unwrap_or(0)).collect::<Vec<_>>();
    let expected_output_bindings = (u32::try_from(input_count).unwrap_or(0)
        ..u32::try_from(input_count + request.output_bindings.len()).unwrap_or(0))
        .collect::<Vec<_>>();
    if layout.descriptor_set != 0
        || layout.input_bindings != expected_input_bindings
        || layout.output_bindings != expected_output_bindings
    {
        return Err(
            "Vulkan SPIR-V descriptor layout does not match the registered runner ABI".to_owned(),
        );
    }
    Ok(VulkanExecutionSessionPlan {
        contract: VULKAN_EXECUTION_SESSION_PLAN_CONTRACT,
        status: VULKAN_EXECUTION_SESSION_PLAN_STATUS,
        asset_id: asset.id.clone(),
        asset_path: asset_path.display().to_string(),
        entry: asset.entry.clone(),
        element_count,
        input_byte_length: expected_bytes,
        descriptor_set: layout.descriptor_set,
        input_bindings: layout.input_bindings,
        output_layouts,
        dispatch,
    })
}

fn vulkan_u32_output_layout(
    output: &crate::provider_output_binding::ProviderOutputBinding,
    element_count: u32,
    binding: u32,
) -> Option<VulkanOutputLayout> {
    if output.element_type != "u32" {
        return None;
    }
    let logical_element_count = output
        .shape
        .iter()
        .try_fold(1usize, |count, dimension| count.checked_mul(*dimension))?;
    let dispatch_element_count = usize::try_from(element_count).ok()?;
    if logical_element_count == 0 || logical_element_count > dispatch_element_count {
        return None;
    }
    let element_width = std::mem::size_of::<u32>();
    let logical_byte_length = logical_element_count.checked_mul(element_width)?;
    let (row_byte_length, row_stride_bytes, row_count) = match output.layout.as_str() {
        "tensor-contiguous" => (logical_byte_length, logical_byte_length, 1),
        "tensor-row-major" => {
            let [width, height] = output.shape.as_slice() else {
                return None;
            };
            (
                width.checked_mul(element_width)?,
                output.row_stride_bytes,
                *height,
            )
        }
        _ => return None,
    };
    if row_stride_bytes < row_byte_length
        || row_stride_bytes.checked_mul(row_count)? != output.byte_length
    {
        return None;
    }
    Some(VulkanOutputLayout {
        binding,
        logical_byte_length,
        carrier_byte_length: output.byte_length,
        row_byte_length,
        row_stride_bytes,
        row_count,
    })
}

#[cfg(any(target_os = "linux", test))]
fn render_vulkan_output_layout_manifest(outputs: &[VulkanOutputLayout]) -> String {
    outputs
        .iter()
        .map(|output| {
            format!(
                "{}:{}:{}:{}:{}",
                output.logical_byte_length,
                output.carrier_byte_length,
                output.row_byte_length,
                output.row_stride_bytes,
                output.row_count
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn vulkan_dense_u32_layout(request: &ProviderRequest) -> bool {
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

fn asset_descriptor_matches_vulkan_u32(
    asset: &ProviderCodeAssetDescriptor,
    operation: &str,
) -> bool {
    asset.format == VULKAN_SPIRV_FORMAT
        && asset.target == VULKAN_SPIRV_TARGET
        && vulkan_u32_entry_for_operation(operation).is_some_and(|entry| asset.entry == entry)
}

fn vulkan_u32_entry_for_operation(operation: &str) -> Option<String> {
    let stem = operation.strip_suffix("-u32")?;
    if stem.is_empty()
        || stem.starts_with('-')
        || stem.ends_with('-')
        || !stem
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return None;
    }
    Some(format!("nuis_vulkan_{}", operation.replace('-', "_")))
}

fn vulkan_input_source_is_supported(source: &str) -> bool {
    matches!(source, "artifact" | "dependency")
}

fn u32_scalar(request: &ProviderRequest, name: &str) -> Option<u32> {
    request
        .kernel
        .scalar_bindings
        .iter()
        .find(|binding| binding.name == name && binding.value_type == "u32")?
        .value
        .parse()
        .ok()
}

fn parse_vulkan_worker_protocol(protocol: &[u8]) -> Result<VulkanWorkerProtocol, String> {
    let protocol = std::str::from_utf8(protocol)
        .map_err(|_| "Vulkan adapter protocol is not UTF-8".to_owned())?;
    let mut fields = std::collections::BTreeMap::new();
    for line in protocol.lines() {
        let Some((key, value)) = line.split_once('=') else {
            return Err("Vulkan adapter protocol field is malformed".to_owned());
        };
        if fields.insert(key, value).is_some() {
            return Err(format!("Vulkan adapter protocol duplicates `{key}`"));
        }
    }
    let required = |key: &str| {
        fields
            .get(key)
            .copied()
            .ok_or_else(|| format!("Vulkan adapter protocol is missing `{key}`"))
    };
    if required("protocol")? != "nuis-vulkan-spirv-provider-runner-v1"
        || required("status")? != "ready"
        || required("device_inventory_contract")? != "nuis-vulkan-device-inventory-v1"
        || required("device_selection_contract")? != "nuis-vulkan-device-selection-v1"
        || required("device_selection_status")? != "verified"
    {
        return Err("Vulkan adapter protocol device selection is unverified".to_owned());
    }
    let parse_u32 = |key: &str| {
        required(key)?
            .parse::<u32>()
            .map_err(|error| format!("Vulkan adapter protocol `{key}` is invalid: {error}"))
    };
    let output_count = required("output_count")?
        .parse::<usize>()
        .map_err(|error| format!("Vulkan adapter protocol `output_count` is invalid: {error}"))?;
    let output_bytes = required("output_bytes")?
        .parse::<usize>()
        .map_err(|error| format!("Vulkan adapter protocol `output_bytes` is invalid: {error}"))?;
    let output_byte_lengths = required("output_byte_lengths")?
        .split(',')
        .map(|length| {
            length.parse::<usize>().map_err(|error| {
                format!("Vulkan adapter protocol output byte length is invalid: {error}")
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let evidence = VulkanWorkerProtocol {
        device_inventory_count: parse_u32("device_inventory_count")?,
        selected_device_index: parse_u32("selected_device_index")?,
        selected_queue_family_index: parse_u32("selected_queue_family_index")?,
        instance_api_version: parse_u32("instance_api_version")?,
        output_byte_lengths,
    };
    if evidence.device_inventory_count == 0
        || evidence.selected_device_index >= evidence.device_inventory_count
        || evidence.instance_api_version < (1 << 22)
        || !(1..=8).contains(&output_count)
        || evidence.output_byte_lengths.len() != output_count
        || evidence.output_byte_lengths.contains(&0)
        || evidence.output_byte_lengths[0] != output_bytes
    {
        return Err(
            "Vulkan adapter protocol device selection does not match the request".to_owned(),
        );
    }
    Ok(evidence)
}

#[cfg(test)]
#[path = "provider_execution_vulkan_tests.rs"]
mod tests;
