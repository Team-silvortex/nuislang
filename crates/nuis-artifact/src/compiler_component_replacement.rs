use std::{fs, path::Path};

use ed25519_dalek::{Signature, Signer, SigningKey};
use sha2::{Digest, Sha256};

use crate::{
    compiler_component_attestation_registry::{decode_array, encode_hex, public_key_id},
    compiler_component_replacement_registry::resolve_replacement_authorizer_key,
    parse_compiler_component_attestation_from_source,
    parse_compiler_component_replacement_authorizer_registry_from_source,
    parse_compiler_component_reproducibility_from_source, render_compiler_component_attestation,
    render_compiler_component_replacement_authorizer_registry,
    render_compiler_component_reproducibility,
    toml::{
        escape_toml_string, parse_required_toml_bool, parse_required_toml_string,
        parse_required_toml_usize,
    },
    verify_compiler_component_attestation, ArtifactError, CompilerComponentAttestation,
    CompilerComponentAttesterTrustRegistry, CompilerComponentReplacementAuthorizerRegistry,
    CompilerComponentReproducibility, COMPILER_COMPONENT_ATTESTATION_FILE,
    COMPILER_COMPONENT_ATTESTATION_PROTOCOL,
    COMPILER_COMPONENT_REPLACEMENT_AUTHORIZER_REGISTRY_FILE,
    COMPILER_COMPONENT_REPRODUCIBILITY_FILE, COMPILER_COMPONENT_REPRODUCIBILITY_PROTOCOL,
};

pub const COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_PROTOCOL: &str =
    "nuis-compiler-component-replacement-authorization-v1";
pub const COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_FILE: &str =
    "nuis.compiler-component-replacement-authorization.toml";
pub const COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_AUTHORITY: &str =
    "independent-ed25519-component-owner-transition";
pub const COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_SIGNATURE_CONTRACT: &str =
    "nuis-compiler-component-replacement-authorization-ed25519-v1";
pub const COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_ACTION: &str = "activate-candidate";
pub const COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_VERDICT: &str =
    "candidate-activation-authorized-reversible";

const GENESIS_GENERATION: usize = 1;
const GENESIS_PREDECESSOR_SHA256: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

#[derive(Debug, Clone, Copy)]
pub struct CompilerComponentReplacementAuthorizationInput<'a> {
    pub reproducibility: &'a CompilerComponentReproducibility,
    pub reproducibility_source: &'a str,
    pub attestation: &'a CompilerComponentAttestation,
    pub attestation_source: &'a str,
    pub challenge_sha256: &'a str,
    pub authorization_id: &'a str,
    pub authorizer_id: &'a str,
    pub environment_id: &'a str,
}

#[derive(Debug, Clone, Copy)]
pub struct CompilerComponentReplacementVerificationInput<'a> {
    pub reproducibility: &'a CompilerComponentReproducibility,
    pub reproducibility_source: &'a str,
    pub attestation: &'a CompilerComponentAttestation,
    pub attestation_source: &'a str,
    pub attester_registry: &'a CompilerComponentAttesterTrustRegistry,
    pub attester_registry_source: &'a str,
    pub expected_attester_registry_sha256: &'a str,
    pub expected_attestation_challenge_sha256: &'a str,
    pub authorizer_registry: &'a CompilerComponentReplacementAuthorizerRegistry,
    pub authorizer_registry_source: &'a str,
    pub expected_authorizer_registry_sha256: &'a str,
    pub expected_authorization_challenge_sha256: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerComponentReplacementAuthorization {
    pub protocol: String,
    pub authority: String,
    pub signature_contract: String,
    pub action: String,
    pub component_id: String,
    pub authorization_id: String,
    pub generation: usize,
    pub predecessor_authorization_sha256: String,
    pub challenge_sha256: String,
    pub reproducibility_protocol: String,
    pub reproducibility_file: String,
    pub reproducibility_file_bytes: usize,
    pub reproducibility_file_sha256: String,
    pub reproducibility_aggregate_sha256: String,
    pub attestation_protocol: String,
    pub attestation_file: String,
    pub attestation_file_bytes: usize,
    pub attestation_file_sha256: String,
    pub attestation_proof_sha256: String,
    pub attestation_challenge_sha256: String,
    pub attester_id: String,
    pub attester_environment_id: String,
    pub attester_public_key_id: String,
    pub from_reproducible_build_sha256: String,
    pub to_reproducible_build_sha256: String,
    pub rollback_reproducible_build_sha256: String,
    pub candidate_compiler_image_sha256: String,
    pub native_output_sha256: String,
    pub reversible: bool,
    pub attestation_replacement_authorized: bool,
    pub replacement_authorized: bool,
    pub authorizer_id: String,
    pub authorizer_environment_id: String,
    pub authorizer_public_key_id: String,
    pub verdict: String,
    pub proof_sha256: String,
    pub signature_hex: String,
}

