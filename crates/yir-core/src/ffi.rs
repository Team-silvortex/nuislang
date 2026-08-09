pub const FFI_SYMBOL_HASH_PREFIX: &str = "fnv1a64:";
pub const FFI_SYMBOL_HASH_CANONICAL_VERSION: &str = "nuis-ffi-symbol-v1";
pub const FFI_MEMORY_CAPABILITY_HASH_CANONICAL_VERSION: &str = "nuis-ffi-memory-v1";

pub fn ffi_symbol_signature_hash(abi: &str, symbol: &str, signature: &str) -> String {
    fnv1a64_token(&ffi_symbol_signature_canonical_input(
        abi, symbol, signature,
    ))
}

pub fn ffi_memory_capability_hash(
    abi: &str,
    symbol: &str,
    signature_hash: &str,
    descriptor: &str,
) -> String {
    fnv1a64_token(&ffi_memory_capability_canonical_input(
        abi,
        symbol,
        signature_hash,
        descriptor,
    ))
}

fn fnv1a64_token(canonical: &str) -> String {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

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

pub fn ffi_memory_capability_canonical_input(
    abi: &str,
    symbol: &str,
    signature_hash: &str,
    descriptor: &str,
) -> String {
    format!(
        "{FFI_MEMORY_CAPABILITY_HASH_CANONICAL_VERSION}|{abi}|{symbol}|{signature_hash}|{descriptor}"
    )
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
        ffi_memory_capability_canonical_input, ffi_memory_capability_hash,
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

    #[test]
    fn ffi_memory_capability_hash_is_stable() {
        let descriptor = "kind=borrowed_utf8,slot=arg:0,length=nul_terminated,mutability=read_only,lifetime=call,destructor=none";
        let signature_hash = ffi_symbol_signature_hash("libc", "puts", "i32(String)");
        assert_eq!(
            ffi_memory_capability_canonical_input("libc", "puts", &signature_hash, descriptor),
            format!("nuis-ffi-memory-v1|libc|puts|{signature_hash}|{descriptor}")
        );
        assert_eq!(
            ffi_memory_capability_hash("libc", "puts", &signature_hash, descriptor),
            "fnv1a64:588094eacdd1e033"
        );
    }
}
