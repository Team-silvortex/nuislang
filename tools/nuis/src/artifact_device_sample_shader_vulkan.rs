use crate::artifact_device_sample_registration::DeviceSampleInputRegistration;
use std::{fs, path::Path};

const PACKAGE_ID: &str = "official.shader";
const LOWERING_TARGET: &str = "vulkan.discrete-or-integrated-gpu";
const DIGEST_CONTRACT: &str = "nuis-code-asset-digest-fnv1a64-v1";
const SPIRV_MAGIC: u32 = 0x0723_0203;
const SPIRV_VERSION_1_6: u32 = 0x0001_0600;
const INPUT: &[u8] = &[1, 0, 0, 0, 8, 0, 0, 0, 13, 0, 0, 0, 21, 0, 0, 0];
const ADD_EXPECTED: &[u8] = &[2, 0, 0, 0, 16, 0, 0, 0, 26, 0, 0, 0, 42, 0, 0, 0];
const SUB_EXPECTED: &[u8] = &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
const MUL_EXPECTED: &[u8] = &[1, 0, 0, 0, 64, 0, 0, 0, 169, 0, 0, 0, 185, 1, 0, 0];

#[derive(Clone, Copy)]
struct VulkanSampleSpec {
    registration_id: &'static str,
    metadata_selector: Option<&'static str>,
    asset_id: &'static str,
    kernel_id: &'static str,
    operation: &'static str,
    input_file_name: &'static str,
    expected_file_name: &'static str,
    expected: &'static [u8],
}

const COPY_SAMPLE: VulkanSampleSpec = VulkanSampleSpec {
    registration_id: "official.shader.vulkan-copy-u32",
    metadata_selector: None,
    asset_id: "shader.vulkan.copy-u32.spirv",
    kernel_id: "shader.vulkan.copy-u32",
    operation: "copy-u32",
    input_file_name: "nuis.shader.vulkan.copy-u32.input.u32.bin",
    expected_file_name: "nuis.shader.vulkan.copy-u32.expected.u32.bin",
    expected: INPUT,
};

const ADD_SAMPLE: VulkanSampleSpec = VulkanSampleSpec {
    registration_id: "official.shader.vulkan-add-u32",
    metadata_selector: Some("official.shader:provider-sample=vulkan-add-u32"),
    asset_id: "shader.vulkan.add-u32.spirv",
    kernel_id: "shader.vulkan.add-u32",
    operation: "add-u32",
    input_file_name: "nuis.shader.vulkan.add-u32.input.u32.bin",
    expected_file_name: "nuis.shader.vulkan.add-u32.expected.u32.bin",
    expected: ADD_EXPECTED,
};

const SUB_SAMPLE: VulkanSampleSpec = VulkanSampleSpec {
    registration_id: "official.shader.vulkan-sub-u32",
    metadata_selector: Some("official.shader:provider-sample=vulkan-sub-u32"),
    asset_id: "shader.vulkan.sub-u32.spirv",
    kernel_id: "shader.vulkan.sub-u32",
    operation: "sub-u32",
    input_file_name: "nuis.shader.vulkan.sub-u32.input.u32.bin",
    expected_file_name: "nuis.shader.vulkan.sub-u32.expected.u32.bin",
    expected: SUB_EXPECTED,
};

const MUL_SAMPLE: VulkanSampleSpec = VulkanSampleSpec {
    registration_id: "official.shader.vulkan-mul-u32",
    metadata_selector: Some("official.shader:provider-sample=vulkan-mul-u32"),
    asset_id: "shader.vulkan.mul-u32.spirv",
    kernel_id: "shader.vulkan.mul-u32",
    operation: "mul-u32",
    input_file_name: "nuis.shader.vulkan.mul-u32.input.u32.bin",
    expected_file_name: "nuis.shader.vulkan.mul-u32.expected.u32.bin",
    expected: MUL_EXPECTED,
};

const SAMPLES: &[VulkanSampleSpec] = &[COPY_SAMPLE, ADD_SAMPLE, SUB_SAMPLE, MUL_SAMPLE];

