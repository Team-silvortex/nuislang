use std::{
    env, fs,
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
};

use nuis_artifact::{
    build_compiler_component_attestation, read_compiler_component_attestation,
    read_compiler_component_reproducibility, render_compiler_component_attestation,
    CompilerComponentAttestationInput,
};

pub(crate) const COMPILER_ATTESTER_SIGNING_KEY_ENV: &str = "NUIS_COMPILER_ATTESTER_SIGNING_KEY_HEX";

pub(crate) struct BootstrapAttestationInput {
    pub(crate) aggregate: PathBuf,
    pub(crate) first_root: PathBuf,
    pub(crate) second_root: PathBuf,
    pub(crate) challenge_sha256: String,
    pub(crate) attester_id: String,
    pub(crate) environment_id: String,
    pub(crate) output: PathBuf,
}

pub(crate) struct BootstrapAttestationVerificationInput {
    pub(crate) aggregate: PathBuf,
    pub(crate) attestation: PathBuf,
    pub(crate) trust_registry: PathBuf,
    pub(crate) registry_sha256: String,
    pub(crate) challenge_sha256: String,
}

pub(crate) fn handle_bootstrap_attest_reproducibility(
    input: BootstrapAttestationInput,
) -> Result<(), String> {
    let roots = vec![input.first_root, input.second_root];
    let report = read_compiler_component_reproducibility(&input.aggregate, &roots)
        .map_err(|error| format!("failed to verify attested clean build roots: {error}"))?;
    let report_source = fs::read_to_string(&input.aggregate).map_err(|error| {
        format!(
            "failed to read compiler reproducibility aggregate `{}`: {error}",
            input.aggregate.display()
        )
    })?;
    let signing_key = env::var(COMPILER_ATTESTER_SIGNING_KEY_ENV).map_err(|_| {
        format!(
            "{COMPILER_ATTESTER_SIGNING_KEY_ENV} must contain a 32-byte lowercase hexadecimal Ed25519 signing key"
        )
    })?;
    let attestation = build_compiler_component_attestation(
        CompilerComponentAttestationInput {
            reproducibility: &report,
            reproducibility_source: &report_source,
            challenge_sha256: &input.challenge_sha256,
            attester_id: &input.attester_id,
            environment_id: &input.environment_id,
        },
        &signing_key,
    )
    .map_err(|error| format!("failed to build compiler attestation: {error}"))?;
    write_new(
        &input.output,
        render_compiler_component_attestation(&attestation).as_bytes(),
    )?;

    println!("bootstrap compiler attestation: signed");
    println!("  attester_id: {}", attestation.attester_id);
    println!("  environment_id: {}", attestation.environment_id);
    println!("  public_key_id: {}", attestation.attester_public_key_id);
    println!("  proof_sha256: {}", attestation.proof_sha256);
    println!("  replacement_authorized: false");
    println!("  attestation: {}", input.output.display());
    Ok(())
}

pub(crate) fn handle_bootstrap_verify_reproducibility_attestation(
    input: BootstrapAttestationVerificationInput,
) -> Result<(), String> {
    let attestation = read_compiler_component_attestation(
        &input.attestation,
        &input.aggregate,
        &input.trust_registry,
        &input.registry_sha256,
        &input.challenge_sha256,
    )
    .map_err(|error| format!("failed to verify compiler attestation: {error}"))?;

    println!("bootstrap compiler attestation: verified");
    println!("  trust_scope: {}", attestation.required_trust_scope);
    println!("  attester_id: {}", attestation.attester_id);
    println!("  environment_id: {}", attestation.environment_id);
    println!(
        "  candidate_production: {}",
        attestation.candidate_production_protocol
    );
    println!("  proof_sha256: {}", attestation.proof_sha256);
    println!("  replacement_authorized: false");
    Ok(())
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| {
            format!(
                "failed to create compiler attestation `{}` without replacement: {error}",
                path.display()
            )
        })?;
    file.write_all(bytes).map_err(|error| {
        format!(
            "failed to write compiler attestation `{}`: {error}",
            path.display()
        )
    })?;
    file.sync_all().map_err(|error| {
        format!(
            "failed to sync compiler attestation `{}`: {error}",
            path.display()
        )
    })
}
