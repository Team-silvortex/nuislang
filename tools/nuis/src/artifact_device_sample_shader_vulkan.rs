use crate::artifact_device_sample_registration::DeviceSampleInputRegistration;
use std::{fs, path::Path};

const PACKAGE_ID: &str = "official.shader";
const ASSET_ID: &str = "shader.vulkan.copy-u32.spirv";
const LOWERING_TARGET: &str = "vulkan.discrete-or-integrated-gpu";
const KERNEL_ID: &str = "shader.vulkan.copy-u32";
const INPUT_FILE_NAME: &str = "nuis.shader.vulkan.copy-u32.input.u32.bin";
const EXPECTED_FILE_NAME: &str = "nuis.shader.vulkan.copy-u32.expected.u32.bin";
const DIGEST_CONTRACT: &str = "nuis-code-asset-digest-fnv1a64-v1";
const SPIRV_MAGIC: u32 = 0x0723_0203;
const SPIRV_VERSION_1_6: u32 = 0x0001_0600;
const INPUT: &[u8] = &[1, 0, 0, 0, 8, 0, 0, 0, 13, 0, 0, 0, 21, 0, 0, 0];
const EXPECTED: &[u8] = INPUT;

pub(crate) fn registration() -> DeviceSampleInputRegistration {
    DeviceSampleInputRegistration {
        package_id: PACKAGE_ID,
        provider_family: "spirv:vulkan-gpu",
        supports: supports_vulkan,
        enrich_evidence: vulkan_copy_u32_evidence,
        resolve_evidence: Some(resolve_vulkan_code_asset_evidence),
        persist_payloads: persist_vulkan_copy_payloads,
    }
}

fn supports_vulkan(backend_family: &str, target_device: &str) -> bool {
    backend_family == "vulkan" && target_device == "discrete-or-integrated-gpu"
}

fn vulkan_copy_u32_evidence(_base: &str) -> String {
    let asset = vulkan_asset().expect("Shader Nustar Vulkan SPIR-V asset must be registered");
    render_request_evidence(&asset, &asset.bytes)
}

fn render_request_evidence(
    asset: &nuisc::registry::NustarCodeAssetRegistration,
    bytes: &[u8],
) -> String {
    format!(
        "provider_buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1;provider_buffer_id=input.values;provider_buffer_element_type=u32;provider_buffer_layout=tensor-contiguous;provider_buffer_shape=4;provider_buffer_row_stride_bytes=16;provider_buffer_byte_length={};provider_buffer_payload_path={INPUT_FILE_NAME};provider_buffer_content_hash={};provider_kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1;provider_kernel_id={KERNEL_ID};provider_kernel_operation=copy-u32;provider_kernel_input_buffer=input.values;provider_kernel_input_buffers=input.values;provider_kernel_output_buffer=output.values;provider_kernel_dispatch=4x1x1;provider_kernel_scalar_bindings=element_count:u32:4;provider_code_asset_descriptor_contract=nuis-provider-code-asset-descriptor-v1;provider_code_asset_id={};provider_code_asset_format={};provider_code_asset_target={};provider_code_asset_entry={};provider_code_asset_path={};provider_code_asset_byte_length={};provider_code_asset_digest_contract={DIGEST_CONTRACT};provider_code_asset_content_hash={};provider_output_binding_contract=nuis-provider-output-binding-v1;provider_output_binding_count=1;provider_output_binding_0_role=output.result;provider_output_binding_0_buffer=output.values;provider_output_binding_0_element_type=u32;provider_output_binding_0_shape=4;provider_output_binding_0_byte_length={};provider_output_binding_0_comparison_id=comparison.output.values;provider_output_comparison_id=comparison.output.values;provider_output_comparison_descriptor_contract=nuis-provider-output-comparison-descriptor-v1;provider_output_comparison_output_buffer=output.values;provider_output_comparison_element_type=u32;provider_output_comparison_shape=4;provider_output_comparison_expected_path={EXPECTED_FILE_NAME};provider_output_comparison_expected_byte_length={};provider_output_comparison_expected_content_hash={};provider_output_comparison_absolute_tolerance=0;provider_output_comparison_relative_tolerance=0;provider_output_comparison_non_finite_policy=reject;provider_dependency_contract=nuis-provider-request-dependency-v1;provider_dependency_count=0;provider_input_binding_contract=nuis-provider-input-binding-v1;provider_input_binding_count=1;provider_input_binding_0_name=input.values;provider_input_binding_0_source=artifact;provider_input_binding_0_element_type=u32;provider_input_binding_0_shape=4;provider_input_binding_0_byte_length={};provider_input_binding_0_content_hash={};provider_input_binding_0_payload_path={INPUT_FILE_NAME};provider_input_binding_0_producer_request_id=none;provider_input_binding_0_producer_output_buffer=none;provider_adapter_binding_contract=nuis-provider-request-adapter-binding-v1;provider_adapter_binding_provider_family=spirv:vulkan-gpu;provider_adapter_binding_execution_requirement=real-device",
        INPUT.len(),
        fnv1a64_hex(INPUT),
        asset.asset_id,
        asset.format,
        asset.target,
        asset.entry,
        asset.file_name,
        bytes.len(),
        fnv1a64_hex(bytes),
        EXPECTED.len(),
        EXPECTED.len(),
        fnv1a64_hex(EXPECTED),
        INPUT.len(),
        fnv1a64_hex(INPUT),
    )
}

