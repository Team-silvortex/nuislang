use crate::{
    artifact_device_sample_registration::DeviceSampleInputRegistration,
    artifact_device_sample_shader_common::{
        fnv1a64_hex, render_u32_dependency_binding, render_u32_dependency_edge,
        render_u32_prefixed_request_evidence_with_scalars, replace_code_asset_identity_fields,
        validate_code_asset_contribution_selection, validate_code_asset_request_evidence,
        U32BindingShape, U32RequestEvidence, U32_INPUT as INPUT,
        U32_PAIR_ADD_EXPECTED as SUM_EXPECTED, U32_PAIR_RIGHT_INPUT as RIGHT_INPUT,
    },
    artifact_device_sample_shader_vulkan_fan_out as vulkan_fan_out,
};
use std::{fs, path::Path};

const PACKAGE_ID: &str = "official.shader";
const REGISTRATION_ID: &str = "official.shader.vulkan-cuda-reduced-output-fan-out-u32";
const METADATA_SELECTOR: &str =
    "official.shader:provider-sample=vulkan-cuda-reduced-output-fan-out-u32";
const CUDA_ASSET_ID: &str = "kernel.cuda.copy-u32.ptx";
const CUDA_ENTRY: &str = "nuis_kernel_copy_u32";
const PRODUCER_ID: &str = "shader.vulkan-cuda-fan-out.add-xor-pair-u32";
const CUDA_CONSUMER_ID: &str = "kernel.cuda.fan-out.copy-sum-u32";
const VULKAN_CONSUMER_ID: &str = "shader.vulkan-cuda-fan-out.xor-reduced-u32";
const SUM_TRANSPORT_TOKEN: &str = "glm:provider-edge:shader.vulkan-cuda-fan-out.add-xor-pair-u32:output.values->kernel.cuda.fan-out.copy-sum-u32:input.values";
const XOR_TRANSPORT_TOKEN: &str = "glm:provider-edge:shader.vulkan-cuda-fan-out.add-xor-pair-u32:output.xor->shader.vulkan-cuda-fan-out.xor-reduced-u32:input.values";

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
    let producer = shader_asset(
        vulkan_fan_out::REDUCED_ASSET_ID,
        vulkan_fan_out::REDUCED_ENTRY,
    );
    let cuda = kernel_asset();
    let vulkan = shader_asset(vulkan_fan_out::XOR_ASSET_ID, vulkan_fan_out::XOR_ENTRY);
    let identity = code_asset_identity(&producer, &cuda, &vulkan)
        .expect("cross-provider fan-out identity must assemble");
    format!(
        "provider_request_collection_contract=nuis-provider-request-collection-v1;provider_request_count=3;{};{};{};{}",
        vulkan_fan_out::render_sample_evidence(vulkan_fan_out::VulkanFanOutSampleEvidence {
            prefix: "provider_request_0_",
            asset_id: vulkan_fan_out::REDUCED_ASSET_ID,
            entry: vulkan_fan_out::REDUCED_ENTRY,
            kernel_id: PRODUCER_ID,
            operation: "add-xor-pair-reduced-u32",
            xor_file: vulkan_fan_out::REDUCED_XOR_FILE,
            xor_expected: vulkan_fan_out::REDUCED_XOR_EXPECTED,
            xor_shape: "2x1",
            xor_row_stride_bytes: 8,
        }),
        render_consumer(Consumer {
            request_index: 1,
            provider_family: "cuda:nvidia-gpu",
            asset: &cuda,
            kernel_id: CUDA_CONSUMER_ID,
            operation: "copy-u32",
            producer_output_buffer: "output.values",
            shape: "2x2",
            input: SUM_EXPECTED,
            expected_file_name: vulkan_fan_out::SUM_FILE,
            expected: SUM_EXPECTED,
            dispatch: "4x1x1",
            element_count: 4,
            ownership_token: SUM_TRANSPORT_TOKEN,
            additional_scalars: "device_selection_policy:u32:1,minimum_compute_capability:u32:80",
        }),
        render_consumer(Consumer {
            request_index: 2,
            provider_family: "spirv:vulkan-gpu",
            asset: &vulkan,
            kernel_id: VULKAN_CONSUMER_ID,
            operation: "xor-u32",
            producer_output_buffer: "output.xor",
            shape: "2x1",
            input: vulkan_fan_out::REDUCED_XOR_EXPECTED,
            expected_file_name: vulkan_fan_out::REDUCED_ZERO_FILE,
            expected: vulkan_fan_out::REDUCED_ZERO_EXPECTED,
            dispatch: "2x1x1",
            element_count: 2,
            ownership_token: XOR_TRANSPORT_TOKEN,
            additional_scalars: "",
        }),
        identity.identity_evidence,
    )
}

