use std::collections::BTreeMap;

use crate::provider_request::{ProviderRequest, ProviderRequestDependency};

pub(crate) const PROVIDER_EDGE_TRANSPORT_CONTRACT: &str = "nuis-provider-edge-transport-v1";
pub(crate) const PROVIDER_EDGE_TRANSPORT_RECEIPT_CONTRACT: &str =
    "nuis-provider-edge-transport-receipt-v1";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ProviderEdgeTransportDescriptor {
    pub(crate) ownership_token: String,
    pub(crate) staging_mode: String,
    pub(crate) producer_clock_evidence: String,
    pub(crate) consumer_clock_evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderEdgeTransportReceipt {
    pub(crate) ownership_token: String,
    pub(crate) staging_registry_contract: String,
    pub(crate) staging_registry_source: String,
    pub(crate) staging_adapter_id: String,
    pub(crate) staging_adapter_capability_status: String,
    pub(crate) carrier_input_contract: String,
    pub(crate) carrier_input_kind: String,
    pub(crate) carrier_input_handle: String,
    pub(crate) carrier_identity: String,
    pub(crate) byte_length: usize,
    pub(crate) materialize_status: String,
    pub(crate) materialize_payload_hash: String,
    pub(crate) consume_status: String,
    pub(crate) consume_payload_hash: String,
    pub(crate) release_status: String,
    pub(crate) release_payload_hash: String,
}

pub(crate) fn parse_edge_transport(
    fields: &BTreeMap<String, String>,
    prefix: &str,
) -> Option<Option<ProviderEdgeTransportDescriptor>> {
    let Some(contract) = fields.get(&format!("{prefix}transport_contract")) else {
        return Some(None);
    };
    (contract == PROVIDER_EDGE_TRANSPORT_CONTRACT).then_some(())?;
    let descriptor = ProviderEdgeTransportDescriptor {
        ownership_token: fields
            .get(&format!("{prefix}transport_ownership_token"))?
            .clone(),
        staging_mode: fields
            .get(&format!("{prefix}transport_staging_mode"))?
            .clone(),
        producer_clock_evidence: fields
            .get(&format!("{prefix}transport_producer_clock_evidence"))?
            .clone(),
        consumer_clock_evidence: fields
            .get(&format!("{prefix}transport_consumer_clock_evidence"))?
            .clone(),
    };
    (descriptor.ownership_token.starts_with("glm:provider-edge:")
        && matches!(
            descriptor.staging_mode.as_str(),
            "host-visible-owned-file" | "auto"
        )
        && descriptor
            .producer_clock_evidence
            .starts_with("provider-clock:request-")
        && descriptor
            .consumer_clock_evidence
            .starts_with("provider-clock:request-"))
    .then_some(Some(descriptor))
}

pub(crate) fn validate_edge_transport(
    descriptor: &ProviderEdgeTransportDescriptor,
    producer_request_id: &str,
    producer_output_buffer: &str,
    producer_index: usize,
    consumer_request_id: &str,
    consumer_input_buffer: &str,
    consumer_index: usize,
) -> bool {
    descriptor.ownership_token
        == format!(
            "glm:provider-edge:{producer_request_id}:{producer_output_buffer}->{consumer_request_id}:{consumer_input_buffer}"
        )
        && descriptor.producer_clock_evidence
            == format!("provider-clock:request-{producer_index}:completed")
        && descriptor.consumer_clock_evidence
            == format!("provider-clock:request-{consumer_index}:dispatch-ready")
}

pub(crate) fn validate_dependency_transport(
    producer: &ProviderRequest,
    producer_index: usize,
    consumer: &ProviderRequest,
    consumer_index: usize,
    dependency: &ProviderRequestDependency,
) -> bool {
    let crosses_provider = producer
        .adapter_binding
        .as_ref()
        .map(|binding| &binding.provider_family)
        != consumer
            .adapter_binding
            .as_ref()
            .map(|binding| &binding.provider_family);
    dependency
        .transport
        .as_ref()
        .map_or(!crosses_provider, |transport| {
            validate_edge_transport(
                transport,
                &producer.kernel.id,
                &dependency.producer_output_buffer,
                producer_index,
                &consumer.kernel.id,
                &dependency.consumer_input_buffer,
                consumer_index,
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_validates_bound_edge_transport() {
        let prefix = "edge_";
        let fields = BTreeMap::from([
            (
                format!("{prefix}transport_contract"),
                PROVIDER_EDGE_TRANSPORT_CONTRACT.to_owned(),
            ),
            (
                format!("{prefix}transport_ownership_token"),
                "glm:provider-edge:a:out->b:in".to_owned(),
            ),
            (
                format!("{prefix}transport_staging_mode"),
                "host-visible-owned-file".to_owned(),
            ),
            (
                format!("{prefix}transport_producer_clock_evidence"),
                "provider-clock:request-0:completed".to_owned(),
            ),
            (
                format!("{prefix}transport_consumer_clock_evidence"),
                "provider-clock:request-1:dispatch-ready".to_owned(),
            ),
        ]);
        let descriptor = parse_edge_transport(&fields, prefix)
            .expect("valid descriptor")
            .expect("transport");
        assert!(validate_edge_transport(
            &descriptor,
            "a",
            "out",
            0,
            "b",
            "in",
            1
        ));
        assert!(!validate_edge_transport(
            &descriptor,
            "a",
            "out",
            1,
            "b",
            "in",
            0
        ));
    }
}
