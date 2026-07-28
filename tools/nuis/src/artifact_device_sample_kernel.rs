use crate::artifact_device_sample_registration::DeviceSampleInputRegistration;
use std::{fs, path::Path};

const LEFT_FILE_NAME: &str = "nuis.kernel.cuda.vector-add.left.f32.bin";
const RIGHT_FILE_NAME: &str = "nuis.kernel.cuda.vector-add.right.f32.bin";
const EXPECTED_FILE_NAME: &str = "nuis.kernel.cuda.vector-add.expected.f32.bin";
const SCALED_EXPECTED_FILE_NAME: &str = "nuis.kernel.cuda.scale.expected.f32.bin";
const VECTOR_ADD_KERNEL_ID: &str = "kernel.cuda.vector-add.f32";
const SCALE_KERNEL_ID: &str = "kernel.cuda.scale.f32";
const SCALE_ENTRY: &str = "nuis_kernel_scale_f32";
const CUDA_DEVICE_SELECTION_REGISTRY_CONTRACT: &str = "nuis-cuda-device-selection-registry-v1";
const CUDA_DEVICE_SELECTION_CONTRACT: &str = "nuis-cuda-device-selection-v1";
const CUDA_DEVICE_INVENTORY_CONTRACT: &str = "nuis-cuda-device-inventory-v1";
const CUDA_DEVICE_SELECTION_POLICY: &str = "capability-ranked-lowest-ordinal";
const CUDA_DEVICE_SELECTION_POLICY_CODE: u32 = 1;
const LEFT: &[u8] = &[
    0x00, 0x00, 0x80, 0x3f, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x40, 0x40, 0x00, 0x00, 0x80, 0x40,
];
const RIGHT: &[u8] = &[
    0x00, 0x00, 0x20, 0x41, 0x00, 0x00, 0xa0, 0x41, 0x00, 0x00, 0xf0, 0x41, 0x00, 0x00, 0x20, 0x42,
];
const EXPECTED: &[u8] = &[
    0x00, 0x00, 0x30, 0x41, 0x00, 0x00, 0xb0, 0x41, 0x00, 0x00, 0x04, 0x42, 0x00, 0x00, 0x30, 0x42,
];
const SCALED_EXPECTED: &[u8] = &[
    0x00, 0x00, 0xb0, 0x41, 0x00, 0x00, 0x30, 0x42, 0x00, 0x00, 0x84, 0x42, 0x00, 0x00, 0xb0, 0x42,
];

pub(crate) fn registration() -> DeviceSampleInputRegistration {
    DeviceSampleInputRegistration {
        package_id: "official.kernel",
        supports: supports_cuda,
        enrich_evidence: cuda_vector_add_evidence,
        persist_payloads: persist_cuda_vector_add_payloads,
    }
}

fn supports_cuda(backend_family: &str, target_device: &str) -> bool {
    backend_family == "cuda" && target_device == "nvidia-gpu"
}

fn cuda_vector_add_evidence(_base: &str) -> String {
    let asset = nuisc::kernel_code_asset::select_kernel_code_asset("cuda.nvidia-gpu")
        .expect("Kernel Nustar CUDA target must own a code asset");
    let singular = vector_add_request("provider_", asset);
    let vector_add = vector_add_request("provider_request_0_", asset);
    let scale = scale_request("provider_request_1_", asset);
    format!(
        "cuda_device_selection_registry_contract={CUDA_DEVICE_SELECTION_REGISTRY_CONTRACT};cuda_device_inventory_contract={CUDA_DEVICE_INVENTORY_CONTRACT};cuda_device_selection_contract={CUDA_DEVICE_SELECTION_CONTRACT};cuda_device_selection_policy={CUDA_DEVICE_SELECTION_POLICY};cuda_device_selection_policy_code={CUDA_DEVICE_SELECTION_POLICY_CODE};cuda_device_selection_minimum_compute_capability={};{singular};provider_request_collection_contract=nuis-provider-request-collection-v1;provider_request_count=2;{vector_add};{scale}",
        asset.minimum_compute_capability,
    )
}

