use crate::{
    provider_carrier_channel_registry::PreparedProviderCarrierChannel,
    provider_carrier_channel_unix::InheritedFdCarrier,
    provider_output_carrier_registry::{ProviderOutputCarrierConsumption, ProviderOutputPayload},
};
use std::{fs::File, os::fd::OwnedFd};

pub(crate) fn consume_worker_result(
    descriptors: &mut Vec<OwnedFd>,
    mode: &str,
    packet_len: usize,
    packet_hash: &str,
    adapter_protocol: &[u8],
) -> Result<Option<ProviderOutputCarrierConsumption>, String> {
    if mode != "nuispfd1-result" {
        return Ok(None);
    }
    if descriptors.len() != 1 {
        return Err("provider worker direct result requires one descriptor".to_owned());
    }
    let packet_hash = parse_hex_hash(packet_hash, "packet")?;
    let output_hash = parse_protocol_hash(adapter_protocol)?;
    let file = File::from(
        descriptors
            .pop()
            .expect("validated provider worker result descriptor"),
    );
    let carrier = InheritedFdCarrier::from_received_single_frame(file, packet_len, packet_hash)?;
    let (frame, carrier) = carrier.verify_written_output(output_hash)?;
    Ok(Some(ProviderOutputCarrierConsumption {
        payload: Some(ProviderOutputPayload::InheritedFd(frame)),
        transferable: Some(PreparedProviderCarrierChannel::InheritedFd(carrier)),
    }))
}

fn parse_hex_hash(value: &str, kind: &str) -> Result<u64, String> {
    value
        .strip_prefix("0x")
        .filter(|digits| digits.len() == 16)
        .ok_or_else(|| format!("provider worker {kind} hash is invalid"))
        .and_then(|digits| {
            u64::from_str_radix(digits, 16)
                .map_err(|error| format!("provider worker {kind} hash is invalid: {error}"))
        })
}

fn parse_protocol_hash(protocol: &[u8]) -> Result<u64, String> {
    let protocol = std::str::from_utf8(protocol)
        .map_err(|_| "provider worker adapter protocol is not UTF-8".to_owned())?;
    protocol
        .lines()
        .find_map(|line| line.strip_prefix("output_hash="))
        .ok_or_else(|| "provider worker adapter protocol omitted output hash".to_owned())?
        .parse::<u64>()
        .map_err(|error| format!("provider worker output hash is invalid: {error}"))
}