pub fn build_compiler_component_replacement_authorization(
    input: CompilerComponentReplacementAuthorizationInput<'_>,
    signing_key_hex: &str,
) -> Result<CompilerComponentReplacementAuthorization, ArtifactError> {
    validate_sha256(
        input.challenge_sha256,
        "replacement authorization challenge",
    )?;
    validate_token(input.authorization_id, "replacement authorization id")?;
    validate_token(input.authorizer_id, "replacement authorizer id")?;
    validate_token(
        input.environment_id,
        "replacement authorizer environment id",
    )?;
    validate_source_lineage(
        input.reproducibility,
        input.reproducibility_source,
        input.attestation,
        input.attestation_source,
    )?;
    if input.authorizer_id == input.attestation.attester_id {
        return Err(ArtifactError::new(
            "compiler replacement authorizer identity must differ from attester identity",
        ));
    }
    let signing_key = SigningKey::from_bytes(&decode_array::<32>(
        signing_key_hex,
        "replacement authorizer signing key",
    )?);
    let authorizer_public_key_id = public_key_id(&signing_key.verifying_key());
    if authorizer_public_key_id == input.attestation.attester_public_key_id {
        return Err(ArtifactError::new(
            "compiler replacement authorizer key must differ from attester key",
        ));
    }
    if input.reproducibility.stage0_reproducible_build_sha256
        == input.reproducibility.candidate_reproducible_build_sha256
    {
        return Err(ArtifactError::new(
            "compiler replacement transition must change reproducible build identity",
        ));
    }

    let mut authorization = CompilerComponentReplacementAuthorization {
        protocol: COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_PROTOCOL.to_owned(),
        authority: COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_AUTHORITY.to_owned(),
        signature_contract: COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_SIGNATURE_CONTRACT
            .to_owned(),
        action: COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_ACTION.to_owned(),
        component_id: input.reproducibility.component_id.clone(),
        authorization_id: input.authorization_id.to_owned(),
        generation: GENESIS_GENERATION,
        predecessor_authorization_sha256: GENESIS_PREDECESSOR_SHA256.to_owned(),
        challenge_sha256: input.challenge_sha256.to_owned(),
        reproducibility_protocol: input.reproducibility.protocol.clone(),
        reproducibility_file: COMPILER_COMPONENT_REPRODUCIBILITY_FILE.to_owned(),
        reproducibility_file_bytes: input.reproducibility_source.len(),
        reproducibility_file_sha256: sha256_hex(input.reproducibility_source.as_bytes()),
        reproducibility_aggregate_sha256: input.reproducibility.aggregate_sha256.clone(),
        attestation_protocol: input.attestation.protocol.clone(),
        attestation_file: COMPILER_COMPONENT_ATTESTATION_FILE.to_owned(),
        attestation_file_bytes: input.attestation_source.len(),
        attestation_file_sha256: sha256_hex(input.attestation_source.as_bytes()),
        attestation_proof_sha256: input.attestation.proof_sha256.clone(),
        attestation_challenge_sha256: input.attestation.challenge_sha256.clone(),
        attester_id: input.attestation.attester_id.clone(),
        attester_environment_id: input.attestation.environment_id.clone(),
        attester_public_key_id: input.attestation.attester_public_key_id.clone(),
        from_reproducible_build_sha256: input
            .reproducibility
            .stage0_reproducible_build_sha256
            .clone(),
        to_reproducible_build_sha256: input
            .reproducibility
            .candidate_reproducible_build_sha256
            .clone(),
        rollback_reproducible_build_sha256: input
            .reproducibility
            .stage0_reproducible_build_sha256
            .clone(),
        candidate_compiler_image_sha256: input
            .reproducibility
            .candidate_compiler_image_sha256
            .clone(),
        native_output_sha256: input.reproducibility.native_output_sha256.clone(),
        reversible: true,
        attestation_replacement_authorized: false,
        replacement_authorized: true,
        authorizer_id: input.authorizer_id.to_owned(),
        authorizer_environment_id: input.environment_id.to_owned(),
        authorizer_public_key_id,
        verdict: COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_VERDICT.to_owned(),
        proof_sha256: String::new(),
        signature_hex: String::new(),
    };
    authorization.proof_sha256 = authorization_identity(&authorization);
    authorization.signature_hex = encode_hex(
        &signing_key
            .sign(&signature_message(&authorization.proof_sha256))
            .to_bytes(),
    );
    validate_authorization(&authorization)?;
    Ok(authorization)
}

