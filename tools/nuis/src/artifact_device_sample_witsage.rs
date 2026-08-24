use std::{collections::BTreeMap, fs, path::Path};

const COMPARISON_PROFILE_CONTRACT: &str = "nuis-witsage-output-comparison-profile-v1";
const CROSS_PROVIDER_COMPARISON_PROFILE_SOURCE: &str =
    include_str!("../../../stdlib/witsage/provider-comparison-profiles/cross-provider-f32.nwcp");
const MODEL_PREDICT_COMPARISON_PROFILE_SOURCE: &str =
    include_str!("../../../stdlib/witsage/provider-comparison-profiles/model-predict-f32.nwcp");
const WITSAGE_VECTOR_PAYLOAD_FILE_NAME: &str = "nuis.witsage.vector.f32.bin";
const WITSAGE_VECTOR_MODEL_FILE_NAME: &str = "nuis.witsage.vector-affine.mlmodel";
const WITSAGE_VECTOR_EXPECTED_FILE_NAME: &str = "nuis.witsage.vector-affine.expected.f32.bin";
const WITSAGE_CHAINED_EXPECTED_FILE_NAME: &str =
    "nuis.witsage.vector-affine-chained.expected.f32.bin";
const WITSAGE_DENSE_PAYLOAD_FILE_NAME: &str = "nuis.witsage.feature-grid.f32.bin";
const WITSAGE_DENSE_MODEL_FILE_NAME: &str = "nuis.witsage.feature-grid-projection.mlmodel";
const WITSAGE_ADD_MODEL_FILE_NAME: &str = "nuis.witsage.vector-add.mlmodel";
const WITSAGE_ADD_EXPECTED_FILE_NAME: &str = "nuis.witsage.vector-add.expected.f32.bin";
const WITSAGE_METAL_EXPECTED_FILE_NAME: &str = "nuis.witsage.vector-metal-bias.expected.f32.bin";
const WITSAGE_METAL_BIAS_ASSET_ID: &str = "shader.witsage.vector-bias.metal";
const WITSAGE_METAL_ARGMAX_ASSET_ID: &str = "shader.witsage.argmax.metal";
const WITSAGE_KMEANS_MODEL_FILE_NAME: &str = "nuis.witsage.kmeans-centroid-score.mlmodel";
const WITSAGE_KMEANS_EXPECTED_FILE_NAME: &str =
    "nuis.witsage.kmeans-centroid-score.expected.f32.bin";
const WITSAGE_KMEANS_ASSIGNMENT_EXPECTED_FILE_NAME: &str =
    "nuis.witsage.kmeans-assignment.expected.u32.bin";
const WITSAGE_VECTOR_PAYLOAD: &[u8] = &[
    0x00, 0x00, 0x80, 0x3f, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x40, 0x40, 0x00, 0x00, 0x80, 0x40,
];
const WITSAGE_VECTOR_EXPECTED: &[u8] = &[
    0x00, 0x00, 0x40, 0x40, 0x00, 0x00, 0xa0, 0x40, 0x00, 0x00, 0xe0, 0x40, 0x00, 0x00, 0x10, 0x41,
];
const WITSAGE_CHAINED_EXPECTED: &[u8] = &[
    0x00, 0x00, 0xe0, 0x40, 0x00, 0x00, 0x30, 0x41, 0x00, 0x00, 0x70, 0x41, 0x00, 0x00, 0x98, 0x41,
];
const WITSAGE_ADD_EXPECTED: &[u8] = &[
    0x00, 0x00, 0x20, 0x41, 0x00, 0x00, 0x80, 0x41, 0x00, 0x00, 0xb0, 0x41, 0x00, 0x00, 0xe0, 0x41,
];
const WITSAGE_METAL_EXPECTED: &[u8] = &[
    0x00, 0x00, 0x30, 0x41, 0x00, 0x00, 0x88, 0x41, 0x00, 0x00, 0xb8, 0x41, 0x00, 0x00, 0xe8, 0x41,
];
const WITSAGE_KMEANS_EXPECTED: &[u8] = &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x41];
const WITSAGE_KMEANS_ASSIGNMENT_EXPECTED: &[u8] = &[0x01, 0x00, 0x00, 0x00];
struct ComparisonProfile {
    package_id: String,
    profile_id: String,
    absolute_tolerance: String,
    relative_tolerance: String,
    non_finite_policy: String,
    source_hash: String,
}

