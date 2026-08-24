use crate::{
    artifact_device_sample_registration::DeviceSampleInputRegistration,
    artifact_device_sample_shader_common::{
        fnv1a64_hex, render_dependency_count_zero, render_u32_dependency_binding,
        render_u32_dependency_edge, render_u32_output_evidence, render_u32_pair_artifact_binding,
        render_u32_prefixed_request_evidence, replace_code_asset_identity_fields,
        validate_code_asset_contribution_selection, validate_code_asset_request_evidence,
        U32BindingShape, U32OutputEvidence, U32RequestEvidence, U32_INPUT as INPUT,
        U32_PAIR_ADD_EXPECTED as SUM_EXPECTED, U32_PAIR_RIGHT_INPUT as RIGHT_INPUT,
    },
};
use std::{fs, path::Path};

const PACKAGE_ID: &str = "official.shader";
const REGISTRATION_ID: &str = "official.shader.vulkan-add-xor-pair-u32";
const METADATA_SELECTOR: &str = "official.shader:provider-sample=vulkan-add-xor-pair-u32";
const PADDED_REGISTRATION_ID: &str = "official.shader.vulkan-add-xor-pair-padded-u32";
const PADDED_METADATA_SELECTOR: &str =
    "official.shader:provider-sample=vulkan-add-xor-pair-padded-u32";
const REDUCED_REGISTRATION_ID: &str = "official.shader.vulkan-add-xor-pair-reduced-u32";
const REDUCED_METADATA_SELECTOR: &str =
    "official.shader:provider-sample=vulkan-add-xor-pair-reduced-u32";
const REDUCED_GRAPH_REGISTRATION_ID: &str = "official.shader.vulkan-reduced-output-fan-out-u32";
const REDUCED_GRAPH_METADATA_SELECTOR: &str =
    "official.shader:provider-sample=vulkan-reduced-output-fan-out-u32";
const ASSET_ID: &str = "shader.vulkan.add-xor-pair-u32.spirv";
const ENTRY: &str = "nuis_vulkan_add_xor_pair_u32";
pub(super) const REDUCED_ASSET_ID: &str = "shader.vulkan.add-xor-pair-reduced-u32.spirv";
pub(super) const REDUCED_ENTRY: &str = "nuis_vulkan_add_xor_pair_reduced_u32";
const COPY_ASSET_ID: &str = "shader.vulkan.copy-u32.spirv";
const COPY_ENTRY: &str = "nuis_vulkan_copy_u32";
pub(super) const XOR_ASSET_ID: &str = "shader.vulkan.xor-u32.spirv";
pub(super) const XOR_ENTRY: &str = "nuis_vulkan_xor_u32";
const LOWERING_TARGET: &str = "vulkan.discrete-or-integrated-gpu";
pub(super) const INPUT_FILE: &str = "nuis.shader.vulkan.add-xor-pair-u32.left.u32.bin";
pub(super) const RIGHT_FILE: &str = "nuis.shader.vulkan.add-xor-pair-u32.right.u32.bin";
pub(super) const SUM_FILE: &str = "nuis.shader.vulkan.add-xor-pair-u32.sum.expected.u32.bin";
const XOR_FILE: &str = "nuis.shader.vulkan.add-xor-pair-u32.xor.expected.u32.bin";
const PADDED_XOR_FILE: &str = "nuis.shader.vulkan.add-xor-pair-u32.xor-padded.expected.u32.bin";
pub(super) const REDUCED_XOR_FILE: &str =
    "nuis.shader.vulkan.add-xor-pair-reduced-u32.xor.expected.u32.bin";
pub(super) const REDUCED_ZERO_FILE: &str =
    "nuis.shader.vulkan.reduced-output-fan-out.zero.expected.u32.bin";
