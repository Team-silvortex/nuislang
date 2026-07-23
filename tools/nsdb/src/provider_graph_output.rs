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
        self.outputs.clear();
        ProviderGraphOutputCloseReceipt {
            contract: PROVIDER_GRAPH_OUTPUT_OWNERSHIP_CONTRACT,
            released_output_count,
            released_output_roles,
        }
    }
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
