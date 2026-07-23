pub(crate) use crate::provider_sample_artifact::{fnv1a64_hex, provider_output_payload_file_name};
pub(crate) use crate::provider_sample_output_model::PixelMagicNativeOutputSummary;
pub(crate) type ProviderNativeOutputSummary = PixelMagicNativeOutputSummary;
use crate::{
    model::NsdbDeviceProviderSampleRecordInfo,
    provider_edge_transport::ProviderEdgeTransportReceipt,
    provider_native_output_payload::{
        native_output_collection_hash, push_indexed_native_output, push_native_output_summary,
    },
    provider_request::provider_request_from_evidence,
    provider_sample_runner::{provider_execution_outcome_for_runner, ProviderSampleRunner},
    provider_transport_receipt_payload::push_transport_receipts,
};
use std::{fs, path::Path};

pub(crate) struct ProviderOutputPayload {
    pub(crate) evidence: String,
    pub(crate) detail: String,
    pub(crate) status: String,
    pub(crate) evidence_status: String,
}

pub(crate) struct ProviderOutputPayloadSummary {
    pub(crate) path: String,
    pub(crate) hash: String,
    pub(crate) attach_status: String,
}

const PROVIDER_OUTPUT_PAYLOAD_PROTOCOL: &str = "nuis-provider-output-payload-v1";
const PROVIDER_OUTPUT_PAYLOAD_SCHEMA: &str = "nsdb-provider-output-payload-v1";
const PROVIDER_SAMPLE_EXECUTION_CONTRACT: &str = "nuis-provider-sample-execution-v1";

pub(crate) fn provider_output_payload_summary(
    payload: Option<&ProviderOutputPayload>,
) -> ProviderOutputPayloadSummary {
    payload
        .map(|payload| provider_output_payload_summary_from_evidence(&payload.evidence))
        .unwrap_or_else(|| ProviderOutputPayloadSummary {
            path: "none".to_owned(),
            hash: "none".to_owned(),
            attach_status: "none".to_owned(),
        })
}

fn provider_output_payload_summary_from_evidence(evidence: &str) -> ProviderOutputPayloadSummary {
    let mut parts = evidence.split(':');
    let path = parts.next().unwrap_or("none").to_owned();
    let mut hash = "none".to_owned();
    let mut attach_status = "none".to_owned();
    for part in parts {
        if let Some(value) = part.strip_prefix("hash=") {
            hash = value.to_owned();
        } else if let Some(value) = part.strip_prefix("status=") {
            attach_status = value.to_owned();
        }
    }
    ProviderOutputPayloadSummary {
        path,
        hash,
        attach_status,
    }
}

pub(crate) fn provider_output_payload(
    output_dir: &Path,
    record: &NsdbDeviceProviderSampleRecordInfo,
    runner: &ProviderSampleRunner,
) -> ProviderOutputPayload {
    let outcome = provider_execution_outcome_for_runner(runner);
    if runner.real_device_capable {
        if let Some(payload) = existing_provider_output_payload(output_dir, record, runner) {
            return payload;
        }
        return ProviderOutputPayload {
            evidence: "not-materialized".to_owned(),
            detail: outcome.detail.to_owned(),
            status: outcome.output_payload_status.to_owned(),
            evidence_status: outcome.output_payload_evidence_status.to_owned(),
        };
    }
    let file_name = provider_output_payload_file_name(&record.provider_family);
    let path = output_dir.join(&file_name);
    let content = render_provider_output_payload(record, runner);
    let hash = fnv1a64_hex(content.as_bytes());
    let write_status = match fs::write(&path, content) {
        Ok(()) => "written",
        Err(_) => "write-failed",
    };
    ProviderOutputPayload {
        evidence: format!("{file_name}:hash={hash}:status={write_status}"),
        detail: format!("deterministic-provider-output-payload:{file_name}:{hash}:{write_status}"),
        status: "host-fallback-output-payload-ready".to_owned(),
        evidence_status: "deterministic-provider-output-anchor".to_owned(),
    }
}

pub(crate) fn provider_output_payload_from_record(
    record: &NsdbDeviceProviderSampleRecordInfo,
) -> Option<ProviderOutputPayload> {
    (record.provider_output_payload_evidence != "none").then(|| ProviderOutputPayload {
        evidence: record.provider_output_payload_evidence.clone(),
        detail: record.provider_output_payload_detail.clone(),
        status: record.provider_output_payload_status.clone(),
        evidence_status: record.provider_output_payload_evidence_status.clone(),
    })
}