const XOR_EXPECTED: &[u8] = &[3, 0, 0, 0, 11, 0, 0, 0, 8, 0, 0, 0, 29, 0, 0, 0];
pub(super) const REDUCED_XOR_EXPECTED: &[u8] = &[3, 0, 0, 0, 11, 0, 0, 0];
pub(super) const REDUCED_ZERO_EXPECTED: &[u8] = &[0; 8];
const REDUCED_GRAPH_PRODUCER_ID: &str = "shader.vulkan.reduced-fan-out.add-xor-pair-u32";
const REDUCED_GRAPH_SUM_CONSUMER_ID: &str = "shader.vulkan.reduced-fan-out.copy-sum-u32";
const REDUCED_GRAPH_XOR_CONSUMER_ID: &str = "shader.vulkan.reduced-fan-out.xor-reduced-u32";
const REDUCED_GRAPH_SUM_TRANSPORT_TOKEN: &str = "glm:provider-edge:shader.vulkan.reduced-fan-out.add-xor-pair-u32:output.values->shader.vulkan.reduced-fan-out.copy-sum-u32:input.values";
const REDUCED_GRAPH_XOR_TRANSPORT_TOKEN: &str = "glm:provider-edge:shader.vulkan.reduced-fan-out.add-xor-pair-u32:output.xor->shader.vulkan.reduced-fan-out.xor-reduced-u32:input.values";
const PADDED_XOR_EXPECTED: &[u8] = &[
    3, 0, 0, 0, 11, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 29, 0, 0, 0, 0, 0, 0, 0,
];

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

pub(crate) fn padded_registration() -> DeviceSampleInputRegistration {
    DeviceSampleInputRegistration {
        package_id: PACKAGE_ID,
        registration_id: PADDED_REGISTRATION_ID,
        provider_family: "spirv:vulkan-gpu",
        supports: |backend, device| backend == "vulkan" && device == "discrete-or-integrated-gpu",
        metadata_selector: Some(selects_padded_sample),
        enrich_evidence: padded_sample_evidence,
        resolve_evidence: Some(resolve_code_asset_evidence),
        persist_payloads,
    }
}

pub(crate) fn reduced_registration() -> DeviceSampleInputRegistration {
    DeviceSampleInputRegistration {
        package_id: PACKAGE_ID,
        registration_id: REDUCED_REGISTRATION_ID,
        provider_family: "spirv:vulkan-gpu",
        supports: |backend, device| backend == "vulkan" && device == "discrete-or-integrated-gpu",
        metadata_selector: Some(selects_reduced_sample),
        enrich_evidence: reduced_sample_evidence,
        resolve_evidence: Some(resolve_reduced_code_asset_evidence),
        persist_payloads,
    }
}

pub(crate) fn reduced_graph_registration() -> DeviceSampleInputRegistration {
    DeviceSampleInputRegistration {
        package_id: PACKAGE_ID,
        registration_id: REDUCED_GRAPH_REGISTRATION_ID,
        provider_family: "spirv:vulkan-gpu",
        supports: |backend, device| backend == "vulkan" && device == "discrete-or-integrated-gpu",
        metadata_selector: Some(selects_reduced_graph_sample),
        enrich_evidence: reduced_graph_evidence,
        resolve_evidence: Some(resolve_reduced_graph_code_asset_evidence),
        persist_payloads,
    }
}

fn selects_sample(base: &str) -> bool {
    selects_metadata(base, METADATA_SELECTOR)
}

fn selects_padded_sample(base: &str) -> bool {
    selects_metadata(base, PADDED_METADATA_SELECTOR)
}

fn selects_reduced_sample(base: &str) -> bool {
    selects_metadata(base, REDUCED_METADATA_SELECTOR)
}

fn selects_reduced_graph_sample(base: &str) -> bool {
    selects_metadata(base, REDUCED_GRAPH_METADATA_SELECTOR)
}

fn selects_metadata(base: &str, selector: &str) -> bool {
    base.split(';')
        .filter_map(|field| field.split_once('='))
        .any(|(key, value)| key.starts_with("artifact_provider_metadata_") && value == selector)
}

