pub const FFI_SYMBOL_HASH_PREFIX: &str = "fnv1a64:";
pub const FFI_SYMBOL_HASH_CANONICAL_VERSION: &str = "nuis-ffi-symbol-v1";
pub const FFI_MEMORY_CAPABILITY_HASH_CANONICAL_VERSION: &str = "nuis-ffi-memory-v1";
pub const OWNED_BUFFER_RETURN_PROTOCOL: &str = "nuis-ffi-owned-buffer-v1";
pub const OWNED_BUFFER_RETURN_LENGTH_POLICY: &str = "runtime_header";
pub const OWNED_BUFFER_RETURN_METADATA_LEN: usize = 9;
pub const OWNED_BUFFER_DESTRUCTOR_SIGNATURE: &str = "i64(ref_Buffer)";
pub const OWNED_UTF8_RETURN_PROTOCOL: &str = "nuis-ffi-owned-utf8-v1";
pub const OWNED_UTF8_RETURN_LENGTH_POLICY: &str = "runtime_header";
pub const OWNED_UTF8_RETURN_METADATA_LEN: usize = 9;
pub const OWNED_UTF8_DESTRUCTOR_SIGNATURE: &str = "i64(ref_String)";
pub const OWNED_OBJECT_RETURN_PROTOCOL: &str = "nuis-ffi-owned-object-v1";
pub const OWNED_OBJECT_RETURN_SIZE_POLICY: &str = "static:16";
pub const OWNED_OBJECT_RETURN_READ_POLICY: &str = "i64_slots";
pub const OWNED_OBJECT_RETURN_METADATA_LEN: usize = 10;
pub const OWNED_OBJECT_DESTRUCTOR_SIGNATURE: &str = "i64(ref_FfiObject)";
pub const OWNED_BUFFER_BRANCH_TRANSFER_ACTION: &str = "take_owned_buffer_drop_other_v1";
pub const OWNED_BUFFER_FUNCTION_TRANSFER_PROTOCOL: &str =
    "nuis-ffi-owned-buffer-function-transfer-v1";