fn vector_add_request(
    prefix: &str,
    asset: &nuisc::kernel_code_asset::RegisteredKernelCodeAsset,
) -> String {
    format!(
        "{prefix}buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1;{prefix}buffer_id=input.left;{prefix}buffer_element_type=f32;{prefix}buffer_layout=tensor-contiguous;{prefix}buffer_shape=4;{prefix}buffer_row_stride_bytes=16;{prefix}buffer_byte_length={};{prefix}buffer_payload_path={LEFT_FILE_NAME};{prefix}buffer_content_hash={};{prefix}kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1;{prefix}kernel_id={VECTOR_ADD_KERNEL_ID};{prefix}kernel_operation=vector-add;{prefix}kernel_input_buffer=input.left;{prefix}kernel_input_buffers=input.left,input.right;{prefix}kernel_output_buffer=output.values;{prefix}kernel_dispatch=4x1x1;{prefix}kernel_scalar_bindings=element_count:u32:4,device_selection_policy:u32:{CUDA_DEVICE_SELECTION_POLICY_CODE},minimum_compute_capability:u32:{};{prefix}code_asset_descriptor_contract=nuis-provider-code-asset-descriptor-v1;{prefix}code_asset_id={};{prefix}code_asset_format={};{prefix}code_asset_target={};{prefix}code_asset_entry={};{prefix}code_asset_path={};{prefix}code_asset_byte_length={};{prefix}code_asset_digest_contract={};{prefix}code_asset_content_hash={};{prefix}output_binding_contract=nuis-provider-output-binding-v1;{prefix}output_binding_count=1;{prefix}output_binding_0_role=output.result;{prefix}output_binding_0_buffer=output.values;{prefix}output_binding_0_element_type=f32;{prefix}output_binding_0_shape=4;{prefix}output_binding_0_byte_length={};{prefix}output_binding_0_comparison_id=comparison.output.values;{prefix}output_comparison_id=comparison.output.values;{prefix}output_comparison_descriptor_contract=nuis-provider-output-comparison-descriptor-v1;{prefix}output_comparison_output_buffer=output.values;{prefix}output_comparison_element_type=f32;{prefix}output_comparison_shape=4;{prefix}output_comparison_expected_path={EXPECTED_FILE_NAME};{prefix}output_comparison_expected_byte_length={};{prefix}output_comparison_expected_content_hash={};{prefix}output_comparison_absolute_tolerance=0;{prefix}output_comparison_relative_tolerance=0;{prefix}output_comparison_non_finite_policy=reject;{prefix}dependency_contract=nuis-provider-request-dependency-v1;{prefix}dependency_count=0;{prefix}input_binding_contract=nuis-provider-input-binding-v1;{prefix}input_binding_count=2;{prefix}input_binding_0_name=input.left;{prefix}input_binding_0_source=artifact;{prefix}input_binding_0_element_type=f32;{prefix}input_binding_0_shape=4;{prefix}input_binding_0_byte_length={};{prefix}input_binding_0_content_hash={};{prefix}input_binding_0_payload_path={LEFT_FILE_NAME};{prefix}input_binding_0_producer_request_id=none;{prefix}input_binding_0_producer_output_buffer=none;{prefix}input_binding_1_name=input.right;{prefix}input_binding_1_source=artifact;{prefix}input_binding_1_element_type=f32;{prefix}input_binding_1_shape=4;{prefix}input_binding_1_byte_length={};{prefix}input_binding_1_content_hash={};{prefix}input_binding_1_payload_path={RIGHT_FILE_NAME};{prefix}input_binding_1_producer_request_id=none;{prefix}input_binding_1_producer_output_buffer=none;{prefix}adapter_binding_contract=nuis-provider-request-adapter-binding-v1;{prefix}adapter_binding_provider_family=cuda:nvidia-gpu;{prefix}adapter_binding_execution_requirement=real-device",
        LEFT.len(),
        fnv1a64_hex(LEFT),
        asset.minimum_compute_capability,
        asset.id,
        asset.format,
        asset.target,
        asset.entry,
        asset.file_name,
        asset.bytes.len(),
        asset.digest_contract,
        fnv1a64_hex(asset.bytes),
        EXPECTED.len(),
        EXPECTED.len(),
        fnv1a64_hex(EXPECTED),
        LEFT.len(),
        fnv1a64_hex(LEFT),
        RIGHT.len(),
        fnv1a64_hex(RIGHT),
    )
}