pub(super) struct VulkanFanOutSampleEvidence<'a> {
    pub(super) prefix: &'a str,
    pub(super) asset_id: &'a str,
    pub(super) entry: &'a str,
    pub(super) kernel_id: &'a str,
    pub(super) operation: &'a str,
    pub(super) xor_file: &'a str,
    pub(super) xor_expected: &'a [u8],
    pub(super) xor_shape: &'a str,
    pub(super) xor_row_stride_bytes: usize,
}

fn sample_evidence(_base: &str) -> String {
    render_sample_evidence(VulkanFanOutSampleEvidence {
        prefix: "provider_",
        asset_id: ASSET_ID,
        entry: ENTRY,
        kernel_id: "shader.vulkan.add-xor-pair-u32",
        operation: "add-xor-pair-u32",
        xor_file: XOR_FILE,
        xor_expected: XOR_EXPECTED,
        xor_shape: "2x2",
        xor_row_stride_bytes: 8,
    })
}

fn padded_sample_evidence(_base: &str) -> String {
    render_sample_evidence(VulkanFanOutSampleEvidence {
        prefix: "provider_",
        asset_id: ASSET_ID,
        entry: ENTRY,
        kernel_id: "shader.vulkan.add-xor-pair-u32",
        operation: "add-xor-pair-u32",
        xor_file: PADDED_XOR_FILE,
        xor_expected: PADDED_XOR_EXPECTED,
        xor_shape: "2x2",
        xor_row_stride_bytes: 12,
    })
}

fn reduced_sample_evidence(_base: &str) -> String {
    render_sample_evidence(VulkanFanOutSampleEvidence {
        prefix: "provider_",
        asset_id: REDUCED_ASSET_ID,
        entry: REDUCED_ENTRY,
        kernel_id: "shader.vulkan.add-xor-pair-reduced-u32",
        operation: "add-xor-pair-reduced-u32",
        xor_file: REDUCED_XOR_FILE,
        xor_expected: REDUCED_XOR_EXPECTED,
        xor_shape: "2x1",
        xor_row_stride_bytes: 8,
    })
}

pub(super) fn render_sample_evidence(args: VulkanFanOutSampleEvidence<'_>) -> String {
    let VulkanFanOutSampleEvidence {
        prefix,
        asset_id,
        entry,
        kernel_id,
        operation,
        xor_file,
        xor_expected,
        xor_shape,
        xor_row_stride_bytes,
    } = args;
    let asset =
        asset(asset_id, entry).expect("Shader Nustar Vulkan fan-out asset must be registered");
    let output_evidence = render_u32_output_evidence(
        prefix,
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
                shape: xor_shape,
                row_stride_bytes: xor_row_stride_bytes,
                comparison_id: "comparison.output.xor",
                expected_file_name: xor_file,
                expected: xor_expected,
            },
        ],
    );
    render_u32_prefixed_request_evidence(U32RequestEvidence {
        prefix,
        provider_family: "spirv:vulkan-gpu",
        kernel_id,
        operation,
        kernel_input_buffers: "input.values,input.right",
        buffer_layout: "tensor-row-major",
        buffer_shape: "2x2",
        row_stride_bytes: 8,
        dispatch: "4x1x1",
        element_count: 4,
        input_file_name: INPUT_FILE,
        input_hash: fnv1a64_hex(INPUT),
        input_byte_length: INPUT.len(),
        expected_file_name: SUM_FILE,
        expected: SUM_EXPECTED,
        asset: &asset,
        bytes: &asset.bytes,
        input_binding: render_u32_pair_artifact_binding(
            U32BindingShape {
                prefix,
                layout: "tensor-row-major",
                shape: "2x2",
                row_stride_bytes: 8,
            },
            INPUT_FILE,
            INPUT,
            RIGHT_FILE,
            RIGHT_INPUT,
        ),
        dependency: render_dependency_count_zero(prefix),
        output_evidence: Some(output_evidence),
    })
}