pub const OWNED_BUFFER_FUNCTION_TRANSFER_METADATA_LEN: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OwnedBufferReturnContract<'a> {
    pub abi: &'a str,
    pub symbol: &'a str,
    pub signature: &'a str,
    pub signature_hash: &'a str,
    pub capability_hash: &'a str,
    pub destructor_symbol: &'a str,
    pub destructor_signature_hash: &'a str,
    pub inputs: &'a [String],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OwnedUtf8ReturnContract<'a> {
    pub abi: &'a str,
    pub symbol: &'a str,
    pub signature: &'a str,
    pub signature_hash: &'a str,
    pub capability_hash: &'a str,
    pub destructor_symbol: &'a str,
    pub destructor_signature_hash: &'a str,
    pub inputs: &'a [String],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OwnedObjectReturnContract<'a> {
    pub abi: &'a str,
    pub symbol: &'a str,
    pub signature: &'a str,
    pub signature_hash: &'a str,
    pub capability_hash: &'a str,
    pub size_policy: &'a str,
    pub read_policy: &'a str,
    pub destructor_symbol: &'a str,
    pub destructor_signature_hash: &'a str,
    pub inputs: &'a [String],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OwnedBufferFunctionTransferContract<'a> {
    pub abi: &'a str,
    pub destructor_symbol: &'a str,
    pub destructor_signature_hash: &'a str,
    pub inputs: &'a [String],
}

pub fn owned_buffer_function_transfer_metadata(
    abi: &str,
    destructor_symbol: &str,
    destructor_signature_hash: &str,
) -> [String; OWNED_BUFFER_FUNCTION_TRANSFER_METADATA_LEN] {
    [
        OWNED_BUFFER_FUNCTION_TRANSFER_PROTOCOL.to_owned(),
        abi.to_owned(),
        destructor_symbol.to_owned(),
        destructor_signature_hash.to_owned(),
    ]
}

pub fn parse_owned_buffer_function_transfer_contract(
    args: &[String],
) -> Result<OwnedBufferFunctionTransferContract<'_>, String> {
    if args.len() < OWNED_BUFFER_FUNCTION_TRANSFER_METADATA_LEN {
        return Err(format!(
            "owned FFI buffer function transfer expects at least {OWNED_BUFFER_FUNCTION_TRANSFER_METADATA_LEN} metadata arguments, found {}",
            args.len()
        ));
    }
    if args[0] != OWNED_BUFFER_FUNCTION_TRANSFER_PROTOCOL {
        return Err(format!(
            "owned FFI buffer function transfer protocol must be `{OWNED_BUFFER_FUNCTION_TRANSFER_PROTOCOL}`, found `{}`",
            args[0]
        ));
    }
    if args[1].is_empty() || args[2].is_empty() {
        return Err(
            "owned FFI buffer function transfer contains an empty ABI or destructor symbol"
                .to_owned(),
        );
    }
    if !is_ffi_symbol_hash_token(&args[3]) {
        return Err(format!(
            "owned FFI buffer function transfer destructor signature hash `{}` is malformed",
            args[3]
        ));
    }
    let expected_hash =
        ffi_symbol_signature_hash(&args[1], &args[2], OWNED_BUFFER_DESTRUCTOR_SIGNATURE);
    if args[3] != expected_hash {
        return Err(format!(
            "owned FFI buffer function transfer destructor signature hash mismatch: expected `{expected_hash}`, found `{}`",
            args[3]
        ));
    }
    Ok(OwnedBufferFunctionTransferContract {
        abi: &args[1],
        destructor_symbol: &args[2],
        destructor_signature_hash: &args[3],
        inputs: &args[OWNED_BUFFER_FUNCTION_TRANSFER_METADATA_LEN..],
    })
}

pub fn owned_buffer_return_descriptor(
    destructor_symbol: &str,
    destructor_signature_hash: &str,
) -> String {
    format!(
        "kind=owned_return_buffer,slot=return,length={OWNED_BUFFER_RETURN_LENGTH_POLICY},mutability=unique,lifetime=owned,destructor={destructor_symbol}@{destructor_signature_hash}"
    )
}

pub fn parse_owned_buffer_return_contract(
    args: &[String],
) -> Result<OwnedBufferReturnContract<'_>, String> {
    if args.len() < OWNED_BUFFER_RETURN_METADATA_LEN {
        return Err(format!(
            "owned FFI buffer call expects at least {OWNED_BUFFER_RETURN_METADATA_LEN} contract arguments, found {}",
            args.len()
        ));
    }
    if args[0] != OWNED_BUFFER_RETURN_PROTOCOL {
        return Err(format!(
            "owned FFI buffer call protocol must be `{OWNED_BUFFER_RETURN_PROTOCOL}`, found `{}`",
            args[0]
        ));
    }
    if args[1].is_empty() || args[2].is_empty() || args[3].is_empty() || args[7].is_empty() {
        return Err("owned FFI buffer call contract contains an empty ABI, symbol, signature, or destructor symbol".to_owned());
    }
    if args[6] != OWNED_BUFFER_RETURN_LENGTH_POLICY {
        return Err(format!(
            "owned FFI buffer call length policy must be `{OWNED_BUFFER_RETURN_LENGTH_POLICY}`, found `{}`",
            args[6]
        ));
    }
    let expected_signature_hash = ffi_symbol_signature_hash(&args[1], &args[2], &args[3]);
    if args[4] != expected_signature_hash {
        return Err(format!(
            "owned FFI buffer call signature hash mismatch: expected `{expected_signature_hash}`, found `{}`",
            args[4]
        ));
    }
    if !is_ffi_symbol_hash_token(&args[8]) {
        return Err(format!(
            "owned FFI buffer call destructor signature hash `{}` is malformed",
            args[8]
        ));
    }
    let expected_destructor_hash =
        ffi_symbol_signature_hash(&args[1], &args[7], OWNED_BUFFER_DESTRUCTOR_SIGNATURE);
    if args[8] != expected_destructor_hash {
        return Err(format!(
            "owned FFI buffer call destructor signature hash mismatch: expected `{expected_destructor_hash}`, found `{}`",
            args[8]
        ));
    }
    let descriptor = owned_buffer_return_descriptor(&args[7], &args[8]);
    let expected_capability_hash =
        ffi_memory_capability_hash(&args[1], &args[2], &args[4], &descriptor);
    if args[5] != expected_capability_hash {
        return Err(format!(
            "owned FFI buffer call capability hash mismatch: expected `{expected_capability_hash}`, found `{}`",
            args[5]
        ));
    }
    Ok(OwnedBufferReturnContract {
        abi: &args[1],
        symbol: &args[2],
        signature: &args[3],
        signature_hash: &args[4],
        capability_hash: &args[5],
        destructor_symbol: &args[7],
        destructor_signature_hash: &args[8],
        inputs: &args[OWNED_BUFFER_RETURN_METADATA_LEN..],
    })
}

