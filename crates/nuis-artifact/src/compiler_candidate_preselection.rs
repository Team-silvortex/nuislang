use std::path::Path;

use ed25519_dalek::{Signature, Signer, SigningKey};
use sha2::{Digest, Sha256};

use crate::{
    compiler_component_attestation_registry::{decode_array, encode_hex, public_key_id},
    compiler_component_replacement_registry::resolve_replacement_authorizer_key,
    parse_compiler_candidate_compile_capability_from_source,
    parse_compiler_candidate_production_from_source,
    parse_compiler_component_transition_from_source, render_compiler_candidate_compile_capability,
    render_compiler_candidate_production, render_compiler_component_transition,
    verify_compiler_component_build, verify_compiler_component_transition, ArtifactError,
    CompilerCandidateCompileCapability, CompilerCandidateProduction, CompilerComponentBuild,
    CompilerComponentTransition, CompilerComponentTransitionVerificationInput,
    COMPILER_CANDIDATE_COMPILE_CAPABILITY_FILE, COMPILER_CANDIDATE_COMPILE_CAPABILITY_PROTOCOL,
    COMPILER_CANDIDATE_PRODUCTION_FILE, COMPILER_CANDIDATE_PRODUCTION_PROTOCOL,
    COMPILER_COMPONENT_STAGE0_ROLE, COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE,
    COMPILER_COMPONENT_TRANSITION_FILE, COMPILER_COMPONENT_TRANSITION_PROTOCOL,
};

#[path = "compiler_candidate_preselection_parse.rs"]
mod parse;

pub use parse::{
    parse_compiler_candidate_preselection, parse_compiler_candidate_preselection_from_source,
};

pub const COMPILER_CANDIDATE_PRESELECTION_PROTOCOL: &str =
    "nuis-compiler-candidate-preselection-v1";
pub const COMPILER_CANDIDATE_PRESELECTION_FILE: &str = "nuis.compiler-candidate-preselection.toml";
pub const COMPILER_CANDIDATE_PRESELECTION_AUTHORITY: &str =
    "independent-ed25519-component-owner-preselection";
pub const COMPILER_CANDIDATE_PRESELECTION_SIGNATURE_CONTRACT: &str =
    "nuis-compiler-candidate-preselection-ed25519-v1";
pub const COMPILER_CANDIDATE_PRESELECTION_ACTION: &str =
    "preselect-stage1-candidate-for-generation-three";
pub const COMPILER_CANDIDATE_PRESELECTION_PROVIDER_CONTRACT: &str =
    "verified-stage0-provider-dependency-v1";
pub const COMPILER_CANDIDATE_PRESELECTION_VERDICT: &str =
    "generation-three-candidate-preselected-no-selection-authority";

const TARGET_GENERATION: usize = 3;
const PREDECESSOR_GENERATION: usize = 2;

#[derive(Debug, Clone, Copy)]
pub struct CompilerCandidatePreselectionInput<'a> {
    pub transition: &'a CompilerComponentTransition,
    pub transition_source: &'a str,
    pub transition_verification: CompilerComponentTransitionVerificationInput<'a>,
    pub stage0: &'a CompilerComponentBuild,
    pub candidate: &'a CompilerComponentBuild,
    pub production: &'a CompilerCandidateProduction,
    pub production_source: &'a str,
    pub capability: &'a CompilerCandidateCompileCapability,
    pub capability_source: &'a str,
    pub challenge_sha256: &'a str,
    pub preselection_id: &'a str,
    pub authorizer_id: &'a str,
    pub environment_id: &'a str,
}

