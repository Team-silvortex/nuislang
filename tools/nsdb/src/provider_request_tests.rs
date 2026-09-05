use super::*;
use crate::provider_output_binding::LEGACY_PROVIDER_OUTPUT_BINDING_CONTRACT;

const REGISTERED: &str = "provider_buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1;provider_buffer_id=input.pixels;provider_buffer_element_type=u8;provider_buffer_layout=image-2d-row-major:pixel-format=gray8;provider_buffer_shape=2x2;provider_buffer_row_stride_bytes=2;provider_buffer_byte_length=4;provider_buffer_payload_path=pixels.bin;provider_buffer_content_hash=0x1234;provider_kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1;provider_kernel_id=pixelmagic.gray8.invert;provider_kernel_operation=invert;provider_kernel_input_buffer=input.pixels;provider_kernel_output_buffer=output.pixels;provider_kernel_dispatch=2x2x1;provider_kernel_scalar_bindings=max_value:u8:15";

fn indexed_request(index: usize, kernel_id: &str) -> String {
    REGISTERED
        .replace(
            "provider_buffer_",
            &format!("provider_request_{index}_buffer_"),
        )
        .replace(
            "provider_kernel_",
            &format!("provider_request_{index}_kernel_"),
        )
        .replace("pixelmagic.gray8.invert", kernel_id)
}

fn project_asset_request(index: usize, kernel_id: &str, entry: &str, asset_id: &str) -> String {
    format!(
        "{};provider_request_{index}_code_asset_descriptor_contract={};provider_request_{index}_code_asset_id={asset_id};provider_request_{index}_code_asset_format=ptx;provider_request_{index}_code_asset_target=sm_80;provider_request_{index}_code_asset_entry={entry};provider_request_{index}_code_asset_path=kernel.ptx;provider_request_{index}_code_asset_byte_length=512;provider_request_{index}_code_asset_digest_contract={};provider_request_{index}_code_asset_content_hash=0x0123456789abcdef",
        indexed_request(index, kernel_id),
        crate::provider_code_asset::PROVIDER_CODE_ASSET_DESCRIPTOR_CONTRACT,
        crate::provider_code_asset::CODE_ASSET_FNV1A64_DIGEST_CONTRACT,
    )
}

fn project_asset_collection() -> String {
    let entries = ["project_map", "project_reduce"];
    let source_fnv1a64 = "0x1111111111111111";
    let lowering_target = "cuda.nvidia-gpu";
    let identity_hash = crate::provider_code_asset_identity::project_code_asset_identity_hash(
        source_fnv1a64,
        lowering_target,
        &entries,
    );
    let asset_id = format!("kernel.cuda.project.{}", &identity_hash[2..]);
    let identity_set_root_hash =
        crate::provider_code_asset_identity::code_asset_identity_set_root_hash(&[(
            &asset_id,
            crate::provider_code_asset_identity::PROJECT_CODE_ASSET_IDENTITY_CONTRACT,
            &identity_hash,
        )]);
    format!(
        "provider_request_collection_contract={PROVIDER_REQUEST_COLLECTION_CONTRACT};provider_request_count=2;provider_code_asset_identity_contract={};provider_code_asset_identity_asset_id={asset_id};provider_code_asset_identity_source_fnv1a64={source_fnv1a64};provider_code_asset_identity_lowering_target={lowering_target};provider_code_asset_identity_entry_count=2;provider_code_asset_identity_entries={};provider_code_asset_identity_hash={identity_hash};provider_code_asset_identity_set_contract={};provider_code_asset_identity_set_count=1;provider_code_asset_identity_set_root_hash={identity_set_root_hash};provider_code_asset_identity_item_0_asset_id={asset_id};provider_code_asset_identity_item_0_contract={};provider_code_asset_identity_item_0_hash={identity_hash};provider_code_asset_identity_item_0_source_fnv1a64={source_fnv1a64};provider_code_asset_identity_item_0_lowering_target={lowering_target};provider_code_asset_identity_item_0_entry_count=2;provider_code_asset_identity_item_0_entries={};{};{}",
        crate::provider_code_asset_identity::PROJECT_CODE_ASSET_IDENTITY_CONTRACT,
        entries.join(","),
        crate::provider_code_asset_identity::CODE_ASSET_IDENTITY_SET_CONTRACT,
        crate::provider_code_asset_identity::PROJECT_CODE_ASSET_IDENTITY_CONTRACT,
        entries.join(","),
        project_asset_request(0, "project.map", entries[0], &asset_id),
        project_asset_request(1, "project.reduce", entries[1], &asset_id),
    )
}

