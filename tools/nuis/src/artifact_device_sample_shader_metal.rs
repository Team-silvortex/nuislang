use crate::artifact_device_sample_registration::DeviceSampleInputRegistration;
use std::{fs, path::Path};

const PACKAGE_ID: &str = "official.shader";
const LOWERING_TARGET: &str = "metal.apple-silicon-gpu";
const DIGEST_CONTRACT: &str = "nuis-code-asset-digest-fnv1a64-v1";
const INPUT: &[u8] = &[1, 0, 0, 0, 8, 0, 0, 0, 13, 0, 0, 0, 21, 0, 0, 0];
const ADD_EXPECTED: &[u8] = &[2, 0, 0, 0, 16, 0, 0, 0, 26, 0, 0, 0, 42, 0, 0, 0];
const SUB_EXPECTED: &[u8] = &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
const MUL_EXPECTED: &[u8] = &[1, 0, 0, 0, 64, 0, 0, 0, 169, 0, 0, 0, 185, 1, 0, 0];
const CHAIN_EXPECTED: &[u8] = &[4, 0, 0, 0, 0, 1, 0, 0, 164, 2, 0, 0, 228, 6, 0, 0];
const CHAIN_REGISTRATION_ID: &str = "official.shader.metal-u32-chain";
const CHAIN_METADATA_SELECTOR: &str = "official.shader:provider-sample=metal-u32-chain";
const CHAIN_INPUT_FILE_NAME: &str = "nuis.shader.metal.chain.input.u32.bin";
const CHAIN_ADD_EXPECTED_FILE_NAME: &str = "nuis.shader.metal.chain.add.expected.u32.bin";
const CHAIN_EXPECTED_FILE_NAME: &str = "nuis.shader.metal.chain.expected.u32.bin";

#[derive(Clone, Copy)]
struct MetalSampleSpec {
    registration_id: &'static str,
    metadata_selector: &'static str,
    asset_id: &'static str,
    kernel_id: &'static str,
    operation: &'static str,
    input_file_name: &'static str,
    expected_file_name: &'static str,
    entry_proof: &'static str,
    expected: &'static [u8],
}

const COPY_SAMPLE: MetalSampleSpec = MetalSampleSpec {
    registration_id: "official.shader.metal-copy-u32",
    metadata_selector: "official.shader:provider-sample=metal-copy-u32",
    asset_id: "shader.metal.copy-u32.msl",
    kernel_id: "shader.metal.copy-u32",
    operation: "copy-u32",
    input_file_name: "nuis.shader.metal.copy-u32.input.u32.bin",
    expected_file_name: "nuis.shader.metal.copy-u32.expected.u32.bin",
    entry_proof: "kernel void nuis_metal_copy_u32",
    expected: INPUT,
};

const ADD_SAMPLE: MetalSampleSpec = MetalSampleSpec {
    registration_id: "official.shader.metal-add-u32",
    metadata_selector: "official.shader:provider-sample=metal-add-u32",
    asset_id: "shader.metal.add-u32.msl",
    kernel_id: "shader.metal.add-u32",
    operation: "add-u32",
    input_file_name: "nuis.shader.metal.add-u32.input.u32.bin",
    expected_file_name: "nuis.shader.metal.add-u32.expected.u32.bin",
    entry_proof: "kernel void nuis_metal_add_u32",
    expected: ADD_EXPECTED,
};

const SUB_SAMPLE: MetalSampleSpec = MetalSampleSpec {
    registration_id: "official.shader.metal-sub-u32",
    metadata_selector: "official.shader:provider-sample=metal-sub-u32",
    asset_id: "shader.metal.sub-u32.msl",
    kernel_id: "shader.metal.sub-u32",
    operation: "sub-u32",
    input_file_name: "nuis.shader.metal.sub-u32.input.u32.bin",
    expected_file_name: "nuis.shader.metal.sub-u32.expected.u32.bin",
    entry_proof: "kernel void nuis_metal_sub_u32",
    expected: SUB_EXPECTED,
};