#[derive(Debug, Clone, Copy)]
pub struct CompilerCandidatePreselectionVerificationInput<'a> {
    pub transition: &'a CompilerComponentTransition,
    pub transition_source: &'a str,
    pub transition_verification: CompilerComponentTransitionVerificationInput<'a>,
    pub stage0: &'a CompilerComponentBuild,
    pub candidate: &'a CompilerComponentBuild,
    pub production: &'a CompilerCandidateProduction,
    pub production_source: &'a str,
    pub capability: &'a CompilerCandidateCompileCapability,
    pub capability_source: &'a str,
    pub expected_challenge_sha256: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerCandidatePreselection {
    pub protocol: String,
    pub authority: String,
    pub signature_contract: String,
    pub action: String,
    pub component_id: String,
    pub component_domain: String,
    pub component_unit: String,
    pub preselection_id: String,
    pub target_generation: usize,
    pub predecessor_transition_protocol: String,
    pub predecessor_transition_file: String,
    pub predecessor_transition_file_bytes: usize,
    pub predecessor_transition_file_sha256: String,
    pub predecessor_transition_id: String,
    pub predecessor_transition_generation: usize,
    pub predecessor_transition_proof_sha256: String,
    pub challenge_sha256: String,
    pub current_stage_role: String,
    pub current_record_sha256: String,
    pub current_reproducible_build_sha256: String,
    pub current_compiler_image_sha256: String,
    pub candidate_stage_role: String,
    pub candidate_record_sha256: String,
    pub candidate_reproducible_build_sha256: String,
    pub candidate_producer_id: String,
    pub candidate_compiler_image_sha256: String,
    pub production_protocol: String,
    pub production_file: String,
    pub production_file_bytes: usize,
    pub production_file_sha256: String,
    pub production_proof_sha256: String,
    pub compile_capability_protocol: String,
    pub compile_capability_file: String,
    pub compile_capability_file_bytes: usize,
    pub compile_capability_file_sha256: String,
    pub compile_capability_proof_sha256: String,
    pub compile_driver_contract: String,
    pub compile_provider_contract: String,
    pub compiled_artifact_semantic_sha256: String,
    pub compile_result_record_sha256: String,
    pub compile_result_reproducible_build_sha256: String,
    pub compile_result_native_binary_sha256: String,
    pub provider_dependency_contract: String,
    pub provider_dependency_required: bool,
    pub direct_stage1_compile: bool,
    pub replacement_authorized: bool,
    pub selection_authorized: bool,
    pub preselection_authorized: bool,
    pub authorizer_id: String,
    pub authorizer_environment_id: String,
    pub authorizer_public_key_id: String,
    pub verdict: String,
    pub proof_sha256: String,
    pub signature_hex: String,
}

