use std::{fs, path::Path};

use ed25519_dalek::{Signature, Signer, SigningKey};
use sha2::{Digest, Sha256};

use crate::{
    compiler_component_attestation_registry::{
        decode_array, encode_hex, public_key_id, resolve_attester_key,
    },
    parse_compiler_component_attester_trust_registry_from_source,
    parse_compiler_component_reproducibility_from_source,
    render_compiler_component_reproducibility,
    toml::{
        escape_toml_string, parse_required_toml_bool, parse_required_toml_string,
        parse_required_toml_usize,
    },
    ArtifactError, CompilerComponentAttesterTrustRegistry, CompilerComponentReproducibility,
    COMPILER_CANDIDATE_PRODUCTION_PROTOCOL, COMPILER_COMPONENT_ATTESTER_TRUST_REGISTRY_FILE,
    COMPILER_COMPONENT_REPRODUCIBILITY_FILE, COMPILER_COMPONENT_REPRODUCIBILITY_PROTOCOL,
};

pub const COMPILER_COMPONENT_ATTESTATION_PROTOCOL: &str = "nuis-compiler-component-attestation-v1";
pub const COMPILER_COMPONENT_ATTESTATION_AUTHORITY: &str =
    "external-ed25519-attester-claim-no-replacement";
pub const COMPILER_COMPONENT_ATTESTATION_SIGNATURE_CONTRACT: &str =
    "nuis-compiler-component-attestation-ed25519-v1";
pub const COMPILER_COMPONENT_ATTESTATION_TRUST_SCOPE: &str =
    "independent-machine-compiler-reproducibility-v1";
pub const COMPILER_COMPONENT_ATTESTATION_FILE: &str = "nuis.compiler-component-attestation.toml";
pub const COMPILER_COMPONENT_ATTESTATION_VERDICT: &str =
    "attester-claims-reproducible-equivalent-awaiting-authorization";

const EXPECTED_RUN_COUNT: usize = 2;

#[derive(Debug, Clone, Copy)]
pub struct CompilerComponentAttestationInput<'a> {
    pub reproducibility: &'a CompilerComponentReproducibility,
    pub reproducibility_source: &'a str,
    pub challenge_sha256: &'a str,
    pub attester_id: &'a str,
    pub environment_id: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerComponentAttestation {
    pub protocol: String,
    pub authority: String,
    pub signature_contract: String,
    pub required_trust_scope: String,
    pub component_id: String,
    pub attester_id: String,
    pub environment_id: String,
    pub challenge_sha256: String,
    pub reproducibility_protocol: String,
    pub reproducibility_file: String,
    pub reproducibility_file_bytes: usize,
    pub reproducibility_file_sha256: String,
    pub reproducibility_aggregate_sha256: String,
    pub candidate_production_protocol: String,
    pub run_count: usize,
    pub first_production_proof_sha256: String,
    pub second_production_proof_sha256: String,
    pub candidate_reproducible_build_sha256: String,
    pub candidate_compiler_image_sha256: String,
    pub native_output_sha256: String,
    pub attester_public_key_id: String,
    pub replacement_authorized: bool,
    pub verdict: String,
    pub proof_sha256: String,
    pub signature_hex: String,
}

