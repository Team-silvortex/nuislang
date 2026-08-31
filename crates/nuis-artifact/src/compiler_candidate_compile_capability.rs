use std::{fs, path::Path};

use sha2::{Digest, Sha256};

use crate::{
    compiler_component_attestation_registry::encode_hex,
    compiler_component_compile_dispatch::{
        compiled_artifact_semantic_identity, verify_artifact_bytes,
        verify_compiler_component_rebuild,
    },
    decode_nuis_compiled_artifact_binary,
    toml::{
        escape_toml_string, parse_required_toml_bool, parse_required_toml_string,
        parse_required_toml_usize,
    },
    verify_compiler_component_build, verify_compiler_component_build_image, ArtifactError,
    CompilerCandidateProduction, CompilerComponentBuild, COMPILER_CANDIDATE_ADAPTER_FILE,
    COMPILER_CANDIDATE_PRODUCTION_PROTOCOL, COMPILER_COMPONENT_COMPILED_ARTIFACT_IDENTITY_CONTRACT,
    COMPILER_COMPONENT_STAGE0_ROLE, COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE,
};

pub const COMPILER_CANDIDATE_COMPILE_CAPABILITY_PROTOCOL: &str =
    "nuis-compiler-candidate-compile-capability-v1";
pub const COMPILER_CANDIDATE_COMPILE_CAPABILITY_FILE: &str =
    "nuis.compiler-candidate-compile-capability.toml";
pub const COMPILER_CANDIDATE_COMPILE_DRIVER_CONTRACT: &str =
    "nuis-stage1-candidate-delegating-compile-driver-v1";
pub const COMPILER_CANDIDATE_COMPILE_CAPABILITY_AUTHORITY: &str =
    "compile-capability-only-no-replacement-or-selection";
pub const COMPILER_CANDIDATE_COMPILE_REQUEST_CONTRACT: &str =
    "canonical-stage0-rebuild-through-production-bound-candidate-v1";
pub const COMPILER_CANDIDATE_COMPILE_PROVIDER_CONTRACT: &str =
    "verified-stage0-compiler-image-exact-exec-v1";
pub const COMPILER_CANDIDATE_COMPILE_ADMISSION_CONTRACT: &str =
    "nuis-owned-complete-request-byte-fold-v1";
pub const COMPILER_CANDIDATE_COMPILE_COMMAND: &str = "bootstrap-build";
pub const COMPILER_CANDIDATE_COMPILE_ARGUMENT_CONTRACT: &str =
    "exact-command-project-output-no-shell-v1";
pub const COMPILER_CANDIDATE_COMPILE_PROVIDER_ENVIRONMENT: &str =
    "NUIS_BOOTSTRAP_STAGE0_PROVIDER_V1";
pub const COMPILER_CANDIDATE_COMPILE_CAPABILITY_VERDICT: &str =
    "candidate-compile-capability-verified-no-selection-authority";

const CLOSED_STDIN_CONTRACT: &str = "closed-stdin-v1";
const ADMISSION_MARKER: &[u8] = b"candidate_compile_admission=nuis-owned-stage-fold-v1\n";

