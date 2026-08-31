use std::path::Path;

use ed25519_dalek::{Signature, Signer, SigningKey};
use sha2::{Digest, Sha256};

use crate::{
    compiler_component_attestation_registry::{decode_array, encode_hex, public_key_id},
    compiler_component_replacement_registry::resolve_replacement_authorizer_key,
    parse_compiler_candidate_direct_compile_capability_from_source,
    parse_compiler_candidate_frontend_result_bytes,
    parse_compiler_candidate_preselection_from_source,
    render_compiler_candidate_direct_compile_capability, render_compiler_candidate_frontend_result,
    render_compiler_candidate_preselection,
    toml::escape_toml_string,
    verify_compiler_candidate_direct_compile_capability, verify_compiler_candidate_preselection,
    ArtifactError, CompilerCandidateDirectCompileCapability,
    CompilerCandidateDirectCompileCapabilityInput, CompilerCandidatePreselection,
    CompilerCandidatePreselectionVerificationInput, CompilerComponentReplacementAuthorizerRegistry,
    COMPILER_CANDIDATE_DIRECT_COMPILE_CAPABILITY_FILE,
    COMPILER_CANDIDATE_DIRECT_COMPILE_CAPABILITY_PROTOCOL,
    COMPILER_CANDIDATE_DIRECT_COMPILE_DRIVER_CONTRACT,
    COMPILER_CANDIDATE_DIRECT_COMPILE_PROVIDER_CONTRACT, COMPILER_CANDIDATE_FRONTEND_RESULT_FILE,
    COMPILER_CANDIDATE_FRONTEND_RESULT_PROTOCOL, COMPILER_CANDIDATE_PRESELECTION_FILE,
    COMPILER_CANDIDATE_PRESELECTION_PROTOCOL, COMPILER_CANDIDATE_PRODUCTION_PROTOCOL,
    COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE,
};

#[path = "compiler_candidate_successor_parse.rs"]
mod parse;

pub use parse::{
    parse_compiler_candidate_successor, parse_compiler_candidate_successor_from_source,
};

pub const COMPILER_CANDIDATE_SUCCESSOR_PROTOCOL: &str = "nuis-compiler-candidate-successor-v1";
pub const COMPILER_CANDIDATE_SUCCESSOR_FILE: &str = "nuis.compiler-candidate-successor.toml";
pub const COMPILER_CANDIDATE_SUCCESSOR_AUTHORITY: &str =
    "independent-ed25519-component-owner-generation-three-successor";
pub const COMPILER_CANDIDATE_SUCCESSOR_SIGNATURE_CONTRACT: &str =
    "nuis-compiler-candidate-successor-ed25519-v1";
pub const COMPILER_CANDIDATE_SUCCESSOR_ACTION: &str =
    "strengthen-generation-three-with-direct-front-end-capability";
pub const COMPILER_CANDIDATE_SUCCESSOR_RELATION_CONTRACT: &str =
    "same-generation-capability-strengthening-v1";
pub const COMPILER_CANDIDATE_SUCCESSOR_VERDICT: &str =
    "generation-three-direct-front-end-successor-no-native-or-selection-authority";

const TARGET_GENERATION: usize = 3;

#[derive(Debug, Clone, Copy)]
pub struct CompilerCandidateSuccessorInput<'a> {
    pub preselection: &'a CompilerCandidatePreselection,
    pub preselection_source: &'a str,
    pub preselection_verification: CompilerCandidatePreselectionVerificationInput<'a>,
    pub direct_capability: &'a CompilerCandidateDirectCompileCapability,
    pub direct_capability_source: &'a str,
    pub direct_compile_verification: CompilerCandidateDirectCompileCapabilityInput<'a>,
    pub frontend_result_source: &'a [u8],
    pub challenge_sha256: &'a str,
    pub successor_id: &'a str,
    pub authorizer_id: &'a str,
    pub environment_id: &'a str,
}