pub fn build_compiler_candidate_preselection(
    input: CompilerCandidatePreselectionInput<'_>,
    signing_key_hex: &str,
) -> Result<CompilerCandidatePreselection, ArtifactError> {
    validate_bound_sources(verification_input(&input))?;
    validate_sha256(input.challenge_sha256, "candidate preselection challenge")?;
    validate_token(input.preselection_id, "candidate preselection id")?;
    validate_token(input.authorizer_id, "candidate preselection authorizer id")?;
    validate_token(
        input.environment_id,
        "candidate preselection authorizer environment id",
    )?;
    if input.authorizer_id != input.transition.authorizer_id
        || input.environment_id != input.transition.authorizer_environment_id
    {
        return Err(ArtifactError::new(
            "compiler candidate preselection must retain the generation-two component owner",
        ));
    }
    let signing_key = SigningKey::from_bytes(&decode_array::<32>(
        signing_key_hex,
        "candidate preselection authorizer signing key",
    )?);
    let authorizer_public_key_id = public_key_id(&signing_key.verifying_key());
    if authorizer_public_key_id != input.transition.authorizer_public_key_id {
        return Err(ArtifactError::new(
            "compiler candidate preselection must retain the generation-two component-owner key",
        ));
    }

    let mut preselection = CompilerCandidatePreselection {
        protocol: COMPILER_CANDIDATE_PRESELECTION_PROTOCOL.to_owned(),
        authority: COMPILER_CANDIDATE_PRESELECTION_AUTHORITY.to_owned(),
        signature_contract: COMPILER_CANDIDATE_PRESELECTION_SIGNATURE_CONTRACT.to_owned(),
        action: COMPILER_CANDIDATE_PRESELECTION_ACTION.to_owned(),
        component_id: input.stage0.component_id.clone(),
        component_domain: input.stage0.component_domain.clone(),
        component_unit: input.stage0.component_unit.clone(),
        preselection_id: input.preselection_id.to_owned(),
        target_generation: TARGET_GENERATION,
        predecessor_transition_protocol: input.transition.protocol.clone(),
        predecessor_transition_file: COMPILER_COMPONENT_TRANSITION_FILE.to_owned(),
        predecessor_transition_file_bytes: input.transition_source.len(),
        predecessor_transition_file_sha256: sha256_hex(input.transition_source.as_bytes()),
        predecessor_transition_id: input.transition.transition_id.clone(),
        predecessor_transition_generation: input.transition.generation,
        predecessor_transition_proof_sha256: input.transition.proof_sha256.clone(),
        challenge_sha256: input.challenge_sha256.to_owned(),
        current_stage_role: input.stage0.stage_role.clone(),
        current_record_sha256: input.stage0.record_sha256.clone(),
        current_reproducible_build_sha256: input.stage0.reproducible_build_sha256.clone(),
        current_compiler_image_sha256: input.stage0.compiler_image_sha256.clone(),
        candidate_stage_role: input.candidate.stage_role.clone(),
        candidate_record_sha256: input.candidate.record_sha256.clone(),
        candidate_reproducible_build_sha256: input.candidate.reproducible_build_sha256.clone(),
        candidate_producer_id: input.candidate.producer_id.clone(),
        candidate_compiler_image_sha256: input.candidate.compiler_image_sha256.clone(),
        production_protocol: input.production.protocol.clone(),
        production_file: COMPILER_CANDIDATE_PRODUCTION_FILE.to_owned(),
        production_file_bytes: input.production_source.len(),
        production_file_sha256: sha256_hex(input.production_source.as_bytes()),
        production_proof_sha256: input.production.proof_sha256.clone(),
        compile_capability_protocol: input.capability.protocol.clone(),
        compile_capability_file: COMPILER_CANDIDATE_COMPILE_CAPABILITY_FILE.to_owned(),
        compile_capability_file_bytes: input.capability_source.len(),
        compile_capability_file_sha256: sha256_hex(input.capability_source.as_bytes()),
        compile_capability_proof_sha256: input.capability.proof_sha256.clone(),
        compile_driver_contract: input.capability.driver_contract.clone(),
        compile_provider_contract: input.capability.provider_contract.clone(),
        compiled_artifact_semantic_sha256: input
            .capability
            .compiled_artifact_semantic_sha256
            .clone(),
        compile_result_record_sha256: input.capability.result_record_sha256.clone(),
        compile_result_reproducible_build_sha256: input
            .capability
            .result_reproducible_build_sha256
            .clone(),
        compile_result_native_binary_sha256: input.capability.result_native_binary_sha256.clone(),
        provider_dependency_contract: COMPILER_CANDIDATE_PRESELECTION_PROVIDER_CONTRACT.to_owned(),
        provider_dependency_required: true,
        direct_stage1_compile: false,
        replacement_authorized: false,
        selection_authorized: false,
        preselection_authorized: true,
        authorizer_id: input.authorizer_id.to_owned(),
        authorizer_environment_id: input.environment_id.to_owned(),
        authorizer_public_key_id,
        verdict: COMPILER_CANDIDATE_PRESELECTION_VERDICT.to_owned(),
        proof_sha256: String::new(),
        signature_hex: String::new(),
    };
    preselection.proof_sha256 = preselection_identity(&preselection);
    preselection.signature_hex = encode_hex(
        &signing_key
            .sign(&signature_message(&preselection.proof_sha256))
            .to_bytes(),
    );
    validate_preselection(&preselection)?;
    Ok(preselection)
}

pub fn verify_compiler_candidate_preselection(
    preselection: &CompilerCandidatePreselection,
    input: CompilerCandidatePreselectionVerificationInput<'_>,
) -> Result<(), ArtifactError> {
    validate_preselection(preselection)?;
    validate_sha256(
        input.expected_challenge_sha256,
        "expected candidate preselection challenge",
    )?;
    if preselection.challenge_sha256 != input.expected_challenge_sha256 {
        return Err(ArtifactError::new(
            "compiler candidate preselection challenge does not match the verifier request",
        ));
    }
    validate_bound_sources(input)?;
    validate_preselection_lineage(preselection, input)?;

    let registry = input.transition_verification.authorizer_registry;
    let verifying_key = resolve_replacement_authorizer_key(
        registry,
        &preselection.component_id,
        &preselection.authorizer_id,
        &preselection.authorizer_environment_id,
        &preselection.authorizer_public_key_id,
    )?;
    let signature = Signature::from_bytes(&decode_array::<64>(
        &preselection.signature_hex,
        "candidate preselection signature",
    )?);
    verifying_key
        .verify_strict(&signature_message(&preselection.proof_sha256), &signature)
        .map_err(|_| ArtifactError::new("compiler candidate preselection signature mismatch"))
}

