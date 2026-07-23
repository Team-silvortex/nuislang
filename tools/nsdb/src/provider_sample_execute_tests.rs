#![cfg(unix)]

use crate::{
    provider_sample_execute::execute_provider_samples, provider_sample_payload::fnv1a64_hex,
};
use std::{
    env, fs,
    sync::atomic::{AtomicU64, Ordering},
};

static TEST_NONCE: AtomicU64 = AtomicU64::new(0);

#[test]
fn executes_registered_native_worker_with_two_graph_outputs() {
    let nonce = TEST_NONCE.fetch_add(1, Ordering::Relaxed);
    let output_dir = env::temp_dir().join(format!(
        "nsdb-provider-native-frontdoor-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let input = [1_u8, 2, 3, 4];
    fs::write(output_dir.join("input.bin"), input).unwrap();
    let input_hash = fnv1a64_hex(&input);
    let base_evidence = format!(
        "provider_buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1;\
provider_buffer_id=input.bytes;\
provider_buffer_element_type=u8;\
provider_buffer_layout=tensor-contiguous;\
provider_buffer_shape=4;\
provider_buffer_row_stride_bytes=4;\
provider_buffer_byte_length=4;\
provider_buffer_payload_path=input.bin;\
provider_buffer_content_hash={input_hash};\
provider_kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1;\
provider_kernel_id=provider.fan-out;\
provider_kernel_operation=fan-out;\
provider_kernel_input_buffer=input.bytes;\
provider_kernel_output_buffer=output.primary;\
provider_kernel_dispatch=4x1x1;\
provider_output_binding_contract=nuis-provider-output-binding-v1;\
provider_output_binding_count=2;\
provider_output_binding_0_role=output.primary;\
provider_output_binding_0_buffer=output.primary;\
provider_output_binding_0_element_type=u64;\
provider_output_binding_0_shape=3;\
provider_output_binding_0_byte_length=24;\
provider_output_binding_0_comparison_id=none;\
provider_output_binding_1_role=output.audit;\
provider_output_binding_1_buffer=output.audit;\
provider_output_binding_1_element_type=u64;\
provider_output_binding_1_shape=3;\
provider_output_binding_1_byte_length=24;\
provider_output_binding_1_comparison_id=none"
    );
    let primary_expected = (1_u8..=24).collect::<Vec<_>>();
    let audit_expected = (31_u8..=54).collect::<Vec<_>>();
    fs::write(output_dir.join("expected-primary.bin"), &primary_expected).unwrap();
    fs::write(output_dir.join("expected-audit.bin"), &audit_expected).unwrap();
    let evidence = format!(
        "{};provider_output_comparison_collection_contract=nuis-provider-output-comparison-collection-v1;provider_output_comparison_collection_count=2;provider_output_comparison_item_0_id=comparison.primary;provider_output_comparison_item_0_descriptor_contract=nuis-provider-output-comparison-descriptor-v1;provider_output_comparison_item_0_output_buffer=output.primary;provider_output_comparison_item_0_element_type=u64;provider_output_comparison_item_0_shape=3;provider_output_comparison_item_0_expected_path=expected-primary.bin;provider_output_comparison_item_0_expected_byte_length=24;provider_output_comparison_item_0_expected_content_hash={};provider_output_comparison_item_0_absolute_tolerance=0;provider_output_comparison_item_0_relative_tolerance=0;provider_output_comparison_item_0_non_finite_policy=reject;provider_output_comparison_item_1_id=comparison.audit;provider_output_comparison_item_1_descriptor_contract=nuis-provider-output-comparison-descriptor-v1;provider_output_comparison_item_1_output_buffer=output.audit;provider_output_comparison_item_1_element_type=u64;provider_output_comparison_item_1_shape=3;provider_output_comparison_item_1_expected_path=expected-audit.bin;provider_output_comparison_item_1_expected_byte_length=24;provider_output_comparison_item_1_expected_content_hash={};provider_output_comparison_item_1_absolute_tolerance=0;provider_output_comparison_item_1_relative_tolerance=0;provider_output_comparison_item_1_non_finite_policy=reject",
        base_evidence
            .replace(
                "provider_output_binding_0_comparison_id=none",
                "provider_output_binding_0_comparison_id=comparison.primary"
            )
            .replace(
                "provider_output_binding_1_comparison_id=none",
                "provider_output_binding_1_comparison_id=comparison.audit"
            ),
        fnv1a64_hex(&primary_expected),
        fnv1a64_hex(&audit_expected),
    );
    let manifest = format!(
        r#"protocol = "nuis-device-provider-samples-v1"
schema = "nsdb-yir-device-provider-sample-v1"
source = "provider-native-frontdoor-test"
status = "ready"
record_count = 1
ready_record_count = 1
pending_record_count = 0

[[device_provider_samples]]
trace_id = "hetero-trace:data:host"
provider = "native-worker-test"
provider_family = "data:host"
input_evidence = "{evidence}"
materialization_status = "provider-sample-materialized"
"#
    );
    fs::write(
        output_dir.join("nuis.nsdb.device-provider-samples.toml"),
        manifest,
    )
    .unwrap();

    let report = execute_provider_samples(&output_dir, Some("data:host")).unwrap();
    let payload =
        fs::read_to_string(output_dir.join("nuis.nsdb.provider-output.data-host.toml")).unwrap();

    assert_eq!(report.status, "provider-output-payloads-ready");
    assert_eq!(report.output_payload_count, 1);
    assert_eq!(
        report.first_provider_runner_adapter_id,
        "data.host.provider-worker-native"
    );
    assert_eq!(
        report.first_provider_runner_real_device_probe_status,
        "native-provider-worker-available"
    );
    assert_eq!(
        report.first_output_payload_native_execution_contract,
        "nuis-provider-worker-native-execution-v1"
    );
    assert!(payload.contains(
        "native_output_worker_execution_capsule_invocation_mode = \"nuis-provider-worker-process-adapter-v5\""
    ));
    assert!(payload.contains(
        "native_output_worker_adapter_cache_contract = \"nuis-provider-process-adapter-cache-v1\""
    ));
    assert!(payload.contains("native_output_worker_adapter_cache_status = \"compiled\""));
    assert!(payload.contains("native_output_worker_output_descriptor_count = \"2\""));
    assert!(payload.contains(
        "native_output_worker_output_descriptor_roles = \"output.primary,output.audit\""
    ));
    assert!(payload
        .contains("native_output_output_binding_contract = \"nuis-provider-output-binding-v1\""));
    assert!(payload.contains("native_output_output_binding_count = \"2\""));
    assert!(
        payload.contains("native_output_output_binding_roles = \"output.primary,output.audit\"")
    );
    assert!(
        payload.contains("native_output_output_binding_buffers = \"output.primary,output.audit\"")
    );
    assert!(payload.contains("native_output_output_binding_element_types = \"u64,u64\""));
    assert!(payload.contains("native_output_output_binding_shapes = \"3,3\""));
    assert!(payload.contains("native_output_output_binding_byte_lengths = \"24,24\""));
    assert!(payload.contains(
        "native_output_output_binding_comparison_ids = \"comparison.primary,comparison.audit\""
    ));
    assert!(payload.contains(
        "native_output_comparison_collection_contract = \"nuis-provider-output-comparison-collection-result-v1\""
    ));
    assert!(payload.contains("native_output_comparison_collection_count = \"2\""));
    assert!(payload.contains(
        "native_output_comparison_collection_ids = \"comparison.primary,comparison.audit\""
    ));
    assert!(payload.contains(
        "native_output_comparison_collection_output_buffers = \"output.primary,output.audit\""
    ));
    assert!(payload.contains(
        "native_output_comparison_collection_statuses = \"comparison-passed,comparison-passed\""
    ));
    assert!(payload.contains("native_output_comparison_collection_element_counts = \"3,3\""));
    assert!(payload.contains("native_output_comparison_collection_mismatch_counts = \"0,0\""));
    assert!(payload.contains(
        "native_output_worker_additional_output_retention_statuses = \"transferable-carrier\""
    ));
    assert!(payload.contains(
        "native_output_graph_output_ownership_contract = \"nuis-provider-graph-output-ownership-v1\""
    ));
    assert!(payload.contains("native_output_graph_output_release_count = \"2\""));
    assert!(
        payload
            .contains("native_output_graph_output_release_roles = \"output.audit,output.primary\"")
            || payload.contains(
                "native_output_graph_output_release_roles = \"output.primary,output.audit\""
            )
    );

    fs::remove_dir_all(output_dir).unwrap();
}