pub fn render_compiler_component_replacement_authorization(
    authorization: &CompilerComponentReplacementAuthorization,
) -> String {
    format!(
        "protocol = \"{}\"\nauthority = \"{}\"\nsignature_contract = \"{}\"\naction = \"{}\"\ncomponent_id = \"{}\"\nauthorization_id = \"{}\"\ngeneration = {}\npredecessor_authorization_sha256 = \"{}\"\nchallenge_sha256 = \"{}\"\nreproducibility_protocol = \"{}\"\nreproducibility_file = \"{}\"\nreproducibility_file_bytes = {}\nreproducibility_file_sha256 = \"{}\"\nreproducibility_aggregate_sha256 = \"{}\"\nattestation_protocol = \"{}\"\nattestation_file = \"{}\"\nattestation_file_bytes = {}\nattestation_file_sha256 = \"{}\"\nattestation_proof_sha256 = \"{}\"\nattestation_challenge_sha256 = \"{}\"\nattester_id = \"{}\"\nattester_environment_id = \"{}\"\nattester_public_key_id = \"{}\"\nfrom_reproducible_build_sha256 = \"{}\"\nto_reproducible_build_sha256 = \"{}\"\nrollback_reproducible_build_sha256 = \"{}\"\ncandidate_compiler_image_sha256 = \"{}\"\nnative_output_sha256 = \"{}\"\nreversible = {}\nattestation_replacement_authorized = {}\nreplacement_authorized = {}\nauthorizer_id = \"{}\"\nauthorizer_environment_id = \"{}\"\nauthorizer_public_key_id = \"{}\"\nverdict = \"{}\"\nproof_sha256 = \"{}\"\nsignature_hex = \"{}\"\n",
        authorization.protocol,
        authorization.authority,
        authorization.signature_contract,
        authorization.action,
        escape_toml_string(&authorization.component_id),
        escape_toml_string(&authorization.authorization_id),
        authorization.generation,
        authorization.predecessor_authorization_sha256,
        authorization.challenge_sha256,
        authorization.reproducibility_protocol,
        authorization.reproducibility_file,
        authorization.reproducibility_file_bytes,
        authorization.reproducibility_file_sha256,
        authorization.reproducibility_aggregate_sha256,
        authorization.attestation_protocol,
        authorization.attestation_file,
        authorization.attestation_file_bytes,
        authorization.attestation_file_sha256,
        authorization.attestation_proof_sha256,
        authorization.attestation_challenge_sha256,
        escape_toml_string(&authorization.attester_id),
        escape_toml_string(&authorization.attester_environment_id),
        authorization.attester_public_key_id,
        authorization.from_reproducible_build_sha256,
        authorization.to_reproducible_build_sha256,
        authorization.rollback_reproducible_build_sha256,
        authorization.candidate_compiler_image_sha256,
        authorization.native_output_sha256,
        authorization.reversible,
        authorization.attestation_replacement_authorized,
        authorization.replacement_authorized,
        escape_toml_string(&authorization.authorizer_id),
        escape_toml_string(&authorization.authorizer_environment_id),
        authorization.authorizer_public_key_id,
        authorization.verdict,
        authorization.proof_sha256,
        authorization.signature_hex,
    )
}

pub fn parse_compiler_component_replacement_authorization(
    path: &Path,
) -> Result<CompilerComponentReplacementAuthorization, ArtifactError> {
    let source = fs::read_to_string(path).map_err(|error| {
        ArtifactError::new(format!(
            "failed to read compiler replacement authorization `{}`: {error}",
            path.display()
        ))
    })?;
    parse_compiler_component_replacement_authorization_from_source(&source, path)
}

