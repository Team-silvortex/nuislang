use crate::{
    provider_carrier_channel::PROVIDER_CARRIER_CHANNEL_CONTRACT,
    provider_carrier_channel_registry::{
        select_provider_carrier_channel_adapter, PreparedProviderCarrierChannel,
        PROVIDER_CARRIER_CHANNEL_REGISTRY_CONTRACT, PROVIDER_CARRIER_CHANNEL_REGISTRY_SOURCE,
    },
    provider_carrier_input::{ProviderCarrierInput, PROVIDER_CARRIER_INPUT_CONTRACT},
    provider_edge_staging_registry::{
        cleanup_provider_edge_carrier, consume_provider_edge_carrier,
        materialize_provider_edge_carrier, provider_output_transfer_staging_adapter,
        release_provider_edge_carrier, select_provider_edge_staging_adapter,
        ProviderEdgeStagingAdapter, ProviderEdgeStagingCarrier,
        PROVIDER_EDGE_STAGING_REGISTRY_CONTRACT, PROVIDER_EDGE_STAGING_REGISTRY_SOURCE,
    },
    provider_edge_transport::{ProviderEdgeTransportDescriptor, ProviderEdgeTransportReceipt},
    provider_input_binding::ProviderInputBinding,
    provider_output_carrier_registry::ProviderOutputPayload,
    provider_sample_execute::resolve_provider_payload_path,
    provider_sample_payload::fnv1a64_hex,
};
use std::{collections::BTreeMap, fs, path::Path};

pub(crate) struct CompletedProviderOutput {
    pub(crate) payload: ProviderOutputPayload,
    pub(crate) transferable: Option<PreparedProviderCarrierChannel>,
}

pub(crate) struct PreparedProviderInput {
    artifact_input: Option<ProviderCarrierInput>,
    staging_adapter: Option<ProviderEdgeStagingAdapter>,
    carrier: Option<ProviderEdgeStagingCarrier>,
    direct_channel: Option<PreparedProviderCarrierChannel>,
    transport_receipt: Option<ProviderEdgeTransportReceipt>,
}

