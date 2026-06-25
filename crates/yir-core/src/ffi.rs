pub const FFI_SYMBOL_HASH_PREFIX: &str = "fnv1a64:";
pub const FFI_SYMBOL_HASH_CANONICAL_VERSION: &str = "nuis-ffi-symbol-v1";

pub fn ffi_symbol_signature_hash(abi: &str, symbol: &str, signature: &str) -> String {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let canonical = ffi_symbol_signature_canonical_input(abi, symbol, signature);
    let mut hash = FNV_OFFSET;
    for byte in canonical.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("{FFI_SYMBOL_HASH_PREFIX}{hash:016x}")
}

pub fn ffi_symbol_signature_canonical_input(abi: &str, symbol: &str, signature: &str) -> String {
    format!("{FFI_SYMBOL_HASH_CANONICAL_VERSION}|{abi}|{symbol}|{signature}")
}

pub fn is_ffi_symbol_hash_token(value: &str) -> bool {
    let Some(hex) = value.strip_prefix(FFI_SYMBOL_HASH_PREFIX) else {
        return false;
    };
    hex.len() == 16 && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::{
        ffi_symbol_signature_canonical_input, ffi_symbol_signature_hash, is_ffi_symbol_hash_token,
    };

    #[test]
    fn ffi_symbol_signature_hash_is_stable() {
        assert_eq!(
            ffi_symbol_signature_canonical_input("c", "host_i32_curve", "i32(i32)"),
            "nuis-ffi-symbol-v1|c|host_i32_curve|i32(i32)"
        );
        assert_eq!(
            ffi_symbol_signature_hash("c", "host_i32_curve", "i32(i32)"),
            "fnv1a64:b0042e2b5ee2c2aa"
        );
    }

    #[test]
    fn ffi_symbol_hash_token_validation_is_strict() {
        assert!(is_ffi_symbol_hash_token("fnv1a64:b0042e2b5ee2c2aa"));
        assert!(!is_ffi_symbol_hash_token("sha256:b0042e2b5ee2c2aa"));
        assert!(!is_ffi_symbol_hash_token("fnv1a64:b0042e2b5ee2c2a"));
        assert!(!is_ffi_symbol_hash_token("fnv1a64:b0042e2b5ee2c2ag"));
    }
}