pub(crate) fn provider_sample_status_for_payload(payload: &ProviderOutputPayload) -> &'static str {
    if payload.status == "real-device-output-payload-invalid" {
        "provider-execution-blocked"
    } else {
        "provider-execution-ready"
    }
}

pub(crate) fn provider_validation_status_for_payload(
    payload: &ProviderOutputPayload,
) -> &'static str {
    if payload.status == "real-device-output-payload-invalid" {
        "provider-output-payload-invalid"
    } else {
        "provider-execution-validated"
    }
}

pub(crate) fn provider_materialization_status_for_payload(
    payload: &ProviderOutputPayload,
) -> &'static str {
    if payload.status == "real-device-output-payload-invalid" {
        "provider-sample-blocked"
    } else {
        "provider-sample-materialized"
    }
}

pub(crate) fn provider_next_action_for_payload(payload: &ProviderOutputPayload) -> &'static str {
    if payload.status == "real-device-output-payload-invalid" {
        "repair-provider-output-payload"
    } else {
        "replay-device-sample"
    }
}

fn existing_provider_output_payload(
    output_dir: &Path,
    record: &NsdbDeviceProviderSampleRecordInfo,
    runner: &ProviderSampleRunner,
) -> Option<ProviderOutputPayload> {
    let file_name = provider_output_payload_file_name(&record.provider_family);
    let path = output_dir.join(&file_name);
    let content = fs::read(&path).ok()?;
    let hash = fnv1a64_hex(&content);
    if let Err(error) = validate_provider_output_payload(record, &content) {
        return Some(ProviderOutputPayload {
            evidence: format!("{file_name}:hash={hash}:status=rejected"),
            detail: format!(
                "real-device-provider-output-payload-invalid:{file_name}:{hash}:{error}"
            ),
            status: "real-device-output-payload-invalid".to_owned(),
            evidence_status: "provider-output-payload-rejected".to_owned(),
        });
    }
    Some(ProviderOutputPayload {
        evidence: format!("{file_name}:hash={hash}:status=attached"),
        detail: format!(
            "real-device-provider-output-payload:{file_name}:{hash}:attached:{}",
            runner.adapter_id
        ),
        status: "real-device-output-payload-attached".to_owned(),
        evidence_status: "provider-output-payload-attached".to_owned(),
    })
}

fn validate_provider_output_payload(
    record: &NsdbDeviceProviderSampleRecordInfo,
    content: &[u8],
) -> Result<(), String> {
    let text = String::from_utf8_lossy(content);
    let expected_hash = fnv1a64_hex(record.input_evidence.as_bytes());
    for (field, needle) in [
        (
            "protocol",
            format!("protocol = \"{PROVIDER_OUTPUT_PAYLOAD_PROTOCOL}\""),
        ),
        (
            "schema",
            format!("schema = \"{PROVIDER_OUTPUT_PAYLOAD_SCHEMA}\""),
        ),
        (
            "sample_execution_contract",
            format!("sample_execution_contract = \"{PROVIDER_SAMPLE_EXECUTION_CONTRACT}\""),
        ),
        (
            "provider_family",
            format!("provider_family = \"{}\"", record.provider_family),
        ),
        (
            "input_evidence_hash",
            format!("input_evidence_hash = \"{expected_hash}\""),
        ),
    ] {
        if !text.contains(&needle) {
            return Err(format!("missing-{field}"));
        }
    }
    Ok(())
}