pub fn build_compiler_component_attestation(
    input: CompilerComponentAttestationInput<'_>,
    signing_key_hex: &str,
) -> Result<CompilerComponentAttestation, ArtifactError> {
    validate_sha256(input.challenge_sha256, "attestation challenge")?;
    validate_token(input.attester_id, "attester id")?;
    validate_token(input.environment_id, "attester environment id")?;
    let parsed = parse_compiler_component_reproducibility_from_source(
        input.reproducibility_source,
        Path::new(COMPILER_COMPONENT_REPRODUCIBILITY_FILE),
    )?;
    if &parsed != input.reproducibility
        || render_compiler_component_reproducibility(input.reproducibility)
            != input.reproducibility_source
    {
        return Err(ArtifactError::new(
            "compiler attestation source does not match its reproducibility aggregate",
        ));
    }
    if input.reproducibility.runs.len() != EXPECTED_RUN_COUNT {
        return Err(ArtifactError::new(
            "compiler attestation requires exactly two reproducibility runs",
        ));
    }
    let signing_key = SigningKey::from_bytes(&decode_array::<32>(
        signing_key_hex,
        "attester signing key",
    )?);
    let mut attestation = CompilerComponentAttestation {
        protocol: COMPILER_COMPONENT_ATTESTATION_PROTOCOL.to_owned(),
        authority: COMPILER_COMPONENT_ATTESTATION_AUTHORITY.to_owned(),
        signature_contract: COMPILER_COMPONENT_ATTESTATION_SIGNATURE_CONTRACT.to_owned(),
        required_trust_scope: COMPILER_COMPONENT_ATTESTATION_TRUST_SCOPE.to_owned(),
        component_id: input.reproducibility.component_id.clone(),
        attester_id: input.attester_id.to_owned(),
        environment_id: input.environment_id.to_owned(),
        challenge_sha256: input.challenge_sha256.to_owned(),
        reproducibility_protocol: input.reproducibility.protocol.clone(),
        reproducibility_file: COMPILER_COMPONENT_REPRODUCIBILITY_FILE.to_owned(),
        reproducibility_file_bytes: input.reproducibility_source.len(),
        reproducibility_file_sha256: sha256_hex(input.reproducibility_source.as_bytes()),
        reproducibility_aggregate_sha256: input.reproducibility.aggregate_sha256.clone(),
        candidate_production_protocol: COMPILER_CANDIDATE_PRODUCTION_PROTOCOL.to_owned(),
        run_count: input.reproducibility.run_count,
        first_production_proof_sha256: input.reproducibility.runs[0]
            .production_proof_sha256
            .clone(),
        second_production_proof_sha256: input.reproducibility.runs[1]
            .production_proof_sha256
            .clone(),
        candidate_reproducible_build_sha256: input
            .reproducibility
            .candidate_reproducible_build_sha256
            .clone(),
        candidate_compiler_image_sha256: input
            .reproducibility
            .candidate_compiler_image_sha256
            .clone(),
        native_output_sha256: input.reproducibility.native_output_sha256.clone(),
        attester_public_key_id: public_key_id(&signing_key.verifying_key()),
        replacement_authorized: false,
        verdict: COMPILER_COMPONENT_ATTESTATION_VERDICT.to_owned(),
        proof_sha256: String::new(),
        signature_hex: String::new(),
    };
    attestation.proof_sha256 = attestation_identity(&attestation);
    attestation.signature_hex = encode_hex(
        &signing_key
            .sign(&signature_message(&attestation.proof_sha256))
            .to_bytes(),
    );
    validate_claim(&attestation)?;
    Ok(attestation)
}

pub fn render_compiler_component_attestation(attestation: &CompilerComponentAttestation) -> String {
    format!(
        "protocol = \"{}\"\nauthority = \"{}\"\nsignature_contract = \"{}\"\nrequired_trust_scope = \"{}\"\ncomponent_id = \"{}\"\nattester_id = \"{}\"\nenvironment_id = \"{}\"\nchallenge_sha256 = \"{}\"\nreproducibility_protocol = \"{}\"\nreproducibility_file = \"{}\"\nreproducibility_file_bytes = {}\nreproducibility_file_sha256 = \"{}\"\nreproducibility_aggregate_sha256 = \"{}\"\ncandidate_production_protocol = \"{}\"\nrun_count = {}\nfirst_production_proof_sha256 = \"{}\"\nsecond_production_proof_sha256 = \"{}\"\ncandidate_reproducible_build_sha256 = \"{}\"\ncandidate_compiler_image_sha256 = \"{}\"\nnative_output_sha256 = \"{}\"\nattester_public_key_id = \"{}\"\nreplacement_authorized = {}\nverdict = \"{}\"\nproof_sha256 = \"{}\"\nsignature_hex = \"{}\"\n",
        attestation.protocol,
        attestation.authority,
        attestation.signature_contract,
        attestation.required_trust_scope,
        escape_toml_string(&attestation.component_id),
        escape_toml_string(&attestation.attester_id),
        escape_toml_string(&attestation.environment_id),
        attestation.challenge_sha256,
        attestation.reproducibility_protocol,
        attestation.reproducibility_file,
        attestation.reproducibility_file_bytes,
        attestation.reproducibility_file_sha256,
        attestation.reproducibility_aggregate_sha256,
        attestation.candidate_production_protocol,
        attestation.run_count,
        attestation.first_production_proof_sha256,
        attestation.second_production_proof_sha256,
        attestation.candidate_reproducible_build_sha256,
        attestation.candidate_compiler_image_sha256,
        attestation.native_output_sha256,
        attestation.attester_public_key_id,
        attestation.replacement_authorized,
        attestation.verdict,
        attestation.proof_sha256,
        attestation.signature_hex,
    )
}