pub fn parse_compiler_component_replacement_authorization_from_source(
    source: &str,
    path: &Path,
) -> Result<CompilerComponentReplacementAuthorization, ArtifactError> {
    validate_text(source, path)?;
    let authorization = CompilerComponentReplacementAuthorization {
        protocol: parse_required_toml_string(source, "protocol", path)?,
        authority: parse_required_toml_string(source, "authority", path)?,
        signature_contract: parse_required_toml_string(source, "signature_contract", path)?,
        action: parse_required_toml_string(source, "action", path)?,
        component_id: parse_required_toml_string(source, "component_id", path)?,
        authorization_id: parse_required_toml_string(source, "authorization_id", path)?,
        generation: parse_required_toml_usize(source, "generation", path)?,
        predecessor_authorization_sha256: parse_required_toml_string(
            source,
            "predecessor_authorization_sha256",
            path,
        )?,
        challenge_sha256: parse_required_toml_string(source, "challenge_sha256", path)?,
        reproducibility_protocol: parse_required_toml_string(
            source,
            "reproducibility_protocol",
            path,
        )?,
        reproducibility_file: parse_required_toml_string(source, "reproducibility_file", path)?,
        reproducibility_file_bytes: parse_required_toml_usize(
            source,
            "reproducibility_file_bytes",
            path,
        )?,
        reproducibility_file_sha256: parse_required_toml_string(
            source,
            "reproducibility_file_sha256",
            path,
        )?,
        reproducibility_aggregate_sha256: parse_required_toml_string(
            source,
            "reproducibility_aggregate_sha256",
            path,
        )?,
        attestation_protocol: parse_required_toml_string(source, "attestation_protocol", path)?,
        attestation_file: parse_required_toml_string(source, "attestation_file", path)?,
        attestation_file_bytes: parse_required_toml_usize(source, "attestation_file_bytes", path)?,
        attestation_file_sha256: parse_required_toml_string(
            source,
            "attestation_file_sha256",
            path,
        )?,
        attestation_proof_sha256: parse_required_toml_string(
            source,
            "attestation_proof_sha256",
            path,
        )?,
        attestation_challenge_sha256: parse_required_toml_string(
            source,
            "attestation_challenge_sha256",
            path,
        )?,
        attester_id: parse_required_toml_string(source, "attester_id", path)?,
        attester_environment_id: parse_required_toml_string(
            source,
            "attester_environment_id",
            path,
        )?,
        attester_public_key_id: parse_required_toml_string(source, "attester_public_key_id", path)?,
        from_reproducible_build_sha256: parse_required_toml_string(
            source,
            "from_reproducible_build_sha256",
            path,
        )?,
        to_reproducible_build_sha256: parse_required_toml_string(
            source,
            "to_reproducible_build_sha256",
            path,
        )?,
        rollback_reproducible_build_sha256: parse_required_toml_string(
            source,
            "rollback_reproducible_build_sha256",
            path,
        )?,
        candidate_compiler_image_sha256: parse_required_toml_string(
            source,
            "candidate_compiler_image_sha256",
            path,
        )?,
        native_output_sha256: parse_required_toml_string(source, "native_output_sha256", path)?,
        reversible: parse_required_toml_bool(source, "reversible", path)?,
        attestation_replacement_authorized: parse_required_toml_bool(
            source,
            "attestation_replacement_authorized",
            path,
        )?,
        replacement_authorized: parse_required_toml_bool(source, "replacement_authorized", path)?,
        authorizer_id: parse_required_toml_string(source, "authorizer_id", path)?,
        authorizer_environment_id: parse_required_toml_string(
            source,
            "authorizer_environment_id",
            path,
        )?,
        authorizer_public_key_id: parse_required_toml_string(
            source,
            "authorizer_public_key_id",
            path,
        )?,
        verdict: parse_required_toml_string(source, "verdict", path)?,
        proof_sha256: parse_required_toml_string(source, "proof_sha256", path)?,
        signature_hex: parse_required_toml_string(source, "signature_hex", path)?,
    };
    validate_authorization(&authorization)?;
    if render_compiler_component_replacement_authorization(&authorization) != source {
        return Err(ArtifactError::new(format!(
            "compiler replacement authorization `{}` is not canonically encoded",
            path.display()
        )));
    }
    Ok(authorization)
}