fn mixed_asset_collection() -> String {
    let project_entry = "project_map";
    let source_hash = "0x1111111111111111";
    let target = "cuda.nvidia-gpu";
    let project_hash = crate::provider_code_asset_identity::project_code_asset_identity_hash(
        source_hash,
        target,
        &[project_entry],
    );
    let project_id = format!("kernel.cuda.project.{}", &project_hash[2..]);
    let descriptor_id = "shader.future.descriptor";
    let descriptor_entry = "future_entry";
    let descriptor_hash = crate::provider_code_asset_identity::descriptor_code_asset_identity_hash(
        descriptor_id,
        "ptx",
        "sm_80",
        "kernel.ptx",
        512,
        crate::provider_code_asset::CODE_ASSET_FNV1A64_DIGEST_CONTRACT,
        "0x0123456789abcdef",
        &[descriptor_entry],
    );
    let set_root = crate::provider_code_asset_identity::code_asset_identity_set_root_hash(&[
        (
            &project_id,
            crate::provider_code_asset_identity::PROJECT_CODE_ASSET_IDENTITY_CONTRACT,
            &project_hash,
        ),
        (
            descriptor_id,
            crate::provider_code_asset_identity::DESCRIPTOR_CODE_ASSET_IDENTITY_CONTRACT,
            &descriptor_hash,
        ),
    ]);
    format!(
        "provider_request_collection_contract={PROVIDER_REQUEST_COLLECTION_CONTRACT};provider_request_count=2;provider_code_asset_identity_contract={};provider_code_asset_identity_asset_id={project_id};provider_code_asset_identity_source_fnv1a64={source_hash};provider_code_asset_identity_lowering_target={target};provider_code_asset_identity_entry_count=1;provider_code_asset_identity_entries={project_entry};provider_code_asset_identity_hash={project_hash};provider_code_asset_identity_set_contract={};provider_code_asset_identity_set_count=2;provider_code_asset_identity_set_root_hash={set_root};provider_code_asset_identity_item_0_asset_id={project_id};provider_code_asset_identity_item_0_contract={};provider_code_asset_identity_item_0_hash={project_hash};provider_code_asset_identity_item_0_source_fnv1a64={source_hash};provider_code_asset_identity_item_0_lowering_target={target};provider_code_asset_identity_item_0_entry_count=1;provider_code_asset_identity_item_0_entries={project_entry};provider_code_asset_identity_item_1_asset_id={descriptor_id};provider_code_asset_identity_item_1_contract={};provider_code_asset_identity_item_1_hash={descriptor_hash};{};{}",
        crate::provider_code_asset_identity::PROJECT_CODE_ASSET_IDENTITY_CONTRACT,
        crate::provider_code_asset_identity::CODE_ASSET_IDENTITY_SET_CONTRACT,
        crate::provider_code_asset_identity::PROJECT_CODE_ASSET_IDENTITY_CONTRACT,
        crate::provider_code_asset_identity::DESCRIPTOR_CODE_ASSET_IDENTITY_CONTRACT,
        project_asset_request(0, "project.map", project_entry, &project_id),
        project_asset_request(1, "future.map", descriptor_entry, descriptor_id),
    )
}

#[test]
fn parses_registered_buffer_and_kernel_descriptors() {
    let request = provider_request_from_evidence(REGISTERED).expect("registered request");
    assert_eq!(request.source, "registered-descriptors");
    assert_eq!(request.buffer.shape, [2, 2]);
    assert_eq!(request.kernel.dispatch, [2, 2, 1]);
    assert_eq!(request.scalar_u8("max_value"), Some(15));
    assert_eq!(
        request.input_bindings[0].layout,
        "image-2d-row-major:pixel-format=gray8"
    );
    assert_eq!(request.input_bindings[0].row_stride_bytes, 2);
}

#[test]
fn parses_runtime_result_binding_and_rejects_invalid_source_identity() {
    let evidence = format!(
        "{REGISTERED};provider_runtime_result_binding_contract={PROVIDER_RUNTIME_RESULT_BINDING_CONTRACT};provider_runtime_result_source_yir_fnv1a64=0x0123456789abcdef;provider_runtime_result_module=shader;provider_runtime_result_instruction=draw_instanced;provider_runtime_result_node=draw.frame;provider_runtime_result_resource=shader0"
    );
    let request = provider_request_from_evidence(&evidence).expect("runtime result binding");
    let binding = request.runtime_result_binding.expect("binding");
    assert_eq!(binding.module, "shader");
    assert_eq!(binding.instruction, "draw_instanced");
    assert_eq!(binding.node, "draw.frame");

    let invalid = evidence.replace("0x0123456789abcdef", "not-a-hash");
    assert!(provider_request_from_evidence(&invalid).is_none());
}

