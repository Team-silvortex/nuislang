use ed25519_dalek::VerifyingKey;
use std::{collections::BTreeSet, env, fs};

use crate::provider_completion_trust_anchor::{self, AnchorCheck};

pub(crate) const REGISTRY_PROTOCOL: &str = "nuis-provider-completion-trust-registry-v1";
pub(crate) const REGISTRY_PATH_ENV: &str = "NUIS_PROVIDER_COMPLETION_TRUST_REGISTRY";
pub(crate) const INLINE_KEYS_ENV: &str = "NUIS_PROVIDER_COMPLETION_TRUSTED_PUBLIC_KEYS";

pub(crate) enum TrustedKeyLookup {
    Active(VerifyingKey),
    Revoked,
    Missing,
    Invalid,
    Rollback,
    Fork,
    AnchorInvalid,
}

pub(crate) fn lookup_from_environment(key_id: &str) -> TrustedKeyLookup {
    if let Ok(path) = env::var(REGISTRY_PATH_ENV) {
        let Ok(source) = fs::read_to_string(&path) else {
            return TrustedKeyLookup::Invalid;
        };
        let selected = lookup(&source, key_id);
        if matches!(selected, TrustedKeyLookup::Invalid) {
            return selected;
        }
        let generation = usize_field(
            source.split("[[keys]]").next().unwrap_or(&source),
            "generation",
        )
        .expect("validated registry generation");
        let hash = crate::digest_sha256::sha256_hex(source.as_bytes());
        return match provider_completion_trust_anchor::enforce(
            std::path::Path::new(&path),
            REGISTRY_PROTOCOL,
            generation,
            &hash,
        ) {
            AnchorCheck::Accepted => selected,
            AnchorCheck::Rollback => TrustedKeyLookup::Rollback,
            AnchorCheck::Fork => TrustedKeyLookup::Fork,
            AnchorCheck::Invalid => TrustedKeyLookup::AnchorInvalid,
        };
    }
    env::var(INLINE_KEYS_ENV)
        .ok()
        .map_or(TrustedKeyLookup::Missing, |inline| {
            lookup_inline(&inline, key_id)
        })
}

fn lookup(source: &str, key_id: &str) -> TrustedKeyLookup {
    let header = source.split("[[keys]]").next().unwrap_or(source);
    if string_field(header, "protocol").as_deref() != Some(REGISTRY_PROTOCOL)
        || usize_field(header, "generation").is_none_or(|generation| generation == 0)
    {
        return TrustedKeyLookup::Invalid;
    }
    let mut seen = BTreeSet::new();
    let mut selected = TrustedKeyLookup::Missing;
    let mut count = 0usize;
    for record in source.split("[[keys]]").skip(1) {
        count += 1;
        let (Some(candidate_id), Some(encoded), Some(status)) = (
            string_field(record, "key_id"),
            string_field(record, "public_key_hex"),
            string_field(record, "status"),
        ) else {
            return TrustedKeyLookup::Invalid;
        };
        if !seen.insert(candidate_id.clone()) || !matches!(status.as_str(), "active" | "revoked") {
            return TrustedKeyLookup::Invalid;
        }
        let Ok(bytes) = decode_key(&encoded) else {
            return TrustedKeyLookup::Invalid;
        };
        let Ok(key) = VerifyingKey::from_bytes(&bytes) else {
            return TrustedKeyLookup::Invalid;
        };
        if public_key_id(&key) != candidate_id {
            return TrustedKeyLookup::Invalid;
        }
        if candidate_id == key_id {
            selected = if status == "active" {
                TrustedKeyLookup::Active(key)
            } else {
                TrustedKeyLookup::Revoked
            };
        }
    }
    if count == 0 {
        TrustedKeyLookup::Invalid
    } else {
        selected
    }
}

fn lookup_inline(source: &str, key_id: &str) -> TrustedKeyLookup {
    for entry in source.split([',', ';']) {
        let Some((candidate_id, encoded)) = entry.trim().split_once('=') else {
            continue;
        };
        if candidate_id.trim() != key_id {
            continue;
        }
        let Ok(bytes) = decode_key(encoded.trim()) else {
            return TrustedKeyLookup::Invalid;
        };
        let Ok(key) = VerifyingKey::from_bytes(&bytes) else {
            return TrustedKeyLookup::Invalid;
        };
        return if public_key_id(&key) == key_id {
            TrustedKeyLookup::Active(key)
        } else {
            TrustedKeyLookup::Invalid
        };
    }
    TrustedKeyLookup::Missing
}