#[derive(Debug, Clone, Copy)]
pub struct CompilerCandidateCompileCapabilityInput<'a> {
    pub stage0: &'a CompilerComponentBuild,
    pub candidate: &'a CompilerComponentBuild,
    pub production: &'a CompilerCandidateProduction,
    pub adapter: &'a [u8],
    pub provider_image: &'a [u8],
    pub request_compiled_artifact: &'a [u8],
    pub result: &'a CompilerComponentBuild,
    pub result_compiled_artifact: &'a [u8],
    pub exit_code: usize,
    pub stdout: &'a [u8],
    pub stderr: &'a [u8],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerCandidateCompileCapability {
    pub protocol: String,
    pub driver_contract: String,
    pub authority: String,
    pub request_contract: String,
    pub provider_contract: String,
    pub admission_contract: String,
    pub command: String,
    pub argument_contract: String,
    pub provider_environment: String,
    pub stdin_contract: String,
    pub bootstrap_subset_protocol: String,
    pub component_id: String,
    pub component_domain: String,
    pub component_unit: String,
    pub stage0_record_sha256: String,
    pub stage0_reproducible_build_sha256: String,
    pub provider_image_bytes: usize,
    pub provider_image_sha256: String,
    pub candidate_record_sha256: String,
    pub candidate_reproducible_build_sha256: String,
    pub candidate_producer_id: String,
    pub candidate_compiler_image_sha256: String,
    pub production_protocol: String,
    pub production_proof_sha256: String,
    pub adapter_file: String,
    pub adapter_bytes: usize,
    pub adapter_sha256: String,
    pub request_compiled_artifact_bytes: usize,
    pub request_compiled_artifact_sha256: String,
    pub compiled_artifact_identity_contract: String,
    pub compiled_artifact_semantic_sha256: String,
    pub result_record_sha256: String,
    pub result_reproducible_build_sha256: String,
    pub result_compiled_artifact_bytes: usize,
    pub result_compiled_artifact_sha256: String,
    pub result_native_binary_bytes: usize,
    pub result_native_binary_sha256: String,
    pub exit_code: usize,
    pub stdout_bytes: usize,
    pub stdout_sha256: String,
    pub stderr_bytes: usize,
    pub stderr_sha256: String,
    pub replacement_authorized: bool,
    pub selection_authorized: bool,
    pub verdict: String,
    pub proof_sha256: String,
}

pub fn build_compiler_candidate_compile_capability(
    input: &CompilerCandidateCompileCapabilityInput<'_>,
) -> Result<CompilerCandidateCompileCapability, ArtifactError> {
    verify_inputs(input)?;
    let request_artifact = decode_nuis_compiled_artifact_binary(input.request_compiled_artifact)?;
    let result_artifact = decode_nuis_compiled_artifact_binary(input.result_compiled_artifact)?;
    let semantic_sha256 = compiled_artifact_semantic_identity(&request_artifact);
    if semantic_sha256 != compiled_artifact_semantic_identity(&result_artifact) {
        return Err(ArtifactError::new(
            "compiler candidate compile capability changed compiled artifact semantics",
        ));
    }

    let mut capability = CompilerCandidateCompileCapability {
        protocol: COMPILER_CANDIDATE_COMPILE_CAPABILITY_PROTOCOL.to_owned(),
        driver_contract: COMPILER_CANDIDATE_COMPILE_DRIVER_CONTRACT.to_owned(),
        authority: COMPILER_CANDIDATE_COMPILE_CAPABILITY_AUTHORITY.to_owned(),
        request_contract: COMPILER_CANDIDATE_COMPILE_REQUEST_CONTRACT.to_owned(),
        provider_contract: COMPILER_CANDIDATE_COMPILE_PROVIDER_CONTRACT.to_owned(),
        admission_contract: COMPILER_CANDIDATE_COMPILE_ADMISSION_CONTRACT.to_owned(),
        command: COMPILER_CANDIDATE_COMPILE_COMMAND.to_owned(),
        argument_contract: COMPILER_CANDIDATE_COMPILE_ARGUMENT_CONTRACT.to_owned(),
        provider_environment: COMPILER_CANDIDATE_COMPILE_PROVIDER_ENVIRONMENT.to_owned(),
        stdin_contract: CLOSED_STDIN_CONTRACT.to_owned(),
        bootstrap_subset_protocol: input.stage0.bootstrap_subset_protocol.clone(),
        component_id: input.stage0.component_id.clone(),
        component_domain: input.stage0.component_domain.clone(),
        component_unit: input.stage0.component_unit.clone(),
        stage0_record_sha256: input.stage0.record_sha256.clone(),
        stage0_reproducible_build_sha256: input.stage0.reproducible_build_sha256.clone(),
        provider_image_bytes: input.provider_image.len(),
        provider_image_sha256: input.stage0.compiler_image_sha256.clone(),
        candidate_record_sha256: input.candidate.record_sha256.clone(),
        candidate_reproducible_build_sha256: input.candidate.reproducible_build_sha256.clone(),
        candidate_producer_id: input.candidate.producer_id.clone(),
        candidate_compiler_image_sha256: input.candidate.compiler_image_sha256.clone(),
        production_protocol: input.production.protocol.clone(),
        production_proof_sha256: input.production.proof_sha256.clone(),
        adapter_file: input.production.adapter_file.clone(),
        adapter_bytes: input.adapter.len(),
        adapter_sha256: sha256_hex(input.adapter),
        request_compiled_artifact_bytes: input.request_compiled_artifact.len(),
        request_compiled_artifact_sha256: sha256_hex(input.request_compiled_artifact),
        compiled_artifact_identity_contract: COMPILER_COMPONENT_COMPILED_ARTIFACT_IDENTITY_CONTRACT
            .to_owned(),
        compiled_artifact_semantic_sha256: semantic_sha256,
        result_record_sha256: input.result.record_sha256.clone(),
        result_reproducible_build_sha256: input.result.reproducible_build_sha256.clone(),
        result_compiled_artifact_bytes: input.result_compiled_artifact.len(),
        result_compiled_artifact_sha256: sha256_hex(input.result_compiled_artifact),
        result_native_binary_bytes: input.result.native_binary_bytes,
        result_native_binary_sha256: input.result.native_binary_sha256.clone(),
        exit_code: input.exit_code,
        stdout_bytes: input.stdout.len(),
        stdout_sha256: sha256_hex(input.stdout),
        stderr_bytes: input.stderr.len(),
        stderr_sha256: sha256_hex(input.stderr),
        replacement_authorized: false,
        selection_authorized: false,
        verdict: COMPILER_CANDIDATE_COMPILE_CAPABILITY_VERDICT.to_owned(),
        proof_sha256: String::new(),
    };
    capability.proof_sha256 = capability_identity(&capability);
    validate_compiler_candidate_compile_capability(&capability)?;
    Ok(capability)
}

