use std::{fs, path::Path};

use sha2::{Digest, Sha256};

use crate::{
    read_compiler_component_build,
    toml::{escape_toml_string, parse_required_toml_string, parse_required_toml_usize},
    ArtifactError, CompilerComponentBuild, COMPILER_COMPONENT_BUILD_FILE,
    COMPILER_COMPONENT_BUILD_PROTOCOL, COMPILER_COMPONENT_STAGE0_ROLE,
};

pub const COMPILER_CANDIDATE_EXECUTION_PROTOCOL: &str = "nuis-compiler-candidate-execution-v1";
pub const COMPILER_CANDIDATE_RUNNER_CONTRACT: &str = "nuis-bootstrap-candidate-runner-v1";
pub const COMPILER_CANDIDATE_EXECUTION_ROLE: &str = "stage1-candidate-probe";
pub const COMPILER_CANDIDATE_EXECUTION_AUTHORITY: &str = "execution-only-no-component-production";
pub const COMPILER_CANDIDATE_EXECUTION_FILE: &str = "nuis.compiler-candidate-execution.toml";
const EMPTY_ARGUMENTS_CONTRACT: &str = "empty-argv-v1";
const CLOSED_STDIN_CONTRACT: &str = "closed-stdin-v1";

#[derive(Debug, Clone, Copy)]
pub struct CompilerCandidateExecutionInput<'a> {
    pub component: &'a CompilerComponentBuild,
    pub exit_code: usize,
    pub stdout: &'a [u8],
    pub stderr: &'a [u8],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerCandidateExecution {
    pub protocol: String,
    pub runner_contract: String,
    pub probe_role: String,
    pub authority: String,
    pub component_record_file: String,
    pub component_record_sha256: String,
    pub component_reproducible_build_sha256: String,
    pub component_id: String,
    pub source_component_role: String,
    pub bootstrap_subset_protocol: String,
    pub candidate_binary_file: String,
    pub candidate_binary_bytes: usize,
    pub candidate_binary_sha256: String,
    pub arguments_contract: String,
    pub stdin_contract: String,
    pub exit_code: usize,
    pub stdout_bytes: usize,
    pub stdout_sha256: String,
    pub stderr_bytes: usize,
    pub stderr_sha256: String,
    pub execution_sha256: String,
}

pub fn build_compiler_candidate_execution(
    input: &CompilerCandidateExecutionInput<'_>,
) -> Result<CompilerCandidateExecution, ArtifactError> {
    validate_source_component(input.component)?;
    if input.exit_code != 0 || !input.stdout.is_empty() || !input.stderr.is_empty() {
        return Err(ArtifactError::new(
            "compiler candidate execution v1 requires exit 0 with empty stdout and stderr",
        ));
    }

    let mut execution = CompilerCandidateExecution {
        protocol: COMPILER_CANDIDATE_EXECUTION_PROTOCOL.to_owned(),
        runner_contract: COMPILER_CANDIDATE_RUNNER_CONTRACT.to_owned(),
        probe_role: COMPILER_CANDIDATE_EXECUTION_ROLE.to_owned(),
        authority: COMPILER_CANDIDATE_EXECUTION_AUTHORITY.to_owned(),
        component_record_file: COMPILER_COMPONENT_BUILD_FILE.to_owned(),
        component_record_sha256: input.component.record_sha256.clone(),
        component_reproducible_build_sha256: input.component.reproducible_build_sha256.clone(),
        component_id: input.component.component_id.clone(),
        source_component_role: input.component.stage_role.clone(),
        bootstrap_subset_protocol: input.component.bootstrap_subset_protocol.clone(),
        candidate_binary_file: input.component.native_binary_file.clone(),
        candidate_binary_bytes: input.component.native_binary_bytes,
        candidate_binary_sha256: input.component.native_binary_sha256.clone(),
        arguments_contract: EMPTY_ARGUMENTS_CONTRACT.to_owned(),
        stdin_contract: CLOSED_STDIN_CONTRACT.to_owned(),
        exit_code: input.exit_code,
        stdout_bytes: input.stdout.len(),
        stdout_sha256: sha256_hex(input.stdout),
        stderr_bytes: input.stderr.len(),
        stderr_sha256: sha256_hex(input.stderr),
        execution_sha256: String::new(),
    };
    execution.execution_sha256 = execution_identity(&execution);
    validate_compiler_candidate_execution(&execution)?;
    Ok(execution)
}