pub fn owned_utf8_return_descriptor(
    destructor_symbol: &str,
    destructor_signature_hash: &str,
) -> String {
    format!(
        "kind=owned_return_utf8,slot=return,length={OWNED_UTF8_RETURN_LENGTH_POLICY},mutability=read_only,lifetime=owned,destructor={destructor_symbol}@{destructor_signature_hash}"
    )
}

pub fn parse_owned_utf8_return_contract(
    args: &[String],
) -> Result<OwnedUtf8ReturnContract<'_>, String> {
    if args.len() < OWNED_UTF8_RETURN_METADATA_LEN {
        return Err(format!(
            "owned FFI UTF-8 call expects at least {OWNED_UTF8_RETURN_METADATA_LEN} contract arguments, found {}",
            args.len()
        ));
    }
    if args[0] != OWNED_UTF8_RETURN_PROTOCOL {
        return Err(format!(
            "owned FFI UTF-8 call protocol must be `{OWNED_UTF8_RETURN_PROTOCOL}`, found `{}`",
            args[0]
        ));
    }
    if args[1].is_empty() || args[2].is_empty() || args[3].is_empty() || args[7].is_empty() {
        return Err("owned FFI UTF-8 call contract contains an empty ABI, symbol, signature, or destructor symbol".to_owned());
    }
    if args[6] != OWNED_UTF8_RETURN_LENGTH_POLICY {
        return Err(format!(
            "owned FFI UTF-8 call length policy must be `{OWNED_UTF8_RETURN_LENGTH_POLICY}`, found `{}`",
            args[6]
        ));
    }
    let expected_signature_hash = ffi_symbol_signature_hash(&args[1], &args[2], &args[3]);
    if args[4] != expected_signature_hash {
        return Err(format!(
            "owned FFI UTF-8 call signature hash mismatch: expected `{expected_signature_hash}`, found `{}`",
            args[4]
        ));
    }
    if !is_ffi_symbol_hash_token(&args[8]) {
        return Err(format!(
            "owned FFI UTF-8 call destructor signature hash `{}` is malformed",
            args[8]
        ));
    }
    let expected_destructor_hash =
        ffi_symbol_signature_hash(&args[1], &args[7], OWNED_UTF8_DESTRUCTOR_SIGNATURE);
    if args[8] != expected_destructor_hash {
        return Err(format!(
            "owned FFI UTF-8 call destructor signature hash mismatch: expected `{expected_destructor_hash}`, found `{}`",
            args[8]
        ));
    }
    let descriptor = owned_utf8_return_descriptor(&args[7], &args[8]);
    let expected_capability_hash =
        ffi_memory_capability_hash(&args[1], &args[2], &args[4], &descriptor);
    if args[5] != expected_capability_hash {
        return Err(format!(
            "owned FFI UTF-8 call capability hash mismatch: expected `{expected_capability_hash}`, found `{}`",
            args[5]
        ));
    }
    Ok(OwnedUtf8ReturnContract {
        abi: &args[1],
        symbol: &args[2],
        signature: &args[3],
        signature_hash: &args[4],
        capability_hash: &args[5],
        destructor_symbol: &args[7],
        destructor_signature_hash: &args[8],
        inputs: &args[OWNED_UTF8_RETURN_METADATA_LEN..],
    })
}

