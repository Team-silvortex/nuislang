#![cfg(unix)]

use crate::{
    provider_bundle_registry::provider_bundle_registrations,
    provider_sample_execute::execute_provider_samples, provider_sample_payload::fnv1a64_hex,
};
use std::{
    env, fs,
    sync::atomic::{AtomicU64, Ordering},
};

static TEST_NONCE: AtomicU64 = AtomicU64::new(0);

#[test]
fn rejects_declared_provider_request_that_fails_validation() {
    let nonce = TEST_NONCE.fetch_add(1, Ordering::Relaxed);
    let output_dir = env::temp_dir().join(format!(
        "nsdb-provider-invalid-request-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let evidence = "provider_buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1;provider_buffer_id=input.bytes;provider_buffer_element_type=u8;provider_buffer_layout=tensor-contiguous;provider_buffer_shape=4;provider_buffer_row_stride_bytes=1;provider_buffer_byte_length=4;provider_buffer_payload_path=input.bin;provider_buffer_content_hash=0x1234;provider_kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1;provider_kernel_id=invalid.copy;provider_kernel_operation=copy;provider_kernel_input_buffer=input.bytes;provider_kernel_output_buffer=output.bytes;provider_kernel_dispatch=4x1x1";
    let manifest = format!(
        r#"protocol = "nuis-device-provider-samples-v1"
schema = "nsdb-yir-device-provider-sample-v1"
source = "provider-invalid-request-test"
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
        &manifest,
    )
    .unwrap();

    let error = match execute_provider_samples(&output_dir, Some("data:host")) {
        Ok(_) => panic!("invalid declared request must fail closed"),
        Err(error) => error,
    };
    assert!(error.contains("declares a request contract but failed validation"));
    assert!(!output_dir
        .join("nuis.nsdb.provider-output.data-host.toml")
        .exists());
    fs::remove_dir_all(output_dir).unwrap();
}

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
        &manifest,
    )
    .unwrap();

    let report = execute_provider_samples(&output_dir, Some("data:host")).unwrap();
    let payload =
        fs::read_to_string(output_dir.join("nuis.nsdb.provider-output.data-host.toml")).unwrap();
    let bundle_count = provider_bundle_registrations().len();

    assert_eq!(report.status, "provider-output-payloads-ready");
    assert_eq!(
        report.final_image_dispatch_authority_status,
        "pre-seal-acquisition"
    );
    assert_eq!(report.final_image_dispatch_count, 0);
    assert_eq!(report.output_payload_count, 1);
    assert_eq!(
        report.provider_bundle_registry_contract,
        "nuis-provider-bundle-registry-v1"
    );
    assert_eq!(
        report.provider_bundle_manifest_contract,
        "nuis-provider-bundle-manifest-v1"
    );
    assert_eq!(report.provider_bundle_manifest_entry_count, bundle_count);
    assert_eq!(report.first_provider_bundle_package_id, "official.data");
    assert_eq!(report.first_provider_bundle_id, "data.host.bundle.v1");
    assert_eq!(
        report.selected_provider_bundle_set_contract,
        "nuis-selected-provider-bundle-set-v1"
    );
    assert_eq!(report.selected_provider_bundle_count, 1);
    assert_eq!(
        report.selected_provider_bundle_set_hash,
        "fnv1a64:d59cbb3377c76e54"
    );
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
    assert!(payload
        .contains("provider_bundle_registry_contract = \"nuis-provider-bundle-registry-v1\""));
    assert!(payload
        .contains("provider_bundle_manifest_contract = \"nuis-provider-bundle-manifest-v1\""));
    assert!(payload.contains(&format!(
        "provider_bundle_manifest_entry_count = {bundle_count}"
    )));
    assert!(payload.contains("provider_bundle_package_id = \"official.data\""));
    assert!(payload.contains("provider_bundle_id = \"data.host.bundle.v1\""));
    assert!(payload.contains(
        "provider_capability_registry_contract = \"nuis-provider-capability-registry-v1\""
    ));
    assert!(payload.contains(
        "provider_capability_manifest_contract = \"nuis-provider-capability-manifest-v1\""
    ));
    assert!(payload.contains("provider_capability_manifest_hash = \"fnv1a64:4e27319a33087b95\""));
    assert!(payload.contains("provider_capability_manifest_entry_count = 1"));
    assert!(payload
        .contains("provider_capability_record_contract = \"nuis-provider-capability-record-v1\""));
    assert!(payload.contains("provider_capability_provider_id = \"data.cpu-memory.reference.v1\""));
    assert!(payload.contains("provider_capability_priority = 100"));
    assert!(payload.contains(
        "provider_capability_values = \"clock.fabric-monotonic,completion.verified,execution.reference,glm.owned-transfer,memory.cpu,movement.copy,residency.host\""
    ));
    assert!(payload.contains(
        "provider_capability_availability_contract = \"nuis-provider-capability-availability-v1\""
    ));
    assert!(
        payload.contains("provider_capability_probe_status = \"native-provider-worker-available\"")
    );
    assert!(payload.contains("provider_capability_availability_status = \"available\""));
    assert!(payload.contains("provider_capability_selection_hash = \"fnv1a64:01cb51b49b12a49e\""));
    assert!(payload.contains(
        "provider_conformance_capsule_contract = \"nuis-provider-conformance-capsule-v1\""
    ));
    assert!(payload.contains(
        "provider_conformance_scenario_contract = \"nuis-data-reference-copy-conformance-v1\""
    ));
    assert!(payload
        .contains("provider_conformance_capability_selection_hash = \"fnv1a64:6d712122a1132927\""));
    assert!(payload
        .contains("provider_conformance_expected_output_hash = \"fnv1a64:1c3b67c65206fb6d\""));
    assert!(payload.contains("provider_conformance_physical_execution_claimed = false"));
    assert!(payload.contains("provider_conformance_capsule_hash = \"fnv1a64:82270e31b99f2c0b\""));
    assert!(payload.contains(
        "provider_conformance_replay_contract = \"nuis-provider-conformance-replay-v1\""
    ));
    assert!(payload.contains("provider_conformance_replay_submission_tick = 1"));
    assert!(payload.contains("provider_conformance_replay_completion_tick = 2"));
    assert!(payload.contains("provider_conformance_replay_release_tick = 3"));
    assert!(
        payload.contains("provider_conformance_replay_execution_authority = \"conformance-only\"")
    );
    assert!(payload.contains("provider_conformance_replay_physical_execution_claimed = false"));
    assert!(payload.contains("provider_conformance_replay_hash = \"fnv1a64:7ee93c8f8a4ae011\""));
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
        .contains("native_output_output_binding_contract = \"nuis-provider-output-binding-v2\""));
    assert!(payload.contains("native_output_output_binding_count = \"2\""));
    assert!(
        payload.contains("native_output_output_binding_roles = \"output.primary,output.audit\"")
    );
    assert!(
        payload.contains("native_output_output_binding_buffers = \"output.primary,output.audit\"")
    );
    assert!(payload.contains("native_output_output_binding_element_types = \"u64,u64\""));
    assert!(payload.contains(
        "native_output_output_binding_layouts = \"tensor-contiguous,tensor-contiguous\""
    ));
    assert!(payload.contains("native_output_output_binding_shapes = \"3,3\""));
    assert!(payload.contains("native_output_output_binding_row_stride_bytes = \"24,24\""));
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
    assert!(payload.contains(
        "native_output_completion_evidence_contract = \"nuis-provider-completion-evidence-v1\""
    ));
    assert!(payload.contains(
        "native_output_completion_clock_evidence = \"nuis-provider-completion-clock-v1:"
    ));
    assert!(payload.contains("native_output_completion_status = \"worker-output-verified\""));
    assert!(payload.contains(
        "native_output_glm_release_contract = \"nuis-provider-glm-release-evidence-v1\""
    ));
    assert!(payload.contains("native_output_glm_release_status = \"released-at-graph-close\""));
    assert!(
        payload
            .contains("native_output_graph_output_release_roles = \"output.audit,output.primary\"")
            || payload.contains(
                "native_output_graph_output_release_roles = \"output.primary,output.audit\""
            )
    );
    let completion = crate::provider_completion_evidence::from_output_payload(
        &output_dir,
        &format!(
            "nuis.nsdb.provider-output.data-host.toml:hash={}:status=written",
            fnv1a64_hex(payload.as_bytes())
        ),
    )
    .expect("verified completion evidence");
    assert_eq!(completion.count, 1);
    assert_eq!(completion.status, "verified");
    assert!(completion
        .completion_tokens
        .starts_with("provider-completion:0x"));
    assert!(completion.glm_release_tokens.starts_with("glm-release:0x"));
    let tampered = payload
        .lines()
        .map(|line| {
            if line.starts_with("native_output_0_completion_token = ") {
                "native_output_0_completion_token = \"provider-completion:0x0000000000000000\""
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(
        output_dir.join("nuis.nsdb.provider-output.data-host.toml"),
        &tampered,
    )
    .unwrap();
    let tampered_evidence = format!(
        "nuis.nsdb.provider-output.data-host.toml:hash={}:status=written",
        fnv1a64_hex(tampered.as_bytes())
    );
    assert!(crate::provider_completion_evidence::from_output_payload(
        &output_dir,
        &tampered_evidence
    )
    .unwrap_err()
    .contains("completion token mismatch"));

    fs::remove_dir_all(output_dir).unwrap();
}

#[cfg(target_os = "linux")]
#[test]
fn executes_registered_cuda_ptx_through_persistent_worker() {
    if (crate::provider_runner_cuda::RUNNER_PROFILE.probe_status)()
        != crate::provider_runner_cuda::RUNNER_PROFILE.available_probe_status
    {
        return;
    }
    let nonce = TEST_NONCE.fetch_add(1, Ordering::Relaxed);
    let output_dir = env::temp_dir().join(format!(
        "nsdb-provider-cuda-frontdoor-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let left = [
        0x00, 0x00, 0x80, 0x3f, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x40, 0x40, 0x00, 0x00, 0x80,
        0x40,
    ];
    let right = [
        0x00, 0x00, 0x20, 0x41, 0x00, 0x00, 0xa0, 0x41, 0x00, 0x00, 0xf0, 0x41, 0x00, 0x00, 0x20,
        0x42,
    ];
    let expected = [
        0x00, 0x00, 0x30, 0x41, 0x00, 0x00, 0xb0, 0x41, 0x00, 0x00, 0x04, 0x42, 0x00, 0x00, 0x30,
        0x42,
    ];
    let asset = nuisc::kernel_code_asset::select_kernel_code_asset("cuda.nvidia-gpu").unwrap();
    fs::write(output_dir.join("left.f32.bin"), left).unwrap();
    fs::write(output_dir.join("right.f32.bin"), right).unwrap();
    fs::write(output_dir.join("expected.f32.bin"), expected).unwrap();
    fs::write(output_dir.join(asset.file_name), asset.bytes).unwrap();
    let evidence = format!(
        "provider_buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1;\
provider_buffer_id=input.left;\
provider_buffer_element_type=f32;\
provider_buffer_layout=tensor-contiguous;\
provider_buffer_shape=4;\
provider_buffer_row_stride_bytes=16;\
provider_buffer_byte_length=16;\
provider_buffer_payload_path=left.f32.bin;\
provider_buffer_content_hash={};\
provider_kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1;\
provider_kernel_id=kernel.cuda.vector-add.f32;\
provider_kernel_operation=vector-add;\
provider_kernel_input_buffer=input.left;\
provider_kernel_input_buffers=input.left,input.right;\
provider_kernel_output_buffer=output.values;\
provider_kernel_dispatch=4x1x1;\
provider_kernel_scalar_bindings=element_count:u32:4,device_selection_policy:u32:1,minimum_compute_capability:u32:80;\
provider_code_asset_descriptor_contract=nuis-provider-code-asset-descriptor-v1;\
provider_code_asset_id={};\
provider_code_asset_format={};\
provider_code_asset_target={};\
provider_code_asset_entry={};\
provider_code_asset_path={};\
provider_code_asset_byte_length={};\
provider_code_asset_digest_contract={};\
provider_code_asset_content_hash={};\
provider_output_binding_contract=nuis-provider-output-binding-v1;\
provider_output_binding_count=1;\
provider_output_binding_0_role=output.result;\
provider_output_binding_0_buffer=output.values;\
provider_output_binding_0_element_type=f32;\
provider_output_binding_0_shape=4;\
provider_output_binding_0_byte_length=16;\
provider_output_binding_0_comparison_id=comparison.output;\
provider_output_comparison_id=comparison.output;\
provider_output_comparison_descriptor_contract=nuis-provider-output-comparison-descriptor-v1;\
provider_output_comparison_output_buffer=output.values;\
provider_output_comparison_element_type=f32;\
provider_output_comparison_shape=4;\
provider_output_comparison_expected_path=expected.f32.bin;\
provider_output_comparison_expected_byte_length=16;\
provider_output_comparison_expected_content_hash={};\
provider_output_comparison_absolute_tolerance=0;\
provider_output_comparison_relative_tolerance=0;\
provider_output_comparison_non_finite_policy=reject;\
provider_input_binding_contract=nuis-provider-input-binding-v1;\
provider_input_binding_count=2;\
provider_input_binding_0_name=input.left;\
provider_input_binding_0_source=artifact;\
provider_input_binding_0_element_type=f32;\
provider_input_binding_0_shape=4;\
provider_input_binding_0_byte_length=16;\
provider_input_binding_0_content_hash={};\
provider_input_binding_0_payload_path=left.f32.bin;\
provider_input_binding_0_producer_request_id=none;\
provider_input_binding_0_producer_output_buffer=none;\
provider_input_binding_1_name=input.right;\
provider_input_binding_1_source=artifact;\
provider_input_binding_1_element_type=f32;\
provider_input_binding_1_shape=4;\
provider_input_binding_1_byte_length=16;\
provider_input_binding_1_content_hash={};\
provider_input_binding_1_payload_path=right.f32.bin;\
provider_input_binding_1_producer_request_id=none;\
provider_input_binding_1_producer_output_buffer=none;\
provider_adapter_binding_contract=nuis-provider-request-adapter-binding-v1;\
provider_adapter_binding_provider_family=cuda:nvidia-gpu;\
provider_adapter_binding_execution_requirement=real-device",
        fnv1a64_hex(&left),
        asset.id,
        asset.format,
        asset.target,
        asset.entry,
        asset.file_name,
        asset.bytes.len(),
        asset.digest_contract,
        fnv1a64_hex(asset.bytes),
        fnv1a64_hex(&expected),
        fnv1a64_hex(&left),
        fnv1a64_hex(&right),
    );
    let manifest = format!(
        r#"protocol = "nuis-device-provider-samples-v1"
schema = "nsdb-yir-device-provider-sample-v1"
source = "provider-cuda-frontdoor-test"
status = "ready"
record_count = 1
ready_record_count = 1
pending_record_count = 0

[[device_provider_samples]]
trace_id = "hetero-trace:kernel:cuda:nvidia-gpu"
provider = "kernel-cuda-worker-test"
provider_family = "cuda:nvidia-gpu"
provider_runner_adapter_id = "cuda.nvidia-gpu.real-device"
input_evidence = "{evidence}"
materialization_status = "provider-sample-materialized"
"#
    );
    fs::write(
        output_dir.join("nuis.nsdb.device-provider-samples.toml"),
        manifest,
    )
    .unwrap();

    let report = execute_provider_samples(&output_dir, Some("cuda:nvidia-gpu")).unwrap();
    let payload =
        fs::read_to_string(output_dir.join("nuis.nsdb.provider-output.cuda-nvidia-gpu.toml"))
            .unwrap();
    assert_eq!(report.output_payload_count, 1);
    assert_eq!(
        report.first_output_payload_native_execution_contract,
        "nuis-cuda-ptx-driver-provider-execution-v1"
    );
    assert_eq!(
        report.first_output_payload_native_execution_status,
        "cuda-driver-kernel-completed"
    );
    assert_eq!(
        report.first_output_payload_native_output_hash,
        fnv1a64_hex(&expected)
    );
    assert!(payload.contains(
        "native_output_worker_execution_capsule_invocation_mode = \"nuis-provider-worker-process-adapter-v5\""
    ));
    assert!(payload.contains("native_output_comparison_status = \"comparison-passed\""));
    assert!(payload.contains("native_output_graph_output_release_count = \"1\""));
    assert!(payload.contains(
        "native_output_completion_evidence_contract = \"nuis-provider-completion-evidence-v1\""
    ));
    assert!(payload.contains("native_output_completion_token = \"provider-completion:0x"));
    assert!(payload.contains(
        "native_output_glm_release_contract = \"nuis-provider-glm-release-evidence-v1\""
    ));
    assert!(payload.contains("native_output_glm_release_token = \"glm-release:0x"));
    let completion = crate::provider_completion_evidence::from_output_payload(
        &output_dir,
        &format!(
            "nuis.nsdb.provider-output.cuda-nvidia-gpu.toml:hash={}:status=written",
            fnv1a64_hex(payload.as_bytes())
        ),
    )
    .expect("verified CUDA completion evidence");
    assert_eq!(completion.count, 1);
    assert_eq!(completion.status, "verified");
    assert!(completion
        .clock_evidence
        .starts_with("nuis-provider-completion-clock-v1:"));

    fs::remove_dir_all(output_dir).unwrap();
}
