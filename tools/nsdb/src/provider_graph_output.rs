#[cfg(unix)]
use crate::provider_worker_lease::ProviderWorkerOutput;
use crate::{
    provider_carrier_channel_registry::PreparedProviderCarrierChannel,
    provider_output_carrier_registry::ProviderOutputPayload,
    provider_request::{ProviderRequest, PROVIDER_OUTPUT_BINDING_CONTRACT},
    provider_sample_payload::fnv1a64_hex,
    provider_sample_payload::PixelMagicNativeOutputSummary,
};
use std::collections::BTreeMap;

pub(crate) const PROVIDER_GRAPH_OUTPUT_OWNERSHIP_CONTRACT: &str =
    "nuis-provider-graph-output-ownership-v1";
pub(crate) const PROVIDER_COMPLETION_EVIDENCE_CONTRACT: &str =
    "nuis-provider-completion-evidence-v1";
pub(crate) const PROVIDER_COMPLETION_CLOCK_CONTRACT: &str = "nuis-provider-completion-clock-v1";
pub(crate) const PROVIDER_GLM_RELEASE_EVIDENCE_CONTRACT: &str =
    "nuis-provider-glm-release-evidence-v1";

pub(crate) fn bind_output_binding_summary(
    summary: &mut PixelMagicNativeOutputSummary,
    request: &ProviderRequest,
) {
    summary.output_binding_contract = PROVIDER_OUTPUT_BINDING_CONTRACT.to_owned();
    summary.output_binding_count = request.output_bindings.len().to_string();
    summary.output_binding_roles = output_binding_manifest(request, |binding| binding.role.clone());
    summary.output_binding_buffers =
        output_binding_manifest(request, |binding| binding.buffer.clone());
    summary.output_binding_element_types =
        output_binding_manifest(request, |binding| binding.element_type.clone());
    summary.output_binding_shapes = output_binding_manifest(request, |binding| {
        binding
            .shape
            .iter()
            .map(usize::to_string)
            .collect::<Vec<_>>()
            .join("x")
    });
    summary.output_binding_byte_lengths =
        output_binding_manifest(request, |binding| binding.byte_length.to_string());
    summary.output_binding_comparison_ids =
        output_binding_manifest(request, |binding| binding.comparison_id.clone());
}