impl PreparedProviderInput {
    pub(crate) fn new(
        output_dir: &Path,
        binding: &ProviderInputBinding,
        transport: Option<&ProviderEdgeTransportDescriptor>,
        completed: &BTreeMap<String, CompletedProviderOutput>,
        allow_direct_transfer: bool,
    ) -> Result<Self, String> {
        if binding.source == "dependency" {
            let completed_output =
                completed.get(&binding.producer_request_id).ok_or_else(|| {
                    format!(
                        "provider dependency `{}` has no completed output",
                        binding.producer_request_id
                    )
                })?;
            let bytes = completed_output.payload.as_bytes();
            validate_input_bytes(binding, bytes)?;
            let direct_channel = if allow_direct_transfer {
                completed_output
                    .transferable
                    .as_ref()
                    .map(|carrier| carrier.try_clone_transferable())
                    .transpose()?
                    .flatten()
            } else {
                None
            };
            if let Some(direct_channel) = direct_channel {
                let staging_adapter = provider_output_transfer_staging_adapter();
                return Ok(Self {
                    artifact_input: None,
                    staging_adapter: None,
                    carrier: None,
                    direct_channel: Some(direct_channel),
                    transport_receipt: transport.map(|descriptor| ProviderEdgeTransportReceipt {
                        ownership_token: descriptor.ownership_token.clone(),
                        staging_registry_contract: PROVIDER_EDGE_STAGING_REGISTRY_CONTRACT
                            .to_owned(),
                        staging_registry_source: PROVIDER_EDGE_STAGING_REGISTRY_SOURCE.to_owned(),
                        staging_adapter_id: staging_adapter.adapter_id.to_owned(),
                        staging_adapter_capability_status: staging_adapter
                            .capability_status
                            .to_owned(),
                        carrier_input_contract: PROVIDER_CARRIER_INPUT_CONTRACT.to_owned(),
                        carrier_input_kind: "inherited-frame".to_owned(),
                        carrier_input_handle: format!("output:{}", binding.producer_request_id),
                        carrier_channel_registry_contract:
                            PROVIDER_CARRIER_CHANNEL_REGISTRY_CONTRACT.to_owned(),
                        carrier_channel_registry_source: PROVIDER_CARRIER_CHANNEL_REGISTRY_SOURCE
                            .to_owned(),
                        carrier_channel_adapter_id: "inherited.fd.v1".to_owned(),
                        carrier_channel_adapter_capability_status: "registered-available"
                            .to_owned(),
                        carrier_channel_contract: PROVIDER_CARRIER_CHANNEL_CONTRACT.to_owned(),
                        carrier_channel_mode: "inherited-fd".to_owned(),
                        carrier_identity: format!(
                            "transferred-output:{}",
                            binding.producer_request_id
                        ),
                        byte_length: bytes.len(),
                        materialize_status: "materialized".to_owned(),
                        materialize_payload_hash: fnv1a64_hex(bytes),
                        consume_status: "pending".to_owned(),
                        consume_payload_hash: "pending".to_owned(),
                        release_status: "pending".to_owned(),
                        release_payload_hash: "pending".to_owned(),
                    }),
                });
            }
            let requested_mode = transport
                .map(|descriptor| descriptor.staging_mode.as_str())
                .unwrap_or("host-visible-owned-file");
            let staging_adapter =
                select_provider_edge_staging_adapter(requested_mode).ok_or_else(|| {
                    format!("no provider edge staging adapter supports `{requested_mode}`")
                })?;
            let owner_hash = transport
                .map(|descriptor| fnv1a64_hex(descriptor.ownership_token.as_bytes()))
                .unwrap_or_else(|| "legacy".to_owned());
            let carrier = materialize_provider_edge_carrier(staging_adapter, &owner_hash, bytes)?;
            let channel_adapter = if carrier.input.kind() == "opaque-bytes" {
                Some(
                    select_provider_carrier_channel_adapter("auto").ok_or_else(|| {
                        "no provider carrier channel adapter supports opaque bytes".to_owned()
                    })?,
                )
            } else {
                None
            };
            return Ok(Self {
                artifact_input: None,
                staging_adapter: Some(staging_adapter),
                transport_receipt: transport.map(|descriptor| ProviderEdgeTransportReceipt {
                    ownership_token: descriptor.ownership_token.clone(),
                    staging_registry_contract: PROVIDER_EDGE_STAGING_REGISTRY_CONTRACT.to_owned(),
                    staging_registry_source: PROVIDER_EDGE_STAGING_REGISTRY_SOURCE.to_owned(),
                    staging_adapter_id: staging_adapter.adapter_id.to_owned(),
                    staging_adapter_capability_status: staging_adapter.capability_status.to_owned(),
                    carrier_input_contract: PROVIDER_CARRIER_INPUT_CONTRACT.to_owned(),
                    carrier_input_kind: carrier.input.kind().to_owned(),
                    carrier_input_handle: carrier.input.handle().unwrap_or("none").to_owned(),
                    carrier_channel_registry_contract: PROVIDER_CARRIER_CHANNEL_REGISTRY_CONTRACT
                        .to_owned(),
                    carrier_channel_registry_source: PROVIDER_CARRIER_CHANNEL_REGISTRY_SOURCE
                        .to_owned(),
                    carrier_channel_adapter_id: channel_adapter
                        .map(|adapter| adapter.adapter_id)
                        .unwrap_or("none")
                        .to_owned(),
                    carrier_channel_adapter_capability_status: channel_adapter
                        .map(|adapter| adapter.capability_status)
                        .unwrap_or("not-required")
                        .to_owned(),
                    carrier_channel_contract: PROVIDER_CARRIER_CHANNEL_CONTRACT.to_owned(),
                    carrier_channel_mode: channel_adapter
                        .map(|adapter| adapter.mode)
                        .unwrap_or("not-required")
                        .to_owned(),
                    carrier_identity: carrier.identity.clone(),
                    byte_length: bytes.len(),
                    materialize_status: "materialized".to_owned(),
                    materialize_payload_hash: fnv1a64_hex(bytes),
                    consume_status: "pending".to_owned(),
                    consume_payload_hash: "pending".to_owned(),
                    release_status: "pending".to_owned(),
                    release_payload_hash: "pending".to_owned(),
                }),
                carrier: Some(carrier),
                direct_channel: None,
            });
        }
        let path = resolve_provider_payload_path(output_dir, &binding.payload_path)?;
        let bytes = fs::read(&path).map_err(|error| {
            format!(
                "failed to read provider input buffer `{}`: {error}",
                path.display()
            )
        })?;
        validate_input_bytes(binding, &bytes)?;
        Ok(Self {
            artifact_input: Some(ProviderCarrierInput::Path(path)),
            staging_adapter: None,
            carrier: None,
            direct_channel: None,
            transport_receipt: None,
        })
    }