const MUL_SAMPLE: MetalSampleSpec = MetalSampleSpec {
    registration_id: "official.shader.metal-mul-u32",
    metadata_selector: "official.shader:provider-sample=metal-mul-u32",
    asset_id: "shader.metal.mul-u32.msl",
    kernel_id: "shader.metal.mul-u32",
    operation: "mul-u32",
    input_file_name: "nuis.shader.metal.mul-u32.input.u32.bin",
    expected_file_name: "nuis.shader.metal.mul-u32.expected.u32.bin",
    entry_proof: "kernel void nuis_metal_mul_u32",
    expected: MUL_EXPECTED,
};

const SAMPLES: &[MetalSampleSpec] = &[COPY_SAMPLE, ADD_SAMPLE, SUB_SAMPLE, MUL_SAMPLE];

pub(crate) fn registration() -> DeviceSampleInputRegistration {
    registration_for(
        COPY_SAMPLE,
        selects_shader_metal_copy,
        metal_copy_u32_evidence,
    )
}

pub(crate) fn add_registration() -> DeviceSampleInputRegistration {
    registration_for(ADD_SAMPLE, selects_shader_metal_add, metal_add_u32_evidence)
}

pub(crate) fn sub_registration() -> DeviceSampleInputRegistration {
    registration_for(SUB_SAMPLE, selects_shader_metal_sub, metal_sub_u32_evidence)
}

pub(crate) fn mul_registration() -> DeviceSampleInputRegistration {
    registration_for(MUL_SAMPLE, selects_shader_metal_mul, metal_mul_u32_evidence)
}

pub(crate) fn chain_registration() -> DeviceSampleInputRegistration {
    DeviceSampleInputRegistration {
        package_id: PACKAGE_ID,
        registration_id: CHAIN_REGISTRATION_ID,
        provider_family: "metal:apple-silicon-gpu",
        supports: supports_metal,
        metadata_selector: Some(selects_shader_metal_chain),
        enrich_evidence: metal_chain_u32_evidence,
        resolve_evidence: Some(resolve_metal_code_asset_evidence),
        persist_payloads: persist_metal_payloads,
    }
}

fn registration_for(
    sample: MetalSampleSpec,
    metadata_selector: fn(&str) -> bool,
    enrich_evidence: fn(&str) -> String,
) -> DeviceSampleInputRegistration {
    DeviceSampleInputRegistration {
        package_id: PACKAGE_ID,
        registration_id: sample.registration_id,
        provider_family: "metal:apple-silicon-gpu",
        supports: supports_metal,
        metadata_selector: Some(metadata_selector),
        enrich_evidence,
        resolve_evidence: Some(resolve_metal_code_asset_evidence),
        persist_payloads: persist_metal_payloads,
    }
}

fn supports_metal(backend_family: &str, target_device: &str) -> bool {
    backend_family == "metal" && target_device == "apple-silicon-gpu"
}

fn selects_shader_metal_copy(base: &str) -> bool {
    selects_metadata(base, COPY_SAMPLE.metadata_selector)
}

fn selects_shader_metal_add(base: &str) -> bool {
    selects_metadata(base, ADD_SAMPLE.metadata_selector)
}

fn selects_shader_metal_sub(base: &str) -> bool {
    selects_metadata(base, SUB_SAMPLE.metadata_selector)
}

fn selects_shader_metal_mul(base: &str) -> bool {
    selects_metadata(base, MUL_SAMPLE.metadata_selector)
}

fn selects_shader_metal_chain(base: &str) -> bool {
    selects_metadata(base, CHAIN_METADATA_SELECTOR)
}

fn selects_metadata(base: &str, selector: &str) -> bool {
    base.split(';')
        .filter_map(|field| field.split_once('='))
        .any(|(key, value)| key.starts_with("artifact_provider_metadata_") && value == selector)
}

fn metal_copy_u32_evidence(_base: &str) -> String {
    metal_sample_evidence(COPY_SAMPLE)
}

fn metal_add_u32_evidence(_base: &str) -> String {
    metal_sample_evidence(ADD_SAMPLE)
}

fn metal_sub_u32_evidence(_base: &str) -> String {
    metal_sample_evidence(SUB_SAMPLE)
}

fn metal_mul_u32_evidence(_base: &str) -> String {
    metal_sample_evidence(MUL_SAMPLE)
}

