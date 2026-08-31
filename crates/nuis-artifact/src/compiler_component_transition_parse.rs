use std::{fs, path::Path};

use crate::{
    toml::{parse_required_toml_bool, parse_required_toml_string, parse_required_toml_usize},
    ArtifactError,
};

use super::{
    render_compiler_component_transition, validate_transition, CompilerComponentTransition,
};

pub fn parse_compiler_component_transition(
    path: &Path,
) -> Result<CompilerComponentTransition, ArtifactError> {
    let source = fs::read_to_string(path).map_err(|error| {
        ArtifactError::new(format!(
            "failed to read compiler component transition `{}`: {error}",
            path.display()
        ))
    })?;
    parse_compiler_component_transition_from_source(&source, path)
}

pub fn parse_compiler_component_transition_from_source(
    source: &str,
    path: &Path,
) -> Result<CompilerComponentTransition, ArtifactError> {
    validate_text(source, path)?;
    let transition = CompilerComponentTransition {
        protocol: parse_required_toml_string(source, "protocol", path)?,
        authority: parse_required_toml_string(source, "authority", path)?,
        signature_contract: parse_required_toml_string(source, "signature_contract", path)?,
        action: parse_required_toml_string(source, "action", path)?,
        component_id: parse_required_toml_string(source, "component_id", path)?,
        transition_id: parse_required_toml_string(source, "transition_id", path)?,
        generation: parse_required_toml_usize(source, "generation", path)?,
        predecessor_authorization_protocol: parse_required_toml_string(
            source,
            "predecessor_authorization_protocol",
            path,
        )?,
        predecessor_authorization_file: parse_required_toml_string(
            source,
            "predecessor_authorization_file",
            path,
        )?,
        predecessor_authorization_file_bytes: parse_required_toml_usize(
            source,
            "predecessor_authorization_file_bytes",
            path,
        )?,
        predecessor_authorization_file_sha256: parse_required_toml_string(
            source,
            "predecessor_authorization_file_sha256",
            path,
        )?,
        predecessor_authorization_id: parse_required_toml_string(
            source,
            "predecessor_authorization_id",
            path,
        )?,
        predecessor_authorization_generation: parse_required_toml_usize(
            source,
            "predecessor_authorization_generation",
            path,
        )?,
        predecessor_authorization_proof_sha256: parse_required_toml_string(
            source,
            "predecessor_authorization_proof_sha256",
            path,
        )?,
        predecessor_state_protocol: parse_required_toml_string(
            source,
            "predecessor_state_protocol",
            path,
        )?,
        predecessor_state_file: parse_required_toml_string(source, "predecessor_state_file", path)?,
        predecessor_state_file_bytes: parse_required_toml_usize(
            source,
            "predecessor_state_file_bytes",
            path,
        )?,
        predecessor_state_file_sha256: parse_required_toml_string(
            source,
            "predecessor_state_file_sha256",
            path,
        )?,
        predecessor_state_generation: parse_required_toml_usize(
            source,
            "predecessor_state_generation",
            path,
        )?,
        predecessor_state_sha256: parse_required_toml_string(
            source,
            "predecessor_state_sha256",
            path,
        )?,
        challenge_sha256: parse_required_toml_string(source, "challenge_sha256", path)?,
        from_selector: parse_required_toml_string(source, "from_selector", path)?,
        from_stage_role: parse_required_toml_string(source, "from_stage_role", path)?,
        from_reproducible_build_sha256: parse_required_toml_string(
            source,
            "from_reproducible_build_sha256",
            path,
        )?,
        current_selector: parse_required_toml_string(source, "current_selector", path)?,
        current_stage_role: parse_required_toml_string(source, "current_stage_role", path)?,
        current_reproducible_build_sha256: parse_required_toml_string(
            source,
            "current_reproducible_build_sha256",
            path,
        )?,
        forward_selector: parse_required_toml_string(source, "forward_selector", path)?,
        forward_stage_role: parse_required_toml_string(source, "forward_stage_role", path)?,
        forward_reproducible_build_sha256: parse_required_toml_string(
            source,
            "forward_reproducible_build_sha256",
            path,
        )?,
        candidate_compiler_image_sha256: parse_required_toml_string(
            source,
            "candidate_compiler_image_sha256",
            path,
        )?,
        native_output_sha256: parse_required_toml_string(source, "native_output_sha256", path)?,
        reversible: parse_required_toml_bool(source, "reversible", path)?,
        authorizer_id: parse_required_toml_string(source, "authorizer_id", path)?,
        authorizer_environment_id: parse_required_toml_string(
            source,
            "authorizer_environment_id",
            path,
        )?,
        authorizer_public_key_id: parse_required_toml_string(
            source,
            "authorizer_public_key_id",
            path,
        )?,
        verdict: parse_required_toml_string(source, "verdict", path)?,
        proof_sha256: parse_required_toml_string(source, "proof_sha256", path)?,
        signature_hex: parse_required_toml_string(source, "signature_hex", path)?,
    };
    validate_transition(&transition)?;
    if render_compiler_component_transition(&transition) != source {
        return Err(ArtifactError::new(format!(
            "compiler component transition `{}` is not canonically encoded",
            path.display()
        )));
    }
    Ok(transition)
}

fn validate_text(source: &str, path: &Path) -> Result<(), ArtifactError> {
    if source.is_empty()
        || !source.ends_with('\n')
        || source.contains('\r')
        || source.contains('\0')
    {
        return Err(ArtifactError::new(format!(
            "compiler component transition `{}` must be UTF-8/LF text",
            path.display()
        )));
    }
    Ok(())
}
