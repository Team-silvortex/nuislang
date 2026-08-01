use crate::{
    artifact_device_sample_registration::DeviceSampleInputRegistration,
    artifact_device_sample_shader_common::{
        fnv1a64_hex, render_dependency_count_zero, render_u32_output_evidence,
        render_u32_pair_artifact_binding, render_u32_prefixed_request_evidence,
        replace_code_asset_identity_fields, validate_code_asset_contribution_selection,
        validate_code_asset_request_evidence, U32OutputEvidence, U32RequestEvidence,
        U32_INPUT as INPUT, U32_PAIR_ADD_EXPECTED as SUM_EXPECTED,
        U32_PAIR_RIGHT_INPUT as RIGHT_INPUT,
    },
};
use std::{fs, path::Path};

const PACKAGE_ID: &str = "official.shader";
const REGISTRATION_ID: &str = "official.shader.vulkan-add-xor-pair-u32";
const METADATA_SELECTOR: &str = "official.shader:provider-sample=vulkan-add-xor-pair-u32";
const ASSET_ID: &str = "shader.vulkan.add-xor-pair-u32.spirv";
const ENTRY: &str = "nuis_vulkan_add_xor_pair_u32";
const LOWERING_TARGET: &str = "vulkan.discrete-or-integrated-gpu";
const INPUT_FILE: &str = "nuis.shader.vulkan.add-xor-pair-u32.left.u32.bin";
const RIGHT_FILE: &str = "nuis.shader.vulkan.add-xor-pair-u32.right.u32.bin";
const SUM_FILE: &str = "nuis.shader.vulkan.add-xor-pair-u32.sum.expected.u32.bin";
const XOR_FILE: &str = "nuis.shader.vulkan.add-xor-pair-u32.xor.expected.u32.bin";
const XOR_EXPECTED: &[u8] = &[3, 0, 0, 0, 11, 0, 0, 0, 8, 0, 0, 0, 29, 0, 0, 0];

pub(crate) fn registration() -> DeviceSampleInputRegistration {
    DeviceSampleInputRegistration {
        package_id: PACKAGE_ID,
        registration_id: REGISTRATION_ID,
        provider_family: "spirv:vulkan-gpu",
        supports: |backend, device| backend == "vulkan" && device == "discrete-or-integrated-gpu",
        metadata_selector: Some(selects_sample),
        enrich_evidence: sample_evidence,
        resolve_evidence: Some(resolve_code_asset_evidence),
        persist_payloads,
    }
}

fn selects_sample(base: &str) -> bool {
    base.split(';')
        .filter_map(|field| field.split_once('='))
        .any(|(key, value)| {
            key.starts_with("artifact_provider_metadata_") && value == METADATA_SELECTOR
        })
}

fn sample_evidence(_base: &str) -> String {
    let asset = asset().expect("Shader Nustar Vulkan fan-out asset must be registered");
    let output_evidence = render_u32_output_evidence(
        "provider_",
        &[
            U32OutputEvidence {
                role: "output.sum",
                buffer: "output.values",
                layout: "tensor-row-major",
                shape: "2x2",
                row_stride_bytes: 8,
                comparison_id: "comparison.output.sum",
                expected_file_name: SUM_FILE,
                expected: SUM_EXPECTED,
            },
            U32OutputEvidence {
                role: "output.xor",
                buffer: "output.xor",
                layout: "tensor-row-major",
                shape: "2x2",
                row_stride_bytes: 8,
                comparison_id: "comparison.output.xor",
                expected_file_name: XOR_FILE,
                expected: XOR_EXPECTED,
            },
        ],
    );
    render_u32_prefixed_request_evidence(U32RequestEvidence {
        prefix: "provider_",
        provider_family: "spirv:vulkan-gpu",
        kernel_id: "shader.vulkan.add-xor-pair-u32",
        operation: "add-xor-pair-u32",
        kernel_input_buffers: "input.values,input.right",
        buffer_layout: "tensor-row-major",
        buffer_shape: "2x2",
        row_stride_bytes: 8,
        dispatch: "4x1x1",
        input_file_name: INPUT_FILE,
        input_hash: fnv1a64_hex(INPUT),
        input_byte_length: INPUT.len(),
        expected_file_name: SUM_FILE,
        expected: SUM_EXPECTED,
        asset: &asset,
        bytes: &asset.bytes,
        input_binding: render_u32_pair_artifact_binding(
            "provider_",
            "tensor-row-major",
            "2x2",
            8,
            INPUT_FILE,
            INPUT,
            RIGHT_FILE,
            RIGHT_INPUT,
        ),
        dependency: render_dependency_count_zero("provider_"),
        output_evidence: Some(output_evidence),
    })
}

