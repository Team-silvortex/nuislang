use std::{
    env, fs,
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
};

use nuis_artifact::{
    build_compiler_component_active_state, build_compiler_component_replacement_authorization,
    build_compiler_component_transition, parse_compiler_component_attestation,
    parse_compiler_component_replacement_authorization,
    parse_compiler_component_replacement_authorizer_registry,
    parse_compiler_component_reproducibility, parse_compiler_component_transition,
    render_compiler_component_active_state, render_compiler_component_replacement_authorization,
    render_compiler_component_transition, select_compiler_component_active_target,
    select_compiler_component_transition_target, verify_compiler_component_active_state,
    verify_compiler_component_attestation, verify_compiler_component_replacement_authorization,
    verify_compiler_component_transition, CompilerComponentActiveSelection,
    CompilerComponentActiveState, CompilerComponentReplacementAuthorization,
    CompilerComponentReplacementAuthorizationInput, CompilerComponentReplacementAuthorizerRegistry,
    CompilerComponentReplacementVerificationInput, CompilerComponentTransitionInput,
    CompilerComponentTransitionSelection,
    CompilerComponentTransitionVerificationInput as ArtifactTransitionVerificationInput,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BootstrapComponentActivationInput {
    pub(crate) verification: BootstrapComponentReplacementVerificationInput,
    pub(crate) output: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BootstrapComponentRollbackInput {
    pub(crate) verification: BootstrapComponentReplacementVerificationInput,
    pub(crate) active_state: PathBuf,
    pub(crate) transition_challenge_sha256: String,
    pub(crate) authorizer_id: String,
    pub(crate) environment_id: String,
    pub(crate) transition_id: String,
    pub(crate) output: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BootstrapComponentTransitionVerificationInput {
    pub(crate) verification: BootstrapComponentReplacementVerificationInput,
    pub(crate) active_state: PathBuf,
    pub(crate) transition: PathBuf,
    pub(crate) transition_challenge_sha256: String,
}

pub(crate) struct VerifiedComponentTransition {
    transition: nuis_artifact::CompilerComponentTransition,
    predecessor: VerifiedTransitionPredecessor,
}

impl VerifiedComponentTransition {
    pub(crate) fn transition(&self) -> &nuis_artifact::CompilerComponentTransition {
        &self.transition
    }

    pub(crate) fn verification_input<'a>(
        &'a self,
        input: &'a BootstrapComponentTransitionVerificationInput,
    ) -> ArtifactTransitionVerificationInput<'a> {
        transition_verification_input(
            &self.predecessor,
            &input.verification,
            &input.transition_challenge_sha256,
        )
    }
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
        "compiler replacement authorization",
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
    let (authorization, _) = verify_replacement_input(&input)?;

    println!("bootstrap component replacement: verified");
    println!("  component_id: {}", authorization.component_id);
    println!("  authorization_id: {}", authorization.authorization_id);
    println!("  authorizer_id: {}", authorization.authorizer_id);
    println!("  action: {}", authorization.action);
    println!("  reversible: true");
    println!("  replacement_authorized: true");
    Ok(())
}

pub(crate) fn handle_bootstrap_activate_component(
    input: BootstrapComponentActivationInput,
) -> Result<(), String> {
    let (authorization, authorization_source) = verify_replacement_input(&input.verification)?;
    let state = build_compiler_component_active_state(&authorization, &authorization_source)
        .map_err(|error| format!("failed to build compiler active-component state: {error}"))?;
    verify_compiler_component_active_state(&state, &authorization, &authorization_source).map_err(
        |error| format!("failed to self-verify compiler active-component state: {error}"),
    )?;
    let active = select_compiler_component_active_target(
        &state,
        &authorization,
        &authorization_source,
        CompilerComponentActiveSelection::Active,
    )
    .map_err(|error| format!("failed to select active compiler component: {error}"))?;
    let rollback = select_compiler_component_active_target(
        &state,
        &authorization,
        &authorization_source,
        CompilerComponentActiveSelection::Rollback,
    )
    .map_err(|error| format!("failed to select rollback compiler component: {error}"))?;
    write_new(
        &input.output,
        render_compiler_component_active_state(&state).as_bytes(),
        "compiler active-component state",
    )?;

    println!("bootstrap compiler component: activated");
    println!("  component_id: {}", state.component_id);
    println!("  state_sha256: {}", state.state_sha256);
    println!("  active_stage_role: {}", active.stage_role);
    println!(
        "  active_reproducible_build_sha256: {}",
        active.reproducible_build_sha256
    );
    println!("  rollback_stage_role: {}", rollback.stage_role);
    println!(
        "  rollback_reproducible_build_sha256: {}",
        rollback.reproducible_build_sha256
    );
    println!("  reversible: true");
    println!("  active_state: {}", input.output.display());
    Ok(())
}

pub(crate) fn handle_bootstrap_rollback_component(
    input: BootstrapComponentRollbackInput,
) -> Result<(), String> {
    let predecessor = verify_transition_predecessors(&input.verification, &input.active_state)?;
    let signing_key = env::var(COMPILER_REPLACEMENT_SIGNING_KEY_ENV).map_err(|_| {
        format!(
            "{COMPILER_REPLACEMENT_SIGNING_KEY_ENV} must contain a 32-byte lowercase hexadecimal Ed25519 signing key"
        )
    })?;
    let transition = build_compiler_component_transition(
        CompilerComponentTransitionInput {
            authorization: &predecessor.authorization,
            authorization_source: &predecessor.authorization_source,
            active_state: &predecessor.active_state,
            active_state_source: &predecessor.active_state_source,
            challenge_sha256: &input.transition_challenge_sha256,
            transition_id: &input.transition_id,
            authorizer_id: &input.authorizer_id,
            environment_id: &input.environment_id,
        },
        &signing_key,
    )
    .map_err(|error| format!("failed to build compiler rollback transition: {error}"))?;
    let verification = transition_verification_input(
        &predecessor,
        &input.verification,
        &input.transition_challenge_sha256,
    );
    verify_compiler_component_transition(&transition, verification)
        .map_err(|error| format!("failed to self-verify compiler rollback transition: {error}"))?;
    let current = select_compiler_component_transition_target(
        &transition,
        verification,
        CompilerComponentTransitionSelection::Current,
    )
    .map_err(|error| format!("failed to select restored stage0 component: {error}"))?;
    let forward = select_compiler_component_transition_target(
        &transition,
        verification,
        CompilerComponentTransitionSelection::Forward,
    )
    .map_err(|error| format!("failed to select retained forward component: {error}"))?;
    write_new(
        &input.output,
        render_compiler_component_transition(&transition).as_bytes(),
        "compiler component transition",
    )?;

    println!("bootstrap compiler component: rolled back");
    println!("  component_id: {}", transition.component_id);
    println!("  transition_id: {}", transition.transition_id);
    println!("  generation: {}", transition.generation);
    println!("  proof_sha256: {}", transition.proof_sha256);
    println!("  current_stage_role: {}", current.stage_role);
    println!(
        "  current_reproducible_build_sha256: {}",
        current.reproducible_build_sha256
    );
    println!("  forward_stage_role: {}", forward.stage_role);
    println!(
        "  forward_reproducible_build_sha256: {}",
        forward.reproducible_build_sha256
    );
    println!("  reversible: true");
    println!("  transition: {}", input.output.display());
    Ok(())
}

pub(crate) fn handle_bootstrap_verify_component_transition(
    input: BootstrapComponentTransitionVerificationInput,
) -> Result<(), String> {
    let verified = load_verified_component_transition(&input)?;
    let transition = verified.transition();
    let verification = verified.verification_input(&input);
    let current = select_compiler_component_transition_target(
        transition,
        verification,
        CompilerComponentTransitionSelection::Current,
    )
    .map_err(|error| format!("failed to select restored stage0 component: {error}"))?;
    let forward = select_compiler_component_transition_target(
        transition,
        verification,
        CompilerComponentTransitionSelection::Forward,
    )
    .map_err(|error| format!("failed to select retained forward component: {error}"))?;

    println!("bootstrap compiler component transition: verified");
    println!("  component_id: {}", transition.component_id);
    println!("  transition_id: {}", transition.transition_id);
    println!("  generation: {}", transition.generation);
    println!("  current_stage_role: {}", current.stage_role);
    println!(
        "  current_reproducible_build_sha256: {}",
        current.reproducible_build_sha256
    );
    println!("  forward_stage_role: {}", forward.stage_role);
    println!(
        "  forward_reproducible_build_sha256: {}",
        forward.reproducible_build_sha256
    );
    println!("  reversible: true");
    Ok(())
}

pub(crate) fn load_verified_component_transition(
    input: &BootstrapComponentTransitionVerificationInput,
) -> Result<VerifiedComponentTransition, String> {
    let predecessor = verify_transition_predecessors(&input.verification, &input.active_state)?;
    let transition = parse_compiler_component_transition(&input.transition)
        .map_err(|error| format!("failed to parse compiler component transition: {error}"))?;
    verify_compiler_component_transition(
        &transition,
        transition_verification_input(
            &predecessor,
            &input.verification,
            &input.transition_challenge_sha256,
        ),
    )
    .map_err(|error| format!("failed to verify compiler component transition: {error}"))?;
    Ok(VerifiedComponentTransition {
        transition,
        predecessor,
    })
}

struct VerifiedTransitionPredecessor {
    authorization: CompilerComponentReplacementAuthorization,
    authorization_source: String,
    active_state: CompilerComponentActiveState,
    active_state_source: String,
    authorizer_registry: CompilerComponentReplacementAuthorizerRegistry,
    authorizer_registry_source: String,
}

fn verify_transition_predecessors(
    verification: &BootstrapComponentReplacementVerificationInput,
    active_state_path: &Path,
) -> Result<VerifiedTransitionPredecessor, String> {
    let (authorization, authorization_source) = verify_replacement_input(verification)?;
    let active_state = nuis_artifact::parse_compiler_component_active_state(active_state_path)
        .map_err(|error| format!("failed to parse compiler active-component state: {error}"))?;
    let active_state_source = read_text(active_state_path, "compiler active-component state")?;
    verify_compiler_component_active_state(&active_state, &authorization, &authorization_source)
        .map_err(|error| format!("failed to verify compiler active-component state: {error}"))?;
    let authorizer_registry =
        parse_compiler_component_replacement_authorizer_registry(&verification.authorizer_registry)
            .map_err(|error| format!("failed to parse replacement authorizer registry: {error}"))?;
    let authorizer_registry_source = read_text(
        &verification.authorizer_registry,
        "replacement authorizer registry",
    )?;
    Ok(VerifiedTransitionPredecessor {
        authorization,
        authorization_source,
        active_state,
        active_state_source,
        authorizer_registry,
        authorizer_registry_source,
    })
}

fn transition_verification_input<'a>(
    predecessor: &'a VerifiedTransitionPredecessor,
    verification: &'a BootstrapComponentReplacementVerificationInput,
    challenge_sha256: &'a str,
) -> ArtifactTransitionVerificationInput<'a> {
    ArtifactTransitionVerificationInput {
        authorization: &predecessor.authorization,
        authorization_source: &predecessor.authorization_source,
        active_state: &predecessor.active_state,
        active_state_source: &predecessor.active_state_source,
        authorizer_registry: &predecessor.authorizer_registry,
        authorizer_registry_source: &predecessor.authorizer_registry_source,
        expected_authorizer_registry_sha256: &verification.authorizer_registry_sha256,
        expected_transition_challenge_sha256: challenge_sha256,
    }
}

fn verify_replacement_input(
    input: &BootstrapComponentReplacementVerificationInput,
) -> Result<(CompilerComponentReplacementAuthorization, String), String> {
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
    let authorization_source =
        read_text(&input.authorization, "compiler replacement authorization")?;

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
    Ok((authorization, authorization_source))
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

fn write_new(path: &Path, bytes: &[u8], label: &str) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| {
            format!(
                "failed to create {label} `{}` without replacement: {error}",
                path.display()
            )
        })?;
    file.write_all(bytes)
        .map_err(|error| format!("failed to write {label} `{}`: {error}", path.display()))?;
    file.sync_all()
        .map_err(|error| format!("failed to sync {label} `{}`: {error}", path.display()))
}
