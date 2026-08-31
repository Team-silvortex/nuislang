use std::{fs, path::Path};

use crate::{
    toml::{parse_required_toml_bool, parse_required_toml_string, parse_required_toml_usize},
    ArtifactError,
};

use super::{
    render_compiler_candidate_direct_compile_capability,
    validate_compiler_candidate_direct_compile_capability,
    CompilerCandidateDirectCompileCapability,
};

pub fn parse_compiler_candidate_direct_compile_capability(
    path: &Path,
) -> Result<CompilerCandidateDirectCompileCapability, ArtifactError> {
    let source = fs::read_to_string(path).map_err(|error| {
        ArtifactError::new(format!(
            "failed to read compiler candidate direct compile capability `{}`: {error}",
            path.display()
        ))
    })?;
    parse_compiler_candidate_direct_compile_capability_from_source(&source, path)
}

pub fn parse_compiler_candidate_direct_compile_capability_from_source(
    source: &str,
    path: &Path,
) -> Result<CompilerCandidateDirectCompileCapability, ArtifactError> {
    if source.is_empty()
        || source.contains('\r')
        || source.contains('\0')
        || !source.ends_with('\n')
    {
        return Err(ArtifactError::new(format!(
            "compiler candidate direct compile capability `{}` must be canonical UTF-8/LF text",
            path.display()
        )));
    }
    let string = |key| parse_required_toml_string(source, key, path);
    let number = |key| parse_required_toml_usize(source, key, path);
    let boolean = |key| parse_required_toml_bool(source, key, path);
    let capability = CompilerCandidateDirectCompileCapability {
        protocol: string("protocol")?,
        driver_contract: string("driver_contract")?,
        authority: string("authority")?,
        request_contract: string("request_contract")?,
        provider_contract: string("provider_contract")?,
        environment_contract: string("environment_contract")?,
        input_contract: string("input_contract")?,
        argument_contract: string("argument_contract")?,
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
        handoff_protocol: string("handoff_protocol")?,
        handoff_bundle_sha256: string("handoff_bundle_sha256")?,
        input_record_count: number("input_record_count")?,
        input_identity_sha256: string("input_identity_sha256")?,
        result_protocol: string("result_protocol")?,
        result_file: string("result_file")?,
        result_bytes: number("result_bytes")?,
        result_sha256: string("result_sha256")?,
        result_bundle_fold: number("result_bundle_fold")?,
        exit_code: number("exit_code")?,
        stderr_bytes: number("stderr_bytes")?,
        stderr_sha256: string("stderr_sha256")?,
        provider_dependency_required: boolean("provider_dependency_required")?,
        direct_stage1_compile: boolean("direct_stage1_compile")?,
        native_materialization: boolean("native_materialization")?,
        replacement_authorized: boolean("replacement_authorized")?,
        selection_authorized: boolean("selection_authorized")?,
        verdict: string("verdict")?,
        proof_sha256: string("proof_sha256")?,
    };
    validate_compiler_candidate_direct_compile_capability(&capability)?;
    if render_compiler_candidate_direct_compile_capability(&capability) != source {
        return Err(ArtifactError::new(format!(
            "compiler candidate direct compile capability `{}` is not canonically encoded",
            path.display()
        )));
    }
    Ok(capability)
}