pub(crate) fn registration() -> DeviceSampleInputRegistration {
    registration_for(COPY_SAMPLE, None, vulkan_copy_u32_evidence)
}

pub(crate) fn add_registration() -> DeviceSampleInputRegistration {
    registration_for(
        ADD_SAMPLE,
        Some(selects_shader_vulkan_add),
        vulkan_add_u32_evidence,
    )
}

pub(crate) fn sub_registration() -> DeviceSampleInputRegistration {
    registration_for(
        SUB_SAMPLE,
        Some(selects_shader_vulkan_sub),
        vulkan_sub_u32_evidence,
    )
}

pub(crate) fn mul_registration() -> DeviceSampleInputRegistration {
    registration_for(
        MUL_SAMPLE,
        Some(selects_shader_vulkan_mul),
        vulkan_mul_u32_evidence,
    )
}

fn registration_for(
    sample: VulkanSampleSpec,
    metadata_selector: Option<fn(&str) -> bool>,
    enrich_evidence: fn(&str) -> String,
) -> DeviceSampleInputRegistration {
    DeviceSampleInputRegistration {
        package_id: PACKAGE_ID,
        registration_id: sample.registration_id,
        provider_family: "spirv:vulkan-gpu",
        supports: supports_vulkan,
        metadata_selector,
        enrich_evidence,
        resolve_evidence: Some(resolve_vulkan_code_asset_evidence),
        persist_payloads: persist_vulkan_payloads,
    }
}

fn supports_vulkan(backend_family: &str, target_device: &str) -> bool {
    backend_family == "vulkan" && target_device == "discrete-or-integrated-gpu"
}

fn selects_shader_vulkan_add(base: &str) -> bool {
    selects_metadata(base, ADD_SAMPLE.metadata_selector.expect("selector"))
}

fn selects_shader_vulkan_sub(base: &str) -> bool {
    selects_metadata(base, SUB_SAMPLE.metadata_selector.expect("selector"))
}

fn selects_shader_vulkan_mul(base: &str) -> bool {
    selects_metadata(base, MUL_SAMPLE.metadata_selector.expect("selector"))
}

fn selects_metadata(base: &str, selector: &str) -> bool {
    base.split(';')
        .filter_map(|field| field.split_once('='))
        .any(|(key, value)| key.starts_with("artifact_provider_metadata_") && value == selector)
}

fn vulkan_copy_u32_evidence(_base: &str) -> String {
    vulkan_sample_evidence(COPY_SAMPLE)
}

fn vulkan_add_u32_evidence(_base: &str) -> String {
    vulkan_sample_evidence(ADD_SAMPLE)
}

fn vulkan_sub_u32_evidence(_base: &str) -> String {
    vulkan_sample_evidence(SUB_SAMPLE)
}

fn vulkan_mul_u32_evidence(_base: &str) -> String {
    vulkan_sample_evidence(MUL_SAMPLE)
}

fn vulkan_sample_evidence(sample: VulkanSampleSpec) -> String {
    let asset = vulkan_asset(sample).expect("Shader Nustar Vulkan SPIR-V asset must be registered");
    render_request_evidence(sample, &asset, &asset.bytes)
}

