use std::path::Path;

use ed25519_dalek::{Signature, Signer, SigningKey};
use sha2::{Digest, Sha256};

use crate::{
    compiler_component_attestation_registry::{decode_array, encode_hex, public_key_id},
    compiler_component_replacement_registry::resolve_replacement_authorizer_key,
    parse_compiler_component_active_state_from_source,
    parse_compiler_component_replacement_authorization_from_source,
    parse_compiler_component_replacement_authorizer_registry_from_source,
    render_compiler_component_active_state, render_compiler_component_replacement_authorization,
    render_compiler_component_replacement_authorizer_registry,
    toml::escape_toml_string,
    verify_compiler_component_active_state, ArtifactError, CompilerComponentActiveState,
    CompilerComponentReplacementAuthorization, CompilerComponentReplacementAuthorizerRegistry,
    COMPILER_COMPONENT_ACTIVE_STATE_FILE, COMPILER_COMPONENT_ACTIVE_STATE_PROTOCOL,
    COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_FILE,
    COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_PROTOCOL,
    COMPILER_COMPONENT_REPLACEMENT_AUTHORIZER_REGISTRY_FILE,
};

#[path = "compiler_component_transition_parse.rs"]
mod parse;

pub use parse::{
    parse_compiler_component_transition, parse_compiler_component_transition_from_source,
};

pub const COMPILER_COMPONENT_TRANSITION_PROTOCOL: &str = "nuis-compiler-component-transition-v2";
pub const COMPILER_COMPONENT_TRANSITION_FILE: &str = "nuis.compiler-component-transition.toml";
pub const COMPILER_COMPONENT_TRANSITION_AUTHORITY: &str =
    "independent-ed25519-component-owner-transition";
pub const COMPILER_COMPONENT_TRANSITION_SIGNATURE_CONTRACT: &str =
    "nuis-compiler-component-transition-ed25519-v2";
pub const COMPILER_COMPONENT_TRANSITION_ACTION: &str = "rollback-stage0";
pub const COMPILER_COMPONENT_TRANSITION_VERDICT: &str =
    "stage0-restored-candidate-forward-retained";

const TRANSITION_GENERATION: usize = 2;
const PREDECESSOR_GENERATION: usize = 1;
const FROM_SELECTOR: &str = "active";
const FROM_STAGE_ROLE: &str = "stage1-candidate";
const CURRENT_SELECTOR: &str = "current";
const CURRENT_STAGE_ROLE: &str = "stage0";
const FORWARD_SELECTOR: &str = "forward";
const FORWARD_STAGE_ROLE: &str = "stage1-candidate";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilerComponentTransitionSelection {
    Current,
    Forward,
}

