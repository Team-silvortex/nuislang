use super::*;

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

#[test]
fn parses_registered_buffer_and_kernel_descriptors() {
    let request = provider_request_from_evidence(REGISTERED).expect("registered request");
    assert_eq!(request.source, "registered-descriptors");
    assert_eq!(request.buffer.shape, [2, 2]);
    assert_eq!(request.kernel.dispatch, [2, 2, 1]);
    assert_eq!(request.scalar_u8("max_value"), Some(15));
}

#[test]
fn parses_ordered_multi_output_bindings_with_compatibility_primary() {
    let evidence = format!(
        "{REGISTERED};provider_output_binding_contract={PROVIDER_OUTPUT_BINDING_CONTRACT};provider_output_binding_count=2;provider_output_binding_0_role=output.primary;provider_output_binding_0_buffer=output.pixels;provider_output_binding_0_element_type=u8;provider_output_binding_0_shape=2x2;provider_output_binding_0_byte_length=4;provider_output_binding_0_comparison_id=none;provider_output_binding_1_role=output.audit;provider_output_binding_1_buffer=output.audit;provider_output_binding_1_element_type=u64;provider_output_binding_1_shape=3;provider_output_binding_1_byte_length=24;provider_output_binding_1_comparison_id=none"
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
    assert_eq!(request.output_bindings[0].shape, [2, 2]);
    assert_eq!(request.output_bindings[0].byte_length, 4);
    assert_eq!(request.output_bindings[1].element_type, "u64");
    assert_eq!(request.output_bindings[1].shape, [3]);
    assert_eq!(request.output_bindings[1].byte_length, 24);
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
        "{REGISTERED};provider_output_binding_contract={PROVIDER_OUTPUT_BINDING_CONTRACT};provider_output_binding_count=2;provider_output_binding_0_role=output.primary;provider_output_binding_0_buffer=output.pixels;provider_output_binding_0_element_type=u64;provider_output_binding_0_shape=3;provider_output_binding_0_byte_length=24;provider_output_binding_0_comparison_id=comparison.primary;provider_output_binding_1_role=output.audit;provider_output_binding_1_buffer=output.audit;provider_output_binding_1_element_type=u64;provider_output_binding_1_shape=3;provider_output_binding_1_byte_length=24;provider_output_binding_1_comparison_id=comparison.audit;provider_output_comparison_collection_contract={PROVIDER_OUTPUT_COMPARISON_COLLECTION_CONTRACT};provider_output_comparison_collection_count=2;provider_output_comparison_item_0_id=comparison.primary;provider_output_comparison_item_0_descriptor_contract={PROVIDER_OUTPUT_COMPARISON_DESCRIPTOR_CONTRACT};provider_output_comparison_item_0_output_buffer=output.pixels;provider_output_comparison_item_0_element_type=u64;provider_output_comparison_item_0_shape=3;provider_output_comparison_item_0_expected_path=primary.bin;provider_output_comparison_item_0_expected_byte_length=24;provider_output_comparison_item_0_expected_content_hash=0xprimary;provider_output_comparison_item_0_absolute_tolerance=0;provider_output_comparison_item_0_relative_tolerance=0;provider_output_comparison_item_0_non_finite_policy=reject;provider_output_comparison_item_1_id=comparison.audit;provider_output_comparison_item_1_descriptor_contract={PROVIDER_OUTPUT_COMPARISON_DESCRIPTOR_CONTRACT};provider_output_comparison_item_1_output_buffer=output.audit;provider_output_comparison_item_1_element_type=u64;provider_output_comparison_item_1_shape=3;provider_output_comparison_item_1_expected_path=audit.bin;provider_output_comparison_item_1_expected_byte_length=24;provider_output_comparison_item_1_expected_content_hash=0xaudit;provider_output_comparison_item_1_absolute_tolerance=0;provider_output_comparison_item_1_relative_tolerance=0;provider_output_comparison_item_1_non_finite_policy=reject"
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
        "provider_request_1_input_binding_contract={};provider_request_1_input_binding_count=2;provider_request_1_input_binding_0_name=input.pixels;provider_request_1_input_binding_0_source=artifact;provider_request_1_input_binding_0_element_type=u8;provider_request_1_input_binding_0_shape=2x2;provider_request_1_input_binding_0_byte_length=4;provider_request_1_input_binding_0_content_hash=0x1234;provider_request_1_input_binding_0_payload_path=pixels.bin;provider_request_1_input_binding_0_producer_request_id=none;provider_request_1_input_binding_0_producer_output_buffer=none;provider_request_1_input_binding_1_name={second_name};provider_request_1_input_binding_1_source=dependency;provider_request_1_input_binding_1_element_type=u8;provider_request_1_input_binding_1_shape=2x2;provider_request_1_input_binding_1_byte_length=4;provider_request_1_input_binding_1_content_hash=0xabcd;provider_request_1_input_binding_1_payload_path=none;provider_request_1_input_binding_1_producer_request_id=first;provider_request_1_input_binding_1_producer_output_buffer=output.pixels",
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
        collection.requests[1].input_bindings[1].source,
        "dependency"
    );
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