#[test]
fn parses_rank_three_contiguous_tensor_with_flat_span_stride() {
    let evidence = "provider_buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1;provider_buffer_id=input.features;provider_buffer_element_type=f32;provider_buffer_layout=tensor-contiguous;provider_buffer_shape=16x64x64;provider_buffer_row_stride_bytes=262144;provider_buffer_byte_length=262144;provider_buffer_payload_path=features.bin;provider_buffer_content_hash=0x1234;provider_kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1;provider_kernel_id=tensor.rank3.copy;provider_kernel_operation=copy;provider_kernel_input_buffer=input.features;provider_kernel_output_buffer=output.features;provider_kernel_dispatch=16x64x64";
    let request = provider_request_from_evidence(evidence).expect("rank-three tensor request");
    assert_eq!(request.buffer.shape, [16, 64, 64]);
    assert_eq!(request.buffer.row_stride_bytes, 262_144);
    assert_eq!(request.input_bindings[0].row_stride_bytes, 262_144);
    assert_eq!(request.output_bindings[0].row_stride_bytes, 262_144);
}

#[test]
fn parses_ordered_multi_output_bindings_with_compatibility_primary() {
    let evidence = format!(
        "{REGISTERED};provider_output_binding_contract={LEGACY_PROVIDER_OUTPUT_BINDING_CONTRACT};provider_output_binding_count=2;provider_output_binding_0_role=output.primary;provider_output_binding_0_buffer=output.pixels;provider_output_binding_0_element_type=u8;provider_output_binding_0_shape=2x2;provider_output_binding_0_byte_length=4;provider_output_binding_0_comparison_id=none;provider_output_binding_1_role=output.audit;provider_output_binding_1_buffer=output.audit;provider_output_binding_1_element_type=u64;provider_output_binding_1_shape=3;provider_output_binding_1_byte_length=24;provider_output_binding_1_comparison_id=none"
    );
    let request = provider_request_from_evidence(&evidence).expect("multi-output request");
    assert_eq!(
        request
            .output_bindings
            .iter()
            .map(|binding| (binding.role.as_str(), binding.buffer.as_str()))
            .collect::<Vec<_>>(),
        [
            ("output.primary", "output.pixels"),
            ("output.audit", "output.audit")
        ]
    );
    assert_eq!(request.output_bindings[0].element_type, "u8");
    assert_eq!(
        request.output_bindings[0].layout,
        "image-2d-row-major:pixel-format=gray8"
    );
    assert_eq!(request.output_bindings[0].shape, [2, 2]);
    assert_eq!(request.output_bindings[0].row_stride_bytes, 2);
    assert_eq!(request.output_bindings[0].byte_length, 4);
    assert_eq!(request.output_bindings[1].element_type, "u64");
    assert_eq!(request.output_bindings[1].layout, "tensor-contiguous");
    assert_eq!(request.output_bindings[1].shape, [3]);
    assert_eq!(request.output_bindings[1].row_stride_bytes, 24);
    assert_eq!(request.output_bindings[1].byte_length, 24);
}

#[test]
fn parses_typed_output_binding_and_rejects_short_row_stride() {
    let evidence = format!(
        "{REGISTERED};provider_output_binding_contract={PROVIDER_OUTPUT_BINDING_CONTRACT};provider_output_binding_count=1;provider_output_binding_0_role=output.primary;provider_output_binding_0_buffer=output.pixels;provider_output_binding_0_element_type=u8;provider_output_binding_0_layout=tensor-row-major;provider_output_binding_0_shape=2x2;provider_output_binding_0_row_stride_bytes=2;provider_output_binding_0_byte_length=4;provider_output_binding_0_comparison_id=none"
    );
    let request = provider_request_from_evidence(&evidence).expect("typed output binding");
    assert_eq!(request.output_bindings[0].layout, "tensor-row-major");
    assert_eq!(request.output_bindings[0].row_stride_bytes, 2);

    let invalid = evidence.replace(
        "provider_output_binding_0_row_stride_bytes=2",
        "provider_output_binding_0_row_stride_bytes=1",
    );
    assert!(provider_request_from_evidence(&invalid).is_none());
}

#[test]
fn rejects_registered_descriptor_with_mismatched_buffer_binding() {
    let invalid = REGISTERED.replace(
        "provider_kernel_input_buffer=input.pixels",
        "provider_kernel_input_buffer=missing",
    );
    assert!(provider_request_from_evidence(&invalid).is_none());
}

#[test]
fn declared_collection_never_falls_back_to_singular_request() {
    let evidence = format!(
        "{REGISTERED};provider_request_collection_contract={PROVIDER_REQUEST_COLLECTION_CONTRACT};provider_request_count=1"
    );
    assert!(provider_request_collection_from_evidence(&evidence).is_none());
}

#[test]
fn parses_hash_bound_model_asset_descriptor() {
    let evidence = format!(
        "{REGISTERED};provider_model_asset_descriptor_contract={PROVIDER_MODEL_ASSET_DESCRIPTOR_CONTRACT};provider_model_asset_id=model;provider_model_asset_format=coreml-specification;provider_model_asset_path=model.mlmodel;provider_model_asset_byte_length=128;provider_model_asset_content_hash=0xabcd;provider_model_asset_input_feature=input.pixels;provider_model_asset_output_feature=output.pixels"
    );
    let model = provider_request_from_evidence(&evidence)
        .expect("model request")
        .model_asset
        .expect("model asset");
    assert_eq!(model.path, "model.mlmodel");
    assert_eq!(model.byte_length, 128);
}