pub(super) fn input_evidence(base: &str) -> String {
    let cross_provider_profile = load_comparison_profile(
        CROSS_PROVIDER_COMPARISON_PROFILE_SOURCE,
        "witsage.cross-provider.f32",
    )
    .expect("checked-in WitSage cross-provider profile must remain valid");
    let model_predict_profile = load_comparison_profile(
        MODEL_PREDICT_COMPARISON_PROFILE_SOURCE,
        "witsage.model-predict.f32",
    )
    .expect("checked-in WitSage model-predict profile must remain valid");
    let payload = dense_payload();
    let model = crate::artifact_coreml_model::witsage_dense_transform_model();
    let singular = format!(
        "{base};provider_buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1;provider_buffer_id=input.features;provider_buffer_element_type=f32;provider_buffer_layout=tensor-contiguous;provider_buffer_shape=16x64x64;provider_buffer_row_stride_bytes={};provider_buffer_byte_length={};provider_buffer_payload_path={WITSAGE_DENSE_PAYLOAD_FILE_NAME};provider_buffer_content_hash={};provider_kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1;provider_kernel_id=witsage.feature-grid.projection;provider_kernel_operation=model-predict;provider_kernel_input_buffer=input.features;provider_kernel_output_buffer=output.features;provider_kernel_dispatch=16x64x64;provider_model_asset_descriptor_contract=nuis-provider-model-asset-descriptor-v1;provider_model_asset_id=witsage.feature-grid-projection.coreml;provider_model_asset_format=coreml-specification;provider_model_asset_path={WITSAGE_DENSE_MODEL_FILE_NAME};provider_model_asset_byte_length={};provider_model_asset_content_hash={};provider_model_asset_input_feature=input.features;provider_model_asset_output_feature=output.features;provider_output_comparison_descriptor_contract=nuis-provider-output-comparison-descriptor-v1;provider_output_comparison_output_buffer=output.features;provider_output_comparison_element_type=f32;provider_output_comparison_shape=16x64x64;provider_output_comparison_expected_path={WITSAGE_DENSE_PAYLOAD_FILE_NAME};provider_output_comparison_expected_byte_length={};provider_output_comparison_expected_content_hash={};provider_output_comparison_absolute_tolerance=0;provider_output_comparison_relative_tolerance=0;provider_output_comparison_non_finite_policy=reject",
        payload.len(),
        payload.len(),
        fnv1a64_hex(&payload),
        model.len(),
        fnv1a64_hex(&model),
        payload.len(),
        fnv1a64_hex(&payload)
    );
    let dense = dense_collection_request(0, &payload, &model);
    let affine_model = crate::artifact_coreml_model::witsage_vector_affine_model();
    let affine = affine_collection_request(1, &affine_model);
    let chained = chained_affine_collection_request(2, &affine_model);
    let add_model = crate::artifact_coreml_model::witsage_vector_add_model();
    let add = add_collection_request(3, &add_model);
    let code_assets = witsage_code_asset_identity()
        .expect("registered Shader Nustar code asset assembly must remain valid");
    let metal = metal_bias_collection_request(
        4,
        &cross_provider_profile,
        code_assets
            .descriptor_for(4)
            .expect("Metal bias descriptor"),
    );
    let kmeans_model = crate::artifact_coreml_model::witsage_kmeans_centroid_score_model();
    let kmeans = kmeans_collection_request(5, &kmeans_model, &model_predict_profile);
    let assignment = kmeans_assignment_collection_request(
        6,
        code_assets
            .descriptor_for(6)
            .expect("Metal argmax descriptor"),
    );
    format!(
        "{singular};provider_request_collection_contract=nuis-provider-request-collection-v1;provider_request_count=7;{dense};{affine};{chained};{add};{metal};{kmeans};{assignment};{}",
        code_assets.identity_evidence
    )
}

fn witsage_code_asset_identity(
) -> Result<crate::artifact_code_asset_identity::AssembledCodeAssetIdentity, String> {
    use crate::artifact_code_asset_identity::NustarCodeAssetContribution;
    let assets = witsage_shader_code_assets()?;
    let bias = registered_asset(&assets, WITSAGE_METAL_BIAS_ASSET_ID)?;
    let argmax = registered_asset(&assets, WITSAGE_METAL_ARGMAX_ASSET_ID)?;
    crate::artifact_code_asset_identity::assemble_nustar_code_asset_identity(
        Path::new("nustar-packages"),
        &[
            NustarCodeAssetContribution {
                request_index: 4,
                owner_package_id: &bias.package_id,
                provider_family: "metal:apple-silicon-gpu",
                asset_id: &bias.asset_id,
                format: &bias.format,
                target: &bias.target,
                entry: &bias.entry,
                path: &bias.file_name,
                bytes: &bias.bytes,
            },
            NustarCodeAssetContribution {
                request_index: 6,
                owner_package_id: &argmax.package_id,
                provider_family: "metal:apple-silicon-gpu",
                asset_id: &argmax.asset_id,
                format: &argmax.format,
                target: &argmax.target,
                entry: &argmax.entry,
                path: &argmax.file_name,
                bytes: &argmax.bytes,
            },
        ],
    )
}

fn witsage_shader_code_assets() -> Result<Vec<nuisc::registry::NustarCodeAssetRegistration>, String>
{
    let root = Path::new("nustar-packages");
    let manifest = nuisc::registry::load_manifest_for_domain(root, "shader")?;
    [WITSAGE_METAL_BIAS_ASSET_ID, WITSAGE_METAL_ARGMAX_ASSET_ID]
        .into_iter()
        .map(|id| {
            nuisc::registry::code_asset_registration_by_id(root, &manifest, id)?
                .ok_or_else(|| format!("Shader Nustar code asset `{id}` is not registered"))
        })
        .collect()
}

fn registered_asset<'a>(
    assets: &'a [nuisc::registry::NustarCodeAssetRegistration],
    id: &str,
) -> Result<&'a nuisc::registry::NustarCodeAssetRegistration, String> {
    assets
        .iter()
        .find(|asset| asset.asset_id == id)
        .ok_or_else(|| format!("Shader Nustar code asset `{id}` is not registered"))
}

pub(super) fn resolve_code_asset_evidence(
    output_dir: &Path,
    evidence: &str,
) -> Result<String, String> {
    let assets = witsage_shader_code_assets()?;
    let selected_assets = assets
        .iter()
        .filter(|asset| evidence.contains(&format!("_code_asset_id={}", asset.asset_id)))
        .collect::<Vec<_>>();
    if selected_assets.is_empty() {
        return Ok(evidence.to_owned());
    }
    let mut selections = Vec::with_capacity(selected_assets.len());
    for asset in selected_assets {
        let selection =
            crate::artifact_code_asset_contribution_table::select_compiled_code_asset_contribution(
                output_dir,
                &asset.package_id,
                &asset.domain_family,
                &asset.lowering_target,
                &asset.format,
                &asset.target,
                std::slice::from_ref(&asset.entry),
            )?
            .ok_or_else(|| {
                format!(
                    "compiled Shader contribution for `{}` is unavailable",
                    asset.asset_id
                )
            })?;
        selections.push(selection);
    }
    selections.sort_by_key(|selection| {
        evidence
            .find(&format!("_code_asset_id={}", selection.asset_id))
            .unwrap_or(usize::MAX)
    });
    let set =
        crate::artifact_code_asset_contribution_table::render_selected_contribution_set_evidence(
            &selections,
        )?;
    Ok(format!("{evidence};{set}"))
}