pub(crate) fn render_real_device_provider_output_payload(
    record: &NsdbDeviceProviderSampleRecordInfo,
    adapter: &crate::provider_runner_registry::ProviderRunnerAdapter,
    native_outputs: &[PixelMagicNativeOutputSummary],
    transport_receipts: &[ProviderEdgeTransportReceipt],
) -> String {
    let mut out = render_provider_output_payload_header(
        record,
        "nsdb-execute-provider-samples",
        adapter.adapter_id,
        adapter.capability_status,
        adapter.execution_mode,
    );
    push_toml_string(
        &mut out,
        "output_payload_kind",
        if !native_outputs.is_empty() {
            "real-device-api-output"
        } else {
            "real-device-adapter-output"
        },
    );
    push_toml_string(
        &mut out,
        "output_payload_status",
        if !native_outputs.is_empty() {
            "native-api-output-ready"
        } else {
            "adapter-output-ready"
        },
    );
    let comparison_status = if native_outputs.iter().any(|output| {
        output.comparison_contract != "none" && output.comparison_status == "comparison-passed"
    }) && native_outputs
        .iter()
        .filter(|output| output.comparison_contract != "none")
        .all(|output| output.comparison_status == "comparison-passed")
    {
        "comparison-passed"
    } else {
        "ready-for-comparison"
    };
    push_toml_string(&mut out, "comparison_status", comparison_status);
    push_transport_receipts(&mut out, transport_receipts);
    if let Some(summary) = native_outputs.first() {
        push_native_output_summary(&mut out, summary);
        push_toml_string(
            &mut out,
            "native_output_collection_contract",
            "nuis-provider-output-collection-v1",
        );
        push_toml_string(
            &mut out,
            "native_output_count",
            &native_outputs.len().to_string(),
        );
        push_toml_string(
            &mut out,
            "native_output_collection_hash",
            &native_output_collection_hash(native_outputs),
        );
        for (index, output) in native_outputs.iter().enumerate() {
            push_indexed_native_output(&mut out, index, output);
        }
    } else {
        push_pixelmagic_image_output_summary(&mut out, record);
    }
    out
}

fn render_provider_output_payload(
    record: &NsdbDeviceProviderSampleRecordInfo,
    runner: &ProviderSampleRunner,
) -> String {
    let mut out = render_provider_output_payload_header(
        record,
        "nsdb-materialize-provider-samples",
        runner.adapter_id,
        runner.adapter_capability_status,
        runner.execution_mode,
    );
    push_toml_string(&mut out, "output_payload_kind", "host-fallback-anchor");
    push_toml_string(
        &mut out,
        "output_payload_status",
        "host-fallback-anchor-ready",
    );
    push_toml_string(&mut out, "comparison_status", "ready-for-comparison");
    push_pixelmagic_image_output_summary(&mut out, record);
    out
}