#[test]
fn parses_hash_bound_provider_code_asset_descriptor() {
    let evidence = format!(
        "{REGISTERED};provider_code_asset_descriptor_contract={};provider_code_asset_id=kernel.vector-add;provider_code_asset_format=ptx;provider_code_asset_target=sm_80;provider_code_asset_entry=nuis_kernel_vector_add_f32;provider_code_asset_path=payload/kernel.ptx;provider_code_asset_byte_length=512;provider_code_asset_digest_contract={};provider_code_asset_content_hash=0x0123456789abcdef",
        crate::provider_code_asset::PROVIDER_CODE_ASSET_DESCRIPTOR_CONTRACT,
        crate::provider_code_asset::CODE_ASSET_FNV1A64_DIGEST_CONTRACT,
    );
    let asset = provider_request_from_evidence(&evidence)
        .expect("request with code asset")
        .code_asset
        .expect("code asset");
    assert_eq!(asset.path, "payload/kernel.ptx");
    assert_eq!(asset.entry, "nuis_kernel_vector_add_f32");
    let rendered = crate::provider_request_payload::render_provider_request_evidence(&evidence);
    assert!(rendered.contains(
        "provider_code_asset_descriptor_contract = \"nuis-provider-code-asset-descriptor-v1\""
    ));
    assert!(rendered.contains("provider_code_asset_entry = \"nuis_kernel_vector_add_f32\""));
    assert!(
        rendered.contains("provider_input_binding_contract = \"nuis-provider-input-binding-v2\"")
    );
    assert!(rendered
        .contains("provider_input_binding_0_layout = \"image-2d-row-major:pixel-format=gray8\""));
    assert!(rendered.contains("provider_input_binding_0_row_stride_bytes = \"2\""));
    assert!(
        rendered.contains("provider_output_binding_contract = \"nuis-provider-output-binding-v2\"")
    );
    assert!(rendered
        .contains("provider_output_binding_0_layout = \"image-2d-row-major:pixel-format=gray8\""));
    assert!(rendered.contains("provider_output_binding_0_row_stride_bytes = \"2\""));
}

#[test]
fn rejects_partial_provider_code_asset_descriptor() {
    let evidence = format!("{REGISTERED};provider_code_asset_id=kernel.vector-add");
    assert!(provider_request_from_evidence(&evidence).is_none());
}

#[test]
fn accepts_model_request_without_scalar_bindings() {
    let evidence = REGISTERED
        .replace(
            ";provider_kernel_scalar_bindings=max_value:u8:15",
            ";provider_model_asset_descriptor_contract=nuis-provider-model-asset-descriptor-v1;provider_model_asset_id=model;provider_model_asset_format=coreml-specification;provider_model_asset_path=model.mlmodel;provider_model_asset_byte_length=128;provider_model_asset_content_hash=0xabcd;provider_model_asset_input_feature=input.pixels;provider_model_asset_output_feature=output.pixels",
        );
    let request = provider_request_from_evidence(&evidence).expect("scalar-free model request");
    assert!(request.kernel.scalar_bindings.is_empty());
    assert!(request.model_asset.is_some());
}

#[test]
fn parses_hash_bound_output_comparison_descriptor() {
    let evidence = format!(
        "{REGISTERED};provider_output_comparison_descriptor_contract={PROVIDER_OUTPUT_COMPARISON_DESCRIPTOR_CONTRACT};provider_output_comparison_output_buffer=output.pixels;provider_output_comparison_element_type=f32;provider_output_comparison_shape=2x2;provider_output_comparison_expected_path=expected.bin;provider_output_comparison_expected_byte_length=16;provider_output_comparison_expected_content_hash=0xabcd;provider_output_comparison_absolute_tolerance=0.001;provider_output_comparison_relative_tolerance=0.01;provider_output_comparison_non_finite_policy=reject"
    );
    let comparison = provider_request_from_evidence(&evidence)
        .expect("request with comparison")
        .output_comparison
        .expect("comparison descriptor");
    assert_eq!(comparison.id, "comparison.output.pixels");
    assert_eq!(comparison.shape, [2, 2]);
    assert_eq!(comparison.expected_byte_length, 16);
    assert_eq!(comparison.non_finite_policy, "reject");
}

