use crate::{
    provider_carrier_channel_registry::PreparedProviderCarrierChannel,
    provider_output_carrier_registry::ProviderOutputPayload,
    provider_request::ProviderRequest,
    provider_sample_payload::{
        fnv1a64_hex, pixelmagic_native_output_summary, PixelMagicNativeOutputSummary,
    },
    provider_worker_lease::ProviderWorkerDispatchReceipt,
};

pub(crate) fn take_provider_worker_native_output(
    input_evidence: &str,
    provider_family: &str,
    request: &ProviderRequest,
    worker_receipt: &mut ProviderWorkerDispatchReceipt,
) -> Result<
    (
        PixelMagicNativeOutputSummary,
        ProviderOutputPayload,
        Option<PreparedProviderCarrierChannel>,
    ),
    String,
> {
    let (output_payload, transferable_output) = match worker_receipt.worker_output_result.take() {
        Some(result) => (
            result.payload.ok_or_else(|| {
                format!(
                    "provider worker request `{}` omitted its primary payload",
                    request.kernel.id
                )
            })?,
            result.transferable,
        ),
        None => (
            ProviderOutputPayload::owned(std::mem::take(&mut worker_receipt.worker_output_payload)),
            None,
        ),
    };
    let summary = provider_worker_native_output_summary(
        input_evidence,
        provider_family,
        request,
        output_payload.as_bytes(),
    )?;
    Ok((summary, output_payload, transferable_output))
}

fn provider_worker_native_output_summary(
    input_evidence: &str,
    provider_family: &str,
    request: &ProviderRequest,
    payload: &[u8],
) -> Result<PixelMagicNativeOutputSummary, String> {
    let mut summary = pixelmagic_native_output_summary(input_evidence, provider_family)
        .ok_or_else(|| "provider worker native output has no registered request".to_owned())?;
    summary.request_id = request.kernel.id.clone();
    summary.kind = format!(
        "provider-worker-output-{}",
        request.output_bindings[0].element_type
    );
    summary.status = "provider-worker-native-output-ready".to_owned();
    summary.bytes = payload.len().to_string();
    summary.hash = fnv1a64_hex(payload);
    summary.execution_contract = "nuis-provider-worker-native-execution-v1".to_owned();
    summary.execution_status = "provider-worker-native-completed".to_owned();
    summary.device = provider_family.to_owned();
    summary.output_carrier_registry_contract =
        crate::provider_output_carrier_registry::PROVIDER_OUTPUT_CARRIER_REGISTRY_CONTRACT
            .to_owned();
    summary.output_carrier_registry_source =
        crate::provider_output_carrier_registry::PROVIDER_OUTPUT_CARRIER_REGISTRY_SOURCE.to_owned();
    summary.output_carrier_adapter_id = "provider-worker.descriptor.v1".to_owned();
    summary.output_carrier_mode = "protocol-stdout".to_owned();
    summary.output_residency_contract =
        crate::provider_output_carrier_registry::PROVIDER_OUTPUT_RESIDENCY_CONTRACT.to_owned();
    summary.output_residency_kind = "host-owned-bytes".to_owned();
    summary.output_transfer_scope = "graph-owned".to_owned();
    summary.output_observation_mode = "verified-descriptor".to_owned();
    summary.output_device_retention_status = "unsupported".to_owned();
    Ok(summary)
}