struct Consumer<'a> {
    request_index: usize,
    provider_family: &'a str,
    asset: &'a nuisc::registry::NustarCodeAssetRegistration,
    kernel_id: &'a str,
    operation: &'a str,
    producer_output_buffer: &'a str,
    shape: &'a str,
    input: &'a [u8],
    expected_file_name: &'a str,
    expected: &'a [u8],
    dispatch: &'a str,
    element_count: usize,
    ownership_token: &'a str,
    additional_scalars: &'a str,
}

fn render_consumer(args: Consumer<'_>) -> String {
    let prefix = format!("provider_request_{}_", args.request_index);
    let input_hash = fnv1a64_hex(args.input);
    render_u32_prefixed_request_evidence_with_scalars(
        U32RequestEvidence {
            prefix: &prefix,
            provider_family: args.provider_family,
            kernel_id: args.kernel_id,
            operation: args.operation,
            kernel_input_buffers: "input.values",
            buffer_layout: "tensor-row-major",
            buffer_shape: args.shape,
            row_stride_bytes: 8,
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
                    row_stride_bytes: 8,
                },
                &input_hash,
                args.input.len(),
                PRODUCER_ID,
                args.producer_output_buffer,
            ),
            dependency: render_u32_dependency_edge(
                &prefix,
                0,
                args.request_index,
                PRODUCER_ID,
                args.producer_output_buffer,
                "input.values",
                args.ownership_token,
            ),
            output_evidence: None,
        },
        args.additional_scalars,
    )
}

fn code_asset_identity(
    producer: &nuisc::registry::NustarCodeAssetRegistration,
    cuda: &nuisc::registry::NustarCodeAssetRegistration,
    vulkan: &nuisc::registry::NustarCodeAssetRegistration,
) -> Result<crate::artifact_code_asset_identity::AssembledCodeAssetIdentity, String> {
    crate::artifact_code_asset_identity::assemble_nustar_code_asset_identity(
        Path::new("nustar-packages"),
        &[
            contribution(0, producer, "spirv:vulkan-gpu"),
            contribution(1, cuda, "cuda:nvidia-gpu"),
            contribution(2, vulkan, "spirv:vulkan-gpu"),
        ],
    )
}

fn contribution<'a>(
    request_index: usize,
    asset: &'a nuisc::registry::NustarCodeAssetRegistration,
    provider_family: &'a str,
) -> crate::artifact_code_asset_identity::NustarCodeAssetContribution<'a> {
    crate::artifact_code_asset_identity::NustarCodeAssetContribution {
        request_index,
        owner_package_id: &asset.package_id,
        provider_family,
        asset_id: &asset.asset_id,
        format: &asset.format,
        target: &asset.target,
        entry: &asset.entry,
        path: &asset.file_name,
        bytes: &asset.bytes,
    }
}

fn persist_payloads(output_dir: &Path, evidence: &[&str]) -> Result<(), String> {
    if !evidence.iter().any(|item| {
        item.split(';')
            .any(|field| field == format!("provider_sample_registration_id={REGISTRATION_ID}"))
    }) {
        return Ok(());
    }
    for asset in [
        shader_asset(
            vulkan_fan_out::REDUCED_ASSET_ID,
            vulkan_fan_out::REDUCED_ENTRY,
        ),
        kernel_asset(),
        shader_asset(vulkan_fan_out::XOR_ASSET_ID, vulkan_fan_out::XOR_ENTRY),
    ] {
        let actual = fs::read(output_dir.join(&asset.file_name))
            .map_err(|error| format!("failed to read cross-provider code asset: {error}"))?;
        validate_asset(&asset, &actual)?;
    }
    for (name, bytes) in [
        (vulkan_fan_out::INPUT_FILE, INPUT),
        (vulkan_fan_out::RIGHT_FILE, RIGHT_INPUT),
        (vulkan_fan_out::SUM_FILE, SUM_EXPECTED),
        (
            vulkan_fan_out::REDUCED_XOR_FILE,
            vulkan_fan_out::REDUCED_XOR_EXPECTED,
        ),
        (
            vulkan_fan_out::REDUCED_ZERO_FILE,
            vulkan_fan_out::REDUCED_ZERO_EXPECTED,
        ),
    ] {
        fs::write(output_dir.join(name), bytes)
            .map_err(|error| format!("failed to persist cross-provider payload: {error}"))?;
    }
    Ok(())
}

