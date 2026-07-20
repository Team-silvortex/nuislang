use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use std::env;

pub(crate) const SIGNATURE_CONTRACT: &str = "nuis-provider-completion-signature-ed25519-v1";
pub(crate) const SIGNING_KEY_ENV: &str = "NUIS_PROVIDER_COMPLETION_SIGNING_KEY_HEX";
pub(crate) const TRUSTED_KEYS_ENV: &str = "NUIS_PROVIDER_COMPLETION_TRUSTED_PUBLIC_KEYS";

pub(crate) struct ProviderCompletionSignatureClaim {
    pub(crate) public_key_id: String,
    pub(crate) signature_hex: String,
}

pub(crate) fn signing_key_configured() -> bool {
    env::var_os(SIGNING_KEY_ENV).is_some()
}

pub(crate) fn sign_from_environment(
    message: &[u8],
) -> Result<Option<ProviderCompletionSignatureClaim>, String> {
    let Ok(encoded) = env::var(SIGNING_KEY_ENV) else {
        return Ok(None);
    };
    let signing_key = SigningKey::from_bytes(&decode_array::<32>(&encoded, "signing key")?);
    Ok(Some(sign_with_key(&signing_key, message)))
}

fn sign_with_key(signing_key: &SigningKey, message: &[u8]) -> ProviderCompletionSignatureClaim {
    ProviderCompletionSignatureClaim {
        public_key_id: public_key_id(&signing_key.verifying_key()),
        signature_hex: encode_hex(&signing_key.sign(message).to_bytes()),
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
    let Ok(signature_bytes) = decode_array::<64>(signature_hex, "signature") else {
        return "signature-malformed";
    };
    let signature = Signature::from_bytes(&signature_bytes);
    if verifying_key.verify_strict(message, &signature).is_ok() {
        "signature-verified"
    } else {
        "signature-mismatch"
    }
}

fn trusted_key(key_id: &str) -> Option<VerifyingKey> {
    if let Ok(registry) = env::var(TRUSTED_KEYS_ENV) {
        for entry in registry.split([',', ';']) {
            let Some((candidate_id, encoded_key)) = entry.trim().split_once('=') else {
                continue;
            };
            if candidate_id.trim() != key_id {
                continue;
            }
            let bytes = decode_array::<32>(encoded_key.trim(), "trusted public key").ok()?;
            let key = VerifyingKey::from_bytes(&bytes).ok()?;
            return (public_key_id(&key) == key_id).then_some(key);
        }
    }
    let encoded = env::var(SIGNING_KEY_ENV).ok()?;
    let signing_key = SigningKey::from_bytes(&decode_array::<32>(&encoded, "signing key").ok()?);
    let key = signing_key.verifying_key();
    (public_key_id(&key) == key_id).then_some(key)
}

fn public_key_id(key: &VerifyingKey) -> String {
    format!(
        "ed25519:sha256:{}",
        crate::digest_sha256::sha256_hex(key.as_bytes())
    )
}

fn decode_array<const N: usize>(encoded: &str, label: &str) -> Result<[u8; N], String> {
    if encoded.len() != N * 2 {
        return Err(format!(
            "{label} must contain exactly {} hexadecimal bytes",
            N
        ));
    }
    let mut out = [0u8; N];
    for (index, slot) in out.iter_mut().enumerate() {
        *slot = u8::from_str_radix(&encoded[index * 2..index * 2 + 2], 16)
            .map_err(|_| format!("{label} contains non-hexadecimal bytes"))?;
    }
    Ok(out)
}

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strict_verification_rejects_tampered_claim() {
        let signing_key = SigningKey::from_bytes(&[7u8; 32]);
        let signature = signing_key.sign(b"provider-completion-set-v5");
        assert!(signing_key
            .verifying_key()
            .verify_strict(b"provider-completion-set-v5", &signature)
            .is_ok());
        assert!(signing_key
            .verifying_key()
            .verify_strict(b"provider-completion-set-v4", &signature)
            .is_err());
    }

    #[test]
    fn signing_claim_uses_key_identity_and_rejects_tampering() {
        let signing_key = SigningKey::from_bytes(&[7u8; 32]);
        let message = b"provider-completion-signature-v1\0canonical";
        let claim = sign_with_key(&signing_key, message);
        let signature =
            Signature::from_bytes(&decode_array::<64>(&claim.signature_hex, "signature").unwrap());
        assert_eq!(
            claim.public_key_id,
            public_key_id(&signing_key.verifying_key())
        );
        assert!(signing_key
            .verifying_key()
            .verify_strict(message, &signature)
            .is_ok());
        assert!(signing_key
            .verifying_key()
            .verify_strict(b"provider-completion-signature-v1\0tampered", &signature)
            .is_err());
    }
}