pub(super) fn persist_assets_if_requested(
    output_dir: &Path,
    evidence: &[&str],
) -> Result<(), String> {
    if !evidence
        .iter()
        .any(|item| item.contains("provider_model_asset_id=witsage."))
    {
        return Ok(());
    }
    for (name, bytes, detail) in [
        (
            WITSAGE_VECTOR_PAYLOAD_FILE_NAME,
            WITSAGE_VECTOR_PAYLOAD.to_vec(),
            "vector payload",
        ),
        (
            WITSAGE_VECTOR_MODEL_FILE_NAME,
            crate::artifact_coreml_model::witsage_vector_affine_model(),
            "CoreML model",
        ),
        (
            WITSAGE_VECTOR_EXPECTED_FILE_NAME,
            WITSAGE_VECTOR_EXPECTED.to_vec(),
            "expected vector output",
        ),
        (
            WITSAGE_CHAINED_EXPECTED_FILE_NAME,
            WITSAGE_CHAINED_EXPECTED.to_vec(),
            "chained expected output",
        ),
        (
            WITSAGE_DENSE_PAYLOAD_FILE_NAME,
            dense_payload(),
            "dense payload",
        ),
        (
            WITSAGE_DENSE_MODEL_FILE_NAME,
            crate::artifact_coreml_model::witsage_dense_transform_model(),
            "dense CoreML model",
        ),
        (
            WITSAGE_ADD_MODEL_FILE_NAME,
            crate::artifact_coreml_model::witsage_vector_add_model(),
            "add CoreML model",
        ),
        (
            WITSAGE_ADD_EXPECTED_FILE_NAME,
            WITSAGE_ADD_EXPECTED.to_vec(),
            "add expected output",
        ),
        (
            WITSAGE_METAL_EXPECTED_FILE_NAME,
            WITSAGE_METAL_EXPECTED.to_vec(),
            "Metal expected output",
        ),
        (
            WITSAGE_KMEANS_MODEL_FILE_NAME,
            crate::artifact_coreml_model::witsage_kmeans_centroid_score_model(),
            "KMeans centroid-score CoreML model",
        ),
        (
            WITSAGE_KMEANS_EXPECTED_FILE_NAME,
            WITSAGE_KMEANS_EXPECTED.to_vec(),
            "KMeans centroid-score expected output",
        ),
        (
            WITSAGE_KMEANS_ASSIGNMENT_EXPECTED_FILE_NAME,
            WITSAGE_KMEANS_ASSIGNMENT_EXPECTED.to_vec(),
            "KMeans assignment expected output",
        ),
    ] {
        fs::write(output_dir.join(name), bytes)
            .map_err(|error| format!("failed to persist WitSage {detail}: {error}"))?;
    }
    for asset in witsage_shader_code_assets()? {
        fs::write(output_dir.join(&asset.file_name), &asset.bytes).map_err(|error| {
            format!(
                "failed to persist registered Shader code asset `{}`: {error}",
                asset.asset_id
            )
        })?;
    }
    Ok(())
}

fn load_comparison_profile(
    source: &str,
    expected_profile_id: &str,
) -> Result<ComparisonProfile, String> {
    let mut fields = BTreeMap::new();
    for (line_index, line) in source.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, value) = line.split_once('=').ok_or_else(|| {
            format!(
                "WitSage comparison profile line {} is malformed",
                line_index + 1
            )
        })?;
        if !matches!(
            key,
            "protocol"
                | "package_id"
                | "profile_id"
                | "absolute_tolerance"
                | "relative_tolerance"
                | "non_finite_policy"
        ) {
            return Err(format!(
                "WitSage comparison profile field `{key}` is unsupported"
            ));
        }
        if fields.insert(key, value).is_some() {
            return Err(format!(
                "WitSage comparison profile field `{key}` is duplicated"
            ));
        }
    }
    if fields.get("protocol") != Some(&COMPARISON_PROFILE_CONTRACT)
        || fields.get("package_id") != Some(&"nuis.witsage")
    {
        return Err("WitSage comparison profile identity is invalid".to_owned());
    }
    let profile_id = required(&fields, "profile_id")?;
    if profile_id != expected_profile_id || !profile_id.starts_with("witsage.") {
        return Err("WitSage comparison profile id is invalid".to_owned());
    }
    let absolute_tolerance = valid_non_zero_tolerance(&fields, "absolute_tolerance")?;
    let relative_tolerance = valid_non_zero_tolerance(&fields, "relative_tolerance")?;
    let non_finite_policy = required(&fields, "non_finite_policy")?;
    if non_finite_policy != "reject" {
        return Err("WitSage comparison profile must reject non-finite values".to_owned());
    }
    Ok(ComparisonProfile {
        package_id: "nuis.witsage".to_owned(),
        profile_id,
        absolute_tolerance,
        relative_tolerance,
        non_finite_policy,
        source_hash: fnv1a64_hex(source.as_bytes()),
    })
}

fn required(fields: &BTreeMap<&str, &str>, key: &str) -> Result<String, String> {
    fields
        .get(key)
        .filter(|value| !value.is_empty())
        .map(|value| (*value).to_owned())
        .ok_or_else(|| format!("WitSage comparison profile field `{key}` is missing"))
}

fn valid_non_zero_tolerance(fields: &BTreeMap<&str, &str>, key: &str) -> Result<String, String> {
    let value = required(fields, key)?;
    value
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite() && *value > 0.0)
        .map(|_| value)
        .ok_or_else(|| format!("WitSage comparison profile field `{key}` is invalid"))
}

fn dense_payload() -> Vec<u8> {
    vec![1.0f32; 16 * 64 * 64]
        .into_iter()
        .flat_map(f32::to_le_bytes)
        .collect()
}

