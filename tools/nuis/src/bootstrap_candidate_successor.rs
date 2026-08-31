use std::{env, fs, path::PathBuf};

use nuis_artifact::{
    build_compiler_candidate_successor, parse_compiler_candidate_compile_capability_from_source,
    parse_compiler_candidate_direct_compile_capability_from_source,
    parse_compiler_candidate_preselection_from_source, parse_compiler_candidate_successor,
    read_compiler_stage_handoff, read_compiler_stage_transformations,
    render_compiler_candidate_successor, verify_compiler_candidate_successor,
    CompilerCandidateDirectCompileCapabilityInput, CompilerCandidatePreselectionVerificationInput,
    CompilerCandidateSuccessorInput, CompilerCandidateSuccessorVerificationInput,
    COMPILER_STAGE_TRANSFORMATION_FILE,
};

use crate::{
    bootstrap_candidate_compile_capability::load_verified_candidate_compile_lineage,
    bootstrap_component_image::write_new,
    bootstrap_component_replacement::{
        load_verified_component_transition, BootstrapComponentTransitionVerificationInput,
        COMPILER_REPLACEMENT_SIGNING_KEY_ENV,
    },
};

const CANDIDATE_DIR: &str = "stage1-candidate";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BootstrapCandidateSuccessorInput {
    pub(crate) transition_verification: BootstrapComponentTransitionVerificationInput,
    pub(crate) candidate_root: PathBuf,
    pub(crate) delegated_capability: PathBuf,
    pub(crate) preselection: PathBuf,
    pub(crate) preselection_challenge_sha256: String,
    pub(crate) direct_capability: PathBuf,
    pub(crate) frontend_result: PathBuf,
    pub(crate) challenge_sha256: String,
    pub(crate) authorizer_id: String,
    pub(crate) environment_id: String,
    pub(crate) successor_id: String,
    pub(crate) output: PathBuf,
}

