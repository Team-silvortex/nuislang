use super::*;

pub(super) fn assert_provider_bundle_audit_evidence(source: &str, label: &str) {
    for evidence in [
        "\"artifact_device_provider_sample_manifest_provider_bundle_registry_contract\":\"nuis-provider-bundle-registry-v1\"",
        "\"artifact_device_provider_sample_manifest_provider_bundle_manifest_hash\":\"fnv1a64:9831a33035211556\"",
        "\"artifact_device_provider_sample_manifest_first_provider_bundle_package_id\":\"official.",
        "\"artifact_device_provider_sample_manifest_provider_bundle_evidence_status\":\"verified\"",
        "\"nsld_final_executable_output_object_package_provider_bundle_manifest_hash\":\"fnv1a64:9831a33035211556\"",
        "\"closure_summary_object_package_provider_bundle_evidence_status\":\"verified\"",
        "\"artifact_device_provider_sample_manifest_selected_provider_bundle_set_contract\":\"nuis-selected-provider-bundle-set-v1\"",
        "\"artifact_device_provider_sample_manifest_selected_provider_bundle_set_validation_status\":\"verified\"",
        "\"nsld_final_executable_output_object_package_selected_provider_bundle_set_hash\":\"fnv1a64:",
        "\"closure_summary_object_package_selected_provider_bundle_set_validation_status\":\"verified\"",
    ] {
        assert!(
            source.contains(evidence),
            "official galaxy provider bundle audit evidence missing `{evidence}`"
        );
    }
    let (count, hash) = match label {
        "pixelmagic_pipeline_demo" => (2, "fnv1a64:0126ed9d38f1895f"),
        "pixelmagic_threshold_provider_demo" => (1, "fnv1a64:5c7ac5158d84aa8b"),
        "witsage_kernel_demo" => (1, "fnv1a64:e9a82b052c861b93"),
        other => panic!("missing selected provider bundle expectation for `{other}`"),
    };
    for evidence in [
        format!(
            "\"artifact_device_provider_sample_manifest_selected_provider_bundle_count\":{count}"
        ),
        format!(
            "\"artifact_device_provider_sample_manifest_selected_provider_bundle_set_hash\":\"{hash}\""
        ),
        format!(
            "\"nsld_final_executable_output_object_package_selected_provider_bundle_count\":{count}"
        ),
        format!(
            "\"nsld_final_executable_output_object_package_selected_provider_bundle_set_hash\":\"{hash}\""
        ),
        format!(
            "\"closure_summary_object_package_selected_provider_bundle_count\":{count}"
        ),
        format!(
            "\"closure_summary_object_package_selected_provider_bundle_set_hash\":\"{hash}\""
        ),
    ] {
        assert!(
            source.contains(&evidence),
            "official galaxy selected provider bundle evidence missing `{evidence}`"
        );
    }
}