pub fn parse_compiler_component_attestation(
    path: &Path,
) -> Result<CompilerComponentAttestation, ArtifactError> {
    let source = fs::read_to_string(path).map_err(|error| {
        ArtifactError::new(format!(
            "failed to read compiler component attestation `{}`: {error}",
            path.display()
        ))
    })?;
    parse_compiler_component_attestation_from_source(&source, path)
}

pub fn parse_compiler_component_attestation_from_source(
    source: &str,
    path: &Path,
) -> Result<CompilerComponentAttestation, ArtifactError> {
    validate_text(source, path)?;
    let attestation = CompilerComponentAttestation {
        protocol: parse_required_toml_string(source, "protocol", path)?,
        authority: parse_required_toml_string(source, "authority", path)?,
        signature_contract: parse_required_toml_string(source, "signature_contract", path)?,
        required_trust_scope: parse_required_toml_string(source, "required_trust_scope", path)?,
        component_id: parse_required_toml_string(source, "component_id", path)?,
        attester_id: parse_required_toml_string(source, "attester_id", path)?,
        environment_id: parse_required_toml_string(source, "environment_id", path)?,
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
        candidate_production_protocol: parse_required_toml_string(
            source,
            "candidate_production_protocol",
            path,
        )?,
        run_count: parse_required_toml_usize(source, "run_count", path)?,
        first_production_proof_sha256: parse_required_toml_string(
            source,
            "first_production_proof_sha256",
            path,
        )?,
        second_production_proof_sha256: parse_required_toml_string(
            source,
            "second_production_proof_sha256",
            path,
        )?,
        candidate_reproducible_build_sha256: parse_required_toml_string(
            source,
            "candidate_reproducible_build_sha256",
            path,
        )?,
        candidate_compiler_image_sha256: parse_required_toml_string(
            source,
            "candidate_compiler_image_sha256",
            path,
        )?,
        native_output_sha256: parse_required_toml_string(source, "native_output_sha256", path)?,
        attester_public_key_id: parse_required_toml_string(source, "attester_public_key_id", path)?,
        replacement_authorized: parse_required_toml_bool(source, "replacement_authorized", path)?,
        verdict: parse_required_toml_string(source, "verdict", path)?,
        proof_sha256: parse_required_toml_string(source, "proof_sha256", path)?,
        signature_hex: parse_required_toml_string(source, "signature_hex", path)?,
    };
    validate_claim(&attestation)?;
    if render_compiler_component_attestation(&attestation) != source {
        return Err(ArtifactError::new(format!(
            "compiler component attestation `{}` is not canonically encoded",
            path.display()
        )));
    }
    Ok(attestation)
}

pub fn verify_compiler_component_attestation(
    attestation: &CompilerComponentAttestation,
    reproducibility: &CompilerComponentReproducibility,
    reproducibility_source: &str,
    registry: &CompilerComponentAttesterTrustRegistry,
    registry_source: &str,
    expected_registry_sha256: &str,
    expected_challenge_sha256: &str,
) -> Result<(), ArtifactError> {
    validate_claim(attestation)?;
    validate_sha256(expected_registry_sha256, "pinned trust registry")?;
    validate_sha256(expected_challenge_sha256, "expected attestation challenge")?;
    if attestation.challenge_sha256 != expected_challenge_sha256 {
        return Err(ArtifactError::new(
            "compiler attestation challenge does not match the verifier request",
        ));
    }
    let parsed_reproducibility = parse_compiler_component_reproducibility_from_source(
        reproducibility_source,
        Path::new(COMPILER_COMPONENT_REPRODUCIBILITY_FILE),
    )?;
    if &parsed_reproducibility != reproducibility
        || render_compiler_component_reproducibility(reproducibility) != reproducibility_source
    {
        return Err(ArtifactError::new(
            "compiler attestation reproducibility source is not canonical",
        ));
    }
    let parsed_registry = parse_compiler_component_attester_trust_registry_from_source(
        registry_source,
        Path::new(COMPILER_COMPONENT_ATTESTER_TRUST_REGISTRY_FILE),
    )?;
    if &parsed_registry != registry {
        return Err(ArtifactError::new(
            "compiler attestation registry source does not match its parsed registry",
        ));
    }
    if sha256_hex(registry_source.as_bytes()) != expected_registry_sha256 {
        return Err(ArtifactError::new(
            "compiler attestation trust registry does not match its pinned SHA-256",
        ));
    }
    validate_reproducibility_binding(attestation, reproducibility, reproducibility_source)?;
    let verifying_key = resolve_attester_key(
        registry,
        &attestation.attester_id,
        &attestation.environment_id,
        &attestation.attester_public_key_id,
    )?;
    let signature = Signature::from_bytes(&decode_array::<64>(
        &attestation.signature_hex,
        "attestation signature",
    )?);
    verifying_key
        .verify_strict(&signature_message(&attestation.proof_sha256), &signature)
        .map_err(|_| ArtifactError::new("compiler attestation Ed25519 signature mismatch"))?;
    Ok(())
}

