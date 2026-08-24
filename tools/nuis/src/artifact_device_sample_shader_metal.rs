use crate::{
    artifact_device_sample_registration::DeviceSampleInputRegistration,
    artifact_device_sample_shader_common::{
        fnv1a64_hex, render_dependency_count_zero, render_u32_artifact_binding,
        render_u32_dependency_binding, render_u32_dependency_edge,
        render_u32_pair_artifact_binding, render_u32_prefixed_request_evidence,
        render_u32_sample_request_evidence, replace_code_asset_identity_fields,
        validate_code_asset_contribution_selection, validate_code_asset_request_evidence,
        U32BindingShape, U32RequestEvidence, U32SampleRequestEvidence,
        U32_ADD_EXPECTED as ADD_EXPECTED, U32_INPUT as INPUT, U32_MUL_EXPECTED as MUL_EXPECTED,
        U32_PAIR_ADD_EXPECTED as PAIR_ADD_EXPECTED, U32_PAIR_RIGHT_INPUT as PAIR_RIGHT_INPUT,
        U32_ZERO_EXPECTED as SUB_EXPECTED,
    },
};
use std::{fs, path::Path};

const PACKAGE_ID: &str = "official.shader";
const LOWERING_TARGET: &str = "metal.apple-silicon-gpu";
const XOR_EXPECTED: &[u8] = SUB_EXPECTED;
const CHAIN_EXPECTED: &[u8] = XOR_EXPECTED;
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
    aux_input: Option<(&'static str, &'static [u8])>,
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
    aux_input: None,
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
    aux_input: None,
    expected_file_name: "nuis.shader.metal.add-u32.expected.u32.bin",
    entry_proof: "kernel void nuis_metal_add_u32",
    expected: ADD_EXPECTED,
};

const ADD_PAIR_SAMPLE: MetalSampleSpec = MetalSampleSpec {
    registration_id: "official.shader.metal-add-pair-u32",
    metadata_selector: "official.shader:provider-sample=metal-add-pair-u32",
    asset_id: "shader.metal.add-pair-u32.msl",
    kernel_id: "shader.metal.add-pair-u32",
    operation: "add-pair-u32",
    input_file_name: "nuis.shader.metal.add-pair-u32.left.u32.bin",
    aux_input: Some((
        "nuis.shader.metal.add-pair-u32.right.u32.bin",
        PAIR_RIGHT_INPUT,
    )),
    expected_file_name: "nuis.shader.metal.add-pair-u32.expected.u32.bin",
    entry_proof: "kernel void nuis_metal_add_pair_u32",
    expected: PAIR_ADD_EXPECTED,
};

const SUB_SAMPLE: MetalSampleSpec = MetalSampleSpec {
    registration_id: "official.shader.metal-sub-u32",
    metadata_selector: "official.shader:provider-sample=metal-sub-u32",
    asset_id: "shader.metal.sub-u32.msl",
    kernel_id: "shader.metal.sub-u32",
    operation: "sub-u32",
    input_file_name: "nuis.shader.metal.sub-u32.input.u32.bin",
    aux_input: None,
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
    aux_input: None,
    expected_file_name: "nuis.shader.metal.mul-u32.expected.u32.bin",
    entry_proof: "kernel void nuis_metal_mul_u32",
    expected: MUL_EXPECTED,
};

const XOR_SAMPLE: MetalSampleSpec = MetalSampleSpec {
    registration_id: "official.shader.metal-xor-u32",
    metadata_selector: "official.shader:provider-sample=metal-xor-u32",
    asset_id: "shader.metal.xor-u32.msl",
    kernel_id: "shader.metal.xor-u32",
    operation: "xor-u32",
    input_file_name: "nuis.shader.metal.xor-u32.input.u32.bin",
    aux_input: None,
    expected_file_name: "nuis.shader.metal.xor-u32.expected.u32.bin",
    entry_proof: "kernel void nuis_metal_xor_u32",
    expected: XOR_EXPECTED,
};

const SAMPLES: &[MetalSampleSpec] = &[
    COPY_SAMPLE,
    ADD_SAMPLE,
    ADD_PAIR_SAMPLE,
    SUB_SAMPLE,
    MUL_SAMPLE,
    XOR_SAMPLE,
];

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

pub(crate) fn add_pair_registration() -> DeviceSampleInputRegistration {
    registration_for(
        ADD_PAIR_SAMPLE,
        selects_shader_metal_add_pair,
        metal_add_pair_u32_evidence,
    )
}

pub(crate) fn sub_registration() -> DeviceSampleInputRegistration {
    registration_for(SUB_SAMPLE, selects_shader_metal_sub, metal_sub_u32_evidence)
}

pub(crate) fn mul_registration() -> DeviceSampleInputRegistration {
    registration_for(MUL_SAMPLE, selects_shader_metal_mul, metal_mul_u32_evidence)
}