pub(crate) fn pixelmagic_native_output_summary(
    input_evidence: &str,
    provider_family: &str,
) -> Option<PixelMagicNativeOutputSummary> {
    let input_bytes = std_preprocessed_pgm_input_bytes(input_evidence).or_else(|| {
        provider_request_from_evidence(input_evidence).map(|request| request.buffer.byte_length)
    })?;
    let output_bytes = provider_request_from_evidence(input_evidence)
        .map(|request| request.buffer.byte_length)
        .unwrap_or_else(|| pixelmagic_deterministic_output_bytes(input_bytes, provider_family));
    Some(PixelMagicNativeOutputSummary {
        request_id: provider_request_from_evidence(input_evidence)
            .map(|request| request.kernel.id)
            .unwrap_or_else(|| "pixelmagic.legacy".to_owned()),
        kind: "pixelmagic-image-bytes".to_owned(),
        status: "deterministic-provider-output-ready".to_owned(),
        bytes: output_bytes.to_string(),
        hash: fnv1a64_hex(format!("{input_evidence}:{output_bytes}").as_bytes()),
        execution_contract: "nuis-deterministic-provider-output-v1".to_owned(),
        execution_status: "host-deterministic-output-ready".to_owned(),
        device: "host-deterministic-fallback".to_owned(),
        output_carrier_registry_contract: "none".to_owned(),
        output_carrier_registry_source: "none".to_owned(),
        output_carrier_adapter_id: "none".to_owned(),
        output_carrier_mode: "none".to_owned(),
        output_residency_contract: "none".to_owned(),
        output_residency_kind: "host-deterministic".to_owned(),
        output_transfer_scope: "observation-only".to_owned(),
        output_observation_mode: "precomputed".to_owned(),
        output_device_retention_status: "not-applicable".to_owned(),
        session_registry_contract: "none".to_owned(),
        session_registry_source: "none".to_owned(),
        session_lease_contract: "none".to_owned(),
        session_lease_id: "none".to_owned(),
        session_adapter_id: "none".to_owned(),
        session_mode: "none".to_owned(),
        session_continuity: "none".to_owned(),
        session_lifecycle_hooks: "none".to_owned(),
        session_request_sequence: "none".to_owned(),
        worker_lease_contract: "none".to_owned(),
        worker_resolver_contract: "none".to_owned(),
        worker_cache_status: "none".to_owned(),
        worker_adapter_cache_contract: "none".to_owned(),
        worker_adapter_cache_identity: "none".to_owned(),
        worker_adapter_cache_status: "none".to_owned(),
        worker_adapter_executable_hash: "none".to_owned(),
        worker_adapter_control_contract: "none".to_owned(),
        worker_adapter_control_mode: "none".to_owned(),
        worker_pid: "none".to_owned(),
        worker_request_sequence: "none".to_owned(),
        worker_descriptor_count: "none".to_owned(),
        worker_descriptor_capability_contract: "none".to_owned(),
        worker_max_semantic_descriptors: "none".to_owned(),
        worker_max_control_descriptors: "none".to_owned(),
        worker_output_descriptor_capability_contract: "none".to_owned(),
        worker_max_output_descriptors: "none".to_owned(),
        worker_payload_hash: "none".to_owned(),
        worker_operation_token: "none".to_owned(),
        worker_execution_capsule_contract: "none".to_owned(),
        worker_execution_capsule_id: "none".to_owned(),
        worker_execution_capsule_token: "none".to_owned(),
        worker_execution_capsule_invocation_mode: "none".to_owned(),
        worker_execution_capsule_input_roles: "none".to_owned(),
        worker_execution_capsule_output_roles: "none".to_owned(),
        worker_execution_capsule_status: "none".to_owned(),
        worker_execution_capsule_invoker_contract: "none".to_owned(),
        worker_execution_capsule_invoker_id: "none".to_owned(),
        worker_execution_capsule_invoker_status: "none".to_owned(),
        worker_output_descriptor_contract: "none".to_owned(),
        worker_output_descriptor_roles: "none".to_owned(),
        worker_output_descriptor_count: "none".to_owned(),
        worker_output_descriptor_byte_length: "none".to_owned(),
        worker_output_descriptor_hash: "none".to_owned(),
        worker_additional_output_roles: "none".to_owned(),
        worker_additional_output_byte_lengths: "none".to_owned(),
        worker_additional_output_hashes: "none".to_owned(),
        worker_additional_output_retention_statuses: "none".to_owned(),
        worker_output_receipt_status: "none".to_owned(),
        worker_dispatch_permit_contract: "none".to_owned(),
        worker_dispatch_permit_status: "none".to_owned(),
        worker_dispatch_status: "none".to_owned(),
        output_handle_contract: "none".to_owned(),
        output_handle_id: "none".to_owned(),
        output_handle_ownership_token: "none".to_owned(),
        output_handle_roles: "none".to_owned(),
        output_handle_ids: "none".to_owned(),
        output_handle_ownership_tokens: "none".to_owned(),
        output_handle_release_status: "not-applicable".to_owned(),
        graph_output_ownership_contract: "none".to_owned(),
        graph_output_release_count: "0".to_owned(),
        graph_output_release_roles: "none".to_owned(),
        output_binding_contract: "none".to_owned(),
        output_binding_count: "0".to_owned(),
        output_binding_roles: "none".to_owned(),
        output_binding_buffers: "none".to_owned(),
        output_binding_element_types: "none".to_owned(),
        output_binding_shapes: "none".to_owned(),
        output_binding_byte_lengths: "none".to_owned(),
        output_binding_comparison_ids: "none".to_owned(),
        comparison_collection_contract: "none".to_owned(),
        comparison_collection_count: "0".to_owned(),
        comparison_collection_ids: "none".to_owned(),
        comparison_collection_output_buffers: "none".to_owned(),
        comparison_collection_statuses: "none".to_owned(),
        comparison_collection_element_counts: "none".to_owned(),
        comparison_collection_mismatch_counts: "none".to_owned(),
        compute_plan_contract: "none".to_owned(),
        compute_plan_status: "not-applicable".to_owned(),
        compute_plan_layer_count: "0".to_owned(),
        compute_plan_preferred_devices: "none".to_owned(),
        compute_plan_supported_devices: "none".to_owned(),
        comparison_contract: "none".to_owned(),
        comparison_status: "not-applicable".to_owned(),
        comparison_element_count: "0".to_owned(),
        comparison_mismatch_count: "0".to_owned(),
        comparison_max_absolute_error: "0".to_owned(),
        comparison_max_relative_error: "0".to_owned(),
        comparison_non_finite_count: "0".to_owned(),
    })
}