pub fn read_compiler_component_attestation(
    attestation_path: &Path,
    reproducibility_path: &Path,
    registry_path: &Path,
    expected_registry_sha256: &str,
    expected_challenge_sha256: &str,
) -> Result<CompilerComponentAttestation, ArtifactError> {
    let attestation = parse_compiler_component_attestation(attestation_path)?;
    let reproducibility_source = fs::read_to_string(reproducibility_path).map_err(|error| {
        ArtifactError::new(format!(
            "failed to read attested compiler reproducibility aggregate `{}`: {error}",
            reproducibility_path.display()
        ))
    })?;
    let reproducibility = parse_compiler_component_reproducibility_from_source(
        &reproducibility_source,
        reproducibility_path,
    )?;
    let registry_source = fs::read_to_string(registry_path).map_err(|error| {
        ArtifactError::new(format!(
            "failed to read compiler attester trust registry `{}`: {error}",
            registry_path.display()
        ))
    })?;
    let registry = parse_compiler_component_attester_trust_registry_from_source(
        &registry_source,
        registry_path,
    )?;
    verify_compiler_component_attestation(
        &attestation,
        &reproducibility,
        &reproducibility_source,
        &registry,
        &registry_source,
        expected_registry_sha256,
        expected_challenge_sha256,
    )?;
    Ok(attestation)
}

fn validate_reproducibility_binding(
    attestation: &CompilerComponentAttestation,
    reproducibility: &CompilerComponentReproducibility,
    reproducibility_source: &str,
) -> Result<(), ArtifactError> {
    if reproducibility.runs.len() != EXPECTED_RUN_COUNT
        || attestation.component_id != reproducibility.component_id
        || attestation.reproducibility_protocol != reproducibility.protocol
        || attestation.reproducibility_file_bytes != reproducibility_source.len()
        || attestation.reproducibility_file_sha256 != sha256_hex(reproducibility_source.as_bytes())
        || attestation.reproducibility_aggregate_sha256 != reproducibility.aggregate_sha256
        || attestation.run_count != reproducibility.run_count
        || attestation.first_production_proof_sha256
            != reproducibility.runs[0].production_proof_sha256
        || attestation.second_production_proof_sha256
            != reproducibility.runs[1].production_proof_sha256
        || attestation.candidate_reproducible_build_sha256
            != reproducibility.candidate_reproducible_build_sha256
        || attestation.candidate_compiler_image_sha256
            != reproducibility.candidate_compiler_image_sha256
        || attestation.native_output_sha256 != reproducibility.native_output_sha256
        || reproducibility.replacement_authorized
    {
        return Err(ArtifactError::new(
            "compiler attestation does not bind its exact reproducibility lineage",
        ));
    }
    Ok(())
}

