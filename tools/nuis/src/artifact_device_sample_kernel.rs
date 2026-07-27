use crate::artifact_device_sample_registration::DeviceSampleInputRegistration;
use std::{fs, path::Path};

const LEFT_FILE_NAME: &str = "nuis.kernel.cuda.vector-add.left.f32.bin";
const RIGHT_FILE_NAME: &str = "nuis.kernel.cuda.vector-add.right.f32.bin";
const EXPECTED_FILE_NAME: &str = "nuis.kernel.cuda.vector-add.expected.f32.bin";
const LEFT: &[u8] = &[
    0x00, 0x00, 0x80, 0x3f, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x40, 0x40, 0x00, 0x00, 0x80, 0x40,
];
const RIGHT: &[u8] = &[
    0x00, 0x00, 0x20, 0x41, 0x00, 0x00, 0xa0, 0x41, 0x00, 0x00, 0xf0, 0x41, 0x00, 0x00, 0x20, 0x42,
];
const EXPECTED: &[u8] = &[
    0x00, 0x00, 0x30, 0x41, 0x00, 0x00, 0xb0, 0x41, 0x00, 0x00, 0x04, 0x42, 0x00, 0x00, 0x30, 0x42,
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
    format!(
        "provider_buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1;provider_buffer_id=input.left;provider_buffer_element_type=f32;provider_buffer_layout=tensor-contiguous;provider_buffer_shape=4;provider_buffer_row_stride_bytes=16;provider_buffer_byte_length={};provider_buffer_payload_path={LEFT_FILE_NAME};provider_buffer_content_hash={};provider_kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1;provider_kernel_id=kernel.cuda.vector-add.f32;provider_kernel_operation=vector-add;provider_kernel_input_buffer=input.left;provider_kernel_input_buffers=input.left,input.right;provider_kernel_output_buffer=output.values;provider_kernel_dispatch=4x1x1;provider_kernel_scalar_bindings=element_count:u32:4;provider_code_asset_descriptor_contract=nuis-provider-code-asset-descriptor-v1;provider_code_asset_id={};provider_code_asset_format={};provider_code_asset_target={};provider_code_asset_entry={};provider_code_asset_path={};provider_code_asset_byte_length={};provider_code_asset_digest_contract={};provider_code_asset_content_hash={};provider_output_binding_contract=nuis-provider-output-binding-v1;provider_output_binding_count=1;provider_output_binding_0_role=output.result;provider_output_binding_0_buffer=output.values;provider_output_binding_0_element_type=f32;provider_output_binding_0_shape=4;provider_output_binding_0_byte_length={};provider_output_binding_0_comparison_id=comparison.output.values;provider_output_comparison_id=comparison.output.values;provider_output_comparison_descriptor_contract=nuis-provider-output-comparison-descriptor-v1;provider_output_comparison_output_buffer=output.values;provider_output_comparison_element_type=f32;provider_output_comparison_shape=4;provider_output_comparison_expected_path={EXPECTED_FILE_NAME};provider_output_comparison_expected_byte_length={};provider_output_comparison_expected_content_hash={};provider_output_comparison_absolute_tolerance=0;provider_output_comparison_relative_tolerance=0;provider_output_comparison_non_finite_policy=reject;provider_input_binding_contract=nuis-provider-input-binding-v1;provider_input_binding_count=2;provider_input_binding_0_name=input.left;provider_input_binding_0_source=artifact;provider_input_binding_0_element_type=f32;provider_input_binding_0_shape=4;provider_input_binding_0_byte_length={};provider_input_binding_0_content_hash={};provider_input_binding_0_payload_path={LEFT_FILE_NAME};provider_input_binding_0_producer_request_id=none;provider_input_binding_0_producer_output_buffer=none;provider_input_binding_1_name=input.right;provider_input_binding_1_source=artifact;provider_input_binding_1_element_type=f32;provider_input_binding_1_shape=4;provider_input_binding_1_byte_length={};provider_input_binding_1_content_hash={};provider_input_binding_1_payload_path={RIGHT_FILE_NAME};provider_input_binding_1_producer_request_id=none;provider_input_binding_1_producer_output_buffer=none;provider_adapter_binding_contract=nuis-provider-request-adapter-binding-v1;provider_adapter_binding_provider_family=cuda:nvidia-gpu;provider_adapter_binding_execution_requirement=real-device",
        LEFT.len(),
        fnv1a64_hex(LEFT),
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
    fn registration_owns_cuda_vector_add_request_roles() {
        let registration = registration();
        let evidence = (registration.enrich_evidence)("ignored");
        assert_eq!(registration.package_id, "official.kernel");
        assert!((registration.supports)("cuda", "nvidia-gpu"));
        assert!(evidence.contains("provider_kernel_input_buffers=input.left,input.right"));
        assert!(evidence.contains("provider_code_asset_format=ptx"));
        assert!(evidence.contains("provider_code_asset_entry=nuis_kernel_vector_add_f32"));
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
        fs::remove_dir_all(output_dir).unwrap();
    }
}