pub fn render_compiler_candidate_preselection(
    preselection: &CompilerCandidatePreselection,
) -> String {
    format!(
        "protocol = \"{}\"\nauthority = \"{}\"\nsignature_contract = \"{}\"\naction = \"{}\"\ncomponent_id = \"{}\"\ncomponent_domain = \"{}\"\ncomponent_unit = \"{}\"\npreselection_id = \"{}\"\ntarget_generation = {}\npredecessor_transition_protocol = \"{}\"\npredecessor_transition_file = \"{}\"\npredecessor_transition_file_bytes = {}\npredecessor_transition_file_sha256 = \"{}\"\npredecessor_transition_id = \"{}\"\npredecessor_transition_generation = {}\npredecessor_transition_proof_sha256 = \"{}\"\nchallenge_sha256 = \"{}\"\ncurrent_stage_role = \"{}\"\ncurrent_record_sha256 = \"{}\"\ncurrent_reproducible_build_sha256 = \"{}\"\ncurrent_compiler_image_sha256 = \"{}\"\ncandidate_stage_role = \"{}\"\ncandidate_record_sha256 = \"{}\"\ncandidate_reproducible_build_sha256 = \"{}\"\ncandidate_producer_id = \"{}\"\ncandidate_compiler_image_sha256 = \"{}\"\nproduction_protocol = \"{}\"\nproduction_file = \"{}\"\nproduction_file_bytes = {}\nproduction_file_sha256 = \"{}\"\nproduction_proof_sha256 = \"{}\"\ncompile_capability_protocol = \"{}\"\ncompile_capability_file = \"{}\"\ncompile_capability_file_bytes = {}\ncompile_capability_file_sha256 = \"{}\"\ncompile_capability_proof_sha256 = \"{}\"\ncompile_driver_contract = \"{}\"\ncompile_provider_contract = \"{}\"\ncompiled_artifact_semantic_sha256 = \"{}\"\ncompile_result_record_sha256 = \"{}\"\ncompile_result_reproducible_build_sha256 = \"{}\"\ncompile_result_native_binary_sha256 = \"{}\"\nprovider_dependency_contract = \"{}\"\nprovider_dependency_required = {}\ndirect_stage1_compile = {}\nreplacement_authorized = {}\nselection_authorized = {}\npreselection_authorized = {}\nauthorizer_id = \"{}\"\nauthorizer_environment_id = \"{}\"\nauthorizer_public_key_id = \"{}\"\nverdict = \"{}\"\nproof_sha256 = \"{}\"\nsignature_hex = \"{}\"\n",
        preselection.protocol,
        preselection.authority,
        preselection.signature_contract,
        preselection.action,
        preselection.component_id,
        preselection.component_domain,
        preselection.component_unit,
        preselection.preselection_id,
        preselection.target_generation,
        preselection.predecessor_transition_protocol,
        preselection.predecessor_transition_file,
        preselection.predecessor_transition_file_bytes,
        preselection.predecessor_transition_file_sha256,
        preselection.predecessor_transition_id,
        preselection.predecessor_transition_generation,
        preselection.predecessor_transition_proof_sha256,
        preselection.challenge_sha256,
        preselection.current_stage_role,
        preselection.current_record_sha256,
        preselection.current_reproducible_build_sha256,
        preselection.current_compiler_image_sha256,
        preselection.candidate_stage_role,
        preselection.candidate_record_sha256,
        preselection.candidate_reproducible_build_sha256,
        preselection.candidate_producer_id,
        preselection.candidate_compiler_image_sha256,
        preselection.production_protocol,
        preselection.production_file,
        preselection.production_file_bytes,
        preselection.production_file_sha256,
        preselection.production_proof_sha256,
        preselection.compile_capability_protocol,
        preselection.compile_capability_file,
        preselection.compile_capability_file_bytes,
        preselection.compile_capability_file_sha256,
        preselection.compile_capability_proof_sha256,
        preselection.compile_driver_contract,
        preselection.compile_provider_contract,
        preselection.compiled_artifact_semantic_sha256,
        preselection.compile_result_record_sha256,
        preselection.compile_result_reproducible_build_sha256,
        preselection.compile_result_native_binary_sha256,
        preselection.provider_dependency_contract,
        preselection.provider_dependency_required,
        preselection.direct_stage1_compile,
        preselection.replacement_authorized,
        preselection.selection_authorized,
        preselection.preselection_authorized,
        preselection.authorizer_id,
        preselection.authorizer_environment_id,
        preselection.authorizer_public_key_id,
        preselection.verdict,
        preselection.proof_sha256,
        preselection.signature_hex,
    )
}