fn validate_claim(attestation: &CompilerComponentAttestation) -> Result<(), ArtifactError> {
    if attestation.protocol != COMPILER_COMPONENT_ATTESTATION_PROTOCOL
        || attestation.authority != COMPILER_COMPONENT_ATTESTATION_AUTHORITY
        || attestation.signature_contract != COMPILER_COMPONENT_ATTESTATION_SIGNATURE_CONTRACT
        || attestation.required_trust_scope != COMPILER_COMPONENT_ATTESTATION_TRUST_SCOPE
        || attestation.reproducibility_protocol != COMPILER_COMPONENT_REPRODUCIBILITY_PROTOCOL
        || attestation.reproducibility_file != COMPILER_COMPONENT_REPRODUCIBILITY_FILE
        || attestation.candidate_production_protocol != COMPILER_CANDIDATE_PRODUCTION_PROTOCOL
        || attestation.run_count != EXPECTED_RUN_COUNT
        || attestation.replacement_authorized
        || attestation.verdict != COMPILER_COMPONENT_ATTESTATION_VERDICT
    {
        return Err(ArtifactError::new(
            "compiler component attestation contract mismatch",
        ));
    }
    validate_token(&attestation.attester_id, "attester id")?;
    validate_token(&attestation.environment_id, "attester environment id")?;
    if attestation.component_id.is_empty() || attestation.reproducibility_file_bytes == 0 {
        return Err(ArtifactError::new(
            "compiler component attestation identity is incomplete",
        ));
    }
    for (label, value) in [
        ("attestation challenge", &attestation.challenge_sha256),
        (
            "reproducibility file",
            &attestation.reproducibility_file_sha256,
        ),
        (
            "reproducibility aggregate",
            &attestation.reproducibility_aggregate_sha256,
        ),
        (
            "first production proof",
            &attestation.first_production_proof_sha256,
        ),
        (
            "second production proof",
            &attestation.second_production_proof_sha256,
        ),
        (
            "candidate reproducible build",
            &attestation.candidate_reproducible_build_sha256,
        ),
        (
            "candidate compiler image",
            &attestation.candidate_compiler_image_sha256,
        ),
        ("native output", &attestation.native_output_sha256),
        ("attestation proof", &attestation.proof_sha256),
    ] {
        validate_sha256(value, label)?;
    }
    if !attestation
        .attester_public_key_id
        .strip_prefix("ed25519:sha256:")
        .is_some_and(is_sha256)
    {
        return Err(ArtifactError::new(
            "compiler attestation public key id is malformed",
        ));
    }
    decode_array::<64>(&attestation.signature_hex, "attestation signature")?;
    if attestation.proof_sha256 != attestation_identity(attestation) {
        return Err(ArtifactError::new(
            "compiler component attestation proof identity mismatch",
        ));
    }
    Ok(())
}

fn attestation_identity(attestation: &CompilerComponentAttestation) -> String {
    let mut hash = Sha256::new();
    for value in [
        attestation.protocol.as_bytes(),
        attestation.authority.as_bytes(),
        attestation.signature_contract.as_bytes(),
        attestation.required_trust_scope.as_bytes(),
        attestation.component_id.as_bytes(),
        attestation.attester_id.as_bytes(),
        attestation.environment_id.as_bytes(),
        attestation.challenge_sha256.as_bytes(),
        attestation.reproducibility_protocol.as_bytes(),
        attestation.reproducibility_file.as_bytes(),
        attestation.reproducibility_file_sha256.as_bytes(),
        attestation.reproducibility_aggregate_sha256.as_bytes(),
        attestation.candidate_production_protocol.as_bytes(),
        attestation.first_production_proof_sha256.as_bytes(),
        attestation.second_production_proof_sha256.as_bytes(),
        attestation.candidate_reproducible_build_sha256.as_bytes(),
        attestation.candidate_compiler_image_sha256.as_bytes(),
        attestation.native_output_sha256.as_bytes(),
        attestation.attester_public_key_id.as_bytes(),
        attestation.verdict.as_bytes(),
    ] {
        hash_field(&mut hash, value);
    }
    for value in [
        attestation.reproducibility_file_bytes,
        attestation.run_count,
        usize::from(attestation.replacement_authorized),
    ] {
        hash_field(&mut hash, &(value as u64).to_le_bytes());
    }
    encode_hex(&hash.finalize())
}

fn signature_message(proof_sha256: &str) -> Vec<u8> {
    let mut message = Vec::with_capacity(
        COMPILER_COMPONENT_ATTESTATION_SIGNATURE_CONTRACT.len() + proof_sha256.len() + 1,
    );
    message.extend_from_slice(COMPILER_COMPONENT_ATTESTATION_SIGNATURE_CONTRACT.as_bytes());
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
            "compiler component attestation `{}` must be UTF-8/LF text",
            path.display()
        )));
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    encode_hex(&Sha256::digest(bytes))
}

#[cfg(test)]
#[path = "compiler_component_attestation_tests.rs"]
mod tests;