fn render_request_evidence(
    sample: VulkanSampleSpec,
    asset: &nuisc::registry::NustarCodeAssetRegistration,
    bytes: &[u8],
) -> String {
    format!(
        "provider_buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1;provider_buffer_id=input.values;provider_buffer_element_type=u32;provider_buffer_layout=tensor-contiguous;provider_buffer_shape=4;provider_buffer_row_stride_bytes=16;provider_buffer_byte_length={};provider_buffer_payload_path={};provider_buffer_content_hash={};provider_kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1;provider_kernel_id={};provider_kernel_operation={};provider_kernel_input_buffer=input.values;provider_kernel_input_buffers=input.values;provider_kernel_output_buffer=output.values;provider_kernel_dispatch=4x1x1;provider_kernel_scalar_bindings=element_count:u32:4;provider_code_asset_descriptor_contract=nuis-provider-code-asset-descriptor-v1;provider_code_asset_id={};provider_code_asset_format={};provider_code_asset_target={};provider_code_asset_entry={};provider_code_asset_path={};provider_code_asset_byte_length={};provider_code_asset_digest_contract={DIGEST_CONTRACT};provider_code_asset_content_hash={};provider_output_binding_contract=nuis-provider-output-binding-v1;provider_output_binding_count=1;provider_output_binding_0_role=output.result;provider_output_binding_0_buffer=output.values;provider_output_binding_0_element_type=u32;provider_output_binding_0_shape=4;provider_output_binding_0_byte_length={};provider_output_binding_0_comparison_id=comparison.output.values;provider_output_comparison_id=comparison.output.values;provider_output_comparison_descriptor_contract=nuis-provider-output-comparison-descriptor-v1;provider_output_comparison_output_buffer=output.values;provider_output_comparison_element_type=u32;provider_output_comparison_shape=4;provider_output_comparison_expected_path={};provider_output_comparison_expected_byte_length={};provider_output_comparison_expected_content_hash={};provider_output_comparison_absolute_tolerance=0;provider_output_comparison_relative_tolerance=0;provider_output_comparison_non_finite_policy=reject;provider_dependency_contract=nuis-provider-request-dependency-v1;provider_dependency_count=0;provider_input_binding_contract=nuis-provider-input-binding-v1;provider_input_binding_count=1;provider_input_binding_0_name=input.values;provider_input_binding_0_source=artifact;provider_input_binding_0_element_type=u32;provider_input_binding_0_shape=4;provider_input_binding_0_byte_length={};provider_input_binding_0_content_hash={};provider_input_binding_0_payload_path={};provider_input_binding_0_producer_request_id=none;provider_input_binding_0_producer_output_buffer=none;provider_adapter_binding_contract=nuis-provider-request-adapter-binding-v1;provider_adapter_binding_provider_family=spirv:vulkan-gpu;provider_adapter_binding_execution_requirement=real-device",
        INPUT.len(),
        sample.input_file_name,
        fnv1a64_hex(INPUT),
        sample.kernel_id,
        sample.operation,
        asset.asset_id,
        asset.format,
        asset.target,
        asset.entry,
        asset.file_name,
        bytes.len(),
        fnv1a64_hex(bytes),
        sample.expected.len(),
        sample.expected_file_name,
        sample.expected.len(),
        fnv1a64_hex(sample.expected),
        INPUT.len(),
        fnv1a64_hex(INPUT),
        sample.input_file_name,
    )
}

fn persist_vulkan_payloads(output_dir: &Path, evidence: &[&str]) -> Result<(), String> {
    let Some(sample) = sample_from_evidence_items(evidence) else {
        return Ok(());
    };
    let asset = vulkan_asset(sample)?;
    let actual = fs::read(output_dir.join(&asset.file_name))
        .map_err(|error| format!("failed to read Nuis-emitted Vulkan SPIR-V asset: {error}"))?;
    validate_vulkan_code_asset(&asset, &actual)?;
    for (name, bytes) in [
        (sample.input_file_name, INPUT),
        (sample.expected_file_name, sample.expected),
    ] {
        fs::write(output_dir.join(name), bytes)
            .map_err(|error| format!("failed to persist Vulkan u32 payload: {error}"))?;
    }
    Ok(())
}