fn dense_collection_request(index: usize, payload: &[u8], model: &[u8]) -> String {
    let prefix = format!("provider_request_{index}_");
    let request = format!(
        "{prefix}buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1;{prefix}buffer_id=input.features;{prefix}buffer_element_type=f32;{prefix}buffer_layout=tensor-contiguous;{prefix}buffer_shape=16x64x64;{prefix}buffer_row_stride_bytes={};{prefix}buffer_byte_length={};{prefix}buffer_payload_path={WITSAGE_DENSE_PAYLOAD_FILE_NAME};{prefix}buffer_content_hash={};{prefix}kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1;{prefix}kernel_id=witsage.feature-grid.projection;{prefix}kernel_operation=model-predict;{prefix}kernel_input_buffer=input.features;{prefix}kernel_output_buffer=output.features;{prefix}kernel_dispatch=16x64x64;{prefix}model_asset_descriptor_contract=nuis-provider-model-asset-descriptor-v1;{prefix}model_asset_id=witsage.feature-grid-projection.coreml;{prefix}model_asset_format=coreml-specification;{prefix}model_asset_path={WITSAGE_DENSE_MODEL_FILE_NAME};{prefix}model_asset_byte_length={};{prefix}model_asset_content_hash={};{prefix}model_asset_input_feature=input.features;{prefix}model_asset_output_feature=output.features;{prefix}output_comparison_descriptor_contract=nuis-provider-output-comparison-descriptor-v1;{prefix}output_comparison_output_buffer=output.features;{prefix}output_comparison_element_type=f32;{prefix}output_comparison_shape=16x64x64;{prefix}output_comparison_expected_path={WITSAGE_DENSE_PAYLOAD_FILE_NAME};{prefix}output_comparison_expected_byte_length={};{prefix}output_comparison_expected_content_hash={};{prefix}output_comparison_absolute_tolerance=0;{prefix}output_comparison_relative_tolerance=0;{prefix}output_comparison_non_finite_policy=reject;{prefix}dependency_contract=nuis-provider-request-dependency-v1;{prefix}dependency_count=0",
        payload.len(),
        payload.len(),
        fnv1a64_hex(payload),
        model.len(),
        fnv1a64_hex(model),
        payload.len(),
        fnv1a64_hex(payload)
    );
    format!(
        "{request};{}",
        input_binding(
            &prefix,
            "artifact",
            "16x64x64",
            payload.len(),
            &fnv1a64_hex(payload),
            WITSAGE_DENSE_PAYLOAD_FILE_NAME,
            "none",
            "none",
        )
    )
}

fn affine_collection_request(index: usize, model: &[u8]) -> String {
    let prefix = format!("provider_request_{index}_");
    let request = format!(
        "{prefix}buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1;{prefix}buffer_id=input.features;{prefix}buffer_element_type=f32;{prefix}buffer_layout=tensor-contiguous;{prefix}buffer_shape=1x1x4;{prefix}buffer_row_stride_bytes=16;{prefix}buffer_byte_length={};{prefix}buffer_payload_path={WITSAGE_VECTOR_PAYLOAD_FILE_NAME};{prefix}buffer_content_hash={};{prefix}kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1;{prefix}kernel_id=witsage.vector.affine;{prefix}kernel_operation=affine;{prefix}kernel_input_buffer=input.features;{prefix}kernel_output_buffer=output.features;{prefix}kernel_dispatch=1x1x4;{prefix}kernel_scalar_bindings=scale:f32:2,bias:f32:1;{prefix}model_asset_descriptor_contract=nuis-provider-model-asset-descriptor-v1;{prefix}model_asset_id=witsage.vector-affine.coreml;{prefix}model_asset_format=coreml-specification;{prefix}model_asset_path={WITSAGE_VECTOR_MODEL_FILE_NAME};{prefix}model_asset_byte_length={};{prefix}model_asset_content_hash={};{prefix}model_asset_input_feature=input.features;{prefix}model_asset_output_feature=output.features;{prefix}output_comparison_descriptor_contract=nuis-provider-output-comparison-descriptor-v1;{prefix}output_comparison_output_buffer=output.features;{prefix}output_comparison_element_type=f32;{prefix}output_comparison_shape=1x1x4;{prefix}output_comparison_expected_path={WITSAGE_VECTOR_EXPECTED_FILE_NAME};{prefix}output_comparison_expected_byte_length={};{prefix}output_comparison_expected_content_hash={};{prefix}output_comparison_absolute_tolerance=0;{prefix}output_comparison_relative_tolerance=0;{prefix}output_comparison_non_finite_policy=reject;{prefix}dependency_contract=nuis-provider-request-dependency-v1;{prefix}dependency_count=0",
        WITSAGE_VECTOR_PAYLOAD.len(),
        fnv1a64_hex(WITSAGE_VECTOR_PAYLOAD),
        model.len(),
        fnv1a64_hex(model),
        WITSAGE_VECTOR_EXPECTED.len(),
        fnv1a64_hex(WITSAGE_VECTOR_EXPECTED)
    );
    format!(
        "{request};{}",
        input_binding(
            &prefix,
            "artifact",
            "1x1x4",
            WITSAGE_VECTOR_PAYLOAD.len(),
            &fnv1a64_hex(WITSAGE_VECTOR_PAYLOAD),
            WITSAGE_VECTOR_PAYLOAD_FILE_NAME,
            "none",
            "none",
        )
    )
}