pub fn verify_compiler_component_replacement_authorization(
    authorization: &CompilerComponentReplacementAuthorization,
    input: CompilerComponentReplacementVerificationInput<'_>,
) -> Result<(), ArtifactError> {
    validate_authorization(authorization)?;
    validate_sha256(
        input.expected_authorizer_registry_sha256,
        "pinned replacement authorizer registry",
    )?;
    validate_sha256(
        input.expected_authorization_challenge_sha256,
        "expected replacement authorization challenge",
    )?;
    if authorization.challenge_sha256 != input.expected_authorization_challenge_sha256 {
        return Err(ArtifactError::new(
            "compiler replacement authorization challenge does not match the verifier request",
        ));
    }
    verify_compiler_component_attestation(
        input.attestation,
        input.reproducibility,
        input.reproducibility_source,
        input.attester_registry,
        input.attester_registry_source,
        input.expected_attester_registry_sha256,
        input.expected_attestation_challenge_sha256,
    )?;
    validate_source_lineage(
        input.reproducibility,
        input.reproducibility_source,
        input.attestation,
        input.attestation_source,
    )?;
    validate_authorization_lineage(authorization, input)?;

    let parsed_registry = parse_compiler_component_replacement_authorizer_registry_from_source(
        input.authorizer_registry_source,
        Path::new(COMPILER_COMPONENT_REPLACEMENT_AUTHORIZER_REGISTRY_FILE),
    )?;
    if &parsed_registry != input.authorizer_registry
        || render_compiler_component_replacement_authorizer_registry(input.authorizer_registry)
            != input.authorizer_registry_source
    {
        return Err(ArtifactError::new(
            "compiler replacement authorizer registry source is not canonical",
        ));
    }
    if sha256_hex(input.authorizer_registry_source.as_bytes())
        != input.expected_authorizer_registry_sha256
    {
        return Err(ArtifactError::new(
            "compiler replacement authorizer registry does not match its pinned SHA-256",
        ));
    }
    if authorization.authorizer_public_key_id == authorization.attester_public_key_id
        || authorization.authorizer_id == authorization.attester_id
    {
        return Err(ArtifactError::new(
            "compiler replacement authority must remain distinct from attester trust",
        ));
    }
    let verifying_key = resolve_replacement_authorizer_key(
        input.authorizer_registry,
        &authorization.component_id,
        &authorization.authorizer_id,
        &authorization.authorizer_environment_id,
        &authorization.authorizer_public_key_id,
    )?;
    let signature = Signature::from_bytes(&decode_array::<64>(
        &authorization.signature_hex,
        "replacement authorization signature",
    )?);
    verifying_key
        .verify_strict(&signature_message(&authorization.proof_sha256), &signature)
        .map_err(|_| ArtifactError::new("compiler replacement Ed25519 signature mismatch"))?;
    Ok(())
}

fn validate_source_lineage(
    reproducibility: &CompilerComponentReproducibility,
    reproducibility_source: &str,
    attestation: &CompilerComponentAttestation,
    attestation_source: &str,
) -> Result<(), ArtifactError> {
    let parsed_reproducibility = parse_compiler_component_reproducibility_from_source(
        reproducibility_source,
        Path::new(COMPILER_COMPONENT_REPRODUCIBILITY_FILE),
    )?;
    let parsed_attestation = parse_compiler_component_attestation_from_source(
        attestation_source,
        Path::new(COMPILER_COMPONENT_ATTESTATION_FILE),
    )?;
    if &parsed_reproducibility != reproducibility
        || render_compiler_component_reproducibility(reproducibility) != reproducibility_source
        || &parsed_attestation != attestation
        || render_compiler_component_attestation(attestation) != attestation_source
        || reproducibility.replacement_authorized
        || attestation.replacement_authorized
        || attestation.component_id != reproducibility.component_id
        || attestation.reproducibility_aggregate_sha256 != reproducibility.aggregate_sha256
        || attestation.candidate_reproducible_build_sha256
            != reproducibility.candidate_reproducible_build_sha256
        || attestation.candidate_compiler_image_sha256
            != reproducibility.candidate_compiler_image_sha256
        || attestation.native_output_sha256 != reproducibility.native_output_sha256
    {
        return Err(ArtifactError::new(
            "compiler replacement authorization does not bind canonical attested lineage",
        ));
    }
    Ok(())
}