fn public_key_id(key: &VerifyingKey) -> String {
    format!(
        "ed25519:sha256:{}",
        crate::digest_sha256::sha256_hex(key.as_bytes())
    )
}

fn decode_key(encoded: &str) -> Result<[u8; 32], ()> {
    if encoded.len() != 64 {
        return Err(());
    }
    let mut out = [0u8; 32];
    for (index, slot) in out.iter_mut().enumerate() {
        *slot = u8::from_str_radix(&encoded[index * 2..index * 2 + 2], 16).map_err(|_| ())?;
    }
    Ok(out)
}

fn string_field(source: &str, key: &str) -> Option<String> {
    source.lines().find_map(|line| {
        let (candidate, value) = line.split_once('=')?;
        (candidate.trim() == key).then(|| {
            value
                .trim()
                .strip_prefix('"')?
                .strip_suffix('"')
                .map(str::to_owned)
        })?
    })
}

fn usize_field(source: &str, key: &str) -> Option<usize> {
    source.lines().find_map(|line| {
        let (candidate, value) = line.split_once('=')?;
        (candidate.trim() == key)
            .then(|| value.trim().parse().ok())
            .flatten()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;

    fn record(seed: u8, status: &str) -> (String, String) {
        let key = SigningKey::from_bytes(&[seed; 32]).verifying_key();
        let id = public_key_id(&key);
        let hex = key
            .as_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        (
            id.clone(),
            format!(
                "[[keys]]\nkey_id = \"{id}\"\npublic_key_hex = \"{hex}\"\nstatus = \"{status}\"\n"
            ),
        )
    }

    #[test]
    fn rotation_accepts_new_key_and_rejects_revoked_key() {
        let (old_id, old) = record(7, "revoked");
        let (new_id, new) = record(9, "active");
        let source = format!("protocol = \"{REGISTRY_PROTOCOL}\"\ngeneration = 2\n{old}{new}");
        assert!(matches!(
            lookup(&source, &old_id),
            TrustedKeyLookup::Revoked
        ));
        assert!(matches!(
            lookup(&source, &new_id),
            TrustedKeyLookup::Active(_)
        ));
    }

    #[test]
    fn duplicate_or_zero_generation_registry_is_invalid() {
        let (id, record) = record(7, "active");
        let duplicate =
            format!("protocol = \"{REGISTRY_PROTOCOL}\"\ngeneration = 1\n{record}{record}");
        assert!(matches!(lookup(&duplicate, &id), TrustedKeyLookup::Invalid));
        let zero = format!("protocol = \"{REGISTRY_PROTOCOL}\"\ngeneration = 0\n{record}");
        assert!(matches!(lookup(&zero, &id), TrustedKeyLookup::Invalid));
    }

    #[test]
    fn environment_lookup_enforces_generation_anchor() {
        let _guard = crate::provider_completion_trust_anchor::TEST_ENV_LOCK
            .lock()
            .unwrap();
        let root = env::temp_dir().join(format!("nsdb-registry-anchor-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let path = root.join("registry.toml");
        let (id, record) = record(17, "active");
        env::set_var(REGISTRY_PATH_ENV, &path);
        env::set_var(
            "NUIS_PROVIDER_COMPLETION_TRUST_ANCHOR",
            root.join("anchor.toml"),
        );
        let source = |generation| {
            format!("protocol = \"{REGISTRY_PROTOCOL}\"\ngeneration = {generation}\n{record}")
        };
        fs::write(&path, source(2)).unwrap();
        assert!(matches!(
            lookup_from_environment(&id),
            TrustedKeyLookup::Active(_)
        ));
        fs::write(&path, source(1)).unwrap();
        assert!(matches!(
            lookup_from_environment(&id),
            TrustedKeyLookup::Rollback
        ));
        fs::write(&path, format!("{}\n# fork", source(2))).unwrap();
        assert!(matches!(
            lookup_from_environment(&id),
            TrustedKeyLookup::Fork
        ));
        fs::write(&path, source(3)).unwrap();
        assert!(matches!(
            lookup_from_environment(&id),
            TrustedKeyLookup::Active(_)
        ));
        env::remove_var(REGISTRY_PATH_ENV);
        env::remove_var("NUIS_PROVIDER_COMPLETION_TRUST_ANCHOR");
        fs::remove_dir_all(root).unwrap();
    }
}