fn metal_chain_u32_evidence(_base: &str) -> String {
    let add = metal_asset(ADD_SAMPLE).expect("Shader Nustar Metal add asset must be registered");
    let mul = metal_asset(MUL_SAMPLE).expect("Shader Nustar Metal mul asset must be registered");
    let identity = metal_chain_code_asset_identity(&add, &mul)
        .expect("Shader Nustar Metal chain identity must assemble");
    format!(
        "provider_request_collection_contract=nuis-provider-request-collection-v1;provider_request_count=2;{};{};{}",
        render_chain_artifact_request(0, ADD_SAMPLE, &add, &add.bytes),
        render_chain_dependency_request(1, MUL_SAMPLE, &mul, &mul.bytes),
        identity.identity_evidence,
    )
}

fn metal_sample_evidence(sample: MetalSampleSpec) -> String {
    let asset = metal_asset(sample).expect("Shader Nustar Metal MSL asset must be registered");
    render_request_evidence(sample, &asset, &asset.bytes)
}

fn metal_chain_code_asset_identity(
    add: &nuisc::registry::NustarCodeAssetRegistration,
    mul: &nuisc::registry::NustarCodeAssetRegistration,
) -> Result<crate::artifact_code_asset_identity::AssembledCodeAssetIdentity, String> {
    use crate::artifact_code_asset_identity::NustarCodeAssetContribution;
    crate::artifact_code_asset_identity::assemble_nustar_code_asset_identity(
        Path::new("nustar-packages"),
        &[
            NustarCodeAssetContribution {
                request_index: 0,
                owner_package_id: &add.package_id,
                provider_family: "metal:apple-silicon-gpu",
                asset_id: &add.asset_id,
                format: &add.format,
                target: &add.target,
                entry: &add.entry,
                path: &add.file_name,
                bytes: &add.bytes,
            },
            NustarCodeAssetContribution {
                request_index: 1,
                owner_package_id: &mul.package_id,
                provider_family: "metal:apple-silicon-gpu",
                asset_id: &mul.asset_id,
                format: &mul.format,
                target: &mul.target,
                entry: &mul.entry,
                path: &mul.file_name,
                bytes: &mul.bytes,
            },
        ],
    )
}