pub(crate) fn gray8_metal_output_summary(
    input_evidence: &str,
    execution: &crate::provider_runner_metal::MetalProviderExecution,
) -> ProviderNativeOutputSummary {
    metal_native_output_summary(
        provider_request_from_evidence(input_evidence)
            .map(|request| request.kernel.id)
            .unwrap_or_else(|| "pixelmagic.legacy".to_owned()),
        "pixelmagic-image-bytes",
        execution,
        None,
    )
}

pub(crate) fn metal_native_output_summary(
    request_id: String,
    kind: &str,
    execution: &crate::provider_runner_metal::MetalProviderExecution,
    comparison: Option<&crate::provider_output_comparison::ProviderOutputComparisonResult>,
) -> PixelMagicNativeOutputSummary {
    let output = execution.output_payload.as_bytes();
    let bytes = output.len().to_string();
    PixelMagicNativeOutputSummary {
        request_id,
        kind: kind.to_owned(),
        status: "metal-api-output-ready".to_owned(),
        hash: fnv1a64_hex(output),
        bytes,
        execution_contract: execution.contract.to_owned(),
        execution_status: execution.status.to_owned(),
        device: execution.device.clone(),
        output_carrier_registry_contract: execution.output_carrier_registry_contract.clone(),
        output_carrier_registry_source: execution.output_carrier_registry_source.clone(),
        output_carrier_adapter_id: execution.output_carrier_adapter_id.clone(),
        output_carrier_mode: execution.output_carrier_mode.clone(),
        output_residency_contract: execution.output_residency_contract.clone(),
        output_residency_kind: execution.output_residency_kind.clone(),
        output_transfer_scope: execution.output_transfer_scope.clone(),
        output_observation_mode: execution.output_observation_mode.clone(),
        output_device_retention_status: execution.output_device_retention_status.clone(),
        session_registry_contract: "pending".to_owned(),
        session_registry_source: "pending".to_owned(),
        session_lease_contract: "pending".to_owned(),
        session_lease_id: "pending".to_owned(),
        session_adapter_id: "pending".to_owned(),
        session_mode: "pending".to_owned(),
        session_continuity: "pending".to_owned(),
        session_lifecycle_hooks: "pending".to_owned(),
        session_request_sequence: "pending".to_owned(),
        worker_lease_contract: "pending".to_owned(),
        worker_resolver_contract: "pending".to_owned(),
        worker_cache_status: "pending".to_owned(),
        worker_adapter_cache_contract: "pending".to_owned(),
        worker_adapter_cache_identity: "pending".to_owned(),
        worker_adapter_cache_status: "pending".to_owned(),
        worker_adapter_executable_hash: "pending".to_owned(),
        worker_adapter_control_contract: "pending".to_owned(),
        worker_adapter_control_mode: "pending".to_owned(),
        worker_pid: "pending".to_owned(),
        worker_request_sequence: "pending".to_owned(),
        worker_descriptor_count: "pending".to_owned(),
        worker_descriptor_capability_contract: "pending".to_owned(),
        worker_max_semantic_descriptors: "pending".to_owned(),
        worker_max_control_descriptors: "pending".to_owned(),
        worker_output_descriptor_capability_contract: "pending".to_owned(),
        worker_max_output_descriptors: "pending".to_owned(),
        worker_payload_hash: "pending".to_owned(),
        worker_operation_token: "pending".to_owned(),
        worker_execution_capsule_contract: "pending".to_owned(),
        worker_execution_capsule_id: "pending".to_owned(),
        worker_execution_capsule_token: "pending".to_owned(),
        worker_execution_capsule_invocation_mode: "pending".to_owned(),
        worker_execution_capsule_input_roles: "pending".to_owned(),
        worker_execution_capsule_output_roles: "pending".to_owned(),
        worker_execution_capsule_status: "pending".to_owned(),
        worker_execution_capsule_invoker_contract: "pending".to_owned(),
        worker_execution_capsule_invoker_id: "pending".to_owned(),
        worker_execution_capsule_invoker_status: "pending".to_owned(),
        worker_output_descriptor_contract: "pending".to_owned(),
        worker_output_descriptor_roles: "pending".to_owned(),
        worker_output_descriptor_count: "pending".to_owned(),
        worker_output_descriptor_byte_length: "pending".to_owned(),
        worker_output_descriptor_hash: "pending".to_owned(),
        worker_additional_output_roles: "pending".to_owned(),
        worker_additional_output_byte_lengths: "pending".to_owned(),
        worker_additional_output_hashes: "pending".to_owned(),
        worker_additional_output_retention_statuses: "pending".to_owned(),
        worker_output_receipt_status: "pending".to_owned(),
        worker_dispatch_permit_contract: "pending".to_owned(),
        worker_dispatch_permit_status: "pending".to_owned(),
        worker_dispatch_status: "pending".to_owned(),
        output_handle_contract: "pending".to_owned(),
        output_handle_id: "pending".to_owned(),
        output_handle_ownership_token: "pending".to_owned(),
        output_handle_roles: "pending".to_owned(),
        output_handle_ids: "pending".to_owned(),
        output_handle_ownership_tokens: "pending".to_owned(),
        output_handle_release_status: "pending".to_owned(),
        graph_output_ownership_contract: "pending".to_owned(),
        graph_output_release_count: "pending".to_owned(),
        graph_output_release_roles: "pending".to_owned(),
        output_binding_contract: "pending".to_owned(),
        output_binding_count: "pending".to_owned(),
        output_binding_roles: "pending".to_owned(),
        output_binding_buffers: "pending".to_owned(),
        output_binding_element_types: "pending".to_owned(),
        output_binding_shapes: "pending".to_owned(),
        output_binding_byte_lengths: "pending".to_owned(),
        output_binding_comparison_ids: "pending".to_owned(),
        comparison_collection_contract: "pending".to_owned(),
        comparison_collection_count: "pending".to_owned(),
        comparison_collection_ids: "pending".to_owned(),
        comparison_collection_output_buffers: "pending".to_owned(),
        comparison_collection_statuses: "pending".to_owned(),
        comparison_collection_element_counts: "pending".to_owned(),
        comparison_collection_mismatch_counts: "pending".to_owned(),
        compute_plan_contract: "none".to_owned(),
        compute_plan_status: "not-applicable".to_owned(),
        compute_plan_layer_count: "0".to_owned(),
        compute_plan_preferred_devices: "none".to_owned(),
        compute_plan_supported_devices: "none".to_owned(),
        comparison_contract: comparison
            .map(|value| value.contract)
            .unwrap_or("none")
            .to_owned(),
        comparison_status: comparison
            .map(|value| value.status)
            .unwrap_or("not-applicable")
            .to_owned(),
        comparison_element_count: comparison
            .map(|value| value.compared_elements.to_string())
            .unwrap_or_else(|| "0".to_owned()),
        comparison_mismatch_count: comparison
            .map(|value| value.mismatch_count.to_string())
            .unwrap_or_else(|| "0".to_owned()),
        comparison_max_absolute_error: comparison
            .map(|value| value.max_absolute_error.clone())
            .unwrap_or_else(|| "0".to_owned()),
        comparison_max_relative_error: comparison
            .map(|value| value.max_relative_error.clone())
            .unwrap_or_else(|| "0".to_owned()),
        comparison_non_finite_count: comparison
            .map(|value| value.non_finite_count.to_string())
            .unwrap_or_else(|| "0".to_owned()),
    }
}

