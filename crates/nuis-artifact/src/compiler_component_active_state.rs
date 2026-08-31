use std::{fs, path::Path};

use sha2::{Digest, Sha256};

use crate::{
    compiler_component_attestation_registry::encode_hex,
    parse_compiler_component_replacement_authorization_from_source,
    render_compiler_component_replacement_authorization,
    toml::{
        escape_toml_string, parse_required_toml_bool, parse_required_toml_string,
        parse_required_toml_usize,
    },
    ArtifactError, CompilerComponentReplacementAuthorization,
    COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_ACTION,
    COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_FILE,
    COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_PROTOCOL,
};

pub const COMPILER_COMPONENT_ACTIVE_STATE_PROTOCOL: &str =
    "nuis-compiler-component-active-state-v1";
pub const COMPILER_COMPONENT_ACTIVE_STATE_FILE: &str = "nuis.compiler-component-active-state.toml";
pub const COMPILER_COMPONENT_ACTIVE_STATE_AUTHORITY: &str =
    "verified-replacement-authorization-consumer";
pub const COMPILER_COMPONENT_ACTIVE_SELECTION_CONTRACT: &str =
    "nuis-compiler-active-component-selector-v1";
pub const COMPILER_COMPONENT_ACTIVE_STATE_VERDICT: &str =
    "candidate-active-stage0-rollback-retained";
pub const COMPILER_COMPONENT_ACTIVE_SELECTOR: &str = "active";
pub const COMPILER_COMPONENT_ROLLBACK_SELECTOR: &str = "rollback";

const GENESIS_PREDECESSOR_STATE_SHA256: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";
const ACTIVE_STAGE_ROLE: &str = "stage1-candidate";
const ROLLBACK_STAGE_ROLE: &str = "stage0";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilerComponentActiveSelection {
    Active,
    Rollback,
}