pub fn render_compiler_candidate_execution(execution: &CompilerCandidateExecution) -> String {
    format!(
        "protocol = \"{}\"\nrunner_contract = \"{}\"\nprobe_role = \"{}\"\nauthority = \"{}\"\ncomponent_record_file = \"{}\"\ncomponent_record_sha256 = \"{}\"\ncomponent_reproducible_build_sha256 = \"{}\"\ncomponent_id = \"{}\"\nsource_component_role = \"{}\"\nbootstrap_subset_protocol = \"{}\"\ncandidate_binary_file = \"{}\"\ncandidate_binary_bytes = {}\ncandidate_binary_sha256 = \"{}\"\narguments_contract = \"{}\"\nstdin_contract = \"{}\"\nexit_code = {}\nstdout_bytes = {}\nstdout_sha256 = \"{}\"\nstderr_bytes = {}\nstderr_sha256 = \"{}\"\nexecution_sha256 = \"{}\"\n",
        execution.protocol,
        execution.runner_contract,
        execution.probe_role,
        execution.authority,
        execution.component_record_file,
        execution.component_record_sha256,
        execution.component_reproducible_build_sha256,
        escape_toml_string(&execution.component_id),
        execution.source_component_role,
        escape_toml_string(&execution.bootstrap_subset_protocol),
        escape_toml_string(&execution.candidate_binary_file),
        execution.candidate_binary_bytes,
        execution.candidate_binary_sha256,
        execution.arguments_contract,
        execution.stdin_contract,
        execution.exit_code,
        execution.stdout_bytes,
        execution.stdout_sha256,
        execution.stderr_bytes,
        execution.stderr_sha256,
        execution.execution_sha256,
    )
}

pub fn parse_compiler_candidate_execution(
    path: &Path,
) -> Result<CompilerCandidateExecution, ArtifactError> {
    let source = fs::read_to_string(path).map_err(|error| {
        ArtifactError::new(format!(
            "failed to read compiler candidate execution `{}`: {error}",
            path.display()
        ))
    })?;
    parse_compiler_candidate_execution_from_source(&source, path)
}

pub fn parse_compiler_candidate_execution_from_source(
    source: &str,
    path: &Path,
) -> Result<CompilerCandidateExecution, ArtifactError> {
    if source.is_empty()
        || source.contains('\r')
        || source.contains('\0')
        || !source.ends_with('\n')
    {
        return Err(ArtifactError::new(format!(
            "compiler candidate execution `{}` must be canonical UTF-8/LF text",
            path.display()
        )));
    }
    let execution = CompilerCandidateExecution {
        protocol: parse_required_toml_string(source, "protocol", path)?,
        runner_contract: parse_required_toml_string(source, "runner_contract", path)?,
        probe_role: parse_required_toml_string(source, "probe_role", path)?,
        authority: parse_required_toml_string(source, "authority", path)?,
        component_record_file: parse_required_toml_string(source, "component_record_file", path)?,
        component_record_sha256: parse_required_toml_string(
            source,
            "component_record_sha256",
            path,
        )?,
        component_reproducible_build_sha256: parse_required_toml_string(
            source,
            "component_reproducible_build_sha256",
            path,
        )?,
        component_id: parse_required_toml_string(source, "component_id", path)?,
        source_component_role: parse_required_toml_string(source, "source_component_role", path)?,
        bootstrap_subset_protocol: parse_required_toml_string(
            source,
            "bootstrap_subset_protocol",
            path,
        )?,
        candidate_binary_file: parse_required_toml_string(source, "candidate_binary_file", path)?,
        candidate_binary_bytes: parse_required_toml_usize(source, "candidate_binary_bytes", path)?,
        candidate_binary_sha256: parse_required_toml_string(
            source,
            "candidate_binary_sha256",
            path,
        )?,
        arguments_contract: parse_required_toml_string(source, "arguments_contract", path)?,
        stdin_contract: parse_required_toml_string(source, "stdin_contract", path)?,
        exit_code: parse_required_toml_usize(source, "exit_code", path)?,
        stdout_bytes: parse_required_toml_usize(source, "stdout_bytes", path)?,
        stdout_sha256: parse_required_toml_string(source, "stdout_sha256", path)?,
        stderr_bytes: parse_required_toml_usize(source, "stderr_bytes", path)?,
        stderr_sha256: parse_required_toml_string(source, "stderr_sha256", path)?,
        execution_sha256: parse_required_toml_string(source, "execution_sha256", path)?,
    };
    validate_compiler_candidate_execution(&execution)?;
    if render_compiler_candidate_execution(&execution) != source {
        return Err(ArtifactError::new(format!(
            "compiler candidate execution `{}` is not canonically encoded",
            path.display()
        )));
    }
    Ok(execution)
}

