use crate::{
    artifact_device_sample_registration::DeviceSampleInputRegistration,
    artifact_device_sample_shader_common::{
        fnv1a64_hex, render_dependency_count_zero, render_u32_artifact_binding,
        render_u32_dependency_binding, render_u32_dependency_edge,
        render_u32_pair_artifact_binding, render_u32_prefixed_request_evidence,
        render_u32_sample_request_evidence, replace_code_asset_identity_fields,
        validate_code_asset_contribution_selection, validate_code_asset_request_evidence,
        U32RequestEvidence, U32_ADD_EXPECTED as ADD_EXPECTED, U32_INPUT as INPUT,
        U32_MUL_EXPECTED as MUL_EXPECTED, U32_PAIR_ADD_EXPECTED as PAIR_ADD_EXPECTED,
        U32_PAIR_RIGHT_INPUT as PAIR_RIGHT_INPUT, U32_ZERO_EXPECTED as SUB_EXPECTED,
    },
};
use std::{fs, path::Path};

const PACKAGE_ID: &str = "official.shader";
const LOWERING_TARGET: &str = "vulkan.discrete-or-integrated-gpu";
const SPIRV_MAGIC: u32 = 0x0723_0203;
const SPIRV_VERSION_1_6: u32 = 0x0001_0600;
const CHAIN_EXPECTED: &[u8] = SUB_EXPECTED;
const CHAIN_REGISTRATION_ID: &str = "official.shader.vulkan-u32-chain";
const CHAIN_METADATA_SELECTOR: &str = "official.shader:provider-sample=vulkan-u32-chain";
const CHAIN_INPUT_FILE_NAME: &str = "nuis.shader.vulkan.chain.input.u32.bin";
const CHAIN_ADD_EXPECTED_FILE_NAME: &str = "nuis.shader.vulkan.chain.add.expected.u32.bin";
const CHAIN_EXPECTED_FILE_NAME: &str = "nuis.shader.vulkan.chain.expected.u32.bin";

#[derive(Clone, Copy)]
struct VulkanSampleSpec {
    registration_id: &'static str,
    metadata_selector: Option<&'static str>,
    asset_id: &'static str,
    kernel_id: &'static str,
    operation: &'static str,
    input_file_name: &'static str,
    aux_input: Option<(&'static str, &'static [u8])>,
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
    aux_input: None,
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
    aux_input: None,
    expected_file_name: "nuis.shader.vulkan.add-u32.expected.u32.bin",
    expected: ADD_EXPECTED,
};

const ADD_PAIR_SAMPLE: VulkanSampleSpec = VulkanSampleSpec {
    registration_id: "official.shader.vulkan-add-pair-u32",
    metadata_selector: Some("official.shader:provider-sample=vulkan-add-pair-u32"),
    asset_id: "shader.vulkan.add-pair-u32.spirv",
    kernel_id: "shader.vulkan.add-pair-u32",
    operation: "add-pair-u32",
    input_file_name: "nuis.shader.vulkan.add-pair-u32.left.u32.bin",
    aux_input: Some((
        "nuis.shader.vulkan.add-pair-u32.right.u32.bin",
        PAIR_RIGHT_INPUT,
    )),
    expected_file_name: "nuis.shader.vulkan.add-pair-u32.expected.u32.bin",
    expected: PAIR_ADD_EXPECTED,
};

const SUB_SAMPLE: VulkanSampleSpec = VulkanSampleSpec {
    registration_id: "official.shader.vulkan-sub-u32",
    metadata_selector: Some("official.shader:provider-sample=vulkan-sub-u32"),
    asset_id: "shader.vulkan.sub-u32.spirv",
    kernel_id: "shader.vulkan.sub-u32",
    operation: "sub-u32",
    input_file_name: "nuis.shader.vulkan.sub-u32.input.u32.bin",
    aux_input: None,
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
    aux_input: None,
    expected_file_name: "nuis.shader.vulkan.mul-u32.expected.u32.bin",
    expected: MUL_EXPECTED,
};

const XOR_SAMPLE: VulkanSampleSpec = VulkanSampleSpec {
    registration_id: "official.shader.vulkan-xor-u32",
    metadata_selector: Some("official.shader:provider-sample=vulkan-xor-u32"),
    asset_id: "shader.vulkan.xor-u32.spirv",
    kernel_id: "shader.vulkan.xor-u32",
    operation: "xor-u32",
    input_file_name: "nuis.shader.vulkan.xor-u32.input.u32.bin",
    aux_input: None,
    expected_file_name: "nuis.shader.vulkan.xor-u32.expected.u32.bin",
    expected: SUB_EXPECTED,
};

