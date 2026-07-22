use crate::{
    provider_edge_transport::{
        ProviderEdgeTransportReceipt, PROVIDER_EDGE_TRANSPORT_RECEIPT_CONTRACT,
    },
    provider_sample_payload::{fnv1a64_hex, push_toml_string},
};

pub(crate) fn push_transport_receipts(out: &mut String, receipts: &[ProviderEdgeTransportReceipt]) {
    push_toml_string(
        out,
        "provider_edge_transport_receipt_contract",
        PROVIDER_EDGE_TRANSPORT_RECEIPT_CONTRACT,
    );
    push_toml_string(
        out,
        "provider_edge_transport_receipt_count",
        &receipts.len().to_string(),
    );
    let mut collection_evidence = Vec::with_capacity(receipts.len());
    for (index, receipt) in receipts.iter().enumerate() {
        let prefix = format!("provider_edge_transport_receipt_{index}_");
        for (name, value) in [
            ("ownership_token", receipt.ownership_token.as_str()),
            (
                "staging_registry_contract",
                receipt.staging_registry_contract.as_str(),
            ),
            (
                "staging_registry_source",
                receipt.staging_registry_source.as_str(),
            ),
            ("staging_adapter_id", receipt.staging_adapter_id.as_str()),
            (
                "staging_adapter_capability_status",
                receipt.staging_adapter_capability_status.as_str(),
            ),
            (
                "carrier_input_contract",
                receipt.carrier_input_contract.as_str(),
            ),
            ("carrier_input_kind", receipt.carrier_input_kind.as_str()),
            (
                "carrier_input_handle",
                receipt.carrier_input_handle.as_str(),
            ),
            (
                "carrier_channel_registry_contract",
                receipt.carrier_channel_registry_contract.as_str(),
            ),
            (
                "carrier_channel_registry_source",
                receipt.carrier_channel_registry_source.as_str(),
            ),
            (
                "carrier_channel_adapter_id",
                receipt.carrier_channel_adapter_id.as_str(),
            ),
            (
                "carrier_channel_adapter_capability_status",
                receipt.carrier_channel_adapter_capability_status.as_str(),
            ),
            (
                "carrier_channel_contract",
                receipt.carrier_channel_contract.as_str(),
            ),
            (
                "carrier_channel_mode",
                receipt.carrier_channel_mode.as_str(),
            ),
            ("carrier_identity", receipt.carrier_identity.as_str()),
            ("materialize_status", receipt.materialize_status.as_str()),
            (
                "materialize_payload_hash",
                receipt.materialize_payload_hash.as_str(),
            ),
            ("consume_status", receipt.consume_status.as_str()),
            (
                "consume_payload_hash",
                receipt.consume_payload_hash.as_str(),
            ),
            ("release_status", receipt.release_status.as_str()),
            (
                "release_payload_hash",
                receipt.release_payload_hash.as_str(),
            ),
        ] {
            push_toml_string(out, &format!("{prefix}{name}"), value);
        }
        push_toml_string(
            out,
            &format!("{prefix}byte_length"),
            &receipt.byte_length.to_string(),
        );
        collection_evidence.push(format!(
            "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
            receipt.ownership_token,
            receipt.staging_adapter_id,
            receipt.carrier_input_kind,
            receipt.carrier_input_handle,
            receipt.carrier_channel_adapter_id,
            receipt.carrier_channel_mode,
            receipt.carrier_identity,
            receipt.byte_length,
            receipt.materialize_payload_hash,
            receipt.consume_payload_hash,
            receipt.release_payload_hash,
        ));
    }
    push_toml_string(
        out,
        "provider_edge_transport_receipt_collection_hash",
        &fnv1a64_hex(collection_evidence.join(";").as_bytes()),
    );
}