pub(crate) fn xor_registration() -> DeviceSampleInputRegistration {
    registration_for(XOR_SAMPLE, selects_shader_metal_xor, metal_xor_u32_evidence)
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

fn selects_shader_metal_add_pair(base: &str) -> bool {
    selects_metadata(base, ADD_PAIR_SAMPLE.metadata_selector)
}

fn selects_shader_metal_sub(base: &str) -> bool {
    selects_metadata(base, SUB_SAMPLE.metadata_selector)
}

fn selects_shader_metal_mul(base: &str) -> bool {
    selects_metadata(base, MUL_SAMPLE.metadata_selector)
}

fn selects_shader_metal_xor(base: &str) -> bool {
    selects_metadata(base, XOR_SAMPLE.metadata_selector)
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

fn metal_add_pair_u32_evidence(_base: &str) -> String {
    metal_sample_evidence(ADD_PAIR_SAMPLE)
}

fn metal_sub_u32_evidence(_base: &str) -> String {
    metal_sample_evidence(SUB_SAMPLE)
}

fn metal_mul_u32_evidence(_base: &str) -> String {
    metal_sample_evidence(MUL_SAMPLE)
}

fn metal_xor_u32_evidence(_base: &str) -> String {
    metal_sample_evidence(XOR_SAMPLE)
}

fn metal_chain_u32_evidence(_base: &str) -> String {
    let add = metal_asset(ADD_SAMPLE).expect("Shader Nustar Metal add asset must be registered");
    let xor = metal_asset(XOR_SAMPLE).expect("Shader Nustar Metal xor asset must be registered");
    let identity = metal_chain_code_asset_identity(&add, &xor)
        .expect("Shader Nustar Metal chain identity must assemble");
    format!(
        "provider_request_collection_contract=nuis-provider-request-collection-v1;provider_request_count=2;{};{};{}",
        render_chain_artifact_request(0, ADD_SAMPLE, &add, &add.bytes),
        render_chain_dependency_request(1, XOR_SAMPLE, &xor, &xor.bytes),
        identity.identity_evidence,
    )
}

fn metal_sample_evidence(sample: MetalSampleSpec) -> String {
    let asset = metal_asset(sample).expect("Shader Nustar Metal MSL asset must be registered");
    if let Some((right_file_name, right)) = sample.aux_input {
        return render_u32_prefixed_request_evidence(U32RequestEvidence {
            prefix: "provider_",
            provider_family: "metal:apple-silicon-gpu",
            kernel_id: sample.kernel_id,
            operation: sample.operation,
            kernel_input_buffers: "input.values,input.right",
            buffer_layout: "tensor-row-major",
            buffer_shape: "2x2",
            row_stride_bytes: 8,
            dispatch: "4x1x1",
            element_count: 4,
            input_file_name: sample.input_file_name,
            input_hash: fnv1a64_hex(INPUT),
            input_byte_length: INPUT.len(),
            expected_file_name: sample.expected_file_name,
            expected: sample.expected,
            asset: &asset,
            bytes: &asset.bytes,
            input_binding: render_u32_pair_artifact_binding(
                U32BindingShape {
                    prefix: "provider_",
                    layout: "tensor-row-major",
                    shape: "2x2",
                    row_stride_bytes: 8,
                },
                sample.input_file_name,
                INPUT,
                right_file_name,
                right,
            ),
            dependency: render_dependency_count_zero("provider_"),
            output_evidence: None,
        });
    }
    render_u32_sample_request_evidence(U32SampleRequestEvidence {
        provider_family: "metal:apple-silicon-gpu",
        kernel_id: sample.kernel_id,
        operation: sample.operation,
        input_file_name: sample.input_file_name,
        input: INPUT,
        expected_file_name: sample.expected_file_name,
        expected: sample.expected,
        asset: &asset,
        bytes: &asset.bytes,
    })
}

fn metal_chain_code_asset_identity(
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
                owner_package_id: &xor.package_id,
                provider_family: "metal:apple-silicon-gpu",
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
    sample: MetalSampleSpec,
    asset: &nuisc::registry::NustarCodeAssetRegistration,
    bytes: &[u8],
) -> String {
    let prefix = format!("provider_request_{index}_");
    render_u32_prefixed_request_evidence(U32RequestEvidence {
        prefix: &prefix,
        provider_family: "metal:apple-silicon-gpu",
        kernel_id: "shader.metal.chain.add-u32",
        operation: sample.operation,
        kernel_input_buffers: "input.values",
        buffer_layout: "tensor-contiguous",
        buffer_shape: "4",
        row_stride_bytes: INPUT.len(),
        dispatch: "4x1x1",
        element_count: 4,
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
    sample: MetalSampleSpec,
    asset: &nuisc::registry::NustarCodeAssetRegistration,
    bytes: &[u8],
) -> String {
    let prefix = format!("provider_request_{index}_");
    let input_hash = fnv1a64_hex(ADD_EXPECTED);
    render_u32_prefixed_request_evidence(U32RequestEvidence {
        prefix: &prefix,
        provider_family: "metal:apple-silicon-gpu",
        kernel_id: "shader.metal.chain.xor-u32",
        operation: sample.operation,
        kernel_input_buffers: "input.values",
        buffer_layout: "tensor-contiguous",
        buffer_shape: "4",
        row_stride_bytes: ADD_EXPECTED.len(),
        dispatch: "4x1x1",
        element_count: 4,
        input_file_name: "none",
        input_hash: input_hash.clone(),
        input_byte_length: ADD_EXPECTED.len(),
        expected_file_name: CHAIN_EXPECTED_FILE_NAME,
        expected: CHAIN_EXPECTED,
        asset,
        bytes,
        input_binding: render_u32_dependency_binding(
            U32BindingShape {
                prefix: &prefix,
                layout: "tensor-contiguous",
                shape: "4",
                row_stride_bytes: ADD_EXPECTED.len(),
            },
            &input_hash,
            ADD_EXPECTED.len(),
            "shader.metal.chain.add-u32",
            "output.values",
        ),
        dependency: render_u32_dependency_edge(
            &prefix,
            0,
            1,
            "shader.metal.chain.add-u32",
            "output.values",
            "input.values",
            "glm:provider-edge:shader.metal.chain.add-u32:output.values->shader.metal.chain.xor-u32:input.values",
        ),
        output_evidence: None,
    })
}

fn persist_metal_payloads(output_dir: &Path, evidence: &[&str]) -> Result<(), String> {
    if evidence.iter().any(|item| evidence_is_chain(item)) {
        for sample in [ADD_SAMPLE, XOR_SAMPLE] {
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
    if let Some((name, bytes)) = sample.aux_input {
        fs::write(output_dir.join(name), bytes)
            .map_err(|error| format!("failed to persist Metal u32 pair payload: {error}"))?;
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
    validate_code_asset_request_evidence("Metal", &asset, &actual, evidence, "provider_")?;
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
            validate_code_asset_contribution_selection("Metal", selection, evidence, "provider_")?;
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
    for (index, sample) in [(0usize, ADD_SAMPLE), (1, XOR_SAMPLE)] {
        let prefix = format!("provider_request_{index}_");
        let asset = metal_asset(sample)?;
        let actual = fs::read(output_dir.join(&asset.file_name))
            .map_err(|error| format!("failed to read Nuis-emitted Metal MSL asset: {error}"))?;
        validate_metal_code_asset(sample, &asset, &actual)?;
        validate_code_asset_request_evidence("Metal", &asset, &actual, evidence, &prefix)?;
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
        validate_code_asset_contribution_selection("Metal", &selection, evidence, &prefix)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn registrations_are_explicit_and_own_generated_msl_requests() {
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
        assert!((chain.supports)("metal", "apple-silicon-gpu"));
        assert!((chain.metadata_selector.unwrap())(&format!(
            "artifact_provider_metadata_0={CHAIN_METADATA_SELECTOR}"
        )));
        assert!(evidence.contains("provider_request_collection_contract="));
        assert!(evidence.contains("provider_request_count=2"));
        assert!(evidence.contains("provider_request_0_kernel_operation=add-u32"));
        assert!(evidence.contains("provider_request_1_kernel_operation=xor-u32"));
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
        let asset = metal_asset(ADD_PAIR_SAMPLE).unwrap();
        fs::write(output_dir.join(&asset.file_name), &asset.bytes).unwrap();
        persist_metal_payloads(
            &output_dir,
            &["provider_sample_registration_id=official.shader.metal-add-pair-u32"],
        )
        .unwrap();
        assert_eq!(
            fs::read(output_dir.join(ADD_PAIR_SAMPLE.input_file_name)).unwrap(),
            INPUT
        );
        assert_eq!(
            fs::read(output_dir.join(ADD_PAIR_SAMPLE.aux_input.unwrap().0)).unwrap(),
            PAIR_RIGHT_INPUT
        );
        assert_eq!(
            fs::read(output_dir.join(ADD_PAIR_SAMPLE.expected_file_name)).unwrap(),
            PAIR_ADD_EXPECTED
        );
        let tampered = String::from_utf8(asset.bytes.clone())
            .unwrap()
            .replace("nuis-module-lowering-plan", "nuis-module-lowering-drift");
        fs::write(output_dir.join(&asset.file_name), tampered).unwrap();
        assert!(persist_metal_payloads(
            &output_dir,
            &["provider_sample_registration_id=official.shader.metal-add-pair-u32"]
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
        for sample in [ADD_SAMPLE, XOR_SAMPLE] {
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