fn render_request_evidence(
    sample: MetalSampleSpec,
    asset: &nuisc::registry::NustarCodeAssetRegistration,
    bytes: &[u8],
) -> String {
    format!(
        "provider_buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1;provider_buffer_id=input.values;provider_buffer_element_type=u32;provider_buffer_layout=tensor-contiguous;provider_buffer_shape=4;provider_buffer_row_stride_bytes=16;provider_buffer_byte_length={};provider_buffer_payload_path={};provider_buffer_content_hash={};provider_kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1;provider_kernel_id={};provider_kernel_operation={};provider_kernel_input_buffer=input.values;provider_kernel_input_buffers=input.values;provider_kernel_output_buffer=output.values;provider_kernel_dispatch=4x1x1;provider_kernel_scalar_bindings=element_count:u32:4;provider_code_asset_descriptor_contract=nuis-provider-code-asset-descriptor-v1;provider_code_asset_id={};provider_code_asset_format={};provider_code_asset_target={};provider_code_asset_entry={};provider_code_asset_path={};provider_code_asset_byte_length={};provider_code_asset_digest_contract={DIGEST_CONTRACT};provider_code_asset_content_hash={};provider_output_binding_contract=nuis-provider-output-binding-v1;provider_output_binding_count=1;provider_output_binding_0_role=output.result;provider_output_binding_0_buffer=output.values;provider_output_binding_0_element_type=u32;provider_output_binding_0_shape=4;provider_output_binding_0_byte_length={};provider_output_binding_0_comparison_id=comparison.output.values;provider_output_comparison_id=comparison.output.values;provider_output_comparison_descriptor_contract=nuis-provider-output-comparison-descriptor-v1;provider_output_comparison_output_buffer=output.values;provider_output_comparison_element_type=u32;provider_output_comparison_shape=4;provider_output_comparison_expected_path={};provider_output_comparison_expected_byte_length={};provider_output_comparison_expected_content_hash={};provider_output_comparison_absolute_tolerance=0;provider_output_comparison_relative_tolerance=0;provider_output_comparison_non_finite_policy=reject;provider_dependency_contract=nuis-provider-request-dependency-v1;provider_dependency_count=0;provider_input_binding_contract=nuis-provider-input-binding-v1;provider_input_binding_count=1;provider_input_binding_0_name=input.values;provider_input_binding_0_source=artifact;provider_input_binding_0_element_type=u32;provider_input_binding_0_shape=4;provider_input_binding_0_byte_length={};provider_input_binding_0_content_hash={};provider_input_binding_0_payload_path={};provider_input_binding_0_producer_request_id=none;provider_input_binding_0_producer_output_buffer=none;provider_adapter_binding_contract=nuis-provider-request-adapter-binding-v1;provider_adapter_binding_provider_family=metal:apple-silicon-gpu;provider_adapter_binding_execution_requirement=real-device",
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

fn render_chain_artifact_request(
    index: usize,
    sample: MetalSampleSpec,
    asset: &nuisc::registry::NustarCodeAssetRegistration,
    bytes: &[u8],
) -> String {
    let prefix = format!("provider_request_{index}_");
    render_prefixed_request_evidence(
        &prefix,
        sample,
        asset,
        bytes,
        "shader.metal.chain.add-u32",
        CHAIN_INPUT_FILE_NAME,
        fnv1a64_hex(INPUT),
        CHAIN_ADD_EXPECTED_FILE_NAME,
        ADD_EXPECTED,
        render_artifact_binding(&prefix, CHAIN_INPUT_FILE_NAME, &fnv1a64_hex(INPUT)),
        render_dependency_count_zero(&prefix),
    )
}

fn render_chain_dependency_request(
    index: usize,
    sample: MetalSampleSpec,
    asset: &nuisc::registry::NustarCodeAssetRegistration,
    bytes: &[u8],
) -> String {
    let prefix = format!("provider_request_{index}_");
    render_prefixed_request_evidence(
        &prefix,
        sample,
        asset,
        bytes,
        "shader.metal.chain.mul-u32",
        "none",
        fnv1a64_hex(ADD_EXPECTED),
        CHAIN_EXPECTED_FILE_NAME,
        CHAIN_EXPECTED,
        render_dependency_binding(&prefix, &fnv1a64_hex(ADD_EXPECTED)),
        render_chain_dependency(&prefix),
    )
}

#[allow(clippy::too_many_arguments)]
fn render_prefixed_request_evidence(
    prefix: &str,
    sample: MetalSampleSpec,
    asset: &nuisc::registry::NustarCodeAssetRegistration,
    bytes: &[u8],
    kernel_id: &str,
    input_file_name: &str,
    input_hash: String,
    expected_file_name: &str,
    expected: &[u8],
    input_binding: String,
    dependency: String,
) -> String {
    format!(
        "{prefix}buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1;{prefix}buffer_id=input.values;{prefix}buffer_element_type=u32;{prefix}buffer_layout=tensor-contiguous;{prefix}buffer_shape=4;{prefix}buffer_row_stride_bytes=16;{prefix}buffer_byte_length={};{prefix}buffer_payload_path={input_file_name};{prefix}buffer_content_hash={input_hash};{prefix}kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1;{prefix}kernel_id={kernel_id};{prefix}kernel_operation={};{prefix}kernel_input_buffer=input.values;{prefix}kernel_input_buffers=input.values;{prefix}kernel_output_buffer=output.values;{prefix}kernel_dispatch=4x1x1;{prefix}kernel_scalar_bindings=element_count:u32:4;{prefix}code_asset_descriptor_contract=nuis-provider-code-asset-descriptor-v1;{prefix}code_asset_id={};{prefix}code_asset_format={};{prefix}code_asset_target={};{prefix}code_asset_entry={};{prefix}code_asset_path={};{prefix}code_asset_byte_length={};{prefix}code_asset_digest_contract={DIGEST_CONTRACT};{prefix}code_asset_content_hash={};{prefix}output_binding_contract=nuis-provider-output-binding-v1;{prefix}output_binding_count=1;{prefix}output_binding_0_role=output.result;{prefix}output_binding_0_buffer=output.values;{prefix}output_binding_0_element_type=u32;{prefix}output_binding_0_shape=4;{prefix}output_binding_0_byte_length={};{prefix}output_binding_0_comparison_id=comparison.output.values;{prefix}output_comparison_id=comparison.output.values;{prefix}output_comparison_descriptor_contract=nuis-provider-output-comparison-descriptor-v1;{prefix}output_comparison_output_buffer=output.values;{prefix}output_comparison_element_type=u32;{prefix}output_comparison_shape=4;{prefix}output_comparison_expected_path={expected_file_name};{prefix}output_comparison_expected_byte_length={};{prefix}output_comparison_expected_content_hash={};{prefix}output_comparison_absolute_tolerance=0;{prefix}output_comparison_relative_tolerance=0;{prefix}output_comparison_non_finite_policy=reject;{dependency};{input_binding};{prefix}adapter_binding_contract=nuis-provider-request-adapter-binding-v1;{prefix}adapter_binding_provider_family=metal:apple-silicon-gpu;{prefix}adapter_binding_execution_requirement=real-device",
        INPUT.len(),
        sample.operation,
        asset.asset_id,
        asset.format,
        asset.target,
        asset.entry,
        asset.file_name,
        bytes.len(),
        fnv1a64_hex(bytes),
        expected.len(),
        expected.len(),
        fnv1a64_hex(expected),
    )
}

fn render_dependency_count_zero(prefix: &str) -> String {
    format!(
        "{prefix}dependency_contract=nuis-provider-request-dependency-v1;{prefix}dependency_count=0"
    )
}

fn render_chain_dependency(prefix: &str) -> String {
    format!(
        "{prefix}dependency_contract=nuis-provider-request-dependency-v1;{prefix}dependency_count=1;{prefix}dependency_0_producer_request_id=shader.metal.chain.add-u32;{prefix}dependency_0_producer_output_buffer=output.values;{prefix}dependency_0_consumer_input_buffer=input.values;{prefix}dependency_0_transport_contract=nuis-provider-edge-transport-v1;{prefix}dependency_0_transport_ownership_token=glm:provider-edge:shader.metal.chain.add-u32:output.values->shader.metal.chain.mul-u32:input.values;{prefix}dependency_0_transport_staging_mode=auto;{prefix}dependency_0_transport_producer_clock_evidence=provider-clock:request-0:completed;{prefix}dependency_0_transport_consumer_clock_evidence=provider-clock:request-1:dispatch-ready"
    )
}

fn render_artifact_binding(prefix: &str, input_file_name: &str, input_hash: &str) -> String {
    format!(
        "{prefix}input_binding_contract=nuis-provider-input-binding-v1;{prefix}input_binding_count=1;{prefix}input_binding_0_name=input.values;{prefix}input_binding_0_source=artifact;{prefix}input_binding_0_element_type=u32;{prefix}input_binding_0_shape=4;{prefix}input_binding_0_byte_length={};{prefix}input_binding_0_content_hash={input_hash};{prefix}input_binding_0_payload_path={input_file_name};{prefix}input_binding_0_producer_request_id=none;{prefix}input_binding_0_producer_output_buffer=none",
        INPUT.len()
    )
}

fn render_dependency_binding(prefix: &str, input_hash: &str) -> String {
    format!(
        "{prefix}input_binding_contract=nuis-provider-input-binding-v1;{prefix}input_binding_count=1;{prefix}input_binding_0_name=input.values;{prefix}input_binding_0_source=dependency;{prefix}input_binding_0_element_type=u32;{prefix}input_binding_0_shape=4;{prefix}input_binding_0_byte_length={};{prefix}input_binding_0_content_hash={input_hash};{prefix}input_binding_0_payload_path=none;{prefix}input_binding_0_producer_request_id=shader.metal.chain.add-u32;{prefix}input_binding_0_producer_output_buffer=output.values",
        ADD_EXPECTED.len()
    )
}

fn persist_metal_payloads(output_dir: &Path, evidence: &[&str]) -> Result<(), String> {
    if evidence.iter().any(|item| evidence_is_chain(item)) {
        for sample in [ADD_SAMPLE, MUL_SAMPLE] {
            let asset = metal_asset(sample)?;
            let actual = fs::read(output_dir.join(&asset.file_name))
                .map_err(|error| format!("failed to read Nuis-emitted Metal MSL asset: {error}"))?;
            validate_metal_code_asset(sample, &asset, &actual)?;
        }
        for (name, bytes) in [
            (CHAIN_INPUT_FILE_NAME, INPUT),
            (CHAIN_ADD_EXPECTED_FILE_NAME, ADD_EXPECTED),
            (CHAIN_EXPECTED_FILE_NAME, CHAIN_EXPECTED),
        ] {
            fs::write(output_dir.join(name), bytes)
                .map_err(|error| format!("failed to persist Metal u32 graph payload: {error}"))?;
        }
        return Ok(());
    }
    let Some(sample) = sample_from_evidence_items(evidence) else {
        return Ok(());
    };
    let asset = metal_asset(sample)?;
    let actual = fs::read(output_dir.join(&asset.file_name))
        .map_err(|error| format!("failed to read Nuis-emitted Metal MSL asset: {error}"))?;
    validate_metal_code_asset(sample, &asset, &actual)?;
    for (name, bytes) in [
        (sample.input_file_name, INPUT),
        (sample.expected_file_name, sample.expected),
    ] {
        fs::write(output_dir.join(name), bytes)
            .map_err(|error| format!("failed to persist Metal u32 payload: {error}"))?;
    }
    Ok(())
}

fn resolve_metal_code_asset_evidence(output_dir: &Path, evidence: &str) -> Result<String, String> {
    if evidence_is_chain(evidence) {
        return resolve_metal_chain_code_asset_evidence(output_dir, evidence);
    }
    let sample = sample_from_evidence(evidence)
        .ok_or_else(|| "Metal provider sample evidence is missing registration ID".to_owned())?;
    let asset = metal_asset(sample)?;
    let actual = fs::read(output_dir.join(&asset.file_name))
        .map_err(|error| format!("failed to read Nuis-emitted Metal MSL asset: {error}"))?;
    validate_metal_code_asset(sample, &asset, &actual)?;
    validate_metal_request_asset_evidence(&asset, &actual, evidence, "provider_")?;
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
            validate_metal_contribution_selection(selection, evidence, "provider_")?;
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

fn resolve_metal_chain_code_asset_evidence(
    output_dir: &Path,
    evidence: &str,
) -> Result<String, String> {
    let mut selections = Vec::new();
    let mut replacements = Vec::new();
    for (index, sample) in [(0usize, ADD_SAMPLE), (1, MUL_SAMPLE)] {
        let prefix = format!("provider_request_{index}_");
        let asset = metal_asset(sample)?;
        let actual = fs::read(output_dir.join(&asset.file_name))
            .map_err(|error| format!("failed to read Nuis-emitted Metal MSL asset: {error}"))?;
        validate_metal_code_asset(sample, &asset, &actual)?;
        validate_metal_request_asset_evidence(&asset, &actual, evidence, &prefix)?;
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
                    "compiled Metal contribution for `{}` is unavailable",
                    asset.asset_id
                )
            })?;
        validate_metal_contribution_selection(&selection, evidence, &prefix)?;
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

fn replace_code_asset_identity_fields(
    evidence: &str,
    replacements: &[(String, usize, String)],
) -> String {
    evidence
        .split(';')
        .map(|field| {
            let Some((key, value)) = field.split_once('=') else {
                return field.to_owned();
            };
            for (prefix, byte_length, content_hash) in replacements {
                if key == format!("{prefix}code_asset_byte_length") {
                    return format!("{key}={byte_length}");
                }
                if key == format!("{prefix}code_asset_content_hash") {
                    return format!("{key}={content_hash}");
                }
            }
            format!("{key}={value}")
        })
        .collect::<Vec<_>>()
        .join(";")
}

fn validate_metal_request_asset_evidence(
    asset: &nuisc::registry::NustarCodeAssetRegistration,
    bytes: &[u8],
    evidence: &str,
    prefix: &str,
) -> Result<(), String> {
    for (field, expected) in [
        ("code_asset_id", asset.asset_id.as_str()),
        ("code_asset_format", asset.format.as_str()),
        ("code_asset_target", asset.target.as_str()),
        ("code_asset_entry", asset.entry.as_str()),
        ("code_asset_path", asset.file_name.as_str()),
        ("code_asset_digest_contract", DIGEST_CONTRACT),
    ] {
        let needle = format!("{prefix}{field}={expected}");
        if !evidence.split(';').any(|item| item == needle) {
            return Err(format!(
                "Metal provider request code asset field `{field}` does not match `{expected}`"
            ));
        }
    }
    let expected_length = format!("{prefix}code_asset_byte_length={}", bytes.len());
    let expected_hash = format!("{prefix}code_asset_content_hash={}", fnv1a64_hex(bytes));
    if !evidence.split(';').any(|item| item == expected_length)
        || !evidence.split(';').any(|item| item == expected_hash)
    {
        return Err("Metal provider request code asset byte identity does not match".to_owned());
    }
    Ok(())
}

fn validate_metal_contribution_selection(
    selection: &crate::artifact_code_asset_contribution_table::SelectedCodeAssetContribution,
    evidence: &str,
    prefix: &str,
) -> Result<(), String> {
    for (field, expected) in [
        ("code_asset_id", selection.asset_id.as_str()),
        ("code_asset_format", selection.format.as_str()),
        ("code_asset_target", selection.target.as_str()),
        ("code_asset_path", selection.path.as_str()),
        ("code_asset_content_hash", selection.content_hash.as_str()),
    ] {
        let needle = format!("{prefix}{field}={expected}");
        if !evidence.split(';').any(|item| item == needle) {
            return Err(format!(
                "Metal provider request does not match compiled contribution field `{field}`"
            ));
        }
    }
    Ok(())
}

fn validate_metal_code_asset(
    sample: MetalSampleSpec,
    asset: &nuisc::registry::NustarCodeAssetRegistration,
    bytes: &[u8],
) -> Result<(), String> {
    if bytes != asset.bytes {
        return Err("Nuis-emitted Metal MSL asset does not match registry ownership".to_owned());
    }
    let source = std::str::from_utf8(bytes)
        .map_err(|_| "Nuis-emitted Metal MSL asset is not UTF-8".to_owned())?;
    for required in [
        "nuis-module-lowering-plan",
        "nuis-yir.shader.backend-lowering-plan.v1",
        "msl:metal-gpu",
        "msl2.4",
        sample.entry_proof,
    ] {
        if !source.contains(required) {
            return Err(format!(
                "Nuis-emitted Metal MSL asset is missing proof `{required}`"
            ));
        }
    }
    Ok(())
}

fn metal_asset(
    sample: MetalSampleSpec,
) -> Result<nuisc::registry::NustarCodeAssetRegistration, String> {
    let root = Path::new("nustar-packages");
    let manifest = nuisc::registry::load_manifest_for_domain(root, "shader")?;
    nuisc::registry::code_asset_registration_by_id(root, &manifest, sample.asset_id)?
        .filter(|asset| {
            asset.asset_id == sample.asset_id
                && asset.package_id == PACKAGE_ID
                && asset.lowering_target == LOWERING_TARGET
        })
        .ok_or_else(|| "Shader Nustar Metal MSL code asset is not registered".to_owned())
}

fn sample_from_evidence_items(evidence: &[&str]) -> Option<MetalSampleSpec> {
    evidence.iter().find_map(|item| sample_from_evidence(item))
}

fn evidence_is_chain(evidence: &str) -> bool {
    evidence
        .split(';')
        .any(|field| field == format!("provider_sample_registration_id={CHAIN_REGISTRATION_ID}"))
}

fn sample_from_evidence(evidence: &str) -> Option<MetalSampleSpec> {
    evidence.split(';').find_map(|field| {
        field
            .strip_prefix("provider_sample_registration_id=")
            .and_then(sample_by_registration_id)
    })
}

fn sample_by_registration_id(registration_id: &str) -> Option<MetalSampleSpec> {
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
    fn registrations_are_explicit_and_own_generated_msl_requests() {
        for (registration, sample) in [
            (registration(), COPY_SAMPLE),
            (add_registration(), ADD_SAMPLE),
            (sub_registration(), SUB_SAMPLE),
            (mul_registration(), MUL_SAMPLE),
        ] {
            let evidence = (registration.enrich_evidence)("ignored");

            assert_eq!(registration.package_id, PACKAGE_ID);
            assert_eq!(registration.registration_id, sample.registration_id);
            assert!((registration.supports)("metal", "apple-silicon-gpu"));
            assert!((registration.metadata_selector.unwrap())(&format!(
                "artifact_provider_metadata_0={}",
                sample.metadata_selector
            )));
            assert!(evidence.contains(&format!("provider_kernel_operation={}", sample.operation)));
            assert!(evidence.contains("provider_code_asset_format=metal-source"));
            assert!(evidence.contains(&format!("provider_code_asset_id={}", sample.asset_id)));
            assert!(evidence.contains("provider_code_asset_target=msl2.4"));
            assert!(evidence.contains(&format!(
                "provider_code_asset_entry={}",
                sample.entry_proof.trim_start_matches("kernel void ")
            )));
            assert!(evidence.contains("provider_output_comparison_expected_content_hash="));
            assert!(evidence
                .contains("provider_adapter_binding_provider_family=metal:apple-silicon-gpu"));
            assert!(nsdb::validate_provider_request_evidence(&evidence));
        }

        let chain = chain_registration();
        let evidence = (chain.enrich_evidence)("ignored");
        assert_eq!(chain.package_id, PACKAGE_ID);
        assert_eq!(chain.registration_id, CHAIN_REGISTRATION_ID);
        assert!((chain.supports)("metal", "apple-silicon-gpu"));
        assert!((chain.metadata_selector.unwrap())(&format!(
            "artifact_provider_metadata_0={CHAIN_METADATA_SELECTOR}"
        )));
        assert!(evidence.contains("provider_request_collection_contract="));
        assert!(evidence.contains("provider_request_count=2"));
        assert!(evidence.contains("provider_request_0_kernel_operation=add-u32"));
        assert!(evidence.contains("provider_request_1_kernel_operation=mul-u32"));
        assert!(evidence.contains(
            "provider_request_1_dependency_0_producer_request_id=shader.metal.chain.add-u32"
        ));
        assert!(evidence.contains(
            "provider_request_1_dependency_0_transport_contract=nuis-provider-edge-transport-v1"
        ));
        assert!(evidence.contains("provider_request_1_input_binding_0_source=dependency"));
        assert!(evidence.contains("provider_code_asset_identity_set_count=2"));
        assert!(nsdb::validate_provider_request_evidence(&evidence));
    }

    #[test]
    fn registration_verifies_generated_msl_before_persisting_inputs() {
        let output_dir = env::temp_dir().join(format!(
            "nuis-shader-metal-provider-registration-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&output_dir);
        fs::create_dir_all(&output_dir).unwrap();
        let asset = metal_asset(ADD_SAMPLE).unwrap();
        fs::write(output_dir.join(&asset.file_name), &asset.bytes).unwrap();
        persist_metal_payloads(
            &output_dir,
            &["provider_sample_registration_id=official.shader.metal-add-u32"],
        )
        .unwrap();
        assert_eq!(
            fs::read(output_dir.join(ADD_SAMPLE.input_file_name)).unwrap(),
            INPUT
        );
        assert_eq!(
            fs::read(output_dir.join(ADD_SAMPLE.expected_file_name)).unwrap(),
            ADD_EXPECTED
        );
        let tampered = String::from_utf8(asset.bytes.clone())
            .unwrap()
            .replace("nuis-module-lowering-plan", "nuis-module-lowering-drift");
        fs::write(output_dir.join(&asset.file_name), tampered).unwrap();
        assert!(persist_metal_payloads(
            &output_dir,
            &["provider_sample_registration_id=official.shader.metal-add-u32"]
        )
        .unwrap_err()
        .contains("registry ownership"));
        fs::remove_dir_all(output_dir).unwrap();
    }

    #[test]
    fn graph_registration_persists_chained_payloads_and_rejects_msl_drift() {
        let output_dir = env::temp_dir().join(format!(
            "nuis-shader-metal-provider-chain-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&output_dir);
        fs::create_dir_all(&output_dir).unwrap();
        for sample in [ADD_SAMPLE, MUL_SAMPLE] {
            let asset = metal_asset(sample).unwrap();
            fs::write(output_dir.join(&asset.file_name), &asset.bytes).unwrap();
        }
        persist_metal_payloads(
            &output_dir,
            &["provider_sample_registration_id=official.shader.metal-u32-chain"],
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
        let asset = metal_asset(ADD_SAMPLE).unwrap();
        let tampered = String::from_utf8(asset.bytes.clone())
            .unwrap()
            .replace("nuis-module-lowering-plan", "nuis-module-lowering-drift");
        fs::write(output_dir.join(&asset.file_name), tampered).unwrap();
        assert!(persist_metal_payloads(
            &output_dir,
            &["provider_sample_registration_id=official.shader.metal-u32-chain"]
        )
        .unwrap_err()
        .contains("registry ownership"));
        fs::remove_dir_all(output_dir).unwrap();
    }
}