pub(crate) fn handle_bootstrap_sign_candidate_successor(
    input: BootstrapCandidateSuccessorInput,
) -> Result<(), String> {
    if input.output.exists() {
        return Err(format!(
            "compiler candidate successor `{}` already exists",
            input.output.display()
        ));
    }
    let verified_transition = load_verified_component_transition(&input.transition_verification)?;
    let lineage = load_verified_candidate_compile_lineage(&input.candidate_root)?;
    let candidate_dir = input.candidate_root.join(CANDIDATE_DIR);
    let (handoff, payloads) =
        read_compiler_stage_handoff(&candidate_dir.join(&lineage.candidate.stage_handoff_file))
            .map_err(|error| format!("failed to verify successor candidate handoff: {error}"))?;
    let transformations = read_compiler_stage_transformations(
        &candidate_dir.join(COMPILER_STAGE_TRANSFORMATION_FILE),
        &handoff,
        &payloads,
    )
    .map_err(|error| format!("failed to verify successor transformations: {error}"))?;

    let transition_source = read_text(
        &input.transition_verification.transition,
        "generation-two transition",
    )?;
    let delegated_capability_source = read_text(
        &input.delegated_capability,
        "delegated candidate compile capability",
    )?;
    let delegated_capability = parse_compiler_candidate_compile_capability_from_source(
        &delegated_capability_source,
        &input.delegated_capability,
    )
    .map_err(|error| format!("failed to parse delegated compile capability: {error}"))?;
    let preselection_source = read_text(&input.preselection, "candidate preselection")?;
    let preselection = parse_compiler_candidate_preselection_from_source(
        &preselection_source,
        &input.preselection,
    )
    .map_err(|error| format!("failed to parse candidate preselection: {error}"))?;
    let direct_capability_source = read_text(
        &input.direct_capability,
        "candidate direct compile capability",
    )?;
    let direct_capability = parse_compiler_candidate_direct_compile_capability_from_source(
        &direct_capability_source,
        &input.direct_capability,
    )
    .map_err(|error| format!("failed to parse direct compile capability: {error}"))?;
    let frontend_result = fs::read(&input.frontend_result).map_err(|error| {
        format!(
            "failed to read candidate front-end result `{}`: {error}",
            input.frontend_result.display()
        )
    })?;
    let signing_key = env::var(COMPILER_REPLACEMENT_SIGNING_KEY_ENV).map_err(|_| {
        format!(
            "{COMPILER_REPLACEMENT_SIGNING_KEY_ENV} must contain the continuing generation-three component-owner Ed25519 signing key"
        )
    })?;

    let transition = verified_transition.transition();
    let transition_verification =
        verified_transition.verification_input(&input.transition_verification);
    let preselection_verification = CompilerCandidatePreselectionVerificationInput {
        transition,
        transition_source: &transition_source,
        transition_verification,
        stage0: &lineage.stage0,
        candidate: &lineage.candidate,
        production: &lineage.production,
        production_source: &lineage.production_source,
        capability: &delegated_capability,
        capability_source: &delegated_capability_source,
        expected_challenge_sha256: &input.preselection_challenge_sha256,
    };
    let direct_compile_verification = CompilerCandidateDirectCompileCapabilityInput {
        candidate: &lineage.candidate,
        production: &lineage.production,
        adapter: &lineage.adapter,
        handoff: &handoff,
        payloads: &payloads,
        transformations: &transformations,
        result: &frontend_result,
        exit_code: 0,
        stderr: &[],
    };
    let successor = build_compiler_candidate_successor(
        CompilerCandidateSuccessorInput {
            preselection: &preselection,
            preselection_source: &preselection_source,
            preselection_verification,
            direct_capability: &direct_capability,
            direct_capability_source: &direct_capability_source,
            direct_compile_verification,
            frontend_result_source: &frontend_result,
            challenge_sha256: &input.challenge_sha256,
            successor_id: &input.successor_id,
            authorizer_id: &input.authorizer_id,
            environment_id: &input.environment_id,
        },
        &signing_key,
    )
    .map_err(|error| format!("failed to build compiler candidate successor: {error}"))?;
    verify_compiler_candidate_successor(
        &successor,
        CompilerCandidateSuccessorVerificationInput {
            preselection: &preselection,
            preselection_source: &preselection_source,
            preselection_verification,
            direct_capability: &direct_capability,
            direct_capability_source: &direct_capability_source,
            direct_compile_verification,
            frontend_result_source: &frontend_result,
            expected_challenge_sha256: &input.challenge_sha256,
        },
    )
    .map_err(|error| format!("failed to self-verify candidate successor: {error}"))?;
    write_new(
        &input.output,
        render_compiler_candidate_successor(&successor).as_bytes(),
        "compiler candidate successor",
    )?;
    let persisted = parse_compiler_candidate_successor(&input.output)
        .map_err(|error| format!("failed to reread candidate successor: {error}"))?;
    if persisted != successor {
        return Err("compiler candidate successor changed after persistence".to_owned());
    }

    println!("bootstrap compiler candidate: successor signed");
    println!("  component_id: {}", successor.component_id);
    println!("  target_generation: {}", successor.target_generation);
    println!(
        "  predecessor_preselection_proof_sha256: {}",
        successor.predecessor_preselection_proof_sha256
    );
    println!(
        "  direct_compile_capability_proof_sha256: {}",
        successor.direct_compile_capability_proof_sha256
    );
    println!("  provider_dependency_required: false");
    println!("  direct_stage1_compile: true");
    println!("  fresh_source_compile: false");
    println!("  native_materialization: false");
    println!("  replacement_authorized: false");
    println!("  selection_authorized: false");
    println!("  successor_authorized: true");
    println!("  successor: {}", input.output.display());
    Ok(())
}

fn read_text(path: &std::path::Path, label: &str) -> Result<String, String> {
    fs::read_to_string(path)
        .map_err(|error| format!("failed to read {label} `{}`: {error}", path.display()))
}