#[derive(Debug, Clone, Copy)]
pub struct CompilerCandidateSuccessorVerificationInput<'a> {
    pub preselection: &'a CompilerCandidatePreselection,
    pub preselection_source: &'a str,
    pub preselection_verification: CompilerCandidatePreselectionVerificationInput<'a>,
    pub direct_capability: &'a CompilerCandidateDirectCompileCapability,
    pub direct_capability_source: &'a str,
    pub direct_compile_verification: CompilerCandidateDirectCompileCapabilityInput<'a>,
    pub frontend_result_source: &'a [u8],
    pub expected_challenge_sha256: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerCandidateSuccessor {
    pub protocol: String,
    pub authority: String,
    pub signature_contract: String,
    pub action: String,
    pub relation_contract: String,
    pub component_id: String,
    pub component_domain: String,
    pub component_unit: String,
    pub successor_id: String,
    pub target_generation: usize,
    pub predecessor_preselection_protocol: String,
    pub predecessor_preselection_file: String,
    pub predecessor_preselection_file_bytes: usize,
    pub predecessor_preselection_file_sha256: String,
    pub predecessor_preselection_id: String,
    pub predecessor_preselection_proof_sha256: String,
    pub challenge_sha256: String,
    pub candidate_stage_role: String,
    pub candidate_record_sha256: String,
    pub candidate_reproducible_build_sha256: String,
    pub candidate_producer_id: String,
    pub candidate_compiler_image_sha256: String,
    pub production_protocol: String,
    pub production_proof_sha256: String,
    pub direct_compile_capability_protocol: String,
    pub direct_compile_capability_file: String,
    pub direct_compile_capability_file_bytes: usize,
    pub direct_compile_capability_file_sha256: String,
    pub direct_compile_capability_proof_sha256: String,
    pub direct_compile_driver_contract: String,
    pub direct_compile_provider_contract: String,
    pub direct_compile_input_identity_sha256: String,
    pub frontend_result_protocol: String,
    pub frontend_result_file: String,
    pub frontend_result_bytes: usize,
    pub frontend_result_sha256: String,
    pub frontend_result_bundle_fold: usize,
    pub provider_dependency_required: bool,
    pub direct_stage1_compile: bool,
    pub fresh_source_compile: bool,
    pub native_materialization: bool,
    pub replacement_authorized: bool,
    pub selection_authorized: bool,
    pub preselection_authorized: bool,
    pub successor_authorized: bool,
    pub authorizer_id: String,
    pub authorizer_environment_id: String,
    pub authorizer_public_key_id: String,
    pub verdict: String,
    pub proof_sha256: String,
    pub signature_hex: String,
}