fn verify_inputs(input: &CompilerCandidateCompileCapabilityInput<'_>) -> Result<(), ArtifactError> {
    verify_compiler_component_build(input.stage0)?;
    verify_compiler_component_build(input.candidate)?;
    verify_compiler_component_build_image(input.stage0, input.provider_image)?;
    verify_compiler_component_rebuild(input.stage0, input.result)?;
    verify_artifact_bytes(input.stage0, input.request_compiled_artifact)?;
    verify_artifact_bytes(input.result, input.result_compiled_artifact)?;
    if input.stage0.stage_role != COMPILER_COMPONENT_STAGE0_ROLE
        || input.candidate.stage_role != COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE
        || input.stage0.component_id != input.candidate.component_id
        || input.stage0.component_domain != input.candidate.component_domain
        || input.stage0.component_unit != input.candidate.component_unit
        || input.production.protocol != COMPILER_CANDIDATE_PRODUCTION_PROTOCOL
        || input.production.stage0_component_sha256 != input.stage0.record_sha256
        || input.production.candidate_component_sha256 != input.candidate.record_sha256
        || input.production.candidate_producer_id != input.candidate.producer_id
        || input.production.candidate_compiler_image_sha256 != input.candidate.compiler_image_sha256
        || input.production.adapter_file != COMPILER_CANDIDATE_ADAPTER_FILE
        || input.production.adapter_bytes != input.adapter.len()
        || input.production.adapter_sha256 != sha256_hex(input.adapter)
        || input.production.replacement_authorized
    {
        return Err(ArtifactError::new(
            "compiler candidate compile capability input lineage is inconsistent",
        ));
    }
    if input.exit_code != 0
        || input.stdout.is_empty()
        || !input.stderr.is_empty()
        || !contains_bytes(input.stdout, ADMISSION_MARKER)
    {
        return Err(ArtifactError::new(
            "compiler candidate compile capability requires exit 0, empty stderr, and the Nuis-owned admission marker",
        ));
    }
    Ok(())
}