fn reduced_graph_evidence(_base: &str) -> String {
    let producer = asset(REDUCED_ASSET_ID, REDUCED_ENTRY)
        .expect("Shader Nustar reduced Vulkan fan-out asset must be registered");
    let sum_consumer = asset(COPY_ASSET_ID, COPY_ENTRY)
        .expect("Shader Nustar Vulkan copy asset must be registered");
    let xor_consumer =
        asset(XOR_ASSET_ID, XOR_ENTRY).expect("Shader Nustar Vulkan xor asset must be registered");
    let identity = reduced_graph_code_asset_identity(&producer, &sum_consumer, &xor_consumer)
        .expect("Shader Nustar reduced Vulkan graph identity must assemble");
    format!(
        "provider_request_collection_contract=nuis-provider-request-collection-v1;provider_request_count=3;{};{};{};{}",
        render_sample_evidence(VulkanFanOutSampleEvidence {
            prefix: "provider_request_0_",
            asset_id: REDUCED_ASSET_ID,
            entry: REDUCED_ENTRY,
            kernel_id: REDUCED_GRAPH_PRODUCER_ID,
            operation: "add-xor-pair-reduced-u32",
            xor_file: REDUCED_XOR_FILE,
            xor_expected: REDUCED_XOR_EXPECTED,
            xor_shape: "2x1",
            xor_row_stride_bytes: 8,
        }),
        render_reduced_graph_consumer(ReducedGraphConsumer {
            request_index: 1,
            asset: &sum_consumer,
            kernel_id: REDUCED_GRAPH_SUM_CONSUMER_ID,
            operation: "copy-u32",
            producer_output_buffer: "output.values",
            shape: "2x2",
            row_stride_bytes: 8,
            input: SUM_EXPECTED,
            expected_file_name: SUM_FILE,
            expected: SUM_EXPECTED,
            dispatch: "4x1x1",
            element_count: 4,
            ownership_token: REDUCED_GRAPH_SUM_TRANSPORT_TOKEN,
        }),
        render_reduced_graph_consumer(ReducedGraphConsumer {
            request_index: 2,
            asset: &xor_consumer,
            kernel_id: REDUCED_GRAPH_XOR_CONSUMER_ID,
            operation: "xor-u32",
            producer_output_buffer: "output.xor",
            shape: "2x1",
            row_stride_bytes: 8,
            input: REDUCED_XOR_EXPECTED,
            expected_file_name: REDUCED_ZERO_FILE,
            expected: REDUCED_ZERO_EXPECTED,
            dispatch: "2x1x1",
            element_count: 2,
            ownership_token: REDUCED_GRAPH_XOR_TRANSPORT_TOKEN,
        }),
        identity.identity_evidence,
    )
}

struct ReducedGraphConsumer<'a> {
    request_index: usize,
    asset: &'a nuisc::registry::NustarCodeAssetRegistration,
    kernel_id: &'a str,
    operation: &'a str,
    producer_output_buffer: &'a str,
    shape: &'a str,
    row_stride_bytes: usize,
    input: &'a [u8],
    expected_file_name: &'a str,
    expected: &'a [u8],
    dispatch: &'a str,
    element_count: usize,
    ownership_token: &'a str,
}