fn validate_authorization_lineage(
    authorization: &CompilerComponentReplacementAuthorization,
    input: CompilerComponentReplacementVerificationInput<'_>,
) -> Result<(), ArtifactError> {
    if authorization.component_id != input.reproducibility.component_id
        || authorization.reproducibility_protocol != input.reproducibility.protocol
        || authorization.reproducibility_file_bytes != input.reproducibility_source.len()
        || authorization.reproducibility_file_sha256
            != sha256_hex(input.reproducibility_source.as_bytes())
        || authorization.reproducibility_aggregate_sha256 != input.reproducibility.aggregate_sha256
        || authorization.attestation_protocol != input.attestation.protocol
        || authorization.attestation_file_bytes != input.attestation_source.len()
        || authorization.attestation_file_sha256 != sha256_hex(input.attestation_source.as_bytes())
        || authorization.attestation_proof_sha256 != input.attestation.proof_sha256
        || authorization.attestation_challenge_sha256 != input.attestation.challenge_sha256
        || authorization.attester_id != input.attestation.attester_id
        || authorization.attester_environment_id != input.attestation.environment_id
        || authorization.attester_public_key_id != input.attestation.attester_public_key_id
        || authorization.from_reproducible_build_sha256
            != input.reproducibility.stage0_reproducible_build_sha256
        || authorization.to_reproducible_build_sha256
            != input.reproducibility.candidate_reproducible_build_sha256
        || authorization.rollback_reproducible_build_sha256
            != input.reproducibility.stage0_reproducible_build_sha256
        || authorization.candidate_compiler_image_sha256
            != input.reproducibility.candidate_compiler_image_sha256
        || authorization.native_output_sha256 != input.reproducibility.native_output_sha256
    {
        return Err(ArtifactError::new(
            "compiler replacement authorization lineage mismatch",
        ));
    }
    Ok(())
}

fn validate_authorization(
    authorization: &CompilerComponentReplacementAuthorization,
) -> Result<(), ArtifactError> {
    if authorization.protocol != COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_PROTOCOL
        || authorization.authority != COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_AUTHORITY
        || authorization.signature_contract
            != COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_SIGNATURE_CONTRACT
        || authorization.action != COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_ACTION
        || authorization.generation != GENESIS_GENERATION
        || authorization.predecessor_authorization_sha256 != GENESIS_PREDECESSOR_SHA256
        || authorization.reproducibility_protocol != COMPILER_COMPONENT_REPRODUCIBILITY_PROTOCOL
        || authorization.reproducibility_file != COMPILER_COMPONENT_REPRODUCIBILITY_FILE
        || authorization.attestation_protocol != COMPILER_COMPONENT_ATTESTATION_PROTOCOL
        || authorization.attestation_file != COMPILER_COMPONENT_ATTESTATION_FILE
        || !authorization.reversible
        || authorization.attestation_replacement_authorized
        || !authorization.replacement_authorized
        || authorization.verdict != COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_VERDICT
    {
        return Err(ArtifactError::new(
            "compiler component replacement authorization contract mismatch",
        ));
    }
    for (value, label) in [
        (&authorization.component_id, "replacement component id"),
        (
            &authorization.authorization_id,
            "replacement authorization id",
        ),
        (&authorization.attester_id, "attester id"),
        (
            &authorization.attester_environment_id,
            "attester environment id",
        ),
        (&authorization.authorizer_id, "replacement authorizer id"),
        (
            &authorization.authorizer_environment_id,
            "replacement authorizer environment id",
        ),
    ] {
        validate_token(value, label)?;
    }
    if authorization.authorizer_id == authorization.attester_id
        || authorization.authorizer_public_key_id == authorization.attester_public_key_id
        || authorization.from_reproducible_build_sha256
            == authorization.to_reproducible_build_sha256
        || authorization.rollback_reproducible_build_sha256
            != authorization.from_reproducible_build_sha256
        || authorization.reproducibility_file_bytes == 0
        || authorization.attestation_file_bytes == 0
    {
        return Err(ArtifactError::new(
            "compiler replacement authorization separation or transition mismatch",
        ));
    }
    for (label, value) in [
        (
            "predecessor authorization",
            &authorization.predecessor_authorization_sha256,
        ),
        ("authorization challenge", &authorization.challenge_sha256),
        (
            "reproducibility file",
            &authorization.reproducibility_file_sha256,
        ),
        (
            "reproducibility aggregate",
            &authorization.reproducibility_aggregate_sha256,
        ),
        ("attestation file", &authorization.attestation_file_sha256),
        ("attestation proof", &authorization.attestation_proof_sha256),
        (
            "attestation challenge",
            &authorization.attestation_challenge_sha256,
        ),
        (
            "from reproducible build",
            &authorization.from_reproducible_build_sha256,
        ),
        (
            "to reproducible build",
            &authorization.to_reproducible_build_sha256,
        ),
        (
            "rollback reproducible build",
            &authorization.rollback_reproducible_build_sha256,
        ),
        (
            "candidate compiler image",
            &authorization.candidate_compiler_image_sha256,
        ),
        ("native output", &authorization.native_output_sha256),
        (
            "replacement authorization proof",
            &authorization.proof_sha256,
        ),
    ] {
        validate_sha256(value, label)?;
    }
    for (value, label) in [
        (
            &authorization.attester_public_key_id,
            "attester public key id",
        ),
        (
            &authorization.authorizer_public_key_id,
            "replacement authorizer public key id",
        ),
    ] {
        if !value.strip_prefix("ed25519:sha256:").is_some_and(is_sha256) {
            return Err(ArtifactError::new(format!("compiler {label} is malformed")));
        }
    }
    decode_array::<64>(
        &authorization.signature_hex,
        "replacement authorization signature",
    )?;
    if authorization.proof_sha256 != authorization_identity(authorization) {
        return Err(ArtifactError::new(
            "compiler replacement authorization proof identity mismatch",
        ));
    }
    Ok(())
}

