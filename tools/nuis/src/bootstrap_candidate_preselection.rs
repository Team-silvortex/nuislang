use std::{env, fs, path::PathBuf};

use nuis_artifact::{
    build_compiler_candidate_preselection, parse_compiler_candidate_compile_capability_from_source,
    parse_compiler_candidate_preselection, render_compiler_candidate_preselection,
    verify_compiler_candidate_preselection, CompilerCandidatePreselectionInput,
    CompilerCandidatePreselectionVerificationInput,
};

use crate::{
    bootstrap_candidate_compile_capability::load_verified_candidate_compile_lineage,
    bootstrap_component_image::write_new,
    bootstrap_component_replacement::{
        load_verified_component_transition, BootstrapComponentTransitionVerificationInput,
        COMPILER_REPLACEMENT_SIGNING_KEY_ENV,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BootstrapCandidatePreselectionInput {
    pub(crate) transition_verification: BootstrapComponentTransitionVerificationInput,
    pub(crate) candidate_root: PathBuf,
    pub(crate) capability: PathBuf,
    pub(crate) challenge_sha256: String,
    pub(crate) authorizer_id: String,
    pub(crate) environment_id: String,
    pub(crate) preselection_id: String,
    pub(crate) output: PathBuf,
}

pub(crate) fn handle_bootstrap_preselect_candidate(
    input: BootstrapCandidatePreselectionInput,
) -> Result<(), String> {
    if input.output.exists() {
        return Err(format!(
            "compiler candidate preselection `{}` already exists",
            input.output.display()
        ));
    }
    let verified_transition = load_verified_component_transition(&input.transition_verification)?;
    let lineage = load_verified_candidate_compile_lineage(&input.candidate_root)?;
    let transition_source = fs::read_to_string(&input.transition_verification.transition)
        .map_err(|error| format!("failed to read verified generation-two transition: {error}"))?;
    let capability_source = fs::read_to_string(&input.capability).map_err(|error| {
        format!(
            "failed to read candidate compile capability `{}`: {error}",
            input.capability.display()
        )
    })?;
    let capability = parse_compiler_candidate_compile_capability_from_source(
        &capability_source,
        &input.capability,
    )
    .map_err(|error| format!("failed to parse candidate compile capability: {error}"))?;
    let signing_key = env::var(COMPILER_REPLACEMENT_SIGNING_KEY_ENV).map_err(|_| {
        format!(
            "{COMPILER_REPLACEMENT_SIGNING_KEY_ENV} must contain the generation-two component-owner Ed25519 signing key"
        )
    })?;
    let transition = verified_transition.transition();
    let transition_verification =
        verified_transition.verification_input(&input.transition_verification);
    let preselection = build_compiler_candidate_preselection(
        CompilerCandidatePreselectionInput {
            transition,
            transition_source: &transition_source,
            transition_verification,
            stage0: &lineage.stage0,
            candidate: &lineage.candidate,
            production: &lineage.production,
            production_source: &lineage.production_source,
            capability: &capability,
            capability_source: &capability_source,
            challenge_sha256: &input.challenge_sha256,
            preselection_id: &input.preselection_id,
            authorizer_id: &input.authorizer_id,
            environment_id: &input.environment_id,
        },
        &signing_key,
    )
    .map_err(|error| format!("failed to build compiler candidate preselection: {error}"))?;
    verify_compiler_candidate_preselection(
        &preselection,
        CompilerCandidatePreselectionVerificationInput {
            transition,
            transition_source: &transition_source,
            transition_verification,
            stage0: &lineage.stage0,
            candidate: &lineage.candidate,
            production: &lineage.production,
            production_source: &lineage.production_source,
            capability: &capability,
            capability_source: &capability_source,
            expected_challenge_sha256: &input.challenge_sha256,
        },
    )
    .map_err(|error| format!("failed to self-verify candidate preselection: {error}"))?;
    write_new(
        &input.output,
        render_compiler_candidate_preselection(&preselection).as_bytes(),
        "compiler candidate preselection",
    )?;
    let parsed = parse_compiler_candidate_preselection(&input.output)
        .map_err(|error| format!("failed to reread candidate preselection: {error}"))?;
    if parsed != preselection {
        return Err("compiler candidate preselection changed after persistence".to_owned());
    }

    println!("bootstrap compiler candidate: preselected");
    println!("  component_id: {}", preselection.component_id);
    println!("  target_generation: {}", preselection.target_generation);
    println!(
        "  predecessor_transition_proof_sha256: {}",
        preselection.predecessor_transition_proof_sha256
    );
    println!(
        "  compile_capability_proof_sha256: {}",
        preselection.compile_capability_proof_sha256
    );
    println!("  provider_dependency_required: true");
    println!("  direct_stage1_compile: false");
    println!("  replacement_authorized: false");
    println!("  selection_authorized: false");
    println!("  preselection_authorized: true");
    println!("  preselection: {}", input.output.display());
    Ok(())
}