fn verification_input<'a>(
    input: &'a CompilerCandidatePreselectionInput<'a>,
) -> CompilerCandidatePreselectionVerificationInput<'a> {
    CompilerCandidatePreselectionVerificationInput {
        transition: input.transition,
        transition_source: input.transition_source,
        transition_verification: input.transition_verification,
        stage0: input.stage0,
        candidate: input.candidate,
        production: input.production,
        production_source: input.production_source,
        capability: input.capability,
        capability_source: input.capability_source,
        expected_challenge_sha256: input.challenge_sha256,
    }
}

fn validate_bound_sources(
    input: CompilerCandidatePreselectionVerificationInput<'_>,
) -> Result<(), ArtifactError> {
    verify_compiler_component_transition(input.transition, input.transition_verification)?;
    let transition = parse_compiler_component_transition_from_source(
        input.transition_source,
        Path::new(COMPILER_COMPONENT_TRANSITION_FILE),
    )?;
    let production = parse_compiler_candidate_production_from_source(
        input.production_source,
        Path::new(COMPILER_CANDIDATE_PRODUCTION_FILE),
    )?;
    let capability = parse_compiler_candidate_compile_capability_from_source(
        input.capability_source,
        Path::new(COMPILER_CANDIDATE_COMPILE_CAPABILITY_FILE),
    )?;
    if &transition != input.transition
        || render_compiler_component_transition(input.transition) != input.transition_source
        || &production != input.production
        || render_compiler_candidate_production(input.production) != input.production_source
        || &capability != input.capability
        || render_compiler_candidate_compile_capability(input.capability) != input.capability_source
    {
        return Err(ArtifactError::new(
            "compiler candidate preselection sources are not canonical",
        ));
    }
    verify_compiler_component_build(input.stage0)?;
    verify_compiler_component_build(input.candidate)?;
    validate_source_lineage(input)
}

fn validate_source_lineage(
    input: CompilerCandidatePreselectionVerificationInput<'_>,
) -> Result<(), ArtifactError> {
    if input.stage0.stage_role != COMPILER_COMPONENT_STAGE0_ROLE
        || input.candidate.stage_role != COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE
        || input.stage0.component_id != input.candidate.component_id
        || input.stage0.component_domain != input.candidate.component_domain
        || input.stage0.component_unit != input.candidate.component_unit
        || input.transition.protocol != COMPILER_COMPONENT_TRANSITION_PROTOCOL
        || input.transition.generation != PREDECESSOR_GENERATION
        || input.transition.component_id != input.stage0.component_id
        || input.transition.current_reproducible_build_sha256
            != input.stage0.reproducible_build_sha256
        || input.transition.forward_reproducible_build_sha256
            != input.candidate.reproducible_build_sha256
        || input.transition.candidate_compiler_image_sha256 != input.candidate.compiler_image_sha256
        || input.production.protocol != COMPILER_CANDIDATE_PRODUCTION_PROTOCOL
        || input.production.stage0_component_sha256 != input.stage0.record_sha256
        || input.production.candidate_component_sha256 != input.candidate.record_sha256
        || input.production.candidate_producer_id != input.candidate.producer_id
        || input.production.candidate_compiler_image_sha256 != input.candidate.compiler_image_sha256
        || input.production.replacement_authorized
        || input.capability.protocol != COMPILER_CANDIDATE_COMPILE_CAPABILITY_PROTOCOL
        || input.capability.stage0_record_sha256 != input.stage0.record_sha256
        || input.capability.stage0_reproducible_build_sha256
            != input.stage0.reproducible_build_sha256
        || input.capability.provider_image_sha256 != input.stage0.compiler_image_sha256
        || input.capability.candidate_record_sha256 != input.candidate.record_sha256
        || input.capability.candidate_reproducible_build_sha256
            != input.candidate.reproducible_build_sha256
        || input.capability.candidate_compiler_image_sha256 != input.candidate.compiler_image_sha256
        || input.capability.production_proof_sha256 != input.production.proof_sha256
        || input.capability.result_reproducible_build_sha256
            != input.stage0.reproducible_build_sha256
        || input.capability.replacement_authorized
        || input.capability.selection_authorized
    {
        return Err(ArtifactError::new(
            "compiler candidate preselection source lineage is inconsistent",
        ));
    }
    Ok(())
}