fn resolve_code_asset_evidence(output_dir: &Path, evidence: &str) -> Result<String, String> {
    let assets = [
        (
            shader_asset(
                vulkan_fan_out::REDUCED_ASSET_ID,
                vulkan_fan_out::REDUCED_ENTRY,
            ),
            "official.shader",
            "shader",
            "vulkan.discrete-or-integrated-gpu",
        ),
        (
            kernel_asset(),
            "official.kernel",
            "kernel",
            "cuda.nvidia-gpu",
        ),
        (
            shader_asset(vulkan_fan_out::XOR_ASSET_ID, vulkan_fan_out::XOR_ENTRY),
            "official.shader",
            "shader",
            "vulkan.discrete-or-integrated-gpu",
        ),
    ];
    let mut selections = Vec::new();
    let mut replacements = Vec::new();
    for (index, (asset, owner, domain, lowering_target)) in assets.into_iter().enumerate() {
        let prefix = format!("provider_request_{index}_");
        let actual = fs::read(output_dir.join(&asset.file_name))
            .map_err(|error| format!("failed to read cross-provider code asset: {error}"))?;
        validate_asset(&asset, &actual)?;
        validate_code_asset_request_evidence(
            "Vulkan/CUDA fan-out graph",
            &asset,
            &actual,
            evidence,
            &prefix,
        )?;
        let selection =
            crate::artifact_code_asset_contribution_table::select_compiled_code_asset_contribution(
                output_dir,
                owner,
                domain,
                lowering_target,
                &asset.format,
                &asset.target,
                std::slice::from_ref(&asset.entry),
            )?
            .ok_or_else(|| {
                format!(
                    "compiled contribution for `{}` is unavailable",
                    asset.asset_id
                )
            })?;
        validate_code_asset_contribution_selection(
            "Vulkan/CUDA fan-out graph",
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

fn validate_asset(
    asset: &nuisc::registry::NustarCodeAssetRegistration,
    actual: &[u8],
) -> Result<(), String> {
    if actual != asset.bytes
        || !actual
            .windows(asset.entry.len())
            .any(|window| window == asset.entry.as_bytes())
    {
        return Err(format!(
            "cross-provider code asset `{}` does not match registry ownership",
            asset.asset_id
        ));
    }
    Ok(())
}

fn shader_asset(asset_id: &str, entry: &str) -> nuisc::registry::NustarCodeAssetRegistration {
    vulkan_fan_out::asset(asset_id, entry)
        .expect("cross-provider Shader Nustar asset must be registered")
}

fn kernel_asset() -> nuisc::registry::NustarCodeAssetRegistration {
    let root = Path::new("nustar-packages");
    let manifest = nuisc::registry::load_manifest_for_domain(root, "kernel")
        .expect("Kernel Nustar manifest must load");
    nuisc::registry::code_asset_registration_by_id(root, &manifest, CUDA_ASSET_ID)
        .expect("Kernel Nustar code asset registry must load")
        .filter(|asset| {
            asset.package_id == "official.kernel"
                && asset.lowering_target == "cuda.nvidia-gpu"
                && asset.entry == CUDA_ENTRY
        })
        .expect("Kernel Nustar CUDA u32 copy asset must be registered")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registration_owns_open_vulkan_cuda_fan_out_graph() {
        let evidence = (registration().enrich_evidence)("ignored");

        assert!(evidence.contains("provider_request_count=3"));
        assert!(
            evidence.contains("provider_request_1_adapter_binding_provider_family=cuda:nvidia-gpu")
        );
        assert!(evidence.contains("provider_request_1_code_asset_id=kernel.cuda.copy-u32.ptx"));
        assert!(evidence.contains("provider_request_1_kernel_operation=copy-u32"));
        assert!(evidence.contains("provider_request_1_input_binding_0_shape=2x2"));
        assert!(evidence.contains("provider_request_1_input_binding_0_byte_length=16"));
        assert!(evidence
            .contains("provider_request_2_adapter_binding_provider_family=spirv:vulkan-gpu"));
        assert!(evidence.contains("provider_request_2_input_binding_0_shape=2x1"));
        assert!(evidence.contains("provider_request_2_input_binding_0_byte_length=8"));
        assert!(evidence.contains("provider_code_asset_identity_set_count=3"));
        assert!(nsdb::validate_provider_request_evidence(&evidence));
    }
}
