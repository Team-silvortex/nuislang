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
        registration_id: "official.kernel.cuda-vector",
        provider_family: "cuda:nvidia-gpu",
        supports: supports_cuda,
        metadata_selector: None,
        enrich_evidence: cuda_vector_add_evidence,
        resolve_evidence: Some(resolve_cuda_code_asset_evidence),
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
    let uses_project_collection =
        crate::artifact_device_sample_kernel_project::uses_project_request_collection(output_dir)?;
    validate_cuda_code_asset(asset, &actual, !uses_project_collection)?;
    if !uses_project_collection {
        for (name, bytes) in [
            (LEFT_FILE_NAME, LEFT),
            (RIGHT_FILE_NAME, RIGHT),
            (EXPECTED_FILE_NAME, EXPECTED),
            (SCALED_EXPECTED_FILE_NAME, SCALED_EXPECTED),
        ] {
            fs::write(output_dir.join(name), bytes)
                .map_err(|error| format!("failed to persist CUDA vector-add payload: {error}"))?;
        }
    }
    crate::artifact_device_sample_kernel_project::persist_payloads(output_dir)?;
    Ok(())
}

fn resolve_cuda_code_asset_evidence(output_dir: &Path, evidence: &str) -> Result<String, String> {
    let asset = nuisc::kernel_code_asset::select_kernel_code_asset("cuda.nvidia-gpu")
        .ok_or_else(|| "Kernel Nustar CUDA code asset is not registered".to_owned())?;
    let actual = fs::read(output_dir.join(asset.file_name))
        .map_err(|error| format!("failed to read Nuis-emitted CUDA PTX asset: {error}"))?;
    let uses_project_collection =
        crate::artifact_device_sample_kernel_project::uses_project_request_collection(output_dir)?;
    validate_cuda_code_asset(asset, &actual, !uses_project_collection)?;
    let evidence = crate::artifact_device_sample_kernel_project::augment_evidence(
        output_dir, evidence, asset, &actual,
    )?;
    validate_cuda_request_asset_evidence(asset, &actual, &evidence)?;
    let requested_entries = code_asset_entries(&evidence);
    let selection =
        crate::artifact_code_asset_contribution_table::select_compiled_code_asset_contribution(
            output_dir,
            "official.kernel",
            "kernel",
            "cuda.nvidia-gpu",
            asset.format,
            asset.target,
            &requested_entries,
        )?;
    let selection_evidence = selection
        .as_ref()
        .map(|selection| -> Result<String, String> {
            validate_cuda_contribution_selection(selection, &evidence)?;
            Ok(crate::artifact_code_asset_contribution_table::render_selected_contribution_evidence(
                selection,
            ))
        })
        .transpose()?;
    let byte_length = actual.len().to_string();
    let content_hash = fnv1a64_hex(&actual);
    let mut resolved = evidence
        .split(';')
        .map(|field| {
            let Some((key, value)) = field.split_once('=') else {
                return field.to_owned();
            };
            if key.ends_with("_code_asset_byte_length") {
                format!("{key}={byte_length}")
            } else if key.ends_with("_code_asset_content_hash") {
                format!("{key}={content_hash}")
            } else {
                format!("{key}={value}")
            }
        })
        .collect::<Vec<_>>()
        .join(";");
    if let Some(selection_evidence) = selection_evidence {
        resolved.push(';');
        resolved.push_str(&selection_evidence);
    }
    Ok(resolved)
}

fn code_asset_entries(evidence: &str) -> Vec<String> {
    let indexed = evidence
        .split(';')
        .filter_map(|field| field.split_once('='))
        .filter(|(key, _)| {
            key.starts_with("provider_request_") && key.ends_with("_code_asset_entry")
        })
        .map(|(_, value)| value.to_owned())
        .collect::<Vec<_>>();
    if indexed.is_empty() {
        evidence
            .split(';')
            .filter_map(|field| field.split_once('='))
            .filter(|(key, _)| *key == "provider_code_asset_entry")
            .map(|(_, value)| value.to_owned())
            .collect()
    } else {
        indexed
    }
}