fn chained_affine_collection_request(index: usize, model: &[u8]) -> String {
    let prefix = format!("provider_request_{index}_");
    let request = format!(
        "{prefix}buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1;{prefix}buffer_id=input.features;{prefix}buffer_element_type=f32;{prefix}buffer_layout=tensor-contiguous;{prefix}buffer_shape=1x1x4;{prefix}buffer_row_stride_bytes=16;{prefix}buffer_byte_length={};{prefix}buffer_payload_path={WITSAGE_VECTOR_EXPECTED_FILE_NAME};{prefix}buffer_content_hash={};{prefix}kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1;{prefix}kernel_id=witsage.vector.affine.chained;{prefix}kernel_operation=affine;{prefix}kernel_input_buffer=input.features;{prefix}kernel_output_buffer=output.features;{prefix}kernel_dispatch=1x1x4;{prefix}kernel_scalar_bindings=scale:f32:2,bias:f32:1;{prefix}model_asset_descriptor_contract=nuis-provider-model-asset-descriptor-v1;{prefix}model_asset_id=witsage.vector-affine-chained.coreml;{prefix}model_asset_format=coreml-specification;{prefix}model_asset_path={WITSAGE_VECTOR_MODEL_FILE_NAME};{prefix}model_asset_byte_length={};{prefix}model_asset_content_hash={};{prefix}model_asset_input_feature=input.features;{prefix}model_asset_output_feature=output.features;{prefix}output_comparison_descriptor_contract=nuis-provider-output-comparison-descriptor-v1;{prefix}output_comparison_output_buffer=output.features;{prefix}output_comparison_element_type=f32;{prefix}output_comparison_shape=1x1x4;{prefix}output_comparison_expected_path={WITSAGE_CHAINED_EXPECTED_FILE_NAME};{prefix}output_comparison_expected_byte_length={};{prefix}output_comparison_expected_content_hash={};{prefix}output_comparison_absolute_tolerance=0;{prefix}output_comparison_relative_tolerance=0;{prefix}output_comparison_non_finite_policy=reject;{prefix}dependency_contract=nuis-provider-request-dependency-v1;{prefix}dependency_count=1;{prefix}dependency_0_producer_request_id=witsage.vector.affine;{prefix}dependency_0_producer_output_buffer=output.features;{prefix}dependency_0_consumer_input_buffer=input.features",
        WITSAGE_VECTOR_EXPECTED.len(),
        fnv1a64_hex(WITSAGE_VECTOR_EXPECTED),
        model.len(),
        fnv1a64_hex(model),
        WITSAGE_CHAINED_EXPECTED.len(),
        fnv1a64_hex(WITSAGE_CHAINED_EXPECTED)
    );
    let request = format!(
        "{request};{}",
        input_binding(
            &prefix,
            "dependency",
            "1x1x4",
            WITSAGE_VECTOR_EXPECTED.len(),
            &fnv1a64_hex(WITSAGE_VECTOR_EXPECTED),
            "none",
            "witsage.vector.affine",
            "output.features",
        )
    );
    with_dependency_transport(
        request,
        index,
        0,
        "input.features",
        "glm:provider-edge:witsage.vector.affine:output.features->witsage.vector.affine.chained:input.features",
        1,
    )
}

fn add_collection_request(index: usize, model: &[u8]) -> String {
    let prefix = format!("provider_request_{index}_");
    let request = format!(
        "{prefix}buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1;{prefix}buffer_id=input.left;{prefix}buffer_element_type=f32;{prefix}buffer_layout=tensor-contiguous;{prefix}buffer_shape=1x1x4;{prefix}buffer_row_stride_bytes=16;{prefix}buffer_byte_length={};{prefix}buffer_payload_path={WITSAGE_VECTOR_EXPECTED_FILE_NAME};{prefix}buffer_content_hash={};{prefix}kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1;{prefix}kernel_id=witsage.vector.add;{prefix}kernel_operation=add;{prefix}kernel_input_buffer=input.left;{prefix}kernel_input_buffers=input.left,input.right;{prefix}kernel_output_buffer=output.features;{prefix}kernel_dispatch=1x1x4;{prefix}model_asset_descriptor_contract=nuis-provider-model-asset-descriptor-v1;{prefix}model_asset_id=witsage.vector-add.coreml;{prefix}model_asset_format=coreml-specification;{prefix}model_asset_path={WITSAGE_ADD_MODEL_FILE_NAME};{prefix}model_asset_byte_length={};{prefix}model_asset_content_hash={};{prefix}model_asset_input_feature=input.left;{prefix}model_asset_input_features=input.left,input.right;{prefix}model_asset_output_feature=output.features;{prefix}output_comparison_descriptor_contract=nuis-provider-output-comparison-descriptor-v1;{prefix}output_comparison_output_buffer=output.features;{prefix}output_comparison_element_type=f32;{prefix}output_comparison_shape=1x1x4;{prefix}output_comparison_expected_path={WITSAGE_ADD_EXPECTED_FILE_NAME};{prefix}output_comparison_expected_byte_length={};{prefix}output_comparison_expected_content_hash={};{prefix}output_comparison_absolute_tolerance=0;{prefix}output_comparison_relative_tolerance=0;{prefix}output_comparison_non_finite_policy=reject;{prefix}dependency_contract=nuis-provider-request-dependency-v1;{prefix}dependency_count=2;{prefix}dependency_0_producer_request_id=witsage.vector.affine;{prefix}dependency_0_producer_output_buffer=output.features;{prefix}dependency_0_consumer_input_buffer=input.left;{prefix}dependency_1_producer_request_id=witsage.vector.affine.chained;{prefix}dependency_1_producer_output_buffer=output.features;{prefix}dependency_1_consumer_input_buffer=input.right;{prefix}input_binding_contract=nuis-provider-input-binding-v1;{prefix}input_binding_count=2;{prefix}input_binding_0_name=input.left;{prefix}input_binding_0_source=dependency;{prefix}input_binding_0_element_type=f32;{prefix}input_binding_0_shape=1x1x4;{prefix}input_binding_0_byte_length={};{prefix}input_binding_0_content_hash={};{prefix}input_binding_0_payload_path=none;{prefix}input_binding_0_producer_request_id=witsage.vector.affine;{prefix}input_binding_0_producer_output_buffer=output.features;{prefix}input_binding_1_name=input.right;{prefix}input_binding_1_source=dependency;{prefix}input_binding_1_element_type=f32;{prefix}input_binding_1_shape=1x1x4;{prefix}input_binding_1_byte_length={};{prefix}input_binding_1_content_hash={};{prefix}input_binding_1_payload_path=none;{prefix}input_binding_1_producer_request_id=witsage.vector.affine.chained;{prefix}input_binding_1_producer_output_buffer=output.features;{prefix}adapter_binding_contract=nuis-provider-request-adapter-binding-v1;{prefix}adapter_binding_provider_family=coreml:apple-ane;{prefix}adapter_binding_execution_requirement=real-device",
        WITSAGE_VECTOR_EXPECTED.len(),
        fnv1a64_hex(WITSAGE_VECTOR_EXPECTED),
        model.len(),
        fnv1a64_hex(model),
        WITSAGE_ADD_EXPECTED.len(),
        fnv1a64_hex(WITSAGE_ADD_EXPECTED),
        WITSAGE_VECTOR_EXPECTED.len(),
        fnv1a64_hex(WITSAGE_VECTOR_EXPECTED),
        WITSAGE_CHAINED_EXPECTED.len(),
        fnv1a64_hex(WITSAGE_CHAINED_EXPECTED),
    );
    let request = with_dependency_transport(
        request,
        index,
        0,
        "input.left",
        "glm:provider-edge:witsage.vector.affine:output.features->witsage.vector.add:input.left",
        1,
    );
    with_dependency_transport(
        request,
        index,
        1,
        "input.right",
        "glm:provider-edge:witsage.vector.affine.chained:output.features->witsage.vector.add:input.right",
        2,
    )
}

