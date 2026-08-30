use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use ed25519_dalek::VerifyingKey;
use sha2::{Digest, Sha256};

use crate::{
    toml::{
        escape_toml_string, parse_required_map_string_in_block, parse_required_toml_string,
        parse_required_toml_usize,
    },
    ArtifactError,
};

use crate::compiler_component_attestation::COMPILER_COMPONENT_ATTESTATION_TRUST_SCOPE;

pub const COMPILER_COMPONENT_ATTESTER_TRUST_REGISTRY_PROTOCOL: &str =
    "nuis-compiler-component-attester-trust-registry-v1";
pub const COMPILER_COMPONENT_ATTESTER_TRUST_REGISTRY_FILE: &str =
    "nuis.compiler-component-attester-trust-registry.toml";

const ACTIVE_STATUS: &str = "active";
const REVOKED_STATUS: &str = "revoked";

#[derive(Debug, Clone, Copy)]
pub struct CompilerComponentAttesterTrustEntryInput<'a> {
    pub attester_id: &'a str,
    pub environment_id: &'a str,
    pub public_key_hex: &'a str,
    pub status: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerComponentAttesterTrustEntry {
    pub attester_id: String,
    pub environment_id: String,
    pub trust_scope: String,
    pub public_key_id: String,
    pub public_key_hex: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerComponentAttesterTrustRegistry {
    pub protocol: String,
    pub generation: usize,
    pub entries: Vec<CompilerComponentAttesterTrustEntry>,
}

pub fn build_compiler_component_attester_trust_registry(
    generation: usize,
    inputs: &[CompilerComponentAttesterTrustEntryInput<'_>],
) -> Result<CompilerComponentAttesterTrustRegistry, ArtifactError> {
    let entries = inputs
        .iter()
        .map(|input| {
            Ok(CompilerComponentAttesterTrustEntry {
                attester_id: input.attester_id.to_owned(),
                environment_id: input.environment_id.to_owned(),
                trust_scope: COMPILER_COMPONENT_ATTESTATION_TRUST_SCOPE.to_owned(),
                public_key_id: compiler_component_attester_public_key_id(input.public_key_hex)?,
                public_key_hex: input.public_key_hex.to_owned(),
                status: input.status.to_owned(),
            })
        })
        .collect::<Result<Vec<_>, ArtifactError>>()?;
    let mut registry = CompilerComponentAttesterTrustRegistry {
        protocol: COMPILER_COMPONENT_ATTESTER_TRUST_REGISTRY_PROTOCOL.to_owned(),
        generation,
        entries,
    };
    registry.entries.sort_by(|left, right| {
        (&left.attester_id, &left.environment_id, &left.public_key_id).cmp(&(
            &right.attester_id,
            &right.environment_id,
            &right.public_key_id,
        ))
    });
    validate_registry(&registry)?;
    Ok(registry)
}

pub fn render_compiler_component_attester_trust_registry(
    registry: &CompilerComponentAttesterTrustRegistry,
) -> String {
    let mut out = format!(
        "protocol = \"{}\"\ngeneration = {}\n",
        registry.protocol, registry.generation
    );
    for entry in &registry.entries {
        out.push_str(&format!(
            "\n[[attester]]\nattester_id = \"{}\"\nenvironment_id = \"{}\"\ntrust_scope = \"{}\"\npublic_key_id = \"{}\"\npublic_key_hex = \"{}\"\nstatus = \"{}\"\n",
            escape_toml_string(&entry.attester_id),
            escape_toml_string(&entry.environment_id),
            entry.trust_scope,
            entry.public_key_id,
            entry.public_key_hex,
            entry.status,
        ));
    }
    out
}

pub fn parse_compiler_component_attester_trust_registry(
    path: &Path,
) -> Result<CompilerComponentAttesterTrustRegistry, ArtifactError> {
    let source = fs::read_to_string(path).map_err(|error| {
        ArtifactError::new(format!(
            "failed to read compiler attester trust registry `{}`: {error}",
            path.display()
        ))
    })?;
    parse_compiler_component_attester_trust_registry_from_source(&source, path)
}

pub fn parse_compiler_component_attester_trust_registry_from_source(
    source: &str,
    path: &Path,
) -> Result<CompilerComponentAttesterTrustRegistry, ArtifactError> {
    validate_text(source, path)?;
    let registry = CompilerComponentAttesterTrustRegistry {
        protocol: parse_required_toml_string(source, "protocol", path)?,
        generation: parse_required_toml_usize(source, "generation", path)?,
        entries: parse_attester_blocks(source, path)?,
    };
    validate_registry(&registry)?;
    if render_compiler_component_attester_trust_registry(&registry) != source {
        return Err(ArtifactError::new(format!(
            "compiler attester trust registry `{}` is not canonically encoded",
            path.display()
        )));
    }
    Ok(registry)
}

pub fn compiler_component_attester_trust_registry_sha256(source: &str) -> String {
    sha256_hex(source.as_bytes())
}

pub fn compiler_component_attester_public_key_id(
    public_key_hex: &str,
) -> Result<String, ArtifactError> {
    let bytes = decode_array::<32>(public_key_hex, "attester public key")?;
    let key = VerifyingKey::from_bytes(&bytes)
        .map_err(|_| ArtifactError::new("compiler attester public key is invalid Ed25519"))?;
    Ok(public_key_id(&key))
}

pub(super) fn resolve_attester_key(
    registry: &CompilerComponentAttesterTrustRegistry,
    attester_id: &str,
    environment_id: &str,
    public_key_id_claim: &str,
) -> Result<VerifyingKey, ArtifactError> {
    validate_registry(registry)?;
    let matching = registry
        .entries
        .iter()
        .find(|entry| {
            entry.attester_id == attester_id
                && entry.environment_id == environment_id
                && entry.public_key_id == public_key_id_claim
        })
        .ok_or_else(|| {
            ArtifactError::new("compiler attestation key is not in the pinned registry")
        })?;
    if matching.status == REVOKED_STATUS {
        return Err(ArtifactError::new(
            "compiler attestation key is revoked in the pinned registry",
        ));
    }
    if matching.status != ACTIVE_STATUS
        || matching.trust_scope != COMPILER_COMPONENT_ATTESTATION_TRUST_SCOPE
    {
        return Err(ArtifactError::new(
            "compiler attestation key lacks independent-machine trust scope",
        ));
    }
    let bytes = decode_array::<32>(&matching.public_key_hex, "attester public key")?;
    VerifyingKey::from_bytes(&bytes)
        .map_err(|_| ArtifactError::new("compiler attester public key is invalid Ed25519"))
}

pub(super) fn public_key_id(key: &VerifyingKey) -> String {
    format!("ed25519:sha256:{}", sha256_hex(key.as_bytes()))
}

pub(super) fn decode_array<const N: usize>(
    encoded: &str,
    label: &str,
) -> Result<[u8; N], ArtifactError> {
    if encoded.len() != N * 2
        || !encoded
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(ArtifactError::new(format!(
            "compiler {label} must contain exactly {N} lowercase hexadecimal bytes"
        )));
    }
    let mut out = [0u8; N];
    for (index, slot) in out.iter_mut().enumerate() {
        *slot = u8::from_str_radix(&encoded[index * 2..index * 2 + 2], 16)
            .map_err(|_| ArtifactError::new(format!("compiler {label} is malformed")))?;
    }
    Ok(out)
}

pub(super) fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn validate_registry(
    registry: &CompilerComponentAttesterTrustRegistry,
) -> Result<(), ArtifactError> {
    if registry.protocol != COMPILER_COMPONENT_ATTESTER_TRUST_REGISTRY_PROTOCOL
        || registry.generation == 0
        || registry.entries.is_empty()
    {
        return Err(ArtifactError::new(
            "compiler attester trust registry header is invalid",
        ));
    }
    let mut identities = BTreeSet::new();
    let mut active_environments = BTreeSet::new();
    let mut previous = None;
    for entry in &registry.entries {
        validate_token(&entry.attester_id, "attester id")?;
        validate_token(&entry.environment_id, "attester environment id")?;
        if entry.trust_scope != COMPILER_COMPONENT_ATTESTATION_TRUST_SCOPE
            || !matches!(entry.status.as_str(), ACTIVE_STATUS | REVOKED_STATUS)
        {
            return Err(ArtifactError::new(
                "compiler attester registry entry has invalid scope or status",
            ));
        }
        let expected_id = compiler_component_attester_public_key_id(&entry.public_key_hex)?;
        if entry.public_key_id != expected_id {
            return Err(ArtifactError::new(
                "compiler attester registry public key identity mismatch",
            ));
        }
        let identity = (
            entry.attester_id.clone(),
            entry.environment_id.clone(),
            entry.public_key_id.clone(),
        );
        if previous.as_ref().is_some_and(|value| value >= &identity) {
            return Err(ArtifactError::new(
                "compiler attester registry entries must be uniquely sorted",
            ));
        }
        previous = Some(identity.clone());
        if !identities.insert(identity) {
            return Err(ArtifactError::new(
                "compiler attester registry repeats a key identity",
            ));
        }
        if entry.status == ACTIVE_STATUS
            && !active_environments
                .insert((entry.attester_id.clone(), entry.environment_id.clone()))
        {
            return Err(ArtifactError::new(
                "compiler attester registry permits only one active key per environment",
            ));
        }
    }
    Ok(())
}

fn parse_attester_blocks(
    source: &str,
    path: &Path,
) -> Result<Vec<CompilerComponentAttesterTrustEntry>, ArtifactError> {
    let mut entries = Vec::new();
    let mut values = BTreeMap::new();
    let mut in_block = false;
    for raw in source.lines() {
        let line = raw.trim();
        if line == "[[attester]]" {
            if in_block {
                entries.push(parse_attester(&values, path)?);
                values.clear();
            }
            in_block = true;
            continue;
        }
        if in_block && !line.is_empty() && !line.starts_with('#') {
            let Some((key, value)) = line.split_once('=') else {
                return Err(ArtifactError::new(format!(
                    "`{}` contains malformed attester registry line `{line}`",
                    path.display()
                )));
            };
            let key = key.trim().to_owned();
            if values
                .insert(key.clone(), value.trim().to_owned())
                .is_some()
            {
                return Err(ArtifactError::new(format!(
                    "`{}` attester entry repeats key `{key}`",
                    path.display()
                )));
            }
        }
    }
    if in_block {
        entries.push(parse_attester(&values, path)?);
    }
    Ok(entries)
}

fn parse_attester(
    values: &BTreeMap<String, String>,
    path: &Path,
) -> Result<CompilerComponentAttesterTrustEntry, ArtifactError> {
    let string = |key| parse_required_map_string_in_block(values, key, path, "attester");
    Ok(CompilerComponentAttesterTrustEntry {
        attester_id: string("attester_id")?,
        environment_id: string("environment_id")?,
        trust_scope: string("trust_scope")?,
        public_key_id: string("public_key_id")?,
        public_key_hex: string("public_key_hex")?,
        status: string("status")?,
    })
}

fn validate_token(value: &str, label: &str) -> Result<(), ArtifactError> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
    {
        return Err(ArtifactError::new(format!(
            "compiler {label} must be a non-path ASCII token"
        )));
    }
    Ok(())
}

fn validate_text(source: &str, path: &Path) -> Result<(), ArtifactError> {
    if source.is_empty()
        || !source.ends_with('\n')
        || source.contains('\r')
        || source.contains('\0')
    {
        return Err(ArtifactError::new(format!(
            "compiler attester trust registry `{}` must be UTF-8/LF text",
            path.display()
        )));
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