pub(crate) fn coreml_native_output_summary(
    request_id: &str,
    execution: &crate::provider_runner_coreml::CoreMlProviderExecution,
    comparison: Option<&crate::provider_output_comparison::ProviderOutputComparisonResult>,
) -> PixelMagicNativeOutputSummary {
    PixelMagicNativeOutputSummary {
        request_id: request_id.to_owned(),
        kind: "provider-tensor-f32".to_owned(),
        status: "coreml-api-output-ready".to_owned(),
        hash: fnv1a64_hex(execution.output_payload.as_bytes()),
        bytes: execution.output_payload.as_bytes().len().to_string(),
        execution_contract: execution.contract.to_owned(),
        execution_status: execution.status.to_owned(),
        device: execution.device.clone(),
        output_carrier_registry_contract: execution.output_carrier_registry_contract.clone(),
        output_carrier_registry_source: execution.output_carrier_registry_source.clone(),
        output_carrier_adapter_id: execution.output_carrier_adapter_id.clone(),
        output_carrier_mode: execution.output_carrier_mode.clone(),
        output_residency_contract: execution.output_residency_contract.clone(),
        output_residency_kind: execution.output_residency_kind.clone(),
        output_transfer_scope: execution.output_transfer_scope.clone(),
        output_observation_mode: execution.output_observation_mode.clone(),
        output_device_retention_status: execution.output_device_retention_status.clone(),
        session_registry_contract: "pending".to_owned(),
        session_registry_source: "pending".to_owned(),
        session_lease_contract: "pending".to_owned(),
        session_lease_id: "pending".to_owned(),
        session_adapter_id: "pending".to_owned(),
        session_mode: "pending".to_owned(),
        session_continuity: "pending".to_owned(),
        session_lifecycle_hooks: "pending".to_owned(),
        session_request_sequence: "pending".to_owned(),
        worker_lease_contract: "pending".to_owned(),
        worker_resolver_contract: "pending".to_owned(),
        worker_cache_status: "pending".to_owned(),
        worker_adapter_cache_contract: "pending".to_owned(),
        worker_adapter_cache_identity: "pending".to_owned(),
        worker_adapter_cache_status: "pending".to_owned(),
        worker_adapter_executable_hash: "pending".to_owned(),
        worker_adapter_control_contract: "pending".to_owned(),
        worker_adapter_control_mode: "pending".to_owned(),
        worker_pid: "pending".to_owned(),
        worker_request_sequence: "pending".to_owned(),
        worker_descriptor_count: "pending".to_owned(),
        worker_descriptor_capability_contract: "pending".to_owned(),
        worker_max_semantic_descriptors: "pending".to_owned(),
        worker_max_control_descriptors: "pending".to_owned(),
        worker_output_descriptor_capability_contract: "pending".to_owned(),
        worker_max_output_descriptors: "pending".to_owned(),
        worker_payload_hash: "pending".to_owned(),
        worker_operation_token: "pending".to_owned(),
        worker_execution_capsule_contract: "pending".to_owned(),
        worker_execution_capsule_id: "pending".to_owned(),
        worker_execution_capsule_token: "pending".to_owned(),
        worker_execution_capsule_invocation_mode: "pending".to_owned(),
        worker_execution_capsule_input_roles: "pending".to_owned(),
        worker_execution_capsule_output_roles: "pending".to_owned(),
        worker_execution_capsule_status: "pending".to_owned(),
        worker_execution_capsule_invoker_contract: "pending".to_owned(),
        worker_execution_capsule_invoker_id: "pending".to_owned(),
        worker_execution_capsule_invoker_status: "pending".to_owned(),
        worker_output_descriptor_contract: "pending".to_owned(),
        worker_output_descriptor_roles: "pending".to_owned(),
        worker_output_descriptor_count: "pending".to_owned(),
        worker_output_descriptor_byte_length: "pending".to_owned(),
        worker_output_descriptor_hash: "pending".to_owned(),
        worker_additional_output_roles: "pending".to_owned(),
        worker_additional_output_byte_lengths: "pending".to_owned(),
        worker_additional_output_hashes: "pending".to_owned(),
        worker_additional_output_retention_statuses: "pending".to_owned(),
        worker_output_receipt_status: "pending".to_owned(),
        worker_dispatch_permit_contract: "pending".to_owned(),
        worker_dispatch_permit_status: "pending".to_owned(),
        worker_dispatch_status: "pending".to_owned(),
        output_handle_contract: "pending".to_owned(),
        output_handle_id: "pending".to_owned(),
        output_handle_ownership_token: "pending".to_owned(),
        output_handle_roles: "pending".to_owned(),
        output_handle_ids: "pending".to_owned(),
        output_handle_ownership_tokens: "pending".to_owned(),
        output_handle_release_status: "pending".to_owned(),
        graph_output_ownership_contract: "pending".to_owned(),
        graph_output_release_count: "pending".to_owned(),
        graph_output_release_roles: "pending".to_owned(),
        output_binding_contract: "pending".to_owned(),
        output_binding_count: "pending".to_owned(),
        output_binding_roles: "pending".to_owned(),
        output_binding_buffers: "pending".to_owned(),
        output_binding_element_types: "pending".to_owned(),
        output_binding_shapes: "pending".to_owned(),
        output_binding_byte_lengths: "pending".to_owned(),
        output_binding_comparison_ids: "pending".to_owned(),
        comparison_collection_contract: "pending".to_owned(),
        comparison_collection_count: "pending".to_owned(),
        comparison_collection_ids: "pending".to_owned(),
        comparison_collection_output_buffers: "pending".to_owned(),
        comparison_collection_statuses: "pending".to_owned(),
        comparison_collection_element_counts: "pending".to_owned(),
        comparison_collection_mismatch_counts: "pending".to_owned(),
        compute_plan_contract: execution.compute_plan_contract.clone(),
        compute_plan_status: execution.compute_plan_status.clone(),
        compute_plan_layer_count: execution.compute_plan_layer_count.to_string(),
        compute_plan_preferred_devices: execution.compute_plan_preferred_devices.clone(),
        compute_plan_supported_devices: execution.compute_plan_supported_devices.clone(),
        comparison_contract: comparison
            .map(|comparison| comparison.contract)
            .unwrap_or("none")
            .to_owned(),
        comparison_status: comparison
            .map(|comparison| comparison.status)
            .unwrap_or("not-applicable")
            .to_owned(),
        comparison_element_count: comparison
            .map(|comparison| comparison.compared_elements.to_string())
            .unwrap_or_else(|| "0".to_owned()),
        comparison_mismatch_count: comparison
            .map(|comparison| comparison.mismatch_count.to_string())
            .unwrap_or_else(|| "0".to_owned()),
        comparison_max_absolute_error: comparison
            .map(|comparison| comparison.max_absolute_error.clone())
            .unwrap_or_else(|| "0".to_owned()),
        comparison_max_relative_error: comparison
            .map(|comparison| comparison.max_relative_error.clone())
            .unwrap_or_else(|| "0".to_owned()),
        comparison_non_finite_count: comparison
            .map(|comparison| comparison.non_finite_count.to_string())
            .unwrap_or_else(|| "0".to_owned()),
    }
}

