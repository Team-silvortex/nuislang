use std::{fs, path::Path};

use crate::{
    toml::{parse_required_toml_bool, parse_required_toml_string, parse_required_toml_usize},
    ArtifactError,
};

use super::{
    render_compiler_candidate_fresh_source_capability,
    validate_compiler_candidate_fresh_source_capability, CompilerCandidateFreshSourceCapability,
};

pub fn parse_compiler_candidate_fresh_source_capability(
    path: &Path,
) -> Result<CompilerCandidateFreshSourceCapability, ArtifactError> {
    let source = fs::read_to_string(path).map_err(|error| {
        ArtifactError::new(format!(
            "failed to read compiler candidate fresh-source capability `{}`: {error}",
            path.display()
        ))
    })?;
    parse_compiler_candidate_fresh_source_capability_from_source(&source, path)
}

pub fn parse_compiler_candidate_fresh_source_capability_from_source(
    source: &str,
    path: &Path,
) -> Result<CompilerCandidateFreshSourceCapability, ArtifactError> {
    if source.is_empty()
        || source.contains('\r')
        || source.contains('\0')
        || !source.ends_with('\n')
    {
        return Err(ArtifactError::new(format!(
            "compiler candidate fresh-source capability `{}` must be canonical UTF-8/LF text",
            path.display()
        )));
    }
    let string = |key| parse_required_toml_string(source, key, path);
    let number = |key| parse_required_toml_usize(source, key, path);
    let boolean = |key| parse_required_toml_bool(source, key, path);
    let capability = CompilerCandidateFreshSourceCapability {
        protocol: string("protocol")?,
        driver_contract: string("driver_contract")?,
        authority: string("authority")?,
        snapshot_contract: string("snapshot_contract")?,
        abi_contract: string("abi_contract")?,
        input_contract: string("input_contract")?,
        argument_contract: string("argument_contract")?,
        environment_contract: string("environment_contract")?,
        stdin_contract: string("stdin_contract")?,
        native_contract: string("native_contract")?,
        bootstrap_subset_protocol: string("bootstrap_subset_protocol")?,
        component_id: string("component_id")?,
        component_domain: string("component_domain")?,
        component_unit: string("component_unit")?,
        candidate_record_sha256: string("candidate_record_sha256")?,
        candidate_reproducible_build_sha256: string("candidate_reproducible_build_sha256")?,
        candidate_producer_id: string("candidate_producer_id")?,
        candidate_compiler_image_sha256: string("candidate_compiler_image_sha256")?,
        production_protocol: string("production_protocol")?,
        production_proof_sha256: string("production_proof_sha256")?,
        adapter_file: string("adapter_file")?,
        adapter_bytes: number("adapter_bytes")?,
        adapter_sha256: string("adapter_sha256")?,
        predecessor_successor_protocol: string("predecessor_successor_protocol")?,
        predecessor_successor_file: string("predecessor_successor_file")?,
        predecessor_successor_file_bytes: number("predecessor_successor_file_bytes")?,
        predecessor_successor_file_sha256: string("predecessor_successor_file_sha256")?,
        predecessor_successor_proof_sha256: string("predecessor_successor_proof_sha256")?,
        source_bytes: number("source_bytes")?,
        source_lines: number("source_lines")?,
        source_sha256: string("source_sha256")?,
        stage_count: number("stage_count")?,
        token_record_count: number("token_record_count")?,
        ast_record_count: number("ast_record_count")?,
        nir_record_count: number("nir_record_count")?,
        yir_record_count: number("yir_record_count")?,
        token_identity: number("token_identity")?,
        ast_identity: number("ast_identity")?,
        nir_identity: number("nir_identity")?,
        yir_identity: number("yir_identity")?,
        result_protocol: string("result_protocol")?,
        result_file: string("result_file")?,
        result_bytes: number("result_bytes")?,
        result_sha256: string("result_sha256")?,
        result_bundle_fold: number("result_bundle_fold")?,
        exit_code: number("exit_code")?,
        stderr_bytes: number("stderr_bytes")?,
        stderr_sha256: string("stderr_sha256")?,
        stage0_handoff_required: boolean("stage0_handoff_required")?,
        provider_dependency_required: boolean("provider_dependency_required")?,
        candidate_owned_source_processing: boolean("candidate_owned_source_processing")?,
        direct_stage1_compile: boolean("direct_stage1_compile")?,
        fresh_source_compile: boolean("fresh_source_compile")?,
        native_materialization: boolean("native_materialization")?,
        replacement_authorized: boolean("replacement_authorized")?,
        selection_authorized: boolean("selection_authorized")?,
        verdict: string("verdict")?,
        proof_sha256: string("proof_sha256")?,
    };
    validate_compiler_candidate_fresh_source_capability(&capability)?;
    if render_compiler_candidate_fresh_source_capability(&capability) != source {
        return Err(ArtifactError::new(format!(
            "compiler candidate fresh-source capability `{}` is not canonically encoded",
            path.display()
        )));
    }
    Ok(capability)
}