pub(super) fn assert_provider_execution_evidence(provider_output_payload_path: &Path) {
    for evidence in [
        "native_output_4_output_residency_contract = \"nuis-provider-output-residency-v1\"",
        "native_output_4_output_residency_kind = \"host-visible-file\"",
        "native_output_4_output_carrier_mode = \"inherited-fd-output\"",
        "native_output_5_output_residency_kind = \"host-visible-file\"",
        "native_output_5_output_carrier_mode = \"inherited-fd-output\"",
        "native_output_6_output_residency_kind = \"host-visible-file\"",
        "native_output_6_output_carrier_mode = \"inherited-fd-output\"",
        "native_output_1_output_residency_kind = \"host-visible-file\"",
        "native_output_1_output_carrier_mode = \"inherited-fd-output\"",
        "native_output_0_session_lease_contract = \"nuis-provider-session-lease-v1\"",
        "native_output_0_session_adapter_id = \"logical.request-process.v1\"",
        "native_output_0_session_lifecycle_hooks = \"graph-open,request-begin,request-complete,graph-close\"",
        "native_output_3_session_request_sequence = \"3\"",
        "native_output_4_session_request_sequence = \"0\"",
        "native_output_5_session_request_sequence = \"4\"",
        "native_output_6_session_request_sequence = \"1\"",
        "native_output_0_worker_lease_contract = \"nuis-provider-worker-lease-v1\"",
        "native_output_0_worker_resolver_contract = \"nuis-provider-worker-image-resolver-v1\"",
        "native_output_3_worker_request_sequence = \"3\"",
        "native_output_4_worker_request_sequence = \"0\"",
        "native_output_5_worker_request_sequence = \"4\"",
        "native_output_6_worker_request_sequence = \"1\"",
        "native_output_3_worker_descriptor_count = \"2\"",
        "native_output_4_worker_descriptor_count = \"1\"",
        "native_output_5_worker_descriptor_count = \"1\"",
        "native_output_6_worker_descriptor_count = \"1\"",
        "native_output_0_worker_descriptor_capability_contract = \"nuis-provider-worker-descriptor-capability-v1\"",
        "native_output_0_worker_max_semantic_descriptors = \"31\"",
        "native_output_0_worker_max_control_descriptors = \"1\"",
        "native_output_0_worker_output_descriptor_capability_contract = \"nuis-provider-worker-output-descriptor-capability-v1\"",
        "native_output_0_worker_max_output_descriptors = \"8\"",
        "native_output_0_worker_adapter_cache_contract = \"nuis-provider-process-adapter-cache-v1\"",
        "native_output_0_worker_adapter_cache_identity = \"adapter:0x",
        "native_output_0_worker_adapter_cache_status = \"compiled\"",
        "native_output_0_worker_adapter_control_contract = \"nuis-provider-worker-adapter-control-v2\"",
        "native_output_0_worker_adapter_control_mode = \"carrier\"",
        "native_output_1_worker_adapter_cache_status = \"hit\"",
        "native_output_2_worker_adapter_cache_status = \"hit\"",
        "native_output_3_worker_adapter_cache_status = \"hit\"",
        "native_output_4_worker_adapter_cache_status = \"compiled\"",
        "native_output_5_worker_adapter_cache_status = \"hit\"",
        "native_output_6_worker_adapter_cache_status = \"compiled\"",
        "native_output_0_worker_adapter_executable_hash = \"0x",
        "native_output_0_worker_pid = \"",
        "native_output_0_worker_payload_hash = \"0x",
        "native_output_0_worker_operation_token = \"operation:",
        "native_output_0_worker_execution_capsule_contract = \"nuis-provider-execution-capsule-v1\"",
        "native_output_0_worker_execution_capsule_id = \"capsule:",
        "native_output_0_worker_execution_capsule_token = \"capsule-token:",
        "native_output_0_worker_execution_capsule_invocation_mode = \"nuis-provider-worker-process-adapter-v5\"",
        "native_output_0_worker_execution_capsule_input_roles = \"input.0\"",
        "native_output_3_worker_execution_capsule_input_roles = \"input.0,input.1\"",
        "native_output_4_worker_execution_capsule_output_roles = \"output.result\"",
        "native_output_4_worker_execution_capsule_status = \"worker-invoked\"",
        "native_output_4_worker_execution_capsule_invocation_mode = \"nuis-provider-worker-process-adapter-v5\"",
        "native_output_1_worker_execution_capsule_invocation_mode = \"nuis-provider-worker-process-adapter-v5\"",
        "native_output_2_worker_execution_capsule_invocation_mode = \"nuis-provider-worker-process-adapter-v5\"",
        "native_output_3_worker_execution_capsule_invocation_mode = \"nuis-provider-worker-process-adapter-v5\"",
        "native_output_0_worker_execution_capsule_invoker_contract = \"nuis-provider-execution-capsule-invoker-v1\"",
        "native_output_0_worker_execution_capsule_invoker_id = \"capsule-invoker:",
        "native_output_4_worker_execution_capsule_invoker_status = \"registered-invoked\"",
        "native_output_0_worker_output_descriptor_contract = \"nuis-provider-worker-output-descriptor-v1\"",
        "native_output_0_worker_output_descriptor_roles = \"output.result\"",
        "native_output_4_worker_output_descriptor_count = \"1\"",
        "native_output_4_worker_output_descriptor_byte_length = \"",
        "native_output_0_worker_output_descriptor_hash = \"0x",
        "native_output_4_worker_output_receipt_status = \"verified\"",
        "native_output_0_worker_dispatch_permit_contract = \"nuis-provider-worker-dispatch-permit-v1\"",
        "native_output_0_worker_dispatch_permit_status = \"granted\"",
        "native_output_4_worker_dispatch_permit_status = \"granted\"",
        "native_output_0_worker_dispatch_status = \"1\"",
        "native_output_3_worker_dispatch_status = \"4\"",
        "native_output_4_worker_dispatch_status = \"1\"",
        "native_output_5_worker_dispatch_status = \"5\"",
        "native_output_6_worker_dispatch_status = \"2\"",
        "native_output_4_output_handle_ownership_token = \"glm:provider-session-output:metal:apple-silicon-gpu:0:witsage.vector.metal-bias:output.result\"",
        "native_output_4_output_handle_roles = \"output.result\"",
        "native_output_4_output_handle_ids = \"provider-session:",
        "native_output_4_output_handle_ownership_tokens = \"glm:provider-session-output:",
        "native_output_4_output_handle_release_status = \"released-at-graph-close\"",
        "native_output_4_graph_output_ownership_contract = \"nuis-provider-graph-output-ownership-v1\"",
        "native_output_4_graph_output_release_count = \"7\"",
        "native_output_4_graph_output_release_roles = \"output.result,output.assignment,output.result,output.result,output.result,output.result,output.result\"",
        "native_output_5_request_id = \"witsage.kmeans.centroid-score\"",
        "native_output_5_execution_contract = \"nuis-coreml-model-prediction-provider-runner-v1\"",
        "native_output_5_hash = \"0xaa7b3732298c3952\"",
        "native_output_5_comparison_status = \"comparison-passed\"",
        "native_output_5_comparison_element_count = \"2\"",
        "native_output_5_comparison_mismatch_count = \"0\"",
        "native_output_6_request_id = \"witsage.kmeans.assignment\"",
        "native_output_6_kind = \"provider-scalar-u32\"",
        "native_output_6_execution_contract = \"nuis-metal-f32-argmax-provider-runner-v1\"",
        "native_output_6_hash = \"0xad2aca7747985764\"",
        "native_output_6_comparison_status = \"comparison-passed\"",
        "native_output_6_comparison_element_count = \"1\"",
        "native_output_6_comparison_mismatch_count = \"0\"",
    ] {
        assert_file_contains(
            provider_output_payload_path,
            evidence,
            "official galaxy provider execution",
        );
    }
}