fn scale_request(
    prefix: &str,
    asset: &nuisc::kernel_code_asset::RegisteredKernelCodeAsset,
) -> String {
    format!(
        "{prefix}buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1;{prefix}buffer_id=input.values;{prefix}buffer_element_type=f32;{prefix}buffer_layout=tensor-contiguous;{prefix}buffer_shape=4;{prefix}buffer_row_stride_bytes=16;{prefix}buffer_byte_length={};{prefix}buffer_payload_path={EXPECTED_FILE_NAME};{prefix}buffer_content_hash={};{prefix}kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1;{prefix}kernel_id={SCALE_KERNEL_ID};{prefix}kernel_operation=scale;{prefix}kernel_input_buffer=input.values;{prefix}kernel_output_buffer=output.scaled;{prefix}kernel_dispatch=4x1x1;{prefix}kernel_scalar_bindings=element_count:u32:4,scale:f32:2,device_selection_policy:u32:{CUDA_DEVICE_SELECTION_POLICY_CODE},minimum_compute_capability:u32:{};{prefix}code_asset_descriptor_contract=nuis-provider-code-asset-descriptor-v1;{prefix}code_asset_id={};{prefix}code_asset_format={};{prefix}code_asset_target={};{prefix}code_asset_entry={SCALE_ENTRY};{prefix}code_asset_path={};{prefix}code_asset_byte_length={};{prefix}code_asset_digest_contract={};{prefix}code_asset_content_hash={};{prefix}output_binding_contract=nuis-provider-output-binding-v1;{prefix}output_binding_count=1;{prefix}output_binding_0_role=output.scaled;{prefix}output_binding_0_buffer=output.scaled;{prefix}output_binding_0_element_type=f32;{prefix}output_binding_0_shape=4;{prefix}output_binding_0_byte_length={};{prefix}output_binding_0_comparison_id=comparison.output.scaled;{prefix}output_comparison_id=comparison.output.scaled;{prefix}output_comparison_descriptor_contract=nuis-provider-output-comparison-descriptor-v1;{prefix}output_comparison_output_buffer=output.scaled;{prefix}output_comparison_element_type=f32;{prefix}output_comparison_shape=4;{prefix}output_comparison_expected_path={SCALED_EXPECTED_FILE_NAME};{prefix}output_comparison_expected_byte_length={};{prefix}output_comparison_expected_content_hash={};{prefix}output_comparison_absolute_tolerance=0;{prefix}output_comparison_relative_tolerance=0;{prefix}output_comparison_non_finite_policy=reject;{prefix}dependency_contract=nuis-provider-request-dependency-v1;{prefix}dependency_count=1;{prefix}dependency_0_producer_request_id={VECTOR_ADD_KERNEL_ID};{prefix}dependency_0_producer_output_buffer=output.values;{prefix}dependency_0_consumer_input_buffer=input.values;{prefix}dependency_0_transport_contract=nuis-provider-edge-transport-v1;{prefix}dependency_0_transport_ownership_token=glm:provider-edge:{VECTOR_ADD_KERNEL_ID}:output.values->{SCALE_KERNEL_ID}:input.values;{prefix}dependency_0_transport_staging_mode=auto;{prefix}dependency_0_transport_producer_clock_evidence=provider-clock:request-0:completed;{prefix}dependency_0_transport_consumer_clock_evidence=provider-clock:request-1:dispatch-ready;{prefix}input_binding_contract=nuis-provider-input-binding-v1;{prefix}input_binding_count=1;{prefix}input_binding_0_name=input.values;{prefix}input_binding_0_source=dependency;{prefix}input_binding_0_element_type=f32;{prefix}input_binding_0_shape=4;{prefix}input_binding_0_byte_length={};{prefix}input_binding_0_content_hash={};{prefix}input_binding_0_payload_path=none;{prefix}input_binding_0_producer_request_id={VECTOR_ADD_KERNEL_ID};{prefix}input_binding_0_producer_output_buffer=output.values;{prefix}adapter_binding_contract=nuis-provider-request-adapter-binding-v1;{prefix}adapter_binding_provider_family=cuda:nvidia-gpu;{prefix}adapter_binding_execution_requirement=real-device",
        EXPECTED.len(),
        fnv1a64_hex(EXPECTED),
        asset.minimum_compute_capability,
        asset.id,
        asset.format,
        asset.target,
        asset.file_name,
        asset.bytes.len(),
        asset.digest_contract,
        fnv1a64_hex(asset.bytes),
        SCALED_EXPECTED.len(),
        SCALED_EXPECTED.len(),
        fnv1a64_hex(SCALED_EXPECTED),
        EXPECTED.len(),
        fnv1a64_hex(EXPECTED),
    )
}

