use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use ed25519_dalek::VerifyingKey;
use sha2::{Digest, Sha256};

use crate::{
    compiler_component_attestation_registry::{decode_array, public_key_id},
    toml::{
        escape_toml_string, parse_required_map_string_in_block, parse_required_toml_string,
        parse_required_toml_usize,
    },
    ArtifactError,
};

pub const COMPILER_COMPONENT_REPLACEMENT_AUTHORIZER_REGISTRY_PROTOCOL: &str =
    "nuis-compiler-component-replacement-authorizer-registry-v1";
pub const COMPILER_COMPONENT_REPLACEMENT_AUTHORIZER_REGISTRY_FILE: &str =
    "nuis.compiler-component-replacement-authorizer-registry.toml";
pub const COMPILER_COMPONENT_REPLACEMENT_AUTHORIZER_TRUST_SCOPE: &str =
    "compiler-component-replacement-owner-v1";

const ACTIVE_STATUS: &str = "active";
const REVOKED_STATUS: &str = "revoked";

#[derive(Debug, Clone, Copy)]
pub struct CompilerComponentReplacementAuthorizerEntryInput<'a> {
    pub authorizer_id: &'a str,
    pub environment_id: &'a str,
    pub component_id: &'a str,
    pub public_key_hex: &'a str,
    pub status: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerComponentReplacementAuthorizerEntry {
    pub authorizer_id: String,
    pub environment_id: String,
    pub component_id: String,
    pub trust_scope: String,
    pub public_key_id: String,
    pub public_key_hex: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerComponentReplacementAuthorizerRegistry {
    pub protocol: String,
    pub generation: usize,
    pub entries: Vec<CompilerComponentReplacementAuthorizerEntry>,
}

pub fn build_compiler_component_replacement_authorizer_registry(
    generation: usize,
    inputs: &[CompilerComponentReplacementAuthorizerEntryInput<'_>],
) -> Result<CompilerComponentReplacementAuthorizerRegistry, ArtifactError> {
    let entries = inputs
        .iter()
        .map(|input| {
            Ok(CompilerComponentReplacementAuthorizerEntry {
                authorizer_id: input.authorizer_id.to_owned(),
                environment_id: input.environment_id.to_owned(),
                component_id: input.component_id.to_owned(),
                trust_scope: COMPILER_COMPONENT_REPLACEMENT_AUTHORIZER_TRUST_SCOPE.to_owned(),
                public_key_id: compiler_component_replacement_authorizer_public_key_id(
                    input.public_key_hex,
                )?,
                public_key_hex: input.public_key_hex.to_owned(),
                status: input.status.to_owned(),
            })
        })
        .collect::<Result<Vec<_>, ArtifactError>>()?;
    let mut registry = CompilerComponentReplacementAuthorizerRegistry {
        protocol: COMPILER_COMPONENT_REPLACEMENT_AUTHORIZER_REGISTRY_PROTOCOL.to_owned(),
        generation,
        entries,
    };
    registry.entries.sort_by(|left, right| {
        (
            &left.component_id,
            &left.authorizer_id,
            &left.environment_id,
            &left.public_key_id,
        )
            .cmp(&(
                &right.component_id,
                &right.authorizer_id,
                &right.environment_id,
                &right.public_key_id,
            ))
    });
    validate_registry(&registry)?;
    Ok(registry)
}

pub fn render_compiler_component_replacement_authorizer_registry(
    registry: &CompilerComponentReplacementAuthorizerRegistry,
) -> String {
    let mut out = format!(
        "protocol = \"{}\"\ngeneration = {}\n",
        registry.protocol, registry.generation
    );
    for entry in &registry.entries {
        out.push_str(&format!(
            "\n[[authorizer]]\nauthorizer_id = \"{}\"\nenvironment_id = \"{}\"\ncomponent_id = \"{}\"\ntrust_scope = \"{}\"\npublic_key_id = \"{}\"\npublic_key_hex = \"{}\"\nstatus = \"{}\"\n",
            escape_toml_string(&entry.authorizer_id),
            escape_toml_string(&entry.environment_id),
            escape_toml_string(&entry.component_id),
            entry.trust_scope,
            entry.public_key_id,
            entry.public_key_hex,
            entry.status,
        ));
    }
    out
}

pub fn parse_compiler_component_replacement_authorizer_registry(
    path: &Path,
) -> Result<CompilerComponentReplacementAuthorizerRegistry, ArtifactError> {
    let source = fs::read_to_string(path).map_err(|error| {
        ArtifactError::new(format!(
            "failed to read compiler replacement authorizer registry `{}`: {error}",
            path.display()
        ))
    })?;
    parse_compiler_component_replacement_authorizer_registry_from_source(&source, path)
}

pub fn parse_compiler_component_replacement_authorizer_registry_from_source(
    source: &str,
    path: &Path,
) -> Result<CompilerComponentReplacementAuthorizerRegistry, ArtifactError> {
    validate_text(source, path)?;
    let registry = CompilerComponentReplacementAuthorizerRegistry {
        protocol: parse_required_toml_string(source, "protocol", path)?,
        generation: parse_required_toml_usize(source, "generation", path)?,
        entries: parse_authorizer_blocks(source, path)?,
    };
    validate_registry(&registry)?;
    if render_compiler_component_replacement_authorizer_registry(&registry) != source {
        return Err(ArtifactError::new(format!(
            "compiler replacement authorizer registry `{}` is not canonically encoded",
            path.display()
        )));
    }
    Ok(registry)
}

pub fn compiler_component_replacement_authorizer_registry_sha256(source: &str) -> String {
    sha256_hex(source.as_bytes())
}

pub fn compiler_component_replacement_authorizer_public_key_id(
    public_key_hex: &str,
) -> Result<String, ArtifactError> {
    let bytes = decode_array::<32>(public_key_hex, "replacement authorizer public key")?;
    let key = VerifyingKey::from_bytes(&bytes).map_err(|_| {
        ArtifactError::new("compiler replacement authorizer public key is invalid Ed25519")
    })?;
    Ok(public_key_id(&key))
}

pub(crate) fn resolve_replacement_authorizer_key(
    registry: &CompilerComponentReplacementAuthorizerRegistry,
    component_id: &str,
    authorizer_id: &str,
    environment_id: &str,
    public_key_id_claim: &str,
) -> Result<VerifyingKey, ArtifactError> {
    validate_registry(registry)?;
    let matching = registry
        .entries
        .iter()
        .find(|entry| {
            entry.component_id == component_id
                && entry.authorizer_id == authorizer_id
                && entry.environment_id == environment_id
                && entry.public_key_id == public_key_id_claim
        })
        .ok_or_else(|| {
            ArtifactError::new("compiler replacement authorizer key is not in the pinned registry")
        })?;
    if matching.status == REVOKED_STATUS {
        return Err(ArtifactError::new(
            "compiler replacement authorizer key is revoked in the pinned registry",
        ));
    }
    if matching.status != ACTIVE_STATUS
        || matching.trust_scope != COMPILER_COMPONENT_REPLACEMENT_AUTHORIZER_TRUST_SCOPE
    {
        return Err(ArtifactError::new(
            "compiler replacement authorizer key lacks component-owner trust scope",
        ));
    }
    let bytes = decode_array::<32>(
        &matching.public_key_hex,
        "replacement authorizer public key",
    )?;
    VerifyingKey::from_bytes(&bytes).map_err(|_| {
        ArtifactError::new("compiler replacement authorizer public key is invalid Ed25519")
    })
}

fn validate_registry(
    registry: &CompilerComponentReplacementAuthorizerRegistry,
) -> Result<(), ArtifactError> {
    if registry.protocol != COMPILER_COMPONENT_REPLACEMENT_AUTHORIZER_REGISTRY_PROTOCOL
        || registry.generation == 0
        || registry.entries.is_empty()
    {
        return Err(ArtifactError::new(
            "compiler replacement authorizer registry header is invalid",
        ));
    }
    let mut identities = BTreeSet::new();
    let mut active_components = BTreeSet::new();
    let mut previous = None;
    for entry in &registry.entries {
        validate_token(&entry.authorizer_id, "replacement authorizer id")?;
        validate_token(
            &entry.environment_id,
            "replacement authorizer environment id",
        )?;
        validate_token(&entry.component_id, "replacement component id")?;
        if entry.trust_scope != COMPILER_COMPONENT_REPLACEMENT_AUTHORIZER_TRUST_SCOPE
            || !matches!(entry.status.as_str(), ACTIVE_STATUS | REVOKED_STATUS)
        {
            return Err(ArtifactError::new(
                "compiler replacement authorizer entry has invalid scope or status",
            ));
        }
        let expected_id =
            compiler_component_replacement_authorizer_public_key_id(&entry.public_key_hex)?;
        if entry.public_key_id != expected_id {
            return Err(ArtifactError::new(
                "compiler replacement authorizer public key identity mismatch",
            ));
        }
        let identity = (
            entry.component_id.clone(),
            entry.authorizer_id.clone(),
            entry.environment_id.clone(),
            entry.public_key_id.clone(),
        );
        if previous.as_ref().is_some_and(|value| value >= &identity) {
            return Err(ArtifactError::new(
                "compiler replacement authorizer entries must be uniquely sorted",
            ));
        }
        previous = Some(identity.clone());
        if !identities.insert(identity) {
            return Err(ArtifactError::new(
                "compiler replacement authorizer registry repeats a key identity",
            ));
        }
        if entry.status == ACTIVE_STATUS
            && !active_components.insert((
                entry.component_id.clone(),
                entry.authorizer_id.clone(),
                entry.environment_id.clone(),
            ))
        {
            return Err(ArtifactError::new(
                "compiler replacement authorizer registry permits one active key per component role",
            ));
        }
    }
    Ok(())
}

fn parse_authorizer_blocks(
    source: &str,
    path: &Path,
) -> Result<Vec<CompilerComponentReplacementAuthorizerEntry>, ArtifactError> {
    let mut entries = Vec::new();
    let mut values = BTreeMap::new();
    let mut in_block = false;
    for raw in source.lines() {
        let line = raw.trim();
        if line == "[[authorizer]]" {
            if in_block {
                entries.push(parse_authorizer(&values, path)?);
                values.clear();
            }
            in_block = true;
            continue;
        }
        if in_block && !line.is_empty() && !line.starts_with('#') {
            let Some((key, value)) = line.split_once('=') else {
                return Err(ArtifactError::new(format!(
                    "`{}` contains malformed replacement authorizer line `{line}`",
                    path.display()
                )));
            };
            let key = key.trim().to_owned();
            if values
                .insert(key.clone(), value.trim().to_owned())
                .is_some()
            {
                return Err(ArtifactError::new(format!(
                    "`{}` replacement authorizer repeats key `{key}`",
                    path.display()
                )));
            }
        }
    }
    if in_block {
        entries.push(parse_authorizer(&values, path)?);
    }
    Ok(entries)
}

fn parse_authorizer(
    values: &BTreeMap<String, String>,
    path: &Path,
) -> Result<CompilerComponentReplacementAuthorizerEntry, ArtifactError> {
    let string = |key| parse_required_map_string_in_block(values, key, path, "authorizer");
    Ok(CompilerComponentReplacementAuthorizerEntry {
        authorizer_id: string("authorizer_id")?,
        environment_id: string("environment_id")?,
        component_id: string("component_id")?,
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
            "compiler replacement authorizer registry `{}` must be UTF-8/LF text",
            path.display()
        )));
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