pub fn render_compiler_candidate_compile_capability(
    capability: &CompilerCandidateCompileCapability,
) -> String {
    format!(
        "protocol = \"{}\"\ndriver_contract = \"{}\"\nauthority = \"{}\"\nrequest_contract = \"{}\"\nprovider_contract = \"{}\"\nadmission_contract = \"{}\"\ncommand = \"{}\"\nargument_contract = \"{}\"\nprovider_environment = \"{}\"\nstdin_contract = \"{}\"\nbootstrap_subset_protocol = \"{}\"\ncomponent_id = \"{}\"\ncomponent_domain = \"{}\"\ncomponent_unit = \"{}\"\nstage0_record_sha256 = \"{}\"\nstage0_reproducible_build_sha256 = \"{}\"\nprovider_image_bytes = {}\nprovider_image_sha256 = \"{}\"\ncandidate_record_sha256 = \"{}\"\ncandidate_reproducible_build_sha256 = \"{}\"\ncandidate_producer_id = \"{}\"\ncandidate_compiler_image_sha256 = \"{}\"\nproduction_protocol = \"{}\"\nproduction_proof_sha256 = \"{}\"\nadapter_file = \"{}\"\nadapter_bytes = {}\nadapter_sha256 = \"{}\"\nrequest_compiled_artifact_bytes = {}\nrequest_compiled_artifact_sha256 = \"{}\"\ncompiled_artifact_identity_contract = \"{}\"\ncompiled_artifact_semantic_sha256 = \"{}\"\nresult_record_sha256 = \"{}\"\nresult_reproducible_build_sha256 = \"{}\"\nresult_compiled_artifact_bytes = {}\nresult_compiled_artifact_sha256 = \"{}\"\nresult_native_binary_bytes = {}\nresult_native_binary_sha256 = \"{}\"\nexit_code = {}\nstdout_bytes = {}\nstdout_sha256 = \"{}\"\nstderr_bytes = {}\nstderr_sha256 = \"{}\"\nreplacement_authorized = {}\nselection_authorized = {}\nverdict = \"{}\"\nproof_sha256 = \"{}\"\n",
        capability.protocol,
        capability.driver_contract,
        capability.authority,
        capability.request_contract,
        capability.provider_contract,
        capability.admission_contract,
        capability.command,
        capability.argument_contract,
        capability.provider_environment,
        capability.stdin_contract,
        escape_toml_string(&capability.bootstrap_subset_protocol),
        escape_toml_string(&capability.component_id),
        escape_toml_string(&capability.component_domain),
        escape_toml_string(&capability.component_unit),
        capability.stage0_record_sha256,
        capability.stage0_reproducible_build_sha256,
        capability.provider_image_bytes,
        capability.provider_image_sha256,
        capability.candidate_record_sha256,
        capability.candidate_reproducible_build_sha256,
        escape_toml_string(&capability.candidate_producer_id),
        capability.candidate_compiler_image_sha256,
        capability.production_protocol,
        capability.production_proof_sha256,
        escape_toml_string(&capability.adapter_file),
        capability.adapter_bytes,
        capability.adapter_sha256,
        capability.request_compiled_artifact_bytes,
        capability.request_compiled_artifact_sha256,
        capability.compiled_artifact_identity_contract,
        capability.compiled_artifact_semantic_sha256,
        capability.result_record_sha256,
        capability.result_reproducible_build_sha256,
        capability.result_compiled_artifact_bytes,
        capability.result_compiled_artifact_sha256,
        capability.result_native_binary_bytes,
        capability.result_native_binary_sha256,
        capability.exit_code,
        capability.stdout_bytes,
        capability.stdout_sha256,
        capability.stderr_bytes,
        capability.stderr_sha256,
        capability.replacement_authorized,
        capability.selection_authorized,
        capability.verdict,
        capability.proof_sha256,
    )
}

