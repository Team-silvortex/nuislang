use std::{fs, path::Path};

use crate::{
    toml::{parse_required_toml_bool, parse_required_toml_string, parse_required_toml_usize},
    ArtifactError,
};

use super::{render_compiler_candidate_successor, validate_successor, CompilerCandidateSuccessor};

pub fn parse_compiler_candidate_successor(
    path: &Path,
) -> Result<CompilerCandidateSuccessor, ArtifactError> {
    let source = fs::read_to_string(path).map_err(|error| {
        ArtifactError::new(format!(
            "failed to read compiler candidate successor `{}`: {error}",
            path.display()
        ))
    })?;
    parse_compiler_candidate_successor_from_source(&source, path)
}

pub fn parse_compiler_candidate_successor_from_source(
    source: &str,
    path: &Path,
) -> Result<CompilerCandidateSuccessor, ArtifactError> {
    if source.is_empty()
        || source.contains('\r')
        || source.contains('\0')
        || !source.ends_with('\n')
    {
        return Err(ArtifactError::new(format!(
            "compiler candidate successor `{}` must be canonical UTF-8/LF text",
            path.display()
        )));
    }
    let string = |key| parse_required_toml_string(source, key, path);
    let number = |key| parse_required_toml_usize(source, key, path);
    let boolean = |key| parse_required_toml_bool(source, key, path);
    let successor = CompilerCandidateSuccessor {
        protocol: string("protocol")?,
        authority: string("authority")?,
        signature_contract: string("signature_contract")?,
        action: string("action")?,
        relation_contract: string("relation_contract")?,
        component_id: string("component_id")?,
        component_domain: string("component_domain")?,
        component_unit: string("component_unit")?,
        successor_id: string("successor_id")?,
        target_generation: number("target_generation")?,
        predecessor_preselection_protocol: string("predecessor_preselection_protocol")?,
        predecessor_preselection_file: string("predecessor_preselection_file")?,
        predecessor_preselection_file_bytes: number("predecessor_preselection_file_bytes")?,
        predecessor_preselection_file_sha256: string("predecessor_preselection_file_sha256")?,
        predecessor_preselection_id: string("predecessor_preselection_id")?,
        predecessor_preselection_proof_sha256: string("predecessor_preselection_proof_sha256")?,
        challenge_sha256: string("challenge_sha256")?,
        candidate_stage_role: string("candidate_stage_role")?,
        candidate_record_sha256: string("candidate_record_sha256")?,
        candidate_reproducible_build_sha256: string("candidate_reproducible_build_sha256")?,
        candidate_producer_id: string("candidate_producer_id")?,
        candidate_compiler_image_sha256: string("candidate_compiler_image_sha256")?,
        production_protocol: string("production_protocol")?,
        production_proof_sha256: string("production_proof_sha256")?,
        direct_compile_capability_protocol: string("direct_compile_capability_protocol")?,
        direct_compile_capability_file: string("direct_compile_capability_file")?,
        direct_compile_capability_file_bytes: number("direct_compile_capability_file_bytes")?,
        direct_compile_capability_file_sha256: string("direct_compile_capability_file_sha256")?,
        direct_compile_capability_proof_sha256: string("direct_compile_capability_proof_sha256")?,
        direct_compile_driver_contract: string("direct_compile_driver_contract")?,
        direct_compile_provider_contract: string("direct_compile_provider_contract")?,
        direct_compile_input_identity_sha256: string("direct_compile_input_identity_sha256")?,
        frontend_result_protocol: string("frontend_result_protocol")?,
        frontend_result_file: string("frontend_result_file")?,
        frontend_result_bytes: number("frontend_result_bytes")?,
        frontend_result_sha256: string("frontend_result_sha256")?,
        frontend_result_bundle_fold: number("frontend_result_bundle_fold")?,
        provider_dependency_required: boolean("provider_dependency_required")?,
        direct_stage1_compile: boolean("direct_stage1_compile")?,
        fresh_source_compile: boolean("fresh_source_compile")?,
        native_materialization: boolean("native_materialization")?,
        replacement_authorized: boolean("replacement_authorized")?,
        selection_authorized: boolean("selection_authorized")?,
        preselection_authorized: boolean("preselection_authorized")?,
        successor_authorized: boolean("successor_authorized")?,
        authorizer_id: string("authorizer_id")?,
        authorizer_environment_id: string("authorizer_environment_id")?,
        authorizer_public_key_id: string("authorizer_public_key_id")?,
        verdict: string("verdict")?,
        proof_sha256: string("proof_sha256")?,
        signature_hex: string("signature_hex")?,
    };
    validate_successor(&successor)?;
    if render_compiler_candidate_successor(&successor) != source {
        return Err(ArtifactError::new(format!(
            "compiler candidate successor `{}` is not canonically encoded",
            path.display()
        )));
    }
    Ok(successor)
}