impl CompilerComponentTransitionSelection {
    pub fn parse(value: &str) -> Result<Self, ArtifactError> {
        match value {
            CURRENT_SELECTOR => Ok(Self::Current),
            FORWARD_SELECTOR => Ok(Self::Forward),
            _ => Err(ArtifactError::new(
                "compiler transition selector must be `current` or `forward`",
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerComponentTransitionTarget {
    pub component_id: String,
    pub selector: String,
    pub stage_role: String,
    pub reproducible_build_sha256: String,
}

#[derive(Debug, Clone, Copy)]
pub struct CompilerComponentTransitionInput<'a> {
    pub authorization: &'a CompilerComponentReplacementAuthorization,
    pub authorization_source: &'a str,
    pub active_state: &'a CompilerComponentActiveState,
    pub active_state_source: &'a str,
    pub challenge_sha256: &'a str,
    pub transition_id: &'a str,
    pub authorizer_id: &'a str,
    pub environment_id: &'a str,
}

#[derive(Debug, Clone, Copy)]
pub struct CompilerComponentTransitionVerificationInput<'a> {
    pub authorization: &'a CompilerComponentReplacementAuthorization,
    pub authorization_source: &'a str,
    pub active_state: &'a CompilerComponentActiveState,
    pub active_state_source: &'a str,
    pub authorizer_registry: &'a CompilerComponentReplacementAuthorizerRegistry,
    pub authorizer_registry_source: &'a str,
    pub expected_authorizer_registry_sha256: &'a str,
    pub expected_transition_challenge_sha256: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerComponentTransition {
    pub protocol: String,
    pub authority: String,
    pub signature_contract: String,
    pub action: String,
    pub component_id: String,
    pub transition_id: String,
    pub generation: usize,
    pub predecessor_authorization_protocol: String,
    pub predecessor_authorization_file: String,
    pub predecessor_authorization_file_bytes: usize,
    pub predecessor_authorization_file_sha256: String,
    pub predecessor_authorization_id: String,
    pub predecessor_authorization_generation: usize,
    pub predecessor_authorization_proof_sha256: String,
    pub predecessor_state_protocol: String,
    pub predecessor_state_file: String,
    pub predecessor_state_file_bytes: usize,
    pub predecessor_state_file_sha256: String,
    pub predecessor_state_generation: usize,
    pub predecessor_state_sha256: String,
    pub challenge_sha256: String,
    pub from_selector: String,
    pub from_stage_role: String,
    pub from_reproducible_build_sha256: String,
    pub current_selector: String,
    pub current_stage_role: String,
    pub current_reproducible_build_sha256: String,
    pub forward_selector: String,
    pub forward_stage_role: String,
    pub forward_reproducible_build_sha256: String,
    pub candidate_compiler_image_sha256: String,
    pub native_output_sha256: String,
    pub reversible: bool,
    pub authorizer_id: String,
    pub authorizer_environment_id: String,
    pub authorizer_public_key_id: String,
    pub verdict: String,
    pub proof_sha256: String,
    pub signature_hex: String,
}

pub fn build_compiler_component_transition(
    input: CompilerComponentTransitionInput<'_>,
    signing_key_hex: &str,
) -> Result<CompilerComponentTransition, ArtifactError> {
    validate_sha256(input.challenge_sha256, "component transition challenge")?;
    validate_token(input.transition_id, "component transition id")?;
    validate_token(input.authorizer_id, "component transition authorizer id")?;
    validate_token(
        input.environment_id,
        "component transition authorizer environment id",
    )?;
    validate_predecessor_sources(
        input.authorization,
        input.authorization_source,
        input.active_state,
        input.active_state_source,
    )?;
    if input.authorizer_id != input.authorization.authorizer_id
        || input.environment_id != input.authorization.authorizer_environment_id
    {
        return Err(ArtifactError::new(
            "compiler generation-two transition must retain the genesis component-owner role",
        ));
    }
    let signing_key = SigningKey::from_bytes(&decode_array::<32>(
        signing_key_hex,
        "component transition authorizer signing key",
    )?);
    let authorizer_public_key_id = public_key_id(&signing_key.verifying_key());
    if authorizer_public_key_id != input.authorization.authorizer_public_key_id {
        return Err(ArtifactError::new(
            "compiler generation-two transition must retain the genesis component-owner key",
        ));
    }

    let mut transition = CompilerComponentTransition {
        protocol: COMPILER_COMPONENT_TRANSITION_PROTOCOL.to_owned(),
        authority: COMPILER_COMPONENT_TRANSITION_AUTHORITY.to_owned(),
        signature_contract: COMPILER_COMPONENT_TRANSITION_SIGNATURE_CONTRACT.to_owned(),
        action: COMPILER_COMPONENT_TRANSITION_ACTION.to_owned(),
        component_id: input.authorization.component_id.clone(),
        transition_id: input.transition_id.to_owned(),
        generation: TRANSITION_GENERATION,
        predecessor_authorization_protocol: input.authorization.protocol.clone(),
        predecessor_authorization_file: COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_FILE
            .to_owned(),
        predecessor_authorization_file_bytes: input.authorization_source.len(),
        predecessor_authorization_file_sha256: sha256_hex(input.authorization_source.as_bytes()),
        predecessor_authorization_id: input.authorization.authorization_id.clone(),
        predecessor_authorization_generation: input.authorization.generation,
        predecessor_authorization_proof_sha256: input.authorization.proof_sha256.clone(),
        predecessor_state_protocol: input.active_state.protocol.clone(),
        predecessor_state_file: COMPILER_COMPONENT_ACTIVE_STATE_FILE.to_owned(),
        predecessor_state_file_bytes: input.active_state_source.len(),
        predecessor_state_file_sha256: sha256_hex(input.active_state_source.as_bytes()),
        predecessor_state_generation: input.active_state.generation,
        predecessor_state_sha256: input.active_state.state_sha256.clone(),
        challenge_sha256: input.challenge_sha256.to_owned(),
        from_selector: FROM_SELECTOR.to_owned(),
        from_stage_role: FROM_STAGE_ROLE.to_owned(),
        from_reproducible_build_sha256: input.active_state.active_reproducible_build_sha256.clone(),
        current_selector: CURRENT_SELECTOR.to_owned(),
        current_stage_role: CURRENT_STAGE_ROLE.to_owned(),
        current_reproducible_build_sha256: input
            .active_state
            .rollback_reproducible_build_sha256
            .clone(),
        forward_selector: FORWARD_SELECTOR.to_owned(),
        forward_stage_role: FORWARD_STAGE_ROLE.to_owned(),
        forward_reproducible_build_sha256: input
            .active_state
            .active_reproducible_build_sha256
            .clone(),
        candidate_compiler_image_sha256: input.active_state.active_compiler_image_sha256.clone(),
        native_output_sha256: input.active_state.native_output_sha256.clone(),
        reversible: true,
        authorizer_id: input.authorizer_id.to_owned(),
        authorizer_environment_id: input.environment_id.to_owned(),
        authorizer_public_key_id,
        verdict: COMPILER_COMPONENT_TRANSITION_VERDICT.to_owned(),
        proof_sha256: String::new(),
        signature_hex: String::new(),
    };
    transition.proof_sha256 = transition_identity(&transition);
    transition.signature_hex = encode_hex(
        &signing_key
            .sign(&signature_message(&transition.proof_sha256))
            .to_bytes(),
    );
    validate_transition(&transition)?;
    Ok(transition)
}

pub fn verify_compiler_component_transition(
    transition: &CompilerComponentTransition,
    input: CompilerComponentTransitionVerificationInput<'_>,
) -> Result<(), ArtifactError> {
    validate_transition(transition)?;
    validate_sha256(
        input.expected_authorizer_registry_sha256,
        "pinned component transition authorizer registry",
    )?;
    validate_sha256(
        input.expected_transition_challenge_sha256,
        "expected component transition challenge",
    )?;
    if transition.challenge_sha256 != input.expected_transition_challenge_sha256 {
        return Err(ArtifactError::new(
            "compiler component transition challenge does not match the verifier request",
        ));
    }
    validate_predecessor_sources(
        input.authorization,
        input.authorization_source,
        input.active_state,
        input.active_state_source,
    )?;
    validate_transition_lineage(transition, input)?;

    let parsed_registry = parse_compiler_component_replacement_authorizer_registry_from_source(
        input.authorizer_registry_source,
        Path::new(COMPILER_COMPONENT_REPLACEMENT_AUTHORIZER_REGISTRY_FILE),
    )?;
    if &parsed_registry != input.authorizer_registry
        || render_compiler_component_replacement_authorizer_registry(input.authorizer_registry)
            != input.authorizer_registry_source
    {
        return Err(ArtifactError::new(
            "compiler component transition authorizer registry source is not canonical",
        ));
    }
    if sha256_hex(input.authorizer_registry_source.as_bytes())
        != input.expected_authorizer_registry_sha256
    {
        return Err(ArtifactError::new(
            "compiler component transition authorizer registry does not match its pinned SHA-256",
        ));
    }
    let verifying_key = resolve_replacement_authorizer_key(
        input.authorizer_registry,
        &transition.component_id,
        &transition.authorizer_id,
        &transition.authorizer_environment_id,
        &transition.authorizer_public_key_id,
    )?;
    let signature = Signature::from_bytes(&decode_array::<64>(
        &transition.signature_hex,
        "component transition signature",
    )?);
    verifying_key
        .verify_strict(&signature_message(&transition.proof_sha256), &signature)
        .map_err(|_| {
            ArtifactError::new("compiler component transition Ed25519 signature mismatch")
        })?;
    Ok(())
}

pub fn select_compiler_component_transition_target(
    transition: &CompilerComponentTransition,
    input: CompilerComponentTransitionVerificationInput<'_>,
    selection: CompilerComponentTransitionSelection,
) -> Result<CompilerComponentTransitionTarget, ArtifactError> {
    verify_compiler_component_transition(transition, input)?;
    let (selector, stage_role, reproducible_build_sha256) = match selection {
        CompilerComponentTransitionSelection::Current => (
            &transition.current_selector,
            &transition.current_stage_role,
            &transition.current_reproducible_build_sha256,
        ),
        CompilerComponentTransitionSelection::Forward => (
            &transition.forward_selector,
            &transition.forward_stage_role,
            &transition.forward_reproducible_build_sha256,
        ),
    };
    Ok(CompilerComponentTransitionTarget {
        component_id: transition.component_id.clone(),
        selector: selector.clone(),
        stage_role: stage_role.clone(),
        reproducible_build_sha256: reproducible_build_sha256.clone(),
    })
}

pub fn render_compiler_component_transition(transition: &CompilerComponentTransition) -> String {
    format!(
        "protocol = \"{}\"\nauthority = \"{}\"\nsignature_contract = \"{}\"\naction = \"{}\"\ncomponent_id = \"{}\"\ntransition_id = \"{}\"\ngeneration = {}\npredecessor_authorization_protocol = \"{}\"\npredecessor_authorization_file = \"{}\"\npredecessor_authorization_file_bytes = {}\npredecessor_authorization_file_sha256 = \"{}\"\npredecessor_authorization_id = \"{}\"\npredecessor_authorization_generation = {}\npredecessor_authorization_proof_sha256 = \"{}\"\npredecessor_state_protocol = \"{}\"\npredecessor_state_file = \"{}\"\npredecessor_state_file_bytes = {}\npredecessor_state_file_sha256 = \"{}\"\npredecessor_state_generation = {}\npredecessor_state_sha256 = \"{}\"\nchallenge_sha256 = \"{}\"\nfrom_selector = \"{}\"\nfrom_stage_role = \"{}\"\nfrom_reproducible_build_sha256 = \"{}\"\ncurrent_selector = \"{}\"\ncurrent_stage_role = \"{}\"\ncurrent_reproducible_build_sha256 = \"{}\"\nforward_selector = \"{}\"\nforward_stage_role = \"{}\"\nforward_reproducible_build_sha256 = \"{}\"\ncandidate_compiler_image_sha256 = \"{}\"\nnative_output_sha256 = \"{}\"\nreversible = {}\nauthorizer_id = \"{}\"\nauthorizer_environment_id = \"{}\"\nauthorizer_public_key_id = \"{}\"\nverdict = \"{}\"\nproof_sha256 = \"{}\"\nsignature_hex = \"{}\"\n",
        transition.protocol,
        transition.authority,
        transition.signature_contract,
        transition.action,
        escape_toml_string(&transition.component_id),
        escape_toml_string(&transition.transition_id),
        transition.generation,
        transition.predecessor_authorization_protocol,
        transition.predecessor_authorization_file,
        transition.predecessor_authorization_file_bytes,
        transition.predecessor_authorization_file_sha256,
        escape_toml_string(&transition.predecessor_authorization_id),
        transition.predecessor_authorization_generation,
        transition.predecessor_authorization_proof_sha256,
        transition.predecessor_state_protocol,
        transition.predecessor_state_file,
        transition.predecessor_state_file_bytes,
        transition.predecessor_state_file_sha256,
        transition.predecessor_state_generation,
        transition.predecessor_state_sha256,
        transition.challenge_sha256,
        transition.from_selector,
        transition.from_stage_role,
        transition.from_reproducible_build_sha256,
        transition.current_selector,
        transition.current_stage_role,
        transition.current_reproducible_build_sha256,
        transition.forward_selector,
        transition.forward_stage_role,
        transition.forward_reproducible_build_sha256,
        transition.candidate_compiler_image_sha256,
        transition.native_output_sha256,
        transition.reversible,
        escape_toml_string(&transition.authorizer_id),
        escape_toml_string(&transition.authorizer_environment_id),
        transition.authorizer_public_key_id,
        transition.verdict,
        transition.proof_sha256,
        transition.signature_hex,
    )
}

fn validate_predecessor_sources(
    authorization: &CompilerComponentReplacementAuthorization,
    authorization_source: &str,
    active_state: &CompilerComponentActiveState,
    active_state_source: &str,
) -> Result<(), ArtifactError> {
    let parsed_authorization = parse_compiler_component_replacement_authorization_from_source(
        authorization_source,
        Path::new(COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_FILE),
    )?;
    let parsed_state = parse_compiler_component_active_state_from_source(
        active_state_source,
        Path::new(COMPILER_COMPONENT_ACTIVE_STATE_FILE),
    )?;
    if &parsed_authorization != authorization
        || render_compiler_component_replacement_authorization(authorization)
            != authorization_source
        || &parsed_state != active_state
        || render_compiler_component_active_state(active_state) != active_state_source
    {
        return Err(ArtifactError::new(
            "compiler component transition predecessor sources are not canonical",
        ));
    }
    verify_compiler_component_active_state(active_state, authorization, authorization_source)?;
    Ok(())
}

fn validate_transition_lineage(
    transition: &CompilerComponentTransition,
    input: CompilerComponentTransitionVerificationInput<'_>,
) -> Result<(), ArtifactError> {
    if transition.component_id != input.authorization.component_id
        || transition.predecessor_authorization_protocol != input.authorization.protocol
        || transition.predecessor_authorization_file_bytes != input.authorization_source.len()
        || transition.predecessor_authorization_file_sha256
            != sha256_hex(input.authorization_source.as_bytes())
        || transition.predecessor_authorization_id != input.authorization.authorization_id
        || transition.predecessor_authorization_generation != input.authorization.generation
        || transition.predecessor_authorization_proof_sha256 != input.authorization.proof_sha256
        || transition.predecessor_state_protocol != input.active_state.protocol
        || transition.predecessor_state_file_bytes != input.active_state_source.len()
        || transition.predecessor_state_file_sha256
            != sha256_hex(input.active_state_source.as_bytes())
        || transition.predecessor_state_generation != input.active_state.generation
        || transition.predecessor_state_sha256 != input.active_state.state_sha256
        || transition.from_reproducible_build_sha256
            != input.active_state.active_reproducible_build_sha256
        || transition.current_reproducible_build_sha256
            != input.active_state.rollback_reproducible_build_sha256
        || transition.forward_reproducible_build_sha256
            != input.active_state.active_reproducible_build_sha256
        || transition.candidate_compiler_image_sha256
            != input.active_state.active_compiler_image_sha256
        || transition.native_output_sha256 != input.active_state.native_output_sha256
        || transition.authorizer_id != input.authorization.authorizer_id
        || transition.authorizer_environment_id != input.authorization.authorizer_environment_id
        || transition.authorizer_public_key_id != input.authorization.authorizer_public_key_id
    {
        return Err(ArtifactError::new(
            "compiler component transition predecessor lineage mismatch",
        ));
    }
    Ok(())
}

fn validate_transition(transition: &CompilerComponentTransition) -> Result<(), ArtifactError> {
    if transition.protocol != COMPILER_COMPONENT_TRANSITION_PROTOCOL
        || transition.authority != COMPILER_COMPONENT_TRANSITION_AUTHORITY
        || transition.signature_contract != COMPILER_COMPONENT_TRANSITION_SIGNATURE_CONTRACT
        || transition.action != COMPILER_COMPONENT_TRANSITION_ACTION
        || transition.generation != TRANSITION_GENERATION
        || transition.predecessor_authorization_protocol
            != COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_PROTOCOL
        || transition.predecessor_authorization_file
            != COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_FILE
        || transition.predecessor_authorization_generation != PREDECESSOR_GENERATION
        || transition.predecessor_state_protocol != COMPILER_COMPONENT_ACTIVE_STATE_PROTOCOL
        || transition.predecessor_state_file != COMPILER_COMPONENT_ACTIVE_STATE_FILE
        || transition.predecessor_state_generation != PREDECESSOR_GENERATION
        || transition.from_selector != FROM_SELECTOR
        || transition.from_stage_role != FROM_STAGE_ROLE
        || transition.current_selector != CURRENT_SELECTOR
        || transition.current_stage_role != CURRENT_STAGE_ROLE
        || transition.forward_selector != FORWARD_SELECTOR
        || transition.forward_stage_role != FORWARD_STAGE_ROLE
        || !transition.reversible
        || transition.verdict != COMPILER_COMPONENT_TRANSITION_VERDICT
        || transition.from_reproducible_build_sha256 == transition.current_reproducible_build_sha256
        || transition.forward_reproducible_build_sha256 != transition.from_reproducible_build_sha256
        || transition.predecessor_authorization_file_bytes == 0
        || transition.predecessor_state_file_bytes == 0
    {
        return Err(ArtifactError::new(
            "compiler component transition contract mismatch",
        ));
    }
    for (value, label) in [
        (
            &transition.component_id,
            "component transition component id",
        ),
        (&transition.transition_id, "component transition id"),
        (
            &transition.predecessor_authorization_id,
            "predecessor authorization id",
        ),
        (
            &transition.authorizer_id,
            "component transition authorizer id",
        ),
        (
            &transition.authorizer_environment_id,
            "component transition authorizer environment id",
        ),
    ] {
        validate_token(value, label)?;
    }
    for (label, value) in [
        (
            "predecessor authorization file",
            &transition.predecessor_authorization_file_sha256,
        ),
        (
            "predecessor authorization proof",
            &transition.predecessor_authorization_proof_sha256,
        ),
        (
            "predecessor active state file",
            &transition.predecessor_state_file_sha256,
        ),
        (
            "predecessor active state",
            &transition.predecessor_state_sha256,
        ),
        (
            "component transition challenge",
            &transition.challenge_sha256,
        ),
        (
            "component transition from build",
            &transition.from_reproducible_build_sha256,
        ),
        (
            "component transition current build",
            &transition.current_reproducible_build_sha256,
        ),
        (
            "component transition forward build",
            &transition.forward_reproducible_build_sha256,
        ),
        (
            "component transition candidate image",
            &transition.candidate_compiler_image_sha256,
        ),
        (
            "component transition native output",
            &transition.native_output_sha256,
        ),
        ("component transition proof", &transition.proof_sha256),
    ] {
        validate_sha256(value, label)?;
    }
    if !transition
        .authorizer_public_key_id
        .strip_prefix("ed25519:sha256:")
        .is_some_and(is_sha256)
    {
        return Err(ArtifactError::new(
            "compiler component transition authorizer public key id is malformed",
        ));
    }
    decode_array::<64>(&transition.signature_hex, "component transition signature")?;
    if transition.proof_sha256 != transition_identity(transition) {
        return Err(ArtifactError::new(
            "compiler component transition proof identity mismatch",
        ));
    }
    Ok(())
}

fn transition_identity(transition: &CompilerComponentTransition) -> String {
    let mut hash = Sha256::new();
    for value in [
        transition.protocol.as_bytes(),
        transition.authority.as_bytes(),
        transition.signature_contract.as_bytes(),
        transition.action.as_bytes(),
        transition.component_id.as_bytes(),
        transition.transition_id.as_bytes(),
        transition.predecessor_authorization_protocol.as_bytes(),
        transition.predecessor_authorization_file.as_bytes(),
        transition.predecessor_authorization_file_sha256.as_bytes(),
        transition.predecessor_authorization_id.as_bytes(),
        transition.predecessor_authorization_proof_sha256.as_bytes(),
        transition.predecessor_state_protocol.as_bytes(),
        transition.predecessor_state_file.as_bytes(),
        transition.predecessor_state_file_sha256.as_bytes(),
        transition.predecessor_state_sha256.as_bytes(),
        transition.challenge_sha256.as_bytes(),
        transition.from_selector.as_bytes(),
        transition.from_stage_role.as_bytes(),
        transition.from_reproducible_build_sha256.as_bytes(),
        transition.current_selector.as_bytes(),
        transition.current_stage_role.as_bytes(),
        transition.current_reproducible_build_sha256.as_bytes(),
        transition.forward_selector.as_bytes(),
        transition.forward_stage_role.as_bytes(),
        transition.forward_reproducible_build_sha256.as_bytes(),
        transition.candidate_compiler_image_sha256.as_bytes(),
        transition.native_output_sha256.as_bytes(),
        transition.authorizer_id.as_bytes(),
        transition.authorizer_environment_id.as_bytes(),
        transition.authorizer_public_key_id.as_bytes(),
        transition.verdict.as_bytes(),
    ] {
        hash_field(&mut hash, value);
    }
    for value in [
        transition.generation,
        transition.predecessor_authorization_file_bytes,
        transition.predecessor_authorization_generation,
        transition.predecessor_state_file_bytes,
        transition.predecessor_state_generation,
        usize::from(transition.reversible),
    ] {
        hash_field(&mut hash, &(value as u64).to_le_bytes());
    }
    encode_hex(&hash.finalize())
}

fn signature_message(proof_sha256: &str) -> Vec<u8> {
    let mut message = Vec::with_capacity(
        COMPILER_COMPONENT_TRANSITION_SIGNATURE_CONTRACT.len() + proof_sha256.len() + 1,
    );
    message.extend_from_slice(COMPILER_COMPONENT_TRANSITION_SIGNATURE_CONTRACT.as_bytes());
    message.push(0);
    message.extend_from_slice(proof_sha256.as_bytes());
    message
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
    if is_sha256(value) {
        Ok(())
    } else {
        Err(ArtifactError::new(format!(
            "compiler {label} must be lowercase SHA-256"
        )))
    }
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn sha256_hex(bytes: &[u8]) -> String {
    encode_hex(&Sha256::digest(bytes))
}

#[cfg(test)]
#[path = "compiler_component_transition_tests.rs"]
mod tests;