pub fn parse_compiler_candidate_compile_capability(
    path: &Path,
) -> Result<CompilerCandidateCompileCapability, ArtifactError> {
    let source = fs::read_to_string(path).map_err(|error| {
        ArtifactError::new(format!(
            "failed to read compiler candidate compile capability `{}`: {error}",
            path.display()
        ))
    })?;
    parse_compiler_candidate_compile_capability_from_source(&source, path)
}

pub fn parse_compiler_candidate_compile_capability_from_source(
    source: &str,
    path: &Path,
) -> Result<CompilerCandidateCompileCapability, ArtifactError> {
    if source.is_empty()
        || source.contains('\r')
        || source.contains('\0')
        || !source.ends_with('\n')
    {
        return Err(ArtifactError::new(format!(
            "compiler candidate compile capability `{}` must be canonical UTF-8/LF text",
            path.display()
        )));
    }
    let string = |key| parse_required_toml_string(source, key, path);
    let number = |key| parse_required_toml_usize(source, key, path);
    let boolean = |key| parse_required_toml_bool(source, key, path);
    let capability = CompilerCandidateCompileCapability {
        protocol: string("protocol")?,
        driver_contract: string("driver_contract")?,
        authority: string("authority")?,
        request_contract: string("request_contract")?,
        provider_contract: string("provider_contract")?,
        admission_contract: string("admission_contract")?,
        command: string("command")?,
        argument_contract: string("argument_contract")?,
        provider_environment: string("provider_environment")?,
        stdin_contract: string("stdin_contract")?,
        bootstrap_subset_protocol: string("bootstrap_subset_protocol")?,
        component_id: string("component_id")?,
        component_domain: string("component_domain")?,
        component_unit: string("component_unit")?,
        stage0_record_sha256: string("stage0_record_sha256")?,
        stage0_reproducible_build_sha256: string("stage0_reproducible_build_sha256")?,
        provider_image_bytes: number("provider_image_bytes")?,
        provider_image_sha256: string("provider_image_sha256")?,
        candidate_record_sha256: string("candidate_record_sha256")?,
        candidate_reproducible_build_sha256: string("candidate_reproducible_build_sha256")?,
        candidate_producer_id: string("candidate_producer_id")?,
        candidate_compiler_image_sha256: string("candidate_compiler_image_sha256")?,
        production_protocol: string("production_protocol")?,
        production_proof_sha256: string("production_proof_sha256")?,
        adapter_file: string("adapter_file")?,
        adapter_bytes: number("adapter_bytes")?,
        adapter_sha256: string("adapter_sha256")?,
        request_compiled_artifact_bytes: number("request_compiled_artifact_bytes")?,
        request_compiled_artifact_sha256: string("request_compiled_artifact_sha256")?,
        compiled_artifact_identity_contract: string("compiled_artifact_identity_contract")?,
        compiled_artifact_semantic_sha256: string("compiled_artifact_semantic_sha256")?,
        result_record_sha256: string("result_record_sha256")?,
        result_reproducible_build_sha256: string("result_reproducible_build_sha256")?,
        result_compiled_artifact_bytes: number("result_compiled_artifact_bytes")?,
        result_compiled_artifact_sha256: string("result_compiled_artifact_sha256")?,
        result_native_binary_bytes: number("result_native_binary_bytes")?,
        result_native_binary_sha256: string("result_native_binary_sha256")?,
        exit_code: number("exit_code")?,
        stdout_bytes: number("stdout_bytes")?,
        stdout_sha256: string("stdout_sha256")?,
        stderr_bytes: number("stderr_bytes")?,
        stderr_sha256: string("stderr_sha256")?,
        replacement_authorized: boolean("replacement_authorized")?,
        selection_authorized: boolean("selection_authorized")?,
        verdict: string("verdict")?,
        proof_sha256: string("proof_sha256")?,
    };
    validate_compiler_candidate_compile_capability(&capability)?;
    if render_compiler_candidate_compile_capability(&capability) != source {
        return Err(ArtifactError::new(format!(
            "compiler candidate compile capability `{}` is not canonically encoded",
            path.display()
        )));
    }
    Ok(capability)
}