fn validate_cuda_contribution_selection(
    selection: &crate::artifact_code_asset_contribution_table::SelectedCodeAssetContribution,
    evidence: &str,
) -> Result<(), String> {
    let request_fields = evidence
        .split(';')
        .filter_map(|field| field.split_once('='))
        .filter(|(key, _)| key.contains("_code_asset_"))
        .collect::<Vec<_>>();
    for (suffix, expected) in [
        ("_code_asset_id", selection.asset_id.as_str()),
        ("_code_asset_format", selection.format.as_str()),
        ("_code_asset_target", selection.target.as_str()),
        ("_code_asset_path", selection.path.as_str()),
        (
            "_code_asset_byte_length",
            &selection.byte_length.to_string(),
        ),
        ("_code_asset_content_hash", selection.content_hash.as_str()),
    ] {
        if request_fields
            .iter()
            .filter(|(key, _)| key.ends_with(suffix))
            .any(|(_, value)| *value != expected)
        {
            return Err(format!(
                "CUDA provider request does not match compiled contribution field `{suffix}`"
            ));
        }
    }
    Ok(())
}

fn validate_cuda_request_asset_evidence(
    asset: &nuisc::kernel_code_asset::RegisteredKernelCodeAsset,
    bytes: &[u8],
    evidence: &str,
) -> Result<(), String> {
    let ptx = std::str::from_utf8(bytes)
        .map_err(|_| "Nuis-emitted CUDA PTX asset is not UTF-8".to_owned())?;
    let mut entry_count = 0usize;
    for field in evidence.split(';') {
        let Some((key, value)) = field.split_once('=') else {
            continue;
        };
        if key.ends_with("_code_asset_format") && value != asset.format {
            return Err(format!(
                "CUDA provider request code asset format `{value}` does not match `{}`",
                asset.format
            ));
        }
        if key.ends_with("_code_asset_target") && value != asset.target {
            return Err(format!(
                "CUDA provider request code asset target `{value}` does not match `{}`",
                asset.target
            ));
        }
        if key.ends_with("_code_asset_entry") {
            entry_count += 1;
            if !ptx.contains(&format!(".visible .entry {value}(")) {
                return Err(format!(
                    "Nuis-emitted CUDA PTX asset is missing requested entry `{value}`"
                ));
            }
        }
    }
    if entry_count == 0 {
        return Err("CUDA provider request evidence has no code asset entry".to_owned());
    }
    Ok(())
}