pub(super) fn assert_pixelmagic_trace_evidence(label: &str, source: &str, output_dir: &Path) {
    for evidence in [
        "std-preprocessed-pgm:input_bytes=20",
        "provider_sample_registration_contract=nuis-device-sample-input-registration-v1",
        "provider_filter_plan_contract=nuis-pixelmagic-filter-plan-v1",
        "provider_filter_plan_package=nuis.pixelmagic",
        "provider_filter_plan_validation_status=verified",
        "provider_buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1",
        "provider_kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1",
        "provider_buffer_id=input.pixels",
        "pixel_format=gray8",
        "pixel_width=2",
        "pixel_height=2",
        "pixel_payload_path=nuis.pixelmagic.std-preprocessed.gray8.bin",
    ] {
        assert!(
            source.contains(evidence),
            "PixelMagic trace for {label} is missing `{evidence}`"
        );
    }
    let plan_evidence: &[&str] = match label {
        "pixelmagic_pipeline_demo" => &[
            "provider_filter_plan_id=pixelmagic.gray8.invert-threshold",
            "provider_filter_plan_stage_order=pixelmagic.gray8.invert,pixelmagic.gray8.threshold",
            "provider_kernel_id=pixelmagic.gray8.invert",
            "provider_request_1_kernel_id=pixelmagic.gray8.threshold",
        ],
        "pixelmagic_threshold_provider_demo" => &[
            "provider_filter_plan_id=pixelmagic.gray8.threshold-only",
            "provider_filter_plan_stage_order=pixelmagic.gray8.threshold-only",
            "provider_kernel_id=pixelmagic.gray8.threshold-only",
            "provider_request_count=1",
        ],
        other => panic!("missing PixelMagic trace expectation for `{other}`"),
    };
    for evidence in plan_evidence {
        assert!(
            source.contains(evidence),
            "PixelMagic trace for {label} is missing plan evidence `{evidence}`"
        );
    }
    assert_eq!(
        fs::read(output_dir.join("nuis.pixelmagic.std-preprocessed.gray8.bin"))
            .expect("read persisted PixelMagic input payload"),
        [0, 4, 9, 8]
    );
}

