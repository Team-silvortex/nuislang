use crate::provider_sample_payload::{
    fnv1a64_hex, push_toml_string, PixelMagicNativeOutputSummary,
};

pub(crate) fn push_native_output_summary(
    out: &mut String,
    summary: &PixelMagicNativeOutputSummary,
) {
    for (name, value) in [
        ("kind", summary.kind.as_str()),
        ("status", summary.status.as_str()),
        ("bytes", summary.bytes.as_str()),
        ("hash", summary.hash.as_str()),
        ("execution_contract", summary.execution_contract.as_str()),
        ("execution_status", summary.execution_status.as_str()),
        ("device", summary.device.as_str()),
        (
            "carrier_registry_contract",
            summary.output_carrier_registry_contract.as_str(),
        ),
        (
            "carrier_registry_source",
            summary.output_carrier_registry_source.as_str(),
        ),
        (
            "carrier_adapter_id",
            summary.output_carrier_adapter_id.as_str(),
        ),
        ("carrier_mode", summary.output_carrier_mode.as_str()),
        (
            "residency_contract",
            summary.output_residency_contract.as_str(),
        ),
        ("residency_kind", summary.output_residency_kind.as_str()),
        ("transfer_scope", summary.output_transfer_scope.as_str()),
        ("observation_mode", summary.output_observation_mode.as_str()),
        (
            "device_retention_status",
            summary.output_device_retention_status.as_str(),
        ),
        (
            "session_registry_contract",
            summary.session_registry_contract.as_str(),
        ),
        (
            "session_registry_source",
            summary.session_registry_source.as_str(),
        ),
        (
            "session_lease_contract",
            summary.session_lease_contract.as_str(),
        ),
        ("session_lease_id", summary.session_lease_id.as_str()),
        ("session_adapter_id", summary.session_adapter_id.as_str()),
        ("session_mode", summary.session_mode.as_str()),
        ("session_continuity", summary.session_continuity.as_str()),
        (
            "session_lifecycle_hooks",
            summary.session_lifecycle_hooks.as_str(),
        ),
        (
            "session_request_sequence",
            summary.session_request_sequence.as_str(),
        ),
        (
            "worker_lease_contract",
            summary.worker_lease_contract.as_str(),
        ),
        (
            "worker_resolver_contract",
            summary.worker_resolver_contract.as_str(),
        ),
        ("worker_cache_status", summary.worker_cache_status.as_str()),
        ("worker_pid", summary.worker_pid.as_str()),
        (
            "worker_request_sequence",
            summary.worker_request_sequence.as_str(),
        ),
        (
            "worker_descriptor_count",
            summary.worker_descriptor_count.as_str(),
        ),
        ("worker_payload_hash", summary.worker_payload_hash.as_str()),
        (
            "worker_operation_token",
            summary.worker_operation_token.as_str(),
        ),
        (
            "worker_execution_capsule_contract",
            summary.worker_execution_capsule_contract.as_str(),
        ),
        (
            "worker_execution_capsule_id",
            summary.worker_execution_capsule_id.as_str(),
        ),
        (
            "worker_execution_capsule_token",
            summary.worker_execution_capsule_token.as_str(),
        ),
        (
            "worker_execution_capsule_invocation_mode",
            summary.worker_execution_capsule_invocation_mode.as_str(),
        ),
        (
            "worker_execution_capsule_input_roles",
            summary.worker_execution_capsule_input_roles.as_str(),
        ),
        (
            "worker_execution_capsule_output_roles",
            summary.worker_execution_capsule_output_roles.as_str(),
        ),
        (
            "worker_execution_capsule_status",
            summary.worker_execution_capsule_status.as_str(),
        ),
        (
            "worker_execution_capsule_invoker_contract",
            summary.worker_execution_capsule_invoker_contract.as_str(),
        ),
        (
            "worker_execution_capsule_invoker_id",
            summary.worker_execution_capsule_invoker_id.as_str(),
        ),
        (
            "worker_execution_capsule_invoker_status",
            summary.worker_execution_capsule_invoker_status.as_str(),
        ),
        (
            "worker_output_descriptor_contract",
            summary.worker_output_descriptor_contract.as_str(),
        ),
        (
            "worker_output_descriptor_roles",
            summary.worker_output_descriptor_roles.as_str(),
        ),
        (
            "worker_output_descriptor_count",
            summary.worker_output_descriptor_count.as_str(),
        ),
        (
            "worker_output_descriptor_byte_length",
            summary.worker_output_descriptor_byte_length.as_str(),
        ),
        (
            "worker_output_descriptor_hash",
            summary.worker_output_descriptor_hash.as_str(),
        ),
        (
            "worker_output_receipt_status",
            summary.worker_output_receipt_status.as_str(),
        ),
        (
            "worker_dispatch_permit_contract",
            summary.worker_dispatch_permit_contract.as_str(),
        ),
        (
            "worker_dispatch_permit_status",
            summary.worker_dispatch_permit_status.as_str(),
        ),
        (
            "worker_dispatch_status",
            summary.worker_dispatch_status.as_str(),
        ),
        (
            "output_handle_contract",
            summary.output_handle_contract.as_str(),
        ),
        ("output_handle_id", summary.output_handle_id.as_str()),
        (
            "output_handle_ownership_token",
            summary.output_handle_ownership_token.as_str(),
        ),
        (
            "output_handle_release_status",
            summary.output_handle_release_status.as_str(),
        ),
        (
            "compute_plan_contract",
            summary.compute_plan_contract.as_str(),
        ),
        ("compute_plan_status", summary.compute_plan_status.as_str()),
        (
            "compute_plan_layer_count",
            summary.compute_plan_layer_count.as_str(),
        ),
        (
            "compute_plan_preferred_devices",
            summary.compute_plan_preferred_devices.as_str(),
        ),
        (
            "compute_plan_supported_devices",
            summary.compute_plan_supported_devices.as_str(),
        ),
        ("comparison_contract", summary.comparison_contract.as_str()),
        ("comparison_status", summary.comparison_status.as_str()),
        (
            "comparison_element_count",
            summary.comparison_element_count.as_str(),
        ),
        (
            "comparison_mismatch_count",
            summary.comparison_mismatch_count.as_str(),
        ),
        (
            "comparison_max_absolute_error",
            summary.comparison_max_absolute_error.as_str(),
        ),
        (
            "comparison_max_relative_error",
            summary.comparison_max_relative_error.as_str(),
        ),
        (
            "comparison_non_finite_count",
            summary.comparison_non_finite_count.as_str(),
        ),
    ] {
        push_toml_string(out, &format!("native_output_{name}"), value);
    }
}