pub fn build_compiler_candidate_successor(
    input: CompilerCandidateSuccessorInput<'_>,
    signing_key_hex: &str,
) -> Result<CompilerCandidateSuccessor, ArtifactError> {
    let verification = verification_input(&input);
    validate_bound_sources(verification)?;
    validate_sha256(input.challenge_sha256, "candidate successor challenge")?;
    validate_token(input.successor_id, "candidate successor id")?;
    validate_token(input.authorizer_id, "candidate successor authorizer id")?;
    validate_token(
        input.environment_id,
        "candidate successor authorizer environment id",
    )?;
    if input.authorizer_id != input.preselection.authorizer_id
        || input.environment_id != input.preselection.authorizer_environment_id
    {
        return Err(ArtifactError::new(
            "compiler candidate successor must retain the generation-three component owner",
        ));
    }
    let signing_key = SigningKey::from_bytes(&decode_array::<32>(
        signing_key_hex,
        "candidate successor authorizer signing key",
    )?);
    let authorizer_public_key_id = public_key_id(&signing_key.verifying_key());
    if authorizer_public_key_id != input.preselection.authorizer_public_key_id {
        return Err(ArtifactError::new(
            "compiler candidate successor must retain the preselection component-owner key",
        ));
    }
    let result = parse_compiler_candidate_frontend_result_bytes(
        input.frontend_result_source,
        Path::new(COMPILER_CANDIDATE_FRONTEND_RESULT_FILE),
    )?;
    let capability = input.direct_capability;
    let preselection = input.preselection;
    let mut successor = CompilerCandidateSuccessor {
        protocol: COMPILER_CANDIDATE_SUCCESSOR_PROTOCOL.to_owned(),
        authority: COMPILER_CANDIDATE_SUCCESSOR_AUTHORITY.to_owned(),
        signature_contract: COMPILER_CANDIDATE_SUCCESSOR_SIGNATURE_CONTRACT.to_owned(),
        action: COMPILER_CANDIDATE_SUCCESSOR_ACTION.to_owned(),
        relation_contract: COMPILER_CANDIDATE_SUCCESSOR_RELATION_CONTRACT.to_owned(),
        component_id: capability.component_id.clone(),
        component_domain: capability.component_domain.clone(),
        component_unit: capability.component_unit.clone(),
        successor_id: input.successor_id.to_owned(),
        target_generation: TARGET_GENERATION,
        predecessor_preselection_protocol: preselection.protocol.clone(),
        predecessor_preselection_file: COMPILER_CANDIDATE_PRESELECTION_FILE.to_owned(),
        predecessor_preselection_file_bytes: input.preselection_source.len(),
        predecessor_preselection_file_sha256: sha256_hex(input.preselection_source.as_bytes()),
        predecessor_preselection_id: preselection.preselection_id.clone(),
        predecessor_preselection_proof_sha256: preselection.proof_sha256.clone(),
        challenge_sha256: input.challenge_sha256.to_owned(),
        candidate_stage_role: COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE.to_owned(),
        candidate_record_sha256: capability.candidate_record_sha256.clone(),
        candidate_reproducible_build_sha256: capability.candidate_reproducible_build_sha256.clone(),
        candidate_producer_id: capability.candidate_producer_id.clone(),
        candidate_compiler_image_sha256: capability.candidate_compiler_image_sha256.clone(),
        production_protocol: capability.production_protocol.clone(),
        production_proof_sha256: capability.production_proof_sha256.clone(),
        direct_compile_capability_protocol: capability.protocol.clone(),
        direct_compile_capability_file: COMPILER_CANDIDATE_DIRECT_COMPILE_CAPABILITY_FILE
            .to_owned(),
        direct_compile_capability_file_bytes: input.direct_capability_source.len(),
        direct_compile_capability_file_sha256: sha256_hex(
            input.direct_capability_source.as_bytes(),
        ),
        direct_compile_capability_proof_sha256: capability.proof_sha256.clone(),
        direct_compile_driver_contract: capability.driver_contract.clone(),
        direct_compile_provider_contract: capability.provider_contract.clone(),
        direct_compile_input_identity_sha256: capability.input_identity_sha256.clone(),
        frontend_result_protocol: result.protocol,
        frontend_result_file: COMPILER_CANDIDATE_FRONTEND_RESULT_FILE.to_owned(),
        frontend_result_bytes: input.frontend_result_source.len(),
        frontend_result_sha256: sha256_hex(input.frontend_result_source),
        frontend_result_bundle_fold: result.bundle_fold,
        provider_dependency_required: false,
        direct_stage1_compile: true,
        fresh_source_compile: false,
        native_materialization: false,
        replacement_authorized: false,
        selection_authorized: false,
        preselection_authorized: true,
        successor_authorized: true,
        authorizer_id: input.authorizer_id.to_owned(),
        authorizer_environment_id: input.environment_id.to_owned(),
        authorizer_public_key_id,
        verdict: COMPILER_CANDIDATE_SUCCESSOR_VERDICT.to_owned(),
        proof_sha256: String::new(),
        signature_hex: String::new(),
    };
    successor.proof_sha256 = successor_identity(&successor);
    successor.signature_hex = encode_hex(
        &signing_key
            .sign(&signature_message(&successor.proof_sha256))
            .to_bytes(),
    );
    validate_successor(&successor)?;
    Ok(successor)
}