fn authorization_identity(authorization: &CompilerComponentReplacementAuthorization) -> String {
    let mut hash = Sha256::new();
    for value in [
        authorization.protocol.as_bytes(),
        authorization.authority.as_bytes(),
        authorization.signature_contract.as_bytes(),
        authorization.action.as_bytes(),
        authorization.component_id.as_bytes(),
        authorization.authorization_id.as_bytes(),
        authorization.predecessor_authorization_sha256.as_bytes(),
        authorization.challenge_sha256.as_bytes(),
        authorization.reproducibility_protocol.as_bytes(),
        authorization.reproducibility_file.as_bytes(),
        authorization.reproducibility_file_sha256.as_bytes(),
        authorization.reproducibility_aggregate_sha256.as_bytes(),
        authorization.attestation_protocol.as_bytes(),
        authorization.attestation_file.as_bytes(),
        authorization.attestation_file_sha256.as_bytes(),
        authorization.attestation_proof_sha256.as_bytes(),
        authorization.attestation_challenge_sha256.as_bytes(),
        authorization.attester_id.as_bytes(),
        authorization.attester_environment_id.as_bytes(),
        authorization.attester_public_key_id.as_bytes(),
        authorization.from_reproducible_build_sha256.as_bytes(),
        authorization.to_reproducible_build_sha256.as_bytes(),
        authorization.rollback_reproducible_build_sha256.as_bytes(),
        authorization.candidate_compiler_image_sha256.as_bytes(),
        authorization.native_output_sha256.as_bytes(),
        authorization.authorizer_id.as_bytes(),
        authorization.authorizer_environment_id.as_bytes(),
        authorization.authorizer_public_key_id.as_bytes(),
        authorization.verdict.as_bytes(),
    ] {
        hash_field(&mut hash, value);
    }
    for value in [
        authorization.generation,
        authorization.reproducibility_file_bytes,
        authorization.attestation_file_bytes,
        usize::from(authorization.reversible),
        usize::from(authorization.attestation_replacement_authorized),
        usize::from(authorization.replacement_authorized),
    ] {
        hash_field(&mut hash, &(value as u64).to_le_bytes());
    }
    encode_hex(&hash.finalize())
}

fn signature_message(proof_sha256: &str) -> Vec<u8> {
    let mut message = Vec::with_capacity(
        COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_SIGNATURE_CONTRACT.len()
            + proof_sha256.len()
            + 1,
    );
    message.extend_from_slice(
        COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_SIGNATURE_CONTRACT.as_bytes(),
    );
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

fn validate_text(source: &str, path: &Path) -> Result<(), ArtifactError> {
    if source.is_empty()
        || !source.ends_with('\n')
        || source.contains('\r')
        || source.contains('\0')
    {
        return Err(ArtifactError::new(format!(
            "compiler replacement authorization `{}` must be UTF-8/LF text",
            path.display()
        )));
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    encode_hex(&Sha256::digest(bytes))
}

#[cfg(test)]
#[path = "compiler_component_replacement_tests.rs"]
mod tests;
