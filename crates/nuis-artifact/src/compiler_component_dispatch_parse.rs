use std::{fs, path::Path};

use crate::{
    toml::{parse_required_toml_string, parse_required_toml_usize},
    ArtifactError,
};

use super::{
    render_compiler_component_dispatch_receipt, validate_dispatch_receipt,
    CompilerComponentDispatchReceipt,
};

pub fn parse_compiler_component_dispatch_receipt(
    path: &Path,
) -> Result<CompilerComponentDispatchReceipt, ArtifactError> {
    let source = fs::read_to_string(path).map_err(|error| {
        ArtifactError::new(format!(
            "failed to read compiler component dispatch receipt `{}`: {error}",
            path.display()
        ))
    })?;
    parse_compiler_component_dispatch_receipt_from_source(&source, path)
}

pub fn parse_compiler_component_dispatch_receipt_from_source(
    source: &str,
    path: &Path,
) -> Result<CompilerComponentDispatchReceipt, ArtifactError> {
    if source.is_empty()
        || !source.ends_with('\n')
        || source.contains('\r')
        || source.contains('\0')
    {
        return Err(ArtifactError::new(format!(
            "compiler component dispatch receipt `{}` must be canonical UTF-8/LF text",
            path.display()
        )));
    }
    let receipt = CompilerComponentDispatchReceipt {
        protocol: parse_required_toml_string(source, "protocol", path)?,
        driver_contract: parse_required_toml_string(source, "driver_contract", path)?,
        authority: parse_required_toml_string(source, "authority", path)?,
        inventory_contract: parse_required_toml_string(source, "inventory_contract", path)?,
        request_contract: parse_required_toml_string(source, "request_contract", path)?,
        transition_protocol: parse_required_toml_string(source, "transition_protocol", path)?,
        transition_generation: parse_required_toml_usize(source, "transition_generation", path)?,
        transition_proof_sha256: parse_required_toml_string(
            source,
            "transition_proof_sha256",
            path,
        )?,
        component_id: parse_required_toml_string(source, "component_id", path)?,
        selected_selector: parse_required_toml_string(source, "selected_selector", path)?,
        selected_stage_role: parse_required_toml_string(source, "selected_stage_role", path)?,
        selected_reproducible_build_sha256: parse_required_toml_string(
            source,
            "selected_reproducible_build_sha256",
            path,
        )?,
        selected_record_sha256: parse_required_toml_string(source, "selected_record_sha256", path)?,
        selected_compiler_image_bytes: parse_required_toml_usize(
            source,
            "selected_compiler_image_bytes",
            path,
        )?,
        selected_compiler_image_sha256: parse_required_toml_string(
            source,
            "selected_compiler_image_sha256",
            path,
        )?,
        forward_selector: parse_required_toml_string(source, "forward_selector", path)?,
        forward_stage_role: parse_required_toml_string(source, "forward_stage_role", path)?,
        forward_reproducible_build_sha256: parse_required_toml_string(
            source,
            "forward_reproducible_build_sha256",
            path,
        )?,
        forward_record_sha256: parse_required_toml_string(source, "forward_record_sha256", path)?,
        forward_compiler_image_bytes: parse_required_toml_usize(
            source,
            "forward_compiler_image_bytes",
            path,
        )?,
        forward_compiler_image_sha256: parse_required_toml_string(
            source,
            "forward_compiler_image_sha256",
            path,
        )?,
        argument_count: parse_required_toml_usize(source, "argument_count", path)?,
        argument_0: parse_required_toml_string(source, "argument_0", path)?,
        stdin_contract: parse_required_toml_string(source, "stdin_contract", path)?,
        exit_code: parse_required_toml_usize(source, "exit_code", path)?,
        stdout_bytes: parse_required_toml_usize(source, "stdout_bytes", path)?,
        stdout_sha256: parse_required_toml_string(source, "stdout_sha256", path)?,
        stderr_bytes: parse_required_toml_usize(source, "stderr_bytes", path)?,
        stderr_sha256: parse_required_toml_string(source, "stderr_sha256", path)?,
        verdict: parse_required_toml_string(source, "verdict", path)?,
        dispatch_sha256: parse_required_toml_string(source, "dispatch_sha256", path)?,
    };
    validate_dispatch_receipt(&receipt)?;
    if render_compiler_component_dispatch_receipt(&receipt) != source {
        return Err(ArtifactError::new(format!(
            "compiler component dispatch receipt `{}` is not canonically encoded",
            path.display()
        )));
    }
    Ok(receipt)
}