    pub(crate) fn input(&self) -> &ProviderCarrierInput {
        self.carrier
            .as_ref()
            .map(|carrier| &carrier.input)
            .or(self.artifact_input.as_ref())
            .expect("prepared provider input must own one carrier")
    }

    pub(crate) fn direct_channel(&self) -> Option<&PreparedProviderCarrierChannel> {
        self.direct_channel.as_ref()
    }

    #[cfg(unix)]
    pub(crate) fn try_clone_worker_descriptor(&self) -> Result<Option<fs::File>, String> {
        if let Some(channel) = &self.direct_channel {
            return channel.try_clone_worker_descriptor();
        }
        self.input()
            .path()
            .map(|path| {
                fs::File::open(path).map_err(|error| {
                    format!(
                        "failed to open provider input descriptor `{}`: {error}",
                        path.display()
                    )
                })
            })
            .transpose()
    }

    pub(crate) fn finish(mut self) -> Result<Option<ProviderEdgeTransportReceipt>, String> {
        let mut receipt = self.transport_receipt.take();
        if self.direct_channel.take().is_some() {
            if let Some(receipt) = receipt.as_mut() {
                receipt.consume_status = "consumed".to_owned();
                receipt.consume_payload_hash = receipt.materialize_payload_hash.clone();
                receipt.release_status = "released".to_owned();
                receipt.release_payload_hash = receipt.materialize_payload_hash.clone();
            }
            return Ok(receipt);
        }
        if let (Some(adapter), Some(carrier)) = (self.staging_adapter, self.carrier.as_mut()) {
            let bytes = consume_provider_edge_carrier(adapter, carrier)?;
            let payload_hash = fnv1a64_hex(&bytes);
            if let Some(receipt) = receipt.as_mut() {
                if bytes.len() != receipt.byte_length
                    || payload_hash != receipt.materialize_payload_hash
                {
                    return Err("provider edge carrier changed before consumption".to_owned());
                }
                receipt.consume_status = "consumed".to_owned();
                receipt.consume_payload_hash = payload_hash.clone();
            }
            release_provider_edge_carrier(adapter, carrier)?;
            self.carrier = None;
            if let Some(receipt) = receipt.as_mut() {
                receipt.release_status = "released".to_owned();
                receipt.release_payload_hash = payload_hash;
            }
        }
        Ok(receipt)
    }
}

fn validate_input_bytes(binding: &ProviderInputBinding, bytes: &[u8]) -> Result<(), String> {
    if bytes.len() != binding.byte_length || fnv1a64_hex(bytes) != binding.content_hash {
        return Err(format!(
            "provider input binding `{}` size/hash evidence mismatch",
            binding.name
        ));
    }
    Ok(())
}

impl Drop for PreparedProviderInput {
    fn drop(&mut self) {
        if let Some(carrier) = self.carrier.as_mut() {
            cleanup_provider_edge_carrier(carrier);
        }
    }
}
