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
const PADDED_REGISTRATION_ID: &str = "official.shader.vulkan-add-xor-pair-padded-u32";
const PADDED_METADATA_SELECTOR: &str =
    "official.shader:provider-sample=vulkan-add-xor-pair-padded-u32";
const REDUCED_REGISTRATION_ID: &str = "official.shader.vulkan-add-xor-pair-reduced-u32";
const REDUCED_METADATA_SELECTOR: &str =
    "official.shader:provider-sample=vulkan-add-xor-pair-reduced-u32";
const ASSET_ID: &str = "shader.vulkan.add-xor-pair-u32.spirv";
const ENTRY: &str = "nuis_vulkan_add_xor_pair_u32";
const REDUCED_ASSET_ID: &str = "shader.vulkan.add-xor-pair-reduced-u32.spirv";
const REDUCED_ENTRY: &str = "nuis_vulkan_add_xor_pair_reduced_u32";
const LOWERING_TARGET: &str = "vulkan.discrete-or-integrated-gpu";
const INPUT_FILE: &str = "nuis.shader.vulkan.add-xor-pair-u32.left.u32.bin";
const RIGHT_FILE: &str = "nuis.shader.vulkan.add-xor-pair-u32.right.u32.bin";
const SUM_FILE: &str = "nuis.shader.vulkan.add-xor-pair-u32.sum.expected.u32.bin";
const XOR_FILE: &str = "nuis.shader.vulkan.add-xor-pair-u32.xor.expected.u32.bin";
const PADDED_XOR_FILE: &str = "nuis.shader.vulkan.add-xor-pair-u32.xor-padded.expected.u32.bin";
const REDUCED_XOR_FILE: &str = "nuis.shader.vulkan.add-xor-pair-reduced-u32.xor.expected.u32.bin";
const XOR_EXPECTED: &[u8] = &[3, 0, 0, 0, 11, 0, 0, 0, 8, 0, 0, 0, 29, 0, 0, 0];
const REDUCED_XOR_EXPECTED: &[u8] = &[3, 0, 0, 0, 11, 0, 0, 0];
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

fn selects_sample(base: &str) -> bool {
    selects_metadata(base, METADATA_SELECTOR)
}

fn selects_padded_sample(base: &str) -> bool {
    selects_metadata(base, PADDED_METADATA_SELECTOR)
}

fn selects_reduced_sample(base: &str) -> bool {
    selects_metadata(base, REDUCED_METADATA_SELECTOR)
}

fn selects_metadata(base: &str, selector: &str) -> bool {
    base.split(';')
        .filter_map(|field| field.split_once('='))
        .any(|(key, value)| key.starts_with("artifact_provider_metadata_") && value == selector)
}

fn sample_evidence(_base: &str) -> String {
    render_sample_evidence(
        ASSET_ID,
        ENTRY,
        "shader.vulkan.add-xor-pair-u32",
        "add-xor-pair-u32",
        XOR_FILE,
        XOR_EXPECTED,
        "2x2",
        8,
    )
}

fn padded_sample_evidence(_base: &str) -> String {
    render_sample_evidence(
        ASSET_ID,
        ENTRY,
        "shader.vulkan.add-xor-pair-u32",
        "add-xor-pair-u32",
        PADDED_XOR_FILE,
        PADDED_XOR_EXPECTED,
        "2x2",
        12,
    )
}

fn reduced_sample_evidence(_base: &str) -> String {
    render_sample_evidence(
        REDUCED_ASSET_ID,
        REDUCED_ENTRY,
        "shader.vulkan.add-xor-pair-reduced-u32",
        "add-xor-pair-reduced-u32",
        REDUCED_XOR_FILE,
        REDUCED_XOR_EXPECTED,
        "2x1",
        8,
    )
}

fn render_sample_evidence(
    asset_id: &'static str,
    entry: &'static str,
    kernel_id: &'static str,
    operation: &'static str,
    xor_file: &'static str,
    xor_expected: &'static [u8],
    xor_shape: &'static str,
    xor_row_stride_bytes: usize,
) -> String {
    let asset =
        asset(asset_id, entry).expect("Shader Nustar Vulkan fan-out asset must be registered");
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
                shape: xor_shape,
                row_stride_bytes: xor_row_stride_bytes,
                comparison_id: "comparison.output.xor",
                expected_file_name: xor_file,
                expected: xor_expected,
            },
        ],
    );
    render_u32_prefixed_request_evidence(U32RequestEvidence {
        prefix: "provider_",
        provider_family: "spirv:vulkan-gpu",
        kernel_id,
        operation,
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
    let owns = |registration_id: &str| {
        evidence.iter().any(|item| {
            item.split(';')
                .any(|field| field == format!("provider_sample_registration_id={registration_id}"))
        })
    };
    let standard = owns(REGISTRATION_ID);
    let padded = owns(PADDED_REGISTRATION_ID);
    let reduced = owns(REDUCED_REGISTRATION_ID);
    if !standard && !padded && !reduced {
        return Ok(());
    }
    for (enabled, asset_id, entry) in [
        (standard || padded, ASSET_ID, ENTRY),
        (reduced, REDUCED_ASSET_ID, REDUCED_ENTRY),
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
        (reduced, REDUCED_XOR_FILE, REDUCED_XOR_EXPECTED),
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

fn validate_asset(
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

fn asset(
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
    }
}