fn render_reduced_graph_consumer(args: ReducedGraphConsumer<'_>) -> String {
    let prefix = format!("provider_request_{}_", args.request_index);
    let input_hash = fnv1a64_hex(args.input);
    render_u32_prefixed_request_evidence(U32RequestEvidence {
        prefix: &prefix,
        provider_family: "spirv:vulkan-gpu",
        kernel_id: args.kernel_id,
        operation: args.operation,
        kernel_input_buffers: "input.values",
        buffer_layout: "tensor-row-major",
        buffer_shape: args.shape,
        row_stride_bytes: args.row_stride_bytes,
        dispatch: args.dispatch,
        element_count: args.element_count,
        input_file_name: "none",
        input_hash: input_hash.clone(),
        input_byte_length: args.input.len(),
        expected_file_name: args.expected_file_name,
        expected: args.expected,
        asset: args.asset,
        bytes: &args.asset.bytes,
        input_binding: render_u32_dependency_binding(
            U32BindingShape {
                prefix: &prefix,
                layout: "tensor-row-major",
                shape: args.shape,
                row_stride_bytes: args.row_stride_bytes,
            },
            &input_hash,
            args.input.len(),
            REDUCED_GRAPH_PRODUCER_ID,
            args.producer_output_buffer,
        ),
        dependency: render_u32_dependency_edge(
            &prefix,
            0,
            args.request_index,
            REDUCED_GRAPH_PRODUCER_ID,
            args.producer_output_buffer,
            "input.values",
            args.ownership_token,
        ),
        output_evidence: None,
    })
}

fn reduced_graph_code_asset_identity(
    producer: &nuisc::registry::NustarCodeAssetRegistration,
    sum_consumer: &nuisc::registry::NustarCodeAssetRegistration,
    xor_consumer: &nuisc::registry::NustarCodeAssetRegistration,
) -> Result<crate::artifact_code_asset_identity::AssembledCodeAssetIdentity, String> {
    use crate::artifact_code_asset_identity::NustarCodeAssetContribution;
    crate::artifact_code_asset_identity::assemble_nustar_code_asset_identity(
        Path::new("nustar-packages"),
        &[
            NustarCodeAssetContribution {
                request_index: 0,
                owner_package_id: &producer.package_id,
                provider_family: "spirv:vulkan-gpu",
                asset_id: &producer.asset_id,
                format: &producer.format,
                target: &producer.target,
                entry: &producer.entry,
                path: &producer.file_name,
                bytes: &producer.bytes,
            },
            NustarCodeAssetContribution {
                request_index: 1,
                owner_package_id: &sum_consumer.package_id,
                provider_family: "spirv:vulkan-gpu",
                asset_id: &sum_consumer.asset_id,
                format: &sum_consumer.format,
                target: &sum_consumer.target,
                entry: &sum_consumer.entry,
                path: &sum_consumer.file_name,
                bytes: &sum_consumer.bytes,
            },
            NustarCodeAssetContribution {
                request_index: 2,
                owner_package_id: &xor_consumer.package_id,
                provider_family: "spirv:vulkan-gpu",
                asset_id: &xor_consumer.asset_id,
                format: &xor_consumer.format,
                target: &xor_consumer.target,
                entry: &xor_consumer.entry,
                path: &xor_consumer.file_name,
                bytes: &xor_consumer.bytes,
            },
        ],
    )
}