fn resolve_vulkan_code_asset_evidence(output_dir: &Path, evidence: &str) -> Result<String, String> {
    let sample = sample_from_evidence(evidence)
        .ok_or_else(|| "Vulkan provider sample evidence is missing registration ID".to_owned())?;
    let asset = vulkan_asset(sample)?;
    let actual = fs::read(output_dir.join(&asset.file_name))
        .map_err(|error| format!("failed to read Nuis-emitted Vulkan SPIR-V asset: {error}"))?;
    validate_vulkan_code_asset(&asset, &actual)?;
    validate_vulkan_request_asset_evidence(&asset, &actual, evidence)?;
    let selection =
        crate::artifact_code_asset_contribution_table::select_compiled_code_asset_contribution(
            output_dir,
            PACKAGE_ID,
            "shader",
            LOWERING_TARGET,
            &asset.format,
            &asset.target,
            std::slice::from_ref(&asset.entry),
        )?;
    let selection_evidence = selection
        .as_ref()
        .map(|selection| -> Result<String, String> {
            validate_vulkan_contribution_selection(selection, evidence)?;
            crate::artifact_code_asset_contribution_table::render_selected_contribution_set_evidence(
                std::slice::from_ref(selection),
            )
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

fn validate_vulkan_request_asset_evidence(
    asset: &nuisc::registry::NustarCodeAssetRegistration,
    bytes: &[u8],
    evidence: &str,
) -> Result<(), String> {
    let expected = [
        ("provider_code_asset_id", asset.asset_id.as_str()),
        ("provider_code_asset_format", asset.format.as_str()),
        ("provider_code_asset_target", asset.target.as_str()),
        ("provider_code_asset_entry", asset.entry.as_str()),
        ("provider_code_asset_path", asset.file_name.as_str()),
        ("provider_code_asset_digest_contract", DIGEST_CONTRACT),
    ];
    for (field, expected) in expected {
        let needle = format!("{field}={expected}");
        if !evidence.split(';').any(|item| item == needle) {
            return Err(format!(
                "Vulkan provider request code asset field `{field}` does not match `{expected}`"
            ));
        }
    }
    let expected_length = format!("provider_code_asset_byte_length={}", bytes.len());
    let expected_hash = format!("provider_code_asset_content_hash={}", fnv1a64_hex(bytes));
    if !evidence.split(';').any(|item| item == expected_length)
        || !evidence.split(';').any(|item| item == expected_hash)
    {
        return Err("Vulkan provider request code asset byte identity does not match".to_owned());
    }
    Ok(())
}

fn validate_vulkan_contribution_selection(
    selection: &crate::artifact_code_asset_contribution_table::SelectedCodeAssetContribution,
    evidence: &str,
) -> Result<(), String> {
    for (field, expected) in [
        ("provider_code_asset_id", selection.asset_id.as_str()),
        ("provider_code_asset_format", selection.format.as_str()),
        ("provider_code_asset_target", selection.target.as_str()),
        ("provider_code_asset_path", selection.path.as_str()),
        (
            "provider_code_asset_content_hash",
            selection.content_hash.as_str(),
        ),
    ] {
        let needle = format!("{field}={expected}");
        if !evidence.split(';').any(|item| item == needle) {
            return Err(format!(
                "Vulkan provider request does not match compiled contribution field `{field}`"
            ));
        }
    }
    Ok(())
}

fn validate_vulkan_code_asset(
    asset: &nuisc::registry::NustarCodeAssetRegistration,
    bytes: &[u8],
) -> Result<(), String> {
    if bytes != asset.bytes {
        return Err(
            "Nuis-emitted Vulkan SPIR-V asset does not match registry ownership".to_owned(),
        );
    }
    if bytes.len() < 20 || bytes.len() % 4 != 0 {
        return Err("Nuis-emitted Vulkan SPIR-V asset has invalid word alignment".to_owned());
    }
    let magic = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
    let version = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
    if magic != SPIRV_MAGIC || version != SPIRV_VERSION_1_6 {
        return Err("Nuis-emitted Vulkan SPIR-V asset has invalid header".to_owned());
    }
    if !bytes
        .windows(asset.entry.len())
        .any(|window| window == asset.entry.as_bytes())
    {
        return Err(format!(
            "Nuis-emitted Vulkan SPIR-V asset is missing entry `{}`",
            asset.entry
        ));
    }
    Ok(())
}

fn vulkan_asset(
    sample: VulkanSampleSpec,
) -> Result<nuisc::registry::NustarCodeAssetRegistration, String> {
    let root = Path::new("nustar-packages");
    let manifest = nuisc::registry::load_manifest_for_domain(root, "shader")?;
    nuisc::registry::code_asset_registration_by_id(root, &manifest, sample.asset_id)?
        .filter(|asset| {
            asset.asset_id == sample.asset_id
                && asset.package_id == PACKAGE_ID
                && asset.lowering_target == LOWERING_TARGET
        })
        .ok_or_else(|| "Shader Nustar Vulkan SPIR-V code asset is not registered".to_owned())
}

fn sample_from_evidence_items(evidence: &[&str]) -> Option<VulkanSampleSpec> {
    evidence.iter().find_map(|item| sample_from_evidence(item))
}

fn sample_from_evidence(evidence: &str) -> Option<VulkanSampleSpec> {
    evidence.split(';').find_map(|field| {
        field
            .strip_prefix("provider_sample_registration_id=")
            .and_then(sample_by_registration_id)
    })
}

fn sample_by_registration_id(registration_id: &str) -> Option<VulkanSampleSpec> {
    SAMPLES
        .iter()
        .copied()
        .find(|sample| sample.registration_id == registration_id)
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
    fn registrations_own_vulkan_u32_request_and_payload_contracts() {
        for (registration, sample) in [
            (registration(), COPY_SAMPLE),
            (add_registration(), ADD_SAMPLE),
            (sub_registration(), SUB_SAMPLE),
            (mul_registration(), MUL_SAMPLE),
        ] {
            let evidence = (registration.enrich_evidence)("ignored");

            assert_eq!(registration.package_id, PACKAGE_ID);
            assert_eq!(registration.registration_id, sample.registration_id);
            assert!((registration.supports)(
                "vulkan",
                "discrete-or-integrated-gpu"
            ));
            assert!(!(registration.supports)("metal", "apple-silicon-gpu"));
            if let Some(selector) = sample.metadata_selector {
                assert!((registration.metadata_selector.unwrap())(&format!(
                    "artifact_provider_metadata_0={selector}"
                )));
            } else {
                assert!(registration.metadata_selector.is_none());
            }
            assert!(evidence.contains(&format!("provider_kernel_operation={}", sample.operation)));
            assert!(evidence.contains("provider_code_asset_format=spirv-binary"));
            assert!(evidence.contains(&format!("provider_code_asset_id={}", sample.asset_id)));
            assert!(evidence.contains("provider_code_asset_target=vulkan1.3-spirv1.6"));
            assert!(evidence.contains("provider_code_asset_entry=nuis_vulkan_"));
            assert!(evidence.contains("provider_adapter_binding_provider_family=spirv:vulkan-gpu"));
            assert!(evidence.contains("provider_output_binding_0_element_type=u32"));
            assert!(nsdb::validate_provider_request_evidence(&evidence));
        }
    }

    #[test]
    fn registration_verifies_spirv_before_persisting_inputs() {
        let output_dir = env::temp_dir().join(format!(
            "nuis-shader-vulkan-provider-registration-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&output_dir);
        fs::create_dir_all(&output_dir).unwrap();
        let asset = vulkan_asset(MUL_SAMPLE).unwrap();
        fs::write(output_dir.join(&asset.file_name), &asset.bytes).unwrap();
        persist_vulkan_payloads(
            &output_dir,
            &["provider_sample_registration_id=official.shader.vulkan-mul-u32"],
        )
        .unwrap();
        assert_eq!(
            fs::read(output_dir.join(MUL_SAMPLE.input_file_name)).unwrap(),
            INPUT
        );
        assert_eq!(
            fs::read(output_dir.join(MUL_SAMPLE.expected_file_name)).unwrap(),
            MUL_EXPECTED
        );
        let mut tampered = asset.bytes.clone();
        tampered[0] ^= 1;
        fs::write(output_dir.join(&asset.file_name), tampered).unwrap();
        assert!(persist_vulkan_payloads(
            &output_dir,
            &["provider_sample_registration_id=official.shader.vulkan-mul-u32"]
        )
        .unwrap_err()
        .contains("registry ownership"));
        fs::remove_dir_all(output_dir).unwrap();
    }
}