pub fn verify_compiler_candidate_successor(
    successor: &CompilerCandidateSuccessor,
    input: CompilerCandidateSuccessorVerificationInput<'_>,
) -> Result<(), ArtifactError> {
    validate_successor(successor)?;
    validate_sha256(
        input.expected_challenge_sha256,
        "expected candidate successor challenge",
    )?;
    if successor.challenge_sha256 != input.expected_challenge_sha256 {
        return Err(ArtifactError::new(
            "compiler candidate successor challenge does not match the verifier request",
        ));
    }
    validate_bound_sources(input)?;
    validate_successor_lineage(successor, input)?;
    verify_successor_signature(
        successor,
        input
            .preselection_verification
            .transition_verification
            .authorizer_registry,
    )
}

pub fn render_compiler_candidate_successor(successor: &CompilerCandidateSuccessor) -> String {
    format!(
        "protocol = \"{}\"\nauthority = \"{}\"\nsignature_contract = \"{}\"\naction = \"{}\"\nrelation_contract = \"{}\"\ncomponent_id = \"{}\"\ncomponent_domain = \"{}\"\ncomponent_unit = \"{}\"\nsuccessor_id = \"{}\"\ntarget_generation = {}\npredecessor_preselection_protocol = \"{}\"\npredecessor_preselection_file = \"{}\"\npredecessor_preselection_file_bytes = {}\npredecessor_preselection_file_sha256 = \"{}\"\npredecessor_preselection_id = \"{}\"\npredecessor_preselection_proof_sha256 = \"{}\"\nchallenge_sha256 = \"{}\"\ncandidate_stage_role = \"{}\"\ncandidate_record_sha256 = \"{}\"\ncandidate_reproducible_build_sha256 = \"{}\"\ncandidate_producer_id = \"{}\"\ncandidate_compiler_image_sha256 = \"{}\"\nproduction_protocol = \"{}\"\nproduction_proof_sha256 = \"{}\"\ndirect_compile_capability_protocol = \"{}\"\ndirect_compile_capability_file = \"{}\"\ndirect_compile_capability_file_bytes = {}\ndirect_compile_capability_file_sha256 = \"{}\"\ndirect_compile_capability_proof_sha256 = \"{}\"\ndirect_compile_driver_contract = \"{}\"\ndirect_compile_provider_contract = \"{}\"\ndirect_compile_input_identity_sha256 = \"{}\"\nfrontend_result_protocol = \"{}\"\nfrontend_result_file = \"{}\"\nfrontend_result_bytes = {}\nfrontend_result_sha256 = \"{}\"\nfrontend_result_bundle_fold = {}\nprovider_dependency_required = {}\ndirect_stage1_compile = {}\nfresh_source_compile = {}\nnative_materialization = {}\nreplacement_authorized = {}\nselection_authorized = {}\npreselection_authorized = {}\nsuccessor_authorized = {}\nauthorizer_id = \"{}\"\nauthorizer_environment_id = \"{}\"\nauthorizer_public_key_id = \"{}\"\nverdict = \"{}\"\nproof_sha256 = \"{}\"\nsignature_hex = \"{}\"\n",
        successor.protocol,
        successor.authority,
        successor.signature_contract,
        successor.action,
        successor.relation_contract,
        escape_toml_string(&successor.component_id),
        escape_toml_string(&successor.component_domain),
        escape_toml_string(&successor.component_unit),
        escape_toml_string(&successor.successor_id),
        successor.target_generation,
        successor.predecessor_preselection_protocol,
        successor.predecessor_preselection_file,
        successor.predecessor_preselection_file_bytes,
        successor.predecessor_preselection_file_sha256,
        escape_toml_string(&successor.predecessor_preselection_id),
        successor.predecessor_preselection_proof_sha256,
        successor.challenge_sha256,
        successor.candidate_stage_role,
        successor.candidate_record_sha256,
        successor.candidate_reproducible_build_sha256,
        escape_toml_string(&successor.candidate_producer_id),
        successor.candidate_compiler_image_sha256,
        successor.production_protocol,
        successor.production_proof_sha256,
        successor.direct_compile_capability_protocol,
        successor.direct_compile_capability_file,
        successor.direct_compile_capability_file_bytes,
        successor.direct_compile_capability_file_sha256,
        successor.direct_compile_capability_proof_sha256,
        successor.direct_compile_driver_contract,
        successor.direct_compile_provider_contract,
        successor.direct_compile_input_identity_sha256,
        successor.frontend_result_protocol,
        successor.frontend_result_file,
        successor.frontend_result_bytes,
        successor.frontend_result_sha256,
        successor.frontend_result_bundle_fold,
        successor.provider_dependency_required,
        successor.direct_stage1_compile,
        successor.fresh_source_compile,
        successor.native_materialization,
        successor.replacement_authorized,
        successor.selection_authorized,
        successor.preselection_authorized,
        successor.successor_authorized,
        escape_toml_string(&successor.authorizer_id),
        escape_toml_string(&successor.authorizer_environment_id),
        successor.authorizer_public_key_id,
        successor.verdict,
        successor.proof_sha256,
        successor.signature_hex,
    )
}