#[test]
fn parses_output_comparison_collection_bound_to_distinct_outputs() {
    let evidence = format!(
        "{REGISTERED};provider_output_binding_contract={LEGACY_PROVIDER_OUTPUT_BINDING_CONTRACT};provider_output_binding_count=2;provider_output_binding_0_role=output.primary;provider_output_binding_0_buffer=output.pixels;provider_output_binding_0_element_type=u64;provider_output_binding_0_shape=3;provider_output_binding_0_byte_length=24;provider_output_binding_0_comparison_id=comparison.primary;provider_output_binding_1_role=output.audit;provider_output_binding_1_buffer=output.audit;provider_output_binding_1_element_type=u64;provider_output_binding_1_shape=3;provider_output_binding_1_byte_length=24;provider_output_binding_1_comparison_id=comparison.audit;provider_output_comparison_collection_contract={PROVIDER_OUTPUT_COMPARISON_COLLECTION_CONTRACT};provider_output_comparison_collection_count=2;provider_output_comparison_item_0_id=comparison.primary;provider_output_comparison_item_0_descriptor_contract={PROVIDER_OUTPUT_COMPARISON_DESCRIPTOR_CONTRACT};provider_output_comparison_item_0_output_buffer=output.pixels;provider_output_comparison_item_0_element_type=u64;provider_output_comparison_item_0_shape=3;provider_output_comparison_item_0_expected_path=primary.bin;provider_output_comparison_item_0_expected_byte_length=24;provider_output_comparison_item_0_expected_content_hash=0xprimary;provider_output_comparison_item_0_absolute_tolerance=0;provider_output_comparison_item_0_relative_tolerance=0;provider_output_comparison_item_0_non_finite_policy=reject;provider_output_comparison_item_1_id=comparison.audit;provider_output_comparison_item_1_descriptor_contract={PROVIDER_OUTPUT_COMPARISON_DESCRIPTOR_CONTRACT};provider_output_comparison_item_1_output_buffer=output.audit;provider_output_comparison_item_1_element_type=u64;provider_output_comparison_item_1_shape=3;provider_output_comparison_item_1_expected_path=audit.bin;provider_output_comparison_item_1_expected_byte_length=24;provider_output_comparison_item_1_expected_content_hash=0xaudit;provider_output_comparison_item_1_absolute_tolerance=0;provider_output_comparison_item_1_relative_tolerance=0;provider_output_comparison_item_1_non_finite_policy=reject"
    );
    let request = provider_request_from_evidence(&evidence).expect("comparison collection");
    assert_eq!(request.output_comparisons.len(), 2);
    assert_eq!(request.output_comparisons[0].id, "comparison.primary");
    assert_eq!(request.output_comparisons[1].output_buffer, "output.audit");
    assert_eq!(
        request
            .output_bindings
            .iter()
            .map(|binding| binding.comparison_id.as_str())
            .collect::<Vec<_>>(),
        ["comparison.primary", "comparison.audit"]
    );
}

#[test]
fn rejects_output_comparison_with_mismatched_shape_bytes() {
    let evidence = format!(
        "{REGISTERED};provider_output_comparison_descriptor_contract={PROVIDER_OUTPUT_COMPARISON_DESCRIPTOR_CONTRACT};provider_output_comparison_output_buffer=output.pixels;provider_output_comparison_element_type=f32;provider_output_comparison_shape=2x2;provider_output_comparison_expected_path=expected.bin;provider_output_comparison_expected_byte_length=8;provider_output_comparison_expected_content_hash=0xabcd;provider_output_comparison_absolute_tolerance=0;provider_output_comparison_relative_tolerance=0;provider_output_comparison_non_finite_policy=reject"
    );
    assert!(provider_request_from_evidence(&evidence).is_none());
}

#[test]
fn rejects_output_comparison_with_invalid_tolerance_policy() {
    let evidence = format!(
        "{REGISTERED};provider_output_comparison_descriptor_contract={PROVIDER_OUTPUT_COMPARISON_DESCRIPTOR_CONTRACT};provider_output_comparison_output_buffer=output.pixels;provider_output_comparison_element_type=f32;provider_output_comparison_shape=2x2;provider_output_comparison_expected_path=expected.bin;provider_output_comparison_expected_byte_length=16;provider_output_comparison_expected_content_hash=0xabcd;provider_output_comparison_absolute_tolerance=-1;provider_output_comparison_relative_tolerance=0;provider_output_comparison_non_finite_policy=permit"
    );
    assert!(provider_request_from_evidence(&evidence).is_none());
}

#[test]
fn parses_ordered_provider_request_collection() {
    let evidence = format!(
        "provider_request_collection_contract={PROVIDER_REQUEST_COLLECTION_CONTRACT};provider_request_count=2;{};{}",
        indexed_request(0, "first"),
        indexed_request(1, "second")
    );
    let collection = provider_request_collection_from_evidence(&evidence).expect("collection");
    assert_eq!(collection.source, "registered-collection");
    assert_eq!(collection.requests.len(), 2);
    assert_eq!(collection.requests[0].kernel.id, "first");
    assert_eq!(collection.requests[1].kernel.id, "second");
}