pub(super) fn assert_pixelmagic_execution(provider_output_payload_path: &Path, label: &str) {
    for evidence in [
        "artifact_provider_metadata_contract=nuis-artifact-provider-metadata-v1",
        "artifact_provider_metadata_scope_contract=nuis-artifact-provider-metadata-scope-v1",
        "artifact_provider_metadata_scope_status=verified",
        "artifact_provider_metadata_scope_domain=shader",
        "artifact_provider_metadata_scope_trace=hetero-trace:shader:metal:apple-silicon-gpu",
        "artifact_provider_metadata_source_count=1",
        "artifact_provider_metadata_count=1",
        "provider_filter_plan_catalog_contract=nuis-pixelmagic-filter-plan-catalog-v1",
        "provider_filter_plan_catalog_count=2",
        "provider_filter_plan_catalog_hash=0x",
        "provider_filter_plan_catalog_default_id=pixelmagic.gray8.invert-threshold",
        "provider_filter_plan_catalog_selection_status=artifact-request-selected",
        "provider_filter_plan_contract=nuis-pixelmagic-filter-plan-v1",
        "provider_filter_plan_package=nuis.pixelmagic",
        "provider_filter_plan_hash=0x",
        "provider_filter_plan_validation_status=verified",
        "provider_request_source = \"registered-collection\"",
        "native_output_0_comparison_status = \"comparison-passed\"",
    ] {
        assert_file_contains(
            provider_output_payload_path,
            evidence,
            "official PixelMagic unary execution",
        );
    }
    let plan_evidence: &[&str] = match label {
        "pixelmagic_pipeline_demo" => &[
            "artifact_provider_metadata_0=nuis.pixelmagic:filter-plan=pixelmagic.gray8.invert-threshold",
            "provider_filter_plan_catalog_selected_path=provider-plans/gray8-invert-threshold.nspf",
            "provider_filter_plan_artifact_request_id=pixelmagic.gray8.invert-threshold",
            "provider_filter_plan_id=pixelmagic.gray8.invert-threshold",
            "provider_filter_plan_stage_count=2",
            "provider_filter_plan_stage_order=pixelmagic.gray8.invert,pixelmagic.gray8.threshold",
            "native_output_count = \"2\"",
            "provider_request_dependency_edge_count = \"1\"",
            "provider_request_dependency_edges = \"pixelmagic.gray8.invert.output.pixels.invert->pixelmagic.gray8.threshold.input.pixels\"",
            "provider_edge_transport_count = \"1\"",
            "provider_edge_transport_receipt_count = \"1\"",
            "provider_edge_transport_receipt_0_staging_adapter_id = \"provider.output.transfer.v1\"",
            "provider_edge_transport_receipt_0_consume_status = \"consumed\"",
            "provider_edge_transport_receipt_0_release_status = \"released\"",
            "native_output_0_request_id = \"pixelmagic.gray8.invert\"",
            "native_output_1_request_id = \"pixelmagic.gray8.threshold\"",
            "native_output_1_execution_contract = \"nuis-metal-gray8-threshold-provider-runner-v1\"",
            "native_output_1_comparison_status = \"comparison-passed\"",
            "native_output_1_comparison_element_count = \"4\"",
            "native_output_1_comparison_mismatch_count = \"0\"",
            "native_output_1_hash = \"0xfc6f93a90d12d41b\"",
        ],
        "pixelmagic_threshold_provider_demo" => &[
            "artifact_provider_metadata_0=nuis.pixelmagic:filter-plan=pixelmagic.gray8.threshold-only",
            "provider_filter_plan_catalog_selected_path=provider-plans/gray8-threshold.nspf",
            "provider_filter_plan_artifact_request_id=pixelmagic.gray8.threshold-only",
            "provider_filter_plan_id=pixelmagic.gray8.threshold-only",
            "provider_filter_plan_stage_count=1",
            "provider_filter_plan_stage_order=pixelmagic.gray8.threshold-only",
            "native_output_count = \"1\"",
            "provider_request_dependency_edge_count = \"0\"",
            "provider_edge_transport_count = \"0\"",
            "native_output_0_request_id = \"pixelmagic.gray8.threshold-only\"",
            "native_output_0_execution_contract = \"nuis-metal-gray8-threshold-provider-runner-v1\"",
            "native_output_0_comparison_element_count = \"4\"",
            "native_output_0_comparison_mismatch_count = \"0\"",
            "native_output_0_hash = \"0x4d00177f9dae564b\"",
        ],
        other => panic!("missing PixelMagic execution expectation for `{other}`"),
    };
    for evidence in plan_evidence {
        assert_file_contains(
            provider_output_payload_path,
            evidence,
            "official PixelMagic selected plan execution",
        );
    }
}