fn verification_input<'a>(
    input: &'a CompilerCandidateSuccessorInput<'a>,
) -> CompilerCandidateSuccessorVerificationInput<'a> {
    CompilerCandidateSuccessorVerificationInput {
        preselection: input.preselection,
        preselection_source: input.preselection_source,
        preselection_verification: input.preselection_verification,
        direct_capability: input.direct_capability,
        direct_capability_source: input.direct_capability_source,
        direct_compile_verification: input.direct_compile_verification,
        frontend_result_source: input.frontend_result_source,
        expected_challenge_sha256: input.challenge_sha256,
    }
}

fn validate_bound_sources(
    input: CompilerCandidateSuccessorVerificationInput<'_>,
) -> Result<(), ArtifactError> {
    verify_compiler_candidate_preselection(input.preselection, input.preselection_verification)?;
    verify_compiler_candidate_direct_compile_capability(
        input.direct_capability,
        &input.direct_compile_verification,
    )?;
    let preselection = parse_compiler_candidate_preselection_from_source(
        input.preselection_source,
        Path::new(COMPILER_CANDIDATE_PRESELECTION_FILE),
    )?;
    let capability = parse_compiler_candidate_direct_compile_capability_from_source(
        input.direct_capability_source,
        Path::new(COMPILER_CANDIDATE_DIRECT_COMPILE_CAPABILITY_FILE),
    )?;
    let result = parse_compiler_candidate_frontend_result_bytes(
        input.frontend_result_source,
        Path::new(COMPILER_CANDIDATE_FRONTEND_RESULT_FILE),
    )?;
    if &preselection != input.preselection
        || render_compiler_candidate_preselection(input.preselection) != input.preselection_source
        || &capability != input.direct_capability
        || render_compiler_candidate_direct_compile_capability(input.direct_capability)
            != input.direct_capability_source
        || render_compiler_candidate_frontend_result(&result).as_bytes()
            != input.frontend_result_source
        || input.direct_compile_verification.result != input.frontend_result_source
    {
        return Err(ArtifactError::new(
            "compiler candidate successor sources are not canonical",
        ));
    }
    validate_source_lineage(input)
}

fn validate_source_lineage(
    input: CompilerCandidateSuccessorVerificationInput<'_>,
) -> Result<(), ArtifactError> {
    let preselection = input.preselection;
    let capability = input.direct_capability;
    let candidate = input.direct_compile_verification.candidate;
    let production = input.direct_compile_verification.production;
    if preselection.protocol != COMPILER_CANDIDATE_PRESELECTION_PROTOCOL
        || preselection.target_generation != TARGET_GENERATION
        || !preselection.provider_dependency_required
        || preselection.direct_stage1_compile
        || preselection.replacement_authorized
        || preselection.selection_authorized
        || !preselection.preselection_authorized
        || capability.protocol != COMPILER_CANDIDATE_DIRECT_COMPILE_CAPABILITY_PROTOCOL
        || capability.component_id != preselection.component_id
        || capability.component_domain != preselection.component_domain
        || capability.component_unit != preselection.component_unit
        || capability.candidate_record_sha256 != preselection.candidate_record_sha256
        || capability.candidate_reproducible_build_sha256
            != preselection.candidate_reproducible_build_sha256
        || capability.candidate_producer_id != preselection.candidate_producer_id
        || capability.candidate_compiler_image_sha256
            != preselection.candidate_compiler_image_sha256
        || capability.production_protocol != COMPILER_CANDIDATE_PRODUCTION_PROTOCOL
        || capability.production_proof_sha256 != preselection.production_proof_sha256
        || capability.candidate_record_sha256 != candidate.record_sha256
        || capability.production_proof_sha256 != production.proof_sha256
        || capability.provider_dependency_required
        || !capability.direct_stage1_compile
        || capability.native_materialization
        || capability.replacement_authorized
        || capability.selection_authorized
    {
        return Err(ArtifactError::new(
            "compiler candidate successor source lineage is inconsistent",
        ));
    }
    Ok(())
}