pub(crate) fn push_indexed_native_output(
    out: &mut String,
    index: usize,
    summary: &PixelMagicNativeOutputSummary,
) {
    let prefix = format!("native_output_{index}_");
    for (name, value) in [
        ("request_id", summary.request_id.as_str()),
        ("kind", summary.kind.as_str()),
        ("status", summary.status.as_str()),
        ("bytes", summary.bytes.as_str()),
        ("hash", summary.hash.as_str()),
        ("execution_contract", summary.execution_contract.as_str()),
        ("execution_status", summary.execution_status.as_str()),
        ("device", summary.device.as_str()),
        (
            "output_carrier_registry_contract",
            summary.output_carrier_registry_contract.as_str(),
        ),
        (
            "output_carrier_registry_source",
            summary.output_carrier_registry_source.as_str(),
        ),
        (
            "output_carrier_adapter_id",
            summary.output_carrier_adapter_id.as_str(),
        ),
        ("output_carrier_mode", summary.output_carrier_mode.as_str()),
        (
            "output_residency_contract",
            summary.output_residency_contract.as_str(),
        ),
        (
            "output_residency_kind",
            summary.output_residency_kind.as_str(),
        ),
        (
            "output_transfer_scope",
            summary.output_transfer_scope.as_str(),
        ),
        (
            "output_observation_mode",
            summary.output_observation_mode.as_str(),
        ),
        (
            "output_device_retention_status",
            summary.output_device_retention_status.as_str(),
        ),
        (
            "session_registry_contract",
            summary.session_registry_contract.as_str(),
        ),
        (
            "session_registry_source",
            summary.session_registry_source.as_str(),
        ),
        (
            "session_lease_contract",
            summary.session_lease_contract.as_str(),
        ),
        ("session_lease_id", summary.session_lease_id.as_str()),
        ("session_adapter_id", summary.session_adapter_id.as_str()),
        ("session_mode", summary.session_mode.as_str()),
        ("session_continuity", summary.session_continuity.as_str()),
        (
            "session_lifecycle_hooks",
            summary.session_lifecycle_hooks.as_str(),
        ),
        (
            "session_request_sequence",
            summary.session_request_sequence.as_str(),
        ),
        (
            "worker_lease_contract",
            summary.worker_lease_contract.as_str(),
        ),
        (
            "worker_resolver_contract",
            summary.worker_resolver_contract.as_str(),
        ),
        ("worker_cache_status", summary.worker_cache_status.as_str()),
        ("worker_pid", summary.worker_pid.as_str()),
        (
            "worker_request_sequence",
            summary.worker_request_sequence.as_str(),
        ),
        (
            "worker_descriptor_count",
            summary.worker_descriptor_count.as_str(),
        ),
        ("worker_payload_hash", summary.worker_payload_hash.as_str()),
        (
            "worker_operation_token",
            summary.worker_operation_token.as_str(),
        ),
        (
            "worker_execution_capsule_contract",
            summary.worker_execution_capsule_contract.as_str(),
        ),
        (
            "worker_execution_capsule_id",
            summary.worker_execution_capsule_id.as_str(),
        ),
        (
            "worker_execution_capsule_token",
            summary.worker_execution_capsule_token.as_str(),
        ),
        (
            "worker_execution_capsule_invocation_mode",
            summary.worker_execution_capsule_invocation_mode.as_str(),
        ),
        (
            "worker_execution_capsule_input_roles",
            summary.worker_execution_capsule_input_roles.as_str(),
        ),
        (
            "worker_execution_capsule_output_roles",
            summary.worker_execution_capsule_output_roles.as_str(),
        ),
        (
            "worker_execution_capsule_status",
            summary.worker_execution_capsule_status.as_str(),
        ),
        (
            "worker_execution_capsule_invoker_contract",
            summary.worker_execution_capsule_invoker_contract.as_str(),
        ),
        (
            "worker_execution_capsule_invoker_id",
            summary.worker_execution_capsule_invoker_id.as_str(),
        ),
        (
            "worker_execution_capsule_invoker_status",
            summary.worker_execution_capsule_invoker_status.as_str(),
        ),
        (
            "worker_output_descriptor_contract",
            summary.worker_output_descriptor_contract.as_str(),
        ),
        (
            "worker_output_descriptor_roles",
            summary.worker_output_descriptor_roles.as_str(),
        ),
        (
            "worker_output_descriptor_count",
            summary.worker_output_descriptor_count.as_str(),
        ),
        (
            "worker_output_descriptor_byte_length",
            summary.worker_output_descriptor_byte_length.as_str(),
        ),
        (
            "worker_output_descriptor_hash",
            summary.worker_output_descriptor_hash.as_str(),
        ),
        (
            "worker_output_receipt_status",
            summary.worker_output_receipt_status.as_str(),
        ),
        (
            "worker_dispatch_permit_contract",
            summary.worker_dispatch_permit_contract.as_str(),
        ),
        (
            "worker_dispatch_permit_status",
            summary.worker_dispatch_permit_status.as_str(),
        ),
        (
            "worker_dispatch_status",
            summary.worker_dispatch_status.as_str(),
        ),
        (
            "output_handle_contract",
            summary.output_handle_contract.as_str(),
        ),
        ("output_handle_id", summary.output_handle_id.as_str()),
        (
            "output_handle_ownership_token",
            summary.output_handle_ownership_token.as_str(),
        ),
        (
            "output_handle_release_status",
            summary.output_handle_release_status.as_str(),
        ),
        (
            "compute_plan_contract",
            summary.compute_plan_contract.as_str(),
        ),
        ("compute_plan_status", summary.compute_plan_status.as_str()),
        (
            "compute_plan_layer_count",
            summary.compute_plan_layer_count.as_str(),
        ),
        (
            "compute_plan_preferred_devices",
            summary.compute_plan_preferred_devices.as_str(),
        ),
        (
            "compute_plan_supported_devices",
            summary.compute_plan_supported_devices.as_str(),
        ),
        ("comparison_contract", summary.comparison_contract.as_str()),
        ("comparison_status", summary.comparison_status.as_str()),
        (
            "comparison_element_count",
            summary.comparison_element_count.as_str(),
        ),
        (
            "comparison_mismatch_count",
            summary.comparison_mismatch_count.as_str(),
        ),
        (
            "comparison_max_absolute_error",
            summary.comparison_max_absolute_error.as_str(),
        ),
        (
            "comparison_max_relative_error",
            summary.comparison_max_relative_error.as_str(),
        ),
        (
            "comparison_non_finite_count",
            summary.comparison_non_finite_count.as_str(),
        ),
    ] {
        push_toml_string(out, &format!("{prefix}{name}"), value);
    }
}

pub(crate) fn native_output_collection_hash(outputs: &[PixelMagicNativeOutputSummary]) -> String {
    let canonical = outputs
        .iter()
        .enumerate()
        .map(|(index, output)| {
            format!(
                "{index}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{};",
                output.request_id,
                output.hash,
                output.output_carrier_adapter_id,
                output.output_residency_kind,
                output.output_transfer_scope,
                output.output_observation_mode,
                output.output_device_retention_status,
                output.session_lease_id,
                output.output_handle_id,
                output.output_handle_ownership_token,
                output.comparison_contract,
                output.comparison_status
            )
        })
        .collect::<String>();
    fnv1a64_hex(canonical.as_bytes())
}