pub fn owned_object_return_descriptor(
    destructor_symbol: &str,
    destructor_signature_hash: &str,
) -> String {
    format!(
        "kind=owned_return_object,slot=return,size={OWNED_OBJECT_RETURN_SIZE_POLICY},read={OWNED_OBJECT_RETURN_READ_POLICY},mutability=read_only,lifetime=owned,destructor={destructor_symbol}@{destructor_signature_hash}"
    )
}

pub fn parse_owned_object_return_contract(
    args: &[String],
) -> Result<OwnedObjectReturnContract<'_>, String> {
    if args.len() < OWNED_OBJECT_RETURN_METADATA_LEN {
        return Err(format!(
            "owned FFI object call expects at least {OWNED_OBJECT_RETURN_METADATA_LEN} contract arguments, found {}",
            args.len()
        ));
    }
    if args[0] != OWNED_OBJECT_RETURN_PROTOCOL {
        return Err(format!(
            "owned FFI object call protocol must be `{OWNED_OBJECT_RETURN_PROTOCOL}`, found `{}`",
            args[0]
        ));
    }
    if args[1].is_empty() || args[2].is_empty() || args[3].is_empty() || args[8].is_empty() {
        return Err("owned FFI object call contract contains an empty ABI, symbol, signature, or destructor symbol".to_owned());
    }
    if args[6] != OWNED_OBJECT_RETURN_SIZE_POLICY {
        return Err(format!(
            "owned FFI object size policy must be `{OWNED_OBJECT_RETURN_SIZE_POLICY}`, found `{}`",
            args[6]
        ));
    }
    if args[7] != OWNED_OBJECT_RETURN_READ_POLICY {
        return Err(format!(
            "owned FFI object read policy must be `{OWNED_OBJECT_RETURN_READ_POLICY}`, found `{}`",
            args[7]
        ));
    }
    let expected_signature_hash = ffi_symbol_signature_hash(&args[1], &args[2], &args[3]);
    if args[4] != expected_signature_hash {
        return Err(format!(
            "owned FFI object signature hash mismatch: expected `{expected_signature_hash}`, found `{}`",
            args[4]
        ));
    }
    if !is_ffi_symbol_hash_token(&args[9]) {
        return Err(format!(
            "owned FFI object destructor signature hash `{}` is malformed",
            args[9]
        ));
    }
    let expected_destructor_hash =
        ffi_symbol_signature_hash(&args[1], &args[8], OWNED_OBJECT_DESTRUCTOR_SIGNATURE);
    if args[9] != expected_destructor_hash {
        return Err(format!(
            "owned FFI object destructor signature hash mismatch: expected `{expected_destructor_hash}`, found `{}`",
            args[9]
        ));
    }
    let descriptor = owned_object_return_descriptor(&args[8], &args[9]);
    let expected_capability_hash =
        ffi_memory_capability_hash(&args[1], &args[2], &args[4], &descriptor);
    if args[5] != expected_capability_hash {
        return Err(format!(
            "owned FFI object capability hash mismatch: expected `{expected_capability_hash}`, found `{}`",
            args[5]
        ));
    }
    Ok(OwnedObjectReturnContract {
        abi: &args[1],
        symbol: &args[2],
        signature: &args[3],
        signature_hash: &args[4],
        capability_hash: &args[5],
        size_policy: &args[6],
        read_policy: &args[7],
        destructor_symbol: &args[8],
        destructor_signature_hash: &args[9],
        inputs: &args[OWNED_OBJECT_RETURN_METADATA_LEN..],
    })
}

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
        owned_buffer_return_descriptor, owned_object_return_descriptor,
        owned_utf8_return_descriptor, parse_owned_buffer_function_transfer_contract,
        parse_owned_buffer_return_contract, parse_owned_object_return_contract,
        parse_owned_utf8_return_contract, OWNED_BUFFER_DESTRUCTOR_SIGNATURE,
        OWNED_BUFFER_FUNCTION_TRANSFER_PROTOCOL, OWNED_BUFFER_RETURN_LENGTH_POLICY,
        OWNED_BUFFER_RETURN_PROTOCOL, OWNED_OBJECT_DESTRUCTOR_SIGNATURE,
        OWNED_OBJECT_RETURN_PROTOCOL, OWNED_OBJECT_RETURN_READ_POLICY,
        OWNED_OBJECT_RETURN_SIZE_POLICY, OWNED_UTF8_DESTRUCTOR_SIGNATURE,
        OWNED_UTF8_RETURN_LENGTH_POLICY, OWNED_UTF8_RETURN_PROTOCOL,
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

    #[test]
    fn owned_buffer_return_contract_revalidates_embedded_hashes() {
        let signature = "ref_Buffer(i64)";
        let signature_hash = ffi_symbol_signature_hash("c", "host_owned_buffer_make", signature);
        let destructor_hash = ffi_symbol_signature_hash(
            "c",
            "host_owned_buffer_destroy",
            OWNED_BUFFER_DESTRUCTOR_SIGNATURE,
        );
        let descriptor =
            owned_buffer_return_descriptor("host_owned_buffer_destroy", &destructor_hash);
        let capability_hash =
            ffi_memory_capability_hash("c", "host_owned_buffer_make", &signature_hash, &descriptor);
        let args = vec![
            OWNED_BUFFER_RETURN_PROTOCOL.to_owned(),
            "c".to_owned(),
            "host_owned_buffer_make".to_owned(),
            signature.to_owned(),
            signature_hash,
            capability_hash,
            OWNED_BUFFER_RETURN_LENGTH_POLICY.to_owned(),
            "host_owned_buffer_destroy".to_owned(),
            destructor_hash,
            "seed".to_owned(),
        ];
        let contract = parse_owned_buffer_return_contract(&args).unwrap();
        assert_eq!(contract.inputs, ["seed"]);

        let mut tampered = args;
        tampered[7] = "other_destroy".to_owned();
        assert!(parse_owned_buffer_return_contract(&tampered)
            .unwrap_err()
            .contains("destructor signature hash mismatch"));

        tampered[8] = "fnv1a64:0000000000000000".to_owned();
        let forged_descriptor = owned_buffer_return_descriptor("other_destroy", &tampered[8]);
        tampered[5] = ffi_memory_capability_hash(
            "c",
            "host_owned_buffer_make",
            &tampered[4],
            &forged_descriptor,
        );
        assert!(parse_owned_buffer_return_contract(&tampered)
            .unwrap_err()
            .contains("destructor signature hash mismatch"));
    }

    #[test]
    fn owned_buffer_function_transfer_revalidates_destructor_identity() {
        let hash = ffi_symbol_signature_hash(
            "c",
            "host_owned_buffer_destroy",
            OWNED_BUFFER_DESTRUCTOR_SIGNATURE,
        );
        let args = vec![
            OWNED_BUFFER_FUNCTION_TRANSFER_PROTOCOL.to_owned(),
            "c".to_owned(),
            "host_owned_buffer_destroy".to_owned(),
            hash,
            "seed".to_owned(),
        ];
        let contract = parse_owned_buffer_function_transfer_contract(&args).unwrap();
        assert_eq!(contract.inputs, ["seed"]);

        let mut tampered = args;
        tampered[2] = "other_destroy".to_owned();
        assert!(parse_owned_buffer_function_transfer_contract(&tampered)
            .unwrap_err()
            .contains("destructor signature hash mismatch"));
    }

    #[test]
    fn owned_utf8_return_contract_revalidates_all_authority_hashes() {
        let signature = "ref_String(i64)";
        let signature_hash = ffi_symbol_signature_hash("c", "host_owned_utf8_make", signature);
        let destructor_hash = ffi_symbol_signature_hash(
            "c",
            "host_owned_utf8_destroy",
            OWNED_UTF8_DESTRUCTOR_SIGNATURE,
        );
        let descriptor = owned_utf8_return_descriptor("host_owned_utf8_destroy", &destructor_hash);
        let capability_hash =
            ffi_memory_capability_hash("c", "host_owned_utf8_make", &signature_hash, &descriptor);
        let args = vec![
            OWNED_UTF8_RETURN_PROTOCOL.to_owned(),
            "c".to_owned(),
            "host_owned_utf8_make".to_owned(),
            signature.to_owned(),
            signature_hash,
            capability_hash,
            OWNED_UTF8_RETURN_LENGTH_POLICY.to_owned(),
            "host_owned_utf8_destroy".to_owned(),
            destructor_hash,
            "seed".to_owned(),
        ];
        let contract = parse_owned_utf8_return_contract(&args).unwrap();
        assert_eq!(contract.inputs, ["seed"]);

        let mut tampered = args;
        tampered[7] = "other_destroy".to_owned();
        assert!(parse_owned_utf8_return_contract(&tampered)
            .unwrap_err()
            .contains("destructor signature hash mismatch"));
    }

    #[test]
    fn owned_object_contract_binds_size_read_and_destructor_authority() {
        let signature = "ref_FfiObject(i64)";
        let signature_hash = ffi_symbol_signature_hash("c", "host_owned_object_make", signature);
        let destructor_hash = ffi_symbol_signature_hash(
            "c",
            "host_owned_object_destroy",
            OWNED_OBJECT_DESTRUCTOR_SIGNATURE,
        );
        let descriptor =
            owned_object_return_descriptor("host_owned_object_destroy", &destructor_hash);
        let capability_hash =
            ffi_memory_capability_hash("c", "host_owned_object_make", &signature_hash, &descriptor);
        let args = vec![
            OWNED_OBJECT_RETURN_PROTOCOL.to_owned(),
            "c".to_owned(),
            "host_owned_object_make".to_owned(),
            signature.to_owned(),
            signature_hash,
            capability_hash,
            OWNED_OBJECT_RETURN_SIZE_POLICY.to_owned(),
            OWNED_OBJECT_RETURN_READ_POLICY.to_owned(),
            "host_owned_object_destroy".to_owned(),
            destructor_hash,
            "seed".to_owned(),
        ];
        let contract = parse_owned_object_return_contract(&args).unwrap();
        assert_eq!(contract.inputs, ["seed"]);

        let mut size_drift = args.clone();
        size_drift[6] = "static:24".to_owned();
        assert!(parse_owned_object_return_contract(&size_drift)
            .unwrap_err()
            .contains("size policy"));

        let mut read_drift = args.clone();
        read_drift[7] = "raw_bytes".to_owned();
        assert!(parse_owned_object_return_contract(&read_drift)
            .unwrap_err()
            .contains("read policy"));

        let mut destructor_drift = args;
        destructor_drift[8] = "other_destroy".to_owned();
        assert!(parse_owned_object_return_contract(&destructor_drift)
            .unwrap_err()
            .contains("destructor signature hash mismatch"));
    }
}