fn validate_successor_lineage(
    successor: &CompilerCandidateSuccessor,
    input: CompilerCandidateSuccessorVerificationInput<'_>,
) -> Result<(), ArtifactError> {
    let preselection = input.preselection;
    let capability = input.direct_capability;
    if successor.component_id != capability.component_id
        || successor.component_domain != capability.component_domain
        || successor.component_unit != capability.component_unit
        || successor.predecessor_preselection_file_bytes != input.preselection_source.len()
        || successor.predecessor_preselection_file_sha256
            != sha256_hex(input.preselection_source.as_bytes())
        || successor.predecessor_preselection_id != preselection.preselection_id
        || successor.predecessor_preselection_proof_sha256 != preselection.proof_sha256
        || successor.candidate_record_sha256 != capability.candidate_record_sha256
        || successor.candidate_reproducible_build_sha256
            != capability.candidate_reproducible_build_sha256
        || successor.candidate_producer_id != capability.candidate_producer_id
        || successor.candidate_compiler_image_sha256 != capability.candidate_compiler_image_sha256
        || successor.production_protocol != capability.production_protocol
        || successor.production_proof_sha256 != capability.production_proof_sha256
        || successor.direct_compile_capability_file_bytes != input.direct_capability_source.len()
        || successor.direct_compile_capability_file_sha256
            != sha256_hex(input.direct_capability_source.as_bytes())
        || successor.direct_compile_capability_proof_sha256 != capability.proof_sha256
        || successor.direct_compile_driver_contract != capability.driver_contract
        || successor.direct_compile_provider_contract != capability.provider_contract
        || successor.direct_compile_input_identity_sha256 != capability.input_identity_sha256
        || successor.frontend_result_bytes != input.frontend_result_source.len()
        || successor.frontend_result_sha256 != sha256_hex(input.frontend_result_source)
        || successor.frontend_result_bundle_fold != capability.result_bundle_fold
        || successor.authorizer_id != preselection.authorizer_id
        || successor.authorizer_environment_id != preselection.authorizer_environment_id
        || successor.authorizer_public_key_id != preselection.authorizer_public_key_id
    {
        return Err(ArtifactError::new(
            "compiler candidate successor bound lineage mismatch",
        ));
    }
    Ok(())
}