fn validate_preselection_lineage(
    preselection: &CompilerCandidatePreselection,
    input: CompilerCandidatePreselectionVerificationInput<'_>,
) -> Result<(), ArtifactError> {
    if preselection.component_id != input.stage0.component_id
        || preselection.component_domain != input.stage0.component_domain
        || preselection.component_unit != input.stage0.component_unit
        || preselection.predecessor_transition_protocol != input.transition.protocol
        || preselection.predecessor_transition_file_bytes != input.transition_source.len()
        || preselection.predecessor_transition_file_sha256
            != sha256_hex(input.transition_source.as_bytes())
        || preselection.predecessor_transition_id != input.transition.transition_id
        || preselection.predecessor_transition_generation != input.transition.generation
        || preselection.predecessor_transition_proof_sha256 != input.transition.proof_sha256
        || preselection.current_record_sha256 != input.stage0.record_sha256
        || preselection.current_reproducible_build_sha256 != input.stage0.reproducible_build_sha256
        || preselection.current_compiler_image_sha256 != input.stage0.compiler_image_sha256
        || preselection.candidate_record_sha256 != input.candidate.record_sha256
        || preselection.candidate_reproducible_build_sha256
            != input.candidate.reproducible_build_sha256
        || preselection.candidate_producer_id != input.candidate.producer_id
        || preselection.candidate_compiler_image_sha256 != input.candidate.compiler_image_sha256
        || preselection.production_file_bytes != input.production_source.len()
        || preselection.production_file_sha256 != sha256_hex(input.production_source.as_bytes())
        || preselection.production_proof_sha256 != input.production.proof_sha256
        || preselection.compile_capability_file_bytes != input.capability_source.len()
        || preselection.compile_capability_file_sha256
            != sha256_hex(input.capability_source.as_bytes())
        || preselection.compile_capability_proof_sha256 != input.capability.proof_sha256
        || preselection.compile_driver_contract != input.capability.driver_contract
        || preselection.compile_provider_contract != input.capability.provider_contract
        || preselection.compiled_artifact_semantic_sha256
            != input.capability.compiled_artifact_semantic_sha256
        || preselection.compile_result_record_sha256 != input.capability.result_record_sha256
        || preselection.compile_result_reproducible_build_sha256
            != input.capability.result_reproducible_build_sha256
        || preselection.compile_result_native_binary_sha256
            != input.capability.result_native_binary_sha256
        || preselection.authorizer_id != input.transition.authorizer_id
        || preselection.authorizer_environment_id != input.transition.authorizer_environment_id
        || preselection.authorizer_public_key_id != input.transition.authorizer_public_key_id
    {
        return Err(ArtifactError::new(
            "compiler candidate preselection bound lineage mismatch",
        ));
    }
    Ok(())
}