pub fn read_compiler_candidate_execution(
    path: &Path,
) -> Result<CompilerCandidateExecution, ArtifactError> {
    let execution = parse_compiler_candidate_execution(path)?;
    let root = path.parent().unwrap_or_else(|| Path::new("."));
    let component =
        read_compiler_component_build(root.join(&execution.component_record_file).as_path())?;
    validate_source_component(&component)?;
    if execution.component_record_sha256 != component.record_sha256
        || execution.component_reproducible_build_sha256 != component.reproducible_build_sha256
        || execution.component_id != component.component_id
        || execution.source_component_role != component.stage_role
        || execution.bootstrap_subset_protocol != component.bootstrap_subset_protocol
        || execution.candidate_binary_file != component.native_binary_file
        || execution.candidate_binary_bytes != component.native_binary_bytes
        || execution.candidate_binary_sha256 != component.native_binary_sha256
    {
        return Err(ArtifactError::new(
            "compiler candidate execution does not match its component build",
        ));
    }
    Ok(execution)
}

fn validate_source_component(component: &CompilerComponentBuild) -> Result<(), ArtifactError> {
    if component.protocol != COMPILER_COMPONENT_BUILD_PROTOCOL
        || component.stage_role != COMPILER_COMPONENT_STAGE0_ROLE
    {
        return Err(ArtifactError::new(
            "compiler candidate execution v1 requires a verified stage0 component build",
        ));
    }
    for (label, value) in [
        ("component record", component.record_sha256.as_str()),
        (
            "component reproducible build",
            component.reproducible_build_sha256.as_str(),
        ),
        ("candidate binary", component.native_binary_sha256.as_str()),
    ] {
        validate_sha256(value, label)?;
    }
    if component.component_id.is_empty()
        || component.component_id.chars().any(char::is_control)
        || component.native_binary_bytes == 0
    {
        return Err(ArtifactError::new(
            "compiler candidate execution source component is invalid",
        ));
    }
    Ok(())
}