pub(super) fn validate_successor(
    successor: &CompilerCandidateSuccessor,
) -> Result<(), ArtifactError> {
    if successor.protocol != COMPILER_CANDIDATE_SUCCESSOR_PROTOCOL
        || successor.authority != COMPILER_CANDIDATE_SUCCESSOR_AUTHORITY
        || successor.signature_contract != COMPILER_CANDIDATE_SUCCESSOR_SIGNATURE_CONTRACT
        || successor.action != COMPILER_CANDIDATE_SUCCESSOR_ACTION
        || successor.relation_contract != COMPILER_CANDIDATE_SUCCESSOR_RELATION_CONTRACT
        || successor.target_generation != TARGET_GENERATION
        || successor.predecessor_preselection_protocol != COMPILER_CANDIDATE_PRESELECTION_PROTOCOL
        || successor.predecessor_preselection_file != COMPILER_CANDIDATE_PRESELECTION_FILE
        || successor.candidate_stage_role != COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE
        || successor.production_protocol != COMPILER_CANDIDATE_PRODUCTION_PROTOCOL
        || successor.direct_compile_capability_protocol
            != COMPILER_CANDIDATE_DIRECT_COMPILE_CAPABILITY_PROTOCOL
        || successor.direct_compile_capability_file
            != COMPILER_CANDIDATE_DIRECT_COMPILE_CAPABILITY_FILE
        || successor.direct_compile_driver_contract
            != COMPILER_CANDIDATE_DIRECT_COMPILE_DRIVER_CONTRACT
        || successor.direct_compile_provider_contract
            != COMPILER_CANDIDATE_DIRECT_COMPILE_PROVIDER_CONTRACT
        || successor.frontend_result_protocol != COMPILER_CANDIDATE_FRONTEND_RESULT_PROTOCOL
        || successor.frontend_result_file != COMPILER_CANDIDATE_FRONTEND_RESULT_FILE
        || successor.provider_dependency_required
        || !successor.direct_stage1_compile
        || successor.fresh_source_compile
        || successor.native_materialization
        || successor.replacement_authorized
        || successor.selection_authorized
        || !successor.preselection_authorized
        || !successor.successor_authorized
        || successor.verdict != COMPILER_CANDIDATE_SUCCESSOR_VERDICT
        || successor.predecessor_preselection_file_bytes == 0
        || successor.direct_compile_capability_file_bytes == 0
        || successor.frontend_result_bytes == 0
        || successor.frontend_result_bundle_fold == 0
    {
        return Err(ArtifactError::new(
            "compiler candidate successor contract mismatch",
        ));
    }
    for (value, label) in [
        (&successor.component_id, "candidate successor component id"),
        (
            &successor.component_domain,
            "candidate successor component domain",
        ),
        (
            &successor.component_unit,
            "candidate successor component unit",
        ),
        (&successor.successor_id, "candidate successor id"),
        (
            &successor.predecessor_preselection_id,
            "predecessor candidate preselection id",
        ),
        (&successor.candidate_producer_id, "candidate producer id"),
        (
            &successor.authorizer_id,
            "candidate successor authorizer id",
        ),
        (
            &successor.authorizer_environment_id,
            "candidate successor authorizer environment id",
        ),
    ] {
        validate_token(value, label)?;
    }
    for (value, label) in successor_hashes(successor) {
        validate_sha256(value, label)?;
    }
    if !successor
        .authorizer_public_key_id
        .strip_prefix("ed25519:sha256:")
        .is_some_and(is_sha256)
    {
        return Err(ArtifactError::new(
            "compiler candidate successor authorizer public key id is malformed",
        ));
    }
    decode_array::<64>(&successor.signature_hex, "candidate successor signature")?;
    if successor.proof_sha256 != successor_identity(successor) {
        return Err(ArtifactError::new(
            "compiler candidate successor proof identity mismatch",
        ));
    }
    Ok(())
}

fn verify_successor_signature(
    successor: &CompilerCandidateSuccessor,
    registry: &CompilerComponentReplacementAuthorizerRegistry,
) -> Result<(), ArtifactError> {
    let verifying_key = resolve_replacement_authorizer_key(
        registry,
        &successor.component_id,
        &successor.authorizer_id,
        &successor.authorizer_environment_id,
        &successor.authorizer_public_key_id,
    )?;
    let signature = Signature::from_bytes(&decode_array::<64>(
        &successor.signature_hex,
        "candidate successor signature",
    )?);
    verifying_key
        .verify_strict(&signature_message(&successor.proof_sha256), &signature)
        .map_err(|_| ArtifactError::new("compiler candidate successor signature mismatch"))
}