fn validate_compiler_candidate_compile_capability(
    capability: &CompilerCandidateCompileCapability,
) -> Result<(), ArtifactError> {
    if capability.protocol != COMPILER_CANDIDATE_COMPILE_CAPABILITY_PROTOCOL
        || capability.driver_contract != COMPILER_CANDIDATE_COMPILE_DRIVER_CONTRACT
        || capability.authority != COMPILER_CANDIDATE_COMPILE_CAPABILITY_AUTHORITY
        || capability.request_contract != COMPILER_CANDIDATE_COMPILE_REQUEST_CONTRACT
        || capability.provider_contract != COMPILER_CANDIDATE_COMPILE_PROVIDER_CONTRACT
        || capability.admission_contract != COMPILER_CANDIDATE_COMPILE_ADMISSION_CONTRACT
        || capability.command != COMPILER_CANDIDATE_COMPILE_COMMAND
        || capability.argument_contract != COMPILER_CANDIDATE_COMPILE_ARGUMENT_CONTRACT
        || capability.provider_environment != COMPILER_CANDIDATE_COMPILE_PROVIDER_ENVIRONMENT
        || capability.stdin_contract != CLOSED_STDIN_CONTRACT
        || capability.production_protocol != COMPILER_CANDIDATE_PRODUCTION_PROTOCOL
        || capability.adapter_file != COMPILER_CANDIDATE_ADAPTER_FILE
        || capability.compiled_artifact_identity_contract
            != COMPILER_COMPONENT_COMPILED_ARTIFACT_IDENTITY_CONTRACT
        || capability.replacement_authorized
        || capability.selection_authorized
        || capability.verdict != COMPILER_CANDIDATE_COMPILE_CAPABILITY_VERDICT
    {
        return Err(ArtifactError::new(
            "compiler candidate compile capability declares an unsupported contract",
        ));
    }
    for value in [
        &capability.bootstrap_subset_protocol,
        &capability.component_id,
        &capability.component_domain,
        &capability.component_unit,
        &capability.candidate_producer_id,
    ] {
        if value.is_empty() || value.trim() != value || value.chars().any(char::is_control) {
            return Err(ArtifactError::new(
                "compiler candidate compile capability text identity is invalid",
            ));
        }
    }
    if capability.provider_image_bytes == 0
        || capability.adapter_bytes == 0
        || capability.request_compiled_artifact_bytes == 0
        || capability.result_compiled_artifact_bytes == 0
        || capability.result_native_binary_bytes == 0
        || capability.stage0_record_sha256 == capability.candidate_record_sha256
        || capability.stage0_reproducible_build_sha256
            == capability.candidate_reproducible_build_sha256
        || capability.stage0_reproducible_build_sha256
            != capability.result_reproducible_build_sha256
        || capability.exit_code != 0
        || capability.stdout_bytes == 0
        || capability.stderr_bytes != 0
        || capability.stderr_sha256 != sha256_hex(&[])
    {
        return Err(ArtifactError::new(
            "compiler candidate compile capability identities or execution are inconsistent",
        ));
    }
    for value in capability_hashes(capability) {
        validate_sha256(value)?;
    }
    if capability_identity(capability) != capability.proof_sha256 {
        return Err(ArtifactError::new(
            "compiler candidate compile capability proof identity mismatch",
        ));
    }
    Ok(())
}

