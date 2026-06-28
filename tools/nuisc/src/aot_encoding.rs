pub(crate) fn fnv1a64_hex(bytes: &[u8]) -> String {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    let mut hash = FNV_OFFSET;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("0x{hash:016x}")
}

pub(crate) fn hex_encode_bytes(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{byte:02x}");
    }
    out
}

pub(crate) fn hex_decode_bytes(value: &str) -> Result<Vec<u8>, String> {
    if value.len() % 2 != 0 {
        return Err("hex payload length must be even".to_owned());
    }
    let mut out = Vec::with_capacity(value.len() / 2);
    let bytes = value.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        let chunk = std::str::from_utf8(&bytes[index..index + 2])
            .map_err(|_| "hex payload is not valid UTF-8".to_owned())?;
        let byte =
            u8::from_str_radix(chunk, 16).map_err(|_| format!("invalid hex byte `{chunk}`"))?;
        out.push(byte);
        index += 2;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fnv1a64_hash_is_stable() {
        assert_eq!(fnv1a64_hex(b"nuis"), "0x5bace3ba528fb0c4");
    }

    #[test]
    fn hex_roundtrip_preserves_bytes() {
        let bytes = b"\x00nuis\xff";
        let encoded = hex_encode_bytes(bytes);

        assert_eq!(encoded, "006e756973ff");
        assert_eq!(hex_decode_bytes(&encoded).as_deref(), Ok(bytes.as_slice()));
    }
}
