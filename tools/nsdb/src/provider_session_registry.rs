use crate::provider_sample_payload::fnv1a64_hex;
use std::collections::BTreeSet;

pub(crate) const PROVIDER_SESSION_REGISTRY_CONTRACT: &str = "nuis-provider-session-registry-v1";
pub(crate) const PROVIDER_SESSION_REGISTRY_SOURCE: &str = "builtin-provider-session-registry";
pub(crate) const PROVIDER_SESSION_LEASE_CONTRACT: &str = "nuis-provider-session-lease-v1";
pub(crate) const PROVIDER_OUTPUT_HANDLE_CONTRACT: &str = "nuis-provider-output-handle-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ProviderSessionAdapter {
    pub(crate) adapter_id: &'static str,
    pub(crate) mode: &'static str,
    pub(crate) continuity: &'static str,
    pub(crate) lifecycle_hooks: &'static str,
    pub(crate) device_handle_retention_status: &'static str,
}

const LOGICAL_REQUEST_PROCESS: ProviderSessionAdapter = ProviderSessionAdapter {
    adapter_id: "logical.request-process.v1",
    mode: "logical-request-process",
    continuity: "graph-lease-only",
    lifecycle_hooks: "graph-open,request-begin,request-complete,graph-close",
    device_handle_retention_status: "unsupported",
};

pub(crate) fn select_provider_session_adapter(
    runner_execution_mode: &str,
) -> Option<ProviderSessionAdapter> {
    (runner_execution_mode == "real-device-provider-runner").then_some(LOGICAL_REQUEST_PROCESS)
}

pub(crate) struct ProviderSessionLease {
    lease_id: String,
    provider_family: String,
    adapter: ProviderSessionAdapter,
    next_sequence: usize,
    active_request: Option<String>,
    status: &'static str,
}

pub(crate) struct ProviderSessionRequest {
    pub(crate) lease_id: String,
    pub(crate) session_adapter_id: &'static str,
    pub(crate) session_mode: &'static str,
    pub(crate) session_continuity: &'static str,
    pub(crate) session_lifecycle_hooks: &'static str,
    pub(crate) sequence: usize,
    pub(crate) output_handle_id: String,
    pub(crate) output_ownership_token: String,
    pub(crate) output_handles: Vec<ProviderSessionOutputHandle>,
}

pub(crate) struct ProviderSessionOutputHandle {
    pub(crate) role: String,
    pub(crate) handle_id: String,
    pub(crate) ownership_token: String,
}

impl ProviderSessionLease {
    pub(crate) fn open(
        trace_id: &str,
        provider_family: &str,
        adapter: ProviderSessionAdapter,
    ) -> Self {
        let lease_hash =
            fnv1a64_hex(format!("{trace_id}:{provider_family}:{}", adapter.adapter_id).as_bytes());
        Self {
            lease_id: format!("provider-session:{provider_family}:{lease_hash}"),
            provider_family: provider_family.to_owned(),
            adapter,
            next_sequence: 0,
            active_request: None,
            status: "open",
        }
    }

    pub(crate) fn begin_request_with_output_roles(
        &mut self,
        request_id: &str,
        output_roles: &[String],
    ) -> Result<ProviderSessionRequest, String> {
        if self.status != "open" || self.active_request.is_some() {
            return Err("provider session lease cannot begin another request".to_owned());
        }
        let unique_roles = output_roles.iter().collect::<BTreeSet<_>>();
        if output_roles.is_empty()
            || output_roles.len() > 8
            || unique_roles.len() != output_roles.len()
            || output_roles.iter().any(|role| !is_output_role(role))
        {
            return Err("provider session output roles are invalid".to_owned());
        }
        let sequence = self.next_sequence;
        self.active_request = Some(request_id.to_owned());
        let output_handles = output_roles
            .iter()
            .map(|role| ProviderSessionOutputHandle {
                role: role.clone(),
                handle_id: format!("{}:output:{sequence}:{request_id}:{role}", self.lease_id),
                ownership_token: format!(
                    "glm:provider-session-output:{}:{sequence}:{request_id}:{role}",
                    self.provider_family
                ),
            })
            .collect::<Vec<_>>();
        Ok(ProviderSessionRequest {
            lease_id: self.lease_id.clone(),
            session_adapter_id: self.adapter.adapter_id,
            session_mode: self.adapter.mode,
            session_continuity: self.adapter.continuity,
            session_lifecycle_hooks: self.adapter.lifecycle_hooks,
            sequence,
            output_handle_id: output_handles[0].handle_id.clone(),
            output_ownership_token: output_handles[0].ownership_token.clone(),
            output_handles,
        })
    }

    pub(crate) fn complete_request(&mut self, request_id: &str) -> Result<(), String> {
        if self.active_request.as_deref() != Some(request_id) {
            return Err("provider session request completion is out of order".to_owned());
        }
        self.active_request = None;
        self.next_sequence += 1;
        Ok(())
    }

    pub(crate) fn close(&mut self) -> Result<(), String> {
        if self.status != "open" || self.active_request.is_some() {
            return Err("provider session lease cannot close with an active request".to_owned());
        }
        self.status = "closed";
        Ok(())
    }
}

fn is_output_role(value: &str) -> bool {
    value.starts_with("output.")
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lease_orders_requests_and_owns_output_handles() {
        let adapter =
            select_provider_session_adapter("real-device-provider-runner").expect("adapter");
        let mut lease = ProviderSessionLease::open("trace", "coreml:apple-ane", adapter);
        let roles = ["output.result".to_owned()];
        let first = lease
            .begin_request_with_output_roles("affine", &roles)
            .expect("first");
        assert_eq!(first.sequence, 0);
        assert!(first.output_ownership_token.starts_with("glm:"));
        assert!(lease
            .begin_request_with_output_roles("add", &roles)
            .is_err());
        lease.complete_request("affine").expect("complete");
        assert_eq!(
            lease
                .begin_request_with_output_roles("add", &roles)
                .expect("second")
                .sequence,
            1
        );
    }

    #[test]
    fn request_process_adapter_does_not_claim_device_retention() {
        let adapter =
            select_provider_session_adapter("real-device-provider-runner").expect("adapter");
        assert_eq!(adapter.continuity, "graph-lease-only");
        assert_eq!(adapter.device_handle_retention_status, "unsupported");
        assert!(select_provider_session_adapter("host-fallback").is_none());
    }

    #[test]
    fn request_allocates_ordered_role_bound_output_handles() {
        let adapter =
            select_provider_session_adapter("real-device-provider-runner").expect("adapter");
        let mut lease = ProviderSessionLease::open("trace", "data:host", adapter);
        let request = lease
            .begin_request_with_output_roles(
                "fan-out",
                &["output.primary".to_owned(), "output.audit".to_owned()],
            )
            .expect("multi-output request");
        assert_eq!(request.output_handles.len(), 2);
        assert_eq!(
            request
                .output_handles
                .iter()
                .map(|handle| handle.role.as_str())
                .collect::<Vec<_>>(),
            ["output.primary", "output.audit"]
        );
        assert_ne!(
            request.output_handles[0].ownership_token,
            request.output_handles[1].ownership_token
        );
    }
}