fn capability_hashes(capability: &CompilerCandidateCompileCapability) -> [&str; 17] {
    [
        &capability.stage0_record_sha256,
        &capability.stage0_reproducible_build_sha256,
        &capability.provider_image_sha256,
        &capability.candidate_record_sha256,
        &capability.candidate_reproducible_build_sha256,
        &capability.candidate_compiler_image_sha256,
        &capability.production_proof_sha256,
        &capability.adapter_sha256,
        &capability.request_compiled_artifact_sha256,
        &capability.compiled_artifact_semantic_sha256,
        &capability.result_record_sha256,
        &capability.result_reproducible_build_sha256,
        &capability.result_compiled_artifact_sha256,
        &capability.result_native_binary_sha256,
        &capability.stdout_sha256,
        &capability.stderr_sha256,
        &capability.proof_sha256,
    ]
}

fn capability_identity(capability: &CompilerCandidateCompileCapability) -> String {
    let mut hash = Sha256::new();
    for value in [
        capability.protocol.as_bytes(),
        capability.driver_contract.as_bytes(),
        capability.authority.as_bytes(),
        capability.request_contract.as_bytes(),
        capability.provider_contract.as_bytes(),
        capability.admission_contract.as_bytes(),
        capability.command.as_bytes(),
        capability.argument_contract.as_bytes(),
        capability.provider_environment.as_bytes(),
        capability.stdin_contract.as_bytes(),
        capability.bootstrap_subset_protocol.as_bytes(),
        capability.component_id.as_bytes(),
        capability.component_domain.as_bytes(),
        capability.component_unit.as_bytes(),
        capability.stage0_record_sha256.as_bytes(),
        capability.stage0_reproducible_build_sha256.as_bytes(),
        capability.provider_image_sha256.as_bytes(),
        capability.candidate_record_sha256.as_bytes(),
        capability.candidate_reproducible_build_sha256.as_bytes(),
        capability.candidate_producer_id.as_bytes(),
        capability.candidate_compiler_image_sha256.as_bytes(),
        capability.production_protocol.as_bytes(),
        capability.production_proof_sha256.as_bytes(),
        capability.adapter_file.as_bytes(),
        capability.adapter_sha256.as_bytes(),
        capability.request_compiled_artifact_sha256.as_bytes(),
        capability.compiled_artifact_identity_contract.as_bytes(),
        capability.compiled_artifact_semantic_sha256.as_bytes(),
        capability.result_record_sha256.as_bytes(),
        capability.result_reproducible_build_sha256.as_bytes(),
        capability.result_compiled_artifact_sha256.as_bytes(),
        capability.result_native_binary_sha256.as_bytes(),
        capability.stdout_sha256.as_bytes(),
        capability.stderr_sha256.as_bytes(),
        capability.verdict.as_bytes(),
    ] {
        hash_field(&mut hash, value);
    }
    for value in [
        capability.provider_image_bytes,
        capability.adapter_bytes,
        capability.request_compiled_artifact_bytes,
        capability.result_compiled_artifact_bytes,
        capability.result_native_binary_bytes,
        capability.exit_code,
        capability.stdout_bytes,
        capability.stderr_bytes,
    ] {
        hash_field(&mut hash, &(value as u64).to_le_bytes());
    }
    for value in [
        capability.replacement_authorized,
        capability.selection_authorized,
    ] {
        hash_field(&mut hash, &[u8::from(value)]);
    }
    encode_hex(&hash.finalize())
}

fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

fn hash_field(hash: &mut Sha256, value: &[u8]) {
    hash.update((value.len() as u64).to_le_bytes());
    hash.update(value);
}

fn validate_sha256(value: &str) -> Result<(), ArtifactError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(ArtifactError::new(
            "compiler candidate compile capability requires lowercase SHA-256 identities",
        ))
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    encode_hex(&Sha256::digest(bytes))
}

#[cfg(test)]
#[path = "compiler_candidate_compile_capability_tests.rs"]
mod tests;