fn metal_bias_collection_request(
    index: usize,
    profile: &ComparisonProfile,
    code_asset: &str,
) -> String {
    let prefix = format!("provider_request_{index}_");
    format!(
        "{prefix}buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1;{prefix}buffer_id=input.features;{prefix}buffer_element_type=f32;{prefix}buffer_layout=tensor-contiguous;{prefix}buffer_shape=1x1x4;{prefix}buffer_row_stride_bytes=16;{prefix}buffer_byte_length={};{prefix}buffer_payload_path={WITSAGE_ADD_EXPECTED_FILE_NAME};{prefix}buffer_content_hash={};{prefix}kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1;{prefix}kernel_id=witsage.vector.metal-bias;{prefix}kernel_operation=bias;{prefix}kernel_input_buffer=input.features;{prefix}kernel_output_buffer=output.features;{prefix}kernel_dispatch=1x1x4;{prefix}kernel_scalar_bindings=bias:f32:1;{prefix}output_comparison_profile_contract={COMPARISON_PROFILE_CONTRACT};{prefix}output_comparison_profile_package={};{prefix}output_comparison_profile_id={};{prefix}output_comparison_profile_source_hash={};{prefix}output_comparison_descriptor_contract=nuis-provider-output-comparison-descriptor-v1;{prefix}output_comparison_output_buffer=output.features;{prefix}output_comparison_element_type=f32;{prefix}output_comparison_shape=1x1x4;{prefix}output_comparison_expected_path={WITSAGE_METAL_EXPECTED_FILE_NAME};{prefix}output_comparison_expected_byte_length={};{prefix}output_comparison_expected_content_hash={};{prefix}output_comparison_absolute_tolerance={};{prefix}output_comparison_relative_tolerance={};{prefix}output_comparison_non_finite_policy={};{prefix}dependency_contract=nuis-provider-request-dependency-v1;{prefix}dependency_count=1;{prefix}dependency_0_producer_request_id=witsage.vector.add;{prefix}dependency_0_producer_output_buffer=output.features;{prefix}dependency_0_consumer_input_buffer=input.features;{prefix}dependency_0_transport_contract=nuis-provider-edge-transport-v1;{prefix}dependency_0_transport_ownership_token=glm:provider-edge:witsage.vector.add:output.features->witsage.vector.metal-bias:input.features;{prefix}dependency_0_transport_staging_mode=auto;{prefix}dependency_0_transport_producer_clock_evidence=provider-clock:request-3:completed;{prefix}dependency_0_transport_consumer_clock_evidence=provider-clock:request-4:dispatch-ready;{prefix}input_binding_contract=nuis-provider-input-binding-v1;{prefix}input_binding_count=1;{prefix}input_binding_0_name=input.features;{prefix}input_binding_0_source=dependency;{prefix}input_binding_0_element_type=f32;{prefix}input_binding_0_shape=1x1x4;{prefix}input_binding_0_byte_length={};{prefix}input_binding_0_content_hash={};{prefix}input_binding_0_payload_path=none;{prefix}input_binding_0_producer_request_id=witsage.vector.add;{prefix}input_binding_0_producer_output_buffer=output.features;{prefix}adapter_binding_contract=nuis-provider-request-adapter-binding-v1;{prefix}adapter_binding_provider_family=metal:apple-silicon-gpu;{prefix}adapter_binding_execution_requirement=real-device",
        WITSAGE_ADD_EXPECTED.len(),
        fnv1a64_hex(WITSAGE_ADD_EXPECTED),
        profile.package_id,
        profile.profile_id,
        profile.source_hash,
        WITSAGE_METAL_EXPECTED.len(),
        fnv1a64_hex(WITSAGE_METAL_EXPECTED),
        profile.absolute_tolerance,
        profile.relative_tolerance,
        profile.non_finite_policy,
        WITSAGE_ADD_EXPECTED.len(),
        fnv1a64_hex(WITSAGE_ADD_EXPECTED),
    ) + ";" + code_asset
}