#[test]
fn verifies_project_code_asset_identity_across_ordered_collection() {
    let evidence = project_asset_collection();
    let collection =
        provider_request_collection_from_evidence(&evidence).expect("project code asset identity");
    let asset_ids = collection
        .requests
        .iter()
        .map(|request| request.code_asset.as_ref().unwrap().id.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(asset_ids.len(), 1);
    assert!(asset_ids
        .into_iter()
        .next()
        .unwrap()
        .starts_with("kernel.cuda.project."));
    let identity = collection.code_asset_identity.expect("verified identity");
    assert_eq!(identity.status, "verified");
    assert_eq!(
        identity.contract,
        "nuis-kernel-project-code-asset-identity-v1"
    );
    assert!(identity.asset_id.starts_with("kernel.cuda.project."));
    assert!(identity.identity_hash.starts_with("0x"));
    assert_eq!(identity.identity_set.asset_ids, [identity.asset_id]);
    assert!(identity.identity_set.root_hash.starts_with("0x"));
}

#[test]
fn validates_ordered_mixed_asset_identity_partitions() {
    let evidence = mixed_asset_collection();
    let collection =
        provider_request_collection_from_evidence(&evidence).expect("mixed identity set");
    let identity = collection.code_asset_identity.expect("identity set");
    assert_eq!(identity.identity_set.asset_ids.len(), 2);
    assert_eq!(
        identity.identity_set.contracts[1],
        crate::provider_code_asset_identity::DESCRIPTOR_CODE_ASSET_IDENTITY_CONTRACT
    );
    assert_eq!(identity.identity_set.identity_hashes.len(), 2);
    for drifted in [
        evidence
            .replace(
                "provider_code_asset_identity_item_0_asset_id=kernel.cuda.project.",
                "provider_code_asset_identity_item_0_asset_id=shader.future.descriptor;ignored=kernel.cuda.project.",
            ),
        evidence.replace(
            "provider_request_1_code_asset_id=shader.future.descriptor",
            "provider_request_1_code_asset_id=kernel.cuda.project.invalid",
        ),
    ] {
        assert!(provider_request_collection_from_evidence(&drifted).is_none());
    }
}

#[test]
fn rejects_missing_or_drifting_project_code_asset_identity() {
    let evidence = project_asset_collection();
    let without_identity = evidence
        .split(';')
        .filter(|field| !field.starts_with("provider_code_asset_identity_"))
        .collect::<Vec<_>>()
        .join(";");
    assert!(provider_request_collection_from_evidence(&without_identity).is_none());
    for drifted in [
        evidence.replace("0x1111111111111111", "0x1111111111111112"),
        evidence.replace(
            "provider_code_asset_identity_entries=project_map,project_reduce",
            "provider_code_asset_identity_entries=project_reduce,project_map",
        ),
        evidence.replace(
            "provider_code_asset_identity_hash=0x",
            "provider_code_asset_identity_hash=0f",
        ),
        evidence.replacen(
            "provider_request_1_code_asset_id=kernel.cuda.project.",
            "provider_request_1_code_asset_id=kernel.cuda.project.drift.",
            1,
        ),
        evidence.replace(
            "provider_code_asset_identity_set_root_hash=0x",
            "provider_code_asset_identity_set_root_hash=0f",
        ),
    ] {
        assert!(provider_request_collection_from_evidence(&drifted).is_none());
    }
}

#[test]
fn rejects_duplicate_collection_request_identity() {
    let evidence = format!(
        "provider_request_collection_contract={PROVIDER_REQUEST_COLLECTION_CONTRACT};provider_request_count=2;{};{}",
        indexed_request(0, "duplicate"),
        indexed_request(1, "duplicate")
    );
    assert!(provider_request_collection_from_evidence(&evidence).is_none());
}

fn dependency(index: usize, producer: &str) -> String {
    format!(
        "provider_request_{index}_dependency_contract={PROVIDER_REQUEST_DEPENDENCY_CONTRACT};provider_request_{index}_dependency_count=1;provider_request_{index}_dependency_0_producer_request_id={producer};provider_request_{index}_dependency_0_producer_output_buffer=output.pixels;provider_request_{index}_dependency_0_consumer_input_buffer=input.pixels"
    )
}

#[test]
fn parses_backward_provider_request_dependency() {
    let evidence = format!(
        "provider_request_collection_contract={PROVIDER_REQUEST_COLLECTION_CONTRACT};provider_request_count=2;{};{};{}",
        indexed_request(0, "first"),
        indexed_request(1, "second"),
        dependency(1, "first")
    );
    let collection = provider_request_collection_from_evidence(&evidence).expect("dependency");
    assert_eq!(collection.requests[1].dependencies.len(), 1);
    assert_eq!(
        collection.requests[1].dependencies[0].producer_request_id,
        "first"
    );
}

#[test]
fn rejects_missing_self_or_forward_dependency_target() {
    for producer in ["missing", "first", "second"] {
        let evidence = format!(
            "provider_request_collection_contract={PROVIDER_REQUEST_COLLECTION_CONTRACT};provider_request_count=2;{};{};{}",
            indexed_request(0, "first"),
            indexed_request(1, "second"),
            dependency(0, producer)
        );
        assert!(provider_request_collection_from_evidence(&evidence).is_none());
    }
}

#[test]
fn rejects_duplicate_dependency_edge() {
    let duplicate = dependency(1, "first")
        .replace("dependency_count=1", "dependency_count=2")
        + ";provider_request_1_dependency_1_producer_request_id=first;provider_request_1_dependency_1_producer_output_buffer=output.pixels;provider_request_1_dependency_1_consumer_input_buffer=input.pixels";
    let evidence = format!(
        "provider_request_collection_contract={PROVIDER_REQUEST_COLLECTION_CONTRACT};provider_request_count=2;{};{};{duplicate}",
        indexed_request(0, "first"),
        indexed_request(1, "second")
    );
    assert!(provider_request_collection_from_evidence(&evidence).is_none());
}

#[test]
fn rejects_cyclic_dependency_graph() {
    let evidence = format!(
        "provider_request_collection_contract={PROVIDER_REQUEST_COLLECTION_CONTRACT};provider_request_count=2;{};{};{};{}",
        indexed_request(0, "first"),
        indexed_request(1, "second"),
        dependency(0, "second"),
        dependency(1, "first")
    );
    assert!(provider_request_collection_from_evidence(&evidence).is_none());
}

fn fan_in_bindings(second_name: &str) -> String {
    format!(
        "provider_request_1_input_binding_contract={};provider_request_1_input_binding_count=2;provider_request_1_input_binding_0_name=input.pixels;provider_request_1_input_binding_0_source=artifact;provider_request_1_input_binding_0_element_type=u8;provider_request_1_input_binding_0_layout=image-2d-row-major:pixel-format=gray8;provider_request_1_input_binding_0_shape=2x2;provider_request_1_input_binding_0_row_stride_bytes=2;provider_request_1_input_binding_0_byte_length=4;provider_request_1_input_binding_0_content_hash=0x1234;provider_request_1_input_binding_0_payload_path=pixels.bin;provider_request_1_input_binding_0_producer_request_id=none;provider_request_1_input_binding_0_producer_output_buffer=none;provider_request_1_input_binding_1_name={second_name};provider_request_1_input_binding_1_source=dependency;provider_request_1_input_binding_1_element_type=u8;provider_request_1_input_binding_1_layout=image-2d-row-major:pixel-format=gray8;provider_request_1_input_binding_1_shape=2x2;provider_request_1_input_binding_1_row_stride_bytes=2;provider_request_1_input_binding_1_byte_length=4;provider_request_1_input_binding_1_content_hash=0xabcd;provider_request_1_input_binding_1_payload_path=none;provider_request_1_input_binding_1_producer_request_id=first;provider_request_1_input_binding_1_producer_output_buffer=output.pixels",
        crate::provider_input_binding::PROVIDER_INPUT_BINDING_CONTRACT
    )
}

#[test]
fn parses_named_multi_input_fan_in_bindings() {
    let second = indexed_request(1, "second").replace(
        "provider_request_1_kernel_input_buffer=input.pixels",
        "provider_request_1_kernel_input_buffer=input.pixels;provider_request_1_kernel_input_buffers=input.pixels,input.aux",
    );
    let edge = dependency(1, "first").replace(
        "consumer_input_buffer=input.pixels",
        "consumer_input_buffer=input.aux",
    );
    let evidence = format!(
        "provider_request_collection_contract={PROVIDER_REQUEST_COLLECTION_CONTRACT};provider_request_count=2;{};{second};{edge};{}",
        indexed_request(0, "first"),
        fan_in_bindings("input.aux")
    );
    let collection = provider_request_collection_from_evidence(&evidence).expect("fan-in");
    assert_eq!(collection.requests[1].kernel.input_buffers.len(), 2);
    assert_eq!(collection.requests[1].input_bindings.len(), 2);
    assert_eq!(
        collection.requests[1].input_bindings[1].layout,
        "image-2d-row-major:pixel-format=gray8"
    );
    assert_eq!(collection.requests[1].input_bindings[1].row_stride_bytes, 2);
    assert_eq!(
        collection.requests[1].input_bindings[1].source,
        "dependency"
    );
}

#[test]
fn rejects_typed_fan_in_binding_with_short_row_stride() {
    let second = indexed_request(1, "second").replace(
        "provider_request_1_kernel_input_buffer=input.pixels",
        "provider_request_1_kernel_input_buffer=input.pixels;provider_request_1_kernel_input_buffers=input.pixels,input.aux",
    );
    let edge = dependency(1, "first").replace(
        "consumer_input_buffer=input.pixels",
        "consumer_input_buffer=input.aux",
    );
    let bindings = fan_in_bindings("input.aux").replace(
        "provider_request_1_input_binding_1_row_stride_bytes=2",
        "provider_request_1_input_binding_1_row_stride_bytes=1",
    );
    let evidence = format!(
        "provider_request_collection_contract={PROVIDER_REQUEST_COLLECTION_CONTRACT};provider_request_count=2;{};{second};{edge};{bindings}",
        indexed_request(0, "first"),
    );

    assert!(provider_request_collection_from_evidence(&evidence).is_none());
}

#[test]
fn rejects_duplicate_named_input_binding() {
    let second = indexed_request(1, "second").replace(
        "provider_request_1_kernel_input_buffer=input.pixels",
        "provider_request_1_kernel_input_buffer=input.pixels;provider_request_1_kernel_input_buffers=input.pixels,input.aux",
    );
    let edge = dependency(1, "first").replace(
        "consumer_input_buffer=input.pixels",
        "consumer_input_buffer=input.aux",
    );
    let evidence = format!(
        "provider_request_collection_contract={PROVIDER_REQUEST_COLLECTION_CONTRACT};provider_request_count=2;{};{second};{edge};{}",
        indexed_request(0, "first"),
        fan_in_bindings("input.pixels")
    );
    assert!(provider_request_collection_from_evidence(&evidence).is_none());
}

#[test]
fn collection_dependency_selects_additional_output_semantics() {
    let producer = indexed_request(0, "producer").replace(
        "provider_request_0_kernel_output_buffer=output.pixels",
        "provider_request_0_kernel_output_buffer=output.primary;provider_request_0_output_binding_contract=nuis-provider-output-binding-v1;provider_request_0_output_binding_count=2;provider_request_0_output_binding_0_role=output.primary;provider_request_0_output_binding_0_buffer=output.primary;provider_request_0_output_binding_0_element_type=u8;provider_request_0_output_binding_0_shape=2x2;provider_request_0_output_binding_0_byte_length=4;provider_request_0_output_binding_0_comparison_id=none;provider_request_0_output_binding_1_role=output.audit;provider_request_0_output_binding_1_buffer=output.audit;provider_request_0_output_binding_1_element_type=u64;provider_request_0_output_binding_1_shape=3;provider_request_0_output_binding_1_byte_length=24;provider_request_0_output_binding_1_comparison_id=none",
    );
    let consumer = indexed_request(1, "consumer").replace(
        "provider_request_1_kernel_input_buffer=input.pixels",
        "provider_request_1_kernel_input_buffer=input.pixels;provider_request_1_kernel_input_buffers=input.pixels,input.audit",
    );
    let dependency = "provider_request_1_dependency_contract=nuis-provider-request-dependency-v1;provider_request_1_dependency_count=1;provider_request_1_dependency_0_producer_request_id=producer;provider_request_1_dependency_0_producer_output_buffer=output.audit;provider_request_1_dependency_0_consumer_input_buffer=input.audit";
    let bindings = "provider_request_1_input_binding_contract=nuis-provider-input-binding-v1;provider_request_1_input_binding_count=2;provider_request_1_input_binding_0_name=input.pixels;provider_request_1_input_binding_0_source=artifact;provider_request_1_input_binding_0_element_type=u8;provider_request_1_input_binding_0_shape=2x2;provider_request_1_input_binding_0_byte_length=4;provider_request_1_input_binding_0_content_hash=0x1234;provider_request_1_input_binding_0_payload_path=pixels.bin;provider_request_1_input_binding_0_producer_request_id=none;provider_request_1_input_binding_0_producer_output_buffer=none;provider_request_1_input_binding_1_name=input.audit;provider_request_1_input_binding_1_source=dependency;provider_request_1_input_binding_1_element_type=u64;provider_request_1_input_binding_1_shape=3;provider_request_1_input_binding_1_byte_length=24;provider_request_1_input_binding_1_content_hash=0xaudit;provider_request_1_input_binding_1_payload_path=none;provider_request_1_input_binding_1_producer_request_id=producer;provider_request_1_input_binding_1_producer_output_buffer=output.audit";
    let evidence = format!(
        "provider_request_collection_contract={PROVIDER_REQUEST_COLLECTION_CONTRACT};provider_request_count=2;{producer};{consumer};{dependency};{bindings}"
    );
    let collection =
        provider_request_collection_from_evidence(&evidence).expect("additional output dependency");
    let dependency_binding = &collection.requests[1].input_bindings[1];
    assert_eq!(dependency_binding.producer_output_buffer, "output.audit");
    assert_eq!(dependency_binding.element_type, "u64");
    assert_eq!(dependency_binding.shape, [3]);
    assert_eq!(dependency_binding.byte_length, 24);
}