pub(super) fn validate_preselection(
    preselection: &CompilerCandidatePreselection,
) -> Result<(), ArtifactError> {
    if preselection.protocol != COMPILER_CANDIDATE_PRESELECTION_PROTOCOL
        || preselection.authority != COMPILER_CANDIDATE_PRESELECTION_AUTHORITY
        || preselection.signature_contract != COMPILER_CANDIDATE_PRESELECTION_SIGNATURE_CONTRACT
        || preselection.action != COMPILER_CANDIDATE_PRESELECTION_ACTION
        || preselection.target_generation != TARGET_GENERATION
        || preselection.predecessor_transition_protocol != COMPILER_COMPONENT_TRANSITION_PROTOCOL
        || preselection.predecessor_transition_file != COMPILER_COMPONENT_TRANSITION_FILE
        || preselection.predecessor_transition_generation != PREDECESSOR_GENERATION
        || preselection.current_stage_role != COMPILER_COMPONENT_STAGE0_ROLE
        || preselection.candidate_stage_role != COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE
        || preselection.production_protocol != COMPILER_CANDIDATE_PRODUCTION_PROTOCOL
        || preselection.production_file != COMPILER_CANDIDATE_PRODUCTION_FILE
        || preselection.compile_capability_protocol
            != COMPILER_CANDIDATE_COMPILE_CAPABILITY_PROTOCOL
        || preselection.compile_capability_file != COMPILER_CANDIDATE_COMPILE_CAPABILITY_FILE
        || preselection.provider_dependency_contract
            != COMPILER_CANDIDATE_PRESELECTION_PROVIDER_CONTRACT
        || !preselection.provider_dependency_required
        || preselection.direct_stage1_compile
        || preselection.replacement_authorized
        || preselection.selection_authorized
        || !preselection.preselection_authorized
        || preselection.verdict != COMPILER_CANDIDATE_PRESELECTION_VERDICT
        || preselection.predecessor_transition_file_bytes == 0
        || preselection.production_file_bytes == 0
        || preselection.compile_capability_file_bytes == 0
        || preselection.current_record_sha256 == preselection.candidate_record_sha256
        || preselection.current_reproducible_build_sha256
            == preselection.candidate_reproducible_build_sha256
        || preselection.compile_result_reproducible_build_sha256
            != preselection.current_reproducible_build_sha256
    {
        return Err(ArtifactError::new(
            "compiler candidate preselection contract mismatch",
        ));
    }
    for (value, label) in [
        (
            &preselection.component_id,
            "candidate preselection component id",
        ),
        (
            &preselection.component_domain,
            "candidate preselection component domain",
        ),
        (
            &preselection.component_unit,
            "candidate preselection component unit",
        ),
        (&preselection.preselection_id, "candidate preselection id"),
        (
            &preselection.predecessor_transition_id,
            "predecessor transition id",
        ),
        (&preselection.candidate_producer_id, "candidate producer id"),
        (
            &preselection.authorizer_id,
            "candidate preselection authorizer id",
        ),
        (
            &preselection.authorizer_environment_id,
            "candidate preselection authorizer environment id",
        ),
    ] {
        validate_token(value, label)?;
    }
    for (value, label) in preselection_hashes(preselection) {
        validate_sha256(value, label)?;
    }
    if !preselection
        .authorizer_public_key_id
        .strip_prefix("ed25519:sha256:")
        .is_some_and(is_sha256)
    {
        return Err(ArtifactError::new(
            "compiler candidate preselection authorizer public key id is malformed",
        ));
    }
    decode_array::<64>(
        &preselection.signature_hex,
        "candidate preselection signature",
    )?;
    if preselection.proof_sha256 != preselection_identity(preselection) {
        return Err(ArtifactError::new(
            "compiler candidate preselection proof identity mismatch",
        ));
    }
    Ok(())
}

fn preselection_hashes(preselection: &CompilerCandidatePreselection) -> [(&str, &str); 18] {
    [
        (
            &preselection.predecessor_transition_file_sha256,
            "predecessor transition file",
        ),
        (
            &preselection.predecessor_transition_proof_sha256,
            "predecessor transition proof",
        ),
        (
            &preselection.challenge_sha256,
            "candidate preselection challenge",
        ),
        (
            &preselection.current_record_sha256,
            "current component record",
        ),
        (
            &preselection.current_reproducible_build_sha256,
            "current reproducible build",
        ),
        (
            &preselection.current_compiler_image_sha256,
            "current compiler image",
        ),
        (
            &preselection.candidate_record_sha256,
            "candidate component record",
        ),
        (
            &preselection.candidate_reproducible_build_sha256,
            "candidate reproducible build",
        ),
        (
            &preselection.candidate_compiler_image_sha256,
            "candidate compiler image",
        ),
        (
            &preselection.production_file_sha256,
            "candidate production file",
        ),
        (
            &preselection.production_proof_sha256,
            "candidate production proof",
        ),
        (
            &preselection.compile_capability_file_sha256,
            "candidate compile capability file",
        ),
        (
            &preselection.compile_capability_proof_sha256,
            "candidate compile capability proof",
        ),
        (
            &preselection.compiled_artifact_semantic_sha256,
            "compiled artifact semantics",
        ),
        (
            &preselection.compile_result_record_sha256,
            "compile result record",
        ),
        (
            &preselection.compile_result_reproducible_build_sha256,
            "compile result reproducible build",
        ),
        (
            &preselection.compile_result_native_binary_sha256,
            "compile result native binary",
        ),
        (&preselection.proof_sha256, "candidate preselection proof"),
    ]
}