fn output_binding_manifest(
    request: &ProviderRequest,
    value: impl Fn(&crate::provider_request::ProviderOutputBinding) -> String,
) -> String {
    request
        .output_bindings
        .iter()
        .map(value)
        .collect::<Vec<_>>()
        .join(",")
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct CompletedProviderOutputKey {
    pub(crate) request_id: String,
    pub(crate) output_buffer: String,
}

impl CompletedProviderOutputKey {
    pub(crate) fn new(request_id: &str, output_buffer: &str) -> Self {
        Self {
            request_id: request_id.to_owned(),
            output_buffer: output_buffer.to_owned(),
        }
    }
}

pub(crate) struct CompletedProviderOutput {
    pub(crate) role: String,
    pub(crate) buffer: String,
    pub(crate) payload: ProviderOutputPayload,
    pub(crate) transferable: Option<PreparedProviderCarrierChannel>,
}

pub(crate) struct CompletedProviderOutputs {
    outputs: BTreeMap<CompletedProviderOutputKey, CompletedProviderOutput>,
}

pub(crate) struct ProviderGraphOutputCloseReceipt {
    pub(crate) contract: &'static str,
    pub(crate) released_output_count: usize,
    pub(crate) released_output_roles: String,
    released_outputs: Vec<ReleasedProviderOutput>,
}

struct ReleasedProviderOutput {
    request_id: String,
    role: String,
    buffer: String,
}

impl CompletedProviderOutputs {
    pub(crate) fn new() -> Self {
        Self {
            outputs: BTreeMap::new(),
        }
    }

    pub(crate) fn get(&self, key: &CompletedProviderOutputKey) -> Option<&CompletedProviderOutput> {
        self.outputs.get(key)
    }

    pub(crate) fn insert(
        &mut self,
        request_id: &str,
        output: CompletedProviderOutput,
    ) -> Result<(), String> {
        let key = CompletedProviderOutputKey::new(request_id, &output.buffer);
        if self.outputs.insert(key, output).is_some() {
            return Err(format!(
                "provider request `{request_id}` completed output buffer more than once"
            ));
        }
        Ok(())
    }

    pub(crate) fn close(mut self) -> ProviderGraphOutputCloseReceipt {
        let released_output_count = self.outputs.len();
        let released_output_roles = self
            .outputs
            .values()
            .map(|output| output.role.as_str())
            .collect::<Vec<_>>()
            .join(",");
        let released_outputs = self
            .outputs
            .iter()
            .map(|(key, output)| ReleasedProviderOutput {
                request_id: key.request_id.clone(),
                role: output.role.clone(),
                buffer: output.buffer.clone(),
            })
            .collect();
        self.outputs.clear();
        ProviderGraphOutputCloseReceipt {
            contract: PROVIDER_GRAPH_OUTPUT_OWNERSHIP_CONTRACT,
            released_output_count,
            released_output_roles,
            released_outputs,
        }
    }
}

pub(crate) fn bind_provider_completion_evidence(
    summary: &mut PixelMagicNativeOutputSummary,
    close: &ProviderGraphOutputCloseReceipt,
) -> Result<(), String> {
    let session_sequence = summary
        .session_request_sequence
        .parse::<usize>()
        .map_err(|_| "provider completion has no valid session sequence".to_owned())?;
    let worker_sequence = summary
        .worker_request_sequence
        .parse::<usize>()
        .map_err(|_| "provider completion has no valid worker sequence".to_owned())?;
    if session_sequence != worker_sequence {
        return Err("provider completion session and worker clocks diverged".to_owned());
    }
    let dispatch_status = summary
        .worker_dispatch_status
        .parse::<i64>()
        .map_err(|_| "provider completion has no valid worker dispatch status".to_owned())?;
    if summary.worker_output_receipt_status != "verified" || dispatch_status <= 0 {
        return Err("provider completion has no successful verified worker receipt".to_owned());
    }
    let releases = close
        .released_outputs
        .iter()
        .filter(|output| output.request_id == summary.request_id)
        .collect::<Vec<_>>();
    let expected_count = summary
        .output_binding_count
        .parse::<usize>()
        .map_err(|_| "provider completion has no valid output binding count".to_owned())?;
    let released_roles = releases
        .iter()
        .map(|output| output.role.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let handle_roles = summary
        .output_handle_roles
        .split(',')
        .collect::<std::collections::BTreeSet<_>>();
    if releases.len() != expected_count || released_roles != handle_roles {
        return Err("provider completion release set does not match GLM output handles".to_owned());
    }

    let completion_clock = format!(
        "{PROVIDER_COMPLETION_CLOCK_CONTRACT}:domain={}:session={session_sequence}:worker={worker_sequence}",
        summary.session_lease_id
    );
    let completion_hash = fnv1a64_hex(
        format!(
            "{completion_clock}:{}:{}:{}:{}:{}",
            summary.worker_operation_token,
            summary.worker_execution_capsule_token,
            summary.worker_output_descriptor_roles,
            summary.worker_output_descriptor_hash,
            summary.worker_additional_output_hashes
        )
        .as_bytes(),
    );
    let completion_token = format!("provider-completion:{completion_hash}");
    let release_manifest = releases
        .iter()
        .map(|output| format!("{}={}", output.role, output.buffer))
        .collect::<Vec<_>>()
        .join(",");
    let release_hash = fnv1a64_hex(
        format!(
            "{completion_token}:{}:{}:{release_manifest}",
            close.contract, summary.output_handle_ownership_tokens
        )
        .as_bytes(),
    );

    summary.output_handle_release_status = "released-at-graph-close".to_owned();
    summary.graph_output_ownership_contract = close.contract.to_owned();
    summary.graph_output_release_count = close.released_output_count.to_string();
    summary.graph_output_release_roles = close.released_output_roles.clone();
    summary.completion_evidence_contract = PROVIDER_COMPLETION_EVIDENCE_CONTRACT.to_owned();
    summary.completion_clock_evidence = completion_clock;
    summary.completion_token = completion_token;
    summary.completion_status = "worker-output-verified".to_owned();
    summary.glm_release_contract = PROVIDER_GLM_RELEASE_EVIDENCE_CONTRACT.to_owned();
    summary.glm_release_token = format!("glm-release:{release_hash}");
    summary.glm_release_status = "released-at-graph-close".to_owned();
    Ok(())
}

#[cfg(unix)]
pub(crate) fn completed_additional_worker_outputs(
    request: &ProviderRequest,
    outputs: Vec<ProviderWorkerOutput>,
) -> Result<Vec<CompletedProviderOutput>, String> {
    let bindings = request.output_bindings.iter().skip(1).collect::<Vec<_>>();
    if outputs.len() != bindings.len() {
        return Err(format!(
            "provider request `{}` returned {} additional outputs for {} registered bindings",
            request.kernel.id,
            outputs.len(),
            bindings.len()
        ));
    }
    bindings
        .into_iter()
        .zip(outputs)
        .map(|(binding, output)| {
            if output.role != binding.role {
                return Err(format!(
                    "provider request `{}` returned role `{}` for registered role `{}`",
                    request.kernel.id, output.role, binding.role
                ));
            }
            let (payload, transferable, payload_hash_valid) = match output.result {
                Some(result) => (
                    result.payload.ok_or_else(|| {
                        format!(
                            "provider request `{}` output `{}` omitted its verified payload",
                            request.kernel.id, binding.role
                        )
                    })?,
                    result.transferable,
                    true,
                ),
                None => {
                    let payload_hash_valid = fnv1a64_hex(&output.payload) == output.payload_hash;
                    (
                        ProviderOutputPayload::owned(output.payload),
                        None,
                        payload_hash_valid,
                    )
                }
            };
            if payload.as_bytes().len() != binding.byte_length || !payload_hash_valid {
                return Err(format!(
                    "provider request `{}` output `{}` changed after lease verification",
                    request.kernel.id, binding.role
                ));
            }
            Ok(CompletedProviderOutput {
                role: binding.role.clone(),
                buffer: binding.buffer.clone(),
                payload,
                transferable,
            })
        })
        .collect()
}

#[cfg(test)]
#[path = "provider_graph_output_tests.rs"]
mod tests;