fn kmeans_collection_request(index: usize, model: &[u8], profile: &ComparisonProfile) -> String {
    let prefix = format!("provider_request_{index}_");
    let request = format!(
        "{prefix}buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1;{prefix}buffer_id=input.features;{prefix}buffer_element_type=f32;{prefix}buffer_layout=tensor-contiguous;{prefix}buffer_shape=1x1x4;{prefix}buffer_row_stride_bytes=16;{prefix}buffer_byte_length={};{prefix}buffer_payload_path={WITSAGE_VECTOR_PAYLOAD_FILE_NAME};{prefix}buffer_content_hash={};{prefix}kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1;{prefix}kernel_id=witsage.kmeans.centroid-score;{prefix}kernel_operation=model-predict;{prefix}kernel_input_buffer=input.features;{prefix}kernel_output_buffer=output.features;{prefix}kernel_dispatch=1x1x2;{prefix}model_asset_descriptor_contract=nuis-provider-model-asset-descriptor-v1;{prefix}model_asset_id=witsage.kmeans-centroid-score.coreml;{prefix}model_asset_format=coreml-specification;{prefix}model_asset_path={WITSAGE_KMEANS_MODEL_FILE_NAME};{prefix}model_asset_byte_length={};{prefix}model_asset_content_hash={};{prefix}model_asset_input_feature=input.features;{prefix}model_asset_output_feature=output.features;{prefix}output_comparison_profile_contract={COMPARISON_PROFILE_CONTRACT};{prefix}output_comparison_profile_package={};{prefix}output_comparison_profile_id={};{prefix}output_comparison_profile_source_hash={};{prefix}output_comparison_descriptor_contract=nuis-provider-output-comparison-descriptor-v1;{prefix}output_comparison_output_buffer=output.features;{prefix}output_comparison_element_type=f32;{prefix}output_comparison_shape=1x1x2;{prefix}output_comparison_expected_path={WITSAGE_KMEANS_EXPECTED_FILE_NAME};{prefix}output_comparison_expected_byte_length={};{prefix}output_comparison_expected_content_hash={};{prefix}output_comparison_absolute_tolerance={};{prefix}output_comparison_relative_tolerance={};{prefix}output_comparison_non_finite_policy={};{prefix}dependency_contract=nuis-provider-request-dependency-v1;{prefix}dependency_count=0",
        WITSAGE_VECTOR_PAYLOAD.len(),
        fnv1a64_hex(WITSAGE_VECTOR_PAYLOAD),
        model.len(),
        fnv1a64_hex(model),
        profile.package_id,
        profile.profile_id,
        profile.source_hash,
        WITSAGE_KMEANS_EXPECTED.len(),
        fnv1a64_hex(WITSAGE_KMEANS_EXPECTED),
        profile.absolute_tolerance,
        profile.relative_tolerance,
        profile.non_finite_policy,
    );
    format!(
        "{request};{}",
        input_binding(
            &prefix,
            "artifact",
            "1x1x4",
            WITSAGE_VECTOR_PAYLOAD.len(),
            &fnv1a64_hex(WITSAGE_VECTOR_PAYLOAD),
            WITSAGE_VECTOR_PAYLOAD_FILE_NAME,
            "none",
            "none",
        )
    )
}

fn kmeans_assignment_collection_request(index: usize, code_asset: &str) -> String {
    let prefix = format!("provider_request_{index}_");
    let request = format!(
        "{prefix}buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1;{prefix}buffer_id=input.scores;{prefix}buffer_element_type=f32;{prefix}buffer_layout=tensor-contiguous;{prefix}buffer_shape=1x1x2;{prefix}buffer_row_stride_bytes=8;{prefix}buffer_byte_length={};{prefix}buffer_payload_path={WITSAGE_KMEANS_EXPECTED_FILE_NAME};{prefix}buffer_content_hash={};{prefix}kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1;{prefix}kernel_id=witsage.kmeans.assignment;{prefix}kernel_operation=argmax;{prefix}kernel_input_buffer=input.scores;{prefix}kernel_output_buffer=output.assignment;{prefix}kernel_dispatch=1x1x1;{prefix}output_binding_contract=nuis-provider-output-binding-v1;{prefix}output_binding_count=1;{prefix}output_binding_0_role=output.assignment;{prefix}output_binding_0_buffer=output.assignment;{prefix}output_binding_0_element_type=u32;{prefix}output_binding_0_shape=1;{prefix}output_binding_0_byte_length={};{prefix}output_binding_0_comparison_id=comparison.output.assignment;{prefix}output_comparison_id=comparison.output.assignment;{prefix}output_comparison_descriptor_contract=nuis-provider-output-comparison-descriptor-v1;{prefix}output_comparison_output_buffer=output.assignment;{prefix}output_comparison_element_type=u32;{prefix}output_comparison_shape=1;{prefix}output_comparison_expected_path={WITSAGE_KMEANS_ASSIGNMENT_EXPECTED_FILE_NAME};{prefix}output_comparison_expected_byte_length={};{prefix}output_comparison_expected_content_hash={};{prefix}output_comparison_absolute_tolerance=0;{prefix}output_comparison_relative_tolerance=0;{prefix}output_comparison_non_finite_policy=reject;{prefix}dependency_contract=nuis-provider-request-dependency-v1;{prefix}dependency_count=1;{prefix}dependency_0_producer_request_id=witsage.kmeans.centroid-score;{prefix}dependency_0_producer_output_buffer=output.features;{prefix}dependency_0_consumer_input_buffer=input.scores;{prefix}input_binding_contract=nuis-provider-input-binding-v1;{prefix}input_binding_count=1;{prefix}input_binding_0_name=input.scores;{prefix}input_binding_0_source=dependency;{prefix}input_binding_0_element_type=f32;{prefix}input_binding_0_shape=1x1x2;{prefix}input_binding_0_byte_length={};{prefix}input_binding_0_content_hash={};{prefix}input_binding_0_payload_path=none;{prefix}input_binding_0_producer_request_id=witsage.kmeans.centroid-score;{prefix}input_binding_0_producer_output_buffer=output.features;{prefix}adapter_binding_contract=nuis-provider-request-adapter-binding-v1;{prefix}adapter_binding_provider_family=metal:apple-silicon-gpu;{prefix}adapter_binding_execution_requirement=real-device;{prefix}assignment_contract=nuis-witsage-cluster-assignment-v1;{prefix}assignment_package=nuis.witsage;{prefix}assignment_policy=highest-centroid-score;{prefix}assignment_output_buffer=output.assignment;{prefix}assignment_element_type=u32",
        WITSAGE_KMEANS_EXPECTED.len(),
        fnv1a64_hex(WITSAGE_KMEANS_EXPECTED),
        WITSAGE_KMEANS_ASSIGNMENT_EXPECTED.len(),
        WITSAGE_KMEANS_ASSIGNMENT_EXPECTED.len(),
        fnv1a64_hex(WITSAGE_KMEANS_ASSIGNMENT_EXPECTED),
        WITSAGE_KMEANS_EXPECTED.len(),
        fnv1a64_hex(WITSAGE_KMEANS_EXPECTED),
    );
    let request = with_dependency_transport(
        request,
        index,
        0,
        "input.scores",
        "glm:provider-edge:witsage.kmeans.centroid-score:output.features->witsage.kmeans.assignment:input.scores",
        5,
    );
    request + ";" + code_asset
}

