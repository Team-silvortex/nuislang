pub(crate) const PROVIDER_CARRIER_CHANNEL_CONTRACT: &str = "nuis-provider-carrier-channel-v1";
pub(crate) const PROVIDER_CARRIER_CHANNEL_MAGIC: &[u8; 8] = b"NUISPCV1";

pub(crate) fn encode_provider_carrier_frames(frames: &[&[u8]]) -> Result<Vec<u8>, String> {
    let frame_count = u32::try_from(frames.len())
        .map_err(|_| "provider carrier channel has too many frames".to_owned())?;
    let mut out = Vec::new();
    out.extend_from_slice(PROVIDER_CARRIER_CHANNEL_MAGIC);
    out.extend_from_slice(&frame_count.to_le_bytes());
    for (index, payload) in frames.iter().enumerate() {
        let index =
            u32::try_from(index).map_err(|_| "provider carrier frame index overflow".to_owned())?;
        let byte_length = u64::try_from(payload.len())
            .map_err(|_| "provider carrier frame byte length overflow".to_owned())?;
        out.extend_from_slice(&index.to_le_bytes());
        out.extend_from_slice(&byte_length.to_le_bytes());
        out.extend_from_slice(&fnv1a64(payload).to_le_bytes());
        out.extend_from_slice(payload);
    }
    Ok(out)
}

pub(crate) fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_ordered_length_and_hash_bound_frames() {
        let packet = encode_provider_carrier_frames(&[b"nuis", b"lang"]).unwrap();
        assert_eq!(&packet[..8], PROVIDER_CARRIER_CHANNEL_MAGIC);
        assert_eq!(u32::from_le_bytes(packet[8..12].try_into().unwrap()), 2);
        assert_eq!(u32::from_le_bytes(packet[12..16].try_into().unwrap()), 0);
        assert_eq!(u64::from_le_bytes(packet[16..24].try_into().unwrap()), 4);
        assert_eq!(
            u64::from_le_bytes(packet[24..32].try_into().unwrap()),
            fnv1a64(b"nuis")
        );
        assert_eq!(&packet[32..36], b"nuis");
    }
}