fn preselection_identity(preselection: &CompilerCandidatePreselection) -> String {
    let mut hash = Sha256::new();
    for value in [
        preselection.protocol.as_bytes(),
        preselection.authority.as_bytes(),
        preselection.signature_contract.as_bytes(),
        preselection.action.as_bytes(),
        preselection.component_id.as_bytes(),
        preselection.component_domain.as_bytes(),
        preselection.component_unit.as_bytes(),
        preselection.preselection_id.as_bytes(),
        preselection.predecessor_transition_protocol.as_bytes(),
        preselection.predecessor_transition_file.as_bytes(),
        preselection.predecessor_transition_file_sha256.as_bytes(),
        preselection.predecessor_transition_id.as_bytes(),
        preselection.predecessor_transition_proof_sha256.as_bytes(),
        preselection.challenge_sha256.as_bytes(),
        preselection.current_stage_role.as_bytes(),
        preselection.current_record_sha256.as_bytes(),
        preselection.current_reproducible_build_sha256.as_bytes(),
        preselection.current_compiler_image_sha256.as_bytes(),
        preselection.candidate_stage_role.as_bytes(),
        preselection.candidate_record_sha256.as_bytes(),
        preselection.candidate_reproducible_build_sha256.as_bytes(),
        preselection.candidate_producer_id.as_bytes(),
        preselection.candidate_compiler_image_sha256.as_bytes(),
        preselection.production_protocol.as_bytes(),
        preselection.production_file.as_bytes(),
        preselection.production_file_sha256.as_bytes(),
        preselection.production_proof_sha256.as_bytes(),
        preselection.compile_capability_protocol.as_bytes(),
        preselection.compile_capability_file.as_bytes(),
        preselection.compile_capability_file_sha256.as_bytes(),
        preselection.compile_capability_proof_sha256.as_bytes(),
        preselection.compile_driver_contract.as_bytes(),
        preselection.compile_provider_contract.as_bytes(),
        preselection.compiled_artifact_semantic_sha256.as_bytes(),
        preselection.compile_result_record_sha256.as_bytes(),
        preselection
            .compile_result_reproducible_build_sha256
            .as_bytes(),
        preselection.compile_result_native_binary_sha256.as_bytes(),
        preselection.provider_dependency_contract.as_bytes(),
        preselection.authorizer_id.as_bytes(),
        preselection.authorizer_environment_id.as_bytes(),
        preselection.authorizer_public_key_id.as_bytes(),
        preselection.verdict.as_bytes(),
    ] {
        hash_field(&mut hash, value);
    }
    for value in [
        preselection.target_generation,
        preselection.predecessor_transition_file_bytes,
        preselection.predecessor_transition_generation,
        preselection.production_file_bytes,
        preselection.compile_capability_file_bytes,
    ] {
        hash_field(&mut hash, &(value as u64).to_le_bytes());
    }
    for value in [
        preselection.provider_dependency_required,
        preselection.direct_stage1_compile,
        preselection.replacement_authorized,
        preselection.selection_authorized,
        preselection.preselection_authorized,
    ] {
        hash_field(&mut hash, &[u8::from(value)]);
    }
    encode_hex(&hash.finalize())
}

fn signature_message(proof_sha256: &str) -> Vec<u8> {
    let mut message = Vec::with_capacity(
        COMPILER_CANDIDATE_PRESELECTION_SIGNATURE_CONTRACT.len() + proof_sha256.len() + 1,
    );
    message.extend_from_slice(COMPILER_CANDIDATE_PRESELECTION_SIGNATURE_CONTRACT.as_bytes());
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
            .all(|byte| byte.is_ascii_alphanumeric() || b"-_.:@".contains(&byte))
    {
        return Err(ArtifactError::new(format!(
            "{label} must be a non-empty portable token"
        )));
    }
    Ok(())
}

fn validate_sha256(value: &str, label: &str) -> Result<(), ArtifactError> {
    if is_sha256(value) {
        Ok(())
    } else {
        Err(ArtifactError::new(format!(
            "{label} must be a lowercase SHA-256 identity"
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
#[path = "compiler_candidate_preselection_tests.rs"]
mod tests;