fn persist_payloads(output_dir: &Path, evidence: &[&str]) -> Result<(), String> {
    let owns = |registration_id: &str| {
        evidence.iter().any(|item| {
            item.split(';')
                .any(|field| field == format!("provider_sample_registration_id={registration_id}"))
        })
    };
    let standard = owns(REGISTRATION_ID);
    let padded = owns(PADDED_REGISTRATION_ID);
    let reduced = owns(REDUCED_REGISTRATION_ID);
    let reduced_graph = owns(REDUCED_GRAPH_REGISTRATION_ID);
    if !standard && !padded && !reduced && !reduced_graph {
        return Ok(());
    }
    for (enabled, asset_id, entry) in [
        (standard || padded, ASSET_ID, ENTRY),
        (reduced || reduced_graph, REDUCED_ASSET_ID, REDUCED_ENTRY),
        (reduced_graph, COPY_ASSET_ID, COPY_ENTRY),
        (reduced_graph, XOR_ASSET_ID, XOR_ENTRY),
    ] {
        if enabled {
            let asset = asset(asset_id, entry)?;
            let actual = fs::read(output_dir.join(&asset.file_name)).map_err(|error| {
                format!("failed to read Nuis-emitted Vulkan fan-out asset: {error}")
            })?;
            validate_asset(&asset, &actual, entry)?;
        }
    }
    for (name, bytes) in [
        (INPUT_FILE, INPUT),
        (RIGHT_FILE, RIGHT_INPUT),
        (SUM_FILE, SUM_EXPECTED),
    ] {
        fs::write(output_dir.join(name), bytes)
            .map_err(|error| format!("failed to persist Vulkan fan-out payload: {error}"))?;
    }
    for (enabled, name, bytes) in [
        (standard, XOR_FILE, XOR_EXPECTED),
        (padded, PADDED_XOR_FILE, PADDED_XOR_EXPECTED),
        (
            reduced || reduced_graph,
            REDUCED_XOR_FILE,
            REDUCED_XOR_EXPECTED,
        ),
        (reduced_graph, REDUCED_ZERO_FILE, REDUCED_ZERO_EXPECTED),
    ] {
        if enabled {
            fs::write(output_dir.join(name), bytes)
                .map_err(|error| format!("failed to persist Vulkan fan-out payload: {error}"))?;
        }
    }
    Ok(())
}

fn resolve_code_asset_evidence(output_dir: &Path, evidence: &str) -> Result<String, String> {
    resolve_code_asset_evidence_for(output_dir, evidence, ASSET_ID, ENTRY)
}

fn resolve_reduced_code_asset_evidence(
    output_dir: &Path,
    evidence: &str,
) -> Result<String, String> {
    resolve_code_asset_evidence_for(output_dir, evidence, REDUCED_ASSET_ID, REDUCED_ENTRY)
}

fn resolve_reduced_graph_code_asset_evidence(
    output_dir: &Path,
    evidence: &str,
) -> Result<String, String> {
    let mut selections = Vec::new();
    let mut replacements = Vec::new();
    for (index, asset_id, entry) in [
        (0usize, REDUCED_ASSET_ID, REDUCED_ENTRY),
        (1, COPY_ASSET_ID, COPY_ENTRY),
        (2, XOR_ASSET_ID, XOR_ENTRY),
    ] {
        let prefix = format!("provider_request_{index}_");
        let asset = asset(asset_id, entry)?;
        let actual = fs::read(output_dir.join(&asset.file_name)).map_err(|error| {
            format!("failed to read Nuis-emitted reduced Vulkan graph asset: {error}")
        })?;
        validate_asset(&asset, &actual, entry)?;
        validate_code_asset_request_evidence(
            "Vulkan reduced fan-out graph",
            &asset,
            &actual,
            evidence,
            &prefix,
        )?;
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
                    "compiled reduced Vulkan graph contribution for `{}` is unavailable",
                    asset.asset_id
                )
            })?;
        validate_code_asset_contribution_selection(
            "Vulkan reduced fan-out graph",
            &selection,
            evidence,
            &prefix,
        )?;
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