#[allow(clippy::too_many_arguments)]
fn input_binding(
    prefix: &str,
    source: &str,
    shape: &str,
    byte_length: usize,
    content_hash: &str,
    payload_path: &str,
    producer_request_id: &str,
    producer_output_buffer: &str,
) -> String {
    format!(
        "{prefix}input_binding_contract=nuis-provider-input-binding-v1;{prefix}input_binding_count=1;{prefix}input_binding_0_name=input.features;{prefix}input_binding_0_source={source};{prefix}input_binding_0_element_type=f32;{prefix}input_binding_0_shape={shape};{prefix}input_binding_0_byte_length={byte_length};{prefix}input_binding_0_content_hash={content_hash};{prefix}input_binding_0_payload_path={payload_path};{prefix}input_binding_0_producer_request_id={producer_request_id};{prefix}input_binding_0_producer_output_buffer={producer_output_buffer};{prefix}adapter_binding_contract=nuis-provider-request-adapter-binding-v1;{prefix}adapter_binding_provider_family=coreml:apple-ane;{prefix}adapter_binding_execution_requirement=real-device"
    )
}

fn with_dependency_transport(
    request: String,
    request_index: usize,
    edge_index: usize,
    consumer_input: &str,
    ownership_token: &str,
    producer_index: usize,
) -> String {
    let prefix = format!("provider_request_{request_index}_dependency_{edge_index}_");
    let anchor = format!("{prefix}consumer_input_buffer={consumer_input}");
    let transport = format!(
        "{anchor};{prefix}transport_contract=nuis-provider-edge-transport-v1;{prefix}transport_ownership_token={ownership_token};{prefix}transport_staging_mode=auto;{prefix}transport_producer_clock_evidence=provider-clock:request-{producer_index}:completed;{prefix}transport_consumer_clock_evidence=provider-clock:request-{request_index}:dispatch-ready"
    );
    request.replacen(&anchor, &transport, 1)
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

    #[test]
    fn package_profile_is_non_zero_and_fail_closed() {
        for (source, profile_id) in [
            (
                CROSS_PROVIDER_COMPARISON_PROFILE_SOURCE,
                "witsage.cross-provider.f32",
            ),
            (
                MODEL_PREDICT_COMPARISON_PROFILE_SOURCE,
                "witsage.model-predict.f32",
            ),
        ] {
            let profile = load_comparison_profile(source, profile_id).expect("profile");
            assert_eq!(profile.package_id, "nuis.witsage");
            assert_eq!(profile.profile_id, profile_id);
            assert!(profile.absolute_tolerance.parse::<f64>().unwrap() > 0.0);
            assert!(profile.relative_tolerance.parse::<f64>().unwrap() > 0.0);
            assert_eq!(profile.non_finite_policy, "reject");
            assert!(profile.source_hash.starts_with("0x"));
        }
    }

    #[test]
    fn package_manifest_declares_profile_surface_and_asset() {
        let manifest = include_str!("../../../stdlib/witsage/module.toml");
        assert!(manifest.contains("contract.witsage.output-comparison-profile.v1"));
        assert!(manifest.contains("provider_comparison_profiles"));
        assert!(manifest.contains("provider-comparison-profiles/cross-provider-f32.nwcp"));
        assert!(manifest.contains("provider-comparison-profiles/model-predict-f32.nwcp"));
    }

    #[test]
    fn metal_request_uses_profile_without_weakening_expected_hash() {
        let evidence = input_evidence("base");
        assert!(evidence.contains(
            "provider_request_4_output_comparison_profile_contract=nuis-witsage-output-comparison-profile-v1"
        ));
        assert!(
            evidence.contains("provider_request_4_output_comparison_absolute_tolerance=0.00001")
        );
        assert!(
            evidence.contains("provider_request_4_output_comparison_relative_tolerance=0.000001")
        );
        assert!(evidence.contains("provider_request_4_output_comparison_non_finite_policy=reject"));
        assert!(evidence.contains("provider_request_4_output_comparison_expected_content_hash=0x"));
        assert!(evidence.contains("provider_request_3_output_comparison_absolute_tolerance=0"));
        assert!(evidence.contains("provider_request_count=7"));
        assert!(evidence.contains("provider_request_0_buffer_row_stride_bytes=262144"));
        assert!(evidence.contains("provider_request_5_kernel_id=witsage.kmeans.centroid-score"));
        assert!(evidence
            .contains("provider_request_5_output_comparison_profile_id=witsage.model-predict.f32"));
        assert!(evidence.contains("provider_request_5_output_comparison_expected_content_hash=0x"));
        assert!(evidence.contains("provider_request_5_output_comparison_non_finite_policy=reject"));
        assert!(evidence.contains("provider_request_6_kernel_id=witsage.kmeans.assignment"));
        assert!(evidence.contains("provider_request_6_kernel_operation=argmax"));
        assert!(
            evidence.contains("provider_request_4_code_asset_id=shader.witsage.vector-bias.metal")
        );
        assert!(evidence.contains("provider_request_6_code_asset_id=shader.witsage.argmax.metal"));
        assert!(evidence.contains("provider_code_asset_identity_set_count=2"));
        assert!(evidence
            .contains("provider_code_asset_identity_item_0_owner_package_id=official.shader"));
        assert!(evidence.contains(
            "provider_code_asset_identity_item_1_provider_family=metal:apple-silicon-gpu"
        ));
        assert!(evidence
            .contains("provider_request_6_assignment_contract=nuis-witsage-cluster-assignment-v1"));
        assert!(evidence.contains("provider_request_6_output_binding_0_element_type=u32"));
        assert!(evidence.contains(
            "provider_request_6_dependency_0_producer_request_id=witsage.kmeans.centroid-score"
        ));
    }
}
