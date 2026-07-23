use crate::{
    provider_sample_payload::PixelMagicNativeOutputSummary,
    provider_session_registry::{
        ProviderSessionRequest, PROVIDER_OUTPUT_HANDLE_CONTRACT, PROVIDER_SESSION_LEASE_CONTRACT,
        PROVIDER_SESSION_REGISTRY_CONTRACT, PROVIDER_SESSION_REGISTRY_SOURCE,
    },
};

pub(crate) fn bind_session_output(
    summary: &mut PixelMagicNativeOutputSummary,
    request: &ProviderSessionRequest,
) {
    summary.session_registry_contract = PROVIDER_SESSION_REGISTRY_CONTRACT.to_owned();
    summary.session_registry_source = PROVIDER_SESSION_REGISTRY_SOURCE.to_owned();
    summary.session_lease_contract = PROVIDER_SESSION_LEASE_CONTRACT.to_owned();
    summary.session_lease_id = request.lease_id.clone();
    summary.session_adapter_id = request.session_adapter_id.to_owned();
    summary.session_mode = request.session_mode.to_owned();
    summary.session_continuity = request.session_continuity.to_owned();
    summary.session_lifecycle_hooks = request.session_lifecycle_hooks.to_owned();
    summary.session_request_sequence = request.sequence.to_string();
    summary.output_handle_contract = PROVIDER_OUTPUT_HANDLE_CONTRACT.to_owned();
    summary.output_handle_id = request.output_handle_id.clone();
    summary.output_handle_ownership_token = request.output_ownership_token.clone();
    summary.output_handle_roles = request
        .output_handles
        .iter()
        .map(|handle| handle.role.as_str())
        .collect::<Vec<_>>()
        .join(",");
    summary.output_handle_ids = request
        .output_handles
        .iter()
        .map(|handle| handle.handle_id.as_str())
        .collect::<Vec<_>>()
        .join(",");
    summary.output_handle_ownership_tokens = request
        .output_handles
        .iter()
        .map(|handle| handle.ownership_token.as_str())
        .collect::<Vec<_>>()
        .join(",");
    summary.output_handle_release_status = "lease-bound".to_owned();
}