fn persist_payloads(output_dir: &Path, evidence: &[&str]) -> Result<(), String> {
    if !evidence.iter().any(|item| {
        item.split(';')
            .any(|field| field == format!("provider_sample_registration_id={REGISTRATION_ID}"))
    }) {
        return Ok(());
    }
    let asset = asset()?;
    let actual = fs::read(output_dir.join(&asset.file_name))
        .map_err(|error| format!("failed to read Nuis-emitted Vulkan fan-out asset: {error}"))?;
    validate_asset(&asset, &actual)?;
    for (name, bytes) in [
        (INPUT_FILE, INPUT),
        (RIGHT_FILE, RIGHT_INPUT),
        (SUM_FILE, SUM_EXPECTED),
        (XOR_FILE, XOR_EXPECTED),
    ] {
        fs::write(output_dir.join(name), bytes)
            .map_err(|error| format!("failed to persist Vulkan fan-out payload: {error}"))?;
    }
    Ok(())
}

fn resolve_code_asset_evidence(output_dir: &Path, evidence: &str) -> Result<String, String> {
    let asset = asset()?;
    let actual = fs::read(output_dir.join(&asset.file_name))
        .map_err(|error| format!("failed to read Nuis-emitted Vulkan fan-out asset: {error}"))?;
    validate_asset(&asset, &actual)?;
    validate_code_asset_request_evidence("Vulkan fan-out", &asset, &actual, evidence, "provider_")?;
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
    let mut resolved = replace_code_asset_identity_fields(
        evidence,
        &[("provider_".to_owned(), actual.len(), fnv1a64_hex(&actual))],
    );
    if let Some(selection) = selection {
        validate_code_asset_contribution_selection(
            "Vulkan fan-out",
            &selection,
            &resolved,
            "provider_",
        )?;
        resolved.push(';');
        resolved.push_str(
            &crate::artifact_code_asset_contribution_table::render_selected_contribution_set_evidence(
                std::slice::from_ref(&selection),
            )?,
        );
    }
    Ok(resolved)
}

fn validate_asset(
    asset: &nuisc::registry::NustarCodeAssetRegistration,
    bytes: &[u8],
) -> Result<(), String> {
    if bytes != asset.bytes {
        return Err(
            "Nuis-emitted Vulkan fan-out asset does not match registry ownership".to_owned(),
        );
    }
    if bytes.len() < 20
        || u32::from_le_bytes(bytes[0..4].try_into().expect("SPIR-V magic")) != 0x0723_0203
        || !bytes
            .windows(ENTRY.len())
            .any(|window| window == ENTRY.as_bytes())
    {
        return Err("Nuis-emitted Vulkan fan-out asset is not registered SPIR-V".to_owned());
    }
    Ok(())
}

fn asset() -> Result<nuisc::registry::NustarCodeAssetRegistration, String> {
    let root = Path::new("nustar-packages");
    let manifest = nuisc::registry::load_manifest_for_domain(root, "shader")?;
    nuisc::registry::code_asset_registration_by_id(root, &manifest, ASSET_ID)?
        .filter(|asset| {
            asset.package_id == PACKAGE_ID
                && asset.lowering_target == LOWERING_TARGET
                && asset.entry == ENTRY
        })
        .ok_or_else(|| "Shader Nustar Vulkan fan-out code asset is not registered".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fan_out_registration_owns_two_output_request() {
        let registration = registration();
        let evidence = (registration.enrich_evidence)("ignored");

        assert!(evidence.contains("provider_output_binding_count=2"));
        assert!(evidence.contains("provider_output_binding_1_buffer=output.xor"));
        assert!(evidence.contains("provider_output_comparison_collection_count=2"));
        assert!(evidence.contains("provider_kernel_operation=add-xor-pair-u32"));
        assert_eq!(fnv1a64_hex(XOR_EXPECTED), "0x73bb5b39fe3ab738");
        assert!(nsdb::validate_provider_request_evidence(&evidence));
    }
}