fn persist_vulkan_copy_payloads(output_dir: &Path, evidence: &[&str]) -> Result<(), String> {
    if !evidence.iter().any(|item| {
        item.contains(&format!(
            "provider_sample_registration_package={PACKAGE_ID}"
        ))
    }) {
        return Ok(());
    }
    let asset = vulkan_asset()?;
    let actual = fs::read(output_dir.join(&asset.file_name))
        .map_err(|error| format!("failed to read Nuis-emitted Vulkan SPIR-V asset: {error}"))?;
    validate_vulkan_code_asset(&asset, &actual)?;
    for (name, bytes) in [(INPUT_FILE_NAME, INPUT), (EXPECTED_FILE_NAME, EXPECTED)] {
        fs::write(output_dir.join(name), bytes)
            .map_err(|error| format!("failed to persist Vulkan copy-u32 payload: {error}"))?;
    }
    Ok(())
}

fn resolve_vulkan_code_asset_evidence(output_dir: &Path, evidence: &str) -> Result<String, String> {
    let asset = vulkan_asset()?;
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

fn vulkan_asset() -> Result<nuisc::registry::NustarCodeAssetRegistration, String> {
    let root = Path::new("nustar-packages");
    let manifest = nuisc::registry::load_manifest_for_domain(root, "shader")?;
    nuisc::registry::code_asset_registrations(root, &manifest)?
        .into_iter()
        .find(|asset| {
            asset.asset_id == ASSET_ID
                && asset.package_id == PACKAGE_ID
                && asset.lowering_target == LOWERING_TARGET
        })
        .ok_or_else(|| "Shader Nustar Vulkan SPIR-V code asset is not registered".to_owned())
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
    fn registration_owns_vulkan_copy_request_and_payload_contract() {
        let registration = registration();
        let evidence = (registration.enrich_evidence)("ignored");

        assert_eq!(registration.package_id, PACKAGE_ID);
        assert!((registration.supports)(
            "vulkan",
            "discrete-or-integrated-gpu"
        ));
        assert!(!(registration.supports)("metal", "apple-silicon-gpu"));
        assert!(evidence.contains("provider_kernel_operation=copy-u32"));
        assert!(evidence.contains("provider_code_asset_format=spirv-binary"));
        assert!(evidence.contains("provider_code_asset_id=shader.vulkan.copy-u32.spirv"));
        assert!(evidence.contains("provider_code_asset_target=vulkan1.3-spirv1.6"));
        assert!(evidence.contains("provider_code_asset_entry=nuis_vulkan_copy_u32"));
        assert!(evidence.contains("provider_adapter_binding_provider_family=spirv:vulkan-gpu"));
        assert!(evidence.contains("provider_output_binding_0_element_type=u32"));
        assert!(nsdb::validate_provider_request_evidence(&evidence));
    }

    #[test]
    fn registration_verifies_spirv_before_persisting_inputs() {
        let output_dir = env::temp_dir().join(format!(
            "nuis-shader-vulkan-provider-registration-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&output_dir);
        fs::create_dir_all(&output_dir).unwrap();
        let asset = vulkan_asset().unwrap();
        fs::write(output_dir.join(&asset.file_name), &asset.bytes).unwrap();
        persist_vulkan_copy_payloads(
            &output_dir,
            &["provider_sample_registration_package=official.shader"],
        )
        .unwrap();
        assert_eq!(fs::read(output_dir.join(INPUT_FILE_NAME)).unwrap(), INPUT);
        assert_eq!(
            fs::read(output_dir.join(EXPECTED_FILE_NAME)).unwrap(),
            EXPECTED
        );
        let mut tampered = asset.bytes.clone();
        tampered[0] ^= 1;
        fs::write(output_dir.join(&asset.file_name), tampered).unwrap();
        assert!(persist_vulkan_copy_payloads(
            &output_dir,
            &["provider_sample_registration_package=official.shader"]
        )
        .unwrap_err()
        .contains("registry ownership"));
        fs::remove_dir_all(output_dir).unwrap();
    }
}