fn resolve_code_asset_evidence_for(
    output_dir: &Path,
    evidence: &str,
    asset_id: &str,
    entry: &str,
) -> Result<String, String> {
    let asset = asset(asset_id, entry)?;
    let actual = fs::read(output_dir.join(&asset.file_name))
        .map_err(|error| format!("failed to read Nuis-emitted Vulkan fan-out asset: {error}"))?;
    validate_asset(&asset, &actual, entry)?;
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

pub(super) fn validate_asset(
    asset: &nuisc::registry::NustarCodeAssetRegistration,
    bytes: &[u8],
    entry: &str,
) -> Result<(), String> {
    if bytes != asset.bytes {
        return Err(
            "Nuis-emitted Vulkan fan-out asset does not match registry ownership".to_owned(),
        );
    }
    if bytes.len() < 20
        || u32::from_le_bytes(bytes[0..4].try_into().expect("SPIR-V magic")) != 0x0723_0203
        || !bytes
            .windows(entry.len())
            .any(|window| window == entry.as_bytes())
    {
        return Err("Nuis-emitted Vulkan fan-out asset is not registered SPIR-V".to_owned());
    }
    Ok(())
}

pub(super) fn asset(
    asset_id: &str,
    entry: &str,
) -> Result<nuisc::registry::NustarCodeAssetRegistration, String> {
    let root = Path::new("nustar-packages");
    let manifest = nuisc::registry::load_manifest_for_domain(root, "shader")?;
    nuisc::registry::code_asset_registration_by_id(root, &manifest, asset_id)?
        .filter(|asset| {
            asset.package_id == PACKAGE_ID
                && asset.lowering_target == LOWERING_TARGET
                && asset.entry == entry
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

        let padded = padded_registration();
        let padded_evidence = (padded.enrich_evidence)("ignored");
        assert!(padded_evidence.contains("provider_output_binding_1_row_stride_bytes=12"));
        assert!(padded_evidence.contains("provider_output_binding_1_byte_length=24"));
        assert_eq!(fnv1a64_hex(PADDED_XOR_EXPECTED), "0x9adad3c97291d1e8");
        assert!(nsdb::validate_provider_request_evidence(&padded_evidence));

        let reduced = reduced_registration();
        let reduced_evidence = (reduced.enrich_evidence)("ignored");
        assert!(reduced_evidence.contains("provider_output_binding_1_shape=2x1"));
        assert!(reduced_evidence.contains("provider_output_binding_1_byte_length=8"));
        assert!(reduced_evidence
            .contains("provider_code_asset_id=shader.vulkan.add-xor-pair-reduced-u32.spirv"));
        assert!(reduced_evidence.contains("provider_kernel_operation=add-xor-pair-reduced-u32"));
        assert_eq!(fnv1a64_hex(REDUCED_XOR_EXPECTED), "0x279d73758e81abdd");
        assert!(nsdb::validate_provider_request_evidence(&reduced_evidence));

        let graph = reduced_graph_registration();
        let graph_evidence = (graph.enrich_evidence)("ignored");
        assert!(graph_evidence.contains("provider_request_count=3"));
        assert!(
            graph_evidence.contains("provider_request_1_input_binding_0_layout=tensor-row-major")
        );
        assert!(graph_evidence.contains("provider_request_1_input_binding_0_shape=2x2"));
        assert!(graph_evidence.contains("provider_request_1_input_binding_0_byte_length=16"));
        assert!(graph_evidence
            .contains("provider_request_1_dependency_0_producer_output_buffer=output.values"));
        assert!(graph_evidence.contains("provider_request_1_kernel_dispatch=4x1x1"));
        assert!(graph_evidence
            .contains("provider_request_1_kernel_scalar_bindings=element_count:u32:4"));
        assert!(graph_evidence.contains("provider_request_2_input_binding_0_shape=2x1"));
        assert!(graph_evidence.contains("provider_request_2_input_binding_0_byte_length=8"));
        assert!(graph_evidence
            .contains("provider_request_2_dependency_0_producer_output_buffer=output.xor"));
        assert!(graph_evidence.contains("provider_request_2_kernel_dispatch=2x1x1"));
        assert!(graph_evidence
            .contains("provider_request_2_kernel_scalar_bindings=element_count:u32:2"));
        assert!(graph_evidence.contains(
            "provider_request_2_dependency_0_transport_consumer_clock_evidence=provider-clock:request-2:dispatch-ready"
        ));
        assert!(graph_evidence.contains("provider_code_asset_identity_set_count=3"));
        assert_eq!(fnv1a64_hex(REDUCED_ZERO_EXPECTED), "0xa8c7f832281a39c5");
        assert!(nsdb::validate_provider_request_evidence(&graph_evidence));
    }
}
