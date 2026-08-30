use std::{
    env, fs,
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
};

use nuis_artifact::{
    build_compiler_component_replacement_authorization, parse_compiler_component_attestation,
    parse_compiler_component_replacement_authorization,
    parse_compiler_component_replacement_authorizer_registry,
    parse_compiler_component_reproducibility, render_compiler_component_replacement_authorization,
    verify_compiler_component_attestation, verify_compiler_component_replacement_authorization,
    CompilerComponentReplacementAuthorizationInput, CompilerComponentReplacementVerificationInput,
};

pub(crate) const COMPILER_REPLACEMENT_SIGNING_KEY_ENV: &str =
    "NUIS_COMPILER_REPLACEMENT_SIGNING_KEY_HEX";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BootstrapComponentReplacementInput {
    pub(crate) aggregate: PathBuf,
    pub(crate) attestation: PathBuf,
    pub(crate) attester_registry: PathBuf,
    pub(crate) attester_registry_sha256: String,
    pub(crate) attestation_challenge_sha256: String,
    pub(crate) authorizer_registry: PathBuf,
    pub(crate) authorizer_registry_sha256: String,
    pub(crate) authorization_challenge_sha256: String,
    pub(crate) authorizer_id: String,
    pub(crate) environment_id: String,
    pub(crate) authorization_id: String,
    pub(crate) output: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BootstrapComponentReplacementVerificationInput {
    pub(crate) aggregate: PathBuf,
    pub(crate) attestation: PathBuf,
    pub(crate) attester_registry: PathBuf,
    pub(crate) attester_registry_sha256: String,
    pub(crate) attestation_challenge_sha256: String,
    pub(crate) authorization: PathBuf,
    pub(crate) authorizer_registry: PathBuf,
    pub(crate) authorizer_registry_sha256: String,
    pub(crate) authorization_challenge_sha256: String,
}

pub(crate) fn handle_bootstrap_authorize_component_replacement(
    input: BootstrapComponentReplacementInput,
) -> Result<(), String> {
    let sources = read_sources(
        &input.aggregate,
        &input.attestation,
        &input.attester_registry,
        &input.authorizer_registry,
    )?;
    let parsed = parse_sources(&input)?;
    verify_compiler_component_attestation(
        &parsed.attestation,
        &parsed.reproducibility,
        &sources.reproducibility,
        &parsed.attester_registry,
        &sources.attester_registry,
        &input.attester_registry_sha256,
        &input.attestation_challenge_sha256,
    )
    .map_err(|error| format!("failed to verify authorization attestation: {error}"))?;

    let signing_key = env::var(COMPILER_REPLACEMENT_SIGNING_KEY_ENV).map_err(|_| {
        format!(
            "{COMPILER_REPLACEMENT_SIGNING_KEY_ENV} must contain a 32-byte lowercase hexadecimal Ed25519 signing key"
        )
    })?;
    let authorization = build_compiler_component_replacement_authorization(
        CompilerComponentReplacementAuthorizationInput {
            reproducibility: &parsed.reproducibility,
            reproducibility_source: &sources.reproducibility,
            attestation: &parsed.attestation,
            attestation_source: &sources.attestation,
            challenge_sha256: &input.authorization_challenge_sha256,
            authorization_id: &input.authorization_id,
            authorizer_id: &input.authorizer_id,
            environment_id: &input.environment_id,
        },
        &signing_key,
    )
    .map_err(|error| format!("failed to build compiler replacement authorization: {error}"))?;
    verify_compiler_component_replacement_authorization(
        &authorization,
        verification_input(&input, &sources, &parsed),
    )
    .map_err(|error| format!("failed to self-verify replacement authorization: {error}"))?;
    write_new(
        &input.output,
        render_compiler_component_replacement_authorization(&authorization).as_bytes(),
    )?;

    println!("bootstrap component replacement: authorized");
    println!("  component_id: {}", authorization.component_id);
    println!("  authorization_id: {}", authorization.authorization_id);
    println!("  action: {}", authorization.action);
    println!(
        "  from_reproducible_build_sha256: {}",
        authorization.from_reproducible_build_sha256
    );
    println!(
        "  to_reproducible_build_sha256: {}",
        authorization.to_reproducible_build_sha256
    );
    println!(
        "  rollback_reproducible_build_sha256: {}",
        authorization.rollback_reproducible_build_sha256
    );
    println!("  proof_sha256: {}", authorization.proof_sha256);
    println!("  replacement_authorized: true");
    println!("  authorization: {}", input.output.display());
    Ok(())
}

pub(crate) fn handle_bootstrap_verify_component_replacement(
    input: BootstrapComponentReplacementVerificationInput,
) -> Result<(), String> {
    let sources = read_sources(
        &input.aggregate,
        &input.attestation,
        &input.attester_registry,
        &input.authorizer_registry,
    )?;
    let reproducibility = parse_compiler_component_reproducibility(&input.aggregate)
        .map_err(|error| format!("failed to parse compiler reproducibility aggregate: {error}"))?;
    let attestation = parse_compiler_component_attestation(&input.attestation)
        .map_err(|error| format!("failed to parse compiler attestation: {error}"))?;
    let attester_registry =
        nuis_artifact::parse_compiler_component_attester_trust_registry(&input.attester_registry)
            .map_err(|error| format!("failed to parse compiler attester registry: {error}"))?;
    let authorizer_registry =
        parse_compiler_component_replacement_authorizer_registry(&input.authorizer_registry)
            .map_err(|error| format!("failed to parse replacement authorizer registry: {error}"))?;
    let authorization = parse_compiler_component_replacement_authorization(&input.authorization)
        .map_err(|error| format!("failed to parse compiler replacement authorization: {error}"))?;

    verify_compiler_component_replacement_authorization(
        &authorization,
        CompilerComponentReplacementVerificationInput {
            reproducibility: &reproducibility,
            reproducibility_source: &sources.reproducibility,
            attestation: &attestation,
            attestation_source: &sources.attestation,
            attester_registry: &attester_registry,
            attester_registry_source: &sources.attester_registry,
            expected_attester_registry_sha256: &input.attester_registry_sha256,
            expected_attestation_challenge_sha256: &input.attestation_challenge_sha256,
            authorizer_registry: &authorizer_registry,
            authorizer_registry_source: &sources.authorizer_registry,
            expected_authorizer_registry_sha256: &input.authorizer_registry_sha256,
            expected_authorization_challenge_sha256: &input.authorization_challenge_sha256,
        },
    )
    .map_err(|error| format!("failed to verify compiler replacement authorization: {error}"))?;

    println!("bootstrap component replacement: verified");
    println!("  component_id: {}", authorization.component_id);
    println!("  authorization_id: {}", authorization.authorization_id);
    println!("  authorizer_id: {}", authorization.authorizer_id);
    println!("  action: {}", authorization.action);
    println!("  reversible: true");
    println!("  replacement_authorized: true");
    Ok(())
}

struct Sources {
    reproducibility: String,
    attestation: String,
    attester_registry: String,
    authorizer_registry: String,
}

struct Parsed {
    reproducibility: nuis_artifact::CompilerComponentReproducibility,
    attestation: nuis_artifact::CompilerComponentAttestation,
    attester_registry: nuis_artifact::CompilerComponentAttesterTrustRegistry,
    authorizer_registry: nuis_artifact::CompilerComponentReplacementAuthorizerRegistry,
}

fn read_sources(
    aggregate: &Path,
    attestation: &Path,
    attester_registry: &Path,
    authorizer_registry: &Path,
) -> Result<Sources, String> {
    Ok(Sources {
        reproducibility: read_text(aggregate, "compiler reproducibility aggregate")?,
        attestation: read_text(attestation, "compiler attestation")?,
        attester_registry: read_text(attester_registry, "compiler attester registry")?,
        authorizer_registry: read_text(authorizer_registry, "replacement authorizer registry")?,
    })
}

fn parse_sources(input: &BootstrapComponentReplacementInput) -> Result<Parsed, String> {
    Ok(Parsed {
        reproducibility: parse_compiler_component_reproducibility(&input.aggregate).map_err(
            |error| format!("failed to parse compiler reproducibility aggregate: {error}"),
        )?,
        attestation: parse_compiler_component_attestation(&input.attestation)
            .map_err(|error| format!("failed to parse compiler attestation: {error}"))?,
        attester_registry: nuis_artifact::parse_compiler_component_attester_trust_registry(
            &input.attester_registry,
        )
        .map_err(|error| format!("failed to parse compiler attester registry: {error}"))?,
        authorizer_registry: parse_compiler_component_replacement_authorizer_registry(
            &input.authorizer_registry,
        )
        .map_err(|error| format!("failed to parse replacement authorizer registry: {error}"))?,
    })
}

fn verification_input<'a>(
    input: &'a BootstrapComponentReplacementInput,
    sources: &'a Sources,
    parsed: &'a Parsed,
) -> CompilerComponentReplacementVerificationInput<'a> {
    CompilerComponentReplacementVerificationInput {
        reproducibility: &parsed.reproducibility,
        reproducibility_source: &sources.reproducibility,
        attestation: &parsed.attestation,
        attestation_source: &sources.attestation,
        attester_registry: &parsed.attester_registry,
        attester_registry_source: &sources.attester_registry,
        expected_attester_registry_sha256: &input.attester_registry_sha256,
        expected_attestation_challenge_sha256: &input.attestation_challenge_sha256,
        authorizer_registry: &parsed.authorizer_registry,
        authorizer_registry_source: &sources.authorizer_registry,
        expected_authorizer_registry_sha256: &input.authorizer_registry_sha256,
        expected_authorization_challenge_sha256: &input.authorization_challenge_sha256,
    }
}

fn read_text(path: &Path, label: &str) -> Result<String, String> {
    fs::read_to_string(path)
        .map_err(|error| format!("failed to read {label} `{}`: {error}", path.display()))
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| {
            format!(
                "failed to create compiler replacement authorization `{}` without replacement: {error}",
                path.display()
            )
        })?;
    file.write_all(bytes).map_err(|error| {
        format!(
            "failed to write compiler replacement authorization `{}`: {error}",
            path.display()
        )
    })?;
    file.sync_all().map_err(|error| {
        format!(
            "failed to sync compiler replacement authorization `{}`: {error}",
            path.display()
        )
    })
}