fn push_pixelmagic_image_output_summary(
    out: &mut String,
    record: &NsdbDeviceProviderSampleRecordInfo,
) {
    let Some(summary) =
        pixelmagic_native_output_summary(&record.input_evidence, &record.provider_family)
    else {
        return;
    };
    push_toml_string(out, "comparison_input_kind", "std-preprocessed-pgm");
    push_native_output_summary(out, &summary);
}

pub(crate) fn std_preprocessed_pgm_input_bytes(input_evidence: &str) -> Option<usize> {
    let marker = "std-preprocessed-pgm:input_bytes=";
    let start = input_evidence.find(marker)? + marker.len();
    let digits = input_evidence[start..]
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    (!digits.is_empty()).then(|| digits.parse().ok()).flatten()
}

fn pixelmagic_deterministic_output_bytes(input_bytes: usize, provider_family: &str) -> usize {
    let provider_bias = if provider_family.starts_with("metal") {
        4
    } else if provider_family.starts_with("coreml") {
        8
    } else {
        2
    };
    input_bytes.saturating_add(provider_bias)
}

fn render_provider_output_payload_header(
    record: &NsdbDeviceProviderSampleRecordInfo,
    source: &str,
    adapter_id: &str,
    adapter_capability_status: &str,
    execution_mode: &str,
) -> String {
    let mut out = String::new();
    push_toml_string(&mut out, "protocol", PROVIDER_OUTPUT_PAYLOAD_PROTOCOL);
    push_toml_string(&mut out, "schema", PROVIDER_OUTPUT_PAYLOAD_SCHEMA);
    push_toml_string(&mut out, "source", source);
    push_toml_string(
        &mut out,
        "sample_execution_contract",
        PROVIDER_SAMPLE_EXECUTION_CONTRACT,
    );
    push_toml_string(&mut out, "trace_id", &record.trace_id);
    push_toml_string(&mut out, "provider_family", &record.provider_family);
    push_toml_string(&mut out, "provider_runner_adapter_id", adapter_id);
    push_toml_string(
        &mut out,
        "provider_runner_adapter_capability_status",
        adapter_capability_status,
    );
    push_toml_string(
        &mut out,
        "provider_runner_real_device_probe_status",
        crate::provider_runner_registry::provider_runner_real_device_probe_status(
            &record.provider_family,
        ),
    );
    push_toml_string(&mut out, "provider_execution_mode", execution_mode);
    push_toml_string(&mut out, "input_evidence", &record.input_evidence);
    push_toml_string(
        &mut out,
        "input_evidence_hash",
        &fnv1a64_hex(record.input_evidence.as_bytes()),
    );
    out.push_str(
        &crate::provider_request_payload::render_provider_request_evidence(&record.input_evidence),
    );
    out
}

pub(crate) fn push_toml_string(out: &mut String, key: &str, value: &str) {
    out.push_str(key);
    out.push_str(" = \"");
    out.push_str(&value.replace('\\', "\\\\").replace('"', "\\\""));
    out.push_str("\"\n");
}