fn persist_cuda_vector_add_payloads(output_dir: &Path, evidence: &[&str]) -> Result<(), String> {
    if !evidence
        .iter()
        .any(|item| item.contains("provider_sample_registration_package=official.kernel"))
    {
        return Ok(());
    }
    let asset = nuisc::kernel_code_asset::select_kernel_code_asset("cuda.nvidia-gpu")
        .ok_or_else(|| "Kernel Nustar CUDA code asset is not registered".to_owned())?;
    let actual = fs::read(output_dir.join(asset.file_name))
        .map_err(|error| format!("failed to read Nuis-emitted CUDA PTX asset: {error}"))?;
    if actual != asset.bytes {
        return Err("Nuis-emitted CUDA PTX asset does not match its registry bytes".to_owned());
    }
    for (name, bytes) in [
        (LEFT_FILE_NAME, LEFT),
        (RIGHT_FILE_NAME, RIGHT),
        (EXPECTED_FILE_NAME, EXPECTED),
        (SCALED_EXPECTED_FILE_NAME, SCALED_EXPECTED),
    ] {
        fs::write(output_dir.join(name), bytes)
            .map_err(|error| format!("failed to persist CUDA vector-add payload: {error}"))?;
    }
    Ok(())
}

fn fnv1a64_hex(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("0x{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn registration_owns_ordered_cuda_request_graph() {
        let registration = registration();
        let evidence = (registration.enrich_evidence)("ignored");
        assert_eq!(registration.package_id, "official.kernel");
        assert!((registration.supports)("cuda", "nvidia-gpu"));
        assert!(evidence.contains("provider_kernel_input_buffers=input.left,input.right"));
        assert!(evidence.contains("provider_code_asset_format=ptx"));
        assert!(evidence.contains("provider_code_asset_entry=nuis_kernel_vector_add_f32"));
        assert!(evidence.contains("provider_request_count=2"));
        assert!(evidence.contains("cuda_device_inventory_contract=nuis-cuda-device-inventory-v1"));
        assert!(evidence.contains("cuda_device_selection_contract=nuis-cuda-device-selection-v1"));
        assert!(evidence.contains("cuda_device_selection_policy=capability-ranked-lowest-ordinal"));
        assert!(evidence.contains("provider_kernel_scalar_bindings=element_count:u32:4,device_selection_policy:u32:1,minimum_compute_capability:u32:80"));
        assert!(evidence.contains("provider_request_1_code_asset_entry=nuis_kernel_scale_f32"));
        assert!(evidence.contains(
            "provider_request_1_kernel_scalar_bindings=element_count:u32:4,scale:f32:2,device_selection_policy:u32:1,minimum_compute_capability:u32:80"
        ));
        assert!(evidence.contains("provider_request_1_input_binding_0_source=dependency"));
        assert!(evidence.contains(
            "provider_request_1_dependency_0_transport_ownership_token=glm:provider-edge:kernel.cuda.vector-add.f32:output.values->kernel.cuda.scale.f32:input.values"
        ));
        assert!(evidence.contains("provider_input_binding_count=2"));
        assert!(evidence.contains("provider_output_binding_0_role=output.result"));
        assert!(evidence.contains("provider_adapter_binding_provider_family=cuda:nvidia-gpu"));
        assert!(nsdb::validate_provider_request_evidence(&evidence));
    }

    #[test]
    fn registration_verifies_ptx_before_persisting_inputs() {
        let output_dir = env::temp_dir().join(format!(
            "nuis-kernel-cuda-provider-registration-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&output_dir);
        fs::create_dir_all(&output_dir).unwrap();
        let asset = nuisc::kernel_code_asset::select_kernel_code_asset("cuda.nvidia-gpu").unwrap();
        fs::write(output_dir.join(asset.file_name), asset.bytes).unwrap();
        let evidence = format!("provider_sample_registration_package=official.kernel");
        persist_cuda_vector_add_payloads(&output_dir, &[&evidence]).unwrap();
        assert_eq!(fs::read(output_dir.join(LEFT_FILE_NAME)).unwrap(), LEFT);
        assert_eq!(fs::read(output_dir.join(RIGHT_FILE_NAME)).unwrap(), RIGHT);
        assert_eq!(
            fs::read(output_dir.join(EXPECTED_FILE_NAME)).unwrap(),
            EXPECTED
        );
        assert_eq!(
            fs::read(output_dir.join(SCALED_EXPECTED_FILE_NAME)).unwrap(),
            SCALED_EXPECTED
        );
        fs::remove_dir_all(output_dir).unwrap();
    }
}
