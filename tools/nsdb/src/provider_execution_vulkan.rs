use crate::{
    provider_code_asset::ProviderCodeAssetDescriptor,
    provider_execution_adapter::{
        ProviderExecutionAdapterRegistration, ProviderRequestExecution,
        PROVIDER_EXECUTION_ADAPTER_REGISTRY_CONTRACT,
    },
    provider_prepared_input::PreparedProviderInput,
    provider_request::ProviderRequest,
    provider_sample_payload::fnv1a64_hex,
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
const VULKAN_COPY_U32_OPERATION: &str = "copy-u32";
const VULKAN_COPY_U32_ENTRY: &str = "nuis_vulkan_copy_u32";
const VULKAN_SPIRV_FORMAT: &str = "spirv-binary";
const VULKAN_SPIRV_TARGET: &str = "vulkan1.3-spirv1.6";
const SPIRV_MAGIC: u32 = 0x0723_0203;
const SPIRV_VERSION_1_6: u32 = 0x0001_0600;

pub(crate) const REGISTRATION: ProviderExecutionAdapterRegistration =
    ProviderExecutionAdapterRegistration {
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
            format!("literal:{}", plan.element_count),
        ],
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
    let _plan =
        validate_vulkan_execution_session_plan(output_dir, provider_family, request, inputs.len())?;
    if worker_receipt.execution_capsule_invocation_mode
        != crate::provider_worker_lease::PROVIDER_WORKER_PROCESS_ADAPTER_CONTRACT
    {
        return Err("Vulkan SPIR-V provider requires its registered process adapter".to_owned());
    }
    let selection = parse_vulkan_worker_protocol(&worker_receipt.worker_output_payload)?;
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
    output_byte_length: usize,
    dispatch: [usize; 3],
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VulkanWorkerProtocol {
    device_inventory_count: u32,
    selected_device_index: u32,
    selected_queue_family_index: u32,
    instance_api_version: u32,
    output_bytes: usize,
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
        .ok_or_else(|| "Vulkan copy-u32 request is missing u32 `element_count`".to_owned())?;
    let expected_bytes = usize::try_from(element_count)
        .ok()
        .and_then(|count| count.checked_mul(std::mem::size_of::<u32>()))
        .ok_or_else(|| "Vulkan copy-u32 byte length overflows host size".to_owned())?;
    let dispatch = request
        .kernel
        .dispatch
        .as_slice()
        .try_into()
        .map_err(|_| "Vulkan copy-u32 dispatch must have three dimensions".to_owned())?;
    if request.buffer.element_type != "u32"
        || request.buffer.layout != "tensor-contiguous"
        || request.buffer.byte_length != expected_bytes
        || request.kernel.operation != VULKAN_COPY_U32_OPERATION
        || request.kernel.input_buffers != [request.buffer.id.clone()]
        || request.kernel.output_buffer != "output.values"
        || dispatch != [usize::try_from(element_count).unwrap_or(0), 1, 1]
        || input_count != 1
        || request.input_bindings.len() != 1
        || request.input_bindings[0].source != "artifact"
        || request.input_bindings[0].element_type != "u32"
        || request.input_bindings[0].byte_length != expected_bytes
        || request.output_bindings.len() != 1
        || request.output_bindings[0].element_type != "u32"
        || request.output_bindings[0].byte_length != expected_bytes
        || binding.provider_family != VULKAN_PROVIDER_FAMILY
        || binding.execution_requirement != "real-device"
        || !asset_descriptor_matches_vulkan_copy_u32(asset)
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
    validate_spirv_copy_u32_module(asset, &bytes)?;
    Ok(VulkanExecutionSessionPlan {
        contract: VULKAN_EXECUTION_SESSION_PLAN_CONTRACT,
        status: VULKAN_EXECUTION_SESSION_PLAN_STATUS,
        asset_id: asset.id.clone(),
        asset_path: asset_path.display().to_string(),
        entry: asset.entry.clone(),
        element_count,
        input_byte_length: expected_bytes,
        output_byte_length: expected_bytes,
        dispatch,
    })
}

fn asset_descriptor_matches_vulkan_copy_u32(asset: &ProviderCodeAssetDescriptor) -> bool {
    asset.format == VULKAN_SPIRV_FORMAT
        && asset.target == VULKAN_SPIRV_TARGET
        && asset.entry == VULKAN_COPY_U32_ENTRY
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

fn validate_spirv_copy_u32_module(
    asset: &ProviderCodeAssetDescriptor,
    bytes: &[u8],
) -> Result<(), String> {
    if bytes.len() < 20 || bytes.len() % 4 != 0 {
        return Err("Vulkan SPIR-V asset has invalid word alignment".to_owned());
    }
    let magic = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
    let version = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
    if magic != SPIRV_MAGIC || version != SPIRV_VERSION_1_6 {
        return Err("Vulkan SPIR-V asset has invalid module header".to_owned());
    }
    if fnv1a64_hex(bytes) != asset.content_hash {
        return Err("Vulkan SPIR-V asset hash evidence drifted".to_owned());
    }
    if !bytes
        .windows(asset.entry.len())
        .any(|window| window == asset.entry.as_bytes())
    {
        return Err(format!(
            "Vulkan SPIR-V asset is missing requested entry `{}`",
            asset.entry
        ));
    }
    Ok(())
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
    let output_bytes = required("output_bytes")?
        .parse::<usize>()
        .map_err(|error| format!("Vulkan adapter protocol `output_bytes` is invalid: {error}"))?;
    let evidence = VulkanWorkerProtocol {
        device_inventory_count: parse_u32("device_inventory_count")?,
        selected_device_index: parse_u32("selected_device_index")?,
        selected_queue_family_index: parse_u32("selected_queue_family_index")?,
        instance_api_version: parse_u32("instance_api_version")?,
        output_bytes,
    };
    if evidence.device_inventory_count == 0
        || evidence.selected_device_index >= evidence.device_inventory_count
        || evidence.instance_api_version < (1 << 22)
        || evidence.output_bytes == 0
    {
        return Err(
            "Vulkan adapter protocol device selection does not match the request".to_owned(),
        );
    }
    Ok(evidence)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        provider_adapter_binding::ProviderAdapterBinding,
        provider_code_asset::CODE_ASSET_FNV1A64_DIGEST_CONTRACT,
        provider_input_binding::ProviderInputBinding,
        provider_request::{
            ProviderBufferDescriptor, ProviderKernelDescriptor, ProviderOutputBinding,
            ProviderOutputComparisonDescriptor, ProviderRequest, ProviderScalarBinding,
        },
    };
    use std::env;

    #[test]
    fn vulkan_execution_registration_still_fails_closed_at_probe_boundary() {
        assert_eq!(
            REGISTRATION.registry_contract,
            PROVIDER_EXECUTION_ADAPTER_REGISTRY_CONTRACT
        );
        assert_eq!(REGISTRATION.adapter_kind, "vulkan-spirv-real-device-runner");
        assert!(REGISTRATION.requires_worker_descriptors);
        #[cfg(target_os = "linux")]
        assert!(REGISTRATION.prepare_worker_adapter.is_some());
        #[cfg(not(target_os = "linux"))]
        assert!(REGISTRATION.prepare_worker_adapter.is_none());
    }

    #[test]
    fn vulkan_session_plan_accepts_registered_copy_u32_shape() {
        let output_dir =
            env::temp_dir().join(format!("nsdb-vulkan-session-plan-{}", std::process::id()));
        let _ = fs::remove_dir_all(&output_dir);
        fs::create_dir_all(&output_dir).unwrap();
        let spirv = spirv_fixture();
        fs::write(output_dir.join("nuis.shader.vulkan.copy-u32.spv"), &spirv).unwrap();
        let request = copy_u32_request(&spirv);
        let plan = validate_vulkan_execution_session_plan(
            &output_dir,
            VULKAN_PROVIDER_FAMILY,
            &request,
            1,
        )
        .expect("validated Vulkan execution plan");
        fs::remove_dir_all(output_dir).unwrap();

        assert_eq!(plan.contract, VULKAN_EXECUTION_SESSION_PLAN_CONTRACT);
        assert_eq!(plan.status, VULKAN_EXECUTION_SESSION_PLAN_STATUS);
        assert_eq!(plan.asset_id, "shader.vulkan.copy-u32.spirv");
        assert_eq!(plan.entry, VULKAN_COPY_U32_ENTRY);
        assert_eq!(plan.element_count, 4);
        assert_eq!(plan.input_byte_length, 16);
        assert_eq!(plan.output_byte_length, 16);
        assert_eq!(plan.dispatch, [4, 1, 1]);
    }

    #[test]
    fn vulkan_session_plan_rejects_asset_or_binding_drift() {
        let output_dir =
            env::temp_dir().join(format!("nsdb-vulkan-session-drift-{}", std::process::id()));
        let _ = fs::remove_dir_all(&output_dir);
        fs::create_dir_all(&output_dir).unwrap();
        let spirv = spirv_fixture();
        fs::write(output_dir.join("nuis.shader.vulkan.copy-u32.spv"), &spirv).unwrap();
        let mut request = copy_u32_request(&spirv);
        request.code_asset.as_mut().expect("code asset").format = "metal-source".to_owned();
        assert!(validate_vulkan_execution_session_plan(
            &output_dir,
            VULKAN_PROVIDER_FAMILY,
            &request,
            1,
        )
        .unwrap_err()
        .contains("registered SPIR-V ABI"));
        let mut request = copy_u32_request(&spirv);
        request
            .adapter_binding
            .as_mut()
            .expect("adapter binding")
            .provider_family = "cuda:nvidia-gpu".to_owned();
        assert!(validate_vulkan_execution_session_plan(
            &output_dir,
            VULKAN_PROVIDER_FAMILY,
            &request,
            1,
        )
        .unwrap_err()
        .contains("registered SPIR-V ABI"));
        fs::remove_dir_all(output_dir).unwrap();
    }

    fn copy_u32_request(spirv: &[u8]) -> ProviderRequest {
        ProviderRequest {
            source: "test",
            buffer: ProviderBufferDescriptor {
                id: "input.values".to_owned(),
                element_type: "u32".to_owned(),
                layout: "tensor-contiguous".to_owned(),
                shape: vec![4],
                row_stride_bytes: 16,
                byte_length: 16,
                payload_path: "input.bin".to_owned(),
                content_hash: "0x1111111111111111".to_owned(),
            },
            kernel: ProviderKernelDescriptor {
                id: "shader.vulkan.copy-u32".to_owned(),
                operation: VULKAN_COPY_U32_OPERATION.to_owned(),
                input_buffer: "input.values".to_owned(),
                input_buffers: vec!["input.values".to_owned()],
                output_buffer: "output.values".to_owned(),
                dispatch: vec![4, 1, 1],
                scalar_bindings: vec![ProviderScalarBinding {
                    name: "element_count".to_owned(),
                    value_type: "u32".to_owned(),
                    value: "4".to_owned(),
                }],
            },
            output_bindings: vec![ProviderOutputBinding {
                role: "output.result".to_owned(),
                buffer: "output.values".to_owned(),
                element_type: "u32".to_owned(),
                shape: vec![4],
                byte_length: 16,
                comparison_id: "comparison.output.values".to_owned(),
            }],
            model_asset: None,
            code_asset: Some(ProviderCodeAssetDescriptor {
                id: "shader.vulkan.copy-u32.spirv".to_owned(),
                format: VULKAN_SPIRV_FORMAT.to_owned(),
                target: VULKAN_SPIRV_TARGET.to_owned(),
                entry: VULKAN_COPY_U32_ENTRY.to_owned(),
                path: "nuis.shader.vulkan.copy-u32.spv".to_owned(),
                byte_length: spirv.len(),
                digest_contract: CODE_ASSET_FNV1A64_DIGEST_CONTRACT.to_owned(),
                content_hash: fnv1a64_hex(spirv),
            }),
            output_comparison: Some(ProviderOutputComparisonDescriptor {
                id: "comparison.output.values".to_owned(),
                output_buffer: "output.values".to_owned(),
                element_type: "u32".to_owned(),
                shape: vec![4],
                expected_path: "expected.bin".to_owned(),
                expected_byte_length: 16,
                expected_content_hash: "0x2222222222222222".to_owned(),
                absolute_tolerance: "0".to_owned(),
                relative_tolerance: "0".to_owned(),
                non_finite_policy: "reject".to_owned(),
            }),
            output_comparisons: Vec::new(),
            dependencies: Vec::new(),
            input_bindings: vec![ProviderInputBinding {
                name: "input.values".to_owned(),
                source: "artifact".to_owned(),
                element_type: "u32".to_owned(),
                shape: vec![4],
                byte_length: 16,
                content_hash: "0x1111111111111111".to_owned(),
                payload_path: "input.bin".to_owned(),
                producer_request_id: "none".to_owned(),
                producer_output_buffer: "none".to_owned(),
            }],
            adapter_binding: Some(ProviderAdapterBinding {
                provider_family: VULKAN_PROVIDER_FAMILY.to_owned(),
                execution_requirement: "real-device".to_owned(),
            }),
        }
    }

    fn spirv_fixture() -> Vec<u8> {
        let mut bytes = [SPIRV_MAGIC, SPIRV_VERSION_1_6, 0, 21, 0]
            .into_iter()
            .flat_map(u32::to_le_bytes)
            .collect::<Vec<_>>();
        bytes.extend_from_slice(VULKAN_COPY_U32_ENTRY.as_bytes());
        while bytes.len() % 4 != 0 {
            bytes.push(0);
        }
        bytes
    }

    #[test]
    fn vulkan_worker_protocol_is_request_bound() {
        let protocol = b"protocol=nuis-vulkan-spirv-provider-runner-v1\nstatus=ready\ndevice_inventory_contract=nuis-vulkan-device-inventory-v1\ndevice_inventory_count=2\ndevice_selection_contract=nuis-vulkan-device-selection-v1\ndevice_selection_status=verified\nselected_device_index=0\nselected_queue_family_index=3\ninstance_api_version=4206592\noutput_bytes=16\noutput_hash=1\n";
        let evidence = parse_vulkan_worker_protocol(protocol).expect("selection evidence");
        assert_eq!(evidence.device_inventory_count, 2);
        assert_eq!(evidence.selected_device_index, 0);
        assert_eq!(evidence.selected_queue_family_index, 3);
        assert_eq!(evidence.output_bytes, 16);
        let drifted = String::from_utf8_lossy(protocol).replace("status=ready", "status=drift");
        assert!(parse_vulkan_worker_protocol(drifted.as_bytes()).is_err());
    }
}
