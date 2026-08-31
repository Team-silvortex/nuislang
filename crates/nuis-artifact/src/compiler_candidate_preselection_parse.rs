use std::{fs, path::Path};

use crate::{
    toml::{parse_required_toml_bool, parse_required_toml_string, parse_required_toml_usize},
    ArtifactError,
};

use super::{
    render_compiler_candidate_preselection, validate_preselection, CompilerCandidatePreselection,
};

pub fn parse_compiler_candidate_preselection(
    path: &Path,
) -> Result<CompilerCandidatePreselection, ArtifactError> {
    let source = fs::read_to_string(path).map_err(|error| {
        ArtifactError::new(format!(
            "failed to read compiler candidate preselection `{}`: {error}",
            path.display()
        ))
    })?;
    parse_compiler_candidate_preselection_from_source(&source, path)
}

pub fn parse_compiler_candidate_preselection_from_source(
    source: &str,
    path: &Path,
) -> Result<CompilerCandidatePreselection, ArtifactError> {
    if source.is_empty()
        || source.contains('\r')
        || source.contains('\0')
        || !source.ends_with('\n')
    {
        return Err(ArtifactError::new(format!(
            "compiler candidate preselection `{}` must be canonical UTF-8/LF text",
            path.display()
        )));
    }
    let string = |key| parse_required_toml_string(source, key, path);
    let number = |key| parse_required_toml_usize(source, key, path);
    let boolean = |key| parse_required_toml_bool(source, key, path);
    let preselection = CompilerCandidatePreselection {
        protocol: string("protocol")?,
        authority: string("authority")?,
        signature_contract: string("signature_contract")?,
        action: string("action")?,
        component_id: string("component_id")?,
        component_domain: string("component_domain")?,
        component_unit: string("component_unit")?,
        preselection_id: string("preselection_id")?,
        target_generation: number("target_generation")?,
        predecessor_transition_protocol: string("predecessor_transition_protocol")?,
        predecessor_transition_file: string("predecessor_transition_file")?,
        predecessor_transition_file_bytes: number("predecessor_transition_file_bytes")?,
        predecessor_transition_file_sha256: string("predecessor_transition_file_sha256")?,
        predecessor_transition_id: string("predecessor_transition_id")?,
        predecessor_transition_generation: number("predecessor_transition_generation")?,
        predecessor_transition_proof_sha256: string("predecessor_transition_proof_sha256")?,
        challenge_sha256: string("challenge_sha256")?,
        current_stage_role: string("current_stage_role")?,
        current_record_sha256: string("current_record_sha256")?,
        current_reproducible_build_sha256: string("current_reproducible_build_sha256")?,
        current_compiler_image_sha256: string("current_compiler_image_sha256")?,
        candidate_stage_role: string("candidate_stage_role")?,
        candidate_record_sha256: string("candidate_record_sha256")?,
        candidate_reproducible_build_sha256: string("candidate_reproducible_build_sha256")?,
        candidate_producer_id: string("candidate_producer_id")?,
        candidate_compiler_image_sha256: string("candidate_compiler_image_sha256")?,
        production_protocol: string("production_protocol")?,
        production_file: string("production_file")?,
        production_file_bytes: number("production_file_bytes")?,
        production_file_sha256: string("production_file_sha256")?,
        production_proof_sha256: string("production_proof_sha256")?,
        compile_capability_protocol: string("compile_capability_protocol")?,
        compile_capability_file: string("compile_capability_file")?,
        compile_capability_file_bytes: number("compile_capability_file_bytes")?,
        compile_capability_file_sha256: string("compile_capability_file_sha256")?,
        compile_capability_proof_sha256: string("compile_capability_proof_sha256")?,
        compile_driver_contract: string("compile_driver_contract")?,
        compile_provider_contract: string("compile_provider_contract")?,
        compiled_artifact_semantic_sha256: string("compiled_artifact_semantic_sha256")?,
        compile_result_record_sha256: string("compile_result_record_sha256")?,
        compile_result_reproducible_build_sha256: string(
            "compile_result_reproducible_build_sha256",
        )?,
        compile_result_native_binary_sha256: string("compile_result_native_binary_sha256")?,
        provider_dependency_contract: string("provider_dependency_contract")?,
        provider_dependency_required: boolean("provider_dependency_required")?,
        direct_stage1_compile: boolean("direct_stage1_compile")?,
        replacement_authorized: boolean("replacement_authorized")?,
        selection_authorized: boolean("selection_authorized")?,
        preselection_authorized: boolean("preselection_authorized")?,
        authorizer_id: string("authorizer_id")?,
        authorizer_environment_id: string("authorizer_environment_id")?,
        authorizer_public_key_id: string("authorizer_public_key_id")?,
        verdict: string("verdict")?,
        proof_sha256: string("proof_sha256")?,
        signature_hex: string("signature_hex")?,
    };
    validate_preselection(&preselection)?;
    if render_compiler_candidate_preselection(&preselection) != source {
        return Err(ArtifactError::new(format!(
            "compiler candidate preselection `{}` is not canonically encoded",
            path.display()
        )));
    }
    Ok(preselection)
}