const SAMPLES: &[VulkanSampleSpec] = &[
    COPY_SAMPLE,
    ADD_SAMPLE,
    ADD_PAIR_SAMPLE,
    SUB_SAMPLE,
    MUL_SAMPLE,
    XOR_SAMPLE,
];

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

pub(crate) fn add_pair_registration() -> DeviceSampleInputRegistration {
    registration_for(
        ADD_PAIR_SAMPLE,
        Some(selects_shader_vulkan_add_pair),
        vulkan_add_pair_u32_evidence,
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

pub(crate) fn xor_registration() -> DeviceSampleInputRegistration {
    registration_for(
        XOR_SAMPLE,
        Some(selects_shader_vulkan_xor),
        vulkan_xor_u32_evidence,
    )
}

pub(crate) fn chain_registration() -> DeviceSampleInputRegistration {
    DeviceSampleInputRegistration {
        package_id: PACKAGE_ID,
        registration_id: CHAIN_REGISTRATION_ID,
        provider_family: "spirv:vulkan-gpu",
        supports: supports_vulkan,
        metadata_selector: Some(selects_shader_vulkan_chain),
        enrich_evidence: vulkan_chain_u32_evidence,
        resolve_evidence: Some(resolve_vulkan_code_asset_evidence),
        persist_payloads: persist_vulkan_payloads,
    }
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

fn selects_shader_vulkan_add_pair(base: &str) -> bool {
    selects_metadata(
        base,
        ADD_PAIR_SAMPLE.metadata_selector.expect("pair selector"),
    )
}

fn selects_shader_vulkan_sub(base: &str) -> bool {
    selects_metadata(base, SUB_SAMPLE.metadata_selector.expect("selector"))
}

fn selects_shader_vulkan_mul(base: &str) -> bool {
    selects_metadata(base, MUL_SAMPLE.metadata_selector.expect("selector"))
}

fn selects_shader_vulkan_xor(base: &str) -> bool {
    selects_metadata(base, XOR_SAMPLE.metadata_selector.expect("selector"))
}

fn selects_shader_vulkan_chain(base: &str) -> bool {
    selects_metadata(base, CHAIN_METADATA_SELECTOR)
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

fn vulkan_add_pair_u32_evidence(_base: &str) -> String {
    vulkan_sample_evidence(ADD_PAIR_SAMPLE)
}

fn vulkan_sub_u32_evidence(_base: &str) -> String {
    vulkan_sample_evidence(SUB_SAMPLE)
}

fn vulkan_mul_u32_evidence(_base: &str) -> String {
    vulkan_sample_evidence(MUL_SAMPLE)
}

fn vulkan_xor_u32_evidence(_base: &str) -> String {
    vulkan_sample_evidence(XOR_SAMPLE)
}

fn vulkan_chain_u32_evidence(_base: &str) -> String {
    let add = vulkan_asset(ADD_SAMPLE).expect("Shader Nustar Vulkan add asset must be registered");
    let xor = vulkan_asset(XOR_SAMPLE).expect("Shader Nustar Vulkan xor asset must be registered");
    let identity = vulkan_chain_code_asset_identity(&add, &xor)
        .expect("Shader Nustar Vulkan chain identity must assemble");
    format!(
        "provider_request_collection_contract=nuis-provider-request-collection-v1;provider_request_count=2;{};{};{}",
        render_chain_artifact_request(0, ADD_SAMPLE, &add, &add.bytes),
        render_chain_dependency_request(1, XOR_SAMPLE, &xor, &xor.bytes),
        identity.identity_evidence,
    )
}

fn vulkan_sample_evidence(sample: VulkanSampleSpec) -> String {
    let asset = vulkan_asset(sample).expect("Shader Nustar Vulkan SPIR-V asset must be registered");
    if let Some((right_file_name, right)) = sample.aux_input {
        return render_u32_prefixed_request_evidence(U32RequestEvidence {
            prefix: "provider_",
            provider_family: "spirv:vulkan-gpu",
            kernel_id: sample.kernel_id,
            operation: sample.operation,
            kernel_input_buffers: "input.values,input.right",
            buffer_layout: "tensor-row-major",
            buffer_shape: "2x2",
            row_stride_bytes: 8,
            dispatch: "4x1x1",
            input_file_name: sample.input_file_name,
            input_hash: fnv1a64_hex(INPUT),
            input_byte_length: INPUT.len(),
            expected_file_name: sample.expected_file_name,
            expected: sample.expected,
            asset: &asset,
            bytes: &asset.bytes,
            input_binding: render_u32_pair_artifact_binding(
                "provider_",
                "tensor-row-major",
                "2x2",
                8,
                sample.input_file_name,
                INPUT,
                right_file_name,
                right,
            ),
            dependency: render_dependency_count_zero("provider_"),
            output_evidence: None,
        });
    }
    render_u32_sample_request_evidence(
        "spirv:vulkan-gpu",
        sample.kernel_id,
        sample.operation,
        sample.input_file_name,
        INPUT,
        sample.expected_file_name,
        sample.expected,
        &asset,
        &asset.bytes,
    )
}

fn vulkan_chain_code_asset_identity(
    add: &nuisc::registry::NustarCodeAssetRegistration,
    xor: &nuisc::registry::NustarCodeAssetRegistration,
) -> Result<crate::artifact_code_asset_identity::AssembledCodeAssetIdentity, String> {
    use crate::artifact_code_asset_identity::NustarCodeAssetContribution;
    crate::artifact_code_asset_identity::assemble_nustar_code_asset_identity(
        Path::new("nustar-packages"),
        &[
            NustarCodeAssetContribution {
                request_index: 0,
                owner_package_id: &add.package_id,
                provider_family: "spirv:vulkan-gpu",
                asset_id: &add.asset_id,
                format: &add.format,
                target: &add.target,
                entry: &add.entry,
                path: &add.file_name,
                bytes: &add.bytes,
            },
            NustarCodeAssetContribution {
                request_index: 1,
                owner_package_id: &xor.package_id,
                provider_family: "spirv:vulkan-gpu",
                asset_id: &xor.asset_id,
                format: &xor.format,
                target: &xor.target,
                entry: &xor.entry,
                path: &xor.file_name,
                bytes: &xor.bytes,
            },
        ],
    )
}

fn render_chain_artifact_request(
    index: usize,
    sample: VulkanSampleSpec,
    asset: &nuisc::registry::NustarCodeAssetRegistration,
    bytes: &[u8],
) -> String {
    let prefix = format!("provider_request_{index}_");
    render_u32_prefixed_request_evidence(U32RequestEvidence {
        prefix: &prefix,
        provider_family: "spirv:vulkan-gpu",
        kernel_id: "shader.vulkan.chain.add-u32",
        operation: sample.operation,
        kernel_input_buffers: "input.values",
        buffer_layout: "tensor-contiguous",
        buffer_shape: "4",
        row_stride_bytes: INPUT.len(),
        dispatch: "4x1x1",
        input_file_name: CHAIN_INPUT_FILE_NAME,
        input_hash: fnv1a64_hex(INPUT),
        input_byte_length: INPUT.len(),
        expected_file_name: CHAIN_ADD_EXPECTED_FILE_NAME,
        expected: ADD_EXPECTED,
        asset,
        bytes,
        input_binding: render_u32_artifact_binding(&prefix, CHAIN_INPUT_FILE_NAME, INPUT),
        dependency: render_dependency_count_zero(&prefix),
        output_evidence: None,
    })
}

fn render_chain_dependency_request(
    index: usize,
    sample: VulkanSampleSpec,
    asset: &nuisc::registry::NustarCodeAssetRegistration,
    bytes: &[u8],
) -> String {
    let prefix = format!("provider_request_{index}_");
    let input_hash = fnv1a64_hex(ADD_EXPECTED);
    render_u32_prefixed_request_evidence(U32RequestEvidence {
        prefix: &prefix,
        provider_family: "spirv:vulkan-gpu",
        kernel_id: "shader.vulkan.chain.xor-u32",
        operation: sample.operation,
        kernel_input_buffers: "input.values",
        buffer_layout: "tensor-contiguous",
        buffer_shape: "4",
        row_stride_bytes: ADD_EXPECTED.len(),
        dispatch: "4x1x1",
        input_file_name: "none",
        input_hash: input_hash.clone(),
        input_byte_length: ADD_EXPECTED.len(),
        expected_file_name: CHAIN_EXPECTED_FILE_NAME,
        expected: CHAIN_EXPECTED,
        asset,
        bytes,
        input_binding: render_u32_dependency_binding(
            &prefix,
            &input_hash,
            ADD_EXPECTED.len(),
            "shader.vulkan.chain.add-u32",
            "output.values",
        ),
        dependency: render_u32_dependency_edge(
            &prefix,
            "shader.vulkan.chain.add-u32",
            "output.values",
            "input.values",
            "glm:provider-edge:shader.vulkan.chain.add-u32:output.values->shader.vulkan.chain.xor-u32:input.values",
        ),
        output_evidence: None,
    })
}

fn persist_vulkan_payloads(output_dir: &Path, evidence: &[&str]) -> Result<(), String> {
    if evidence.iter().any(|item| evidence_is_chain(item)) {
        let add = vulkan_asset(ADD_SAMPLE)?;
        let xor = vulkan_asset(XOR_SAMPLE)?;
        for asset in [&add, &xor] {
            let actual = fs::read(output_dir.join(&asset.file_name)).map_err(|error| {
                format!("failed to read Nuis-emitted Vulkan SPIR-V asset: {error}")
            })?;
            validate_vulkan_code_asset(asset, &actual)?;
        }
        for (name, bytes) in [
            (CHAIN_INPUT_FILE_NAME, INPUT),
            (CHAIN_ADD_EXPECTED_FILE_NAME, ADD_EXPECTED),
            (CHAIN_EXPECTED_FILE_NAME, CHAIN_EXPECTED),
        ] {
            fs::write(output_dir.join(name), bytes)
                .map_err(|error| format!("failed to persist Vulkan u32 graph payload: {error}"))?;
        }
        return Ok(());
    }
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
    if let Some((name, bytes)) = sample.aux_input {
        fs::write(output_dir.join(name), bytes)
            .map_err(|error| format!("failed to persist Vulkan u32 pair payload: {error}"))?;
    }
    Ok(())
}

fn resolve_vulkan_code_asset_evidence(output_dir: &Path, evidence: &str) -> Result<String, String> {
    if evidence_is_chain(evidence) {
        return resolve_vulkan_chain_code_asset_evidence(output_dir, evidence);
    }
    let sample = sample_from_evidence(evidence)
        .ok_or_else(|| "Vulkan provider sample evidence is missing registration ID".to_owned())?;
    let asset = vulkan_asset(sample)?;
    let actual = fs::read(output_dir.join(&asset.file_name))
        .map_err(|error| format!("failed to read Nuis-emitted Vulkan SPIR-V asset: {error}"))?;
    validate_vulkan_code_asset(&asset, &actual)?;
    validate_code_asset_request_evidence("Vulkan", &asset, &actual, evidence, "provider_")?;
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
            validate_code_asset_contribution_selection("Vulkan", selection, evidence, "provider_")?;
            crate::artifact_code_asset_contribution_table::render_selected_contribution_set_evidence(
                std::slice::from_ref(selection),
            )
        })
        .transpose()?;
    let mut resolved = replace_code_asset_identity_fields(
        evidence,
        &[("provider_".to_owned(), actual.len(), fnv1a64_hex(&actual))],
    );
    if let Some(selection_evidence) = selection_evidence {
        resolved.push(';');
        resolved.push_str(&selection_evidence);
    }
    Ok(resolved)
}

fn resolve_vulkan_chain_code_asset_evidence(
    output_dir: &Path,
    evidence: &str,
) -> Result<String, String> {
    let mut selections = Vec::new();
    let mut replacements = Vec::new();
    for (index, sample) in [(0usize, ADD_SAMPLE), (1, XOR_SAMPLE)] {
        let prefix = format!("provider_request_{index}_");
        let asset = vulkan_asset(sample)?;
        let actual = fs::read(output_dir.join(&asset.file_name))
            .map_err(|error| format!("failed to read Nuis-emitted Vulkan SPIR-V asset: {error}"))?;
        validate_vulkan_code_asset(&asset, &actual)?;
        validate_code_asset_request_evidence("Vulkan", &asset, &actual, evidence, &prefix)?;
        let selection =
            crate::artifact_code_asset_contribution_table::select_compiled_code_asset_contribution(
                output_dir,
                PACKAGE_ID,
                "shader",
                LOWERING_TARGET,
                &asset.format,
                &asset.target,
                std::slice::from_ref(&asset.entry),
            )?
            .ok_or_else(|| {
                format!(
                    "compiled Vulkan contribution for `{}` is unavailable",
                    asset.asset_id
                )
            })?;
        validate_code_asset_contribution_selection("Vulkan", &selection, evidence, &prefix)?;
        selections.push(selection);
        replacements.push((prefix, actual.len(), fnv1a64_hex(&actual)));
    }
    let mut resolved = replace_code_asset_identity_fields(evidence, &replacements);
    resolved.push(';');
    resolved.push_str(
        &crate::artifact_code_asset_contribution_table::render_selected_contribution_set_evidence(
            &selections,
        )?,
    );
    Ok(resolved)
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

fn evidence_is_chain(evidence: &str) -> bool {
    evidence
        .split(';')
        .any(|field| field == format!("provider_sample_registration_id={CHAIN_REGISTRATION_ID}"))
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn registrations_own_vulkan_u32_request_and_payload_contracts() {
        for (registration, sample) in [
            (registration(), COPY_SAMPLE),
            (add_registration(), ADD_SAMPLE),
            (add_pair_registration(), ADD_PAIR_SAMPLE),
            (sub_registration(), SUB_SAMPLE),
            (mul_registration(), MUL_SAMPLE),
            (xor_registration(), XOR_SAMPLE),
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
            if sample.aux_input.is_some() {
                assert!(evidence
                    .contains("provider_input_binding_contract=nuis-provider-input-binding-v2"));
                assert!(evidence.contains("provider_buffer_layout=tensor-row-major"));
                assert!(evidence.contains("provider_buffer_shape=2x2"));
                assert!(evidence.contains("provider_input_binding_1_row_stride_bytes=8"));
            }
            assert!(nsdb::validate_provider_request_evidence(&evidence));
        }

        let chain = chain_registration();
        let evidence = (chain.enrich_evidence)("ignored");
        assert_eq!(chain.package_id, PACKAGE_ID);
        assert_eq!(chain.registration_id, CHAIN_REGISTRATION_ID);
        assert!((chain.supports)("vulkan", "discrete-or-integrated-gpu"));
        assert!((chain.metadata_selector.unwrap())(&format!(
            "artifact_provider_metadata_0={CHAIN_METADATA_SELECTOR}"
        )));
        assert!(evidence.contains("provider_request_collection_contract="));
        assert!(evidence.contains("provider_request_count=2"));
        assert!(evidence.contains("provider_request_0_kernel_operation=add-u32"));
        assert!(evidence.contains("provider_request_1_kernel_operation=xor-u32"));
        assert!(evidence.contains(
            "provider_request_1_dependency_0_producer_request_id=shader.vulkan.chain.add-u32"
        ));
        assert!(evidence.contains(
            "provider_request_1_dependency_0_transport_contract=nuis-provider-edge-transport-v1"
        ));
        assert!(evidence.contains("provider_request_1_input_binding_0_source=dependency"));
        assert!(evidence.contains("provider_code_asset_identity_set_count=2"));
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

    #[test]
    fn graph_registration_persists_chained_payloads_and_rejects_spirv_drift() {
        let output_dir = env::temp_dir().join(format!(
            "nuis-shader-vulkan-provider-chain-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&output_dir);
        fs::create_dir_all(&output_dir).unwrap();
        for sample in [ADD_SAMPLE, XOR_SAMPLE] {
            let asset = vulkan_asset(sample).unwrap();
            fs::write(output_dir.join(&asset.file_name), &asset.bytes).unwrap();
        }
        persist_vulkan_payloads(
            &output_dir,
            &["provider_sample_registration_id=official.shader.vulkan-u32-chain"],
        )
        .unwrap();
        assert_eq!(
            fs::read(output_dir.join(CHAIN_INPUT_FILE_NAME)).unwrap(),
            INPUT
        );
        assert_eq!(
            fs::read(output_dir.join(CHAIN_ADD_EXPECTED_FILE_NAME)).unwrap(),
            ADD_EXPECTED
        );
        assert_eq!(
            fs::read(output_dir.join(CHAIN_EXPECTED_FILE_NAME)).unwrap(),
            CHAIN_EXPECTED
        );
        let asset = vulkan_asset(ADD_SAMPLE).unwrap();
        let mut tampered = asset.bytes.clone();
        tampered[0] ^= 1;
        fs::write(output_dir.join(&asset.file_name), tampered).unwrap();
        assert!(persist_vulkan_payloads(
            &output_dir,
            &["provider_sample_registration_id=official.shader.vulkan-u32-chain"]
        )
        .unwrap_err()
        .contains("registry ownership"));
        fs::remove_dir_all(output_dir).unwrap();
    }
}
