use ed25519_dalek::{Signature, VerifyingKey};
use std::env;

pub(crate) const SIGNATURE_CONTRACT: &str = "nuis-provider-completion-signature-ed25519-v1";
const TRUSTED_KEYS_ENV: &str = "NUIS_PROVIDER_COMPLETION_TRUSTED_PUBLIC_KEYS";

pub(crate) struct ParsedProviderCompletionSignature {
    pub(crate) contract: Option<String>,
    pub(crate) public_key_id: Option<String>,
    pub(crate) status: String,
}

pub(crate) fn parse_and_verify(
    source: &str,
    has_provider_completions: bool,
    signature_required: bool,
    message: &[u8],
) -> ParsedProviderCompletionSignature {
    let contract = parse_string_field(source, "provider_completion_signature_contract");
    let public_key_id = parse_string_field(source, "provider_completion_signature_public_key_id");
    let signature = parse_string_field(source, "provider_completion_signature");
    let status = if !has_provider_completions {
        "not-applicable"
    } else if !signature_required
        && contract.is_none()
        && public_key_id.is_none()
        && signature.is_none()
    {
        "legacy-unsigned"
    } else {
        verify_from_environment(
            contract.as_deref().unwrap_or("none"),
            public_key_id.as_deref().unwrap_or("none"),
            signature.as_deref().unwrap_or("none"),
            message,
        )
    };
    ParsedProviderCompletionSignature {
        contract,
        public_key_id,
        status: status.to_owned(),
    }
}

pub(crate) fn verify_from_environment(
    contract: &str,
    public_key_id_claim: &str,
    signature_hex: &str,
    message: &[u8],
) -> &'static str {
    if contract == "none" || public_key_id_claim == "none" || signature_hex == "none" {
        return "signature-missing";
    }
    if contract != SIGNATURE_CONTRACT {
        return "unsupported-signature-contract";
    }
    let Some(verifying_key) = trusted_key(public_key_id_claim) else {
        return "signature-key-untrusted";
    };
    let Ok(signature_bytes) = decode_array::<64>(signature_hex) else {
        return "signature-malformed";
    };
    if verifying_key
        .verify_strict(message, &Signature::from_bytes(&signature_bytes))
        .is_ok()
    {
        "signature-verified"
    } else {
        "signature-mismatch"
    }
}

pub(crate) fn validation_error(status: &str) -> Option<String> {
    match status {
        "signature-missing" => Some("provider-completion-signature-missing"),
        "unsupported-signature-contract" => {
            Some("provider-completion-signature-contract-unsupported")
        }
        "signature-key-untrusted" => Some("provider-completion-signature-key-untrusted"),
        "signature-malformed" => Some("provider-completion-signature-malformed"),
        "signature-mismatch" => Some("provider-completion-signature-mismatch"),
        _ => None,
    }
    .map(str::to_owned)
}

fn trusted_key(key_id: &str) -> Option<VerifyingKey> {
    let registry = env::var(TRUSTED_KEYS_ENV).ok()?;
    for entry in registry.split([',', ';']) {
        let Some((candidate_id, encoded_key)) = entry.trim().split_once('=') else {
            continue;
        };
        if candidate_id.trim() != key_id {
            continue;
        }
        let key = VerifyingKey::from_bytes(&decode_array::<32>(encoded_key.trim()).ok()?).ok()?;
        let actual_id = format!(
            "ed25519:sha256:{}",
            crate::digest_sha256::sha256_hex(key.as_bytes())
        );
        return (actual_id == key_id).then_some(key);
    }
    None
}

fn decode_array<const N: usize>(encoded: &str) -> Result<[u8; N], ()> {
    if encoded.len() != N * 2 {
        return Err(());
    }
    let mut out = [0u8; N];
    for (index, slot) in out.iter_mut().enumerate() {
        *slot = u8::from_str_radix(&encoded[index * 2..index * 2 + 2], 16).map_err(|_| ())?;
    }
    Ok(out)
}

fn parse_string_field(source: &str, key: &str) -> Option<String> {
    source.lines().find_map(|line| {
        let (candidate, value) = line.split_once('=')?;
        (candidate.trim() == key)
            .then(|| value.trim().strip_prefix('"')?.strip_suffix('"'))
            .flatten()
            .filter(|value| *value != "none" && !value.is_empty())
            .map(str::to_owned)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn strict_verification_primitive_rejects_modified_message() {
        let signing_key = SigningKey::from_bytes(&[11u8; 32]);
        let signature = signing_key.sign(b"canonical-claim");
        assert!(signing_key
            .verifying_key()
            .verify_strict(b"canonical-claim", &signature)
            .is_ok());
        assert!(signing_key
            .verifying_key()
            .verify_strict(b"modified-claim", &signature)
            .is_err());
    }

    #[test]
    fn independent_trust_registry_verifies_and_rejects_tampering() {
        let _guard = ENV_LOCK.lock().unwrap();
        let signing_key = SigningKey::from_bytes(&[11u8; 32]);
        let key = signing_key.verifying_key();
        let key_id = format!(
            "ed25519:sha256:{}",
            crate::digest_sha256::sha256_hex(key.as_bytes())
        );
        let public_key_hex = key
            .as_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        env::set_var(TRUSTED_KEYS_ENV, format!("{key_id}={public_key_hex}"));
        let message = b"provider-completion-signature-v1\0canonical";
        let signature_hex = signing_key
            .sign(message)
            .to_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        assert_eq!(
            verify_from_environment(SIGNATURE_CONTRACT, &key_id, &signature_hex, message),
            "signature-verified"
        );
        assert_eq!(
            verify_from_environment(
                SIGNATURE_CONTRACT,
                &key_id,
                &signature_hex,
                b"provider-completion-signature-v1\0tampered",
            ),
            "signature-mismatch"
        );
        env::remove_var(TRUSTED_KEYS_ENV);
    }

    #[test]
    fn signed_contract_cannot_downgrade_by_removing_signature_fields() {
        assert_eq!(
            parse_and_verify("", true, true, b"canonical").status,
            "signature-missing"
        );
        assert_eq!(
            parse_and_verify("", true, false, b"canonical").status,
            "legacy-unsigned"
        );
    }
}