fn validate_cuda_code_asset(
    asset: &nuisc::kernel_code_asset::RegisteredKernelCodeAsset,
    bytes: &[u8],
    require_registered_entries: bool,
) -> Result<(), String> {
    let ptx = std::str::from_utf8(bytes)
        .map_err(|_| "Nuis-emitted CUDA PTX asset is not UTF-8".to_owned())?;
    if !ptx.lines().any(|line| line.trim() == ".version 8.0") {
        return Err("Nuis-emitted CUDA PTX asset has an unsupported PTX version".to_owned());
    }
    if !ptx
        .lines()
        .any(|line| line.trim() == format!(".target {}", asset.target))
    {
        return Err(format!(
            "Nuis-emitted CUDA PTX asset does not target `{}`",
            asset.target
        ));
    }
    if require_registered_entries {
        for entry in asset.visible_entries {
            let declaration = format!(".visible .entry {entry}(");
            if !ptx.contains(&declaration) {
                return Err(format!(
                    "Nuis-emitted CUDA PTX asset is missing registered entry `{entry}`"
                ));
            }
        }
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
        assert!(evidence.contains("provider_code_asset_id=kernel.vector-arithmetic.f32.cuda.ptx"));
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
        let evidence = "provider_sample_registration_package=official.kernel".to_owned();
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
        let missing_registered_entry = std::str::from_utf8(asset.bytes)
            .unwrap()
            .replace("nuis_kernel_scale_f32", "nuis_kernel_scale_missing");
        fs::write(output_dir.join(asset.file_name), missing_registered_entry).unwrap();
        assert!(persist_cuda_vector_add_payloads(&output_dir, &[&evidence])
            .unwrap_err()
            .contains("missing registered entry `nuis_kernel_scale_f32`"));
        fs::remove_dir_all(output_dir).unwrap();
    }

    #[test]
    fn registration_binds_provider_evidence_to_project_derived_ptx() {
        let output_dir = env::temp_dir().join(format!(
            "nuis-kernel-cuda-derived-provider-registration-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&output_dir);
        fs::create_dir_all(&output_dir).unwrap();
        let asset = nuisc::kernel_code_asset::select_kernel_code_asset("cuda.nvidia-gpu").unwrap();
        let derived = b".version 8.0\n.target sm_80\n.address_size 64\n\n\
.visible .entry nuis_project_main_mapped_i64()\n{\n    ret;\n}\n\
.visible .entry nuis_project_main_reduced_i64()\n{\n    ret;\n}\n"
            .to_vec();
        fs::write(output_dir.join(asset.file_name), &derived).unwrap();
        fs::write(
            output_dir.join("nuis.domain.kernel.codegen-table.toml"),
            r#"schema = "nuis-kernel-yir-codegen-table-v1"
source_fnv1a64 = "0x0123456789abcdef"
lowering_target = "cuda.nvidia-gpu"
function_count = 2
project_code_asset_identity_contract = "nuis-kernel-project-code-asset-identity-v1"
project_code_asset_id = "kernel.cuda.project.7519a228f04318e8"
project_code_asset_source_fnv1a64 = "0x0123456789abcdef"
project_code_asset_lowering_target = "cuda.nvidia-gpu"
project_code_asset_entry_count = 2
project_code_asset_entries = ["nuis_project_main_mapped_i64", "nuis_project_main_reduced_i64"]
project_code_asset_identity_hash = "0x7519a228f04318e8"
project_code_asset_identity_set_contract = "nuis-provider-code-asset-identity-set-v1"
project_code_asset_identity_set_count = 1
project_code_asset_identity_set_asset_ids = ["kernel.cuda.project.7519a228f04318e8"]
project_code_asset_identity_set_contracts = ["nuis-kernel-project-code-asset-identity-v1"]
project_code_asset_identity_set_hashes = ["0x7519a228f04318e8"]
project_code_asset_identity_set_root_hash = "0x7724b43039b33d4a"

[[source_adaptation]]
contract = "nuis-kernel-yir-source-adapter-v1"
source_function = "main"
source_node = "mapped"
source_instruction = "add_scalar_axis"
status = "adapted"
generated_entry = "nuis_project_main_mapped_i64"
request_projection_contract = "nuis-kernel-source-request-projection-v1"
request_operation = "add-scalar-i64"
request_element_type = "i64"
request_input_shape = [1, 4]
request_output_shape = [1, 4]
request_input_values = [1, 2, 3, 4]
request_scalar = 10
request_expected_values = [11, 12, 13, 14]
diagnostic = "verified"

[[source_adaptation]]
contract = "nuis-kernel-yir-source-adapter-v1"
source_function = "main"
source_node = "reduced"
source_instruction = "reduce_sum_axis"
status = "adapted"
generated_entry = "nuis_project_main_reduced_i64"
request_projection_contract = "nuis-kernel-source-request-projection-v1"
request_operation = "reduce-sum-i64"
request_element_type = "i64"
request_input_shape = [1, 4]
request_output_shape = [1, 1]
request_input_values = [11, 12, 13, 14]
request_input_source_node = "mapped"
request_expected_values = [50]
diagnostic = "verified dependency"

[[source_adaptation]]
contract = "nuis-kernel-yir-source-adapter-v1"
source_function = "main"
source_node = "selected"
source_instruction = "element_at"
status = "projected"
result_projection_contract = "nuis-kernel-source-result-projection-v1"
result_element_type = "i64"
result_input_source_node = "reduced"
result_row = 0
result_col = 0
result_expected_i64 = 50
diagnostic = "verified result"
"#,
        )
        .unwrap();

        let evidence = cuda_vector_add_evidence("ignored");
        persist_cuda_vector_add_payloads(
            &output_dir,
            &["provider_sample_registration_package=official.kernel"],
        )
        .unwrap();
        let resolved = resolve_cuda_code_asset_evidence(&output_dir, &evidence).unwrap();
        let expected_length = format!(
            "provider_request_0_code_asset_byte_length={}",
            derived.len()
        );
        let expected_hash = format!(
            "provider_request_0_code_asset_content_hash={}",
            fnv1a64_hex(&derived)
        );
        assert!(resolved.contains("provider_request_count=2"));
        assert!(resolved
            .contains("provider_request_0_code_asset_id=kernel.cuda.project.7519a228f04318e8"));
        assert!(resolved
            .contains("provider_request_1_code_asset_id=kernel.cuda.project.7519a228f04318e8"));
        assert!(resolved.contains(
            "provider_code_asset_identity_set_contract=nuis-provider-code-asset-identity-set-v1"
        ));
        assert!(resolved.contains("provider_code_asset_identity_set_count=1"));
        assert!(resolved.contains("provider_code_asset_identity_set_root_hash=0x7724b43039b33d4a"));
        assert!(!resolved.contains("provider_kernel_id="));
        assert!(!resolved.contains("provider_code_asset_entry="));
        assert!(!resolved.contains("provider_request_0_kernel_operation=vector-add"));
        assert!(!resolved.contains("provider_request_1_kernel_operation=scale"));
        assert!(
            resolved.contains("provider_request_0_code_asset_entry=nuis_project_main_mapped_i64")
        );
        assert!(resolved.contains("provider_request_0_kernel_operation=add-scalar-i64"));
        assert!(resolved.contains("provider_request_0_buffer_element_type=i64"));
        assert!(resolved.contains(
            "provider_request_0_kernel_scalar_bindings=element_count:u32:4,scalar:i64:10"
        ));
        assert!(
            resolved.contains("provider_request_1_code_asset_entry=nuis_project_main_reduced_i64")
        );
        assert!(resolved.contains("provider_request_1_kernel_operation=reduce-sum-i64"));
        assert!(resolved.contains("provider_request_1_kernel_dispatch=1x1x1"));
        assert!(resolved.contains("provider_request_1_dependency_count=1"));
        assert!(resolved.contains(
            "provider_request_1_dependency_0_producer_request_id=kernel.cuda.source.main.mapped.i64"
        ));
        assert!(resolved.contains("provider_request_1_input_binding_0_source=dependency"));
        assert!(resolved.contains(
            "provider_request_1_dependency_0_transport_producer_clock_evidence=provider-clock:request-0:completed"
        ));
        assert!(resolved.contains(
            "provider_request_1_dependency_0_transport_consumer_clock_evidence=provider-clock:request-1:dispatch-ready"
        ));
        assert!(resolved.contains(
            "provider_result_projection_collection_contract=nuis-provider-result-projection-collection-v1"
        ));
        assert!(resolved.contains("provider_result_projection_count=1"));
        assert!(resolved.contains(
            "provider_result_projection_0_producer_request_id=kernel.cuda.source.main.reduced.i64"
        ));
        assert!(resolved
            .contains("provider_result_projection_0_producer_output_buffer=output.main.reduced"));
        assert!(resolved.contains("provider_result_projection_0_expected_i64=50"));
        assert!(resolved
            .contains("provider_result_projection_0_expected_content_hash=0xf71115b38f042bf7"));
        assert!(resolved.contains(&expected_length));
        assert!(resolved.contains(&expected_hash));
        assert!(resolved.contains(&format!(
            "provider_request_1_code_asset_byte_length={}",
            derived.len()
        )));
        assert!(nsdb::validate_provider_request_evidence(&resolved));
        assert!(!output_dir.join(LEFT_FILE_NAME).exists());
        assert!(!output_dir.join(RIGHT_FILE_NAME).exists());
        assert!(!output_dir.join(EXPECTED_FILE_NAME).exists());
        assert!(!output_dir.join(SCALED_EXPECTED_FILE_NAME).exists());
        assert_eq!(
            fs::read(output_dir.join("nuis.kernel.cuda.source.main.mapped.input.i64.bin")).unwrap(),
            [1i64, 2, 3, 4]
                .into_iter()
                .flat_map(i64::to_le_bytes)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            fs::read(output_dir.join("nuis.kernel.cuda.source.main.mapped.expected.i64.bin"))
                .unwrap(),
            [11i64, 12, 13, 14]
                .into_iter()
                .flat_map(i64::to_le_bytes)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            fs::read(output_dir.join("nuis.kernel.cuda.source.main.reduced.expected.i64.bin"))
                .unwrap(),
            50i64.to_le_bytes()
        );
        assert!(!output_dir
            .join("nuis.kernel.cuda.source.main.reduced.input.i64.bin")
            .exists());

        let codegen_table_path = output_dir.join("nuis.domain.kernel.codegen-table.toml");
        let codegen_table = fs::read_to_string(&codegen_table_path).unwrap();
        fs::write(
            &codegen_table_path,
            codegen_table.replace("0x7519a228f04318e8", "0x7519a228f04318e9"),
        )
        .unwrap();
        assert!(resolve_cuda_code_asset_evidence(&output_dir, &evidence)
            .unwrap_err()
            .contains("code asset identity is inconsistent"));
        fs::write(&codegen_table_path, codegen_table).unwrap();

        let missing_project_entry = String::from_utf8(derived.clone()).unwrap().replace(
            "nuis_project_main_mapped_i64",
            "nuis_project_main_missing_i64",
        );
        fs::write(output_dir.join(asset.file_name), missing_project_entry).unwrap();
        assert!(resolve_cuda_code_asset_evidence(&output_dir, &evidence)
            .unwrap_err()
            .contains("missing requested entry `nuis_project_main_mapped_i64`"));

        fs::remove_dir_all(output_dir).unwrap();
    }
}
