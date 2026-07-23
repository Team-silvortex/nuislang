use crate::{
    provider_sample_payload::PixelMagicNativeOutputSummary,
    provider_worker_lease::{
        ProviderWorkerAdapterLaunch, ProviderWorkerDispatchReceipt,
        PROVIDER_WORKER_ADAPTER_CONTROL_CONTRACT,
    },
};

pub(crate) fn bind_worker_output(
    summary: &mut PixelMagicNativeOutputSummary,
    receipt: &ProviderWorkerDispatchReceipt,
    adapter_launch: Option<&ProviderWorkerAdapterLaunch<'_>>,
) {
    summary.worker_lease_contract = receipt.lease_contract.to_owned();
    summary.worker_resolver_contract = receipt.resolver_contract.to_owned();
    summary.worker_cache_status = receipt.cache_status.to_owned();
    summary.worker_adapter_control_mode = receipt.adapter_control_mode.to_owned();
    if let Some(launch) = adapter_launch {
        summary.worker_adapter_cache_contract = launch.cache_contract.to_owned();
        summary.worker_adapter_cache_identity = launch.cache_identity.to_owned();
        summary.worker_adapter_cache_status = launch.cache_status.to_owned();
        summary.worker_adapter_executable_hash = launch.executable_hash.to_owned();
        summary.worker_adapter_control_contract =
            PROVIDER_WORKER_ADAPTER_CONTROL_CONTRACT.to_owned();
    } else {
        summary.worker_adapter_cache_contract = "none".to_owned();
        summary.worker_adapter_cache_identity = "none".to_owned();
        summary.worker_adapter_cache_status = "none".to_owned();
        summary.worker_adapter_executable_hash = "none".to_owned();
        summary.worker_adapter_control_contract = "none".to_owned();
    }
    summary.worker_pid = receipt.worker_pid.to_string();
    summary.worker_request_sequence = receipt.sequence.to_string();
    summary.worker_descriptor_count = receipt.descriptor_count.to_string();
    summary.worker_descriptor_capability_contract =
        receipt.descriptor_capability_contract.to_owned();
    summary.worker_max_semantic_descriptors = receipt.max_semantic_descriptors.to_string();
    summary.worker_max_control_descriptors = receipt.max_control_descriptors.to_string();
    summary.worker_output_descriptor_capability_contract =
        receipt.output_descriptor_capability_contract.to_owned();
    summary.worker_max_output_descriptors = receipt.max_output_descriptors.to_string();
    summary.worker_payload_hash = receipt.payload_hash.clone();
    summary.worker_operation_token = receipt.operation_token.clone();
    summary.worker_execution_capsule_contract = receipt.execution_capsule_contract.to_owned();
    summary.worker_execution_capsule_id = receipt.execution_capsule_id.clone();
    summary.worker_execution_capsule_token = receipt.execution_capsule_token.clone();
    summary.worker_execution_capsule_invocation_mode =
        receipt.execution_capsule_invocation_mode.to_owned();
    summary.worker_execution_capsule_input_roles = receipt.execution_capsule_input_roles.clone();
    summary.worker_execution_capsule_output_roles = receipt.execution_capsule_output_roles.clone();
    summary.worker_execution_capsule_status = receipt.execution_capsule_status.to_owned();
    summary.worker_execution_capsule_invoker_contract =
        receipt.execution_capsule_invoker_contract.to_owned();
    summary.worker_execution_capsule_invoker_id = receipt.execution_capsule_invoker_id.clone();
    summary.worker_execution_capsule_invoker_status =
        receipt.execution_capsule_invoker_status.to_owned();
    summary.worker_output_descriptor_contract =
        receipt.worker_output_descriptor_contract.to_owned();
    summary.worker_output_descriptor_roles = receipt.worker_output_descriptor_roles.clone();
    summary.worker_output_descriptor_count = receipt.worker_output_descriptor_count.to_string();
    summary.worker_output_descriptor_byte_length =
        receipt.worker_output_descriptor_byte_length.to_string();
    summary.worker_output_descriptor_hash = receipt.worker_output_descriptor_hash.clone();
    summary.worker_additional_output_roles = receipt
        .additional_worker_outputs
        .iter()
        .map(|output| output.role.as_str())
        .collect::<Vec<_>>()
        .join(",");
    summary.worker_additional_output_byte_lengths = receipt
        .additional_worker_outputs
        .iter()
        .map(|output| output.byte_length.to_string())
        .collect::<Vec<_>>()
        .join(",");
    summary.worker_additional_output_hashes = receipt
        .additional_worker_outputs
        .iter()
        .map(|output| output.payload_hash.as_str())
        .collect::<Vec<_>>()
        .join(",");
    summary.worker_additional_output_retention_statuses = receipt
        .additional_worker_outputs
        .iter()
        .map(|output| output.retention_status())
        .collect::<Vec<_>>()
        .join(",");
    summary.worker_output_receipt_status = receipt.worker_output_receipt_status.to_owned();
    summary.worker_dispatch_permit_contract = receipt.dispatch_permit_contract.to_owned();
    summary.worker_dispatch_permit_status = receipt.dispatch_permit_status.to_owned();
    summary.worker_dispatch_status = receipt.dispatch_status.to_string();
}