fn validate_compiler_candidate_execution(
    execution: &CompilerCandidateExecution,
) -> Result<(), ArtifactError> {
    if execution.protocol != COMPILER_CANDIDATE_EXECUTION_PROTOCOL
        || execution.runner_contract != COMPILER_CANDIDATE_RUNNER_CONTRACT
        || execution.probe_role != COMPILER_CANDIDATE_EXECUTION_ROLE
        || execution.authority != COMPILER_CANDIDATE_EXECUTION_AUTHORITY
        || execution.component_record_file != COMPILER_COMPONENT_BUILD_FILE
        || execution.source_component_role != COMPILER_COMPONENT_STAGE0_ROLE
        || execution.arguments_contract != EMPTY_ARGUMENTS_CONTRACT
        || execution.stdin_contract != CLOSED_STDIN_CONTRACT
    {
        return Err(ArtifactError::new(
            "compiler candidate execution declares an unsupported contract",
        ));
    }
    if execution.exit_code != 0 || execution.stdout_bytes != 0 || execution.stderr_bytes != 0 {
        return Err(ArtifactError::new(
            "compiler candidate execution v1 requires an output-free successful process",
        ));
    }
    for (label, value) in [
        ("component id", execution.component_id.as_str()),
        (
            "bootstrap subset protocol",
            execution.bootstrap_subset_protocol.as_str(),
        ),
    ] {
        if value.is_empty() || value.chars().any(char::is_control) {
            return Err(ArtifactError::new(format!(
                "compiler candidate {label} is invalid"
            )));
        }
    }
    let candidate_path = Path::new(&execution.candidate_binary_file);
    if execution.candidate_binary_bytes == 0
        || candidate_path.components().count() != 1
        || candidate_path.file_name().and_then(|value| value.to_str())
            != Some(execution.candidate_binary_file.as_str())
    {
        return Err(ArtifactError::new(
            "compiler candidate binary must be a non-empty sibling file",
        ));
    }
    for (label, value) in [
        (
            "component record",
            execution.component_record_sha256.as_str(),
        ),
        (
            "component reproducible build",
            execution.component_reproducible_build_sha256.as_str(),
        ),
        (
            "candidate binary",
            execution.candidate_binary_sha256.as_str(),
        ),
        ("stdout", execution.stdout_sha256.as_str()),
        ("stderr", execution.stderr_sha256.as_str()),
        ("execution", execution.execution_sha256.as_str()),
    ] {
        validate_sha256(value, label)?;
    }
    let empty_sha256 = sha256_hex(&[]);
    if execution.stdout_sha256 != empty_sha256
        || execution.stderr_sha256 != empty_sha256
        || execution.execution_sha256 != execution_identity(execution)
    {
        return Err(ArtifactError::new(
            "compiler candidate execution identity mismatch",
        ));
    }
    Ok(())
}

fn execution_identity(execution: &CompilerCandidateExecution) -> String {
    sha256_fields(&[
        execution.protocol.as_bytes(),
        execution.runner_contract.as_bytes(),
        execution.probe_role.as_bytes(),
        execution.authority.as_bytes(),
        execution.component_record_file.as_bytes(),
        execution.component_record_sha256.as_bytes(),
        execution.component_reproducible_build_sha256.as_bytes(),
        execution.component_id.as_bytes(),
        execution.source_component_role.as_bytes(),
        execution.bootstrap_subset_protocol.as_bytes(),
        execution.candidate_binary_file.as_bytes(),
        &(execution.candidate_binary_bytes as u64).to_le_bytes(),
        execution.candidate_binary_sha256.as_bytes(),
        execution.arguments_contract.as_bytes(),
        execution.stdin_contract.as_bytes(),
        &(execution.exit_code as u64).to_le_bytes(),
        &(execution.stdout_bytes as u64).to_le_bytes(),
        execution.stdout_sha256.as_bytes(),
        &(execution.stderr_bytes as u64).to_le_bytes(),
        execution.stderr_sha256.as_bytes(),
    ])
}

fn validate_sha256(value: &str, label: &str) -> Result<(), ArtifactError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Ok(());
    }
    Err(ArtifactError::new(format!(
        "compiler candidate {label} identity must be lowercase SHA-256"
    )))
}

fn sha256_fields(fields: &[&[u8]]) -> String {
    let mut hasher = Sha256::new();
    for field in fields {
        hasher.update((field.len() as u64).to_le_bytes());
        hasher.update(field);
    }
    format!("{:x}", hasher.finalize())
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
#[path = "compiler_candidate_execution_tests.rs"]
mod tests;