impl CompilerComponentActiveSelection {
    pub fn parse(value: &str) -> Result<Self, ArtifactError> {
        match value {
            COMPILER_COMPONENT_ACTIVE_SELECTOR => Ok(Self::Active),
            COMPILER_COMPONENT_ROLLBACK_SELECTOR => Ok(Self::Rollback),
            _ => Err(ArtifactError::new(
                "compiler active-component selector must be `active` or `rollback`",
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerComponentActiveTarget {
    pub component_id: String,
    pub selector: String,
    pub stage_role: String,
    pub reproducible_build_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerComponentActiveState {
    pub protocol: String,
    pub authority: String,
    pub selection_contract: String,
    pub component_id: String,
    pub generation: usize,
    pub predecessor_state_sha256: String,
    pub authorization_protocol: String,
    pub authorization_file: String,
    pub authorization_file_bytes: usize,
    pub authorization_file_sha256: String,
    pub authorization_id: String,
    pub authorization_generation: usize,
    pub authorization_predecessor_sha256: String,
    pub authorization_proof_sha256: String,
    pub authorization_attestation_proof_sha256: String,
    pub active_selector: String,
    pub active_stage_role: String,
    pub active_reproducible_build_sha256: String,
    pub active_compiler_image_sha256: String,
    pub rollback_selector: String,
    pub rollback_stage_role: String,
    pub rollback_reproducible_build_sha256: String,
    pub native_output_sha256: String,
    pub reversible: bool,
    pub verdict: String,
    pub state_sha256: String,
}

pub fn build_compiler_component_active_state(
    authorization: &CompilerComponentReplacementAuthorization,
    authorization_source: &str,
) -> Result<CompilerComponentActiveState, ArtifactError> {
    validate_authorization_source(authorization, authorization_source)?;
    let mut state = CompilerComponentActiveState {
        protocol: COMPILER_COMPONENT_ACTIVE_STATE_PROTOCOL.to_owned(),
        authority: COMPILER_COMPONENT_ACTIVE_STATE_AUTHORITY.to_owned(),
        selection_contract: COMPILER_COMPONENT_ACTIVE_SELECTION_CONTRACT.to_owned(),
        component_id: authorization.component_id.clone(),
        generation: authorization.generation,
        predecessor_state_sha256: GENESIS_PREDECESSOR_STATE_SHA256.to_owned(),
        authorization_protocol: authorization.protocol.clone(),
        authorization_file: COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_FILE.to_owned(),
        authorization_file_bytes: authorization_source.len(),
        authorization_file_sha256: sha256_hex(authorization_source.as_bytes()),
        authorization_id: authorization.authorization_id.clone(),
        authorization_generation: authorization.generation,
        authorization_predecessor_sha256: authorization.predecessor_authorization_sha256.clone(),
        authorization_proof_sha256: authorization.proof_sha256.clone(),
        authorization_attestation_proof_sha256: authorization.attestation_proof_sha256.clone(),
        active_selector: COMPILER_COMPONENT_ACTIVE_SELECTOR.to_owned(),
        active_stage_role: ACTIVE_STAGE_ROLE.to_owned(),
        active_reproducible_build_sha256: authorization.to_reproducible_build_sha256.clone(),
        active_compiler_image_sha256: authorization.candidate_compiler_image_sha256.clone(),
        rollback_selector: COMPILER_COMPONENT_ROLLBACK_SELECTOR.to_owned(),
        rollback_stage_role: ROLLBACK_STAGE_ROLE.to_owned(),
        rollback_reproducible_build_sha256: authorization
            .rollback_reproducible_build_sha256
            .clone(),
        native_output_sha256: authorization.native_output_sha256.clone(),
        reversible: authorization.reversible,
        verdict: COMPILER_COMPONENT_ACTIVE_STATE_VERDICT.to_owned(),
        state_sha256: String::new(),
    };
    state.state_sha256 = active_state_identity(&state);
    validate_active_state(&state)?;
    Ok(state)
}

pub fn verify_compiler_component_active_state(
    state: &CompilerComponentActiveState,
    authorization: &CompilerComponentReplacementAuthorization,
    authorization_source: &str,
) -> Result<(), ArtifactError> {
    validate_active_state(state)?;
    let expected = build_compiler_component_active_state(authorization, authorization_source)?;
    if state != &expected {
        return Err(ArtifactError::new(
            "compiler active-component state does not match its verified authorization",
        ));
    }
    Ok(())
}

pub fn select_compiler_component_active_target(
    state: &CompilerComponentActiveState,
    authorization: &CompilerComponentReplacementAuthorization,
    authorization_source: &str,
    selection: CompilerComponentActiveSelection,
) -> Result<CompilerComponentActiveTarget, ArtifactError> {
    verify_compiler_component_active_state(state, authorization, authorization_source)?;
    let (selector, stage_role, reproducible_build_sha256) = match selection {
        CompilerComponentActiveSelection::Active => (
            &state.active_selector,
            &state.active_stage_role,
            &state.active_reproducible_build_sha256,
        ),
        CompilerComponentActiveSelection::Rollback => (
            &state.rollback_selector,
            &state.rollback_stage_role,
            &state.rollback_reproducible_build_sha256,
        ),
    };
    Ok(CompilerComponentActiveTarget {
        component_id: state.component_id.clone(),
        selector: selector.clone(),
        stage_role: stage_role.clone(),
        reproducible_build_sha256: reproducible_build_sha256.clone(),
    })
}

pub fn render_compiler_component_active_state(state: &CompilerComponentActiveState) -> String {
    format!(
        "protocol = \"{}\"\nauthority = \"{}\"\nselection_contract = \"{}\"\ncomponent_id = \"{}\"\ngeneration = {}\npredecessor_state_sha256 = \"{}\"\nauthorization_protocol = \"{}\"\nauthorization_file = \"{}\"\nauthorization_file_bytes = {}\nauthorization_file_sha256 = \"{}\"\nauthorization_id = \"{}\"\nauthorization_generation = {}\nauthorization_predecessor_sha256 = \"{}\"\nauthorization_proof_sha256 = \"{}\"\nauthorization_attestation_proof_sha256 = \"{}\"\nactive_selector = \"{}\"\nactive_stage_role = \"{}\"\nactive_reproducible_build_sha256 = \"{}\"\nactive_compiler_image_sha256 = \"{}\"\nrollback_selector = \"{}\"\nrollback_stage_role = \"{}\"\nrollback_reproducible_build_sha256 = \"{}\"\nnative_output_sha256 = \"{}\"\nreversible = {}\nverdict = \"{}\"\nstate_sha256 = \"{}\"\n",
        state.protocol,
        state.authority,
        state.selection_contract,
        escape_toml_string(&state.component_id),
        state.generation,
        state.predecessor_state_sha256,
        state.authorization_protocol,
        state.authorization_file,
        state.authorization_file_bytes,
        state.authorization_file_sha256,
        escape_toml_string(&state.authorization_id),
        state.authorization_generation,
        state.authorization_predecessor_sha256,
        state.authorization_proof_sha256,
        state.authorization_attestation_proof_sha256,
        state.active_selector,
        state.active_stage_role,
        state.active_reproducible_build_sha256,
        state.active_compiler_image_sha256,
        state.rollback_selector,
        state.rollback_stage_role,
        state.rollback_reproducible_build_sha256,
        state.native_output_sha256,
        state.reversible,
        state.verdict,
        state.state_sha256,
    )
}

pub fn parse_compiler_component_active_state(
    path: &Path,
) -> Result<CompilerComponentActiveState, ArtifactError> {
    let source = fs::read_to_string(path).map_err(|error| {
        ArtifactError::new(format!(
            "failed to read compiler active-component state `{}`: {error}",
            path.display()
        ))
    })?;
    parse_compiler_component_active_state_from_source(&source, path)
}

pub fn parse_compiler_component_active_state_from_source(
    source: &str,
    path: &Path,
) -> Result<CompilerComponentActiveState, ArtifactError> {
    validate_text(source, path)?;
    let state = CompilerComponentActiveState {
        protocol: parse_required_toml_string(source, "protocol", path)?,
        authority: parse_required_toml_string(source, "authority", path)?,
        selection_contract: parse_required_toml_string(source, "selection_contract", path)?,
        component_id: parse_required_toml_string(source, "component_id", path)?,
        generation: parse_required_toml_usize(source, "generation", path)?,
        predecessor_state_sha256: parse_required_toml_string(
            source,
            "predecessor_state_sha256",
            path,
        )?,
        authorization_protocol: parse_required_toml_string(source, "authorization_protocol", path)?,
        authorization_file: parse_required_toml_string(source, "authorization_file", path)?,
        authorization_file_bytes: parse_required_toml_usize(
            source,
            "authorization_file_bytes",
            path,
        )?,
        authorization_file_sha256: parse_required_toml_string(
            source,
            "authorization_file_sha256",
            path,
        )?,
        authorization_id: parse_required_toml_string(source, "authorization_id", path)?,
        authorization_generation: parse_required_toml_usize(
            source,
            "authorization_generation",
            path,
        )?,
        authorization_predecessor_sha256: parse_required_toml_string(
            source,
            "authorization_predecessor_sha256",
            path,
        )?,
        authorization_proof_sha256: parse_required_toml_string(
            source,
            "authorization_proof_sha256",
            path,
        )?,
        authorization_attestation_proof_sha256: parse_required_toml_string(
            source,
            "authorization_attestation_proof_sha256",
            path,
        )?,
        active_selector: parse_required_toml_string(source, "active_selector", path)?,
        active_stage_role: parse_required_toml_string(source, "active_stage_role", path)?,
        active_reproducible_build_sha256: parse_required_toml_string(
            source,
            "active_reproducible_build_sha256",
            path,
        )?,
        active_compiler_image_sha256: parse_required_toml_string(
            source,
            "active_compiler_image_sha256",
            path,
        )?,
        rollback_selector: parse_required_toml_string(source, "rollback_selector", path)?,
        rollback_stage_role: parse_required_toml_string(source, "rollback_stage_role", path)?,
        rollback_reproducible_build_sha256: parse_required_toml_string(
            source,
            "rollback_reproducible_build_sha256",
            path,
        )?,
        native_output_sha256: parse_required_toml_string(source, "native_output_sha256", path)?,
        reversible: parse_required_toml_bool(source, "reversible", path)?,
        verdict: parse_required_toml_string(source, "verdict", path)?,
        state_sha256: parse_required_toml_string(source, "state_sha256", path)?,
    };
    validate_active_state(&state)?;
    if render_compiler_component_active_state(&state) != source {
        return Err(ArtifactError::new(format!(
            "compiler active-component state `{}` is not canonically encoded",
            path.display()
        )));
    }
    Ok(state)
}

fn validate_authorization_source(
    authorization: &CompilerComponentReplacementAuthorization,
    source: &str,
) -> Result<(), ArtifactError> {
    let parsed = parse_compiler_component_replacement_authorization_from_source(
        source,
        Path::new(COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_FILE),
    )?;
    if &parsed != authorization
        || render_compiler_component_replacement_authorization(authorization) != source
        || authorization.protocol != COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_PROTOCOL
        || authorization.action != COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_ACTION
        || !authorization.reversible
        || !authorization.replacement_authorized
        || authorization.attestation_replacement_authorized
    {
        return Err(ArtifactError::new(
            "compiler active-component state requires one canonical reversible replacement authorization",
        ));
    }
    Ok(())
}

fn validate_active_state(state: &CompilerComponentActiveState) -> Result<(), ArtifactError> {
    if state.protocol != COMPILER_COMPONENT_ACTIVE_STATE_PROTOCOL
        || state.authority != COMPILER_COMPONENT_ACTIVE_STATE_AUTHORITY
        || state.selection_contract != COMPILER_COMPONENT_ACTIVE_SELECTION_CONTRACT
        || state.generation != 1
        || state.predecessor_state_sha256 != GENESIS_PREDECESSOR_STATE_SHA256
        || state.authorization_protocol != COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_PROTOCOL
        || state.authorization_file != COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_FILE
        || state.authorization_generation != 1
        || state.authorization_predecessor_sha256 != GENESIS_PREDECESSOR_STATE_SHA256
        || state.active_selector != COMPILER_COMPONENT_ACTIVE_SELECTOR
        || state.active_stage_role != ACTIVE_STAGE_ROLE
        || state.rollback_selector != COMPILER_COMPONENT_ROLLBACK_SELECTOR
        || state.rollback_stage_role != ROLLBACK_STAGE_ROLE
        || !state.reversible
        || state.verdict != COMPILER_COMPONENT_ACTIVE_STATE_VERDICT
        || state.active_reproducible_build_sha256 == state.rollback_reproducible_build_sha256
        || state.authorization_file_bytes == 0
    {
        return Err(ArtifactError::new(
            "compiler active-component state contract mismatch",
        ));
    }
    validate_token(&state.component_id, "active-state component id")?;
    validate_token(&state.authorization_id, "active-state authorization id")?;
    for (label, value) in [
        ("predecessor state", &state.predecessor_state_sha256),
        ("authorization file", &state.authorization_file_sha256),
        (
            "authorization predecessor",
            &state.authorization_predecessor_sha256,
        ),
        ("authorization proof", &state.authorization_proof_sha256),
        (
            "authorization attestation proof",
            &state.authorization_attestation_proof_sha256,
        ),
        (
            "active reproducible build",
            &state.active_reproducible_build_sha256,
        ),
        ("active compiler image", &state.active_compiler_image_sha256),
        (
            "rollback reproducible build",
            &state.rollback_reproducible_build_sha256,
        ),
        ("native output", &state.native_output_sha256),
        ("active state", &state.state_sha256),
    ] {
        validate_sha256(value, label)?;
    }
    if state.state_sha256 != active_state_identity(state) {
        return Err(ArtifactError::new(
            "compiler active-component state identity mismatch",
        ));
    }
    Ok(())
}

fn active_state_identity(state: &CompilerComponentActiveState) -> String {
    let mut hash = Sha256::new();
    for value in [
        state.protocol.as_bytes(),
        state.authority.as_bytes(),
        state.selection_contract.as_bytes(),
        state.component_id.as_bytes(),
        state.predecessor_state_sha256.as_bytes(),
        state.authorization_protocol.as_bytes(),
        state.authorization_file.as_bytes(),
        state.authorization_file_sha256.as_bytes(),
        state.authorization_id.as_bytes(),
        state.authorization_predecessor_sha256.as_bytes(),
        state.authorization_proof_sha256.as_bytes(),
        state.authorization_attestation_proof_sha256.as_bytes(),
        state.active_selector.as_bytes(),
        state.active_stage_role.as_bytes(),
        state.active_reproducible_build_sha256.as_bytes(),
        state.active_compiler_image_sha256.as_bytes(),
        state.rollback_selector.as_bytes(),
        state.rollback_stage_role.as_bytes(),
        state.rollback_reproducible_build_sha256.as_bytes(),
        state.native_output_sha256.as_bytes(),
        state.verdict.as_bytes(),
    ] {
        hash_field(&mut hash, value);
    }
    for value in [
        state.generation,
        state.authorization_file_bytes,
        state.authorization_generation,
        usize::from(state.reversible),
    ] {
        hash_field(&mut hash, &(value as u64).to_le_bytes());
    }
    encode_hex(&hash.finalize())
}

fn hash_field(hash: &mut Sha256, value: &[u8]) {
    hash.update((value.len() as u64).to_le_bytes());
    hash.update(value);
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

fn validate_sha256(value: &str, label: &str) -> Result<(), ArtifactError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(ArtifactError::new(format!(
            "compiler {label} must be lowercase SHA-256"
        )))
    }
}

fn validate_text(source: &str, path: &Path) -> Result<(), ArtifactError> {
    if source.is_empty()
        || !source.ends_with('\n')
        || source.contains('\r')
        || source.contains('\0')
    {
        return Err(ArtifactError::new(format!(
            "compiler active-component state `{}` must be UTF-8/LF text",
            path.display()
        )));
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    encode_hex(&Sha256::digest(bytes))
}

#[cfg(test)]
#[path = "compiler_component_active_state_tests.rs"]
mod tests;