fn successor_hashes(successor: &CompilerCandidateSuccessor) -> [(&str, &str); 12] {
    [
        (
            &successor.predecessor_preselection_file_sha256,
            "predecessor preselection file",
        ),
        (
            &successor.predecessor_preselection_proof_sha256,
            "predecessor preselection proof",
        ),
        (&successor.challenge_sha256, "candidate successor challenge"),
        (
            &successor.candidate_record_sha256,
            "candidate component record",
        ),
        (
            &successor.candidate_reproducible_build_sha256,
            "candidate reproducible build",
        ),
        (
            &successor.candidate_compiler_image_sha256,
            "candidate compiler image",
        ),
        (
            &successor.production_proof_sha256,
            "candidate production proof",
        ),
        (
            &successor.direct_compile_capability_file_sha256,
            "direct compile capability file",
        ),
        (
            &successor.direct_compile_capability_proof_sha256,
            "direct compile capability proof",
        ),
        (
            &successor.direct_compile_input_identity_sha256,
            "direct compile input identity",
        ),
        (&successor.frontend_result_sha256, "front-end result"),
        (&successor.proof_sha256, "candidate successor proof"),
    ]
}

fn successor_identity(successor: &CompilerCandidateSuccessor) -> String {
    let mut hash = Sha256::new();
    for value in [
        successor.protocol.as_bytes(),
        successor.authority.as_bytes(),
        successor.signature_contract.as_bytes(),
        successor.action.as_bytes(),
        successor.relation_contract.as_bytes(),
        successor.component_id.as_bytes(),
        successor.component_domain.as_bytes(),
        successor.component_unit.as_bytes(),
        successor.successor_id.as_bytes(),
        successor.predecessor_preselection_protocol.as_bytes(),
        successor.predecessor_preselection_file.as_bytes(),
        successor.predecessor_preselection_file_sha256.as_bytes(),
        successor.predecessor_preselection_id.as_bytes(),
        successor.predecessor_preselection_proof_sha256.as_bytes(),
        successor.challenge_sha256.as_bytes(),
        successor.candidate_stage_role.as_bytes(),
        successor.candidate_record_sha256.as_bytes(),
        successor.candidate_reproducible_build_sha256.as_bytes(),
        successor.candidate_producer_id.as_bytes(),
        successor.candidate_compiler_image_sha256.as_bytes(),
        successor.production_protocol.as_bytes(),
        successor.production_proof_sha256.as_bytes(),
        successor.direct_compile_capability_protocol.as_bytes(),
        successor.direct_compile_capability_file.as_bytes(),
        successor.direct_compile_capability_file_sha256.as_bytes(),
        successor.direct_compile_capability_proof_sha256.as_bytes(),
        successor.direct_compile_driver_contract.as_bytes(),
        successor.direct_compile_provider_contract.as_bytes(),
        successor.direct_compile_input_identity_sha256.as_bytes(),
        successor.frontend_result_protocol.as_bytes(),
        successor.frontend_result_file.as_bytes(),
        successor.frontend_result_sha256.as_bytes(),
        successor.authorizer_id.as_bytes(),
        successor.authorizer_environment_id.as_bytes(),
        successor.authorizer_public_key_id.as_bytes(),
        successor.verdict.as_bytes(),
    ] {
        hash_field(&mut hash, value);
    }
    for value in [
        successor.target_generation,
        successor.predecessor_preselection_file_bytes,
        successor.direct_compile_capability_file_bytes,
        successor.frontend_result_bytes,
        successor.frontend_result_bundle_fold,
    ] {
        hash_field(&mut hash, &(value as u64).to_le_bytes());
    }
    for value in [
        successor.provider_dependency_required,
        successor.direct_stage1_compile,
        successor.fresh_source_compile,
        successor.native_materialization,
        successor.replacement_authorized,
        successor.selection_authorized,
        successor.preselection_authorized,
        successor.successor_authorized,
    ] {
        hash_field(&mut hash, &[u8::from(value)]);
    }
    encode_hex(&hash.finalize())
}

fn signature_message(proof_sha256: &str) -> Vec<u8> {
    let mut message = Vec::with_capacity(
        COMPILER_CANDIDATE_SUCCESSOR_SIGNATURE_CONTRACT.len() + proof_sha256.len() + 1,
    );
    message.extend_from_slice(COMPILER_CANDIDATE_SUCCESSOR_SIGNATURE_CONTRACT.as_bytes());
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
#[path = "compiler_candidate_successor_tests.rs"]
mod tests;
