use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use std::env;

use crate::provider_completion_trust_registry::{lookup_from_environment, TrustedKeyLookup};

pub(crate) const SIGNATURE_CONTRACT: &str = "nuis-provider-completion-signature-ed25519-v1";
pub(crate) const SIGNING_KEY_ENV: &str = "NUIS_PROVIDER_COMPLETION_SIGNING_KEY_HEX";

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
    let verifying_key = match lookup_from_environment(public_key_id_claim) {
        TrustedKeyLookup::Active(key) => key,
        TrustedKeyLookup::Revoked => return "signature-key-revoked",
        TrustedKeyLookup::Missing => return "signature-key-untrusted",
        TrustedKeyLookup::Invalid => return "signature-trust-registry-invalid",
        TrustedKeyLookup::Rollback => return "signature-trust-registry-rollback",
        TrustedKeyLookup::Fork => return "signature-trust-registry-fork",
        TrustedKeyLookup::AnchorInvalid => return "signature-trust-anchor-invalid",
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

pub(crate) fn handoff_error_status(status: &str) -> Option<&'static str> {
    match status {
        "signature-missing" => Some("provider-completion-signature-missing"),
        "unsupported-signature-contract" => {
            Some("provider-completion-signature-contract-unsupported")
        }
        "signature-key-untrusted" => Some("provider-completion-signature-key-untrusted"),
        "signature-key-revoked" => Some("provider-completion-signature-key-revoked"),
        "signature-trust-registry-invalid" => {
            Some("provider-completion-signature-trust-registry-invalid")
        }
        "signature-trust-registry-rollback" => {
            Some("provider-completion-signature-trust-registry-rollback")
        }
        "signature-trust-registry-fork" => {
            Some("provider-completion-signature-trust-registry-fork")
        }
        "signature-trust-anchor-invalid" => {
            Some("provider-completion-signature-trust-anchor-invalid")
        }
        "signature-malformed" => Some("provider-completion-signature-malformed"),
        "signature-mismatch" => Some("provider-completion-signature-mismatch"),
        _ => None,
    }
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
